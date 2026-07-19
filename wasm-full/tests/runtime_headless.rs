//! `rws_wasm_full::dispatch_and_render_headless` の native（rlib）統合テスト
//! （TASK-11.2d・#77）。
//!
//! [`rws_wasm_full::dispatch_and_render_headless`] は DOM・`wasm-bindgen` に
//! 依存しない DOM 非依存のヘッドレス補助 API（`docs/design/wasm-full-architecture.md`
//! 第 3.2 節）であり、native `cargo test` から wasm32 ターゲット・実 DOM を
//! 介さずに「dispatch 後の状態」を検証できる。
//!
//! `Runtime::mount`/`Runtime::hydrate`・イベント配線経由の
//! `should_repaint == false`（input 中の再描画スキップ）契約は
//! `web_sys::Element` に依存する wasm32 専用コードのため、本ファイルでは
//! 検証できない（`wasm-full/tests/runtime_browser.rs` の実ブラウザ統合テストへ
//! 委ねる）。本ファイルはヘッドレス経路（状態遷移・XSS 回帰）のみを扱う。

use rws_core::render;
use rws_interactive::AppState;
use rws_wasm_full::dispatch_and_render_headless;

/// dispatch 後の状態遷移が `Node` 木へ反映されること（`docs/design/wasm-full-architecture.md`
/// 第 3.2 節の `dispatch_and_render_headless` 契約）。
#[test]
fn dispatch_and_render_headless_reflects_state_after_dispatch() {
    let mut state = AppState::new();

    let node = dispatch_and_render_headless(&mut state, "increment", "");
    assert!(render(&node).contains("カウント: 1"));

    let node = dispatch_and_render_headless(&mut state, "increment", "");
    assert!(render(&node).contains("カウント: 2"));

    let node = dispatch_and_render_headless(&mut state, "decrement", "");
    assert!(render(&node).contains("カウント: 1"));
}

/// 未知アクションの安全側 no-op（`rws_interactive::dispatch` の不変条件 4）:
/// 状態を変更せず、その時点の描画を返す。
#[test]
fn dispatch_and_render_headless_is_noop_for_unknown_action() {
    let mut state = AppState::new();
    let before = render(&dispatch_and_render_headless(&mut state, "increment", ""));

    let after = render(&dispatch_and_render_headless(
        &mut state,
        "no_such_action",
        "payload",
    ));

    assert_eq!(
        before, after,
        "未知アクション後も dispatch_and_render_headless の出力が不変であること"
    );
}

/// REQ-1 回帰: `dispatch_and_render_headless` 経由でも既定エスケープが
/// 効いていること（`Runtime::mount`/`hydrate` の内部で使う `dom::paint` とは
/// 別経路だが、`rws_core::render` を通す点は共通の不変条件）。
#[test]
fn dispatch_and_render_headless_escapes_xss_payload() {
    let mut state = AppState::new();
    let payload = "<script>alert(1)</script>";

    dispatch_and_render_headless(&mut state, "set_draft", payload);
    let node = dispatch_and_render_headless(&mut state, "add_item", "");

    let html = render(&node);
    assert!(
        !html.contains("<script>alert"),
        "生の <script> タグが出力に含まれてはならない: {html}"
    );
    assert!(
        html.contains("&lt;script&gt;alert"),
        "エスケープ済みペイロードが出力に含まれること: {html}"
    );
}
