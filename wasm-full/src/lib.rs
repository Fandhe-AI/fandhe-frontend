//! `rws-wasm-full`: WASM 完全方式（イベント処理・DOM 更新を Rust/web-sys 側で
//! 行うクライアントランタイム）のクレート入口。
//!
//! 設計の正は `docs/wasm-full-architecture.md`（TASK-11.2a・#198 でマージ済み）。
//! 実装と本書に乖離が生じた場合は同書を正とする。
//!
//! # 本ファイルのスコープ（TASK-11.2c・#76）
//!
//! `docs/wasm-full-architecture.md` 第 3.1 節はモジュール構成を次のように
//! 割り当てている。
//!
//! - `lib.rs`（`Runtime<C>` 定義・公開 API）: TASK-11.2b（#75）/ TASK-11.2c（#76）
//! - `events`（内部・イベント委譲配線）: TASK-11.2b（#75）
//! - `dom`（内部・`paint()`）: TASK-11.2c（#76）＝本タスク
//!
//! 調査時点（2026-07-17）で TASK-11.2b（#75）は未マージであり、
//! `Runtime<C>` 骨格・`events` モジュールがまだ存在しない。そのため本コミットは
//! 実装計画の安全側フォールバックに従い、`dom` モジュールが提供する純粋関数
//! （DOM 非依存・`web-sys` 不使用）である [`dom::render_component_html`] のみを
//! 実装する。`set_inner_html` を伴う `dom::paint`（`web-sys::Element` 依存）と
//! `Runtime::mount` への統合は、#75 マージ後に rebase して追加する
//! （`Cargo.toml` 冒頭のコメント参照。CI の `forbid-unsafe` ジョブが
//! `cargo check --workspace` を絞り込みなしで実行するため、`web-sys` 依存の
//! 追加は #75 側の CI 調整とセットでなければ workspace 全体を壊す）。
//!
//! `events` モジュール・イベント委譲配線の実装は本タスクのスコープ外であり、
//! 責務混線を避けるため先取りしない（#75 のスコープ）。

// `wasm-bindgen` 生成コードの unsafe を許容する必要が生じる将来（#75 以降）に
// 備え、`docs/wasm-full-architecture.md` 第 2 節の方針に従い `forbid` ではなく
// `deny` を採用する。本コミット時点では自作コード・依存とも unsafe を含まない。
#![deny(unsafe_code)]
#![warn(missing_docs)]

mod dom;

// integration test（`tests/dom_update.rs`）から呼べるよう再エクスポートする。
// `dom` モジュール自体は crate 内部実装（`docs/wasm-full-architecture.md` 第 3.1 節の
// 「内部」区分）のため非 pub のままとし、公開面はこの再エクスポートのみに絞る。
pub use dom::render_component_html;
