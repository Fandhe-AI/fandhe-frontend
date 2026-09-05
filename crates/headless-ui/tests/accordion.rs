//! `fandhe-frontend-headless-ui` の Accordion（[`accordion`] モジュール、
//! イシュー #527）の公開 API 経由の統合テスト。
//!
//! `crates/headless-ui/src/accordion.rs` 内の `#[cfg(test)]` ユニットテストが
//! 内部実装を含めた網羅を担うのに対し、本ファイルは
//! `fandhe_frontend_headless_ui::accordion` の公開 API（自由関数群 +
//! [`Accordion`]）のみを経由し、`fandhe-frontend-pre-styled-ui`（#546〜）が
//! 実際に使う想定の外部からの利用形態（SSR 組み立て・dispatch・
//! hydration・XSS 回帰）を固定する回帰テスト。イシュー #1636 で参考サイト
//! （ark-ui/Radix）と突合し `AccordionProps`（`orientation`/`disabled`）を
//! 追加した後もこの契約を維持する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::accordion::{
    item, item_content, item_indicator, item_trigger, root, Accordion, AccordionProps,
    MultiAccordion,
};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::OpenState;
use fandhe_frontend_interactive::{dispatch, render_for_hydration, Component, Hydrate};

/// 2 項目 Accordion（a/b）を id 相互参照付きで組み立てるヘルパ。
/// Tabs（#528）の `"{id}-trigger-{value}"`/`"{id}-content-{value}"` 命名規約を踏襲する。
fn two_item_accordion(open_value: &str) -> String {
    let props = AccordionProps::default();
    let items = [("a", "Panel A"), ("b", "Panel B")];
    let children = items
        .iter()
        .map(|(value, label)| {
            let trigger_id = format!("acc-trigger-{value}");
            let content_id = format!("acc-content-{value}");
            let state = if *value == open_value {
                OpenState::Open
            } else {
                OpenState::Closed
            };
            item(
                state,
                false,
                &props,
                vec![],
                vec![
                    item_trigger(
                        state,
                        false,
                        &props,
                        value,
                        Some(&trigger_id),
                        Some(&content_id),
                        vec![],
                        vec![text(*label)],
                    ),
                    item_content(
                        state,
                        false,
                        &props,
                        Some(&content_id),
                        Some(&trigger_id),
                        vec![],
                        vec![text(*label)],
                    ),
                ],
            )
        })
        .collect();
    render(&root(&props, vec![], children))
}

#[test]
fn ssr_assembly_reflects_selected_item_open_state_and_id_cross_reference() {
    let html = two_item_accordion("a");

    assert!(html.contains(r#"id="acc-trigger-a""#));
    assert!(html.contains(r#"aria-controls="acc-content-a""#));
    assert!(html.contains(r#"id="acc-content-a""#));
    assert!(html.contains(r#"aria-labelledby="acc-trigger-a""#));

    // a は open: aria-expanded="true"・hidden なし。b は closed。
    assert!(html.matches(r#"aria-expanded="true""#).count() == 1);
    assert!(html.matches(r#"aria-expanded="false""#).count() == 1);
    // b は closed のため content が hidden。
    assert_eq!(html.matches(r#"hidden="""#).count(), 1);
}

#[test]
fn ssr_assembly_can_express_multiple_simultaneous_open_items_without_dispatch() {
    // 自由関数パーツは項目ごとに OpenState を受け取るため、dispatch 統合
    // （single モードの Accordion 型）を経由しない SSR マークアップとしては
    // 複数項目同時 open を表現できる（モジュール doc §out-of-scope 参照）。
    let props = AccordionProps::default();
    let a_open = item(OpenState::Open, false, &props, vec![], vec![]);
    let b_open = item(OpenState::Open, false, &props, vec![], vec![]);
    let html = render(&root(&props, vec![], vec![a_open, b_open]));
    assert_eq!(html.matches(r#"data-state="open""#).count(), 2);
}

// --- イシュー #1636: AccordionProps（orientation/disabled）の統合契約 ---

#[test]
fn accordion_props_default_is_vertical_and_not_disabled() {
    let props = AccordionProps::default();
    assert_eq!(props.orientation, Orientation::Vertical);
    assert!(!props.disabled);
}

#[test]
fn horizontal_orientation_propagates_to_all_parts_data_orientation() {
    let props = AccordionProps {
        orientation: Orientation::Horizontal,
        disabled: false,
    };
    let node = root(
        &props,
        vec![],
        vec![item(
            OpenState::Open,
            false,
            &props,
            vec![],
            vec![
                item_trigger(
                    OpenState::Open,
                    false,
                    &props,
                    "a",
                    None,
                    None,
                    vec![],
                    vec![item_indicator(
                        OpenState::Open,
                        false,
                        &props,
                        vec![],
                        vec![],
                    )],
                ),
                item_content(OpenState::Open, false, &props, None, None, vec![], vec![]),
            ],
        )],
    );
    let html = render(&node);
    assert_eq!(html.matches(r#"data-orientation="horizontal""#).count(), 5);
}

#[test]
fn root_level_disabled_propagates_to_trigger_indicator_and_content() {
    let props = AccordionProps {
        orientation: Orientation::Vertical,
        disabled: true,
    };
    // 項目単位の disabled=false でも、root の disabled=true が実効 disabled
    // として trigger/indicator/content の data-disabled へ伝播する。
    let trigger_html = render(&item_trigger(
        OpenState::Closed,
        false,
        &props,
        "a",
        None,
        None,
        vec![],
        vec![],
    ));
    assert!(trigger_html.contains(r#"data-disabled="""#));
    assert!(trigger_html.contains(r#"disabled="""#));
    assert!(trigger_html.contains(r#"aria-disabled="true""#));

    let indicator_html = render(&item_indicator(
        OpenState::Closed,
        false,
        &props,
        vec![],
        vec![],
    ));
    assert!(indicator_html.contains(r#"data-disabled="""#));

    let content_html = render(&item_content(
        OpenState::Closed,
        false,
        &props,
        None,
        None,
        vec![],
        vec![],
    ));
    assert!(content_html.contains(r#"data-disabled="""#));
}

#[test]
fn item_indicator_always_carries_aria_hidden_true() {
    let props = AccordionProps::default();
    let open = render(&item_indicator(
        OpenState::Open,
        false,
        &props,
        vec![],
        vec![],
    ));
    let closed = render(&item_indicator(
        OpenState::Closed,
        false,
        &props,
        vec![],
        vec![],
    ));
    assert!(open.contains(r#"aria-hidden="true""#));
    assert!(closed.contains(r#"aria-hidden="true""#));
}

#[test]
fn intentionally_omitted_attributes_are_absent() {
    // 参考サイト突合（イシュー #1636）で意図的に非採用とした属性・パートが
    // 出力されないことを外部契約としても固定する。
    let props = AccordionProps {
        orientation: Orientation::Horizontal,
        disabled: true,
    };
    let node = root(
        &props,
        vec![],
        vec![item(
            OpenState::Open,
            true,
            &props,
            vec![],
            vec![
                item_trigger(
                    OpenState::Open,
                    true,
                    &props,
                    "a",
                    None,
                    None,
                    vec![],
                    vec![item_indicator(
                        OpenState::Open,
                        true,
                        &props,
                        vec![],
                        vec![],
                    )],
                ),
                item_content(OpenState::Open, true, &props, None, None, vec![], vec![]),
            ],
        )],
    );
    let html = render(&node);
    assert!(!html.contains("data-focus"));
    assert!(!html.contains("data-motion"));
    assert!(!html.contains("data-ownedby"));
    assert!(!html.contains("data-controls"));
    assert!(!html.contains("--height"));
    assert!(!html.contains("--width"));
    assert!(!html.contains(r#"data-part="header""#));
}

#[test]
fn accordion_component_full_cycle_ssr_then_dispatch_then_hydration() {
    // SSR: 状態なし初期描画（Default = 全項目 closed）。
    let initial = Accordion::default();
    let ssr_view_html = render(&initial.view());
    assert!(!ssr_view_html.contains("data-hydrate-"));
    assert!(!initial.is_open("a"));
    assert!(!initial.is_open("b"));

    // クライアント側（wasm-full 相当）の dispatch で選択。
    let mut client_state = initial;
    assert!(dispatch(&mut client_state, "select", "a"));
    assert!(client_state.is_open("a"));
    assert!(!client_state.is_open("b"));

    // 高々 1 個選択の制約: 別項目を select すると a は自動的に閉じる。
    assert!(dispatch(&mut client_state, "select", "b"));
    assert!(!client_state.is_open("a"));
    assert!(client_state.is_open("b"));

    // 利便メソッド経由の描画が状態機械と一致する。
    let props = AccordionProps::default();
    let trigger_b_html =
        render(&client_state.item_trigger("b", false, &props, None, None, vec![], vec![]));
    assert!(trigger_b_html.contains(r#"aria-expanded="true""#));

    // 別の SSR リクエストはハイドレーション属性込みで出力される。
    let hydrated_html = render(&render_for_hydration(&client_state));
    assert!(hydrated_html.contains("data-hydrate-selected="));

    // クライアント側は data-hydrate-* 属性から状態を復元できる（ラウンドトリップ）。
    let restored = Accordion::from_hydration_attrs(&client_state.hydration_attrs()).unwrap();
    assert_eq!(restored, client_state);
}

#[test]
fn accordion_component_collapsible_toggle_cycle() {
    let mut a = Accordion::default();
    assert!(dispatch(&mut a, "toggle", "a"));
    assert!(a.is_open("a"));
    assert!(dispatch(&mut a, "toggle", "a"));
    assert!(!a.is_open("a"));
    assert_eq!(a.expanded(), None);
}

#[test]
fn accordion_component_ignores_unknown_dispatch_action() {
    let mut a = Accordion::default();
    dispatch(&mut a, "select", "a");
    assert!(!dispatch(&mut a, "unknown", "b"));
    assert!(a.is_open("a"));
}

// --- XSS 回帰: 統合レベルでも value/id/children にペイロードを渡してエスケープを確認する ---

#[test]
fn xss_payload_in_ids_and_children_is_escaped_on_render() {
    let props = AccordionProps::default();
    let payload = "\"><script>alert(1)</script>";
    let html = render(&root(
        &props,
        vec![],
        vec![item(
            OpenState::Open,
            false,
            &props,
            vec![],
            vec![
                item_trigger(
                    OpenState::Open,
                    false,
                    &props,
                    payload,
                    Some(payload),
                    Some(payload),
                    vec![],
                    vec![text(payload)],
                ),
                item_content(
                    OpenState::Open,
                    false,
                    &props,
                    Some(payload),
                    Some(payload),
                    vec![],
                    vec![text(payload)],
                ),
                item_indicator(OpenState::Open, false, &props, vec![], vec![text(payload)]),
            ],
        )],
    ));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(!html.contains(r#""><script"#));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(html.contains("&quot;"));
}

#[test]
fn xss_payload_in_dispatch_select_value_is_escaped_on_render_for_hydration() {
    let mut a = Accordion::default();
    let payload = "\"><script>alert(1)</script>";
    assert!(dispatch(&mut a, "select", payload));

    let rendered = render(&render_for_hydration(&a));
    assert!(rendered.contains("data-hydrate-selected="));
    assert!(rendered.contains("&lt;script&gt;"));
    assert!(!rendered.contains("<script>alert(1)</script>"));
    assert!(!rendered.contains(r#""><script"#));
}

// --- MultiAccordion: multiple モード（イシュー #594） ---

#[test]
fn multi_accordion_component_full_cycle_ssr_then_dispatch_then_hydration() {
    // SSR: 状態なし初期描画（Default = 全項目 closed）。
    let initial = MultiAccordion::default();
    let ssr_view_html = render(&initial.view());
    assert!(!ssr_view_html.contains("data-hydrate-"));
    assert!(!initial.is_open("a"));
    assert!(!initial.is_open("b"));

    // クライアント側（wasm-full 相当）の dispatch で複数項目を同時選択。
    let mut client_state = initial;
    assert!(dispatch(&mut client_state, "select", "a"));
    assert!(client_state.is_open("a"));
    assert!(!client_state.is_open("b"));

    // MultiAccordion は Accordion と異なり、別項目の select が既存の open を
    // 閉じない（複数同時 open が本イシューの存在理由）。
    assert!(dispatch(&mut client_state, "select", "b"));
    assert!(client_state.is_open("a"));
    assert!(client_state.is_open("b"));

    // 利便メソッド経由の描画が状態機械と一致する（2 項目とも aria-expanded="true"）。
    let props = AccordionProps::default();
    let trigger_a_html =
        render(&client_state.item_trigger("a", false, &props, None, None, vec![], vec![]));
    assert!(trigger_a_html.contains(r#"aria-expanded="true""#));
    let trigger_b_html =
        render(&client_state.item_trigger("b", false, &props, None, None, vec![], vec![]));
    assert!(trigger_b_html.contains(r#"aria-expanded="true""#));

    // 別の SSR リクエストはハイドレーション属性込みで出力される。
    let hydrated_html = render(&render_for_hydration(&client_state));
    assert!(hydrated_html.contains("data-hydrate-selected="));
    assert!(hydrated_html.contains('a'));
    assert!(hydrated_html.contains('b'));

    // クライアント側は data-hydrate-* 属性から状態を復元できる（ラウンドトリップ）。
    let restored = MultiAccordion::from_hydration_attrs(&client_state.hydration_attrs()).unwrap();
    assert_eq!(restored, client_state);

    // 項目単位の deselect（a のみ）。
    assert!(dispatch(&mut client_state, "deselect", "a"));
    assert!(!client_state.is_open("a"));
    assert!(client_state.is_open("b"));
}

#[test]
fn multi_accordion_component_toggle_cycle() {
    let mut a = MultiAccordion::default();
    assert!(dispatch(&mut a, "toggle", "a"));
    assert!(a.is_open("a"));
    assert!(dispatch(&mut a, "toggle", "a"));
    assert!(!a.is_open("a"));
    assert_eq!(a.expanded(), &[] as &[String]);
}

#[test]
fn multi_accordion_component_ignores_unknown_dispatch_action() {
    let mut a = MultiAccordion::default();
    dispatch(&mut a, "select", "a");
    assert!(!dispatch(&mut a, "unknown", "b"));
    assert!(a.is_open("a"));
}

#[test]
fn multi_accordion_xss_payload_in_dispatch_select_value_is_escaped_on_render_for_hydration() {
    let mut a = MultiAccordion::default();
    let payload = "\"><script>alert(1)</script>";
    assert!(dispatch(&mut a, "select", payload));

    let rendered = render(&render_for_hydration(&a));
    assert!(rendered.contains("data-hydrate-selected="));
    assert!(rendered.contains("&lt;script&gt;"));
    assert!(!rendered.contains("<script>alert(1)</script>"));
    assert!(!rendered.contains(r#""><script"#));
}
