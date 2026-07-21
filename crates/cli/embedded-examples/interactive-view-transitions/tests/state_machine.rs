//! `examples/interactive-view-transitions` の integration test。
//!
//! `fandhe_frontend_interactive` の状態機械 API（`dispatch` / `decode_action` /
//! `render_for_hydration`）の契約を、このサンプルが実演する範囲で固定する。
//! `src/main.rs` はバイナリクレートのため本ファイルからは `use` できず、
//! `fandhe_frontend_interactive::AppState`（クレート公開の参照コンポーネント）
//! を直接使う（`examples/ssr-routing/tests/routing.rs` と同じ方針）。

use fandhe_frontend_core::render;
use fandhe_frontend_interactive::{dispatch, render_for_hydration, AppState};

/// `dispatch("increment")` で counter が増え、戻り値が `true`
/// （decode_action が既知アクションを復号できた）になることを固定する。
#[test]
fn dispatch_increment_increases_counter_and_returns_true() {
    let mut state = AppState::new();
    let before = state.counter;

    let applied = dispatch(&mut state, "increment", "");

    assert!(applied);
    assert_eq!(state.counter, before + 1);
}

/// 未知アクション名は `decode_action` の復号失敗として no-op になり、
/// `dispatch` は `false` を返し状態を変更しない（不変条件 4、安全側
/// フォールバック）。
#[test]
fn dispatch_unknown_action_is_no_op_and_returns_false() {
    let mut state = AppState::new();
    let before = state.clone();

    let applied = dispatch(&mut state, "no-such-action", "");

    assert!(!applied);
    assert_eq!(state, before);
}

/// `render_for_hydration` はルート要素へ `data-hydrate-*` 属性を付与する
/// （`AppState::hydration_attrs` の契約、`HYDRATE_ATTR_PREFIX` 参照）。
#[test]
fn render_for_hydration_adds_hydrate_attrs_to_root_element() {
    let state = AppState::new();

    let node = render_for_hydration(&state);
    let html = render(&node);

    assert!(html.contains("data-hydrate-counter="), "html was: {html}");
    assert!(
        html.contains(r#"id="interactive-root""#),
        "html was: {html}"
    );
}

/// 既定エスケープ回帰（REQ-1）: `<script>` を含む draft を `set_draft` で
/// 反映したのち `render_for_hydration` の出力に、生の `<script>` タグとして
/// 現れないことを固定する（ハイドレーション属性値のエスケープも含む）。
#[test]
fn render_for_hydration_escapes_script_payload_in_draft_and_items() {
    let mut state = AppState::new();
    let payload = "<script>alert(1)</script>";

    dispatch(&mut state, "set_draft", payload);
    dispatch(&mut state, "add_item", "");

    let node = render_for_hydration(&state);
    let html = render(&node);

    assert!(
        !html.contains("<script>alert"),
        "raw <script> tag leaked into rendered HTML: {html}"
    );
    assert!(
        html.contains("&lt;script&gt;"),
        "expected escaped script payload in html: {html}"
    );
}
