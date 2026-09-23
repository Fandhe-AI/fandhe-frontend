//! `divider` 部品の契約テスト（イシュー #2612）。
//!
//! `crates/wireframe-ui/tests/annotation.rs`・`crates/docs-site/tests/wireframes_contract.rs`
//! と同型の観点（非対話制約・XSS 回帰・CSS 配線）を wireframe-ui 側で
//! 単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{divider, wireframe_css, Orientation, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = divider(None, size, Orientation::Horizontal);
        let html = render(&node);
        let expected_class = format!(
            r#"class="fw-wire-divider {} fw-wire-horizontal""#,
            size.class()
        );
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn orientation_controls_horizontal_or_vertical_class_exclusively() {
    let horizontal = render(&divider(None, Size::Md, Orientation::Horizontal));
    assert!(horizontal.contains("fw-wire-horizontal"));
    assert!(!horizontal.contains("fw-wire-vertical"));

    let vertical = render(&divider(None, Size::Md, Orientation::Vertical));
    assert!(vertical.contains("fw-wire-vertical"));
    assert!(!vertical.contains("fw-wire-horizontal"));
}

#[test]
fn label_some_renders_part_and_none_omits_it_entirely() {
    let with_label = render(&divider(Some("または"), Size::Md, Orientation::Horizontal));
    assert!(with_label.contains(r#"class="fw-wire-divider-label""#));
    assert!(with_label.contains("または"));

    let without_label = render(&divider(None, Size::Md, Orientation::Horizontal));
    assert!(!without_label.contains("fw-wire-divider-label"));
}

#[test]
fn xss_regression_label_is_escaped() {
    let payload_a = "<script>alert(1)</script>";
    let payload_b = "\"><img src=x onerror=alert(1)>";

    let html_a = render(&divider(Some(payload_a), Size::Md, Orientation::Horizontal));
    assert!(!html_a.contains(payload_a));
    assert!(html_a.contains("&lt;script&gt;"));

    let html_b = render(&divider(Some(payload_b), Size::Md, Orientation::Horizontal));
    assert!(!html_b.contains(payload_b));
    assert!(html_b.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style_or_data_attributes() {
    let html = render(&divider(Some("l"), Size::Md, Orientation::Vertical));
    for forbidden in [
        " role=\"",
        " aria-",
        " tabindex=\"",
        " style=\"",
        "<button",
        "<a ",
        "href=",
        "javascript:",
        " onclick=\"",
        " onload=\"",
        "data-",
        "<hr",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn divider_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::divider::DIVIDER_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::divider::DIVIDER_CSS));
}

#[test]
fn divider_css_declares_expected_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::divider::DIVIDER_CSS;
    for selector in [
        ".fw-wire-divider {",
        ".fw-wire-divider::before,",
        ".fw-wire-divider.fw-wire-horizontal {",
        ".fw-wire-divider.fw-wire-horizontal::before,",
        ".fw-wire-divider.fw-wire-vertical {",
        ".fw-wire-divider.fw-wire-vertical::before,",
        ".fw-wire-divider-label {",
        ".fw-wire-divider.fw-wire-vertical .fw-wire-divider-label {",
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
