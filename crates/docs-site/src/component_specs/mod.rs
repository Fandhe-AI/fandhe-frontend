//! 部品ページ原稿データ（Phase 4「部品ページ充填」#928 が供給する
//! [`crate::component_page::ComponentPageSpec`] の集約モジュール）。
//!
//! # 責務境界
//!
//! [`crate::showcase`] が Demo（`/components/pre-styled-ui/` ショーケース
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
//! - [`forms`]: Forms カテゴリ 31 件（イシュー #945）
//!
//! 後続 issue（#946〜#948）が `interactive` / `data_display` /
//! `typography` 等を追加する想定。

pub mod forms;
