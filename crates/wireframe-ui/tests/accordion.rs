//! `accordion` 部品の契約テスト（イシュー #2641）。
//!
//! `crates/wireframe-ui/tests/tabs.rs`・`question.rs` と同型の観点
//! （非対話制約・XSS 回帰・CSS 配線）に加え、項目単位の展開状態
//! （`props::Active` 再利用の `data-active`）・折りたたみ項目が本文
//! スロットを出力しないこと・見出し末尾キャレットの向きを wireframe-ui
//! 側で単体固定する。

use fandhe_frontend_core::{div, render, text};
use fandhe_frontend_wireframe_ui::{accordion, wireframe_css, Size, PARTS};

fn probe_body(marker: &str) -> fandhe_frontend_core::Node {
    div(vec![("class", "probe")], vec![text(marker)])
}

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = accordion(vec![("見出し", probe_body("本文"), false)], size);
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-accordion {}""#, size.class());
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
    let items = vec![
        ("項目1", probe_body("本文1"), false),
        ("項目2", probe_body("本文2"), true),
        ("項目3", probe_body("本文3"), false),
    ];
    let html = render(&accordion(items, Size::Md));
    assert_eq!(html.matches(r#"class="fw-wire-accordion-item""#).count(), 3);
}

#[test]
fn empty_items_does_not_panic_and_renders_no_items() {
    let html = render(&accordion(vec![], Size::Md));
    assert!(!html.contains("fw-wire-accordion-item"));
    assert!(html.contains(r#"class="fw-wire-accordion fw-wire-size-md""#));
}

#[test]
fn expanded_items_render_body_and_data_active() {
    let items = vec![
        ("展開", probe_body("展開本文"), true),
        ("折りたたみ", probe_body("折りたたみ本文"), false),
    ];
    let html = render(&accordion(items, Size::Md));
    assert_eq!(html.matches(r#"data-active="""#).count(), 1);
    assert_eq!(html.matches(r#"class="fw-wire-accordion-body""#).count(), 1);
    assert!(html.contains("展開本文"));
}

#[test]
fn collapsed_items_omit_body() {
    let items = vec![("折りたたみ", probe_body("隠れる本文"), false)];
    let html = render(&accordion(items, Size::Md));
    assert!(!html.contains("fw-wire-accordion-body"));
    assert!(!html.contains("隠れる本文"));
    assert!(!html.contains("probe"));
}

#[test]
fn caret_direction_follows_expanded() {
    let expanded = render(&accordion(
        vec![("展開", probe_body("本文"), true)],
        Size::Md,
    ));
    assert!(expanded.contains(r#"data-icon="caret-up""#));
    assert!(!expanded.contains(r#"data-icon="caret-down""#));

    let collapsed = render(&accordion(
        vec![("折りたたみ", probe_body("本文"), false)],
        Size::Md,
    ));
    assert!(collapsed.contains(r#"data-icon="caret-down""#));
    assert!(!collapsed.contains(r#"data-icon="caret-up""#));
}

#[test]
fn titles_preserve_order() {
    let items = vec![
        ("最初", probe_body("a"), false),
        ("次", probe_body("b"), false),
        ("最後", probe_body("c"), false),
    ];
    let html = render(&accordion(items, Size::Md));
    let first_pos = html.find("最初").unwrap();
    let second_pos = html.find("次").unwrap();
    let third_pos = html.find("最後").unwrap();
    assert!(first_pos < second_pos);
    assert!(second_pos < third_pos);
}

#[test]
fn xss_regression_title_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&accordion(
        vec![(payload, probe_body("本文"), false)],
        Size::Md,
    ));
    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn xss_regression_title_with_quote_payload_is_escaped() {
    let payload = "\"><img src=x onerror=alert(1)>";
    let html = render(&accordion(
        vec![(payload, probe_body("本文"), false)],
        Size::Md,
    ));
    assert!(!html.contains(payload));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style() {
    let items = vec![
        ("展開", probe_body("本文A"), true),
        ("折りたたみ", probe_body("本文B"), false),
    ];
    let html = render(&accordion(items, Size::Md));

    for forbidden in [
        "<details",
        "<summary",
        "<button",
        "<a ",
        "<input",
        "<select",
        " role=\"",
        " tabindex=\"",
        " style=\"",
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

    // `aria-hidden="true"` はアイコン基盤（`crate::icon`）が装飾用途として
    // 付与する既定の属性であり、対話的 ARIA ではないため許容する。それ
    // 以外の `aria-` 属性（`aria-expanded` を含む）は一切出力しない。
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
fn accordion_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::accordion::ACCORDION_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::accordion::ACCORDION_CSS));
}

#[test]
fn accordion_css_declares_expected_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::accordion::ACCORDION_CSS;
    for selector in [
        ".fw-wire-accordion {",
        ".fw-wire-accordion-item {",
        ".fw-wire-accordion-header {",
        ".fw-wire-accordion-title {",
        ".fw-wire-accordion-indicator {",
        ".fw-wire-accordion-body {",
        ".fw-wire-accordion-item[data-active] {",
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
