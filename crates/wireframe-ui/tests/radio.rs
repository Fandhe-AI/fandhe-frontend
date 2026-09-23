//! `radio` 部品の契約テスト（イシュー #2626）。
//!
//! `crates/wireframe-ui/tests/select.rs`（`Active`/`Disabled` を併用する
//! 先例）と同型の観点（非対話制約・XSS 回帰・CSS 配線）に加え、`label`
//! （`Option<&str>`）の条件付き出力と、選択状態を表す `Active`（`data-active`）
//! の再利用を wireframe-ui 側で単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{radio, wireframe_css, Active, Disabled, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = radio(Some("選択肢"), size, Active(false), Disabled(false));
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-radio {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn control_part_is_always_rendered() {
    let with_label = render(&radio(Some("t"), Size::Md, Active(false), Disabled(false)));
    assert_eq!(
        with_label
            .matches(r#"class="fw-wire-radio-control""#)
            .count(),
        1
    );

    let without_label = render(&radio(None, Size::Md, Active(false), Disabled(false)));
    assert_eq!(
        without_label
            .matches(r#"class="fw-wire-radio-control""#)
            .count(),
        1
    );
}

#[test]
fn label_some_renders_part_and_none_omits_it_entirely() {
    let with_label = render(&radio(
        Some("選択肢 A"),
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    assert!(with_label.contains(r#"class="fw-wire-radio-label""#));
    assert!(with_label.contains("選択肢 A"));

    let without_label = render(&radio(None, Size::Md, Active(false), Disabled(false)));
    assert!(!without_label.contains("fw-wire-radio-label"));
}

#[test]
fn active_true_adds_data_active_and_false_omits_it() {
    let with_active = render(&radio(Some("t"), Size::Md, Active(true), Disabled(false)));
    assert!(with_active.contains(r#"data-active="""#));

    let without_active = render(&radio(Some("t"), Size::Md, Active(false), Disabled(false)));
    assert!(!without_active.contains("data-active"));
}

#[test]
fn disabled_true_adds_data_disabled_and_false_omits_it() {
    let with_disabled = render(&radio(Some("t"), Size::Md, Active(false), Disabled(true)));
    assert!(with_disabled.contains(r#"data-disabled="""#));

    let without_disabled = render(&radio(Some("t"), Size::Md, Active(false), Disabled(false)));
    assert!(!without_disabled.contains("data-disabled"));
}

#[test]
fn active_and_disabled_can_be_combined() {
    let html = render(&radio(Some("t"), Size::Md, Active(true), Disabled(true)));
    assert!(html.contains(r#"data-active="""#));
    assert!(html.contains(r#"data-disabled="""#));
}

#[test]
fn xss_regression_label_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&radio(
        Some(payload),
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn xss_regression_label_with_quote_payload_is_escaped() {
    let payload = "\"><img src=x onerror=alert(1)>";
    let html = render(&radio(
        Some(payload),
        Size::Md,
        Active(false),
        Disabled(false),
    ));
    assert!(!html.contains(payload));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style() {
    let html = render(&radio(Some("t"), Size::Md, Active(true), Disabled(true)));
    for forbidden in [
        "<input",
        "<label",
        "<button",
        "<select",
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
fn radio_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::radio::RADIO_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::radio::RADIO_CSS));
}

#[test]
fn radio_css_declares_expected_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::radio::RADIO_CSS;
    for selector in [
        ".fw-wire-radio {",
        ".fw-wire-radio-control {",
        ".fw-wire-radio-label {",
        ".fw-wire-radio[data-active] .fw-wire-radio-control {",
        ".fw-wire-radio[data-active] .fw-wire-radio-control::after {",
        ".fw-wire-radio[data-disabled] {",
        ".fw-wire-radio[data-disabled] .fw-wire-radio-control {",
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

    // 黒丸（`::after` の `content`）は `data-active` 連結セレクタ配下にのみ
    // 存在する（選択済み表現が Active 状態にのみ現れる不変条件）。上で
    // `.fw-wire-radio[data-active] .fw-wire-radio-control::after {` の完全一致
    // 検証済みのため、`::after` の出現が 1 回のみであることのみ追加で
    // 確認する（`[data-disabled]` 側には存在しないことの裏取り）。
    assert_eq!(css.matches("::after").count(), 1);
}
