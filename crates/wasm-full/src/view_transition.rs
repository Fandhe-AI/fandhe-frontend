//! `document.startViewTransition()` ラップの共有実装（イシュー #2400）。
//!
//! [`nav`](crate::nav) モジュールがイシュー #404 で router 遷移専用に導入した
//! `with_view_transition` を、本モジュールへ切り出して
//! `Runtime::apply_with_view_transition`（任意の状態更新から呼べる新規公開
//! API）と共有する。本モジュール自体・[`with_view_transition`] は feature
//! ゲートを持たない（`nav.rs` 側の router 経由呼び出しが feature の有無に
//! 関わらず無条件で動作し続けるという受け入れ条件のため。feature
//! `"view-transitions"` でゲートされるのは呼び出し元の
//! `Runtime::apply_with_view_transition` のみ）。

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::Document;

/// `document.startViewTransition` の機能検出・呼び出し専用の duck-typing
/// extern バインディング（イシュー #404、#2400 で `nav.rs` から移設）。
///
/// web-sys 0.3 系の `Document::start_view_transition` は
/// `#[cfg(web_sys_unstable_apis)]` ゲート付きであり、有効化には
/// `RUSTFLAGS='--cfg web_sys_unstable_apis'` をワークスペース全体へ適用
/// する必要がある（共有 `CARGO_TARGET_DIR` 運用・他クレートのビルド
/// フラグ汚染を招くため不採用、`docs/design/wasm-full-architecture.md`
/// 第 4 節・判断 10）。本 extern ブロックは安定版 wasm-bindgen のみで
/// 完結し、ソーステキスト上に `unsafe` トークンを含まない（マクロ展開後の
/// グルーコードのみが `unsafe` を含む、`docs/policy/unsafe-boundary.md`
/// 第 2 節の許容境界 2 点目と同区分）。新規外部パッケージの追加はゼロ
/// （既存の `wasm-bindgen`/`web-sys` のみで完結）。
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    type DocumentViewTransitions;

    /// 機能検出用: `document.startViewTransition` プロパティの取得。
    /// 非対応ブラウザでは `undefined`（`JsValue::is_function()` が
    /// `false` を返す）になる。
    #[wasm_bindgen::prelude::wasm_bindgen(method, getter, js_name = startViewTransition)]
    fn start_view_transition_prop(this: &DocumentViewTransitions) -> JsValue;

    /// `document.startViewTransition(updateCallback)` の呼び出し。
    /// `updateCallback` は同期または Promise を返す関数（本モジュールでは
    /// 常に同期の update コールバックを渡す）。呼び出しが throw した場合は
    /// `catch` 属性により `Err` として返る。
    #[wasm_bindgen::prelude::wasm_bindgen(method, catch, js_name = startViewTransition)]
    fn start_view_transition(
        this: &DocumentViewTransitions,
        update: &JsValue,
    ) -> Result<JsValue, JsValue>;
}

/// `document.startViewTransition` が関数として利用可能か（機能検出）を
/// 返す（イシュー #2536、[`with_view_transition`] 内の判定を抽出。
/// `crate::shared_layout` の VT 委譲判定——View Transitions が使える場合は
/// 共有レイアウト遷移を起動しない——が同じ判定を必要とするため共有する）。
#[cfg(target_arch = "wasm32")]
pub(crate) fn is_supported(document: &Document) -> bool {
    let doc_vt = document.clone().unchecked_into::<DocumentViewTransitions>();
    doc_vt.start_view_transition_prop().is_function()
}

/// `apply`（DOM 差し替え等の副作用のみを行うクロージャ）を
/// `document.startViewTransition()` でラップして呼び出す（イシュー #404、
/// #2400 で `nav.rs` から移設し [`crate::Runtime::apply_with_view_transition`]
/// と共有）。
///
/// `document` が `startViewTransition` を関数として持たない場合
/// （非対応ブラウザ、機能検出）、または呼び出し自体が throw した場合は
/// `apply` を同期的に直接実行する（graceful degradation。遷移が
/// 失敗しても描画は必ず完了させる、fail-closed にしない）。
///
/// `apply` を `Rc<RefCell<Option<F>>>` で包み、update コールバック
/// （`Closure::once_into_js` で JS 側へ所有権を移し、呼び出し後に自己
/// 解放する。`forget` 不使用）と throw 時フォールバックの双方から
/// 「`take()` できた側のみが 1 回だけ実行する」形にすることで、
/// 呼び出しが throw した場合でも `apply` を確実に一度だけ実行する
/// （update コールバックが呼ばれずに throw するケースへの対処。
/// `startViewTransition` の update コールバックは遷移がスキップされても
/// 仕様上必ず一度呼ばれるため、通常経路では throw 側の `take()` は
/// 常に空になり二重実行は起きない）。
///
/// `preset`（イシュー #2516）は
/// [`crate::view_transition_preset::VIEW_TRANSITION_PRESET_ATTR`] の
/// 設定/除去を一元管理する。`Some` なら該当プリセットの属性値を設定し、
/// `None` なら既存の属性を除去する。この除去は呼び出し元ごとの対処
/// ではなく本共有関数が無条件で行うため、[`crate::nav`] の router 遷移
/// （常に `None` で呼ぶ）も含め全呼び出し元が「前回の named プリセットが
/// 残留する」不具合の恩恵を受ける（`Runtime::apply_with_view_transition`
/// 側で個別に `remove_attribute` していた旧実装は本関数への一元化に伴い
/// 削除。属性の設定/除去は `document.startViewTransition()` 呼び出し
/// **前**に同期的に完了させる必要がある。UA が old/new スナップショット
/// 間の疑似要素へ CSS を適用する際、`documentElement` の現在の属性値を
/// 参照するため、遷移開始前に確定していなければプリセット CSS が
/// 一致しない）。`ViewTransitionPreset` は `view-transition-preset`
/// feature の有無に関わらず常時コンパイルされる型のため（`wiring` サブ
/// モジュールのみが feature ゲート対象）、本関数もその feature に依存
/// しない。
#[cfg(target_arch = "wasm32")]
pub(crate) fn with_view_transition<F>(
    document: &Document,
    preset: Option<crate::view_transition_preset::ViewTransitionPreset>,
    apply: F,
) where
    F: FnOnce() + 'static,
{
    match preset {
        // `set_attribute` は fw gate `url_validation_check`（U1）が
        // 同一ファイル内の URL 検証ガード 4 種の co-location を要求する
        // DOM 属性 sink のため、直接呼ばずガード co-located 済みの
        // `view_transition_preset::wiring::set_preset_attr` へ委譲する
        // （`remove_attribute` は U1 の sink needle 対象外のためガード不要）。
        Some(preset) => crate::view_transition_preset::wiring::set_preset_attr(document, preset),
        None => {
            if let Some(el) = document.document_element() {
                let _ =
                    el.remove_attribute(crate::view_transition_preset::VIEW_TRANSITION_PRESET_ATTR);
            }
        }
    }
    if !is_supported(document) {
        // 非対応ブラウザ: 同期フォールバック。
        apply();
        return;
    }
    let doc_vt = document.clone().unchecked_into::<DocumentViewTransitions>();

    let slot = std::rc::Rc::new(std::cell::RefCell::new(Some(apply)));
    let update_slot = slot.clone();
    let update = Closure::once_into_js(move || {
        if let Some(apply) = update_slot.borrow_mut().take() {
            apply();
        }
    });
    if let Err(err) = doc_vt.start_view_transition(&update) {
        // 呼び出し自体が throw し、update コールバックが未実行のまま
        // 終わった場合の同期フォールバック（警告ログのみ、内部状態は
        // 含めない不変条件 6）。
        web_sys::console::warn_1(
            &"fandhe-frontend-wasm-full: document.startViewTransition threw, view transition skipped"
                .into(),
        );
        let _ = err;
        if let Some(apply) = slot.borrow_mut().take() {
            apply();
        }
    }
}
