//! `stat` 部品の契約テスト（イシュー #2656）。
//!
//! `crates/wireframe-ui/tests/alert.rs` と同型の観点（非対話制約・XSS
//! 回帰・CSS 配線）に加え、`stat` 固有の増減インジケータ切り替え（3 段）・
//! delta 省略時のパート省略・向きごとのグリフ有無を単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::stat::{StatDelta, StatTrend};
use fandhe_frontend_wireframe_ui::{stat, wireframe_css, Size, PARTS};

const STAT_CSS: &str = fandhe_frontend_wireframe_ui::stat::STAT_CSS;

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let html = render(&stat("ラベル", "42", None, size));
        let expected_class = format!(r#"class="fw-wire-stat {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn trend_default_is_flat_and_all_is_declaration_order() {
    assert_eq!(StatTrend::default(), StatTrend::Flat);
    assert_eq!(
        StatTrend::ALL,
        [StatTrend::Up, StatTrend::Down, StatTrend::Flat]
    );
}

#[test]
fn as_str_matches_expected_literals() {
    assert_eq!(StatTrend::Up.as_str(), "up");
    assert_eq!(StatTrend::Down.as_str(), "down");
    assert_eq!(StatTrend::Flat.as_str(), "flat");
}

#[test]
fn each_trend_adds_exactly_its_own_modifier_class() {
    for trend in StatTrend::ALL {
        let html = render(&stat(
            "ラベル",
            "42",
            Some(StatDelta::new("+1", trend)),
            Size::Md,
        ));
        for candidate in StatTrend::ALL {
            let occurrences = html.matches(candidate.class()).count();
            if candidate == trend {
                assert_eq!(occurrences, 1, "trend={trend:?} html={html:?}");
            } else {
                assert_eq!(occurrences, 0, "trend={trend:?} html={html:?}");
            }
        }
    }
}

#[test]
fn up_and_down_render_svg_glyph_flat_does_not() {
    for (trend, expect_svg) in [
        (StatTrend::Up, true),
        (StatTrend::Down, true),
        (StatTrend::Flat, false),
    ] {
        let html = render(&stat(
            "ラベル",
            "42",
            Some(StatDelta::new("1", trend)),
            Size::Md,
        ));
        assert_eq!(
            html.contains("<svg"),
            expect_svg,
            "trend={trend:?} html={html:?}"
        );
        if expect_svg {
            assert!(html.contains(r#"class="fw-wire-stat-delta-icon""#));
        } else {
            assert!(!html.contains("fw-wire-stat-delta-icon"));
        }
    }
}

#[test]
fn up_uses_caret_up_and_down_uses_caret_down() {
    let up_html = render(&stat(
        "ラベル",
        "42",
        Some(StatDelta::new("1", StatTrend::Up)),
        Size::Md,
    ));
    assert!(up_html.contains(r#"data-icon="caret-up""#));

    let down_html = render(&stat(
        "ラベル",
        "42",
        Some(StatDelta::new("1", StatTrend::Down)),
        Size::Md,
    ));
    assert!(down_html.contains(r#"data-icon="caret-down""#));
}

#[test]
fn delta_none_omits_delta_part() {
    let html = render(&stat("ラベル", "42", None, Size::Md));
    assert!(!html.contains("fw-wire-stat-delta"));
    assert!(!html.contains("<svg"));
}

#[test]
fn label_and_value_are_always_rendered() {
    let html = render(&stat("売上", "¥1,234,567", None, Size::Md));
    assert!(html.contains(r#"class="fw-wire-stat-label""#));
    assert!(html.contains("売上"));
    assert!(html.contains(r#"class="fw-wire-stat-value""#));
    assert!(html.contains("¥1,234,567"));
}

#[test]
fn delta_value_is_rendered() {
    let html = render(&stat(
        "売上",
        "¥1,234,567",
        Some(StatDelta::new("+12%", StatTrend::Up)),
        Size::Md,
    ));
    assert!(html.contains(r#"class="fw-wire-stat-delta-value""#));
    assert!(html.contains("+12%"));
}

#[test]
fn xss_regression_label_value_and_delta_value_are_escaped() {
    let label_payload = "<script>alert(1)</script>";
    let value_payload = "\"><img src=x onerror=alert(1)>";
    let delta_payload = "<script>alert(2)</script>";
    let html = render(&stat(
        label_payload,
        value_payload,
        Some(StatDelta::new(delta_payload, StatTrend::Up)),
        Size::Md,
    ));
    assert!(!html.contains(label_payload));
    assert!(!html.contains(value_payload));
    assert!(!html.contains(delta_payload));
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_without_delta() {
    let html = render(&stat("ラベル", "42", None, Size::Md));
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
        "data-",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn output_has_no_role_or_aria_live_even_with_delta_icon() {
    // アイコンスロット基盤（icon.rs）自身は装飾用途として aria-hidden・
    // data-icon を付与するため、role/aria-live の不在のみを確認する
    // （`icon.rs` モジュール doc「出力契約」節参照）。
    let html = render(&stat(
        "ラベル",
        "42",
        Some(StatDelta::new("+1", StatTrend::Up)),
        Size::Md,
    ));
    assert!(!html.contains(" role=\""));
    assert!(!html.contains("aria-live"));
    for forbidden in [
        " tabindex=\"",
        " style=\"",
        "<button",
        "<a ",
        "href=",
        "<input",
        "<select",
        "javascript:",
        " onclick=\"",
    ] {
        assert!(!html.contains(forbidden), "unexpected {forbidden:?}");
    }
}

#[test]
fn stat_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS.iter().filter(|part| **part == STAT_CSS).count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(STAT_CSS));
}

#[test]
fn stat_css_declares_the_expected_selectors_with_fw_wire_prefix_only() {
    for selector in [
        ".fw-wire-stat {",
        ".fw-wire-stat-label",
        ".fw-wire-stat-value",
        ".fw-wire-stat-delta {",
        ".fw-wire-stat-delta-icon",
        ".fw-wire-stat-up",
    ] {
        assert!(STAT_CSS.contains(selector), "missing selector {selector:?}");
    }

    for line in STAT_CSS.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('.') {
            assert!(
                trimmed.starts_with(".fw-wire-"),
                "selector line should start with .fw-wire-: {line:?}"
            );
        }
    }

    assert!(!STAT_CSS.contains("--fandhe-"));
    assert!(!STAT_CSS.contains(" fd-"));
}
