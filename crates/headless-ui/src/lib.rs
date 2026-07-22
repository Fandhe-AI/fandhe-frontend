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
//!   抽象へ乗る開閉系状態機械（[`state::Disclosure`]/[`state::SingleSelect`]、#524）。
//!   Dialog / Accordion / Tabs / Collapsible / Popover / Tooltip（Phase 2 の
//!   #526〜#533）が共通で使う「open/closed・selected」の dispatch 契約・
//!   `data-state` 整合・SSR/hydration 契約をここに一度だけ実装し、各コンポーネントは
//!   フィールドとして埋め込んで再利用する。
//! - [`mod@tabs`]: WAI-ARIA APG の Tabs パターンに準拠したマークアップを組み立てる
//!   [`tabs::tabs`]（#528）。SSR 時点の静的な選択状態のみを扱い、クリック/
//!   キーボード操作・状態機械連携は後続イシューのスコープ。
//! - [`mod@collapsible`]: Root/Trigger/Indicator/Content の anatomy パーツ関数群と、
//!   [`state::Disclosure`] を埋め込んだ [`collapsible::Collapsible`] 状態機械
//!   （#529、親 #526）。Phase 2 で [`state`] を具象コンポーネントへ適用する最初の例。
//! - [`mod@accordion`]: Root / Item / ItemTrigger / ItemIndicator / ItemContent の
//!   5 anatomy パーツと [`state::SingleSelect`] を埋め込んだ single モード
//!   Accordion（[`accordion::Accordion`]、#527）。
//! - [`mod@tooltip`]: Root/Trigger/Positioner/Content/Arrow/ArrowTip の anatomy
//!   パーツ関数群と、[`state::Disclosure`] を埋め込んだ [`tooltip::Tooltip`]
//!   状態機械（#533、親 #530）。WAI-ARIA tooltip パターンに従い `aria-describedby`
//!   を使う点が [`mod@collapsible`] との違い。
//! - [`mod@dialog`]: [`dialog::Dialog`] — Root / Trigger / Backdrop /
//!   Positioner / Content / Title / Description / CloseTrigger の 8 anatomy
//!   パーツと [`state::Disclosure`] を埋め込んだモーダルダイアログ（#531）。
//! - [`popover`]: Root / Trigger / Anchor / Positioner / Arrow / ArrowTip /
//!   Content / Title / Description / CloseTrigger / Indicator の 11 anatomy
//!   パーツと [`state::Disclosure`] を埋め込んだ [`popover::Popover`] を提供する
//!   headless Popover コンポーネント（#532）。
//! - [`mod@switch`]: Root / Control / Thumb / Label / HiddenInput の 5 anatomy
//!   パーツと、`"checked"`/`"unchecked"` 語彙の [`switch::Switch`] 状態機械
//!   （#537、親 #534）。ark-ui 準拠の値語彙が [`state::Disclosure`] の
//!   `"open"`/`"closed"` と異なるため、[`state`] を埋め込まず
//!   [`fandhe_frontend_interactive::Component`]/
//!   [`fandhe_frontend_interactive::Hydrate`] を直接実装する点が
//!   [`mod@collapsible`] との違い。
//!
//! いずれも [`fandhe_frontend_core::el`] への薄い委譲・属性タプルの組み立てに
//! 留め、独自のエスケープ経路や HTML 文字列組み立てを持たない
//! （`docs/api/component-api.md` 不変条件準拠）。`data-state` 属性名自体は
//! [`data_attrs::data_state`] が一元管理し、[`state`] モジュールはそれを
//! 呼び出して値（`"open"`/`"closed"`）を決める側に徹する（属性名の重複定義を
//! 避ける）。各コンポーネントの anatomy 定義（Accordion / Dialog 等の parts
//! 一覧）は Phase 2（#526〜#544）のスコープ。

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod accordion;
pub mod anatomy;
pub mod aria;
pub mod collapsible;
pub mod data_attrs;
pub mod dialog;
pub mod popover;
pub mod state;
pub mod switch;
pub mod tabs;
pub mod tooltip;

pub use anatomy::{anatomy, Anatomy};
pub use aria::{
    aria_checked, aria_controls, aria_describedby, aria_disabled, aria_expanded, aria_haspopup,
    aria_hidden, aria_label, aria_labelledby, aria_modal, aria_orientation, aria_selected, role,
    AriaChecked, AriaPopup,
};
pub use data_attrs::{
    data_disabled, data_invalid, data_orientation, data_readonly, data_required, data_state,
    Orientation,
};
pub use dialog::Dialog;
pub use state::{
    Disclosure, DisclosureAction, OpenState, SingleSelect, SingleSelectAction, DATA_STATE_CLOSED,
    DATA_STATE_OPEN,
};
pub use switch::{Switch, SwitchAction};
pub use tabs::{tabs, TabItem, TabsProps};
pub use tooltip::Tooltip;
