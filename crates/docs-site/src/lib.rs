//! `fandhe-frontend-docs-site` のライブラリ入口。
//!
//! `main.rs`（起動バイナリ）から呼ばれるロジックをテスト可能な形で
//! 切り出すためのクレートルート。イシュー #468 では [`nav`] モジュール
//! （`site/nav.toml` のパース・サイドバー / 前後ナビ生成）のみを追加する。
//! `main.rs` 側の統合（`nav::parse_nav` の呼び出し・`generate_pages()` への
//! 引き渡し）は後続イシュー #470 のスコープ。

#![forbid(unsafe_code)]

pub mod nav;
