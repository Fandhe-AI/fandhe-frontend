//! `pagination` 部品の契約テスト（イシュー #2640）。
//!
//! `crates/wireframe-ui/tests/tabs.rs`（`active: Option<usize>` の
//! fail-closed 検証先例）・`calendar.rs`（アイコンの `aria-hidden` を
//! 許容する非対話制約テストの先例）と同型の観点（非対話制約・XSS 回帰・
//! CSS 配線）に加え、ページ項目（`Some`/ギャップ `None`）の出力順・
//! 先頭/前/次/末尾コントロールの表示可否を wireframe-ui 側で単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{pagination, wireframe_css, Size, PARTS};

const SAMPLE_PAGES: [Option<&str>; 5] = [Some("1"), Some("2"), None, Some("16"), Some("17")];

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = pagination(&SAMPLE_PAGES, Some(0), false, false, size);
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-pagination {}""#, size.class());
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
    let html = render(&pagination(&SAMPLE_PAGES, Some(0), true, true, Size::Md));
    let root_open_tag_end = html.find('>').expect("root open tag");
    let root_open_tag = &html[..=root_open_tag_end];
    assert!(root_open_tag.starts_with(r#"<div class="fw-wire-pagination"#));
    assert!(!root_open_tag.contains("data-"));
    assert!(!root_open_tag.contains(" role=\""));
    assert!(!root_open_tag.contains(" aria-"));
    assert!(!root_open_tag.contains(" tabindex=\""));
    assert!(!root_open_tag.contains(" style=\""));
}

#[test]
fn item_and_gap_counts_and_order_match_input() {
    let html = render(&pagination(&SAMPLE_PAGES, None, false, false, Size::Md));
    assert_eq!(
        html.matches(r#"class="fw-wire-pagination-item""#).count(),
        4
    );
    assert_eq!(html.matches(r#"class="fw-wire-pagination-gap""#).count(), 1);
    // 入力順どおりにラベルが並ぶ。
    let pos1 = html.find(">1<").expect("label 1");
    let pos2 = html.find(">2<").expect("label 2");
    let pos16 = html.find(">16<").expect("label 16");
    let pos17 = html.find(">17<").expect("label 17");
    assert!(pos1 < pos2);
    assert!(pos2 < pos16);
    assert!(pos16 < pos17);
}

#[test]
fn active_marks_only_matching_page_cell() {
    let html = render(&pagination(&SAMPLE_PAGES, Some(1), false, false, Size::Md));
    assert_eq!(html.matches(r#"data-active="""#).count(), 1);
}

#[test]
fn active_none_leaves_no_indicator() {
    let html = render(&pagination(&SAMPLE_PAGES, None, false, false, Size::Md));
    assert!(!html.contains("data-active"));
}

#[test]
fn active_out_of_range_leaves_no_indicator() {
    let html = render(&pagination(&SAMPLE_PAGES, Some(99), false, false, Size::Md));
    assert!(!html.contains("data-active"));
    let html_max = render(&pagination(
        &SAMPLE_PAGES,
        Some(usize::MAX),
        false,
        false,
        Size::Md,
    ));
    assert!(!html_max.contains("data-active"));
}

#[test]
fn active_pointing_at_gap_leaves_no_indicator() {
    // SAMPLE_PAGES[2] はギャップ（None）。
    let html = render(&pagination(&SAMPLE_PAGES, Some(2), false, false, Size::Md));
    assert!(!html.contains("data-active"));
}

#[test]
fn controls_are_shown_in_order_first_previous_items_next_last() {
    let html = render(&pagination(&SAMPLE_PAGES, None, true, true, Size::Md));
    let first = html.find("fw-wire-pagination-first").expect("first");
    let previous = html.find("fw-wire-pagination-previous").expect("previous");
    let item = html.find("fw-wire-pagination-item").expect("item");
    let next = html.find("fw-wire-pagination-next").expect("next");
    let last = html.find("fw-wire-pagination-last").expect("last");
    assert!(first < previous);
    assert!(previous < item);
    assert!(item < next);
    assert!(next < last);
}

#[test]
fn prev_next_only_shows_two_controls() {
    let html = render(&pagination(&SAMPLE_PAGES, None, true, false, Size::Md));
    assert!(html.contains("fw-wire-pagination-previous"));
    assert!(html.contains("fw-wire-pagination-next"));
    assert!(!html.contains("fw-wire-pagination-first"));
    assert!(!html.contains("fw-wire-pagination-last"));
}

#[test]
fn first_last_only_shows_two_controls() {
    let html = render(&pagination(&SAMPLE_PAGES, None, false, true, Size::Md));
    assert!(html.contains("fw-wire-pagination-first"));
    assert!(html.contains("fw-wire-pagination-last"));
    assert!(!html.contains("fw-wire-pagination-previous"));
    assert!(!html.contains("fw-wire-pagination-next"));
}

#[test]
fn no_controls_when_both_flags_false() {
    let html = render(&pagination(&SAMPLE_PAGES, None, false, false, Size::Md));
    assert!(!html.contains("fw-wire-pagination-control"));
}

#[test]
fn first_and_last_controls_render_two_caret_icons_each() {
    let html = render(&pagination(&[], None, false, true, Size::Md));
    // caret-left/caret-right アイコンはそれぞれ 2 個ずつ（先頭・末尾で計 4）。
    assert_eq!(html.matches(r#"data-icon="caret-left""#).count(), 2);
    assert_eq!(html.matches(r#"data-icon="caret-right""#).count(), 2);
}

#[test]
fn prev_next_controls_render_one_caret_icon_each() {
    let html = render(&pagination(&[], None, true, false, Size::Md));
    assert_eq!(html.matches(r#"data-icon="caret-left""#).count(), 1);
    assert_eq!(html.matches(r#"data-icon="caret-right""#).count(), 1);
}

#[test]
fn empty_pages_slice_does_not_panic() {
    let html = render(&pagination(&[], None, false, false, Size::Md));
    assert!(!html.contains("fw-wire-pagination-item"));
    assert!(!html.contains("fw-wire-pagination-gap"));
    assert!(html.starts_with("<div"));
    assert!(html.trim_end().ends_with("</div>"));
}

#[test]
fn xss_regression_page_label_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let pages = [Some(payload)];
    let html = render(&pagination(&pages, None, false, false, Size::Md));
    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn xss_regression_page_label_with_quote_payload_is_escaped() {
    let payload = "\"><img src=x onerror=alert(1)>";
    let pages = [Some(payload)];
    let html = render(&pagination(&pages, None, false, false, Size::Md));
    assert!(!html.contains(payload));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style() {
    let html = render(&pagination(&SAMPLE_PAGES, Some(0), true, true, Size::Md));
    for forbidden in [
        "<nav",
        "<a ",
        "href=",
        "javascript:",
        "<button",
        "<input",
        "<select",
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
fn pagination_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::pagination::PAGINATION_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::pagination::PAGINATION_CSS));
}

#[test]
fn pagination_css_declares_expected_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::pagination::PAGINATION_CSS;
    for selector in [
        ".fw-wire-pagination {",
        ".fw-wire-pagination-item {",
        ".fw-wire-pagination-item[data-active] {",
        ".fw-wire-pagination-control {",
        ".fw-wire-pagination-gap {",
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
