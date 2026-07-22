//! Tooltip の `openDelay`/`closeDelay`/`interactive`（イシュー #587、親 #584）。
//!
//! `fandhe-frontend-headless-ui` の Tooltip（#564、`crates/headless-ui/src/tooltip.rs`）は
//! anatomy・開閉状態機械までを提供し、表示・非表示の遅延タイマーと
//! `interactive`（content 内へのポインタ移動時の維持）を「タイマー・
//! ポインタ座標などのクライアントサイド実行時挙動であり wasm 層の後続
//! スコープ」として明示的にスコープ外としていた（同モジュール冒頭 doc の
//! 「スコープ外」節参照）。本モジュールはその欠落を埋める。
//!
//! [`crate::events`]/[`crate::overlay`]/[`crate::keynav`] と同じ 2 層構成を
//! 踏襲する: web-sys に依存しない純粋ロジック層（[`TooltipDelayConfig`]・
//! [`DelayState`]・[`transition`]、native の `cargo test` で検証可能）と、
//! `#[cfg(target_arch = "wasm32")]` でゲートした配線層
//! （[`wiring::TooltipDelayController`]）に分離する。
//!
//! # 他モジュール・他クレートとの契約
//!
//! - [`TooltipDelayConfig::from_attrs`] は呼び出し側が Tooltip の
//!   `root`（`data-scope="tooltip"` `data-part="root"`）要素の `attrs`
//!   引数（`fandhe_frontend_headless_ui::tooltip::root` の `attrs: Vec<(&str, &str)>`）
//!   経由で付与する `data-open-delay`/`data-close-delay`/`data-interactive` を
//!   読む。headless-ui 側に専用 API を追加せず、[`crate::overlay`] の
//!   opt-out 属性と同じ「呼び出し側が anatomy パーツの `attrs` でオプトイン
//!   する」方式を踏襲する（本イシューでは headless-ui クレートを変更しない）。
//! - [`crate::overlay`] は `OverlayKind::Tooltip` を
//!   `close_on_interact_outside = false`（オーバーレイスタック非参加）と
//!   既定化しており、「本モジュールの closeDelay/interactive と競合しない
//!   ため」と明記済み（`overlay.rs` 冒頭 doc・`OverlayKind::close_on_interact_outside`
//!   doc 参照）。本モジュールは overlay.rs のスタック処理に一切関与しない。
//! - 本モジュールは実際の `"open"`/`"close"` dispatch
//!   （`fandhe_frontend_interactive::dispatch`、
//!   `fandhe_frontend_headless_ui::tooltip::Tooltip::decode_action` の語彙）・
//!   再描画・DOM 更新を一切行わない。[`wiring::TooltipDelayController`] は
//!   要求発生時にコールバック（[`TooltipDelayRequest`]）へ通知するのみで
//!   あり、`dispatch` の実呼び出しは呼び出し側（イシュー #580 の DOM
//!   イベント配線統合層）の責務とする（[`crate::overlay`] と同じ責務分離
//!   方針）。
//!
//! # フォーカスと遅延の使い分け
//!
//! ポインタ操作（`pointerenter`/`pointerleave`）には設定された遅延を適用する
//! 一方、`focusin`/`focusout`（キーボード操作でのトリガー到達・離脱）は
//! 遅延なしで即時に開閉する。これは ark-ui/zag.js の tooltip 実装
//! （`.claude/skills/ark-ui/references/components/overlays/tooltip.md`）が
//! キーボード操作時は Tab で即時表示・Tab 移動や Escape で即時非表示とする
//! 挙動に合わせた意図的な判断である（WAI-ARIA tooltip パターン: フォーカス
//! 可能なトリガーにフォーカスが当たった瞬間に説明が読めることを保証する
//! 必要があり、遅延を挟むとキーボード操作者の体験を損なう）。
//!
//! ポインタとフォーカスは独立した入力チャネルであり、どちらか一方が
//! まだ「表示継続の理由」を持っている限り非表示にしてはならない
//! （WAI-ARIA tooltip パターン: フォーカスされたトリガーは、ポインタが
//! 離脱しても説明を表示し続ける必要がある）。そのため [`DelayState`] は
//! `phase` に加えて `pointer_over_trigger`/`pointer_over_content`/`focused`
//! を独立に保持し、[`transition`] は非表示へつながるイベント
//! （`PointerLeaveTrigger`/`BlurTrigger`/`PointerLeaveContent`）到着時に
//! 「もう一方のチャネルがまだ表示継続を要求していないか」を必ず確認して
//! から遷移を決定する（`transition` 内 `stay_open` 判定）。片方のチャネル
//! 由来の離脱イベントだけを見て即座に非表示化すると、Tab 操作後もポインタ
//! がホバーしたままの状態・ポインタ離脱後もフォーカスが残ったままの状態の
//! いずれでも tooltip が消えてしまう不具合になる（イシュー #587 の
//! Cursor Bugbot 指摘・回帰は `tests/tooltip_delay_browser.rs::
//! blur_does_not_close_while_pointer_still_hovers_trigger`/
//! `pointer_leave_does_not_close_while_trigger_still_focused` 参照）。
//!
//! # セキュリティ不変条件
//!
//! - 本モジュールは HTML 文字列の直接組み立て・`set_inner_html` を一切
//!   行わない。DOM への作用はタイマー・リスナー登録・コールバック通知の
//!   みであり、属性書き込みすら行わない（既定エスケープ迂回経路を持たない）。
//! - `data-open-delay`/`data-close-delay`/`data-interactive` は改ざんされ
//!   うるクライアント入力として扱う。欠落・非数値・不正値は
//!   [`TooltipDelayConfig::from_attrs`] が文書化された既定へ決定的に
//!   フォールバックし、panic しない（fail-closed）。

use crate::events::AttrSource;

/// [`TooltipDelayConfig`] の既定 `openDelay`（ミリ秒）。
///
/// ark-ui の Tooltip 既定値（`.claude/skills/ark-ui/references/components/overlays/tooltip.md`）
/// に合わせる。
pub const DEFAULT_OPEN_DELAY_MS: u32 = 400;

/// [`TooltipDelayConfig`] の既定 `closeDelay`（ミリ秒）。ark-ui 既定値に合わせる。
pub const DEFAULT_CLOSE_DELAY_MS: u32 = 150;

/// 遅延値の上限（ミリ秒）。改ざんされた `data-open-delay`/`data-close-delay`
/// に非現実的に巨大な値（例: `u32::MAX`）を与えることで tooltip を実質的に
/// 無効化する DoS 的な入力を拒否するためのクランプ（fail-closed）。
pub const MAX_DELAY_MS: u32 = 60_000;

/// Tooltip の表示・非表示遅延と `interactive` 設定
/// （[`TooltipDelayConfig::from_attrs`] で `data-*` 属性から復元する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TooltipDelayConfig {
    /// トリガーへのポインタ進入から表示までの遅延（ミリ秒）。
    pub open_delay_ms: u32,
    /// トリガー（または `interactive` 時は content）からのポインタ離脱から
    /// 非表示までの遅延（ミリ秒）。
    pub close_delay_ms: u32,
    /// `true` のとき、content 内へのポインタ移動が close タイマーを取消し
    /// 表示を維持する（ark-ui の `interactive` prop 相当）。
    pub interactive: bool,
}

impl Default for TooltipDelayConfig {
    fn default() -> Self {
        Self {
            open_delay_ms: DEFAULT_OPEN_DELAY_MS,
            close_delay_ms: DEFAULT_CLOSE_DELAY_MS,
            interactive: false,
        }
    }
}

impl TooltipDelayConfig {
    /// `root` パーツの `data-open-delay`/`data-close-delay`/`data-interactive`
    /// 属性から設定を復元する。
    ///
    /// fail-closed 方針: 属性の欠落・非数値・負値相当（`u32` パース失敗）は
    /// 既定値（[`DEFAULT_OPEN_DELAY_MS`]/[`DEFAULT_CLOSE_DELAY_MS`]/
    /// `interactive = false`）へ決定的にフォールバックする。パースに成功
    /// した値も [`MAX_DELAY_MS`] でクランプする（巨大値による実質無効化を
    /// 防ぐ）。`data-interactive` は文字列 `"true"` のときのみ有効
    /// （他のあらゆる値・欠落は `false`）。
    #[must_use]
    pub fn from_attrs<T: AttrSource>(root: &T) -> Self {
        let open_delay_ms = root
            .attr("data-open-delay")
            .and_then(|value| value.parse::<u32>().ok())
            .map(|value| value.min(MAX_DELAY_MS))
            .unwrap_or(DEFAULT_OPEN_DELAY_MS);
        let close_delay_ms = root
            .attr("data-close-delay")
            .and_then(|value| value.parse::<u32>().ok())
            .map(|value| value.min(MAX_DELAY_MS))
            .unwrap_or(DEFAULT_CLOSE_DELAY_MS);
        let interactive = root.attr("data-interactive").as_deref() == Some("true");
        Self {
            open_delay_ms,
            close_delay_ms,
            interactive,
        }
    }
}

/// Tooltip 表示制御の遅延フェーズ状態機械が取りうる状態。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelayPhase {
    /// 非表示。表示要求は未発生。
    Closed,
    /// トリガーへの進入を受け、`openDelay` タイマー満了待ち。
    OpenPending,
    /// 表示中。
    Open,
    /// トリガー（または interactive 時は content）からの離脱を受け、
    /// `closeDelay` タイマー満了待ち。
    ClosePending,
}

/// [`transition`] が保持する完全な状態（表示フェーズ + 独立した入力
/// チャネルの現況）。
///
/// `phase` のみでは「ポインタとフォーカスのどちらが表示継続を要求して
/// いるか」を区別できず、一方の離脱イベントだけで非表示にしてしまう
/// （モジュール冒頭 doc「フォーカスと遅延の使い分け」節参照、イシュー #587
/// の Cursor Bugbot 指摘）。`pointer_over_trigger`/`pointer_over_content`/
/// `focused` を独立に追跡し、[`transition`] がいずれかの離脱イベントを
/// 受けた際に「もう一方のチャネルがまだ表示継続を要求していないか」を
/// 判定できるようにする。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DelayState {
    /// 現在の表示フェーズ。
    pub phase: DelayPhase,
    /// ポインタが現在 trigger 上にあるか。
    pointer_over_trigger: bool,
    /// ポインタが現在 content 上にあるか（`interactive` 時のみ意味を持つ）。
    pointer_over_content: bool,
    /// trigger が現在フォーカスされているか。
    focused: bool,
}

impl DelayState {
    /// 初期状態（非表示・いずれの入力チャネルも非アクティブ）を返す。
    #[must_use]
    pub fn closed() -> Self {
        Self {
            phase: DelayPhase::Closed,
            pointer_over_trigger: false,
            pointer_over_content: false,
            focused: false,
        }
    }
}

/// [`transition`] への入力イベント（web-sys 非依存の抽象化。実 DOM イベント
/// との対応は [`wiring`] が担う）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelayEvent {
    /// トリガーへポインタが進入した（`pointerenter` 相当）。
    PointerEnterTrigger,
    /// トリガーからポインタが離脱した（`pointerleave` 相当）。
    PointerLeaveTrigger,
    /// トリガーがフォーカスを得た（`focusin` 相当）。
    FocusTrigger,
    /// トリガーがフォーカスを失った（`focusout` 相当）。
    BlurTrigger,
    /// content へポインタが進入した（`pointerenter` 相当）。
    PointerEnterContent,
    /// content からポインタが離脱した（`pointerleave` 相当）。
    PointerLeaveContent,
    /// `openDelay` タイマーが満了した。
    OpenTimerFired,
    /// `closeDelay` タイマーが満了した。
    CloseTimerFired,
}

/// [`transition`] が返す、呼び出し側（[`wiring::TooltipDelayController`]）が
/// 実行すべき副作用。
///
/// 呼び出し側は `effect` が [`DelayEffect::None`] 以外のとき、新しい効果を
/// 適用する前に当該 tooltip エントリの保留中タイマー（あれば）を必ず先に
/// キャンセルしてから本効果を処理する契約とする
/// （[`wiring::TooltipDelayController`] doc 参照）。これにより「タイマー
/// 満了前の即時遷移（フォーカス等）がタイマーの停止と表示/非表示要求の
/// 両方を意味する」ケースを、本 enum に複合バリアントを持たせずに表現
/// できる。**`DelayEffect::None` のときは保留中タイマーへ一切干渉しない**
/// （現フェーズで意味を持たないイベント・未列挙の組み合わせは、進行中の
/// 遅延タイマーを巻き添えでキャンセルしてはならない。例:
/// `interactive=false` での content `pointerenter` は no-op だが、これが
/// トリガー離脱由来の `closeDelay` タイマーを誤って止めてはならない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelayEffect {
    /// 何もしない（保留中タイマーがあれば呼び出し側がキャンセルするのみ）。
    None,
    /// `openDelay` タイマーを（再）開始する。
    StartOpenTimer(u32),
    /// `closeDelay` タイマーを（再）開始する。
    StartCloseTimer(u32),
    /// 保留中タイマーをキャンセルするのみ（新規タイマーは張らない）。
    CancelTimer,
    /// 表示を要求する（呼び出し側が `dispatch("open")` 等を行う）。
    RequestOpen,
    /// 非表示を要求する（呼び出し側が `dispatch("close")` 等を行う）。
    RequestClose,
}

/// 現在の [`DelayState`] と [`DelayEvent`] から次の状態と副作用を
/// 決定する（web-sys 非依存の純粋関数、native `cargo test` で検証可能）。
///
/// 各フェーズ・イベントの組み合わせで未列挙のもの（当該フェーズで意味を
/// 持たないイベント）はいずれも `(state, DelayEffect::None)`
/// （フェーズ変更なし・副作用なし）とし、panic しない（fail-closed）。
///
/// # ポインタ/フォーカス競合の解決（`stay_open`）
///
/// `PointerLeaveTrigger`/`BlurTrigger`/`PointerLeaveContent`（非表示へ
/// つながりうるイベント）は、まず [`DelayState`] の該当チャネルを更新した
/// 上で `stay_open`（`focused || pointer_over_trigger ||
/// (interactive && pointer_over_content)`）を評価し、**もう一方の
/// チャネルがまだ表示継続を要求している場合は非表示へ遷移しない**
/// （タイマー開始・`RequestClose` のいずれも発行しない）。これにより
/// 「フォーカスされたトリガーは、ポインタが離脱しても説明を表示し続ける」
/// という WAI-ARIA tooltip パターンの要求を満たす（モジュール冒頭 doc
/// 「フォーカスと遅延の使い分け」節参照、イシュー #587 の Cursor Bugbot
/// 指摘の回帰）。この契約は `Open` フェーズだけでなく `OpenPending`
/// フェーズの `PointerLeaveTrigger`/`BlurTrigger`/`PointerLeaveContent`
/// にも同様に適用する（`openDelay` 満了待ち中の早期離脱でも、もう一方の
/// チャネルが表示継続を要求していれば pending open を取消さず
/// `OpenPending` に留まる。`PointerLeaveContent` arm は PR #619 の
/// Cursor Bugbot 再指摘で追加: 欠落時は catch-all no-op に落ち、
/// `pointer_over_content` は更新されるのに open タイマーはキャンセル
/// されないままだった）。
///
/// また `closeDelay == 0` かつ `interactive` のときも、`Open` からの
/// `PointerLeaveTrigger`/`PointerLeaveContent` は直接 `Closed` へは
/// 遷移せず `ClosePending` + `StartCloseTimer(0)` を経由する（PR #619
/// の Cursor Bugbot 再指摘: 直接 `Closed` へ遷移すると「トリガー離脱 →
/// content 進入」の通常シーケンスで content 進入を待たずに閉じてしまい、
/// `interactive` の意味が失われていた。`StartCloseTimer(0)` は 0ms の
/// `set_timeout` を予約するだけで同期的即時発火ではないため、直後に
/// 届く `PointerEnterContent` がキャンセルする猶予が残る）。
/// `interactive == false` のときは content へ移動する余地がないため、
/// 従来通り直接 `Closed` へ遷移する。
#[must_use]
pub fn transition(
    mut state: DelayState,
    event: DelayEvent,
    config: &TooltipDelayConfig,
) -> (DelayState, DelayEffect) {
    use DelayEffect::{
        CancelTimer, None as NoEffect, RequestClose, RequestOpen, StartCloseTimer, StartOpenTimer,
    };
    use DelayEvent::{
        BlurTrigger, CloseTimerFired, FocusTrigger, OpenTimerFired, PointerEnterContent,
        PointerEnterTrigger, PointerLeaveContent, PointerLeaveTrigger,
    };
    use DelayPhase::{ClosePending, Closed, Open, OpenPending};

    // 各イベントの意味に沿って、まず入力チャネルの現況を更新する。
    // `OpenTimerFired`/`CloseTimerFired` はどちらのチャネルにも属さない
    // 内部イベントのため対象外。
    match event {
        PointerEnterTrigger => state.pointer_over_trigger = true,
        PointerLeaveTrigger => state.pointer_over_trigger = false,
        FocusTrigger => state.focused = true,
        BlurTrigger => state.focused = false,
        PointerEnterContent => state.pointer_over_content = true,
        PointerLeaveContent => state.pointer_over_content = false,
        DelayEvent::OpenTimerFired | DelayEvent::CloseTimerFired => {}
    }

    // もう一方のチャネルがまだ表示継続を要求しているか（doc 上記参照）。
    let stay_open = state.focused
        || state.pointer_over_trigger
        || (config.interactive && state.pointer_over_content);

    let phase = state.phase;
    let (next_phase, effect) = match (phase, event) {
        // --- Closed: トリガーへの進入・フォーカスのみが表示へつながる ---
        (Closed, PointerEnterTrigger) => {
            if config.open_delay_ms == 0 {
                (Open, RequestOpen)
            } else {
                (OpenPending, StartOpenTimer(config.open_delay_ms))
            }
        }
        (Closed, FocusTrigger) => (Open, RequestOpen),

        // --- OpenPending: 満了待ち。早期離脱でタイマー取消・フォーカスは
        // 即時 open へ昇格。ただし `stay_open`（もう一方のチャネルがまだ
        // 表示継続を要求している）が真のときは pending open を取消さず
        // 留まる（`Open` フェーズと同じ multi-channel 契約、イシュー #587
        // Cursor Bugbot 指摘の回帰: `openDelay` 中の focusout や、
        // interactive content がまだホバーされている状態でのトリガー離脱で
        // pending open を誤ってキャンセルしない）---
        (OpenPending, PointerLeaveTrigger) => {
            if stay_open {
                (OpenPending, NoEffect)
            } else {
                (Closed, CancelTimer)
            }
        }
        (OpenPending, OpenTimerFired) => (Open, RequestOpen),
        (OpenPending, FocusTrigger) => (Open, RequestOpen),
        (OpenPending, BlurTrigger) => {
            if stay_open {
                (OpenPending, NoEffect)
            } else {
                (Closed, CancelTimer)
            }
        }
        // `interactive` な content が `openDelay` 満了待ち中に既に描画・
        // ホバーされている状態からの content 離脱（イシュー #587 Cursor
        // Bugbot 指摘: 本 arm が欠落していると catch-all（no-op）に落ち、
        // `pointer_over_content` は false 更新されるのに open タイマーは
        // キャンセルされず、他チャネルもすべて clear のまま
        // `OpenTimerFired` が発火して無操作なのに tooltip が開いてしまう）。
        // `PointerLeaveTrigger`/`BlurTrigger` と同じ `stay_open` 判定に従う。
        (OpenPending, PointerLeaveContent) if config.interactive => {
            if stay_open {
                (OpenPending, NoEffect)
            } else {
                (Closed, CancelTimer)
            }
        }

        // --- Open: 表示中。トリガー離脱で close 遅延予約、interactive
        // なら content 側の進入/離脱も同様に扱う。ただし `stay_open` が
        // 真（もう一方のチャネルが表示継続を要求している）のときは
        // 非表示へ遷移せず Open に留まる（`stay_open` doc 参照） ---
        (Open, PointerLeaveTrigger) => {
            if stay_open {
                (Open, NoEffect)
            } else if config.close_delay_ms == 0 && !config.interactive {
                // `interactive` が false のときは content へ移動する余地が
                // ないため、`closeDelay == 0` は即時 `Closed` で問題ない。
                (Closed, RequestClose)
            } else {
                // `interactive` なら `closeDelay == 0` でも `ClosePending`
                // を経由させる（`StartCloseTimer(0)` は 0ms の
                // `set_timeout` を予約するのみで即時発火ではないため、
                // 直後に届く `PointerEnterContent` がタイマーをキャンセル
                // する猶予が残る。イシュー #587 Cursor Bugbot 指摘: 直接
                // `Closed` へ遷移すると「トリガー離脱 → content 進入」の
                // 通常シーケンスが content 進入を待たずに閉じてしまい
                // `interactive` を破壊していた）。
                (ClosePending, StartCloseTimer(config.close_delay_ms))
            }
        }
        (Open, BlurTrigger) => {
            if stay_open {
                (Open, NoEffect)
            } else {
                (Closed, RequestClose)
            }
        }
        (Open, PointerLeaveContent) if config.interactive => {
            if stay_open {
                (Open, NoEffect)
            } else {
                // `interactive` 経路のため上記 `PointerLeaveTrigger` と同じ
                // 理由で `closeDelay == 0` でも常に `ClosePending` を経由し、
                // `StartCloseTimer(0)` として直後の再進入に猶予を残す。
                (ClosePending, StartCloseTimer(config.close_delay_ms))
            }
        }

        // --- ClosePending: 満了待ち。再進入・フォーカスでタイマー取消し
        // Open へ復帰 ---
        (ClosePending, PointerEnterTrigger) => (Open, CancelTimer),
        (ClosePending, FocusTrigger) => (Open, CancelTimer),
        (ClosePending, PointerEnterContent) if config.interactive => (Open, CancelTimer),
        (ClosePending, CloseTimerFired) => (Closed, RequestClose),
        (ClosePending, BlurTrigger) => (Closed, RequestClose),

        // --- 上記いずれにも一致しない組み合わせ: フェーズ非変更・no-op
        // （改ざん・未知の順序で発火したイベントに対する fail-closed） ---
        (current, _) => (current, NoEffect),
    };

    state.phase = next_phase;
    (state, effect)
}

// ---------------------------------------------------------------------
// 配線層: web-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、native の
// `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`events.rs`/`overlay.rs`/`keynav.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{transition, DelayEffect, DelayEvent, DelayState, TooltipDelayConfig};
    use crate::events::AttrSource;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};
    use web_sys::{Element, Event, Window};

    /// `web_sys::Element` を [`AttrSource`] へ橋渡しする薄いラッパー
    /// （`events.rs::wiring::ElementAttrSource`/`overlay.rs::wiring::ElementAttrSource`
    /// と同じ意図の配線層専用アダプタ）。
    struct ElementAttrSource<'a>(&'a Element);

    impl AttrSource for ElementAttrSource<'_> {
        fn attr(&self, name: &str) -> Option<String> {
            self.0.get_attribute(name)
        }
    }

    /// [`TooltipDelayController`] が発する、表示・非表示要求の通知。
    ///
    /// `dispatch`（`"open"`/`"close"` アクション、
    /// `fandhe_frontend_headless_ui::tooltip::Tooltip::decode_action` の語彙）の
    /// 実呼び出し・再描画・DOM 更新は呼び出し側（イシュー #580 の統合層）の
    /// 責務であり、本モジュールはこの通知を渡すのみで完結する
    /// （[`super::overlay::OverlayCloseRequest`] と同じ責務分離、モジュール
    /// 冒頭 doc 参照）。
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TooltipDelayRequest {
        /// [`TooltipDelayController::register_tooltip`] が返した登録 index。
        ///
        /// # 不変条件: index は [`TooltipDelayController::remove_tooltip`] で再利用されうる
        ///
        /// 本コントローラは `overlay.rs::OverlayCloseController` の
        /// `Vec::remove`（上位 index の一括シフト）とは異なり、削除済み
        /// スロットを `None` のまま残し、次の [`TooltipDelayController::register_tooltip`]
        /// 呼び出しでその空きスロットを再利用する（index はシフトしないが、
        /// 削除後に再利用され得る）。Tooltip は Dialog/Popover のような
        /// 「常に最上位のみ操作するスタック」ではなく、複数個が独立した
        /// ライフサイクルで頻繁に mount/unmount される想定であるため、
        /// シフトを避けてハンドルの安定性を優先する設計判断とする。
        /// 呼び出し側は `remove_tooltip` 済みの index を保持し続けず、
        /// 対応する tooltip の DOM 要素が破棄されたら速やかに対応表からも
        /// 除去すること。
        pub index: usize,
        /// 要求内容（表示/非表示）。
        pub action: TooltipDelayAction,
    }

    /// [`TooltipDelayRequest`] が示す要求種別。
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TooltipDelayAction {
        /// 表示要求（`dispatch("open")` 相当）。
        Open,
        /// 非表示要求（`dispatch("close")` 相当）。
        Close,
    }

    /// 登録済み 1 リスナー分の `(target 要素, event 名, closure)`
    /// （[`MountedTooltip::listeners`]/[`add_delay_listener`] で使う型エイリアス。
    /// clippy `type_complexity` 回避も兼ねる）。
    type DelayListener = (Element, &'static str, Closure<dyn FnMut(Event)>);

    /// 保留中タイマー 1 件分の `(handle, closure)`。
    ///
    /// `handle` のみを保持して `Closure` を `forget()` すると、キャンセル・
    /// 発火のたびに Rust 側の `Closure` が回収されず恒久的にリークする
    /// （イシュー #587 の Cursor Bugbot 指摘）。`Closure` を本構造体へ
    /// 保持し、[`apply_event`] の保留中タイマー解除処理・
    /// [`TooltipDelayController::remove_tooltip`]・[`Drop`] のいずれかで
    /// `Some(PendingTimer)` を `take()` して破棄することで解放する。
    ///
    /// タイマー発火時（[`apply_event`] がこの `_closure` 自身の呼び出し中に
    /// 実行される再帰呼び出し）に本構造体を破棄しても安全である:
    /// `wasm_bindgen::closure::Closure` は内部で参照カウントを用いており、
    /// 「自身の呼び出し中に `Closure` を drop しても、呼び出しが完了するまで
    /// 実体の破棄を遅延する」設計になっている
    /// （`wasm-bindgen` `convert/closures.rs` の `into_js_function` 実装
    /// コメント・`Closure::once_into_js` が同じパターンで自己解放している
    /// ことを根拠とする）。
    struct PendingTimer {
        handle: i32,
        _closure: Closure<dyn FnMut()>,
    }

    /// [`TooltipDelayController`] が管理する 1 tooltip エントリの実体。
    struct MountedTooltip {
        config: TooltipDelayConfig,
        state: DelayState,
        /// 保留中タイマー（handle + closure）。`None` は保留中タイマーなし。
        timer: Option<PendingTimer>,
        /// このエントリ専用の登録済みリスナー一覧（`(target element, event
        /// 名, closure)`）。[`TooltipDelayController::remove_tooltip`]/[`Drop`]
        /// で対称的に解除するために保持する。
        listeners: Vec<DelayListener>,
    }

    /// Tooltip の `openDelay`/`closeDelay`/`interactive` を扱う配線層の中核型。
    ///
    /// [`crate::overlay::OverlayCloseController`]・[`crate::keynav`] は
    /// document/root へ委譲リスナーを **1 回だけ** 登録するが、
    /// `pointerenter`/`pointerleave` はバブリングしないイベント種別のため
    /// document への委譲登録では捕捉できない。そのため本コントローラは
    /// [`Self::register_tooltip`] で登録された各 trigger/content 要素へ
    /// **直接** リスナーを付ける設計とする（overlay.rs の document 委譲方式
    /// との意図的な差異）。[`Self::remove_tooltip`]・[`Drop`] でエントリ
    /// ごとのリスナーとタイマーを対称的に解除し、リスナー・タイマーの
    /// リーク（A04 安全でない設計対策）を防ぐ。
    pub struct TooltipDelayController {
        window: Window,
        entries: std::rc::Rc<std::cell::RefCell<Vec<Option<MountedTooltip>>>>,
        on_request: std::rc::Rc<std::cell::RefCell<dyn FnMut(TooltipDelayRequest)>>,
    }

    impl TooltipDelayController {
        /// `window` を保持するコントローラを組み立てる（`setTimeout`/
        /// `clearTimeout` に使う。`document` は各 `register_tooltip` 呼び出しが
        /// 引数で要素を直接受け取るため保持しない）。
        ///
        /// `on_request` は状態変更・DOM 更新を一切行わず、呼び出し側
        /// （#580 統合層）へ「どの tooltip が表示/非表示になるべきか」を
        /// 通知するだけの役割に限定する（モジュール冒頭 doc の責務分離）。
        #[must_use]
        pub fn new(window: &Window, on_request: impl FnMut(TooltipDelayRequest) + 'static) -> Self {
            Self {
                window: window.clone(),
                entries: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
                on_request: std::rc::Rc::new(std::cell::RefCell::new(on_request)),
            }
        }

        /// tooltip 1 個を登録し、`trigger`/`content` へ直接リスナーを付ける。
        ///
        /// `root` の `data-open-delay`/`data-close-delay`/`data-interactive`
        /// を登録時点で 1 回読み取り [`TooltipDelayConfig`] へスナップショット
        /// する（`overlay.rs::push_overlay` の opt-out 属性スナップショットと
        /// 同じ方針。開閉のたびに設定属性が変わる想定はない）。
        ///
        /// 戻り値の index は [`TooltipDelayRequest::index`] と対応し、呼び出し
        /// 側が [`Self::remove_tooltip`] を呼ぶ際に使う。
        ///
        /// # Errors
        ///
        /// `add_event_listener_with_callback` が失敗した場合に `Err` を返す
        /// （登録途中で失敗した場合、それまでに付けた本エントリ分のリスナーは
        /// 全て解除してから返す。他エントリのリスナーには影響しない）。
        pub fn register_tooltip(
            &self,
            root: &Element,
            trigger: &Element,
            content: &Element,
        ) -> Result<usize, JsValue> {
            let source = ElementAttrSource(root);
            let config = TooltipDelayConfig::from_attrs(&source);

            let entries = self.entries.clone();
            let on_request = self.on_request.clone();
            let window = self.window.clone();

            // index はこの後 entries へ push/再利用する位置で確定するが、
            // タイマークロージャ・イベントリスナーのクロージャは「登録後に
            // 確定した index」を捕捉する必要があるため、まず空きスロットの
            // 位置（無ければ push 予定の末尾位置）を予約してから listeners を
            // 組み立てる。
            let index = {
                let mut guard = entries.borrow_mut();
                if let Some(slot) = guard.iter().position(Option::is_none) {
                    slot
                } else {
                    guard.push(None);
                    guard.len() - 1
                }
            };

            let mut listeners: Vec<DelayListener> = Vec::new();
            let register_result = (|| -> Result<(), JsValue> {
                add_delay_listener(
                    trigger,
                    "pointerenter",
                    DelayEvent::PointerEnterTrigger,
                    index,
                    &entries,
                    &on_request,
                    &window,
                    &mut listeners,
                )?;
                add_delay_listener(
                    trigger,
                    "pointerleave",
                    DelayEvent::PointerLeaveTrigger,
                    index,
                    &entries,
                    &on_request,
                    &window,
                    &mut listeners,
                )?;
                add_delay_listener(
                    trigger,
                    "focusin",
                    DelayEvent::FocusTrigger,
                    index,
                    &entries,
                    &on_request,
                    &window,
                    &mut listeners,
                )?;
                add_delay_listener(
                    trigger,
                    "focusout",
                    DelayEvent::BlurTrigger,
                    index,
                    &entries,
                    &on_request,
                    &window,
                    &mut listeners,
                )?;
                add_delay_listener(
                    content,
                    "pointerenter",
                    DelayEvent::PointerEnterContent,
                    index,
                    &entries,
                    &on_request,
                    &window,
                    &mut listeners,
                )?;
                add_delay_listener(
                    content,
                    "pointerleave",
                    DelayEvent::PointerLeaveContent,
                    index,
                    &entries,
                    &on_request,
                    &window,
                    &mut listeners,
                )?;
                Ok(())
            })();

            if let Err(err) = register_result {
                // 途中まで登録済みのリスナーを解除してからエラーを返す
                // （リスナーリーク防止。overlay.rs::OverlayCloseController::new
                // の keydown/pointerdown 登録失敗時の対称解除と同方針）。
                for (element, name, closure) in listeners.drain(..) {
                    let _ = element.remove_event_listener_with_callback(
                        name,
                        closure.as_ref().unchecked_ref(),
                    );
                }
                return Err(err);
            }

            let mut guard = entries.borrow_mut();
            let mounted = MountedTooltip {
                config,
                state: DelayState::closed(),
                timer: None,
                listeners,
            };
            if index < guard.len() {
                guard[index] = Some(mounted);
            } else {
                guard.push(Some(mounted));
            }
            Ok(index)
        }

        /// `index` の tooltip を登録解除する。リスナー・保留中タイマーを
        /// 対称的に解除し、スロットを空き（`None`）に戻す（後続の
        /// [`Self::register_tooltip`] で再利用されうる、
        /// [`TooltipDelayRequest::index`] doc 参照）。
        ///
        /// `index` が範囲外・既に空きの場合は panic せず no-op とする
        /// （呼び出し側の二重 remove・契約違反に対する安全側フォールバック）。
        pub fn remove_tooltip(&self, index: usize) {
            let mut guard = self.entries.borrow_mut();
            let Some(slot) = guard.get_mut(index) else {
                return;
            };
            let Some(mounted) = slot.take() else {
                return;
            };
            if let Some(timer) = mounted.timer {
                self.window.clear_timeout_with_handle(timer.handle);
            }
            for (element, name, closure) in mounted.listeners {
                let _ = element
                    .remove_event_listener_with_callback(name, closure.as_ref().unchecked_ref());
            }
        }

        /// 現在登録されている（かつ削除されていない）tooltip の件数
        /// （テスト・デバッグ用途）。
        #[must_use]
        pub fn active_len(&self) -> usize {
            self.entries.borrow().iter().filter(|e| e.is_some()).count()
        }
    }

    impl Drop for TooltipDelayController {
        /// 残存する全エントリのリスナー・保留中タイマーを解除する
        /// （[`Self::remove_tooltip`] を呼び忘れたまま `TooltipDelayController`
        /// 自体が破棄されるケースの安全網。A04 対策）。
        fn drop(&mut self) {
            let mut guard = self.entries.borrow_mut();
            for slot in guard.iter_mut() {
                if let Some(mounted) = slot.take() {
                    if let Some(timer) = mounted.timer {
                        self.window.clear_timeout_with_handle(timer.handle);
                    }
                    for (element, name, closure) in mounted.listeners {
                        let _ = element.remove_event_listener_with_callback(
                            name,
                            closure.as_ref().unchecked_ref(),
                        );
                    }
                }
            }
        }
    }

    /// `target` へ `event_name` のリスナーを登録し、発火時に
    /// [`apply_event`]（[`transition`] 適用 + 効果実行）を呼ぶ。
    ///
    /// `listeners` へ `(target, event_name, closure)` を積んで呼び出し元へ
    /// 返す（登録失敗時の対称解除・[`TooltipDelayController::remove_tooltip`]
    /// での解除の双方に使う）。
    #[allow(clippy::too_many_arguments)]
    fn add_delay_listener(
        target: &Element,
        event_name: &'static str,
        delay_event: DelayEvent,
        index: usize,
        entries: &std::rc::Rc<std::cell::RefCell<Vec<Option<MountedTooltip>>>>,
        on_request: &std::rc::Rc<std::cell::RefCell<dyn FnMut(TooltipDelayRequest)>>,
        window: &Window,
        listeners: &mut Vec<DelayListener>,
    ) -> Result<(), JsValue> {
        let entries = entries.clone();
        let on_request = on_request.clone();
        let window = window.clone();
        let closure = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
            apply_event(index, delay_event, &entries, &on_request, &window);
        });
        target.add_event_listener_with_callback(event_name, closure.as_ref().unchecked_ref())?;
        listeners.push((target.clone(), event_name, closure));
        Ok(())
    }

    /// `index` の tooltip に対して `event` を [`transition`] へ適用し、
    /// 返された [`DelayEffect`] を実行する（タイマー開始・キャンセル・
    /// [`TooltipDelayRequest`] 通知）。
    ///
    /// [`DelayEffect`] doc の契約通り、新しい効果を適用する前に必ず
    /// 保留中タイマー（あれば）を先にキャンセルしてから処理する。
    fn apply_event(
        index: usize,
        event: DelayEvent,
        entries: &std::rc::Rc<std::cell::RefCell<Vec<Option<MountedTooltip>>>>,
        on_request: &std::rc::Rc<std::cell::RefCell<dyn FnMut(TooltipDelayRequest)>>,
        window: &Window,
    ) {
        let effect = {
            let mut guard = entries.borrow_mut();
            let Some(Some(mounted)) = guard.get_mut(index) else {
                return;
            };
            let (next_state, effect) = transition(mounted.state, event, &mounted.config);
            mounted.state = next_state;
            // `DelayEffect::None`（未列挙の組み合わせ・現フェーズで意味を
            // 持たないイベント）は保留中タイマーへ一切干渉しない。
            // 例えば「非 interactive での content pointerenter」は no-op だが、
            // これがトリガー離脱由来の closeDelay タイマーを巻き添えで
            // キャンセルしてしまうと、`interactive=false` でも表示が
            // 維持され続ける不具合になる（実ブラウザ回帰テストで検出、
            // `tests/tooltip_delay_browser.rs::interactive_false_closes_even_when_pointer_moves_into_content`）。
            // 保留中タイマーのキャンセルは `effect` が実際にタイマーへ
            // 影響する場合（`StartOpenTimer`/`StartCloseTimer`/`CancelTimer`/
            // `RequestOpen`/`RequestClose`）に限定する。`timer.take()` は
            // handle・`Closure` の双方を同時に破棄する（[`PendingTimer`] doc
            // の「自己呼び出し中の drop も安全」根拠を参照。本呼び出しが
            // まさにそのタイマー自身の発火経由であっても安全）。
            if !matches!(effect, DelayEffect::None) {
                if let Some(timer) = mounted.timer.take() {
                    window.clear_timeout_with_handle(timer.handle);
                }
            }
            effect
        };

        match effect {
            DelayEffect::None | DelayEffect::CancelTimer => {}
            DelayEffect::StartOpenTimer(ms) | DelayEffect::StartCloseTimer(ms) => {
                let fired_event = if matches!(effect, DelayEffect::StartOpenTimer(_)) {
                    DelayEvent::OpenTimerFired
                } else {
                    DelayEvent::CloseTimerFired
                };
                let timer_entries = entries.clone();
                let timer_on_request = on_request.clone();
                let timer_window = window.clone();
                let timer_closure = Closure::<dyn FnMut()>::new(move || {
                    apply_event(
                        index,
                        fired_event,
                        &timer_entries,
                        &timer_on_request,
                        &timer_window,
                    );
                });
                let handle = window
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        timer_closure.as_ref().unchecked_ref(),
                        ms as i32,
                    )
                    .ok();
                // タイマー起動後は `handle`/クロージャ本体を [`PendingTimer`]
                // としてエントリへ格納する。`Closure` は `forget()` せず、
                // 本エントリ（[`MountedTooltip::timer`]）の生存期間に束縛
                // する（キャンセル時・発火時・`remove_tooltip`/`Drop` の
                // いずれかで確実に破棄され、`forget()` によるホバーごとの
                // 恒久リークを避ける。[`PendingTimer`] doc 参照、
                // イシュー #587 の Cursor Bugbot 指摘）。
                // `set_timeout` が失敗し `handle` が `None` の場合は
                // `Closure` を保持し続けても発火しないため、素直に drop する
                // （タイマーが張られなかった以上、保留中タイマーは存在しない）。
                if let Some(handle) = handle {
                    let mut guard = entries.borrow_mut();
                    if let Some(Some(mounted)) = guard.get_mut(index) {
                        mounted.timer = Some(PendingTimer {
                            handle,
                            _closure: timer_closure,
                        });
                    }
                }
            }
            DelayEffect::RequestOpen => {
                (on_request.borrow_mut())(TooltipDelayRequest {
                    index,
                    action: TooltipDelayAction::Open,
                });
            }
            DelayEffect::RequestClose => {
                (on_request.borrow_mut())(TooltipDelayRequest {
                    index,
                    action: TooltipDelayAction::Close,
                });
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wiring::{TooltipDelayAction, TooltipDelayController, TooltipDelayRequest};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// native `cargo test` 用のテストダブル（`events.rs::tests::FakeElement`/
    /// `overlay.rs::tests::FakeElement` と同じ意図）。
    struct FakeElement {
        attrs: HashMap<&'static str, &'static str>,
    }

    impl AttrSource for FakeElement {
        fn attr(&self, name: &str) -> Option<String> {
            self.attrs.get(name).map(|v| v.to_string())
        }
    }

    fn element(attrs: &[(&'static str, &'static str)]) -> FakeElement {
        FakeElement {
            attrs: attrs.iter().copied().collect(),
        }
    }

    // --- TooltipDelayConfig::from_attrs（fail-closed 既定値） ---

    #[test]
    fn from_attrs_defaults_when_all_absent() {
        let config = TooltipDelayConfig::from_attrs(&element(&[]));
        assert_eq!(config.open_delay_ms, DEFAULT_OPEN_DELAY_MS);
        assert_eq!(config.close_delay_ms, DEFAULT_CLOSE_DELAY_MS);
        assert!(!config.interactive);
    }

    #[test]
    fn from_attrs_reads_valid_values() {
        let config = TooltipDelayConfig::from_attrs(&element(&[
            ("data-open-delay", "10"),
            ("data-close-delay", "20"),
            ("data-interactive", "true"),
        ]));
        assert_eq!(config.open_delay_ms, 10);
        assert_eq!(config.close_delay_ms, 20);
        assert!(config.interactive);
    }

    #[test]
    fn from_attrs_falls_back_on_non_numeric() {
        for bogus in ["", "abc", "-1", "1.5"] {
            let config = TooltipDelayConfig::from_attrs(&element(&[("data-open-delay", bogus)]));
            assert_eq!(
                config.open_delay_ms, DEFAULT_OPEN_DELAY_MS,
                "value={bogus:?}"
            );
        }
    }

    #[test]
    fn from_attrs_clamps_huge_values() {
        let config = TooltipDelayConfig::from_attrs(&element(&[("data-open-delay", "4294967295")]));
        assert_eq!(config.open_delay_ms, MAX_DELAY_MS);
    }

    #[test]
    fn from_attrs_interactive_only_true_string_enables() {
        for bogus in ["True", "1", "yes", ""] {
            let config = TooltipDelayConfig::from_attrs(&element(&[("data-interactive", bogus)]));
            assert!(!config.interactive, "value={bogus:?}");
        }
        let config = TooltipDelayConfig::from_attrs(&element(&[("data-interactive", "true")]));
        assert!(config.interactive);
    }

    // --- transition: open 遅延予約と早期 leave 取消 ---

    fn config(open_delay_ms: u32, close_delay_ms: u32, interactive: bool) -> TooltipDelayConfig {
        TooltipDelayConfig {
            open_delay_ms,
            close_delay_ms,
            interactive,
        }
    }

    /// 指定 `phase` かつ全入力チャネル非アクティブな [`DelayState`] を作る
    /// （既存の「フェーズのみ」の単体テストが暗黙に仮定していた初期状態）。
    fn state(phase: DelayPhase) -> DelayState {
        DelayState {
            phase,
            pointer_over_trigger: false,
            pointer_over_content: false,
            focused: false,
        }
    }

    /// 入力チャネルを明示指定した [`DelayState`] を作る（ポインタ/フォーカス
    /// 競合の回帰テスト用）。
    fn state_with(
        phase: DelayPhase,
        pointer_over_trigger: bool,
        pointer_over_content: bool,
        focused: bool,
    ) -> DelayState {
        DelayState {
            phase,
            pointer_over_trigger,
            pointer_over_content,
            focused,
        }
    }

    #[test]
    fn closed_pointer_enter_trigger_schedules_open_timer() {
        let cfg = config(400, 150, false);
        let (next, effect) = transition(
            state(DelayPhase::Closed),
            DelayEvent::PointerEnterTrigger,
            &cfg,
        );
        assert_eq!(next.phase, DelayPhase::OpenPending);
        assert_eq!(effect, DelayEffect::StartOpenTimer(400));
    }

    #[test]
    fn closed_pointer_enter_trigger_zero_delay_opens_immediately() {
        let cfg = config(0, 150, false);
        let (next, effect) = transition(
            state(DelayPhase::Closed),
            DelayEvent::PointerEnterTrigger,
            &cfg,
        );
        assert_eq!(next.phase, DelayPhase::Open);
        assert_eq!(effect, DelayEffect::RequestOpen);
    }

    #[test]
    fn open_pending_early_leave_cancels_timer_without_opening() {
        let cfg = config(400, 150, false);
        let (next, effect) = transition(
            state(DelayPhase::OpenPending),
            DelayEvent::PointerLeaveTrigger,
            &cfg,
        );
        assert_eq!(next.phase, DelayPhase::Closed);
        assert_eq!(effect, DelayEffect::CancelTimer);
    }

    #[test]
    fn open_pending_timer_fired_opens() {
        let cfg = config(400, 150, false);
        let (next, effect) = transition(
            state(DelayPhase::OpenPending),
            DelayEvent::OpenTimerFired,
            &cfg,
        );
        assert_eq!(next.phase, DelayPhase::Open);
        assert_eq!(effect, DelayEffect::RequestOpen);
    }

    // --- transition: close 遅延と再 enter 取消 ---

    #[test]
    fn open_pointer_leave_trigger_schedules_close_timer() {
        let cfg = config(400, 150, false);
        let (next, effect) = transition(
            state(DelayPhase::Open),
            DelayEvent::PointerLeaveTrigger,
            &cfg,
        );
        assert_eq!(next.phase, DelayPhase::ClosePending);
        assert_eq!(effect, DelayEffect::StartCloseTimer(150));
    }

    #[test]
    fn open_pointer_leave_trigger_zero_delay_closes_immediately() {
        let cfg = config(400, 0, false);
        let (next, effect) = transition(
            state(DelayPhase::Open),
            DelayEvent::PointerLeaveTrigger,
            &cfg,
        );
        assert_eq!(next.phase, DelayPhase::Closed);
        assert_eq!(effect, DelayEffect::RequestClose);
    }

    #[test]
    fn close_pending_re_enter_trigger_cancels_timer_and_stays_open() {
        let cfg = config(400, 150, false);
        let (next, effect) = transition(
            state(DelayPhase::ClosePending),
            DelayEvent::PointerEnterTrigger,
            &cfg,
        );
        assert_eq!(next.phase, DelayPhase::Open);
        assert_eq!(effect, DelayEffect::CancelTimer);
    }

    #[test]
    fn close_pending_timer_fired_closes() {
        let cfg = config(400, 150, false);
        let (next, effect) = transition(
            state(DelayPhase::ClosePending),
            DelayEvent::CloseTimerFired,
            &cfg,
        );
        assert_eq!(next.phase, DelayPhase::Closed);
        assert_eq!(effect, DelayEffect::RequestClose);
    }

    // --- transition: interactive on/off での content enter/leave の効果差 ---

    #[test]
    fn interactive_false_content_leave_from_open_is_noop() {
        let cfg = config(400, 150, false);
        let (next, effect) = transition(
            state(DelayPhase::Open),
            DelayEvent::PointerLeaveContent,
            &cfg,
        );
        assert_eq!(
            next.phase,
            DelayPhase::Open,
            "非 interactive では content leave は無視される"
        );
        assert_eq!(effect, DelayEffect::None);
    }

    #[test]
    fn interactive_true_content_leave_from_open_schedules_close_timer() {
        let cfg = config(400, 150, true);
        let (next, effect) = transition(
            state(DelayPhase::Open),
            DelayEvent::PointerLeaveContent,
            &cfg,
        );
        assert_eq!(next.phase, DelayPhase::ClosePending);
        assert_eq!(effect, DelayEffect::StartCloseTimer(150));
    }

    #[test]
    fn interactive_true_content_enter_while_close_pending_cancels_timer() {
        let cfg = config(400, 150, true);
        let (next, effect) = transition(
            state(DelayPhase::ClosePending),
            DelayEvent::PointerEnterContent,
            &cfg,
        );
        assert_eq!(next.phase, DelayPhase::Open);
        assert_eq!(effect, DelayEffect::CancelTimer);
    }

    #[test]
    fn interactive_false_content_enter_while_close_pending_is_noop() {
        let cfg = config(400, 150, false);
        let (next, effect) = transition(
            state(DelayPhase::ClosePending),
            DelayEvent::PointerEnterContent,
            &cfg,
        );
        assert_eq!(
            next.phase,
            DelayPhase::ClosePending,
            "非 interactive では content enter は close タイマーを取消さない"
        );
        assert_eq!(effect, DelayEffect::None);
    }

    // --- transition: focus 即時 open・blur 即時 close ---

    #[test]
    fn closed_focus_trigger_opens_immediately_ignoring_delay() {
        let cfg = config(400, 150, false);
        let (next, effect) = transition(state(DelayPhase::Closed), DelayEvent::FocusTrigger, &cfg);
        assert_eq!(next.phase, DelayPhase::Open);
        assert_eq!(effect, DelayEffect::RequestOpen);
    }

    #[test]
    fn open_pending_focus_trigger_opens_immediately() {
        let cfg = config(400, 150, false);
        let (next, effect) = transition(
            state(DelayPhase::OpenPending),
            DelayEvent::FocusTrigger,
            &cfg,
        );
        assert_eq!(next.phase, DelayPhase::Open);
        assert_eq!(effect, DelayEffect::RequestOpen);
    }

    #[test]
    fn open_blur_trigger_closes_immediately_ignoring_delay() {
        let cfg = config(400, 150, false);
        let (next, effect) = transition(state(DelayPhase::Open), DelayEvent::BlurTrigger, &cfg);
        assert_eq!(next.phase, DelayPhase::Closed);
        assert_eq!(effect, DelayEffect::RequestClose);
    }

    #[test]
    fn close_pending_blur_trigger_closes_immediately() {
        let cfg = config(400, 150, false);
        let (next, effect) = transition(
            state(DelayPhase::ClosePending),
            DelayEvent::BlurTrigger,
            &cfg,
        );
        assert_eq!(next.phase, DelayPhase::Closed);
        assert_eq!(effect, DelayEffect::RequestClose);
    }

    // --- transition: ポインタ/フォーカス競合の解決（stay_open、イシュー #587
    // Cursor Bugbot 指摘・回帰）---

    #[test]
    fn open_blur_trigger_stays_open_while_pointer_still_hovers_trigger() {
        // Tab でフォーカスして Open した後、ポインタが trigger 上にまだ
        // ある状態で Tab 移動（blur）しても、ポインタがまだ表示継続を
        // 要求しているため非表示にしてはならない。
        let cfg = config(400, 150, false);
        let hovering_and_focused = state_with(DelayPhase::Open, true, false, true);
        let (next, effect) = transition(hovering_and_focused, DelayEvent::BlurTrigger, &cfg);
        assert_eq!(
            next.phase,
            DelayPhase::Open,
            "ポインタがまだ trigger 上にある間は blur で非表示にしてはならない"
        );
        assert_eq!(effect, DelayEffect::None);
        assert!(
            !next.focused,
            "focused フラグ自体は blur で false に更新される"
        );
    }

    #[test]
    fn open_pointer_leave_trigger_stays_open_while_trigger_still_focused() {
        // フォーカスされたトリガーから素早くポインタが離脱しても、
        // フォーカスがまだ trigger にある限り非表示にしてはならない
        // （WAI-ARIA tooltip パターン）。
        let cfg = config(400, 150, false);
        let hovering_and_focused = state_with(DelayPhase::Open, true, false, true);
        let (next, effect) =
            transition(hovering_and_focused, DelayEvent::PointerLeaveTrigger, &cfg);
        assert_eq!(
            next.phase,
            DelayPhase::Open,
            "trigger がまだフォーカスされている間は pointerleave で非表示にしてはならない"
        );
        assert_eq!(effect, DelayEffect::None);
        assert!(
            !next.pointer_over_trigger,
            "pointer_over_trigger フラグ自体は leave で false に更新される"
        );
    }

    #[test]
    fn open_blur_trigger_closes_when_pointer_already_left() {
        // ポインタも trigger 上になければ、blur は従来通り即時に非表示化する
        // （回帰: stay_open 判定の追加が「両方離脱時の即時 close」を弱めて
        // いないことを固定する）。
        let cfg = config(400, 150, false);
        let focused_only = state_with(DelayPhase::Open, false, false, true);
        let (next, effect) = transition(focused_only, DelayEvent::BlurTrigger, &cfg);
        assert_eq!(next.phase, DelayPhase::Closed);
        assert_eq!(effect, DelayEffect::RequestClose);
    }

    #[test]
    fn open_pointer_leave_trigger_schedules_close_when_not_focused() {
        // フォーカスもされていなければ、pointerleave は従来通り closeDelay
        // タイマーを予約する（回帰）。
        let cfg = config(400, 150, false);
        let hovering_only = state_with(DelayPhase::Open, true, false, false);
        let (next, effect) = transition(hovering_only, DelayEvent::PointerLeaveTrigger, &cfg);
        assert_eq!(next.phase, DelayPhase::ClosePending);
        assert_eq!(effect, DelayEffect::StartCloseTimer(150));
    }

    #[test]
    fn interactive_true_content_leave_stays_open_while_trigger_still_hovered() {
        // interactive=true で content から離脱しても、ポインタが trigger
        // 上にまだある間は非表示にしてはならない。
        let cfg = config(400, 150, true);
        let hovering_trigger = state_with(DelayPhase::Open, true, true, false);
        let (next, effect) = transition(hovering_trigger, DelayEvent::PointerLeaveContent, &cfg);
        assert_eq!(next.phase, DelayPhase::Open);
        assert_eq!(effect, DelayEffect::None);
    }

    #[test]
    fn open_pending_blur_trigger_stays_pending_while_pointer_still_hovers_trigger() {
        // `openDelay` 待ち中にフォーカスが外れても、ポインタが trigger
        // 上にまだある間は pending open を取消してはならない
        // （イシュー #587 Cursor Bugbot 指摘・回帰）。
        let cfg = config(400, 150, false);
        let hovering_and_focused = state_with(DelayPhase::OpenPending, true, false, true);
        let (next, effect) = transition(hovering_and_focused, DelayEvent::BlurTrigger, &cfg);
        assert_eq!(
            next.phase,
            DelayPhase::OpenPending,
            "ポインタがまだ trigger 上にある間は blur で pending open を取消してはならない"
        );
        assert_eq!(effect, DelayEffect::None);
        assert!(
            !next.focused,
            "focused フラグ自体は blur で false に更新される"
        );
    }

    #[test]
    fn open_pending_pointer_leave_trigger_stays_pending_while_trigger_still_focused() {
        // `openDelay` 待ち中にポインタが trigger から離脱しても、
        // フォーカスがまだ trigger にある限り pending open を取消しては
        // ならない（イシュー #587 Cursor Bugbot 指摘・回帰）。
        let cfg = config(400, 150, false);
        let hovering_and_focused = state_with(DelayPhase::OpenPending, true, false, true);
        let (next, effect) =
            transition(hovering_and_focused, DelayEvent::PointerLeaveTrigger, &cfg);
        assert_eq!(
            next.phase,
            DelayPhase::OpenPending,
            "trigger がまだフォーカスされている間は pointerleave で pending open を取消してはならない"
        );
        assert_eq!(effect, DelayEffect::None);
        assert!(
            !next.pointer_over_trigger,
            "pointer_over_trigger フラグ自体は leave で false に更新される"
        );
    }

    #[test]
    fn open_pending_blur_trigger_cancels_when_pointer_already_left() {
        // ポインタも trigger 上になければ、`openDelay` 待ち中の blur は
        // 従来通り pending open を即時取消する（回帰: stay_open 判定の
        // 追加が「両方離脱時の即時キャンセル」を弱めていないことを固定
        // する）。
        let cfg = config(400, 150, false);
        let focused_only = state_with(DelayPhase::OpenPending, false, false, true);
        let (next, effect) = transition(focused_only, DelayEvent::BlurTrigger, &cfg);
        assert_eq!(next.phase, DelayPhase::Closed);
        assert_eq!(effect, DelayEffect::CancelTimer);
    }

    #[test]
    fn open_pending_pointer_leave_trigger_cancels_when_not_focused() {
        // フォーカスもされていなければ、`openDelay` 待ち中の pointerleave
        // は従来通り pending open を即時取消する（回帰）。
        let cfg = config(400, 150, false);
        let hovering_only = state_with(DelayPhase::OpenPending, true, false, false);
        let (next, effect) = transition(hovering_only, DelayEvent::PointerLeaveTrigger, &cfg);
        assert_eq!(next.phase, DelayPhase::Closed);
        assert_eq!(effect, DelayEffect::CancelTimer);
    }

    #[test]
    fn open_pending_pointer_leave_trigger_stays_pending_while_interactive_content_hovered() {
        // interactive=true で `openDelay` 待ち中に content がまだホバー
        // されていれば、トリガー離脱で pending open を取消してはならない
        // （Bugbot 指摘のもう一方のシナリオ: interactive content hover）。
        let cfg = config(400, 150, true);
        let hovering_content = state_with(DelayPhase::OpenPending, true, true, false);
        let (next, effect) = transition(hovering_content, DelayEvent::PointerLeaveTrigger, &cfg);
        assert_eq!(
            next.phase,
            DelayPhase::OpenPending,
            "interactive content がまだホバーされている間は pending open を取消してはならない"
        );
        assert_eq!(effect, DelayEffect::None);
    }

    #[test]
    fn open_pending_pointer_leave_content_cancels_when_no_other_channel_stays_open() {
        // イシュー #587 Cursor Bugbot 指摘（PR #619 レビュー）:
        // interactive content が `openDelay` 満了待ち中にホバーされていた
        // 状態から content を離脱すると、他の入力チャネル（トリガー
        // ホバー・フォーカス）が clear のままなら pending open を
        // 取消さなければならない。取消さない回帰が起きると、無操作なのに
        // `OpenTimerFired` が `RequestOpen` を発行してしまう。
        let cfg = config(400, 150, true);
        let hovering_content_only = state_with(DelayPhase::OpenPending, false, true, false);
        let (next, effect) =
            transition(hovering_content_only, DelayEvent::PointerLeaveContent, &cfg);
        assert_eq!(next.phase, DelayPhase::Closed);
        assert_eq!(effect, DelayEffect::CancelTimer);
    }

    #[test]
    fn open_pending_pointer_leave_content_stays_pending_while_trigger_still_hovered() {
        // 上記と対の回帰: もう一方のチャネル（トリガーホバー）がまだ表示
        // 継続を要求していれば、content 離脱だけで pending open を
        // 取消してはならない（`stay_open` 契約は content leave arm にも
        // 一貫して適用する）。
        let cfg = config(400, 150, true);
        let hovering_trigger_and_content = state_with(DelayPhase::OpenPending, true, true, false);
        let (next, effect) = transition(
            hovering_trigger_and_content,
            DelayEvent::PointerLeaveContent,
            &cfg,
        );
        assert_eq!(next.phase, DelayPhase::OpenPending);
        assert_eq!(effect, DelayEffect::None);
    }

    #[test]
    fn open_pending_pointer_leave_content_is_noop_when_not_interactive() {
        // `interactive=false` では content leave イベント自体が意味を
        // 持たないため、従来通り catch-all の no-op に落ちる（回帰）。
        let cfg = config(400, 150, false);
        let hovering_content_only = state_with(DelayPhase::OpenPending, false, true, false);
        let (next, effect) =
            transition(hovering_content_only, DelayEvent::PointerLeaveContent, &cfg);
        assert_eq!(next.phase, DelayPhase::OpenPending);
        assert_eq!(effect, DelayEffect::None);
    }

    #[test]
    fn open_pointer_leave_trigger_zero_delay_interactive_goes_through_close_pending() {
        // イシュー #587 Cursor Bugbot 指摘（PR #619 レビュー）:
        // `interactive=true` かつ `closeDelay == 0` でも、トリガー離脱は
        // 直接 `Closed` へ遷移してはならない。`ClosePending` +
        // `StartCloseTimer(0)` を経由させることで、直後に届く
        // `PointerEnterContent`（トリガー離脱 → content 進入の通常
        // シーケンス）がタイマーをキャンセルする猶予を残す。直接
        // `Closed` へ遷移すると content 進入を待たずに閉じてしまい
        // `interactive` が機能しない。
        let cfg = config(400, 0, true);
        let (next, effect) = transition(
            state(DelayPhase::Open),
            DelayEvent::PointerLeaveTrigger,
            &cfg,
        );
        assert_eq!(next.phase, DelayPhase::ClosePending);
        assert_eq!(effect, DelayEffect::StartCloseTimer(0));
    }

    #[test]
    fn open_pointer_leave_content_zero_delay_interactive_goes_through_close_pending() {
        // 上記と対のシナリオ: content からの離脱（トリガー未進入のまま
        // content 内を経由して離脱するケース）でも同様に `ClosePending`
        // を経由させる。
        let cfg = config(400, 0, true);
        let (next, effect) = transition(
            state(DelayPhase::Open),
            DelayEvent::PointerLeaveContent,
            &cfg,
        );
        assert_eq!(next.phase, DelayPhase::ClosePending);
        assert_eq!(effect, DelayEffect::StartCloseTimer(0));
    }

    // --- transition: 未知の遷移は no-op（fail-closed） ---

    #[test]
    fn closed_pointer_leave_trigger_is_noop() {
        let cfg = config(400, 150, false);
        let (next, effect) = transition(
            state(DelayPhase::Closed),
            DelayEvent::PointerLeaveTrigger,
            &cfg,
        );
        assert_eq!(next.phase, DelayPhase::Closed);
        assert_eq!(effect, DelayEffect::None);
    }

    #[test]
    fn open_pointer_enter_trigger_is_noop() {
        let cfg = config(400, 150, false);
        let (next, effect) = transition(
            state(DelayPhase::Open),
            DelayEvent::PointerEnterTrigger,
            &cfg,
        );
        assert_eq!(next.phase, DelayPhase::Open);
        assert_eq!(effect, DelayEffect::None);
    }

    #[test]
    fn close_pending_pointer_leave_content_is_noop() {
        let cfg = config(400, 150, true);
        let (next, effect) = transition(
            state(DelayPhase::ClosePending),
            DelayEvent::PointerLeaveContent,
            &cfg,
        );
        assert_eq!(next.phase, DelayPhase::ClosePending);
        assert_eq!(effect, DelayEffect::None);
    }

    #[test]
    fn all_phase_event_combinations_never_panic() {
        // 全フェーズ × 全イベント × interactive on/off × 入力チャネルの
        // 組み合わせを走査し、未列挙の組み合わせも panic しないことを
        // 回帰として固定する。
        let phases = [
            DelayPhase::Closed,
            DelayPhase::OpenPending,
            DelayPhase::Open,
            DelayPhase::ClosePending,
        ];
        let events = [
            DelayEvent::PointerEnterTrigger,
            DelayEvent::PointerLeaveTrigger,
            DelayEvent::FocusTrigger,
            DelayEvent::BlurTrigger,
            DelayEvent::PointerEnterContent,
            DelayEvent::PointerLeaveContent,
            DelayEvent::OpenTimerFired,
            DelayEvent::CloseTimerFired,
        ];
        for interactive in [false, true] {
            let cfg = config(400, 150, interactive);
            for phase in phases {
                for pointer_over_trigger in [false, true] {
                    for pointer_over_content in [false, true] {
                        for focused in [false, true] {
                            let s = state_with(
                                phase,
                                pointer_over_trigger,
                                pointer_over_content,
                                focused,
                            );
                            for event in events {
                                let _ = transition(s, event, &cfg);
                            }
                        }
                    }
                }
            }
        }
    }
}
