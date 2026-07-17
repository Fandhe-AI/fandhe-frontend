//! `xtask check-loc` の CLI 契約に対する回帰テスト（TASK-8.2b, イシュー #62,
//! REQ-8 受け入れ基準）。
//!
//! `.github/workflows/ci.yml` の `loc-check` ジョブは本テストが固定する契約
//! （終了コード・1 行サマリ書式）に依拠して CI の PASS/FAIL を判定する。
//! すなわち本ファイルはワークフローの実質的な単体保証であり、ここで
//! 固定した契約を崩す変更は CI ワークフローの破壊に直結する。
//!
//! 契約（`xtask/src/main.rs` の `run_check_loc` / `check_loc::format_loc_report` 参照）:
//! - 終了コード 0: `check_loc::LOC_CHECK_TARGETS` の全ファイルが実効 LOC
//!   `check_loc::MAX_EFFECTIVE_LOC`（10 行）以内（PASS）
//! - 終了コード 1: しきい値超過・対象ファイル不在・読み取り失敗のいずれか
//!   （fail-closed。CI はこれを失敗として扱う）
//! - 終了コード 2: 引数不備（本サブコマンドは引数を一切取らない）
//! - stdout の 1 行サマリ書式は
//!   `loc-check: file=<path> effective_loc=<n>/<limit> result=<PASS|FAIL>`
//!   （`grep '^loc-check:'` で抽出可能）
//!
//! `check_loc::LOC_CHECK_TARGETS` はコード定数で固定されており（`static/view-transitions.js`）
//! CLI 引数での差し替えはできない。そのため対象ファイルの有無を切り替えるには、
//! バイナリの `current_dir` をテスト専用の一時ディレクトリに設定し、
//! そこに `static/view-transitions.js` フィクスチャを配置する（TASK-8.2a／イシュー #61 の
//! 成果物マージ有無に依存せずに本 CLI 契約を検証するための構成）。

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

/// workspace ルート（`xtask/` の親ディレクトリ）の絶対パスを返す。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask/ には親ディレクトリ（workspace ルート）が存在する")
        .to_path_buf()
}

/// `dir` をカレントディレクトリとして xtask バイナリを `check-loc` 系の引数で起動する。
fn run_check_loc_in(dir: &std::path::Path, extra_args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("check-loc")
        .args(extra_args)
        .current_dir(dir)
        .output()
        .expect("xtask バイナリの起動に失敗した")
}

/// テスト専用の一時ディレクトリに `static/view-transitions.js` フィクスチャを配置する。
///
/// プロセス PID とテスト名を組み合わせて一意なディレクトリ名にすることで、
/// 並列テスト実行時の衝突を避ける。
fn make_fixture_dir(test_name: &str, fixture_body: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "xtask-check-loc-test-{test_name}-{}",
        std::process::id()
    ));
    let static_dir = dir.join("static");
    fs::create_dir_all(&static_dir).expect("フィクスチャ用ディレクトリの作成に失敗した");
    fs::write(static_dir.join("view-transitions.js"), fixture_body)
        .expect("フィクスチャファイルの書き込みに失敗した");
    dir
}

#[test]
fn check_loc_passes_for_fixture_within_limit() {
    // ちょうど 10 行の実効コード（コメント・空行を除く）。
    let body = (1..=10)
        .map(|i| format!("const line{i} = {i};"))
        .collect::<Vec<_>>()
        .join("\n");
    let dir = make_fixture_dir("within-limit", &body);

    let output = run_check_loc_in(&dir, &[]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        output.status.code(),
        Some(0),
        "10 行以内は PASS のはず。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.lines().any(|line| {
            line.starts_with("loc-check: ")
                && line.contains("file=static/view-transitions.js")
                && line.contains("effective_loc=10/10")
                && line.contains("result=PASS")
        }),
        "1 行サマリ（result=PASS）が stdout に見つからない: {stdout}"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn check_loc_fails_for_fixture_over_limit() {
    // 11 行の実効コード（しきい値超過）。
    let body = (1..=11)
        .map(|i| format!("const line{i} = {i};"))
        .collect::<Vec<_>>()
        .join("\n");
    let dir = make_fixture_dir("over-limit", &body);

    let output = run_check_loc_in(&dir, &[]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        output.status.code(),
        Some(1),
        "11 行はしきい値超過で FAIL（終了コード 1）のはず"
    );
    assert!(
        stdout.lines().any(|line| {
            line.starts_with("loc-check: ")
                && line.contains("effective_loc=11/10")
                && line.contains("result=FAIL")
        }),
        "1 行サマリ（result=FAIL）が stdout に見つからない: {stdout}"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn check_loc_fails_closed_when_target_file_is_missing() {
    let dir = std::env::temp_dir().join(format!(
        "xtask-check-loc-test-missing-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("空のテスト用ディレクトリの作成に失敗した");

    let output = run_check_loc_in(&dir, &[]);

    assert_eq!(
        output.status.code(),
        Some(1),
        "対象ファイル不在は fail-closed（終了コード 1）のはず。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn check_loc_with_unknown_argument_exits_two() {
    let dir = workspace_root();
    let output = run_check_loc_in(&dir, &["--target", "foo"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "check-loc は引数を一切取らない契約（不明な引数は usage エラー）"
    );
}
