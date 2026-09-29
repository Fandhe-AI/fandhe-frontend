//! `table-sortable-bulk` block（イシュー #2945。親トラッキング #2892
//! 「Blocks アプリケーション A」配下、Application / Table カテゴリの
//! 最初の block）。列見出しに並び替え印を付け、先頭列に行選択
//! チェックボックスを置くデータテーブル。1 行以上選択された状態では
//! 見出し行の上に一括操作ツールバー（件数 + 操作ボタン）を重ねて表示する。
//! 対応表 ID R1335（主参照・代表構成）を軸に、R1331（ソート可能な列見出しの
//! みの構成）を「未選択」インスタンスへ集約する（`_/blocks-intake/` の
//! 対応ファイルは本イシュー着手時点で本 worktree に存在しないため、原稿・
//! 本コメントには対応表 ID のみを記す。`list_title_meta`/`list_people` と
//! 同じ扱い）。
//!
//! # 使用部品
//!
//! `data-table` / `table` / `checkbox` / `button` / `icon` の 5 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 「未選択」「選択中」を静的に併記する理由
//!
//! docs サイトは JS ハイドレーションを行わない（`crate::blocks` モジュール
//! doc 参照）。実運用では行チェックボックスの操作に応じて一括操作
//! ツールバーの表示/非表示が切り替わるが、本 Demo は無 JS の静的表示
//! しかできないため、`data-blocks-table-sortable-bulk-variant`（`"none"`/
//! `"selected"`）で 2 パネルを縦に並べて両状態を同時に示す
//! （`list_people`/`cta_signup_celebrate` 等と同型の判断）。
//!
//! # Themes 推奨の組み立て（`table::column_header` へ委譲する理由）
//!
//! `fandhe_frontend_pre_styled_ui::data_table` モジュール doc「Themes 推奨
//! の組み立て」節が定める契約どおり、表本体（`<table>`/`<thead>`/
//! `<tbody>`/`<tr>`）は `data_table` 側の `column_header`/`sort_trigger`
//! （node を生成するパーツ、Primitives 経路向け）を使わず、
//! `data_table::column_header_attrs`/`row_attrs`/`column_attrs`（node を
//! 作らない属性ヘルパ）を [`crate::table`] の `column_header`/`row`/`cell`
//! の `attrs` へ渡す形で合成する。`data_table::sort_trigger` はそのまま
//! 使う（`th`/`td` の中身であり `table` 側に対応するパーツがないため）。
//!
//! # 一括操作ツールバーを見出し行へ重ねる実装（`@container` 幅切替）
//!
//! `data_table::toolbar` を表本体のラッパー
//! `.blocks-table-sortable-bulk-table-wrap`（`position: relative;
//! container-type: inline-size;`）の子として配置し、
//! `position: absolute; top: 0; inset-inline-start/-end` で見出し行に
//! 重ねて表示する。狭幅（`@container ... (max-width: 40rem)`）では副次列
//! （役割・最終更新）を隠し、ツールバーは全幅帯（`inset-inline-start: 0`）
//! として残す。「未選択」パネルはツールバー自体を出力しない
//! （1 行も選択されていない状態を JS 無しで正しく表す）。
//!
//! # `menu`/ボタンを disabled にしない理由
//!
//! 一括操作ボタン（アーカイブ・削除）は `type="button"` で送信先を
//! 持たず、押しても何も起きない（`<form>` を出力しないため暗黙 submit も
//! 起きない）。`list_title_meta` と異なり、これらのボタンは「実運用で
//! 有効化される操作」を示す静的な実例であり、無効化して操作不能に見せる
//! 必要はないと判断した（並び替え可能な列見出しの `sort-trigger` も同様
//! に `disabled` を持たない。押下しても no-op であることは `<form>` 不在・
//! JS 非配線から自明であり、`list_title_meta` の「押しても何も起きない
//! 要素を操作可能に見せない」注意は行選択・一括削除のような破壊的操作を
//! 無効化して隠す趣旨ではなく menu/button の disabled 軸を持つ部品のみに
//! 適用した判断である）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。並び替え・行選択・一括操作の実処理（送信・永続化）は
//! アプリケーション責務であり本 block は静的表示のみを担う
//! （`docs/policy/intentional-non-adoption.md` §3.25）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps};
use fandhe_frontend_pre_styled_ui::data_table::{
    self, ColumnHeaderProps, ColumnProps, DataTable, DataTableProps, SortDirection,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 行選択チェックボックス（`crate::showcase::data_table_section` の
/// `row_select_checkbox` と同型）。`name` は panel・行ごとに一意にし
/// （選択中/未選択パネル双方を同じページへ静的併記するため）、
/// `demo_output_has_no_dangling_aria_references_or_duplicate_ids`
/// （`crates/docs-site/tests/blocks_contract.rs`）の id 重複検知に抵触
/// しないよう `id` 属性自体を持たない（`aria-label` のみで名前付け）。
fn row_select_checkbox(name: &str, checked: checkbox::CheckedState, label: &str) -> Node {
    let props = CheckboxProps {
        checked,
        ..CheckboxProps::default()
    };
    checkbox::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![],
        vec![
            checkbox::hidden_input(&props, name, "on", vec![("aria-label", label)]),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
        ],
    )
}

/// アーカイブボタンの自作アイコン（トレイ + 下矢印の抽象図形。参照元の
/// アイコンは持ち込まず単純な幾何図形とする、`stats_row`/
/// `error_page_popular_links` と同型の判断）。
fn archive_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![
            el(
                "path",
                vec![
                    ("d", "M3 5h18v4H3z"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linejoin", "round"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "M5 9v9a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V9"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linecap", "round"),
                    ("stroke-linejoin", "round"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "M10 13h4"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linecap", "round"),
                ],
                vec![],
            ),
        ],
    )
}

/// 削除ボタンの自作アイコン（ゴミ箱の抽象図形）。
fn delete_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![
            el(
                "path",
                vec![
                    ("d", "M4 7h16"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linecap", "round"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "M9 7V4h6v3"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linejoin", "round"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "M6 7l1 13h10l1-13"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linejoin", "round"),
                ],
                vec![],
            ),
        ],
    )
}

/// 行 1 件分のダミーデータ（架空・`dummy_assets` 由来）。
struct RowData {
    name: &'static str,
    status: &'static str,
    role: &'static str,
    updated: (&'static str, &'static str),
    assignee: &'static str,
}

/// 4 行分の共通ダミーデータ（「未選択」「選択中」両パネルで共有する）。
/// 名前列を `aria-sort="ascending"` 表示するため、行の並びも名前の
/// 昇順（Elena, Haruto, Kwame, Mei）にする（表示順とソート表示の矛盾を
/// 防ぐ。Codex レビュー指摘、イシュー #2945 PR #3393）。
fn rows_data() -> [RowData; 4] {
    [
        RowData {
            name: dummy_assets::PERSON_NAMES[1],
            status: "Pending",
            role: dummy_assets::JOB_TITLES[1],
            updated: ("2026-09-18", "Sep 18, 2026"),
            assignee: dummy_assets::PERSON_NAMES[5],
        },
        RowData {
            name: dummy_assets::PERSON_NAMES[0],
            status: "Active",
            role: dummy_assets::JOB_TITLES[0],
            updated: ("2026-09-20", "Sep 20, 2026"),
            assignee: dummy_assets::PERSON_NAMES[4],
        },
        RowData {
            name: dummy_assets::PERSON_NAMES[2],
            status: "Active",
            role: dummy_assets::JOB_TITLES[2],
            updated: ("2026-09-15", "Sep 15, 2026"),
            assignee: dummy_assets::PERSON_NAMES[6],
        },
        RowData {
            name: dummy_assets::PERSON_NAMES[3],
            status: "Archived",
            role: dummy_assets::JOB_TITLES[3],
            updated: ("2026-09-02", "Sep 2, 2026"),
            assignee: dummy_assets::PERSON_NAMES[7],
        },
    ]
}

/// 1 パネル分（「未選択」または「選択中」）を組み立てる。
///
/// `variant` は `data-blocks-table-sortable-bulk-variant` の値
/// （`"none"`/`"selected"`）、`selected_count` は選択中行数（0 なら
/// ツールバーを出力しない）。
fn panel(variant: &'static str, selected_count: usize) -> Node {
    let rows = rows_data();
    let table_state = DataTable::new(Some(("name".to_string(), SortDirection::Ascending)), vec![]);

    let name_column = ColumnProps {
        id: "name",
        hidden: false,
    };
    let status_column = ColumnProps {
        id: "status",
        hidden: false,
    };
    let role_column = ColumnProps {
        id: "role",
        hidden: false,
    };
    let updated_column = ColumnProps {
        id: "updated",
        hidden: false,
    };
    let assignee_column = ColumnProps {
        id: "assignee",
        hidden: false,
    };

    let secondary_attr = ("data-blocks-table-sortable-bulk-secondary", "");

    let header_row = table::row(
        vec![],
        vec![
            table::column_header(
                vec![("scope", "col")],
                vec![row_select_checkbox(
                    match variant {
                        "selected" => "select-selected-all",
                        _ => "select-none-all",
                    },
                    match selected_count {
                        0 => checkbox::CheckedState::Unchecked,
                        n if n >= rows.len() => checkbox::CheckedState::Checked,
                        _ => checkbox::CheckedState::Indeterminate,
                    },
                    "Select all rows",
                )],
            ),
            table::column_header(
                data_table::column_header_attrs(&ColumnHeaderProps {
                    column: name_column,
                    sort: Some(table_state.sort_direction_of("name").unwrap()),
                }),
                vec![data_table::sort_trigger(
                    &table_state,
                    "name",
                    vec![],
                    vec![text("名前")],
                )],
            ),
            table::column_header(
                data_table::column_header_attrs(&ColumnHeaderProps {
                    column: status_column,
                    sort: Some(
                        table_state
                            .sort_direction_of("status")
                            .unwrap_or(SortDirection::None),
                    ),
                }),
                vec![data_table::sort_trigger(
                    &table_state,
                    "status",
                    vec![],
                    vec![text("ステータス")],
                )],
            ),
            table::column_header(
                {
                    let mut attrs = data_table::column_header_attrs(&ColumnHeaderProps {
                        column: role_column,
                        sort: None,
                    });
                    attrs.push(secondary_attr);
                    attrs
                },
                vec![text("役割")],
            ),
            table::column_header(
                {
                    let mut attrs = data_table::column_header_attrs(&ColumnHeaderProps {
                        column: updated_column,
                        sort: None,
                    });
                    attrs.push(secondary_attr);
                    attrs
                },
                vec![text("最終更新")],
            ),
            table::column_header(
                data_table::column_header_attrs(&ColumnHeaderProps {
                    column: assignee_column,
                    sort: None,
                }),
                vec![text("担当")],
            ),
        ],
    );

    let mut body_rows = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        let is_selected = variant == "selected" && index < selected_count;
        let row_checkbox_name = format!("select-{variant}-row-{index}");
        body_rows.push(table::row(
            data_table::row_attrs(is_selected),
            vec![
                table::cell(
                    vec![],
                    vec![row_select_checkbox(
                        &row_checkbox_name,
                        if is_selected {
                            checkbox::CheckedState::Checked
                        } else {
                            checkbox::CheckedState::Unchecked
                        },
                        &format!("Select row: {}", row.name),
                    )],
                ),
                table::cell(data_table::column_attrs(&name_column), vec![text(row.name)]),
                table::cell(
                    data_table::column_attrs(&status_column),
                    vec![text(row.status)],
                ),
                table::cell(
                    {
                        let mut attrs = data_table::column_attrs(&role_column);
                        attrs.push(secondary_attr);
                        attrs
                    },
                    vec![text(row.role)],
                ),
                table::cell(
                    {
                        let mut attrs = data_table::column_attrs(&updated_column);
                        attrs.push(secondary_attr);
                        attrs
                    },
                    vec![el(
                        "time",
                        vec![("datetime", row.updated.0)],
                        vec![text(row.updated.1)],
                    )],
                ),
                table::cell(
                    data_table::column_attrs(&assignee_column),
                    vec![text(row.assignee)],
                ),
            ],
        ));
    }

    let mut wrap_children = Vec::new();
    if selected_count > 0 {
        wrap_children.push(data_table::toolbar(
            vec![("data-blocks-table-sortable-bulk-toolbar", "")],
            vec![
                data_table::selection_count(
                    vec![],
                    vec![text(format!("{selected_count} 件選択中"))],
                ),
                button::button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        size: Size::Sm,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![archive_icon(), text("アーカイブ")],
                ),
                button::button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        size: Size::Sm,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![delete_icon(), text("削除")],
                ),
            ],
        ));
    }
    wrap_children.push(table::root(
        TableProps {
            interactive: true,
            ..TableProps::default()
        },
        vec![],
        vec![
            table::header(vec![], vec![header_row]),
            table::body(vec![], body_rows),
        ],
    ));

    data_table::root(
        DataTableProps::default(),
        vec![("data-blocks-table-sortable-bulk-variant", variant)],
        vec![div(
            vec![("class", "blocks-table-sortable-bulk-table-wrap")],
            wrap_children,
        )],
    )
}

/// `table-sortable-bulk` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。「未選択」（R1331 相当・0 行選択）と「選択中」（R1335 相当・
/// 4 行中 3 行選択）の 2 パネルを縦に併記する。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-table-sortable-bulk-layout")],
        vec![
            div(
                vec![("class", "blocks-table-sortable-bulk-panel")],
                vec![panel("none", 0)],
            ),
            div(
                vec![("class", "blocks-table-sortable-bulk-panel")],
                vec![panel("selected", 3)],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/table-sortable-bulk/",
    title: "table-sortable-bulk",
    category: BlockCategory::Table,
    rust_source: "crates/docs-site/src/blocks/application/table/table_sortable_bulk.rs",
    demo_class: "blocks-table-sortable-bulk",
    parts: &[
        Part {
            label: "Data Table",
            path: "/themes/data-table/",
        },
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Checkbox",
            path: "/themes/checkbox/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `table_sortable_bulk` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。すべて `--fandhe-*` トークンを
/// 参照し、`[data-scope=` セレクタは書かない（`css_var_scope_prefix.rs` の
/// 対象外に保つ）。
const LAYOUT_CSS: &str = "\
.blocks-table-sortable-bulk-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-table-sortable-bulk-panel {\n  display: flex;\n  flex-direction: column;\n}\n\
.blocks-table-sortable-bulk-table-wrap {\n  position: relative;\n  container-type: inline-size;\n  container-name: blocks-table-sortable-bulk;\n}\n\
.blocks-table-sortable-bulk-table-wrap thead th {\n  height: 3rem;\n}\n\
[data-blocks-table-sortable-bulk-toolbar] {\n  position: absolute;\n  top: 0;\n  inset-inline-start: var(--fandhe-data-table-select-width, 2.5rem);\n  inset-inline-end: 0;\n  height: 3rem;\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  padding-inline: var(--fandhe-space-3);\n  background: var(--fandhe-color-bg);\n  z-index: 1;\n}\n\
@container blocks-table-sortable-bulk (max-width: 40rem) {\n  [data-blocks-table-sortable-bulk-secondary] {\n    display: none;\n  }\n\n  [data-blocks-table-sortable-bulk-toolbar] {\n    inset-inline-start: 0;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_uses_all_declared_parts_scopes() {
        let html = html();
        for scope in ["data-table", "table", "checkbox", "button", "icon"] {
            assert!(
                html.contains(&format!(r#"data-scope="{scope}""#)),
                "missing data-scope=\"{scope}\""
            );
        }
    }

    #[test]
    fn demo_renders_both_variants() {
        let html = html();
        assert!(html.contains(r#"data-blocks-table-sortable-bulk-variant="none""#));
        assert!(html.contains(r#"data-blocks-table-sortable-bulk-variant="selected""#));
    }

    #[test]
    fn demo_shows_sort_direction_marks() {
        let html = html();
        assert!(html.contains(r#"aria-sort="ascending""#));
        assert!(html.contains(r#"data-sort="none""#));
    }

    #[test]
    fn selected_panel_has_three_selected_rows_and_indeterminate_select_all() {
        let html = html();
        assert_eq!(html.matches("data-selected").count(), 3);
        assert!(html.contains(r#"data-state="indeterminate""#));
    }

    #[test]
    fn toolbar_hook_appears_once_for_selected_panel_only() {
        let html = html();
        assert_eq!(
            html.matches("data-blocks-table-sortable-bulk-toolbar")
                .count(),
            1
        );
    }

    #[test]
    fn buttons_are_type_button_and_no_form_or_data_uri() {
        let html = html();
        let button_count = html.matches("<button").count();
        let type_button_count = html.matches(r#"type="button""#).count();
        assert!(type_button_count >= button_count);
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn layout_css_has_no_style_breakout_and_declares_container_query() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(!LAYOUT_CSS.contains("</style"));
        assert!(LAYOUT_CSS.contains("@container blocks-table-sortable-bulk (max-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("position: absolute"));
    }
}
