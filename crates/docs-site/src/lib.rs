//! `fandhe-frontend-docs-site` のライブラリ入口。
//!
//! 公式 docs サイト（自前 SSG ドッグフーディング、親イシュー #459 / ルート #456）の
//! ページ生成ロジックを束ねるクレート。`main.rs`（バイナリエントリ）は
//! `fandhe_frontend_server::ssg::generate_pages()` へ渡す `(String, Node)` 列を
//! 組み立てるためにここの公開関数を呼び出す想定（接続自体は後続イシュー #470）。
//!
//! - [`layout`]: docs レイアウトコンポーネント（イシュー #469）
//! - [`nav`]: `site/nav.toml` のパース・サイドバー / 前後ナビ生成（イシュー #468）
//!
//! `main.rs` 側の統合（`nav::parse_nav` の呼び出し・`layout` との結合・
//! `generate_pages()` への引き渡し）は後続イシュー #470 のスコープ。
//!
//! `fandhe-frontend-core` / `fandhe-frontend-app` / `fandhe-frontend-server` のみに
//! 依存し、外部クレートは追加しない（`Cargo.toml` の REQ-3 非影響コメント参照）。

#![forbid(unsafe_code)]

pub mod layout;
pub mod nav;
