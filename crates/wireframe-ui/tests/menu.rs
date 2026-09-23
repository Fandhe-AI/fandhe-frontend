//! `menu` 部品の契約テスト（イシュー #2637）。
//!
//! `crates/wireframe-ui/tests/tabs.rs`/`nav_item.rs`/`select.rs` と同型の
//! 観点（非対話制約・XSS 回帰・CSS 配線）に加え、[`MenuItem`] スライス・
//! `active: Option<usize>`（無効項目優先の fail-closed）・
//! `search: Option<&str>`（先頭固定の `icon::search`、`<input>` 非出力）を
//! wireframe-ui 側で単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::menu::{menu, MenuItem};
use fandhe_frontend_wireframe_ui::{Size, PARTS};

fn sample_items() -> [MenuItem<'static>; 3] {
    [
        MenuItem::new("プロフィール"),
        MenuItem::new("設定"),
        MenuItem::disabled("請求情報"),
    ]
}

#[test]
fn renders_root_class_for_every_size() {
    let items = sample_items();
    for size in Size::ALL {
        let node = menu(&items, None, None, size);
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-menu {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn item_count_matches_input_and_empty_slice_does_not_panic() {
    let items = sample_items();
    let html = render(&menu(&items, None, None, Size::Md));
    assert_eq!(html.matches(r#"class="fw-wire-menu-item""#).count(), 3);

    let empty = render(&menu(&[], None, None, Size::Md));
    assert!(!empty.contains("fw-wire-menu-item"));
}

#[test]
fn disabled_items_render_data_disabled_for_each_disabled_item() {
    let items = sample_items();
    let html = render(&menu(&items, None, None, Size::Md));
    assert_eq!(html.matches(r#"data-disabled="""#).count(), 1);
}

#[test]
fn active_some_valid_enabled_index_renders_exactly_one_data_active() {
    let items = sample_items();
    let html = render(&menu(&items, Some(0), None, Size::Md));
    assert_eq!(html.matches(r#"data-active="""#).count(), 1);
}

#[test]
fn active_none_renders_no_data_active() {
    let items = sample_items();
    let html = render(&menu(&items, None, None, Size::Md));
    assert!(!html.contains("data-active"));
}

#[test]
fn active_out_of_range_renders_no_data_active() {
    let items = sample_items();
    let len_index = render(&menu(&items, Some(items.len()), None, Size::Md));
    assert!(!len_index.contains("data-active"));

    let max_index = render(&menu(&items, Some(usize::MAX), None, Size::Md));
    assert!(!max_index.contains("data-active"));
}

#[test]
fn active_pointing_to_disabled_item_renders_no_data_active() {
    let items = sample_items();
    // index 2 は disabled 項目（「請求情報」）。
    let html = render(&menu(&items, Some(2), None, Size::Md));
    assert!(!html.contains("data-active"));
}

#[test]
fn search_none_omits_search_row_and_icon() {
    let items = sample_items();
    let html = render(&menu(&items, None, None, Size::Md));
    assert!(!html.contains("fw-wire-menu-search"));
    assert!(!html.contains(r#"data-icon="search""#));
    assert!(!html.contains("<svg"));
}

#[test]
fn search_some_renders_search_row_before_items_with_icon_and_placeholder() {
    let items = sample_items();
    let html = render(&menu(&items, None, Some("検索..."), Size::Md));
    assert!(html.contains(r#"class="fw-wire-menu-search""#));
    assert!(html.contains(r#"data-icon="search""#));
    assert!(html.contains("検索..."));

    let search_pos = html
        .find("fw-wire-menu-search")
        .expect("search row missing");
    let item_pos = html
        .find(r#"class="fw-wire-menu-item""#)
        .expect("item missing");
    assert!(search_pos < item_pos, "search row should precede items");
}

#[test]
fn search_empty_string_renders_search_row() {
    let items = sample_items();
    let html = render(&menu(&items, None, Some(""), Size::Md));
    assert!(html.contains(r#"class="fw-wire-menu-search""#));
}

#[test]
fn item_label_output_order_matches_input_order() {
    let items = sample_items();
    let html = render(&menu(&items, None, None, Size::Md));
    let profile_pos = html.find("プロフィール").expect("missing label");
    let settings_pos = html.find("設定").expect("missing label");
    let billing_pos = html.find("請求情報").expect("missing label");
    assert!(profile_pos < settings_pos);
    assert!(settings_pos < billing_pos);
}

#[test]
fn xss_regression_label_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let items = [MenuItem::new(payload)];
    let html = render(&menu(&items, None, None, Size::Md));
    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn xss_regression_label_with_quote_payload_is_escaped() {
    let payload = "\"><img src=x onerror=alert(1)>";
    let items = [MenuItem::new(payload)];
    let html = render(&menu(&items, None, None, Size::Md));
    assert!(!html.contains(payload));
    assert!(html.contains("&quot;"));
}

#[test]
fn xss_regression_search_placeholder_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let items = sample_items();
    let html = render(&menu(&items, None, Some(payload), Size::Md));
    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn xss_regression_search_placeholder_with_quote_payload_is_escaped() {
    let payload = "\"><img src=x onerror=alert(1)>";
    let items = sample_items();
    let html = render(&menu(&items, None, Some(payload), Size::Md));
    assert!(!html.contains(payload));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_native_form_elements() {
    let items = sample_items();
    let html = render(&menu(&items, Some(0), Some("検索..."), Size::Md));
    for forbidden in [
        "<input",
        "<button",
        "<select",
        "<a ",
        "href=",
        "javascript:",
        " role=\"",
        " tabindex=\"",
        " style=\"",
        " onclick=\"",
        " onload=\"",
        "aria-expanded",
        "aria-haspopup",
        "role=\"menu\"",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
    // `aria-hidden="true"` はアイコン基盤（`crate::icon`）が装飾用途として
    // 付与する既定の属性であり、対話的 ARIA ではないため許容する。それ
    // 以外の `aria-` 属性は一切出力しない（`tests/select.rs` と同じ判断）。
    let aria_attrs: Vec<&str> = html
        .split(' ')
        .filter(|token| token.starts_with("aria-"))
        .collect();
    for attr in aria_attrs {
        assert!(
            attr.starts_with("aria-hidden="),
            "unexpected non-decorative aria attribute {attr:?} in {html:?}"
        );
    }
}

#[test]
fn menu_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::menu::MENU_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = fandhe_frontend_wireframe_ui::wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::menu::MENU_CSS));
}

#[test]
fn menu_css_declares_the_expected_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::menu::MENU_CSS;
    for selector in [
        ".fw-wire-menu {",
        ".fw-wire-menu-search {",
        ".fw-wire-menu-search-text {",
        ".fw-wire-menu .fw-wire-icon-glyph {",
        ".fw-wire-menu-item {",
        ".fw-wire-menu-item[data-active] {",
        ".fw-wire-menu-item[data-disabled] {",
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
