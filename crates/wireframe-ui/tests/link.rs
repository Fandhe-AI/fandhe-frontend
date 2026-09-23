//! `link` 部品の契約テスト（イシュー #2618）。
//!
//! `crates/wireframe-ui/tests/annotation.rs` と同型の観点（非対話制約・
//! XSS 回帰・CSS 配線）に加え、`trailing` アイコンスロット（`Option<Node>`）
//! と `a[href]` 非出力を wireframe-ui 側で単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{icon, link, wireframe_css, Bold, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = link("詳細を見る", None, size, Bold(false));
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-link {}""#, size.class());
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
    let with_bold = render(&link("t", None, Size::Md, Bold(true)));
    assert!(with_bold.contains(r#"class="fw-wire-link fw-wire-size-md fw-wire-bold""#));

    let without_bold = render(&link("t", None, Size::Md, Bold(false)));
    assert!(!without_bold.contains("fw-wire-bold"));
}

#[test]
fn trailing_some_renders_icon_and_none_omits_it_entirely() {
    let with_trailing = render(&link(
        "t",
        Some(icon::external(Size::Md)),
        Size::Md,
        Bold(false),
    ));
    assert!(with_trailing.contains("<svg"));

    let without_trailing = render(&link("t", None, Size::Md, Bold(false)));
    assert!(!without_trailing.contains("<svg"));
}

#[test]
fn label_part_class_is_always_present() {
    let html = render(&link("ラベル", None, Size::Md, Bold(false)));
    assert!(html.contains(r#"class="fw-wire-link-label""#));
    assert!(html.contains("ラベル"));
}

#[test]
fn xss_regression_label_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&link(payload, None, Size::Md, Bold(false)));

    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn xss_regression_label_with_quote_payload_is_escaped() {
    let payload = "\"><img src=x onerror=alert(1)>";
    let html = render(&link(payload, None, Size::Md, Bold(false)));

    assert!(!html.contains(payload));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_link_attributes_or_data_attributes() {
    let html = render(&link("t", None, Size::Md, Bold(true)));
    for forbidden in [
        " role=\"",
        " aria-",
        " tabindex=\"",
        " style=\"",
        "<button",
        "<a ",
        "href=",
        "rel=",
        "target=",
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
fn link_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::link::LINK_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::link::LINK_CSS));
}

#[test]
fn link_css_declares_the_three_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::link::LINK_CSS;
    for selector in [
        ".fw-wire-link {",
        ".fw-wire-link-label {",
        ".fw-wire-link.fw-wire-bold {",
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
