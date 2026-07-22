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
//! [`DelayPhase`]・[`transition`]、native の `cargo test` で検証可能）と、
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

/// 現在の [`DelayPhase`] と [`DelayEvent`] から次のフェーズと副作用を
/// 決定する（web-sys 非依存の純粋関数、native `cargo test` で検証可能）。
///
/// 各フェーズ・イベントの組み合わせで未列挙のもの（当該フェーズで意味を
/// 持たないイベント）はいずれも `(phase, DelayEffect::None)`
/// （フェーズ変更なし・副作用なし）とし、panic しない（fail-closed）。
#[must_use]
pub fn transition(
    phase: DelayPhase,
    event: DelayEvent,
    config: &TooltipDelayConfig,
) -> (DelayPhase, DelayEffect) {
    use DelayEffect::{
        CancelTimer, None as NoEffect, RequestClose, RequestOpen, StartCloseTimer, StartOpenTimer,
    };
    use DelayEvent::{
        BlurTrigger, CloseTimerFired, FocusTrigger, OpenTimerFired, PointerEnterContent,
        PointerEnterTrigger, PointerLeaveContent, PointerLeaveTrigger,
    };
    use DelayPhase::{ClosePending, Closed, Open, OpenPending};

    match (phase, event) {
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
        // 即時 open へ昇格 ---
        (OpenPending, PointerLeaveTrigger) => (Closed, CancelTimer),
        (OpenPending, OpenTimerFired) => (Open, RequestOpen),
        (OpenPending, FocusTrigger) => (Open, RequestOpen),
        (OpenPending, BlurTrigger) => (Closed, CancelTimer),

        // --- Open: 表示中。トリガー離脱で close 遅延予約、interactive
        // なら content 側の進入/離脱も同様に扱う ---
        (Open, PointerLeaveTrigger) => {
            if config.close_delay_ms == 0 {
                (Closed, RequestClose)
            } else {
                (ClosePending, StartCloseTimer(config.close_delay_ms))
            }
        }
        (Open, BlurTrigger) => (Closed, RequestClose),
        (Open, PointerLeaveContent) if config.interactive => {
            if config.close_delay_ms == 0 {
                (Closed, RequestClose)
            } else {
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
    }
}

// ---------------------------------------------------------------------
// 配線層: web-sys 依存。wasm32 ターゲットでのみコンパイル対象とし、native の
// `cargo test --workspace` に本層の DOM 依存コードを混入させない
// （`events.rs`/`overlay.rs`/`keynav.rs` と同じ 2 層構成方針）。
// ---------------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
mod wiring {
    use super::{transition, DelayEffect, DelayEvent, DelayPhase, TooltipDelayConfig};
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

    /// [`TooltipDelayController`] が管理する 1 tooltip エントリの実体。
    struct MountedTooltip {
        config: TooltipDelayConfig,
        phase: DelayPhase,
        /// 保留中タイマーの handle（`Window::set_timeout_with_callback_and_timeout_and_arguments_0`
        /// の戻り値）。`None` は保留中タイマーなし。
        timer_handle: Option<i32>,
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
                phase: DelayPhase::Closed,
                timer_handle: None,
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
            if let Some(handle) = mounted.timer_handle {
                self.window.clear_timeout_with_handle(handle);
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
                    if let Some(handle) = mounted.timer_handle {
                        self.window.clear_timeout_with_handle(handle);
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
            let (next_phase, effect) = transition(mounted.phase, event, &mounted.config);
            mounted.phase = next_phase;
            // `DelayEffect::None`（未列挙の組み合わせ・現フェーズで意味を
            // 持たないイベント）は保留中タイマーへ一切干渉しない。
            // 例えば「非 interactive での content pointerenter」は no-op だが、
            // これがトリガー離脱由来の closeDelay タイマーを巻き添えで
            // キャンセルしてしまうと、`interactive=false` でも表示が
            // 維持され続ける不具合になる（実ブラウザ回帰テストで検出、
            // `tests/tooltip_delay_browser.rs::interactive_false_closes_even_when_pointer_moves_into_content`）。
            // 保留中タイマーのキャンセルは `effect` が実際にタイマーへ
            // 影響する場合（`StartOpenTimer`/`StartCloseTimer`/`CancelTimer`/
            // `RequestOpen`/`RequestClose`）に限定する。
            if !matches!(effect, DelayEffect::None) {
                if let Some(handle) = mounted.timer_handle.take() {
                    window.clear_timeout_with_handle(handle);
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
                // タイマー起動後は `handle`/クロージャ本体をエントリへ格納する。
                // クロージャは `timer_handle` を介して間接的に生存を制御する
                // （`Closure` を `forget` せず、コントローラ/エントリの生存期間に
                // 束縛する。`overlay.rs` の `Closure::forget` 非採用と同方針）。
                let mut guard = entries.borrow_mut();
                if let Some(Some(mounted)) = guard.get_mut(index) {
                    mounted.timer_handle = handle;
                }
                // クロージャ自体は `set_timeout` 呼び出し後、JS 側が保持する
                // 関数値としてのみ生存すればよいが、Rust 側の `Closure` を
                // 即座に drop すると呼び出し前に解放されてしまうため、
                // `forget` して JS 側のタイマーが担当する生存期間に委ねる。
                // `clear_timeout_with_handle` により、対応するクロージャが
                // 実際に呼ばれない場合でも JS 側の GC 対象になる
                // （`web_sys`/`wasm-bindgen` の `Closure::forget` の一般的な
                // 用法、`events.rs::wire_events` と同方針）。
                timer_closure.forget();
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

    #[test]
    fn closed_pointer_enter_trigger_schedules_open_timer() {
        let cfg = config(400, 150, false);
        let (phase, effect) = transition(DelayPhase::Closed, DelayEvent::PointerEnterTrigger, &cfg);
        assert_eq!(phase, DelayPhase::OpenPending);
        assert_eq!(effect, DelayEffect::StartOpenTimer(400));
    }

    #[test]
    fn closed_pointer_enter_trigger_zero_delay_opens_immediately() {
        let cfg = config(0, 150, false);
        let (phase, effect) = transition(DelayPhase::Closed, DelayEvent::PointerEnterTrigger, &cfg);
        assert_eq!(phase, DelayPhase::Open);
        assert_eq!(effect, DelayEffect::RequestOpen);
    }

    #[test]
    fn open_pending_early_leave_cancels_timer_without_opening() {
        let cfg = config(400, 150, false);
        let (phase, effect) = transition(
            DelayPhase::OpenPending,
            DelayEvent::PointerLeaveTrigger,
            &cfg,
        );
        assert_eq!(phase, DelayPhase::Closed);
        assert_eq!(effect, DelayEffect::CancelTimer);
    }

    #[test]
    fn open_pending_timer_fired_opens() {
        let cfg = config(400, 150, false);
        let (phase, effect) = transition(DelayPhase::OpenPending, DelayEvent::OpenTimerFired, &cfg);
        assert_eq!(phase, DelayPhase::Open);
        assert_eq!(effect, DelayEffect::RequestOpen);
    }

    // --- transition: close 遅延と再 enter 取消 ---

    #[test]
    fn open_pointer_leave_trigger_schedules_close_timer() {
        let cfg = config(400, 150, false);
        let (phase, effect) = transition(DelayPhase::Open, DelayEvent::PointerLeaveTrigger, &cfg);
        assert_eq!(phase, DelayPhase::ClosePending);
        assert_eq!(effect, DelayEffect::StartCloseTimer(150));
    }

    #[test]
    fn open_pointer_leave_trigger_zero_delay_closes_immediately() {
        let cfg = config(400, 0, false);
        let (phase, effect) = transition(DelayPhase::Open, DelayEvent::PointerLeaveTrigger, &cfg);
        assert_eq!(phase, DelayPhase::Closed);
        assert_eq!(effect, DelayEffect::RequestClose);
    }

    #[test]
    fn close_pending_re_enter_trigger_cancels_timer_and_stays_open() {
        let cfg = config(400, 150, false);
        let (phase, effect) = transition(
            DelayPhase::ClosePending,
            DelayEvent::PointerEnterTrigger,
            &cfg,
        );
        assert_eq!(phase, DelayPhase::Open);
        assert_eq!(effect, DelayEffect::CancelTimer);
    }

    #[test]
    fn close_pending_timer_fired_closes() {
        let cfg = config(400, 150, false);
        let (phase, effect) =
            transition(DelayPhase::ClosePending, DelayEvent::CloseTimerFired, &cfg);
        assert_eq!(phase, DelayPhase::Closed);
        assert_eq!(effect, DelayEffect::RequestClose);
    }

    // --- transition: interactive on/off での content enter/leave の効果差 ---

    #[test]
    fn interactive_false_content_leave_from_open_is_noop() {
        let cfg = config(400, 150, false);
        let (phase, effect) = transition(DelayPhase::Open, DelayEvent::PointerLeaveContent, &cfg);
        assert_eq!(
            phase,
            DelayPhase::Open,
            "非 interactive では content leave は無視される"
        );
        assert_eq!(effect, DelayEffect::None);
    }

    #[test]
    fn interactive_true_content_leave_from_open_schedules_close_timer() {
        let cfg = config(400, 150, true);
        let (phase, effect) = transition(DelayPhase::Open, DelayEvent::PointerLeaveContent, &cfg);
        assert_eq!(phase, DelayPhase::ClosePending);
        assert_eq!(effect, DelayEffect::StartCloseTimer(150));
    }

    #[test]
    fn interactive_true_content_enter_while_close_pending_cancels_timer() {
        let cfg = config(400, 150, true);
        let (phase, effect) = transition(
            DelayPhase::ClosePending,
            DelayEvent::PointerEnterContent,
            &cfg,
        );
        assert_eq!(phase, DelayPhase::Open);
        assert_eq!(effect, DelayEffect::CancelTimer);
    }

    #[test]
    fn interactive_false_content_enter_while_close_pending_is_noop() {
        let cfg = config(400, 150, false);
        let (phase, effect) = transition(
            DelayPhase::ClosePending,
            DelayEvent::PointerEnterContent,
            &cfg,
        );
        assert_eq!(
            phase,
            DelayPhase::ClosePending,
            "非 interactive では content enter は close タイマーを取消さない"
        );
        assert_eq!(effect, DelayEffect::None);
    }

    // --- transition: focus 即時 open・blur 即時 close ---

    #[test]
    fn closed_focus_trigger_opens_immediately_ignoring_delay() {
        let cfg = config(400, 150, false);
        let (phase, effect) = transition(DelayPhase::Closed, DelayEvent::FocusTrigger, &cfg);
        assert_eq!(phase, DelayPhase::Open);
        assert_eq!(effect, DelayEffect::RequestOpen);
    }

    #[test]
    fn open_pending_focus_trigger_opens_immediately() {
        let cfg = config(400, 150, false);
        let (phase, effect) = transition(DelayPhase::OpenPending, DelayEvent::FocusTrigger, &cfg);
        assert_eq!(phase, DelayPhase::Open);
        assert_eq!(effect, DelayEffect::RequestOpen);
    }

    #[test]
    fn open_blur_trigger_closes_immediately_ignoring_delay() {
        let cfg = config(400, 150, false);
        let (phase, effect) = transition(DelayPhase::Open, DelayEvent::BlurTrigger, &cfg);
        assert_eq!(phase, DelayPhase::Closed);
        assert_eq!(effect, DelayEffect::RequestClose);
    }

    #[test]
    fn close_pending_blur_trigger_closes_immediately() {
        let cfg = config(400, 150, false);
        let (phase, effect) = transition(DelayPhase::ClosePending, DelayEvent::BlurTrigger, &cfg);
        assert_eq!(phase, DelayPhase::Closed);
        assert_eq!(effect, DelayEffect::RequestClose);
    }

    // --- transition: 未知の遷移は no-op（fail-closed） ---

    #[test]
    fn closed_pointer_leave_trigger_is_noop() {
        let cfg = config(400, 150, false);
        let (phase, effect) = transition(DelayPhase::Closed, DelayEvent::PointerLeaveTrigger, &cfg);
        assert_eq!(phase, DelayPhase::Closed);
        assert_eq!(effect, DelayEffect::None);
    }

    #[test]
    fn open_pointer_enter_trigger_is_noop() {
        let cfg = config(400, 150, false);
        let (phase, effect) = transition(DelayPhase::Open, DelayEvent::PointerEnterTrigger, &cfg);
        assert_eq!(phase, DelayPhase::Open);
        assert_eq!(effect, DelayEffect::None);
    }

    #[test]
    fn close_pending_pointer_leave_content_is_noop() {
        let cfg = config(400, 150, true);
        let (phase, effect) = transition(
            DelayPhase::ClosePending,
            DelayEvent::PointerLeaveContent,
            &cfg,
        );
        assert_eq!(phase, DelayPhase::ClosePending);
        assert_eq!(effect, DelayEffect::None);
    }

    #[test]
    fn all_phase_event_combinations_never_panic() {
        // 全フェーズ × 全イベント × interactive on/off の組み合わせを走査し、
        // 未列挙の組み合わせも panic しないことを回帰として固定する。
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
                for event in events {
                    let _ = transition(phase, event, &cfg);
                }
            }
        }
    }
}
