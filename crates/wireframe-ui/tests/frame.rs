//! `frame` 部品の契約テスト（イシュー #2609）。
//!
//! `crates/wireframe-ui/tests/annotation.rs` と同型の観点（非対話制約・
//! XSS 回帰・CSS 配線）を wireframe-ui 側で単体固定する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_wireframe_ui::{frame, wireframe_css, Size, PARTS};

#[test]
fn renders_root_class_for_every_padding_size() {
    for size in Size::ALL {
        let node = frame(vec![], size, false);
        let html = render(&node);
        let expected_class = format!(
            r#"class="fw-wire-frame fw-wire-frame-padding-{}""#,
            size.as_str()
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
fn bordered_true_appends_bordered_class_and_false_omits_it() {
    let with_border = render(&frame(vec![], Size::Md, true));
    assert!(with_border
        .contains(r#"class="fw-wire-frame fw-wire-frame-padding-md fw-wire-frame-bordered""#));

    let without_border = render(&frame(vec![], Size::Md, false));
    assert!(!without_border.contains("fw-wire-frame-bordered"));
}

#[test]
fn children_are_rendered_in_the_given_order_and_empty_vec_leaves_an_empty_element() {
    let node = frame(vec![text("一つ目"), text("二つ目")], Size::Md, false);
    let html = render(&node);
    let pos_first = html.find("一つ目").expect("first child should render");
    let pos_second = html.find("二つ目").expect("second child should render");
    assert!(pos_first < pos_second);

    let empty = render(&frame(vec![], Size::Md, false));
    assert!(empty.contains(r#"class="fw-wire-frame fw-wire-frame-padding-md"></div>"#));
}

#[test]
fn nested_frames_render_both_root_classes() {
    let inner = frame(vec![text("内側")], Size::Sm, true);
    let outer = frame(vec![inner], Size::Lg, false);
    let html = render(&outer);
    assert!(html.contains(r#"class="fw-wire-frame fw-wire-frame-padding-lg""#));
    assert!(
        html.contains(r#"class="fw-wire-frame fw-wire-frame-padding-sm fw-wire-frame-bordered""#)
    );
    assert!(html.contains("内側"));
}

#[test]
fn xss_regression_child_text_is_escaped() {
    let payload_a = "<script>alert(1)</script>";
    let payload_b = "\"><img src=x onerror=alert(1)>";
    let html = render(&frame(
        vec![text(payload_a), text(payload_b)],
        Size::Md,
        true,
    ));

    assert!(!html.contains(payload_a));
    assert!(!html.contains(payload_b));
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style_or_data_attributes() {
    let html = render(&frame(vec![text("子")], Size::Md, true));
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
fn frame_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::frame::FRAME_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::frame::FRAME_CSS));
}

#[test]
fn frame_css_declares_the_seven_selectors_with_fw_wire_prefix_only_and_no_font_size() {
    let css = fandhe_frontend_wireframe_ui::frame::FRAME_CSS;
    for selector in [
        ".fw-wire-frame {",
        ".fw-wire-frame.fw-wire-frame-bordered {",
        ".fw-wire-frame.fw-wire-frame-padding-xs {",
        ".fw-wire-frame.fw-wire-frame-padding-sm {",
        ".fw-wire-frame.fw-wire-frame-padding-md {",
        ".fw-wire-frame.fw-wire-frame-padding-lg {",
        ".fw-wire-frame.fw-wire-frame-padding-xl {",
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
    assert!(!css.contains("font-size"));
}

#[test]
fn padding_class_does_not_share_the_size_scoped_custom_property_class() {
    // コードレビュー指摘（イシュー #2609、PR #2679）の回帰: padding は
    // Frame 専用の `fw-wire-frame-padding-*` class のみで表現し、
    // `crate::size::css` が生成する共有 `fw-wire-size-*`
    // （`--fw-wire-font-size`/`--fw-wire-control-size` を同時定義する）を
    // 経由しない。共有 class を子孫が継承すると、独自の size class を
    // 再宣言しない子部品の寸法が Frame の padding 引数で意図せず変化する
    // （`crate::stack` の同種回帰テストと対をなす）。
    let html = render(&frame(vec![text("a")], Size::Md, true));
    assert!(!html.contains("fw-wire-size-"));

    let css = fandhe_frontend_wireframe_ui::frame::FRAME_CSS;
    assert!(!css.contains("--fw-wire-font-size"));
    assert!(!css.contains("--fw-wire-control-size"));
}
