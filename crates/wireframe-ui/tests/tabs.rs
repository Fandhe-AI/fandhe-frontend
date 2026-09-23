//! `tabs` 部品の契約テスト（イシュー #2638）。
//!
//! `crates/wireframe-ui/tests/radio.rs`（選択状態を既存 `Active` で表す
//! 先例）と同型の観点（非対話制約・XSS 回帰・CSS 配線）に加え、`items`
//! スライスの項目数可変性、空スライスでの panic なし、`active` の
//! `None`/範囲外/正常値それぞれの `data-active` 出力を単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{tabs, wireframe_css, Orientation, Size, PARTS};

const ITEMS: [&str; 3] = ["概要", "詳細", "設定"];

#[test]
fn renders_root_class_for_every_size_and_orientation() {
    for size in Size::ALL {
        for orientation in [Orientation::Horizontal, Orientation::Vertical] {
            let node = tabs(&ITEMS, Some(0), orientation, size);
            let html = render(&node);
            let expected_class = format!(
                r#"class="fw-wire-tabs {} {}""#,
                size.class(),
                orientation.class()
            );
            assert!(
                html.contains(&expected_class),
                "expected {expected_class:?} in {html:?}"
            );
            assert!(html.starts_with("<div"));
            assert!(html.trim_end().ends_with("</div>"));
        }
    }
}

#[test]
fn item_count_matches_items_len() {
    let html = render(&tabs(&ITEMS, None, Orientation::Horizontal, Size::Md));
    assert_eq!(
        html.matches(r#"class="fw-wire-tabs-item""#).count(),
        ITEMS.len()
    );
}

#[test]
fn empty_items_does_not_panic_and_renders_no_items() {
    let html = render(&tabs(&[], None, Orientation::Horizontal, Size::Md));
    assert!(!html.contains("fw-wire-tabs-item"));
    assert!(html.contains(r#"class="fw-wire-tabs fw-wire-size-md fw-wire-horizontal""#));
}

#[test]
fn active_some_marks_exactly_one_item() {
    let html = render(&tabs(&ITEMS, Some(1), Orientation::Horizontal, Size::Md));
    assert_eq!(html.matches(r#"data-active="""#).count(), 1);

    // 選択中の項目（詳細）の直前にだけ data-active が付く。
    let idx_label = html.find("詳細").expect("label must be present");
    let idx_active = html
        .find(r#"data-active="""#)
        .expect("data-active must be present");
    assert!(idx_active < idx_label);
}

#[test]
fn active_none_adds_no_data_active() {
    let html = render(&tabs(&ITEMS, None, Orientation::Horizontal, Size::Md));
    assert!(!html.contains("data-active"));
}

#[test]
fn active_out_of_range_adds_no_data_active() {
    let at_len = render(&tabs(
        &ITEMS,
        Some(ITEMS.len()),
        Orientation::Horizontal,
        Size::Md,
    ));
    assert!(!at_len.contains("data-active"));

    let at_max = render(&tabs(
        &ITEMS,
        Some(usize::MAX),
        Orientation::Horizontal,
        Size::Md,
    ));
    assert!(!at_max.contains("data-active"));
}

#[test]
fn labels_preserve_order() {
    let html = render(&tabs(&ITEMS, None, Orientation::Horizontal, Size::Md));
    let positions: Vec<usize> = ITEMS
        .iter()
        .map(|label| html.find(label).expect("label must be present"))
        .collect();
    assert!(positions.windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn xss_regression_label_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&tabs(&[payload], None, Orientation::Horizontal, Size::Md));
    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn xss_regression_label_with_quote_payload_is_escaped() {
    let payload = "\"><img src=x onerror=alert(1)>";
    let html = render(&tabs(&[payload], None, Orientation::Horizontal, Size::Md));
    assert!(!html.contains(payload));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style() {
    let html = render(&tabs(&ITEMS, Some(0), Orientation::Vertical, Size::Md));
    for forbidden in [
        "<input",
        "<label",
        "<button",
        "<select",
        " role=\"",
        " tabindex=\"",
        " style=\"",
        "<a ",
        "href=",
        "javascript:",
        " onclick=\"",
        " aria-",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn tabs_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::tabs::TABS_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::tabs::TABS_CSS));
}

#[test]
fn tabs_css_declares_expected_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::tabs::TABS_CSS;
    for selector in [
        ".fw-wire-tabs {",
        ".fw-wire-tabs.fw-wire-horizontal {",
        ".fw-wire-tabs.fw-wire-vertical {",
        ".fw-wire-tabs-item {",
        ".fw-wire-tabs.fw-wire-vertical .fw-wire-tabs-item {",
        ".fw-wire-tabs-item[data-active] {",
        ".fw-wire-tabs.fw-wire-vertical .fw-wire-tabs-item[data-active] {",
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
