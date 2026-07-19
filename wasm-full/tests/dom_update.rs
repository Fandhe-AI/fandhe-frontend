//! `wasm-full/src/dom.rs` の native（rlib）統合テスト（TASK-11.2c・#76）。
//!
//! `dom::render_component_html`（`lib.rs` で再エクスポート）は DOM・
//! `wasm-bindgen` に依存しない純粋関数のため、native `cargo test` で
//! REQ-11 受け入れ基準「クライアント WASM のイベント処理・DOM 更新を
//! 経由した出力にも同一のエスケープ保証が及ぶこと（REQ-1 関連）」を検証できる。
//! （`docs/design/wasm-full-architecture.md` 第 5 節・テスト設計 §5 参照）。
//!
//! `set_inner_html` を伴う `paint()` の実ブラウザ検証は本コミット時点では
//! 未実装であり、TASK-11.2d（#77）の統合テストへ引き継ぐ（実装計画 §5-4）。

use rws_interactive::{dispatch, AppState};
use rws_wasm_full::render_component_html;

/// REQ-1 回帰: 状態にスクリプトタグ等の XSS ペイロードを持たせても、
/// `render_component_html` の出力に生の `<script>` タグが現れないこと
/// （`rws_core::render` の既定エスケープが `dom` モジュール経由でも
/// 効いていることの確認）。
#[test]
fn render_component_html_escapes_xss_payload_in_list_items() {
    let mut state = AppState::new();
    let payload = "<script>alert(1)</script>";

    // draft へ XSS ペイロードを設定し AddItem で items へ確定させる。
    // items[i] は `text(item.clone())` としてテキストノードに載るため
    // （interactive/src/lib.rs の render_with_root_attrs）、既定エスケープの
    // 対象になる。
    assert!(dispatch(&mut state, "set_draft", payload));
    assert!(dispatch(&mut state, "add_item", ""));

    let html = render_component_html(&state);

    assert!(
        !html.contains("<script>"),
        "生の <script> タグが出力に含まれてはならない: {html}"
    );
    assert!(
        html.contains("&lt;script&gt;"),
        "エスケープ済みペイロードが出力に含まれること: {html}"
    );
}

/// REQ-1 回帰: 属性値（`value="..."`）経由でも `"` `'` `&` 等がエスケープされ、
/// 属性境界を破壊する生の引用符が出力に現れないこと。
#[test]
fn render_component_html_escapes_xss_payload_in_attribute_value() {
    let mut state = AppState::new();
    let payload = "\" onmouseover=\"alert(1)\" data-x=\"'&";

    assert!(dispatch(&mut state, "set_draft", payload));

    let html = render_component_html(&state);

    assert!(
        !html.contains("onmouseover=\"alert(1)\""),
        "属性値の生の引用符による属性注入が発生してはならない: {html}"
    );
    assert!(
        html.contains("&quot;"),
        "二重引用符がエスケープされること: {html}"
    );
}

/// dispatch → 再描画のラウンドトリップ: 状態遷移後の
/// `render_component_html` 出力が遷移後の状態を反映すること
/// （`docs/design/wasm-full-architecture.md` 第 3.2 節の
/// `dispatch_and_render_headless` 相当の経路を `dom` モジュール側からも確認）。
#[test]
fn render_component_html_reflects_state_after_dispatch() {
    let mut state = AppState::new();
    assert!(dispatch(&mut state, "increment", ""));
    assert!(dispatch(&mut state, "increment", ""));
    assert!(dispatch(&mut state, "decrement", ""));

    let html = render_component_html(&state);

    // カウンター値はイシュー #345 で静的テキストと分離した束縛点
    // （`<span data-bind-text="counter">`）に出力されるため、
    // 「カウント: 1」は連続した部分文字列にならない
    // （`interactive/src/lib.rs` の `render_with_root_attrs` 参照）。
    assert!(
        html.contains(r#"data-bind-text="counter">1</span>"#),
        "dispatch 後の状態（counter=1）が描画へ反映されること: {html}"
    );
}

/// 未知アクションの安全側 no-op: `dispatch` が `false` を返した場合、
/// `render_component_html` の出力が dispatch 前と変わらないこと
/// （`rws-interactive` 不変条件 4 の継承確認、実装計画 §5-3）。
#[test]
fn render_component_html_is_unchanged_after_unknown_action_noop() {
    let mut state = AppState::new();
    let before = render_component_html(&state);

    let dispatched = dispatch(&mut state, "unknown_action", "payload");

    let after = render_component_html(&state);

    assert!(!dispatched, "未知アクションは dispatch が false を返すこと");
    assert_eq!(before, after, "未知アクション後も描画出力が不変であること");
}
