//! `textarea` 部品の契約テスト（イシュー #2623）。
//!
//! `crates/wireframe-ui/tests/grid.rs`/`annotation.rs` と同型の観点
//! （root class・XSS 回帰・非対話制約・CSS 配線）に加え、`textarea` 固有の
//! `rows` 丸め・`text` の先頭行への流し込み・`data-active`/`data-disabled`
//! のみを許容する表示状態制約を固定する。共有の非対話制約リストには
//! `<textarea` が含まれないため、本ファイルで明示的に禁止する
//! （`crates/docs-site/tests/wireframes_contract.rs` は `<form`/`<button`/
//! `<input`/`<select`/`<a href` のみを対象にしており、ネイティブ
//! `<textarea>` 非出力は部品固有の契約として本ファイルが担う）。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{textarea, Active, Disabled, Size, MAX_ROWS, PARTS};

#[test]
fn max_rows_is_twenty() {
    assert_eq!(MAX_ROWS, 20);
}

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = textarea("本文", 2, size, Active(false), Disabled(false));
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-textarea {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn rows_are_clamped_to_one_through_max() {
    let cases: [(u32, usize); 6] = [
        (0, 1),
        (1, 1),
        (5, 5),
        (20, 20),
        (21, 20),
        (u32::MAX as usize as u32, 20),
    ];
    for (rows, expected_count) in cases {
        let html = render(&textarea(
            "x",
            rows,
            Size::Md,
            Active(false),
            Disabled(false),
        ));
        let actual_count = html.matches("fw-wire-textarea-line").count();
        assert_eq!(
            actual_count, expected_count,
            "rows={rows}: expected {expected_count} line placeholders in {html:?}"
        );
    }
}

#[test]
fn non_empty_text_is_placed_in_the_first_line_only() {
    let html = render(&textarea(
        "本文",
        3,
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    assert_eq!(html.matches(r#"class="fw-wire-textarea-text""#).count(), 1);
    assert!(html.contains("本文"));
}

#[test]
fn empty_text_renders_no_text_part() {
    let html = render(&textarea("", 3, Size::Md, Active(false), Disabled(false)));
    assert!(!html.contains("fw-wire-textarea-text"));
}

#[test]
fn display_state_renders_only_data_active_or_data_disabled() {
    let neither = render(&textarea("x", 1, Size::Md, Active(false), Disabled(false)));
    assert_eq!(neither.matches("data-").count(), 0);

    let active_only = render(&textarea("x", 1, Size::Md, Active(true), Disabled(false)));
    assert!(active_only.contains(r#"data-active="""#));
    assert!(!active_only.contains("data-disabled"));
    assert_eq!(active_only.matches("data-").count(), 1);

    let disabled_only = render(&textarea("x", 1, Size::Md, Active(false), Disabled(true)));
    assert!(disabled_only.contains(r#"data-disabled="""#));
    assert!(!disabled_only.contains("data-active"));
    assert_eq!(disabled_only.matches("data-").count(), 1);

    let both = render(&textarea("x", 1, Size::Md, Active(true), Disabled(true)));
    assert!(both.contains(r#"data-active="""#));
    assert!(both.contains(r#"data-disabled="""#));
    assert_eq!(both.matches("data-").count(), 2);
}

#[test]
fn xss_regression_text_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&textarea(
        payload,
        1,
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));

    let attr_payload = "\"><img src=x onerror=alert(1)>";
    let html_attr = render(&textarea(
        attr_payload,
        1,
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    assert!(!html_attr.contains(attr_payload));
    assert!(html_attr.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_style_or_native_textarea() {
    let html = render(&textarea("本文", 2, Size::Md, Active(true), Disabled(true)));
    for forbidden in [
        " role=\"",
        " aria-",
        " tabindex=\"",
        " style=\"",
        "<textarea",
        "contenteditable",
        "<button",
        "<input",
        "<form",
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
fn textarea_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::textarea::TEXTAREA_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = fandhe_frontend_wireframe_ui::wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::textarea::TEXTAREA_CSS));
}

#[test]
fn textarea_css_declares_the_six_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::textarea::TEXTAREA_CSS;

    let expected_selectors = [
        ".fw-wire-textarea {",
        ".fw-wire-textarea-line {",
        ".fw-wire-textarea-text {",
        ".fw-wire-textarea::after {",
        ".fw-wire-textarea[data-active] {",
        ".fw-wire-textarea[data-disabled] {",
    ];
    assert_eq!(expected_selectors.len(), 6);

    for selector in expected_selectors {
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
