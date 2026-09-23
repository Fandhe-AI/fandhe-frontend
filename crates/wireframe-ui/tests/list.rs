//! `list` 部品の契約テスト（イシュー #2657）。
//!
//! `crates/wireframe-ui/tests/avatar.rs`・`crates/wireframe-ui/tests/breadcrumbs.rs`
//! と同型の観点（非対話制約・XSS 回帰・CSS 配線）に加え、番号付きモード
//! のカウンタがそのモード専用セレクタへ正しくスコープされていることを
//! 固定する。
use fandhe_frontend_core::{render, text};
use fandhe_frontend_wireframe_ui::{list, wireframe_css, PARTS};

#[test]
fn unordered_renders_root_class_without_ordered_modifier() {
    let node = list(vec![text("A"), text("B")], false);
    let html = render(&node);
    assert!(html.contains(r#"class="fw-wire-list""#));
    assert!(!html.contains("fw-wire-list-ordered"));
    assert!(html.starts_with("<div"));
    assert!(html.trim_end().ends_with("</div>"));
}

#[test]
fn ordered_appends_ordered_modifier() {
    let node = list(vec![text("A"), text("B")], true);
    let html = render(&node);
    assert!(html.contains(r#"class="fw-wire-list fw-wire-list-ordered""#));
}

#[test]
fn renders_one_item_wrapper_per_item_in_order() {
    let node = list(vec![text("A"), text("B"), text("C")], false);
    let html = render(&node);
    assert_eq!(html.matches(r#"class="fw-wire-list-item""#).count(), 3);

    let idx_a = html.find('A').unwrap();
    let idx_b = html.find('B').unwrap();
    let idx_c = html.find('C').unwrap();
    assert!(idx_a < idx_b);
    assert!(idx_b < idx_c);
}

#[test]
fn empty_items_renders_root_only() {
    let unordered = render(&list(vec![], false));
    assert!(!unordered.contains("fw-wire-list-item"));
    assert!(unordered.contains(r#"class="fw-wire-list""#));

    let ordered = render(&list(vec![], true));
    assert!(!ordered.contains("fw-wire-list-item"));
    assert!(ordered.contains(r#"class="fw-wire-list fw-wire-list-ordered""#));
}

#[test]
fn xss_regression_item_text_is_escaped() {
    let payload_a = "<script>alert(1)</script>";
    let payload_b = "\"><img src=x onerror=alert(1)>";
    let html_a = render(&list(vec![text(payload_a)], false));
    let html_b = render(&list(vec![text(payload_b)], false));

    assert!(!html_a.contains(payload_a));
    assert!(html_a.contains("&lt;script&gt;"));
    assert!(!html_b.contains(payload_b));
    assert!(html_b.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics() {
    let html = render(&list(vec![text("A"), text("B")], true));
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
        "<ul",
        "<ol",
        "<li",
        "javascript:",
        " onclick=\"",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn no_data_attributes() {
    let html = render(&list(vec![text("A")], true));
    assert!(!html.contains("data-"));
}

#[test]
fn list_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::list::LIST_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::list::LIST_CSS));
}

#[test]
fn list_css_selectors_use_fw_wire_prefix_and_no_pre_styled_ui_prefix() {
    let css = fandhe_frontend_wireframe_ui::list::LIST_CSS;
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
}

#[test]
fn counter_rules_are_scoped_to_ordered_modifier() {
    let css = fandhe_frontend_wireframe_ui::list::LIST_CSS;

    // `.fw-wire-list { ... }` 形式のルールをセレクタ単位に分割し、
    // `counter(`/`counter-increment` を含むルールのセレクタが必ず
    // `fw-wire-list-ordered` を含むことを確認する。
    let mut remaining = css;
    while let Some(open) = remaining.find('{') {
        let selector = remaining[..open].trim();
        let close = remaining[open..]
            .find('}')
            .map(|end| open + end)
            .expect("rule must be closed");
        let body = &remaining[open + 1..close];

        if body.contains("counter(") || body.contains("counter-increment") {
            assert!(
                selector.contains("fw-wire-list-ordered"),
                "counter rule must be scoped to fw-wire-list-ordered: {selector:?}"
            );
        }

        remaining = &remaining[close + 1..];
    }
}
