//! FloatingPanel（ドラッグ移動・リサイズ可能な浮遊パネル）headless
//! コンポーネント（イシュー #827、`docs/design/component-coverage-map.md`
//! overlays 表の保留解除、`docs/policy/intentional-non-adoption.md` §7 参照）。
//!
//! ark-ui の FloatingPanel
//!（`.claude/skills/ark-ui/references/components/overlays/floating-panel.md`）を
//! 参考に、Root / Trigger / Positioner / Content / Header / Title / Control /
//! StageTrigger / CloseTrigger / Body の 10 anatomy パーツと、開閉・
//! stage（default/minimized/maximized）・座標を持つ状態機械 [`FloatingPanel`]
//! を提供する。
//!
//! # 既存基盤の再利用
//!
//! 1. [`crate::state::Disclosure`]（開閉）: [`popover::Popover`](crate::popover::Popover)/
//!    [`crate::dialog::Dialog`] と同じく、開閉遷移そのものは新設せず埋め込む。
//! 2. [`crate::positioning`] の CSS 変数語彙（`--fandhe-x`/`--fandhe-y`、ADR
//!    §4.4）: [`FloatingPanel::position_style`] が同じ変数名で座標を出力する
//!    （位置決めアルゴリズム自体は使わず、変数名の語彙のみを再利用する。
//!    FloatingPanel の座標は anchor 相対の placement 計算ではなく、
//!    ドラッグ操作によって決まるビューポート絶対座標のため）。
//! 3. dialog / popover の overlay 慣習: closed 時の `positioner`/`content`
//!    `hidden` 存在属性、`content` の `role="dialog"` を踏襲する。
//!
//! # `Stage` について（[`crate::state::Disclosure`]/[`crate::state::SingleSelect`] を使わない理由）
//!
//! `default`/`minimized`/`maximized` の 3 値は [`crate::state::Disclosure`]
//! の `"open"`/`"closed"` にも [`crate::state::SingleSelect`] の任意項目選択
//! にも写像できないため、[`crate::steps::Steps`]/[`crate::progress::Progress`]
//! と同じ判断で本モジュール内に独自 enum（[`Stage`]）として実装する。
//!
//! # 呼び出し文脈
//!
//! SSR は本モジュールの自由関数（[`root`]/[`trigger`]/[`positioner`]/
//! [`content`]/[`header`]/[`title`]/[`control`]/[`stage_trigger`]/
//! [`close_trigger`]/[`body`]、純粋関数で完結）を直接呼んで組み立てる。
//! CSR/hydration は [`FloatingPanel`]（[`Component`]/[`Hydrate`] 実装）を
//! 経由し、dispatch（`"open"`/`"close"`/`"toggle"`/`"minimize"`/`"maximize"`/
//! `"restore"`/`"set_position"`）で状態遷移する。`fandhe-frontend-pre-styled-ui`
//! が本モジュールを呼んでスタイル済み FloatingPanel を組み立てる想定である。
//!
//! # セキュリティ不変条件
//!
//! - 属性名（`data-*`/`aria-*`/`role`/`type`/`hidden`/`id`）はすべて
//!   `&'static str` リテラルで固定しており、動的値が属性名スロットへ混入する
//!   経路はない（[`crate::anatomy`]/[`crate::aria`]/[`crate::data_attrs`] の
//!   既存不変条件をそのまま継承する）。
//! - 動的値（`controls`/`id`/`labelledby`/呼び出し側 `attrs`/`children`
//!   テキスト/[`FloatingPanel::position_style`] の出力）は
//!   [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!   `raw_html()` は使用せず、HTML 文字列を直接組み立てない。
//! - `data-state` 値語彙（`"open"`/`"closed"`）は [`crate::state::OpenState`]
//!   に、`data-stage` 値語彙（`"default"`/`"minimized"`/`"maximized"`）は
//!   [`Stage`] に一元化し、本モジュールで独自の値を作らない。
//! - `content` は `role="dialog"` を固定付与するが `aria-modal` は出力しない
//!   （非モーダル overlay。ユーザーは他の要素を操作し続けられ、支援技術へ
//!   誤ったモーダル通知を送らない）。
//! - hydration 属性（`data-hydrate-state`/`data-hydrate-stage`/
//!   `data-hydrate-x`/`data-hydrate-y`）はクライアント側で改ざんされうる
//!   入力として扱う。[`FloatingPanel`] の [`Hydrate`] 実装は未知語彙・
//!   非有限座標をすべて拒否し、panic せず [`HydrateError`] を返す。
//! - [`FloatingPanelAction::SetPosition`] の payload（`"x,y"` 形式の文字列）は
//!   有限 `f64` としてパースできる場合のみ受理し、`NaN`/`inf`・パース不能な
//!   場合は dispatch 自体を no-op にする（[`crate::slider::Slider`] の
//!   `"set"` と同じ fail-closed 方針）。
//!
//! # スコープ外（イシュー #827）
//!
//! - ドラッグ移動・リサイズの実 DOM 配線（ark-ui の DragTrigger /
//!   ResizeTrigger 相当のポインタイベント処理）: `fandhe-frontend-wasm-full`
//!   の将来イシューのスコープ。本モジュールは [`FloatingPanelAction::SetPosition`]
//!   という到達点（型付きアクション）のみを提供する。
//! - フォーカストラップ・Escape キー閉鎖・`lazyMount`・topmost（複数パネルの
//!   重なり順）管理: クライアントランタイム側の責務であり、[`crate::dialog`]/
//!   [`crate::popover`] と同じくスコープ外。
//! - リサイズ用のハンドル anatomy（ark-ui の `resizeTrigger`）: ドラッグ同様
//!   DOM 配線が前提のため本イシューでは追加しない。

use crate::anatomy::{anatomy, Anatomy};
use crate::aria::{aria_controls, aria_expanded, aria_haspopup, aria_labelledby, role, AriaPopup};
use crate::data_attrs::data_state;
use crate::positioning::css_vars;
use crate::state::{Disclosure, DisclosureAction, OpenState};
use fandhe_frontend_core::Node;
use fandhe_frontend_interactive::{Component, Hydrate, HydrateError, HYDRATE_ATTR_PREFIX};

/// FloatingPanel の anatomy（`data-scope="floating-panel"`）。
const ANATOMY: Anatomy = anatomy("floating-panel");

/// `data-stage` 属性値 "default"（開いた直後の通常表示）。
const DATA_STAGE_DEFAULT: &str = "default";
/// `data-stage` 属性値 "minimized"（ヘッダのみ表示、[`body`] は非表示）。
const DATA_STAGE_MINIMIZED: &str = "minimized";
/// `data-stage` 属性値 "maximized"（ビューポート全面表示）。
const DATA_STAGE_MAXIMIZED: &str = "maximized";

/// `data-stage` 属性を組み立てる（[`crate::data_attrs::data_state`] の
/// stage 版。属性名 `"data-stage"` 自体は本モジュールのみが使うため、
/// [`crate::data_attrs`] へは昇格せずここへ留める）。
#[must_use]
fn data_stage(value: &str) -> (&'static str, &str) {
    ("data-stage", value)
}

/// FloatingPanel の表示段階。ark-ui の `stage` に相当する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Stage {
    /// 通常表示（トリガー・座標に基づく既定サイズ）。
    #[default]
    Default,
    /// 最小化（ヘッダのみ表示、本文は非表示）。
    Minimized,
    /// 最大化（ビューポート全面表示）。
    Maximized,
}

impl Stage {
    /// `data-stage` 属性値へ変換する（`"default"`/`"minimized"`/`"maximized"`）。
    #[must_use]
    pub const fn as_data_stage(self) -> &'static str {
        match self {
            Stage::Default => DATA_STAGE_DEFAULT,
            Stage::Minimized => DATA_STAGE_MINIMIZED,
            Stage::Maximized => DATA_STAGE_MAXIMIZED,
        }
    }

    /// `data-stage`/`data-hydrate-stage` 属性値から復元する。
    ///
    /// 未知の値（改ざん・タイポ）は `None` を返す（安全側、呼び出し元が
    /// [`HydrateError::InvalidValue`] 等へ変換する）。
    #[must_use]
    pub fn from_data_stage(s: &str) -> Option<Self> {
        match s {
            DATA_STAGE_DEFAULT => Some(Stage::Default),
            DATA_STAGE_MINIMIZED => Some(Stage::Minimized),
            DATA_STAGE_MAXIMIZED => Some(Stage::Maximized),
            _ => None,
        }
    }
}

/// 決定的な既定初期座標（x, y）。`fandhe-frontend-wasm-full` のドラッグ配線
/// 実装以前でも、SSR/初期描画が常に同じ位置へ決定的にパネルを置くための
/// 固定値（rustdoc §状態モデル参照。ユーザー環境やロケールに依存しない）。
const DEFAULT_X: f64 = 24.0;
const DEFAULT_Y: f64 = 24.0;

/// f64 数値属性値の文字列化を一元化するヘルパ（[`crate::slider`] の同名
/// ヘルパと同じ方針。モジュール間の相互依存を避けるため個別に定義する）。
fn fmt_num(value: f64) -> String {
    format!("{value}")
}

/// Root パーツ（`div`）。開閉状態・stage を `data-*` へ反映する。
#[must_use]
pub fn root<'a>(
    state: OpenState,
    stage: Stage,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        data_state(state.as_data_state()),
        data_stage(stage.as_data_stage()),
    ];
    merged.extend(attrs);
    ANATOMY.part("root", "div", merged, children)
}

/// Trigger パーツ（`button`）。
///
/// フォーム内配置時の意図しない submit を防ぐため `type="button"` を固定で
/// 付与する（[`crate::popover::trigger`] と同じ判断）。`aria-haspopup="dialog"`
/// を固定付与し、`controls` が `Some` のとき `aria-controls` で [`content`]
/// と関連付ける。`disabled` はネイティブ `disabled` 存在属性のみで表現する
/// （[`crate::popover::trigger`] の `data-disabled` 併用とは異なり、
/// FloatingPanel のトリガーは開閉トグルのみの単純な用途に絞るための最小
/// 主義）。
#[must_use]
pub fn trigger<'a>(
    state: OpenState,
    disabled: bool,
    controls: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        ("type", "button"),
        aria_haspopup(AriaPopup::Dialog),
        aria_expanded(state.is_open()),
        data_state(state.as_data_state()),
    ];
    if let Some(id) = controls {
        merged.push(aria_controls(id));
    }
    if disabled {
        merged.push(("disabled", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("trigger", "button", merged, children)
}

/// Positioner パーツ（`div`）。ドラッグ移動先の座標を `attrs` 経由の
/// `style`（[`FloatingPanel::position_style`]）で受け取るコンテナ。
/// 開閉状態・stage を `data-*` へ反映し、closed のとき `hidden` 存在属性を
/// 付与する（[`crate::popover::positioner`] と同じ判断。arrow 等の
/// ポインタ層は持たないが、closed 時に子の [`content`] を含めて SSR/no-JS
/// マークアップから隠す構造は共通）。
#[must_use]
pub fn positioner<'a>(
    state: OpenState,
    stage: Stage,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        data_state(state.as_data_state()),
        data_stage(stage.as_data_stage()),
    ];
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("positioner", "div", merged, children)
}

/// Content パーツ（`div`）。
///
/// `role="dialog"` を固定付与するが、非モーダル overlay のため `aria-modal`
/// は出力しない（モジュール doc §セキュリティ不変条件参照）。`labelledby`
/// が `Some` のとき [`title`] の `id` と対で関連付ける。closed のとき
/// `hidden` 存在属性を付与する。
#[must_use]
pub fn content<'a>(
    state: OpenState,
    stage: Stage,
    id: Option<&'a str>,
    labelledby: Option<&'a str>,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![
        role("dialog"),
        data_state(state.as_data_state()),
        data_stage(stage.as_data_stage()),
    ];
    if let Some(id) = id {
        merged.push(("id", id));
    }
    if let Some(labelledby) = labelledby {
        merged.push(aria_labelledby(labelledby));
    }
    if !state.is_open() {
        merged.push(("hidden", ""));
    }
    merged.extend(attrs);
    ANATOMY.part("content", "div", merged, children)
}

/// Header パーツ（`div`）。[`title`]/[`control`] のコンテナ（ドラッグハンドル
/// 相当の見た目は styled 層の責務、本関数は anatomy 属性のみを付与する）。
#[must_use]
pub fn header<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("header", "div", attrs, children)
}

/// Title パーツ（`h2`）。`id` が `Some` のとき [`content`] の `labelledby` と
/// 対で `aria-labelledby` 関連付けを成立させる。
#[must_use]
pub fn title<'a>(id: Option<&'a str>, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = Vec::new();
    if let Some(id) = id {
        merged.push(("id", id));
    }
    merged.extend(attrs);
    ANATOMY.part("title", "h2", merged, children)
}

/// Control パーツ（`div`）。[`stage_trigger`]/[`close_trigger`] を横並びに
/// まとめるボタン群コンテナ（anatomy 属性のみを付与する装飾用パーツ）。
#[must_use]
pub fn control<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    ANATOMY.part("control", "div", attrs, children)
}

/// StageTrigger パーツ（`button`）。`data-part="stage-trigger"`（ark-ui 準拠
/// の kebab-case）。`target` は遷移先 stage を表し `data-stage` へ反映する
/// （クリック先が何をするボタンかを CSS セレクタ・支援技術から判別可能に
/// する。実際の dispatch 配線は呼び出し側/wasm 層の責務）。[`trigger`] と
/// 同じくフォーム内配置時の意図しない submit を防ぐため `type="button"` を
/// 固定で付与する。
#[must_use]
pub fn stage_trigger<'a>(
    target: Stage,
    attrs: Vec<(&'a str, &'a str)>,
    children: Vec<Node>,
) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> =
        vec![("type", "button"), data_stage(target.as_data_stage())];
    merged.extend(attrs);
    ANATOMY.part("stage-trigger", "button", merged, children)
}

/// CloseTrigger パーツ（`button`）。`data-part="close-trigger"`（ark-ui 準拠
/// の kebab-case）。[`crate::popover::close_trigger`] と同じく `type="button"`
/// を固定で付与する。アクセシブルネームは呼び出し側の `attrs`/`children` が
/// 担う責務とする（[`crate::popover::close_trigger`] と同じ最小主義）。
#[must_use]
pub fn close_trigger<'a>(attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![("type", "button")];
    merged.extend(attrs);
    ANATOMY.part("close-trigger", "button", merged, children)
}

/// Body パーツ（`div`）。実際のパネル本文。stage を `data-*` へ反映し、
/// styled 層が `data-stage="minimized"` を折り畳み（`display: none`）の
/// フックとして使う（headless 層自体は `hidden` 等を付与しない。minimized
/// はヘッダのみを隠さない仕様であり、Popover/Dialog の closed 相当の
/// 「まるごと非表示」とは異なるため既存の `hidden` 慣習を流用しない）。
#[must_use]
pub fn body<'a>(stage: Stage, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
    let mut merged: Vec<(&'a str, &'a str)> = vec![data_stage(stage.as_data_stage())];
    merged.extend(attrs);
    ANATOMY.part("body", "div", merged, children)
}

/// [`FloatingPanelAction::SetPosition`] に対する型付きアクション。
///
/// WASM 境界の文字列 dispatch（`name`/`payload`）とは
/// [`FloatingPanel::decode_action`] で接続する。[`crate::state::Disclosure`]
/// の `"open"`/`"close"`/`"toggle"` と衝突しない名前空間
/// （`"minimize"`/`"maximize"`/`"restore"`/`"set_position"`）を使う。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloatingPanelAction {
    /// パネルを開く（[`crate::state::Disclosure`] へ委譲）。
    Open,
    /// パネルを閉じる（[`crate::state::Disclosure`] へ委譲）。stage・座標は
    /// 保持したまま閉じる（rustdoc §状態モデル「決定的往復」参照）。
    Close,
    /// 開閉を反転する（[`crate::state::Disclosure`] へ委譲）。
    Toggle,
    /// [`Stage::Minimized`] へ遷移する。
    Minimize,
    /// [`Stage::Maximized`] へ遷移する。
    Maximize,
    /// [`Stage::Default`] へ遷移する（ark-ui の `restore` 相当。任意の
    /// stage からの遷移を許容し、決定的である）。
    Restore,
    /// 座標を置き換える（ドラッグ操作の到達点。`fandhe-frontend-wasm-full`
    /// の将来イシューが dispatch する想定、モジュール doc §スコープ外
    /// 参照）。非有限値はデコード段階（[`FloatingPanel::decode_action`]）で
    /// 拒否済みであり、本バリアントには常に有限値が入る。
    SetPosition {
        /// 新しい x 座標（px 相当、ビューポート絶対値）。
        x: f64,
        /// 新しい y 座標（px 相当、ビューポート絶対値）。
        y: f64,
    },
}

/// [`state::Disclosure`](crate::state::Disclosure) + [`Stage`] + 座標
/// （`x`, `y`）を持つ FloatingPanel の状態機械。
///
/// `data-state`/`data-stage` と実際の状態の整合を型レベルで保証する入口
/// として、状態を取る各パーツ関数（[`root`]/[`trigger`]/[`positioner`]/
/// [`content`]/[`stage_trigger`]/[`body`]）へ現在状態を注入する利便メソッド
/// を提供する。状態を取らないパーツ（[`header`]/[`title`]/[`control`]/
/// [`close_trigger`]）は自由関数のみを提供し、`FloatingPanel` のメソッドと
/// しては公開しない。SSR での自由関数直接利用（本型を経由しない構成）も
/// 引き続き可能。
///
/// `Default` は closed・[`Stage::Default`]・決定的な既定初期座標
/// （[`DEFAULT_X`]/[`DEFAULT_Y`]、SSR の状態なし初期描画に対応する既定値）。
/// close しても stage・座標は保持する（決定的往復。ark-ui の FloatingPanel
/// も閉じた状態からの再オープンでレイアウトが失われない挙動に合わせる）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatingPanel {
    disclosure: Disclosure,
    stage: Stage,
    x: f64,
    y: f64,
}

impl Default for FloatingPanel {
    fn default() -> Self {
        Self {
            disclosure: Disclosure::default(),
            stage: Stage::default(),
            x: DEFAULT_X,
            y: DEFAULT_Y,
        }
    }
}

impl FloatingPanel {
    /// `data-hydrate-stage` 属性名のフィールド部分。
    pub const FIELD_STAGE: &'static str = "stage";
    /// `data-hydrate-x` 属性名のフィールド部分。
    pub const FIELD_X: &'static str = "x";
    /// `data-hydrate-y` 属性名のフィールド部分。
    pub const FIELD_Y: &'static str = "y";

    /// 指定した初期状態で FloatingPanel を生成する。
    #[must_use]
    pub fn new(initial: OpenState, stage: Stage, x: f64, y: f64) -> Self {
        Self {
            disclosure: Disclosure::new(initial),
            stage,
            x,
            y,
        }
    }

    /// 現在の開閉状態。
    #[must_use]
    pub fn state(&self) -> OpenState {
        self.disclosure.state()
    }

    /// 開いているかどうか。
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.disclosure.state().is_open()
    }

    /// 現在の表示段階。
    #[must_use]
    pub fn stage(&self) -> Stage {
        self.stage
    }

    /// 現在の x 座標。
    #[must_use]
    pub fn x(&self) -> f64 {
        self.x
    }

    /// 現在の y 座標。
    #[must_use]
    pub fn y(&self) -> f64 {
        self.y
    }

    /// 現在の座標から `--fandhe-x`/`--fandhe-y`（[`crate::positioning::css_vars`]、
    /// ADR §4.4 の既存語彙）を出力する `style` 属性値。呼び出し側が
    /// [`positioner`] の `attrs` へ `("style", &value)` として渡す契約
    /// （[`crate::positioning::css_vars_style`] と同じ出力契約、本関数自体は
    /// HTML を組み立てない）。
    #[must_use]
    pub fn position_style(&self) -> String {
        format!(
            "{}: {}px; {}: {}px;",
            css_vars::X,
            fmt_num(self.x),
            css_vars::Y,
            fmt_num(self.y),
        )
    }

    /// [`root`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn root<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        root(self.state(), self.stage(), attrs, children)
    }

    /// [`trigger`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn trigger<'a>(
        &self,
        disabled: bool,
        controls: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        trigger(self.state(), disabled, controls, attrs, children)
    }

    /// [`positioner`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn positioner<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        positioner(self.state(), self.stage(), attrs, children)
    }

    /// [`content`] へ現在の状態を注入する利便メソッド。
    #[must_use]
    pub fn content<'a>(
        &self,
        id: Option<&'a str>,
        labelledby: Option<&'a str>,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        content(self.state(), self.stage(), id, labelledby, attrs, children)
    }

    /// [`stage_trigger`] へ遷移先 stage を注入する利便メソッド。
    #[must_use]
    pub fn stage_trigger<'a>(
        &self,
        target: Stage,
        attrs: Vec<(&'a str, &'a str)>,
        children: Vec<Node>,
    ) -> Node {
        stage_trigger(target, attrs, children)
    }

    /// [`body`] へ現在の stage を注入する利便メソッド。
    #[must_use]
    pub fn body<'a>(&self, attrs: Vec<(&'a str, &'a str)>, children: Vec<Node>) -> Node {
        body(self.stage(), attrs, children)
    }
}

impl Component for FloatingPanel {
    type Action = FloatingPanelAction;

    fn update(&mut self, action: FloatingPanelAction) {
        match action {
            FloatingPanelAction::Open => self.disclosure.update(DisclosureAction::Open),
            FloatingPanelAction::Close => self.disclosure.update(DisclosureAction::Close),
            FloatingPanelAction::Toggle => self.disclosure.update(DisclosureAction::Toggle),
            FloatingPanelAction::Minimize => self.stage = Stage::Minimized,
            FloatingPanelAction::Maximize => self.stage = Stage::Maximized,
            FloatingPanelAction::Restore => self.stage = Stage::Default,
            FloatingPanelAction::SetPosition { x, y } => {
                self.x = x;
                self.y = y;
            }
        }
    }

    /// 共通契約（`data-state`/`data-stage` 整合・hydration ルート）のみを
    /// 表す最小正準ビュー（root > trigger + positioner > content、children
    /// 空・id なし）。[`crate::popover::Popover`] と同じ位置付けであり、
    /// 公開 UI としての利用は想定しない。
    fn view(&self) -> Node {
        let state = self.state();
        let stage = self.stage();
        self.root(
            Vec::new(),
            vec![
                trigger(state, false, None, Vec::new(), Vec::new()),
                positioner(
                    state,
                    stage,
                    Vec::new(),
                    vec![content(state, stage, None, None, Vec::new(), Vec::new())],
                ),
            ],
        )
    }

    /// `"open"`/`"close"`/`"toggle"` は [`Disclosure::decode_action`] へ委譲する。
    /// `"minimize"`/`"maximize"`/`"restore"` は payload 不使用。`"set_position"`
    /// は payload を `"x,y"` 形式として `f64` 2 個へパースし、両方とも有限値
    /// の場合のみ受理する（区切り文字が 1 個でない・パース不能・非有限の
    /// いずれかで `None`、dispatch は no-op。モジュール doc §セキュリティ
    /// 不変条件参照）。
    fn decode_action(name: &str, payload: &str) -> Option<FloatingPanelAction> {
        match name {
            "open" | "close" | "toggle" => {
                Disclosure::decode_action(name, payload).map(|a| match a {
                    DisclosureAction::Open => FloatingPanelAction::Open,
                    DisclosureAction::Close => FloatingPanelAction::Close,
                    DisclosureAction::Toggle => FloatingPanelAction::Toggle,
                })
            }
            "minimize" => Some(FloatingPanelAction::Minimize),
            "maximize" => Some(FloatingPanelAction::Maximize),
            "restore" => Some(FloatingPanelAction::Restore),
            "set_position" => {
                let (x_raw, y_raw) = payload.split_once(',')?;
                let x = x_raw.parse::<f64>().ok().filter(|v| v.is_finite())?;
                let y = y_raw.parse::<f64>().ok().filter(|v| v.is_finite())?;
                Some(FloatingPanelAction::SetPosition { x, y })
            }
            _ => None,
        }
    }
}

impl Hydrate for FloatingPanel {
    fn hydration_attrs(&self) -> Vec<(String, String)> {
        let mut attrs = self.disclosure.hydration_attrs();
        attrs.push((
            format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STAGE),
            self.stage.as_data_stage().to_string(),
        ));
        attrs.push((
            format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_X),
            fmt_num(self.x),
        ));
        attrs.push((
            format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_Y),
            fmt_num(self.y),
        ));
        attrs
    }

    fn from_hydration_attrs(attrs: &[(String, String)]) -> Result<Self, HydrateError> {
        let disclosure = Disclosure::from_hydration_attrs(attrs)?;

        let find = |field: &str| -> Result<&str, HydrateError> {
            let attr_name = format!("{HYDRATE_ATTR_PREFIX}{field}");
            attrs
                .iter()
                .find(|(k, _)| *k == attr_name)
                .map(|(_, v)| v.as_str())
                .ok_or(HydrateError::MissingAttr(attr_name))
        };

        let stage_raw = find(Self::FIELD_STAGE)?;
        let stage =
            Stage::from_data_stage(stage_raw).ok_or_else(|| HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_STAGE),
                reason: "expected \"default\", \"minimized\" or \"maximized\"".to_string(),
            })?;

        let x_raw = find(Self::FIELD_X)?;
        let x = x_raw
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
            .ok_or_else(|| HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_X),
                reason: "expected a finite number".to_string(),
            })?;

        let y_raw = find(Self::FIELD_Y)?;
        let y = y_raw
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
            .ok_or_else(|| HydrateError::InvalidValue {
                attr: format!("{HYDRATE_ATTR_PREFIX}{}", Self::FIELD_Y),
                reason: "expected a finite number".to_string(),
            })?;

        Ok(Self {
            disclosure,
            stage,
            x,
            y,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::{render, text};
    use fandhe_frontend_interactive::{dispatch, render_for_hydration};

    // --- Stage ---

    #[test]
    fn stage_round_trips_through_data_stage_string() {
        for stage in [Stage::Default, Stage::Minimized, Stage::Maximized] {
            assert_eq!(Stage::from_data_stage(stage.as_data_stage()), Some(stage));
        }
    }

    #[test]
    fn stage_from_data_stage_rejects_unknown_value() {
        assert_eq!(Stage::from_data_stage("DEFAULT"), None);
        assert_eq!(Stage::from_data_stage(""), None);
        assert_eq!(Stage::from_data_stage("<script>"), None);
    }

    // --- 各パーツの data-scope/data-part/data-state/data-stage 出力 ---

    #[test]
    fn root_outputs_scope_part_state_and_stage() {
        let html = render(&root(OpenState::Open, Stage::Minimized, vec![], vec![]));
        assert!(html.contains(r#"data-scope="floating-panel""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(html.contains(r#"data-state="open""#));
        assert!(html.contains(r#"data-stage="minimized""#));
    }

    #[test]
    fn trigger_has_type_button_haspopup_dialog_and_aria_expanded() {
        let html = render(&trigger(OpenState::Closed, false, None, vec![], vec![]));
        assert!(html.contains(r#"<button"#));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"aria-haspopup="dialog""#));
        assert!(html.contains(r#"aria-expanded="false""#));
        assert!(!html.contains("aria-controls"));
        assert!(!html.contains("disabled"));

        let html_open = render(&trigger(OpenState::Open, false, None, vec![], vec![]));
        assert!(html_open.contains(r#"aria-expanded="true""#));
    }

    #[test]
    fn trigger_controls_some_outputs_aria_controls() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            Some("fp-content-1"),
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"aria-controls="fp-content-1""#));
    }

    #[test]
    fn trigger_disabled_true_adds_native_disabled_attr() {
        let html = render(&trigger(OpenState::Closed, true, None, vec![], vec![]));
        assert!(html.contains(r#"disabled="""#));
    }

    #[test]
    fn positioner_closed_has_hidden_attr_open_does_not() {
        let closed = render(&positioner(
            OpenState::Closed,
            Stage::Default,
            vec![],
            vec![],
        ));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&positioner(OpenState::Open, Stage::Default, vec![], vec![]));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn positioner_accepts_position_style_via_attrs() {
        let style = "--fandhe-x: 10px; --fandhe-y: 20px;";
        let html = render(&positioner(
            OpenState::Open,
            Stage::Default,
            vec![("style", style)],
            vec![],
        ));
        assert!(html.contains("--fandhe-x:"));
        assert!(html.contains("--fandhe-y:"));
    }

    #[test]
    fn content_has_role_dialog_and_no_aria_modal() {
        let html = render(&content(
            OpenState::Open,
            Stage::Default,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(html.contains(r#"role="dialog""#));
        assert!(!html.contains("aria-modal"));
    }

    #[test]
    fn content_closed_has_hidden_attr_open_does_not() {
        let closed = render(&content(
            OpenState::Closed,
            Stage::Default,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(closed.contains(r#"hidden="""#));

        let open = render(&content(
            OpenState::Open,
            Stage::Default,
            None,
            None,
            vec![],
            vec![],
        ));
        assert!(!open.contains("hidden"));
    }

    #[test]
    fn content_labelledby_and_title_id_correspond() {
        let content_html = render(&content(
            OpenState::Open,
            Stage::Default,
            Some("fp-content"),
            Some("fp-title"),
            vec![],
            vec![],
        ));
        let title_html = render(&title(Some("fp-title"), vec![], vec![text("Panel")]));
        assert!(content_html.contains(r#"id="fp-content""#));
        assert!(content_html.contains(r#"aria-labelledby="fp-title""#));
        assert!(title_html.contains(r#"id="fp-title""#));
    }

    #[test]
    fn stage_trigger_outputs_type_button_and_target_data_stage() {
        let html = render(&stage_trigger(Stage::Maximized, vec![], vec![]));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"data-part="stage-trigger""#));
        assert!(html.contains(r#"data-stage="maximized""#));
    }

    #[test]
    fn close_trigger_has_type_button_and_kebab_case_part() {
        let html = render(&close_trigger(vec![], vec![]));
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains(r#"data-part="close-trigger""#));
    }

    #[test]
    fn body_outputs_stage_only() {
        let html = render(&body(Stage::Minimized, vec![], vec![text("body")]));
        assert!(html.contains(r#"data-part="body""#));
        assert!(html.contains(r#"data-stage="minimized""#));
    }

    // --- Anatomy::part fail-closed 回帰（呼び出し側の data-scope/data-part 偽装除去） ---

    #[test]
    fn caller_supplied_scope_and_part_are_dropped() {
        let html = render(&root(
            OpenState::Closed,
            Stage::Default,
            vec![("data-scope", "attacker"), ("data-part", "attacker")],
            vec![],
        ));
        assert!(html.contains(r#"data-scope="floating-panel""#));
        assert!(html.contains(r#"data-part="root""#));
        assert!(!html.contains("attacker"));
    }

    // --- FloatingPanel: dispatch 統合 ---

    #[test]
    fn floating_panel_default_is_closed_default_stage_and_default_position() {
        let p = FloatingPanel::default();
        assert_eq!(p.state(), OpenState::Closed);
        assert_eq!(p.stage(), Stage::Default);
        assert_eq!(p.x(), DEFAULT_X);
        assert_eq!(p.y(), DEFAULT_Y);
    }

    #[test]
    fn floating_panel_dispatch_open_close_toggle() {
        let mut p = FloatingPanel::default();
        assert!(dispatch(&mut p, "open", ""));
        assert_eq!(p.state(), OpenState::Open);
        assert!(dispatch(&mut p, "close", ""));
        assert_eq!(p.state(), OpenState::Closed);
        assert!(dispatch(&mut p, "toggle", ""));
        assert_eq!(p.state(), OpenState::Open);
    }

    #[test]
    fn floating_panel_dispatch_minimize_maximize_restore_from_any_stage() {
        let mut p = FloatingPanel::default();

        assert!(dispatch(&mut p, "minimize", ""));
        assert_eq!(p.stage(), Stage::Minimized);

        assert!(dispatch(&mut p, "maximize", ""));
        assert_eq!(p.stage(), Stage::Maximized);

        assert!(dispatch(&mut p, "restore", ""));
        assert_eq!(p.stage(), Stage::Default);

        // Minimized からの maximize 遷移も許容する（任意 stage からの遷移）。
        assert!(dispatch(&mut p, "minimize", ""));
        assert!(dispatch(&mut p, "maximize", ""));
        assert_eq!(p.stage(), Stage::Maximized);
    }

    #[test]
    fn floating_panel_close_preserves_stage_and_position() {
        let mut p = FloatingPanel::default();
        dispatch(&mut p, "open", "");
        dispatch(&mut p, "maximize", "");
        dispatch(&mut p, "set_position", "100,200");

        dispatch(&mut p, "close", "");
        assert_eq!(p.state(), OpenState::Closed);
        assert_eq!(p.stage(), Stage::Maximized);
        assert_eq!(p.x(), 100.0);
        assert_eq!(p.y(), 200.0);
    }

    #[test]
    fn floating_panel_dispatch_set_position_updates_coordinates() {
        let mut p = FloatingPanel::default();
        assert!(dispatch(&mut p, "set_position", "12.5,-3.25"));
        assert_eq!(p.x(), 12.5);
        assert_eq!(p.y(), -3.25);
    }

    #[test]
    fn floating_panel_dispatch_set_position_rejects_non_finite_and_malformed_payload() {
        let mut p = FloatingPanel::default();
        for bogus in ["nan,1", "1,inf", "1", "1,2,3", "", "a,b"] {
            assert!(!dispatch(&mut p, "set_position", bogus));
            assert_eq!(p.x(), DEFAULT_X);
            assert_eq!(p.y(), DEFAULT_Y);
        }
    }

    #[test]
    fn floating_panel_dispatch_ignores_unknown_action() {
        let mut p = FloatingPanel::new(OpenState::Open, Stage::Maximized, 1.0, 2.0);
        assert!(!dispatch(&mut p, "no_such_action", "x"));
        assert_eq!(p.state(), OpenState::Open);
        assert_eq!(p.stage(), Stage::Maximized);
    }

    // --- FloatingPanel: position_style ---

    #[test]
    fn position_style_outputs_fandhe_x_and_y_css_vars() {
        let p = FloatingPanel::new(OpenState::Open, Stage::Default, 10.0, 20.0);
        let style = p.position_style();
        assert!(style.contains("--fandhe-x: 10px;"));
        assert!(style.contains("--fandhe-y: 20px;"));
    }

    #[test]
    fn position_style_is_escaped_when_rendered_via_positioner_attrs() {
        let p = FloatingPanel::new(OpenState::Open, Stage::Default, 10.0, 20.0);
        let style = p.position_style();
        let html = render(&p.positioner(vec![("style", &style)], vec![]));
        assert!(html.contains("--fandhe-x: 10px;"));
    }

    // --- FloatingPanel: SSR 状態なし初期描画 ---

    #[test]
    fn floating_panel_default_ssr_view_has_no_hydrate_attr() {
        let rendered = render(&FloatingPanel::default().view());
        assert!(rendered.contains(r#"data-state="closed""#));
        assert!(rendered.contains(r#"data-stage="default""#));
        assert!(!rendered.contains("data-hydrate-"));
    }

    // --- FloatingPanel: hydration 経路 ---

    #[test]
    fn floating_panel_hydration_round_trip() {
        let p = FloatingPanel::new(OpenState::Open, Stage::Maximized, 42.0, -7.5);
        let rendered = render(&render_for_hydration(&p));
        assert!(rendered.contains(r#"data-hydrate-state="open""#));
        assert!(rendered.contains(r#"data-hydrate-stage="maximized""#));

        let restored = FloatingPanel::from_hydration_attrs(&p.hydration_attrs()).unwrap();
        assert_eq!(restored, p);
    }

    #[test]
    fn floating_panel_from_hydration_attrs_missing_attr_does_not_panic() {
        let err = FloatingPanel::from_hydration_attrs(&[]).unwrap_err();
        assert_eq!(
            err,
            HydrateError::MissingAttr("data-hydrate-state".to_string())
        );
    }

    #[test]
    fn floating_panel_from_hydration_attrs_invalid_stage_does_not_panic() {
        let mut attrs = Disclosure::new(OpenState::Open).hydration_attrs();
        attrs.push(("data-hydrate-stage".to_string(), "SIDEWAYS".to_string()));
        attrs.push(("data-hydrate-x".to_string(), "1".to_string()));
        attrs.push(("data-hydrate-y".to_string(), "2".to_string()));
        let err = FloatingPanel::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }

    #[test]
    fn floating_panel_from_hydration_attrs_rejects_non_finite_coordinates() {
        for bogus_pair in [("NaN", "0"), ("0", "Infinity"), ("not-a-number", "0")] {
            let mut attrs = Disclosure::new(OpenState::Open).hydration_attrs();
            attrs.push(("data-hydrate-stage".to_string(), "default".to_string()));
            attrs.push(("data-hydrate-x".to_string(), bogus_pair.0.to_string()));
            attrs.push(("data-hydrate-y".to_string(), bogus_pair.1.to_string()));
            let err = FloatingPanel::from_hydration_attrs(&attrs).unwrap_err();
            assert!(matches!(err, HydrateError::InvalidValue { .. }));
        }
    }

    // --- XSS 回帰: controls/id/labelledby/呼び出し側 attrs/children にペイロードを渡してもエスケープされる ---

    const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

    #[test]
    fn trigger_controls_payload_is_escaped_on_render() {
        let html = render(&trigger(
            OpenState::Closed,
            false,
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn content_id_and_labelledby_payload_is_escaped_on_render() {
        let html = render(&content(
            OpenState::Open,
            Stage::Default,
            Some(ATTR_BREAK_PAYLOAD),
            Some(ATTR_BREAK_PAYLOAD),
            vec![],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn caller_attrs_payload_is_escaped_on_render() {
        let html = render(&root(
            OpenState::Closed,
            Stage::Default,
            vec![("data-testid", ATTR_BREAK_PAYLOAD)],
            vec![],
        ));
        assert!(!html.contains("onmouseover=\"alert(1)"));
    }

    #[test]
    fn children_text_is_escaped_on_render() {
        let html = render(&title(
            None,
            vec![],
            vec![text("<script>alert(1)</script>")],
        ));
        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn floating_panel_xss_payload_in_hydration_stage_is_rejected_not_rendered() {
        // data-hydrate-stage はサーバーが Stage::as_data_stage() から生成する
        // 固定語彙のみを出力するため攻撃者が任意値を注入する経路はないが、
        // クライアント改ざん入力の復元経路（from_hydration_attrs）が
        // 未知値を拒否することを FloatingPanel 経由でも固定する。
        let mut attrs = Disclosure::new(OpenState::Open).hydration_attrs();
        attrs.push((
            "data-hydrate-stage".to_string(),
            "<script>alert(1)</script>".to_string(),
        ));
        attrs.push(("data-hydrate-x".to_string(), "0".to_string()));
        attrs.push(("data-hydrate-y".to_string(), "0".to_string()));
        let err = FloatingPanel::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}
