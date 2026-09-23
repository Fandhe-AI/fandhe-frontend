//! `question` 部品の契約テスト（イシュー #2630）。
//!
//! `crates/wireframe-ui/tests/annotation.rs`・`select.rs` と同型の観点
//! （非対話制約・XSS 回帰・CSS 配線）を wireframe-ui 側で単体固定する。
//! `question` はコントロールを `Node` スロットで受けるため、素朴な
//! `div`/`text` を渡すケースと、実部品（`select`）を渡すケースの両方で
//! 非対話性を検証する（2 層契約）。

use fandhe_frontend_core::{div, render, text, Node};
use fandhe_frontend_wireframe_ui::{
    question, select, wireframe_css, Active, Disabled, Size, PARTS,
};

fn probe_control() -> Node {
    div(vec![("class", "probe")], vec![text("probe-content")])
}

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = question("質問文", None, probe_control(), None, size);
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-question {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn label_is_always_rendered() {
    let html = render(&question("質問文", None, probe_control(), None, Size::Md));
    assert!(html.contains(r#"class="fw-wire-question-label""#));
    assert!(html.contains("質問文"));
}

#[test]
fn description_some_renders_part_and_none_omits_it_entirely() {
    let with_description = render(&question(
        "質問文",
        Some("補足説明"),
        probe_control(),
        None,
        Size::Md,
    ));
    assert!(with_description.contains(r#"class="fw-wire-question-description""#));
    assert!(with_description.contains("補足説明"));

    let without_description = render(&question("質問文", None, probe_control(), None, Size::Md));
    assert!(!without_description.contains("fw-wire-question-description"));
}

#[test]
fn hint_some_renders_part_and_none_omits_it_entirely() {
    let with_hint = render(&question(
        "質問文",
        None,
        probe_control(),
        Some("ヒント文言"),
        Size::Md,
    ));
    assert!(with_hint.contains(r#"class="fw-wire-question-hint""#));
    assert!(with_hint.contains("ヒント文言"));

    let without_hint = render(&question("質問文", None, probe_control(), None, Size::Md));
    assert!(!without_hint.contains("fw-wire-question-hint"));
}

#[test]
fn control_slot_is_passed_through_as_is() {
    let html = render(&question("質問文", None, probe_control(), None, Size::Md));
    assert!(html.contains(r#"class="fw-wire-question-control""#));
    assert!(html.contains(r#"class="probe""#));
    assert!(html.contains("probe-content"));
}

#[test]
fn xss_regression_label_description_and_hint_are_escaped() {
    let payload_a = "<script>alert(1)</script>";
    let payload_b = "\"><img src=x onerror=alert(1)>";
    let payload_c = "<script>alert(2)</script>";
    let html = render(&question(
        payload_a,
        Some(payload_b),
        probe_control(),
        Some(payload_c),
        Size::Md,
    ));

    assert!(!html.contains(payload_a));
    assert!(!html.contains(payload_b));
    assert!(!html.contains(payload_c));
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_with_naive_control() {
    let html = render(&question(
        "質問文",
        Some("補足"),
        probe_control(),
        Some("ヒント"),
        Size::Md,
    ));
    for forbidden in [
        " role=\"",
        " aria-",
        " tabindex=\"",
        " style=\"",
        "<label",
        "<input",
        "<fieldset",
        "<legend",
        "<button",
        "<a ",
        "href=",
        "javascript:",
        " onclick=\"",
        " onload=\"",
        "data-",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn root_tag_has_no_attributes_other_than_class_when_control_carries_data_attrs() {
    // 実部品（select）を control に渡すと、select 自身の data-active/
    // aria-hidden（アイコンの装飾用）は control スロット内に現れてよいが、
    // question ルートの開始タグには class 以外の属性が現れないことを固定する。
    let control = select("山田太郎", None, Size::Md, Active(true), Disabled(false));
    let html = render(&question("質問文", None, control, None, Size::Md));

    // control 由来の data-active はどこかに含まれる（スロットは透過する）。
    assert!(html.contains("data-active"));

    // question ルートの開始タグ自体（最初の <div ...> ）は class のみを持つ。
    let root_open_tag_end = html.find('>').expect("root open tag should exist");
    let root_open_tag = &html[..=root_open_tag_end];
    assert!(root_open_tag.starts_with("<div class=\"fw-wire-question"));
    assert!(!root_open_tag.contains("data-"));
    assert!(!root_open_tag.contains(" role=\""));
    assert!(!root_open_tag.contains(" aria-"));
    assert!(!root_open_tag.contains(" tabindex=\""));
    assert!(!root_open_tag.contains(" style=\""));
}

#[test]
fn question_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::question::QUESTION_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::question::QUESTION_CSS));
}

#[test]
fn question_css_declares_the_five_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::question::QUESTION_CSS;
    for selector in [
        ".fw-wire-question {",
        ".fw-wire-question-label {",
        ".fw-wire-question-description {",
        ".fw-wire-question-control {",
        ".fw-wire-question-hint {",
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
