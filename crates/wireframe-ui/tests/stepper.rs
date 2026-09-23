//! `stepper` 部品の契約テスト（イシュー #2634）。
//!
//! `crates/wireframe-ui/tests/question.rs` と同型の観点（非対話制約・
//! XSS 回帰・CSS 配線）に加え、`stepper` 固有の状態写像（`data-complete`/
//! `data-active`）・範囲外 `active`・空スライスの規則を固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{stepper, wireframe_css, Size, PARTS};

const STEPS: [&str; 3] = ["アカウント作成", "プラン選択", "支払い"];

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let html = render(&stepper(&STEPS, 1, size));
        let expected_class = format!(r#"class="fw-wire-stepper {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn step_count_matches_steps_len_and_labels_and_numbers_are_rendered() {
    let html = render(&stepper(&STEPS, 1, Size::Md));
    assert_eq!(html.matches("fw-wire-stepper-step\"").count(), STEPS.len());
    for label in STEPS {
        assert!(html.contains(label), "missing label {label:?} in {html:?}");
    }
    for n in 1..=STEPS.len() {
        assert!(html.contains(&n.to_string()));
    }
}

#[test]
fn active_step_has_data_active_and_preceding_steps_have_data_complete() {
    let html = render(&stepper(&STEPS, 1, Size::Md));
    assert_eq!(html.matches("data-active").count(), 1);
    assert_eq!(html.matches("data-complete").count(), 1);
}

#[test]
fn active_zero_has_no_complete_steps() {
    let html = render(&stepper(&STEPS, 0, Size::Md));
    assert_eq!(html.matches("data-complete").count(), 0);
    assert_eq!(html.matches("data-active").count(), 1);
}

#[test]
fn active_out_of_range_marks_every_step_complete_and_no_active() {
    for active in [STEPS.len(), STEPS.len() + 10, usize::MAX] {
        let html = render(&stepper(&STEPS, active, Size::Md));
        assert_eq!(
            html.matches("data-complete").count(),
            STEPS.len(),
            "active={active} html={html:?}"
        );
        assert_eq!(html.matches("data-active").count(), 0, "active={active}");
    }
}

#[test]
fn empty_steps_renders_root_only_without_panicking() {
    let html = render(&stepper(&[], 0, Size::Md));
    assert!(html.contains(r#"class="fw-wire-stepper fw-wire-size-md""#));
    assert!(!html.contains("fw-wire-stepper-step"));
}

#[test]
fn xss_regression_step_labels_are_escaped() {
    let payload = "<script>alert(1)</script>";
    let payload2 = "\"><img src=x onerror=alert(1)>";
    let html = render(&stepper(&[payload, payload2], 0, Size::Md));
    assert!(!html.contains(payload));
    assert!(!html.contains(payload2));
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics() {
    let html = render(&stepper(&STEPS, 1, Size::Md));
    for forbidden in [
        " role=\"",
        " aria-",
        " tabindex=\"",
        " style=\"",
        "<button",
        "<a ",
        "href=",
        "<input",
        "<select",
        "<ol",
        "<li",
        "javascript:",
        " onclick=\"",
        " onload=\"",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn only_data_active_and_data_complete_appear_as_data_attributes() {
    let html = render(&stepper(&STEPS, 1, Size::Md));
    let total_data_attrs = html.matches("data-").count();
    let known_data_attrs =
        html.matches("data-active").count() + html.matches("data-complete").count();
    assert_eq!(
        total_data_attrs, known_data_attrs,
        "unexpected data-* attribute other than data-active/data-complete in {html:?}"
    );
}

#[test]
fn stepper_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::stepper::STEPPER_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::stepper::STEPPER_CSS));
}

#[test]
fn stepper_css_declares_the_expected_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::stepper::STEPPER_CSS;
    for selector in [
        ".fw-wire-stepper {",
        ".fw-wire-stepper-step {",
        ".fw-wire-stepper-indicator {",
        ".fw-wire-stepper-label {",
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
