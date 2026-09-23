//! `checkbox` 部品の契約テスト（イシュー #2625）。
//!
//! `crates/wireframe-ui/tests/button.rs` と同型の観点（非対話制約・XSS
//! 回帰・CSS 配線）に加え、`active`（`Active`、`data-active` + チェック
//! グリフの条件付き出力）と `label`（`Option<&str>`、ラベルパート要素の
//! 条件付き出力）を wireframe-ui 側で単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{checkbox, wireframe_css, Active, Disabled, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = checkbox(Some("同意する"), size, Active(false), Disabled(false));
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-checkbox {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn active_true_emits_data_active_and_check_glyph_and_false_omits_both() {
    let active = render(&checkbox(
        Some("t"),
        Size::Md,
        Active(true),
        Disabled(false),
    ));
    assert!(active.contains(r#"data-active="""#));
    assert!(active.contains("<svg"));
    assert!(active.contains(r#"data-icon="check""#));

    let inactive = render(&checkbox(
        Some("t"),
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    assert!(!inactive.contains("data-active"));
    assert!(!inactive.contains("<svg"));
}

#[test]
fn disabled_true_emits_data_disabled_and_false_omits_all_data_attributes() {
    let with_disabled = render(&checkbox(
        Some("t"),
        Size::Md,
        Active(false),
        Disabled(true),
    ));
    assert!(with_disabled.contains(r#"data-disabled="""#));

    let without_disabled = render(&checkbox(
        Some("t"),
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    assert!(!without_disabled.contains("data-"));
}

#[test]
fn box_part_is_always_present_and_precedes_label() {
    let html = render(&checkbox(
        Some("ラベル"),
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    assert!(html.contains(r#"class="fw-wire-checkbox-box""#));
    let box_pos = html.find("fw-wire-checkbox-box").unwrap();
    let label_pos = html.find("fw-wire-checkbox-label").unwrap();
    assert!(box_pos < label_pos, "expected box part before label part");
}

#[test]
fn label_none_omits_label_part_but_keeps_box_part() {
    let html = render(&checkbox(None, Size::Md, Active(false), Disabled(false)));
    assert!(!html.contains("fw-wire-checkbox-label"));
    assert!(html.contains("fw-wire-checkbox-box"));
}

#[test]
fn label_some_is_wrapped_in_label_part() {
    let html = render(&checkbox(
        Some("利用規約に同意する"),
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    assert!(html.contains(r#"class="fw-wire-checkbox-label""#));
    assert!(html.contains("利用規約に同意する"));
}

#[test]
fn xss_regression_label_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&checkbox(
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
    let html = render(&checkbox(
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
    let html = render(&checkbox(Some("t"), Size::Md, Active(true), Disabled(true)));
    // `aria-hidden="true"` はアイコン基盤（`crate::icon`）が装飾用途として
    // 付与する既定の属性であり、対話的 ARIA（`role`/`aria-checked` 等）
    // ではないため許容する。それ以外の `aria-` 属性は一切出力しない。
    for forbidden in [
        "<input",
        "<button",
        " role=\"",
        "aria-checked",
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
fn checkbox_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::checkbox::CHECKBOX_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::checkbox::CHECKBOX_CSS));
}

#[test]
fn checkbox_css_declares_expected_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::checkbox::CHECKBOX_CSS;
    for selector in [
        ".fw-wire-checkbox {",
        ".fw-wire-checkbox-box {",
        ".fw-wire-checkbox-label {",
        ".fw-wire-checkbox[data-active] .fw-wire-checkbox-box {",
        ".fw-wire-checkbox[data-disabled] {",
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
