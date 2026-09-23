//! `alert` 部品の契約テスト（イシュー #2646）。
//!
//! `crates/wireframe-ui/tests/tooltip.rs`・`crates/wireframe-ui/tests/file_drop.rs`
//! と同型の観点（非対話制約・XSS 回帰・CSS 配線）に加え、`alert` 固有の
//! 重要度切り替え（3 段）・description/icon 省略時のパート省略を単体
//!固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::alert::Severity;
use fandhe_frontend_wireframe_ui::{alert, icon, wireframe_css, Size, PARTS};

const ALERT_CSS: &str = fandhe_frontend_wireframe_ui::alert::ALERT_CSS;

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let html = render(&alert(Severity::Info, "件名", None, None, size));
        let expected_class = format!(
            r#"class="fw-wire-alert fw-wire-alert-info {}""#,
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
fn severity_default_is_info_and_all_is_declaration_order() {
    assert_eq!(Severity::default(), Severity::Info);
    assert_eq!(
        Severity::ALL,
        [Severity::Info, Severity::Warning, Severity::Error]
    );
}

#[test]
fn as_str_matches_expected_literals() {
    assert_eq!(Severity::Info.as_str(), "info");
    assert_eq!(Severity::Warning.as_str(), "warning");
    assert_eq!(Severity::Error.as_str(), "error");
}

#[test]
fn each_severity_adds_exactly_its_own_modifier_class() {
    for severity in Severity::ALL {
        let html = render(&alert(severity, "件名", None, None, Size::Md));
        for candidate in Severity::ALL {
            let occurrences = html.matches(candidate.class()).count();
            if candidate == severity {
                assert_eq!(occurrences, 1, "severity={severity:?} html={html:?}");
            } else {
                assert_eq!(occurrences, 0, "severity={severity:?} html={html:?}");
            }
        }
    }
}

#[test]
fn title_and_description_are_rendered() {
    let html = render(&alert(
        Severity::Warning,
        "ストレージ容量が残りわずかです",
        Some("空き容量が 10% を下回りました。"),
        None,
        Size::Md,
    ));
    assert!(html.contains(r#"class="fw-wire-alert-title""#));
    assert!(html.contains("ストレージ容量が残りわずかです"));
    assert!(html.contains(r#"class="fw-wire-alert-description""#));
    assert!(html.contains("空き容量が 10% を下回りました。"));
}

#[test]
fn description_none_omits_description_part() {
    let html = render(&alert(Severity::Info, "件名のみ", None, None, Size::Md));
    assert!(!html.contains("fw-wire-alert-description"));
}

#[test]
fn icon_none_omits_icon_part_and_svg() {
    let html = render(&alert(Severity::Info, "件名", None, None, Size::Md));
    assert!(!html.contains("fw-wire-alert-icon"));
    assert!(!html.contains("<svg"));
}

#[test]
fn icon_some_renders_icon_part_with_data_icon() {
    let html = render(&alert(
        Severity::Info,
        "件名",
        None,
        Some(icon::bell(Size::Md)),
        Size::Md,
    ));
    assert!(html.contains(r#"class="fw-wire-alert-icon""#));
    assert!(html.contains("<svg"));
    assert!(html.contains(r#"data-icon="bell""#));
}

#[test]
fn xss_regression_title_and_description_are_escaped() {
    let payload = "<script>alert(1)</script>";
    let payload2 = "\"><img src=x onerror=alert(1)>";
    let html = render(&alert(
        Severity::Error,
        payload,
        Some(payload2),
        None,
        Size::Md,
    ));
    assert!(!html.contains(payload));
    assert!(!html.contains(payload2));
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_without_icon_slot() {
    let html = render(&alert(
        Severity::Warning,
        "件名",
        Some("説明"),
        None,
        Size::Md,
    ));
    for forbidden in [
        " role=\"",
        " aria-",
        " tabindex=\"",
        " style=\"",
        "<button",
        "<a ",
        "href=",
        "<input",
        "<select",
        "javascript:",
        " onclick=\"",
        " onload=\"",
        "aria-live",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn output_has_no_role_or_aria_live_even_with_icon_slot() {
    // アイコンスロット基盤（icon.rs）自身は装飾用途として aria-hidden を
    // 付与するため、role/aria-live の不在のみを確認する
    // （`icon.rs` モジュール doc「出力契約」節参照）。
    let html = render(&alert(
        Severity::Error,
        "件名",
        Some("説明"),
        Some(icon::bell(Size::Md)),
        Size::Md,
    ));
    assert!(!html.contains(" role=\""));
    assert!(!html.contains("aria-live"));
}

#[test]
fn no_data_attributes_without_icon_slot() {
    let html = render(&alert(Severity::Info, "件名", Some("説明"), None, Size::Md));
    assert!(!html.contains("data-"));
}

#[test]
fn alert_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS.iter().filter(|part| **part == ALERT_CSS).count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(ALERT_CSS));
}

#[test]
fn alert_css_declares_the_expected_selectors_with_fw_wire_prefix_only() {
    for selector in [
        ".fw-wire-alert {",
        ".fw-wire-alert-icon",
        ".fw-wire-alert-body",
        ".fw-wire-alert-title",
        ".fw-wire-alert-description",
        ".fw-wire-alert-warning",
        ".fw-wire-alert-error",
    ] {
        assert!(
            ALERT_CSS.contains(selector),
            "missing selector {selector:?}"
        );
    }

    for line in ALERT_CSS.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('.') {
            assert!(
                trimmed.starts_with(".fw-wire-"),
                "selector line should start with .fw-wire-: {line:?}"
            );
        }
    }

    assert!(!ALERT_CSS.contains("--fandhe-"));
    assert!(!ALERT_CSS.contains(" fd-"));
}
