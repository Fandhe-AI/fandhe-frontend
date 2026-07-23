//! `crates/docs-site/src/*.rs` の `#[cfg(test)]` ユニットテスト専用
//! スクラッチ基点（`crates/cli/src/test_scratch.rs` と同一パターン）。
//!
//! 統合テスト（`tests/` 配下）は `env!("CARGO_TARGET_TMPDIR")` で
//! コンパイル時に `<target>/tmp` を確定できる（cargo がテストバイナリの
//! コンパイル時のみ設定する、Cargo Book「Environment variables Cargo sets
//! for crates」）。一方、本クレートの `src/*.rs` 内 `#[cfg(test)]` は
//! ユニットテストバイナリとしてコンパイルされるため `CARGO_TARGET_TMPDIR`
//! は設定されず、`env!("CARGO_TARGET_TMPDIR")` はコンパイルエラーになる。
//!
//! このため以下の優先順位で `<target>/tmp` 相当へ解決し、`/tmp`
//! （world-writable な共有領域、symlink/TOCTOU 面の温床）へは一切
//! フォールバックしない（イシュー #637 の事実誤認の再発防止、#658）:
//!
//! 1. 実行時 `CARGO_TARGET_TMPDIR`（明示上書き。特殊なテスト実行環境向け）
//! 2. 実行時 `CARGO_TARGET_DIR`（self-hosted runner の共有
//!    `/cargo-target` 環境下では `/cargo-target/tmp` に収束し、統合テストの
//!    `env!("CARGO_TARGET_TMPDIR")` と同一の管理範囲に閉じる）
//! 3. `env!("CARGO_MANIFEST_DIR")` 基準のローカル既定レイアウト
//!    （`<repo>/target/tmp`。`.cargo/config.toml` の `build.target-dir`
//!    指定は環境変数からは見えない既知の限界だが、その場合もリポジトリ内
//!    `target/tmp` に収まり `/tmp` へは落ちない）
#![cfg(test)]

use std::path::PathBuf;

/// ユニットテスト用スクラッチルートを返す。呼び出し側は返されたディレクトリ
/// 配下にプロセス ID・ナノ秒等で一意な子ディレクトリを作って使う
/// （本関数自身はディレクトリの作成のみ行い、一意性の担保は呼び出し側の
/// 責務。`crates/cli/src/test_scratch.rs` と同一方針）。
pub(crate) fn scratch_root() -> PathBuf {
    let root = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("CARGO_TARGET_DIR")
                .map(|d| PathBuf::from(d).join("tmp"))
                .unwrap_or_else(|_| {
                    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp")
                })
        });
    let _ = std::fs::create_dir_all(&root);
    root
}
