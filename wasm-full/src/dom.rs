//! DOM 更新の内部モジュール（`docs/wasm-full-architecture.md` 第 3.1 節）。
//!
//! 本モジュールは `paint()`（`rws_core::render()` 出力への `set_inner_html`
//! 適用）の実装を TASK-11.2c（#76）として担当するが、`set_inner_html` を
//! 呼び出す層（`web-sys::Element` 依存）・イベント配線（[`crate::events`]）との
//! 統合は TASK-11.2d（#77）のスコープとする
//! （`wasm-full/Cargo.toml` 冒頭コメント・`lib.rs` モジュールコメント参照）。
//!
//! 現時点で提供するのは、DOM を一切参照しない**純粋関数**
//! [`render_component_html`] のみである。`paint()` はこの関数の出力を
//! `set_inner_html` へ渡すだけの薄い層として設計されており（同書第 7 節・
//! 不変条件 1）、文字列生成（エスケープ検証可能・native テスト可）と
//! DOM 適用（wasm 実行時のみ意味を持つ）を分離する本モジュールの構成は
//! その分離を先取りする形になっている。
//!
//! # 契約（`rws-wasm-full` 全体の不変条件、同書第 7 節・不変条件 1）
//!
//! この関数が返す文字列は [`rws_core::render`] の既定エスケープ済み出力の
//! みであり、`format!` 等による HTML 文字列直接組み立てや
//! `rws_core::raw_html()` の呼び出しを一切行わない。将来追加される `paint()`
//! は、この関数（または同等の `rws_core::render` 呼び出し）の出力のみを
//! `set_inner_html` へ渡す契約を守ること。

use rws_interactive::Component;

/// コンポーネントの現在状態を既定エスケープ済み HTML 文字列へ変換する。
///
/// `component.view()`（`rws_core::Node` 木）を [`rws_core::render`] に通すだけの
/// 純粋関数。DOM・`wasm-bindgen` に一切依存しないため、native（`rlib`）テストで
/// XSS 回帰・dispatch 後の再描画内容を検証できる
/// （`wasm-full/tests/dom_update.rs` 参照）。
///
/// 将来追加される `paint()`（TASK-11.2d・#77、`web-sys::Element::set_inner_html`
/// を呼ぶ薄い層）は、この関数の戻り値をそのまま渡す想定であり、`paint()` 自体は
/// 独自にエスケープや文字列組み立てを行わない。
pub fn render_component_html<C: Component>(component: &C) -> String {
    rws_core::render(&component.view())
}
