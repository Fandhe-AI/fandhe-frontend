//! `tooltip` 部品の契約テスト（イシュー #2644）。
//!
//! `crates/wireframe-ui/tests/tabs.rs`（非対話制約・XSS 回帰・CSS 配線の
//! 先例）と同型の観点に加え、4 方向それぞれで修飾 class がちょうど
//! 1 個だけ付くこと・本文と矢印パートの個数と DOM 順序（本文 → 矢印が
//! 方向によらず一定）を単体固定する。パート class をテスト側リテラルで
//! 固定する方針も `tabs.rs`（`fw-wire-tabs-item` を直接文字列で参照する）
//! に倣う。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::tooltip::TooltipSide;
use fandhe_frontend_wireframe_ui::{tooltip, wireframe_css, Size, PARTS};

/// 本文パートの class（[`fandhe_frontend_wireframe_ui::tooltip`] 内部の
/// `CONTENT_CLASS` はクレート外非公開のため、テスト側でリテラルとして
/// 固定する。`tabs.rs` テストが `fw-wire-tabs-item` を直接文字列で参照
/// するのと同じ方針）。
const CONTENT_CLASS: &str = "fw-wire-tooltip-content";
/// 矢印パートの class（同上の理由でリテラル固定）。
const ARROW_CLASS: &str = "fw-wire-tooltip-arrow";
const TOOLTIP_CSS: &str = fandhe_frontend_wireframe_ui::tooltip::TOOLTIP_CSS;

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let html = render(&tooltip("説明", TooltipSide::Top, size));
        let expected_class = format!(
            r#"class="fw-wire-tooltip {} fw-wire-tooltip-side-top""#,
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
fn side_default_is_top_and_all_is_declaration_order() {
    assert_eq!(TooltipSide::default(), TooltipSide::Top);
    assert_eq!(
        TooltipSide::ALL,
        [
            TooltipSide::Top,
            TooltipSide::Right,
            TooltipSide::Bottom,
            TooltipSide::Left,
        ]
    );
}

#[test]
fn each_side_adds_exactly_its_own_modifier_class() {
    for side in TooltipSide::ALL {
        let html = render(&tooltip("説明", side, Size::Md));
        for candidate in TooltipSide::ALL {
            let occurrences = html.matches(candidate.class()).count();
            if candidate == side {
                assert_eq!(occurrences, 1, "{side:?} should add its own class once");
            } else {
                assert_eq!(
                    occurrences, 0,
                    "{side:?} should not add {candidate:?}'s class"
                );
            }
        }
    }
}

#[test]
fn content_and_arrow_parts_appear_exactly_once_each_in_content_then_arrow_order() {
    for side in TooltipSide::ALL {
        let html = render(&tooltip("説明", side, Size::Md));
        assert_eq!(
            html.matches(&format!(r#"class="{CONTENT_CLASS}""#)).count(),
            1
        );
        assert_eq!(
            html.matches(&format!(r#"class="{ARROW_CLASS}""#)).count(),
            1
        );

        let content_idx = html.find(CONTENT_CLASS).expect("content part must exist");
        let arrow_idx = html.find(ARROW_CLASS).expect("arrow part must exist");
        assert!(
            content_idx < arrow_idx,
            "content must precede arrow regardless of side ({side:?})"
        );
    }
}

#[test]
fn empty_label_does_not_panic() {
    let html = render(&tooltip("", TooltipSide::Top, Size::Md));
    assert!(html.contains(&format!(r#"class="{CONTENT_CLASS}""#)));
}

#[test]
fn label_is_rendered() {
    let html = render(&tooltip("補足説明", TooltipSide::Top, Size::Md));
    assert!(html.contains("補足説明"));
}

#[test]
fn xss_regression_label_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&tooltip(payload, TooltipSide::Top, Size::Md));
    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn xss_regression_label_with_quote_payload_is_escaped() {
    let payload = "\"><img src=x onerror=alert(1)>";
    let html = render(&tooltip(payload, TooltipSide::Top, Size::Md));
    assert!(!html.contains(payload));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style() {
    let html = render(&tooltip("説明", TooltipSide::Top, Size::Md));
    for forbidden in [
        "<input",
        "<label",
        "<button",
        "<select",
        " role=\"",
        " tabindex=\"",
        " style=\"",
        "<a ",
        "href=",
        "javascript:",
        " onclick=\"",
        " aria-",
        " title=\"",
        "hidden",
        "data-state",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn tooltip_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS.iter().filter(|part| **part == TOOLTIP_CSS).count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(TOOLTIP_CSS));
}

#[test]
fn tooltip_css_declares_expected_selectors_with_fw_wire_prefix_only() {
    let css = TOOLTIP_CSS;
    for selector in [
        ".fw-wire-tooltip {",
        ".fw-wire-tooltip-content {",
        ".fw-wire-tooltip-arrow {",
        ".fw-wire-tooltip.fw-wire-tooltip-side-top {",
        ".fw-wire-tooltip.fw-wire-tooltip-side-top .fw-wire-tooltip-arrow {",
        ".fw-wire-tooltip.fw-wire-tooltip-side-bottom {",
        ".fw-wire-tooltip.fw-wire-tooltip-side-bottom .fw-wire-tooltip-arrow {",
        ".fw-wire-tooltip.fw-wire-tooltip-side-left {",
        ".fw-wire-tooltip.fw-wire-tooltip-side-left .fw-wire-tooltip-arrow {",
        ".fw-wire-tooltip.fw-wire-tooltip-side-right {",
        ".fw-wire-tooltip.fw-wire-tooltip-side-right .fw-wire-tooltip-arrow {",
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
    assert!(
        !css.contains('#'),
        "CSS should reference tokens, not hex literals"
    );
}

#[test]
fn renders_deterministically() {
    let a = render(&tooltip("説明", TooltipSide::Bottom, Size::Lg));
    let b = render(&tooltip("説明", TooltipSide::Bottom, Size::Lg));
    assert_eq!(a, b);
}
