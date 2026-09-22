//! `fandhe-frontend-wireframe-ui` 共通基盤 API の契約テスト（イシュー #2605）。
//!
//! `Size`・共通型（[`Bold`]/[`Primary`]/[`Active`]/[`Disabled`]/[`Orientation`]）・
//! `class_list`・トークン表・`wireframe_css()` の出力契約と、core 経由の
//! 描画（既定エスケープ）を固定する。

use fandhe_frontend_core::{el_owned, render, text};
use fandhe_frontend_wireframe_ui::{
    class_list, size::css as size_css, tokens::css as tokens_css, tokens::TOKENS, wireframe_css,
    Active, Bold, Disabled, Orientation, Primary, Size, CLASS_PREFIX, PARTS,
};

#[test]
fn size_all_has_five_variants_in_order_with_md_default() {
    assert_eq!(
        Size::ALL,
        [Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl]
    );
    assert_eq!(Size::default(), Size::Md);
}

#[test]
fn size_as_str_matches_pre_styled_ui_literals() {
    let values: Vec<&str> = Size::ALL.iter().map(|s| s.as_str()).collect();
    assert_eq!(values, ["xs", "sm", "md", "lg", "xl"]);
}

#[test]
fn size_class_is_prefixed_and_matches_as_str() {
    for size in Size::ALL {
        let expected = format!("{CLASS_PREFIX}size-{}", size.as_str());
        assert_eq!(size.class(), expected);
        assert!(size.class().starts_with(CLASS_PREFIX));
    }
}

#[test]
fn bold_primary_class_reflect_bool() {
    assert_eq!(Bold(true).class(), Some("fw-wire-bold"));
    assert_eq!(Bold(false).class(), None);
    assert_eq!(Primary(true).class(), Some("fw-wire-primary"));
    assert_eq!(Primary(false).class(), None);
    assert_eq!(Bold::default(), Bold(false));
    assert_eq!(Primary::default(), Primary(false));
}

#[test]
fn active_disabled_attr_reflect_bool() {
    assert_eq!(
        Active(true).attr(),
        Some(("data-active".to_string(), String::new()))
    );
    assert_eq!(Active(false).attr(), None);
    assert_eq!(
        Disabled(true).attr(),
        Some(("data-disabled".to_string(), String::new()))
    );
    assert_eq!(Disabled(false).attr(), None);
    assert_eq!(Active::default(), Active(false));
    assert_eq!(Disabled::default(), Disabled(false));
}

#[test]
fn orientation_default_is_horizontal_and_class_matches() {
    assert_eq!(Orientation::default(), Orientation::Horizontal);
    assert_eq!(Orientation::Horizontal.class(), "fw-wire-horizontal");
    assert_eq!(Orientation::Vertical.class(), "fw-wire-vertical");
}

#[test]
fn from_bool_impls_round_trip() {
    assert_eq!(Bold::from(true), Bold(true));
    assert_eq!(Primary::from(true), Primary(true));
    assert_eq!(Active::from(true), Active(true));
    assert_eq!(Disabled::from(true), Disabled(true));
}

#[test]
fn class_list_joins_only_some_modifiers_without_extra_spaces() {
    let value = class_list(
        "fw-wire-button",
        &[Some("fw-wire-size-md"), None, Some("fw-wire-bold")],
    );
    assert_eq!(value, "fw-wire-button fw-wire-size-md fw-wire-bold");
}

#[test]
fn class_list_with_no_modifiers_is_base_only() {
    assert_eq!(class_list("fw-wire-button", &[]), "fw-wire-button");
    assert_eq!(
        class_list("fw-wire-button", &[None, None]),
        "fw-wire-button"
    );
}

#[test]
fn tokens_css_declares_all_tokens_in_order() {
    let css = tokens_css();
    let lines: Vec<&str> = css.lines().collect();
    assert_eq!(lines.len(), TOKENS.len() + 2);
    assert_eq!(lines.first(), Some(&":root {"));
    assert_eq!(lines.last(), Some(&"}"));
    for (idx, (name, value)) in TOKENS.iter().enumerate() {
        let expected = format!("  --fw-wire-{name}: {value};");
        assert_eq!(lines[idx + 1], expected);
    }
}

#[test]
fn wireframe_css_is_stable_nonempty_and_starts_with_root() {
    let first = wireframe_css();
    let second = wireframe_css();
    assert!(std::ptr::eq(first, second));
    assert!(!first.is_empty());
    assert!(first.starts_with(":root {"));
}

#[test]
fn wireframe_css_contains_all_tokens_and_size_classes() {
    let css = wireframe_css();
    for (name, _) in TOKENS {
        assert!(
            css.contains(&format!("--fw-wire-{name}:")),
            "missing token {name}"
        );
    }
    for size in Size::ALL {
        assert!(
            css.contains(&format!(".{} {{", size.class())),
            "missing size class rule for {:?}",
            size
        );
    }
}

#[test]
fn wireframe_css_does_not_leak_pre_styled_ui_prefixes() {
    let css = wireframe_css();
    assert!(!css.contains("--fandhe-"));
    // `fd-` は pre-styled-ui の class プレフィックスであり本 CSS には現れない。
    assert!(!css.contains(" fd-"));
}

#[test]
fn wireframe_css_selectors_all_use_fw_wire_prefix() {
    let css = wireframe_css();
    for line in css.lines() {
        if let Some(rest) = line.strip_prefix('.') {
            assert!(
                rest.starts_with("fw-wire-"),
                "selector line does not use fw-wire- prefix: {line}"
            );
        }
    }
}

#[test]
fn parts_has_no_duplicate_entries() {
    let mut seen = std::collections::HashSet::new();
    for part in PARTS {
        assert!(seen.insert(*part), "duplicate PARTS entry");
    }
}

#[test]
fn size_css_has_five_scoped_rules() {
    let css = size_css();
    assert_eq!(css.matches("--fw-wire-font-size").count(), 5);
    assert_eq!(css.matches("--fw-wire-control-size").count(), 5);
}

#[test]
fn core_render_escapes_text_and_applies_class_list_and_data_attrs() {
    let attrs: Vec<(String, String)> = vec![
        Some((
            "class".to_string(),
            class_list("fw-wire-card", &[Some(Size::Md.class())]),
        )),
        Active(true).attr(),
    ]
    .into_iter()
    .flatten()
    .collect();

    let node = el_owned("div", attrs, vec![text("<b>x</b>")]);
    let html = render(&node);

    assert!(html.contains(r#"class="fw-wire-card fw-wire-size-md""#));
    assert!(html.contains(r#"data-active="""#));
    assert!(html.contains("&lt;b&gt;x&lt;/b&gt;"));
    assert!(!html.contains("<b>x</b>"));
}
