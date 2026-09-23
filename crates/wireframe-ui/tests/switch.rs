//! `switch` 部品の契約テスト（イシュー #2627）。
//!
//! `crates/wireframe-ui/tests/select.rs`・`annotation.rs` と同型の観点
//! （非対話制約・XSS 回帰・CSS 配線）に加え、`label`（`Option<&str>`）の
//! 条件付き出力と、常に出力されるトラック/つまみパートを wireframe-ui
//! 側で単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{switch, wireframe_css, Active, Disabled, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = switch(None, size, Active(false), Disabled(false));
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-switch {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn track_and_thumb_parts_are_always_present() {
    let with_label = render(&switch(
        Some("通知"),
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    assert!(with_label.contains(r#"class="fw-wire-switch-track""#));
    assert!(with_label.contains(r#"class="fw-wire-switch-thumb""#));

    let without_label = render(&switch(None, Size::Md, Active(false), Disabled(false)));
    assert!(without_label.contains(r#"class="fw-wire-switch-track""#));
    assert!(without_label.contains(r#"class="fw-wire-switch-thumb""#));
}

#[test]
fn label_some_renders_label_part_and_none_omits_it_entirely() {
    let with_label = render(&switch(
        Some("通知"),
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    assert!(with_label.contains(r#"class="fw-wire-switch-label""#));
    assert!(with_label.contains("通知"));

    let without_label = render(&switch(None, Size::Md, Active(false), Disabled(false)));
    assert!(!without_label.contains("fw-wire-switch-label"));
}

#[test]
fn active_true_emits_data_active_and_false_omits_it() {
    let with_active = render(&switch(None, Size::Md, Active(true), Disabled(false)));
    assert!(with_active.contains(r#"data-active="""#));

    let without_active = render(&switch(None, Size::Md, Active(false), Disabled(false)));
    assert!(!without_active.contains("data-active"));
}

#[test]
fn disabled_true_emits_data_disabled_and_false_omits_all_data_attributes() {
    let with_disabled = render(&switch(None, Size::Md, Active(false), Disabled(true)));
    assert!(with_disabled.contains(r#"data-disabled="""#));

    let without_disabled = render(&switch(None, Size::Md, Active(false), Disabled(false)));
    assert!(!without_disabled.contains("data-active"));
    assert!(!without_disabled.contains("data-disabled"));
}

#[test]
fn xss_regression_label_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&switch(
        Some(payload),
        Size::Md,
        Active(false),
        Disabled(false),
    ));

    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn xss_regression_label_with_quote_payload_is_escaped() {
    let payload = "\"><img src=x onerror=alert(1)>";
    let html = render(&switch(
        Some(payload),
        Size::Md,
        Active(false),
        Disabled(false),
    ));

    assert!(!html.contains(payload));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style() {
    let html = render(&switch(
        Some("通知"),
        Size::Md,
        Active(true),
        Disabled(true),
    ));
    for forbidden in [
        "<input",
        "<button",
        "<select",
        " role=\"",
        " aria-",
        " tabindex=\"",
        " style=\"",
        "<a ",
        "href=",
        "javascript:",
        " onclick=\"",
        " onload=\"",
        "type=\"checkbox\"",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn switch_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::switch::SWITCH_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::switch::SWITCH_CSS));
}

#[test]
fn switch_css_declares_the_eight_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::switch::SWITCH_CSS;
    for selector in [
        ".fw-wire-switch {",
        ".fw-wire-switch-track {",
        ".fw-wire-switch-thumb {",
        ".fw-wire-switch-label {",
        ".fw-wire-switch[data-active] .fw-wire-switch-track {",
        ".fw-wire-switch[data-active] .fw-wire-switch-thumb {",
        ".fw-wire-switch[data-disabled] {",
        ".fw-wire-switch[data-disabled] .fw-wire-switch-track {",
    ] {
        assert!(css.contains(selector), "missing selector {selector:?}");
    }

    let selector_count = css.matches('{').count();
    assert_eq!(selector_count, 8, "expected exactly 8 selectors in {css:?}");

    for line in css.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('.') {
            assert!(
                trimmed.starts_with(".fw-wire-"),
                "selector line must start with .fw-wire-: {trimmed:?}"
            );
        }
    }

    assert!(!css.contains("--fandhe-"));
    assert!(!css.contains(" fd-"));
}
