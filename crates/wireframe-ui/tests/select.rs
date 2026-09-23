//! `select` 部品の契約テスト（イシュー #2624）。
//!
//! `crates/wireframe-ui/tests/button.rs`・`link.rs` と同型の観点（非対話
//! 制約・XSS 回帰・CSS 配線）に加え、`leading` アイコンスロット
//! （`Option<Node>`）・`active`/`disabled`（`data-active`/`data-disabled`）
//! の条件付き出力と、常に出力される末尾のドロップダウン指示子
//! （`icon::caret_down`）を wireframe-ui 側で単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{icon, select, wireframe_css, Active, Disabled, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = select("未選択", None, size, Active(false), Disabled(false));
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-select {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn active_true_emits_data_active_and_false_omits_it() {
    let with_active = render(&select("t", None, Size::Md, Active(true), Disabled(false)));
    assert!(with_active.contains(r#"data-active="""#));

    let without_active = render(&select("t", None, Size::Md, Active(false), Disabled(false)));
    assert!(!without_active.contains("data-active"));
}

#[test]
fn disabled_true_emits_data_disabled_and_false_omits_all_data_attributes() {
    let with_disabled = render(&select("t", None, Size::Md, Active(false), Disabled(true)));
    assert!(with_disabled.contains(r#"data-disabled="""#));

    let without_disabled = render(&select("t", None, Size::Md, Active(false), Disabled(false)));
    // `data-icon`（アイコン基盤が常に付与する装飾用の識別子）は
    // `data-active`/`data-disabled` とは別物であり、指示子アイコンが
    // 常に出力されるため残る。表示状態を示す `data-active`/`data-disabled`
    // のみが無いことを確認する。
    assert!(!without_disabled.contains("data-active"));
    assert!(!without_disabled.contains("data-disabled"));
}

#[test]
fn leading_slot_some_renders_svg_before_text_and_none_renders_indicator_only() {
    let with_leading = render(&select(
        "山田太郎",
        Some(icon::user(Size::Md)),
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    let svg_count = with_leading.matches("<svg").count();
    assert_eq!(svg_count, 2, "expected leading icon + indicator svg");
    let svg_pos = with_leading.find("<svg").unwrap();
    let text_pos = with_leading.find("fw-wire-select-text").unwrap();
    assert!(svg_pos < text_pos, "expected leading <svg before text part");

    let without_leading = render(&select(
        "未選択",
        None,
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    let svg_count_without = without_leading.matches("<svg").count();
    assert_eq!(svg_count_without, 1, "indicator svg is always emitted");
}

#[test]
fn indicator_is_always_present_with_caret_down_icon() {
    let html = render(&select(
        "未選択",
        None,
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    assert!(html.contains(r#"class="fw-wire-select-indicator""#));
    assert!(html.contains(r#"data-icon="caret-down""#));
}

#[test]
fn text_is_wrapped_in_text_part() {
    let html = render(&select(
        "選択中の値",
        None,
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    assert!(html.contains(r#"class="fw-wire-select-text""#));
    assert!(html.contains("選択中の値"));
}

#[test]
fn xss_regression_text_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&select(
        payload,
        None,
        Size::Md,
        Active(false),
        Disabled(false),
    ));

    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn xss_regression_text_with_quote_payload_is_escaped() {
    let payload = "\"><img src=x onerror=alert(1)>";
    let html = render(&select(
        payload,
        None,
        Size::Md,
        Active(false),
        Disabled(false),
    ));

    assert!(!html.contains(payload));
    assert!(html.contains("&quot;"));
}

#[test]
fn xss_regression_text_is_escaped_with_leading_icon() {
    let payload = "<script>alert(1)</script>";
    let html = render(&select(
        payload,
        Some(icon::user(Size::Md)),
        Size::Md,
        Active(false),
        Disabled(false),
    ));

    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style() {
    let html = render(&select(
        "t",
        Some(icon::user(Size::Md)),
        Size::Md,
        Active(true),
        Disabled(true),
    ));
    // `aria-hidden="true"` はアイコン基盤（`crate::icon`）が装飾用途として
    // 付与する既定の属性であり、対話的 ARIA（`role`/`aria-expanded`/
    // `aria-haspopup` 等）ではないため許容する。それ以外の `aria-` 属性は
    // 一切出力しない。
    for forbidden in [
        "<select",
        "<button",
        "<input",
        " role=\"",
        " tabindex=\"",
        " style=\"",
        "<a ",
        "href=",
        "javascript:",
        " onclick=\"",
        "aria-expanded",
        "aria-haspopup",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
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
fn select_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::select::SELECT_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::select::SELECT_CSS));
}

#[test]
fn select_css_declares_expected_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::select::SELECT_CSS;
    for selector in [
        ".fw-wire-select {",
        ".fw-wire-select-text {",
        ".fw-wire-select-indicator {",
        ".fw-wire-select .fw-wire-icon-glyph {",
        ".fw-wire-select[data-active] {",
        ".fw-wire-select[data-disabled] {",
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
