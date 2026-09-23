//! `spinner` 部品の契約テスト（イシュー #2649）。
//!
//! `spinner` は `&str` 引数を一切持たない（`size: Size` のみ）ため、XSS
//! 回帰テストの対象になる入力そのものが存在しない。この構造的な充足は
//! `slider`（イシュー #2628、`crates/wireframe-ui/tests/slider.rs` 冒頭
//! doc「label 引数を発明してテストを追加することはしない」）の先例に
//! 倣い、次の 3 点で代わりに固定する。
//!
//! - (a) 出力する属性がすべて `&'static str` リテラル由来であること
//!   （`renders_root_class_for_every_size`）
//! - (b) `Size` 5 段それぞれで `render()` の出力が期待 HTML 文字列と
//!   完全一致すること（決定性、`full_render_matches_exactly_for_every_size`）
//! - (c) forbidden-substring テスト（`divider` と同型、
//!   `output_has_no_interactive_semantics_or_style_or_data_attributes`）

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{spinner, wireframe_css, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = spinner(size);
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-spinner {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<span"));
        assert!(html.trim_end().ends_with("</span>"));
    }
}

#[test]
fn full_render_matches_exactly_for_every_size() {
    for size in Size::ALL {
        let html = render(&spinner(size));
        let expected = format!(r#"<span class="fw-wire-spinner {}"></span>"#, size.class());
        assert_eq!(html, expected);
    }
}

#[test]
fn output_has_no_interactive_semantics_or_style_or_data_attributes() {
    let html = render(&spinner(Size::Md));
    let forbidden = [
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
        "<svg",
    ];
    for needle in forbidden {
        assert!(
            !html.contains(needle),
            "expected {html:?} to not contain {needle:?}"
        );
    }
}

#[test]
fn spinner_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::spinner::SPINNER_CSS)
        .count();
    assert_eq!(occurrences, 1);
    assert!(wireframe_css().contains(fandhe_frontend_wireframe_ui::spinner::SPINNER_CSS));
}

#[test]
fn spinner_css_declares_expected_selectors_with_fw_wire_prefix_only() {
    let css = fandhe_frontend_wireframe_ui::spinner::SPINNER_CSS;
    assert!(css.contains(".fw-wire-spinner {"));

    for line in css.lines() {
        let trimmed = line.trim();
        if trimmed.ends_with('{') {
            let selector = trimmed.trim_end_matches('{').trim();
            assert!(
                selector.starts_with(".fw-wire-"),
                "unexpected selector {selector:?} in spinner CSS"
            );
        }
    }

    assert!(!css.contains("--fandhe-"));
    assert!(!css.contains(" fd-"));
    assert!(!css.contains("@keyframes"));
    assert!(!css.contains("animation"));
    assert!(css.contains("var(--fw-wire-control-size)"));
    assert!(css.contains("var(--fw-wire-line-width)"));
}
