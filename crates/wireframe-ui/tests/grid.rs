//! `grid` 部品の契約テスト（イシュー #2611）。
//!
//! `crates/wireframe-ui/tests/annotation.rs` と同型の観点（root class・
//! XSS 回帰・非対話制約・CSS 配線）に加え、`grid` 固有の列数丸め・
//! 子ノードの item 包み込みを固定する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_wireframe_ui::{grid, wireframe_css, Size, MAX_COLUMNS, PARTS};

#[test]
fn max_columns_is_twelve() {
    assert_eq!(MAX_COLUMNS, 12);
}

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = grid(vec![], 3, size);
        let html = render(&node);
        let expected_class = format!(
            r#"class="fw-wire-grid {} fw-wire-grid-cols-3""#,
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
fn columns_are_clamped_to_one_through_max() {
    let cases: [(u32, &str); 6] = [
        (0, "fw-wire-grid-cols-1"),
        (1, "fw-wire-grid-cols-1"),
        (7, "fw-wire-grid-cols-7"),
        (12, "fw-wire-grid-cols-12"),
        (13, "fw-wire-grid-cols-12"),
        (u32::MAX, "fw-wire-grid-cols-12"),
    ];
    for (columns, expected_class) in cases {
        let html = render(&grid(vec![], columns, Size::Md));
        assert!(
            html.contains(expected_class),
            "columns={columns}: expected {expected_class:?} in {html:?}"
        );
    }
}

#[test]
fn children_are_wrapped_in_grid_item_and_order_is_preserved() {
    let node = grid(vec![text("A"), text("B"), text("C")], 3, Size::Md);
    let html = render(&node);
    assert_eq!(html.matches(r#"class="fw-wire-grid-item""#).count(), 3);
    let pos_a = html.find('A').expect("A should be present");
    let pos_b = html.find('B').expect("B should be present");
    let pos_c = html.find('C').expect("C should be present");
    assert!(
        pos_a < pos_b && pos_b < pos_c,
        "child order should be preserved: {html:?}"
    );
}

#[test]
fn empty_children_renders_no_item() {
    let html = render(&grid(vec![], 4, Size::Md));
    assert!(!html.contains("fw-wire-grid-item"));
}

#[test]
fn xss_regression_child_text_is_escaped() {
    let payload_a = "<script>alert(1)</script>";
    let payload_b = "\"><img src=x onerror=alert(1)>";
    let html = render(&grid(vec![text(payload_a), text(payload_b)], 2, Size::Md));

    assert!(!html.contains(payload_a));
    assert!(!html.contains(payload_b));
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style_or_data_attributes() {
    let html = render(&grid(vec![text("cell")], 2, Size::Md));
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
        "data-",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn grid_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::grid::GRID_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::grid::GRID_CSS));
}

#[test]
fn grid_css_declares_the_nineteen_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::grid::GRID_CSS;

    let mut expected_selectors: Vec<String> = vec![
        ".fw-wire-grid {".to_string(),
        ".fw-wire-grid-item {".to_string(),
    ];
    for n in 1..=12 {
        expected_selectors.push(format!(".fw-wire-grid-cols-{n} {{"));
    }
    for size in ["xs", "sm", "md", "lg", "xl"] {
        expected_selectors.push(format!(".fw-wire-grid.fw-wire-size-{size} {{"));
    }
    assert_eq!(expected_selectors.len(), 19);

    for selector in &expected_selectors {
        assert!(
            css.contains(selector.as_str()),
            "missing selector {selector:?}"
        );
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
