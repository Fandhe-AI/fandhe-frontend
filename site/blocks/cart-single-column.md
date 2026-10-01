# cart-single-column

1 カラム構成のカート画面のブロックです。
`heading` / `image` / `text` / `native-select` / `button` / `separator` /
`data-list` / `link` の 8 部品を合成します。
Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R1248（小計のみの集計 + 削除テキストボタン）、集約元は
R1249（淡い面で囲った 4 行の集計: 小計・送料・税・合計）です。商品名・
属性・価格はすべて架空のデータであり、実在のブランド・商品・PII は含み
ません。商品画像はビルド時生成の同梱プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。数量選択は
初期選択値のみを示す静的表示です。形 A（R1248）・形 B（R1249）を縦に
並記します。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::native_select::{
    native_select, FieldIds, FieldProps, NativeSelectProps,
};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};

/// リンク先固定 URL（`cart-two-column-summary` 等と同じ判断で、実アプリ
/// では呼び出し側が実 URL に差し替える前提のダミー値）。
const REPO_URL: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 架空の商品行データ（商品名, 属性表示, 価格表示, 初期選択数量）。実在の
/// ブランド・商品・PII は含まない。小計 ¥23,600 と整合する。
const CART_ITEMS: &[(&str, &str, &str, u8)] = &[
    ("リネン トートバッグ", "ナチュラル", "¥6,800", 1),
    ("セラミック マグカップ", "ホワイト", "¥2,400", 2),
    ("ウール ブランケット", "グレー", "¥12,000", 1),
];

/// 形 A（R1248、小計のみ）の集計行。
const SUMMARY_ROWS_A: &[(&str, &str, bool)] = &[("小計", "¥23,600", false)];

/// 形 B（R1249、淡色面 + 4 行）の集計行。最終行（合計）のみ強調する。
const SUMMARY_ROWS_B: &[(&str, &str, bool)] = &[
    ("小計", "¥23,600", false),
    ("送料", "¥800", false),
    ("税（10%）", "¥2,360", false),
    ("合計", "¥26,760", true),
];

/// 形ラベル（`contact_split_form_info.rs` の `variant_label` と同型）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// 指定した初期選択数量 `selected` の 1〜5 の `<option>` 列を組み立てる。
fn qty_options(selected: u8) -> Vec<Node> {
    (1..=5u8)
        .map(|n| {
            let mut attrs = vec![("value", n.to_string())];
            if n == selected {
                attrs.push(("selected", String::new()));
            }
            el(
                "option",
                attrs.iter().map(|(k, v)| (*k, v.as_str())).collect(),
                vec![text(n.to_string())],
            )
        })
        .collect()
}

/// 数量 `select`（形ごとに一意な id、モジュール doc「数量 `select` の id・
/// アクセシブルネーム」節参照）。
fn qty_control(variant_key: &str, index: usize, name: &str, qty: u8) -> Node {
    let field_id = format!("blocks-cart-single-column-{variant_key}-qty-{}", index + 1);
    let qty_aria_label = format!("{name} の数量");
    let field = FieldProps {
        id: &field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    native_select(
        &NativeSelectProps::default(),
        &field,
        vec![("aria-label", qty_aria_label.as_str())],
        qty_options(qty),
    )
}

/// 商品行 1 件（サムネイル + 名称/属性 + 価格/数量選択/削除）。`variant_key`
/// は数量 `select` の id を形ごとに区別するために使う。
fn item_row(
    variant_key: &str,
    index: usize,
    name: &str,
    attrs: &str,
    price: &str,
    qty: u8,
) -> Node {
    let remove_aria_label = format!("{name} を削除");
    div(
        vec![("class", "blocks-cart-single-column-item")],
        vec![
            image(
                &ImageProps {
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                },
                vec![("data-blocks-cart-single-column-thumb", "")],
            ),
            div(
                vec![("class", "blocks-cart-single-column-item-body")],
                vec![
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Medium,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(name)],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(attrs)],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-cart-single-column-item-controls")],
                vec![
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Medium,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(price)],
                    ),
                    qty_control(variant_key, index, name, qty),
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            size: Size::Sm,
                            ..ButtonProps::default()
                        },
                        vec![("aria-label", remove_aria_label.as_str())],
                        vec![text("削除")],
                    ),
                ],
            ),
        ],
    )
}

/// 商品行と行間の `separator` を束ねたリスト（両形で共通利用）。
fn item_list(variant_key: &str) -> Node {
    let mut children = vec![];
    for (index, (name, attrs, price, qty)) in CART_ITEMS.iter().enumerate() {
        if index > 0 {
            children.push(separator(&SeparatorProps::default(), vec![]));
        }
        children.push(item_row(variant_key, index, name, attrs, price, *qty));
    }
    div(vec![("class", "blocks-cart-single-column-items")], children)
}

/// 集計行 1 件（`data_list::item` + `item-label` + `item-value`）。
/// `emphasize` の行（合計）だけ強調用の
/// `data-blocks-cart-single-column-total` を付与する（[`LAYOUT_CSS`] が
/// 罫線・フォントウェイトで強調するフック）。
fn summary_row(label: &str, value: &str, emphasize: bool) -> Node {
    let attrs = if emphasize {
        vec![("data-blocks-cart-single-column-total", "")]
    } else {
        vec![]
    };
    data_list::item(
        attrs,
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 集計リスト（横並び）を組み立てる。
fn summary_list(rows: &[(&str, &str, bool)]) -> Node {
    let data_rows = rows
        .iter()
        .map(|(label, value, emphasize)| summary_row(label, value, *emphasize))
        .collect();
    data_list::root(
        DataListProps {
            orientation: DataListOrientation::Horizontal,
            ..DataListProps::default()
        },
        vec![],
        data_rows,
    )
}

/// 購入手続きボタンと「買い物を続ける」リンクの縦積み（両形で共通利用）。
fn checkout_actions() -> Node {
    div(
        vec![("class", "blocks-cart-single-column-actions")],
        vec![
            button(
                &ButtonProps::default(),
                vec![("data-blocks-cart-single-column-checkout", "")],
                vec![text("購入手続きへ")],
            ),
            link::root(
                REPO_URL,
                &LinkProps {
                    external: true,
                    variant: LinkVariant::Underline,
                    ..LinkProps::default()
                },
                vec![],
                vec![text("買い物を続ける")],
            ),
        ],
    )
}

/// 注記（送料・税の扱い）。形 A は小計のみの表示のため未確定である旨、
/// 形 B は送料・税を計上済みの確定合計を表示しているため、表示額が
/// 確定額であることを示す文言にする（形ごとに文言を分ける理由は P2
/// 指摘: 形 B の注記が計上済み金額と矛盾しないようにするため）。
fn shipping_note(text_content: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(text_content)],
    )
}

/// 形 A（R1248）。中央寄せの見出し → 商品行 → 小計のみの集計 → 注記 →
/// 購入手続き。[`cart_with_summary_b`] と同じ理由（`stack` の直接の子へ
/// 平坦に並べる）で `Vec<Node>` を返す。
fn cart_with_summary_a() -> Vec<Node> {
    vec![
        variant_label("A: 小計のみの集計（R1248）"),
        div(
            vec![("class", "blocks-cart-single-column-cart")],
            vec![
                div(
                    vec![("class", "blocks-cart-single-column-heading")],
                    vec![heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![text("ショッピングカート")],
                    )],
                ),
                item_list("a"),
                summary_list(SUMMARY_ROWS_A),
                shipping_note("送料と税は購入手続き時に計算されます。"),
                checkout_actions(),
            ],
        ),
    ]
}

/// 形 B（R1249）。見出し・商品行は形 A と共通。集計は淡色面のパネルで
/// 4 行を囲む。`demo` の `stack`（`display: flex; gap: ...`）へ直接の子
/// として並べる契約のため `Vec<Node>` を返す
/// （`contact_split_form_info::demo` と同型。無地の `div` で包むと `stack`
/// の `gap` が包み `div` 間にしか効かず要素間の余白が消えるため包まない）。
fn cart_with_summary_b() -> Vec<Node> {
    vec![
        variant_label("B: 淡色面に 4 行の集計（R1249）"),
        div(
            vec![("class", "blocks-cart-single-column-cart")],
            vec![
                div(
                    vec![("class", "blocks-cart-single-column-heading")],
                    vec![heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![text("ショッピングカート")],
                    )],
                ),
                item_list("b"),
                div(
                    vec![("class", "blocks-cart-single-column-summary-panel")],
                    vec![summary_list(SUMMARY_ROWS_B)],
                ),
                shipping_note("送料と税を含む確定金額です。"),
                checkout_actions(),
            ],
        ),
    ]
}

/// `cart-single-column` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。形 A・形 B を縦に並記する（モジュール doc「形 A（R1248）・形 B
/// （R1249）の並記」節参照）。
pub fn demo() -> Node {
    let mut children = cart_with_summary_a();
    children.extend(cart_with_summary_b());
    div(vec![("class", "blocks-cart-single-column-stack")], children)
}
```

## 原案差分メモ

- 主参照 R1248（小計のみの集計 + 削除テキストボタン）と集約元 R1249
  （淡い面で囲った 4 行の集計: 小計・送料・税・合計）を、同じ見出し・
  商品行の上に異なる集計表現として形 A・形 B の並記で読み取れるようにして
  います。
- 削除テキストボタンは R1248 由来ですが、形 B にも同じボタンを持たせて
  います（商品行自体は両形で共通部品として扱う判断）。
- 字下げ（集計パネルをサムネイル幅ぶん右へずらす表現）はコンテナ幅
  36rem 以上の広い幅でのみ適用します。狭い幅では字下げせず、どの幅でも
  1 カラム構成を保ちます。
- 購入手続きボタンの `href` は使わず `type="button"` のみとし、リンクは
  すべて固定のリポジトリ URL です（`href="#"` は使いません）。

関連情報: [Heading](../themes/heading.md) / [Image](../themes/image.md) /
[Text](../themes/text.md) / [Native Select](../themes/native-select.md) /
[Button](../themes/button.md) / [Separator](../themes/separator.md) /
[Data List](../themes/data-list.md) / [Link](../themes/link.md)
