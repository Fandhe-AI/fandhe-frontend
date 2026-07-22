//! `fandhe-frontend-headless-ui`: headless UI コンポーネント層（外部依存は
//! `fandhe-frontend-core` / `fandhe-frontend-interactive`（いずれも path）のみ）。
//!
//! ark-ui 相当の headless（unstyled）UI コンポーネント層を提供する。
//! anatomy（部品構成）・`data-*` 属性・WAI-ARIA 属性付与のための共通 API を
//! 本クレートで整備し、その上に styled 層（`fandhe-frontend-pre-styled-ui`、
//! イシュー #546）が重なる 2 層構造の下層を担う（親トラッキング #520、
//! Phase 1 親 #521）。
//!
//! # 本クレートの不変条件（REQ-1・REQ-2・REQ-5、`.claude/rules/coding-rust.md`）
//!
//! 1. コンポーネントは [`fandhe_frontend_core::Node`] を返す通常の Rust 関数として
//!    実装する（REQ-5、マクロ DSL は採用しない）。
//! 2. 出力は [`fandhe_frontend_core::render`] の既定エスケープを必ず経由する。
//!    **本クレート内では `raw_html()` を使用しない**（新たなエスケープ迂回経路を
//!    作らない）。
//! 3. **`unsafe` コード禁止**: `#![forbid(unsafe_code)]` によりクレート全体で
//!    機械的に禁止する（`crates/core/tests/unsafe_boundary.rs` が workspace
//!    member を自動発見して強制する）。
//! 4. **外部依存は `fandhe-frontend-core` / `fandhe-frontend-interactive`
//!    （いずれも path）のみ**: `headless-ui/Cargo.toml` の `[dependencies]` に
//!    サードパーティクレートを追加しない。
//!
//! # 実装済み API（イシュー #523/#524）
//!
//! - [`mod@anatomy`]: `data-scope` / `data-part` を付与してパーツノードを組み立てる
//!   [`anatomy::Anatomy`]（全コンポーネント共通の anatomy 基盤）。
//! - [`data_attrs`]: `data-state` / `data-disabled` 等の状態属性ヘルパ（#523）。
//! - [`aria`]: `role` / `aria-*` の WAI-ARIA 属性ヘルパ（#523）。
//! - [`state`]: `fandhe-frontend-interactive` の
//!   [`fandhe_frontend_interactive::Component`]/[`fandhe_frontend_interactive::Hydrate`]
//!   抽象へ乗る開閉系状態機械（[`state::Disclosure`]/[`state::SingleSelect`]/
//!   [`state::MultiSelect`]、#524/#594）。
//!   Dialog / Accordion / Tabs / Collapsible / Popover / Tooltip（Phase 2 の
//!   #526〜#533）が共通で使う「open/closed・selected」の dispatch 契約・
//!   `data-state` 整合・SSR/hydration 契約をここに一度だけ実装し、各コンポーネントは
//!   フィールドとして埋め込んで再利用する。[`state::MultiSelect`]（#594）は
//!   0 個以上の同時選択（[`accordion::MultiAccordion`] の multiple モード）
//!   向けに [`state::SingleSelect`]（高々 1 個選択）を補完する。
//! - [`mod@tabs`]: WAI-ARIA APG の Tabs パターンに準拠したマークアップを組み立てる
//!   [`tabs::tabs`]（#528）。SSR 時点の静的な選択状態のみを扱い、クリック操作・
//!   状態機械連携は後続イシューのスコープ。[`tabs::TabsProps`] の
//!   `activation_mode`/`loop_focus`（イシュー #582）は `list` パーツへ
//!   `data-activation-mode`/`data-loop-focus` として出力され、
//!   `fandhe-frontend-wasm-full` の `keynav` モジュールがキーボード操作時の
//!   活性化タイミング・フォーカス循環を分岐するために読む契約。
//!   [`tabs::TabsProps::indicator`]（イシュー #601、既定 `false` の opt-in）
//!   は選択タブの位置を示す `indicator` パーツを追加し、SSR では
//!   `data-*` フックと CSS 変数（`--left`/`--top`/`--width`/`--height`）の
//!   初期値のみを出力する（動的計測は wasm/CSR 層の後続責務）。
//! - [`mod@collapsible`]: Root/Trigger/Indicator/Content の anatomy パーツ関数群と、
//!   [`state::Disclosure`] を埋め込んだ [`collapsible::Collapsible`] 状態機械
//!   （#529、親 #526）。Phase 2 で [`state`] を具象コンポーネントへ適用する最初の例。
//! - [`mod@accordion`]: Root / Item / ItemTrigger / ItemIndicator / ItemContent の
//!   5 anatomy パーツと [`state::SingleSelect`] を埋め込んだ single モード
//!   Accordion（[`accordion::Accordion`]、#527）、および
//!   [`state::MultiSelect`] を埋め込んだ multiple モード Accordion
//!   （[`accordion::MultiAccordion`]、#594）。
//! - [`mod@tooltip`]: Root/Trigger/Positioner/Content/Arrow/ArrowTip の anatomy
//!   パーツ関数群と、[`state::Disclosure`] を埋め込んだ [`tooltip::Tooltip`]
//!   状態機械（#533、親 #530）。WAI-ARIA tooltip パターンに従い `aria-describedby`
//!   を使う点が [`mod@collapsible`] との違い。
//! - [`mod@dialog`]: [`dialog::Dialog`] — Root / Trigger / Backdrop /
//!   Positioner / Content / Title / Description / CloseTrigger の 8 anatomy
//!   パーツと [`state::Disclosure`] を埋め込んだモーダルダイアログ（#531）。
//! - [`mod@radio_group`]: Root / Label / Item / ItemControl / ItemText /
//!   ItemHiddenInput の 6 anatomy パーツと [`state::SingleSelect`] を埋め込んだ
//!   [`radio_group::RadioGroup`]（#536、親 #534）。クライアント由来の文字列
//!   dispatch は `"select"` のみを受理する（WAI-ARIA radio パターンに選択解除
//!   ジェスチャは存在しないため、型付き API の `Deselect` のみプログラム的な
//!   選択解除を許す）。
//! - [`popover`]: Root / Trigger / Anchor / Positioner / Arrow / ArrowTip /
//!   Content / Title / Description / CloseTrigger / Indicator の 11 anatomy
//!   パーツと [`state::Disclosure`] を埋め込んだ [`popover::Popover`] を提供する
//!   headless Popover コンポーネント（#532）。
//! - [`mod@field`]: Root / Label / Input / Textarea / Select / HelperText /
//!   ErrorText / RequiredIndicator の 8 anatomy パーツ関数群
//!   （[`field::FieldProps`] から決定的に描画する純粋関数、#538）。
//!   `invalid`/`disabled`/`required`/`readonly` は SSR 静的な props であり、
//!   開閉のような時間変化する内部状態を持たないため [`mod@state`] の状態機械を
//!   適用しない（[`mod@tabs`] と同型の判断）。
//! - [`mod@menu`]: Root / Trigger / Indicator / Positioner / Content / Arrow /
//!   ArrowTip / Item / ItemGroup / ItemGroupLabel / Separator / TriggerItem /
//!   ContextTrigger の 13 anatomy パーツと [`state::Disclosure`] を埋め込んだ
//!   [`menu::Menu`]（headless Menu コンポーネント、#540/#598）。構造上最も
//!   近い先行例は [`popover::Popover`]（trigger 起点のオーバーレイ +
//!   `Disclosure` 埋め込み）であり、本モジュールはそのパターンに完全準拠する。
//! - [`mod@select`]: Root / Label / Control / Trigger / ValueText /
//!   ClearTrigger / Indicator / Positioner / Content / ItemGroup /
//!   ItemGroupLabel / Item / ItemText / ItemIndicator / HiddenSelect の 15
//!   anatomy パーツと、[`state::Disclosure`]（listbox 開閉）+
//!   [`state::SingleSelect`]（選択値）を合成した [`select::Select`] 状態機械
//!   （#541、親 #539）。Disclosure と SingleSelect を 1 コンポーネントに
//!   合成する初の例。[`select::item`] の `highlighted`/`id` 引数と
//!   [`select::content`] の `activedescendant` 引数が `data-highlighted`/
//!   `aria-activedescendant` の SSR 静的表現を提供する（イシュー #599）。
//! - [`mod@switch`]: Root / Control / Thumb / Label / HiddenInput の 5 anatomy
//!   パーツと、`"checked"`/`"unchecked"` 語彙の [`switch::Switch`] 状態機械
//!   （#537、親 #534）。ark-ui 準拠の値語彙が [`state::Disclosure`] の
//!   `"open"`/`"closed"` と異なるため、[`state`] を埋め込まず
//!   [`fandhe_frontend_interactive::Component`]/
//!   [`fandhe_frontend_interactive::Hydrate`] を直接実装する点が
//!   [`mod@collapsible`] との違い。
//! - [`mod@avatar`]: Root / Image / Fallback の 3 anatomy パーツと、画像読み込み
//!   ステータス（`"loading"`/`"loaded"`/`"error"`）の [`avatar::Avatar`] 状態
//!   機械（#543、親 #542）。[`mod@switch`] と同様、[`state`] を埋め込まず
//!   [`fandhe_frontend_interactive::Component`]/
//!   [`fandhe_frontend_interactive::Hydrate`] を直接実装する（3 値ステータス
//!   が [`state::Disclosure`]/[`state::SingleSelect`] のいずれにも写像
//!   できないため）。`data-state`（`"visible"`/`"hidden"`）は Image/Fallback
//!   のみに付与し、ark-ui 準拠で Root には付与しない。
//! - [`mod@progress`]: Root / Label / ValueText / Track / Range の 5 anatomy
//!   パーツと、数値 `value`（`min`..=`max`、または indeterminate を表す
//!   `None`）を持つ [`progress::Progress`] 状態機械（#544、親 #542）。
//!   [`mod@switch`] と同じく `data-state` 値語彙（`"indeterminate"`/
//!   `"loading"`/`"complete"`）が [`state::Disclosure`] と異なるため、
//!   [`state`] を埋め込まず [`fandhe_frontend_interactive::Component`]/
//!   [`fandhe_frontend_interactive::Hydrate`] を直接実装する。加えて
//!   Circular（SVG）用の Circle/CircleTrack/CircleRange の 3 パーツ
//!   （#600、親 #542）を持つ。CSS 変数（`--size`/`--thickness`）参照の
//!   固定 `style` で描画し、状態機械・hydration フォーマットへの追加は
//!   ない（詳細は [`progress`] モジュール doc の circular 節を参照）。
//!
//! # `fandhe-frontend-core` の再エクスポート（イシュー #550）
//!
//! `fandhe-frontend-pre-styled-ui` は方針上 `fandhe-frontend-core` を直接の
//! ランタイム依存に持たず（`crates/pre-styled-ui/Cargo.toml` 参照。`core` は
//! dev-dependency のみ）、styled 部品が組み立てる [`fandhe_frontend_core::Node`]
//! への型参照は本クレート経由の間接依存で得る契約になっている。そのため
//! [`fandhe_frontend_core`] クレート自体を本クレートのルートから再エクスポート
//! する（`pre_styled_ui` 側は `fandhe_frontend_headless_ui::fandhe_frontend_core::Node`
//! のようにアクセスする）。新規の外部依存追加ではなく、既存 path 依存の可視性を
//! 広げるだけであり `structure.toml` の depends_on 検証には影響しない。
//!
//! いずれも [`fandhe_frontend_core::el`] への薄い委譲・属性タプルの組み立てに
//! 留め、独自のエスケープ経路や HTML 文字列組み立てを持たない
//! （`docs/api/component-api.md` 不変条件準拠）。`data-state` 属性名自体は
//! [`data_attrs::data_state`] が一元管理し、[`state`] モジュールはそれを
//! 呼び出して値（`"open"`/`"closed"`）を決める側に徹する（属性名の重複定義を
//! 避ける）。各コンポーネントの anatomy 定義（Accordion / Dialog 等の parts
//! 一覧）は Phase 2（#526〜#544）のスコープ。
//!
//! - [`mod@checkbox`]: ark-ui Checkbox 相当の anatomy（イシュー #535）。
//!   dispatch 統合（クリックトグル等の動的状態遷移）は #524 のスコープ。

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod accordion;
pub mod anatomy;
pub mod aria;
pub mod avatar;
pub mod checkbox;
pub mod collapsible;
pub mod data_attrs;
pub mod dialog;
pub mod field;
pub mod menu;
pub mod popover;
pub mod progress;
pub mod radio_group;
pub mod select;
pub mod state;
pub mod switch;
pub mod tabs;
pub mod tooltip;

// `pub use fandhe_frontend_core;` はクレートそのものの再エクスポート（型/値の
// 再エクスポートではない）。`missing_docs` は extern crate 再エクスポートには
// 適用されないため doc コメントは不要（rustc の既定挙動）。
pub use fandhe_frontend_core;

pub use anatomy::{anatomy, Anatomy};
pub use aria::{
    aria_activedescendant, aria_checked, aria_controls, aria_describedby, aria_disabled,
    aria_expanded, aria_haspopup, aria_hidden, aria_invalid, aria_label, aria_labelledby,
    aria_modal, aria_orientation, aria_selected, role, AriaChecked, AriaPopup,
};
pub use avatar::{Avatar, AvatarAction, ImageStatus};
pub use data_attrs::{
    data_disabled, data_highlighted, data_invalid, data_orientation, data_readonly, data_required,
    data_state, Orientation,
};
pub use dialog::Dialog;
pub use field::FieldProps;
pub use menu::Menu;
pub use progress::{Progress, ProgressAction};
pub use radio_group::RadioGroup;
pub use state::{
    Disclosure, DisclosureAction, MultiSelect, MultiSelectAction, OpenState, SingleSelect,
    SingleSelectAction, DATA_STATE_CLOSED, DATA_STATE_OPEN,
};
pub use switch::{Switch, SwitchAction};
pub use tabs::{tabs, ActivationMode, TabItem, TabsProps};
pub use tooltip::Tooltip;
