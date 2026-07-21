//! `examples/interactive-view-transitions` の integration test。
//!
//! `fandhe_frontend_interactive` の状態機械 API（`dispatch` / `decode_action` /
//! `render_for_hydration`）の契約を、このサンプルが実演する範囲で固定する。
//! `src/main.rs` はバイナリクレートのため本ファイルからは `use` できず、
//! `fandhe_frontend_interactive::AppState`（クレート公開の参照コンポーネント）
//! を直接使う（`examples/ssr-routing/tests/routing.rs` と同じ方針）。

use fandhe_frontend_core::render;
use fandhe_frontend_interactive::{dispatch, render_for_hydration, AppState};
use std::path::Path;

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

/// `static/embed.html` の回帰テスト（PR #510 Bugbot 指摘、review comment
/// 3621300109 "Hydrate mount id collides"）。
///
/// `#interactive-root` を空のまま `hydrate("interactive-root")` を呼ぶと、
/// 状態復元（`hydration::restore_state`）が `data-hydrate-*` 属性なしで
/// 失敗し、CSR フォールバック（`dom::mount_initial`）が `AppState::view()`
/// （自身も `id="interactive-root"` を持つ）をこの `<div>` の中へ丸ごと
/// 差し込んでしまい、同一 id が入れ子で重複する。これを防ぐには
/// `#interactive-root` があらかじめ `data-hydrate-*` 属性付きの SSR
/// 済みマークアップを保持し、`hydrate()` の状態復元が成功する経路のみを
/// 通ることが必須（`dom::mount_initial` を一切呼ばせない）。本テストは
/// その前提となる属性の存在をファイル内容の静的検査で固定する
/// （wasm 実行を伴わない native テストのため、ブラウザでの実際の
/// `hydrate()` 呼び出し結果までは検証できない点に注意）。
#[test]
fn embed_html_interactive_root_has_hydrate_attrs_to_avoid_csr_fallback_id_collision() {
    let embed_html_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("static/embed.html");
    let html = std::fs::read_to_string(&embed_html_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", embed_html_path.display()));

    // 実タグのみを対象にする。`<head>`/`<body>` 双方のコメント内に
    // `<div id="interactive-root">`（属性なしの例示文字列）が出現するため、
    // 実タグにのみ付く `data-testid="interactive-root"` まで含めた接頭辞で
    // 開始位置を一意に特定する（`AppState::view()` / `render_for_hydration`
    // が常に id の直後にこの属性を出力する契約、interactive/src/lib.rs
    // 参照）。開始タグを閉じる最初の `>` までを属性検査対象とする
    // （マルチバイト文字境界を跨がないよう、バイト固定長ではなく `>` の
    // 位置で区切る）。
    let root_start = html
        .find(r#"<div id="interactive-root" data-testid="interactive-root""#)
        .expect(
            "static/embed.html must contain the actual \
             <div id=\"interactive-root\" data-testid=\"interactive-root\" ...> mount tag \
             (not just a mention in a comment)",
        );
    let tag_end = html[root_start..]
        .find('>')
        .map(|offset| root_start + offset)
        .expect("static/embed.html #interactive-root start tag must be closed with '>'");
    let tag_slice = &html[root_start..tag_end];

    for attr in [
        "data-hydrate-counter=",
        "data-hydrate-draft=",
        "data-hydrate-items=",
        "data-hydrate-item-ids=",
    ] {
        assert!(
            tag_slice.contains(attr),
            "static/embed.html の #interactive-root に {attr} がありません。\
             空のまま hydrate() を呼ぶと CSR フォールバックが AppState::view() を \
             二重に差し込み id 衝突が再発します（PR #510 Bugbot 指摘の回帰）。\
             tag_slice was: {tag_slice}"
        );
    }
}
