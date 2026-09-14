//! ビューポート進入検出（in-view、`IntersectionObserver` 配線、イシュー
//! #2396、親 #2394）。
//!
//! `docs/design/motion-reference-adoption-policy.md` §4 が inView を
//! 「群 B: 小さな DOM 配線」に分類するとおり、[`fandhe_frontend_animation`]
//! の `Driver`/`Target` 連携は不要で、`IntersectionObserver` の購読と
//! `data-*` 属性の付け外しのみで完結する（本クレート単独の依存追加なし。
//! `fandhe-frontend-animation` への optional 依存は本モジュールでは
//! 有効化しない）。
//!
//! アプリ側マークアップが `data-in-view`（値なし）を付けた任意の要素が
//! 監視対象になる opt-in マーカーであり、同じ属性が交差状態の書き戻し先
//! （存在マーカー、`hidden`/`data-positioned` と同じ慣習）を兼ねる。
//! `data-in-view-once="true"` を併記すると初回進入後に監視解除する
//! （[`in_view_once_from_attr`]）。`fandhe-frontend-headless-ui` には
//! 依存しない汎用配線であり、任意のアプリマークアップが対象となる。
//!
//! `events.rs`/`keynav.rs`/`headless_avatar.rs` と同じ 2 層構成
//! （DOM 非依存の純粋ロジック層 + `#[cfg(target_arch = "wasm32")]` 配線層）
//! を踏襲する。
//!
//! # Runtime への統合
//!
//! `crate::lib::Runtime::mount`/`Runtime::hydrate` の双方が
//! `Self::wire_data_table` の直後で `Self::wire_in_view` を呼ぶ（feature
//! `in-view`、既定 on）。`dispatch` チャネルを持たない属性専用配線のため
//! （`Self::wire_sidebar`/`Self::wire_chart` と同型）、`Component`/
//! `binding_table`/`keyed_list_cache` は必要としない。
//!
//! # セキュリティ不変条件（REQ-1）
//!
//! `data-in-view` へ書き込む値は空文字列 `""` の `&'static str` リテラルの
//! みであり、DOM から取得した動的文字列（`data-in-view-once` の値等）を
//! 属性値・セレクタへ混ぜない。属性書き込みは
//! `crate::dom::set_dom_attribute_result` を経由する（`headless_avatar.rs`
//! と同じ方針）。

/// `data-in-view` 属性名（opt-in マーカー兼、交差状態の書き戻し先）。
pub const IN_VIEW_ATTR: &str = "data-in-view";
/// `data-in-view-once` 属性名（真偽値属性、`"true"` のときのみ有効）。
pub const IN_VIEW_ONCE_ATTR: &str = "data-in-view-once";
/// 監視対象候補の走査セレクタ。
pub const IN_VIEW_SELECTOR: &str = "[data-in-view]";

/// `data-in-view-once` 属性値から once 判定を行う（DOM 非依存の純粋関数、
/// native `cargo test` で検証可能）。
///
/// `"true"` の厳格一致のみ `true`。未設定・空文字列・大文字小文字違い・
/// 任意文字列はすべて `false` へ fail-closed に倒す
/// （`keynav::menu_loop_focus_from_attr` と同じ流儀）。
#[must_use]
pub fn in_view_once_from_attr(value: Option<&str>) -> bool {
    value == Some("true")
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{in_view_once_from_attr, IN_VIEW_ATTR, IN_VIEW_ONCE_ATTR, IN_VIEW_SELECTOR};
    use crate::dom::set_dom_attribute_result as set_dom_attribute;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{
        Element, IntersectionObserver, IntersectionObserverEntry, MutationObserver,
        MutationObserverInit, MutationRecord, Node, NodeList,
    };

    /// `root` 配下の `[data-in-view]` 要素（複数可）を出現順に集める。
    /// `query_selector_all` の失敗は空 `Vec` として扱う（fail-closed、
    /// panic しない。`headless_avatar.rs::collect_avatar_images` と同じ方針）。
    fn collect_in_view_candidates(root: &Element) -> Vec<Element> {
        let Ok(node_list) = root.query_selector_all(IN_VIEW_SELECTOR) else {
            return Vec::new();
        };
        let len = node_list.length();
        let mut out = Vec::with_capacity(len as usize);
        for i in 0..len {
            if let Some(node) = node_list.get(i) {
                if let Ok(el) = node.dyn_into::<Element>() {
                    out.push(el);
                }
            }
        }
        out
    }

    /// `window.IntersectionObserver` の機能検出（Duck-typing ではなく
    /// `Reflect::get` + `.is_function()`、`web-sys` が正式サポートする API
    /// のため）。
    fn supports_intersection_observer() -> bool {
        let Some(window) = web_sys::window() else {
            return false;
        };
        js_sys::Reflect::get(&window, &JsValue::from_str("IntersectionObserver"))
            .map(|value| value.is_function())
            .unwrap_or(false)
    }

    /// `root` 配下の `[data-in-view]` 要素へ `IntersectionObserver` 配線を
    /// マウント時に 1 回だけ登録する（[`crate::lib::Runtime::wire_in_view`]
    /// から呼ばれる）。
    ///
    /// 非対応ブラウザ（`supports_intersection_observer` が `false`）では
    /// プログレッシブエンハンスメントとして現時点の候補全件へ即座に
    /// `data-in-view` を付与して終了する（JS 非対応でも常に可視という
    /// 安全側デフォルト、`nav.rs::start_view_transition_prop` の同期フォール
    /// バックと同じ方針。動的追加要素の追随は行わない、既知の制約）。
    ///
    /// 対応ブラウザでは単一 `IntersectionObserver` インスタンスで初期候補を
    /// `observe()` したうえで、`root` を対象に `childList: true` +
    /// `subtree: true` の `MutationObserver`（[`wire_in_view_mutation_observer`]）
    /// を追加登録する。`Runtime::rerender_subtree` の DOM 一括差し替え・
    /// keyed list の Insert はいずれも `root` 配下のノード追加・削除として
    /// 現れるため、`IntersectionObserver` 自体を毎回作り直さず「`root` を
    /// 監視し続ける 1 個の `MutationObserver` が新規要素の `observe()`・
    /// 消失要素の `unobserve()` を追随する」設計で対応する
    /// （`headless_avatar.rs::wire_avatar_src_observer` が `root` に
    /// `subtree: true` で 1 個の `MutationObserver` を張り続けることで
    /// DOM 差し替えに追随する方針と同型。個々の要素ではなく `root` を
    /// 監視対象にすることで、`Runtime::wire_in_view` を再呼び出しする
    /// 契約変更なしに追随できる）。両 observer とも「マウント時 1 回・
    /// 定数個リーク」契約（`Closure::forget()` は各 1 回のみ）を維持する。
    ///
    /// # Errors
    ///
    /// `IntersectionObserver::new`・`MutationObserver::new`・`observe`・
    /// 属性書き込みの失敗を伝播する。
    pub fn wire_in_view(root: &Element) -> Result<(), JsValue> {
        let candidates = collect_in_view_candidates(root);

        if !supports_intersection_observer() {
            for el in &candidates {
                set_dom_attribute(el, IN_VIEW_ATTR, "")?;
            }
            return Ok(());
        }

        let callback = Closure::<dyn FnMut(js_sys::Array, IntersectionObserver)>::new(
            move |entries: js_sys::Array, observer: IntersectionObserver| {
                handle_intersections(&entries, &observer);
            },
        );
        let observer = IntersectionObserver::new(callback.as_ref().unchecked_ref())?;
        callback.forget();

        for el in &candidates {
            observer.observe(el);
        }

        wire_in_view_mutation_observer(root, observer)?;

        Ok(())
    }

    /// `root` 配下の追加・削除ノードを追跡し、`observer`（[`wire_in_view`]
    /// が生成した唯一の `IntersectionObserver`）への `observe()`/`unobserve()`
    /// を追随させる（イシュー #2396 codex-review P1 是正）。
    ///
    /// 追加ノード側は自身と子孫のうち `[data-in-view]` に一致する要素を
    /// すべて `observe()` する（`observe()` は同一 target への再呼び出しが
    /// 安全な冪等操作であり、既存監視対象への重複呼び出しでも二重発火は
    /// 起きない）。削除ノード側は同様に `unobserve()` する（未監視 target
    /// への `unobserve()` も安全な no-op）。
    fn wire_in_view_mutation_observer(
        root: &Element,
        observer: IntersectionObserver,
    ) -> Result<(), JsValue> {
        let callback = Closure::<dyn FnMut(js_sys::Array, MutationObserver)>::new(
            move |records: js_sys::Array, _observer: MutationObserver| {
                for record in records.iter() {
                    let Ok(record) = record.dyn_into::<MutationRecord>() else {
                        continue;
                    };
                    for node in node_list_items(&record.added_nodes()) {
                        for el in in_view_elements_of(&node) {
                            observer.observe(&el);
                        }
                    }
                    for node in node_list_items(&record.removed_nodes()) {
                        for el in in_view_elements_of(&node) {
                            observer.unobserve(&el);
                        }
                    }
                }
            },
        );
        let mutation_observer = MutationObserver::new(callback.as_ref().unchecked_ref())?;
        let init = MutationObserverInit::new();
        init.set_child_list(true);
        init.set_subtree(true);
        mutation_observer.observe_with_options(root, &init)?;
        callback.forget();
        Ok(())
    }

    /// `NodeList`（`MutationRecord::added_nodes`/`removed_nodes` が返す型）
    /// を `Vec<Node>` へ集める（`NodeList` は `Iterator` を実装しないため、
    /// `collect_in_view_candidates` と同じ `length`/`get` 走査で代替する）。
    fn node_list_items(list: &NodeList) -> Vec<Node> {
        let len = list.length();
        let mut out = Vec::with_capacity(len as usize);
        for i in 0..len {
            if let Some(node) = list.get(i) {
                out.push(node);
            }
        }
        out
    }

    /// `node` 自身が `[data-in-view]` に一致すればそれを、加えて子孫の
    /// 一致要素をすべて集める（`MutationRecord::added_nodes`/`removed_nodes`
    /// の 1 ノードが `[data-in-view]` 要素を内包するサブツリーである場合に
    /// 対応する。`Element` でないノード（テキストノード等）・`matches`/
    /// `query_selector_all` の失敗は無視する fail-closed 処理）。
    fn in_view_elements_of(node: &Node) -> Vec<Element> {
        let Ok(el) = node.clone().dyn_into::<Element>() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        if matches!(el.matches(IN_VIEW_SELECTOR), Ok(true)) {
            out.push(el.clone());
        }
        out.extend(collect_in_view_candidates(&el));
        out
    }

    /// `IntersectionObserver` コールバック本体。各 `entry` の交差状態に
    /// 応じて `data-in-view` を付け外しし、once 指定かつ進入済みの要素は
    /// `unobserve` する。
    ///
    /// `dyn_into` 失敗（想定外のノード型）はスキップする fail-closed 処理
    /// （`headless_avatar.rs::wire_avatar_src_observer` と同型）。
    fn handle_intersections(entries: &js_sys::Array, observer: &IntersectionObserver) {
        for entry in entries.iter() {
            let Ok(entry) = entry.dyn_into::<IntersectionObserverEntry>() else {
                continue;
            };
            let target = entry.target();
            if entry.is_intersecting() {
                if set_dom_attribute(&target, IN_VIEW_ATTR, "").is_err() {
                    continue;
                }
                let once =
                    in_view_once_from_attr(target.get_attribute(IN_VIEW_ONCE_ATTR).as_deref());
                if once {
                    observer.unobserve(&target);
                }
            } else {
                let _ = target.remove_attribute(IN_VIEW_ATTR);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_in_view;
