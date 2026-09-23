//! `progress` 部品の契約テスト（イシュー #2648）。
//!
//! `crates/wireframe-ui/tests/slider.rs` と同型の観点（root class・非対話
//! 制約・CSS 配線）に加え、`progress` 固有の進捗値の丸め・量子化
//! （0〜255 → 0〜100 へクランプ → 5 刻みへ量子化）と Bar/Circle の
//! anatomy を固定する。
//!
//! `progress` は `&str` 引数を持たない（進捗は `u8` のみ、形状は
//! `ProgressShape` 列挙型）ため、text 引数の XSS 回帰テストは構造的に
//! 充足される（`crates/wireframe-ui/src/progress.rs` モジュール doc・
//! `site/wireframes/progress.md` の「原案差分メモ」節も参照）。label 引数を
//! 発明してテストを追加することはしない。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{progress, wireframe_css, ProgressShape, Size, PARTS};

#[test]
fn renders_root_class_for_every_size_and_shape() {
    for size in Size::ALL {
        for shape in [ProgressShape::Bar, ProgressShape::Circle] {
            let node = progress(40, shape, size);
            let html = render(&node);
            let expected_class = format!(
                r#"class="fw-wire-progress {} {} fw-wire-progress-value-40""#,
                shape.class(),
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
        let html = render(&progress(input, ProgressShape::Bar, Size::Md));
        let expected_class = format!(
            r#"class="fw-wire-progress fw-wire-progress-bar fw-wire-size-md fw-wire-progress-value-{expected_q}""#
        );
        assert!(
            html.contains(&expected_class),
            "input {input}: expected {expected_class:?} in {html:?}"
        );
    }
}

#[test]
fn full_render_matches_for_representative_bar_value() {
    let html = render(&progress(80, ProgressShape::Bar, Size::Md));
    assert_eq!(
        html,
        concat!(
            r#"<div class="fw-wire-progress fw-wire-progress-bar fw-wire-size-md fw-wire-progress-value-80">"#,
            r#"<div class="fw-wire-progress-track">"#,
            r#"<div class="fw-wire-progress-fill"></div>"#,
            "</div>",
            "</div>",
        )
    );
}

#[test]
fn full_render_matches_for_representative_circle_value() {
    let html = render(&progress(80, ProgressShape::Circle, Size::Md));
    assert_eq!(
        html,
        concat!(
            r#"<div class="fw-wire-progress fw-wire-progress-circle fw-wire-size-md fw-wire-progress-value-80">"#,
            r#"<div class="fw-wire-progress-track">"#,
            r#"<div class="fw-wire-progress-hole"></div>"#,
            "</div>",
            "</div>",
        )
    );
}

#[test]
fn shape_switches_class_and_anatomy() {
    let bar = render(&progress(80, ProgressShape::Bar, Size::Md));
    assert!(bar.contains("fw-wire-progress-bar"));
    assert!(!bar.contains("fw-wire-progress-circle"));
    assert!(bar.contains("fw-wire-progress-fill"));
    assert!(!bar.contains("fw-wire-progress-hole"));

    let circle = render(&progress(80, ProgressShape::Circle, Size::Md));
    assert!(circle.contains("fw-wire-progress-circle"));
    assert!(!circle.contains("fw-wire-progress-bar"));
    assert!(circle.contains("fw-wire-progress-hole"));
    assert!(!circle.contains("fw-wire-progress-fill"));
}

#[test]
fn output_has_no_interactive_semantics_or_style_or_dynamic_attributes() {
    for shape in [ProgressShape::Bar, ProgressShape::Circle] {
        let html = render(&progress(80, shape, Size::Md));
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
            "<progress",
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
fn progress_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::progress::PROGRESS_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::progress::PROGRESS_CSS));
}

#[test]
fn progress_css_declares_expected_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::progress::PROGRESS_CSS;
    for selector in [
        ".fw-wire-progress {",
        ".fw-wire-progress-track {",
        ".fw-wire-progress-fill {",
        ".fw-wire-progress-hole {",
        ".fw-wire-progress.fw-wire-progress-circle {",
        ".fw-wire-progress.fw-wire-progress-circle .fw-wire-progress-track {",
    ] {
        assert!(css.contains(selector), "missing selector {selector:?}");
    }

    for q in (0..=100).step_by(5) {
        let selector = format!(".fw-wire-progress-value-{q} {{");
        assert!(css.contains(&selector), "missing value class {selector:?}");
    }
    assert_eq!(
        css.matches("--fw-wire-progress-value:").count(),
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
