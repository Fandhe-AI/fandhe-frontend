//! `cursor` 部品の契約テスト（イシュー #2642）。
//!
//! `crates/wireframe-ui/tests/nav_item.rs`/`link.rs` と同型の観点
//! （非対話制約・XSS 回帰・CSS 配線）に加え、`kind`（[`CursorKind`]）に
//! 応じたグリフ切り替え・`label`（`Option<&str>`）の出力有無を
//! wireframe-ui 側で単体固定する。新規アイコン（`icon::cursor_arrow`/
//! `icon::cursor_hand`）自体の出力契約は `tests/icon.rs` が全アイコン
//! 横断で検証するため、本ファイルでは重複させない。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{cursor, CursorKind, Size, PARTS};

#[test]
fn renders_root_class_for_every_kind_and_size() {
    for kind in CursorKind::ALL {
        for size in Size::ALL {
            let node = cursor(kind, None, size);
            let html = render(&node);
            let expected_class = format!(
                r#"class="fw-wire-cursor {} {}""#,
                kind.class(),
                size.class()
            );
            assert!(
                html.contains(&expected_class),
                "expected {expected_class:?} in {html:?}"
            );
            assert!(html.starts_with("<span"));
            assert!(html.trim_end().ends_with("</span>"));
        }
    }
}

#[test]
fn kind_selects_matching_glyph() {
    let arrow_html = render(&cursor(CursorKind::Arrow, None, Size::Md));
    assert!(arrow_html.contains(r#"data-icon="cursor-arrow""#));
    assert!(!arrow_html.contains(r#"data-icon="cursor-hand""#));

    let hand_html = render(&cursor(CursorKind::Hand, None, Size::Md));
    assert!(hand_html.contains(r#"data-icon="cursor-hand""#));
    assert!(!hand_html.contains(r#"data-icon="cursor-arrow""#));
}

#[test]
fn default_kind_is_arrow() {
    assert_eq!(CursorKind::default(), CursorKind::Arrow);
    assert_eq!(CursorKind::default().as_str(), "arrow");
    assert_eq!(CursorKind::default().class(), "fw-wire-cursor-arrow");
    assert_eq!(CursorKind::Hand.as_str(), "hand");
    assert_eq!(CursorKind::Hand.class(), "fw-wire-cursor-hand");
}

#[test]
fn label_some_renders_part_and_none_omits_it() {
    let with_label = render(&cursor(CursorKind::Arrow, Some("にゃんこ"), Size::Md));
    assert!(with_label.contains(r#"class="fw-wire-cursor-label""#));
    assert!(with_label.contains("にゃんこ"));

    let without_label = render(&cursor(CursorKind::Arrow, None, Size::Md));
    assert!(!without_label.contains("fw-wire-cursor-label"));
}

#[test]
fn xss_regression_label_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&cursor(CursorKind::Arrow, Some(payload), Size::Md));

    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn xss_regression_label_with_quote_payload_is_escaped() {
    let payload = "\"><img src=x onerror=alert(1)>";
    let html = render(&cursor(CursorKind::Arrow, Some(payload), Size::Md));

    assert!(!html.contains(payload));
    assert!(html.contains("&quot;"));
    assert!(!html.contains("<img"));
}

#[test]
fn output_has_no_interactive_semantics_or_native_form_elements() {
    // グリフ（`icon::cursor_arrow`/`icon::cursor_hand`）は常に描画され、
    // 装飾用途の `aria-hidden`/`focusable` を持つ（`crate::icon` の契約、
    // `tests/icon.rs` が検証済み）。`nav_item` のようにスロットを `None`
    // にして回避できないため、本テストでは `aria-hidden`/`focusable` を
    // 除いた対話セマンティクス（`role`・`aria-hidden` 以外の `aria-*`・
    // `tabindex`・`style`・ネイティブ対話要素・イベントハンドラ）のみを
    // 禁止対象とする。
    let html = render(&cursor(CursorKind::Hand, Some("にゃんこ"), Size::Md));
    assert!(!html.contains(" role=\""));
    assert!(!html.contains(" tabindex=\""));
    assert!(!html.contains(" style=\""));
    assert!(!html.contains("<button"));
    assert!(!html.contains("<input"));
    assert!(!html.contains("<select"));
    assert!(!html.contains("<a "));
    assert!(!html.contains("href="));
    assert!(!html.contains("javascript:"));
    assert!(!html.contains(" onclick=\""));
    assert!(!html.contains(" onload=\""));
    for aria in html.match_indices(" aria-") {
        let rest = &html[aria.0..];
        assert!(
            rest.starts_with(" aria-hidden=\"true\""),
            "unexpected non aria-hidden attribute at {rest:?}"
        );
    }
}

#[test]
fn cursor_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::cursor::CURSOR_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = fandhe_frontend_wireframe_ui::wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::cursor::CURSOR_CSS));
}

#[test]
fn cursor_css_declares_the_expected_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::cursor::CURSOR_CSS;
    for selector in [
        ".fw-wire-cursor {",
        ".fw-wire-cursor .fw-wire-icon-glyph {",
        ".fw-wire-cursor-label {",
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
