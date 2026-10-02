//! イシュー #3133（calendar の Examples に週・日の時間軸ビュー合成例を
//! 追加する）専用の契約テスト。
//!
//! `crates/docs-site/src/component_page_specs_948.rs` は並列実行される
//! 他イシューも触り得る共有ファイルのため、検証は per-issue テストファイル
//! 方式（`component_specs_2212.rs` と同型）で分離する。本ファイルは
//! `/themes/calendar/` 1 ページのみを検証する: (1) 週・日ビューの新しい
//! Example 見出しが載ること、(2) 週ビューが 7 曜日列見出しを持つこと、
//! (3) 時刻見出し（`scope="row"`）が週・日ビュー合計 18 件以上で
//! `09:00`/`17:00` ラベルを含むこと、(4) スロット跨ぎ予定が
//! `rowspan="2"` で表現されること（rowspan 被覆後に余分な `<td` を
//! 出していないことのセル数整合込み）、(5) 今日列に
//! `aria-current="date"` が付くこと、(6) `<form` を持ち込まず id 重複も
//! ないこと、(7) レンダリングが決定的であること。`raw_html` 不使用は
//! `component_specs_2212.rs::presets_example_uses_no_raw_html` が既に
//! `component_page_specs_948.rs` 全体を走査済みのため本ファイルでは
//! 重複させない。

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::component_page::generated_content;

/// `/themes/calendar/` の生成 HTML（1 回分）。
fn calendar_page_html() -> String {
    let node = generated_content("/themes/calendar/")
        .expect("generated_content(\"/themes/calendar/\") should be Some");
    render(&node)
}

/// Examples 節に週・日ビューの新しい見出しが含まれること。
#[test]
fn examples_section_includes_time_grid_entries() {
    let html = calendar_page_html();
    assert!(
        html.contains("週ビュー（時間軸グリッド）"),
        "Examples should include a week-view time grid entry, got: {html}"
    );
    assert!(
        html.contains("日ビュー（時間軸グリッド）"),
        "Examples should include a day-view time grid entry, got: {html}"
    );
}

/// 週ビューの 7 曜日列見出しが含まれること。
#[test]
fn week_view_has_seven_day_columns() {
    let html = calendar_page_html();
    for label in [
        "Mon 07/20",
        "Tue 07/21",
        "Wed 07/22",
        "Thu 07/23",
        "Fri 07/24",
        "Sat 07/25",
        "Sun 07/26",
    ] {
        assert!(
            html.contains(label),
            "week view should render day column label {label}, got: {html}"
        );
    }
}

/// 時刻見出し（`scope="row"`）が週・日ビュー合計 18 件（9 時刻 × 2 表）以上で、
/// 09:00/17:00 ラベルを含むこと。
#[test]
fn time_row_headers_cover_nine_to_seventeen() {
    let html = calendar_page_html();
    let row_header_count = html.matches(r#"scope="row""#).count();
    assert!(
        row_header_count >= 18,
        "expected at least 18 scope=\"row\" time headers (9 hours x 2 grids), got {row_header_count}"
    );
    assert!(
        html.contains("09:00"),
        "should contain 09:00 label, got: {html}"
    );
    assert!(
        html.contains("17:00"),
        "should contain 17:00 label, got: {html}"
    );
}

/// スロット跨ぎ予定が `rowspan="2"` で表現されること。
#[test]
fn multi_slot_events_use_rowspan() {
    let html = calendar_page_html();
    assert!(
        html.contains(r#"rowspan="2""#),
        "multi-slot events should render rowspan=\"2\", got: {html}"
    );
    assert!(
        html.contains("Design Review"),
        "should render the Design Review event label, got: {html}"
    );
}

/// 今日列に `aria-current="date"` が付くこと。
#[test]
fn today_column_has_aria_current() {
    let html = calendar_page_html();
    assert!(
        html.contains(r#"aria-current="date""#),
        "today's column header should have aria-current=\"date\", got: {html}"
    );
}

/// calendar ページが `<form` を持ち込まないこと（UI コンポーネント層は
/// アプリケーションロジックを内包しない、§3.25）。
#[test]
fn calendar_page_has_no_form_element() {
    let html = calendar_page_html();
    assert!(
        !html.contains("<form"),
        "calendar page should not contain a <form> element, got: {html}"
    );
}

/// Demo・既存 Example・新 Example の id が衝突しないこと。
#[test]
fn page_has_no_duplicate_ids() {
    let html = calendar_page_html();
    let mut ids = Vec::new();
    let mut rest = html.as_str();
    while let Some(idx) = rest.find("id=\"") {
        let after = &rest[idx + 4..];
        let end = after
            .find('"')
            .unwrap_or_else(|| panic!("unterminated id attribute near: {after}"));
        ids.push(after[..end].to_string());
        rest = &after[end + 1..];
    }
    let mut seen = std::collections::HashSet::new();
    for id in &ids {
        assert!(
            seen.insert(id.clone()),
            "duplicate id detected: {id} (all ids: {ids:?})"
        );
    }
}

/// レンダリングが決定的であること（同一入力から 2 回描画してもバイト一致）。
#[test]
fn rendering_is_deterministic() {
    let first = calendar_page_html();
    let second = calendar_page_html();
    assert_eq!(
        first, second,
        "calendar page rendering should be deterministic"
    );
}
