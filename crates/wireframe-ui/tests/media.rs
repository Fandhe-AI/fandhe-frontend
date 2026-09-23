//! `media` 部品の契約テスト（イシュー #2661）。
//!
//! `crates/wireframe-ui/tests/avatar.rs` と同型の観点（非対話制約・XSS
//! 回帰・CSS 配線）を単体固定する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_wireframe_ui::{icon, media, wireframe_css, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = media(None, size);
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-media {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn none_content_falls_back_to_play_glyph() {
    let html = render(&media(None, Size::Md));
    assert_eq!(html.matches(r#"data-icon="play""#).count(), 1);
    assert!(html.contains("<svg"));
}

#[test]
fn some_content_replaces_default_glyph() {
    let with_image = render(&media(Some(icon::image(Size::Md)), Size::Md));
    assert!(with_image.contains(r#"data-icon="image""#));
    assert!(!with_image.contains(r#"data-icon="play""#));

    let with_text = render(&media(Some(text("REC")), Size::Md));
    assert!(!with_text.contains("<svg"));
    assert!(with_text.contains("REC"));
}

#[test]
fn disc_part_class_appears_exactly_once() {
    let html = render(&media(None, Size::Md));
    assert_eq!(html.matches("fw-wire-media-disc").count(), 1);
}

#[test]
fn xss_regression_slot_text_is_escaped() {
    let payload_a = "<script>alert(1)</script>";
    let payload_b = "\"><img src=x onerror=alert(1)>";
    let html_a = render(&media(Some(text(payload_a)), Size::Md));
    let html_b = render(&media(Some(text(payload_b)), Size::Md));

    assert!(!html_a.contains(payload_a));
    assert!(html_a.contains("&lt;script&gt;"));
    assert!(!html_b.contains(payload_b));
    assert!(html_b.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_style_or_media_elements() {
    let html = render(&media(Some(text("REC")), Size::Md));
    for forbidden in [
        " role=\"",
        " aria-expanded",
        " tabindex=\"",
        " style=\"",
        "href=",
        "src=",
        "poster=",
        "controls",
        "<video",
        "<iframe",
        "<source",
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

    // 既定（None）フォールバックの `icon::play` は装飾用途の
    // `aria-hidden` を持つ（`crate::icon` の出力契約）。対話的 ARIA では
    // ないため許容し、上の判定対象からは区別する。
    let default_html = render(&media(None, Size::Md));
    assert!(default_html.contains(r#"aria-hidden="true""#));
    assert!(!default_html.contains(" aria-expanded"));
}

#[test]
fn media_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::media::MEDIA_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::media::MEDIA_CSS));
}

#[test]
fn media_css_selectors_use_fw_wire_prefix_and_reference_tokens_not_literals() {
    let css = fandhe_frontend_wireframe_ui::media::MEDIA_CSS;
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
    assert!(css.contains("aspect-ratio"));
    assert!(!css.contains("--fandhe-"));
    assert!(!css.contains("@keyframes"));
    assert!(!css.contains("animation"));

    // Size の段階値（例: 2rem 以外の rem リテラル）を直書きしていない
    // ことの弱い保証として、`size::SCALE` の control_size 値を文字列
    // として含まないことを確認する。
    for control_size in fandhe_frontend_wireframe_ui::size::css()
        .lines()
        .filter_map(|line| {
            let marker = "--fw-wire-control-size: ";
            let start = line.find(marker)? + marker.len();
            let end = line[start..].find(';')? + start;
            Some(line[start..end].to_string())
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
                "unexpected literal control_size {control_size:?} in MEDIA_CSS"
            );
        }
    }
}
