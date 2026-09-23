//! `counter` 部品の契約テスト（イシュー #2655）。
//!
//! `crates/wireframe-ui/tests/tag.rs`・`crates/wireframe-ui/tests/avatar.rs`
//! と同型の観点（非対話制約・XSS 回帰・CSS 配線）を単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{counter, wireframe_css, Primary, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = counter("3", size, Primary(false));
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-counter {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<span"));
        assert!(html.trim_end().ends_with("</span>"));
    }
}

#[test]
fn primary_true_appends_primary_class_and_false_omits_it() {
    let primary = render(&counter("3", Size::Md, Primary(true)));
    assert!(primary.contains(r#"class="fw-wire-counter fw-wire-size-md fw-wire-primary""#));

    let plain = render(&counter("3", Size::Md, Primary(false)));
    assert!(!plain.contains("fw-wire-primary"));
}

#[test]
fn count_text_is_rendered_verbatim_for_typical_values() {
    for value in ["3", "42", "99+"] {
        let html = render(&counter(value, Size::Md, Primary(false)));
        assert!(html.contains(value), "expected {value:?} in {html:?}");
    }
}

#[test]
fn empty_count_renders_empty_badge() {
    let html = render(&counter("", Size::Md, Primary(false)));
    assert!(html.contains(r#"class="fw-wire-counter fw-wire-size-md""#));
    assert!(html.contains("></span>") || html.ends_with("></span>"));
}

#[test]
fn xss_regression_count_is_escaped() {
    let payload_a = "<script>alert(1)</script>";
    let payload_b = "\"><img src=x onerror=alert(1)>";
    let html_a = render(&counter(payload_a, Size::Md, Primary(false)));
    let html_b = render(&counter(payload_b, Size::Md, Primary(false)));

    assert!(!html_a.contains(payload_a));
    assert!(html_a.contains("&lt;script&gt;"));
    assert!(!html_b.contains(payload_b));
    assert!(html_b.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style_or_data_attributes() {
    let html = render(&counter("3", Size::Md, Primary(true)));
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
fn counter_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::counter::COUNTER_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::counter::COUNTER_CSS));
}

#[test]
fn counter_css_declares_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::counter::COUNTER_CSS;
    for line in css.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('.') {
            assert!(
                trimmed.starts_with(".fw-wire-"),
                "selector line should start with .fw-wire-: {line:?}"
            );
        }
    }
    assert!(css.contains("var(--fw-wire-font-size"));
    assert!(!css.contains("--fandhe-"));
    assert!(!css.contains(" fd-"));
}

#[test]
fn counter_css_keeps_single_digit_round() {
    let css = fandhe_frontend_wireframe_ui::counter::COUNTER_CSS;
    let root_start = css
        .find(".fw-wire-counter {")
        .expect("root rule must exist");
    let root_end = css[root_start..]
        .find('}')
        .map(|end| root_start + end)
        .expect("root rule must be closed");
    let root_rule = &css[root_start..root_end];
    assert!(root_rule.contains("min-width"));
    assert!(root_rule.contains("border-radius: 999px"));
}
