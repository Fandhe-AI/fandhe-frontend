# cart-two-column-summary

左に商品行のリスト、右に注文サマリの 2 カラムで構成するカート画面のブロックです。
`heading` / `image` / `text` / `native-select` / `button` / `separator` /
`data-list` / `card` の 8 部品を合成します。Blocks は既存部品の合成例であり、
新しい UI 部品は追加しません。

主参照は対応表 ID R1247（集約元 R0324/R0681/R0682）です。商品名・属性・
価格・在庫状態はすべて架空のデータであり、実在のブランド・商品・PII は
含みません。商品画像はビルド時生成の同梱プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。数量選択は
初期選択値のみを示す静的表示です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::native_select::{
    native_select, FieldIds, FieldProps, NativeSelectProps,
};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};

/// 架空の商品行データ（商品名, 属性表示, 在庫状態, 価格表示, 初期選択数量）。
/// 実在のブランド・商品・PII は含まない。
const CART_ITEMS: &[(&str, &str, &str, &str, u8)] = &[
    (
        "エルゴノミック メッシュチェア",
        "カラー: グレー / サイズ: M",
        "在庫あり",
        "¥24,800",
        1,
    ),
    (
        "ノイズキャンセリング ヘッドホン",
        "カラー: ブラック",
        "在庫あり",
        "¥18,200",
        1,
    ),
    (
        "ステンレス タンブラー 500ml",
        "カラー: シルバー",
        "入荷待ち（2〜3 週間）",
        "¥3,600",
        1,
    ),
];

/// 注文サマリの集計行（ラベル, 値）。最終行（合計）は
/// [`summary_totals`] 側で強調用の `data-*` を追加する。
const SUMMARY_ROWS: &[(&str, &str)] = &[
    ("小計", "¥46,600"),
    ("送料", "¥600"),
    ("税", "¥4,660"),
    ("合計", "¥51,860"),
];

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

/// 商品行 1 件（サムネイル + 名称/属性/在庫 + 価格/数量選択/削除）。
/// `index` は 0 始まりで、数量 `select` の一意な `id` の派生に使う。
fn item_row(index: usize, name: &str, attrs: &str, stock: &str, price: &str, qty: u8) -> Node {
    let field_id = format!("blocks-cart-two-column-summary-qty-{}", index + 1);
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
    div(
        vec![("class", "blocks-cart-two-column-summary-item")],
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
                        vec![],
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
                    native_select(
                        &NativeSelectProps::default(),
                        &field,
                        vec![("aria-label", qty_aria_label.as_str())],
                        qty_options(qty),
                    ),
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

/// 商品行と行間の `separator` を束ねた左カラム（商品一覧）。
fn item_list() -> Node {
    let mut children = Vec::new();
    for (index, (name, attrs, stock, price, qty)) in CART_ITEMS.iter().enumerate() {
        if index > 0 {
            children.push(separator(&SeparatorProps::default(), vec![]));
        }
        children.push(item_row(index, name, attrs, stock, price, *qty));
    }
    div(
        vec![("class", "blocks-cart-two-column-summary-items")],
        children,
    )
}

/// 集計行 1 件（`data_list::item` + `item-label` + `item-value`）。最終行
/// （合計）だけ強調用の `data-blocks-cart-two-column-summary-total` を
/// `item` へ付与する（[`LAYOUT_CSS`] が罫線・フォントウェイトで強調する
/// フック）。
fn summary_row(label: &str, value: &str, emphasize: bool) -> Node {
    let attrs = if emphasize {
        vec![("data-blocks-cart-two-column-summary-total", "")]
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

/// 注文サマリの集計リスト（横並び、最終行のみ強調）。
fn summary_totals() -> Node {
    let last = SUMMARY_ROWS.len() - 1;
    let rows = SUMMARY_ROWS
        .iter()
        .enumerate()
        .map(|(i, (label, value))| summary_row(label, value, i == last))
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

/// 右カラム（注文サマリカード）。
fn summary_card() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-cart-two-column-summary-summary", "")],
        vec![
            card::header(
                vec![],
                vec![card::title(
                    vec![],
                    vec![heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![text("注文サマリ")],
                    )],
                )],
            ),
            card::body(vec![], vec![summary_totals()]),
            card::footer(
                vec![],
                vec![button(
                    &ButtonProps::default(),
                    vec![("data-blocks-cart-two-column-summary-checkout", "")],
                    vec![text("購入手続きへ")],
                )],
            ),
        ],
    )
}

/// `cart-two-column-summary` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-cart-two-column-summary-stack")],
        vec![
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
        ],
    )
}
```

## 原案差分メモ

- 左カラムは商品行（サムネイル・商品名・属性・在庫状態・価格・数量選択・
  削除ボタン）を行間の `separator` で区切って並べ、右カラムは `card` で
  注文サマリ（小計・送料・税・合計・購入手続きボタン）をまとめています。
- コンテナ幅（Demo 枠の幅、ビューポート幅ではありません）が 48rem 未満に
  なると `@container` によって 2 カラムが 1 カラムへ切り替わり、注文サマリ
  が商品一覧の下へ回ります。
- Demo を表示する `.blocks-demo` は横スクロールコンテナ（`overflow-x:
  auto`）のため、本 Demo 内では注文サマリカードに `position: sticky` を
  付けていません。実アプリへ組み込む際は呼び出し側で `sticky` を付与できます。
- 送料無料進捗バー（`progress`、対応表 ID R0682）・数量選択のヘルプ吹き出し
  （`tooltip`、R0681）・買い物継続リンク（`link`、R0324）・集計行ごとの
  ヘルプリンク（R1247 の残り）・在庫切れ行や空カートといった状態違いの並記は、
  本 PR（骨格・主要領域）の後続 #3034 で追加予定です。

関連情報: [Heading](../themes/heading.md) / [Image](../themes/image.md) /
[Text](../themes/text.md) / [Native Select](../themes/native-select.md) /
[Button](../themes/button.md) / [Separator](../themes/separator.md) /
[Data List](../themes/data-list.md) / [Card](../themes/card.md)
