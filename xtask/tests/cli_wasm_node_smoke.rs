//! `xtask wasm-node-smoke` の CLI 契約に対する回帰テスト（イシュー #297、
//! TASK-10.2 残課題。出典 PR #220 §10 スコープ外節）。
//!
//! `.github/workflows/ci.yml` の `wasm-node-smoke` ジョブは本テストが固定する
//! 契約（終了コード・1 行サマリ書式）に依拠して CI の PASS/FAIL を判定する。
//!
//! 契約（`xtask/src/main.rs::run_wasm_node_smoke` / `wasm_node_smoke` モジュール参照）:
//! - 終了コード 0: 全ステップ成功（PASS）
//! - 終了コード 1: ツール不在・バージョン不一致・ビルド失敗・bindgen 失敗・
//!   node 実行失敗・エスケープ検証失敗のいずれか（fail-closed）
//! - 終了コード 2: `--build-only` 以外の不明な引数
//! - stdout 1 行サマリ: `wasm-node-smoke: package=fandhe-frontend-wasm-thin target=nodejs
//!   mode=<full|build-only> result=<PASS|FAIL>`（`grep '^wasm-node-smoke:'` で抽出可能）
//!
//! happy path のフル e2e（wasm32 ビルド + wasm-bindgen + node 実行が実際に
//! 揃った環境が必要）は CI ジョブ側（`.github/workflows/ci.yml` の
//! `wasm-node-smoke` ジョブ）で担保し、本ファイルは環境非依存の契約検証に
//! 留める（`#[ignore]` によるごまかしはしない、`coding-rust.md` 参照）。

use std::path::PathBuf;
use std::process::{Command, Output};

/// workspace ルート（`xtask/` の親ディレクトリ）の絶対パスを返す。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask/ には親ディレクトリ（workspace ルート）が存在する")
        .to_path_buf()
}

fn run_wasm_node_smoke(extra_args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("wasm-node-smoke")
        .args(extra_args)
        .current_dir(workspace_root())
        .output()
        .expect("xtask バイナリの起動に失敗した")
}

#[test]
fn wasm_node_smoke_with_unknown_argument_exits_two() {
    let output = run_wasm_node_smoke(&["--unknown-flag"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "`--build-only` 以外の不明な引数は usage エラー（終了コード 2）のはず。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn wasm_node_smoke_fails_closed_when_required_tools_are_absent_from_path() {
    // `PATH` を空ディレクトリに制限し、`wasm-bindgen`/`node`/`cargo` のいずれも
    // 見つからない状態を作る。前提ツール検査（`verify_wasm_bindgen_version`
    // 内の `wasm-bindgen --version`）が最初に fail-closed で失敗するはず
    // （ビルド前の高速失敗、実装計画 §3.1 の設計どおり）。
    let empty_dir = std::env::temp_dir().join(format!(
        "xtask-wasm-node-smoke-test-empty-path-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&empty_dir).expect("空の PATH 用ディレクトリの作成に失敗した");

    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("wasm-node-smoke")
        .current_dir(workspace_root())
        .env("PATH", &empty_dir)
        .output()
        .expect("xtask バイナリの起動に失敗した");

    assert_eq!(
        output.status.code(),
        Some(1),
        "前提ツール（wasm-bindgen 等）不在は fail-closed（終了コード 1）のはず。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout
            .lines()
            .any(|line| { line.starts_with("wasm-node-smoke: ") && line.contains("result=FAIL") }),
        "FAIL の 1 行サマリが stdout に見つからない: {stdout}"
    );

    let _ = std::fs::remove_dir_all(&empty_dir);
}

#[test]
fn wasm_node_smoke_build_only_still_requires_wasm_bindgen_cli() {
    // `--build-only` は node 実行検査のみをスキップする契約であり、
    // wasm-bindgen-cli のバージョン整合検査は省略しない（実装計画 §3.1）。
    let empty_dir = std::env::temp_dir().join(format!(
        "xtask-wasm-node-smoke-test-build-only-empty-path-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&empty_dir).expect("空の PATH 用ディレクトリの作成に失敗した");

    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("wasm-node-smoke")
        .arg("--build-only")
        .current_dir(workspace_root())
        .env("PATH", &empty_dir)
        .output()
        .expect("xtask バイナリの起動に失敗した");

    assert_eq!(
        output.status.code(),
        Some(1),
        "--build-only でも wasm-bindgen-cli 不在は fail-closed のはず。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.lines().any(|line| {
            line.starts_with("wasm-node-smoke: ")
                && line.contains("mode=build-only")
                && line.contains("result=FAIL")
        }),
        "mode=build-only の FAIL サマリが stdout に見つからない: {stdout}"
    );

    let _ = std::fs::remove_dir_all(&empty_dir);
}

#[test]
fn usage_output_mentions_wasm_node_smoke() {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(workspace_root())
        .output()
        .expect("xtask バイナリの起動に失敗した");

    assert_eq!(
        output.status.code(),
        Some(2),
        "サブコマンド未指定は usage エラー（終了コード 2）のはず"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("wasm-node-smoke"),
        "usage 出力に `wasm-node-smoke` が含まれるはず: {stderr}"
    );
}
