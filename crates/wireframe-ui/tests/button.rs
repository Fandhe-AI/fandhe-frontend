//! `button` 部品の契約テスト（イシュー #2621）。
//!
//! `crates/wireframe-ui/tests/annotation.rs`・`link.rs` と同型の観点
//! （非対話制約・XSS 回帰・CSS 配線）に加え、`icon` アイコンスロット
//! （`Option<Node>`）と `disabled`（`data-disabled`）の条件付き出力を
//! wireframe-ui 側で単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{button, icon, wireframe_css, Disabled, Primary, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = button("送信", None, size, Primary(false), Disabled(false));
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-button {}""#, size.class());
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
    let with_primary = render(&button("t", None, Size::Md, Primary(true), Disabled(false)));
    assert!(with_primary.contains(r#"class="fw-wire-button fw-wire-size-md fw-wire-primary""#));

    let without_primary = render(&button(
        "t",
        None,
        Size::Md,
        Primary(false),
        Disabled(false),
    ));
    assert!(!without_primary.contains("fw-wire-primary"));
}

#[test]
fn disabled_true_emits_data_disabled_and_false_omits_all_data_attributes() {
    let with_disabled = render(&button("t", None, Size::Md, Primary(false), Disabled(true)));
    assert!(with_disabled.contains(r#"data-disabled="""#));

    let without_disabled = render(&button(
        "t",
        None,
        Size::Md,
        Primary(false),
        Disabled(false),
    ));
    assert!(!without_disabled.contains("data-"));
}

#[test]
fn icon_slot_some_renders_svg_before_label_and_none_omits_svg() {
    let with_icon = render(&button(
        "追加",
        Some(icon::plus(Size::Md)),
        Size::Md,
        Primary(false),
        Disabled(false),
    ));
    assert!(with_icon.contains("<svg"));
    let svg_pos = with_icon.find("<svg").unwrap();
    let label_pos = with_icon.find("fw-wire-button-label").unwrap();
    assert!(svg_pos < label_pos, "expected <svg before label part");

    let without_icon = render(&button(
        "追加",
        None,
        Size::Md,
        Primary(false),
        Disabled(false),
    ));
    assert!(!without_icon.contains("<svg"));
}

#[test]
fn label_is_wrapped_in_label_part() {
    let html = render(&button(
        "ラベル",
        None,
        Size::Md,
        Primary(false),
        Disabled(false),
    ));
    assert!(html.contains(r#"class="fw-wire-button-label""#));
    assert!(html.contains("ラベル"));
}

#[test]
fn xss_regression_label_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&button(
        payload,
        None,
        Size::Md,
        Primary(false),
        Disabled(false),
    ));

    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn xss_regression_label_with_quote_payload_is_escaped() {
    let payload = "\"><img src=x onerror=alert(1)>";
    let html = render(&button(
        payload,
        None,
        Size::Md,
        Primary(false),
        Disabled(false),
    ));

    assert!(!html.contains(payload));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style() {
    let html = render(&button(
        "t",
        Some(icon::plus(Size::Md)),
        Size::Md,
        Primary(false),
        Disabled(true),
    ));
    // `aria-hidden="true"` はアイコン基盤（`crate::icon`）が装飾用途として
    // 付与する既定の属性であり、対話的 ARIA（`role`/`aria-expanded` 等）
    // ではないため許容する。それ以外の `aria-` 属性は一切出力しない。
    for forbidden in [
        "<button",
        " role=\"",
        " tabindex=\"",
        " style=\"",
        "<a ",
        "href=",
        "javascript:",
        " onclick=\"",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
    let aria_attrs: Vec<&str> = html
        .split(' ')
        .filter(|token| token.starts_with("aria-"))
        .collect();
    for attr in aria_attrs {
        assert!(
            attr.starts_with("aria-hidden="),
            "unexpected non-decorative aria attribute {attr:?} in {html:?}"
        );
    }
}

#[test]
fn button_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::button::BUTTON_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::button::BUTTON_CSS));
}

#[test]
fn button_css_declares_expected_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::button::BUTTON_CSS;
    for selector in [
        ".fw-wire-button {",
        ".fw-wire-button-label {",
        ".fw-wire-button.fw-wire-primary {",
        ".fw-wire-button[data-disabled] {",
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
