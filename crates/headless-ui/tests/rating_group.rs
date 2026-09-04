//! RatingGroup（`fandhe_frontend_headless_ui::rating_group`）の統合テスト
//! （イシュー #742、参考サイト（ark-ui Rating Group）突合はイシュー #1617）。
//!
//! フル anatomy の `render()` 出力固定・data-*/ARIA 属性の検証・dispatch/
//! hydration 統合（[`fandhe_frontend_headless_ui::RatingGroup`]）・XSS 回帰
//! （`crates/headless-ui/tests/radio_group.rs` と同型の攻撃ペイロード）・
//! `Anatomy::part` の fail-closed 挙動（呼び出し側 `data-scope`/`data-part`
//! 偽装除去）が RatingGroup パーツ経由でも維持されることを固定する。
//!
//! イシュー #1617 で追加した参考サイト突合契約（5 パーツの `data-part`
//! 集合固定・`data-*`/ARIA の出力有無・roving `tabindex`・pointer/focus 系
//! `data-*` の非出力）は本ファイル下部の専用セクションで検証する。

use fandhe_frontend_core::{render, text, Node};
use fandhe_frontend_headless_ui::rating_group::{self, RatingGroupProps, RatingItemFlags};
use fandhe_frontend_headless_ui::RatingGroup;
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn full_anatomy_renders_expected_html() {
    let props = RatingGroupProps::default();
    let node = rating_group::root(
        &props,
        vec![],
        vec![
            rating_group::label(&props, Some("rating-label"), vec![], vec![text("Rate")]),
            rating_group::control(
                &props,
                Some("rating-label"),
                vec![],
                vec![
                    rating_group::item(
                        1,
                        RatingItemFlags {
                            highlighted: true,
                            ..RatingItemFlags::default()
                        },
                        "1 star",
                        vec![],
                        vec![],
                    ),
                    rating_group::item(
                        2,
                        RatingItemFlags {
                            checked: true,
                            highlighted: true,
                            ..RatingItemFlags::default()
                        },
                        "2 stars",
                        vec![],
                        vec![],
                    ),
                ],
            ),
            rating_group::hidden_input(&props, Some("rating"), "2", vec![]),
        ],
    );

    let html = render(&node);

    let expected = concat!(
        r#"<div data-scope="rating-group" data-part="root">"#,
        r#"<span data-scope="rating-group" data-part="label" id="rating-label">Rate</span>"#,
        r#"<div data-scope="rating-group" data-part="control" role="radiogroup" aria-labelledby="rating-label">"#,
        r#"<span data-scope="rating-group" data-part="item" data-value="1" role="radio" aria-checked="false" aria-label="1 star" tabindex="-1" data-highlighted=""></span>"#,
        r#"<span data-scope="rating-group" data-part="item" data-value="2" role="radio" aria-checked="true" aria-label="2 stars" tabindex="-1" data-checked="" data-highlighted=""></span>"#,
        r#"</div>"#,
        r#"<input data-scope="rating-group" data-part="hidden-input" type="hidden" value="2" name="rating">"#,
        r#"</div>"#,
    );

    assert_eq!(html, expected);
}

#[test]
fn disabled_and_readonly_root_emit_presence_attrs() {
    let props = RatingGroupProps {
        disabled: true,
        readonly: true,
        required: false,
    };
    let node = rating_group::root(&props, vec![], vec![]);
    let html = render(&node);
    assert!(html.contains(r#"data-disabled=""#));
    assert!(html.contains(r#"data-readonly=""#));
}

#[test]
fn item_data_value_payload_is_escaped_on_render() {
    // 将来の `fandhe-frontend-wasm-full` headless 配線基盤が
    // `(scope, part) = ("rating-group", "item")` クリックの set payload 源
    // として `data-value` を参照する契約を見越した回帰テスト（`radio_group`
    // の #580 と同型の判断）。`index` 自体は `u32` のため XSS ペイロードは
    // `aria_label` 側で検証する。
    let payload = "\"><script>alert(1)</script>";
    let html = render(&rating_group::item(
        1,
        RatingItemFlags::default(),
        payload,
        vec![],
        vec![],
    ));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
}

#[test]
fn name_and_value_are_escaped_on_render() {
    let props = RatingGroupProps::default();
    let node =
        rating_group::hidden_input(&props, Some(ATTR_BREAK_PAYLOAD), ATTR_BREAK_PAYLOAD, vec![]);
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
    let props = RatingGroupProps::default();
    let node = rating_group::label(&props, None, vec![], vec![text(payload)]);
    let html = render(&node);

    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn labelledby_id_is_escaped_on_render() {
    let props = RatingGroupProps::default();
    let node = rating_group::control(&props, Some(ATTR_BREAK_PAYLOAD), vec![], vec![]);
    let html = render(&node);

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_cannot_spoof_data_scope_or_part_via_root() {
    let props = RatingGroupProps::default();
    let node = rating_group::root(
        &props,
        vec![("data-scope", "attacker"), ("data-part", "attacker")],
        vec![],
    );
    let html = render(&node);

    assert!(html.contains(r#"data-scope="rating-group""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(!html.contains("attacker"));
}

// --- RatingGroup: dispatch / hydration 統合（公開 API 経由） ---

#[test]
fn rating_group_dispatch_set_updates_value_within_range() {
    let mut g = RatingGroup::new(5, None, false);
    assert!(dispatch(&mut g, "set", "4"));
    assert_eq!(g.value(), Some(4));
}

#[test]
fn rating_group_dispatch_rejects_unparseable_client_payload() {
    // decode_action の fail-closed 制限: u32 パース不能な payload は
    // 状態機械へ一切到達しない。
    let mut g = RatingGroup::new(5, Some(2), false);
    assert!(!dispatch(&mut g, "set", "abc"));
    assert_eq!(g.value(), Some(2));
}

#[test]
fn rating_group_convenience_methods_reflect_dispatch_state() {
    let mut g = RatingGroup::new(5, None, false);
    dispatch(&mut g, "set", "3");

    let item3 = render(&g.item(3, false, "3 stars", vec![], vec![]));
    assert!(item3.contains(r#"data-checked=""#));

    let item5 = render(&g.item(5, false, "5 stars", vec![], vec![]));
    assert!(!item5.contains("data-checked"));

    let hidden = render(&g.hidden_input(Some("rating"), false, vec![]));
    assert!(hidden.contains(r#"value="3""#));
}

#[test]
fn rating_group_default_ssr_view_has_no_hydrate_attr() {
    let rendered = render(&RatingGroup::default().view());
    assert!(!rendered.contains("data-hydrate-"));
}

#[test]
fn rating_group_hydration_round_trip_via_public_api() {
    let mut g = RatingGroup::new(5, None, false);
    dispatch(&mut g, "set", "4");
    let rendered = render(&render_for_hydration(&g));
    assert!(rendered.contains(r#"data-hydrate-value="4""#));

    let restored = RatingGroup::from_hydration_attrs(&g.hydration_attrs()).unwrap();
    assert_eq!(restored, g);
}

#[test]
fn rating_group_from_hydration_attrs_missing_attr_returns_error_not_panic() {
    let err = RatingGroup::from_hydration_attrs(&[]).unwrap_err();
    assert_eq!(
        err,
        HydrateError::MissingAttr("data-hydrate-count".to_string())
    );
}

#[test]
fn rating_group_view_root_is_element() {
    assert!(matches!(
        RatingGroup::default().view(),
        Node::Element { .. }
    ));
}

// ============================================================================
// 参考サイト突合契約（イシュー #1617、ark-ui Rating Group 基準）
// ============================================================================

/// 5 パーツの `data-scope="rating-group"`/`data-part` 集合が ark-ui の
/// anatomy（Root/Label/Control/Item/HiddenInput）と一致することを固定する
/// （`crates/headless-ui/tests/checkbox.rs::reference_anatomy_part_names_match_ark_ui`
/// と同型）。
#[test]
fn reference_anatomy_part_names_match_ark_ui() {
    let props = RatingGroupProps::default();
    let parts: [(&str, String); 5] = [
        ("root", render(&rating_group::root(&props, vec![], vec![]))),
        (
            "label",
            render(&rating_group::label(&props, None, vec![], vec![])),
        ),
        (
            "control",
            render(&rating_group::control(&props, None, vec![], vec![])),
        ),
        (
            "item",
            render(&rating_group::item(
                1,
                RatingItemFlags::default(),
                "1 star",
                vec![],
                vec![],
            )),
        ),
        (
            "hidden-input",
            render(&rating_group::hidden_input(&props, None, "", vec![])),
        ),
    ];
    for (part, html) in &parts {
        assert!(
            html.contains(r#"data-scope="rating-group""#),
            "{part} が data-scope=\"rating-group\" を持たない: {html}"
        );
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "{part} が期待する data-part を持たない: {html}"
        );
    }
}

/// `label` の `data-disabled`/`data-required` が `RatingGroupProps` の
/// 真偽に応じて出力・省略されることを固定する（ark-ui `Label` 突合）。
#[test]
fn label_reflects_disabled_and_required_from_props() {
    let enabled = RatingGroupProps::default();
    let html = render(&rating_group::label(&enabled, None, vec![], vec![]));
    assert!(!html.contains("data-disabled"));
    assert!(!html.contains("data-required"));

    let disabled_required = RatingGroupProps {
        disabled: true,
        readonly: false,
        required: true,
    };
    let html = render(&rating_group::label(
        &disabled_required,
        None,
        vec![],
        vec![],
    ));
    assert!(html.contains(r#"data-disabled=""#));
    assert!(html.contains(r#"data-required=""#));
}

/// `control` の `data-disabled`/`data-readonly` と、真のときのみ出力される
/// `aria-disabled="true"`/`aria-readonly="true"` を固定する（ark-ui
/// `Control` 突合）。`false` のときはいずれの ARIA 属性も出力しない。
#[test]
fn control_reflects_disabled_and_readonly_from_props() {
    let enabled = RatingGroupProps::default();
    let html = render(&rating_group::control(&enabled, None, vec![], vec![]));
    assert!(!html.contains("data-disabled"));
    assert!(!html.contains("data-readonly"));
    assert!(!html.contains("aria-disabled"));
    assert!(!html.contains("aria-readonly"));

    let disabled_readonly = RatingGroupProps {
        disabled: true,
        readonly: true,
        required: false,
    };
    let html = render(&rating_group::control(
        &disabled_readonly,
        None,
        vec![],
        vec![],
    ));
    assert!(html.contains(r#"data-disabled=""#));
    assert!(html.contains(r#"data-readonly=""#));
    assert!(html.contains(r#"aria-disabled="true""#));
    assert!(html.contains(r#"aria-readonly="true""#));
}

/// `item` の roving `tabindex` 契約: `disabled` なら省略、`focusable` なら
/// `"0"`、それ以外は `"-1"`（イシュー #1617 是正: 是正前は
/// `span[role="radio"]` がキーボード到達不能だった）。
#[test]
fn item_tabindex_follows_focusable_disabled_contract() {
    let focusable = render(&rating_group::item(
        1,
        RatingItemFlags {
            focusable: true,
            ..RatingItemFlags::default()
        },
        "1 star",
        vec![],
        vec![],
    ));
    assert!(focusable.contains(r#"tabindex="0""#));

    let not_focusable = render(&rating_group::item(
        2,
        RatingItemFlags::default(),
        "2 stars",
        vec![],
        vec![],
    ));
    assert!(not_focusable.contains(r#"tabindex="-1""#));

    let disabled = render(&rating_group::item(
        3,
        RatingItemFlags {
            disabled: true,
            focusable: true,
            ..RatingItemFlags::default()
        },
        "3 stars",
        vec![],
        vec![],
    ));
    assert!(
        !disabled.contains("tabindex"),
        "disabled な item は tabindex を一切出力しない契約: {disabled}"
    );
}

/// [`RatingGroup::item`] の利便メソッドが「確定選択中の星、未評価なら
/// 1 番目の星」を `focusable`（roving tabindex の tab stop）として算出する
/// ことを固定する（ark-ui/zag.js 実装に合わせた算出、イシュー #1617）。
#[test]
fn rating_group_convenience_item_computes_focusable_tab_stop() {
    let unrated = RatingGroup::new(5, None, false);
    let item1 = render(&unrated.item(1, false, "1 star", vec![], vec![]));
    assert!(item1.contains(r#"tabindex="0""#));
    let item2 = render(&unrated.item(2, false, "2 stars", vec![], vec![]));
    assert!(item2.contains(r#"tabindex="-1""#));

    let rated = RatingGroup::new(5, Some(3), false);
    let item3 = render(&rated.item(3, false, "3 stars", vec![], vec![]));
    assert!(item3.contains(r#"tabindex="0""#));
    let item1 = render(&rated.item(1, false, "1 star", vec![], vec![]));
    assert!(item1.contains(r#"tabindex="-1""#));
}

/// pointer/focus 系 `data-*`（`data-hover`/`data-active`/`data-focus`/
/// `data-focus-visible`/`data-motion`）・`data-half`（`allow_half` 未提供）・
/// `aria-setsize`/`aria-posinset`/`aria-roledescription`/`aria-orientation`
/// のいずれも出力しないことを固定する（モジュール doc「意図的に参考サイト
/// と合わせなかった事項」節の回帰）。
#[test]
fn no_pointer_focus_or_unadopted_reference_attrs_are_emitted() {
    let props = RatingGroupProps {
        disabled: true,
        readonly: true,
        required: true,
    };
    let mut html = String::new();
    html.push_str(&render(&rating_group::root(&props, vec![], vec![])));
    html.push_str(&render(&rating_group::label(
        &props,
        Some("l"),
        vec![],
        vec![],
    )));
    html.push_str(&render(&rating_group::control(
        &props,
        Some("l"),
        vec![],
        vec![],
    )));
    html.push_str(&render(&rating_group::item(
        1,
        RatingItemFlags {
            checked: true,
            highlighted: true,
            disabled: true,
            readonly: true,
            focusable: true,
        },
        "1 star",
        vec![],
        vec![],
    )));
    html.push_str(&render(&rating_group::hidden_input(
        &props,
        Some("n"),
        "1",
        vec![],
    )));

    for forbidden in [
        "data-hover",
        "data-active",
        "data-focus",
        "data-motion",
        "data-half",
        "aria-setsize",
        "aria-posinset",
        "aria-roledescription",
        "aria-orientation",
    ] {
        assert!(
            !html.contains(forbidden),
            "{forbidden} は意図的に非採用のはずが出力されている: {html}"
        );
    }
}
