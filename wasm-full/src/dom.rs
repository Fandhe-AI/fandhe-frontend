//! DOM 更新の内部モジュール（`docs/design/wasm-full-architecture.md` 第 3.1 節）。
//!
//! 本モジュールは DOM を一切参照しない**純粋関数** [`render_component_html`]
//! （TASK-11.2c・#76）と、それを `web_sys::Element::set_inner_html` へ適用する
//! 薄い層 [`paint`]（TASK-11.2d・#77、`crate::lib::Runtime` の `mount`/`hydrate`
//! フォールバック経路から呼ばれる）を提供する。文字列生成（エスケープ検証可能・
//! native テスト可）と DOM 適用（wasm 実行時のみ意味を持つ）を分離する構成は
//! 維持したまま、[`paint`] のみ `#[cfg(target_arch = "wasm32")]` でゲートする
//! （`events.rs`/`hydration.rs` と同じ 2 層構成方針）。
//!
//! # 契約（`rws-wasm-full` 全体の不変条件、同書第 7 節・不変条件 1）
//!
//! [`render_component_html`] が返す文字列は [`rws_core::render`] の既定
//! エスケープ済み出力のみであり、`format!` 等による HTML 文字列直接組み立てや
//! `rws_core::raw_html()` の呼び出しを一切行わない。[`paint`] はこの関数
//! （または同等の `rws_core::render` 呼び出し）の出力のみを `set_inner_html`
//! へ渡す契約を守り、独自にエスケープや文字列組み立てを行わない。

use rws_interactive::Component;

/// コンポーネントの現在状態を既定エスケープ済み HTML 文字列へ変換する。
///
/// `component.view()`（`rws_core::Node` 木）を [`rws_core::render`] に通すだけの
/// 純粋関数。DOM・`wasm-bindgen` に一切依存しないため、native（`rlib`）テストで
/// XSS 回帰・dispatch 後の再描画内容を検証できる
/// （`wasm-full/tests/dom_update.rs` 参照）。
///
/// [`paint`]（`web-sys::Element::set_inner_html` を呼ぶ薄い層）は、この関数の
/// 戻り値をそのまま渡す想定であり、`paint` 自体は独自にエスケープや文字列
/// 組み立てを行わない。
pub fn render_component_html<C: Component>(component: &C) -> String {
    rws_core::render(&component.view())
}

/// [`render_component_html`] の出力を `root` へ反映する薄い層（TASK-11.2d・#77）。
///
/// `crate::Runtime::mount`（CSR 経路）・`crate::Runtime::hydrate`（ハイドレーション
/// 失敗時の CSR フォールバック経路、`docs/design/wasm-full-architecture.md` 第 4 節・
/// 判断 5）から呼ばれる。この関数自体は文字列生成を一切行わず、
/// [`render_component_html`] の戻り値を `set_inner_html` へ渡すだけであるため、
/// XSS 保証（既定エスケープ済み出力のみを DOM へ挿入する不変条件）は呼び出し元
/// ではなく [`render_component_html`] 側に閉じている。
#[cfg(target_arch = "wasm32")]
pub(crate) fn paint<C: Component>(root: &web_sys::Element, component: &C) {
    root.set_inner_html(&render_component_html(component));
}
