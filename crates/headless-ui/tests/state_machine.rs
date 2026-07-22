//! `fandhe-frontend-headless-ui` の状態機械（[`state`] モジュール、
//! イシュー #524・#595）の公開 API 経由の統合テスト。
//!
//! `crates/headless-ui/src/state.rs` 内の `#[cfg(test)]` ユニットテストが
//! 内部実装を含めた網羅を担うのに対し、本ファイルは
//! `fandhe-frontend-headless-ui` の公開 API（`lib.rs` の再エクスポート）
//! のみを経由し、Phase 2 の具象コンポーネント（Dialog / Switch 等）が実際に
//! 使う想定の外部からの利用形態を固定する回帰テスト。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::{Checkable, Disclosure, MultiSelect, OpenState, SingleSelect};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, DirtyTracked, Hydrate,
};

#[test]
fn disclosure_full_cycle_ssr_then_dispatch_then_hydration() {
    // SSR: 状態なし初期描画（Default = Closed）。
    let initial = Disclosure::default();
    let ssr_html = render(&initial.view());
    assert!(ssr_html.contains(r#"data-state="closed""#));
    assert!(!ssr_html.contains("data-hydrate-"));

    // クライアント側（wasm-full 相当）の dispatch でトグル。
    let mut client_state = initial;
    assert!(dispatch(&mut client_state, "toggle", ""));
    assert_eq!(client_state.state(), OpenState::Open);

    // 別の SSR リクエスト（open 状態）はハイドレーション属性込みで出力される。
    let hydrated_html = render(&render_for_hydration(&client_state));
    assert!(hydrated_html.contains(r#"data-state="open""#));
    assert!(hydrated_html.contains(r#"data-hydrate-state="open""#));

    // クライアント側は data-hydrate-* 属性から状態を復元できる（ラウンドトリップ）。
    let restored = Disclosure::from_hydration_attrs(&client_state.hydration_attrs()).unwrap();
    assert_eq!(restored, client_state);
}

#[test]
fn single_select_full_cycle_ssr_then_dispatch_then_hydration() {
    let initial = SingleSelect::default();
    let ssr_html = render(&initial.view());
    assert!(ssr_html.contains(r#"data-state="closed""#));
    assert!(!ssr_html.contains("data-hydrate-"));

    let mut client_state = initial;
    assert!(dispatch(&mut client_state, "select", "panel-1"));
    assert_eq!(client_state.selected(), Some("panel-1"));
    assert_eq!(client_state.item_data_state("panel-1"), "open");
    assert_eq!(client_state.item_data_state("panel-2"), "closed");

    let hydrated_html = render(&render_for_hydration(&client_state));
    assert!(hydrated_html.contains(r#"data-state="open""#));

    let restored = SingleSelect::from_hydration_attrs(&client_state.hydration_attrs()).unwrap();
    assert_eq!(restored, client_state);
}

#[test]
fn disclosure_and_single_select_ignore_unknown_dispatch_actions() {
    let mut disclosure = Disclosure::new(OpenState::Open);
    assert!(!dispatch(&mut disclosure, "unknown", "payload"));
    assert_eq!(disclosure.state(), OpenState::Open);

    let mut single_select = SingleSelect::default();
    dispatch(&mut single_select, "select", "a");
    assert!(!dispatch(&mut single_select, "unknown", "b"));
    assert_eq!(single_select.selected(), Some("a"));
}

#[test]
fn checkable_full_cycle_ssr_then_dispatch_then_hydration() {
    // SSR: 状態なし初期描画（Default = unchecked）。
    let initial = Checkable::default();
    let ssr_html = render(&initial.view());
    assert!(ssr_html.contains(r#"data-state="unchecked""#));
    assert!(!ssr_html.contains("data-hydrate-"));

    // クライアント側（wasm-full 相当）の dispatch でトグル。
    let mut client_state = initial;
    assert!(dispatch(&mut client_state, "toggle", ""));
    assert!(client_state.is_checked());

    // 別の SSR リクエスト（checked 状態）はハイドレーション属性込みで出力される。
    let hydrated_html = render(&render_for_hydration(&client_state));
    assert!(hydrated_html.contains(r#"data-state="checked""#));
    assert!(hydrated_html.contains(r#"data-hydrate-checked="checked""#));

    // クライアント側は data-hydrate-* 属性から状態を復元できる（ラウンドトリップ）。
    let restored = Checkable::from_hydration_attrs(&client_state.hydration_attrs()).unwrap();
    assert_eq!(restored, client_state);
}

#[test]
fn checkable_ignores_unknown_dispatch_action() {
    let mut checkable = Checkable::new(true);
    assert!(!dispatch(&mut checkable, "unknown", "payload"));
    assert!(checkable.is_checked());
}

#[test]
fn multi_select_full_cycle_ssr_then_dispatch_then_hydration() {
    // SSR: 状態なし初期描画（Default = 空選択）。
    let initial = MultiSelect::default();
    let ssr_html = render(&initial.view());
    assert!(ssr_html.contains(r#"data-state="closed""#));
    assert!(!ssr_html.contains("data-hydrate-"));

    // クライアント側（wasm-full 相当）の dispatch で複数項目を同時選択。
    let mut client_state = initial;
    assert!(dispatch(&mut client_state, "select", "panel-1"));
    assert!(dispatch(&mut client_state, "select", "panel-2"));
    assert_eq!(
        client_state.selected(),
        &["panel-1".to_string(), "panel-2".to_string()]
    );
    assert_eq!(client_state.item_data_state("panel-1"), "open");
    assert_eq!(client_state.item_data_state("panel-2"), "open");
    assert_eq!(client_state.item_data_state("panel-3"), "closed");

    // 別の SSR リクエスト（複数選択中）はハイドレーション属性込みで出力される。
    let hydrated_html = render(&render_for_hydration(&client_state));
    assert!(hydrated_html.contains(r#"data-state="open""#));
    assert!(hydrated_html.contains("data-hydrate-selected="));

    // クライアント側は data-hydrate-* 属性から状態を復元できる（ラウンドトリップ）。
    let restored = MultiSelect::from_hydration_attrs(&client_state.hydration_attrs()).unwrap();
    assert_eq!(restored, client_state);

    // 項目単位の deselect（全解除ではなく指定項目のみ閉じる）を確認する。
    assert!(dispatch(&mut client_state, "deselect", "panel-1"));
    assert_eq!(client_state.selected(), &["panel-2".to_string()]);
}

#[test]
fn multi_select_ignores_unknown_dispatch_action() {
    let mut multi_select = MultiSelect::default();
    dispatch(&mut multi_select, "select", "a");
    assert!(!dispatch(&mut multi_select, "unknown", "b"));
    assert_eq!(multi_select.selected(), &["a".to_string()]);
}

// --- DirtyTracked（イシュー #592） ------------------------------------
//
// `fandhe-frontend-wasm-full`/`fandhe-frontend-wasm-client` は `dispatch`
// （WASM 境界の文字列 dispatch 契約）経由で `Disclosure`/`SingleSelect` を
// 駆動し、直後に `dirty_fields()` を読んで `BindingTable`（束縛点対応表）へ
// 接続する。本テストは `fandhe-frontend-headless-ui` の公開 API のみを
// 経由してこの利用形態を固定する（`crates/headless-ui/src/state.rs` 内の
// ユニットテストは内部実装を含めた網羅を担う）。

#[test]
fn disclosure_dispatch_then_dirty_fields_reflects_changed_field_via_public_api() {
    let mut d = Disclosure::default();
    assert!(dispatch(&mut d, "toggle", ""));
    assert_eq!(d.dirty_fields(), &[Disclosure::FIELD_STATE]);
}

#[test]
fn single_select_dispatch_then_dirty_fields_reflects_changed_field_via_public_api() {
    let mut s = SingleSelect::default();
    assert!(dispatch(&mut s, "select", "panel-1"));
    assert_eq!(s.dirty_fields(), &[SingleSelect::FIELD_SELECTED]);
}
