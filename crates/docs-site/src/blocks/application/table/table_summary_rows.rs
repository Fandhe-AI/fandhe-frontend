//! `table-summary-rows` block（イシュー #2946。Application/Table カテゴリの
//! table-with-heading・table-responsive-stacked に続く 3 件目）。主参照は
//! 対応表 ID R1333 のみで、集約元との差分を並べる対象はない。
//!
//! # 使用部品
//!
//! `table` / `text` / `strong` の 3 部品を合成する（[`BLOCK`] の `parts` に
//! 一致させる契約）。新しい UI 部品は追加しない。
//!
//! # 集計行は `tfoot` + `colspan` の右寄せラベル
//!
//! 小計・消費税・合計の 3 行は [`table::footer`]（`<tfoot>`）へ置く。各行の
//! ラベルは [`table::row_header`]（`<th scope="row" colspan="2">`）で
//! 「品目」「数量」の 2 列分をまとめ、右寄せ（`data-align="end"`）にする。
//! 金額側は通常の [`table::cell`] を右寄せで置く（`table.rs` 既存語彙
//! `data-align` をそのまま使い、新規 CSS は追加しない）。
//!
//! 合計行のみ [`strong::strong`] でラベルと金額を包んで強調し、
//! `data-blocks-table-summary-rows-total` 属性で [`LAYOUT_CSS`] から
//! 上罫線・拡大フォントを当てる。
//!
//! `table.rs` の `data-align` state 規則は `cell`/`column-header` パーツ
//! のみを対象とし `row-header` を含まないため（`row-header` は既定で
//! `text-align: inherit`）、集計行ラベルの `data-align="end"` はそのままでは
//! 効かない。[`LAYOUT_CSS`] で `row-header` パーツに限定した
//! `text-align: end` を明示指定する（イシュー #2946 レビュー指摘）。
//!
//! # 狭幅で数量列を隠し、品目名の下へ補足を回す
//!
//! 数量列（`th`/`td`）は `data-blocks-table-summary-rows-qty` 属性で選択し、
//! `47.99rem` 以下でのみ非表示にする（`:nth-child` は `tfoot` の
//! `colspan="2"` セルと干渉しうるため専用属性を使う、下記 [`LAYOUT_CSS`]
//! 参照）。非表示にした数量は品目名の直下へ「数量 n」という
//! `text` 部品の補足行（`data-blocks-table-summary-rows-note`、既定
//! `display: none`）として残し、狭幅時のみ表示へ切り替える。情報の欠落を
//! 起こさない（列を隠すだけで DOM からは削除しない）。
//!
//! # 金額計算はデータから導出する
//!
//! 明細（品目名・数量・単価）は [`ITEMS`] の 1 か所にのみ持ち、行金額・
//! 小計・消費税（10%）・合計は [`demo`] 内で計算する。金額文字列を手書き
//! せず、[`yen`] helper で 3 桁区切りへ整形するだけに留める（計算結果と
//! 表示の食い違いを構造的に防ぐ）。
//!
//! # デモデータは架空
//!
//! 品目名・数量・単価はすべて独自に書いた架空の値であり、実企業名・実
//! クレデンシャル・PII を含まない。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示であり、リンク・ボタン・送信処理を一切持たない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
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
        vec![table::scroll_area(vec![], vec![root])],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/table-summary-rows/",
    title: "table-summary-rows",
    category: BlockCategory::Table,
    rust_source: "crates/docs-site/src/blocks/application/table/table_summary_rows.rs",
    demo_class: "blocks-table-summary-rows",
    parts: &[
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Strong",
            path: "/themes/strong/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `table_summary_rows` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
const LAYOUT_CSS: &str = "\
.blocks-table-summary-rows-layout {\n  max-width: 40rem;\n  width: 100%;\n}\n\
[data-scope=\"table\"][data-part=\"row-header\"][data-align=\"end\"] {\n  text-align: end;\n}\n\
[data-blocks-table-summary-rows-note] {\n  display: none;\n}\n\
[data-scope=\"table\"][data-part=\"row\"][data-blocks-table-summary-rows-total] > * {\n  border-top: 1px solid var(--fandhe-color-border);\n  font-size: var(--fandhe-font-font-size-lg);\n}\n\
@media (max-width: 47.99rem) {\n  [data-blocks-table-summary-rows-qty] {\n    display: none;\n  }\n  [data-blocks-table-summary-rows-note] {\n    display: block;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, yen, ITEMS, LAYOUT_CSS, TAX_RATE_PERCENT};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"table\"",
            "data-scope=\"text\"",
            "data-scope=\"strong\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<thead").count(), 1);
        assert_eq!(html.matches("<tbody").count(), 1);
        assert_eq!(html.matches("<tfoot").count(), 1);
        assert_eq!(html.matches("<tr").count(), 1 + ITEMS.len() + 3);
        assert_eq!(html.matches("scope=\"row\"").count(), ITEMS.len() + 3);
        assert_eq!(html.matches("colspan=\"2\"").count(), 3);
        assert!(html.contains("class=\"blocks-table-summary-rows-layout\""));
    }

    #[test]
    fn totals_are_consistent_with_items() {
        let html = demo_html();
        let subtotal: u32 = ITEMS.iter().map(|(_, qty, price)| qty * price).sum();
        let tax = subtotal * TAX_RATE_PERCENT / 100;
        let total = subtotal + tax;

        for amount in [subtotal, tax, total] {
            let expected = yen(amount);
            assert!(
                html.contains(&expected),
                "demo html should contain {expected}"
            );
        }
    }

    #[test]
    fn qty_cells_and_notes_carry_toggle_attributes() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-blocks-table-summary-rows-qty").count(),
            ITEMS.len() + 1
        );
        assert_eq!(
            html.matches("data-blocks-table-summary-rows-note").count(),
            ITEMS.len()
        );
    }

    #[test]
    fn no_form_or_unsafe_markup() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("href="));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn layout_css_is_safe() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains(".blocks-table-summary-rows-layout"));
        assert!(LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
    }

    #[test]
    fn layout_css_right_aligns_summary_row_headers() {
        // `table.rs` の `data-align` state 規則は row-header を対象外のため
        // （モジュール doc冒頭参照）、本 block 固有 CSS で明示指定している
        // ことを固定する。
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"table\"][data-part=\"row-header\"][data-align=\"end\"] {\n  text-align: end;\n}"
        ));
    }
}
