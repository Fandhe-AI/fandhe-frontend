//! TASK-13.3（#138）: `fw gate` の統合テスト（外部プロセス起動を伴う）。
//!
//! 本ファイルは `cli/src/gate.rs` の単体テストが検証しない「実バイナリとしての
//! `fw gate`」の fail-closed 経路（引数不正・`structure.toml` 欠落・未レビュー
//! `raw_html()` 検出・`deny.toml` 欠落）に絞って検証する。
//!
//! **スコープ外（計画 §4）**: `cargo check`/`clippy`/`test`/`cargo deny` を実際に
//! 走らせて PASS まで到達させるフル e2e（重量級ツールチェーン依存、実行時間・
//! CI 環境での cargo-deny 有無に左右される）は TASK-13.4（シナリオ統合テスト）・
//! TASK-13.5（負例回帰テスト）のスコープとし、ここでは含めない。

use std::path::{Path, PathBuf};
use std::process::Command;

/// `fw` バイナリを `gate` サブコマンドで実行し、(終了コード, stdout, stderr) を返す。
fn run_fw_gate(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_fw"))
        .arg("gate")
        .args(args)
        .output()
        .expect("failed to spawn `fw` binary");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// 呼び出しごとに一意な一時ディレクトリを作る（並列テスト実行下での衝突回避、
/// `structure_integration.rs` と同じ命名戦略）。
fn tempdir_for_test(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{label}-{}-{:?}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn fw_gate_rejects_unknown_flag_with_usage_error() {
    let (code, _stdout, stderr) = run_fw_gate(&["--unknown-flag", "x"]);
    assert_eq!(
        code, 2,
        "malformed usage must exit 2 (usage error): {stderr}"
    );
}

#[test]
fn fw_gate_reports_blocked_for_missing_structure_manifest() {
    let tmp = tempdir_for_test("fw-gate-missing-manifest");
    let (code, stdout, _stderr) = run_fw_gate(&["--project", tmp.to_str().unwrap()]);
    assert_eq!(
        code, 1,
        "missing structure.toml must be fail-closed BLOCKED, not skipped"
    );
    assert!(stdout.contains("\"gate_result\":\"BLOCKED\""));
    let _ = std::fs::remove_dir_all(&tmp);
}

/// 最小フィクスチャ: `structure.toml` 1 件（依存ゼロの小クレート）+
/// 未レビュー `raw_html()` を含む `app/src/lib.rs` + `deny.toml` なし。
///
/// `role = "component"`（`core` 以外）を選ぶことで `default_escape_check` の
/// 走査対象に含める。`default_escape_check`（`cli/src/gate.rs`）は
/// `{project}/{directory.name}/src` のみを走査するため、`[directories.app]`
/// を宣言する本フィクスチャでは違反ファイルを **`{project}/app/src/`** 配下に
/// 置く必要がある（`{project}/src/` 直下に置くとスキャン対象外になり、
/// 「unreviewed raw_html() call」を検出するはずのアサーションが実際には何も
/// 検証しない偽陽性テストになる。Bugbot 指摘: PR #261 #1）。
fn write_minimal_fixture_with_unreviewed_raw_html(dir: &Path) {
    std::fs::write(
        dir.join("structure.toml"),
        r#"
[manifest]
version = 1

[directories.app]
role = "component"
crate = "gate-fixture-app"
description = "test fixture"
"#,
    )
    .unwrap();

    let app_dir = dir.join("app");
    std::fs::create_dir_all(&app_dir).unwrap();

    // `cargo check`/`clippy`/`test` チェックは fail-closed 経路のみを検証対象と
    // する本ファイルの方針では実際に PASS させる必要はないが、`cargo metadata`
    // 相当のツール呼び出し自体が異常終了で落ちないよう、成立する Cargo.toml を
    // 用意しておく（ワークスペースとしての体裁を保つ）。
    std::fs::write(
        app_dir.join("Cargo.toml"),
        "[package]\nname = \"gate-fixture-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();

    let src = app_dir.join("src");
    std::fs::create_dir_all(&src).unwrap();
    // マーカーなしの `raw_html()` 呼び出し（REQ-1 迂回経路の未レビュー使用）。
    std::fs::write(
        src.join("lib.rs"),
        "pub fn render(input: &str) -> String {\n    raw_html(input)\n}\n",
    )
    .unwrap();

    // 意図的に `deny.toml` を置かない（policy チェックの fail-closed を検証する）。
}

#[test]
fn fw_gate_reports_blocked_with_escape_violation_and_missing_deny_toml() {
    let tmp = tempdir_for_test("fw-gate-escape-and-policy-violation");
    write_minimal_fixture_with_unreviewed_raw_html(&tmp);

    let (code, stdout, stderr) = run_fw_gate(&["--project", tmp.to_str().unwrap()]);
    assert_eq!(
        code, 1,
        "unreviewed raw_html() + missing deny.toml must BLOCK: stderr={stderr}"
    );
    assert!(stdout.contains("\"gate_result\":\"BLOCKED\""));
    assert!(
        stdout.contains("default_escape_check"),
        "JSON report must include the default_escape_check entry: {stdout}"
    );
    assert!(
        stdout.contains("unreviewed raw_html() call"),
        "JSON report must describe the specific violation: {stdout}"
    );
    assert!(
        stdout.contains("deny.toml not found"),
        "JSON report must describe the missing deny.toml as a policy failure: {stdout}"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}
