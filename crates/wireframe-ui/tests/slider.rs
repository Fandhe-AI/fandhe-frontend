//! `slider` 部品の契約テスト（イシュー #2628）。
//!
//! `crates/wireframe-ui/tests/grid.rs`・`select.rs` と同型の観点（root
//! class・非対話制約・CSS 配線）に加え、`slider` 固有の進捗値の丸め・
//! 量子化（0〜255 → 0〜100 へクランプ → 5 刻みへ量子化）と
//! track/fill/thumb の anatomy を固定する。
//!
//! `slider` は `&str` 引数を持たない（進捗は `u8` のみ）ため、text 引数の
//! XSS 回帰テストは構造的に充足される（`crates/wireframe-ui/src/slider.rs`
//! モジュール doc・`site/wireframes/slider.md` の「原案差分メモ」節も
//! 参照）。label 引数を発明してテストを追加することはしない。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{
    slider, wireframe_css, Active, Disabled, Orientation, Size, PARTS,
};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = slider(
            40,
            Orientation::Horizontal,
            size,
            Active(false),
            Disabled(false),
        );
        let html = render(&node);
        let expected_class = format!(
            r#"class="fw-wire-slider fw-wire-horizontal {} fw-wire-slider-value-40""#,
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
        let html = render(&slider(
            input,
            Orientation::Horizontal,
            Size::Md,
            Active(false),
            Disabled(false),
        ));
        let expected_class = format!(
            r#"class="fw-wire-slider fw-wire-horizontal fw-wire-size-md fw-wire-slider-value-{expected_q}""#
        );
        assert!(
            html.contains(&expected_class),
            "input {input}: expected {expected_class:?} in {html:?}"
        );
    }
}

#[test]
fn full_render_matches_for_representative_value() {
    let html = render(&slider(
        40,
        Orientation::Horizontal,
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    assert_eq!(
        html,
        concat!(
            r#"<div class="fw-wire-slider fw-wire-horizontal fw-wire-size-md fw-wire-slider-value-40">"#,
            r#"<div class="fw-wire-slider-track">"#,
            r#"<div class="fw-wire-slider-fill"></div>"#,
            r#"<div class="fw-wire-slider-thumb"></div>"#,
            "</div>",
            "</div>",
        )
    );
}

#[test]
fn vertical_orientation_switches_class() {
    let vertical = render(&slider(
        40,
        Orientation::Vertical,
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    assert!(vertical.contains("fw-wire-vertical"));
    assert!(!vertical.contains("fw-wire-horizontal"));

    let horizontal = render(&slider(
        40,
        Orientation::Horizontal,
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    assert!(horizontal.contains("fw-wire-horizontal"));
    assert!(!horizontal.contains("fw-wire-vertical"));
}

#[test]
fn active_and_disabled_emit_data_attributes_only_when_true() {
    let neither = render(&slider(
        40,
        Orientation::Horizontal,
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    assert!(!neither.contains("data-active"));
    assert!(!neither.contains("data-disabled"));

    let active_only = render(&slider(
        40,
        Orientation::Horizontal,
        Size::Md,
        Active(true),
        Disabled(false),
    ));
    assert!(active_only.contains(r#"data-active="""#));
    assert!(!active_only.contains("data-disabled"));

    let disabled_only = render(&slider(
        40,
        Orientation::Horizontal,
        Size::Md,
        Active(false),
        Disabled(true),
    ));
    assert!(!disabled_only.contains("data-active"));
    assert!(disabled_only.contains(r#"data-disabled="""#));

    let both = render(&slider(
        40,
        Orientation::Horizontal,
        Size::Md,
        Active(true),
        Disabled(true),
    ));
    assert!(both.contains(r#"data-active="""#));
    assert!(both.contains(r#"data-disabled="""#));
}

#[test]
fn anatomy_has_track_fill_thumb_in_order() {
    let html = render(&slider(
        40,
        Orientation::Horizontal,
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    let track_pos = html
        .find(r#"class="fw-wire-slider-track""#)
        .expect("track part should be present");
    let fill_pos = html
        .find(r#"class="fw-wire-slider-fill""#)
        .expect("fill part should be present");
    let thumb_pos = html
        .find(r#"class="fw-wire-slider-thumb""#)
        .expect("thumb part should be present");
    assert!(track_pos < fill_pos, "track should come before fill");
    assert!(fill_pos < thumb_pos, "fill should come before thumb");
}

#[test]
fn output_has_no_interactive_semantics_or_style_or_dynamic_attributes() {
    let html = render(&slider(
        40,
        Orientation::Vertical,
        Size::Md,
        Active(true),
        Disabled(true),
    ));
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
        "<input",
        "data-value",
        "%",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn slider_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::slider::SLIDER_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::slider::SLIDER_CSS));
}

#[test]
fn slider_css_declares_expected_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::slider::SLIDER_CSS;
    for selector in [
        ".fw-wire-slider {",
        ".fw-wire-slider-track {",
        ".fw-wire-slider-fill {",
        ".fw-wire-slider-thumb {",
        ".fw-wire-slider.fw-wire-vertical {",
        ".fw-wire-slider[data-active] .fw-wire-slider-thumb {",
        ".fw-wire-slider[data-disabled] {",
    ] {
        assert!(css.contains(selector), "missing selector {selector:?}");
    }

    for q in (0..=100).step_by(5) {
        let selector = format!(".fw-wire-slider-value-{q} {{");
        assert!(css.contains(&selector), "missing value class {selector:?}");
    }
    assert_eq!(
        css.matches("--fw-wire-slider-value:").count(),
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
