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
//!    間接的に得る（dev-dependency としてのみ利用、後述）。
//!
//! # 本ファイルのスコープ（イシュー #546/#547）
//!
//! イシュー #546 のスコープは「クレートが workspace・`structure.toml`・`fw gate` の
//! 管理下に正しく組み込まれた状態」の確立であった。本イシュー #547 で
//! [`theme`] モジュール（テーマトークン・ダークモード基盤）を追加した。
//! variant API・クラス名/静的 CSS 生成はイシュー #548、styled 部品実装は
//! イシュー #550/#551、examples・利用ガイドはイシュー #552 のスコープ。
//!
//! [`theme`] が生成する CSS は静的 `.css` ファイルとして配信する利用形態を
//! 前提とし、`<style>` タグへの埋め込み（`raw_html` が必要になる）は本クレート
//! では提供しない（不変条件 2 を参照）。

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod theme;
