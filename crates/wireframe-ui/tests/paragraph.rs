//! `paragraph` 部品の契約テスト（イシュー #2615）。
//!
//! `crates/wireframe-ui/tests/annotation.rs`・`crates/docs-site/tests/wireframes_contract.rs`
//! と同型の観点（非対話制約・XSS 回帰・CSS 配線）を wireframe-ui 側で
//! 単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{paragraph, wireframe_css, Bold, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = paragraph("本文", size, Bold(false));
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-paragraph {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn bold_true_appends_bold_class_and_false_omits_it() {
    let with_bold = render(&paragraph("本文", Size::Md, Bold(true)));
    assert!(with_bold.contains(r#"class="fw-wire-paragraph fw-wire-size-md fw-wire-bold""#));

    let without_bold = render(&paragraph("本文", Size::Md, Bold(false)));
    assert!(!without_bold.contains("fw-wire-bold"));
}

#[test]
fn multiline_content_keeps_raw_newlines_and_does_not_convert_to_br() {
    let html = render(&paragraph("1 行目\n2 行目\n3 行目", Size::Md, Bold(false)));
    assert!(html.contains("1 行目\n2 行目\n3 行目"));
    assert!(!html.contains("<br"));
}

#[test]
fn xss_regression_content_with_and_without_newlines_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&paragraph(payload, Size::Md, Bold(false)));
    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));

    let payload_with_newline = "1 行目\n\"><img src=x onerror=alert(1)>";
    let html_with_newline = render(&paragraph(payload_with_newline, Size::Md, Bold(false)));
    assert!(!html_with_newline.contains("<img src=x onerror=alert(1)>"));
    assert!(html_with_newline.contains("&quot;"));
    assert!(html_with_newline.contains("1 行目\n"));
}

#[test]
fn output_has_no_interactive_semantics_or_style_or_data_attributes() {
    let html = render(&paragraph("本文", Size::Md, Bold(true)));
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
fn paragraph_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::paragraph::PARAGRAPH_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::paragraph::PARAGRAPH_CSS));
}

#[test]
fn paragraph_css_declares_the_two_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::paragraph::PARAGRAPH_CSS;
    for selector in [".fw-wire-paragraph {", ".fw-wire-paragraph.fw-wire-bold {"] {
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

    assert!(css.contains("white-space: pre-line"));
    assert!(!css.contains("--fandhe-"));
    assert!(!css.contains(" fd-"));
    assert!(!css.contains('#'));
    assert!(!css.contains("rgb("));
}
