//! `annotation` 部品の契約テスト（イシュー #2617）。
//!
//! `crates/wireframe-ui/src/icon.rs` の doctest・`crates/docs-site/tests/wireframes_contract.rs`
//! と同型の観点（非対話制約・XSS 回帰・CSS 配線）を wireframe-ui 側で
//! 単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{annotation, wireframe_css, Primary, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = annotation("タイトル", None, size, Primary(false));
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-annotation {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn primary_true_appends_primary_class_and_false_omits_it() {
    let with_primary = render(&annotation("t", None, Size::Md, Primary(true)));
    assert!(with_primary.contains(r#"class="fw-wire-annotation fw-wire-size-md fw-wire-primary""#));

    let without_primary = render(&annotation("t", None, Size::Md, Primary(false)));
    assert!(!without_primary.contains("fw-wire-primary"));
}

#[test]
fn description_some_renders_part_and_none_omits_it_entirely() {
    let with_description = render(&annotation("t", Some("補足"), Size::Md, Primary(false)));
    assert!(with_description.contains(r#"class="fw-wire-annotation-description""#));
    assert!(with_description.contains("補足"));

    let without_description = render(&annotation("t", None, Size::Md, Primary(false)));
    assert!(!without_description.contains("fw-wire-annotation-description"));
}

#[test]
fn xss_regression_title_and_description_are_escaped() {
    let payload_a = "<script>alert(1)</script>";
    let payload_b = "\"><img src=x onerror=alert(1)>";
    let html = render(&annotation(
        payload_a,
        Some(payload_b),
        Size::Md,
        Primary(false),
    ));

    assert!(!html.contains(payload_a));
    assert!(!html.contains(payload_b));
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style_or_data_attributes() {
    let html = render(&annotation("t", Some("d"), Size::Md, Primary(true)));
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
fn annotation_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::annotation::ANNOTATION_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::annotation::ANNOTATION_CSS));
}

#[test]
fn annotation_css_declares_the_five_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::annotation::ANNOTATION_CSS;
    for selector in [
        ".fw-wire-annotation {",
        ".fw-wire-annotation-title {",
        ".fw-wire-annotation-description {",
        ".fw-wire-annotation.fw-wire-primary {",
        ".fw-wire-annotation.fw-wire-primary .fw-wire-annotation-description {",
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
