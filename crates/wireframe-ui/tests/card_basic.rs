//! `card_basic` 部品の契約テスト（イシュー #2658）。
//!
//! `crates/wireframe-ui/tests/avatar.rs`・`crates/wireframe-ui/tests/nav_item.rs`
//! と同型の観点（サイズ全段・スロット省略/併用・XSS 回帰・非対話制約・
//! CSS 配線）を単体固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{avatar, card_basic, icon, wireframe_css, Size, PARTS};

#[test]
fn renders_root_class_for_every_size() {
    for size in Size::ALL {
        let node = card_basic("primary", None, None, None, size);
        let html = render(&node);
        let expected_class = format!(r#"class="fw-wire-card-basic {}""#, size.class());
        assert!(
            html.contains(&expected_class),
            "expected {expected_class:?} in {html:?}"
        );
        assert!(html.starts_with("<div"));
        assert!(html.trim_end().ends_with("</div>"));
    }
}

#[test]
fn secondary_none_omits_element_and_some_renders_it() {
    let without = render(&card_basic("primary", None, None, None, Size::Md));
    assert!(!without.contains("fw-wire-card-basic-secondary"));

    let with = render(&card_basic(
        "primary",
        Some("secondary"),
        None,
        None,
        Size::Md,
    ));
    assert!(with.contains(r#"class="fw-wire-card-basic-secondary""#));
    assert!(with.contains("secondary"));
}

#[test]
fn leading_and_trailing_none_omit_svg_and_some_render_icon_and_ordering() {
    let minimal = render(&card_basic("primary", None, None, None, Size::Md));
    assert!(!minimal.contains("<svg"));

    let with_slots = render(&card_basic(
        "山田太郎",
        Some("エンジニア"),
        Some(avatar(None, Size::Md, true)),
        Some(icon::ellipsis(Size::Md)),
        Size::Md,
    ));
    assert_eq!(with_slots.matches(r#"data-icon="user""#).count(), 1);
    assert_eq!(with_slots.matches(r#"data-icon="ellipsis""#).count(), 1);

    // 出力順（leading → body → trailing）: avatar のルート class が
    // primary より前、ellipsis の data-icon が primary より後に現れる。
    let leading_pos = with_slots
        .find("fw-wire-avatar")
        .expect("leading avatar must be present");
    let primary_pos = with_slots
        .find("fw-wire-card-basic-primary")
        .expect("primary must be present");
    let trailing_pos = with_slots
        .find(r#"data-icon="ellipsis""#)
        .expect("trailing icon must be present");
    assert!(leading_pos < primary_pos);
    assert!(primary_pos < trailing_pos);
}

#[test]
fn xss_regression_primary_and_secondary_are_escaped() {
    let payload_a = "<script>alert(1)</script>";
    let payload_b = "\"><img src=x onerror=alert(1)>";

    let html_a = render(&card_basic(
        payload_a,
        Some(payload_a),
        None,
        None,
        Size::Md,
    ));
    assert!(!html_a.contains(payload_a));
    assert!(html_a.contains("&lt;script&gt;"));

    let html_b = render(&card_basic(
        payload_b,
        Some(payload_b),
        None,
        None,
        Size::Md,
    ));
    assert!(!html_b.contains(payload_b));
    assert!(html_b.contains("&quot;"));
}

#[test]
fn output_has_no_interactive_semantics_or_style_or_media() {
    let html = render(&card_basic(
        "primary",
        Some("secondary"),
        Some(avatar(None, Size::Md, false)),
        Some(icon::ellipsis(Size::Md)),
        Size::Md,
    ));
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
    assert!(!html.contains("data-active"));
    assert!(!html.contains("data-disabled"));
}

#[test]
fn card_basic_css_is_registered_exactly_once_in_parts_and_in_aggregate_css() {
    let occurrences = PARTS
        .iter()
        .filter(|part| **part == fandhe_frontend_wireframe_ui::card_basic::CARD_BASIC_CSS)
        .count();
    assert_eq!(occurrences, 1);

    let css = wireframe_css();
    assert!(css.contains(fandhe_frontend_wireframe_ui::card_basic::CARD_BASIC_CSS));
}

#[test]
fn card_basic_css_selectors_use_fw_wire_prefix_and_reference_tokens_not_literals() {
    let css = fandhe_frontend_wireframe_ui::card_basic::CARD_BASIC_CSS;
    for line in css.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('.') {
            assert!(
                trimmed.starts_with(".fw-wire-"),
                "selector line should start with .fw-wire-: {line:?}"
            );
        }
    }

    assert!(css.contains("var(--fw-wire-line)"));
    assert!(!css.contains("--fandhe-"));
}
