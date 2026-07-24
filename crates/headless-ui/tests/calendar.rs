//! `calendar` モジュールの統合テスト（イシュー #835）。
//!
//! 決定性・ARIA grid パターン・範囲クランプの単体テストは
//! `crates/headless-ui/src/calendar.rs` の `#[cfg(test)]` に集約済み
//! （`docs/api/component-api.md` の既存パターンに従う）。本ファイルは
//! それらでは検証できない横断的な観点（現在時刻 API 非使用の機械検査・
//! 公開 API 経由の統合利用）のみを担う。

use fandhe_frontend_headless_ui::calendar::{
    day_trigger, table_body, table_cell, table_row, Calendar, CalendarAction,
};
use fandhe_frontend_headless_ui::date::{PlainDate, Weekday};
use fandhe_frontend_headless_ui::fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::fandhe_frontend_interactive::{dispatch, Component};

fn today() -> PlainDate {
    PlainDate::new(2026, 7, 22).unwrap()
}

#[test]
fn public_api_month_grid_and_selection_integration() {
    let mut cal = Calendar::new(2026, 7, today(), None, None, None, Weekday::Monday).unwrap();
    assert!(dispatch(&mut cal, "select", "2026-07-22"));
    assert_eq!(cal.selected(), Some(today()));

    let html = render(&cal.table_body_from_grid(Vec::new()));
    assert!(html.contains(r#"data-scope="calendar""#));
    assert!(html.contains(r#"data-part="table-body""#));
    assert!(html.contains(r#"data-selected"#));
}

#[test]
fn public_api_parts_compose_into_a_grid_table() {
    let d = today();
    let cell = table_cell(
        true,
        vec![],
        vec![day_trigger(
            d,
            true,
            true,
            false,
            false,
            None,
            vec![],
            vec![],
        )],
    );
    let row = table_row(vec![], vec![cell]);
    let body = table_body(vec![], vec![row]);
    let html = render(&body);
    assert!(html.contains(r#"role="gridcell""#));
    assert!(html.contains(r#"aria-current="date""#));
}

#[test]
fn calendar_action_variants_round_trip_via_dispatch() {
    let mut cal = Calendar::new(2026, 7, today(), None, None, None, Weekday::Monday).unwrap();
    cal.update(CalendarAction::NextMonth);
    assert_eq!((cal.view_year(), cal.view_month()), (2026, 8));
}

// ---------------------------------------------------------------------
// 現在時刻 API 非使用の機械検査（`crates/headless-ui/tests/date.rs` と同型）
// ---------------------------------------------------------------------

#[test]
fn calendar_module_never_reads_the_current_time() {
    let source = include_str!("../src/calendar.rs");
    let forbidden_tokens = ["SystemTime", "std::time", "Instant", "js_sys", "now()"];
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        for token in forbidden_tokens {
            assert!(
                !line.contains(token),
                "calendar.rs の実コード行に現在時刻取得の疑いがあるトークン {token:?} が \
                 見つかった（「今日」は呼び出し側から明示的に受け取る決定的設計であり、\
                 現在時刻 API を呼んではならない）: {line:?}"
            );
        }
    }
}
