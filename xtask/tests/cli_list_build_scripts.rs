//! `xtask list-build-scripts` の CLI 契約に対する回帰テスト（TASK-3.2a）。
//!
//! `xtask/tests/cli_check_deps.rs` と同型の構成。CI ワークフローへの組み込み
//! （TASK-3.2b・イシュー #21）は本テストが固定する契約に依拠する見込みのため、
//! ここで終了コードと出力書式を先に固定しておく。
//!
//! 契約（`xtask/src/main.rs` の `run_list_build_scripts` /
//! `list_build_scripts::format_inventory` 参照）:
//! - 終了コード 0: 指定パッケージすべてが列挙に成功（build.rs 保有クレートが
//!   0 件でも成功。列挙自体は上限判定を伴わない）
//! - 終了コード 1: `cargo metadata` の実行失敗・想定外の出力構造・ルート未検出
//!   （fail-closed。「列挙できなかったのに成功扱い」になる経路を作らない）
//! - 終了コード 2: 引数不備（`--package` 未指定・不明な引数）
//! - stdout の 1 行サマリ書式は `build-scripts: target=<name> count=<n>`
//!   （`--package` 指定ごとに 1 行）。ただし最終契約確定は TASK-3.2b（#21）に委ねる
//!   （`list_build_scripts::format_inventory` のドキュメント参照）。
//!
//! 子プロセスとしてビルド済み xtask バイナリ（`CARGO_BIN_EXE_xtask`）を起動する。
//! `cargo metadata` はさらにその子プロセスとして呼ばれるため、カレントディレクトリを
//! workspace ルート（`CARGO_MANIFEST_DIR` の親）に設定する。

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
fn list_build_scripts_single_package_with_no_build_scripts_exits_zero() {
    let output = run_xtask(&["list-build-scripts", "--package", "rws-core"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        output.status.code(),
        Some(0),
        "rws-core は build.rs 非保有かつ 0 件は正常終了扱いの想定。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout
            .lines()
            .any(|line| line.starts_with("build-scripts: ") && line.contains("count=0")),
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
        "rws-core / xtask はいずれも build.rs 非保有である想定。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| line.starts_with("build-scripts: "))
        .collect();
    assert_eq!(
        summary_lines.len(),
        2,
        "--package を 2 件指定した場合、パッケージごとに 1 行サマリが出力される契約: {stdout}"
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
        "workspace に存在しないパッケージの列挙失敗は fail-closed（終了コード 1）契約"
    );
}
