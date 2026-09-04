//! Listbox（イシュー #750）の統合テスト。
//!
//! `crates/headless-ui/src/listbox.rs` の inline unit tests がパーツ単体の
//! 属性出力・単一パーツの dispatch/hydration を固定するのに対し、本ファイルは
//! `label`・`content(item_group(item_group_label, item(item_text, item_indicator)))`・
//! `value_text` を組み合わせた全体の組み立てにおける data-*/ARIA 対応・single/multiple
//! 双方の dispatch 統合・SSR/hydration 両経路・XSS 回帰をクレート外部から
//! （公開 API のみを使って）固定する（`crates/headless-ui/tests/select.rs` と
//! 同型の観点）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::listbox::{self, Listbox, ListboxProps, MultiListbox};
use fandhe_frontend_headless_ui::OpenState;
use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate, HydrateError};

#[test]
fn full_assembly_wires_aria_labelledby_and_all_parts_appear() {
    let props = ListboxProps::default();
    let label = listbox::label(&props, Some("listbox-label-1"), vec![], vec![text("Fruit")]);

    let item_group_label =
        listbox::item_group_label(Some("listbox-group-label-1"), vec![], vec![text("Citrus")]);
    let item_text = listbox::item_text(
        OpenState::Open,
        false,
        false,
        Some("listbox-item-text-1"),
        vec![],
        vec![text("Orange")],
    );
    let item_indicator = listbox::item_indicator(OpenState::Open, vec![], vec![text("✓")]);
    let item = listbox::item(
        OpenState::Open,
        &props,
        false,
        false,
        "orange",
        Some("listbox-item-1"),
        vec![],
        vec![item_text, item_indicator],
    );
    let item_group = listbox::item_group(
        &props,
        Some("listbox-group-label-1"),
        vec![],
        vec![item_group_label, item],
    );
    let content = listbox::content(
        false,
        &props,
        Some("listbox-content-1"),
        Some("listbox-label-1"),
        Some("listbox-item-1"),
        vec![],
        vec![item_group],
    );
    let value_text = listbox::value_text(false, &props, vec![], vec![text("Orange")]);
    let root = listbox::root(
        OpenState::Open,
        &props,
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
    let content = listbox::content(
        true,
        &ListboxProps::default(),
        None,
        None,
        None,
        vec![],
        vec![],
    );
    let html = render(&content);
    assert!(html.contains(r#"aria-multiselectable="true""#));
}

#[test]
fn item_disabled_pairs_aria_disabled_with_data_disabled() {
    let item = listbox::item(
        OpenState::Closed,
        &ListboxProps::default(),
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

#[test]
fn root_disabled_propagates_to_item_and_orientation_reflects_content_and_item() {
    // 参照突合（イシュー #1611）: root disabled が item へ伝播し、
    // orientation が content/item の双方へ反映されることをクレート外部
    // から固定する。
    let props = ListboxProps {
        disabled: true,
        orientation: Orientation::Horizontal,
    };
    let content = listbox::content(false, &props, None, None, None, vec![], vec![]);
    assert!(render(&content).contains(r#"data-orientation="horizontal""#));

    let item = listbox::item(
        OpenState::Closed,
        &props,
        false,
        false,
        "banana",
        None,
        vec![],
        vec![],
    );
    let html = render(&item);
    assert!(html.contains(r#"aria-disabled="true""#));
    assert!(html.contains(r#"data-disabled="""#));
    assert!(html.contains(r#"data-orientation="horizontal""#));
}

// --- Listbox（single モード）: dispatch/hydration 統合 ---

#[test]
fn listbox_dispatch_select_updates_selection_and_renders_selected_item() {
    let mut l = Listbox::default();
    let props = ListboxProps::default();
    assert!(dispatch(&mut l, "select", "apple"));
    assert_eq!(l.selected(), Some("apple"));

    let html = render(&l.item("apple", &props, false, false, None, vec![], vec![]));
    assert!(html.contains(r#"aria-selected="true""#));
    assert!(html.contains(r#"data-state="open""#));

    let other_html = render(&l.item("banana", &props, false, false, None, vec![], vec![]));
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

    let props = ListboxProps::default();
    let content = listbox::content(true, &props, None, None, None, vec![], vec![]);
    assert!(render(&content).contains(r#"aria-multiselectable="true""#));

    let apple_html = render(&m.item("apple", &props, false, false, None, vec![], vec![]));
    assert!(apple_html.contains(r#"aria-selected="true""#));
    let banana_html = render(&m.item("banana", &props, false, false, None, vec![], vec![]));
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
    let props = ListboxProps::default();

    let label = listbox::label(&props, Some(payload), vec![], vec![text(payload)]);
    let content = listbox::content(
        false,
        &props,
        Some(payload),
        Some(payload),
        Some(payload),
        vec![],
        vec![],
    );
    let item_group = listbox::item_group(&props, Some(payload), vec![], vec![]);
    let item_group_label = listbox::item_group_label(Some(payload), vec![], vec![text(payload)]);
    let item = listbox::item(
        OpenState::Open,
        &props,
        false,
        false,
        payload,
        Some(payload),
        vec![],
        vec![],
    );
    let item_text = listbox::item_text(
        OpenState::Open,
        false,
        false,
        Some(payload),
        vec![],
        vec![text(payload)],
    );
    let value_text = listbox::value_text(false, &props, vec![], vec![text(payload)]);

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

// ---------------------------------------------------------------------
// アクセシブルネーム経路の契約（イシュー #1067、受け入れ条件 2）
//
// `listbox::content` の `labelledby` 引数は opt-in（`Option`）である。
// アクセシブルネームは `aria-labelledby`（`labelledby` 経由）に加えて
// `aria-label`（呼び出し側 `attrs` 経由）でも成立するため、`labelledby`
// を型で必須化すると正当な代替経路を塞いでしまう（`combobox` の
// `controls`/`activedescendant` と異なり型必須化を採らない理由、
// イシュー #1067 計画 §5）。代わりに「命名経路（`aria-labelledby` の
// 実在参照 または `aria-label` 存在）のどちらかが必ず成立する」ことを
// 契約テストで固定する。
// ---------------------------------------------------------------------

/// `role="listbox"` を持つ要素が、実在する `id` を指す `aria-labelledby`
/// または非空の `aria-label` のいずれかを持つことを検証する
/// （[`crates/headless-ui/tests/combobox.rs`] の `verify_combobox_aria_association`
/// と同じ手書き部分文字列走査。属性値中に生の `"` が現れないという
/// 既定エスケープの不変条件に依拠する点も同一）。
fn verify_listbox_has_accessible_name(html: &str) -> Result<(), String> {
    let tags = extract_open_tags(html);

    let mut ids: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for tag in &tags {
        if let Some(id) = attr(tag, "id") {
            ids.insert(id);
        }
    }

    for tag in &tags {
        if attr(tag, "role") != Some("listbox") {
            continue;
        }
        let labelledby = attr(tag, "aria-labelledby");
        let label = attr(tag, "aria-label");

        match (labelledby, label) {
            (Some(target), _) if ids.contains(target) => {}
            (_, Some(_)) => {}
            (Some(target), None) => {
                return Err(format!(
                    "naming violation: aria-labelledby=\"{target}\" target id does not exist (dangling IDREF) and no aria-label fallback: <{tag}>"
                ));
            }
            (None, None) => {
                return Err(format!(
                    "naming violation: role=\"listbox\" element has neither aria-labelledby nor aria-label: <{tag}>"
                ));
            }
        }
    }

    Ok(())
}

/// [`extract_open_tags`]/[`attr`] は `combobox.rs` 側の実装と意図的な重複
/// （クレート境界を跨ぐ test helper 共有は Rust の統合テストでは不可能）。
/// **規則を変更するときは両方のファイルを更新すること。**
fn extract_open_tags(html: &str) -> Vec<&str> {
    let mut tags = Vec::new();
    let mut rest = html;
    while let Some(lt) = rest.find('<') {
        let after_lt = &rest[lt + 1..];
        if after_lt.starts_with('/') {
            match after_lt.find('>') {
                Some(gt) => {
                    rest = &after_lt[gt + 1..];
                    continue;
                }
                None => break,
            }
        }
        match after_lt.find('>') {
            Some(gt) => {
                tags.push(&after_lt[..gt]);
                rest = &after_lt[gt + 1..];
            }
            None => break,
        }
    }
    tags
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!(" {name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

/// `labelledby` 経由の命名（実在参照）が `Ok(())` になることを固定する。
#[test]
fn verify_listbox_has_accessible_name_ok_via_labelledby() {
    let props = ListboxProps::default();
    let label = listbox::label(&props, Some("lb-label"), vec![], vec![text("Fruit")]);
    let content = listbox::content(false, &props, None, Some("lb-label"), None, vec![], vec![]);
    let html = render(&fandhe_frontend_core::el(
        "div",
        vec![],
        vec![label, content],
    ));

    assert_eq!(verify_listbox_has_accessible_name(&html), Ok(()));
}

/// `aria-label`（呼び出し側 `attrs` 経由）のみで命名が成立する代替経路も
/// `Ok(())` になることを固定する（型必須化を採らない根拠の裏付け）。
#[test]
fn verify_listbox_has_accessible_name_ok_via_aria_label_attrs() {
    let content = listbox::content(
        false,
        &ListboxProps::default(),
        None,
        None,
        None,
        vec![("aria-label", "Fruit")],
        vec![],
    );
    let html = render(&content);

    assert_eq!(verify_listbox_has_accessible_name(&html), Ok(()));
}

/// `labelledby`/`aria-label` のいずれも無いときに検知されることを固定
/// する（opt-in 欠落の検知力の証明）。
#[test]
fn verify_listbox_has_accessible_name_detects_missing_naming() {
    let content = listbox::content(
        false,
        &ListboxProps::default(),
        None,
        None,
        None,
        vec![],
        vec![],
    );
    let html = render(&content);

    let result = verify_listbox_has_accessible_name(&html);
    assert!(result.is_err());
    assert!(result.unwrap_err().starts_with("naming violation"));
}

/// `labelledby` が実在しない `id` を指し、かつ `aria-label` 代替も無い
/// ときに検知されることを固定する（dangling IDREF）。
#[test]
fn verify_listbox_has_accessible_name_detects_dangling_labelledby_without_fallback() {
    let content = listbox::content(
        false,
        &ListboxProps::default(),
        None,
        Some("no-such-id"),
        None,
        vec![],
        vec![],
    );
    let html = render(&content);

    let result = verify_listbox_has_accessible_name(&html);
    assert!(result.is_err());
    assert!(result.unwrap_err().starts_with("naming violation"));
}
