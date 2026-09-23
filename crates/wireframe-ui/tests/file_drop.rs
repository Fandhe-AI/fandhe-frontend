//! `file_drop` 部品の契約テスト（イシュー #2633）。
//!
//! `crates/wireframe-ui/tests/link.rs`・`question.rs` と同型の観点
//! （アイコンスロットの透過・非対話制約・XSS 回帰・CSS 配線）を
//! wireframe-ui 側で単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{file_drop, icon, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = file_drop("ここにファイルをドロップ", None, None, size);
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-file-drop {}""#, size.class());
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
    let html = render(&file_drop("説明文", None, None, Size::Md));
    assert!(html.contains(r#"class="fw-wire-file-drop-label""#));
    assert!(html.contains("説明文"));
}

#[test]
fn hint_some_renders_part_and_none_omits_it_entirely() {
    let with_hint = render(&file_drop("説明文", Some("補助文言"), None, Size::Md));
    assert!(with_hint.contains(r#"class="fw-wire-file-drop-hint""#));
    assert!(with_hint.contains("補助文言"));

    let without_hint = render(&file_drop("説明文", None, None, Size::Md));
    assert!(!without_hint.contains("fw-wire-file-drop-hint"));
}

#[test]
fn icon_some_passes_slot_through_and_none_omits_icon_part() {
    let with_icon = render(&file_drop(
        "説明文",
        None,
        Some(icon::image(Size::Md)),
        Size::Md,
    ));
    assert!(with_icon.contains(r#"class="fw-wire-file-drop-icon""#));
    assert!(with_icon.contains("<svg"));

    let without_icon = render(&file_drop("説明文", None, None, Size::Md));
    assert!(!without_icon.contains("fw-wire-file-drop-icon"));
    assert!(!without_icon.contains("<svg"));
}

#[test]
fn xss_regression_label_and_hint_are_escaped() {
    let payload_label = "<script>alert(1)</script>";
    let payload_hint = "\"><img src=x onerror=alert(1)>";
    let html = render(&file_drop(
        payload_label,
        Some(payload_hint),
        None,
        Size::Md,
    ));

    assert!(!html.contains(payload_label));
    assert!(!html.contains(payload_hint));
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics() {
    let html = render(&file_drop("説明文", Some("補助文言"), None, Size::Md));
    for forbidden in [
        " role=\"",
        " aria-",
        " tabindex=\"",
        " style=\"",
        "<input",
        "<label",
        "<button",
        "<form",
        "<a ",
        "href=",
        "type=\"file\"",
        "draggable",
        "ondrop",
        "ondragover",
        " onclick=\"",
        "javascript:",
        "data-",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn root_open_tag_has_only_class_when_icon_slot_is_present() {
    let html = render(&file_drop(
        "説明文",
        None,
        Some(icon::image(Size::Md)),
        Size::Md,
    ));

    // アイコン自体が持つ data-icon/aria-hidden はスロット内なので許容する。
    assert!(html.contains("data-icon"));

    let root_open_tag_end = html.find('>').expect("root open tag should exist");
    let root_open_tag = &html[..=root_open_tag_end];
    assert!(root_open_tag.starts_with("<div class=\"fw-wire-file-drop"));
    assert!(!root_open_tag.contains("data-"));
    assert!(!root_open_tag.contains(" role=\""));
    assert!(!root_open_tag.contains(" aria-"));
    assert!(!root_open_tag.contains(" tabindex=\""));
    assert!(!root_open_tag.contains(" style=\""));
}

#[test]
fn file_drop_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::file_drop::FILE_DROP_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = fandhe_frontend_wireframe_ui::wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::file_drop::FILE_DROP_CSS));
}

#[test]
fn file_drop_css_declares_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::file_drop::FILE_DROP_CSS;
    for selector in [
        ".fw-wire-file-drop {",
        ".fw-wire-file-drop-icon {",
        ".fw-wire-file-drop-icon .fw-wire-icon-glyph {",
        ".fw-wire-file-drop-label {",
        ".fw-wire-file-drop-hint {",
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
    assert!(css.contains("dashed"));
}
