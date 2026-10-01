# cart-mini-panel

店舗ヘッダーのカートボタン直下に開くミニカートパネルのブロックです。
`popover` / `button` / `image` / `text` / `number-input` / `separator` /
`link` / `heading` の 8 部品を合成します。Blocks は既存部品の合成例であり、
新しい UI 部品は追加しません。

主参照は対応表 ID R1252（店舗ヘッダ + カートのポップオーバー）です。集約元は
R0326（数量ステッパー + 合計込みボタン）・R0327（合計表示 + 2 ボタン）です。
店名・商品名・金額はすべて架空のデータであり、実在のブランド・商品・PII は
含みません。商品画像はビルド時生成の同梱プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。パネルは常に
開いた状態で固定し、「基本形」（固定数量表示）・「数量ステッパー付き」
（readonly の数量ステッパー）の 2 インスタンスを縦に並記します。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::number_input::{self, NumberInputFlags};
use fandhe_frontend_pre_styled_ui::popover::{self, OpenState};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 架空の商品行データ（商品名, 属性表示, 単価表示, 数量）。3 行とも金額は
/// 小計（[`SUBTOTAL_TEXT`]）と整合する固定値。実在のブランド・商品・PII は
/// 含まない。
const CART_ITEMS: &[(&str, &str, &str, u32)] = &[
    ("ミニマル デスクランプ", "カラー: ホワイト", "¥4,800", 1),
    (
        "折りたたみ ブックスタンド",
        "素材: ウォルナット",
        "¥3,200",
        2,
    ),
    ("セラミック マグカップ", "カラー: グレー", "¥1,600", 1),
];

/// 小計表示（[`CART_ITEMS`] の単価 × 数量の合計と整合する固定値）。
const SUBTOTAL_TEXT: &str = "¥12,800";

/// stepper インスタンスの購入ボタン文言。送料・税は購入手続きで計算する旨を
/// [`subtotal_row`] の補足行で示しているため、確定していない合計額はボタン
/// 文言に含めない（基本形と同一文言）。
const STEPPER_CHECKOUT_LABEL: &str = "購入手続きへ";

/// 「カートを見る」の遷移先（モジュール doc参照）。
fn cart_summary_href() -> String {
    crate::layout::asset_href(
        &crate::blocks::demo_base_path(),
        "/blocks/cart-two-column-summary/",
    )
}

/// インスタンスの見出し（「基本形」「数量ステッパー付き」）。
fn variant_heading(label: &'static str) -> Node {
    heading(
        HeadingLevel::H3,
        &HeadingProps::default(),
        vec![],
        vec![text(label)],
    )
}

/// 店舗ヘッダーバー（架空の店名 + カートトリガー）。`trigger` は
/// `controls` で `content_id` と関連付け、無 JS の静的 Demo のため常に
/// `OpenState::Open` を渡す（モジュール doc「2 インスタンスの並記」節）。
fn shop_header(shop_name: &str, content_id: &str) -> Node {
    div(
        vec![("class", "blocks-cart-mini-panel-shop-header")],
        vec![
            el("p", vec![], vec![text(shop_name)]),
            popover::trigger(
                OpenState::Open,
                false,
                Some(content_id),
                vec![("data-blocks-cart-mini-panel-trigger", "")],
                vec![text("カート（3）")],
            ),
        ],
    )
}

/// 数量表示（basic: 固定テキスト、stepper: readonly ステッパー）。
fn qty_display(prefix: &str, index: usize, qty: u32, interactive: bool) -> Node {
    if !interactive {
        return el(
            "p",
            vec![("class", "blocks-cart-mini-panel-qty")],
            vec![text(format!("数量 {qty}"))],
        );
    }
    let field_id = format!("{prefix}-qty-{}", index + 1);
    let qty_value = qty.to_string();
    let flags = NumberInputFlags {
        readonly: true,
        ..NumberInputFlags::default()
    };
    number_input::root(
        Size::Sm,
        false,
        false,
        true,
        vec![("class", "blocks-cart-mini-panel-qty")],
        vec![
            number_input::label(flags, Some(&field_id), vec![], vec![text("数量")]),
            number_input::control(
                flags,
                vec![],
                vec![
                    number_input::decrement_trigger(Some(&field_id), true, vec![], vec![text("-")]),
                    number_input::input(
                        "qty",
                        Some(&field_id),
                        Some(&qty_value),
                        "1",
                        "99",
                        flags,
                        vec![],
                    ),
                    number_input::increment_trigger(Some(&field_id), true, vec![], vec![text("+")]),
                ],
            ),
        ],
    )
}

/// 商品行 1 件（サムネイル + 名称/属性 + 数量 + 価格）。`prefix` は
/// stepper インスタンスの数量入力 `id` の一意化に使う。
fn item_row(
    prefix: &str,
    index: usize,
    name: &str,
    attrs: &str,
    price: &str,
    qty: u32,
    interactive: bool,
) -> Node {
    div(
        vec![("class", "blocks-cart-mini-panel-item")],
        vec![
            image(
                &ImageProps {
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                },
                vec![("data-blocks-cart-mini-panel-thumb", "")],
            ),
            div(
                vec![("class", "blocks-cart-mini-panel-item-body")],
                vec![
                    styled_text::text(&TextProps::default(), vec![], vec![text(name)]),
                    el(
                        "p",
                        vec![("class", "blocks-cart-mini-panel-item-attrs")],
                        vec![text(attrs)],
                    ),
                    qty_display(prefix, index, qty, interactive),
                ],
            ),
            el(
                "p",
                vec![("class", "blocks-cart-mini-panel-item-price")],
                vec![text(price)],
            ),
        ],
    )
}

/// 小計 + 補足行（モジュール doc「2 インスタンスの並記」節参照）。
fn subtotal_row() -> Node {
    div(
        vec![("class", "blocks-cart-mini-panel-subtotal")],
        vec![
            div(
                vec![("class", "blocks-cart-mini-panel-subtotal-line")],
                vec![
                    el("span", vec![], vec![text("小計")]),
                    styled_text::text(&TextProps::default(), vec![], vec![text(SUBTOTAL_TEXT)]),
                ],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("送料・税は購入手続きで計算します。")],
            ),
        ],
    )
}

/// 下端の操作 2 つ（購入手続き・カートを見る）。
fn panel_actions(checkout_label: &str) -> Node {
    div(
        vec![("class", "blocks-cart-mini-panel-actions")],
        vec![
            button(
                &ButtonProps::default(),
                vec![("data-blocks-cart-mini-panel-checkout", "")],
                vec![text(checkout_label)],
            ),
            link::root(
                &cart_summary_href(),
                &LinkProps {
                    variant: LinkVariant::Underline,
                    ..LinkProps::default()
                },
                vec![("data-blocks-cart-mini-panel-view-cart", "")],
                vec![text("カートを見る")],
            ),
        ],
    )
}

/// 1 インスタンス分のミニカートパネル（ヘッダー・見出し・パネル本体）を
/// 組み立てる。`prefix` は本インスタンス内の id を一意にする接頭辞。
fn panel_instance(
    prefix: &str,
    variant_title: &'static str,
    shop_name: &str,
    checkout_label: &str,
    interactive: bool,
) -> Vec<Node> {
    let content_id = format!("{prefix}-content");
    let title_id = format!("{prefix}-title");

    let mut rows = vec![];
    for (index, (name, attrs, price, qty)) in CART_ITEMS.iter().enumerate() {
        if index > 0 {
            rows.push(separator(&SeparatorProps::default(), vec![]));
        }
        rows.push(item_row(
            prefix,
            index,
            name,
            attrs,
            price,
            *qty,
            interactive,
        ));
    }
    rows.push(separator(&SeparatorProps::default(), vec![]));
    rows.push(subtotal_row());
    rows.push(panel_actions(checkout_label));

    vec![
        variant_heading(variant_title),
        div(
            vec![("class", "blocks-cart-mini-panel-instance")],
            vec![popover::root(
                OpenState::Open,
                vec![("data-blocks-cart-mini-panel-variant", prefix)],
                vec![
                    shop_header(shop_name, &content_id),
                    popover::positioner(
                        OpenState::Open,
                        vec![],
                        vec![popover::content(
                            OpenState::Open,
                            Some(&content_id),
                            Some(&title_id),
                            None,
                            vec![("data-blocks-cart-mini-panel-content", "")],
                            vec![
                                popover::title(Some(&title_id), vec![], vec![text("カート")]),
                                div(vec![("class", "blocks-cart-mini-panel-items")], rows),
                            ],
                        )],
                    ),
                ],
            )],
        ),
    ]
}

/// `cart-mini-panel` の Demo 本体。呼び出しごとに同一の `Node` を返す純
/// 関数。basic（固定数量表示）・stepper（readonly 数量ステッパー）の 2
/// インスタンスを縦に並記する（モジュール doc「2 インスタンスの並記」節
/// 参照）。
pub fn demo() -> Node {
    let mut children = panel_instance(
        "blocks-cart-mini-panel-basic",
        "基本形",
        dummy_assets::COMPANY_NAMES[0],
        "購入手続きへ",
        false,
    );
    children.extend(panel_instance(
        "blocks-cart-mini-panel-stepper",
        "数量ステッパー付き",
        dummy_assets::COMPANY_NAMES[1],
        STEPPER_CHECKOUT_LABEL,
        true,
    ));
    div(vec![("class", "blocks-cart-mini-panel-layout")], children)
}
```

## 原案差分メモ

- 店舗ヘッダーバー（架空の店名 + 「カート（3）」トリガー）の直下に、
  `popover` で小型パネルを開いた静的な状態を描いています。無 JS のため
  開閉操作はできず、常に `OpenState::Open` 固定です。パネル本体は
  `positioner` の既定の絶対配置ドロップダウンを `.blocks-demo` 枠内に収める
  ため、本 block スコープ限定で `position: static` へ中和し、トリガー直下の
  通常フローへ戻しています。
- 商品行は「基本形」で「数量 1」の固定テキスト表示、「数量ステッパー付き」
  （集約元 R0326）で `number-input` の readonly ステッパーに差し替えていま
  す。無 JS では増減操作が no-op になるため、ステッパーの増減トリガーは
  `disabled: true`、入力自体も `readonly: true` にしています。
- パネル下端に小計・補足（送料・税は購入手続きで計算する旨）と、2 つの操作
  （購入手続き・カートを見る）を置いています（集約元 R0327）。送料・税が
  未確定である旨と矛盾しないよう、「数量ステッパー付き」の購入ボタンにも
  確定額は含めず「基本形」と同一文言にしています。
- 「カートを見る」は `href="#"` ではなく、サイト内に実在する Cart 一覧
  block（`/blocks/cart-two-column-summary/`）へ `asset_href` で base_path を
  反映してリンクしています。2 インスタンスとも同じリンク先です。
- ボタンはすべて `button::button` の既定 `type="button"` のままで、
  `<form>` は出力しません。
- 実機ブラウザでの表示確認（枠内での欠けの有無・狭幅での操作列の折り返し・
  ライト/ダーク両テーマ）はサンドボックス制約により未実施です。

関連情報: [Popover](../themes/popover.md) / [Button](../themes/button.md) /
[Image](../themes/image.md) / [Text](../themes/text.md) /
[Number Input](../themes/number-input.md) /
[Separator](../themes/separator.md) / [Link](../themes/link.md) /
[Heading](../themes/heading.md)
