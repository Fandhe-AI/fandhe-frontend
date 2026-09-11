//! `fandhe_frontend_wasm_full::data_table`（イシュー #2126、親 #2124）の
//! headless-ui 出力とのドリフト検知テスト。
//!
//! `crates/headless-ui/src/data_table.rs`/`menu.rs`/`pagination.rs` の
//! anatomy 出力（`data-scope`/`data-part`/`data-value`/`data-index`/
//! `disabled`/`aria-disabled`/`data-disabled` 3 点セット）と本クレートの
//! [`SCOPE`]/`PART_*`/`MENU_*`/`PAGINATION_*` 定数・
//! [`sort_direction_from_attr`]・[`page_transition`] が前提とする語彙が
//! 一致していることを固定する。
//!
//! 実 DOM 経由の検証（click/MutationObserver 配線）は
//! `wasm-full/tests/data_table_browser.rs` が担当する。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::checkbox::CheckedState;
use fandhe_frontend_headless_ui::data_table::{
    column_attrs, column_toggle_item, ColumnProps, DataTable, DataTableProps, SortDirection,
    COLUMN_TOGGLE_ITEM_MARKER,
};
use fandhe_frontend_headless_ui::menu;
use fandhe_frontend_headless_ui::pagination::{item, prev_trigger, ItemMode, Pagination};
use fandhe_frontend_wasm_full::data_table::{
    page_transition, sort_direction_from_attr, ACTION_PAGE, ACTION_SORT, ACTION_TOGGLE_COLUMN,
    COLUMN_TOGGLE_MARKER, MENU_CHECKBOX_ITEM_PART, MENU_SCOPE, PAGINATION_ITEM_PART,
    PAGINATION_PREV_TRIGGER_PART, PAGINATION_SCOPE, PART_COLUMN_HEADER, PART_ROOT,
    PART_SORT_TRIGGER, SCOPE,
};

#[test]
fn scope_and_part_constants_match_headless_ui_output() {
    let table = DataTable::default();
    let html = render(&DataTable::root(DataTableProps::default(), vec![], vec![]));
    assert!(html.contains(&format!(r#"data-scope="{SCOPE}""#)));
    assert!(html.contains(&format!(r#"data-part="{PART_ROOT}""#)));

    let html = render(&table.column_header("name", true, vec![], vec![]));
    assert!(html.contains(&format!(r#"data-part="{PART_COLUMN_HEADER}""#)));
    assert!(html.contains(r#"data-column="name""#));
    assert!(html.contains(r#"aria-sort="none""#));
    assert!(html.contains(r#"data-sort="none""#));

    let html = render(&table.sort_trigger("name", vec![], vec![]));
    assert!(html.contains(&format!(r#"data-part="{PART_SORT_TRIGGER}""#)));
    assert!(html.contains(r#"data-value="name""#));
}

#[test]
fn column_toggle_item_uses_menu_scope_and_data_value() {
    let column = ColumnProps {
        id: "email",
        hidden: false,
    };
    let html = render(&column_toggle_item(&column, false, false, vec![], vec![]));
    assert!(html.contains(&format!(r#"data-scope="{MENU_SCOPE}""#)));
    assert!(html.contains(&format!(r#"data-part="{MENU_CHECKBOX_ITEM_PART}""#)));
    assert!(html.contains(r#"data-value="email""#));
    // 表示中の列は checked（`checked = !column.hidden`）。
    assert!(html.contains(r#"data-state="checked""#));
    // 列表示切替専用マーカー（codex-review P1 指摘: 列 id が偶然一致する
    // だけの無関係な menu checkbox-item と区別するために wasm-full が
    // 見る属性）が headless-ui 側の出力に実在することを固定する。
    assert!(html.contains(&format!(r#"{COLUMN_TOGGLE_ITEM_MARKER}="""#)));
    assert!(html.contains(r#"aria-checked="true""#));
}

#[test]
fn column_attrs_hidden_state_matches_wiring_expectations() {
    let hidden = column_attrs(&ColumnProps {
        id: "email",
        hidden: true,
    });
    assert!(hidden.contains(&("data-column", "email")));
    assert!(hidden.contains(&("hidden", "")));
    assert!(hidden.contains(&("data-hidden", "")));
}

#[test]
fn pagination_scope_and_part_constants_match_headless_ui_output() {
    let html = render(&item(ItemMode::Button, 2, true, false, vec![], vec![]));
    assert!(html.contains(&format!(r#"data-scope="{PAGINATION_SCOPE}""#)));
    assert!(html.contains(&format!(r#"data-part="{PAGINATION_ITEM_PART}""#)));
    assert!(html.contains(r#"data-index="2""#));
    assert!(html.contains("data-selected"));

    let html = render(&prev_trigger(ItemMode::Button, true, vec![], vec![]));
    assert!(html.contains(&format!(r#"data-part="{PAGINATION_PREV_TRIGGER_PART}""#)));
    assert!(html.contains("disabled"));
    assert!(html.contains(r#"aria-disabled="true""#));
    assert!(html.contains("data-disabled"));
}

#[test]
fn pagination_link_mode_item_has_no_native_disabled_attribute() {
    let html = render(&prev_trigger(
        ItemMode::Link { href: "?page=1" },
        true,
        vec![],
        vec![],
    ));
    // link モードは `disabled` 属性を持たない（無効な HTML のため headless-ui
    // 側が付与しない契約）。本クレートの配線がこれを見て `<a>` を除外する
    // 判定（`tag_name() == "BUTTON"`）の前提となる。
    assert!(!html.contains(r#" disabled"#));
    assert!(html.contains(r#"aria-disabled="true""#));
}

#[test]
fn sort_direction_from_attr_matches_headless_ui_vocabulary() {
    for dir in [
        SortDirection::None,
        SortDirection::Ascending,
        SortDirection::Descending,
        SortDirection::Other,
    ] {
        assert_eq!(
            sort_direction_from_attr(Some(dir.as_aria_sort())),
            Some(dir)
        );
        assert_eq!(
            sort_direction_from_attr(Some(dir.as_data_sort())),
            Some(dir)
        );
    }
}

#[test]
fn page_transition_matches_pagination_decode_action_vocabulary() {
    // headless-ui `Pagination` の `decode_action` 語彙
    // （"goto"/"next"/"prev"/"first"/"last"）と本クレートの
    // `page_transition` が同一の遷移結果を返すことを固定する。
    let pagination = Pagination::new(5, 1, 1, 1, 3);
    assert_eq!(pagination.page(), 3);
    assert_eq!(page_transition(3, 5, "next", ""), Some(4));
    assert_eq!(page_transition(3, 5, "prev", ""), Some(2));
    assert_eq!(page_transition(3, 5, "first", ""), Some(1));
    assert_eq!(page_transition(3, 5, "last", ""), Some(5));
    assert_eq!(page_transition(3, 5, "goto", "1"), Some(1));
}

#[test]
fn action_names_are_namespaced_like_other_wasm_full_notifications() {
    // `questionnaire`/`message_scroller` と同じ `"<module>:<event>"` 命名。
    assert_eq!(ACTION_SORT, "data-table:sort");
    assert_eq!(ACTION_TOGGLE_COLUMN, "data-table:toggle-column");
    assert_eq!(ACTION_PAGE, "data-table:page");
}

#[test]
fn column_toggle_marker_literal_matches_headless_ui_constant() {
    // wasm-full の `COLUMN_TOGGLE_MARKER`（属性名リテラル）は headless-ui
    // 側の `COLUMN_TOGGLE_ITEM_MARKER` と同一でなければならない（往復
    // ドリフト検知。codex-review P1 指摘の主防御であるため、他の
    // `PART_*`/`MENU_*` 定数と同じ厳密さで固定する）。
    assert_eq!(COLUMN_TOGGLE_MARKER, COLUMN_TOGGLE_ITEM_MARKER);
}

#[test]
fn unrelated_checkbox_item_does_not_carry_column_toggle_marker() {
    // `column_toggle_item` を経由しない、同じ `menu`/`checkbox-item`
    // scope/part を持つだけの無関係な checkbox-item（例: 通知方法選択
    // メニュー）は `COLUMN_TOGGLE_MARKER` を持たない。wasm-full の
    // `handle_toggle_column` がこの差でのみ対象を識別できることを固定
    // する（codex-review P1 指摘の再発防止）。
    let html = render(&menu::checkbox_item(
        false,
        "email",
        false,
        false,
        vec![],
        vec![],
    ));
    assert!(html.contains(&format!(r#"data-scope="{MENU_SCOPE}""#)));
    assert!(html.contains(&format!(r#"data-part="{MENU_CHECKBOX_ITEM_PART}""#)));
    assert!(html.contains(r#"data-value="email""#));
    assert!(!html.contains(COLUMN_TOGGLE_MARKER));
}

#[test]
fn column_toggle_item_drops_caller_supplied_marker_spoofing_attempt() {
    // 呼び出し側が `attrs` で `COLUMN_TOGGLE_ITEM_MARKER` を偽装しても
    // （通常フローでは起き得ないが、この属性がマーカーとして機能する
    // ためには単一の付与元しか持ち得ないことを保証する）、二重出力に
    // ならず高々 1 回しか現れない。
    let column = ColumnProps {
        id: "email",
        hidden: false,
    };
    let html = render(&column_toggle_item(
        &column,
        false,
        false,
        vec![(COLUMN_TOGGLE_ITEM_MARKER, "spoofed")],
        vec![],
    ));
    assert_eq!(html.matches(COLUMN_TOGGLE_ITEM_MARKER).count(), 1);
    assert!(html.contains(&format!(r#"{COLUMN_TOGGLE_ITEM_MARKER}="""#)));
}

#[test]
fn checkbox_indeterminate_data_state_matches_hidden_input_output() {
    use fandhe_frontend_headless_ui::checkbox::{hidden_input, CheckboxProps};
    let props = CheckboxProps {
        checked: CheckedState::Indeterminate,
        ..CheckboxProps::default()
    };
    let html = render(&hidden_input(&props, "select-all", "on", vec![]));
    assert!(html.contains(&format!(
        r#"data-state="{}""#,
        CheckedState::Indeterminate.as_data_state()
    )));
}
