//! `nav_item` 部品の契約テスト（イシュー #2636）。
//!
//! `crates/wireframe-ui/tests/input.rs`/`rich_text.rs` と同型の観点
//! （非対話制約・XSS 回帰・CSS 配線）に加え、`leading`/`trailing`
//! アイコンスロット（`Option<Node>`）・`counter`（`Option<&str>`）・
//! `Active` の `data-active` 出力・`Orientation` の縦積み class を
//! wireframe-ui 側で単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{icon, nav_item, Active, Orientation, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = nav_item(
            "ホーム",
            None,
            None,
            None,
            size,
            Active(false),
            Orientation::Horizontal,
        );
        let html = render(&node);
        let expected_class = format!(
            r#"class="fw-wire-nav-item {} fw-wire-horizontal""#,
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
fn leading_and_trailing_some_render_icons_and_none_omits_them() {
    let with_slots = nav_item(
        "ホーム",
        Some(icon::house(Size::Md)),
        Some(icon::caret_right(Size::Md)),
        None,
        Size::Md,
        Active(false),
        Orientation::Horizontal,
    );
    let html = render(&with_slots);
    assert_eq!(html.matches("<svg").count(), 2);

    let without_slots = nav_item(
        "ホーム",
        None,
        None,
        None,
        Size::Md,
        Active(false),
        Orientation::Horizontal,
    );
    assert!(!render(&without_slots).contains("<svg"));
}

#[test]
fn counter_some_renders_pill_and_none_omits_it_entirely() {
    let with_counter = nav_item(
        "通知",
        None,
        None,
        Some("12"),
        Size::Md,
        Active(false),
        Orientation::Horizontal,
    );
    let html = render(&with_counter);
    assert!(html.contains(r#"class="fw-wire-nav-item-counter""#));
    assert!(html.contains("12"));

    let without_counter = nav_item(
        "通知",
        None,
        None,
        None,
        Size::Md,
        Active(false),
        Orientation::Horizontal,
    );
    assert!(!render(&without_counter).contains("fw-wire-nav-item-counter"));
}

#[test]
fn active_true_appends_data_active_and_false_omits_it() {
    let active = render(&nav_item(
        "ホーム",
        None,
        None,
        None,
        Size::Md,
        Active(true),
        Orientation::Horizontal,
    ));
    assert!(active.contains(r#"data-active="""#));

    let inactive = render(&nav_item(
        "ホーム",
        None,
        None,
        None,
        Size::Md,
        Active(false),
        Orientation::Horizontal,
    ));
    assert!(!inactive.contains("data-active"));
}

#[test]
fn orientation_vertical_appends_vertical_class() {
    let vertical = render(&nav_item(
        "ホーム",
        None,
        None,
        None,
        Size::Md,
        Active(false),
        Orientation::Vertical,
    ));
    assert!(vertical.contains("fw-wire-vertical"));
    assert!(!vertical.contains("fw-wire-horizontal"));
}

#[test]
fn label_part_class_is_always_present_even_for_empty_label() {
    let html = render(&nav_item(
        "ホーム",
        None,
        None,
        None,
        Size::Md,
        Active(false),
        Orientation::Horizontal,
    ));
    assert!(html.contains(r#"class="fw-wire-nav-item-label""#));
    assert!(html.contains("ホーム"));

    let empty = render(&nav_item(
        "",
        None,
        None,
        None,
        Size::Md,
        Active(false),
        Orientation::Horizontal,
    ));
    assert!(empty.contains(r#"class="fw-wire-nav-item-label""#));
}

#[test]
fn xss_regression_label_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&nav_item(
        payload,
        None,
        None,
        None,
        Size::Md,
        Active(false),
        Orientation::Horizontal,
    ));

    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn xss_regression_label_with_quote_payload_is_escaped() {
    let payload = "\"><img src=x onerror=alert(1)>";
    let html = render(&nav_item(
        payload,
        None,
        None,
        None,
        Size::Md,
        Active(false),
        Orientation::Horizontal,
    ));

    assert!(!html.contains(payload));
    assert!(html.contains("&quot;"));
}

#[test]
fn xss_regression_counter_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&nav_item(
        "ホーム",
        None,
        None,
        Some(payload),
        Size::Md,
        Active(false),
        Orientation::Horizontal,
    ));

    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn xss_regression_counter_with_quote_payload_is_escaped() {
    let payload = "\"><img src=x onerror=alert(1)>";
    let html = render(&nav_item(
        "ホーム",
        None,
        None,
        Some(payload),
        Size::Md,
        Active(false),
        Orientation::Horizontal,
    ));

    assert!(!html.contains(payload));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_native_form_elements() {
    // `leading`/`trailing` は付与しない: アイコン自体が装飾用途の
    // `aria-hidden`/`focusable` を持つため（`crate::icon` の契約）、
    // 部品ルート側の非対話制約検証とは区別する（`tests/input.rs` と
    // 同じ判断）。
    let html = render(&nav_item(
        "ホーム",
        None,
        None,
        Some("12"),
        Size::Md,
        Active(true),
        Orientation::Horizontal,
    ));
    for forbidden in [
        " role=\"",
        " aria-",
        " tabindex=\"",
        " style=\"",
        "<button",
        "<input",
        "<select",
        "<a ",
        "href=",
        "javascript:",
        " onclick=\"",
        " onload=\"",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn nav_item_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::nav_item::NAV_ITEM_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = fandhe_frontend_wireframe_ui::wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::nav_item::NAV_ITEM_CSS));
}

#[test]
fn nav_item_css_declares_the_expected_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::nav_item::NAV_ITEM_CSS;
    for selector in [
        ".fw-wire-nav-item {",
        ".fw-wire-nav-item-label {",
        ".fw-wire-nav-item-counter {",
        ".fw-wire-nav-item[data-active] {",
        ".fw-wire-nav-item[data-active] .fw-wire-nav-item-counter {",
        ".fw-wire-nav-item.fw-wire-vertical {",
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
