//! `fandhe-frontend-docs-site` のライブラリ入口。
//!
//! バイナリ本体（`src/main.rs`）は fail-closed の未実装終了を維持したまま、
//! 統合テスト（`tests/`）から `markdown` モジュールを直接検証できるようにする
//! ために `[lib]` ターゲットを併設する（イシュー #466 実装計画）。crate 外部への
//! 公開・配布は行わない（`Cargo.toml` の `publish = false`）。
//!
//! `#![forbid(unsafe_code)]` は `crates/core` / `crates/interactive` と同様に
//! 本クレートでも維持する（`.claude/rules/coding-rust.md` の一般規約）。

#![forbid(unsafe_code)]

pub mod markdown;
