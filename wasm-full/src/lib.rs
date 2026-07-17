//! `rws-wasm-full`: WASM 完全方式のクライアントランタイム。
//!
//! REQ-11（`docs/spec/04-requirements.md`）が既定とする「クライアントの
//! イベント処理・DOM 更新を Rust + WASM の safe な範囲に収める」方式の
//! 実装クレート。TASK-11.2 は 4 分割サブタスク（アーキテクチャ設計 #74・
//! イベント処理 #75・DOM 更新 #76・既定実装化と統合 #77）で構成される。
//!
//! 本コミット時点（TASK-11.2b・#75 マージ済み／TASK-11.2c・#76）では
//! [`events`]（イベント委譲配線）と [`dom::render_component_html`]（DOM 非依存の
//! 描画純粋関数）を提供する。`Runtime<C>`・`mount()`/`hydrate()` の公開 API と
//! `set_inner_html` を伴う `paint()` 本体・イベント配線との統合は TASK-11.2d
//! （#77）のスコープ（`docs/wasm-full-architecture.md` 第 3.1 節）。
//!
//! 本クレートの自作コードは safe Rust のみとし、`unsafe` は `wasm-bindgen` /
//! `web-sys` の FFI 境界（依存クレート内部・自動生成コード）に限定する
//! （`docs/unsafe-boundary.md` 第 2 節）。

pub mod events;

mod dom;

// integration test（`tests/dom_update.rs`）から呼べるよう再エクスポートする。
// `dom` モジュール自体は crate 内部実装（`docs/wasm-full-architecture.md` 第 3.1 節の
// 「内部」区分）のため非 pub のままとし、公開面はこの再エクスポートのみに絞る。
pub use dom::render_component_html;
