//! `image` 部品の契約テスト（イシュー #2660）。
//!
//! `crates/wireframe-ui/tests/avatar.rs`・`crates/wireframe-ui/tests/counter.rs`
//! と同型の観点（非対話制約・XSS 回帰・CSS 配線）を単体固定する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_wireframe_ui::{icon, image, wireframe_css, Primary, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = image(None, size, false, Primary(false));
        let html = render(&node);
        let expected_class = format!(
            r#"class="fw-wire-image {} fw-wire-image-placeholder""#,
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
fn none_content_renders_placeholder_class_with_no_children() {
    let html = render(&image(None, Size::Md, false, Primary(false)));
    assert!(html.contains("fw-wire-image-placeholder"));
    assert!(html.contains("></div>") || html.ends_with("></div>"));
    assert!(!html.contains("<svg"));
}

#[test]
fn some_content_omits_placeholder_class_and_shows_slot() {
    let with_icon = render(&image(
        Some(icon::image(Size::Md)),
        Size::Md,
        false,
        Primary(false),
    ));
    assert!(!with_icon.contains("fw-wire-image-placeholder"));
    assert!(with_icon.contains(r#"data-icon="image""#));

    let with_text = render(&image(
        Some(text("Caption")),
        Size::Md,
        false,
        Primary(false),
    ));
    assert!(!with_text.contains("fw-wire-image-placeholder"));
    assert!(with_text.contains("Caption"));
}

#[test]
fn circle_true_appends_circle_class_and_false_omits_it() {
    let circular = render(&image(None, Size::Md, true, Primary(false)));
    assert!(circular.contains("fw-wire-image-circle"));

    let square = render(&image(None, Size::Md, false, Primary(false)));
    assert!(!square.contains("fw-wire-image-circle"));
}

#[test]
fn primary_true_appends_primary_class_and_false_omits_it() {
    let primary = render(&image(None, Size::Md, false, Primary(true)));
    assert!(primary.contains("fw-wire-primary"));

    let plain = render(&image(None, Size::Md, false, Primary(false)));
    assert!(!plain.contains("fw-wire-primary"));
}

#[test]
fn class_order_is_root_size_placeholder_circle_primary() {
    let html = render(&image(None, Size::Md, true, Primary(true)));
    assert!(html.contains(
        r#"class="fw-wire-image fw-wire-size-md fw-wire-image-placeholder fw-wire-image-circle fw-wire-primary""#
    ));
}

#[test]
fn xss_regression_slot_text_is_escaped() {
    let payload_a = "<script>alert(1)</script>";
    let payload_b = "\"><img src=x onerror=alert(1)>";
    let html_a = render(&image(
        Some(text(payload_a)),
        Size::Md,
        false,
        Primary(false),
    ));
    let html_b = render(&image(
        Some(text(payload_b)),
        Size::Md,
        false,
        Primary(false),
    ));

    assert!(!html_a.contains(payload_a));
    assert!(html_a.contains("&lt;script&gt;"));
    assert!(!html_b.contains(payload_b));
    assert!(html_b.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_style_or_image_url_api() {
    let html = render(&image(Some(text("Caption")), Size::Md, true, Primary(true)));
    for forbidden in [
        " role=\"",
        " aria-expanded",
        " tabindex=\"",
        " style=\"",
        "href=",
        "src=",
        "<img",
        "<a ",
        "<button",
        " onclick=\"",
        " onerror=\"",
        "javascript:",
        "data-",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn image_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::image::IMAGE_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::image::IMAGE_CSS));
}

#[test]
fn image_css_declares_selectors_with_fw_wire_prefix_and_no_literal_colors() {
    let css = fandhe_frontend_wireframe_ui::image::IMAGE_CSS;
    for line in css.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('.') {
            assert!(
                trimmed.starts_with(".fw-wire-"),
                "selector line should start with .fw-wire-: {line:?}"
            );
        }
    }
    assert!(css.contains("var(--fw-wire-control-size"));
    assert!(!css.contains("--fandhe-"));
    assert!(!css.contains('#'));
}

#[test]
fn image_css_scopes_border_radius_50_to_circle_rule_only() {
    let css = fandhe_frontend_wireframe_ui::image::IMAGE_CSS;
    let circle_start = css
        .find(".fw-wire-image.fw-wire-image-circle")
        .expect("circle selector must exist");
    let circle_end = css[circle_start..]
        .find('}')
        .map(|end| circle_start + end)
        .expect("circle rule must be closed");
    assert!(css[circle_start..circle_end].contains("border-radius: 50%"));
    assert!(!css[..circle_start].contains("50%"));
}

#[test]
fn image_css_places_linear_gradient_only_in_placeholder_rule() {
    let css = fandhe_frontend_wireframe_ui::image::IMAGE_CSS;
    let placeholder_start = css
        .find(".fw-wire-image.fw-wire-image-placeholder")
        .expect("placeholder selector must exist");
    assert!(!css[..placeholder_start].contains("linear-gradient"));
    assert!(css[placeholder_start..].contains("linear-gradient"));
}
