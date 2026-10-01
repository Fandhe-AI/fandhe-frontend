# cart-two-column-summary

左に商品行のリスト、右に注文サマリの 2 カラムで構成するカート画面のブロックです。
`heading` / `image` / `text` / `native-select` / `button` / `separator` /
`data-list` / `card` / `progress` / `link` / `tooltip` の 11 部品を合成します。
Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R1247（集約元 R0324/R0681/R0682）です。商品名・属性・
価格・在庫状態はすべて架空のデータであり、実在のブランド・商品・PII は
含みません。商品画像はビルド時生成の同梱プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。数量選択は
初期選択値のみを示す静的表示です。形 A（商品あり）・形 B（空カート）を
縦に並記します。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::native_select::{
    native_select, FieldIds, FieldProps, NativeSelectProps,
};
use fandhe_frontend_pre_styled_ui::progress::{self, Orientation, ProgressProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::tooltip::{self, OpenState};

/// リンク先固定 URL（`footer_newsletter.rs` 等と同じ判断で、実アプリでは
/// 呼び出し側が実 URL に差し替える前提のダミー値）。
const REPO_URL: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 数量ヘルプ tooltip の `content` id（ページ内一意、1 行目のみ使用）。
const QTY_HELP_ID: &str = "blocks-cart-two-column-summary-qty-help";

/// 送料無料ライン（円）。[`shipping_progress`] の進捗バーの `max`。
const SHIPPING_FREE_THRESHOLD: f64 = 50_000.0;

/// [`SUMMARY_ROWS`] の小計と整合させた現在の小計（円、進捗バーの `value`）。
const SUBTOTAL: f64 = 46_600.0;

/// 架空の商品行データ（商品名, 属性表示, 在庫状態, 価格表示, 初期選択数量,
/// 入荷待ちで数量変更不可か）。3 行目のみ `true`（モジュール doc「入荷待ち
/// 行は数量変更不可」節参照）。実在のブランド・商品・PII は含まない。
const CART_ITEMS: &[(&str, &str, &str, &str, u8, bool)] = &[
    (
        "エルゴノミック メッシュチェア",
        "カラー: グレー / サイズ: M",
        "在庫あり",
        "¥24,800",
        1,
        false,
    ),
    (
        "ノイズキャンセリング ヘッドホン",
        "カラー: ブラック",
        "在庫あり",
        "¥18,200",
        1,
        false,
    ),
    (
        "ステンレス タンブラー 500ml",
        "カラー: シルバー",
        "入荷待ち（2〜3 週間）",
        "¥3,600",
        1,
        true,
    ),
];

/// 注文サマリの集計行（ラベル, 値, ヘルプリンクを添えるか）。最終行
/// （合計）だけ強調用の `data-*` を [`summary_totals`] 側で追加し、
/// ヘルプリンクは付けない。
const SUMMARY_ROWS: &[(&str, &str, bool)] = &[
    ("小計", "¥46,600", true),
    ("送料", "¥600", true),
    ("税", "¥4,660", true),
    ("合計", "¥51,860", false),
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

/// 数量ヘルプの tooltip（1 行目のみ。モジュール doc「数量選択のヘルプ
/// 吹き出し」節参照）。`OpenState::Open` 固定・`trigger` は `disabled: true`
/// で無 JS の静的 Demo に合わせる。
fn qty_help_tooltip() -> Node {
    tooltip::root(
        OpenState::Open,
        vec![],
        vec![
            tooltip::trigger(
                OpenState::Open,
                true,
                Some(QTY_HELP_ID),
                vec![("aria-label", "数量についてのヘルプ")],
                vec![text("?")],
            ),
            tooltip::positioner(
                OpenState::Open,
                vec![],
                vec![tooltip::content(
                    OpenState::Open,
                    Some(QTY_HELP_ID),
                    vec![],
                    vec![text("1 回のご注文で選べる数量は最大 5 個です。")],
                )],
            ),
        ],
    )
}

/// 数量 `select`（+ 1 行目のみ隣にヘルプ tooltip）をまとめたコントロール。
fn qty_control(index: usize, name: &str, qty: u8, disabled: bool) -> Node {
    let field_id = format!("blocks-cart-two-column-summary-qty-{}", index + 1);
    let qty_aria_label = format!("{name} の数量");
    let field = FieldProps {
        id: &field_id,
        ids: FieldIds::default(),
        disabled,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let show_help = index == 0;
    let mut extra_attrs = vec![("aria-label", qty_aria_label.as_str())];
    if show_help {
        extra_attrs.push(("aria-describedby", QTY_HELP_ID));
    }
    let mut children = vec![native_select(
        &NativeSelectProps::default(),
        &field,
        extra_attrs,
        qty_options(qty),
    )];
    if show_help {
        children.push(qty_help_tooltip());
    }
    div(
        vec![("class", "blocks-cart-two-column-summary-qty")],
        children,
    )
}

/// 商品行 1 件（サムネイル + 名称/属性/在庫 + 価格/数量選択/削除）。
/// `index` は 0 始まりで、数量 `select` の一意な `id` の派生に使う。
fn item_row(
    index: usize,
    name: &str,
    attrs: &str,
    stock: &str,
    price: &str,
    qty: u8,
    backorder: bool,
) -> Node {
    let mut row_attrs = vec![];
    if backorder {
        row_attrs.push((
            "data-blocks-cart-two-column-summary-item-state",
            "backorder",
        ));
    }
    div(
        {
            let mut a = vec![("class", "blocks-cart-two-column-summary-item")];
            a.extend(row_attrs);
            a
        },
        vec![
            image(
                &ImageProps {
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                },
                vec![("data-blocks-cart-two-column-summary-thumb", "")],
            ),
            div(
                vec![("class", "blocks-cart-two-column-summary-item-body")],
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
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![("data-blocks-cart-two-column-summary-stock", "")],
                        vec![text(stock)],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-cart-two-column-summary-item-controls")],
                vec![
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Medium,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(price)],
                    ),
                    qty_control(index, name, qty, backorder),
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            size: Size::Sm,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("削除")],
                    ),
                ],
            ),
        ],
    )
}

/// 商品行と行間の `separator` を束ねた左カラム（送料無料進捗 + 商品一覧）。
fn item_list() -> Node {
    let mut children = vec![shipping_progress()];
    for (index, (name, attrs, stock, price, qty, backorder)) in CART_ITEMS.iter().enumerate() {
        if index > 0 {
            children.push(separator(&SeparatorProps::default(), vec![]));
        }
        children.push(item_row(index, name, attrs, stock, price, *qty, *backorder));
    }
    div(
        vec![("class", "blocks-cart-two-column-summary-items")],
        children,
    )
}

/// 送料無料までの進捗バー（モジュール doc「送料無料までの進捗バー」節
/// 参照）。
fn shipping_progress() -> Node {
    let p = Progress::new(
        0.0,
        SHIPPING_FREE_THRESHOLD,
        Some(SUBTOTAL),
        Orientation::Horizontal,
    );
    div(
        vec![("class", "blocks-cart-two-column-summary-shipping")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    ..TextProps::default()
                },
                vec![],
                vec![text("送料無料まであと ¥3,400")],
            ),
            progress::root(
                &p,
                &ProgressProps {
                    size: Size::Sm,
                    ..ProgressProps::default()
                },
                None,
                vec![("aria-label", "送料無料までの進捗")],
                vec![p.track(vec![], vec![progress::range(&p, vec![])])],
            ),
        ],
    )
}

/// 集計行ラベルへ添えるヘルプリンク（`?`、対応表 ID R1247。合計行には
/// 付けない、モジュール doc「集計行ごとのヘルプリンク」節参照）。
fn summary_help_link(label: &str) -> Node {
    link::root(
        REPO_URL,
        &LinkProps {
            external: true,
            ..LinkProps::default()
        },
        vec![("aria-label", &format!("{label}についてのヘルプ"))],
        vec![text("?")],
    )
}

/// 集計行 1 件（`data_list::item` + `item-label` + `item-value`）。最終行
/// （合計）だけ強調用の `data-blocks-cart-two-column-summary-total` を
/// `item` へ付与する（[`LAYOUT_CSS`] が罫線・フォントウェイトで強調する
/// フック）。`with_help` の行はラベルの隣にヘルプリンクを添える。
fn summary_row(label: &str, value: &str, emphasize: bool, with_help: bool) -> Node {
    let attrs = if emphasize {
        vec![("data-blocks-cart-two-column-summary-total", "")]
    } else {
        vec![]
    };
    let mut label_children = vec![text(label)];
    if with_help {
        label_children.push(summary_help_link(label));
    }
    data_list::item(
        attrs,
        vec![
            data_list::item_label(vec![], label_children),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 注文サマリの集計リスト（横並び、最終行のみ強調）。
fn summary_totals() -> Node {
    let last = SUMMARY_ROWS.len() - 1;
    let rows = SUMMARY_ROWS
        .iter()
        .enumerate()
        .map(|(i, (label, value, with_help))| summary_row(label, value, i == last, *with_help))
        .collect();
    data_list::root(
        DataListProps {
            orientation: DataListOrientation::Horizontal,
            ..DataListProps::default()
        },
        vec![("data-blocks-cart-two-column-summary-totals", "")],
        rows,
    )
}

/// 右カラム（注文サマリカード）。`footer` に購入手続きボタンと買い物継続
/// リンクを縦に積む（モジュール doc「集計行ごとのヘルプリンク・買い物継続
/// リンク」節参照）。
fn summary_card() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-cart-two-column-summary-summary", "")],
        vec![
            card::header(vec![], vec![card::title(vec![], vec![text("注文サマリ")])]),
            card::body(vec![], vec![summary_totals()]),
            card::footer(
                vec![],
                vec![
                    button(
                        &ButtonProps::default(),
                        vec![("data-blocks-cart-two-column-summary-checkout", "")],
                        vec![text("購入手続きへ")],
                    ),
                    link::root(
                        REPO_URL,
                        &LinkProps {
                            external: true,
                            variant: LinkVariant::Underline,
                            ..LinkProps::default()
                        },
                        vec![("data-blocks-cart-two-column-summary-continue", "")],
                        vec![text("買い物を続ける")],
                    ),
                ],
            ),
        ],
    )
}

/// 通常のカート（形 A）。送料無料進捗 + 商品一覧 + 注文サマリの 2 カラム。
/// `demo` の `stack`（`display: flex; gap: ...`）へ直接の子として並べる
/// 契約のため `Vec<Node>` を返す（`contact_split_form_info::demo` と同型。
/// 各要素を無地の `div` へ包むと `stack` の `gap` が包み `div` 間にしか
/// 効かず、ラベル・見出し間の余白が消えるため包まない）。
fn cart_with_items() -> Vec<Node> {
    vec![
        variant_label("A: 商品あり"),
        heading(
            HeadingLevel::H3,
            &HeadingProps::default(),
            vec![],
            vec![text("ショッピングカート")],
        ),
        div(
            vec![("class", "blocks-cart-two-column-summary-columns")],
            vec![item_list(), summary_card()],
        ),
    ]
}

/// 空カート状態（形 B、モジュール doc「空カート状態の並記」節参照）。
/// 見出しは共有せず、`card` 1 枚に文言 + 買い物継続リンクのみを置く。
/// [`cart_with_items`] と同じ理由で `Vec<Node>` を返す。`card::root` は
/// `drop_class_attr`（モジュール doc「`class` と `data-*` の使い分け」節）
/// で呼び出し側 `class` を破棄するため、幅制限用の
/// `blocks-cart-two-column-summary-empty` クラスは `card::root` 自身では
/// なく `card` を包む `div` へ付ける。
fn empty_cart() -> Vec<Node> {
    vec![
        variant_label("B: カートが空の状態"),
        div(
            vec![("class", "blocks-cart-two-column-summary-empty")],
            vec![card::root(
                CardProps::default(),
                vec![],
                vec![card::body(
                    vec![],
                    vec![
                        styled_text::text(
                            &TextProps::default(),
                            vec![("data-blocks-cart-two-column-summary-variant", "empty")],
                            vec![text("カートに商品はありません。")],
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
                )],
            )],
        ),
    ]
}

/// `cart-two-column-summary` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。形 A（商品あり）・形 B（空カート）を縦に並記する
/// （モジュール doc「空カート状態の並記」節参照）。[`cart_with_items`]・
/// [`empty_cart`] の要素を `stack` の直接の子として平坦に並べる
/// （`contact_split_form_info::demo` と同型。包み `div` を挟むと `stack`
/// の `gap` がラベル・見出し/カード間に効かなくなるため）。
pub fn demo() -> Node {
    let mut children = cart_with_items();
    children.extend(empty_cart());
    div(
        vec![("class", "blocks-cart-two-column-summary-stack")],
        children,
    )
}
```

## 原案差分メモ

- 左カラムは送料無料進捗バー（`progress`、対応表 ID R0682）に続けて、商品行
  （サムネイル・商品名・属性・在庫状態・価格・数量選択・削除ボタン）を行間の
  `separator` で区切って並べます。送料無料ライン（¥50,000）と小計（¥46,600）
  から算出した「あと ¥3,400」を表示し、値は小計の集計行と整合させた固定値
  です（実アプリでは呼び出し側が動的に計算します）。
- 1 行目の数量選択の隣にのみ、`tooltip`（対応表 ID R0681）で選択可能数量の
  補足を添えます。3 行すべてに常時 open の吹き出しを出すと縦に重なって読め
  なくなるため、代表として 1 行目のみに絞っています。`OpenState::Open` 固定・
  `trigger` は `disabled: true` で無 JS の静的表示に合わせ、`select` 側には
  `aria-describedby` で tooltip の説明文を関連付けています。
- 3 行目（「ステンレス タンブラー」）は入荷待ちのため、数量 `select` を
  `disabled` にして状態違いを並記しています。在庫テキストには警告色
  （`--fandhe-color-warning-fg-subtle`）を当てています。
- 集計行（小計・送料・税）のラベル隣に `link`（`?`、対応表 ID R1247 の残り）
  でヘルプリンクを添え、合計行には付けていません。サマリカードの `footer`
  には購入手続きボタンの下へ「買い物を続ける」`link`（対応表 ID R0324、
  下線表示）を積んでいます。リンク先はすべて固定のリポジトリ URL です
  （`href="#"` は使いません）。
- 通常のカート（形 A）に続けて、空カート状態（形 B）を縦に並記しています。
  空カートでは見出しを共有せず、`card` 1 枚に「カートに商品はありません。」
  の文言と「買い物を続ける」`link` のみを置き、2 カラムのサマリは出しません。
- コンテナ幅（Demo 枠の幅、ビューポート幅ではありません）が 48rem 未満に
  なると `@container` によって 2 カラムが 1 カラムへ切り替わり、注文サマリ
  が商品一覧の下へ回ります。
- Demo を表示する `.blocks-demo` は横スクロールコンテナ（`overflow-x:
  auto`）のため、本 Demo 内では注文サマリカードに `position: sticky` を
  付けていません。実アプリへ組み込む際は呼び出し側で `sticky` を付与できます。

関連情報: [Heading](../themes/heading.md) / [Image](../themes/image.md) /
[Text](../themes/text.md) / [Native Select](../themes/native-select.md) /
[Button](../themes/button.md) / [Separator](../themes/separator.md) /
[Data List](../themes/data-list.md) / [Card](../themes/card.md) /
[Progress](../themes/progress.md) / [Link](../themes/link.md) /
[Tooltip](../themes/tooltip.md)
