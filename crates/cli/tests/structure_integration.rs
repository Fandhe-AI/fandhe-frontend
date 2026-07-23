//! TASK-13.1d（#131）: ルートの `structure.toml` をフィクスチャとした統合テスト。
//!
//! `cli/src/structure.rs` の単体テストが `#[cfg(test)]` 内でインライン文字列の
//! フィクスチャを検証するのに対し、本ファイルは `cargo test -p fandhe-frontend-cli` の
//! integration test（`cli/tests/`）として `fw structure` の外部プロセス起動まで
//! 通す（TASK-13.1 の受け入れ基準 1 が要求する「ツール出力」を実際に確認する）。
//!
//! 対象がこのリポジトリ自身の場合、`cargo metadata` はコミット済み
//! `Cargo.lock` を解決し直すのみでネットワークアクセスは発生しない想定。

use std::path::PathBuf;
use std::process::Command;

/// ワークスペースルート（このテストバイナリは `crates/cli/` 配下でビルドされる
/// ため、2 段の親ディレクトリを辿る、イシュー #436）。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/cli/ has a workspace root two levels up")
        .to_path_buf()
}

/// `fw` バイナリを `--project <dir>` 付きで実行し、(終了コード, stdout, stderr) を返す。
fn run_fw_structure(project_dir: &std::path::Path) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_fw"))
        .args(["structure", "--project"])
        .arg(project_dir)
        .output()
        .expect("failed to spawn `fw` binary");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn fw_structure_succeeds_on_repository_root_and_emits_four_elements() {
    let root = workspace_root();
    let (code, stdout, stderr) = run_fw_structure(&root);
    assert_eq!(
        code, 0,
        "fw structure should exit 0 on the valid repository root manifest, stderr: {stderr}"
    );
    for key in [
        "\"directories\"",
        "\"routes\"",
        "\"component_boundary\"",
        "\"dependencies\"",
    ] {
        assert!(
            stdout.contains(key),
            "stdout must contain the `{key}` element (REQ-13 acceptance criterion 1): {stdout}"
        );
    }
}

#[test]
fn fw_structure_reports_nonzero_exit_for_unknown_role() {
    let root = workspace_root();
    let original = std::fs::read_to_string(root.join("structure.toml"))
        .expect("repository root structure.toml must be readable");
    let broken = original.replacen("role = \"core\"", "role = \"not-a-role\"", 1);
    assert_ne!(
        broken, original,
        "fixture must actually mutate a `role` value"
    );

    let tmp = tempdir_for_test("fw-structure-broken-role");
    std::fs::write(tmp.join("structure.toml"), broken).unwrap();

    let (code, _stdout, stderr) = run_fw_structure(&tmp);
    assert_eq!(code, 1, "unknown role must be rejected with exit code 1");
    assert!(
        stderr.contains("role"),
        "stderr should mention the offending `role` field: {stderr}"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn fw_structure_reports_nonzero_exit_for_missing_directories() {
    // `directories` を 1 件も持たないマニフェストは NoDirectories として拒否される。
    let tmp = tempdir_for_test("fw-structure-no-directories");
    std::fs::write(tmp.join("structure.toml"), "[manifest]\nversion = 1\n").unwrap();

    let (code, _stdout, stderr) = run_fw_structure(&tmp);
    assert_eq!(code, 1);
    assert!(!stderr.is_empty());
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn fw_structure_reports_nonzero_exit_for_dangling_reference() {
    let tmp = tempdir_for_test("fw-structure-dangling-reference");
    std::fs::write(
        tmp.join("structure.toml"),
        r#"
[manifest]
version = 1

[directories.core]
role = "core"
description = "desc"
depends_on = ["ghost"]
"#,
    )
    .unwrap();

    let (code, _stdout, stderr) = run_fw_structure(&tmp);
    assert_eq!(code, 1);
    assert!(stderr.contains("ghost"));
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn fw_structure_reports_nonzero_exit_for_declared_crate_not_a_workspace_member() {
    // 実体突き合わせ（TASK-13.1c）: 宣言された crate が実際には
    // workspace member でない場合に fail-closed で拒否されること。
    let tmp = tempdir_for_test("fw-structure-unknown-crate");
    std::fs::create_dir_all(tmp.join("core")).unwrap();
    std::fs::write(
        tmp.join("structure.toml"),
        r#"
[manifest]
version = 1

[directories.core]
role = "core"
crate = "this-crate-does-not-exist-anywhere"
description = "desc"
"#,
    )
    .unwrap();
    // 実体突き合わせに到達するには `cargo metadata` が動く最低限の Cargo.toml が要る。
    // 空メンバーの仮想ワークスペースは cargo 側で拒否される
    // （"the workspace has no members"）ため、パッケージとして成立させる。
    // `[workspace]` 空テーブルを明示することで、本フィクスチャがこのリポジトリ
    // 自身の workspace（スクラッチ基点が `<target>/tmp` 配下、すなわちこの
    // ワークスペースの `target/` 内に置かれるため祖先ディレクトリ探索で
    // 発見されてしまう）のメンバーだと誤認されるのを防ぐ（cargo が返す
    // "current package believes it's in a workspace when it's not" の
    // 回避策として cargo 自身が提示する対処、イシュー #658）。
    std::fs::write(
        tmp.join("Cargo.toml"),
        "[package]\nname = \"fw-structure-test-fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
    )
    .unwrap();
    std::fs::create_dir_all(tmp.join("src")).unwrap();
    std::fs::write(tmp.join("src").join("lib.rs"), "").unwrap();

    let (code, _stdout, stderr) = run_fw_structure(&tmp);
    assert_eq!(code, 1, "stderr: {stderr}");
    assert!(stderr.contains("workspace member"));
    let _ = std::fs::remove_dir_all(&tmp);
}

/// 統合テストのスクラッチ基点。`CARGO_TARGET_TMPDIR` は cargo が統合テスト
/// バイナリの**コンパイル時のみ**設定する（Cargo Book）ため `env!` で確定し、
/// 実行時 env による明示上書きのみ許容する。`/tmp` へは一切フォールバック
/// しない（イシュー #637 の事実誤認の再発防止、#658、`tests/support/mod.rs`
/// と同一パターン）。
fn scratch_root() -> PathBuf {
    let root = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
    let _ = std::fs::create_dir_all(&root);
    root
}

/// 呼び出しごとに一意な一時ディレクトリを作る（並列テスト実行下での衝突回避）。
fn tempdir_for_test(label: &str) -> PathBuf {
    let dir = scratch_root().join(format!(
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
