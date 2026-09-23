//! `text` 部品の契約テスト（イシュー #2614）。
//!
//! `crates/wireframe-ui/tests/paragraph.rs`・`crates/docs-site/tests/wireframes_contract.rs`
//! と同型の観点（非対話制約・XSS 回帰・CSS 配線）を wireframe-ui 側で
//! 単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{text, wireframe_css, Bold, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = text("見出し", size, Bold(false));
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-text {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<span"));
        assert!(html.trim_end().ends_with("</span>"));
    }
}

#[test]
fn bold_true_appends_bold_class_and_false_omits_it() {
    let with_bold = render(&text("見出し", Size::Md, Bold(true)));
    assert!(with_bold.contains(r#"class="fw-wire-text fw-wire-size-md fw-wire-bold""#));

    let without_bold = render(&text("見出し", Size::Md, Bold(false)));
    assert!(!without_bold.contains("fw-wire-bold"));
}

#[test]
fn empty_content_does_not_panic_and_renders_empty_span() {
    let html = render(&text("", Size::Md, Bold(false)));
    assert!(html.starts_with("<span"));
    assert!(html.trim_end().ends_with("</span>"));
}

#[test]
fn xss_regression_content_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&text(payload, Size::Md, Bold(false)));
    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));

    let attr_payload = "\"><img src=x onerror=alert(1)>";
    let attr_html = render(&text(attr_payload, Size::Md, Bold(false)));
    assert!(!attr_html.contains("<img src=x onerror=alert(1)>"));
    assert!(attr_html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style_or_data_attributes() {
    let html = render(&text("見出し", Size::Md, Bold(true)));
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
fn text_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::text::TEXT_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::text::TEXT_CSS));
}

#[test]
fn text_css_declares_the_two_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::text::TEXT_CSS;
    for selector in [".fw-wire-text {", ".fw-wire-text.fw-wire-bold {"] {
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

    assert!(css.contains("white-space: nowrap"));
    assert!(css.contains("text-overflow: ellipsis"));
    assert!(!css.contains("--fandhe-"));
    assert!(!css.contains(" fd-"));
    assert!(!css.contains('#'));
    assert!(!css.contains("rgb("));
}
