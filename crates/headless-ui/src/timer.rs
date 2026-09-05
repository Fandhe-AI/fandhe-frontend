//! Timer（カウントダウン/カウントアップ表示）headless コンポーネント
//! （イシュー #836、親トラッキング #520）。
//!
//! ark-ui の Timer
//!（`.claude/skills/ark-ui/references/components/date-time/timer.md`）を
//! 参考に、Root / Area / Item / ItemValue / ItemLabel / Separator / Control /
//! ActionTrigger の 8 anatomy パーツと、idle/running/paused/completed の
//! 4 値状態機械 [`Timer`] を提供する。
//!
//! # 保留解除の経緯（`docs/policy/intentional-non-adoption.md` §7）
//!
//! Timer は `docs/design/component-coverage-map.md` で date-time 系コンポーネント
//! （暦計算の自前実装コストを理由に保留）として扱われてきたが、Timer 自体は
//! 暦・ロケール・タイムゾーンを一切必要としない（ミリ秒の加算とセグメント
//! 分解のみ）。本モジュールは「tick を外部から明示的に与える決定的状態機械
//! （状態機械自身は時計を持たない）」という設計により、時計 API 非依存のまま
//! 実装できることを示し、保留を解除する（再評価トリガー「決定的に実装できる
//! 設計の提示」の充足根拠）。
//!
//! # 時計 API 非依存（レビュー観点）
//!
//! 本モジュールは `std::time`・`Instant`・`js_sys::Date` のいずれも使用しない。
//! 時間の前進は [`TimerAction::Tick`]（デルタミリ秒）の**明示的注入のみ**で
//! 行われ、同一 tick 列を dispatch すれば常に同一の状態列（`phase`・
//! `elapsed_ms`・`data-state`・表示値）に到達する（決定性）。実時間の計測・
//! `setInterval` 予約はクライアント配線層（`fandhe-frontend-wasm-full` の
//! `headless_timer` モジュール、イシュー #836 後続スコープ内）の責務であり、
//! 本モジュールはその純粋な状態遷移のみを担う。
//!
//! # 設定値（countdown/start_ms/target_ms/interval_ms）を状態機械へ持たせる理由
//!
//! [`crate::clipboard::Clipboard`] の `value` は状態機械に持たせず呼び出し側が
//! 都度渡す描画パラメータとしたが、Timer は tick 適用時に完了条件
//! （カウントダウンなら `elapsed_ms >= start_ms`、カウントアップなら
//! `target_ms > 0 && elapsed_ms >= target_ms`）を状態機械内部で判定する必要が
//! あるため、設定値も [`Timer`] のフィールドとして保持し、
//! [`Hydrate::hydration_attrs`]/[`Hydrate::from_hydration_attrs`] でも
//! 往復させる（[`Timer`] 単体で完全に自己完結した復元が可能になる）。
//!
//! # アクション名を `"timer:"` 名前空間で修飾する理由（イシュー #773 PR #816
//! Bugbot 指摘の一般化）
//!
//! `fandhe-frontend-wasm-full` の `Runtime<C>` はマウントされたページのルート
//! 状態機械 `C` の型に関わらず、Timer の trigger 配線を無条件に行う想定
//! （[`crate::clipboard`] モジュール doc 同名節参照）。裸の `"start"`/`"reset"`
//! は他コンポーネント・独自 `AppState` の既存アクション名と衝突しうるため、
//! `"timer:"` 接頭辞で一意な名前空間を確保する。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`area`]/[`item`]/[`item_value`]/
//! [`item_label`]/[`separator`]/[`control`]/[`action_trigger`]、純粋関数で
//! 完結）を直接呼んで組み立てる。CSR/hydration は [`Timer`] を経由し、
//! dispatch（`"timer:start"`/`"timer:pause"`/`"timer:resume"`/`"timer:reset"`/
//! `"timer:tick"`）で状態遷移する。`fandhe-frontend-pre-styled-ui` が本モジュール
//! を呼んでスタイル済み Timer を組み立てる想定である。
//!
//! # ARIA について（イシュー #1632 是正）
//!
//! 当初「ark-ui / Zag.js の Timer は専用の WAI-ARIA パターンを持たない」と
//! 記述していたが、一次ソース（zag.js `packages/machines/timer/src/timer.connect.ts`）
//! を再確認した結果、[`area`] に `role="timer"`・`aria-atomic="true"`・
//! `aria-label`（既定書式、[`Timer::area_label`]）が、[`separator`] に
//! `aria-hidden="true"` がそれぞれ付与されることが判明したため是正した
//! （[`mod@clipboard`] の判断は他パーツについて変わらず有効）。
//!
//! # is_hidden_in / action_trigger の可視性（イシュー #1632）
//!
//! zag.js は [`TimerControl`] 各値の可視性を状態から導出する真偽式を持つ
//! （[`TimerControl::is_hidden_in`] が同じ式をそのまま実装する）。
//! [`action_trigger`] はこの結果に応じ `hidden` 属性を無条件付与する。
//! [`TimerControl::Restart`]（常に可視）は一次ソース
//! （zag.js `packages/machines/timer/src/timer.connect.ts` の
//! `validActions` 集合・`restart: () => false`）に実在する 5 番目の
//! アクションであり、本実装の独自拡張ではない。是正前の実装は Restart を
//! 欠いていたため、本是正（イシュー #1632）で zag.js の 5 値に合わせて
//! 追加した。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`）はすべて `&'static str` リテラルまたは固定スロットで
//!   あり、動的値が属性名スロットへ混入する経路はない（[`crate::anatomy`]/
//!   [`crate::data_attrs`] の既存不変条件をそのまま継承する）。
//! - 数値設定（`elapsed_ms`/`start_ms`/`target_ms`/`interval_ms`）はすべて
//!   `u64` 型から整形するため、属性値スロットへ任意文字列が混入する経路は
//!   ない。
//! - 未知アクション名・`"timer:tick"` の payload パース失敗（非数値・空・
//!   桁あふれ）はすべて no-op（fail-closed、状態を変更しない）。
//! - hydration 属性（`data-hydrate-*`）はクライアント側で改ざんされうる入力
//!   として扱う。[`Timer`] の [`fandhe_frontend_interactive::Hydrate`] 実装は
//!   panic せず [`HydrateError`] を返す。
//!
//! # スコープ外
//!
//! - `setInterval` による実 tick 駆動・`navigator` 系 API 利用は
//!   `fandhe-frontend-wasm-full`（イシュー #836 後続、`headless_timer`
//!   モジュール）のスコープ。
//! - ロケール依存の表示形式（`Intl.NumberFormat` 相当）・タイムゾーン変換は
//!   非採用（本モジュールはミリ秒の加算とゼロ埋め 2 桁整形のみを提供する）。
//! - `asChild`・`ids` オプション（ark-ui 固有機能）は非採用。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_atomic, aria_hidden, role};
use crate::data_attrs::{data_countdown, data_state};
use fandhe_frontend_core::{text, Node};
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// Timer の anatomy（`data-scope="timer"`）。
const ANATOMY: Anatomy = anatomy("timer");

/// Timer が扱う 4 セグメント単位（[`item`]/[`item_value`]/[`item_label`] の
/// `data-type` 属性値の語彙を型で固定し、任意文字列を受け付けない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerUnit {
    /// 日。
    Days,
    /// 時。
    Hours,
    /// 分。
    Minutes,
    /// 秒。
    Seconds,
}

impl TimerUnit {
    /// `data-type` 属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Days => "days",
            Self::Hours => "hours",
            Self::Minutes => "minutes",
            Self::Seconds => "seconds",
        }
    }
}

/// [`action_trigger`] の `data-action` 属性値の語彙（wasm 層の allowlist
/// 変換元、`fandhe-frontend-wasm-full` の `headless_timer` モジュールが
/// この 5 値の完全一致のみを `"timer:*"` アクションへ変換する契約）。
///
/// zag.js（`timer.connect.ts` の `validActions`）は 5 値
/// （start/pause/resume/reset/restart）を持つ（イシュー #1632 で一次ソースを
/// 再確認して判明）。是正前の本実装は 4 値のみで [`Restart`](Self::Restart)
/// を欠いていたため、本是正で zag.js に合わせて追加した（zag.js 側の独自
/// 拡張ではない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerControl {
    /// 開始（Idle → Running）。
    Start,
    /// 一時停止（Running → Paused）。
    Pause,
    /// 再開（Paused → Running）。
    Resume,
    /// リセット（任意 → Idle）。
    Reset,
    /// 再開始（任意 → Running、経過をゼロへ。常に可視）。
    Restart,
}

impl TimerControl {
    /// `data-action` 属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Reset => "reset",
            Self::Restart => "restart",
        }
    }

    /// [`as_str`](Self::as_str) の逆変換（`fandhe-frontend-wasm-full` の
    /// `data-action` allowlist 同期が利用する、イシュー #1632）。未知の値は
    /// `None`（fail-closed、完全一致のみ受理）。
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "start" => Some(Self::Start),
            "pause" => Some(Self::Pause),
            "resume" => Some(Self::Resume),
            "reset" => Some(Self::Reset),
            "restart" => Some(Self::Restart),
            _ => None,
        }
    }

    /// zag.js `timer.connect.ts` の可視性真偽式をそのまま実装する純粋関数
    /// （イシュー #1632）。`phase` から `running`/`paused` の 2 述語を導出し、
    /// このコントロールを `hidden` にすべきかを返す。`Completed` は
    /// `running`/`paused` いずれでもないため `Idle` と同じ可視性になる
    /// （意図的な拡張、モジュール冒頭「is_hidden_in / action_trigger の
    /// 可視性」節参照）。[`Self::Restart`] は常に可視（`false`）。
    #[must_use]
    pub fn is_hidden_in(self, phase: TimerPhase) -> bool {
        let running = phase == TimerPhase::Running;
        let paused = phase == TimerPhase::Paused;
        match self {
            Self::Start => running || paused,
            Self::Pause => !running,
            Self::Resume => !paused,
            Self::Reset => !running && !paused,
            Self::Restart => false,
        }
    }
}

/// [`Timer`] の `data-state` 属性値（idle/running/paused/completed の 4 値）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerPhase {
    /// 未開始。
    Idle,
    /// 計測中。
    Running,
    /// 一時停止中。
    Paused,
    /// 完了（完了境界に到達しクランプ済み）。
    Completed,
}

impl TimerPhase {
    /// `data-state` 属性値文字列を返す。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Completed => "completed",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "idle" => Some(Self::Idle),
            "running" => Some(Self::Running),
            "paused" => Some(Self::Paused),
            "completed" => Some(Self::Completed),
            _ => None,
        }
    }
}

/// `ms` ミリ秒を (days, hours, minutes, seconds) へ分解する純粋関数
/// （暦・タイムゾーンに依存しない単純な整数演算のみ）。
#[must_use]
pub fn segments_from_ms(ms: u64) -> (u64, u64, u64, u64) {
    let total_seconds = ms / 1000;
    let seconds = total_seconds % 60;
    let total_minutes = total_seconds / 60;
    let minutes = total_minutes % 60;
    let total_hours = total_minutes / 60;
    let hours = total_hours % 24;
    let days = total_hours / 24;
    (days, hours, minutes, seconds)
}

/// セグメント値をゼロ埋め 2 桁（またはそれ以上、桁あふれ時は自然に広がる）
/// の文字列へ整形する（`format!` はプレーン文字列整形のみに使用し、タグ
/// 組み立てには使わない）。
#[must_use]
pub fn format_segment(value: u64) -> String {
    format!("{value:02}")
}

/// Root パーツ（`div`）。
///
/// `countdown`/`start_ms`/`target_ms`/`interval_ms`/`elapsed_ms` を
/// `data-*` としてそのまま出力する（クライアント側
/// `fandhe-frontend-wasm-full` がこれらを読み取って tick 駆動・完了判定の
/// 表示反映を行う契約、モジュール冒頭「呼び出し文脈」節参照）。
/// `target_ms` は `0` を「無期限（目標なし）」として扱う。
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn root<'a>(
    countdown: bool,
    start_ms: u64,
    target_ms: u64,
    interval_ms: u64,
    elapsed_ms: u64,
    phase: TimerPhase,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let start_ms_s = start_ms.to_string();
    let target_ms_s = target_ms.to_string();
    let interval_s = interval_ms.to_string();
    let elapsed_s = elapsed_ms.to_string();
    // `start_ms_s`/`target_ms_s`/`interval_s`/`elapsed_s` はローカル String
    // であり、関数シグネチャの `'a`（呼び出し側 attrs の生存期間）へは
    // 結び付けられない（[`crate::rating_group::item`] の `index_s` と同型の
    // 制約）。`merged` の要素型を `'a` と独立した短い有生存期間の借用として
    // 推論させ、`ANATOMY.part` 呼び出し（`el`/`render` が即座に文字列を
    // コピーする）までしか生存させないことで安全に扱う。
    let mut merged: Vec<(&str, &str)> = vec![
        data_state(phase.as_str()),
        ("data-start-ms", start_ms_s.as_str()),
        ("data-target-ms", target_ms_s.as_str()),
        ("data-interval", interval_s.as_str()),
        ("data-elapsed", elapsed_s.as_str()),
    ];
    merged.extend(data_countdown(countdown));
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// [`area`] が固定付与するキー一覧（呼び出し側 `attrs` によるなりすまし除外、
/// `crate::date_input::ROOT_RESERVED` と同型のパターン、イシュー #1632）。
/// `aria-label` は上書き可能キーとして意図的に含めない
/// （[`Timer::area`] が既定値との重複を避けて注入する）。
const AREA_RESERVED: &[&str] = &["role", "aria-atomic"];

/// 呼び出し側 `attrs` からフレームワーク固定キー（ASCII 大文字小文字無視）を
/// 除外する（`crate::date_input::drop_reserved` と同型の重複実装。
/// モジュール間の相互依存を避けるため個別に定義する）。
fn drop_reserved<'a>(
    attrs: Vec<(&'a str, &'a str)>,
    reserved: &'static [&'static str],
) -> Vec<(&'a str, &'a str)> {
    attrs
        .into_iter()
        .filter(|(k, _)| !reserved.iter().any(|r| k.eq_ignore_ascii_case(r)))
        .collect()
}

/// Area パーツ（`div`）。セグメント項目群を内包するラッパー。
///
/// zag.js `timer.connect.ts` の `getAreaProps` に合わせ `role="timer"`・
/// `aria-atomic="true"` を無条件付与する（イシュー #1632 是正）。
/// `aria-label`（既定書式）を注入したい場合は [`Timer::area`] を使う。
#[must_use]
pub fn area<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, AREA_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![role("timer"), aria_atomic(true)];
    merged.extend(attrs);
    ANATOMY.part("area", "div", merged, children)
}

/// Item パーツ（`div`）。1 セグメント単位（例: 秒）を表す。
#[must_use]
pub fn item<'a>(unit: TimerUnit, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("data-type", unit.as_str())];
    merged.extend(attrs);
    ANATOMY.part("item", "div", merged, children)
}

/// ItemValue パーツ（`span`）。セグメント値（例: "05"）を表示する。
///
/// `data-type` を [`item`] と同じ値で重複して持たせる（wasm 配線層が
/// `item` までの DOM 走査をせず `[data-scope="timer"][data-part="item-value"]
/// [data-type="..."]` で直接更新対象を特定できるようにするための冗長化。
/// `fandhe-frontend-wasm-full::headless_timer` 参照）。
#[must_use]
pub fn item_value<'a>(
    unit: TimerUnit,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("data-type", unit.as_str())];
    merged.extend(attrs);
    ANATOMY.part("item-value", "span", merged, children)
}

/// ItemLabel パーツ（`span`）。セグメント単位のラベル（例: "Seconds"）を
/// 表示する装飾用パーツ（`children` は呼び出し側が組み立てる）。
#[must_use]
pub fn item_label<'a>(
    unit: TimerUnit,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("data-type", unit.as_str())];
    merged.extend(attrs);
    ANATOMY.part("item-label", "span", merged, children)
}

/// [`separator`] が固定付与するキー一覧（イシュー #1632）。
const SEPARATOR_RESERVED: &[&str] = &["aria-hidden"];

/// Separator パーツ（`span`）。セグメント間の区切り（例: ":"）を表示する
/// 装飾用パーツ。zag.js `timer.connect.ts` の `getSeparatorProps` に合わせ
/// `aria-hidden="true"` を無条件付与する（イシュー #1632 是正。支援技術に
/// 装飾用の区切り文字を読み上げさせない）。
#[must_use]
pub fn separator<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let attrs = drop_reserved(attrs, SEPARATOR_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> = vec![aria_hidden(true)];
    merged.extend(attrs);
    ANATOMY.part("separator", "span", merged, children)
}

/// Control パーツ（`div`）。[`action_trigger`] 群を内包するラッパー。
#[must_use]
pub fn control<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("control", "div", attrs, children)
}

/// [`action_trigger`] が固定付与するキー一覧（イシュー #1632）。
const ACTION_TRIGGER_RESERVED: &[&str] = &["type", "data-action", "hidden"];

/// ActionTrigger パーツ（`button type="button"`）。
///
/// `data-action`（[`TimerControl::as_str`] の 5 値のいずれか）を付与する。
/// クライアント配線層（`fandhe-frontend-wasm-full::headless_timer`）は
/// この属性値を allowlist（完全一致）で `"timer:*"` アクション名へ変換して
/// dispatch する契約であり、任意文字列は受け付けない（型で固定済み）。
///
/// `phase`（現在の [`TimerPhase`]）から [`TimerControl::is_hidden_in`] で
/// `hidden` 属性の要否を導出する（イシュー #1632 是正、zag.js
/// `getTriggerProps` と同じ真偽式）。
#[must_use]
pub fn action_trigger<'a>(
    control_kind: TimerControl,
    phase: TimerPhase,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let attrs = drop_reserved(attrs, ACTION_TRIGGER_RESERVED);
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![("type", "button"), ("data-action", control_kind.as_str())];
    if control_kind.is_hidden_in(phase) {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("action-trigger", "button", merged, children)
}

/// Timer のアクション（WASM 境界の文字列 dispatch と
/// [`Timer::decode_action`] で接続する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerAction {
    /// 開始（Idle → Running、経過をゼロへ初期化）。
    Start,
    /// 一時停止（Running → Paused）。
    Pause,
    /// 再開（Paused → Running）。
    Resume,
    /// リセット（任意 → Idle、経過をゼロへ）。
    Reset,
    /// 再開始（任意 → Running、経過をゼロへ。[`TimerAction::Start`] と同じ
    /// 遷移、イシュー #1632 で追加）。
    Restart,
    /// 時間経過をミリ秒単位で注入する（Running のときのみ有効）。
    Tick(u64),
}

/// Timer の状態機械。
///
/// `countdown`/`start_ms`/`target_ms`/`interval_ms` を保持し、
/// [`TimerAction::Tick`] 適用時にカウントダウン/カウントアップの完了条件を
/// 内部で判定する（モジュール冒頭「設定値を状態機械へ持たせる理由」節
/// 参照）。`Default` は count-up・未開始・`interval_ms` 既定 1000ms
/// （ark-ui 既定）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timer {
    phase: TimerPhase,
    elapsed_ms: u64,
    countdown: bool,
    start_ms: u64,
    target_ms: u64,
    interval_ms: u64,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            phase: TimerPhase::Idle,
            elapsed_ms: 0,
            countdown: false,
            start_ms: 0,
            target_ms: 0,
            interval_ms: Self::DEFAULT_INTERVAL_MS,
        }
    }
}

impl Timer {
    /// ark-ui Timer の既定 `interval`（1000ms）。
    pub const DEFAULT_INTERVAL_MS: u64 = 1000;

    /// `data-hydrate-phase` 属性名のフィールド部分。
    pub const FIELD_PHASE: &'static str = "phase";
    /// `data-hydrate-elapsed` 属性名のフィールド部分。
    pub const FIELD_ELAPSED: &'static str = "elapsed";
    /// `data-hydrate-countdown` 属性名のフィールド部分。
    pub const FIELD_COUNTDOWN: &'static str = "countdown";
    /// `data-hydrate-start-ms` 属性名のフィールド部分。
    pub const FIELD_START_MS: &'static str = "start-ms";
    /// `data-hydrate-target-ms` 属性名のフィールド部分。
    pub const FIELD_TARGET_MS: &'static str = "target-ms";
    /// `data-hydrate-interval-ms` 属性名のフィールド部分。
    pub const FIELD_INTERVAL_MS: &'static str = "interval-ms";

    /// カウントダウン Timer を生成する（`target_ms` は無期限 = `0`）。
    #[must_use]
    pub fn countdown(start_ms: u64, interval_ms: u64) -> Self {
        Self {
            phase: TimerPhase::Idle,
            elapsed_ms: 0,
            countdown: true,
            start_ms,
            target_ms: 0,
            interval_ms,
        }
    }

    /// カウントアップ Timer を生成する。`target_ms` に `0` を渡すと無期限
    /// （完了条件なし）になる。
    #[must_use]
    pub fn count_up(target_ms: u64, interval_ms: u64) -> Self {
        Self {
            phase: TimerPhase::Idle,
            elapsed_ms: 0,
            countdown: false,
            start_ms: 0,
            target_ms,
            interval_ms,
        }
    }

    /// 現在の状態（`data-state` 属性値）。
    #[must_use]
    pub fn phase(&self) -> TimerPhase {
        self.phase
    }

    /// 経過ミリ秒。
    #[must_use]
    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed_ms
    }

    /// カウントダウンかどうか。
    #[must_use]
    pub fn is_countdown(&self) -> bool {
        self.countdown
    }

    /// tick 間隔（ミリ秒）。`fandhe-frontend-wasm-full` の `headless_timer`
    /// モジュールが `setInterval` の実間隔として読み取る（モジュール冒頭
    /// 「呼び出し文脈」節参照）。
    #[must_use]
    pub fn interval_ms(&self) -> u64 {
        self.interval_ms
    }

    /// 表示対象の残り/経過ミリ秒（カウントダウンなら残り、カウントアップ
    /// なら経過そのもの）を返す。
    #[must_use]
    pub fn display_ms(&self) -> u64 {
        if self.countdown {
            self.start_ms.saturating_sub(self.elapsed_ms)
        } else {
            self.elapsed_ms
        }
    }

    /// [`display_ms`](Self::display_ms) を (days, hours, minutes, seconds)
    /// へ分解する。
    #[must_use]
    pub fn display_segments(&self) -> (u64, u64, u64, u64) {
        segments_from_ms(self.display_ms())
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        root(
            self.countdown,
            self.start_ms,
            self.target_ms,
            self.interval_ms,
            self.elapsed_ms,
            self.phase,
            attrs,
            children,
        )
    }

    /// 現在の [`display_segments`](Self::display_segments) から 4 セグメント分の
    /// [`item`]（[`item_value`] にゼロ埋め整形済みテキストを埋め込んだもの）を
    /// 組み立てる利便メソッド（呼び出し側が [`item_label`]/[`separator`] を
    /// 追加で挟み込むことを想定した最小構成）。
    #[must_use]
    pub fn items(&self) -> Vec<Node> {
        let (days, hours, minutes, seconds) = self.display_segments();
        [
            (TimerUnit::Days, days),
            (TimerUnit::Hours, hours),
            (TimerUnit::Minutes, minutes),
            (TimerUnit::Seconds, seconds),
        ]
        .into_iter()
        .map(|(unit, value)| {
            item(
                unit,
                vec![],
                vec![item_value(unit, vec![], vec![text(format_segment(value))])],
            )
        })
        .collect()
    }

    /// [`area`] の既定 `aria-label` 書式（zag.js `timer.connect.ts` の既定
    /// `translations.areaLabel` に合わせた固定英語書式、イシュー #1632）。
    /// [`display_segments`](Self::display_segments) から
    /// `"{days} days {hh}:{mm}:{ss}"` を組み立てる。ロケール依存の整形は
    /// 行わない（モジュール冒頭「スコープ外」節）。
    #[must_use]
    pub fn area_label(&self) -> String {
        let (days, hours, minutes, seconds) = self.display_segments();
        format!(
            "{days} days {}:{}:{}",
            format_segment(hours),
            format_segment(minutes),
            format_segment(seconds)
        )
    }

    /// [`area`] へ現在の状態から導出した既定 `aria-label` を注入する利便
    /// メソッド（イシュー #1632）。呼び出し側 `attrs` に既に `aria-label`
    /// があれば上書きしない（[`crate::clipboard::trigger`] の
    /// `has_caller_attr` と同型の dedup 判断、fail-closed で利用者の
    /// 独自ラベルを壊さない）。
    #[must_use]
    pub fn area<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let has_caller_label = attrs
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("aria-label"));
        let label_owned = self.area_label();
        let mut merged: Vec<(&str, &str)> = Vec::new();
        if !has_caller_label {
            merged.push(("aria-label", label_owned.as_str()));
        }
        merged.extend(attrs);
        area(merged, children)
    }

    /// [`action_trigger`] へ現在の [`phase`](Self::phase) を注入する利便
    /// メソッド（イシュー #1632）。
    #[must_use]
    pub fn action_trigger<'a>(
        &self,
        control_kind: TimerControl,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        action_trigger(control_kind, self.phase, attrs, children)
    }
}

impl Component for Timer {
    type Action = TimerAction;

    fn update(&mut self, action: TimerAction) {
        match action {
            TimerAction::Start | TimerAction::Restart => {
                self.phase = TimerPhase::Running;
                self.elapsed_ms = 0;
            }
            TimerAction::Pause => {
                if self.phase == TimerPhase::Running {
                    self.phase = TimerPhase::Paused;
                }
            }
            TimerAction::Resume => {
                if self.phase == TimerPhase::Paused {
                    self.phase = TimerPhase::Running;
                }
            }
            TimerAction::Reset => {
                self.phase = TimerPhase::Idle;
                self.elapsed_ms = 0;
            }
            TimerAction::Tick(delta) => {
                // Running 以外での tick は no-op（fail-closed、モジュール冒頭
                // 「時計 API 非依存」節参照: 状態機械は時計を持たず、外部から
                // 与えられた delta のみを反映する）。
                if self.phase != TimerPhase::Running {
                    return;
                }
                self.elapsed_ms = self.elapsed_ms.saturating_add(delta);
                let completed = if self.countdown {
                    self.elapsed_ms >= self.start_ms
                } else {
                    self.target_ms > 0 && self.elapsed_ms >= self.target_ms
                };
                if completed {
                    self.phase = TimerPhase::Completed;
                    // 境界超過分をクランプし、表示値が設定範囲を超えないように
                    // する。
                    self.elapsed_ms = if self.countdown {
                        self.start_ms
                    } else {
                        self.target_ms
                    };
                }
            }
        }
    }

    /// 共通契約（`data-state`/`data-elapsed` 整合・hydration ルート）のみを
    /// 表す最小正準ビュー（root > control > action-trigger[start]、
    /// [`crate::clipboard::Clipboard::view`] と同じ位置付けであり、公開 UI
    /// としての利用は想定しない）。
    fn view(&self) -> Node {
        self.root(
            Vec::new(),
            vec![control(
                Vec::new(),
                vec![action_trigger(
                    TimerControl::Start,
                    self.phase,
                    Vec::new(),
                    Vec::new(),
                )],
            )],
        )
    }

    fn decode_action(name: &str, payload: &str) -> Option<TimerAction> {
        // アクション名は "timer:" 名前空間で修飾する（モジュール冒頭
        // 「アクション名を "timer:" 名前空間で修飾する理由」節参照）。
        match name {
            "timer:start" => Some(TimerAction::Start),
            "timer:pause" => Some(TimerAction::Pause),
            "timer:resume" => Some(TimerAction::Resume),
            "timer:reset" => Some(TimerAction::Reset),
            "timer:restart" => Some(TimerAction::Restart),
            "timer:tick" => payload.parse::<u64>().ok().map(TimerAction::Tick),
            _ => None,
        }
    }
}

impl Hydrate for Timer {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_PHASE),
                self.phase.as_str().to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_ELAPSED),
                self.elapsed_ms.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_COUNTDOWN),
                self.countdown.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_START_MS),
                self.start_ms.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_TARGET_MS),
                self.target_ms.to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_INTERVAL_MS),
                self.interval_ms.to_string(),
            ),
        ]
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let find = |field: &str| -> Result<&str, HydrateError> {
            let attr_name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            attrs
                .iter()
                .find(|(k, _)| *k == attr_name)
                .map(|(_, v)| v.as_str())
                .ok_or(HydrateError::MissingAttr(attr_name))
        };

        let phase_raw = find(Self::FIELD_PHASE)?;
        let phase = TimerPhase::from_str(phase_raw).ok_or_else(|| HydrateError::InvalidValue {
            attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_PHASE),
            reason: "expected one of idle/running/paused/completed".to_string(),
        })?;

        let parse_u64 = |field: &'static str| -> Result<u64, HydrateError> {
            let raw = find(field)?;
            raw.parse::<u64>().map_err(|_| HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{field}"),
                reason: "expected a non-negative integer".to_string(),
            })
        };

        let elapsed_ms = parse_u64(Self::FIELD_ELAPSED)?;
        let start_ms = parse_u64(Self::FIELD_START_MS)?;
        let target_ms = parse_u64(Self::FIELD_TARGET_MS)?;
        let interval_ms = parse_u64(Self::FIELD_INTERVAL_MS)?;

        let countdown_raw = find(Self::FIELD_COUNTDOWN)?;
        let countdown = match countdown_raw {
            "true" => true,
            "false" => false,
            _ => {
                return Err(HydrateError::InvalidValue {
                    attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_COUNTDOWN),
                    reason: "expected \"true\" or \"false\"".to_string(),
                })
            }
        };

        Ok(Self {
            phase,
            elapsed_ms,
            countdown,
            start_ms,
            target_ms,
            interval_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- 各パーツの data-scope/data-part 出力 ---

    #[test]
    fn root_outputs_scope_part_and_config_attrs() {
        let html = render(&root(
            true,
            10_000,
            0,
            500,
            0,
            TimerPhase::Idle,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="timer""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="idle""#));
        assert!(html.contains(r#"data-start-ms="10000""#));
        assert!(html.contains(r#"data-target-ms="0""#));
        assert!(html.contains(r#"data-interval="500""#));
        assert!(html.contains(r#"data-elapsed="0""#));
        assert!(html.contains(r#"data-countdown="""#));
    }

    #[test]
    fn root_omits_data_countdown_when_count_up() {
        let html = render(&root(
            false,
            0,
            5000,
            1000,
            0,
            TimerPhase::Idle,
            vec![],
            vec![],
        ));
        assert!(!html.contains("data-countdown"));
    }

    #[test]
    fn area_outputs_scope_and_part() {
        let html = render(&area(vec![], vec![]));
        assert!(html.contains(r#"data-scope="timer""#));
        assert!(html.contains(r#"data-part="area""#));
    }

    #[test]
    fn item_outputs_scope_part_and_data_type() {
        let html = render(&item(TimerUnit::Seconds, vec![], vec![]));
        assert!(html.contains(r#"data-scope="timer""#));
        assert!(html.contains(r#"data-part="item""#));
        assert!(html.contains(r#"data-type="seconds""#));
    }

    #[test]
    fn item_value_outputs_scope_part_data_type_and_text() {
        let html = render(&item_value(TimerUnit::Minutes, vec![], vec![text("05")]));
        assert!(html.contains(r#"data-part="item-value""#));
        assert!(html.contains(r#"data-type="minutes""#));
        assert!(html.contains("05"));
    }

    #[test]
    fn item_label_outputs_scope_part_and_data_type() {
        let html = render(&item_label(TimerUnit::Hours, vec![], vec![text("Hours")]));
        assert!(html.contains(r#"data-part="item-label""#));
        assert!(html.contains(r#"data-type="hours""#));
        assert!(html.contains("Hours"));
    }

    #[test]
    fn separator_outputs_scope_and_part() {
        let html = render(&separator(vec![], vec![text(":")]));
        assert!(html.contains(r#"data-part="separator""#));
        assert!(html.contains(":"));
    }

    #[test]
    fn control_outputs_scope_and_part() {
        let html = render(&control(vec![], vec![]));
        assert!(html.contains(r#"data-part="control""#));
    }

    #[test]
    fn action_trigger_outputs_type_button_and_data_action() {
        let html = render(&action_trigger(
            TimerControl::Start,
            TimerPhase::Idle,
            vec![],
            vec![text("Start")],
        ));
        assert!(html.contains(r#"data-part="action-trigger""#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"data-action="start""#));
        assert!(html.contains("Start"));
    }

    #[test]
    fn action_trigger_data_action_covers_all_five_controls() {
        for (kind, expected) in [
            (TimerControl::Start, "start"),
            (TimerControl::Pause, "pause"),
            (TimerControl::Resume, "resume"),
            (TimerControl::Reset, "reset"),
            (TimerControl::Restart, "restart"),
        ] {
            let html = render(&action_trigger(kind, TimerPhase::Idle, vec![], vec![]));
            assert!(html.contains(&format!(r#"data-action="{expected}""#)));
        }
    }

    // --- イシュー #1632: area の role/aria-atomic/aria-label ---

    #[test]
    fn area_outputs_role_timer_and_aria_atomic() {
        let html = render(&area(vec![], vec![]));
        assert!(html.contains(r#"role="timer""#));
        assert!(html.contains(r#"aria-atomic="true""#));
    }

    #[test]
    fn area_caller_role_and_aria_atomic_cannot_be_spoofed() {
        let html = render(&area(
            vec![("role", "attacker"), ("aria-atomic", "false")],
            vec![],
        ));
        assert!(html.contains(r#"role="timer""#));
        assert!(html.contains(r#"aria-atomic="true""#));
        assert!(!html.contains("attacker"));
    }

    #[test]
    fn timer_area_label_uses_zag_default_format() {
        let mut t = Timer::countdown(93_784_000, 1000);
        dispatch(&mut t, "timer:start", "");
        assert_eq!(t.area_label(), "1 days 02:03:04");
    }

    #[test]
    fn timer_area_injects_default_aria_label() {
        let t = Timer::default();
        let html = render(&t.area(vec![], vec![]));
        assert!(html.contains(&format!(r#"aria-label="{}""#, t.area_label())));
    }

    #[test]
    fn timer_area_caller_aria_label_overrides_default_without_duplication() {
        let t = Timer::default();
        let html = render(&t.area(vec![("aria-label", "Custom label")], vec![]));
        assert!(html.contains(r#"aria-label="Custom label""#));
        assert_eq!(html.matches("aria-label=").count(), 1);
    }

    // --- イシュー #1632: separator の aria-hidden ---

    #[test]
    fn separator_outputs_aria_hidden() {
        let html = render(&separator(vec![], vec![text(":")]));
        assert!(html.contains(r#"aria-hidden="true""#));
    }

    #[test]
    fn separator_caller_aria_hidden_cannot_be_spoofed() {
        let html = render(&separator(vec![("aria-hidden", "false")], vec![]));
        assert!(html.contains(r#"aria-hidden="true""#));
    }

    // --- イシュー #1632: action_trigger の hidden 導出（5 action × 4 phase） ---

    #[test]
    fn action_trigger_hidden_matrix_matches_zag() {
        use TimerControl::{Pause, Reset, Restart, Resume, Start};
        use TimerPhase::{Completed, Idle, Paused, Running};

        // (control, phase) -> hidden であるべきか。zag.js
        // `timer.connect.ts` の真偽式（running/paused の 2 述語）を手で
        // 展開した期待値表。Completed は Idle と同じ可視性（意図的拡張）。
        let cases = [
            (Start, Idle, false),
            (Start, Running, true),
            (Start, Paused, true),
            (Start, Completed, false),
            (Pause, Idle, true),
            (Pause, Running, false),
            (Pause, Paused, true),
            (Pause, Completed, true),
            (Resume, Idle, true),
            (Resume, Running, true),
            (Resume, Paused, false),
            (Resume, Completed, true),
            (Reset, Idle, true),
            (Reset, Running, false),
            (Reset, Paused, false),
            (Reset, Completed, true),
            (Restart, Idle, false),
            (Restart, Running, false),
            (Restart, Paused, false),
            (Restart, Completed, false),
        ];
        for (control, phase, expect_hidden) in cases {
            assert_eq!(
                control.is_hidden_in(phase),
                expect_hidden,
                "{control:?} in {phase:?}"
            );
            let html = render(&action_trigger(control, phase, vec![], vec![]));
            assert_eq!(
                html.contains("hidden"),
                expect_hidden,
                "rendered hidden mismatch for {control:?} in {phase:?}"
            );
        }
    }

    #[test]
    fn action_trigger_caller_hidden_type_and_data_action_cannot_be_spoofed() {
        let html = render(&action_trigger(
            TimerControl::Pause,
            TimerPhase::Running,
            vec![
                ("hidden", ""),
                ("type", "submit"),
                ("data-action", "attacker"),
            ],
            vec![],
        ));
        // Running 時の Pause は可視のはずだが、呼び出し側の `hidden` 偽装は
        // 除外されるため出力に `hidden` は含まれない。
        assert!(!html.contains("hidden"));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"data-action="pause""#));
        assert!(!html.contains("attacker"));
    }

    // --- イシュー #1632: TimerControl::Restart / TimerAction::Restart ---

    #[test]
    fn restart_transitions_any_phase_to_running_and_resets_elapsed() {
        // running 状態から restart できることを確認する。
        let mut running = Timer::count_up(0, 1000);
        dispatch(&mut running, "timer:start", "");
        dispatch(&mut running, "timer:tick", "500");
        assert!(dispatch(&mut running, "timer:restart", ""));
        assert_eq!(running.phase(), TimerPhase::Running);
        assert_eq!(running.elapsed_ms(), 0);

        // paused 状態からも restart できることを確認する。
        let mut paused = Timer::count_up(0, 1000);
        dispatch(&mut paused, "timer:start", "");
        dispatch(&mut paused, "timer:tick", "500");
        dispatch(&mut paused, "timer:pause", "");
        assert!(dispatch(&mut paused, "timer:restart", ""));
        assert_eq!(paused.phase(), TimerPhase::Running);
        assert_eq!(paused.elapsed_ms(), 0);
    }

    #[test]
    fn restart_from_idle_transitions_to_running() {
        let mut t = Timer::count_up(0, 1000);
        assert!(dispatch(&mut t, "timer:restart", ""));
        assert_eq!(t.phase(), TimerPhase::Running);
        assert_eq!(t.elapsed_ms(), 0);
    }

    #[test]
    fn decode_action_accepts_timer_restart() {
        assert_eq!(
            <Timer as Component>::decode_action("timer:restart", ""),
            Some(TimerAction::Restart)
        );
    }

    #[test]
    fn timer_control_from_str_round_trips_as_str() {
        for kind in [
            TimerControl::Start,
            TimerControl::Pause,
            TimerControl::Resume,
            TimerControl::Reset,
            TimerControl::Restart,
        ] {
            assert_eq!(TimerControl::parse(kind.as_str()), Some(kind));
        }
        assert_eq!(TimerControl::parse("bogus"), None);
        assert_eq!(TimerControl::parse("Start"), None); // 大文字小文字の完全一致のみ
    }

    // --- イシュー #1632: item は style= を出力しない（§3.25 規則 2 非採用） ---

    #[test]
    fn item_and_item_value_do_not_output_style_attribute() {
        let html = render(&item(
            TimerUnit::Seconds,
            vec![],
            vec![item_value(TimerUnit::Seconds, vec![], vec![text("05")])],
        ));
        assert!(!html.contains("style="));
    }

    // --- segments_from_ms / format_segment ---

    #[test]
    fn segments_from_ms_decomposes_correctly() {
        // 1 日 2 時間 3 分 4 秒 = ((1*24+2)*60+3)*60+4 = 93784 秒
        let ms = 93_784_000;
        assert_eq!(segments_from_ms(ms), (1, 2, 3, 4));
    }

    #[test]
    fn segments_from_ms_zero_is_all_zero() {
        assert_eq!(segments_from_ms(0), (0, 0, 0, 0));
    }

    #[test]
    fn format_segment_zero_pads_to_two_digits() {
        assert_eq!(format_segment(5), "05");
        assert_eq!(format_segment(42), "42");
        assert_eq!(format_segment(100), "100");
    }

    // --- 状態機械: 遷移網羅 ---

    #[test]
    fn default_timer_is_idle_count_up_with_default_interval() {
        let t = Timer::default();
        assert_eq!(t.phase(), TimerPhase::Idle);
        assert_eq!(t.elapsed_ms(), 0);
        assert!(!t.is_countdown());
        assert_eq!(t.interval_ms, Timer::DEFAULT_INTERVAL_MS);
    }

    #[test]
    fn start_transitions_idle_to_running_and_resets_elapsed() {
        let mut t = Timer::count_up(0, 1000);
        assert!(dispatch(&mut t, "timer:start", ""));
        assert_eq!(t.phase(), TimerPhase::Running);
        assert_eq!(t.elapsed_ms(), 0);
    }

    #[test]
    fn pause_only_transitions_from_running() {
        let mut t = Timer::count_up(0, 1000);
        assert!(dispatch(&mut t, "timer:pause", ""));
        assert_eq!(t.phase(), TimerPhase::Idle); // no-op: not running

        dispatch(&mut t, "timer:start", "");
        assert!(dispatch(&mut t, "timer:pause", ""));
        assert_eq!(t.phase(), TimerPhase::Paused);
    }

    #[test]
    fn resume_only_transitions_from_paused() {
        let mut t = Timer::count_up(0, 1000);
        assert!(dispatch(&mut t, "timer:resume", ""));
        assert_eq!(t.phase(), TimerPhase::Idle); // no-op: not paused

        dispatch(&mut t, "timer:start", "");
        dispatch(&mut t, "timer:pause", "");
        assert!(dispatch(&mut t, "timer:resume", ""));
        assert_eq!(t.phase(), TimerPhase::Running);
    }

    #[test]
    fn reset_returns_to_idle_from_any_phase() {
        let mut t = Timer::count_up(0, 1000);
        dispatch(&mut t, "timer:start", "");
        dispatch(&mut t, "timer:tick", "500");
        assert!(dispatch(&mut t, "timer:reset", ""));
        assert_eq!(t.phase(), TimerPhase::Idle);
        assert_eq!(t.elapsed_ms(), 0);
    }

    #[test]
    fn tick_is_noop_when_not_running() {
        // `dispatch` の戻り値は「アクション名/payload の decode に成功したか」
        // （`Component::decode_action` が `Some` を返したか）を表すのみで
        // あり、Running 以外での no-op 判定は `Timer::update` 内部（状態が
        // 変化しないこと）で検証する（`fandhe_frontend_interactive::dispatch`
        // の契約、`crates/interactive/src/lib.rs` 参照）。
        let mut t = Timer::count_up(0, 1000);
        assert!(dispatch(&mut t, "timer:tick", "100"));
        assert_eq!(t.elapsed_ms(), 0);
        assert_eq!(t.phase(), TimerPhase::Idle);

        dispatch(&mut t, "timer:start", "");
        dispatch(&mut t, "timer:pause", "");
        assert!(dispatch(&mut t, "timer:tick", "100"));
        assert_eq!(t.elapsed_ms(), 0);
    }

    #[test]
    fn tick_accumulates_elapsed_while_running() {
        let mut t = Timer::count_up(0, 1000);
        dispatch(&mut t, "timer:start", "");
        assert!(dispatch(&mut t, "timer:tick", "100"));
        assert!(dispatch(&mut t, "timer:tick", "250"));
        assert_eq!(t.elapsed_ms(), 350);
        assert_eq!(t.phase(), TimerPhase::Running);
    }

    #[test]
    fn countdown_completes_exactly_at_boundary() {
        let mut t = Timer::countdown(1000, 1000);
        dispatch(&mut t, "timer:start", "");
        dispatch(&mut t, "timer:tick", "1000");
        assert_eq!(t.phase(), TimerPhase::Completed);
        assert_eq!(t.elapsed_ms(), 1000);
        assert_eq!(t.display_ms(), 0);
    }

    #[test]
    fn countdown_completes_and_clamps_when_overshooting() {
        let mut t = Timer::countdown(1000, 1000);
        dispatch(&mut t, "timer:start", "");
        dispatch(&mut t, "timer:tick", "1500");
        assert_eq!(t.phase(), TimerPhase::Completed);
        assert_eq!(t.elapsed_ms(), 1000);
    }

    #[test]
    fn count_up_with_target_completes_and_clamps() {
        let mut t = Timer::count_up(1000, 1000);
        dispatch(&mut t, "timer:start", "");
        dispatch(&mut t, "timer:tick", "1500");
        assert_eq!(t.phase(), TimerPhase::Completed);
        assert_eq!(t.elapsed_ms(), 1000);
    }

    #[test]
    fn count_up_without_target_never_completes() {
        let mut t = Timer::count_up(0, 1000);
        dispatch(&mut t, "timer:start", "");
        dispatch(&mut t, "timer:tick", "1000000");
        assert_eq!(t.phase(), TimerPhase::Running);
    }

    #[test]
    fn tick_saturates_on_overflow_instead_of_panicking() {
        let mut t = Timer::count_up(0, 1000);
        dispatch(&mut t, "timer:start", "");
        assert!(dispatch(&mut t, "timer:tick", &u64::MAX.to_string()));
        assert!(dispatch(&mut t, "timer:tick", &u64::MAX.to_string()));
        // count_up の target が 0（無期限）なので saturating_add 済みの
        // 最大値のままで completed へは遷移しない。
        assert_eq!(t.elapsed_ms(), u64::MAX);
    }

    // --- fail-closed: 未知アクション・payload パース失敗 ---

    #[test]
    fn decode_action_rejects_unknown_action() {
        assert!(<Timer as Component>::decode_action("no_such_action", "").is_none());
        assert!(<Timer as Component>::decode_action("start", "").is_none());
        assert!(<Timer as Component>::decode_action("reset", "").is_none());
    }

    #[test]
    fn tick_payload_non_numeric_is_noop() {
        let mut t = Timer::count_up(0, 1000);
        dispatch(&mut t, "timer:start", "");
        assert!(!dispatch(&mut t, "timer:tick", "not-a-number"));
        assert_eq!(t.elapsed_ms(), 0);
    }

    #[test]
    fn tick_payload_empty_is_noop() {
        let mut t = Timer::count_up(0, 1000);
        dispatch(&mut t, "timer:start", "");
        assert!(!dispatch(&mut t, "timer:tick", ""));
        assert_eq!(t.elapsed_ms(), 0);
    }

    #[test]
    fn tick_payload_overflowing_u64_is_noop() {
        let mut t = Timer::count_up(0, 1000);
        dispatch(&mut t, "timer:start", "");
        let overflow = "999999999999999999999999999999";
        assert!(!dispatch(&mut t, "timer:tick", overflow));
        assert_eq!(t.elapsed_ms(), 0);
    }

    // --- 決定性: 同一 tick 列 → 同一状態列 ---

    #[test]
    fn identical_tick_sequence_produces_identical_state() {
        let ticks = [100u64, 250, 650, 300];

        let mut a = Timer::countdown(2000, 100);
        let mut b = Timer::countdown(2000, 100);
        dispatch(&mut a, "timer:start", "");
        dispatch(&mut b, "timer:start", "");

        for delta in ticks {
            let payload = delta.to_string();
            let da = dispatch(&mut a, "timer:tick", &payload);
            let db = dispatch(&mut b, "timer:tick", &payload);
            assert_eq!(da, db);
            assert_eq!(a, b);
        }
    }

    // --- SSR view: hydrate 属性を出力しない ---

    #[test]
    fn ssr_default_view_has_no_hydrate_attrs() {
        let html = render(&Timer::default().view());
        assert!(!html.contains("data-hydrate"));
    }

    // --- Hydrate: 正常・異常系 ---

    #[test]
    fn ssr_and_hydration_round_trip() {
        let mut t = Timer::countdown(5000, 250);
        dispatch(&mut t, "timer:start", "");
        dispatch(&mut t, "timer:tick", "1200");

        let hydrate_html = render(&render_for_hydration(&t));
        assert!(hydrate_html.contains(r#"data-hydrate-phase="running""#));
        assert!(hydrate_html.contains(r#"data-hydrate-elapsed="1200""#));
        assert!(hydrate_html.contains(r#"data-hydrate-countdown="true""#));
        assert!(hydrate_html.contains(r#"data-hydrate-start-ms="5000""#));
        assert!(hydrate_html.contains(r#"data-hydrate-target-ms="0""#));
        assert!(hydrate_html.contains(r#"data-hydrate-interval-ms="250""#));

        let restored = Timer::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
    }

    #[test]
    fn hydration_missing_attr_is_error() {
        let result = Timer::from_hydration_attrs(&[]);
        assert!(matches!(result, Err(HydrateError::MissingAttr(_))));
    }

    #[test]
    fn hydration_invalid_phase_is_error_not_panic() {
        let attrs = vec![
            ("data-hydrate-phase".to_string(), "flying".to_string()),
            ("data-hydrate-elapsed".to_string(), "0".to_string()),
            ("data-hydrate-countdown".to_string(), "false".to_string()),
            ("data-hydrate-start-ms".to_string(), "0".to_string()),
            ("data-hydrate-target-ms".to_string(), "0".to_string()),
            ("data-hydrate-interval-ms".to_string(), "1000".to_string()),
        ];
        let result = Timer::from_hydration_attrs(&attrs);
        assert!(matches!(result, Err(HydrateError::InvalidValue { .. })));
    }

    #[test]
    fn hydration_invalid_numeric_field_is_error_not_panic() {
        let attrs = vec![
            ("data-hydrate-phase".to_string(), "idle".to_string()),
            (
                "data-hydrate-elapsed".to_string(),
                "not-a-number".to_string(),
            ),
            ("data-hydrate-countdown".to_string(), "false".to_string()),
            ("data-hydrate-start-ms".to_string(), "0".to_string()),
            ("data-hydrate-target-ms".to_string(), "0".to_string()),
            ("data-hydrate-interval-ms".to_string(), "1000".to_string()),
        ];
        let result = Timer::from_hydration_attrs(&attrs);
        assert!(matches!(result, Err(HydrateError::InvalidValue { .. })));
    }

    #[test]
    fn hydration_invalid_countdown_bool_is_error_not_panic() {
        let attrs = vec![
            ("data-hydrate-phase".to_string(), "idle".to_string()),
            ("data-hydrate-elapsed".to_string(), "0".to_string()),
            ("data-hydrate-countdown".to_string(), "maybe".to_string()),
            ("data-hydrate-start-ms".to_string(), "0".to_string()),
            ("data-hydrate-target-ms".to_string(), "0".to_string()),
            ("data-hydrate-interval-ms".to_string(), "1000".to_string()),
        ];
        let result = Timer::from_hydration_attrs(&attrs);
        assert!(matches!(result, Err(HydrateError::InvalidValue { .. })));
    }

    // --- fail-closed: 呼び出し側の data-scope/data-part 偽装は無視される ---

    #[test]
    fn caller_data_scope_and_part_spoofing_is_dropped() {
        let html = render(&root(
            false,
            0,
            0,
            1000,
            0,
            TimerPhase::Idle,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="timer""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- Timer::items 利便メソッド ---

    #[test]
    fn items_renders_four_segments_with_formatted_values() {
        let mut t = Timer::countdown(93_784_000, 1000);
        dispatch(&mut t, "timer:start", "");
        // 経過 0 → 残り 93,784,000ms = 1日2時間3分4秒。
        let items = t.items();
        assert_eq!(items.len(), 4);
        let html: String = items.iter().map(render).collect();
        assert!(html.contains(r#"data-type="days""#));
        assert!(html.contains(r#"data-type="hours""#));
        assert!(html.contains(r#"data-type="minutes""#));
        assert!(html.contains(r#"data-type="seconds""#));
        assert!(html.contains(">01<"));
        assert!(html.contains(">02<"));
        assert!(html.contains(">03<"));
        assert!(html.contains(">04<"));
    }
}
