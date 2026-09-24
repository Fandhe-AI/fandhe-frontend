//! `chart` 部品の契約テスト（イシュー #2663）。
//!
//! `crates/wireframe-ui/tests/progress.rs` と同型の観点（root class・
//! 非対話制約・CSS 配線）に加え、`chart` 固有の棒本数の資源有界化
//! （[`MAX_BARS`](fandhe_frontend_wireframe_ui::MAX_BARS)）・空スライスの
//! panic なし・Orientation 切り替えを固定する。
//!
//! `chart` は `&str` 引数を持たない（値は `u8` 列 `values: &[u8]` のみ、
//! 向きは `Orientation` 列挙型）ため、text 引数の XSS 回帰テストは構造的
//! に充足される（`crates/wireframe-ui/src/chart.rs` モジュール doc・
//! `site/wireframes/chart.md` の「原案差分メモ」節も参照）。label 引数を
//! 発明してテストを追加することはしない。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{chart, wireframe_css, Orientation, Size, PARTS};

#[test]
fn renders_root_class_for_every_size_and_orientation() {
    for size in Size::ALL {
        for orientation in [Orientation::Horizontal, Orientation::Vertical] {
            let node = chart(&[20, 40, 60], orientation, size);
            let html = render(&node);
            let expected_class = format!(
                r#"class="fw-wire-chart {} {}""#,
                orientation.class(),
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
}

#[test]
fn value_is_clamped_and_quantized_to_steps_of_five() {
    let cases: [(u8, u8); 15] = [
        (0, 0),
        (1, 0),
        (2, 0),
        (3, 5),
        (7, 5),
        (8, 10),
        (40, 40),
        (42, 40),
        (43, 45),
        (97, 95),
        (98, 100),
        (99, 100),
        (100, 100),
        (101, 100),
        (255, 100),
    ];
    for (input, expected_q) in cases {
        let html = render(&chart(&[input], Orientation::Vertical, Size::Md));
        let expected_class = format!("fw-wire-chart-bar fw-wire-chart-value-{expected_q}");
        assert!(
            html.contains(&expected_class),
            "input {input}: expected {expected_class:?} in {html:?}"
        );
    }
}

#[test]
fn bar_count_matches_values_and_saturates_at_max_bars() {
    for n in [0usize, 1, 6, 12, 13, 100] {
        let values = vec![10u8; n];
        let html = render(&chart(&values, Orientation::Vertical, Size::Md));
        let expected = n.min(fandhe_frontend_wireframe_ui::MAX_BARS);
        assert_eq!(
            html.matches("fw-wire-chart-bar").count(),
            expected,
            "n={n}: expected {expected} bars in {html:?}"
        );
    }
}

#[test]
fn empty_values_render_empty_plot_without_panic() {
    let html = render(&chart(&[], Orientation::Vertical, Size::Md));
    assert!(html.contains(r#"class="fw-wire-chart-plot""#));
    assert!(!html.contains("fw-wire-chart-bar"));
}

#[test]
fn full_render_matches_for_representative_vertical_input() {
    let html = render(&chart(&[20, 42, 80], Orientation::Vertical, Size::Md));
    assert_eq!(
        html,
        concat!(
            r#"<div class="fw-wire-chart fw-wire-vertical fw-wire-size-md">"#,
            r#"<div class="fw-wire-chart-plot">"#,
            r#"<div class="fw-wire-chart-bar fw-wire-chart-value-20"></div>"#,
            r#"<div class="fw-wire-chart-bar fw-wire-chart-value-40"></div>"#,
            r#"<div class="fw-wire-chart-bar fw-wire-chart-value-80"></div>"#,
            "</div>",
            "</div>",
        )
    );
}

#[test]
fn orientation_switches_class_only() {
    let vertical = render(&chart(&[50], Orientation::Vertical, Size::Md));
    assert!(vertical.contains("fw-wire-vertical"));
    assert!(!vertical.contains("fw-wire-horizontal"));

    let horizontal = render(&chart(&[50], Orientation::Horizontal, Size::Md));
    assert!(horizontal.contains("fw-wire-horizontal"));
    assert!(!horizontal.contains("fw-wire-vertical"));
}

#[test]
fn output_has_no_interactive_semantics_or_style_or_dynamic_attributes() {
    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        let html = render(&chart(&[10, 20, 30], orientation, Size::Md));
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
            "<svg",
            "<canvas",
            "data-value",
            "data-active",
            "data-disabled",
            "%",
        ] {
            assert!(
                !html.contains(forbidden),
                "unexpected {forbidden:?} in {html:?}"
            );
        }
    }
}

#[test]
fn chart_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::chart::CHART_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::chart::CHART_CSS));
}

#[test]
fn chart_css_declares_expected_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::chart::CHART_CSS;
    for selector in [
        ".fw-wire-chart {",
        ".fw-wire-chart-plot {",
        ".fw-wire-chart-bar {",
        ".fw-wire-chart.fw-wire-horizontal .fw-wire-chart-plot {",
        ".fw-wire-chart.fw-wire-horizontal .fw-wire-chart-bar {",
    ] {
        assert!(css.contains(selector), "missing selector {selector:?}");
    }

    for q in (0..=100).step_by(5) {
        let selector = format!(".fw-wire-chart-value-{q} {{");
        assert!(css.contains(&selector), "missing value class {selector:?}");
    }
    assert_eq!(
        css.matches("--fw-wire-chart-value:").count(),
        21,
        "expected exactly 21 value class declarations"
    );

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
