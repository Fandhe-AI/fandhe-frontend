//! `modal` 部品の契約テスト（イシュー #2645）。
//!
//! `crates/wireframe-ui/tests/question.rs` と同型の観点（非対話制約・
//! XSS 回帰・CSS 配線）を単体固定する。`body`/`actions` は `Node` スロット
//! で受けるため、素朴な `div`/`text` を渡すケースと、実部品（`button`）を
//! 渡すケースの両方で非対話性を検証する（2 層契約）。

use fandhe_frontend_core::{div, render, text, Node};
use fandhe_frontend_wireframe_ui::{button, modal, wireframe_css, Disabled, Primary, Size, PARTS};

fn probe_body() -> Node {
    div(vec![("class", "probe-body")], vec![text("probe-content")])
}

fn probe_action(label: &str) -> Node {
    div(vec![("class", "probe-action")], vec![text(label)])
}

#[test]
fn renders_root_class_for_every_size() {
    for (size, max_width_class) in [
        (Size::Xs, "fw-wire-modal-max-width-xs"),
        (Size::Sm, "fw-wire-modal-max-width-sm"),
        (Size::Md, "fw-wire-modal-max-width-md"),
        (Size::Lg, "fw-wire-modal-max-width-lg"),
        (Size::Xl, "fw-wire-modal-max-width-xl"),
    ] {
        let node = modal("タイトル", probe_body(), vec![], size);
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-modal {max_width_class}""#);
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn root_class_is_modal_specific_and_does_not_leak_shared_size_class() {
    // コードレビュー指摘（イシュー #2645）: 共有 `fw-wire-size-*` class を
    // ルートへ付けると `--fw-wire-font-size` がパネル・タイトル・
    // `body`/`actions` スロットへ暗黙に継承されてしまう
    // （`crate::frame`/`crate::stack` が同種の問題を先に回避した前例と
    // 同じ設計、`crates/wireframe-ui/src/modal.rs` の `MODAL_CSS` doc
    // 参照）。Modal 専用の修飾 class のみを使うことを固定する。
    let html = render(&modal("タイトル", probe_body(), vec![], Size::Md));
    assert!(html.contains("fw-wire-modal-max-width-md"));
    assert!(!html.contains("fw-wire-size-"));
}

#[test]
fn panel_title_and_body_are_always_rendered() {
    let html = render(&modal("確認", probe_body(), vec![], Size::Md));
    assert!(html.contains(r#"class="fw-wire-modal-panel""#));
    assert!(html.contains(r#"class="fw-wire-modal-title""#));
    assert!(html.contains("確認"));
    assert!(html.contains(r#"class="fw-wire-modal-body""#));
    assert!(html.contains("probe-body"));
    assert!(html.contains("probe-content"));
}

#[test]
fn actions_empty_omits_the_part_entirely() {
    let html = render(&modal("タイトル", probe_body(), vec![], Size::Md));
    assert!(!html.contains("fw-wire-modal-actions"));
}

#[test]
fn actions_non_empty_renders_part_with_all_children_in_order() {
    let html = render(&modal(
        "タイトル",
        probe_body(),
        vec![probe_action("キャンセル"), probe_action("OK")],
        Size::Md,
    ));
    assert!(html.contains(r#"class="fw-wire-modal-actions""#));
    let cancel_pos = html.find("キャンセル").expect("cancel should be present");
    let ok_pos = html.find(">OK<").expect("ok should be present");
    assert!(cancel_pos < ok_pos, "actions should preserve call order");
}

#[test]
fn xss_regression_title_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&modal(payload, probe_body(), vec![], Size::Md));
    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn output_has_no_interactive_semantics_with_naive_slots() {
    let html = render(&modal(
        "タイトル",
        probe_body(),
        vec![probe_action("キャンセル"), probe_action("OK")],
        Size::Md,
    ));
    for forbidden in [
        "<dialog",
        " role=\"",
        " aria-",
        " tabindex=\"",
        " style=\"",
        "<button",
        "<input",
        "<form",
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
fn root_tag_has_no_attributes_other_than_class_when_actions_carry_data_attrs() {
    // 実部品（button）を actions に渡すと button 自身の属性は actions
    // スロット内に現れてよいが、modal ルートの開始タグには class 以外の
    // 属性が現れないことを固定する。
    let ok = button("OK", None, Size::Md, Primary(true), Disabled(false));
    let html = render(&modal("確認", probe_body(), vec![ok], Size::Md));

    let root_open_tag_end = html.find('>').expect("root open tag should exist");
    let root_open_tag = &html[..=root_open_tag_end];
    assert!(root_open_tag.starts_with("<div class=\"fw-wire-modal"));
    assert!(!root_open_tag.contains("data-"));
    assert!(!root_open_tag.contains(" role=\""));
    assert!(!root_open_tag.contains(" aria-"));
    assert!(!root_open_tag.contains(" tabindex=\""));
    assert!(!root_open_tag.contains(" style=\""));
}

#[test]
fn modal_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::modal::MODAL_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::modal::MODAL_CSS));
}

#[test]
fn modal_css_declares_expected_selectors_with_fw_wire_prefix_only_and_no_fixed_positioning() {
    let css = fandhe_frontend_wireframe_ui::modal::MODAL_CSS;
    for selector in [
        ".fw-wire-modal {",
        ".fw-wire-modal-panel {",
        ".fw-wire-modal-title {",
        ".fw-wire-modal-body {",
        ".fw-wire-modal-actions {",
        ".fw-wire-modal.fw-wire-modal-max-width-xs .fw-wire-modal-panel {",
        ".fw-wire-modal.fw-wire-modal-max-width-sm .fw-wire-modal-panel {",
        ".fw-wire-modal.fw-wire-modal-max-width-md .fw-wire-modal-panel {",
        ".fw-wire-modal.fw-wire-modal-max-width-lg .fw-wire-modal-panel {",
        ".fw-wire-modal.fw-wire-modal-max-width-xl .fw-wire-modal-panel {",
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
    assert!(!css.contains("position: fixed"));
    assert!(!css.contains("position: absolute"));
    assert!(!css.contains("z-index"));

    // コードレビュー指摘（イシュー #2645）: `.fw-wire-modal-panel` は
    // 共有 `--fw-wire-font-size` を参照しない（パネル・スロットへの
    // タイポグラフィ暗黙継承を避ける、`MODAL_CSS` doc 参照）。
    assert!(!css.contains("fw-wire-font-size"));
}
