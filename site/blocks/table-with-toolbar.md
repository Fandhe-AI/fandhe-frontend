# table-with-toolbar

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `input-group` /
`input` / `button` / `icon` / `table` / `badge` / `pagination` /
`scroll-area` の 10 部品を合成した、ツールバー付きテーブルの実例です。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0645、集約は R0715。出典の固有名・ファイル名は記載しません）。

見出し帯（表題「Invoices」・説明文・検索欄・絞り込み/新規作成ボタン）、
横スクロール対応のテーブル（請求書番号・顧客・状態・金額・期日の 5 列、
状態はバッジで色分け）、フッター（件数表示 + ページ送り）の 3 領域で構成
します。検索欄・ボタン群は狭い幅ではコンテナクエリ（`@container`）により
縦積みへ切り替わります。

本 Demo は版 A（代表構成）のみを実装したものです。期間選択ボタン版・
常時縦積み + エクスポート版・タブ型絞り込み版は後続のイシューで追加予定
です。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信を行いません。検索欄は空、ページ送りは 1 ページ目選択・前ページ無効の
固定表示です。ボタンは `type="button"` のまま送信先を持ちません。請求書
番号・顧客名・金額・期日はすべて独自に書いた架空のものであり、実企業名・
実クレデンシャル・PII を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::field::{FieldIds, FieldProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::pagination::{self, ItemMode};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::scroll_area;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 自作の幾何アイコン（線画）。`profile_detail_datalist::geo_icon` と同型
/// （モジュール doc「アイコンは自作の単純図形」節参照）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 検索アイコン（虫眼鏡）。
fn search_icon() -> Node {
    geo_icon("M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14z M16.5 16.5L21 21")
}

/// 絞り込みアイコン（漏斗）。
fn filter_icon() -> Node {
    geo_icon("M4 5h16 M7 12h10 M10 19h4")
}

/// 新規作成アイコン（プラス）。
fn plus_icon() -> Node {
    geo_icon("M12 5v14 M5 12h14")
}

/// 請求書 1 行分のダミーデータ。
struct InvoiceRow {
    /// 請求書番号（`row_header` に出す一意なキー）。
    number: &'static str,
    /// 顧客名（[`dummy_assets::COMPANY_NAMES`] を周回参照する）。
    customer_index: usize,
    /// 状態ラベル + バッジ配色。
    status: (&'static str, ColorPalette),
    /// 金額（整形処理を書かず固定文字列、`.claude/rules/coding-rust.md`
    /// §3.23 相当の判断: 数値・日時整形は UI コンポーネント層/block の
    /// 責務外）。
    amount: &'static str,
    /// 支払期日（固定文字列）。
    due: &'static str,
}

const ROWS: &[InvoiceRow] = &[
    InvoiceRow {
        number: "INV-2041",
        customer_index: 0,
        status: ("支払済み", ColorPalette::Success),
        amount: "¥128,000",
        due: "2026-08-15",
    },
    InvoiceRow {
        number: "INV-2042",
        customer_index: 1,
        status: ("未払い", ColorPalette::Warning),
        amount: "¥64,500",
        due: "2026-09-01",
    },
    InvoiceRow {
        number: "INV-2043",
        customer_index: 2,
        status: ("支払済み", ColorPalette::Success),
        amount: "¥212,300",
        due: "2026-09-10",
    },
    InvoiceRow {
        number: "INV-2044",
        customer_index: 3,
        status: ("期限超過", ColorPalette::Danger),
        amount: "¥38,900",
        due: "2026-08-28",
    },
    InvoiceRow {
        number: "INV-2045",
        customer_index: 4,
        status: ("支払済み", ColorPalette::Success),
        amount: "¥95,000",
        due: "2026-09-20",
    },
    InvoiceRow {
        number: "INV-2046",
        customer_index: 5,
        status: ("未払い", ColorPalette::Warning),
        amount: "¥171,250",
        due: "2026-09-25",
    },
];

/// 見出し帯左側（表題 + 説明）。
fn title_group() -> Node {
    div(
        vec![("data-blocks-table-with-toolbar-title", "")],
        vec![
            heading(
                HeadingLevel::H2,
                &HeadingProps {
                    size: HeadingSize::Xl,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("Invoices")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("直近の請求書をまとめて確認できます。")],
            ),
        ],
    )
}

/// 見出し帯右側（検索 + 絞り込み/新規作成ボタン）。
fn toolbar() -> Node {
    let field = FieldProps {
        id: "blocks-table-with-toolbar-search",
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    div(
        vec![("data-blocks-table-with-toolbar-toolbar", "")],
        vec![
            input_group::root(
                &group_props,
                vec![("data-blocks-table-with-toolbar-search", "")],
                vec![
                    input_group::addon(
                        InputGroupAlign::InlineStart,
                        &group_props,
                        vec![],
                        vec![search_icon()],
                    ),
                    input::input(
                        &InputProps::default(),
                        &field,
                        vec![
                            ("type", "search"),
                            ("placeholder", "請求書を検索"),
                            ("aria-label", "請求書を検索"),
                        ],
                    ),
                ],
            ),
            div(
                vec![("data-blocks-table-with-toolbar-actions", "")],
                vec![
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![filter_icon(), text("絞り込み")],
                    ),
                    button::button(
                        &ButtonProps::default(),
                        vec![],
                        vec![plus_icon(), text("新規作成")],
                    ),
                ],
            ),
        ],
    )
}

/// 見出し帯全体（表題群 + ツールバー）。
fn header() -> Node {
    div(
        vec![("data-blocks-table-with-toolbar-header", "")],
        vec![title_group(), toolbar()],
    )
}

/// 列見出し行（Amount は右寄せの目印として `data-align="end"` を持つ。
/// `table` 部品自体は列寄せの variant を持たないため block 固有の CSS
/// フックとして扱う）。
fn column_headers() -> Node {
    table::row(
        vec![],
        vec![
            table::column_header(vec![], vec![text("請求書番号")]),
            table::column_header(vec![], vec![text("顧客")]),
            table::column_header(vec![], vec![text("状態")]),
            table::column_header(vec![("data-align", "end")], vec![text("金額")]),
            table::column_header(vec![], vec![text("期日")]),
        ],
    )
}

/// 本文 1 行。
fn body_row(row: &InvoiceRow) -> Node {
    let customer =
        dummy_assets::COMPANY_NAMES[row.customer_index % dummy_assets::COMPANY_NAMES.len()];
    let (status_label, status_palette) = row.status;
    table::row(
        vec![],
        vec![
            table::row_header(vec![], vec![text(row.number)]),
            table::cell(vec![], vec![text(customer)]),
            table::cell(
                vec![],
                vec![badge::badge(
                    &BadgeProps {
                        variant: BadgeVariant::Subtle,
                        palette: status_palette,
                        ..BadgeProps::default()
                    },
                    vec![],
                    vec![text(status_label)],
                )],
            ),
            table::cell(vec![("data-align", "end")], vec![text(row.amount)]),
            table::cell(vec![], vec![text(row.due)]),
        ],
    )
}

/// テーブル本体（横スクロール対応の `scroll_area` 包み）。
fn table_section() -> Node {
    let table_node = table::root(
        TableProps {
            variant: TableVariant::Line,
            size: Size::Md,
            ..TableProps::default()
        },
        vec![("data-blocks-table-with-toolbar-table", "")],
        vec![
            table::header(vec![], vec![column_headers()]),
            table::body(vec![], ROWS.iter().map(body_row).collect()),
        ],
    );
    scroll_area::root(
        vec![("data-blocks-table-with-toolbar-scroll", "")],
        vec![scroll_area::viewport(
            vec![("role", "region"), ("aria-label", "請求書一覧")],
            vec![scroll_area::content(vec![], vec![table_node])],
        )],
    )
}

/// フッター（件数表示 + ページ送り。1 ページ目選択・前ページ無効で固定）。
fn footer() -> Node {
    div(
        vec![("data-blocks-table-with-toolbar-footer", "")],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("42 件中 1–6 件を表示")],
            ),
            pagination::root(
                Size::Sm,
                ColorPalette::Accent,
                "請求書ページ",
                vec![("data-blocks-table-with-toolbar-pagination", "")],
                vec![
                    pagination::prev_trigger(ItemMode::Button, true, vec![], vec![text("前へ")]),
                    pagination::item(ItemMode::Button, 1, true, false, vec![], vec![text("1")]),
                    pagination::item(ItemMode::Button, 2, false, false, vec![], vec![text("2")]),
                    pagination::item(ItemMode::Button, 3, false, false, vec![], vec![text("3")]),
                    pagination::ellipsis(vec![], vec![text("…")]),
                    pagination::item(ItemMode::Button, 7, false, false, vec![], vec![text("7")]),
                    pagination::next_trigger(ItemMode::Button, false, vec![], vec![text("次へ")]),
                ],
            ),
        ],
    )
}

/// `table-with-toolbar` の Demo 本体（版 A のみ。呼び出しごとに同一の
/// `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-table-with-toolbar-layout")],
        vec![header(), table_section(), footer()],
    )
}
```

## 原案差分メモ

- 版 A（代表構成）は主参照（対応表 ID R0645、集約 R0715）を軸に、見出し帯
  （表題・説明・検索・絞り込み/新規作成ボタン）・テーブル・フッター
  （件数表示・ページ送り）の基本形を表します。
- 検索欄・絞り込み/新規作成ボタンの行と見出し行は、Demo 枠の幅が
  `40rem` 未満のコンテナクエリで縦積みに切り替わります（`@media` の
  ビューポート幅ではなく block 自体の描画幅で判定します）。
- テーブルは `scroll_area` で包み、横にはみ出す幅（`min-width: 42rem`）を
  与えて横スクロールを確認できるようにしています。
- 期間選択ボタン版・常時縦積み + エクスポート版・タブ型絞り込み版の 3 つ
  の見せ方は後続のイシューで並記する予定です。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Input Group](../themes/input-group.md) / [Input](../themes/input.md) /
[Button](../themes/button.md) / [Icon](../themes/icon.md) /
[Table](../themes/table.md) / [Badge](../themes/badge.md) /
[Pagination](../themes/pagination.md) /
[Scroll Area](../themes/scroll-area.md)
