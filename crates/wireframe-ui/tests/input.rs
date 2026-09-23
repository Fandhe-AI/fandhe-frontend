//! `input` 部品の契約テスト（イシュー #2622）。
//!
//! `crates/wireframe-ui/tests/link.rs` と同型の観点（非対話制約・XSS 回帰・
//! CSS 配線）に加え、`leading` アイコンスロット（`Option<Node>`）と
//! `Active`/`Disabled` の `data-*` 出力・`<input>` 非出力を wireframe-ui
//! 側で単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{icon, input, wireframe_css, Active, Disabled, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = input("メールアドレス", None, size, Active(false), Disabled(false));
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-input {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn leading_some_renders_icon_and_none_omits_it_entirely() {
    let with_leading = input(
        "検索",
        Some(icon::search(Size::Md)),
        Size::Md,
        Active(false),
        Disabled(false),
    );
    assert!(render(&with_leading).contains("<svg"));

    let without_leading = input("検索", None, Size::Md, Active(false), Disabled(false));
    assert!(!render(&without_leading).contains("<svg"));
}

#[test]
fn active_true_appends_data_active_and_false_omits_it() {
    let active = render(&input("t", None, Size::Md, Active(true), Disabled(false)));
    assert!(active.contains(r#"data-active="""#));

    let inactive = render(&input("t", None, Size::Md, Active(false), Disabled(false)));
    assert!(!inactive.contains("data-active"));
}

#[test]
fn disabled_true_appends_data_disabled_and_false_omits_it() {
    let disabled = render(&input("t", None, Size::Md, Active(false), Disabled(true)));
    assert!(disabled.contains(r#"data-disabled="""#));

    let enabled = render(&input("t", None, Size::Md, Active(false), Disabled(false)));
    assert!(!enabled.contains("data-disabled"));
}

#[test]
fn text_part_class_is_always_present_even_for_empty_text() {
    let html = render(&input(
        "入力してください",
        None,
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    assert!(html.contains(r#"class="fw-wire-input-text""#));
    assert!(html.contains("入力してください"));

    let empty = render(&input("", None, Size::Md, Active(false), Disabled(false)));
    assert!(empty.contains(r#"class="fw-wire-input-text""#));
}

#[test]
fn xss_regression_text_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&input(
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
    let html = render(&input(
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
fn output_has_no_interactive_semantics_or_native_form_elements() {
    // `leading` は付与しない: アイコン自体が装飾用途の `aria-hidden`/`focusable`
    // を持つため（`crate::icon` の契約）、部品ルート側の非対話制約検証とは
    // 区別する（`crates/wireframe-ui/tests/link.rs` の同種テストと同じ判断）。
    let html = render(&input("t", None, Size::Md, Active(true), Disabled(true)));
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
        "placeholder=",
        "value=",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn input_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::input::INPUT_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::input::INPUT_CSS));
}

#[test]
fn input_css_declares_the_five_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::input::INPUT_CSS;
    for selector in [
        ".fw-wire-input {",
        ".fw-wire-input-text {",
        ".fw-wire-input .fw-wire-icon-glyph {",
        ".fw-wire-input[data-active] {",
        ".fw-wire-input[data-disabled] {",
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
