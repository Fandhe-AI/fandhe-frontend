//! `stack` 部品の契約テスト（イシュー #2610）。
//!
//! `crates/wireframe-ui/tests/annotation.rs` と同型の観点（非対話制約・
//! XSS 回帰・CSS 配線）を wireframe-ui 側で単体固定する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_wireframe_ui::{stack, wireframe_css, Orientation, Size, PARTS};

#[test]
fn renders_root_class_for_every_size_and_orientation() {
    for size in Size::ALL {
        for orientation in [Orientation::Horizontal, Orientation::Vertical] {
            let node = stack(vec![text("a")], orientation, size);
            let html = render(&node);
            let expected_class = format!(
                r#"class="fw-wire-stack {} {}""#,
                orientation.class(),
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
}

#[test]
fn children_are_rendered_in_the_given_order() {
    let node = stack(
        vec![text("first"), text("second"), text("third")],
        Orientation::Horizontal,
        Size::Md,
    );
    let html = render(&node);

    let first = html.find("first").expect("first should be present");
    let second = html.find("second").expect("second should be present");
    let third = html.find("third").expect("third should be present");
    assert!(first < second, "expected first before second: {html:?}");
    assert!(second < third, "expected second before third: {html:?}");
}

#[test]
fn empty_children_still_renders_root_div() {
    let node = stack(vec![], Orientation::Horizontal, Size::Md);
    let html = render(&node);
    assert!(html.starts_with("<div"));
    assert!(html.trim_end().ends_with("</div>"));
    assert!(html.contains(r#"class="fw-wire-stack fw-wire-horizontal fw-wire-size-md""#));
}

#[test]
fn xss_regression_children_text_is_escaped() {
    let payload_a = "<script>alert(1)</script>";
    let payload_b = "\"><img src=x onerror=alert(1)>";
    let html = render(&stack(
        vec![text(payload_a), text(payload_b)],
        Orientation::Horizontal,
        Size::Md,
    ));

    assert!(!html.contains(payload_a));
    assert!(!html.contains(payload_b));
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style_or_data_attributes() {
    let html = render(&stack(
        vec![text("a"), text("b")],
        Orientation::Vertical,
        Size::Lg,
    ));
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
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn stack_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::stack::STACK_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::stack::STACK_CSS));
}

#[test]
fn stack_css_declares_the_eight_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::stack::STACK_CSS;
    for selector in [
        ".fw-wire-stack {",
        ".fw-wire-stack.fw-wire-horizontal {",
        ".fw-wire-stack.fw-wire-vertical {",
        ".fw-wire-stack.fw-wire-size-xs {",
        ".fw-wire-stack.fw-wire-size-sm {",
        ".fw-wire-stack.fw-wire-size-md {",
        ".fw-wire-stack.fw-wire-size-lg {",
        ".fw-wire-stack.fw-wire-size-xl {",
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
