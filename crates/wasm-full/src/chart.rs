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
//!   の `PartKey`/`DragState` のような安定識別子ベースの追跡は行わない）。
//!   セッションの `svg` が `root` 配下から失われたことは
//!   `wiring::wire_rerender_observer` が登録する `MutationObserver` が
//!   検知して `wiring::discard_stale_session` がセッションを破棄するため、
//!   次の入力イベントで新規にセッションが開始し直される（イシュー #2130
//!   レビュー指摘の是正、旧要素への「復帰しない」不具合を防ぐのが目的で
//!   あり、新 `<svg>` への引き継ぎ・位置追跡は行わない）。
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

/// hover（ポインタ）・focus（キーボード）を「開いたままにする独立した理由」
/// として扱い、セッションを閉じてよいかを判定する純粋関数（web-sys
/// 非依存、native `cargo test` 可）。sticky（タッチ）セッションは
/// `handle_document_pointerdown` のみが閉じる別経路のため対象外（呼び出し側
/// が `sticky` を先にガードする）。`hover_active`/`focus_active` の
/// いずれかが真であれば閉じない（Cursor Bugbot 指摘: 単一の非 sticky
/// セッションが pointermove/pointerout/focusout のいずれからも独立に
/// 閉じられ、互いの「開いたままにする理由」を打ち消していた不具合の是正、
/// イシュー #2130 レビュー）。
#[must_use]
pub fn should_close_session(hover_active: bool, focus_active: bool) -> bool {
    !hover_active && !focus_active
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        anchor_relative, hit_area_anchor, hit_area_next_index, is_sticky_pointer, matches_key,
        should_close_session, ACTIVE_ATTR, HIT_AREA_SELECTOR, INDEXED_SELECTOR, INDEX_ATTR, SCOPE,
        SERIES_ATTR, TOOLTIP_LAYER_PART, TOOLTIP_SELECTOR, VAR_X, VAR_Y,
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

    /// `element.set_attribute(name, value)` の薄いガード付きラッパー
    /// （イシュー #401 の `fw gate` `url_validation_check` 契約に準拠、
    /// `.claude/rules/security.md`）。本モジュールが書き込む属性
    /// （`hidden`/`data-active`/`tabindex`/`pointer-events`）はいずれも
    /// `&'static str` リテラルで固定された非 URL・非イベントハンドラ属性で
    /// あり実害はないが、`fandhe_frontend_core::url` のガード関数群
    /// （`is_event_handler_attr`/`is_url_attr`/`is_safe_url`/
    /// `is_safe_srcset`）を経由することで、将来 `name`/`value` が動的な
    /// 入力から組み立てられるよう変更された場合の防御としても機能する
    /// （`sidebar.rs`/`keynav.rs`/`headless_avatar.rs` の同名ラッパーと
    /// 同じガード方針）。
    fn set_dom_attribute(element: &Element, name: &str, value: &str) {
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

    /// セッションを開始・継続させたイベント種別（codex レビュー指摘:
    /// hover（ポインタ）と focus（キーボード）は独立して「開いたままに
    /// する理由」であり、[`Session::hover_active`]/[`Session::focus_active`]
    /// として個別に保持する。タッチは従来どおり [`Session::sticky`] を
    /// 昇格させるのみで、`hover_active`/`focus_active` には影響しない
    /// （sticky セッションは [`handle_document_pointerdown`] のみが
    /// 閉じる別経路のため）。
    enum Trigger {
        /// `pointermove`/タッチ以外の `pointerdown`。
        Pointer,
        /// タッチ由来の `pointerdown`（sticky セッションを開始）。
        Touch,
        /// `focusin`/矢印キーによるフォーカス移動。
        Keyboard,
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
        /// ポインタ（`pointermove`/`pointerdown` 非タッチ）が現在 hit-area
        /// 上にあるか。`pointermove`/`pointerout`/`pointercancel` が
        /// 更新する（sticky セッションでは未使用）。
        hover_active: bool,
        /// キーボードフォーカスが現在 hit-area 上にあるか。`focusin`/
        /// `focusout`/矢印キー移動が更新する（sticky セッションでは
        /// 未使用）。
        focus_active: bool,
        /// `hover_active` が最後に指していた hit-area（codex-review P1 /
        /// Bugbot 指摘: hover と focus が別の hit-area を指した状態で
        /// 片方が非活性化しても、残っている方が指す対象へ表示を戻せる
        /// よう、hover 由来の対象を個別に保持する。`hover_active` が
        /// `false` の間は参照しない）。
        hover_target: Option<Element>,
        /// `focus_active` が最後に指していた hit-area（`hover_target` と
        /// 対称。`focus_active` が `false` の間は参照しない）。
        focus_target: Option<Element>,
        /// セッション開始時点の `data-active` 既定状態（閉鎖時に復元）。
        initial_active: Vec<(String, Option<String>)>,
        /// セッション開始時点の既定表示 tooltip（閉鎖時に復元）。
        initial_visible_tooltip: Option<(String, Option<String>)>,
    }

    /// 各イベント閉包が共有するセッション状態のハンドル。
    /// 同じ `Runtime` 配下で同時に開き得る複数チャート分のセッションを
    /// 保持するハンドル（codex-review P1 是正、イシュー #2130 PR #2267）。
    /// `Option<Session>` 単一保持だった旧実装は、あるチャートへ
    /// キーボードフォーカスしたまま別チャートをポインタでホバーすると
    /// `begin_or_update_session` の svg 不一致分岐がフォーカス側の
    /// セッションを丸ごと `close_session` で破棄していた（hover と focus
    /// を独立した表示継続理由として扱うモジュール契約に反する）。`svg`
    /// をキーに 1 チャート 1 エントリで管理し、あるチャートへの操作が
    /// 他チャートのセッションへ波及しないようにする。同時に開くセッション
    /// 数は実用上ごく少数（hover 1 件 + focus 1 件程度）のため線形探索で
    /// 十分であり、`HashMap` 等の追加依存は不要。
    type SessionHandle = Rc<RefCell<Vec<Session>>>;

    /// `sessions` から `svg` に対応するセッションのインデックスを探す
    /// （`Element` の等価性は基底 `JsValue`/ノード同一性比較）。
    fn session_index_for_svg(sessions: &[Session], svg: &Element) -> Option<usize> {
        sessions.iter().position(|session| session.svg == *svg)
    }

    /// 現在 `hover_active` なセッションの `svg`（sticky セッションは明示的に
    /// 除外する。マウスでホバー中のセッションをタッチで sticky に昇格
    /// させても `hover_active` フラグ自体はそのまま残るため、フラグだけを
    /// 見ると sticky セッションも返ってしまい、後続の `pointerout`/
    /// `pointercancel` が `deactivate_hover` 経由でタッチ操作対象を誤って
    /// 非活性化・クローズしてしまう（codex P1 / Cursor Bugbot "Sticky
    /// session closes after hover" 指摘、イシュー #2130 PR #2267）。高々
    /// 1 件のはずだが、複数存在しても最初の 1 件を返せば十分＝ポインタは
    /// 単一のためここで選ばれなかった残りは次回の `pointerout`/
    /// `pointermove` で追随して解消される）。
    fn hover_active_svg(handle: &SessionHandle) -> Option<Element> {
        handle
            .borrow()
            .iter()
            .find(|session| session.hover_active && !session.sticky)
            .map(|session| session.svg.clone())
    }

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
                set_dom_attribute(&tooltip, "hidden", "");
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
                set_dom_attribute(&element, ACTIVE_ATTR, "");
            } else {
                let _ = element.remove_attribute(ACTIVE_ATTR);
            }
        }
        set_tooltip_position(layer, client_x, client_y);
    }

    /// 除去済み `session` の `data-active`/tooltip の可視状態をセッ
    /// ション開始時点のスナップショット（モジュール doc「`data-active` の
    /// 既定値との共存」節）へ復元する（[`close_session`] の実処理本体）。
    fn restore_session(session: &Session) {
        for element in query_all(&session.svg, INDEXED_SELECTOR) {
            let is_initial = read_key(&element)
                .map(|key| session.initial_active.contains(&key))
                .unwrap_or(false);
            if is_initial {
                set_dom_attribute(&element, ACTIVE_ATTR, "");
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
                set_dom_attribute(&tooltip, "hidden", "");
            }
        }
    }

    /// `svg` に対応するセッションのみを閉じ、[`restore_session`] で
    /// スナップショットへ復元する。他チャート（他 `svg`）のセッションには
    /// 一切触れない（codex-review P1 是正、イシュー #2130 PR #2267）。
    /// 該当セッションが無ければ no-op。
    fn close_session(handle: &SessionHandle, svg: &Element) {
        let removed = {
            let mut sessions = handle.borrow_mut();
            session_index_for_svg(&sessions, svg).map(|index| sessions.remove(index))
        };
        if let Some(session) = removed {
            restore_session(&session);
        }
    }

    /// hover/フォーカス/タッチ入力を集約し、セッションの開始・継続を行う
    /// （`trigger`: [`Trigger::Touch`] は `pointerout` では閉じない sticky
    /// セッションへ昇格させる）。別チャートへ移った場合は前セッションを
    /// [`close_session`] で閉じてから新規セッションを開く。既に同じ
    /// `svg` のセッションが開いている場合はスナップショットを取り直さず
    /// 強調とツールチップだけを更新する（`sticky`/`hover_active`/
    /// `focus_active` は `trigger` に応じて真のときのみ昇格させ、他方の
    /// 呼び出しで巻き戻さない。hover と focus は独立した「開いたままに
    /// する理由」であり、`should_close_session` が両方を見て判定する）。
    fn begin_or_update_session(
        root: &Element,
        handle: &SessionHandle,
        hit_area: &Element,
        trigger: Trigger,
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

        // ポインタ由来の更新で、別チャート（別 svg）のセッションが
        // `hover_active` を持ったまま残っていれば先に非活性化する
        // （通常は当該チャートを離れる `pointerout`/`pointercancel` が
        // 先行して処理するが、合成イベント等でそれを経由せず直接別
        // チャートへ移った場合の残留防止。`svg` が異なるセッションのみが
        // 対象で、`focus_active` を持つセッションは触れない）。
        if matches!(trigger, Trigger::Pointer) {
            let stale_hover_svg = handle
                .borrow()
                .iter()
                .find(|session| session.hover_active && session.svg != svg)
                .map(|session| session.svg.clone());
            if let Some(stale_svg) = stale_hover_svg {
                deactivate_hover(handle, &stale_svg);
            }
        }

        let existing_index = session_index_for_svg(&handle.borrow(), &svg);
        match existing_index {
            None => {
                let initial_active = snapshot_active_keys(&svg);
                let initial_visible_tooltip = snapshot_visible_tooltip(&layer);
                handle.borrow_mut().push(Session {
                    svg: svg.clone(),
                    layer: layer.clone(),
                    frame,
                    sticky: matches!(trigger, Trigger::Touch),
                    hover_active: matches!(trigger, Trigger::Pointer),
                    focus_active: matches!(trigger, Trigger::Keyboard),
                    hover_target: matches!(trigger, Trigger::Pointer).then(|| hit_area.clone()),
                    focus_target: matches!(trigger, Trigger::Keyboard).then(|| hit_area.clone()),
                    initial_active,
                    initial_visible_tooltip,
                });
            }
            Some(index) => {
                if let Some(session) = handle.borrow_mut().get_mut(index) {
                    match trigger {
                        Trigger::Touch => session.sticky = true,
                        Trigger::Pointer => {
                            session.hover_active = true;
                            session.hover_target = Some(hit_area.clone());
                        }
                        Trigger::Keyboard => {
                            session.focus_active = true;
                            session.focus_target = Some(hit_area.clone());
                        }
                    }
                }
            }
        }

        apply_highlight(&layer, &svg, hit_area, client_x, client_y);
    }

    /// hover/focus の一方が非活性化してもセッションが開いたまま残る場合
    /// （[`should_close_session`] が `false` を返した場合）に呼ぶ。残って
    /// いる入力（`hover_active` を優先、無ければ `focus_active`）が指す
    /// 対象（[`Session::hover_target`]/[`Session::focus_target`]）へ
    /// tooltip・強調表示・位置を再適用する（codex-review P1 / Bugbot
    /// 指摘: 非活性化した側の対象が data-active/tooltip に残留し、残る
    /// 入力側の対象へ表示が戻らない不具合の是正。位置は各対象の hit-area
    /// 自身の座標から求める `hit_area_client_anchor` を使う。ポインタの
    /// 最新クライアント座標は保持していないため、hover 側の再適用でも
    /// 同じ関数で hit-area 基準の位置に揃える）。
    fn reapply_active_target(handle: &SessionHandle, svg: &Element) {
        let target = {
            let sessions = handle.borrow();
            let Some(session) = sessions.iter().find(|session| session.svg == *svg) else {
                return;
            };
            if session.hover_active {
                session.hover_target.clone()
            } else if session.focus_active {
                session.focus_target.clone()
            } else {
                None
            }
        };
        let Some(target) = target else {
            return;
        };
        let Some(svg) = svg_of(&target) else {
            return;
        };
        let Some(layer) = layer_of(&svg) else {
            return;
        };
        let (client_x, client_y) = hit_area_client_anchor(&target);
        apply_highlight(&layer, &svg, &target, client_x, client_y);
    }

    /// hover が非活性化した（`pointermove`/`pointerout`/`pointercancel`
    /// のいずれかが hit-area/svg 外への移動を検知した）ときの共通処理:
    /// `hover_active` を `false` に落とし、focus も非活性なら
    /// [`should_close_session`] に従いセッションを閉じる（codex/Bugbot
    /// 指摘: pointer と focus は独立した「開いたままにする理由」であり、
    /// 片方の消失だけで閉じてはならない）。セッションが開いたまま残る
    /// 場合は、残っている focus 側の対象へ表示を戻す
    /// （[`reapply_active_target`]、codex-review P1 / Bugbot 指摘）。
    /// sticky（タッチ）セッションは対象外（no-op）: `hover_active_svg` が
    /// sticky セッションを除外して返さなくなった後も、呼び出し側の取り
    /// 違え等で sticky な `svg` が渡された場合に備えた二重の防御であり、
    /// タッチで開いたセッションを hover 経路が誤って非活性化・クローズ
    /// しないことを保証する（codex P1 / Cursor Bugbot "Sticky session
    /// closes after hover" 指摘、イシュー #2130 PR #2267）。
    fn deactivate_hover(handle: &SessionHandle, svg: &Element) {
        let should_close = {
            let mut sessions = handle.borrow_mut();
            let Some(session) = sessions.iter_mut().find(|session| session.svg == *svg) else {
                return;
            };
            if session.sticky {
                return;
            }
            session.hover_active = false;
            should_close_session(session.hover_active, session.focus_active)
        };
        if should_close {
            close_session(handle, svg);
        } else {
            reapply_active_target(handle, svg);
        }
    }

    /// `root` へ pointermove（hover 追従）を配線する。イベント対象が属する
    /// チャート（`svg`）が sticky（タッチ）セッション中のときのみ無視する。
    /// hit-area 以外（軸ラベル相当）への移動は hover を非活性化し、focus
    /// も非活性なセッションのみ閉じる（[`deactivate_hover`]）。
    fn handle_pointermove(root: &Element, handle: &SessionHandle, event: &Event) {
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        let Some(target) = event_target_element(event) else {
            return;
        };
        // sticky 判定はイベント対象が属するチャートに限定する。Runtime
        // 全体のいずれかのセッションが sticky というだけで無視すると、
        // タッチ対応 PC でチャート A をタップ後にマウスで別チャート B へ
        // 移動しても B の pointermove が無視されツールチップが開かなく
        // なる（codex P1 指摘、イシュー #2130 PR #2267）。対象が属する
        // svg が無い（chart 外）場合は sticky 判定なしで従来どおり進む。
        if let Some(target_svg) = svg_of(&target) {
            if root.contains(Some(&target_svg)) {
                let sticky = handle
                    .borrow()
                    .iter()
                    .any(|s| s.svg == target_svg && s.sticky);
                if sticky {
                    return;
                }
            }
        }
        let client_x = f64::from(pointer_event.client_x());
        let client_y = f64::from(pointer_event.client_y());
        match closest_hit_area(root, &target) {
            Some(hit_area) => {
                begin_or_update_session(
                    root,
                    handle,
                    &hit_area,
                    Trigger::Pointer,
                    client_x,
                    client_y,
                );
            }
            None => {
                if let Some(svg) = hover_active_svg(handle) {
                    deactivate_hover(handle, &svg);
                }
            }
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
        begin_or_update_session(root, handle, &hit_area, Trigger::Touch, client_x, client_y);
    }

    /// `root` へ pointerout を配線する。現在 `hover_active` なセッション
    /// （[`hover_active_svg`]、sticky セッションは対象外）の `svg` 内へ
    /// `related_target` が留まっていなければ hover を非活性化する
    /// （[`deactivate_hover`]、focus も非活性なセッションのみ閉じる）。
    /// `pointerleave` はバブリングしないため `pointerout` + `related_target`
    /// 判定（`sidebar::wiring` の `pointerover` 判定と同型）。
    fn handle_pointerout(handle: &SessionHandle, event: &Event) {
        let Some(session_svg) = hover_active_svg(handle) else {
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
            deactivate_hover(handle, &session_svg);
        }
    }

    /// `root` へ pointercancel を配線する。現在 `hover_active` なセッション
    /// （sticky セッションは対象外）を非活性化し、focus も非活性なら
    /// 閉じる（[`deactivate_hover`]）。
    fn handle_pointercancel(handle: &SessionHandle) {
        if let Some(svg) = hover_active_svg(handle) {
            deactivate_hover(handle, &svg);
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
        let Some(svg) = svg_of(&hit_area) else {
            return;
        };

        let modifiers = Modifiers {
            ctrl: keyboard_event.ctrl_key(),
            alt: keyboard_event.alt_key(),
            meta: keyboard_event.meta_key(),
        };
        let key = keyboard_event.key();
        if key == "Escape" {
            if !modifiers.any() {
                close_session(handle, &svg);
            }
            return;
        }

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
            set_dom_attribute(element, "tabindex", if i == next { "0" } else { "-1" });
        }
        if let Some(svg_element) = next_element.dyn_ref::<SvgElement>() {
            let _ = svg_element.focus();
        }
        let (client_x, client_y) = hit_area_client_anchor(next_element);
        begin_or_update_session(
            root,
            handle,
            next_element,
            Trigger::Keyboard,
            client_x,
            client_y,
        );
    }

    /// `root` へ focusin を配線する。target が hit-area のとき roving
    /// tabindex を更新し、focus を [`Trigger::Keyboard`] としてセッション
    /// を開始・継続する。
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
            set_dom_attribute(&element, "tabindex", if is_current { "0" } else { "-1" });
        }
        let (client_x, client_y) = hit_area_client_anchor(&hit_area);
        begin_or_update_session(
            root,
            handle,
            &hit_area,
            Trigger::Keyboard,
            client_x,
            client_y,
        );
    }

    /// `root` へ focusout を配線する。フォーカスを離れた hit-area 自身の
    /// チャート（`svg`）が sticky（タッチ）セッション中は無視する（兄弟の
    /// `handle_pointermove`/`handle_pointerout`/`handle_pointercancel` と
    /// 同型のガードだが、判定対象は当該チャートのセッションに限定する。
    /// タッチ由来で開始したセッションはチャート外タップまで開いたままに
    /// する契約を守るため、`handle_document_pointerdown` に閉鎖判定を
    /// 委ねる）。`related_target` が `root` 配下の hit-area でなければ
    /// focus を非活性化し、hover も非活性なセッションのみ閉じる（codex
    /// レビュー指摘: Tab フォーカス後に無関係なポインタ移動でツールチップ
    /// が消えていた不具合の是正、[`should_close_session`] 参照）。
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
        let Some(svg) = svg_of(&hit_area) else {
            return;
        };
        let sticky = handle
            .borrow()
            .iter()
            .find(|session| session.svg == svg)
            .map(|session| session.sticky)
            .unwrap_or(false);
        if sticky {
            return;
        }
        let Some(focus_event) = event.dyn_ref::<FocusEvent>() else {
            return;
        };
        let related = focus_event
            .related_target()
            .and_then(|target| target.dyn_into::<Element>().ok());
        // 移動先 hit-area の svg が移動元（当該チャート）と同一の場合のみ
        // 「内部移動」と判定する。`closest_hit_area(root, element)` だけでは
        // root 配下の他チャートの hit-area も「留まっている」と誤判定し、
        // 別チャートへ Tab 移動しても移動元の focus_active が解除されず
        // ツールチップ・強調表示が残り続ける不具合になる（codex P1 /
        // Cursor Bugbot "Focus session leaks across charts" 指摘、イシュー
        // #2130 PR #2267）。
        let still_within = related
            .as_ref()
            .and_then(|element| closest_hit_area(root, element))
            .and_then(|hit_area| svg_of(&hit_area))
            .is_some_and(|related_svg| related_svg == svg);
        if !still_within {
            let mut had_session = false;
            let should_close = {
                let mut sessions = handle.borrow_mut();
                if let Some(session) = sessions.iter_mut().find(|session| session.svg == svg) {
                    had_session = true;
                    session.focus_active = false;
                    should_close_session(session.hover_active, session.focus_active)
                } else {
                    false
                }
            };
            if should_close {
                close_session(handle, &svg);
            } else if had_session {
                // 残っている hover 側の対象へ表示を戻す（`deactivate_hover`
                // と対称、codex-review P1 / Bugbot 指摘）。
                reapply_active_target(handle, &svg);
            }
        }
    }

    /// document へ登録する pointerdown（sticky セッションのチャート外
    /// タップ閉鎖）。sticky セッションはチャートごとに独立して開き得る
    /// ため、`frame`（`svg`/`layer` の共通親）の外側への tap のとき、
    /// 該当するチャートのセッションのみを個別に閉じる（他チャートの
    /// sticky セッションには触れない）。
    fn handle_document_pointerdown(handle: &SessionHandle, event: &Event) {
        let sticky_entries: Vec<(Element, Element)> = handle
            .borrow()
            .iter()
            .filter(|session| session.sticky)
            .map(|session| (session.svg.clone(), session.frame.clone()))
            .collect();
        if sticky_entries.is_empty() {
            return;
        }
        let Some(target) = event_target_element(event) else {
            return;
        };
        for (svg, frame) in sticky_entries {
            if !frame.contains(Some(&target)) {
                close_session(handle, &svg);
            }
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
                set_dom_attribute(element, "pointer-events", "all");
            }
            let has_roving_tabindex = hit_areas
                .iter()
                .any(|element| element.get_attribute("tabindex").as_deref() == Some("0"));
            if !has_roving_tabindex {
                if let Some(first) = hit_areas.first() {
                    set_dom_attribute(first, "tabindex", "0");
                }
            }
        }
    }

    /// `svg` が `root` 配下から失われたセッション（[`crate::Runtime::
    /// rerender_subtree`] が古い `<svg>`/layer ごと差し替えた場合）を
    /// すべて破棄する（他チャートのセッションは残す）。復元先の要素が
    /// 既に DOM から切り離されているため [`close_session`] の属性復元
    /// （no-op）は経由せず直接ハンドルから取り除く（codex レビュー指摘:
    /// タッチで開いた sticky セッション中に構造再描画で SVG が差し替わる
    /// と、セッションが削除済み SVG を保持したまま以降の `pointermove`
    /// で復帰しなくなっていた不具合の是正）。
    fn discard_stale_session(root: &Element, handle: &SessionHandle) {
        handle
            .borrow_mut()
            .retain(|session| root.contains(Some(&session.svg)));
    }

    /// 再描画（[`crate::Runtime::rerender_subtree`] による `root` 配下の
    /// 丸ごと差し替え）で hit-area が SSR 値（`pointer-events="none"
    /// tabindex="-1"`）へ戻るため、`MutationObserver`（`childList`/
    /// `subtree` のみを監視し `attributes` は監視しない＝自己発火ループを
    /// 構造的に回避する、`sidebar::wiring` と同型）で [`enhance`] を
    /// 再適用する。あわせて進行中セッションの `svg` が再描画で失われて
    /// いないかを確認し、失われていれば破棄する
    /// （[`discard_stale_session`]）。
    fn wire_rerender_observer(root: &Element, handle: SessionHandle) -> Result<(), JsValue> {
        let observed_root = root.clone();
        let callback = Closure::<dyn FnMut(js_sys::Array, MutationObserver)>::new(
            move |_records: js_sys::Array, _observer: MutationObserver| {
                enhance(&observed_root);
                discard_stale_session(&observed_root, &handle);
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
    /// pointercancel/keydown/focusin/focusout の配線と、構造再描画時の
    /// 冪等な再エンハンス用 `MutationObserver`（[`wire_rerender_observer`]）
    /// を 1 回だけ登録する（マウント時 1 回契約、`crate::Runtime::wire_chart`
    /// から呼ばれる）。マウント時点で `root` 配下に hit-area が 1 つも
    /// 無くても登録を省略しない（codex レビュー指摘: 従来は早期リターン
    /// していたため、初期表示にチャートが無いアプリで後から
    /// `rerender_subtree` によりチャートが追加されても `MutationObserver`
    /// が存在せず配線されなかった不具合の是正）。登録するイベント
    /// リスナー自体は `closest_hit_area` 判定で no-op になるため、
    /// チャートを一切使わないアプリでも実害はない。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback`/`MutationObserver::new` の失敗を
    /// 伝播する。
    pub fn wire_chart_events(root: Element) -> Result<(), JsValue> {
        enhance(&root);

        let handle: SessionHandle = Rc::new(RefCell::new(Vec::new()));

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

        wire_rerender_observer(&root, handle.clone())?;

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

    #[test]
    fn should_close_session_only_when_both_reasons_inactive() {
        assert!(should_close_session(false, false));
    }

    #[test]
    fn should_close_session_keeps_open_while_hover_active() {
        assert!(!should_close_session(true, false));
    }

    #[test]
    fn should_close_session_keeps_open_while_focus_active() {
        assert!(!should_close_session(false, true));
    }

    #[test]
    fn should_close_session_keeps_open_while_both_active() {
        assert!(!should_close_session(true, true));
    }
}
