//! `emoji` 部品の契約テスト（イシュー #2654）。
//!
//! `crates/wireframe-ui/tests/text.rs`・`crates/wireframe-ui/tests/avatar.rs`・
//! `crates/docs-site/tests/wireframes_contract.rs` と同型の観点
//! （非対話制約・XSS 回帰・CSS 配線）を wireframe-ui 側で単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{emoji, wireframe_css, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = emoji("🙂", size);
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-emoji {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<span"));
        assert!(html.trim_end().ends_with("</span>"));
    }
}

#[test]
fn glyph_is_rendered_as_text_child() {
    let html = render(&emoji("🙂", Size::Md));
    assert!(html.contains('🙂'));
    assert!(!html.contains("<svg"));
}

#[test]
fn multi_codepoint_sequence_is_passed_through() {
    let sequence = "👩\u{200d}💻";
    let html = render(&emoji(sequence, Size::Md));
    assert!(html.contains(sequence));
}

#[test]
fn empty_glyph_renders_empty_span() {
    let html = render(&emoji("", Size::Md));
    assert_eq!(
        html,
        r#"<span class="fw-wire-emoji fw-wire-size-md"></span>"#
    );
}

#[test]
fn xss_regression_glyph_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&emoji(payload, Size::Md));
    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));

    let attr_payload = "\"><img src=x onerror=alert(1)>";
    let attr_html = render(&emoji(attr_payload, Size::Md));
    assert!(!attr_html.contains("<img src=x onerror=alert(1)>"));
    assert!(attr_html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style_or_media() {
    let html = render(&emoji("🎉", Size::Md));
    for forbidden in [
        " role=\"",
        " aria-",
        " tabindex=\"",
        " style=\"",
        "<button",
        "<a ",
        "href=",
        "src=",
        "javascript:",
        " onclick=\"",
        " onload=\"",
        " onerror=\"",
        "<img",
        "<input",
        "data-active",
        "data-disabled",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn emoji_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::emoji::EMOJI_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::emoji::EMOJI_CSS));
}

#[test]
fn emoji_css_selectors_use_fw_wire_prefix_and_reference_tokens_not_literals() {
    let css = fandhe_frontend_wireframe_ui::emoji::EMOJI_CSS;
    for selector in [".fw-wire-emoji {", ".fw-wire-emoji:empty {"] {
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

    assert!(css.contains("filter: grayscale(1)"));
    assert!(css.contains("var(--fw-wire-font-size"));
    assert!(css.contains(":empty"));
    assert!(!css.contains('#'));
    assert!(!css.contains("rgb("));
}
