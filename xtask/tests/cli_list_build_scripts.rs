//! `xtask list-build-scripts` の CLI 契約に対する回帰テスト（TASK-3.2b）。
//!
//! `.github/workflows/deps-check.yml` は本テストが固定する契約
//! （終了コード・明細行・1 行サマリ書式）に依拠して build.rs 保有クレートの
//! 監査ログを Step Summary へ転記する。すなわち本ファイルはワークフローの
//! 実質的な単体保証であり、ここで固定した契約を崩す変更は CI ワークフローの
//! 破壊に直結する（`xtask/tests/cli_check_deps.rs` と同じ設計方針）。
//!
//! 契約（`xtask/src/main.rs` の `run_list_build_scripts` /
//! `list_build_scripts::format_report` 参照）:
//! - 終了コード 0: 指定パッケージすべての列挙に成功（`build.rs` 保有クレート
//!   0 件を含む。ゲートではないため PASS/FAIL の概念はない）
//! - 終了コード 1: 列挙失敗（`cargo metadata` 失敗・指定パッケージ未検出等。
//!   fail-closed。CI はこれを失敗として扱う）
//! - 終了コード 2: 引数不備（`--package` 未指定・不明な引数）
//! - stdout の明細行（検出クレートごとに 1 行）:
//!   `build-script: <crate-name>@<version>`
//! - stdout の 1 行サマリ（`--package` 指定ごとに 1 行、`grep '^build-scripts:'` で
//!   抽出可能）: `build-scripts: package=<root> count=<n>`
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
fn list_build_scripts_single_package_exits_zero_with_summary_line() {
    // rws-core は REQ-3 上「外部依存ゼロ」が不変条件のため、build.rs 保有クレートも
    // 決定的に 0 件である。
    let output = run_xtask(&["list-build-scripts", "--package", "rws-core"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        output.status.code(),
        Some(0),
        "rws-core は外部依存ゼロ契約のため列挙に成功する想定。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout
            .lines()
            .any(|line| line == "build-scripts: package=rws-core count=0"),
        "1 行サマリ（count=0）が stdout に見つからない: {stdout}"
    );
}

#[test]
fn list_build_scripts_multiple_packages_emits_one_summary_line_per_package() {
    let output = run_xtask(&[
        "list-build-scripts",
        "--package",
        "rws-core",
        "--package",
        "xtask",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        output.status.code(),
        Some(0),
        "rws-core / xtask はいずれも外部依存ゼロ契約。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| line.starts_with("build-scripts: "))
        .collect();
    assert_eq!(
        summary_lines.len(),
        2,
        "--package を 2 件指定した場合、CI が Step Summary へ転記する 1 行サマリも \
         パッケージごとに 1 行ずつ出力される契約: {stdout}"
    );
}

#[test]
fn list_build_scripts_without_package_flag_exits_two() {
    let output = run_xtask(&["list-build-scripts"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "`--package` 未指定は usage エラー（終了コード 2）契約"
    );
}

#[test]
fn list_build_scripts_with_unknown_flag_exits_two() {
    let output = run_xtask(&["list-build-scripts", "--unknown-flag"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "未知の引数は usage エラー（終了コード 2）契約"
    );
}

#[test]
fn list_build_scripts_nonexistent_package_exits_one_fail_closed() {
    let output = run_xtask(&["list-build-scripts", "--package", "no-such-package"]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "workspace に存在しないパッケージの列挙失敗は fail-closed（終了コード 1）契約。\
         CI（deps-check.yml）はこれを PR チェック失敗として扱う"
    );
}
