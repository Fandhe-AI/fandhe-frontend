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

        let opted_in = js_sys::WeakSet::new();
        let observed = js_sys::WeakSet::new();
        let once_done = js_sys::WeakSet::new();

        let callback_observed = observed.clone();
        let callback_once_done = once_done.clone();
        let callback = Closure::<dyn FnMut(js_sys::Array, IntersectionObserver)>::new(
            move |entries: js_sys::Array, observer: IntersectionObserver| {
                handle_intersections(&entries, &observer, &callback_observed, &callback_once_done);
            },
        );
        let observer = IntersectionObserver::new(callback.as_ref().unchecked_ref())?;
        callback.forget();

        for el in &candidates {
            observer.observe(el);
            opted_in.add(el);
            observed.add(el);
        }

        wire_in_view_mutation_observer(root, observer, opted_in, observed, once_done)?;

        Ok(())
    }

    /// `root` 配下の追加・削除ノードを追跡し、`observer`（[`wire_in_view`]
    /// が生成した唯一の `IntersectionObserver`）への `observe()`/`unobserve()`
    /// を追随させる（イシュー #2396 codex-review P1 是正、および同 P1/
    /// Bugbot High/Medium 4 件の再指摘を受けた是正）。
    ///
    /// # 監視状態を `data-in-view` 属性の有無から独立させる不変条件
    ///
    /// [`handle_intersections`] は交差状態に応じて `data-in-view` を
    /// 付け外しするため、この属性は「監視中かどうか」の判定に**使えない**
    /// （非交差時に属性が消え、それを「監視対象外になった」と誤読すると、
    /// 削除ノード側の `unobserve()` が対象を見失いリークする。逆に once
    /// 進入済み要素は属性が付いたまま残るため、それを「新規候補」と
    /// 誤読すると once 契約が破れる）。このため監視状態は 3 個の
    /// `js_sys::WeakSet`（要素識別ベース、`data-in-view` の値に一切依存
    /// しない）で独立に追跡する: `opted_in`（一度でも opt-in マーカー
    /// 一致で候補になった要素、永続）・`observed`（現在
    /// `observer.observe()` 登録中の要素）・`once_done`（once 進入済みで
    /// 今後一切再 observe しない要素、永続）。
    ///
    /// 追加ノード側は自身と子孫の**全要素**（属性の有無を問わない、
    /// [`all_elements_of`]）を対象に、`once_done` 未登録かつ
    /// （現在 `[data-in-view]` に一致する、または過去に `opted_in` 済み）
    /// の要素だけを `observed` へ追加のうえ `observe()` する（`opted_in`
    /// 済み要素の再取り込みは keyed-list の Move — 同一ノードの
    /// remove→add — で非交差中〔属性なし〕の要素が再挿入される場合に
    /// 対応する）。削除ノード側も同様に全要素を対象に `observed` 登録
    /// 済みの要素だけを `unobserve()` して `observed` から外す。
    fn wire_in_view_mutation_observer(
        root: &Element,
        observer: IntersectionObserver,
        opted_in: js_sys::WeakSet,
        observed: js_sys::WeakSet,
        once_done: js_sys::WeakSet,
    ) -> Result<(), JsValue> {
        let callback = Closure::<dyn FnMut(js_sys::Array, MutationObserver)>::new(
            move |records: js_sys::Array, _observer: MutationObserver| {
                for record in records.iter() {
                    let Ok(record) = record.dyn_into::<MutationRecord>() else {
                        continue;
                    };
                    for node in node_list_items(&record.added_nodes()) {
                        for el in all_elements_of(&node) {
                            if once_done.has(&el) {
                                continue;
                            }
                            let opts_in_now = matches!(el.matches(IN_VIEW_SELECTOR), Ok(true));
                            if opts_in_now {
                                opted_in.add(&el);
                            }
                            if (opts_in_now || opted_in.has(&el)) && !observed.has(&el) {
                                observer.observe(&el);
                                observed.add(&el);
                            }
                        }
                    }
                    for node in node_list_items(&record.removed_nodes()) {
                        for el in all_elements_of(&node) {
                            if observed.has(&el) {
                                observer.unobserve(&el);
                                observed.delete(&el);
                            }
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

    /// `node` 自身と子孫の**全要素**を、`data-in-view` 属性の有無を問わず
    /// 集める（`wire_in_view_mutation_observer` の追加・削除いずれの側も
    /// 監視状態は属性ではなく `WeakSet` で判定するため、走査自体は選択的
    /// セレクタに絞らない。`Element` でないノード（テキストノード等）・
    /// `query_selector_all` の失敗は無視する fail-closed 処理）。
    fn all_elements_of(node: &Node) -> Vec<Element> {
        let Ok(el) = node.clone().dyn_into::<Element>() else {
            return Vec::new();
        };
        let mut out = vec![el.clone()];
        if let Ok(node_list) = el.query_selector_all("*") {
            let len = node_list.length();
            for i in 0..len {
                if let Some(child) = node_list.get(i) {
                    if let Ok(child_el) = child.dyn_into::<Element>() {
                        out.push(child_el);
                    }
                }
            }
        }
        out
    }

    /// `IntersectionObserver` コールバック本体。各 `entry` の交差状態に
    /// 応じて `data-in-view` を付け外しし、once 指定かつ進入済みの要素は
    /// `unobserve` して `once_done`（[`wire_in_view_mutation_observer`] と
    /// 共有、要素識別ベースの永続集合）へ登録する。以後この要素は
    /// `data-in-view` が DOM 上に残っていても追加ノード処理の対象から
    /// 除外され、再挿入されても再 observe されない。
    ///
    /// `dyn_into` 失敗（想定外のノード型）はスキップする fail-closed 処理
    /// （`headless_avatar.rs::wire_avatar_src_observer` と同型）。
    fn handle_intersections(
        entries: &js_sys::Array,
        observer: &IntersectionObserver,
        observed: &js_sys::WeakSet,
        once_done: &js_sys::WeakSet,
    ) {
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
                    observed.delete(&target);
                    once_done.add(&target);
                }
            } else {
                let _ = target.remove_attribute(IN_VIEW_ATTR);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_in_view;
