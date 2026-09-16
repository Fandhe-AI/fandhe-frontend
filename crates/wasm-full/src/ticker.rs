//! marquee の JS 駆動拡張（ticker）の DOM 配線層（イシュー #2540、親
//! #2528/#2530/#2476）。
//!
//! # 責務境界
//!
//! 実測に基づく複製数決定・rAF による offset 前進・hover/scroll 速度連動
//! の計算・DOM 適用は [`fandhe_frontend_animation::ticker::Ticker`] の責務
//! であり、本モジュールは以下のみを担う（3 層構成、`docs/design/
//! motion-reference-adoption-policy.md` §6、`crate::magnetic` と同型）:
//!
//! 1. opt-in 要素（[`TICKER_SELECTOR`]）の走査・各要素の `[data-part="content"]`
//!    最初の子の解決・属性からの [`fandhe_frontend_animation::ticker::TickerConfig`]
//!    組み立て
//! 2. `root` へのポインタイベント委譲登録（hover 一時停止、`gesture.rs`/
//!    `magnetic.rs` と同じ「登録回数を定数個に抑える」方針、A04 対策）
//! 3. `window` の `scroll`/`resize` 購読（scroll 速度連動・複製数再調整）
//! 4. `prefers-reduced-motion: reduce` 時は配線自体を行わない（`magnetic`/
//!    `confetti` と同じ「wire 時に検出し、reduced なら配線しない」方針）
//!
//! `Ticker` 自体は [`super::gesture`]/`magnetic` と異なり要素ごとに 1 個
//! 保持する必要がある（各 ticker が自前の rAF ループを持つため）。すべて
//! `Rc<RefCell<Vec<(Element, Ticker)>>>` へ集約し、`wire_ticker*` の
//! 呼び出しフレームを抜けた後も生存させる（`forget()` する Closure と同じ
//! 「呼び出し元へ返さない・プロセス終了まで保持」設計）。
//!
//! # 既知の制約（ponytail 割り切り、`fandhe_frontend_animation::ticker`
//! と同じ制約を継承）
//!
//! 動的に追加された ticker 要素は wire 時の `querySelectorAll` 対象外
//! （`svg_path`/`in_view` と同じ制約）。`[data-part="content"]` が存在
//! しない要素は静かにスキップする（fail-safe、既存 `marquee` 契約上
//! 通常発生しない）。
//!
//! # Runtime への統合
//!
//! `crate::lib::Runtime::mount`/`Runtime::hydrate` の双方が
//! `Self::wire_magnetic` の直後で `Self::wire_ticker` を呼ぶ（feature
//! `ticker`、既定 on）。`dispatch` チャネルを持たない属性専用配線のため
//! （`Self::wire_gesture`/`Self::wire_magnetic` と同型）、
//! `Component`/`binding_table`/`keyed_list_cache` は必要としない。
//!
//! # セキュリティ不変条件（REQ-1・security.md A03）
//!
//! [`TICKER_ATTR`] は存在属性のみ（値を持たない）。速度・係数属性は
//! `fandhe_frontend_animation::ticker::parse_speed`/`parse_factor` が
//! 非有限・範囲外を fail-safe にクランプしてから使う。DOM への書き込みは
//! `fandhe_frontend_animation::ticker::write_offset` が固定のプロパティ
//! 名へ `f64` 演算結果のみを書き込む（同モジュール doc 参照）。

/// opt-in（著者が SSR 出力に静的に付与）: JS 駆動（ticker）を有効化する
/// マーカー（値なし存在属性）。`fandhe_frontend_pre_styled_ui::marquee_motion::
/// TICKER_ATTR` と値が一致する必要がある。
pub const TICKER_ATTR: &str = "data-fandhe-ticker";
/// 基準速度（px/s）を著者が上書きする属性。
pub const TICKER_SPEED_ATTR: &str = "data-fandhe-ticker-speed";
/// hover 中の速度係数を著者が上書きする属性。
pub const TICKER_HOVER_FACTOR_ATTR: &str = "data-fandhe-ticker-hover-factor";
/// scroll 速度連動の係数を著者が上書きする属性。
pub const TICKER_SCROLL_FACTOR_ATTR: &str = "data-fandhe-ticker-scroll-factor";
/// スクロール軸（`horizontal`/`vertical`）を指定する属性。
pub const TICKER_AXIS_ATTR: &str = "data-axis";
/// JS 駆動開始時に root へ付与するマーカー（CSS 側が `animation: none` +
/// `transform` 駆動へ切り替える対象）。
pub const TICKER_ACTIVE_ATTR: &str = "data-fandhe-ticker-active";
/// opt-in 要素の走査セレクタ。
pub const TICKER_SELECTOR: &str = "[data-fandhe-ticker]";

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        TICKER_ACTIVE_ATTR, TICKER_AXIS_ATTR, TICKER_HOVER_FACTOR_ATTR, TICKER_SCROLL_FACTOR_ATTR,
        TICKER_SELECTOR, TICKER_SPEED_ATTR,
    };
    use crate::gesture::is_touch_pointer;
    use fandhe_frontend_animation::ticker::{
        is_in_secondary_copy, neutralize_nested_tickers, parse_factor, parse_speed, Axis, Ticker,
        TickerConfig,
    };
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, FocusEvent, MouseEvent, PointerEvent};

    /// `element.set_attribute(name, value)` の薄いガード付きラッパー
    /// （`crate::tabs_indicator::wiring::set_dom_attribute` と同じ方針・
    /// 同じ 4 種のガードを経由する。`name`/`value` は `TICKER_ACTIVE_ATTR`
    /// の固定リテラルだが、将来の変更に対する防御として同じガードを
    /// 経由する。`fw gate` の `url_validation_check`〔U1〕契約は
    /// `tabs_indicator.rs` の同関数 doc 参照）。
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

    /// wire 中に生成した全 [`Ticker`] を保持する（モジュール doc「responsible
    /// boundary」節参照。`Element` はホバー委譲がどの ticker を制御すべきか
    /// を解決するための鍵として持つ）。
    type ActiveTickers = Rc<RefCell<Vec<(Element, Ticker)>>>;

    /// `attrs` から `TickerConfig` を組み立てる。速度・係数は
    /// [`fandhe_frontend_animation::ticker`] の `parse_*` が fail-safe に
    /// クランプする。`direction_sign` は既存 `marquee` の `--fandhe-marquee-
    /// direction` custom property を直接読まず、`data-direction`（
    /// `fandhe_frontend_pre_styled_ui::marquee::MarqueeDirection` の
    /// `VariantValue::value()` が生成する recipe クラスの axis 値と同義）
    /// の有無ではなく、より単純に root の `class` を介さない
    /// `getComputedStyle` 経由の値読み取りを避けるため、`content` 要素の
    /// `getComputedStyle().animationDirection`（`normal`/`reverse`）から
    /// 符号を導出する。
    fn read_direction_sign(content: &Element) -> f64 {
        let Some(window) = web_sys::window() else {
            return 1.0;
        };
        let Ok(Some(style)) = window.get_computed_style(content) else {
            return 1.0;
        };
        let Ok(direction) = style.get_property_value("animation-direction") else {
            return 1.0;
        };
        if direction.trim() == "reverse" {
            -1.0
        } else {
            1.0
        }
    }

    fn read_attr(element: &Element, name: &str) -> Option<String> {
        element.get_attribute(name)
    }

    fn build_config(element: &Element) -> TickerConfig {
        let axis = match read_attr(element, TICKER_AXIS_ATTR).as_deref() {
            Some("vertical") => Axis::Vertical,
            _ => Axis::Horizontal,
        };
        let speed_px_s = read_attr(element, TICKER_SPEED_ATTR)
            .map(|raw| parse_speed(&raw))
            .unwrap_or(fandhe_frontend_animation::ticker::DEFAULT_SPEED_PX_S);
        let hover_factor = read_attr(element, TICKER_HOVER_FACTOR_ATTR)
            .map(|raw| parse_factor(&raw, 0.0, 1.0))
            .unwrap_or(0.0);
        let scroll_factor = read_attr(element, TICKER_SCROLL_FACTOR_ATTR)
            .map(|raw| parse_factor(&raw, 0.0, f64::MAX))
            .unwrap_or(0.0);
        TickerConfig {
            speed_px_s,
            direction_sign: 1.0,
            axis,
            hover_factor,
            scroll_factor,
        }
    }

    /// `root`（[`super::TICKER_SELECTOR`] 一致要素）配下の最初の
    /// `[data-part="content"]` を返す。
    fn first_content(root: &Element) -> Option<Element> {
        root.query_selector("[data-part=\"content\"]")
            .ok()
            .flatten()
    }

    /// `root` 配下・[`super::TICKER_SELECTOR`] に一致する祖先（自身含む）を
    /// **すべて**返す（入れ子 ticker、イシュー #2540）。
    ///
    /// 旧実装は `target.closest(TICKER_SELECTOR)` で最も近い 1 件だけを
    /// 返していたため、入れ子の ticker（ticker 内に別の ticker を合成した
    /// 構成）でリンクへフォーカス/ホバーすると内側の ticker だけが停止し、
    /// 外側は動き続けていた（既存 CSS の `:focus-within`/`:hover` は
    /// 祖先すべてに独立して一致するため両方止まるのに対し、JS 駆動は
    /// `animation: none` でその CSS 規則自体が効かなくなる。PR #2582
    /// codex-review P1 指摘）。`closest` を「直前に一致した要素の親」から
    /// 繰り返し呼ぶことで、`target` から `scan_root` までの祖先チェーン上の
    /// 一致要素を内側から外側の順にすべて集める。
    fn resolve_ticker_targets(scan_root: &Element, target: &Element) -> Vec<Element> {
        let mut targets = Vec::new();
        let mut cursor = target.closest(TICKER_SELECTOR).ok().flatten();
        while let Some(matched) = cursor {
            if !scan_root.contains(Some(&matched)) {
                break;
            }
            cursor = matched
                .parent_element()
                .and_then(|parent| parent.closest(TICKER_SELECTOR).ok().flatten());
            targets.push(matched);
        }
        targets
    }

    fn event_target_element(event: &Event) -> Option<Element> {
        event.target()?.dyn_into::<Element>().ok()
    }

    /// `relatedTarget`（移動先要素）が `boundary`（含む）配下に留まって
    /// いるか。`pointerout`/`focusout` で「特定の ticker から本当に出た
    /// のか、その ticker 内部の別要素へ移っただけか」を判定する
    /// （`gesture.rs::related_within` と同型、PR #2582 codex-review P1
    /// 指摘: 入れ子 ticker では祖先ごとに判定が異なり得るため、
    /// `resolve_ticker_targets` が返す各要素へ個別に適用する）。
    fn related_target_element(event: &Event) -> Option<Element> {
        if let Some(mouse) = event.dyn_ref::<MouseEvent>() {
            return mouse.related_target()?.dyn_into::<Element>().ok();
        }
        let focus = event.dyn_ref::<FocusEvent>()?;
        focus.related_target()?.dyn_into::<Element>().ok()
    }

    /// `pointerover`/`pointerout` の委譲: タッチ由来のポインタは除外する
    /// （`magnetic.rs`/`gesture.rs` と同じ理由）。
    fn handle_pointer_hover(root: &Element, event: &Event, active: &ActiveTickers, hovered: bool) {
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        if is_touch_pointer(&pointer_event.pointer_type()) {
            return;
        }
        let Some(target) = event_target_element(event) else {
            return;
        };
        let ticker_targets = resolve_ticker_targets(root, &target);
        if ticker_targets.is_empty() {
            return;
        }
        for (element, ticker) in active.borrow().iter() {
            if !ticker_targets.iter().any(|t| t == element) {
                continue;
            }
            // pointerover は入れ子祖先すべてで「ポインタが内側にある」ため
            // 常に反映する。pointerout は「移動先がこの ticker の境界内へ
            // 留まっているか」を確認し、留まっていれば（入れ子内部の別
            // 要素への移動）当該 ticker は停止させない。
            if hovered || !related_target_within(event, element) {
                ticker.set_hovered(hovered);
            }
        }
    }

    /// `related`（`relatedTarget`）が `boundary`（含む）配下に留まって
    /// いるか（`gesture.rs::related_within` と同型）。
    fn related_target_within(event: &Event, boundary: &Element) -> bool {
        related_target_element(event).is_some_and(|related| boundary.contains(Some(&related)))
    }

    /// `focusin`/`focusout` の委譲（`bubbles: true`、キャプチャ不要）:
    /// JS 駆動時（`[data-fandhe-ticker-active]`）は CSS 側の
    /// `root:focus-within` 一時停止規則が `animation: none` 化で効かなく
    /// なるため、キーボードフォーカスも [`Ticker::set_focused`] 経由で
    /// 同じ一時停止契約（WCAG 2.2.2）を満たす（codex-review・Cursor
    /// Bugbot 指摘、イシュー #2540）。入れ子 ticker では祖先すべてへ
    /// 適用する（[`resolve_ticker_targets`] 参照）。
    fn handle_focus_visibility(
        root: &Element,
        event: &Event,
        active: &ActiveTickers,
        focused: bool,
    ) {
        if event.dyn_ref::<FocusEvent>().is_none() {
            return;
        }
        let Some(target) = event_target_element(event) else {
            return;
        };
        let ticker_targets = resolve_ticker_targets(root, &target);
        if ticker_targets.is_empty() {
            return;
        }
        for (element, ticker) in active.borrow().iter() {
            if !ticker_targets.iter().any(|t| t == element) {
                continue;
            }
            if focused || !related_target_within(event, element) {
                ticker.set_focused(focused);
            }
        }
    }

    /// `root` へ opt-in 要素を走査し、[`Ticker`] を起動してポインタ/scroll/
    /// resize イベントを配線する。`reduced_motion` が `true` なら何も
    /// 登録しない（モジュール doc「`prefers-reduced-motion: reduce` 時は
    /// 配線自体を行わない」節）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback*` の失敗を伝播する。
    pub fn wire_ticker_with_reduced_motion(
        root: Element,
        reduced_motion: bool,
    ) -> Result<(), JsValue> {
        if reduced_motion {
            return Ok(());
        }

        // `querySelectorAll` は呼び出し元の要素自身を含まないため、配線
        // ルート自身が opt-in 要素（`[data-fandhe-ticker]`）の場合も対象に
        // 含める（`headless_select::instance_boundary` と同じ `matches`
        // 先行判定。Cursor Bugbot 指摘: ルート自身が ticker だと配線されず
        // 起動しなかった）。
        let mut ticker_roots: Vec<Element> = Vec::new();
        if root.matches(TICKER_SELECTOR).unwrap_or(false) {
            ticker_roots.push(root.clone());
        }
        let nodes = root.query_selector_all(TICKER_SELECTOR)?;
        for i in 0..nodes.length() {
            if let Some(element) = nodes
                .item(i)
                .and_then(|node| node.dyn_into::<Element>().ok())
            {
                ticker_roots.push(element);
            }
        }
        let active: ActiveTickers = Rc::new(RefCell::new(Vec::new()));
        for ticker_root in ticker_roots {
            // SSR（`pre-styled-ui::marquee_motion::ticker`）は同じ children の
            // content を 2 コピー出力するため、入れ子 ticker は最初から
            // 2 回現れる。2 コピー目以降の内側は起動せず、`ensure_copies`
            // の追加複製と同じ静的化を適用して「駆動される内側は元の 1 個
            // だけ」に揃える（Cursor Bugbot 指摘: SSR コピー内の内側が
            // 独立 rAF で駆動され位相がずれていた）。
            if is_in_secondary_copy(&ticker_root) {
                neutralize_nested_tickers(&ticker_root);
                continue;
            }
            let Some(content) = first_content(&ticker_root) else {
                continue;
            };
            let mut config = build_config(&ticker_root);
            config.direction_sign = read_direction_sign(&content);
            set_dom_attribute(&ticker_root, TICKER_ACTIVE_ATTR, "");
            let ticker = Ticker::start(ticker_root.clone(), content, config);
            active.borrow_mut().push((ticker_root, ticker));
        }

        if active.borrow().is_empty() {
            return Ok(());
        }

        let pointerover_root = root.clone();
        let pointerover_active = Rc::clone(&active);
        let pointerover_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointer_hover(&pointerover_root, &event, &pointerover_active, true);
        });
        root.add_event_listener_with_callback_and_bool(
            "pointerover",
            pointerover_closure.as_ref().unchecked_ref(),
            true,
        )?;
        pointerover_closure.forget();

        let pointerout_root = root.clone();
        let pointerout_active = Rc::clone(&active);
        let pointerout_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointer_hover(&pointerout_root, &event, &pointerout_active, false);
        });
        root.add_event_listener_with_callback_and_bool(
            "pointerout",
            pointerout_closure.as_ref().unchecked_ref(),
            true,
        )?;
        pointerout_closure.forget();

        // `focusin`/`focusout` は既定でバブルするため（`focus`/`blur` と
        // 異なる）キャプチャ登録は不要（`add_event_listener_with_callback`）。
        let focusin_root = root.clone();
        let focusin_active = Rc::clone(&active);
        let focusin_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_focus_visibility(&focusin_root, &event, &focusin_active, true);
        });
        root.add_event_listener_with_callback("focusin", focusin_closure.as_ref().unchecked_ref())?;
        focusin_closure.forget();

        let focusout_root = root.clone();
        let focusout_active = Rc::clone(&active);
        let focusout_closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_focus_visibility(&focusout_root, &event, &focusout_active, false);
        });
        root.add_event_listener_with_callback(
            "focusout",
            focusout_closure.as_ref().unchecked_ref(),
        )?;
        focusout_closure.forget();

        if let Some(window) = web_sys::window() {
            let scroll_active = Rc::clone(&active);
            let last_scroll_y: Rc<RefCell<Option<f64>>> = Rc::new(RefCell::new(None));
            let last_scroll_ms: Rc<RefCell<Option<f64>>> = Rc::new(RefCell::new(None));
            let scroll_window = window.clone();
            let scroll_closure = Closure::<dyn FnMut()>::new(move || {
                let Some(y) = scroll_window.scroll_y().ok() else {
                    return;
                };
                let now_ms = scroll_window.performance().map(|p| p.now()).unwrap_or(0.0);
                let prev_y = last_scroll_y.borrow_mut().replace(y);
                let prev_ms = last_scroll_ms.borrow_mut().replace(now_ms);
                if let (Some(prev_y), Some(prev_ms)) = (prev_y, prev_ms) {
                    let dy = y - prev_y;
                    let dt = now_ms - prev_ms;
                    for (_, ticker) in scroll_active.borrow().iter() {
                        ticker.push_scroll_delta(dy, dt);
                    }
                }
            });
            window.add_event_listener_with_callback(
                "scroll",
                scroll_closure.as_ref().unchecked_ref(),
            )?;
            scroll_closure.forget();

            let resize_active = Rc::clone(&active);
            let resize_closure = Closure::<dyn FnMut()>::new(move || {
                for (_, ticker) in resize_active.borrow().iter() {
                    ticker.mark_resize();
                }
            });
            window.add_event_listener_with_callback(
                "resize",
                resize_closure.as_ref().unchecked_ref(),
            )?;
            resize_closure.forget();

            // `prefers-reduced-motion` の実行中切り替え（OS 設定変更）を
            // 監視する: `wire_ticker` は起動時点の判定のみで配線要否を
            // 決めるため（モジュール doc「4. `prefers-reduced-motion:
            // reduce` 時は配線自体を行わない」節）、起動後に reduce へ
            // 切り替わっても `AnimationLoop` は動き続け `transform` 駆動を
            // 止めない不具合があった（PR #2582 codex-review P1 指摘）。
            // `change` イベントで reduce 確定時に全 ticker を停止し
            // `TICKER_ACTIVE_ATTR` を外すことで、CSS 側の既定
            // `@media (prefers-reduced-motion: reduce)` 縮退
            // （`marquee.rs`/`marquee_motion.rs`）へ委ねる。reduce → 非
            // reduce への復帰は再配線（リロード）が必要（`magnetic`/
            // `confetti` と同じ「wire 時 1 回判定」設計を踏襲、対称に扱う
            // 必要はない: 動き始める方向の復帰はアクセシビリティ契約を
            // 破らない）。
            if let Ok(Some(mql)) = window.match_media("(prefers-reduced-motion: reduce)") {
                let mql_active = Rc::clone(&active);
                // `MediaQueryListEvent`（`event.matches()`）は web-sys feature
                // 未有効化のため、`change` イベント自体からではなく `mql`
                // （`MediaQueryList`）を closure へ直接 clone して都度
                // `matches()` を再照会する（同じ結果を feature 追加なしで
                // 得られる）。
                let change_mql = mql.clone();
                let change_closure = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
                    if !change_mql.matches() {
                        return;
                    }
                    for (element, ticker) in mql_active.borrow().iter() {
                        ticker.stop();
                        let _ = element.remove_attribute(TICKER_ACTIVE_ATTR);
                    }
                });
                let _ = mql.add_event_listener_with_callback(
                    "change",
                    change_closure.as_ref().unchecked_ref(),
                );
                change_closure.forget();
            }
        }

        Ok(())
    }

    /// [`wire_ticker_with_reduced_motion`] を実際の `prefers-reduced-motion`
    /// 検出結果で呼ぶ（`Runtime::mount`/`hydrate` から呼ばれる本番経路）。
    ///
    /// # Errors
    ///
    /// [`wire_ticker_with_reduced_motion`] の失敗を伝播する。
    pub fn wire_ticker(root: Element) -> Result<(), JsValue> {
        let reduced_motion = fandhe_frontend_animation::magnetic::detect_reduced_motion();
        wire_ticker_with_reduced_motion(root, reduced_motion)
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{wire_ticker, wire_ticker_with_reduced_motion};

#[cfg(test)]
mod tests {
    use super::{
        TICKER_ACTIVE_ATTR, TICKER_ATTR, TICKER_AXIS_ATTR, TICKER_HOVER_FACTOR_ATTR,
        TICKER_SCROLL_FACTOR_ATTR, TICKER_SELECTOR, TICKER_SPEED_ATTR,
    };

    #[test]
    fn constants_are_stable_strings() {
        assert_eq!(TICKER_ATTR, "data-fandhe-ticker");
        assert_eq!(TICKER_SPEED_ATTR, "data-fandhe-ticker-speed");
        assert_eq!(TICKER_HOVER_FACTOR_ATTR, "data-fandhe-ticker-hover-factor");
        assert_eq!(
            TICKER_SCROLL_FACTOR_ATTR,
            "data-fandhe-ticker-scroll-factor"
        );
        assert_eq!(TICKER_AXIS_ATTR, "data-axis");
        assert_eq!(TICKER_ACTIVE_ATTR, "data-fandhe-ticker-active");
        assert_eq!(TICKER_SELECTOR, "[data-fandhe-ticker]");
    }
}
