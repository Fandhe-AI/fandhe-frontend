//! RadioGroup（`fandhe_frontend_headless_ui::radio_group`）の統合テスト
//! （イシュー #536、参照突合はイシュー #1616）。
//!
//! フル anatomy の `render()` 出力固定・data-*/ARIA 属性の検証・dispatch/
//! hydration 統合（[`fandhe_frontend_headless_ui::RadioGroup`]）・XSS 回帰
//! （`tests/helpers_escape.rs` と同型の攻撃ペイロード）・`Anatomy::part` の
//! fail-closed 挙動（呼び出し側 `data-scope`/`data-part` 偽装除去）が
//! RadioGroup パーツ経由でも維持されることを固定する。加えてイシュー
//! #1616 で ark-ui / Radix Primitives と突合した anatomy パーツ集合・
//! `data-state` 語彙・`RadioGroupProps` の属性反映・呼び出し側 `attrs` に
//! よる状態/ネイティブ属性偽装への fail-closed 防御を契約テストとして
//! 固定する。

use fandhe_frontend_core::{render, text, Node};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::radio_group::RadioGroupProps;
use fandhe_frontend_headless_ui::{radio_group, RadioGroup};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

fn props() -> RadioGroupProps {
    RadioGroupProps::default()
}

#[test]
fn full_anatomy_renders_expected_html() {
    let p = props();
    let node = radio_group::root(
        &p,
        Some(Orientation::Vertical),
        Some("plan-label"),
        vec![],
        vec![
            radio_group::label(&p, Some("plan-label"), vec![], vec![text("Plan")]),
            radio_group::item(
                true,
                &p,
                "basic",
                vec![],
                vec![
                    radio_group::item_hidden_input(true, &p, Some("plan"), "basic", vec![]),
                    radio_group::item_control(true, &p, vec![]),
                    radio_group::item_text(true, &p, vec![], vec![text("Basic")]),
                ],
            ),
            radio_group::item(
                false,
                &p,
                "pro",
                vec![],
                vec![
                    radio_group::item_hidden_input(false, &p, Some("plan"), "pro", vec![]),
                    radio_group::item_control(false, &p, vec![]),
                    radio_group::item_text(false, &p, vec![], vec![text("Pro")]),
                ],
            ),
        ],
    );

    let html = render(&node);

    let expected = concat!(
        r#"<div data-scope="radio-group" data-part="root" role="radiogroup" aria-orientation="vertical" data-orientation="vertical" aria-labelledby="plan-label">"#,
        r#"<span data-scope="radio-group" data-part="label" id="plan-label">Plan</span>"#,
        r#"<label data-scope="radio-group" data-part="item" data-state="checked" data-value="basic">"#,
        r#"<input data-scope="radio-group" data-part="item-hidden-input" type="radio" value="basic" data-state="checked" name="plan" checked="">"#,
        r#"<span data-scope="radio-group" data-part="item-control" data-state="checked" aria-hidden="true"></span>"#,
        r#"<span data-scope="radio-group" data-part="item-text" data-state="checked">Basic</span>"#,
        r#"</label>"#,
        r#"<label data-scope="radio-group" data-part="item" data-state="unchecked" data-value="pro">"#,
        r#"<input data-scope="radio-group" data-part="item-hidden-input" type="radio" value="pro" data-state="unchecked" name="plan">"#,
        r#"<span data-scope="radio-group" data-part="item-control" data-state="unchecked" aria-hidden="true"></span>"#,
        r#"<span data-scope="radio-group" data-part="item-text" data-state="unchecked">Pro</span>"#,
        r#"</label>"#,
        r#"</div>"#,
    );

    assert_eq!(html, expected);
}

#[test]
fn disabled_root_and_item_emit_data_disabled_presence_attr() {
    let disabled = RadioGroupProps {
        disabled: true,
        ..props()
    };
    let node = radio_group::root(&disabled, None, None, vec![], vec![]);
    let html = render(&node);
    assert!(html.contains(r#"data-disabled=""#));
    assert!(html.contains(r#"aria-disabled="true""#));
    assert!(!html.contains("aria-labelledby"));
    assert!(!html.contains("orientation"));

    let item_html = render(&radio_group::item(
        false,
        &disabled,
        "basic",
        vec![],
        vec![],
    ));
    assert!(item_html.contains(r#"data-disabled=""#));

    let input_html = render(&radio_group::item_hidden_input(
        false,
        &disabled,
        Some("plan"),
        "basic",
        vec![],
    ));
    assert!(input_html.contains("disabled"));
}

#[test]
fn item_data_value_payload_is_escaped_on_render() {
    // イシュー #580: `fandhe-frontend-wasm-full` の headless 配線基盤が
    // `(scope, part) = ("radio-group", "item")` クリックの select payload 源
    // として `data-value` を参照する契約を固定する回帰テスト。
    let payload = "\"><script>alert(1)</script>";
    let html = render(&radio_group::item(false, &props(), payload, vec![], vec![]));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
}

#[test]
fn item_control_has_no_radio_role_or_aria_checked() {
    // ネイティブ input が checked semantics を担うため、装飾パーツへの
    // role="radio"/aria-checked 重複付与を行わない（二重読み上げ防止）。
    let html = render(&radio_group::item_control(true, &props(), vec![]));
    assert!(!html.contains("role=\"radio\""));
    assert!(!html.contains("aria-checked"));
}

#[test]
fn name_and_value_are_escaped_on_render() {
    let node = radio_group::item_hidden_input(
        false,
        &props(),
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
    let node = radio_group::item_text(false, &props(), vec![], vec![text(payload)]);
    let html = render(&node);

    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn labelledby_id_is_escaped_on_render() {
    let node = radio_group::root(&props(), None, Some(ATTR_BREAK_PAYLOAD), vec![], vec![]);
    let html = render(&node);

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_cannot_spoof_data_scope_or_part_via_root() {
    let node = radio_group::root(
        &props(),
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
    let p = props();

    let item_basic = render(&g.item("basic", &p, vec![], vec![]));
    assert!(item_basic.contains(r#"data-state="checked""#));

    let input_pro = render(&g.item_hidden_input("pro", &p, Some("plan"), vec![]));
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

// --- 参照突合（イシュー #1616）: ark-ui / Radix Primitives との anatomy・data-*・ARIA 契約 ---

#[test]
fn reference_anatomy_part_names_match_ark_ui() {
    // 6 anatomy パーツ（Root/Label/Item/ItemControl/ItemText/
    // ItemHiddenInput）が data-scope="radio-group" を共有することを固定する。
    let p = props();
    let parts = [
        render(&radio_group::root(&p, None, None, vec![], vec![])),
        render(&radio_group::label(&p, None, vec![], vec![])),
        render(&radio_group::item(false, &p, "v", vec![], vec![])),
        render(&radio_group::item_control(false, &p, vec![])),
        render(&radio_group::item_text(false, &p, vec![], vec![])),
        render(&radio_group::item_hidden_input(
            false,
            &p,
            None,
            "v",
            vec![],
        )),
    ];
    let expected_part_names = [
        "root",
        "label",
        "item",
        "item-control",
        "item-text",
        "item-hidden-input",
    ];
    for (html, part_name) in parts.iter().zip(expected_part_names.iter()) {
        assert!(html.contains(r#"data-scope="radio-group""#));
        assert!(html.contains(&format!(r#"data-part="{part_name}""#)));
    }
}

#[test]
fn data_state_vocabulary_is_checked_or_unchecked_on_item_parts() {
    let p = props();
    for checked in [true, false] {
        let expected = if checked { "checked" } else { "unchecked" };
        let item_html = render(&radio_group::item(checked, &p, "v", vec![], vec![]));
        let control_html = render(&radio_group::item_control(checked, &p, vec![]));
        let text_html = render(&radio_group::item_text(checked, &p, vec![], vec![]));
        let input_html = render(&radio_group::item_hidden_input(
            checked,
            &p,
            None,
            "v",
            vec![],
        ));
        for html in [&item_html, &control_html, &text_html, &input_html] {
            assert!(html.contains(&format!(r#"data-state="{expected}""#)));
        }
    }
}

#[test]
fn root_and_label_reflect_group_props_per_ark_table() {
    let p = RadioGroupProps {
        disabled: true,
        invalid: true,
        required: true,
        readonly: false,
    };
    let root_html = render(&radio_group::root(&p, None, None, vec![], vec![]));
    assert!(root_html.contains(r#"data-disabled="""#));
    assert!(root_html.contains(r#"data-invalid="""#));
    assert!(root_html.contains(r#"data-required="""#));
    assert!(root_html.contains(r#"aria-required="true""#));
    // 本テストでは readonly=false のため aria-readonly は出力しない。
    assert!(!root_html.contains("aria-readonly"));
    assert!(root_html.contains(r#"aria-disabled="true""#));
    // ark-ui の Root Data Attributes 表に data-readonly は無い。
    assert!(!root_html.contains("data-readonly"));

    let label_html = render(&radio_group::label(&p, None, vec![], vec![]));
    assert!(label_html.contains(r#"data-disabled="""#));
    assert!(label_html.contains(r#"data-invalid="""#));
    assert!(label_html.contains(r#"data-required="""#));
}

#[test]
fn item_parts_reflect_readonly_and_invalid() {
    let p = RadioGroupProps {
        readonly: true,
        invalid: true,
        ..props()
    };
    let item_html = render(&radio_group::item(false, &p, "v", vec![], vec![]));
    let control_html = render(&radio_group::item_control(false, &p, vec![]));
    let text_html = render(&radio_group::item_text(false, &p, vec![], vec![]));
    for html in [&item_html, &control_html, &text_html] {
        assert!(html.contains(r#"data-readonly="""#));
        assert!(html.contains(r#"data-invalid="""#));
        // item 系パーツは data-required を持たない（ark-ui の表に無い）。
        assert!(!html.contains("data-required"));
    }
}

#[test]
fn item_control_is_always_aria_hidden_without_radio_role() {
    let html = render(&radio_group::item_control(true, &props(), vec![]));
    assert!(html.contains(r#"aria-hidden="true""#));
    assert!(!html.contains("role="));
}

#[test]
fn hidden_input_outputs_required_and_aria_invalid_only_when_set() {
    let valid = render(&radio_group::item_hidden_input(
        false,
        &props(),
        Some("plan"),
        "basic",
        vec![],
    ));
    assert!(!valid.contains("required"));
    assert!(!valid.contains("aria-invalid"));

    let p = RadioGroupProps {
        required: true,
        invalid: true,
        ..props()
    };
    let html = render(&radio_group::item_hidden_input(
        false,
        &p,
        Some("plan"),
        "basic",
        vec![],
    ));
    assert!(html.contains(r#"required="""#));
    assert!(html.contains(r#"aria-invalid="true""#));
}

#[test]
fn no_part_outputs_pointer_or_focus_interaction_attrs() {
    let p = RadioGroupProps {
        disabled: true,
        readonly: true,
        invalid: true,
        required: true,
    };
    let html = render(&radio_group::root(
        &p,
        Some(Orientation::Horizontal),
        Some("plan-label"),
        vec![],
        vec![radio_group::item(
            true,
            &p,
            "basic",
            vec![],
            vec![
                radio_group::item_hidden_input(true, &p, Some("plan"), "basic", vec![]),
                radio_group::item_control(true, &p, vec![]),
                radio_group::item_text(true, &p, vec![], vec![text("Basic")]),
            ],
        )],
    ));
    assert!(!html.contains("data-hover"));
    assert!(!html.contains("data-active"));
    assert!(!html.contains("data-focus="));
    assert!(!html.contains("data-motion"));
}

#[test]
fn caller_attrs_cannot_spoof_state_or_native_attrs() {
    let root_html = render(&radio_group::root(
        &props(),
        None,
        None,
        vec![
            ("role", "attacker"),
            ("aria-disabled", "false"),
            ("data-disabled", "spoofed"),
        ],
        vec![],
    ));
    assert!(root_html.contains(r#"role="radiogroup""#));
    assert!(!root_html.contains("attacker"));
    assert!(!root_html.contains("spoofed"));
    assert_eq!(root_html.matches("aria-disabled").count(), 0);

    let item_control_html = render(&radio_group::item_control(
        true,
        &props(),
        vec![("aria-hidden", "false"), ("data-state", "unchecked")],
    ));
    assert!(item_control_html.contains(r#"aria-hidden="true""#));
    assert!(item_control_html.contains(r#"data-state="checked""#));
    assert_eq!(item_control_html.matches("aria-hidden").count(), 1);

    let hidden_input_html = render(&radio_group::item_hidden_input(
        true,
        &props(),
        Some("plan"),
        "basic",
        vec![
            ("type", "text"),
            ("checked", "false"),
            ("disabled", ""),
            ("aria-invalid", "true"),
        ],
    ));
    assert!(hidden_input_html.contains(r#"type="radio""#));
    assert!(!hidden_input_html.contains("disabled"));
    assert!(!hidden_input_html.contains("aria-invalid"));
}

#[test]
fn radio_group_props_payload_is_escaped_via_root_and_item() {
    // RadioGroupProps 経由の呼び出しでもエスケープを迂回しないことの回帰
    // （イシュー #1616）。
    let p = RadioGroupProps {
        invalid: true,
        required: true,
        ..props()
    };
    let html = render(&radio_group::item(
        false,
        &p,
        ATTR_BREAK_PAYLOAD,
        vec![],
        vec![text("<script>alert(1)</script>")],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&quot;"));
    assert!(html.contains("&lt;script&gt;"));
}
