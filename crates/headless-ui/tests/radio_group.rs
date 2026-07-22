//! RadioGroup（`fandhe_frontend_headless_ui::radio_group`）の統合テスト
//! （イシュー #536）。
//!
//! フル anatomy の `render()` 出力固定・data-*/ARIA 属性の検証・dispatch/
//! hydration 統合（[`fandhe_frontend_headless_ui::RadioGroup`]）・XSS 回帰
//! （`tests/helpers_escape.rs` と同型の攻撃ペイロード）・`Anatomy::part` の
//! fail-closed 挙動（呼び出し側 `data-scope`/`data-part` 偽装除去）が
//! RadioGroup パーツ経由でも維持されることを固定する。

use fandhe_frontend_core::{render, text, Node};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::{radio_group, RadioGroup};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn full_anatomy_renders_expected_html() {
    let node = radio_group::root(
        false,
        Some(Orientation::Vertical),
        Some("plan-label"),
        vec![],
        vec![
            radio_group::label(Some("plan-label"), vec![], vec![text("Plan")]),
            radio_group::item(
                true,
                false,
                vec![],
                vec![
                    radio_group::item_hidden_input(true, false, Some("plan"), "basic", vec![]),
                    radio_group::item_control(true, false, vec![]),
                    radio_group::item_text(true, false, vec![], vec![text("Basic")]),
                ],
            ),
            radio_group::item(
                false,
                false,
                vec![],
                vec![
                    radio_group::item_hidden_input(false, false, Some("plan"), "pro", vec![]),
                    radio_group::item_control(false, false, vec![]),
                    radio_group::item_text(false, false, vec![], vec![text("Pro")]),
                ],
            ),
        ],
    );

    let html = render(&node);

    let expected = concat!(
        r#"<div data-scope="radio-group" data-part="root" role="radiogroup" aria-orientation="vertical" data-orientation="vertical" aria-labelledby="plan-label">"#,
        r#"<span data-scope="radio-group" data-part="label" id="plan-label">Plan</span>"#,
        r#"<label data-scope="radio-group" data-part="item" data-state="checked">"#,
        r#"<input data-scope="radio-group" data-part="item-hidden-input" type="radio" value="basic" data-state="checked" name="plan" checked=""></input>"#,
        r#"<span data-scope="radio-group" data-part="item-control" data-state="checked"></span>"#,
        r#"<span data-scope="radio-group" data-part="item-text" data-state="checked">Basic</span>"#,
        r#"</label>"#,
        r#"<label data-scope="radio-group" data-part="item" data-state="unchecked">"#,
        r#"<input data-scope="radio-group" data-part="item-hidden-input" type="radio" value="pro" data-state="unchecked" name="plan"></input>"#,
        r#"<span data-scope="radio-group" data-part="item-control" data-state="unchecked"></span>"#,
        r#"<span data-scope="radio-group" data-part="item-text" data-state="unchecked">Pro</span>"#,
        r#"</label>"#,
        r#"</div>"#,
    );

    assert_eq!(html, expected);
}

#[test]
fn disabled_root_and_item_emit_data_disabled_presence_attr() {
    let node = radio_group::root(true, None, None, vec![], vec![]);
    let html = render(&node);
    assert!(html.contains(r#"data-disabled=""#));
    assert!(!html.contains("aria-labelledby"));
    assert!(!html.contains("orientation"));

    let item_html = render(&radio_group::item(false, true, vec![], vec![]));
    assert!(item_html.contains(r#"data-disabled=""#));

    let input_html = render(&radio_group::item_hidden_input(
        false,
        true,
        Some("plan"),
        "basic",
        vec![],
    ));
    assert!(input_html.contains("disabled"));
}

#[test]
fn item_control_has_no_radio_role_or_aria_checked() {
    // ネイティブ input が checked semantics を担うため、装飾パーツへの
    // role="radio"/aria-checked 重複付与を行わない（二重読み上げ防止）。
    let html = render(&radio_group::item_control(true, false, vec![]));
    assert!(!html.contains("role=\"radio\""));
    assert!(!html.contains("aria-checked"));
}

#[test]
fn name_and_value_are_escaped_on_render() {
    let node = radio_group::item_hidden_input(
        false,
        false,
        Some(ATTR_BREAK_PAYLOAD),
        ATTR_BREAK_PAYLOAD,
        vec![],
    );
    let html = render(&node);

    assert!(
        !html.contains("onmouseover=\"alert(1)"),
        "name/value がエスケープされずイベントハンドラとして成立している: {html}"
    );
    assert!(html.contains("&quot;"));
}

#[test]
fn label_text_is_escaped_on_render() {
    let payload = "<script>alert(1)</script>";
    let node = radio_group::item_text(false, false, vec![], vec![text(payload)]);
    let html = render(&node);

    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn labelledby_id_is_escaped_on_render() {
    let node = radio_group::root(false, None, Some(ATTR_BREAK_PAYLOAD), vec![], vec![]);
    let html = render(&node);

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_cannot_spoof_data_scope_or_part_via_root() {
    let node = radio_group::root(
        false,
        None,
        None,
        vec![("data-scope", "attacker"), ("data-part", "attacker")],
        vec![],
    );
    let html = render(&node);

    assert!(html.contains(r#"data-scope="radio-group""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(!html.contains("attacker"));
}

// --- RadioGroup: dispatch / hydration 統合（公開 API 経由） ---

#[test]
fn radio_group_dispatch_select_checks_at_most_one_item() {
    let mut g = RadioGroup::default();
    assert!(dispatch(&mut g, "select", "basic"));
    assert!(g.is_checked("basic"));
    assert!(!g.is_checked("pro"));

    assert!(dispatch(&mut g, "select", "pro"));
    assert!(!g.is_checked("basic"));
    assert!(g.is_checked("pro"));
}

#[test]
fn radio_group_dispatch_rejects_toggle_and_deselect_client_actions() {
    // WAI-ARIA radio パターンには選択解除ジェスチャが存在しないため、
    // クライアント由来の文字列 dispatch 境界は "select" のみを受理する
    // （decode_action の fail-closed 制限）。
    let mut g = RadioGroup::default();
    dispatch(&mut g, "select", "basic");

    assert!(!dispatch(&mut g, "toggle", "basic"));
    assert!(g.is_checked("basic"));

    assert!(!dispatch(&mut g, "deselect", ""));
    assert!(g.is_checked("basic"));
}

#[test]
fn radio_group_convenience_methods_reflect_dispatch_state() {
    let mut g = RadioGroup::default();
    dispatch(&mut g, "select", "basic");

    let item_basic = render(&g.item("basic", false, vec![], vec![]));
    assert!(item_basic.contains(r#"data-state="checked""#));

    let input_pro = render(&g.item_hidden_input("pro", false, Some("plan"), vec![]));
    assert!(!input_pro.contains(r#"checked=""#));
}

#[test]
fn radio_group_default_ssr_view_has_no_hydrate_attr() {
    let rendered = render(&RadioGroup::default().view());
    assert!(!rendered.contains("data-hydrate-"));
}

#[test]
fn radio_group_hydration_round_trip_via_public_api() {
    let mut g = RadioGroup::default();
    dispatch(&mut g, "select", "basic");
    let rendered = render(&render_for_hydration(&g));
    assert!(rendered.contains("data-hydrate-selected="));
    assert!(rendered.contains("basic"));

    let restored = RadioGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
    assert_eq!(restored, g);
}

#[test]
fn radio_group_from_hydration_attrs_missing_attr_returns_error_not_panic() {
    let err = RadioGroup::from_hydration_attrs(&[]).unwrap_err();
    assert_eq!(
        err,
        HydrateError::MissingAttr("data-hydrate-selected".to_string())
    );
}

#[test]
fn radio_group_view_root_is_element() {
    assert!(matches!(RadioGroup::default().view(), Node::Element { .. }));
}
