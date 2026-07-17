//! `xtask check-loc` の CLI 契約に対する回帰テスト（TASK-8.2b・イシュー #62・
//! REQ-8 受け入れ基準、および REQ-11 受け入れ基準 3・イシュー #156 の共用ゲート）。
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
//!   （fail-closed。CI はこれを失敗として扱う。対象が複数ある現在の構成では、
//!   1 ファイルのみの超過・不在でも全体を FAIL として扱う）
//! - 終了コード 2: 引数不備（本サブコマンドは引数を一切取らない）
//! - stdout の 1 行サマリ書式は
//!   `loc-check: file=<path> effective_loc=<n>/<limit> result=<PASS|FAIL>`
//!   （`grep '^loc-check:'` で抽出可能。対象ファイル数だけ出力される）
//!
//! `check_loc::LOC_CHECK_TARGETS` はコード定数で固定されており
//! （`static/view-transitions.js` / `static/wasm-full-init.js`）CLI 引数での
//! 差し替えはできない。そのため対象ファイルの有無を切り替えるには、バイナリの
//! `current_dir` をテスト専用の一時ディレクトリに設定し、そこに各フィクスチャを
//! 配置する（TASK-8.2a／イシュー #61・イシュー #156 の成果物マージ有無に
//! 依存せずに本 CLI 契約を検証するための構成）。

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

/// `n` 行分の実効コード（コメント・空行を含まない）を生成する。
/// 境界値テスト（ちょうど 10 行 / 11 行）を各フィクスチャで使い回すためのヘルパー。
fn effective_lines(n: usize) -> String {
    (1..=n)
        .map(|i| format!("const line{i} = {i};"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// テスト専用の一時ディレクトリに `static/` フィクスチャ群を配置する。
///
/// `check_loc::LOC_CHECK_TARGETS` が複数ファイルを対象とするため（REQ-8 /
/// REQ-11）、`files` に `(ファイル名, 内容)` の組を渡してまとめて配置する。
/// プロセス PID とテスト名を組み合わせて一意なディレクトリ名にすることで、
/// 並列テスト実行時の衝突を避ける。
fn make_fixture_dir(test_name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "xtask-check-loc-test-{test_name}-{}",
        std::process::id()
    ));
    let static_dir = dir.join("static");
    fs::create_dir_all(&static_dir).expect("フィクスチャ用ディレクトリの作成に失敗した");
    for (name, body) in files {
        fs::write(static_dir.join(name), body).expect("フィクスチャファイルの書き込みに失敗した");
    }
    dir
}

#[test]
fn check_loc_passes_for_fixtures_within_limit() {
    // 両対象ファイルともちょうど 10 行の実効コード（コメント・空行を除く）。
    let body = effective_lines(10);
    let dir = make_fixture_dir(
        "within-limit",
        &[("view-transitions.js", &body), ("wasm-full-init.js", &body)],
    );

    let output = run_check_loc_in(&dir, &[]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        output.status.code(),
        Some(0),
        "両ファイルとも 10 行以内は PASS のはず。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for file in ["view-transitions.js", "wasm-full-init.js"] {
        assert!(
            stdout.lines().any(|line| {
                line.starts_with("loc-check: ")
                    && line.contains(&format!("file=static/{file}"))
                    && line.contains("effective_loc=10/10")
                    && line.contains("result=PASS")
            }),
            "1 行サマリ（{file}, result=PASS）が stdout に見つからない: {stdout}"
        );
    }
    assert_eq!(
        stdout
            .lines()
            .filter(|l| l.starts_with("loc-check: "))
            .count(),
        2,
        "対象ファイル 2 件分のサマリが出力されるはず: {stdout}"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn check_loc_fails_when_view_transitions_over_limit() {
    // view-transitions.js のみ 11 行（しきい値超過）、wasm-full-init.js は 10 行以内。
    let over = effective_lines(11);
    let within = effective_lines(10);
    let dir = make_fixture_dir(
        "over-limit-view-transitions",
        &[
            ("view-transitions.js", &over),
            ("wasm-full-init.js", &within),
        ],
    );

    let output = run_check_loc_in(&dir, &[]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        output.status.code(),
        Some(1),
        "1 ファイルでもしきい値超過なら全体が FAIL（終了コード 1）のはず"
    );
    assert!(
        stdout.lines().any(|line| {
            line.starts_with("loc-check: ")
                && line.contains("file=static/view-transitions.js")
                && line.contains("effective_loc=11/10")
                && line.contains("result=FAIL")
        }),
        "view-transitions.js の FAIL サマリが stdout に見つからない: {stdout}"
    );
    assert!(
        stdout.lines().any(|line| {
            line.starts_with("loc-check: ")
                && line.contains("file=static/wasm-full-init.js")
                && line.contains("result=PASS")
        }),
        "wasm-full-init.js は PASS のままのはず: {stdout}"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn check_loc_fails_when_wasm_full_init_over_limit() {
    // wasm-full-init.js のみ 11 行（しきい値超過）、view-transitions.js は 10 行以内。
    let over = effective_lines(11);
    let within = effective_lines(10);
    let dir = make_fixture_dir(
        "over-limit-wasm-full-init",
        &[
            ("view-transitions.js", &within),
            ("wasm-full-init.js", &over),
        ],
    );

    let output = run_check_loc_in(&dir, &[]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        output.status.code(),
        Some(1),
        "wasm-full-init.js のしきい値超過は全体を FAIL（終了コード 1）にするはず"
    );
    assert!(
        stdout.lines().any(|line| {
            line.starts_with("loc-check: ")
                && line.contains("file=static/wasm-full-init.js")
                && line.contains("effective_loc=11/10")
                && line.contains("result=FAIL")
        }),
        "wasm-full-init.js の FAIL サマリが stdout に見つからない: {stdout}"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn check_loc_fails_closed_when_all_target_files_are_missing() {
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
fn check_loc_fails_closed_when_only_one_target_file_is_missing() {
    // view-transitions.js のみ配置し、wasm-full-init.js は不在のまま。
    // 対象が複数になっても、片方のみの不在で見逃されず fail-closed を維持することを固定する。
    let within = effective_lines(10);
    let dir = make_fixture_dir(
        "missing-wasm-full-init-only",
        &[("view-transitions.js", &within)],
    );

    let output = run_check_loc_in(&dir, &[]);

    assert_eq!(
        output.status.code(),
        Some(1),
        "wasm-full-init.js のみ不在でも全体が fail-closed（終了コード 1）のはず。stderr: {}",
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
