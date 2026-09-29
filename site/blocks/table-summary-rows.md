# table-summary-rows

`table` / `text` / `strong` の 3 部品を合成した、合計行付きの明細テーブル
です。品目・数量・金額を並べた明細行の下に、小計・消費税・合計の集計行を
持ちます。Blocks は既存部品の合成例であり、新規部品は追加しません。認証・
送信処理は持たない静的表示で、`<form>` は使いません。金額・品目名はすべて
架空のデータです。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::strong;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 明細データ（品目名・数量・単価）。行金額・小計・税・合計はすべて
/// ここから [`demo`] が計算する（金額の手書きを避けるための単一の正）。
const ITEMS: &[(&str, u32, u32)] = &[
    ("ノートスタンド", 2, 3_200),
    ("ワイヤレスキーボード", 1, 8_900),
    ("USB-C ハブ", 3, 2_400),
    ("デスクマット", 1, 4_500),
];

/// 消費税率（10%）。整数演算で丸め誤差を避けるため、税額は
/// `小計 * TAX_RATE_PERCENT / 100` で求める。
const TAX_RATE_PERCENT: u32 = 10;

/// 金額（円単位の整数）を 3 桁区切りの表示文字列へ整形する（内部専用の
/// 小さな helper。`format!` は数値のみを扱い HTML を組み立てないため
/// REQ-1 の既定エスケープ経路に影響しない）。
fn yen(amount: u32) -> String {
    let digits = amount.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, ch) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    let grouped: String = grouped.chars().rev().collect();
    format!("¥{grouped}")
}

/// `<col>` 1 本を組み立てる（`table.rs` に `colgroup`/`col` の anatomy が
/// ないため `fandhe_frontend_core::el` で直接組み立てる、モジュール doc
/// 「狭幅で数量列を隠し」節参照）。
fn col<'a>(attrs: Vec<(&'a str, &'a str)>) -> Node {
    el("col", attrs, vec![])
}

/// `<colgroup>`（品目・数量・金額の 3 列）を組み立てる。数量列の `<col>`
/// にのみ `data-blocks-table-summary-rows-qty-col` を付け、狭幅時に
/// [`LAYOUT_CSS`] がこの列を `visibility: collapse` で縮める。
fn column_group() -> Node {
    el(
        "colgroup",
        vec![],
        vec![
            col(vec![]),
            col(vec![("data-blocks-table-summary-rows-qty-col", "")]),
            col(vec![]),
        ],
    )
}

/// 明細 1 行（`tbody` の `tr`）を組み立てる。品目名セルには狭幅専用の
/// 「数量 n」補足（[`ITEMS`] の doc「狭幅で数量列を隠し」節参照）を持つ。
fn item_row(name: &str, qty: u32, unit_price: u32) -> Node {
    let amount = qty * unit_price;
    let note = styled_text::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-table-summary-rows-note", "")],
        vec![text(format!("数量 {qty}"))],
    );
    table::row(
        vec![],
        vec![
            table::row_header(vec![], vec![text(name), note]),
            table::cell(
                vec![
                    ("data-align", "end"),
                    ("data-blocks-table-summary-rows-qty", ""),
                ],
                vec![text(qty.to_string())],
            ),
            table::cell(vec![("data-align", "end")], vec![text(yen(amount))]),
        ],
    )
}

/// 集計行 1 行（`tfoot` の `tr`）を組み立てる。ラベルは品目・数量の 2 列分
/// を `colspan="2"` でまとめ右寄せにする。
fn summary_row(label: &str, amount_text: &str, attrs: Vec<(&str, &str)>) -> Node {
    table::row(
        attrs,
        vec![
            table::row_header(
                vec![("colspan", "2"), ("data-align", "end")],
                vec![text(label)],
            ),
            table::cell(vec![("data-align", "end")], vec![text(amount_text)]),
        ],
    )
}

/// `table-summary-rows` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。明細から小計・税・合計を計算し、[`summary_row`] へ渡す。
pub fn demo() -> Node {
    let subtotal: u32 = ITEMS.iter().map(|(_, qty, price)| qty * price).sum();
    let tax = subtotal * TAX_RATE_PERCENT / 100;
    let total = subtotal + tax;

    let header_row = table::row(
        vec![],
        vec![
            table::column_header(vec![], vec![text("品目")]),
            table::column_header(
                vec![
                    ("data-align", "end"),
                    ("data-blocks-table-summary-rows-qty", ""),
                ],
                vec![text("数量")],
            ),
            table::column_header(vec![("data-align", "end")], vec![text("金額")]),
        ],
    );

    let body_rows = ITEMS
        .iter()
        .map(|(name, qty, price)| item_row(name, *qty, *price))
        .collect();

    let total_row = table::row(
        vec![("data-blocks-table-summary-rows-total", "")],
        vec![
            table::row_header(
                vec![("colspan", "2"), ("data-align", "end")],
                vec![strong::strong(vec![], vec![text("合計")])],
            ),
            table::cell(
                vec![("data-align", "end")],
                vec![strong::strong(vec![], vec![text(yen(total))])],
            ),
        ],
    );

    let root = table::root(
        TableProps {
            size: Size::Md,
            ..TableProps::default()
        },
        vec![],
        vec![
            table::caption(vec![], vec![text("注文明細")]),
            column_group(),
            table::header(vec![], vec![header_row]),
            table::body(vec![], body_rows),
            table::footer(
                vec![],
                vec![
                    summary_row("小計", &yen(subtotal), vec![]),
                    summary_row("消費税（10%）", &yen(tax), vec![]),
                    total_row,
                ],
            ),
        ],
    );

    div(
        vec![("class", "blocks-table-summary-rows-layout")],
        vec![table::scroll_area(
            vec![
                ("role", "region"),
                ("aria-label", "注文明細テーブル"),
                ("tabindex", "0"),
            ],
            vec![root],
        )],
    )
}
```

## 原案差分メモ

- 集約元は主参照（対応表 ID R1333）の 1 件のみで、差分を並べる対象はない
- 集計行は `tfoot` へ置き、ラベルは `colspan="2"` の行見出し（`th
  scope="row"`）で右寄せする
- 狭幅（`47.99rem` 以下）では数量列を隠し、数量の補足を品目名の下へ回す
  （情報は DOM から削除せず非表示にするだけ）
- 合計行のみ `strong` で強調する

関連情報: [Table](../themes/table.md) / [Text](../themes/text.md) /
[Strong](../themes/strong.md)
