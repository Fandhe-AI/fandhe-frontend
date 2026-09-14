//! scroll ドライバの配線層（イシュー #2521、親 `#2499`）。
//!
//! # 責務境界
//!
//! DOM 計測（`getBoundingClientRect`/`innerHeight`）・進捗計算
//! （`compute_progress`）・rAF ループ本体のロジックは
//! `fandhe-frontend-animation`（[`fandhe_frontend_animation::scroll_driver`]）
//! の責務であり、本モジュールは以下のみを担う（3 層構成、`docs/design/
//! motion-reference-adoption-policy.md` §6）:
//!
//! 1. `[data-fandhe-scroll-progress]` 要素の収集（opt-in マーカー）
//! 2. 機能検出結果（[`fandhe_frontend_animation::scroll_driver::Env`]）に
//!    応じた分岐（ネイティブ CSS 委譲時は何もしない／reduced-motion 時は
//!    静止値を 1 回書き込むのみ／それ以外は `scroll`/`resize` リスナー +
//!    rAF ループを起動する）
//! 3. `document`/`window` へのイベントリスナー登録
//!
//! # `data-*` 属性の命名
//!
//! `data-fandhe-*` 内部プレフィックス（`gesture.rs`/`in_view.rs` と同型）。
//! opt-in マーカーの値は空文字列のみで、DOM から取得した動的文字列を
//! セレクタ・プロパティ名へ混ぜない（REQ-1・security.md A03）。
//!
//! # Runtime への統合
//!
//! `crate::lib::Runtime::mount`/`Runtime::hydrate` の双方が
//! `Self::wire_gesture` の直後で `Self::wire_scroll_driver` を呼ぶ（feature
//! `scroll-driver`、既定 on）。`dispatch` チャネルを持たない属性専用配線の
//! ため（`Self::wire_sidebar`/`Self::wire_in_view` と同型）、
//! `Component`/`binding_table`/`keyed_list_cache` は必要としない。
//!
//! # 既知の制約
//!
//! 動的に追加される要素（keyed list の Insert 等）への追随
//! （`MutationObserver` によるライブトラッキング）は `in_view.rs` 同様、
//! 本 issue のスコープ外の既知の制約である。

// wasm32 以外（native/SSR）では `mod wiring` 自体がコンパイルされないため
// （下記 `#[cfg(target_arch = "wasm32")]`）、この re-export も同じ cfg で
// ガードする（未使用 import の警告を避ける）。
#[cfg(target_arch = "wasm32")]
use fandhe_frontend_animation::scroll_driver::Env;
use fandhe_frontend_animation::scroll_driver::ProgressRange;

/// opt-in（著者が SSR 出力に静的に付与）: scroll ドライバのフォールバック
/// 対象にする存在属性。
pub const SCROLL_PROGRESS_ATTR: &str = "data-fandhe-scroll-progress";
/// 候補走査セレクタ。
pub const SCROLL_PROGRESS_SELECTOR: &str = "[data-fandhe-scroll-progress]";

/// `data-fandhe-scroll-progress` 属性値を [`ProgressRange`] へ厳格一致で
/// 変換する（イシュー #2534）。
///
/// `""`（存在マーカーのみ・値なし）・`"entry"` は [`ProgressRange::Entry`]、
/// `"cover"` は [`ProgressRange::Cover`]、`"contain"` は
/// [`ProgressRange::Contain`] へ写像し、それ以外の未知値は
/// [`ProgressRange::Entry`] へ fail-closed に倒す（`in_view.rs::
/// in_view_once_from_attr` と同じ「厳格一致・未知値は安全側」方針）。
///
/// DOM から読んだ動的文字列はこの `match` の既知パターンとの比較にのみ
/// 使い、変換結果の `ProgressRange`（enum 値）だけを後段（セレクタ・
/// プロパティ名の組み立てを一切伴わない `compute_progress_for_range` の
/// 引数）へ渡す。動的文字列そのものをセレクタ・プロパティ名へ混ぜない
/// （REQ-1・security.md A03）。
#[must_use]
pub fn progress_range_from_attr(value: &str) -> ProgressRange {
    match value {
        "" | "entry" => ProgressRange::Entry,
        "cover" => ProgressRange::Cover,
        "contain" => ProgressRange::Contain,
        _ => ProgressRange::Entry,
    }
}

#[cfg(test)]
mod progress_range_from_attr_tests {
    use super::progress_range_from_attr;
    use fandhe_frontend_animation::scroll_driver::ProgressRange;

    #[test]
    fn empty_marker_value_is_entry() {
        assert_eq!(progress_range_from_attr(""), ProgressRange::Entry);
    }

    #[test]
    fn explicit_entry_is_entry() {
        assert_eq!(progress_range_from_attr("entry"), ProgressRange::Entry);
    }

    #[test]
    fn cover_maps_to_cover() {
        assert_eq!(progress_range_from_attr("cover"), ProgressRange::Cover);
    }

    #[test]
    fn contain_maps_to_contain() {
        assert_eq!(progress_range_from_attr("contain"), ProgressRange::Contain);
    }

    #[test]
    fn unknown_value_fails_closed_to_entry() {
        assert_eq!(
            progress_range_from_attr("cover; background: red"),
            ProgressRange::Entry
        );
        assert_eq!(progress_range_from_attr("COVER"), ProgressRange::Entry);
        assert_eq!(progress_range_from_attr("unknown"), ProgressRange::Entry);
    }
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{progress_range_from_attr, Env, SCROLL_PROGRESS_ATTR, SCROLL_PROGRESS_SELECTOR};
    use fandhe_frontend_animation::dom_target::DomTarget;
    use fandhe_frontend_animation::scroll_driver::{
        update_element_progress_for_range, ProgressRange, ScrollDriver, SCROLL_PROGRESS_PROPERTY,
    };
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, HtmlElement};

    /// `root` 配下の `[data-fandhe-scroll-progress]` 要素（複数可）を、
    /// 各要素の属性値から解決した [`ProgressRange`] とあわせて出現順に
    /// 集める。`query_selector_all` の失敗は空 `Vec` として扱う
    /// （fail-closed、panic しない。`in_view.rs::collect_in_view_candidates`
    /// と同型）。属性値 → `ProgressRange` の変換は
    /// [`progress_range_from_attr`]（厳格一致・未知値は `Entry` へ
    /// fail-closed）を経由する（イシュー #2534）。
    fn collect_scroll_progress_candidates(root: &Element) -> Vec<(HtmlElement, ProgressRange)> {
        let Ok(node_list) = root.query_selector_all(SCROLL_PROGRESS_SELECTOR) else {
            return Vec::new();
        };
        let len = node_list.length();
        let mut out = Vec::with_capacity(len as usize);
        for i in 0..len {
            if let Some(node) = node_list.get(i) {
                if let Ok(el) = node.dyn_into::<HtmlElement>() {
                    let attr_value = el.get_attribute(SCROLL_PROGRESS_ATTR).unwrap_or_default();
                    let range = progress_range_from_attr(&attr_value);
                    out.push((el, range));
                }
            }
        }
        out
    }

    /// 実ブラウザ検出（[`Env::detect`]）で候補要素へ配線する
    /// （[`crate::lib::Runtime::wire_scroll_driver`] から呼ばれる）。
    ///
    /// # Errors
    ///
    /// [`wire_scroll_driver_with_env`] の失敗を伝播する。
    pub fn wire_scroll_driver(root: &Element) -> Result<(), JsValue> {
        wire_scroll_driver_with_env(root, Env::detect())
    }

    /// `env` を明示注入できる配線本体（テスト・[`wire_scroll_driver`] の
    /// 双方から呼ばれる）。headless Chrome は `animation-timeline: view()`
    /// を既にネイティブ対応するため、フォールバック経路の検証にはこの
    /// 注入口が必須である。
    ///
    /// # Errors
    ///
    /// `query_selector_all`・イベントリスナー登録の失敗を伝播する
    /// （候補 0 件の場合は何もせず `Ok(())`）。
    pub fn wire_scroll_driver_with_env(root: &Element, env: Env) -> Result<(), JsValue> {
        let candidates = collect_scroll_progress_candidates(root);
        if candidates.is_empty() {
            return Ok(());
        }

        // ネイティブ CSS（`animation-timeline: view()`）に委譲できる場合は
        // 進捗 custom property を一切書かず終了する（JS フォールバックは
        // 不要、CSS 側が直接反映する）。
        if env.scroll_timeline_supported {
            return Ok(());
        }

        // `prefers-reduced-motion: reduce` は静止状態（進捗 1 = 完了扱い）
        // を 1 回だけ書き込み、リスナー登録は行わない（AC の reduced-motion
        // 個別規則。継続的なスクロール連動アニメーションを提供しない）。
        if env.reduced_motion {
            for (el, _range) in &candidates {
                let _ = el.style().set_property(SCROLL_PROGRESS_PROPERTY, "1");
            }
            return Ok(());
        }

        let window = web_sys::window().ok_or_else(|| JsValue::from_str("window is unavailable"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("document is unavailable"))?;

        let mut targets: Vec<(HtmlElement, DomTarget, ProgressRange)> = candidates
            .iter()
            .map(|(el, range)| {
                let target = DomTarget::custom_property(el.clone(), SCROLL_PROGRESS_PROPERTY);
                (el.clone(), target, *range)
            })
            .collect();

        let driver = ScrollDriver::start(move || {
            for (element, target, range) in &mut targets {
                let element: &Element = element.as_ref();
                let _ = update_element_progress_for_range(element, target, *range);
            }
        });

        // `ScrollDriver`（内部で保持する `AnimationLoop`/rAF `Closure` を
        // 含む）はマウント時 1 回・ページ寿命ぶん意図的にリークする
        // （A08 対策、`in_view.rs`/`gesture.rs` と同型の「マウント時 1 回・
        // 定数個リーク」契約）。
        let driver = Box::leak(Box::new(driver));

        let scroll_closure = Closure::<dyn FnMut(Event)>::new({
            let driver = &*driver;
            move |_event: Event| {
                driver.mark_dirty();
            }
        });
        // `scroll` はバブルしないため capture フェーズで `document` へ
        // 委譲登録する（`message_scroller.rs:1126` と同じ理由づけ）。
        // `document` を起点にすることでページ全体スクロール・ネストした
        // スクロールコンテナの双方を 1 リスナーで捕捉できる。
        document.add_event_listener_with_callback_and_bool(
            "scroll",
            scroll_closure.as_ref().unchecked_ref(),
            true,
        )?;
        scroll_closure.forget();

        let resize_closure = Closure::<dyn FnMut(Event)>::new({
            let driver = &*driver;
            move |_event: Event| {
                driver.mark_dirty();
            }
        });
        window
            .add_event_listener_with_callback("resize", resize_closure.as_ref().unchecked_ref())?;
        resize_closure.forget();

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{wire_scroll_driver, wire_scroll_driver_with_env};
