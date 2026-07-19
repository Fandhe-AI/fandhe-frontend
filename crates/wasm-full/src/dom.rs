//! DOM 更新の内部モジュール（`docs/design/wasm-full-architecture.md` 第 3.1 節）。
//!
//! 本モジュールは DOM を一切参照しない**純粋関数** [`render_component_html`]
//! （TASK-11.2c・#76）と、それを `web_sys::Element::set_inner_html` へ適用する
//! 薄い層 [`mount_initial`]（旧 `paint`。イシュー #345 で「初回マウント限定
//! API」へ改名・限定した。下記契約参照）を提供する。文字列生成（エスケープ
//! 検証可能・native テスト可）と DOM 適用（wasm 実行時のみ意味を持つ）を
//! 分離する構成は維持したまま、[`mount_initial`] のみ
//! `#[cfg(target_arch = "wasm32")]` でゲートする（`events.rs`/`hydration.rs` と
//! 同じ 2 層構成方針）。
//!
//! # 契約（`fandhe-frontend-wasm-full` 全体の不変条件、同書第 7 節・不変条件 1）
//!
//! [`render_component_html`] が返す文字列は [`fandhe_frontend_core::render`] の既定
//! エスケープ済み出力のみであり、`format!` 等による HTML 文字列直接組み立てや
//! `fandhe_frontend_core::raw_html()` の呼び出しを一切行わない。[`mount_initial`] はこの
//! 関数（または同等の `fandhe_frontend_core::render` 呼び出し）の出力のみを
//! `set_inner_html` へ渡す契約を守り、独自にエスケープや文字列組み立てを
//! 行わない。
//!
//! # `set_inner_html` 全置換の撤去（イシュー #345）
//!
//! `crate::Runtime::wire`（イベント後更新）は #345 以降 `set_inner_html` を
//! 一切呼ばない。イベント後の更新は
//! `fandhe_frontend_wasm_client::BindingTable::apply_update`（テキスト・属性・class の
//! 束縛点更新）と `fandhe_frontend_wasm_client::{find_list_element, apply_keyed_list}`
//! （keyed list の構造変化）に置き換わっている（`crate::lib` 参照）。
//! [`mount_initial`] は「`fandhe_frontend_core::render` 出力のみを渡す初回マウント限定
//! API」として、`Runtime::mount`（CSR 初回描画）・`Runtime::hydrate`
//! （ハイドレーション属性が読めない・不正な場合の CSR フォールバック、
//! いずれも「DOM がまだ何も反映されていない」状態からの初期構築）からのみ
//! 呼ばれる。**イベント後更新の経路からは呼ばない。**
//! `grep -rn set_inner_html wasm-full/src wasm-client/src` で本関数 1 箇所
//! のみが該当することを受け入れ条件 1 の機械確認とする（`wasm-client` 側の
//! `mount_csr` は別機能・別イシュー〈REQ-6 最小ハイドレーション、#48〉の
//! 既存 CSR エントリポイントであり、本イシューのスコープ外）。

use fandhe_frontend_interactive::Component;

/// コンポーネントの現在状態を既定エスケープ済み HTML 文字列へ変換する。
///
/// `component.view()`（`fandhe_frontend_core::Node` 木）を [`fandhe_frontend_core::render`] に通すだけの
/// 純粋関数。DOM・`wasm-bindgen` に一切依存しないため、native（`rlib`）テストで
/// XSS 回帰・dispatch 後の再描画内容を検証できる
/// （`wasm-full/tests/dom_update.rs` 参照）。
///
/// [`mount_initial`]（`web-sys::Element::set_inner_html` を呼ぶ薄い層）は、
/// この関数の戻り値をそのまま渡す想定であり、`mount_initial` 自体は独自に
/// エスケープや文字列組み立てを行わない。
pub fn render_component_html<C: Component>(component: &C) -> String {
    fandhe_frontend_core::render(&component.view())
}

/// [`render_component_html`] の出力を `root` へ反映する、**初回マウント限定**
/// の薄い層（旧 `paint`。イシュー #345 で改名・限定 API 化。上記モジュール
/// doc の「`set_inner_html` 全置換の撤去」節参照）。
///
/// `crate::Runtime::mount`（CSR 初回描画）・`crate::Runtime::hydrate`
/// （ハイドレーション属性が読めない場合の CSR フォールバック、
/// `docs/design/wasm-full-architecture.md` 第 4 節・判断 5）からのみ呼ばれる。
/// **イベント後の更新（`crate::lib::Runtime::wire`）からは呼ばない** —
/// フォーカス・入力途中の値・スクロール位置・IME 状態を破壊するため（#345 の
/// 実装動機そのもの）。
///
/// この関数自体は文字列生成を一切行わず、[`render_component_html`] の
/// 戻り値を `set_inner_html` へ渡すだけであるため、XSS 保証（既定エスケープ
/// 済み出力のみを DOM へ挿入する不変条件）は呼び出し元ではなく
/// [`render_component_html`] 側に閉じている。
#[cfg(target_arch = "wasm32")]
pub(crate) fn mount_initial<C: Component>(root: &web_sys::Element, component: &C) {
    root.set_inner_html(&render_component_html(component));
}
