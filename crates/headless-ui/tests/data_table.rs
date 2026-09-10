//! `data_table::*`/`DataTable`（イシュー #2125）の公開 API 経由統合テスト。
//!
//! クレートルート・`data_table` モジュールからの re-export が実際に使える
//! ことを確認したうえで、8 パーツ全件の `data-part` 網羅・入れ子にした
//! [`checkbox`]/[`menu`]/[`pagination`]/[`field`] の scope 独立・
//! dispatch 巡回・hydration ラウンドトリップの統合挙動を固定する。
//! 値ごとの詳細な属性検証は `crates/headless-ui/src/data_table.rs` 側の
//! ユニットテストに置き、本ファイルは「公開 API 経由で壊れていないか」の
//! 統合確認に絞る（`tests/questionnaire.rs` と同じ方針）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_headless_ui::data_table::{
    column_attrs, column_header_attrs, column_toggle_item, footer, row_attrs, selection_count,
    toolbar, ColumnHeaderProps, ColumnProps, SortDirection,
};
use fandhe_frontend_headless_ui::field::{input, FieldIds, FieldProps};
use fandhe_frontend_headless_ui::menu;
use fandhe_frontend_headless_ui::pagination::{self, ItemMode, Pagination};
use fandhe_frontend_headless_ui::state::OpenState;
use fandhe_frontend_headless_ui::{DataTable, DataTableAction};
use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

fn base_field_props(id: &str) -> FieldProps<'_> {
    FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    }
}

/// 8 パーツ全件が `data-scope="data-table"` の下で正しい `data-part` を
/// 持ち、`toolbar`（`field::input` + `menu` の列表示切替）・`select-all`/
/// `select-row`（`checkbox`）・`footer`（`pagination`）へ入れ子にした
/// 各 scope が独立して共存することを固定する。
#[test]
fn data_table_public_api_covers_all_eight_parts_with_nested_scopes() {
    let t = DataTable::new(
        Some(("name".to_string(), SortDirection::Ascending)),
        vec!["email".to_string()],
    );

    let filter_field = base_field_props("filter");
    let email_column = ColumnProps {
        id: "email",
        hidden: true,
    };

    let toolbar_node = toolbar(
        vec![],
        vec![
            input(&filter_field, vec![]),
            menu::root(
                OpenState::Closed,
                vec![],
                vec![column_toggle_item(
                    &email_column,
                    false,
                    false,
                    vec![],
                    vec![text("Email")],
                )],
            ),
        ],
    );

    let column_header = t.column_header("name", true, vec![], vec![text("Name")]);
    let sort_trigger = t.sort_trigger("name", vec![], vec![text("Sort")]);

    let select_all = DataTable::select_all(
        CheckedState::Indeterminate,
        vec![],
        vec![checkbox::root(
            &CheckboxProps {
                checked: CheckedState::Indeterminate,
                ..Default::default()
            },
            vec![],
            vec![],
        )],
    );
    let select_row = DataTable::select_row(
        CheckedState::Checked,
        vec![],
        vec![checkbox::root(
            &CheckboxProps {
                checked: CheckedState::Checked,
                ..Default::default()
            },
            vec![],
            vec![],
        )],
    );

    let pager = Pagination::new(30, 10, 1, 1, 1);
    let footer_node = footer(
        vec![],
        vec![
            pager.root(
                "Table pagination",
                vec![],
                vec![pagination::item(
                    ItemMode::Button,
                    1,
                    true,
                    false,
                    vec![],
                    vec![],
                )],
            ),
            selection_count(vec![], vec![text("1 of 3 row(s) selected")]),
        ],
    );

    let node = fandhe_frontend_core::el(
        "div",
        vec![],
        vec![
            toolbar_node,
            column_header,
            sort_trigger,
            select_all,
            select_row,
            footer_node,
        ],
    );
    let html = render(&node);

    assert!(html.contains(r#"data-scope="data-table""#));
    for part in [
        "toolbar",
        "column-header",
        "sort-trigger",
        "select-all",
        "select-row",
        "footer",
        "selection-count",
    ] {
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "missing part {part} in {html}"
        );
    }

    // 入れ子 scope が独立して共存する。
    assert!(html.contains(r#"data-scope="field""#));
    assert!(html.contains(r#"data-scope="menu""#));
    assert!(html.contains(r#"data-scope="checkbox""#));
    assert!(html.contains(r#"data-scope="pagination""#));

    // ソート済み列（sortable=true）のみ aria-sort を持つ。
    assert!(html.contains(r#"aria-sort="ascending""#));
}

/// [`column_attrs`]/[`column_header_attrs`]/[`row_attrs`] が node を作らず
/// 呼び出し側の `td`/`th` へ渡せる属性ヘルパであることを固定する
/// （Themes 経路〔#2127〕が pre-styled `table::*` へパススルーする契約）。
#[test]
fn attribute_helpers_are_usable_without_producing_nodes() {
    let column = ColumnProps {
        id: "age",
        hidden: false,
    };
    let cell = fandhe_frontend_core::el("td", column_attrs(&column), vec![text("42")]);
    let html = render(&cell);
    assert!(html.contains(r#"data-column="age""#));

    let header_props = ColumnHeaderProps {
        column,
        sort: Some(SortDirection::Descending),
    };
    let th = fandhe_frontend_core::el("th", column_header_attrs(&header_props), vec![text("Age")]);
    let th_html = render(&th);
    assert!(th_html.contains(r#"aria-sort="descending""#));

    let tr = fandhe_frontend_core::el("tr", row_attrs(true), vec![]);
    assert!(render(&tr).contains("data-selected"));
}

/// dispatch 経由でソート・列表示切替が巡回し、hydration ラウンドトリップが
/// 成立することを固定する（`fandhe-frontend-wasm-full` 配線〔#2126〕の
/// 前提となる `Component`/`Hydrate` 統合契約）。
#[test]
fn dispatch_and_hydration_roundtrip_through_public_api() {
    let mut t = DataTable::default();
    assert!(dispatch(&mut t, "sort", "name"));
    assert!(dispatch(&mut t, "toggle-column", "email"));
    assert!(!dispatch(&mut t, "bogus", "x"));

    assert_eq!(t.sort(), Some(("name", SortDirection::Ascending)));
    assert!(t.is_hidden("email"));

    let node = render_for_hydration(&t);
    let html = render(&node);
    assert!(html.contains("data-hydrate-hidden-columns"));
    assert!(html.contains("data-hydrate-sort-column"));
    assert!(html.contains("data-hydrate-sort-direction"));

    let restored = DataTable::from_hydration_attrs(&t.hydration_attrs()).unwrap();
    assert_eq!(restored, t);

    let _ = DataTableAction::ClearSort;
}
