//! `xtask check-core-deps` の CLI 契約に対する回帰テスト（イシュー #154, REQ-3 受け入れ基準 1）。
//!
//! `.github/workflows/deps-check.yml` は本テストが固定する契約
//! （終了コード・1 行サマリ書式）に依拠して CI の PASS/FAIL を判定する。
//! すなわち本ファイルはワークフローの実質的な単体保証であり、ここで
//! 固定した契約を崩す変更は CI ワークフローの破壊に直結する。
//!
//! 契約（`xtask/src/main.rs` の `run_check_core_deps` /
//! `check_deps::format_zero_report` 参照）:
//! - 終了コード 0: `check_deps::ZERO_DEP_CRATES` ∩ workspace メンバーがすべて
//!   外部依存ゼロ（PASS）
//! - 終了コード 1: 外部依存が 1 件でも検出・計測失敗・積集合が空（fail-closed。
//!   CI はこれを失敗として扱う）
//! - 終了コード 2: 引数不備（本サブコマンドは引数を一切取らない）
//! - stdout の 1 行サマリ書式は
//!   `core-deps-check: package=<name> external=<n> result=<PASS|FAIL>`
//!   （`grep '^core-deps-check:'` で抽出可能。既存の `deps-check:` 行とは
//!   別プレフィックスで衝突しない）
//!
//! 子プロセスとしてビルド済み xtask バイナリ（`CARGO_BIN_EXE_xtask`）を起動する。
//! `cargo metadata` はさらにその子プロセスとして呼ばれるため、カレントディレクトリを
//! workspace ルート（`CARGO_MANIFEST_DIR` の親）に設定する。`Cargo.lock` が
//! 存在する前提でオフライン動作するため、ネットワークアクセスは発生しない。

use std::path::PathBuf;
use std::process::Command;

/// workspace ルート（`xtask/` の親ディレクトリ）の絶対パスを返す。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/xtask/ から 2 段上でワークスペースルートに到達する（イシュー #436）")
        .to_path_buf()
}

/// workspace ルートをカレントディレクトリとして xtask バイナリを起動するヘルパー。
fn run_xtask(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .current_dir(workspace_root())
        .output()
        .expect("xtask バイナリの起動に失敗した")
}

#[test]
fn check_core_deps_passes_for_real_workspace_and_reports_fandhe_frontend_core() {
    let output = run_xtask(&["check-core-deps"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        output.status.code(),
        Some(0),
        "fandhe-frontend-core は外部依存ゼロが不変条件（REQ-3 受け入れ基準 1）。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.lines().any(|line| {
            line.starts_with("core-deps-check: ")
                && line.contains("package=fandhe-frontend-core")
                && line.contains("result=PASS")
        }),
        "fandhe-frontend-core の 1 行サマリ（result=PASS）が stdout に見つからない: {stdout}"
    );
}

#[test]
fn check_core_deps_with_unknown_argument_exits_two() {
    let output = run_xtask(&["check-core-deps", "--package", "fandhe-frontend-core"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "check-core-deps は引数を一切取らない契約（不明な引数は usage エラー）"
    );
}
