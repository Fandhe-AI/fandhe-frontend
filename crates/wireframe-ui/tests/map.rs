//! `map` 部品の契約テスト（イシュー #2664）。
//!
//! `crates/wireframe-ui/tests/chart.rs`・`crates/wireframe-ui/tests/alert.rs`
//! と同型の観点（root class・非対話制約・CSS 配線）に加え、`map` 固有の
//! ズーム 3 段の切り替え・marker スロットの有無を単体固定する。
//!
//! `map` は `&str` 引数を持たない（`zoom` は [`MapZoom`] 列挙型、`marker`
//! は構築済みの `Node`、`size` は [`Size`]）ため、text 引数の XSS 回帰
//! テストは構造的に充足される（`crates/wireframe-ui/src/map.rs` モジュール
//! doc・`site/wireframes/map.md` の「原案差分メモ」節も参照）。label 引数を
//! 発明してテストを追加することはしない。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::map::MapZoom;
use fandhe_frontend_wireframe_ui::{icon, map, wireframe_css, Size, PARTS};

#[test]
fn renders_root_class_for_every_size_and_zoom() {
    for size in Size::ALL {
        for zoom in MapZoom::ALL {
            let node = map(zoom, None, size);
            let html = render(&node);
            let expected_class =
                format!(r#"class="fw-wire-map {} {}""#, zoom.class(), size.class());
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
fn zoom_default_is_medium_and_all_is_declaration_order() {
    assert_eq!(MapZoom::default(), MapZoom::Medium);
    assert_eq!(MapZoom::ALL, [MapZoom::Far, MapZoom::Medium, MapZoom::Near]);
}

#[test]
fn each_zoom_adds_exactly_its_own_modifier_class() {
    for zoom in MapZoom::ALL {
        let html = render(&map(zoom, None, Size::Md));
        for candidate in MapZoom::ALL {
            let occurrences = html.matches(candidate.class()).count();
            if candidate == zoom {
                assert_eq!(occurrences, 1, "zoom={zoom:?} html={html:?}");
            } else {
                assert_eq!(occurrences, 0, "zoom={zoom:?} html={html:?}");
            }
        }
    }
}

#[test]
fn fixed_parts_are_always_present_in_document_order() {
    let html = render(&map(MapZoom::Medium, None, Size::Md));
    let tile = html.find(r#"class="fw-wire-map-tile""#).expect("tile");
    let area = html.find(r#"class="fw-wire-map-area""#).expect("area");
    let road_h = html.find(r#"class="fw-wire-map-road-h""#).expect("road-h");
    let road_v = html.find(r#"class="fw-wire-map-road-v""#).expect("road-v");
    assert!(tile < area, "tile should precede area");
    assert!(area < road_h, "area should precede road-h");
    assert!(road_h < road_v, "road-h should precede road-v");
}

#[test]
fn marker_none_omits_marker_part_and_svg() {
    let html = render(&map(MapZoom::Medium, None, Size::Md));
    assert!(!html.contains("fw-wire-map-marker"));
    assert!(!html.contains("<svg"));
}

#[test]
fn marker_some_renders_marker_part_with_slot_content() {
    let html = render(&map(MapZoom::Medium, Some(icon::house(Size::Md)), Size::Md));
    assert_eq!(html.matches(r#"class="fw-wire-map-marker""#).count(), 1);
    assert!(html.contains("<svg"));
    assert!(html.contains(r#"data-icon="house""#));

    let marker_pos = html.find("fw-wire-map-marker").unwrap();
    let road_v_pos = html.find("fw-wire-map-road-v").unwrap();
    assert!(
        road_v_pos < marker_pos,
        "marker should follow the fixed parts"
    );
}

#[test]
fn full_render_matches_for_representative_input_without_marker() {
    let html = render(&map(MapZoom::Near, None, Size::Md));
    assert_eq!(
        html,
        concat!(
            r#"<div class="fw-wire-map fw-wire-map-zoom-near fw-wire-size-md">"#,
            r#"<div class="fw-wire-map-tile"></div>"#,
            r#"<div class="fw-wire-map-area"></div>"#,
            r#"<div class="fw-wire-map-road-h"></div>"#,
            r#"<div class="fw-wire-map-road-v"></div>"#,
            "</div>",
        )
    );
}

#[test]
fn output_has_no_interactive_semantics_or_style_or_dynamic_attributes_without_marker() {
    for zoom in MapZoom::ALL {
        let html = render(&map(zoom, None, Size::Md));
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
            "<img",
            "<iframe",
            "<svg",
            "<canvas",
            "data-",
        ] {
            assert!(
                !html.contains(forbidden),
                "unexpected {forbidden:?} in {html:?}"
            );
        }
    }
}

#[test]
fn output_has_no_interactive_semantics_outside_marker_slot_when_marker_present() {
    let html = render(&map(MapZoom::Medium, Some(icon::house(Size::Md)), Size::Md));
    for forbidden in [
        " role=\"",
        " tabindex=\"",
        " style=\"",
        "<button",
        "<a ",
        "href=",
        "javascript:",
        " onclick=\"",
        " onload=\"",
        "<img",
        "<iframe",
        "<canvas",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
    // スロット由来の `aria-hidden`/`data-icon` は許容する（§11.4）。
    assert_eq!(html.matches(" aria-hidden").count(), 1);
    assert_eq!(html.matches("data-icon=").count(), 1);
}

#[test]
fn map_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::map::MAP_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::map::MAP_CSS));
}

#[test]
fn map_css_declares_expected_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::map::MAP_CSS;
    for selector in [
        ".fw-wire-map {",
        ".fw-wire-map-zoom-far {",
        ".fw-wire-map-zoom-medium {",
        ".fw-wire-map-zoom-near {",
        ".fw-wire-map-tile {",
        ".fw-wire-map-area {",
        ".fw-wire-map-road-h {",
        ".fw-wire-map-road-v {",
        ".fw-wire-map-marker {",
        ".fw-wire-map-marker .fw-wire-icon-glyph {",
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
    assert!(!css.contains("@keyframes"));
    assert!(!css.contains("animation"));
    assert!(!css.contains("url("));
}
