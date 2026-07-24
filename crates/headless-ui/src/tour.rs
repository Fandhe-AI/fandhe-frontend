//! Tour（オンボーディング向けステップガイド、イシュー #841、
//! `docs/design/component-coverage-map.md` の「保留」解除。保留の根拠は
//! `docs/policy/intentional-non-adoption.md` §7「装飾系」、#735）。
//!
//! ark-ui の Tour
//!（`.claude/skills/ark-ui/references/components/overlays/tour.md`）を参考に、
//! Root / Backdrop / Spotlight / Positioner / Arrow / ArrowTip / Content /
//! Title / Description / ProgressText / CloseTrigger / ActionTrigger の
//! 12 anatomy パーツと、[`fandhe_frontend_interactive::Component`]/
//! [`fandhe_frontend_interactive::Hydrate`] へ直接乗るツアー状態機械
//! [`Tour`] を提供する。
//!
//! # スコープ（本イシューが担うもの・担わないもの）
//!
//! 本モジュールが担うのは**決定的な状態機械と SSR 出力のみ**。対象要素の
//! 実座標追従（`getBoundingClientRect` 相当の計測・スクロール/リサイズ
//! 再計算・`target` セレクタの実解決）は `fandhe-frontend-wasm-full` の
//! 後続イシューへ切り出す（[`crate::positioning`] が SSR では座標計算を
//! 呼ばず静的 CSS フォールバックに留める方針、ADR §4.1、と同型の切り分け）。
//! [`TourStep::target`] は `data-target` 属性としてエスケープ済みで出力する
//! のみで、DOM 解決・`querySelector` 呼び出しは行わない。
//!
//! # `Component`/`Hydrate` を直接実装する理由（[`crate::state`] を使わない理由）
//!
//! Tour は open/closed の 2 値に加え、`skipped`/`completed` という終端状態を
//! 持つ。[`crate::steps::Steps`]/[`crate::toast::Toaster`] と同じ判断で
//! [`crate::state::Disclosure`]/[`crate::state::SingleSelect`] のいずれの
//! 既存語彙にも収まらないため、本モジュールも Phase 1（#524）が確立した
//! dispatch 契約（未知アクション no-op）・fail-closed hydration という
//! **統合様式**にのみ準拠し、
//! [`fandhe_frontend_interactive::Component`]/[`fandhe_frontend_interactive::Hydrate`]
//! を直接実装する。
//!
//! # 状態モデル
//!
//! [`Tour`] は `steps: Vec<TourStep>` と `status: `[`TourStatus`]` を持つ。
//! アクション（[`TourAction`]）による遷移は以下のとおり決定的に振る舞う:
//!
//! - `"start"`: `Idle` → `Active { step: 0 }`（`steps` が空なら直ちに
//!   `Completed` へ遷移する。fail-closed に panic しない）。`Idle` 以外からの
//!   `"start"` は no-op。
//! - `"next"`: `Active { step }` → 最終 step なら `Completed`、それ以外は
//!   `Active { step: step + 1 }`。`Active` 以外からは no-op。
//! - `"prev"`: `Active { step: 0 }` は no-op（境界）、それ以外は
//!   `Active { step: step - 1 }`。`Active` 以外からは no-op。
//! - `"skip"`: `Active { .. }` → `Skipped`。`Active` 以外（終端状態含む）
//!   からは no-op。
//! - `"complete"`: `Active { .. }` → `Completed`。`Active` 以外からは no-op。
//!
//! 終端状態（`Skipped`/`Completed`）からのいずれのアクションも no-op であり、
//! 一度終了したツアーが暗黙に再開しない（呼び出し側が明示的に新しい
//! [`Tour`] を組み立てて再開する）。
//!
//! # 呼び出し文脈
//!
//! SSR は [`Tour::new`] で組み立ててから各パーツメソッド（[`Tour::root`]/
//! [`Tour::backdrop`]/[`Tour::spotlight`]/[`Tour::positioner`]/[`Tour::arrow`]/
//! [`Tour::arrow_tip`]/[`Tour::content`]/[`Tour::title`]/[`Tour::description`]/
//! [`Tour::progress_text`]/[`Tour::close_trigger`]/[`Tour::action_trigger`]）を
//! 呼んで組み立てる。CSR/hydration は [`Tour`] を経由し、dispatch
//! （`"start"`/`"next"`/`"prev"`/`"skip"`/`"complete"`）で状態遷移する。
//! `fandhe-frontend-pre-styled-ui` が本モジュールを呼んでスタイル済み Tour
//! を組み立てる想定である。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`）はすべて `&'static str` リテラルで
//!   固定しており、動的値が属性名スロットへ混入する経路はない
//!   （[`crate::anatomy`]/[`crate::data_attrs`]/[`crate::aria`] の既存不変条件を
//!   そのまま継承する）。
//! - 動的値（`id`/`target`/`title`/`description`/呼び出し側 `attrs`/children
//!   テキスト）は [`fandhe_frontend_core::render`] の既定エスケープを必ず
//!   経由する。`raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `data-state` 値語彙（`"open"`/`"closed"`）・`data-status` 値語彙
//!   （`"idle"`/`"active"`/`"skipped"`/`"completed"`）は本モジュール内で
//!   一元管理し（[`Tour::data_state`]/[`TourStatus::as_data_status`]）、
//!   パーツ関数間で分裂させない。
//! - `placement` は [`crate::positioning::Placement`] 列挙経由のみで受け取り、
//!   任意文字列を受け付けない。SSR は座標計算を行わず
//!   [`crate::positioning::placement_attrs`] による `data-side`/`data-align`
//!   の静的出力のみを行う（ADR §4.1）。
//! - hydration 属性（`data-hydrate-*`）はクライアント側で改ざんされうる
//!   入力として扱う。[`Tour`] の [`fandhe_frontend_interactive::Hydrate`]
//!   実装は panic せず `HydrateError` を返す（5 リストの長さ不一致・
//!   `step` の範囲外・未知の `status`/`placement` 語彙をすべて拒否する）。
//!
//! # out-of-scope（イシュー #841、`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - 対象要素の実座標追従・スポットライトへの実測値注入・スクロール/リサイズ
//!   再計算・`target` セレクタの実解決とバリデーション・クリック/キーボードの
//!   実配線は `fandhe-frontend-wasm-full` の後続イシューのスコープ。
//! - ark-ui の `type`（tooltip/dialog/floating/wait 別のステップ種別）・
//!   `effect` ライフサイクル・`actions` 配列の宣言的定義は初版スコープ外
//!   （[`crate::steps`] の前例に倣う）。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_describedby, aria_labelledby, aria_live, role, AriaLive};
use crate::data_attrs::data_state;
use crate::positioning::{placement_attrs, Align, Placement, Side};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::codec::{decode_list, encode_list};
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// Tour の anatomy（`data-scope="tour"`）。
const ANATOMY: Anatomy = anatomy("tour");

/// `data-state` 属性値 "open"（`status` が `Active` のときの root/backdrop/
/// spotlight/positioner/arrow/arrow-tip/content）。
const DATA_STATE_OPEN: &str = "open";
/// `data-state` 属性値 "closed"（`Active` 以外のとき）。
const DATA_STATE_CLOSED: &str = "closed";

/// `data-status` 属性値 "idle"。
const DATA_STATUS_IDLE: &str = "idle";
/// `data-status` 属性値 "active"。
const DATA_STATUS_ACTIVE: &str = "active";
/// `data-status` 属性値 "skipped"。
const DATA_STATUS_SKIPPED: &str = "skipped";
/// `data-status` 属性値 "completed"。
const DATA_STATUS_COMPLETED: &str = "completed";

/// Tour の 1 ステップ。
///
/// `target` は対象要素の CSS セレクタ（[`Tour::spotlight`] が `data-target`
/// としてエスケープ済みで出力するのみで、実解決は行わない。本モジュール冒頭
/// 「スコープ」節参照）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TourStep {
    /// ステップ識別子（[`Tour::content`]/[`Tour::title`] の `id` 連結に
    /// 呼び出し側が使う想定。本モジュール自体はこの値を属性へ出力しない）。
    pub id: String,
    /// 対象要素の CSS セレクタ（実解決は `fandhe-frontend-wasm-full` の
    /// 後続イシュー、本モジュールは `data-target` 出力のみ）。
    pub target: Option<String>,
    /// タイトル文字列（呼び出し側が [`Tour::title`] の children へ渡す）。
    pub title: String,
    /// 説明文字列（呼び出し側が [`Tour::description`] の children へ渡す）。
    pub description: String,
    /// この step における対象要素からみた Content の配置
    /// （[`Tour::positioner`] が `data-side`/`data-align` へ変換する）。
    pub placement: Placement,
}

/// Tour の状態語彙（`data-status`）。
///
/// `Active { step }` の `step` は常に `0..steps.len()`（`steps` が空でない
/// 限り）。[`Tour::update`] 契約により、この不変条件は
/// [`fandhe_frontend_interactive::Component::update`] 経由の遷移でのみ維持
/// される（[`Tour::from_hydration_attrs`] は独立に検証する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TourStatus {
    /// 未開始（初期状態）。
    Idle,
    /// 進行中（`step` は現在表示中のステップの添字）。
    Active {
        /// 現在のステップ添字（`0..steps.len()`）。
        step: usize,
    },
    /// 利用者がスキップして終了した。
    Skipped,
    /// 全ステップを完了して終了した。
    Completed,
}

impl TourStatus {
    /// `data-status` 属性値文字列。
    fn as_data_status(self) -> &'static str {
        match self {
            Self::Idle => DATA_STATUS_IDLE,
            Self::Active { .. } => DATA_STATUS_ACTIVE,
            Self::Skipped => DATA_STATUS_SKIPPED,
            Self::Completed => DATA_STATUS_COMPLETED,
        }
    }

    /// [`Self::as_data_status`] の逆変換。`step` は `"active"` のときのみ
    /// 使用する（他の語彙では無視する）。未知の値は `None`
    /// （[`Tour::from_hydration_attrs`] が `HydrateError` へ変換する）。
    fn from_data_status(s: &str, step: usize) -> Option<Self> {
        match s {
            DATA_STATUS_IDLE => Some(Self::Idle),
            DATA_STATUS_ACTIVE => Some(Self::Active { step }),
            DATA_STATUS_SKIPPED => Some(Self::Skipped),
            DATA_STATUS_COMPLETED => Some(Self::Completed),
            _ => None,
        }
    }

    /// `Active` かどうか（root 以下の overlay パーツの開閉を決める）。
    fn is_open(self) -> bool {
        matches!(self, Self::Active { .. })
    }
}

/// [`Tour::content`] が `aria-labelledby`/`aria-describedby` を関連付けるため
/// に必要な id 群（[`crate::dialog::ContentIds`] と同型）。
#[derive(Debug, Clone, Copy, Default)]
pub struct ContentIds<'a> {
    /// `content` 自身の `id`。
    pub id: Option<&'a str>,
    /// `aria-labelledby`（[`Tour::title`] の `id` と対）。
    pub labelledby: Option<&'a str>,
    /// `aria-describedby`（[`Tour::description`] の `id` と対）。
    pub describedby: Option<&'a str>,
}

/// Tour のツアー状態機械（ark-ui 準拠）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tour {
    steps: Vec<TourStep>,
    status: TourStatus,
}

impl Default for Tour {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl Tour {
    /// `data-hydrate-ids` 属性名のフィールド部分。
    pub const FIELD_IDS: &'static str = "ids";
    /// `data-hydrate-targets` 属性名のフィールド部分。
    pub const FIELD_TARGETS: &'static str = "targets";
    /// `data-hydrate-titles` 属性名のフィールド部分。
    pub const FIELD_TITLES: &'static str = "titles";
    /// `data-hydrate-descriptions` 属性名のフィールド部分。
    pub const FIELD_DESCRIPTIONS: &'static str = "descriptions";
    /// `data-hydrate-placements` 属性名のフィールド部分。
    pub const FIELD_PLACEMENTS: &'static str = "placements";
    /// `data-hydrate-status` 属性名のフィールド部分。
    pub const FIELD_STATUS: &'static str = "status";
    /// `data-hydrate-step` 属性名のフィールド部分。
    pub const FIELD_STEP: &'static str = "step";

    /// 指定した step 一覧で [`Tour`] を組み立てる（初期状態は常に `Idle`）。
    #[must_use]
    pub fn new(steps: Vec<TourStep>) -> Self {
        Self {
            steps,
            status: TourStatus::Idle,
        }
    }

    /// 全ステップ。
    #[must_use]
    pub fn steps(&self) -> &[TourStep] {
        &self.steps
    }

    /// 現在の状態。
    #[must_use]
    pub fn status(&self) -> TourStatus {
        self.status
    }

    /// 現在表示中のステップ（`Active` のときのみ `Some`）。
    #[must_use]
    pub fn current_step(&self) -> Option<&TourStep> {
        match self.status {
            TourStatus::Active { step } => self.steps.get(step),
            _ => None,
        }
    }

    /// 現在のステップ添字（`Active` のときのみ `Some`）。
    #[must_use]
    pub fn current_index(&self) -> Option<usize> {
        match self.status {
            TourStatus::Active { step } => Some(step),
            _ => None,
        }
    }

    /// overlay パーツ共通の `data-state` 値（`Active` ⇔ `"open"`）。
    fn data_state_value(&self) -> &'static str {
        if self.status.is_open() {
            DATA_STATE_OPEN
        } else {
            DATA_STATE_CLOSED
        }
    }

    /// 非 `Active` 時に overlay パーツへ付与する `hidden` 存在属性
    /// （[`crate::dialog`] の closed 時 `hidden` と同型の契約）。
    fn hidden_attr(&self) -> Option<(&'static str, &'static str)> {
        (!self.status.is_open()).then_some(("hidden", ""))
    }

    /// Root パーツ（`div`）。
    #[must_use]
    pub fn root<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let mut merged: Vec<(&'a str, &'a str)> = vec![
            data_state(self.data_state_value()),
            ("data-status", self.status.as_data_status()),
        ];
        merged.extend(attrs);
        ANATOMY.part("root", "div", merged, children)
    }

    /// Backdrop パーツ（`div`。装飾用の全面オーバーレイ）。
    #[must_use]
    pub fn backdrop<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(self.data_state_value())];
        merged.extend(self.hidden_attr());
        merged.extend(attrs);
        ANATOMY.part("backdrop", "div", merged, children)
    }

    /// Spotlight パーツ（`div`。対象要素をくり抜くハイライト枠）。
    ///
    /// 現在ステップの [`TourStep::target`] が `Some` のとき `data-target`
    /// を付与する（エスケープ済み出力のみ、DOM 解決は行わない。本モジュール
    /// 冒頭「スコープ」節参照）。実座標（`--fandhe-tour-spotlight-*` 相当の
    /// CSS 変数）の注入は `fandhe-frontend-wasm-full` の後続イシューの責務。
    #[must_use]
    pub fn spotlight<'a>(&'a self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(self.data_state_value())];
        merged.extend(self.hidden_attr());
        if let Some(target) = self.current_step().and_then(|s| s.target.as_deref()) {
            merged.push(("data-target", target));
        }
        merged.extend(attrs);
        ANATOMY.part("spotlight", "div", merged, children)
    }

    /// Positioner パーツ（`div`）。現在ステップの [`TourStep::placement`]
    /// から `data-side`/`data-align` を静的出力する（SSR は座標計算を行わ
    /// ない、ADR §4.1）。ステップ未確定（`Idle`/終端状態）時は
    /// `Side::Bottom`/`Align::Center` にフォールバックする。
    #[must_use]
    pub fn positioner<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let placement = self
            .current_step()
            .map(|s| s.placement)
            .unwrap_or_else(|| Placement::new(Side::Bottom, Align::Center));
        let mut merged: Vec<(&'a str, &'a str)> = placement_attrs(placement).to_vec();
        merged.push(data_state(self.data_state_value()));
        merged.extend(self.hidden_attr());
        merged.extend(attrs);
        ANATOMY.part("positioner", "div", merged, children)
    }

    /// Arrow パーツ（`div`。positioner と content を指す矢印の土台）。
    #[must_use]
    pub fn arrow<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(self.data_state_value())];
        merged.extend(attrs);
        ANATOMY.part("arrow", "div", merged, children)
    }

    /// ArrowTip パーツ（`div`。矢印の先端、ark-ui の `Arrow`/`ArrowTip` 分離
    /// に合わせた 2 段構成）。
    #[must_use]
    pub fn arrow_tip<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let mut merged: Vec<(&'a str, &'a str)> = vec![data_state(self.data_state_value())];
        merged.extend(attrs);
        ANATOMY.part("arrow-tip", "div", merged, children)
    }

    /// Content パーツ（`div`）。`role="dialog"` + `ids`（[`ContentIds`]）が
    /// `Some` のときのみ `aria-labelledby`/`aria-describedby` を出力する
    /// （[`crate::dialog::content`] と同型の契約）。
    #[must_use]
    pub fn content<'a>(
        &self,
        ids: ContentIds<'a>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let mut merged: Vec<(&'a str, &'a str)> =
            vec![role("dialog"), data_state(self.data_state_value())];
        if let Some(id) = ids.id {
            merged.push(("id", id));
        }
        if let Some(labelledby) = ids.labelledby {
            merged.push(aria_labelledby(labelledby));
        }
        if let Some(describedby) = ids.describedby {
            merged.push(aria_describedby(describedby));
        }
        merged.extend(self.hidden_attr());
        merged.extend(attrs);
        ANATOMY.part("content", "div", merged, children)
    }

    /// Title パーツ（`h2`）。`id` が `Some` のとき [`Tour::content`] の
    /// `labelledby` と対にする。
    #[must_use]
    pub fn title<'a>(
        &self,
        id: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
        if let Some(id) = id {
            merged.push(("id", id));
        }
        merged.extend(attrs);
        ANATOMY.part("title", "h2", merged, children)
    }

    /// Description パーツ（`p`）。`id` が `Some` のとき [`Tour::content`] の
    /// `describedby` と対にする。
    #[must_use]
    pub fn description<'a>(
        &self,
        id: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
        if let Some(id) = id {
            merged.push(("id", id));
        }
        merged.extend(attrs);
        ANATOMY.part("description", "p", merged, children)
    }

    /// ProgressText パーツ（`div`。"Step 2 of 3" 等、呼び出し側が children
    /// へ渡す）。`aria-live="polite"` を付与し、ステップ遷移時に支援技術へ
    /// 進捗を読み上げさせる（[`crate::toast`] の `aria-live` 前例と同型）。
    #[must_use]
    pub fn progress_text<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let mut merged: Vec<(&'a str, &'a str)> = vec![aria_live(AriaLive::Polite)];
        merged.extend(attrs);
        ANATOMY.part("progress-text", "div", merged, children)
    }

    /// CloseTrigger パーツ（`button type="button"`。ツアーを閉じる。
    /// 実際の dispatch 配線は呼び出し側が担う）。
    #[must_use]
    pub fn close_trigger<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
        merged.extend(attrs);
        ANATOMY.part("close-trigger", "button", merged, children)
    }

    /// ActionTrigger パーツ（`button type="button"`。呼び出し側が定義する
    /// アクションボタン。`"prev"`/`"next"`/`"skip"`/`"complete"` いずれの
    /// dispatch にも使える汎用パーツ）。
    #[must_use]
    pub fn action_trigger<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
        merged.extend(attrs);
        ANATOMY.part("action-trigger", "button", merged, children)
    }
}

/// Tour のアクション（WASM 境界の文字列 dispatch と
/// [`Tour::decode_action`] で接続する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TourAction {
    /// ツアーを開始する（`Idle` からのみ有効）。
    Start,
    /// 次のステップへ進む（`Active` からのみ有効）。
    Next,
    /// 前のステップへ戻る（`Active` からのみ有効、`step == 0` では no-op）。
    Prev,
    /// ツアーをスキップして終了する（`Active` からのみ有効）。
    Skip,
    /// ツアーを完了として終了する（`Active` からのみ有効）。
    Complete,
}

impl Component for Tour {
    type Action = TourAction;

    /// 本モジュール冒頭「状態モデル」節の遷移表をそのまま実装する。
    /// いずれの分岐も現在状態が前提と一致しない場合は no-op のままとし、
    /// 終端状態（`Skipped`/`Completed`）からの遷移を発生させない。
    fn update(&mut self, action: TourAction) {
        match action {
            TourAction::Start => {
                if let TourStatus::Idle = self.status {
                    self.status = if self.steps.is_empty() {
                        TourStatus::Completed
                    } else {
                        TourStatus::Active { step: 0 }
                    };
                }
            }
            TourAction::Next => {
                if let TourStatus::Active { step } = self.status {
                    self.status = if step + 1 >= self.steps.len() {
                        TourStatus::Completed
                    } else {
                        TourStatus::Active { step: step + 1 }
                    };
                }
            }
            TourAction::Prev => {
                if let TourStatus::Active { step } = self.status {
                    if step > 0 {
                        self.status = TourStatus::Active { step: step - 1 };
                    }
                }
            }
            TourAction::Skip => {
                if let TourStatus::Active { .. } = self.status {
                    self.status = TourStatus::Skipped;
                }
            }
            TourAction::Complete => {
                if let TourStatus::Active { .. } = self.status {
                    self.status = TourStatus::Completed;
                }
            }
        }
    }

    /// 共通契約（`data-scope`/`data-part`・hydration ルート）のみを表す
    /// 最小正準ビュー（root > content）。[`crate::steps::Steps::view`] と
    /// 同じ位置付けであり、実際の UI 構築は §パーツメソッド群を呼び出し側
    /// が組み合わせる。
    fn view(&self) -> Node {
        self.root(
            Vec::new(),
            vec![self.content(ContentIds::default(), Vec::new(), Vec::new())],
        )
    }

    /// payload は使用しない（すべて状態のみで決まる遷移のため）。
    fn decode_action(name: &str, _payload: &str) -> Option<TourAction> {
        match name {
            "start" => Some(TourAction::Start),
            "next" => Some(TourAction::Next),
            "prev" => Some(TourAction::Prev),
            "skip" => Some(TourAction::Skip),
            "complete" => Some(TourAction::Complete),
            _ => None,
        }
    }
}

impl Hydrate for Tour {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let ids: Vec<String> = self.steps.iter().map(|s| s.id.clone()).collect();
        // `target` が `None` の場合は空文字列を番兵として使う（`encode_list`
        // はリスト長と項目内容を独立にエンコードするため、空文字列 1 件と
        // 空リストは往復で区別できる。空セレクタは意味を持たないため
        // 空文字列を "対象なし" の表現として安全に流用できる）。
        let targets: Vec<String> = self
            .steps
            .iter()
            .map(|s| s.target.clone().unwrap_or_default())
            .collect();
        let titles: Vec<String> = self.steps.iter().map(|s| s.title.clone()).collect();
        let descriptions: Vec<String> = self.steps.iter().map(|s| s.description.clone()).collect();
        let placements: Vec<String> = self
            .steps
            .iter()
            .map(|s| s.placement.as_str().to_string())
            .collect();

        vec![
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_IDS),
                encode_list(&ids),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_TARGETS),
                encode_list(&targets),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_TITLES),
                encode_list(&titles),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_DESCRIPTIONS),
                encode_list(&descriptions),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_PLACEMENTS),
                encode_list(&placements),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STATUS),
                self.status.as_data_status().to_string(),
            ),
            (
                format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STEP),
                self.current_index().unwrap_or(0).to_string(),
            ),
        ]
    }

    /// クライアント改ざん入力として扱う。欠落は
    /// [`HydrateError::MissingAttr`]、5 リストの長さ不一致・パース不能・
    /// `step` の範囲外・未知の `status`/`placement` 語彙はすべて
    /// [`HydrateError::InvalidValue`]（panic しない）。
    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let find = |field: &str| -> Result<&str, HydrateError> {
            let name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            attrs
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v.as_str())
                .ok_or(HydrateError::MissingAttr(name))
        };

        let ids = decode_list(find(Self::FIELD_IDS)?);
        let targets_raw = decode_list(find(Self::FIELD_TARGETS)?);
        let titles = decode_list(find(Self::FIELD_TITLES)?);
        let descriptions = decode_list(find(Self::FIELD_DESCRIPTIONS)?);
        let placements_raw = decode_list(find(Self::FIELD_PLACEMENTS)?);

        let len = ids.len();
        if targets_raw.len() != len
            || titles.len() != len
            || descriptions.len() != len
            || placements_raw.len() != len
        {
            return Err(HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_IDS),
                reason: "ids/targets/titles/descriptions/placements must have equal length"
                    .to_string(),
            });
        }

        let mut placements = Vec::with_capacity(len);
        for raw in &placements_raw {
            placements.push(Placement::from_str(raw).ok_or_else(|| {
                HydrateError::InvalidValue {
                    attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_PLACEMENTS),
                    reason: "unknown placement value".to_string(),
                }
            })?);
        }

        let steps: Vec<TourStep> = ids
            .into_iter()
            .zip(targets_raw)
            .zip(titles)
            .zip(descriptions)
            .zip(placements)
            .map(
                |((((id, target), title), description), placement)| TourStep {
                    id,
                    target: if target.is_empty() {
                        None
                    } else {
                        Some(target)
                    },
                    title,
                    description,
                    placement,
                },
            )
            .collect();

        let step_attr = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STEP);
        let step_raw = find(Self::FIELD_STEP)?;
        let step: usize = step_raw.parse().map_err(|_| HydrateError::InvalidValue {
            attr: step_attr.clone(),
            reason: "expected a non-negative decimal integer".to_string(),
        })?;

        let status_attr = format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STATUS);
        let status_raw = find(Self::FIELD_STATUS)?;
        let status = TourStatus::from_data_status(status_raw, step).ok_or_else(|| {
            HydrateError::InvalidValue {
                attr: status_attr,
                reason: "expected \"idle\", \"active\", \"skipped\", or \"completed\"".to_string(),
            }
        })?;

        if let TourStatus::Active { step } = status {
            if step >= steps.len() {
                return Err(HydrateError::InvalidValue {
                    attr: step_attr,
                    reason: "expected step within [0, steps.len())".to_string(),
                });
            }
        }

        Ok(Self { steps, status })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    fn step(id: &str, target: Option<&str>, title: &str, description: &str) -> TourStep {
        TourStep {
            id: id.to_string(),
            target: target.map(str::to_string),
            title: title.to_string(),
            description: description.to_string(),
            placement: Placement::new(Side::Bottom, Align::Center),
        }
    }

    fn three_steps() -> Vec<TourStep> {
        vec![
            step("s1", Some("#a"), "One", "first"),
            step("s2", Some("#b"), "Two", "second"),
            step("s3", None, "Three", "third"),
        ]
    }

    // --- 遷移の決定性 ---

    #[test]
    fn default_is_idle() {
        let t = Tour::default();
        assert_eq!(t.status(), TourStatus::Idle);
        assert_eq!(t.current_step(), None);
    }

    #[test]
    fn start_activates_first_step() {
        let mut t = Tour::new(three_steps());
        assert!(dispatch(&mut t, "start", ""));
        assert_eq!(t.status(), TourStatus::Active { step: 0 });
        assert_eq!(t.current_step().unwrap().id, "s1");
    }

    #[test]
    fn start_with_empty_steps_completes_immediately() {
        let mut t = Tour::new(Vec::new());
        assert!(dispatch(&mut t, "start", ""));
        assert_eq!(t.status(), TourStatus::Completed);
    }

    #[test]
    fn start_from_non_idle_is_no_op() {
        let mut t = Tour::new(three_steps());
        assert!(dispatch(&mut t, "start", ""));
        assert!(dispatch(&mut t, "start", ""));
        // 2 回目の "start" は Idle でないため no-op（1 回目の状態のまま）。
        assert_eq!(t.status(), TourStatus::Active { step: 0 });
    }

    #[test]
    fn next_advances_and_completes_at_last_step() {
        let mut t = Tour::new(three_steps());
        dispatch(&mut t, "start", "");
        assert!(dispatch(&mut t, "next", ""));
        assert_eq!(t.status(), TourStatus::Active { step: 1 });
        assert!(dispatch(&mut t, "next", ""));
        assert_eq!(t.status(), TourStatus::Active { step: 2 });
        assert!(dispatch(&mut t, "next", ""));
        assert_eq!(t.status(), TourStatus::Completed);
    }

    #[test]
    fn next_from_idle_is_no_op() {
        // "next" は認識されたアクションのため dispatch は true を返すが、
        // Idle から Active 以外への遷移は発生しない no-op のままである
        // （[`crate::steps::Steps`] の「decode 成功 = dispatch true」契約と同型）。
        let mut t = Tour::new(three_steps());
        assert!(dispatch(&mut t, "next", ""));
        assert_eq!(t.status(), TourStatus::Idle);
    }

    #[test]
    fn prev_retreats_and_stops_at_zero() {
        let mut t = Tour::new(three_steps());
        dispatch(&mut t, "start", "");
        dispatch(&mut t, "next", "");
        dispatch(&mut t, "next", "");
        assert!(dispatch(&mut t, "prev", ""));
        assert_eq!(t.status(), TourStatus::Active { step: 1 });
        assert!(dispatch(&mut t, "prev", ""));
        assert_eq!(t.status(), TourStatus::Active { step: 0 });
        // 境界（step == 0）: no-op のまま留まる。
        assert!(dispatch(&mut t, "prev", ""));
        assert_eq!(t.status(), TourStatus::Active { step: 0 });
    }

    #[test]
    fn skip_ends_tour_from_active_only() {
        let mut t = Tour::new(three_steps());
        dispatch(&mut t, "start", "");
        assert!(dispatch(&mut t, "skip", ""));
        assert_eq!(t.status(), TourStatus::Skipped);
        // 終端状態からの skip は no-op（認識はされるが状態は変わらない）。
        assert!(dispatch(&mut t, "skip", ""));
        assert_eq!(t.status(), TourStatus::Skipped);
    }

    #[test]
    fn complete_ends_tour_from_active_only() {
        let mut t = Tour::new(three_steps());
        dispatch(&mut t, "start", "");
        assert!(dispatch(&mut t, "complete", ""));
        assert_eq!(t.status(), TourStatus::Completed);
    }

    #[test]
    fn terminal_states_ignore_all_actions() {
        let mut t = Tour::new(three_steps());
        dispatch(&mut t, "start", "");
        dispatch(&mut t, "skip", "");
        for action in ["start", "next", "prev", "skip", "complete"] {
            // dispatch 自体は「認識されたアクション」として true を返すが、
            // 状態機械は Skipped のまま変化しない（本モジュール冒頭
            // 「状態モデル」節の終端状態契約）。
            assert!(dispatch(&mut t, action, ""));
            assert_eq!(t.status(), TourStatus::Skipped);
        }
    }

    #[test]
    fn dispatch_ignores_unknown_action() {
        let mut t = Tour::new(three_steps());
        assert!(!dispatch(&mut t, "no_such_action", "x"));
        assert_eq!(t.status(), TourStatus::Idle);
    }

    // --- anatomy / ARIA / data-* ---

    #[test]
    fn root_outputs_scope_part_state_and_status() {
        let t = Tour::new(three_steps());
        let html = render(&t.root(vec![], vec![]));
        assert!(html.contains(r#"data-scope="tour""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(html.contains(r#"data-status="idle""#));
    }

    #[test]
    fn root_reflects_active_and_terminal_status() {
        let mut t = Tour::new(three_steps());
        dispatch(&mut t, "start", "");
        let html = render(&t.root(vec![], vec![]));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains(r#"data-status="active""#));

        dispatch(&mut t, "complete", "");
        let html = render(&t.root(vec![], vec![]));
        assert!(html.contains(r#"data-state="closed""#));
        assert!(html.contains(r#"data-status="completed""#));
    }

    #[test]
    fn overlay_parts_are_hidden_when_not_open() {
        let t = Tour::new(three_steps());
        for html in [
            render(&t.backdrop(vec![], vec![])),
            render(&t.spotlight(vec![], vec![])),
            render(&t.positioner(vec![], vec![])),
            render(&t.content(ContentIds::default(), vec![], vec![])),
        ] {
            assert!(html.contains("hidden"), "{html}");
        }
    }

    #[test]
    fn overlay_parts_are_visible_when_active() {
        let mut t = Tour::new(three_steps());
        dispatch(&mut t, "start", "");
        for html in [
            render(&t.backdrop(vec![], vec![])),
            render(&t.spotlight(vec![], vec![])),
            render(&t.positioner(vec![], vec![])),
            render(&t.content(ContentIds::default(), vec![], vec![])),
        ] {
            assert!(!html.contains("hidden"), "{html}");
        }
    }

    #[test]
    fn spotlight_outputs_data_target_when_present() {
        let mut t = Tour::new(three_steps());
        dispatch(&mut t, "start", "");
        let html = render(&t.spotlight(vec![], vec![]));
        assert!(html.contains("data-target=\"#a\""));

        // 3 番目の step は target なし。
        dispatch(&mut t, "next", "");
        dispatch(&mut t, "next", "");
        let html = render(&t.spotlight(vec![], vec![]));
        assert!(!html.contains("data-target"));
    }

    #[test]
    fn positioner_outputs_current_step_placement() {
        let mut steps = three_steps();
        steps[1].placement = Placement::new(Side::Top, Align::Start);
        let mut t = Tour::new(steps);
        dispatch(&mut t, "start", "");
        dispatch(&mut t, "next", "");
        let html = render(&t.positioner(vec![], vec![]));
        assert!(html.contains(r#"data-side="top""#));
        assert!(html.contains(r#"data-align="start""#));
    }

    #[test]
    fn positioner_falls_back_to_bottom_center_when_idle() {
        let t = Tour::new(three_steps());
        let html = render(&t.positioner(vec![], vec![]));
        assert!(html.contains(r#"data-side="bottom""#));
        assert!(html.contains(r#"data-align="center""#));
    }

    #[test]
    fn content_has_role_dialog_and_aria_linking() {
        let t = Tour::new(three_steps());
        let html = render(&t.content(
            ContentIds {
                id: Some("tour-content"),
                labelledby: Some("tour-title"),
                describedby: Some("tour-desc"),
            },
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"role="dialog""#));
        assert!(html.contains(r#"id="tour-content""#));
        assert!(html.contains(r#"aria-labelledby="tour-title""#));
        assert!(html.contains(r#"aria-describedby="tour-desc""#));
    }

    #[test]
    fn progress_text_has_aria_live_polite() {
        let t = Tour::new(three_steps());
        let html = render(&t.progress_text(vec![], vec![text("Step 1 of 3")]));
        assert!(html.contains(r#"aria-live="polite""#));
        assert!(html.contains("Step 1 of 3"));
    }

    #[test]
    fn close_and_action_trigger_are_type_button() {
        let t = Tour::new(three_steps());
        assert!(render(&t.close_trigger(vec![], vec![])).contains(r#"type="button""#));
        assert!(render(&t.action_trigger(vec![], vec![])).contains(r#"type="button""#));
    }

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let t = Tour::new(three_steps());
        let html = render(&t.root(
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="tour""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- SSR 状態なし初期描画 ---

    #[test]
    fn default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&Tour::default().view());
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- hydration 経路 ---

    #[test]
    fn hydration_round_trip() {
        let mut t = Tour::new(three_steps());
        dispatch(&mut t, "start", "");
        dispatch(&mut t, "next", "");

        let rendered = render(&render_for_hydration(&t));
        assert!(rendered.contains(r#"data-hydrate-status="active""#));
        assert!(rendered.contains(r#"data-hydrate-step="1""#));

        let restored = Tour::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored, t);
    }

    #[test]
    fn hydration_round_trip_idle_and_terminal_states() {
        for status_action in [None, Some("skip"), Some("complete")] {
            let mut t = Tour::new(three_steps());
            if let Some(action) = status_action {
                dispatch(&mut t, "start", "");
                dispatch(&mut t, action, "");
            }
            let restored = Tour::from_hydration_attrs(&t.hydration_attrs()).unwrap();
            assert_eq!(restored, t);
        }
    }

    #[test]
    fn hydration_round_trip_target_none_survives() {
        let mut t = Tour::new(three_steps());
        dispatch(&mut t, "start", "");
        dispatch(&mut t, "next", "");
        dispatch(&mut t, "next", "");
        assert_eq!(t.current_step().unwrap().target, None);

        let restored = Tour::from_hydration_attrs(&t.hydration_attrs()).unwrap();
        assert_eq!(restored.steps()[2].target, None);
    }

    #[test]
    fn from_hydration_attrs_missing_attr_does_not_panic() {
        let err = Tour::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-ids".to_string())
        );
    }

    fn full_attrs(overrides: &[(&str, &str)]) -> Vec<(String, String)> {
        let mut base: Vec<(String, String)> = vec![
            (
                "data-hydrate-ids".to_string(),
                "\u{1f}s1\u{1f}s2".to_string(),
            ),
            (
                "data-hydrate-targets".to_string(),
                "\u{1f}#a\u{1f}".to_string(),
            ),
            (
                "data-hydrate-titles".to_string(),
                "\u{1f}One\u{1f}Two".to_string(),
            ),
            (
                "data-hydrate-descriptions".to_string(),
                "\u{1f}first\u{1f}second".to_string(),
            ),
            (
                "data-hydrate-placements".to_string(),
                "\u{1f}bottom\u{1f}bottom".to_string(),
            ),
            ("data-hydrate-status".to_string(), "active".to_string()),
            ("data-hydrate-step".to_string(), "0".to_string()),
        ];
        for (key, value) in overrides {
            if let Some(entry) = base.iter_mut().find(|(k, _)| k == key) {
                entry.1 = value.to_string();
            }
        }
        base
    }

    #[test]
    fn from_hydration_attrs_invalid_value_does_not_panic() {
        let bogus_cases: Vec<Vec<(String, String)>> = vec![
            // status が未知の語彙。
            full_attrs(&[("data-hydrate-status", "diagonal")]),
            // placements に未知の語彙。
            full_attrs(&[("data-hydrate-placements", "\u{1f}bottom\u{1f}sideways")]),
            // step が active に対して範囲外（steps.len() == 2）。
            full_attrs(&[("data-hydrate-step", "5")]),
            // step が非数値。
            full_attrs(&[("data-hydrate-step", "abc")]),
            // step が XSS ペイロード。
            full_attrs(&[("data-hydrate-step", "<script>alert(1)</script>")]),
        ];
        for attrs in bogus_cases {
            let err = Tour::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    #[test]
    fn from_hydration_attrs_length_mismatch_is_rejected() {
        let mut attrs = full_attrs(&[]);
        // titles だけ 1 件少ないリストへ改ざんする。
        for (k, v) in attrs.iter_mut() {
            if k == "data-hydrate-titles" {
                *v = "\u{1f}One".to_string();
            }
        }
        let err = Tour::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    // --- XSS 回帰: 呼び出し側 attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let t = Tour::new(three_steps());
        let html = render(&t.root(vec![("data-testid", ATTR_BREAK_PAYLOAD)], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let t = Tour::new(three_steps());
        let html = render(&t.title(None, vec![], vec![text("<script>alert(1)</script>")]));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn target_payload_is_escaped_on_render() {
        let mut steps = three_steps();
        steps[0].target = Some("\" onmouseover=\"alert(1)".to_string());
        let mut t = Tour::new(steps);
        dispatch(&mut t, "start", "");
        let html = render(&t.spotlight(vec![], vec![]));
        assert!(!html.contains("onmouseover=\"alert(1)\""));
    }
}
