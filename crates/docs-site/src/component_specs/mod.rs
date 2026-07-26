//! 部品ページ原稿データ（Phase 4「部品ページ充填」#928 が供給する
//! [`crate::component_page::ComponentPageSpec`] の集約モジュール）。
//!
//! # 責務境界
//!
//! [`crate::showcase`] が Demo（`/themes/` ショーケース
//! ページの実体でもある）の**正**であるのに対し、本モジュールは
//! Features・API Reference の引数表・Examples・Accessibility という
//! Radix / Ark UI 流の読み物構造の原稿データのみを持つ。
//! [`crate::component_page::generated_content`] がカテゴリ別テーブル
//! （`crate::component_page::SPEC_TABLES`）越しに参照する。
//!
//! `showcase::COMPONENT_PAGES` に登録の無い部品（Demo 節を持たない部品）は
//! 本モジュール側の [`crate::component_page::ComponentPageSpec::demo`] が
//! Demo 節を供給する。`showcase.rs` は Phase 4 のいずれの子 issue
//! （#945〜#948）からも編集しない（受け入れ条件）。
//!
//! # サブモジュール
//!
//! カテゴリ 1 個につき 1 モジュールを追加する（`site/nav.toml` の
//! `[[section.group]] title = "<カテゴリ名>"` と対応）。
//!
//! - [`forms`]: Forms カテゴリ 26 件（イシュー #945。当初登録した 31 件の
//!   うち 5 件は #948 と path が重複するスタブだったため PR #982 レビュー
//!   指摘を受けて削除済み、`forms.rs` モジュール doc 参照）
//! - [`typography`]: Typography カテゴリ追補 2 件（イシュー #995、Quote /
//!   Strong。既存 6 件は [`crate::component_page_specs_948`] が供給済み）
//!
//! 後続 issue（#946〜#947）が `interactive` / `data_display` 等を追加する
//! 想定。

pub mod forms;
pub mod typography;
