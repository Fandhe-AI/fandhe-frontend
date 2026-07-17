//! `rws-wasm-full`: WASM 完全方式のクライアントランタイム。
//!
//! REQ-11（`docs/spec/04-requirements.md`）が既定とする「クライアントの
//! イベント処理・DOM 更新を Rust + WASM の safe な範囲に収める」方式の
//! 実装クレート。TASK-11.2 は 4 分割サブタスク（アーキテクチャ設計 #74・
//! イベント処理 #75・DOM 更新 #76・既定実装化と統合 #77）で構成され、
//! 本コミット時点（TASK-11.2b・#75）では [`events`] モジュールのみを提供する。
//!
//! `mount()`/`hydrate()` の公開 API・DOM 更新（差分描画）は #76/#77 のスコープ。
//! 本クレートの自作コードは safe Rust のみとし、`unsafe` は `wasm-bindgen` /
//! `web-sys` の FFI 境界（依存クレート内部・自動生成コード）に限定する
//! （`docs/unsafe-boundary.md` 第 2 節）。

pub mod events;
