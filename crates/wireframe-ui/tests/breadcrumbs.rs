//! `breadcrumbs` 部品の契約テスト（イシュー #2639）。
//!
//! `crates/wireframe-ui/tests/tabs.rs`（選択状態と非対話制約の先例）と
//! 同型の観点（XSS 回帰・CSS 配線・非対話制約）に加え、`items` スライスの
//! 項目数可変性、空スライスでの panic なし、現在階層（`data-active`）が
//! 常に最後の項目に固定される不変条件、区切り記号が DOM テキストとして
//! 出力されないことを単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{breadcrumbs, wireframe_css, Size, PARTS};

const ITEMS: [&str; 3] = ["ホーム", "商品", "詳細"];

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = breadcrumbs(&ITEMS, size);
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-breadcrumbs {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn item_count_matches_items_len() {
    let html = render(&breadcrumbs(&ITEMS, Size::Md));
    assert_eq!(
        html.matches(r#"class="fw-wire-breadcrumbs-item""#).count(),
        ITEMS.len()
    );
}

#[test]
fn empty_items_does_not_panic_and_renders_no_items() {
    let html = render(&breadcrumbs(&[], Size::Md));
    assert!(!html.contains("fw-wire-breadcrumbs-item"));
    assert!(!html.contains("data-active"));
    assert!(html.contains(r#"class="fw-wire-breadcrumbs fw-wire-size-md""#));
}

#[test]
fn last_item_always_marked_active() {
    let html = render(&breadcrumbs(&ITEMS, Size::Md));
    assert_eq!(html.matches(r#"data-active="""#).count(), 1);

    // 現在階層（詳細）の直前にだけ data-active が付く。
    let idx_label = html.rfind("詳細").expect("label must be present");
    let idx_active = html
        .find(r#"data-active="""#)
        .expect("data-active must be present");
    assert!(idx_active < idx_label);
}

#[test]
fn single_item_is_marked_active() {
    let html = render(&breadcrumbs(&["ホーム"], Size::Md));
    assert_eq!(html.matches(r#"data-active="""#).count(), 1);
    assert!(html.contains("ホーム"));
}

#[test]
fn non_last_items_are_not_marked_active() {
    let html = render(&breadcrumbs(&ITEMS, Size::Md));
    let idx_active = html.find(r#"data-active="""#).unwrap();
    let idx_home = html.find("ホーム").unwrap();
    let idx_product = html.find("商品").unwrap();
    assert!(idx_home < idx_active);
    assert!(idx_product < idx_active);
}

#[test]
fn labels_preserve_order() {
    let html = render(&breadcrumbs(&ITEMS, Size::Md));
    let positions: Vec<usize> = ITEMS
        .iter()
        .map(|label| html.find(label).expect("label must be present"))
        .collect();
    assert!(positions.windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn separator_is_not_rendered_as_dom_text() {
    let html = render(&breadcrumbs(&ITEMS, Size::Md));
    assert!(!html.contains(">/<"));
    assert!(!html.contains("fw-wire-breadcrumbs-separator"));
}

#[test]
fn xss_regression_label_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&breadcrumbs(&[payload], Size::Md));
    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn xss_regression_label_with_quote_payload_is_escaped() {
    let payload = "\"><img src=x onerror=alert(1)>";
    let html = render(&breadcrumbs(&[payload], Size::Md));
    assert!(!html.contains(payload));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style() {
    let html = render(&breadcrumbs(&ITEMS, Size::Md));
    for forbidden in [
        "<input",
        "<label",
        "<button",
        "<select",
        "<nav",
        "<ol",
        "<li",
        " role=\"",
        " tabindex=\"",
        " style=\"",
        "<a ",
        "href=",
        "javascript:",
        " onclick=\"",
        " aria-",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn breadcrumbs_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::breadcrumbs::BREADCRUMBS_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::breadcrumbs::BREADCRUMBS_CSS));
}

#[test]
fn breadcrumbs_css_declares_expected_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::breadcrumbs::BREADCRUMBS_CSS;
    for selector in [
        ".fw-wire-breadcrumbs {",
        ".fw-wire-breadcrumbs-item {",
        ".fw-wire-breadcrumbs-item + .fw-wire-breadcrumbs-item {",
        ".fw-wire-breadcrumbs-item + .fw-wire-breadcrumbs-item::before {",
        ".fw-wire-breadcrumbs-item[data-active] {",
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
