//! `rich_text` 部品の契約テスト（イシュー #2616）。
//!
//! `crates/wireframe-ui/tests/annotation.rs` と同型の観点（非対話制約・
//! XSS 回帰・CSS 配線）に加え、`leading`/`trailing` の `Option<Node>`
//! スロット（`docs/design/wireframe-ui-architecture.md` §11.4）と
//! `Bold`/`Orientation` の消費を固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{
    icon, rich_text, wireframe_css, Bold, Orientation, Size, PARTS,
};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = rich_text(
            "ラベル",
            None,
            None,
            size,
            Bold(false),
            Orientation::Horizontal,
        );
        let html = render(&node);
        let expected_class = format!(
            r#"class="fw-wire-rich-text {} fw-wire-horizontal""#,
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
fn bold_true_appends_bold_class_and_false_omits_it() {
    let with_bold = render(&rich_text(
        "t",
        None,
        None,
        Size::Md,
        Bold(true),
        Orientation::Horizontal,
    ));
    assert!(with_bold
        .contains(r#"class="fw-wire-rich-text fw-wire-size-md fw-wire-bold fw-wire-horizontal""#));

    let without_bold = render(&rich_text(
        "t",
        None,
        None,
        Size::Md,
        Bold(false),
        Orientation::Horizontal,
    ));
    assert!(!without_bold.contains("fw-wire-bold"));
}

#[test]
fn orientation_controls_horizontal_or_vertical_class() {
    let horizontal = render(&rich_text(
        "t",
        None,
        None,
        Size::Md,
        Bold(false),
        Orientation::Horizontal,
    ));
    assert!(horizontal.contains("fw-wire-horizontal"));
    assert!(!horizontal.contains("fw-wire-vertical"));

    let vertical = render(&rich_text(
        "t",
        None,
        None,
        Size::Md,
        Bold(false),
        Orientation::Vertical,
    ));
    assert!(vertical.contains("fw-wire-vertical"));
    assert!(!vertical.contains("fw-wire-horizontal"));
}

#[test]
fn slots_some_render_svg_and_none_omits_it_entirely() {
    let with_slots = render(&rich_text(
        "t",
        Some(icon::plus(Size::Md)),
        Some(icon::caret_right(Size::Md)),
        Size::Md,
        Bold(false),
        Orientation::Horizontal,
    ));
    assert!(with_slots.contains("<svg"));
    assert!(with_slots.contains(r#"data-icon="plus""#));
    assert!(with_slots.contains(r#"data-icon="caret-right""#));

    let without_slots = render(&rich_text(
        "t",
        None,
        None,
        Size::Md,
        Bold(false),
        Orientation::Horizontal,
    ));
    assert!(!without_slots.contains("<svg"));
}

#[test]
fn slots_are_ordered_leading_label_trailing() {
    let html = render(&rich_text(
        "ラベル本体",
        Some(icon::plus(Size::Md)),
        Some(icon::caret_right(Size::Md)),
        Size::Md,
        Bold(false),
        Orientation::Horizontal,
    ));

    let leading_pos = html.find(r#"data-icon="plus""#).expect("leading svg");
    let label_pos = html
        .find(r#"class="fw-wire-rich-text-label""#)
        .expect("label span");
    let trailing_pos = html
        .find(r#"data-icon="caret-right""#)
        .expect("trailing svg");

    assert!(leading_pos < label_pos, "leading should precede label");
    assert!(label_pos < trailing_pos, "label should precede trailing");
}

#[test]
fn leading_only_omits_trailing_and_vice_versa() {
    let leading_only = render(&rich_text(
        "t",
        Some(icon::plus(Size::Md)),
        None,
        Size::Md,
        Bold(false),
        Orientation::Horizontal,
    ));
    assert!(leading_only.contains(r#"data-icon="plus""#));
    assert!(!leading_only.contains("caret-right"));

    let trailing_only = render(&rich_text(
        "t",
        None,
        Some(icon::caret_right(Size::Md)),
        Size::Md,
        Bold(false),
        Orientation::Horizontal,
    ));
    assert!(trailing_only.contains(r#"data-icon="caret-right""#));
    assert!(!trailing_only.contains("plus"));
}

#[test]
fn xss_regression_label_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let html = render(&rich_text(
        payload,
        None,
        None,
        Size::Md,
        Bold(false),
        Orientation::Horizontal,
    ));

    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn xss_regression_label_with_attribute_breakout_payload_is_escaped() {
    let payload = "\"><img src=x onerror=alert(1)>";
    let html = render(&rich_text(
        payload,
        None,
        None,
        Size::Md,
        Bold(false),
        Orientation::Horizontal,
    ));

    assert!(!html.contains(payload));
    assert!(html.contains("&quot;"));
}

/// アイコンスロットは仕様上 `aria-hidden="true"`/`focusable="false"`/
/// `data-icon` を持つため、Annotation の全文一括禁止テストをそのまま
/// 流用すると偽陽性になる。ルート要素の開始タグ（`class` のみ）を
/// 個別に固定することで、スロットありの描画でも非対話制約を検証する。
#[test]
fn root_start_tag_has_only_class_attribute_even_with_slots() {
    let html = render(&rich_text(
        "t",
        Some(icon::plus(Size::Md)),
        Some(icon::caret_right(Size::Md)),
        Size::Md,
        Bold(true),
        Orientation::Vertical,
    ));

    let start_tag_end = html
        .find('>')
        .expect("root element should have a start tag");
    let start_tag = &html[..=start_tag_end];
    assert!(start_tag.starts_with("<div class=\""));
    assert!(start_tag.ends_with("\">"));
    assert_eq!(
        start_tag.matches('"').count(),
        2,
        "unexpected attribute in {start_tag:?}"
    );
}

#[test]
fn output_without_slots_has_no_interactive_semantics_or_style_or_data_attributes() {
    let html = render(&rich_text(
        "t",
        None,
        None,
        Size::Md,
        Bold(true),
        Orientation::Vertical,
    ));
    for forbidden in [
        " role=\"",
        " aria-",
        " tabindex=\"",
        " style=\"",
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
fn rich_text_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::rich_text::RICH_TEXT_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::rich_text::RICH_TEXT_CSS));
}

#[test]
fn rich_text_css_declares_the_four_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::rich_text::RICH_TEXT_CSS;
    for selector in [
        ".fw-wire-rich-text {",
        ".fw-wire-rich-text-label {",
        ".fw-wire-rich-text.fw-wire-bold {",
        ".fw-wire-rich-text.fw-wire-vertical {",
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
