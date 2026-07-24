//! Timer（`fandhe-frontend-headless-ui` `timer` モジュール）の `setInterval`
//! 実 tick 駆動配線（イシュー #836、親トラッキング #520）。
//!
//! `crates/headless-ui/src/timer.rs` は Root/Area/Item/ItemValue/ItemLabel/
//! Separator/Control/ActionTrigger の 8 anatomy パーツと、tick（経過ミリ秒）
//! を外部から明示的に注入する決定的状態機械 [`fandhe_frontend_headless_ui::timer::Timer`]
//! を提供する一方、実時間計測（`setInterval`/`Date.now()`）によるクライアント
//! 側の実 tick 駆動は同モジュール冒頭 rustdoc「スコープ外」節が明記する
//! とおり本クレート（wasm 層）の後続スコープとされていた。本モジュールが
//! その配線を実装する。
//!
//! # `fandhe_frontend_headless_ui::timer::Timer` を直接利用する（文字列複製
//! しない）理由
//!
//! `crates/wasm-full/Cargo.toml` は `fandhe-frontend-headless-ui` を通常の
//! `[dependencies]`（製品依存）として持つ（イシュー #590 で `position`
//! モジュールが追加した経緯）。そのため [`headless_clipboard`]（イシュー
//! #773）が「クレートの製品依存にないため文字列で複製する」と判断した
//! 制約は本モジュールには当てはまらず、`Timer::from_hydration_attrs`/
//! `Timer::update`（`fandhe_frontend_interactive::dispatch` 経由）を直接
//! 呼んで完了判定・セグメント分解のロジックを一切複製しない。ドリフトの
//! 心配自体が構造的に生じない設計である。
//!
//! # DOM 属性を Timer の一時的な永続化先として扱う
//!
//! 本モジュールは `root` の `data-state`/`data-elapsed`/`data-countdown`/
//! `data-start-ms`/`data-target-ms`/`data-interval` 属性
//! （`crates/headless-ui/src/timer.rs::root` が出力する契約）を
//! `Timer::from_hydration_attrs` が読む `data-hydrate-*` 形式へその場で
//! 変換し、`Timer` を都度再構築する（[`read_timer`]）。tick/click 処理後は
//! `Timer::phase`/`Timer::elapsed_ms` を同じ属性へ書き戻す（[`write_timer`]）。
//! アプリのルート状態機械 `C`（`crate::lib::Runtime<C>`）が `Timer` 自身か
//! どうかに関わらず本モジュールが DOM 上の表示更新を完結できる設計であり、
//! [`crate::headless_avatar`]/[`crate::headless_clipboard`] より一段疎結合
//! である（`C` への dispatch 転送は「`C` が Timer アクションを認識する場合の
//! 追随」という副次的なベストエフォートに留める、下記「`Runtime` への統合」
//! 節参照）。
//!
//! # `Runtime` への統合
//!
//! [`wire_timer_events`] は `crate::lib::Runtime::mount`/`Runtime::hydrate`
//! の双方から `headless_clipboard::wire_clipboard_events` の直後に組み込まれる
//! （`crate::lib::Runtime::wire_timer` 参照）。`events`/`keynav`/
//! `headless_avatar`/`headless_clipboard` と同じ「マウント時 1 回」契約を
//! 維持する。
//!
//! # `data-action` の allowlist 変換
//!
//! ActionTrigger の `data-action` 属性値（`"start"`/`"pause"`/`"resume"`/
//! `"reset"` の 4 値完全一致、`crates/headless-ui/src/timer.rs::TimerControl`
//! 参照）を `"timer:*"` アクション名へ変換する（[`action_from_trigger`]）。
//! 未知の値・欠落は `None`（fail-closed）。
//!
//! # `data-interval` の下限クランプ
//!
//! 改ざんされた `data-interval="0"` 等の極端に小さい値による dispatch
//! ストーム（CPU 枯渇、`.claude/rules/security.md` A04 対応）を防ぐため、
//! [`Timer::from_hydration_attrs`] が返す `interval_ms` を
//! `setInterval` へ渡す直前に [`MIN_INTERVAL_MS`]（16ms、`requestAnimationFrame`
//! 相当の下限）でクランプする（[`clamp_interval_ms`]）。上限側も
//! [`MAX_INTERVAL_MS`]（`i32::MAX`）でクランプする。`interval_ms: u64` は
//! `data-interval`（DOM 属性、クライアント側で改ざん可能）由来のため、
//! `web_sys::Window::set_interval_with_callback_and_timeout_and_arguments_0`
//! が要求する `i32` へキャストする直前に上限を設けないと、`2^31` 以上
//! `2^32` 未満の値が `u64 as i32` で負値へラップし、ブラウザ側で
//! `setInterval(cb, <負数>)` が `0` へ再クランプされて下限クランプの意図
//! （dispatch ストーム防止）が無効化される（イシュー #836 レビュー指摘）。
//!
//! # 実時間の計測は本モジュール（wasm 境界）に隔離する
//!
//! `Timer`（`fandhe-frontend-headless-ui`）自身は `std::time`/`Instant`/
//! `js_sys::Date` のいずれにも依存しない決定的状態機械である
//! （`crates/headless-ui/src/timer.rs` 冒頭 doc「時計 API 非依存」節参照）。
//! 本モジュールは `js_sys::Date::now()` で前回 tick からの実測 delta を計算し
//! `TimerAction::Tick(delta)` として注入する、時計アクセスの**唯一の**箇所である。
//! `setInterval` 自体のドリフト（コールバック起動の遅延）を実測 delta で
//! 吸収するため、状態機械へドリフトが蓄積しない。
//!
//! # セキュリティ不変条件
//!
//! - DOM 反映は `set_attribute`/`remove_attribute`/`set_text_content` のみで
//!   行い、HTML 文字列を一切組み立てない（REQ-1）。属性名はすべて
//!   `&'static str` リテラル。
//! - `data-action` は allowlist（完全一致）でのみ受理する。
//! - `data-interval`/`data-elapsed`/`data-start-ms`/`data-target-ms` の
//!   パース失敗・改ざんは `Timer::from_hydration_attrs` の `Result` で
//!   fail-closed に扱われ（`Err` の場合は当該イベントを no-op とし、
//!   panic しない）、`crate::headless_avatar`/`crate::headless_clipboard` と
//!   同じ fail-closed 方針を維持する。
//! - 新規 `unsafe` コードは追加しない（`web-sys`/`js-sys` の safe API のみ
//!   使用）。

use fandhe_frontend_headless_ui::timer::{format_segment, Timer, TimerUnit};
use fandhe_frontend_interactive::Hydrate;

/// Timer の `data-scope` 属性値（`fandhe_frontend_headless_ui::timer` の
/// `ANATOMY` と一致）。
const TIMER_SCOPE: &str = "timer";
/// Timer ActionTrigger パーツの `data-part` 属性値。
const ACTION_TRIGGER_PART: &str = "action-trigger";
/// Timer ItemValue パーツの `data-part` 属性値（wasm32 配線層専用の定数だが、
/// native の非テストビルドでは未使用と誤検出される。
/// `headless_avatar.rs::AVATAR_FALLBACK_PART` と同じ理由の dead_code 抑制）。
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const ITEM_VALUE_PART: &str = "item-value";

/// `setInterval` へ渡す tick 間隔の下限（ミリ秒）。`requestAnimationFrame`
/// 相当（モジュール冒頭「`data-interval` の下限クランプ」節参照）。
pub const MIN_INTERVAL_MS: u64 = 16;

/// `setInterval` へ渡す tick 間隔の上限（ミリ秒）。
/// `Window::set_interval_with_callback_and_timeout_and_arguments_0` の
/// タイムアウト引数は `i32` のため、`i32::MAX` を超える値は `u64 as i32`
/// キャストで符号が反転し負値へラップする（モジュール冒頭「`data-interval`
/// の下限クランプ」節参照）。`clamp_interval_ms` はこの上限もあわせて
/// 適用し、キャスト前に必ず `i32` の範囲へ収める。
pub const MAX_INTERVAL_MS: u64 = i32::MAX as u64;

/// クリックターゲットが Timer ActionTrigger 要素かどうかを判定する純粋関数
/// （DOM 非依存、native `cargo test` で検証可能）。
#[must_use]
pub fn is_timer_action_trigger(scope: Option<&str>, part: Option<&str>) -> bool {
    scope == Some(TIMER_SCOPE) && part == Some(ACTION_TRIGGER_PART)
}

/// ActionTrigger の `data-action` 属性値を `"timer:*"` アクション名へ変換する
/// allowlist 変換（完全一致のみ、モジュール冒頭「`data-action` の allowlist
/// 変換」節参照）。未知の値・欠落は `None`（fail-closed）。
#[must_use]
pub fn action_from_trigger(data_action: Option<&str>) -> Option<&'static str> {
    match data_action {
        Some("start") => Some("timer:start"),
        Some("pause") => Some("timer:pause"),
        Some("resume") => Some("timer:resume"),
        Some("reset") => Some("timer:reset"),
        _ => None,
    }
}

/// `interval_ms` を [`MIN_INTERVAL_MS`] 未満・[`MAX_INTERVAL_MS`] 超過の
/// いずれにもならないようクランプする純粋関数（モジュール冒頭
/// 「`data-interval` の下限クランプ」節参照）。上限クランプにより、
/// `i32 as i32` キャスト時の符号反転（負値ラップ）で下限クランプの意図が
/// 無効化される経路を防ぐ。
#[must_use]
pub fn clamp_interval_ms(interval_ms: u64) -> u64 {
    interval_ms.clamp(MIN_INTERVAL_MS, MAX_INTERVAL_MS)
}

/// `root` の `data-*` 属性群から [`Timer`] を再構築する純粋関数。
///
/// `crates/headless-ui/src/timer.rs::root` が出力する表示属性
/// （`data-state`/`data-elapsed`/`data-countdown`/`data-start-ms`/
/// `data-target-ms`/`data-interval`）を `Timer::from_hydration_attrs` が
/// 読む `data-hydrate-*` 形式へ変換してから委譲する
/// （モジュール冒頭「DOM 属性を Timer の一時的な永続化先として扱う」節
/// 参照）。改ざん・欠落による復元失敗は `None`（fail-closed）。
#[must_use]
pub fn timer_from_display_attrs(
    data_state: Option<&str>,
    data_elapsed: Option<&str>,
    has_data_countdown: bool,
    data_start_ms: Option<&str>,
    data_target_ms: Option<&str>,
    data_interval: Option<&str>,
) -> Option<Timer> {
    let attrs = vec![
        (
            "data-hydrate-phase".to_string(),
            data_state.unwrap_or("idle").to_string(),
        ),
        (
            "data-hydrate-elapsed".to_string(),
            data_elapsed.unwrap_or("0").to_string(),
        ),
        (
            "data-hydrate-countdown".to_string(),
            has_data_countdown.to_string(),
        ),
        (
            "data-hydrate-start-ms".to_string(),
            data_start_ms.unwrap_or("0").to_string(),
        ),
        (
            "data-hydrate-target-ms".to_string(),
            data_target_ms.unwrap_or("0").to_string(),
        ),
        (
            "data-hydrate-interval-ms".to_string(),
            data_interval.unwrap_or("1000").to_string(),
        ),
    ];
    Timer::from_hydration_attrs(&attrs).ok()
}

/// [`Timer::display_segments`] から 4 セグメント分の (単位, ゼロ埋め済み
/// 文字列) を返す純粋関数（[`TimerUnit`] の 4 値を固定順で返す）。
#[must_use]
pub fn formatted_segments(timer: &Timer) -> [(TimerUnit, String); 4] {
    let (days, hours, minutes, seconds) = timer.display_segments();
    [
        (TimerUnit::Days, format_segment(days)),
        (TimerUnit::Hours, format_segment(hours)),
        (TimerUnit::Minutes, format_segment(minutes)),
        (TimerUnit::Seconds, format_segment(seconds)),
    ]
}

// ---------------------------------------------------------------------
// 配線層: web-sys/js-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、
// native の `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`events.rs`/`headless_avatar.rs`/`headless_clipboard.rs` と同じ 2 層
// 構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        action_from_trigger, clamp_interval_ms, formatted_segments, is_timer_action_trigger,
        timer_from_display_attrs, Timer, ACTION_TRIGGER_PART, ITEM_VALUE_PART, TIMER_SCOPE,
    };
    use crate::events::ActionRef;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, Window};

    /// `target` から `root`（含む）まで祖先方向へ辿り、`data-scope`/
    /// `data-part` が指定値と一致する最初の要素を返す
    /// （`crate::headless_clipboard::wiring::closest_matching` と同型）。
    fn closest_matching(
        root: &Element,
        start: &Element,
        scope: &str,
        part: &str,
    ) -> Option<Element> {
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

    /// `element.set_attribute(name, value)` の薄いガード付きラッパー
    /// （`crate::headless_clipboard::wiring::set_dom_attribute` と同型、
    /// イシュー #401 の `fw gate` `url_validation_check` 契約に準拠）。
    /// 本モジュールが書き込む属性（`data-state`/`data-elapsed`）はいずれも
    /// `&'static str`/数値整形済み文字列で固定された非 URL・非イベント
    /// ハンドラ・非 `srcset` 属性であり実害はないが、
    /// `fandhe_frontend_core::url` のガード関数群を経由することで、将来
    /// `name`/`value` が動的な入力から組み立てられるよう変更された場合の
    /// 防御としても機能する（`headless_clipboard.rs` と同じ判断）。
    fn set_dom_attribute(element: &Element, name: &str, value: &str) -> Result<(), JsValue> {
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

    /// `root` の `data-*` 表示属性を読み取り [`Timer`] を再構築する
    /// （[`super::timer_from_display_attrs`] への薄い DOM 読み取り委譲）。
    fn read_timer(root: &Element) -> Option<Timer> {
        timer_from_display_attrs(
            root.get_attribute("data-state").as_deref(),
            root.get_attribute("data-elapsed").as_deref(),
            root.has_attribute("data-countdown"),
            root.get_attribute("data-start-ms").as_deref(),
            root.get_attribute("data-target-ms").as_deref(),
            root.get_attribute("data-interval").as_deref(),
        )
    }

    /// `timer` の現在状態を `root` の `data-state`/`data-elapsed` 属性、および
    /// 4 セグメント分の item-value テキストへ反映する。
    ///
    /// # Errors
    ///
    /// `query_selector_all`/`set_attribute` の失敗を伝播する。
    fn write_timer(root: &Element, timer: &Timer) -> Result<(), JsValue> {
        set_dom_attribute(root, "data-state", timer.phase().as_str())?;
        set_dom_attribute(root, "data-elapsed", &timer.elapsed_ms().to_string())?;
        for (unit, formatted) in formatted_segments(timer) {
            let selector = format!(
                r#"[data-scope="{TIMER_SCOPE}"][data-part="{ITEM_VALUE_PART}"][data-type="{}"]"#,
                unit.as_str()
            );
            let Ok(node_list) = root.query_selector_all(&selector) else {
                continue;
            };
            let len = node_list.length();
            for i in 0..len {
                let Some(node) = node_list.get(i) else {
                    continue;
                };
                node.set_text_content(Some(&formatted));
            }
        }
        Ok(())
    }

    /// 保留中の `setInterval` ハンドル（`handle` + 保持し続ける必要がある
    /// `Closure`。`crate::headless_clipboard::wiring::PendingTimer` と同型）。
    struct PendingInterval {
        handle: i32,
        _closure: Closure<dyn FnMut()>,
    }

    /// `on_action` へ 1 アクションを通知する（`try_borrow_mut` 失敗＝再入は
    /// no-op、panic 回避。`crate::headless_clipboard::wiring::dispatch_action`
    /// と同型）。
    fn notify_action(
        action: &str,
        payload: &str,
        on_action: &Rc<RefCell<impl FnMut(ActionRef) + 'static>>,
    ) {
        if let Ok(mut cb) = on_action.try_borrow_mut() {
            (cb)(ActionRef {
                action: action.to_string(),
                payload: payload.to_string(),
            });
        }
    }

    /// `root` の現在の `data-state` を見て `setInterval` の予約/解除を同期する。
    ///
    /// - `running`: 既存の保留中インターバルがなければ、`data-interval`
    ///   （[`clamp_interval_ms`] で下限クランプ済み）で新規予約する。
    /// - それ以外: 保留中インターバルがあれば `clear_interval` する。
    ///
    /// 完了到達時にインターバル自身のコールバック終端から呼ばれても
    /// （JS の `clearInterval` を自身の実行中コールバックから呼ぶのと同じ
    /// 安全な操作）問題なく機能する。
    fn sync_interval(
        root: &Element,
        window: &Window,
        on_action: &Rc<RefCell<impl FnMut(ActionRef) + 'static>>,
        pending: &Rc<RefCell<Option<PendingInterval>>>,
        last_tick_ms: &Rc<RefCell<Option<f64>>>,
    ) {
        let running = root.get_attribute("data-state").as_deref() == Some("running");
        if !running {
            if let Some(timer) = pending.borrow_mut().take() {
                window.clear_interval_with_handle(timer.handle);
            }
            *last_tick_ms.borrow_mut() = None;
            return;
        }
        if pending.borrow().is_some() {
            return;
        }
        let Some(timer) = read_timer(root) else {
            return;
        };
        let interval_ms = clamp_interval_ms(timer.interval_ms());
        *last_tick_ms.borrow_mut() = Some(js_sys::Date::now());

        let tick_root = root.clone();
        let tick_window = window.clone();
        let tick_on_action = on_action.clone();
        let tick_pending = pending.clone();
        let tick_last = last_tick_ms.clone();
        let closure = Closure::<dyn FnMut()>::new(move || {
            handle_tick(
                &tick_root,
                &tick_window,
                &tick_on_action,
                &tick_pending,
                &tick_last,
            );
        });
        let Ok(handle) = window.set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            interval_ms as i32,
        ) else {
            return;
        };
        *pending.borrow_mut() = Some(PendingInterval {
            handle,
            _closure: closure,
        });
    }

    /// `setInterval` 1 回分の発火を処理する。実測 delta（`Date.now()`）を
    /// `"timer:tick"` として注入し、DOM を反映後、`sync_interval` で次回
    /// 予約/解除を再判定する（モジュール冒頭「実時間の計測は本モジュール
    /// （wasm 境界）に隔離する」節参照）。
    fn handle_tick(
        root: &Element,
        window: &Window,
        on_action: &Rc<RefCell<impl FnMut(ActionRef) + 'static>>,
        pending: &Rc<RefCell<Option<PendingInterval>>>,
        last_tick_ms: &Rc<RefCell<Option<f64>>>,
    ) {
        let Some(mut timer) = read_timer(root) else {
            if let Some(t) = pending.borrow_mut().take() {
                window.clear_interval_with_handle(t.handle);
            }
            return;
        };
        let now = js_sys::Date::now();
        let delta = last_tick_ms
            .borrow_mut()
            .replace(now)
            .map(|prev| (now - prev).max(0.0) as u64)
            .unwrap_or(0);

        let payload = delta.to_string();
        if fandhe_frontend_interactive::dispatch(&mut timer, "timer:tick", &payload) {
            let _ = write_timer(root, &timer);
        }
        notify_action("timer:tick", &payload, on_action);
        sync_interval(root, window, on_action, pending, last_tick_ms);
    }

    /// click イベント 1 件を処理する。Timer ActionTrigger 上のクリックのみ
    /// 反応し（fail-closed、無関係要素上のクリックは無視）、`data-action`
    /// を allowlist 変換した `"timer:*"` アクションを [`Timer`] へ適用して
    /// DOM を反映後、`sync_interval` で tick 予約/解除を再判定する。
    fn handle_click(
        root: &Element,
        event: &Event,
        window: &Window,
        on_action: &Rc<RefCell<impl FnMut(ActionRef) + 'static>>,
        pending: &Rc<RefCell<Option<PendingInterval>>>,
        last_tick_ms: &Rc<RefCell<Option<f64>>>,
    ) {
        let Some(target) = event.target() else {
            return;
        };
        let target_element: Element = match target.dyn_ref::<Element>() {
            Some(element) => element.clone(),
            None => {
                let Some(node) = target.dyn_ref::<web_sys::Node>() else {
                    return;
                };
                let Some(parent) = node.parent_element() else {
                    return;
                };
                parent
            }
        };

        let scope = target_element.get_attribute("data-scope");
        let part = target_element.get_attribute("data-part");
        let trigger = if is_timer_action_trigger(scope.as_deref(), part.as_deref()) {
            Some(target_element.clone())
        } else {
            closest_matching(root, &target_element, TIMER_SCOPE, ACTION_TRIGGER_PART)
        };
        let Some(trigger) = trigger else {
            return;
        };

        let Some(action) = action_from_trigger(trigger.get_attribute("data-action").as_deref())
        else {
            return;
        };

        let Some(mut timer) = read_timer(root) else {
            return;
        };
        if fandhe_frontend_interactive::dispatch(&mut timer, action, "") {
            let _ = write_timer(root, &timer);
        }
        notify_action(action, "", on_action);
        sync_interval(root, window, on_action, pending, last_tick_ms);
    }

    /// `root` 配下の Timer ActionTrigger クリックへ配線を 1 回だけ登録し、
    /// 既に `running` 状態（例: ハイドレーション直後）であれば直ちに tick
    /// 予約を行う。
    ///
    /// click はバブリングするため、`root` への委譲リスナー 1 個で完結する
    /// （`crate::headless_clipboard::wiring::wire_clipboard_events` と同型）。
    /// `window` が取得できない環境（テストランナー等）では配線をスキップし
    /// `Ok(())` を返す（fail-closed）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback` の失敗を伝播する。
    pub fn wire_timer_events(
        root: Element,
        on_action: impl FnMut(ActionRef) + 'static,
    ) -> Result<(), JsValue> {
        let Some(window) = web_sys::window() else {
            return Ok(());
        };

        let on_action = Rc::new(RefCell::new(on_action));
        let pending: Rc<RefCell<Option<PendingInterval>>> = Rc::new(RefCell::new(None));
        let last_tick_ms: Rc<RefCell<Option<f64>>> = Rc::new(RefCell::new(None));

        // ハイドレーション直後に既に running な Timer が存在する場合、
        // 即座に tick 予約を行う（モジュール冒頭「`Runtime` への統合」節）。
        sync_interval(&root, &window, &on_action, &pending, &last_tick_ms);

        let click_root = root.clone();
        let click_window = window.clone();
        let click_on_action = on_action.clone();
        let click_pending = pending.clone();
        let click_last_tick = last_tick_ms.clone();
        let closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_click(
                &click_root,
                &event,
                &click_window,
                &click_on_action,
                &click_pending,
                &click_last_tick,
            );
        });
        root.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_timer_events;

#[cfg(test)]
mod tests {
    use super::*;

    // --- is_timer_action_trigger ---

    #[test]
    fn action_trigger_matches_only_exact_scope_and_part() {
        assert!(is_timer_action_trigger(
            Some("timer"),
            Some("action-trigger")
        ));
        assert!(!is_timer_action_trigger(Some("timer"), Some("root")));
        assert!(!is_timer_action_trigger(
            Some("attacker"),
            Some("action-trigger")
        ));
        assert!(!is_timer_action_trigger(None, None));
    }

    // --- action_from_trigger ---

    #[test]
    fn action_from_trigger_covers_all_four_controls() {
        assert_eq!(action_from_trigger(Some("start")), Some("timer:start"));
        assert_eq!(action_from_trigger(Some("pause")), Some("timer:pause"));
        assert_eq!(action_from_trigger(Some("resume")), Some("timer:resume"));
        assert_eq!(action_from_trigger(Some("reset")), Some("timer:reset"));
    }

    #[test]
    fn action_from_trigger_rejects_unknown_and_missing() {
        assert_eq!(action_from_trigger(Some("bogus")), None);
        assert_eq!(action_from_trigger(None), None);
        assert_eq!(action_from_trigger(Some("Start")), None); // 大文字小文字の完全一致のみ
    }

    // --- clamp_interval_ms ---

    #[test]
    fn clamp_interval_ms_enforces_minimum() {
        assert_eq!(clamp_interval_ms(0), MIN_INTERVAL_MS);
        assert_eq!(clamp_interval_ms(1), MIN_INTERVAL_MS);
        assert_eq!(clamp_interval_ms(15), MIN_INTERVAL_MS);
        assert_eq!(clamp_interval_ms(16), 16);
        assert_eq!(clamp_interval_ms(1000), 1000);
    }

    #[test]
    fn clamp_interval_ms_enforces_maximum() {
        // 上限クランプがないと `u64 as i32` キャストで負値へラップし、
        // ブラウザ側で `setInterval(cb, 0)` へ再クランプされて下限クランプの
        // 意図（dispatch ストーム防止）が無効化される（イシュー #836
        // レビュー指摘）。
        assert_eq!(clamp_interval_ms(MAX_INTERVAL_MS), MAX_INTERVAL_MS);
        assert_eq!(clamp_interval_ms(MAX_INTERVAL_MS + 1), MAX_INTERVAL_MS);
        // 2^31（i32::MAX 超過・u32 範囲内）: クランプなしでは
        // `u64 as i32` で負値へラップする代表的な改ざん値。
        assert_eq!(clamp_interval_ms(1u64 << 31), MAX_INTERVAL_MS);
        // 2^32（レビュー指摘の具体例）: クランプなしでは下位 32bit のみ
        // 保持され `0` へラップする。
        assert_eq!(clamp_interval_ms(1u64 << 32), MAX_INTERVAL_MS);
        assert_eq!(clamp_interval_ms(u64::MAX), MAX_INTERVAL_MS);

        // クランプ後の値は必ず `i32` へ安全にキャストできる（負値へ
        // ラップしないことの直接検証）。
        assert!(i32::try_from(clamp_interval_ms(u64::MAX)).is_ok());
    }

    // --- timer_from_display_attrs / formatted_segments ---

    #[test]
    fn timer_from_display_attrs_round_trips_basic_countdown() {
        let timer = timer_from_display_attrs(
            Some("running"),
            Some("1200"),
            true,
            Some("5000"),
            Some("0"),
            Some("250"),
        )
        .expect("valid attrs should reconstruct a Timer");
        assert_eq!(timer.phase().as_str(), "running");
        assert_eq!(timer.elapsed_ms(), 1200);
        assert!(timer.is_countdown());
        assert_eq!(timer.interval_ms(), 250);
    }

    #[test]
    fn timer_from_display_attrs_defaults_missing_optional_attrs() {
        let timer = timer_from_display_attrs(Some("idle"), None, false, None, None, None)
            .expect("missing optional attrs should fall back to defaults");
        assert_eq!(timer.elapsed_ms(), 0);
        assert!(!timer.is_countdown());
        assert_eq!(timer.interval_ms(), 1000);
    }

    #[test]
    fn timer_from_display_attrs_rejects_invalid_phase() {
        assert!(timer_from_display_attrs(Some("flying"), None, false, None, None, None).is_none());
    }

    #[test]
    fn timer_from_display_attrs_rejects_non_numeric_elapsed() {
        assert!(timer_from_display_attrs(
            Some("idle"),
            Some("not-a-number"),
            false,
            None,
            None,
            None
        )
        .is_none());
    }

    #[test]
    fn formatted_segments_zero_pads_all_four_units() {
        let timer = timer_from_display_attrs(
            Some("running"),
            Some("0"),
            true,
            Some("93784000"),
            Some("0"),
            Some("1000"),
        )
        .unwrap();
        let segments = formatted_segments(&timer);
        assert_eq!(segments[0].1, "01"); // days
        assert_eq!(segments[1].1, "02"); // hours
        assert_eq!(segments[2].1, "03"); // minutes
        assert_eq!(segments[3].1, "04"); // seconds
    }

    // --- ドリフト検知: headless-ui の dispatch と本モジュールの allowlist 変換
    // 経由 dispatch が同一結果になること ---

    #[test]
    fn allowlisted_action_dispatch_matches_direct_headless_ui_dispatch() {
        use fandhe_frontend_interactive::dispatch;

        let mut a = Timer::count_up(0, 1000);
        let mut b = Timer::count_up(0, 1000);

        let action = action_from_trigger(Some("start")).unwrap();
        assert!(dispatch(&mut a, action, ""));
        assert!(dispatch(&mut b, "timer:start", ""));
        assert_eq!(a, b);
    }
}
