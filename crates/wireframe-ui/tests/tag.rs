//! `tag` 部品の契約テスト（イシュー #2619）。
//!
//! `crates/wireframe-ui/tests/annotation.rs` と同型の観点（非対話制約・
//! XSS 回帰・CSS 配線）を固定する。`remove` の非対話制約テストは、
//! [`crate::icon::x`] が付与する `data-icon="x"` のみを唯一の許容
//! `data-` 出現として検証する（共通の禁止リストから `data-` を削って
//! 弱体化しない）。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{icon, tag, wireframe_css, Primary, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = tag("draft", size, Primary(false), None);
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-tag {}""#, size.class());
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
    let with_primary = render(&tag("t", Size::Md, Primary(true), None));
    assert!(with_primary.contains(r#"class="fw-wire-tag fw-wire-size-md fw-wire-primary""#));

    let without_primary = render(&tag("t", Size::Md, Primary(false), None));
    assert!(!without_primary.contains("fw-wire-primary"));
}

#[test]
fn some_remove_renders_remove_icon_part_and_none_omits_it_entirely() {
    let with_remove = render(&tag("t", Size::Md, Primary(false), Some(icon::x(Size::Md))));
    assert!(with_remove.contains(r#"class="fw-wire-tag-remove""#));
    assert!(with_remove.contains(r#"data-icon="x""#));
    assert!(with_remove.contains("<svg"));

    let without_remove = render(&tag("t", Size::Md, Primary(false), None));
    assert!(!without_remove.contains("fw-wire-tag-remove"));
    assert!(!without_remove.contains("<svg"));
}

#[test]
fn xss_regression_label_is_escaped() {
    let payload_a = "<script>alert(1)</script>";
    let payload_b = "\"><img src=x onerror=alert(1)>";

    let html_a = render(&tag(payload_a, Size::Md, Primary(false), None));
    assert!(!html_a.contains(payload_a));
    assert!(html_a.contains("&lt;script&gt;"));

    let html_b = render(&tag(payload_b, Size::Md, Primary(false), None));
    assert!(!html_b.contains(payload_b));
    assert!(html_b.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style_or_unexpected_data_attributes_when_not_removable() {
    let html = render(&tag("t", Size::Md, Primary(true), None));
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
fn removable_output_has_no_interactive_semantics_and_the_only_data_attribute_is_icon_data_icon() {
    let html = render(&tag("t", Size::Md, Primary(true), Some(icon::x(Size::Md))));
    for forbidden in [
        "data-active",
        "data-disabled",
        " role=\"",
        " tabindex=\"",
        " style=\"",
        "<button",
        "<a ",
        "href=",
        "javascript:",
        " onclick=\"",
        " onload=\"",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
    // アイコン基盤の識別子属性以外に `data-` は 1 回しか出現しない
    // （`icon::x` 自身の `data-icon="x"` のみ）。
    assert_eq!(html.matches("data-").count(), 1);
    assert!(html.contains(r#"data-icon="x""#));
}

#[test]
fn tag_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::tag::TAG_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::tag::TAG_CSS));
}

#[test]
fn tag_css_declares_the_five_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::tag::TAG_CSS;
    for selector in [
        ".fw-wire-tag {",
        ".fw-wire-tag-label {",
        ".fw-wire-tag-remove {",
        ".fw-wire-tag.fw-wire-primary {",
        ".fw-wire-tag.fw-wire-primary .fw-wire-tag-remove {",
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
