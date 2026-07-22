//! `fandhe-frontend-pre-styled-ui`: pre-styled UI コンポーネント層（外部依存は
//! `fandhe-frontend-headless-ui` のみ）。
//!
//! chakra-ui 相当の pre-styled（既定スタイル付き）UI コンポーネント層を提供する。
//! `fandhe-frontend-headless-ui`（anatomy・`data-*`・WAI-ARIA、イシュー #522）の上に
//! テーマトークン・variant API・静的 CSS 生成を重ね、styled 部品を実装する 2 層構造の
//! 上層を担う（親トラッキング #520、Phase 3 親 #545）。
//!
//! # 本クレートの不変条件（REQ-1・REQ-2・REQ-5、`.claude/rules/coding-rust.md`）
//!
//! 1. コンポーネントは [`fandhe_frontend_headless_ui`] 経由で
//!    `fandhe_frontend_core::Node` を返す通常の Rust 関数として実装する
//!    （REQ-5、マクロ DSL は採用しない）。
//! 2. 出力は `fandhe_frontend_core::render` の既定エスケープを必ず経由する。
//!    **本クレート内では `raw_html()` を使用しない**（新たなエスケープ迂回経路を
//!    作らない）。
//! 3. **`unsafe` コード禁止**: `#![forbid(unsafe_code)]` によりクレート全体で
//!    機械的に禁止する（`crates/core/tests/unsafe_boundary.rs` が workspace
//!    member を自動発見して強制する）。
//! 4. **外部依存は `fandhe-frontend-headless-ui`（path）のみ**:
//!    `pre-styled-ui/Cargo.toml` の `[dependencies]` にサードパーティクレートを
//!    追加しない。`fandhe-frontend-core` への直接依存は headless-ui 経由で
//!    間接的に得る（dev-dependency としてのみ利用、後述）。styled 部品の
//!    `Node` 型参照は `fandhe_frontend_headless_ui::fandhe_frontend_core::Node`
//!    （headless-ui が再エクスポートする core、イシュー #550）経由で得る。
//!
//! # 実装済み API（イシュー #546/#547/#548/#550/#551）
//!
//! - [`theme`]（#547）: テーマトークン・ダークモード基盤。
//! - [`css`]（#548）: CSS 宣言の低レベル表現・検証・シリアライズ。
//! - [`recipe`]（#548）: slot recipe 本体・[`recipe::SlotRecipe`]・
//!   [`recipe::VariantValue`]。
//! - 状態機械を要しない単純 styled 部品 5 種（#550）:
//!   - [`mod@button`]: [`button::button`]（単一 recipe、`<button type="button">`。
//!     `loading` 時は [`mod@spinner`] を子ノード先頭へ埋め込む）。
//!   - [`mod@badge`]: [`badge::badge`]（単一 recipe、`<span>`）。
//!   - [`mod@spinner`]: [`spinner::spinner`]（単一 recipe、
//!     `<span role="status">`）。
//!   - [`mod@alert`]: [`alert::root`] ほかパーツ関数群（slot recipe、
//!     root/indicator/content/title/description の 5 パーツ、`role="alert"`）。
//!   - [`mod@card`]: [`card::root`] ほかパーツ関数群（slot recipe、
//!     root/header/body/footer/title/description の 6 パーツ、装飾的コンテナ、
//!     role 付与なし）。
//!
//!   いずれも variant/size/status は Rust enum（[`recipe::VariantValue`] 実装）
//!   として型安全に表現し、クラス名文字列を動的合成しない
//!   （[`recipe::SlotRecipe::variant_classes`] が決定的に生成する）。
//!   呼び出し側 `attrs` に含まれる `class` は `class_attr`（内部専用モジュール）
//!   が除去してから recipe 生成クラスと合成し、`class` 属性が常に単一になる
//!   ことを保証する。
//!
//! - headless 状態機械を持つ複合部品 5 種の styled ラッパー第 1 弾（#551）:
//!   [`mod@dialog`] / [`mod@tabs`] / [`mod@accordion`] / [`mod@menu`] /
//!   [`mod@select`]。examples・利用ガイド（#552）は別イシューのスコープ。
//!
//! # headless ラッパーの設計（#551）
//!
//! [`mod@dialog`]・[`mod@accordion`]・[`mod@menu`]・[`mod@select`]・
//! [`mod@tabs`] はいずれも `fandhe_frontend_headless_ui` の対応モジュールが
//! 出力する `data-scope`/`data-part` 属性セレクタへ [`recipe::SlotRecipe`]
//! で静的 CSS を対応付ける薄い委譲層である。パーツ関数・状態機械
//! （`Dialog`/`Accordion`/`Menu`/`Select`）は headless 層からそのまま
//! 再エクスポートし（`pub use ...::*`）、新たな出力経路・エスケープ迂回は
//! 一切持たない。各モジュールの `stylesheet()` が生成する CSS は静的
//! `.css` ファイルとして配信する利用形態を前提とし、`<style>` タグへの
//! インライン埋め込み（`raw_html()` が必要になる）は本クレートの責務外
//! （不変条件 2 を参照）。variant（size 等）ごとのクラス切り替えは
//! headless ラッパー第 2 弾以降のスコープとする（各モジュール rustdoc の
//! スコープ外節を参照）。
//!
//! [`theme`] が生成する CSS は静的 `.css` ファイルとして配信する利用形態を
//! 前提とし、`<style>` タグへの埋め込み（`raw_html` が必要になる）は本クレート
//! では提供しない（不変条件 2 を参照）。styled 部品各モジュールの `css()` も
//! 同じ利用形態を前提とする。

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod accordion;
pub mod alert;
pub mod badge;
pub mod button;
pub mod card;
mod class_attr;
pub mod css;
pub mod dialog;
pub mod menu;
pub mod recipe;
pub mod select;
pub mod spinner;
pub mod tabs;
pub mod theme;

pub use alert::AlertStatus;
pub use badge::{badge, BadgeProps, BadgeVariant};
pub use button::{button, ButtonProps, ButtonVariant};
pub use card::CardVariant;
pub use css::{decl, Declaration};
pub use recipe::{Size, SlotRecipe, VariantValue};
pub use spinner::{spinner, SpinnerProps};
