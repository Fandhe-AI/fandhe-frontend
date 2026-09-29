//! `table-responsive-stacked` block（イシュー #2943。Application / Table
//! カテゴリ、最初の block）。広幅では通常の多列テーブル、狭幅では副次列
//! （メール・役職・所属）を先頭列（氏名）のセル内へ縦積みの定義リストと
//! して折り畳んで表示する合成例。集約元は R1325（代表構成）のみ。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`profile_detail_datalist` と同じ扱い）。
//!
//! # 使用部品
//!
//! `table` / `data-list` / `heading` / `button` / `visually-hidden` の
//! 5 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。新しい UI 部品は追加しない。
//!
//! # 2 状態の並記（無 JS 制約下での折り畳みデモ）
//!
//! docs サイトは JS ハイドレーションを行わないため、ビューポート連動の
//! リサイズ実演はできない。代わりに同一構造の 2 インスタンス（状態 A:
//! 通常幅、状態 B: `max-inline-size: 24rem` で強制的に狭幅化）を並記し、
//! `@container`（コンテナクエリ）で判定することで折り畳みの違いを静的に
//! 見せる（`profile_detail_datalist`/`description_list_horizontal` と
//! 同型のパターン）。
//!
//! # 副次列の二重出力と a11y
//!
//! メール・役職・所属の値は「セル本体（広幅用）」と「氏名セル内の `dl`
//! （狭幅用）」の 2 か所に出力するが、[`LAYOUT_CSS`] の `@container` 条件で
//! 常にどちらか一方だけが `display: none` になる。`display: none` は
//! アクセシビリティツリーからも除外されるため、スクリーンリーダーに
//! よる二重読み上げは起きない（情報漏えいではなく読み上げ重複の回避策）。
//!
//! # 「操作」列見出しは視覚上空・SR 向けにのみ列名を供給
//!
//! 操作列の `column_header` は可視テキストを持たず、
//! [`fandhe_frontend_pre_styled_ui::visually_hidden::root`] でラップした
//! 列名のみを供給する（`visually-hidden` 部品の使用箇所）。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::table::root`]・
//! [`fandhe_frontend_pre_styled_ui::data_list::root`]・
//! [`fandhe_frontend_pre_styled_ui::button::button`] はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-table-responsive-stacked-*`）。レイアウト用ラッパー
//! （コンテナ・ツールバー）は素の `<div>` のため
//! `class="blocks-table-responsive-stacked-*"` を使う。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。ボタンは
//! `button::button` の既定 `type="button"` のまま用いる。
//!
//! # ダミー素材について
//!
//! 氏名・役職・社名は `crate::blocks::dummy_assets`（架空セット）を使う。
//! メールアドレスは `example.com` ドメインとし、実在の人物・企業・PII は
//! 含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// ラベル・値の 1 行（狭幅時に氏名セル内へ表示する定義リストの内訳）。
fn stacked_row(label: &'static str, value: &'static str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 1 行分のデータ（氏名 + 副次列 3 種）を組み立てる。
fn member_row(
    name: &'static str,
    email: &'static str,
    title: &'static str,
    company: &'static str,
) -> Node {
    table::row(
        vec![],
        vec![
            table::row_header(
                vec![],
                vec![
                    text(name),
                    data_list::root(
                        DataListProps {
                            orientation: DataListOrientation::Vertical,
                            size: Size::Sm,
                            ..DataListProps::default()
                        },
                        vec![("data-blocks-table-responsive-stacked-stacked", "")],
                        vec![
                            stacked_row("メール", email),
                            stacked_row("役職", title),
                            stacked_row("所属", company),
                        ],
                    ),
                ],
            ),
            table::cell(
                vec![("data-blocks-table-responsive-stacked-secondary", "")],
                vec![text(email)],
            ),
            table::cell(
                vec![("data-blocks-table-responsive-stacked-secondary", "")],
                vec![text(title)],
            ),
            table::cell(
                vec![("data-blocks-table-responsive-stacked-secondary", "")],
                vec![text(company)],
            ),
            table::cell(
                vec![],
                vec![button(
                    &ButtonProps {
                        variant: ButtonVariant::Ghost,
                        size: Size::Sm,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("編集")],
                )],
            ),
        ],
    )
}

/// 見出し + 追加ボタンのツールバー（基本形 `table-with-heading` と同じ
/// 位置＝テーブル上部に置く）。
fn toolbar() -> Node {
    div(
        vec![("class", "blocks-table-responsive-stacked-toolbar")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("メンバー")],
            ),
            button(
                &ButtonProps {
                    variant: ButtonVariant::Solid,
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("メンバーを追加")],
            ),
        ],
    )
}

/// テーブル本体（ツールバー + `table::root`）。状態 A/B で共有する。
fn table_section() -> Vec<Node> {
    vec![
        toolbar(),
        table::root(
            TableProps::default(),
            vec![("data-blocks-table-responsive-stacked-table", "")],
            vec![
                table::header(
                    vec![],
                    vec![table::row(
                        vec![],
                        vec![
                            table::column_header(vec![], vec![text("名前")]),
                            table::column_header(
                                vec![("data-blocks-table-responsive-stacked-secondary", "")],
                                vec![text("メール")],
                            ),
                            table::column_header(
                                vec![("data-blocks-table-responsive-stacked-secondary", "")],
                                vec![text("役職")],
                            ),
                            table::column_header(
                                vec![("data-blocks-table-responsive-stacked-secondary", "")],
                                vec![text("所属")],
                            ),
                            table::column_header(
                                vec![],
                                vec![visually_hidden::root(vec![], vec![text("操作")])],
                            ),
                        ],
                    )],
                ),
                table::body(
                    vec![],
                    vec![
                        member_row(
                            dummy_assets::PERSON_NAMES[0],
                            "haruto.fujimaki@example.com",
                            dummy_assets::JOB_TITLES[0],
                            dummy_assets::COMPANY_NAMES[0],
                        ),
                        member_row(
                            dummy_assets::PERSON_NAMES[1],
                            "elena.vasquez@example.com",
                            dummy_assets::JOB_TITLES[1],
                            dummy_assets::COMPANY_NAMES[1],
                        ),
                        member_row(
                            dummy_assets::PERSON_NAMES[2],
                            "kenji.oshiro@example.com",
                            dummy_assets::JOB_TITLES[2],
                            dummy_assets::COMPANY_NAMES[2],
                        ),
                        member_row(
                            dummy_assets::PERSON_NAMES[3],
                            "amara.okafor@example.com",
                            dummy_assets::JOB_TITLES[3],
                            dummy_assets::COMPANY_NAMES[3],
                        ),
                    ],
                ),
            ],
        ),
    ]
}

/// `table-responsive-stacked` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-table-responsive-stacked-stack")],
        vec![
            div(
                vec![("class", "blocks-table-responsive-stacked-frame")],
                table_section(),
            ),
            div(
                vec![(
                    "class",
                    "blocks-table-responsive-stacked-frame blocks-table-responsive-stacked-frame--narrow",
                )],
                table_section(),
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/table-responsive-stacked/",
    title: "table-responsive-stacked",
    category: BlockCategory::Table,
    rust_source: "crates/docs-site/src/blocks/application/table/table_responsive_stacked.rs",
    demo_class: "blocks-table-responsive-stacked",
    parts: &[
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Data List",
            path: "/themes/data-list/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `table_responsive_stacked` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
///
/// 閾値 `40rem` は 5 列テーブルが自然に収まる下限として選んだ
/// （`profile_detail_datalist` の `36rem` と同系統の値）。
const LAYOUT_CSS: &str = "\
.blocks-table-responsive-stacked-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-table-responsive-stacked-frame {\n  container-type: inline-size;\n  container-name: blocks-table-responsive-stacked;\n}\n\
.blocks-table-responsive-stacked-frame--narrow {\n  max-inline-size: 24rem;\n}\n\
.blocks-table-responsive-stacked-toolbar {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n  margin-block-end: var(--fandhe-space-4);\n}\n\
[data-blocks-table-responsive-stacked-stacked] {\n  display: none;\n}\n\
[data-scope=\"data-list\"][data-part=\"root\"][data-blocks-table-responsive-stacked-stacked] {\n  --fandhe-data-list-gap: var(--fandhe-space-1);\n  margin-block-start: var(--fandhe-space-1);\n}\n\
@container blocks-table-responsive-stacked (max-width: 40rem) {\n  \
[data-blocks-table-responsive-stacked-secondary] {\n    display: none;\n  }\n  \
[data-blocks-table-responsive-stacked-stacked] {\n    display: block;\n  }\n  \
.blocks-table-responsive-stacked-toolbar {\n    flex-wrap: wrap;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"table\"",
            "data-scope=\"data-list\"",
            "data-scope=\"heading\"",
            "data-scope=\"button\"",
            "data-scope=\"visually-hidden\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(html.matches("<table").count(), 2);
        assert_eq!(html.matches(r#"scope="row""#).count(), 8);
        assert_eq!(
            html.matches("data-blocks-table-responsive-stacked-secondary")
                .count(),
            2 * (3 + 3 * 4) // 2 states * (3 column headers + 3 secondary cells * 4 rows)
        );
        assert_eq!(
            html.matches("fd-data-list--orientation-vertical").count(),
            8 // 2 states * 4 rows
        );
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn layout_css_is_safe_and_hides_secondary_columns_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(
            LAYOUT_CSS.contains("@container blocks-table-responsive-stacked (max-width: 40rem)")
        );
        assert!(LAYOUT_CSS.contains("max-inline-size: 24rem;"));
    }

    #[test]
    fn operations_column_header_is_screen_reader_only() {
        let html = demo_html();
        assert!(html.contains("data-scope=\"visually-hidden\""));
        assert_eq!(html.matches("data-scope=\"visually-hidden\"").count(), 2);
    }
}
