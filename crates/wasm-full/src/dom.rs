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

/// `element.set_attribute(name, value)` の薄いガード付きラッパー
/// （イシュー #401 の `fw gate` `url_validation_check` 契約に準拠、
/// `.claude/rules/security.md`）。各配線モジュール（`sidebar`/
/// `focus_visible`/`focus_trap`/`position`/`message_scroller` 等）が
/// それぞれ同一実装を独自に持っていたため、本モジュールへ共通化した
/// （REQ-11 gzip バンドルサイズ抑制、イシュー #2122 レビュー指摘）。
/// 呼び出し側は多くの場合 `&'static str` リテラルの属性名・値のみを
/// 渡すが、将来 DOM/アプリ由来の動的な `name`/`value` が渡されても
/// `fandhe_frontend_core::url` のガード関数群
/// （`is_event_handler_attr`/`is_url_attr`/`is_safe_url`/
/// `is_safe_srcset`）を経由する防御を保つ。
#[cfg(target_arch = "wasm32")]
pub(crate) fn set_dom_attribute(element: &web_sys::Element, name: &str, value: &str) {
    if fandhe_frontend_core::is_event_handler_attr(name) {
        return;
    }
    if fandhe_frontend_core::is_url_attr(name) && !fandhe_frontend_core::is_safe_url(value) {
        return;
    }
    if name.eq_ignore_ascii_case("srcset") && !fandhe_frontend_core::is_safe_srcset(value) {
        return;
    }
    let _ = element.set_attribute(name, value);
}

/// `start` から `root`（含む）まで祖先方向へ辿り、`data-scope`/`data-part`
/// が指定値と一致する最初の要素を返す。多数の配線モジュール
/// （`questionnaire`/`headless_timer`/`splitter`/`angle_slider`/`sidebar`/
/// `headless_clipboard`/`message_scroller` 等）がそれぞれ同一実装を独自に
/// 持っていたため、本モジュールへ共通化した（REQ-11 gzip バンドルサイズ
/// 抑制、イシュー #2122 レビュー指摘）。
#[cfg(target_arch = "wasm32")]
pub(crate) fn closest_matching(
    root: &web_sys::Element,
    start: &web_sys::Element,
    scope: &str,
    part: &str,
) -> Option<web_sys::Element> {
    let mut current = Some(start.clone());
    while let Some(element) = current {
        if !root.contains(Some(&element)) {
            break;
        }
        if element.get_attribute("data-scope").as_deref() == Some(scope)
            && element.get_attribute("data-part").as_deref() == Some(part)
        {
            return Some(element);
        }
        if element == *root {
            break;
        }
        current = element.parent_element();
    }
    None
}

/// [`set_dom_attribute`] の `Result` 版（`element.set_attribute` の失敗を
/// 呼び出し元へ伝播したい配線層向け）。`headless_select`/`questionnaire`/
/// `headless_timer`/`headless_file_upload`/`headless_clipboard`/
/// `headless_avatar` が独自に持っていた同一実装を共通化した（REQ-11 gzip
/// バンドルサイズ抑制、イシュー #2122 レビュー指摘）。
#[cfg(target_arch = "wasm32")]
pub(crate) fn set_dom_attribute_result(
    element: &web_sys::Element,
    name: &str,
    value: &str,
) -> Result<(), wasm_bindgen::JsValue> {
    if fandhe_frontend_core::is_event_handler_attr(name) {
        return Ok(());
    }
    if fandhe_frontend_core::is_url_attr(name) && !fandhe_frontend_core::is_safe_url(value) {
        return Ok(());
    }
    if name.eq_ignore_ascii_case("srcset") && !fandhe_frontend_core::is_safe_srcset(value) {
        return Ok(());
    }
    element.set_attribute(name, value)
}

/// `start` から `root`（含む）まで祖先方向を辿り、`data-disabled` を持つ
/// 要素が 1 つでもあれば `true`（disabled な祖先・`root` 自身を境界とする
/// fail-closed 判定）。`sidebar`/`splitter`/`number_input` が独自に持って
/// いた同一実装を共通化した（REQ-11 gzip バンドルサイズ抑制、イシュー
/// #2122 レビュー指摘）。
#[cfg(target_arch = "wasm32")]
pub(crate) fn has_disabled_ancestor(root: &web_sys::Element, start: &web_sys::Element) -> bool {
    let mut current = Some(start.clone());
    while let Some(element) = current {
        if element.has_attribute("data-disabled") {
            return true;
        }
        if !root.contains(Some(&element)) || element == *root {
            break;
        }
        current = element.parent_element();
    }
    false
}
