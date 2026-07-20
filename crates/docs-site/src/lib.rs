//! `fandhe-frontend-docs-site` のライブラリ入口。
//!
//! 公式 docs サイト（自前 SSG ドッグフーディング、親イシュー #459 / ルート #456）の
//! ページ生成ロジックを束ねるクレート。バイナリ本体（`src/main.rs`）は
//! fail-closed の未実装終了を維持したまま、統合テスト（`tests/`）から
//! `layout` / `markdown` / `nav` の各モジュールを直接検証できるように
//! するために `[lib]` ターゲットを併設する。crate 外部への公開・配布は
//! 行わない（`Cargo.toml` の `publish = false`）。
//!
//! - [`layout`]: docs レイアウトコンポーネント（イシュー #469）
//! - [`markdown`]: Markdown ブロック構文 → Node 木レンダラ（イシュー #466）
//! - [`nav`]: `site/nav.toml` のパース・サイドバー / 前後ナビ生成（イシュー #468）
//!
//! `main.rs` 側の統合（`nav::parse_nav` の呼び出し・`layout` との結合・
//! `generate_pages()` への引き渡し）は後続イシュー #470 のスコープ。
//!
//! `fandhe-frontend-core` / `fandhe-frontend-app` / `fandhe-frontend-server` のみに
//! 依存し、外部クレートは追加しない（`Cargo.toml` の REQ-3 非影響コメント参照）。
//!
//! `#![forbid(unsafe_code)]` は `crates/core` / `crates/interactive` と同様に
//! 本クレートでも維持する（`.claude/rules/coding-rust.md` の一般規約）。

#![forbid(unsafe_code)]

pub mod layout;
pub mod markdown;
pub mod nav;
