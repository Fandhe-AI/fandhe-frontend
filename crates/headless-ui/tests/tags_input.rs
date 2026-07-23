//! TagsInput（イシュー #744）の統合テスト。
//!
//! `crates/headless-ui/src/tags_input.rs` の inline unit tests がパーツ単体の
//! 属性出力・状態機械の遷移を固定するのに対し、本ファイルは「root >
//! label + control(item(item-preview(item-text)) × len + input) +
//! clear-trigger + hidden-input」の組み立て全体の data-*/ARIA 対応・dispatch
//! 統合・SSR/hydration 両経路・XSS 回帰をクレート外部から（公開 API のみを
//! 使って）固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::tags_input::{self, TagsInput};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

fn tags(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

#[test]
fn full_assembly_wires_root_label_control_items_and_hidden_input() {
    let t = TagsInput::new(tags(&["rust", "wasm"]), None);
    let items: Vec<_> = t
        .tags()
        .iter()
        .enumerate()
        .map(|(i, tag)| {
            let preview = tags_input::item_preview(
                false,
                vec![],
                vec![tags_input::item_text(
                    vec![],
                    vec![fandhe_frontend_core::text(tag)],
                )],
            );
            t.item(i, false, vec![], vec![preview])
        })
        .collect();
    let mut control_children = items;
    control_children.push(t.input("", false, vec![]));
    let control = t.control(false, "Tags", vec![], control_children);
    let label = t.label(vec![], vec![fandhe_frontend_core::text("Tags")]);
    let clear = t.clear_trigger(false, vec![], vec![fandhe_frontend_core::text("Clear")]);
    let hidden_input = t.hidden_input("tags", false, vec![]);
    let root = t.root(false, vec![], vec![label, control, clear, hidden_input]);

    let html = render(&root);
    assert!(html.contains(r#"data-scope="tags-input""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="label""#));
    assert!(html.contains(r#"data-part="control""#));
    assert!(html.contains(r#"role="listbox""#));
    assert!(html.contains(r#"data-part="item""#));
    assert!(html.contains(r#"data-part="item-preview""#));
    assert!(html.contains(r#"role="option""#));
    assert!(html.contains(r#"data-part="item-text""#));
    assert!(html.contains(r#"data-part="input""#));
    assert!(html.contains(r#"data-part="clear-trigger""#));
    assert!(html.contains(r#"data-part="hidden-input""#));
    assert!(html.contains("rust"));
    assert!(html.contains("wasm"));
    assert!(html.contains(r#"value="rust,wasm""#));
}

#[test]
fn dispatch_add_remove_clear_flow_via_public_api() {
    let mut t = TagsInput::new(Vec::new(), Some(3));

    assert!(dispatch(&mut t, "add", "a"));
    assert!(dispatch(&mut t, "add", "b"));
    assert!(dispatch(&mut t, "add", "c"));
    assert_eq!(t.value(), "a,b,c");
    assert!(t.is_at_max());

    // max 到達後の add は no-op。
    assert!(dispatch(&mut t, "add", "d"));
    assert_eq!(t.value(), "a,b,c");

    assert!(dispatch(&mut t, "remove", "1"));
    assert_eq!(t.value(), "a,c");

    assert!(dispatch(&mut t, "clear", ""));
    assert!(t.is_empty());

    assert!(!dispatch(&mut t, "no_such_action", ""));
}

#[test]
fn dispatch_edit_flow_via_public_api() {
    let mut t = TagsInput::new(tags(&["a", "b"]), None);

    assert!(dispatch(&mut t, "edit-start", "0"));
    assert!(t.is_editing(0));
    assert!(render(&t.item(0, false, vec![], vec![])).contains(r#"data-editing="""#));

    assert!(dispatch(&mut t, "edit-submit", "z"));
    assert_eq!(t.tags(), &tags(&["z", "b"]));
    assert!(t.editing_index().is_none());

    // 重複を submit すると編集破棄・元値維持。
    assert!(dispatch(&mut t, "edit-start", "0"));
    assert!(dispatch(&mut t, "edit-submit", "b"));
    assert_eq!(t.tags(), &tags(&["z", "b"]));
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr() {
    let t = TagsInput::default();
    let html = render(&t.view());
    assert!(!html.contains("data-hydrate-"));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let mut t = TagsInput::new(Vec::new(), Some(5));
    dispatch(&mut t, "add", "rust");
    dispatch(&mut t, "add", "wasm");
    let html = render(&render_for_hydration(&t));
    assert!(html.contains(r#"data-hydrate-max="5""#));

    let restored = TagsInput::from_hydration_attrs(&t.hydration_attrs()).unwrap();
    assert_eq!(restored.tags(), t.tags());
    assert_eq!(restored.max(), t.max());
}

#[test]
fn hydration_tampered_duplicate_tags_returns_error_not_panic() {
    let attrs = vec![
        ("data-hydrate-max".to_string(), "none".to_string()),
        (
            "data-hydrate-tags".to_string(),
            fandhe_frontend_interactive::codec::encode_list(&tags(&["dup", "dup"])),
        ),
    ];
    let err = TagsInput::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

#[test]
fn hydration_tampered_length_exceeds_max_returns_error_not_panic() {
    let attrs = vec![
        ("data-hydrate-max".to_string(), "1".to_string()),
        (
            "data-hydrate-tags".to_string(),
            fandhe_frontend_interactive::codec::encode_list(&tags(&["a", "b"])),
        ),
    ];
    let err = TagsInput::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値すべてに攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";
const SCRIPT_PAYLOAD: &str = "<script>alert(1)</script>";

#[test]
fn tag_text_payload_is_escaped_end_to_end() {
    let mut t = TagsInput::default();
    dispatch(&mut t, "add", SCRIPT_PAYLOAD);
    let tag = &t.tags()[0];
    let item_text = tags_input::item_text(vec![], vec![fandhe_frontend_core::text(tag)]);
    let html = render(&t.root(false, vec![], vec![item_text]));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn delete_trigger_aria_label_payload_is_escaped_end_to_end() {
    let trigger = tags_input::item_delete_trigger(ATTR_BREAK_PAYLOAD, false, vec![]);
    let html = render(&trigger);
    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn hidden_input_value_payload_is_escaped_end_to_end() {
    let hidden_input = tags_input::hidden_input("tags", ATTR_BREAK_PAYLOAD, false, vec![]);
    let html = render(&hidden_input);
    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let html = render(&tags_input::root(
        false,
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}
