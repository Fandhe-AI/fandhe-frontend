//! `fandhe-frontend-headless-ui`: headless UI コンポーネント層（外部依存は
//! `fandhe-frontend-core` のみ）。
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
//! 4. **外部依存は `fandhe-frontend-core`（path）のみ**: `headless-ui/Cargo.toml` の
//!    `[dependencies]` にサードパーティクレートを追加しない。
//!
//! # 実装済み API（イシュー #523）
//!
//! - [`mod@anatomy`]: `data-scope` / `data-part` を付与してパーツノードを組み立てる
//!   [`anatomy::Anatomy`]（全コンポーネント共通の anatomy 基盤）。
//! - [`data_attrs`]: `data-state` / `data-disabled` 等の状態属性ヘルパ。
//! - [`aria`]: `role` / `aria-*` の WAI-ARIA 属性ヘルパ。
//!
//! いずれも [`fandhe_frontend_core::el`] への薄い委譲・属性タプルの組み立てに
//! 留め、独自のエスケープ経路や HTML 文字列組み立てを持たない
//! （`docs/api/component-api.md` 不変条件準拠）。`data-state` と状態機械の
//! 一致保証は `fandhe-frontend-interactive` と連携するイシュー #524 のスコープ。
//! 各コンポーネントの anatomy 定義（Accordion / Dialog 等の parts 一覧）は
//! Phase 2（#526〜#544）のスコープ。

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod anatomy;
pub mod aria;
pub mod data_attrs;

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
