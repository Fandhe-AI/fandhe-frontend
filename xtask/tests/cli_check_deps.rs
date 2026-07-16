//! `xtask check-deps` の CLI 契約に対する回帰テスト（TASK-3.1c）。
//!
//! `.github/workflows/deps-check.yml` は本テストが固定する契約
//! （終了コード・1 行サマリ書式）に依拠して CI の PASS/FAIL を判定する。
//! すなわち本ファイルはワークフローの実質的な単体保証であり、ここで
//! 固定した契約を崩す変更は CI ワークフローの破壊に直結する。
//!
//! 契約（`xtask/src/main.rs` の `run_check_deps` / `check_deps::format_report` 参照）:
//! - 終了コード 0: 指定パッケージすべてが上限内（PASS）
//! - 終了コード 1: 上限超過・計測失敗（fail-closed。CI はこれを失敗として扱う）
//! - 終了コード 2: 引数不備（`--package` 未指定・不明な引数）
//! - stdout の 1 行サマリ書式は
//!   `deps-check: packages=<n>/<limit> depth=<n>/<limit> result=<PASS|FAIL>`
//!   （`--package` 指定ごとに 1 行、`grep '^deps-check:'` で抽出可能）
//!
//! 子プロセスとしてビルド済み xtask バイナリ（`CARGO_BIN_EXE_xtask`）を起動する。
//! `cargo metadata` はさらにその子プロセスとして呼ばれるため、カレントディレクトリを
//! workspace ルート（`CARGO_MANIFEST_DIR` の親）に設定する。`Cargo.lock` が
//! 存在する前提でオフライン動作するため、ネットワークアクセスは発生しない。

use std::path::PathBuf;
use std::process::Command;

/// workspace ルート（`xtask/` の親ディレクトリ）の絶対パスを返す。
///
/// `cargo metadata` はカレントディレクトリ配下の `Cargo.toml` を起点に
/// workspace を解決するため、xtask バイナリを起動する際のカレントディレクトリ
/// として使用する。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask/ には親ディレクトリ（workspace ルート）が存在する")
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
fn check_deps_single_package_within_limits_exits_zero_with_summary_line() {
    let output = run_xtask(&["check-deps", "--package", "rws-core"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        output.status.code(),
        Some(0),
        "rws-core は REQ-3 上限内である想定。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with("deps-check: ") && line.contains("result=PASS")),
        "1 行サマリ（result=PASS）が stdout に見つからない: {stdout}"
    );
}

#[test]
fn check_deps_multiple_packages_emits_one_summary_line_per_package() {
    let output = run_xtask(&["check-deps", "--package", "rws-core", "--package", "xtask"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        output.status.code(),
        Some(0),
        "rws-core / xtask はいずれも REQ-3 上限内である想定。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| line.starts_with("deps-check: "))
        .collect();
    assert_eq!(
        summary_lines.len(),
        2,
        "--package を 2 件指定した場合、CI が Step Summary へ転記する 1 行サマリも \
         パッケージごとに 1 行ずつ出力される契約: {stdout}"
    );
}

#[test]
fn check_deps_without_package_flag_exits_two() {
    let output = run_xtask(&["check-deps"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "`--package` 未指定は usage エラー（終了コード 2）契約"
    );
}

#[test]
fn check_deps_with_unknown_flag_exits_two() {
    let output = run_xtask(&["check-deps", "--unknown-flag"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "未知の引数は usage エラー（終了コード 2）契約"
    );
}

#[test]
fn check_deps_nonexistent_package_exits_one_fail_closed() {
    let output = run_xtask(&["check-deps", "--package", "no-such-package"]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "workspace に存在しないパッケージの計測失敗は fail-closed（終了コード 1）契約。\
         CI（deps-check.yml）はこれを PR チェック失敗として扱う"
    );
}
