//! `table-with-toolbar` block 前半（イシュー #2949。親 #2948「Blocks に
//! Application / Table カテゴリ block `table-with-toolbar` を追加する」
//! の骨格 + 主要領域実装分）。見出し帯（表題・説明・検索・操作ボタン）と
//! テーブル（横スクロール対応）とフッター（件数表示・ページ送り）から
//! なる基本形を、既存部品（`heading` / `text` / `input_group` / `input` /
//! `button` / `icon` / `table` / `badge` / `pagination` / `scroll_area` の
//! 計 10 部品のみ）の合成で示す。対応表 ID は主参照 R0645（代表構成）・
//! 集約 R0715。`_/blocks-intake/` の対応ファイルは本イシュー着手時点で
//! 本 worktree に存在しないため、原稿・本コメントには対応表 ID のみを
//! 記す（`page_heading_meta.rs`〔イシュー #2933〕・
//! `table_with_heading.rs`〔イシュー #2947〕と同じ扱い）。
//!
//! # 後半 #2950 で追加予定の版・部品
//!
//! 本イシューは版 A（代表構成）のみを実装する。後半 #2950 で期間選択
//! ボタン版（R0716）・常時縦積み + エクスポート版（R0717）・タブ型絞り込み
//! 版（R0718）を並記し、D 版でのみ `tabs` 部品を追加する（`parts` 未使用
//! 宣言を避けるため、本 PR の `parts` には含めない）。
//!
//! # `scroll_area` を選ぶ理由（`table::scroll_area` ではなく独立部品）
//!
//! `table` 部品内蔵の [`fandhe_frontend_pre_styled_ui::table::scroll_area`]
//! ではなく [`fandhe_frontend_pre_styled_ui::scroll_area`] を使う。親仕様
//! （対応表 R0645/R0715）の使用部品一覧に「Scroll Area」が明記されており、
//! [`crate::blocks::Part::path`] で `/themes/scroll-area/` へ相互リンクする
//! ため（`list_sticky_groups.rs` と同じ判断）。
//!
//! # `@container` で狭幅レイアウトを切り替える
//!
//! ルート（`.blocks-table-with-toolbar-layout`）に `container-type:
//! inline-size` を与え、Demo 枠自体の幅で見出し帯・ツールバー・フッターの
//! 縦積み/横並びを切り替える（`@media` のビューポート幅ではなく block の
//! 実際の描画幅で判定するため。`docs/design/docs-site-blocks-section.md`
//! の既存 `@media (min-width: 40rem)` 系 block とは異なる判断だが、狭い
//! Demo 枠に埋め込まれる Blocks ページの実際のレイアウト崩れを防ぐには
//! コンテナクエリの方が実態に合う）。
//!
//! # `class` と `data-*` の使い分け
//!
//! `input_group::root`・`button::button`・`table::root`・
//! `pagination::root`・`scroll_area::root` は `drop_class_attr` を経由して
//! 呼び出し側の `class` を除去するため、これらへは
//! `data-blocks-table-with-toolbar-*` 属性でレイアウトフックする。素の
//! `div` へは `.blocks-table-with-toolbar-*` クラスを使う。
//!
//! # アイコンは自作の単純図形（線画）
//!
//! 検索・絞り込み・新規作成の各アイコンは [`profile_detail_datalist`]
//! （`crate::blocks::application::profile::profile_detail_datalist`）の
//! `geo_icon` と同型の自作単純図形（`fill="none"` + `stroke="currentColor"`）
//! で、いずれも装飾用途（`IconProps::label: None` の既定で `aria-hidden`）。
//! 参照元（対応表 R0645/R0715）のアイコンセットは持ち込まない。
//!
//! # 状態は静的固定（無 JS）
//!
//! 検索欄は空、1 ページ目を選択し前ページ送りボタンは無効の固定表示。
//! 開閉・遷移・データ取得は行わない（`crate::blocks` モジュール doc
//! 「`<form>` を使わない」節・「セキュリティ不変条件」節に従う）。
//! ボタンは `button::button` の既定 `type="button"` のまま送信先を持たず
//! （`docs/policy/intentional-non-adoption.md` §3.25：バリデーション・
//! 送信処理は UI コンポーネント層の責務外）、`pagination::item`/
//! `prev_trigger`/`next_trigger` は `ItemMode::Button` を使い `href` を
//! 持たせない（静的表示のため遷移先を持たせない）。架空の請求書番号・
//! 顧客名・金額・期日はすべてダミー（実企業名・PII・クレデンシャルを
//! 含まない。顧客名は [`crate::blocks::dummy_assets::COMPANY_NAMES`] を
//! 再利用する）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/table-with-toolbar/",
    title: "table-with-toolbar",
    category: BlockCategory::Table,
    rust_source: "crates/docs-site/src/blocks/application/table/table_with_toolbar.rs",
    demo_class: "blocks-table-with-toolbar",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Pagination",
            path: "/themes/pagination/",
        },
        Part {
            label: "Scroll Area",
            path: "/themes/scroll-area/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `table_with_toolbar` 固有のレイアウト規則。セレクタは
/// `.blocks-table-with-toolbar-*` と `[data-blocks-table-with-toolbar-*]`
/// のみを用いる。ルート class を `demo_class` と別名にする（既存 block と
/// 同じ Bugbot 教訓の回避）。狭幅判定は `@media` ではなく `@container`
/// （モジュール doc「`@container` で狭幅レイアウトを切り替える」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-table-with-toolbar-layout {\n  container-type: inline-size;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-table-with-toolbar-header] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  justify-content: space-between;\n}\n\
[data-blocks-table-with-toolbar-toolbar] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-table-with-toolbar-actions] {\n  display: flex;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-table-with-toolbar-scroll] {\n  max-width: 100%;\n}\n\
[data-blocks-table-with-toolbar-table] {\n  min-width: 42rem;\n}\n\
[data-blocks-table-with-toolbar-table] [data-align=\"end\"] {\n  text-align: end;\n}\n\
[data-blocks-table-with-toolbar-footer] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  align-items: flex-start;\n  justify-content: space-between;\n  border-top: 1px solid var(--fandhe-color-border);\n  padding-top: var(--fandhe-space-4);\n}\n\
@container (min-width: 40rem) {\n  [data-blocks-table-with-toolbar-header] {\n    flex-direction: row;\n    align-items: flex-end;\n  }\n  [data-blocks-table-with-toolbar-toolbar] {\n    flex-direction: row;\n    align-items: center;\n  }\n  [data-blocks-table-with-toolbar-footer] {\n    flex-direction: row;\n    align-items: center;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（heading/text/input-group/field(input)/button/icon/
    /// table/badge/pagination/scroll-area）の anatomy をすべて実際に
    /// 出力していること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"input-group\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
            "data-scope=\"table\"",
            "data-scope=\"badge\"",
            "data-scope=\"pagination\"",
            "data-scope=\"scroll-area\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// 主要領域の目印（見出し帯の検索・テーブル・ページ送り・スクロール
    /// 領域）が render 出力に現れる。
    #[test]
    fn demo_marks_primary_regions() {
        let html = render(&demo());
        for marker in [
            "data-blocks-table-with-toolbar-search",
            "data-blocks-table-with-toolbar-table",
            "data-blocks-table-with-toolbar-pagination",
            "data-blocks-table-with-toolbar-scroll",
        ] {
            assert!(html.contains(marker), "demo output should contain {marker}");
        }
    }

    /// 請求書行はちょうど 6 件（`table::row_header` の出力件数で数える）。
    #[test]
    fn demo_row_count() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-scope=\"table\" data-part=\"row-header\"")
                .count(),
            6
        );
    }

    /// ページ送りは 1 ページ目選択・前ページ無効の静的表示。
    #[test]
    fn pagination_is_static_first_page() {
        let html = render(&demo());
        assert!(html.contains(r#"aria-current="page""#));
        assert!(html.contains("disabled"));
    }

    /// `<form>`・`href="#"`・`data:` URI・`type="submit"` を出力しない
    /// （`crate::blocks` モジュール doc）。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in ["<form", "type=\"submit\"", "href=\"#\"", "src=\"data:"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が `demo_class`/主要 `data-*` セレクタと `@container`
    /// を含み、`<` を含まない（REQ-1: `</style>` によるスタイル脱出防止）。
    #[test]
    fn layout_css_declares_container_query_and_no_angle_bracket() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains(".blocks-table-with-toolbar-layout"));
        assert!(LAYOUT_CSS.contains("data-blocks-table-with-toolbar-scroll"));
        assert!(LAYOUT_CSS.contains("@container (min-width: 40rem)"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-table-with-toolbar-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-table-with-toolbar-layout");
    }

    /// `parts` 10 件が全て render 出力の `data-scope` に対応する
    /// （未使用宣言の検知）。
    #[test]
    fn all_declared_parts_are_used() {
        let html = render(&demo());
        let expected_scopes = [
            "heading",
            "text",
            "input-group",
            "field",
            "button",
            "icon",
            "table",
            "badge",
            "pagination",
            "scroll-area",
        ];
        assert_eq!(super::BLOCK.parts.len(), expected_scopes.len());
        for scope in expected_scopes {
            let needle = format!("data-scope=\"{scope}\"");
            assert!(html.contains(&needle), "missing {needle} in demo output");
        }
    }
}
