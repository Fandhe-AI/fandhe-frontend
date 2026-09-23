//! `avatar` 部品の契約テスト（イシュー #2651）。
//!
//! `crates/wireframe-ui/tests/frame.rs`・`crates/wireframe-ui/tests/link.rs`
//! と同型の観点（非対話制約・XSS 回帰・CSS 配線）を単体固定する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_wireframe_ui::{avatar, icon, wireframe_css, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = avatar(None, size, false);
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-avatar {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn circle_true_appends_circle_class_and_false_omits_it() {
    let circular = render(&avatar(None, Size::Md, true));
    assert!(circular.contains(r#"class="fw-wire-avatar fw-wire-size-md fw-wire-avatar-circle""#));

    let square = render(&avatar(None, Size::Md, false));
    assert!(!square.contains("fw-wire-avatar-circle"));
}

#[test]
fn none_content_falls_back_to_user_glyph() {
    let html = render(&avatar(None, Size::Md, false));
    assert_eq!(html.matches(r#"data-icon="user""#).count(), 1);
    assert!(html.contains("<svg"));
}

#[test]
fn some_content_replaces_default_glyph() {
    let with_image = render(&avatar(Some(icon::image(Size::Md)), Size::Md, false));
    assert!(with_image.contains(r#"data-icon="image""#));
    assert!(!with_image.contains(r#"data-icon="user""#));

    let with_text = render(&avatar(Some(text("AB")), Size::Md, false));
    assert!(!with_text.contains("<svg"));
    assert!(with_text.contains("AB"));
}

#[test]
fn xss_regression_slot_text_is_escaped() {
    let payload_a = "<script>alert(1)</script>";
    let payload_b = "\"><img src=x onerror=alert(1)>";
    let html_a = render(&avatar(Some(text(payload_a)), Size::Md, false));
    let html_b = render(&avatar(Some(text(payload_b)), Size::Md, false));

    assert!(!html_a.contains(payload_a));
    assert!(html_a.contains("&lt;script&gt;"));
    assert!(!html_b.contains(payload_b));
    assert!(html_b.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style_or_media() {
    let html = render(&avatar(Some(text("AB")), Size::Md, true));
    for forbidden in [
        " role=\"",
        " aria-expanded",
        " tabindex=\"",
        " style=\"",
        "href=",
        "src=",
        "<img",
        "<a ",
        "<button",
        " onclick=\"",
        " onerror=\"",
        "javascript:",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden:?} in {html:?}"
        );
    }

    // 既定（None）フォールバックの `icon::user` は装飾用途の `aria-hidden`
    // を持つ（`crate::icon` の出力契約）。対話的 ARIA ではないため許容し、
    // 上の判定対象からは区別する。
    let default_html = render(&avatar(None, Size::Md, false));
    assert!(default_html.contains(r#"aria-hidden="true""#));
    assert!(!default_html.contains(" aria-expanded"));
}

#[test]
fn avatar_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::avatar::AVATAR_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::avatar::AVATAR_CSS));
}

#[test]
fn avatar_css_selectors_use_fw_wire_prefix_and_reference_tokens_not_literals() {
    let css = fandhe_frontend_wireframe_ui::avatar::AVATAR_CSS;
    for line in css.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('.') {
            assert!(
                trimmed.starts_with(".fw-wire-"),
                "selector line should start with .fw-wire-: {line:?}"
            );
        }
    }

    assert!(css.contains("var(--fw-wire-control-size"));
    assert!(!css.contains("--fandhe-"));

    // circle=true のセレクタにだけ border-radius: 50% が現れる。
    let circle_rule_start = css
        .find(".fw-wire-avatar.fw-wire-avatar-circle")
        .expect("circle selector must exist");
    let circle_rule_end = css[circle_rule_start..]
        .find('}')
        .map(|end| circle_rule_start + end)
        .expect("circle rule must be closed");
    assert!(css[circle_rule_start..circle_rule_end].contains("border-radius: 50%"));

    let root_rule_end = css
        .find(".fw-wire-avatar.fw-wire-avatar-circle")
        .expect("circle selector must exist");
    assert!(!css[..root_rule_end].contains("50%"));

    // Size の段階値（例: 2rem 以外の rem リテラル）を直書きしていない
    // ことの弱い保証として、`size::SCALE` の control_size 値を文字列
    // として含まないことを確認する。
    for (_, _, control_size) in fandhe_frontend_wireframe_ui::size::css()
        .lines()
        .filter_map(|line| {
            let marker = "--fw-wire-control-size: ";
            let start = line.find(marker)? + marker.len();
            let end = line[start..].find(';')? + start;
            Some(((), (), line[start..end].to_string()))
        })
    {
        // 唯一の許容参照は `var(--fw-wire-control-size, 2rem)` の
        // フォールバック値であり、これは Md 段階の control_size と
        // 一致する仕様上の例外（CSS カスタムプロパティ未定義時の
        // フォールバック値）。他段階の値が直書きされていないことを
        // 確認する。
        if control_size != "2rem" {
            assert!(
                !css.contains(&control_size),
                "unexpected literal control_size {control_size:?} in AVATAR_CSS"
            );
        }
    }
}
