//! SegmentGroup（`fandhe_frontend_headless_ui::segment_group`）の統合テスト
//! （イシュー #743）。
//!
//! 公開 API（クレートルート再エクスポート [`SegmentGroup`] とモジュール
//! `segment_group`）経由でのフル anatomy 組み立て・indicator の位置表現・
//! dispatch/hydration 統合（`radio_group::RadioGroup` への委譲が公開 API
//! 越しでも成立すること）を固定する。in-module テスト（`src/segment_group.rs`）
//! は各パーツ関数単体の出力を固定し、本ファイルは公開 API 経由の統合観点を
//! 固定する（`tests/radio_group.rs` と同型の役割分担）。

use fandhe_frontend_core::{render, text, Node};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::{segment_group, SegmentGroup};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn full_anatomy_renders_expected_html_via_public_api() {
    let node = segment_group::root(
        false,
        Some(Orientation::Horizontal),
        Some("view-label"),
        vec![],
        vec![
            segment_group::indicator(Some((0, 2)), Some(Orientation::Horizontal), vec![]),
            segment_group::item(
                true,
                false,
                "list",
                vec![],
                vec![
                    segment_group::item_hidden_input(true, false, Some("view"), "list", vec![]),
                    segment_group::item_control(true, false, vec![]),
                    segment_group::item_text(true, false, vec![], vec![text("List")]),
                ],
            ),
            segment_group::item(
                false,
                false,
                "grid",
                vec![],
                vec![
                    segment_group::item_hidden_input(false, false, Some("view"), "grid", vec![]),
                    segment_group::item_control(false, false, vec![]),
                    segment_group::item_text(false, false, vec![], vec![text("Grid")]),
                ],
            ),
        ],
    );

    let html = render(&node);
    assert!(html.contains(r#"data-scope="segment-group" data-part="root""#));
    assert!(html.contains(r#"role="radiogroup""#));
    assert!(html.contains(r#"aria-orientation="horizontal""#));
    assert!(html.contains(r#"aria-labelledby="view-label""#));
    assert!(html.contains(r#"data-part="indicator""#));
    assert!(html.contains(r#"aria-hidden="true""#));
    assert!(html.contains("--fandhe-segment-group-index: 0;"));
    assert!(html.contains("--fandhe-segment-group-count: 2;"));
    assert!(html.contains(r#"data-part="item-hidden-input""#));
    assert!(html.contains(r#"type="radio""#));
}

#[test]
fn item_control_has_no_radio_role_or_aria_checked() {
    let html = render(&segment_group::item_control(true, false, vec![]));
    assert!(!html.contains(r#"role="radio""#));
    assert!(!html.contains("aria-checked"));
}

#[test]
fn disabled_root_and_item_emit_data_disabled_presence_attr() {
    let root_html = render(&segment_group::root(true, None, None, vec![], vec![]));
    assert!(root_html.contains(r#"data-disabled="""#));

    let item_html = render(&segment_group::item(false, true, "list", vec![], vec![]));
    assert!(item_html.contains(r#"data-disabled="""#));
}

#[test]
fn caller_attrs_cannot_spoof_data_scope_or_part_via_root() {
    let html = render(&segment_group::root(
        false,
        None,
        None,
        vec![("data-scope", "attacker"), ("data-part", "attacker")],
        vec![],
    ));
    assert!(html.contains(r#"data-scope="segment-group""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(!html.contains("attacker"));
}

// --- SegmentGroup: dispatch/hydration 統合（radio_group への委譲が公開 API 越しでも成立） ---

#[test]
fn segment_group_view_root_is_element() {
    let node = SegmentGroup::default().view();
    assert!(matches!(node, Node::Element { .. }));
}

#[test]
fn segment_group_default_ssr_view_has_no_hydrate_attr() {
    let rendered = render(&SegmentGroup::default().view());
    assert!(!rendered.contains("data-hydrate-"));
}

#[test]
fn segment_group_dispatch_select_checks_at_most_one_item() {
    let mut g = SegmentGroup::default();
    assert!(dispatch(&mut g, "select", "list"));
    assert!(g.is_checked("list"));

    assert!(dispatch(&mut g, "select", "grid"));
    assert!(!g.is_checked("list"));
    assert!(g.is_checked("grid"));
}

#[test]
fn segment_group_dispatch_rejects_toggle_and_deselect_client_actions() {
    let mut g = SegmentGroup::default();
    dispatch(&mut g, "select", "list");

    assert!(!dispatch(&mut g, "toggle", "list"));
    assert!(!dispatch(&mut g, "deselect", ""));
    assert!(g.is_checked("list"));
}

#[test]
fn segment_group_convenience_methods_reflect_dispatch_state() {
    let mut g = SegmentGroup::default();
    dispatch(&mut g, "select", "grid");

    let item_html = render(&g.item("grid", false, vec![], vec![]));
    assert!(item_html.contains(r#"data-state="checked""#));

    let indicator_html = render(&g.indicator(&["list", "grid"], None, vec![]));
    assert!(indicator_html.contains("--fandhe-segment-group-index: 1;"));
    assert!(indicator_html.contains("--fandhe-segment-group-count: 2;"));
}

#[test]
fn segment_group_hydration_round_trip_via_public_api() {
    let mut g = SegmentGroup::default();
    dispatch(&mut g, "select", "list");

    let rendered = render(&render_for_hydration(&g));
    assert!(rendered.contains("data-hydrate-selected="));

    let restored = SegmentGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
    assert_eq!(restored, g);
}

#[test]
fn segment_group_from_hydration_attrs_missing_attr_returns_error_not_panic() {
    let err = SegmentGroup::from_hydration_attrs(&[]).unwrap_err();
    assert_eq!(
        err,
        HydrateError::MissingAttr("data-hydrate-selected".to_string())
    );
}

// --- XSS 回帰: labelled_by/name/value/children/dispatch payload の公開 API 越しエスケープ ---

#[test]
fn labelledby_id_is_escaped_on_render() {
    let html = render(&segment_group::root(
        false,
        None,
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn name_and_value_are_escaped_on_render() {
    let html = render(&segment_group::item_hidden_input(
        false,
        false,
        Some(ATTR_BREAK_PAYLOAD),
        ATTR_BREAK_PAYLOAD,
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn item_text_children_payload_is_escaped_on_render() {
    let payload = "<script>alert(1)</script>";
    let html = render(&segment_group::item_text(
        false,
        false,
        vec![],
        vec![text(payload)],
    ));
    assert!(!html.contains(payload));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn segment_group_dispatch_select_payload_is_escaped_on_render() {
    let mut g = SegmentGroup::default();
    let payload = "\"><script>alert(1)</script>";
    assert!(dispatch(&mut g, "select", payload));

    let rendered = render(&render_for_hydration(&g));
    assert!(rendered.contains("&lt;script&gt;"));
    assert!(!rendered.contains(payload));
}
