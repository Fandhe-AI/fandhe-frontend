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
//!    **`raw_html()` の使用は [`stylesheet::StyleSheet::style_element`] 内の
//!    レビュー済み 1 箇所（`#[expect(clippy::disallowed_methods, reason =
//!    "ESCAPE-REVIEWED: ...")]` 付き）に限定する**（イシュー #605）。
//!    [`StyleSheet`] は検証済み CSS のみを保持する型であり、任意文字列からの
//!    直接構築経路を公開しないため、新たなエスケープ迂回経路を作らない
//!    （詳細は [`stylesheet`] モジュール doc 参照）。
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
//! # 実装済み API（イシュー #546/#547/#548/#550/#551/#606）
//!
//! - [`theme`]（#547/#606）: テーマトークン・ダークモード基盤。#606 で
//!   角丸（`radii`）・影（`shadows`）トークングループを追加した。
//! - [`css`]（#548）: CSS 宣言の低レベル表現・検証・シリアライズ。
//! - [`recipe`]（#548/#606）: slot recipe 本体・[`recipe::SlotRecipe`]・
//!   [`recipe::VariantValue`]。#606 で標準 `colorPalette` 軸
//!   （[`recipe::ColorPalette`]）を追加した。
//! - 状態機械を要しない単純 styled 部品 5 種（#550、#606 で colorPalette 軸・
//!   radii/shadow トークン参照へ配線）:
//!   - [`mod@button`]: [`button::button`]（単一 recipe、`<button type="button">`。
//!     `loading` 時は [`mod@spinner`] を子ノード先頭へ埋め込む。`palette`
//!     variant で色を切り替える）。
//!   - [`mod@badge`]: [`badge::badge`]（単一 recipe、`<span>`。`palette` variant
//!     を持つ）。
//!   - [`mod@spinner`]: [`spinner::spinner`]（単一 recipe、
//!     `<span role="status">`。`palette` variant を持つ）。
//!   - [`mod@alert`]: [`alert::root`] ほかパーツ関数群（slot recipe、
//!     root/indicator/content/title/description の 5 パーツ、`role="alert"`。
//!     公開 API は [`alert::AlertStatus`] のまま、内部で `status` を
//!     `--fandhe-palette-*` へ束ねる）。
//!   - [`mod@card`]: [`card::root`] ほかパーツ関数群（slot recipe、
//!     root/header/body/footer/title/description の 6 パーツ、装飾的コンテナ、
//!     role 付与なし。中立コンテナのため colorPalette 軸は付与しない）。
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
//! `.css` ファイルとして配信する、または [`stylesheet::StyleSheet`]（#605）へ
//! 取り込んで `<style>` タグへインライン埋め込む、両方の利用形態を前提とする
//! （不変条件 2 を参照）。variant（size 等）ごとのクラス切り替えは
//! headless ラッパー第 2 弾以降のスコープとする（各モジュール rustdoc の
//! スコープ外節を参照）。
//!
//! [`theme`] が生成する CSS・styled 部品各モジュールの `css()`/`stylesheet()` は
//! いずれも静的 `.css` ファイルとして配信する利用形態、または
//! [`stylesheet::StyleSheet`]（#605）へ取り込んでの `<style>` 要素埋め込みの
//! 両方の利用形態を前提とする。
//!
//! # CSS の書き出し・埋め込みヘルパ（#605）
//!
//! [`stylesheet::StyleSheet`] は [`recipe::SlotRecipe::css`]・[`theme::Theme::to_css`]
//! ・各 styled 部品の `css()`/`stylesheet()` が返す決定的 CSS 文字列を集約し、
//! (a) [`stylesheet::StyleSheet::write_css_file`] による静的 `.css` ファイル
//! 書き出し（SSG・ビルドスクリプト向け）と、(b)
//! [`stylesheet::StyleSheet::style_element`] による SSR 用 `<style>` 要素
//! 埋め込みの 2 経路を提供する。検証済み CSS のみを保持する型で `raw_html()`
//! を内部に閉じ込め、呼び出し側へエスケープ迂回経路を公開しない（不変条件 2 の
//! 唯一の例外、詳細は [`stylesheet`] モジュール doc 参照）。

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
pub mod stylesheet;
pub mod tabs;
pub mod theme;

pub use alert::AlertStatus;
pub use badge::{badge, BadgeProps, BadgeVariant};
pub use button::{button, ButtonProps, ButtonVariant};
pub use card::CardVariant;
pub use css::{decl, Declaration};
pub use recipe::{ColorPalette, Size, SlotRecipe, VariantValue};
pub use spinner::{spinner, SpinnerProps};
pub use stylesheet::{StyleSheet, StylesheetError};
