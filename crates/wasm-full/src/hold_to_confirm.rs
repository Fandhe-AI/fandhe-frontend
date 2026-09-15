//! hold-to-confirm（長押し確定）ボタンの DOM 配線層（イシュー #2538、親
//! button の Motion+ 由来 variant）。
//!
//! # 責務境界
//!
//! 毎フレームの経過時間計測（`Driver::tick`）・DOM への書き込み
//! （`Target::write`）自体のロジックは `fandhe-frontend-animation`
//! （[`fandhe_frontend_animation::raf_driver::RafDriver`]・
//! [`fandhe_frontend_animation::raf_driver::AnimationLoop`]・
//! [`fandhe_frontend_animation::dom_target::DomTarget`]、#2403/#2517）の
//! 既存実装をそのまま消費する（`docs/design/motion-reference-adoption-policy.md`
//! §4「C 群: フレームループ必須」）。本モジュールは以下のみを担う（3 層
//! 構成、`scroll_driver.rs`/`confetti.rs` と同型）:
//!
//! 1. `[data-fandhe-hold-to-confirm]` 要素の収集（opt-in マーカー、
//!    候補はマウント時 1 回のみ走査。動的挿入要素への追随は `in_view.rs`/
//!    `scroll_driver.rs` 同様スコープ外の既知の制約）
//! 2. pointerdown/keydown（Enter/Space）で保持セッション開始、
//!    pointerup/pointercancel/pointerleave/pointermove（暗黙 capture 下の
//!    ヒットテスト補完、`handle_pointermove` doc 参照）/keyup で早期
//!    離脱時の中断、進行度が `1.0` に達したら確定
//! 3. 確定時: `data-state="confirmed"` を書き込み、合成（untrusted）
//!    `click()` を発火し、一定時間後に `data-state` を戻す
//!    （`headless_clipboard.rs` の `PendingTimer` と同型のタイマー）
//! 4. root への capture フェーズ `click` リスナーで、信頼できる（＝実際の
//!    マウス/キーボード活性化由来の）click を常に捕捉・抑止する
//!    （短押しでは絶対に確定しない不変条件、下記「確定の不変条件」節）
//!
//! # 確定の不変条件（A03 相当・意図しない誤操作防止）
//!
//! ネイティブ `<button>` は pointerdown → pointerup の一連の操作や
//! keydown（Enter）だけで即座に `click` イベントを発火する。本モジュールが
//! 意図するのは「[`DEFAULT_HOLD_DURATION_MS`] 以上押し続けたときのみ確定」
//! であり、途中で離した短いクリックが `events.rs` の `data-action` 委譲へ
//! 素通りして意図しない確定として扱われてはならない。このため
//! `wiring::wire_hold_to_confirm`（本ファイル内部実装）は root で
//! **capture フェーズ**の
//! `click` リスナーを登録し、`[data-fandhe-hold-to-confirm]` に一致する
//! （`closest()` で判定）要素上の `click` のうち `Event::is_trusted()` が
//! 真のもの（＝実際のポインタ/キーボード活性化由来）を
//! `stop_immediate_propagation()` で常に止める。進行度 100% 到達時に本
//! モジュール自身が発火する合成 `click()`（[`web_sys::HtmlElement::click`]）
//! は仕様上 `isTrusted: false` を返すため、このガードの対象外として
//! bubble し続け `events.rs` の `data-action` 委譲へ届く。`prevent_default`
//! は呼ばない（`gesture.rs` の既存方針を踏襲。フォーム送信抑止は下記
//! `type="button"` 前提に委ねる）。
//!
//! # `type="button"` の前提
//!
//! 上記ガードは `click` イベント自体の伝播のみを止め、`<button
//! type="submit">` の**既定アクション**（フォーム送信）は妨げない。
//! `crates/pre-styled-ui/src/button_motion.rs::hold_to_confirm_button` は
//! ルート要素へ常に `type="button"` を強制することでこの経路を構造的に
//! 塞ぐ（本モジュール側では `type` 属性を検証・上書きしない——SSR
//! マークアップの責務であり、DOM 配線層が黙って属性を書き換えるのは
//! 驚き最小の原則に反するため）。
//!
//! # `data-*` 属性の命名
//!
//! `data-fandhe-*` 内部プレフィックス（`gesture.rs`/`scroll_driver.rs` と
//! 同型）。進行度は `--fandhe-motion-hold-progress`（CSS カスタム
//! プロパティ、`fandhe_frontend_pre_styled_ui::button_motion` 側の
//! `calc()` 消費契約は同モジュール
//! doc 参照）。確定状態は各部品共通の `data-state`（値 `"confirmed"`）を
//! 用いる（`headless_clipboard.rs` 等の既存部品状態機械と語彙を揃える）。
//!
//! # Runtime への統合
//!
//! `crate::lib::Runtime::mount`/`Runtime::hydrate` の双方が
//! `Self::wire_confetti` の直後で `Self::wire_hold_to_confirm` を呼ぶ
//! （feature `hold-to-confirm`、既定 on）。`dispatch` チャネルを持たない
//! 属性専用配線のため（`Self::wire_sidebar`/`Self::wire_gesture` と同型）、
//! `Component`/`binding_table`/`keyed_list_cache` は必要としない。
//!
//! # セキュリティ不変条件（REQ-1・security.md A03）
//!
//! [`fandhe_frontend_animation::dom_target::DomTarget`] へ書き込む値は
//! [`hold_progress`] が返す `0.0..=1.0` の `f64`（クランプ済み）のみ。
//! [`HOLD_DURATION_MS_ATTR`] は著者が SSR 時に静的に書く属性値であり、
//! [`parse_hold_duration_ms`] がパース失敗・非正値を
//! [`DEFAULT_HOLD_DURATION_MS`] へ fallback する
//! （`stagger_index.rs::stagger_index_value` と同型の防御的パース）。
//! `data-state`/`data-fandhe-*` はいずれも固定リテラルのみを書き込み、
//! 利用者制御の文字列を属性値・セレクタへ混ぜない。

/// opt-in（著者が SSR 出力に静的に付与）: hold-to-confirm を有効化する
/// ルート要素マーカー。
pub const HOLD_TO_CONFIRM_ATTR: &str = "data-fandhe-hold-to-confirm";
/// 候補走査セレクタ。
pub const HOLD_TO_CONFIRM_SELECTOR: &str = "[data-fandhe-hold-to-confirm]";
/// opt-in（任意）: 保持時間（ミリ秒）を著者が上書きする属性。
pub const HOLD_DURATION_MS_ATTR: &str = "data-fandhe-hold-duration-ms";
/// 進行度（`0.0..=1.0`）を書き込む CSS カスタムプロパティ名。
pub const HOLD_PROGRESS_VAR: &str = "--fandhe-motion-hold-progress";
/// 確定状態を表す `data-state` の値。
pub const HOLD_CONFIRMED_STATE: &str = "confirmed";
/// 既定の保持時間（ミリ秒）。`docs/design/motion-reference-adoption-policy.md`
/// 参照実装の目安値であり、実測調整の余地がある既知の暫定値。
pub const DEFAULT_HOLD_DURATION_MS: f64 = 900.0;
/// 確定表示（`data-state="confirmed"`）を保持してから idle へ戻すまでの
/// 既定タイムアウト（ミリ秒）。`headless_clipboard.rs::DEFAULT_RESET_TIMEOUT_MS`
/// と同型の役割。
pub const DEFAULT_CONFIRMED_RESET_TIMEOUT_MS: i32 = 1500;

/// `elapsed_s`（経過秒）と `duration_s`（保持目標秒）から進行度
/// （`0.0..=1.0` にクランプ済み）を計算する（純粋関数、native `cargo test`
/// 可能）。`duration_s` が `0.0` 以下の場合は即座に `1.0`（確定扱い、
/// 呼び出し側で異常値が既にフィルタされている前提の防御）。
#[must_use]
pub fn hold_progress(elapsed_s: f64, duration_s: f64) -> f64 {
    if duration_s <= 0.0 {
        return 1.0;
    }
    (elapsed_s / duration_s).clamp(0.0, 1.0)
}

/// [`HOLD_DURATION_MS_ATTR`] の属性値文字列から保持時間（ミリ秒）を
/// 決定する。未指定・パース失敗・非正値はいずれも
/// [`DEFAULT_HOLD_DURATION_MS`] へフォールバックする（防御的パース、
/// `stagger_index.rs::stagger_index_value` と同型の方針）。
#[must_use]
pub fn parse_hold_duration_ms(attr_value: Option<&str>) -> f64 {
    attr_value
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|ms| *ms > 0.0)
        .unwrap_or(DEFAULT_HOLD_DURATION_MS)
}

#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{
        hold_progress, parse_hold_duration_ms, DEFAULT_CONFIRMED_RESET_TIMEOUT_MS,
        HOLD_CONFIRMED_STATE, HOLD_DURATION_MS_ATTR, HOLD_PROGRESS_VAR, HOLD_TO_CONFIRM_ATTR,
        HOLD_TO_CONFIRM_SELECTOR,
    };
    use crate::dom::set_dom_attribute_result as set_dom_attribute;
    use crate::gesture::is_press_activation_key;
    use fandhe_frontend_animation::dom_target::DomTarget;
    use fandhe_frontend_animation::fandhe_animation::driver::Driver;
    use fandhe_frontend_animation::fandhe_animation::target::Target;
    use fandhe_frontend_animation::raf_driver::{AnimationLoop, RafDriver};
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, HtmlElement, KeyboardEvent, PointerEvent, Window};

    /// 確定後の `data-state` 自動リセットタイマー（`headless_clipboard.rs`
    /// `PendingTimer` と同型のパターン）。
    struct PendingResetTimer {
        handle: i32,
        _closure: Closure<dyn FnMut()>,
    }

    /// 進行度 100% 到達時の確定処理: `data-state="confirmed"` 書き込み・
    /// 合成 `click()` 発火・一定時間後の自動リセット予約。
    ///
    /// `step` クロージャ（現在実行中の rAF コールバック）自身の呼び出し
    /// フレーム内から呼ばれるため、`AnimationLoop` 自体をここで drop
    /// **してはならない**（`raf_driver.rs::AnimationLoop` doc が警告する
    /// 「実行中の `Closure` を `call_mut` 実行中に drop すると
    /// use-after-free」の構造そのもの——`AnimationLoop::drop` は
    /// `stop()` を呼び、`stop()` は現在実行中の rAF コールバック自身を
    /// 保持する `closure_slot` を `take()` して即座に破棄する）。
    /// `confetti.rs::schedule_finished_cleanup` はこれを 0ms
    /// `set_timeout` で次のマクロタスクへ延期して回避するが、本関数は
    /// もっと単純に**never drop here**を選ぶ: `step` が `false` を返す
    /// ことで [`AnimationLoop`] は次フレームの再予約を止めるだけで自身は
    /// 生存し続け、`HoldSession::loop_handle` に保持されたまま次回
    /// [`HoldSession::start`]（pointerdown/keydown ハンドラの呼び出し
    /// フレーム——rAF コールバックの外側）が `Option` を新しい値で
    /// 上書きしたときに初めて安全に drop される。セッションは要素ごとに
    /// 高々 1 個の `AnimationLoop` のみを保持するため無制限な蓄積は
    /// 起きない。
    fn finish_confirmation(
        element: &HtmlElement,
        reset_timer: &Rc<RefCell<Option<PendingResetTimer>>>,
    ) {
        let _ = set_dom_attribute(element, "data-state", HOLD_CONFIRMED_STATE);

        // 合成 click: `HtmlElement::click()` は仕様上 `isTrusted: false`
        // の `click` イベントを発火する（モジュール冒頭「確定の不変条件」
        // 節参照）。`root` の capture リスナーはこのイベントを素通りさせ、
        // `events.rs` の `data-action` 委譲へ届く。
        element.click();

        schedule_confirmed_reset(element.clone(), reset_timer);
    }

    /// [`DEFAULT_CONFIRMED_RESET_TIMEOUT_MS`] 経過後に `data-state` を
    /// 除去し、[`HOLD_PROGRESS_VAR`] を `0.0` へ書き戻す
    /// （`headless_clipboard.rs::schedule_reset` と同型）。既存の保留中
    /// タイマーがあれば先に `clear_timeout` してから置き換える。
    ///
    /// 進行度も戻す理由: 確定処理（[`finish_confirmation`]）は進行度を
    /// `1.0` のまま残し、早期離脱時のみ [`HoldSession::cancel`] が
    /// `0.0` へ戻す設計のため、確定 → 早期離脱を経由しない通常の確定
    /// 経路では本関数が唯一の巻き戻し地点になる（本関数が戻さないと
    /// `data-state` は消えても塗りつぶしが 100% のまま次回保持まで
    /// 残留してしまう）。
    fn schedule_confirmed_reset(
        element: HtmlElement,
        reset_timer: &Rc<RefCell<Option<PendingResetTimer>>>,
    ) {
        let Some(window) = web_sys::window() else {
            return;
        };
        if let Some(timer) = reset_timer.borrow_mut().take() {
            window.clear_timeout_with_handle(timer.handle);
        }
        let closure = Closure::<dyn FnMut()>::new(move || {
            let _ = element.remove_attribute("data-state");
            let mut target = DomTarget::custom_property(element.clone(), HOLD_PROGRESS_VAR);
            target.write(0.0);
        });
        let Ok(handle) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            DEFAULT_CONFIRMED_RESET_TIMEOUT_MS,
        ) else {
            return;
        };
        *reset_timer.borrow_mut() = Some(PendingResetTimer {
            handle,
            _closure: closure,
        });
    }

    /// 1 要素分の保持セッション状態。`pointerdown`/`keydown`/
    /// `pointerup`/`keyup`/`pointerleave`/`pointercancel`/`pointermove`
    /// の各リスナーが共有する。
    struct HoldSession {
        element: HtmlElement,
        duration_ms: f64,
        loop_handle: Rc<RefCell<Option<AnimationLoop>>>,
        reset_timer: Rc<RefCell<Option<PendingResetTimer>>>,
        /// 保持ループが進行中かどうか（pointerdown と keydown の重複
        /// 開始防止・早期離脱ガードの双方に使う）。
        active: Rc<Cell<bool>>,
        /// 現在保持中のポインタ ID（`pointerdown` で設定、
        /// `cancel`/確定で `None` へ戻す）。keydown 起点の保持では
        /// `None` のまま（[`handle_pointermove`] のヒットテスト対象外
        /// になる）。
        pointer_id: Rc<Cell<Option<i32>>>,
        /// 現在保持中の起点キー値（`KeyboardEvent::key()`、`keydown` で
        /// 設定、`cancel`/確定で `None` へ戻す）。pointerdown 起点の
        /// 保持では `None` のまま。保持を開始したキーとは無関係の
        /// `keyup`（例: 起点が Enter のまま Space の keyup が届く）で
        /// 誤って中断しないための対応付け（codex-review P2 指摘）。
        active_key: Rc<RefCell<Option<String>>>,
    }

    impl HoldSession {
        fn new(element: HtmlElement) -> Self {
            let duration_ms =
                parse_hold_duration_ms(element.get_attribute(HOLD_DURATION_MS_ATTR).as_deref());
            Self {
                element,
                duration_ms,
                loop_handle: Rc::new(RefCell::new(None)),
                reset_timer: Rc::new(RefCell::new(None)),
                active: Rc::new(Cell::new(false)),
                pointer_id: Rc::new(Cell::new(None)),
                active_key: Rc::new(RefCell::new(None)),
            }
        }

        /// 進行度を即座に書き戻す（開始直前・中断直後の初期化）。
        fn write_progress(&self, value: f64) {
            let mut target = DomTarget::custom_property(self.element.clone(), HOLD_PROGRESS_VAR);
            target.write(value);
        }

        /// 保持ループを開始する。既に進行中（`active`）なら no-op
        /// （pointerdown と keydown の重複開始防止）。`RafDriver::new()`
        /// が `None`（非ブラウザ環境）の場合も no-op（fail-safe）。
        fn start(&self) {
            if self.active.get() {
                return;
            }
            let Some(mut driver) = RafDriver::new() else {
                return;
            };
            self.active.set(true);
            self.write_progress(0.0);
            // 直前の確定から `DEFAULT_CONFIRMED_RESET_TIMEOUT_MS` 経過前に
            // 再度保持が始まった場合、保留中の「confirmed 解除 + 進行度
            // 0 リセット」タイマーがまだ生きていると、新しい保持の途中で
            // 突然進行度が 0 へ巻き戻る（`schedule_confirmed_reset` rustdoc
            // 参照）。ここで明示的に解除してから新しいループを始める。
            //
            // `data-state="confirmed"` も同時に即座へ除去する必要がある
            // （codex-review P1 指摘・Cursor Bugbot 同根指摘）: この
            // タイマーだけが `data-state` を消す唯一の予約であり、単に
            // `clear_timeout_with_handle` で止めるとタイマー自体が発火し
            // なくなるため `data-state="confirmed"` が誰にも消されず
            // 無期限に残留する（この後 `cancel()` で早期離脱しても
            // `cancel()` は進行度のみを戻し `data-state` には触れない）。
            if let Some(window) = web_sys::window() {
                if let Some(timer) = self.reset_timer.borrow_mut().take() {
                    window.clear_timeout_with_handle(timer.handle);
                    let _ = self.element.remove_attribute("data-state");
                }
            }

            let duration_s = self.duration_ms / 1000.0;
            let elapsed_s = Rc::new(Cell::new(0.0_f64));
            let element = self.element.clone();
            let active = Rc::clone(&self.active);
            let reset_timer = Rc::clone(&self.reset_timer);
            let loop_handle_for_start = Rc::clone(&self.loop_handle);

            let animation_loop = AnimationLoop::start(move || {
                let delta = driver.tick().unwrap_or(0.0);
                let elapsed = elapsed_s.get() + delta;
                elapsed_s.set(elapsed);
                let progress = hold_progress(elapsed, duration_s);
                let mut target = DomTarget::custom_property(element.clone(), HOLD_PROGRESS_VAR);
                target.write(progress);
                if progress >= 1.0 {
                    active.set(false);
                    // `AnimationLoop` 自身はここでは drop しない
                    // （`finish_confirmation` rustdoc「never drop here」
                    // 節参照）。`false` を返して次フレーム予約を止める
                    // だけで、生存中の `Self` は `HoldSession::loop_handle`
                    // に保持され続け、次回 `start()` が上書きするまで
                    // 安全に留まる。
                    finish_confirmation(&element, &reset_timer);
                    return false;
                }
                true
            });
            *loop_handle_for_start.borrow_mut() = Some(animation_loop);
        }

        /// 早期離脱（pointerup/pointercancel/pointerleave/keyup/
        /// pointermove ヒットテスト）時にループを止め、進行度を `0.0`
        /// へ戻す。既に確定済み（`active` が `false`）の場合は no-op。
        fn cancel(&self) {
            self.pointer_id.set(None);
            self.active_key.borrow_mut().take();
            if !self.active.get() {
                return;
            }
            self.active.set(false);
            self.loop_handle.borrow_mut().take();
            self.write_progress(0.0);
        }
    }

    /// `root` 配下の `[data-fandhe-hold-to-confirm]` 要素（複数可）を
    /// 出現順に集める（`scroll_driver.rs::collect_scroll_progress_candidates`
    /// と同型、`query_selector_all` の失敗は空 `Vec` として扱う）。
    fn collect_candidates(root: &Element) -> Vec<HtmlElement> {
        let Ok(node_list) = root.query_selector_all(HOLD_TO_CONFIRM_SELECTOR) else {
            return Vec::new();
        };
        let len = node_list.length();
        let mut out = Vec::with_capacity(len as usize);
        for i in 0..len {
            if let Some(node) = node_list.get(i) {
                if let Ok(el) = node.dyn_into::<HtmlElement>() {
                    out.push(el);
                }
            }
        }
        out
    }

    /// `pointermove`: タッチ/ペンの暗黙 pointer capture 下（要素へ
    /// `pointerdown` した瞬間に W3C Pointer Events §implicit pointer
    /// capture でブラウザが自動的に capture を設定する）では、指/ペン先を
    /// 要素の外へ物理的に動かしても `pointerleave`/`pointerout` は一切
    /// 発火しない（capture 中は全イベントが capture 先要素へ配送され
    /// 続けるため）。このため [`wire_candidate`] の `pointerleave`
    /// リスナーだけでは、要素外へ離脱してもタッチ/ペンでの長押しが中断
    /// されず進行度が 100% に達して誤確定してしまう（codex-review P1
    /// 指摘）。`gesture.rs::handle_pointermove` と同型の対策として、
    /// `pointermove` を購読し `Document::element_from_point()` による
    /// ヒットテストで実際にポインタ直下にある要素が保持対象自身か
    /// どうかを判定する（matrix 変換・`overflow: visible` の子孫要素にも
    /// 頑健、`gesture.rs::handle_pointermove` rustdoc 参照）。ヒットが
    /// `None`（ビューポート外）または保持対象の子孫でなければ離脱とみな
    /// し [`HoldSession::cancel`] を呼ぶ。追跡中の `pointer_id`
    /// （[`HoldSession::pointer_id`]、`pointerdown` で設定）と一致しない
    /// `pointermove` は無視する（マウスホバー等、無関係なポインタの
    /// 移動で誤って中断しないため）。keydown 起点の保持は `pointer_id`
    /// が `None` のままのため本関数の対象外（[`HoldSession::pointer_id`]
    /// doc 参照）。
    fn handle_pointermove(event: &Event, session: &Rc<HoldSession>) {
        if !session.active.get() {
            return;
        }
        let Some(tracked_id) = session.pointer_id.get() else {
            return;
        };
        let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
            return;
        };
        if pointer_event.pointer_id() != tracked_id {
            return;
        }
        let Some(document) = session.element.owner_document() else {
            return;
        };
        let x = pointer_event.client_x() as f32;
        let y = pointer_event.client_y() as f32;
        let hit = document.element_from_point(x, y);
        let still_within = hit.is_some_and(|hit| {
            let element: &Element = &session.element;
            element.contains(Some(&hit))
        });
        if !still_within {
            session.cancel();
        }
    }

    /// 1 要素へ pointerdown/pointerup/pointercancel/pointerleave/
    /// pointermove/keydown/keyup の 7 リスナーを登録する。
    fn wire_candidate(element: &HtmlElement) -> Result<Rc<HoldSession>, JsValue> {
        let session = Rc::new(HoldSession::new(element.clone()));

        let pointerdown_session = Rc::clone(&session);
        let pointerdown = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            if let Ok(pointer_event) = event.dyn_into::<PointerEvent>() {
                if pointer_event.button() == 0 {
                    // 保持中（`active`）の追加 pointerdown は無視する
                    // （codex-review P1 指摘）: 別の指/ポインタが同じ
                    // 要素へ重ねて pointerdown すると、`start()` 自身は
                    // 既に `active` のため no-op で経過時間計測を継続する
                    // 一方、ここで無条件に `pointer_id` を新しいポインタ
                    // へ上書きすると、以後 `handle_pointermove` が
                    // 最初の指の移動を無視し、最初の指が要素外へ出ても
                    // 中断されなくなる（新しい指の保持時間が
                    // `duration_ms` 未満でも、最初の指から継続していた
                    // 経過時間で確定してしまい「一定時間押し続けたときの
                    // み確定」契約に反する）。追跡対象は最初に開始した
                    // ポインタのまま固定し、この pointerdown 自体は
                    // 無視する。
                    if pointerdown_session.active.get() {
                        return;
                    }
                    pointerdown_session
                        .pointer_id
                        .set(Some(pointer_event.pointer_id()));
                    pointerdown_session.start();
                }
            }
        });
        element.add_event_listener_with_callback(
            "pointerdown",
            pointerdown.as_ref().unchecked_ref(),
        )?;
        pointerdown.forget();

        let pointermove_session = Rc::clone(&session);
        let pointermove = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            handle_pointermove(&event, &pointermove_session);
        });
        element.add_event_listener_with_callback(
            "pointermove",
            pointermove.as_ref().unchecked_ref(),
        )?;
        pointermove.forget();

        // pointerup/pointercancel/pointerleave: 保持を開始したポインタと
        // 無関係な入力の終了イベントで誤って中断しないため、追跡中の
        // `pointer_id` と一致する場合のみキャンセルする（codex-review P2
        // 指摘）。マルチタッチで 2 本目の指が同じ要素へ pointerdown
        // すると（上記 pointerdown ハンドラの「保持中の追加 pointerdown
        // は無視する」節参照）、暗黙 pointer capture によりその指自身の
        // `pointerup`/`pointercancel` も当該要素へ配送されるが、
        // 追跡対象は最初の指のままのため一致せず無視される（2 本目の
        // 指の終了イベントで最初の指の保持を誤中断しない）。keydown
        // 起点の保持（`pointer_id` が `None`）は本ガードの対象外——
        // pointer 系イベントでは中断しない（下記 keyup ガードが担う）。
        for event_name in ["pointerup", "pointercancel", "pointerleave"] {
            let cancel_session = Rc::clone(&session);
            let closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
                let Some(pointer_event) = event.dyn_ref::<PointerEvent>() else {
                    return;
                };
                let Some(tracked_id) = cancel_session.pointer_id.get() else {
                    return;
                };
                if pointer_event.pointer_id() != tracked_id {
                    return;
                }
                cancel_session.cancel();
            });
            element
                .add_event_listener_with_callback(event_name, closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        // "blur" を早期離脱イベントへ加える理由（codex-review P1 指摘）:
        // 保持中に Tab で別要素へフォーカス移動すると、ポインタは離されず
        // keyup も届かないため上記 3 イベントだけでは中断できず、進行度が
        // 100% に達して合成 click が発火してしまう（「一定時間押し続けた
        // ときのみ確定」契約違反）。`blur` は要素がフォーカスを失う経路
        // （Tab 移動・他要素へのクリック）を broad にカバーするため、
        // pointerdown/keydown で開始したセッションを問わず入力の種類を
        // 問わず無条件に中断する（要素自身のフォーカス離脱そのものが
        // 中断理由であり、どの入力が起点だったかは無関係）。ウィンドウ
        // 全体のフォーカス喪失（Alt+Tab 等）はこの要素 `blur` では
        // 検知できない別経路のため、[`wire_global_interruption_guards`]
        // が window/document レベルで別途カバーする。
        {
            let cancel_session = Rc::clone(&session);
            let closure = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
                cancel_session.cancel();
            });
            element.add_event_listener_with_callback("blur", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        let keydown_session = Rc::clone(&session);
        let keydown = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            if let Ok(keyboard_event) = event.dyn_into::<KeyboardEvent>() {
                if !keyboard_event.repeat() && is_press_activation_key(&keyboard_event.key()) {
                    // 保持中（`active`）の追加 keydown は無視する
                    // （pointerdown ガードと同型）: 既に別の入力（ポインタ
                    // または別キー）で保持中の場合、ここで無条件に
                    // `active_key` を上書きすると以後の keyup 対応付けが
                    // 崩れる。
                    if keydown_session.active.get() {
                        return;
                    }
                    *keydown_session.active_key.borrow_mut() = Some(keyboard_event.key());
                    keydown_session.start();
                }
            }
        });
        element.add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())?;
        keydown.forget();

        let keyup_session = Rc::clone(&session);
        let keyup = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            if let Ok(keyboard_event) = event.dyn_into::<KeyboardEvent>() {
                if is_press_activation_key(&keyboard_event.key()) {
                    // 保持を開始したキーと同じ場合のみキャンセルする
                    // （codex-review P2 指摘）: 例えば Enter 押下中に
                    // 無関係な Space の keyup が届いても、起点キーが
                    // Enter のままなら保持を継続する。pointerdown 起点の
                    // 保持（`active_key` が `None`）は本ガードの対象外。
                    let matches = keyup_session
                        .active_key
                        .borrow()
                        .as_deref()
                        .is_some_and(|active_key| active_key == keyboard_event.key());
                    if matches {
                        keyup_session.cancel();
                    }
                }
            }
        });
        element.add_event_listener_with_callback("keyup", keyup.as_ref().unchecked_ref())?;
        keyup.forget();

        Ok(session)
    }

    /// root capture フェーズの `click` ガード（モジュール冒頭「確定の
    /// 不変条件」節）。
    fn wire_click_guard(root: &Element) -> Result<(), JsValue> {
        let guard_root = root.clone();
        let closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
            let Some(target) = event.target() else {
                return;
            };
            let Ok(target_element) = target.dyn_into::<Element>() else {
                return;
            };
            let Ok(Some(matched)) = target_element.closest(HOLD_TO_CONFIRM_SELECTOR) else {
                return;
            };
            if !guard_root.contains(Some(&matched)) {
                return;
            }
            if event.is_trusted() {
                event.stop_immediate_propagation();
            }
        });
        root.add_event_listener_with_callback_and_bool(
            "click",
            closure.as_ref().unchecked_ref(),
            true,
        )?;
        closure.forget();
        Ok(())
    }

    /// `window` の `blur`（ウィンドウ全体のフォーカス喪失、Alt+Tab
    /// 等）・`document` の `visibilitychange`（タブ切替・最小化）で
    /// 全セッションを無条件に中断する（codex-review P1 指摘）。
    ///
    /// 要素単位の `blur` リスナー（[`wire_candidate`] 参照）は Tab 移動
    /// 等の**フォーカス先が別要素へ移る**経路をカバーするが、ウィンドウ
    /// 全体が OS レベルでフォーカスを失う場合（Alt+Tab で別アプリへ
    /// 切り替える等）は多くのブラウザで `document.activeElement` が
    /// 変化しないため要素の `blur` は発火しない
    /// （`window.blur`/`document.visibilitychange` のみが発火する）。
    /// この経路を検知できないと、保持中に別ウィンドウへ切り替えて
    /// そちらで（この要素に届かない）`keyup`/ポインタ操作を行っても
    /// 保持ループが継続し、進行度が 100% に達して合成 click が発火して
    /// しまう（「一定時間押し続けたときのみ確定」契約違反）。`blur`/
    /// `visibilitychange` はどのセッションが中断対象か判別する手掛かり
    /// （target 等）を持たないため、全セッションを無条件に中断する
    /// （フォーカスを保持していないセッションを誤って中断しても、
    /// `HoldSession::cancel` は非 active なら no-op のため副作用がない）。
    fn wire_global_interruption_guards(
        window: &Window,
        root: &Element,
        sessions: &Rc<Vec<Rc<HoldSession>>>,
    ) -> Result<(), JsValue> {
        let Some(document) = root.owner_document() else {
            return Ok(());
        };

        let blur_sessions = Rc::clone(sessions);
        let window_blur = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
            for session in blur_sessions.iter() {
                session.cancel();
            }
        });
        window.add_event_listener_with_callback("blur", window_blur.as_ref().unchecked_ref())?;
        window_blur.forget();

        let visibility_sessions = Rc::clone(sessions);
        let visibilitychange = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
            for session in visibility_sessions.iter() {
                session.cancel();
            }
        });
        document.add_event_listener_with_callback(
            "visibilitychange",
            visibilitychange.as_ref().unchecked_ref(),
        )?;
        visibilitychange.forget();

        Ok(())
    }

    /// `root` 配下の hold-to-confirm 候補へ配線する
    /// （[`crate::lib::Runtime::mount`]/[`crate::lib::Runtime::hydrate`]
    /// から呼ばれる）。`window` が取得できない環境（テストランナー等）
    /// では配線をスキップし `Ok(())` を返す（fail-closed、動作しないのは
    /// 安全側、`headless_clipboard.rs::wire_clipboard_events` と同型）。
    ///
    /// # Errors
    ///
    /// `add_event_listener_with_callback`/
    /// `add_event_listener_with_callback_and_bool` の失敗を伝播する。
    pub fn wire_hold_to_confirm(root: Element) -> Result<(), JsValue> {
        let window: Window = match web_sys::window() {
            Some(window) => window,
            None => return Ok(()),
        };
        wire_click_guard(&root)?;
        let mut sessions = Vec::new();
        for candidate in collect_candidates(&root) {
            if candidate.has_attribute(HOLD_TO_CONFIRM_ATTR) {
                sessions.push(wire_candidate(&candidate)?);
            }
        }
        if !sessions.is_empty() {
            wire_global_interruption_guards(&window, &root, &Rc::new(sessions))?;
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::wire_hold_to_confirm;

#[cfg(test)]
mod tests {
    use super::{hold_progress, parse_hold_duration_ms, DEFAULT_HOLD_DURATION_MS};

    #[test]
    fn hold_progress_clamps_to_unit_interval() {
        assert_eq!(hold_progress(0.0, 1.0), 0.0);
        assert!((hold_progress(0.5, 1.0) - 0.5).abs() < f64::EPSILON);
        assert_eq!(hold_progress(1.0, 1.0), 1.0);
        assert_eq!(hold_progress(2.0, 1.0), 1.0);
        assert_eq!(hold_progress(-1.0, 1.0), 0.0);
    }

    #[test]
    fn hold_progress_nonpositive_duration_is_immediately_complete() {
        assert_eq!(hold_progress(0.0, 0.0), 1.0);
        assert_eq!(hold_progress(0.0, -1.0), 1.0);
    }

    #[test]
    fn parse_hold_duration_ms_falls_back_on_missing_or_invalid() {
        assert_eq!(parse_hold_duration_ms(None), DEFAULT_HOLD_DURATION_MS);
        assert_eq!(
            parse_hold_duration_ms(Some("not-a-number")),
            DEFAULT_HOLD_DURATION_MS
        );
        assert_eq!(parse_hold_duration_ms(Some("0")), DEFAULT_HOLD_DURATION_MS);
        assert_eq!(
            parse_hold_duration_ms(Some("-100")),
            DEFAULT_HOLD_DURATION_MS
        );
    }

    #[test]
    fn parse_hold_duration_ms_accepts_positive_value() {
        assert_eq!(parse_hold_duration_ms(Some("1200")), 1200.0);
        assert_eq!(parse_hold_duration_ms(Some("1200.5")), 1200.5);
    }
}
