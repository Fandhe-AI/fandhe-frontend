//! RatingGroup（`fandhe_frontend_headless_ui::rating_group`）の統合テスト
//! （イシュー #742）。
//!
//! フル anatomy の `render()` 出力固定・data-*/ARIA 属性の検証・dispatch/
//! hydration 統合（[`fandhe_frontend_headless_ui::RatingGroup`]）・XSS 回帰
//! （`crates/headless-ui/tests/radio_group.rs` と同型の攻撃ペイロード）・
//! `Anatomy::part` の fail-closed 挙動（呼び出し側 `data-scope`/`data-part`
//! 偽装除去）が RatingGroup パーツ経由でも維持されることを固定する。

use fandhe_frontend_core::{render, text, Node};
use fandhe_frontend_headless_ui::rating_group::{self, RatingItemFlags};
use fandhe_frontend_headless_ui::RatingGroup;
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn full_anatomy_renders_expected_html() {
    let node = rating_group::root(
        false,
        false,
        vec![],
        vec![
            rating_group::label(Some("rating-label"), vec![], vec![text("Rate")]),
            rating_group::control(
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
            rating_group::hidden_input(Some("rating"), "2", false, vec![]),
        ],
    );

    let html = render(&node);

    let expected = concat!(
        r#"<div data-scope="rating-group" data-part="root">"#,
        r#"<span data-scope="rating-group" data-part="label" id="rating-label">Rate</span>"#,
        r#"<div data-scope="rating-group" data-part="control" role="radiogroup" aria-labelledby="rating-label">"#,
        r#"<span data-scope="rating-group" data-part="item" data-value="1" role="radio" aria-checked="false" aria-label="1 star" data-highlighted=""></span>"#,
        r#"<span data-scope="rating-group" data-part="item" data-value="2" role="radio" aria-checked="true" aria-label="2 stars" data-checked="" data-highlighted=""></span>"#,
        r#"</div>"#,
        r#"<input data-scope="rating-group" data-part="hidden-input" type="hidden" value="2" name="rating"></input>"#,
        r#"</div>"#,
    );

    assert_eq!(html, expected);
}

#[test]
fn disabled_and_readonly_root_emit_presence_attrs() {
    let node = rating_group::root(true, true, vec![], vec![]);
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
    let node =
        rating_group::hidden_input(Some(ATTR_BREAK_PAYLOAD), ATTR_BREAK_PAYLOAD, false, vec![]);
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
    let node = rating_group::label(None, vec![], vec![text(payload)]);
    let html = render(&node);

    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn labelledby_id_is_escaped_on_render() {
    let node = rating_group::control(Some(ATTR_BREAK_PAYLOAD), vec![], vec![]);
    let html = render(&node);

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_cannot_spoof_data_scope_or_part_via_root() {
    let node = rating_group::root(
        false,
        false,
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
