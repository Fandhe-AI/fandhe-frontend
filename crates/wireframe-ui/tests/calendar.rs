//! `calendar` 部品の契約テスト（イシュー #2632）。
//!
//! `crates/wireframe-ui/tests/select.rs`（アイコンの `aria-hidden` を
//! 許容する `output_has_no_interactive_semantics_or_style` 先例）・
//! `grid.rs`（資源有界化 clamp のテスト先例）と同型の観点
//! （非対話制約・XSS 回帰・CSS 配線・資源有界化）に加え、選択日を表す
//! `Active`（`data-active`）の再利用・空きマス・6 週飽和を wireframe-ui
//! 側で単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{calendar, wireframe_css, Size, PARTS};

/// テスト用の週配列（2026 年 9 月相当、月初 2 マスが空き、10 日目まで）。
const SAMPLE_WEEKS: [[Option<u32>; 7]; 2] = [
    [None, None, Some(1), Some(2), Some(3), Some(4), Some(5)],
    [
        Some(6),
        Some(7),
        Some(8),
        Some(9),
        Some(10),
        Some(11),
        Some(12),
    ],
];

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = calendar("2026 年 9 月", &SAMPLE_WEEKS, None, size);
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-calendar {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn root_open_tag_has_only_class() {
    let html = render(&calendar("2026 年 9 月", &SAMPLE_WEEKS, Some(8), Size::Md));
    let root_open_tag_end = html.find('>').expect("root open tag");
    let root_open_tag = &html[..=root_open_tag_end];
    assert!(root_open_tag.starts_with(r#"<div class="fw-wire-calendar"#));
    assert!(!root_open_tag.contains("data-"));
    assert!(!root_open_tag.contains(" role=\""));
    assert!(!root_open_tag.contains(" aria-"));
    assert!(!root_open_tag.contains(" tabindex=\""));
    assert!(!root_open_tag.contains(" style=\""));
}

#[test]
fn month_label_is_rendered_in_label_part() {
    let html = render(&calendar("2026 年 9 月", &SAMPLE_WEEKS, None, Size::Md));
    assert!(html.contains(r#"class="fw-wire-calendar-label""#));
    assert!(html.contains("2026 年 9 月"));
}

#[test]
fn weekday_header_has_exactly_seven_textless_cells() {
    let html = render(&calendar("月", &SAMPLE_WEEKS, None, Size::Md));
    assert_eq!(
        html.matches(r#"class="fw-wire-calendar-weekday""#).count(),
        7
    );

    // weeks が空でも曜日ヘッダーは常に 7 個出力される。
    let empty_html = render(&calendar("月", &[], None, Size::Md));
    assert_eq!(
        empty_html
            .matches(r#"class="fw-wire-calendar-weekday""#)
            .count(),
        7
    );
    assert!(!empty_html.contains(r#"class="fw-wire-calendar-week""#));
}

#[test]
fn renders_seven_day_cells_per_week() {
    let html = render(&calendar("月", &SAMPLE_WEEKS, None, Size::Md));
    // SAMPLE_WEEKS は 2 週 × 7 マス = 14 セル（fw-wire-calendar-day /
    // -empty 双方を含む）。class 属性値は必ず "fw-wire-calendar-day" で
    // 始まる（空きマスは "fw-wire-calendar-day fw-wire-calendar-day-empty"
    // と続く）ため、`class="fw-wire-calendar-day` プレフィックス一致で
    // 開始タグ数を正確に数えられる。
    assert_eq!(html.matches(r#"class="fw-wire-calendar-day"#).count(), 14);
    assert_eq!(html.matches(r#"class="fw-wire-calendar-week""#).count(), 2);
}

#[test]
fn none_cells_render_as_empty_cells_without_text() {
    let html = render(&calendar("月", &SAMPLE_WEEKS, None, Size::Md));
    // SAMPLE_WEEKS の先頭週は None が 2 マス。
    assert_eq!(html.matches("fw-wire-calendar-day-empty").count(), 2);
}

#[test]
fn weeks_are_clamped_to_max_weeks() {
    let seven_weeks: [[Option<u32>; 7]; 7] = [
        [Some(1), None, None, None, None, None, None],
        [Some(2), None, None, None, None, None, None],
        [Some(3), None, None, None, None, None, None],
        [Some(4), None, None, None, None, None, None],
        [Some(5), None, None, None, None, None, None],
        [Some(6), None, None, None, None, None, None],
        [Some(7), None, None, None, None, None, None],
    ];
    let html = render(&calendar("月", &seven_weeks, None, Size::Md));
    assert_eq!(html.matches(r#"class="fw-wire-calendar-week""#).count(), 6);
    // 7 週目（Some(7)）は飽和により出力されない。
    assert!(!html.contains(">7<"));
}

#[test]
fn empty_weeks_slice_does_not_panic() {
    let html = render(&calendar("月", &[], None, Size::Md));
    assert_eq!(html.matches(r#"class="fw-wire-calendar-week""#).count(), 0);
}

#[test]
fn selected_day_marks_only_matching_cells_with_data_active() {
    let with_selection = render(&calendar("月", &SAMPLE_WEEKS, Some(8), Size::Md));
    assert_eq!(with_selection.matches(r#"data-active="""#).count(), 1);

    let no_selection = render(&calendar("月", &SAMPLE_WEEKS, None, Size::Md));
    assert_eq!(no_selection.matches("data-active").count(), 0);

    // グリッドに存在しない日を選択しても data-active は 0 件。
    let out_of_range = render(&calendar("月", &SAMPLE_WEEKS, Some(99), Size::Md));
    assert_eq!(out_of_range.matches("data-active").count(), 0);
}

#[test]
fn selected_day_marks_every_matching_cell_when_duplicated() {
    let duplicated_weeks: [[Option<u32>; 7]; 2] = [
        [Some(1), None, None, None, None, None, None],
        [Some(1), None, None, None, None, None, None],
    ];
    let html = render(&calendar("月", &duplicated_weeks, Some(1), Size::Md));
    assert_eq!(html.matches(r#"data-active="""#).count(), 2);
}

#[test]
fn xss_regression_month_label_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&calendar(payload, &SAMPLE_WEEKS, None, Size::Md));
    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn xss_regression_month_label_with_quote_payload_is_escaped() {
    let payload = "\"><img src=x onerror=alert(1)>";
    let html = render(&calendar(payload, &SAMPLE_WEEKS, None, Size::Md));
    assert!(!html.contains(payload));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style() {
    let html = render(&calendar("月", &SAMPLE_WEEKS, Some(8), Size::Md));
    // `aria-hidden="true"` はアイコン基盤（`crate::icon`）が装飾用途として
    // 付与する既定の属性であり、対話的 ARIA ではないため許容する
    // （`crates/wireframe-ui/tests/select.rs` と同型の判定）。
    for forbidden in [
        "<table",
        "<th",
        "<button",
        "<input",
        "<select",
        "<a ",
        "href=",
        "javascript:",
        " role=\"",
        " tabindex=\"",
        " style=\"",
        " onclick=\"",
        "aria-expanded",
        "aria-haspopup",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
    let aria_attrs: Vec<&str> = html
        .split(' ')
        .filter(|token| token.starts_with("aria-"))
        .collect();
    for attr in aria_attrs {
        assert!(
            attr.starts_with("aria-hidden="),
            "unexpected non-decorative aria attribute {attr:?} in {html:?}"
        );
    }
    // data- は data-active / data-icon（アイコン基盤の識別子）以外は出さない。
    let data_attrs: Vec<&str> = html
        .split(' ')
        .filter(|token| token.starts_with("data-"))
        .collect();
    for attr in data_attrs {
        assert!(
            attr.starts_with("data-active=") || attr.starts_with("data-icon="),
            "unexpected data attribute {attr:?} in {html:?}"
        );
    }
}

#[test]
fn calendar_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::calendar::CALENDAR_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::calendar::CALENDAR_CSS));
}

#[test]
fn calendar_css_declares_expected_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::calendar::CALENDAR_CSS;
    for selector in [
        ".fw-wire-calendar {",
        ".fw-wire-calendar-header {",
        ".fw-wire-calendar-nav {",
        ".fw-wire-calendar-label {",
        ".fw-wire-calendar-weekdays {",
        ".fw-wire-calendar-weekday {",
        ".fw-wire-calendar-grid {",
        ".fw-wire-calendar-week {",
        ".fw-wire-calendar-day {",
        ".fw-wire-calendar-day-empty {",
        ".fw-wire-calendar-day[data-active] {",
    ] {
        assert!(css.contains(selector), "missing selector {selector:?}");
    }

    for line in css.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('.') {
            assert!(
                trimmed.starts_with(".fw-wire-"),
                "selector line should start with .fw-wire-: {line:?}"
            );
        }
    }

    assert!(!css.contains("--fandhe-"));
    assert!(!css.contains(" fd-"));
}
