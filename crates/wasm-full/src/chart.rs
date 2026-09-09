//! charts（`fandhe-frontend-pre-styled-ui` `charts::tooltip` モジュール）の
//! ポインタ追従・キーボード移動・タッチ操作でツールチップと hover 強調を
//! 切り替える配線（イシュー #2130、親 #2128、祖父トラッキング参照軸
//! #2001）。
//!
//! `crates/pre-styled-ui/src/charts/tooltip.rs` は bar/line/area/sparkline/
//! pie/donut/radial/radar/scatter の各チャートへ、SSR 時点で
//!
//! - 透明な hit-area（`data-scope="chart" data-part="hit-area"
//!   data-index="<n>" [data-series="<name>"]`。`fill="none"
//!   pointer-events="none" tabindex="-1"`）を `<svg>` 末尾へ、
//! - `<svg>` の直後の兄弟として `tooltip-layer`（`aria-hidden="true"
//!   position: absolute; inset: 0; pointer-events: none`）とその子
//!   `tooltip`（既定 `hidden`）を、
//!
//! それぞれ出力する（イシュー #2129、`crates/pre-styled-ui/src/charts/
//! tooltip.rs` モジュール doc「各チャートへの引き継ぎ契約」節が本モジュール
//! への唯一のロケータ契約）。本モジュールはこの静的マークアップに対し、
//! `pointermove`/タッチ `pointerdown`/`focusin`+矢印キーの 3 経路で
//! 「どの hit-area が指されているか」を判定し、対応する `tooltip` の
//! `hidden`・視覚要素の `data-active`・位置カスタムプロパティ
//! （`--fandhe-chart-tooltip-x/y`）を切り替える。**文字列生成・値の整形は
//! 一切行わない**（REQ-1、`.claude/rules/coding-rust.md` §「数値・日時
//! 整形は UI コンポーネント層の責務外」と同じ判断軸）。`data-active` 等を
//! CSS で消費する側は #2131 が担う。
//!
//! # 2 層構成（`sidebar.rs`/`angle_slider.rs` と同型）
//!
//! - 純粋ロジック層（[`hit_area_next_index`]/[`is_sticky_pointer`]/
//!   [`anchor_relative`]/[`hit_area_anchor`]/[`matches_key`]）は web-sys に
//!   依存せず、native の `cargo test` で検証できる。
//! - 配線層（[`wiring`]）のみ `#[cfg(target_arch = "wasm32")]` でゲート
//!   する。`sidebar`/`angle_slider` と同じく `pointerdown`/`pointermove`
//!   のような click/input 以外のイベント種別を扱うため
//!   `crate::headless::MAPPING_TABLE`（同期的な (scope, part) → action の
//!   静的マッピング）には**乗せない**。
//!
//! # Runtime への統合
//!
//! [`wiring::wire_chart_events`] は `crate::Runtime::mount`/
//! `Runtime::hydrate` の双方から `Self::wire_sidebar` の直後に組み込まれる
//! （`crate::Runtime::wire_chart` 参照）。`dispatch` チャネル（`on_action`）
//! を一切持たない属性専用配線であり（`sidebar::wire_sidebar_events` と
//! 同型）、状態機械へ波及しないため自動配線して安全である
//! （`Self::wire_sidebar_dispatch` のようなオプトイン API は不要）。
//!
//! # ロケータ契約（fail-closed）
//!
//! - hit-area: イベント target から
//!   `closest('[data-scope="chart"][data-part="hit-area"]')`（静的セレクタ）。
//! - svg: hit-area の `closest("svg")`。
//! - layer: `svg.next_element_sibling()` が
//!   `[data-scope="chart"][data-part="tooltip-layer"]` であるもの。
//! - 位置基準: layer は `position: absolute; inset: 0` のため layer 自身の
//!   `getBoundingClientRect()` を containing block として使う。
//!
//! いずれか欠落（`show_tooltip: false` で出力されたチャート・未知構造）は
//! no-op とする。`data-series`/`data-index` は利用者由来の任意文字列の
//! ため、`query_selector` へ絶対に補間しない（`security.md` A03）。マッチ
//! ング判定は常に `svg.query_selector_all("[data-index]")`/`layer` の子の
//! ような**静的セレクタ**で列挙し、`get_attribute` の戻り値を Rust 側で
//! 比較する（[`matches_key`]）。
//!
//! # 「最近傍点」の決定（幾何計算を行わない）
//!
//! 最近傍点はイベント target が属する hit-area そのものである。
//! `crates/pre-styled-ui/src/charts/` は SSR 時点でプロット領域を
//! カテゴリ帯（bar/line/area/sparkline）・扇形/リング（pie/donut/radial/
//! radar）・点円（scatter）に分割済みであり、これが最近傍判定の実体で
//! ある。ランタイムでの幾何計算（bounding rect 距離）は扇形で誤り、
//! scatter は参照実装（shadcn/recharts）も item hover のみのため行わない。
//!
//! # マッチング規則（[`matches_key`]、#2131 が CSS 側で依拠する契約）
//!
//! 同一 `<svg>` と対応 layer の中で、候補要素は次のとき一致とみなす:
//! `data-index` が hit-area の `data-index` と文字列一致し、かつ hit-area
//! が `data-series` を持つ場合は候補の `data-series` も一致必須（hit-area
//! が `data-series` を持たない場合は候補側の `data-series` を無視する。
//! bar は棒に `data-index`+`data-series` を持つが帯 hit-area は
//! `data-series` を持たない。scatter は両方持つ）。
//!
//! # `data-active` の既定値との共存
//!
//! `crates/pre-styled-ui/src/charts/tooltip.rs` は `data-active` を出力
//! しないが、`bar_chart`/`donut_chart` は独立した `active_index` プロパティ
//! で静的な `data-active`（開発者指定のハイライト）を出力し得る
//! （`crates/pre-styled-ui/src/charts/bar_chart.rs` 参照）。セッション
//! 開始時に `data-active` を持つ既存要素と、既定表示中の tooltip を
//! スナップショットし、セッション終了時にその状態へ復元する
//! （[`wiring::close_session`]）。
//!
//! # スコープ外（イシュー #2130 §9）
//!
//! - `svg_root` の `role="img"` 内にフォーカス可能な hit-area を置く a11y
//!   問題（#2261 の申し送り、是正は pre-styled-ui 側の別 Issue）。
//! - bar/scatter 以外の視覚要素への `data-index` 付与・消費 CSS・Demo
//!   （#2131）。
//! - 構造再描画（[`crate::Runtime::rerender_subtree`]）がセッション中の
//!   `<svg>`/layer を差し替えた場合の要素再解決（`angle_slider::wiring`
//!   の `PartKey`/`DragState` のような安定識別子ベースの追跡は行わない。
//!   本モジュールが扱うのは離散的な hover/focus 切替でありドラッグ状態を
//!   跨がないため、再描画を挟むとセッションは次の入力イベントで新規に
//!   開始し直される）。
//! - Shift+矢印での複数ステップ移動、pointer capture を使うドラッグ追従。

/// 本モジュールの anatomy scope（`crates/pre-styled-ui/src/charts/
/// tooltip.rs` の `SCOPE` と共有）。
///
/// 以下の定数群は wasm32 配線層（`wiring`）のみが参照するため、native の
/// 非 wasm ビルドでは未使用と検出される（`angle_slider::ROOT_PART` と
/// 同じ理由・同じ抑制方針。ロジックが不要という意味ではない）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const SCOPE: &str = "chart";
/// tooltip-layer パーツ名。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const TOOLTIP_LAYER_PART: &str = "tooltip-layer";
/// `data-active`（視覚要素の強調表示）属性名。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const ACTIVE_ATTR: &str = "data-active";
/// `data-index` 属性名。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const INDEX_ATTR: &str = "data-index";
/// `data-series` 属性名。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const SERIES_ATTR: &str = "data-series";
/// ツールチップ位置カスタムプロパティ（X 座標、layer 相対 px）。
/// `crates/pre-styled-ui/src/charts/tooltip.rs` の `tooltip` slot CSS が
/// `var(--fandhe-chart-tooltip-x, 0px)` で読む。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const VAR_X: &str = "--fandhe-chart-tooltip-x";
/// ツールチップ位置カスタムプロパティ（Y 座標、layer 相対 px）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const VAR_Y: &str = "--fandhe-chart-tooltip-y";

/// `root` 配下の hit-area 要素を列挙する静的セレクタ（値を補間しない、
/// `security.md` A03）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const HIT_AREA_SELECTOR: &str = "[data-scope=\"chart\"][data-part=\"hit-area\"]";
/// tooltip 要素を列挙する静的セレクタ。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const TOOLTIP_SELECTOR: &str = "[data-scope=\"chart\"][data-part=\"tooltip\"]";
/// `data-index` を持つ全要素（hit-area・視覚要素双方）を列挙する静的
/// セレクタ。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const INDEXED_SELECTOR: &str = "[data-index]";

/// hit-area 集合上でのキーボード移動先インデックスを計算する純粋関数
/// （web-sys 非依存、native `cargo test` 可）。
///
/// `ArrowRight`/`ArrowDown` は `+1`、`ArrowLeft`/`ArrowUp` は `-1`
/// （両軸を同等に扱う。scatter/radar のような 2 次元配置でも hit-area の
/// 文書順は 1 列であるため軸の区別を要求しない）、`Home` は先頭、`End` は
/// 末尾へ移動する。**非循環**（端でのさらなる移動キーは `None`）。修飾
/// キー（Ctrl/Alt/Meta）付き・未知キー・`len == 0`・`current` が範囲外の
/// ときは `None`（no-op、呼び出し側は `prevent_default` を呼ばない）。
#[must_use]
pub fn hit_area_next_index(
    current: usize,
    len: usize,
    key: &str,
    modifiers: crate::keynav::Modifiers,
) -> Option<usize> {
    if modifiers.any() || len == 0 || current >= len {
        return None;
    }
    match key {
        "Home" => Some(0),
        "End" => Some(len - 1),
        "ArrowRight" | "ArrowDown" => {
            if current + 1 < len {
                Some(current + 1)
            } else {
                None
            }
        }
        "ArrowLeft" | "ArrowUp" => current.checked_sub(1),
        _ => None,
    }
}

/// `PointerEvent::pointer_type()` が「タッチ由来でセッションを sticky
/// （チャート外タップまで開いたままにする）にすべき」かどうかを判定する
/// 純粋関数。mouse/pen は hover 経路（`pointermove`/`pointerout`）に
/// 任せるため `false`。
#[must_use]
pub fn is_sticky_pointer(pointer_type: &str) -> bool {
    pointer_type == "touch"
}

/// クライアント座標 `(x, y)` を、`layer` の `getBoundingClientRect()`
/// 左上 `(rect_left, rect_top)` を基準にした相対座標へ変換する（`layer`
/// は `position: absolute; inset: 0` のため、この相対座標がそのまま
/// [`VAR_X`]/[`VAR_Y`] へ書ける値になる）。
#[must_use]
pub fn anchor_relative(rect_left: f64, rect_top: f64, x: f64, y: f64) -> (f64, f64) {
    (x - rect_left, y - rect_top)
}

/// hit-area 自身の `getBoundingClientRect()`（`left`/`top`/`width`）から、
/// キーボード・タッチ操作時のツールチップ位置に使う「上辺中央」の
/// クライアント座標を求める。
#[must_use]
pub fn hit_area_anchor(left: f64, top: f64, width: f64) -> (f64, f64) {
    (left + width / 2.0, top)
}

/// hit-area の `(data-index, data-series)` と候補要素の
/// `(data-index, data-series)` が一致するかを判定する純粋関数（モジュール
/// doc「マッチング規則」節）。
///
/// `hit_index` は候補の `data-index` と文字列一致必須。`hit_series` が
/// `Some` のときは候補の `data-series` も一致必須、`None`（hit-area が
/// `data-series` を持たない、例: bar の帯）のときは候補側の
/// `data-series` を無視する。
#[must_use]
pub fn matches_key(
    hit_index: &str,
    hit_series: Option<&str>,
    candidate_index: Option<&str>,
    candidate_series: Option<&str>,
) -> bool {
    if candidate_index != Some(hit_index) {
        return false;
    }
    match hit_series {
        Some(series) => candidate_series == Some(series),
        None => true,
    }
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        anchor_relative, hit_area_anchor, hit_area_next_index, is_sticky_pointer, matches_key,
        ACTIVE_ATTR, HIT_AREA_SELECTOR, INDEXED_SELECTOR, INDEX_ATTR, SCOPE, SERIES_ATTR,
        TOOLTIP_LAYER_PART, TOOLTIP_SELECTOR, VAR_X, VAR_Y,
    };
    use crate::keynav::Modifiers;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{
        Element, Event, FocusEvent, HtmlElement, KeyboardEvent, MouseEvent, MutationObserver,
        MutationObserverInit, PointerEvent, SvgElement,
    };

    /// `root` 配下で `selector` に一致する要素を文書順の `Vec` として返す
    /// （`sidebar::wiring::query_all` と同型）。
    fn query_all(root: &Element, selector: &str) -> Vec<Element> {
        let Ok(list) = root.query_selector_all(selector) else {
            return Vec::new();
        };
        let len = list.length();
        let mut out = Vec::with_capacity(len as usize);
        for i in 0..len {
            if let Some(node) = list.get(i) {
                if let Ok(element) = node.dyn_into::<Element>() {
                    out.push(element);
                }
            }
        }
        out
    }

    /// `event.target()` を `Element` として取得する（`Text` ノード等は
    /// `None`）。
    fn event_target_element(event: &Event) -> Option<Element> {
        event.target()?.dyn_into::<Element>().ok()
    }

    /// `start` から祖先方向へ `closest` で hit-area を探し、`root` 配下に
    /// 収まっていることを検証する（モジュール doc「ロケータ契約」節）。
    fn closest_hit_area(root: &Element, start: &Element) -> Option<Element> {
        let found = start.closest(HIT_AREA_SELECTOR).ok().flatten()?;
        if root.contains(Some(&found)) {
            Some(found)
        } else {
            None
        }
    }

    /// hit-area の属する `<svg>` を返す。
    fn svg_of(hit_area: &Element) -> Option<Element> {
        hit_area.closest("svg").ok().flatten()
    }

    /// `svg` の直後の兄弟が tooltip-layer であればそれを返す
    /// （モジュール doc「ロケータ契約」節、`crates/pre-styled-ui/src/
    /// charts/tooltip.rs` の SSR 出力契約）。
    fn layer_of(svg: &Element) -> Option<Element> {
        let sibling = svg.next_element_sibling()?;
        if sibling.get_attribute("data-scope").as_deref() == Some(SCOPE)
            && sibling.get_attribute("data-part").as_deref() == Some(TOOLTIP_LAYER_PART)
        {
            Some(sibling)
        } else {
            None
        }
    }

    /// `element` の `(data-index, data-series)` を読む。`data-index` を
    /// 持たない要素は `None`（hit-area・視覚要素とも必ず `data-index` を
    /// 持つ契約）。
    fn read_key(element: &Element) -> Option<(String, Option<String>)> {
        let index = element.get_attribute(INDEX_ATTR)?;
        let series = element.get_attribute(SERIES_ATTR);
        Some((index, series))
    }

    /// svg 内の `[data-index]` 要素のうち `data-active` を持つものの
    /// キー一覧（`active_index` 等が静的に付けた既定強調のスナップ
    /// ショット）。
    fn snapshot_active_keys(svg: &Element) -> Vec<(String, Option<String>)> {
        query_all(svg, INDEXED_SELECTOR)
            .into_iter()
            .filter(|el| el.has_attribute(ACTIVE_ATTR))
            .filter_map(|el| read_key(&el))
            .collect()
    }

    /// layer 内で `hidden` を持たない tooltip（SSR が既定表示にしている
    /// もの）のキー。無ければ `None`。
    fn snapshot_visible_tooltip(layer: &Element) -> Option<(String, Option<String>)> {
        query_all(layer, TOOLTIP_SELECTOR)
            .into_iter()
            .find(|el| !el.has_attribute("hidden"))
            .and_then(|el| read_key(&el))
    }

    /// 現在進行中のホバー/フォーカス/タッチセッション（1 チャート分）。
    struct Session {
        svg: Element,
        layer: Element,
        /// 外側クリックでセッションを閉じる際の判定境界（`frame`/各
        /// styled チャートの `root`。`svg`/`layer` の共通親、
        /// `crates/pre-styled-ui/src/charts/tooltip.rs::frame` 参照）。
        frame: Element,
        /// タッチ由来（`pointerdown`）で開始し、チャート外タップまで
        /// 開いたままにするセッションかどうか。
        sticky: bool,
        /// セッション開始時点の `data-active` 既定状態（閉鎖時に復元）。
        initial_active: Vec<(String, Option<String>)>,
        /// セッション開始時点の既定表示 tooltip（閉鎖時に復元）。
        initial_visible_tooltip: Option<(String, Option<String>)>,
    }

    /// 各イベント閉包が共有するセッション状態のハンドル。
    type SessionHandle = Rc<RefCell<Option<Session>>>;

    /// `layer` へポインタ/hit-area 相対座標からツールチップ位置カスタム
    /// プロパティを書き込む。数値以外の文字列は組み立てない（`format!`
    /// で唯一組み立てるのは `"{n}px"` のみ、REQ-1・`security.md` A03）。
    fn set_tooltip_position(layer: &Element, client_x: f64, client_y: f64) {
        let rect = layer.get_bounding_client_rect();
        let (x, y) = anchor_relative(rect.left(), rect.top(), client_x, client_y);
        if let Some(html) = layer.dyn_ref::<HtmlElement>() {
            let style = html.style();
            let _ = style.set_property(VAR_X, &format!("{x}px"));
            let _ = style.set_property(VAR_Y, &format!("{y}px"));
        }
    }

    /// hit-area の `getBoundingClientRect()` 上辺中央を、キーボード/
    /// タッチ操作時のツールチップ位置に使うクライアント座標として返す。
    fn hit_area_client_anchor(hit_area: &Element) -> (f64, f64) {
        let rect = hit_area.get_bounding_client_rect();
        hit_area_anchor(rect.left(), rect.top(), rect.width())
    }

    /// `hit_area` に対応する tooltip の表示・視覚要素の `data-active`・
    /// 位置カスタムプロパティを適用する（モジュール doc「マッチング
    /// 規則」節）。
    fn apply_highlight(
        layer: &Element,
        svg: &Element,
        hit_area: &Element,
        client_x: f64,
        client_y: f64,
    ) {
        let Some(index) = hit_area.get_attribute(INDEX_ATTR) else {
            return;
        };
        let series = hit_area.get_attribute(SERIES_ATTR);

        for tooltip in query_all(layer, TOOLTIP_SELECTOR) {
            let candidate_index = tooltip.get_attribute(INDEX_ATTR);
            let candidate_series = tooltip.get_attribute(SERIES_ATTR);
            if matches_key(
                &index,
                series.as_deref(),
                candidate_index.as_deref(),
                candidate_series.as_deref(),
            ) {
                let _ = tooltip.remove_attribute("hidden");
            } else {
                let _ = tooltip.set_attribute("hidden", "");
            }
        }
        for element in query_all(svg, INDEXED_SELECTOR) {
            let candidate_index = element.get_attribute(INDEX_ATTR);
            let candidate_series = element.get_attribute(SERIES_ATTR);
            if matches_key(
                &index,
                series.as_deref(),
                candidate_index.as_deref(),
                candidate_series.as_deref(),
            ) {
                let _ = element.set_attribute(ACTIVE_ATTR, "");
            } else {
                let _ = element.remove_attribute(ACTIVE_ATTR);
            }
        }
        set_tooltip_position(layer, client_x, client_y);
    }

    /// 進行中のセッションを閉じ、`data-active`/tooltip の可視状態をセッ
    /// ション開始時点のスナップショット（モジュール doc「`data-active` の
    /// 既定値との共存」節）へ復元する。セッションが無ければ no-op。
    fn close_session(handle: &SessionHandle) {
        let Some(session) = handle.borrow_mut().take() else {
            return;
        };
        for element in query_all(&session.svg, INDEXED_SELECTOR) {
            let is_initial = read_key(&element)
                .map(|key| session.initial_active.contains(&key))
                .unwrap_or(false);
            if is_initial {
                let _ = element.set_attribute(ACTIVE_ATTR, "");
            } else {
                let _ = element.remove_attribute(ACTIVE_ATTR);
            }
        }
        for tooltip in query_all(&session.layer, TOOLTIP_SELECTOR) {
            let is_initial = read_key(&tooltip)
                .map(|key| Some(key) == session.initial_visible_tooltip)
                .unwrap_or(false);
            if is_initial {
                let _ = tooltip.remove_attribute("hidden");
            } else {
                let _ = tooltip.set_attribute("hidden", "");
            }
        }
    }

    /// 現在のセッションが `svg` に対するものであれば `true`。
    fn session_matches_svg(handle: &SessionHandle, svg: &Element) -> bool {
        handle
            .borrow()
            .as_ref()
            .is_some_and(|session| session.svg == *svg)
    }

    /// hover/フォーカス/タッチ入力を集約し、セッションの開始・継続を行う
    /// （`sticky`: タッチ `pointerdown` 由来で `pointerout` では閉じない
    /// ことを示す）。別チャートへ移った場合は前セッションを
    /// [`close_session`] で閉じてから新規セッションを開く。既に同じ
    /// `svg` のセッションが開いている場合はスナップショットを取り直さず
    /// 強調とツールチップだけを更新する（`sticky` は真のときのみ昇格
    /// させ、非 sticky 側の呼び出しで sticky セッションを巻き戻さない）。
    fn begin_or_update_session(
        root: &Element,
        handle: &SessionHandle,
        hit_area: &Element,
        sticky: bool,
        client_x: f64,
        client_y: f64,
    ) {
        let Some(svg) = svg_of(hit_area) else {
            return;
        };
        if !root.contains(Some(&svg)) {
            return;
        }
        let Some(layer) = layer_of(&svg) else {
            return;
        };
        let Some(frame) = layer.parent_element() else {
            return;
        };

        if !session_matches_svg(handle, &svg) {
            close_session(handle);
            let initial_active = snapshot_active_keys(&svg);
            let initial_visible_tooltip = snapshot_visible_tooltip(&layer);
            *handle.borrow_mut() = Some(Session {
                svg: svg.clone(),
                layer: layer.clone(),
                frame,
                sticky,
                initial_active,
                initial_visible_tooltip,
            });
        } else if sticky {
            if let Some(session) = handle.borrow_mut().as_mut() {
                session.sticky = true;
            }
        }

        apply_highlight(&layer, &svg, hit_area, client_x, client_y);
    }

    /// `root` へ pointermove（hover 追従）を配線する。sticky（タッチ）
    /// セッション中は無視する。hit-area 以外（軸ラベル相当）への移動は
    /// 非 sticky セッションを閉じる。
    fn handle_pointermove(root: &Element, handle: &SessionHandle, event: &Event) {
        let sticky = handle.borrow().as_ref().map(|s| s.sticky).unwrap_or(false);
        if sticky {
            return;
        }
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        let Some(target) = event_target_element(event) else {
            return;
        };
        let client_x = f64::from(pointer_event.client_x());
        let client_y = f64::from(pointer_event.client_y());
        match closest_hit_area(root, &target) {
            Some(hit_area) => {
                begin_or_update_session(root, handle, &hit_area, false, client_x, client_y);
            }
            None => close_session(handle),
        }
    }

    /// `root` へ pointerdown（タッチ由来の sticky セッション開始）を
    /// 配線する。mouse/pen は no-op（hover 経路に任せる）。
    fn handle_pointerdown(root: &Element, handle: &SessionHandle, event: &Event) {
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        if !is_sticky_pointer(&pointer_event.pointer_type()) {
            return;
        }
        let Some(target) = event_target_element(event) else {
            return;
        };
        let Some(hit_area) = closest_hit_area(root, &target) else {
            return;
        };
        let client_x = f64::from(pointer_event.client_x());
        let client_y = f64::from(pointer_event.client_y());
        begin_or_update_session(root, handle, &hit_area, true, client_x, client_y);
    }

    /// `root` へ pointerout を配線する。`related_target` が現在セッション
    /// の `svg` 内でなければ非 sticky セッションを閉じる（`pointerleave`
    /// はバブリングしないため `pointerout` + `related_target` 判定、
    /// `sidebar::wiring` の `pointerover` 判定と同型）。
    fn handle_pointerout(handle: &SessionHandle, event: &Event) {
        let sticky = handle.borrow().as_ref().map(|s| s.sticky).unwrap_or(false);
        if sticky {
            return;
        }
        let Some(session_svg) = handle.borrow().as_ref().map(|s| s.svg.clone()) else {
            return;
        };
        let related = event
            .dyn_ref::<MouseEvent>()
            .and_then(web_sys::MouseEvent::related_target)
            .and_then(|target| target.dyn_into::<Element>().ok());
        let still_within = related
            .as_ref()
            .map(|element| session_svg.contains(Some(element)))
            .unwrap_or(false);
        if !still_within {
            close_session(handle);
        }
    }

    /// `root` へ pointercancel を配線する。非 sticky セッションを閉じる。
    fn handle_pointercancel(handle: &SessionHandle) {
        let sticky = handle.borrow().as_ref().map(|s| s.sticky).unwrap_or(false);
        if !sticky {
            close_session(handle);
        }
    }

    /// `root` へ keydown を配線する。target が hit-area のときのみ処理し、
    /// 修飾キー付き・未知キーは no-op（`prevent_default` を呼ばない）。
    fn handle_keydown(root: &Element, handle: &SessionHandle, event: &Event) {
        let Some(keyboard_event) = event.dyn_ref::<KeyboardEvent>() else {
            return;
        };
        let Some(target) = event_target_element(event) else {
            return;
        };
        let Some(hit_area) = closest_hit_area(root, &target) else {
            return;
        };
        if hit_area != target {
            return;
        }

        let modifiers = Modifiers {
            ctrl: keyboard_event.ctrl_key(),
            alt: keyboard_event.alt_key(),
            meta: keyboard_event.meta_key(),
        };
        let key = keyboard_event.key();
        if key == "Escape" {
            if !modifiers.any() {
                close_session(handle);
            }
            return;
        }

        let Some(svg) = svg_of(&hit_area) else {
            return;
        };
        let hit_areas = query_all(&svg, HIT_AREA_SELECTOR);
        let Some(current) = hit_areas.iter().position(|el| *el == hit_area) else {
            return;
        };
        let Some(next) = hit_area_next_index(current, hit_areas.len(), &key, modifiers) else {
            return;
        };
        keyboard_event.prevent_default();
        let Some(next_element) = hit_areas.get(next) else {
            return;
        };
        for (i, element) in hit_areas.iter().enumerate() {
            let _ = element.set_attribute("tabindex", if i == next { "0" } else { "-1" });
        }
        if let Some(svg_element) = next_element.dyn_ref::<SvgElement>() {
            let _ = svg_element.focus();
        }
        let (client_x, client_y) = hit_area_client_anchor(next_element);
        begin_or_update_session(root, handle, next_element, false, client_x, client_y);
    }

    /// `root` へ focusin を配線する。target が hit-area のとき roving
    /// tabindex を更新し、非 sticky セッションを開く。
    fn handle_focusin(root: &Element, handle: &SessionHandle, event: &Event) {
        let Some(target) = event_target_element(event) else {
            return;
        };
        let Some(hit_area) = closest_hit_area(root, &target) else {
            return;
        };
        if hit_area != target {
            return;
        }
        let Some(svg) = svg_of(&hit_area) else {
            return;
        };
        for element in query_all(&svg, HIT_AREA_SELECTOR) {
            let is_current = element == hit_area;
            let _ = element.set_attribute("tabindex", if is_current { "0" } else { "-1" });
        }
        let (client_x, client_y) = hit_area_client_anchor(&hit_area);
        begin_or_update_session(root, handle, &hit_area, false, client_x, client_y);
    }

    /// `root` へ focusout を配線する。`related_target` が同一 svg の
    /// hit-area でなければ非 sticky セッションを閉じる。
    fn handle_focusout(root: &Element, handle: &SessionHandle, event: &Event) {
        let Some(target) = event_target_element(event) else {
            return;
        };
        let Some(hit_area) = closest_hit_area(root, &target) else {
            return;
        };
        if hit_area != target {
            return;
        }
        let Some(focus_event) = event.dyn_ref::<FocusEvent>() else {
            return;
        };
        let related = focus_event
            .related_target()
            .and_then(|target| target.dyn_into::<Element>().ok());
        let still_within = related
            .as_ref()
            .and_then(|element| closest_hit_area(root, element))
            .is_some();
        if !still_within {
            close_session(handle);
        }
    }

    /// document へ登録する pointerdown（sticky セッションのチャート外
    /// タップ閉鎖）。`frame`（`svg`/`layer` の共通親）の外側への tap の
    /// ときのみ閉じる。
    fn handle_document_pointerdown(handle: &SessionHandle, event: &Event) {
        let Some(frame) = handle
            .borrow()
            .as_ref()
            .filter(|session| session.sticky)
            .map(|session| session.frame.clone())
        else {
            return;
        };
        let Some(target) = event_target_element(event) else {
            return;
        };
        if !frame.contains(Some(&target)) {
            close_session(handle);
        }
    }

    /// `root` 配下の全 hit-area を「クリック/タッチ/フォーカス可能」な
    /// 状態へ冪等にエンハンスする（wire 直後、および再描画後の
    /// `MutationObserver` 再適用の双方から呼ばれる）。SSR は
    /// `pointer-events="none" tabindex="-1"` で出力する（JS 無効時に
    /// 既存の `:hover`/`<title>` を妨げない progressive enhancement、
    /// `crates/pre-styled-ui/src/charts/tooltip.rs` 参照）。
    ///
    /// 各 `<svg>` 内の先頭 hit-area にのみ `tabindex="0"` を設定する
    /// （roving tabindex の初期値）。既に `tabindex="0"` を持つ hit-area
    /// が 1 つでもあれば触らない（キーボード操作で移動済みの状態を
    /// 再描画で巻き戻さないため）。
    fn enhance(root: &Element) {
        for svg in query_all(root, "svg") {
            let hit_areas = query_all(&svg, HIT_AREA_SELECTOR);
            if hit_areas.is_empty() {
                continue;
            }
            for element in &hit_areas {
                let _ = element.set_attribute("pointer-events", "all");
            }
            let has_roving_tabindex = hit_areas
                .iter()
                .any(|element| element.get_attribute("tabindex").as_deref() == Some("0"));
            if !has_roving_tabindex {
                if let Some(first) = hit_areas.first() {
                    let _ = first.set_attribute("tabindex", "0");
                }
            }
        }
    }

    /// 再描画（[`crate::Runtime::rerender_subtree`] による `root` 配下の
    /// 丸ごと差し替え）で hit-area が SSR 値（`pointer-events="none"
    /// tabindex="-1"`）へ戻るため、`MutationObserver`（`childList`/
    /// `subtree` のみを監視し `attributes` は監視しない＝自己発火ループを
    /// 構造的に回避する、`sidebar::wiring` と同型）で [`enhance`] を
    /// 再適用する。
    fn wire_rerender_observer(root: &Element) -> Result<(), JsValue> {
        let observed_root = root.clone();
        let callback = Closure::<dyn FnMut(js_sys::Array, MutationObserver)>::new(
            move |_records: js_sys::Array, _observer: MutationObserver| {
                enhance(&observed_root);
            },
        );
        let observer = MutationObserver::new(callback.as_ref().unchecked_ref())?;
        let init = MutationObserverInit::new();
        init.set_child_list(true);
        init.set_subtree(true);
        observer.observe_with_options(root, &init)?;
        callback.forget();
        Ok(())
    }

    /// `root` 配下の chart hit-area へ pointermove/pointerdown/pointerout/
    /// pointercancel/keydown/focusin/focusout の配線を 1 回だけ登録する
    /// （マウント時 1 回契約、`crate::Runtime::wire_chart` から呼ばれる）。
    /// `root` 配下に hit-area が 1 つも無ければリスナーを一切登録せず
    /// `Ok(())` を返す（非搭載アプリ・`show_tooltip: false` 構成への
    /// 副作用なし契約、`splitter::wire_splitter_events` と同型）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback`/`MutationObserver::new` の失敗を
    /// 伝播する。
    pub fn wire_chart_events(root: Element) -> Result<(), JsValue> {
        if query_all(&root, HIT_AREA_SELECTOR).is_empty() {
            return Ok(());
        }

        enhance(&root);

        let handle: SessionHandle = Rc::new(RefCell::new(None));

        macro_rules! wire_root_event {
            ($event_name:literal, $handler:expr) => {{
                let root_ref = root.clone();
                let handle_ref = handle.clone();
                let closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
                    $handler(&root_ref, &handle_ref, &event);
                });
                root.add_event_listener_with_callback(
                    $event_name,
                    closure.as_ref().unchecked_ref(),
                )?;
                closure.forget();
            }};
        }

        wire_root_event!(
            "pointermove",
            |root: &Element, handle: &SessionHandle, event: &Event| {
                handle_pointermove(root, handle, event);
            }
        );
        wire_root_event!(
            "pointerdown",
            |root: &Element, handle: &SessionHandle, event: &Event| {
                handle_pointerdown(root, handle, event);
            }
        );
        wire_root_event!(
            "pointerout",
            |_root: &Element, handle: &SessionHandle, event: &Event| {
                handle_pointerout(handle, event);
            }
        );
        wire_root_event!(
            "pointercancel",
            |_root: &Element, handle: &SessionHandle, _event: &Event| {
                handle_pointercancel(handle);
            }
        );
        wire_root_event!(
            "keydown",
            |root: &Element, handle: &SessionHandle, event: &Event| {
                handle_keydown(root, handle, event);
            }
        );
        wire_root_event!(
            "focusin",
            |root: &Element, handle: &SessionHandle, event: &Event| {
                handle_focusin(root, handle, event);
            }
        );
        wire_root_event!(
            "focusout",
            |root: &Element, handle: &SessionHandle, event: &Event| {
                handle_focusout(root, handle, event);
            }
        );

        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("chart: no document"))?;
        let document_handle = handle.clone();
        let document_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_document_pointerdown(&document_handle, &event);
        });
        document.add_event_listener_with_callback(
            "pointerdown",
            document_closure.as_ref().unchecked_ref(),
        )?;
        document_closure.forget();

        wire_rerender_observer(&root)?;

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_chart_events;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keynav::Modifiers;

    fn no_mods() -> Modifiers {
        Modifiers::default()
    }

    #[test]
    fn hit_area_next_index_arrow_right_and_down_advance() {
        assert_eq!(hit_area_next_index(0, 3, "ArrowRight", no_mods()), Some(1));
        assert_eq!(hit_area_next_index(0, 3, "ArrowDown", no_mods()), Some(1));
    }

    #[test]
    fn hit_area_next_index_arrow_left_and_up_retreat() {
        assert_eq!(hit_area_next_index(1, 3, "ArrowLeft", no_mods()), Some(0));
        assert_eq!(hit_area_next_index(1, 3, "ArrowUp", no_mods()), Some(0));
    }

    #[test]
    fn hit_area_next_index_home_and_end() {
        assert_eq!(hit_area_next_index(1, 5, "Home", no_mods()), Some(0));
        assert_eq!(hit_area_next_index(1, 5, "End", no_mods()), Some(4));
    }

    #[test]
    fn hit_area_next_index_is_non_circular_at_edges() {
        assert_eq!(hit_area_next_index(2, 3, "ArrowRight", no_mods()), None);
        assert_eq!(hit_area_next_index(0, 3, "ArrowLeft", no_mods()), None);
    }

    #[test]
    fn hit_area_next_index_rejects_modifiers_and_unknown_keys() {
        let ctrl = Modifiers {
            ctrl: true,
            ..Modifiers::default()
        };
        assert_eq!(hit_area_next_index(0, 3, "ArrowRight", ctrl), None);
        assert_eq!(hit_area_next_index(0, 3, "PageDown", no_mods()), None);
    }

    #[test]
    fn hit_area_next_index_handles_zero_length_and_out_of_range() {
        assert_eq!(hit_area_next_index(0, 0, "ArrowRight", no_mods()), None);
        assert_eq!(hit_area_next_index(5, 3, "ArrowRight", no_mods()), None);
    }

    #[test]
    fn is_sticky_pointer_only_true_for_touch() {
        assert!(is_sticky_pointer("touch"));
        assert!(!is_sticky_pointer("mouse"));
        assert!(!is_sticky_pointer("pen"));
        assert!(!is_sticky_pointer(""));
    }

    #[test]
    fn anchor_relative_subtracts_rect_origin() {
        assert_eq!(anchor_relative(10.0, 20.0, 15.0, 25.0), (5.0, 5.0));
    }

    #[test]
    fn hit_area_anchor_is_top_center() {
        assert_eq!(hit_area_anchor(10.0, 20.0, 8.0), (14.0, 20.0));
    }

    #[test]
    fn matches_key_requires_index_equality() {
        assert!(!matches_key("0", None, Some("1"), None));
        assert!(!matches_key("0", None, None, None));
    }

    #[test]
    fn matches_key_ignores_candidate_series_when_hit_has_none() {
        assert!(matches_key("2", None, Some("2"), Some("visits")));
        assert!(matches_key("2", None, Some("2"), None));
    }

    #[test]
    fn matches_key_requires_series_equality_when_hit_has_series() {
        assert!(matches_key("1", Some("visits"), Some("1"), Some("visits")));
        assert!(!matches_key("1", Some("visits"), Some("1"), Some("clicks")));
        assert!(!matches_key("1", Some("visits"), Some("1"), None));
    }
}
