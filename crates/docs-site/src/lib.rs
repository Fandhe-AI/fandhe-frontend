//! `fandhe-frontend-docs-site` のライブラリ入口。
//!
//! バイナリ本体（`src/main.rs`）は fail-closed の未実装終了を維持したまま、
//! 統合テスト（`tests/`）から `markdown` / `nav` の各モジュールを直接検証
//! できるようにするために `[lib]` ターゲットを併設する（イシュー #466 実装
//! 計画）。crate 外部への公開・配布は行わない（`Cargo.toml` の
//! `publish = false`）。
//!
//! [`markdown`] は Markdown ブロック構文 → Node 木レンダラ（イシュー #466）、
//! [`nav`] は `site/nav.toml` のパース・サイドバー / 前後ナビ生成（イシュー
//! #468）を担う。`main.rs` 側の統合（`nav::parse_nav` の呼び出し・
//! `generate_pages()` への引き渡し）は後続イシュー #470 のスコープ。
//!
//! `#![forbid(unsafe_code)]` は `crates/core` / `crates/interactive` と同様に
//! 本クレートでも維持する（`.claude/rules/coding-rust.md` の一般規約）。

#![forbid(unsafe_code)]

pub mod markdown;
pub mod nav;
