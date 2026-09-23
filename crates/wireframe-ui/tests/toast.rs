//! `toast` 部品の契約テスト（イシュー #2647）。
//!
//! `crates/wireframe-ui/tests/file_drop.rs`・`link.rs` と同型の観点
//! （アイコンスロットの透過・閉じるグリフの固定パート・非対話制約・
//! XSS 回帰・CSS 配線）を wireframe-ui 側で単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{icon, toast, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = toast("通知本文", None, false, size);
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-toast {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn message_is_always_rendered() {
    let html = render(&toast("通知本文", None, false, Size::Md));
    assert!(html.contains(r#"class="fw-wire-toast-message""#));
    assert!(html.contains("通知本文"));
}

#[test]
fn icon_some_passes_slot_through_and_none_omits_icon_part() {
    let with_icon = render(&toast(
        "通知本文",
        Some(icon::bell(Size::Md)),
        false,
        Size::Md,
    ));
    assert!(with_icon.contains(r#"class="fw-wire-toast-icon""#));
    assert!(with_icon.contains("<svg"));

    let without_icon = render(&toast("通知本文", None, false, Size::Md));
    assert!(!without_icon.contains("fw-wire-toast-icon"));
    assert!(!without_icon.contains("<svg"));
}

#[test]
fn dismissible_true_renders_x_glyph_and_false_omits_part() {
    let dismissible = render(&toast("通知本文", None, true, Size::Md));
    assert!(dismissible.contains(r#"class="fw-wire-toast-dismiss""#));
    assert!(dismissible.contains(r#"data-icon="x""#));

    let not_dismissible = render(&toast("通知本文", None, false, Size::Md));
    assert!(!not_dismissible.contains("fw-wire-toast-dismiss"));
    assert!(!not_dismissible.contains(r#"data-icon="x""#));
}

#[test]
fn dom_order_is_icon_message_dismiss() {
    let html = render(&toast(
        "通知本文",
        Some(icon::bell(Size::Md)),
        true,
        Size::Md,
    ));
    let icon_pos = html.find("fw-wire-toast-icon").expect("icon part missing");
    let message_pos = html
        .find("fw-wire-toast-message")
        .expect("message part missing");
    let dismiss_pos = html
        .find("fw-wire-toast-dismiss")
        .expect("dismiss part missing");
    assert!(icon_pos < message_pos);
    assert!(message_pos < dismiss_pos);
}

#[test]
fn xss_regression_message_is_escaped() {
    let payload = "<script>alert(1)</script>";
    let payload_attr = "\"><img src=x onerror=alert(1)>";

    let html = render(&toast(payload, None, false, Size::Md));
    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));

    let html_attr = render(&toast(payload_attr, None, false, Size::Md));
    assert!(!html_attr.contains(payload_attr));
    assert!(html_attr.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics() {
    // icon スロット・閉じるパートはいずれも省略する: icon::x/アイコン
    // スロットが持つ aria-hidden（アイコン基盤側の既定属性）を「部品が
    // 付与した interactive semantics」と誤検知しないため
    // （root_open_tag_has_only_class_regardless_of_slots が icon あり・
    // dismissible あり構成でもルート開始タグにこれらが付かないことを
    // 別途固定する）。
    let html = render(&toast("通知本文", None, false, Size::Md));
    for forbidden in [
        " role=\"",
        " aria-",
        " tabindex=\"",
        " style=\"",
        " onclick",
        " onerror",
        "<button",
        "<a ",
        "<input",
        "<output",
        "aria-live",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }
}

#[test]
fn root_open_tag_has_only_class_regardless_of_slots() {
    let html = render(&toast(
        "通知本文",
        Some(icon::bell(Size::Md)),
        true,
        Size::Md,
    ));

    let root_open_tag_end = html.find('>').expect("root open tag should exist");
    let root_open_tag = &html[..=root_open_tag_end];
    assert!(root_open_tag.starts_with("<div class=\"fw-wire-toast"));
    assert!(!root_open_tag.contains("data-"));
    assert!(!root_open_tag.contains(" role=\""));
    assert!(!root_open_tag.contains(" aria-"));
    assert!(!root_open_tag.contains(" tabindex=\""));
    assert!(!root_open_tag.contains(" style=\""));
}

#[test]
fn root_has_no_component_data_attributes() {
    let html = render(&toast(
        "通知本文",
        Some(icon::bell(Size::Md)),
        true,
        Size::Md,
    ));
    assert!(!html.contains("data-active"));
    assert!(!html.contains("data-disabled"));
    // data-icon はアイコン基盤側の識別子として透過するので許容する。
    assert!(html.contains("data-icon"));
}

#[test]
fn toast_css_is_registered_in_parts_and_prefixed() {
    assert!(PARTS.contains(&fandhe_frontend_wireframe_ui::toast::TOAST_CSS));

    let css = fandhe_frontend_wireframe_ui::wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::toast::TOAST_CSS));
    assert!(css.contains(".fw-wire-toast"));
    assert!(!fandhe_frontend_wireframe_ui::toast::TOAST_CSS.contains("position: fixed"));
    assert!(!fandhe_frontend_wireframe_ui::toast::TOAST_CSS.contains("position: absolute"));

    for line in fandhe_frontend_wireframe_ui::toast::TOAST_CSS.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('.') {
            assert!(
                trimmed.starts_with(".fw-wire-"),
                "selector line should start with .fw-wire-: {line:?}"
            );
        }
    }
}
