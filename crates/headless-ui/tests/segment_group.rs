//! SegmentGroup（`fandhe_frontend_headless_ui::segment_group`）の統合テスト
//! （イシュー #743、参照突合はイシュー #1618）。
//!
//! 公開 API（クレートルート再エクスポート [`SegmentGroup`]/[`SegmentGroupProps`]
//! とモジュール `segment_group`）経由でのフル anatomy 組み立て・indicator の
//! 位置表現・dispatch/hydration 統合（`radio_group::RadioGroup` への委譲が
//! 公開 API 越しでも成立すること）を固定する。in-module テスト
//! （`src/segment_group.rs`）は各パーツ関数単体の出力を固定し、本ファイルは
//! 公開 API 経由の統合観点と ark-ui 参照突合（イシュー #1618）の契約を
//! 固定する（`tests/radio_group.rs` と同型の役割分担）。

use fandhe_frontend_core::{render, text, Node};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::{segment_group, SegmentGroup, SegmentGroupProps};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn full_anatomy_renders_expected_html_via_public_api() {
    let props = SegmentGroupProps::default();
    let node = segment_group::root(
        &props,
        Some(Orientation::Horizontal),
        Some("view-label"),
        vec![],
        vec![
            segment_group::indicator(Some((0, 2)), &props, Some(Orientation::Horizontal), vec![]),
            segment_group::item(
                true,
                &props,
                "list",
                vec![],
                vec![
                    segment_group::item_hidden_input(true, &props, Some("view"), "list", vec![]),
                    segment_group::item_control(true, &props, vec![]),
                    segment_group::item_text(true, &props, vec![], vec![text("List")]),
                ],
            ),
            segment_group::item(
                false,
                &props,
                "grid",
                vec![],
                vec![
                    segment_group::item_hidden_input(false, &props, Some("view"), "grid", vec![]),
                    segment_group::item_control(false, &props, vec![]),
                    segment_group::item_text(false, &props, vec![], vec![text("Grid")]),
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
    let html = render(&segment_group::item_control(
        true,
        &SegmentGroupProps::default(),
        vec![],
    ));
    assert!(!html.contains(r#"role="radio""#));
    assert!(!html.contains("aria-checked"));
}

#[test]
fn disabled_root_and_item_emit_data_disabled_presence_attr() {
    let props = SegmentGroupProps {
        disabled: true,
        ..Default::default()
    };
    let root_html = render(&segment_group::root(&props, None, None, vec![], vec![]));
    assert!(root_html.contains(r#"data-disabled="""#));

    let item_html = render(&segment_group::item(false, &props, "list", vec![], vec![]));
    assert!(item_html.contains(r#"data-disabled="""#));
}

#[test]
fn caller_attrs_cannot_spoof_data_scope_or_part_via_root() {
    let html = render(&segment_group::root(
        &SegmentGroupProps::default(),
        None,
        None,
        vec![("data-scope", "attacker"), ("data-part", "attacker")],
        vec![],
    ));
    assert!(html.contains(r#"data-scope="segment-group""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(!html.contains("attacker"));
}

// --- 参照突合（イシュー #1618）: ark-ui Data Attributes / Anatomy 契約 ---

#[test]
fn reference_anatomy_part_names_match_ark_ui() {
    let props = SegmentGroupProps::default();
    let node = segment_group::root(
        &props,
        None,
        None,
        vec![],
        vec![
            segment_group::indicator(None, &props, None, vec![]),
            segment_group::item(
                false,
                &props,
                "list",
                vec![],
                vec![
                    segment_group::item_hidden_input(false, &props, Some("view"), "list", vec![]),
                    segment_group::item_control(false, &props, vec![]),
                    segment_group::item_text(false, &props, vec![], vec![]),
                ],
            ),
        ],
    );
    let html = render(&node);
    for part in [
        "root",
        "indicator",
        "item",
        "item-hidden-input",
        "item-control",
        "item-text",
    ] {
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "missing part: {part}"
        );
    }
}

#[test]
fn root_reflects_group_props_per_ark_table() {
    let props = SegmentGroupProps {
        disabled: true,
        readonly: true,
        invalid: true,
        required: true,
    };
    let html = render(&segment_group::root(&props, None, None, vec![], vec![]));
    assert!(html.contains(r#"data-disabled="""#));
    assert!(html.contains(r#"data-invalid="""#));
    assert!(html.contains(r#"data-required="""#));
    assert!(html.contains(r#"aria-disabled="true""#));
    assert!(html.contains(r#"aria-readonly="true""#));
    assert!(html.contains(r#"aria-required="true""#));
    // ark の Root Data Attributes 表に data-readonly は無い
    assert!(!html.contains("data-readonly"));
}

#[test]
fn indicator_reflects_disabled_and_keeps_state_and_style() {
    let props = SegmentGroupProps {
        disabled: true,
        ..Default::default()
    };
    let html = render(&segment_group::indicator(
        Some((1, 3)),
        &props,
        None,
        vec![],
    ));
    assert!(html.contains(r#"data-disabled="""#));
    assert!(html.contains(r#"data-state="checked""#));
    assert!(html.contains("--fandhe-segment-group-index: 1;"));
    assert!(html.contains("--fandhe-segment-group-count: 3;"));
}

#[test]
fn item_parts_reflect_readonly_and_invalid() {
    let props = SegmentGroupProps {
        readonly: true,
        invalid: true,
        ..Default::default()
    };
    let item_html = render(&segment_group::item(false, &props, "list", vec![], vec![]));
    assert!(item_html.contains(r#"data-readonly="""#));
    assert!(item_html.contains(r#"data-invalid="""#));
    assert!(!item_html.contains("data-required"));

    let control_html = render(&segment_group::item_control(false, &props, vec![]));
    assert!(control_html.contains(r#"data-readonly="""#));
    assert!(control_html.contains(r#"data-invalid="""#));

    let text_html = render(&segment_group::item_text(
        false,
        &props,
        vec![],
        vec![text("List")],
    ));
    assert!(text_html.contains(r#"data-readonly="""#));
    assert!(text_html.contains(r#"data-invalid="""#));
}

#[test]
fn item_control_is_always_aria_hidden_without_radio_role() {
    let html = render(&segment_group::item_control(
        true,
        &SegmentGroupProps::default(),
        vec![],
    ));
    assert!(html.contains(r#"aria-hidden="true""#));
    assert!(!html.contains(r#"role="radio""#));
    assert!(!html.contains("aria-checked"));
}

#[test]
fn hidden_input_outputs_required_and_aria_invalid_only_when_set() {
    let props = SegmentGroupProps {
        required: true,
        invalid: true,
        readonly: true,
        ..Default::default()
    };
    let html = render(&segment_group::item_hidden_input(
        false,
        &props,
        Some("view"),
        "list",
        vec![],
    ));
    assert!(html.contains(r#"required="""#));
    assert!(html.contains(r#"aria-invalid="true""#));
    assert!(!html.contains("readonly"));

    let default_html = render(&segment_group::item_hidden_input(
        false,
        &SegmentGroupProps::default(),
        Some("view"),
        "grid",
        vec![],
    ));
    assert!(!default_html.contains("required"));
    assert!(!default_html.contains("aria-invalid"));
}

#[test]
fn no_part_outputs_pointer_or_focus_interaction_attrs() {
    let props = SegmentGroupProps::default();
    let node = segment_group::root(
        &props,
        None,
        None,
        vec![],
        vec![
            segment_group::indicator(Some((0, 1)), &props, None, vec![]),
            segment_group::item(
                true,
                &props,
                "list",
                vec![],
                vec![
                    segment_group::item_hidden_input(true, &props, Some("view"), "list", vec![]),
                    segment_group::item_control(true, &props, vec![]),
                    segment_group::item_text(true, &props, vec![], vec![]),
                ],
            ),
        ],
    );
    let html = render(&node);
    assert!(!html.contains("data-active"));
    assert!(!html.contains("data-hover"));
    assert!(!html.contains("data-focus"));
}

#[test]
fn caller_attrs_cannot_spoof_state_or_native_attrs() {
    let html = render(&segment_group::item_hidden_input(
        false,
        &SegmentGroupProps::default(),
        Some("view"),
        "list",
        vec![
            ("data-state", "checked"),
            ("type", "text"),
            ("checked", "checked"),
            ("disabled", "disabled"),
            ("required", "required"),
            ("aria-invalid", "true"),
        ],
    ));
    assert!(html.contains(r#"data-state="unchecked""#));
    assert!(html.contains(r#"type="radio""#));
    assert!(!html.contains(r#"checked="checked""#));
    assert!(!html.contains("disabled"));
    assert!(!html.contains("required"));
    assert!(!html.contains("aria-invalid"));
}

#[test]
fn props_payload_is_escaped_via_root_and_item() {
    let props = SegmentGroupProps {
        disabled: true,
        invalid: true,
        required: true,
        readonly: true,
    };
    let html = render(&segment_group::root(
        &props,
        None,
        None,
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![segment_group::item(
            false,
            &props,
            ATTR_BREAK_PAYLOAD,
            vec![],
            vec![text("<script>alert(1)</script>")],
        )],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&lt;script&gt;"));
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
    let props = SegmentGroupProps::default();

    let item_html = render(&g.item("grid", &props, vec![], vec![]));
    assert!(item_html.contains(r#"data-state="checked""#));

    let indicator_html = render(&g.indicator(&["list", "grid"], &props, None, vec![]));
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
        &SegmentGroupProps::default(),
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
        &SegmentGroupProps::default(),
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
        &SegmentGroupProps::default(),
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
