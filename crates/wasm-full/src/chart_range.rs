//! charts の期間切替コントロール・凡例系列トグルを DOM 属性へ配線する
//! （イシュー #2134、親 #2132、祖父トラッキング #2001）。
//!
//! `crates/pre-styled-ui/src/charts/legend.rs`（イシュー #2133）は凡例
//! item を `<button data-scope="chart-legend" data-part="trigger"
//! data-series="<name>"|data-index="<n>" aria-pressed="true|false"
//! [aria-controls="<chart root id>"]>` として SSR 出力する。各チャート
//! root は opt-in `range` プロパティで `data-range="<不透明文字列>"` を
//! 出力し、系列/カテゴリの描画要素（`series-line`/`series-area`/
//! `point`/`value-label`/`bar`/`inside-label`/`segment`/radial の
//! `bar`・`label`・`track` 等、いずれも `data-series`/`data-index` を
//! 持つ）は `hidden_series`/`hidden_categories` に応じて値なし属性
//! `data-hidden` を持つ。各チャートの recipe は `[data-hidden] {
//! display: none }` を既に持つ（`crates/pre-styled-ui/src/charts/mod.rs`
//! モジュール doc「凡例トグルの SSR 構造」節が唯一のロケータ契約）。
//!
//! 本モジュールはこの静的マークアップに対し、(1) 凡例 trigger クリックで
//! `aria-pressed` を反転させ、(2) 期間切替コントロール（toggle-group/
//! select の item）クリックでチャート root の `data-range` を更新し、
//! (3) 上記 2 つと連動して描画要素・hit-area・tooltip-item の
//! `data-hidden` を DOM から導出して同期する（[`wiring::sync_chart`]）。
//!
//! # スケール再計算はスコープ外（本イシューでの判断）
//!
//! 親 #2132 は「スケール再計算を追従させるか固定にするかは実装時に
//! 決める」としていたが、本イシューでは REQ-11 の bundle size 制約
//! （実測: ベースライン 195,964/200,000 B、`fandhe-frontend-wasm-client`
//! 0.6.1 依存の headroom は約 4 KB。#2130 の tooltip 配線単体で約 4.8 KB
//! 消費した実績がある）と実装コストを踏まえ、**全チャート種別で
//! 「非表示のみ（hide-only）」に統一し、軸スケール・domain の再計算は
//! 行わない**と判断した（`docs/design/wasm-full-architecture.md` §27
//! 参照、後続 Issue の起票提案あり）。これにより:
//!
//! - `fandhe-frontend-pre-styled-ui` への新規依存を追加しない（`chart.rs`
//!   の [`crate::chart_range::wiring`] 実装方針を踏襲し、テストのみ
//!   `fandhe_frontend_core::el` で SSR 出力契約を手組みする。
//!   `chart_tooltip_browser.rs` と同型）。
//! - `crates/pre-styled-ui` 側の新規 `data-scale-*`/`data-values`/
//!   `data-value` 契約（当初計画 §3.5）は導入しない。
//! - 系列/カテゴリの非表示に伴う軸・domain・tick ラベルの見た目は
//!   非表示前のまま変化しない（bar の 0 基準線・line の折れ線ギャップも
//!   含め、視覚的な「詰め直し」は行わない）。
//!
//! # 2 層構成（`chart.rs`/`sidebar.rs` と同型）
//!
//! - 純粋ロジック層（[`parse_range_bound`]/[`resolve_range`]/
//!   [`category_hidden_by_range`]/[`is_indexed_element_hidden`]）は
//!   web-sys に依存せず、native の `cargo test` で検証できる。
//! - 配線層（[`wiring`]）のみ `#[cfg(target_arch = "wasm32")]` で
//!   ゲートする。
//!
//! # Runtime への統合
//!
//! [`wiring::wire_chart_range_events`] は `crate::Runtime::mount`/
//! `Runtime::hydrate` の双方から `Self::wire_chart` の直後に組み込まれる
//! （`crate::Runtime::wire_chart_range` 参照）。`dispatch` チャネルを
//! 持たない属性専用配線であり、`crate::headless::MAPPING_TABLE` に
//! `chart-legend` scope の行は無いため凡例クリックは dispatch へ流れず、
//! 本モジュールのリスナーとのみ干渉する（`preventDefault`/
//! `stopPropagation` を一切呼ばない設計により、toggle-group/select
//! item クリックの dispatch 配線とも並走できる）。
//!
//! # ロケータ契約（fail-closed、`security.md` A03）
//!
//! - 凡例 trigger → チャート root: `aria-controls="<id>"` を
//!   `document.get_element_by_id` に渡して解決する唯一のリンク
//!   （`crates/pre-styled-ui/src/charts/legend.rs` モジュール doc
//!   「`trigger` slot の語彙」節）。`aria-controls` 無し・未解決 id は
//!   `aria-pressed` の反転のみ行いチャート側は不変（no-op）。
//! - 期間切替コントロール item → チャート root: クリック要素から
//!   `closest('[data-scope="toggle-group"][data-part="item"],
//!   [data-scope="select"][data-part="item"]')`（静的セレクタ）で item を
//!   特定し、その item 自身または祖先の `closest("[aria-controls]")` で
//!   `aria-controls` を得てチャート root を解決する。範囲→カテゴリ index
//!   の写像は item の `data-range-from`/`data-range-to`（10 進の非負
//!   整数、既定 `from=0`/`to=<チャート内カテゴリ数>`）で**アプリが宣言
//!   する**（予約キーではない、`toggle_group::item`/`select::item` の
//!   `attrs` 引数で渡す）。パース失敗・`to <= from` は fail-closed で
//!   範囲による非表示を一切行わない。
//! - 対象要素の列挙は常に静的セレクタ（`[data-index]`）で行い、値比較は
//!   Rust 側で行う（`chart.rs::matches_key` と同方針）。値の書き込みは
//!   `set_attribute`/`remove_attribute` のみ、`query_selector` へ
//!   利用者由来文字列を補間しない。
//!
//! # 同期（[`wiring::sync_chart`]）
//!
//! 状態は DOM から導出して冪等に同期する。判定源は凡例 trigger の
//! `aria-pressed`・チャート root の `data-range`（+ 対応する range
//! item の `data-range-from`/`data-range-to`）のみであり、内部に
//! ミュータブルな状態を持たない。これにより:
//!
//! - `Runtime::rerender_subtree` によるアプリ側の再描画（アプリが
//!   `range`/`hidden_series` を新しい props へ反映）と共存する
//!   （`wiring::wire_rerender_observer` が `MutationObserver`
//!   （`childList`/`subtree` のみ、`attributes` は監視しない＝自己発火
//!   ループを構造的に回避、`chart.rs::wiring::wire_rerender_observer`
//!   と同型）で再描画後に [`wiring::sync_all`] を再適用する）。
//! - 同じ状態への複数回の同期は結果が変化しない（冪等）。
//!
//! # スコープ外（イシュー #2134 §8）
//!
//! - スケール再計算（上記「スケール再計算はスコープ外」節）。
//! - 期間切替に伴うデータ再取得・カテゴリ集合の再構成（アプリ責務、
//!   親 #2132 設計方針 1）。
//! - `chart.rs::handle_keydown` の矢印移動が `display="none"` の範囲外
//!   hit-area へ到達し得る点（`focus()` が失敗して止まるのみで panic
//!   しない。可視 hit-area だけを巡回する改善は後続）。
//! - `examples/headless-pre-styled-ui` への追随（crates.io 公開後の
//!   既存運用方針）。

/// `data-range-from`/`data-range-to` の値をパースする（10 進の非負整数の
/// み許容。負数・非数値・空文字列は `None`）。`str::parse::<usize>` は
/// 先頭 `-` を含む文字列を構造的に拒否するため、追加の符号チェックを
/// 要しない。
#[must_use]
pub fn parse_range_bound(value: &str) -> Option<usize> {
    value.parse::<usize>().ok()
}

/// `from`/`to`（欠落時は `from=0`/`to=total`）から有効な半開区間
/// `[from, to)` を解決する。`to <= from`（`to` 欠落かつ `total == 0` を
/// 含む）は無効な宣言として `None`（fail-closed、範囲による非表示を
/// 行わない）。
#[must_use]
pub fn resolve_range(
    from: Option<usize>,
    to: Option<usize>,
    total: usize,
) -> Option<(usize, usize)> {
    let from = from.unwrap_or(0);
    let to = to.unwrap_or(total);
    if to > from {
        Some((from, to))
    } else {
        None
    }
}

/// `index` が `range`（`[from, to)`、[`resolve_range`] の戻り値）の外に
/// あるかどうか。`range` が `None`（範囲宣言なし・パース失敗・不正な
/// 区間）のときは常に `false`（範囲による非表示なし）。
#[must_use]
pub fn category_hidden_by_range(index: usize, range: Option<(usize, usize)>) -> bool {
    match range {
        Some((from, to)) => index < from || index >= to,
        None => false,
    }
}

/// `data-index`/`data-series` を持つ描画要素・hit-area 1 件が、凡例
/// トグル（`hidden_categories`/`hidden_series`、`aria-pressed="false"`
/// の trigger から合成した集合）または期間範囲（`range`）のいずれかに
/// より非表示になるべきかを判定する純粋関数（web-sys 非依存）。
#[must_use]
pub fn is_indexed_element_hidden(
    index: usize,
    series: Option<&str>,
    hidden_categories: &[usize],
    hidden_series: &[String],
    range: Option<(usize, usize)>,
) -> bool {
    if category_hidden_by_range(index, range) {
        return true;
    }
    if hidden_categories.contains(&index) {
        return true;
    }
    if let Some(series) = series {
        if hidden_series.iter().any(|hidden| hidden == series) {
            return true;
        }
    }
    false
}

#[cfg(target_arch = "wasm32")]
pub(crate) mod wiring {
    use crate::chart::wiring::{event_target_element, query_all, set_dom_attribute};
    use std::collections::BTreeSet;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, MutationObserver, MutationObserverInit};

    use super::{
        category_hidden_by_range, is_indexed_element_hidden, parse_range_bound, resolve_range,
    };

    /// 凡例 trigger を列挙する静的セレクタ（`legend`/`category_legend`
    /// 共通、`crates/pre-styled-ui/src/charts/legend.rs` 参照）。
    const LEGEND_TRIGGER_SELECTOR: &str = "[data-scope=\"chart-legend\"][data-part=\"trigger\"]";
    /// 期間切替コントロールの item を列挙する静的セレクタ（toggle-group/
    /// select の双方に対応、モジュール doc「ロケータ契約」節）。
    const RANGE_ITEM_SELECTOR: &str =
        "[data-scope=\"toggle-group\"][data-part=\"item\"], [data-scope=\"select\"][data-part=\"item\"]";
    /// `data-index` を持つ描画要素・hit-area を列挙する静的セレクタ
    /// （`chart.rs::INDEXED_SELECTOR` と同一語彙）。
    const INDEXED_SELECTOR: &str = "[data-index]";
    /// hit-area を列挙する静的セレクタ（`chart.rs::HIT_AREA_SELECTOR` と
    /// 同一語彙）。
    const HIT_AREA_SELECTOR: &str = "[data-scope=\"chart\"][data-part=\"hit-area\"]";
    /// tooltip 本体（カテゴリ単位、`data-index` を持つ）を列挙する静的
    /// セレクタ。
    const TOOLTIP_SELECTOR: &str = "[data-scope=\"chart\"][data-part=\"tooltip\"]";
    /// ツールチップ内の系列単位行を列挙する静的セレクタ。
    const TOOLTIP_ITEM_SELECTOR: &str = "[data-scope=\"chart\"][data-part=\"tooltip-item\"]";
    /// 非表示の値なし属性（`crates/pre-styled-ui` 側の recipe が
    /// `[data-hidden] { display: none }` を持つ、モジュール doc参照）。
    const HIDDEN_ATTR: &str = "data-hidden";

    /// `element` の `data-index` を `usize` として読む（無い・パース
    /// 不能な要素は `None`、fail-closed で対象から除外する）。
    fn indexed(element: &Element) -> Option<usize> {
        element
            .get_attribute("data-index")
            .and_then(|value| value.parse::<usize>().ok())
    }

    /// `root` 配下で `chart_root` を `aria-controls` の解決先とする凡例
    /// trigger を集め、`(hidden_series, hidden_categories)` を合成する。
    fn legend_hidden_sets(root: &Element, chart_root: &Element) -> (Vec<String>, Vec<usize>) {
        let chart_id = chart_root.id();
        let mut hidden_series = Vec::new();
        let mut hidden_categories = Vec::new();
        if chart_id.is_empty() {
            return (hidden_series, hidden_categories);
        }
        for trigger in query_all(root, LEGEND_TRIGGER_SELECTOR) {
            if trigger.get_attribute("aria-controls").as_deref() != Some(chart_id.as_str()) {
                continue;
            }
            let pressed = trigger.get_attribute("aria-pressed").as_deref() == Some("true");
            if pressed {
                continue;
            }
            if let Some(series) = trigger.get_attribute("data-series") {
                hidden_series.push(series);
            } else if let Some(index) = trigger
                .get_attribute("data-index")
                .and_then(|value| value.parse::<usize>().ok())
            {
                hidden_categories.push(index);
            }
        }
        (hidden_series, hidden_categories)
    }

    /// `root` 配下で `chart_root` を対象とする期間切替コントロール item の
    /// うち、`chart_root` の現在の `data-range` 値と `data-value` が一致
    /// する item を探し、`(from, to)` を解決する（モジュール doc
    /// 「ロケータ契約」節）。`chart_root` が `data-range` を持たない、
    /// 一致する item が無い、パース失敗のときは `None`。
    fn resolve_chart_range(
        root: &Element,
        chart_root: &Element,
        total: usize,
    ) -> Option<(usize, usize)> {
        let range_value = chart_root.get_attribute("data-range")?;
        for item in query_all(root, RANGE_ITEM_SELECTOR) {
            let Ok(Some(anchor)) = item.closest("[aria-controls]") else {
                continue;
            };
            if anchor.get_attribute("aria-controls").as_deref() != Some(chart_root.id().as_str()) {
                continue;
            }
            if item.get_attribute("data-value").as_deref() != Some(range_value.as_str()) {
                continue;
            }
            let from = item
                .get_attribute("data-range-from")
                .and_then(|value| parse_range_bound(&value));
            let to = item
                .get_attribute("data-range-to")
                .and_then(|value| parse_range_bound(&value));
            if item.has_attribute("data-range-from") || item.has_attribute("data-range-to") {
                return resolve_range(from, to, total);
            }
            return None;
        }
        None
    }

    /// `chart_root` 配下の hit-area から総カテゴリ数（`data-index` の
    /// 最大値 + 1）を推定する。hit-area が無ければ `0`（範囲による非表示
    /// を行わない、[`resolve_range`] が `to <= from` で無効化する）。
    fn total_categories(chart_root: &Element) -> usize {
        query_all(chart_root, HIT_AREA_SELECTOR)
            .into_iter()
            .filter_map(|element| indexed(&element))
            .max()
            .map_or(0, |max| max + 1)
    }

    /// hit-area の可視/非可視に応じて roving tabindex（`chart.rs::enhance`
    /// と同じ「可視集合の先頭が `0`、他は `-1`」規則）を再適用する。
    /// 既に可視集合内に `tabindex="0"` を持つ要素があれば触らない
    /// （キーボード操作で移動済みの状態を巻き戻さないため、
    /// `chart.rs::enhance` と同じ配慮）。
    fn reapply_hit_area_tabindex(hit_areas: &[Element]) {
        let visible: Vec<&Element> = hit_areas
            .iter()
            .filter(|element| !element.has_attribute(HIDDEN_ATTR))
            .collect();
        let has_visible_roving = visible
            .iter()
            .any(|element| element.get_attribute("tabindex").as_deref() == Some("0"));
        if let Some(first) = visible.first() {
            if !has_visible_roving {
                set_dom_attribute(first, "tabindex", "0");
            }
        }
        for element in hit_areas {
            if element.has_attribute(HIDDEN_ATTR) {
                set_dom_attribute(element, "tabindex", "-1");
            }
        }
    }

    /// `chart_root` 1 件の表示状態を DOM から導出して同期する
    /// （モジュール doc「同期」節）。`root` は凡例 trigger・期間切替
    /// item を探す走査範囲（`wire_chart_range_events` に渡された
    /// マウント root）。
    pub(crate) fn sync_chart(root: &Element, chart_root: &Element) {
        let (hidden_series, hidden_categories) = legend_hidden_sets(root, chart_root);
        let total = total_categories(chart_root);
        let range = resolve_chart_range(root, chart_root, total);

        for element in query_all(chart_root, INDEXED_SELECTOR) {
            let Some(index) = indexed(&element) else {
                continue;
            };
            let series = element.get_attribute("data-series");
            let hidden = is_indexed_element_hidden(
                index,
                series.as_deref(),
                &hidden_categories,
                &hidden_series,
                range,
            );
            if hidden {
                set_dom_attribute(&element, HIDDEN_ATTR, "");
            } else {
                let _ = element.remove_attribute(HIDDEN_ATTR);
            }
            let is_hit_area = element.get_attribute("data-scope").as_deref() == Some("chart")
                && element.get_attribute("data-part").as_deref() == Some("hit-area");
            if is_hit_area {
                if hidden {
                    set_dom_attribute(&element, "display", "none");
                } else {
                    let _ = element.remove_attribute("display");
                }
            }
        }
        reapply_hit_area_tabindex(&query_all(chart_root, HIT_AREA_SELECTOR));

        // ツールチップ本体（カテゴリ単位）は、既に非表示化した hit-area が
        // 二度と hover/focus セッションを開始しないため通常は自然と表示
        // されないが、範囲変更前に開いていたセッションを確実に閉じる
        // ための多層防御として `hidden` を強制する（`chart.rs::wiring` が
        // 使う `hidden` 属性と同じ語彙、native boolean attribute）。
        let hidden_category_set: BTreeSet<usize> = hidden_categories
            .iter()
            .copied()
            .chain((0..total).filter(|index| category_hidden_by_range(*index, range)))
            .collect();
        for tooltip in query_all(chart_root, TOOLTIP_SELECTOR) {
            if let Some(index) = indexed(&tooltip) {
                if hidden_category_set.contains(&index) {
                    set_dom_attribute(&tooltip, "hidden", "");
                }
            }
        }
        for item in query_all(chart_root, TOOLTIP_ITEM_SELECTOR) {
            let hidden = item
                .get_attribute("data-series")
                .is_some_and(|series| hidden_series.contains(&series));
            if hidden {
                set_dom_attribute(&item, HIDDEN_ATTR, "");
            } else {
                let _ = item.remove_attribute(HIDDEN_ATTR);
            }
        }
    }

    /// `id` を `document.get_element_by_id` で解決し、`root` 配下に
    /// 収まっている場合のみ返す（モジュール doc「ロケータ契約」節、
    /// `security.md` A03: `query_selector` へ利用者由来文字列を補間
    /// しないための `get_element_by_id` + Rust 側の包含確認）。
    fn resolve_chart_root(root: &Element, id: &str) -> Option<Element> {
        if id.is_empty() {
            return None;
        }
        let document = root.owner_document()?;
        let found = document.get_element_by_id(id)?;
        root.contains(Some(&found)).then_some(found)
    }

    /// `root` 配下の凡例 trigger・期間切替 item が参照するチャート root を
    /// すべて集めて [`sync_chart`] を適用する（初期同期・再描画後の
    /// 再同期の双方から呼ぶ）。
    pub(crate) fn sync_all(root: &Element) {
        let mut ids: Vec<String> = Vec::new();
        for trigger in query_all(root, LEGEND_TRIGGER_SELECTOR) {
            if let Some(id) = trigger.get_attribute("aria-controls") {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
        for item in query_all(root, RANGE_ITEM_SELECTOR) {
            let Ok(Some(anchor)) = item.closest("[aria-controls]") else {
                continue;
            };
            if let Some(id) = anchor.get_attribute("aria-controls") {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
        for id in ids {
            if let Some(chart_root) = resolve_chart_root(root, &id) {
                sync_chart(root, &chart_root);
            }
        }
    }

    /// 凡例 trigger クリック（`aria-pressed` の反転 + 同期）を処理する。
    /// 一致しなければ `false` を返す（`handle_range_item_click` との
    /// 排他判定用）。
    fn handle_legend_trigger_click(root: &Element, target: &Element) -> bool {
        let Ok(Some(trigger)) = target.closest(LEGEND_TRIGGER_SELECTOR) else {
            return false;
        };
        if !root.contains(Some(&trigger)) {
            return false;
        }
        let pressed = trigger.get_attribute("aria-pressed").as_deref() == Some("true");
        set_dom_attribute(
            &trigger,
            "aria-pressed",
            if pressed { "false" } else { "true" },
        );
        if let Some(id) = trigger.get_attribute("aria-controls") {
            if let Some(chart_root) = resolve_chart_root(root, &id) {
                sync_chart(root, &chart_root);
            }
        }
        true
    }

    /// 期間切替コントロール item クリック（`data-value` の
    /// チャート root `data-range` への転記 + 同期）を処理する。
    fn handle_range_item_click(root: &Element, target: &Element) {
        let Ok(Some(item)) = target.closest(RANGE_ITEM_SELECTOR) else {
            return;
        };
        if !root.contains(Some(&item)) {
            return;
        }
        let Ok(Some(anchor)) = item.closest("[aria-controls]") else {
            return;
        };
        let Some(id) = anchor.get_attribute("aria-controls") else {
            return;
        };
        let Some(chart_root) = resolve_chart_root(root, &id) else {
            return;
        };
        if let Some(value) = item.get_attribute("data-value") {
            set_dom_attribute(&chart_root, "data-range", &value);
        }
        sync_chart(root, &chart_root);
    }

    /// `root` 配下の click イベントを凡例 trigger・期間切替 item の
    /// 双方へ振り分ける（`preventDefault`/`stopPropagation` は呼ばない、
    /// モジュール doc「Runtime への統合」節）。
    fn handle_click(root: &Element, event: &Event) {
        let Some(target) = event_target_element(event) else {
            return;
        };
        if handle_legend_trigger_click(root, &target) {
            return;
        }
        handle_range_item_click(root, &target);
    }

    /// `root` 配下の構造変化（`Runtime::rerender_subtree` によるアプリ
    /// 再描画等）を検知して [`sync_all`] を再適用する（`chart.rs::
    /// wiring::wire_rerender_observer` と同型。`attributes` は監視せず
    /// `childList`/`subtree` のみのため、本モジュール自身の属性書き込みが
    /// 再度自分自身を呼び出す自己発火ループは起きない）。
    fn wire_rerender_observer(root: Element) -> Result<(), JsValue> {
        let target = root.clone();
        let closure = Closure::wrap(Box::new(
            move |_records: js_sys::Array, _observer: MutationObserver| {
                sync_all(&target);
            },
        ) as Box<dyn FnMut(js_sys::Array, MutationObserver)>);
        let observer = MutationObserver::new(closure.as_ref().unchecked_ref())?;
        let mut init = MutationObserverInit::new();
        init.set_child_list(true);
        init.set_subtree(true);
        observer.observe_with_options(&root, &init)?;
        closure.forget();
        Ok(())
    }

    /// charts の期間切替・凡例系列トグルを `root` へ配線する
    /// エントリポイント（`crate::Runtime::wire_chart_range` から
    /// `Self::wire_chart` の直後に呼ばれる）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback`/`MutationObserver::new` の
    /// 失敗を伝播する。
    pub fn wire_chart_range_events(root: Element) -> Result<(), JsValue> {
        let listener_root = root.clone();
        let closure = Closure::wrap(Box::new(move |event: Event| {
            handle_click(&listener_root, &event);
        }) as Box<dyn FnMut(Event)>);
        root.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();

        // マウント直後に 1 回同期する（SSR は `data-range` の初期値を
        // 出力するのみで範囲→カテゴリの写像・非表示化は行わないため、
        // 期間切替コントロールが既定で絞り込み状態を宣言している場合に
        // 初期表示から反映させる必要がある、モジュール doc「同期」節）。
        sync_all(&root);

        wire_rerender_observer(root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_range_bound_accepts_non_negative_integers() {
        assert_eq!(parse_range_bound("0"), Some(0));
        assert_eq!(parse_range_bound("12"), Some(12));
    }

    #[test]
    fn parse_range_bound_rejects_negative_and_non_numeric() {
        assert_eq!(parse_range_bound("-1"), None);
        assert_eq!(parse_range_bound("abc"), None);
        assert_eq!(parse_range_bound(""), None);
        assert_eq!(parse_range_bound("1.5"), None);
    }

    #[test]
    fn resolve_range_uses_defaults_when_bounds_missing() {
        assert_eq!(resolve_range(None, None, 5), Some((0, 5)));
        assert_eq!(resolve_range(Some(2), None, 5), Some((2, 5)));
        assert_eq!(resolve_range(None, Some(3), 5), Some((0, 3)));
    }

    #[test]
    fn resolve_range_rejects_to_not_greater_than_from() {
        assert_eq!(resolve_range(Some(3), Some(1), 5), None);
        assert_eq!(resolve_range(Some(3), Some(3), 5), None);
        assert_eq!(resolve_range(None, None, 0), None);
    }

    #[test]
    fn category_hidden_by_range_marks_indices_outside_half_open_interval() {
        let range = Some((1, 3));
        assert!(category_hidden_by_range(0, range));
        assert!(!category_hidden_by_range(1, range));
        assert!(!category_hidden_by_range(2, range));
        assert!(category_hidden_by_range(3, range));
    }

    #[test]
    fn category_hidden_by_range_none_hides_nothing() {
        assert!(!category_hidden_by_range(0, None));
        assert!(!category_hidden_by_range(100, None));
    }

    #[test]
    fn is_indexed_element_hidden_combines_range_and_legend_sets() {
        let hidden_categories = vec![5_usize];
        let hidden_series = vec!["b".to_string()];
        // カテゴリ 0: 系列 a は可視。
        assert!(!is_indexed_element_hidden(
            0,
            Some("a"),
            &hidden_categories,
            &hidden_series,
            None
        ));
        // 系列 b は凡例で非表示。
        assert!(is_indexed_element_hidden(
            0,
            Some("b"),
            &hidden_categories,
            &hidden_series,
            None
        ));
        // カテゴリ 5 は category_legend で非表示。
        assert!(is_indexed_element_hidden(
            5,
            Some("a"),
            &hidden_categories,
            &hidden_series,
            None
        ));
        // 範囲外カテゴリ。
        assert!(is_indexed_element_hidden(
            9,
            Some("a"),
            &hidden_categories,
            &hidden_series,
            Some((0, 3))
        ));
        // hit-area は series を持たない場合がある（None）。
        assert!(!is_indexed_element_hidden(
            1,
            None,
            &hidden_categories,
            &hidden_series,
            Some((0, 3))
        ));
    }
}
