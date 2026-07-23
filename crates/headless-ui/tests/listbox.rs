//! Listbox（イシュー #750）の統合テスト。
//!
//! `crates/headless-ui/src/listbox.rs` の inline unit tests がパーツ単体の
//! 属性出力・単一パーツの dispatch/hydration を固定するのに対し、本ファイルは
//! `label + content(item_group(item_group_label, item(item_text, item_indicator)))
//! + value_text` という全体の組み立てにおける data-*/ARIA 対応・single/multiple
//! 双方の dispatch 統合・SSR/hydration 両経路・XSS 回帰をクレート外部から
//! （公開 API のみを使って）固定する（`crates/headless-ui/tests/select.rs` と
//! 同型の観点）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::listbox::{self, Listbox, MultiListbox};
use fandhe_frontend_headless_ui::OpenState;
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

#[test]
fn full_assembly_wires_aria_labelledby_and_all_parts_appear() {
    let label = listbox::label(Some("listbox-label-1"), vec![], vec![text("Fruit")]);

    let item_group_label =
        listbox::item_group_label(Some("listbox-group-label-1"), vec![], vec![text("Citrus")]);
    let item_text = listbox::item_text(Some("listbox-item-text-1"), vec![], vec![text("Orange")]);
    let item_indicator = listbox::item_indicator(OpenState::Open, vec![], vec![text("✓")]);
    let item = listbox::item(
        OpenState::Open,
        false,
        false,
        "orange",
        Some("listbox-item-1"),
        vec![],
        vec![item_text, item_indicator],
    );
    let item_group = listbox::item_group(
        Some("listbox-group-label-1"),
        vec![],
        vec![item_group_label, item],
    );
    let content = listbox::content(
        false,
        Some("listbox-content-1"),
        Some("listbox-label-1"),
        Some("listbox-item-1"),
        vec![],
        vec![item_group],
    );
    let value_text = listbox::value_text(false, vec![], vec![text("Orange")]);
    let root = listbox::root(
        OpenState::Open,
        false,
        vec![],
        vec![label, content, value_text],
    );

    let html = render(&root);

    // anatomy: 全パーツの data-scope/data-part が出現する。
    for part in [
        "root",
        "label",
        "content",
        "item-group",
        "item-group-label",
        "item",
        "item-text",
        "item-indicator",
        "value-text",
    ] {
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "missing part `{part}` in html={html}"
        );
        assert!(html.contains(r#"data-scope="listbox""#));
    }

    // ARIA: content の role=listbox・label 関連付け・activedescendant。
    assert!(html.contains(r#"role="listbox""#));
    assert!(html.contains(r#"aria-labelledby="listbox-label-1""#));
    assert!(html.contains(r#"aria-activedescendant="listbox-item-1""#));
    assert!(!html.contains("aria-multiselectable"));

    // item-group: role=group + aria-labelledby。
    assert!(html.contains(r#"role="group""#));
    assert!(html.contains(r#"aria-labelledby="listbox-group-label-1""#));

    // item: role=option + aria-selected + data-state。
    assert!(html.contains(r#"role="option""#));
    assert!(html.contains(r#"aria-selected="true""#));
    assert!(html.contains(r#"data-state="open""#));
}

#[test]
fn content_multiple_true_outputs_aria_multiselectable_true() {
    let content = listbox::content(true, None, None, None, vec![], vec![]);
    let html = render(&content);
    assert!(html.contains(r#"aria-multiselectable="true""#));
}

#[test]
fn item_disabled_pairs_aria_disabled_with_data_disabled() {
    let item = listbox::item(
        OpenState::Closed,
        true,
        false,
        "banana",
        None,
        vec![],
        vec![],
    );
    let html = render(&item);
    assert!(html.contains(r#"aria-disabled="true""#));
    assert!(html.contains(r#"data-disabled="""#));
}

// --- Listbox（single モード）: dispatch/hydration 統合 ---

#[test]
fn listbox_dispatch_select_updates_selection_and_renders_selected_item() {
    let mut l = Listbox::default();
    assert!(dispatch(&mut l, "select", "apple"));
    assert_eq!(l.selected(), Some("apple"));

    let html = render(&l.item("apple", false, false, None, vec![], vec![]));
    assert!(html.contains(r#"aria-selected="true""#));
    assert!(html.contains(r#"data-state="open""#));

    let other_html = render(&l.item("banana", false, false, None, vec![], vec![]));
    assert!(other_html.contains(r#"aria-selected="false""#));
}

#[test]
fn listbox_ssr_hydration_round_trip() {
    let mut l = Listbox::default();
    assert!(dispatch(&mut l, "select", "apple"));

    let hydrate_html = render(&render_for_hydration(&l));
    assert!(hydrate_html.contains("data-hydrate-selected"));

    let restored = Listbox::from_hydration_attrs(&l.hydration_attrs()).unwrap();
    assert_eq!(restored.selected(), Some("apple"));
}

#[test]
fn listbox_hydration_tampered_multi_value_list_is_rejected() {
    use fandhe_frontend_interactive::codec;
    let bogus = codec::encode_list(&["apple".to_string(), "banana".to_string()]);
    let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
    let err = Listbox::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

// --- MultiListbox（multiple モード）: dispatch/hydration 統合 ---

#[test]
fn multi_listbox_dispatch_select_allows_multiple_simultaneous_selections() {
    let mut m = MultiListbox::default();
    assert!(dispatch(&mut m, "select", "apple"));
    assert!(dispatch(&mut m, "select", "banana"));
    assert_eq!(m.selected(), &["apple".to_string(), "banana".to_string()]);

    let content = listbox::content(true, None, None, None, vec![], vec![]);
    assert!(render(&content).contains(r#"aria-multiselectable="true""#));

    let apple_html = render(&m.item("apple", false, false, None, vec![], vec![]));
    assert!(apple_html.contains(r#"aria-selected="true""#));
    let banana_html = render(&m.item("banana", false, false, None, vec![], vec![]));
    assert!(banana_html.contains(r#"aria-selected="true""#));
}

#[test]
fn multi_listbox_ssr_hydration_round_trip() {
    let mut m = MultiListbox::default();
    assert!(dispatch(&mut m, "select", "apple"));
    assert!(dispatch(&mut m, "select", "banana"));

    let restored = MultiListbox::from_hydration_attrs(&m.hydration_attrs()).unwrap();
    assert_eq!(
        restored.selected(),
        &["apple".to_string(), "banana".to_string()]
    );
}

#[test]
fn multi_listbox_hydration_tampered_duplicate_value_list_is_rejected() {
    use fandhe_frontend_interactive::codec;
    let bogus = codec::encode_list(&["apple".to_string(), "apple".to_string()]);
    let attrs = vec![("data-hydrate-selected".to_string(), bogus)];
    let err = MultiListbox::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

// --- XSS 回帰: item テキスト・値・id・labelledby・hydration dispatch payload ---

#[test]
fn dynamic_values_across_all_parts_are_escaped() {
    let payload = "\"><script>alert(1)</script>";

    let label = listbox::label(Some(payload), vec![], vec![text(payload)]);
    let content = listbox::content(
        false,
        Some(payload),
        Some(payload),
        Some(payload),
        vec![],
        vec![],
    );
    let item_group = listbox::item_group(Some(payload), vec![], vec![]);
    let item_group_label = listbox::item_group_label(Some(payload), vec![], vec![text(payload)]);
    let item = listbox::item(
        OpenState::Open,
        false,
        false,
        payload,
        Some(payload),
        vec![],
        vec![],
    );
    let item_text = listbox::item_text(Some(payload), vec![], vec![text(payload)]);
    let value_text = listbox::value_text(false, vec![], vec![text(payload)]);

    for node in [
        label,
        content,
        item_group,
        item_group_label,
        item,
        item_text,
        value_text,
    ] {
        let html = render(&node);
        assert!(!html.contains("<script>alert(1)</script>"), "html={html}");
        assert!(!html.contains(r#""><script"#), "html={html}");
    }
}

#[test]
fn dispatch_payload_is_escaped_when_rendered_via_hydration_attrs() {
    let mut l = Listbox::default();
    let payload = "\"><script>alert(1)</script>";
    assert!(dispatch(&mut l, "select", payload));

    let rendered = render(&render_for_hydration(&l));
    assert!(rendered.contains("data-hydrate-selected="));
    assert!(rendered.contains("&lt;script&gt;"));
    assert!(!rendered.contains("<script>alert(1)</script>"));
    assert!(!rendered.contains(r#""><script"#));
}
