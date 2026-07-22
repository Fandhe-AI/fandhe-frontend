//! `fandhe-frontend-headless-ui` の開閉状態機械（[`state`] モジュール、
//! イシュー #524）の公開 API 経由の統合テスト。
//!
//! `crates/headless-ui/src/state.rs` 内の `#[cfg(test)]` ユニットテストが
//! 内部実装を含めた網羅を担うのに対し、本ファイルは
//! `fandhe-frontend-headless-ui` の公開 API（`lib.rs` の再エクスポート）
//! のみを経由し、Phase 2 の具象コンポーネント（Dialog 等）が実際に使う
//! 想定の外部からの利用形態を固定する回帰テスト。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::{Disclosure, OpenState, SingleSelect};
use fandhe_frontend_interactive::{dispatch, render_for_hydration, Component, Hydrate};

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
