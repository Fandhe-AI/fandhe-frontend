//! `xtask check-dep-versions` の CLI 契約に対する回帰テスト（イシュー #657）。
//!
//! `.github/workflows/ci.yml` の `dep-version-check` ジョブは本テストが固定
//! する契約（終了コード・1 行サマリ書式・`--fix` の適用範囲）に依拠する。
//!
//! 契約（`xtask/src/main.rs` の `run_check_dep_versions` /
//! `check_dep_versions::format_report` 参照）:
//! - 終了コード 0: 全エッジ PASS（`--fix` 適用後の再検証込み）
//! - 終了コード 1: ルール 1（version 不一致）・ルール 2（公開対象クレートの
//!   version 欠落）いずれかの FAIL・環境エラー
//! - 終了コード 2: 引数不備（不明な引数）
//! - stdout の 1 行サマリ書式は
//!   `dep-version-check: crate=<name> dep=<name> kind=<normal|dev|build> req=<req> actual=<v> result=<PASS|FAIL>`
//!
//! ネットワーク照会は一切行わない（`cargo metadata --no-deps` のみ）ため、
//! `cli_check_version_bump.rs` と異なり curl 可用性チェックは不要。
//!
//! フィクスチャは `CARGO_TARGET_TMPDIR`（共有 `CARGO_TARGET_DIR` 環境下でも
//! 安全、ci.md 参照）配下に一意名の最小 workspace を都度生成する。
//! `cargo metadata` のみを実行しビルドは行わないため、共有 `CARGO_TARGET_DIR`
//! を汚染するキャッシュ誤命中の懸念はない。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// `xtask` バイナリの起動先。
fn xtask_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xtask")
}

fn scratch_root() -> PathBuf {
    std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
}

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_fixture_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = FIXTURE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = scratch_root().join(format!(
        "xtask-check-dep-versions-{label}-{}-{nanos}-{n}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("フィクスチャディレクトリの作成に失敗した");
    dir
}

/// スコープを抜けるタイミングで自身を削除するガード
/// （`cli_check_version_bump.rs::ScratchDir` と同一方針）。
struct ScratchDir(PathBuf);

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// 2 クレート（`a`・`b`、`b` が `a` へ path+version 依存）からなる最小
/// workspace フィクスチャを書き出す。`b` の `a` への `version` 要求は
/// `dep_b_to_a_req` でシナリオごとに差し替える。フィクスチャ専用名を使い
/// 既存クレート・他テストのフィクスチャと衝突しない（ci.md 対策）。
fn init_fixture_workspace(dir: &Path, label: &str, a_version: &str, dep_b_to_a_req: &str) {
    fs::write(
        dir.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crate-a\", \"crate-b\"]\nresolver = \"2\"\n",
    )
    .unwrap();

    let crate_a = dir.join("crate-a");
    fs::create_dir_all(crate_a.join("src")).unwrap();
    fs::write(
        crate_a.join("Cargo.toml"),
        format!(
            "[package]\nname = \"xtask-fixture-dep-versions-{label}-a\"\nversion = \"{a_version}\"\nedition = \"2021\"\n"
        ),
    )
    .unwrap();
    fs::write(crate_a.join("src/lib.rs"), "pub fn a() {}\n").unwrap();

    let crate_b = dir.join("crate-b");
    fs::create_dir_all(crate_b.join("src")).unwrap();
    let version_clause = if dep_b_to_a_req == "*" {
        "{ path = \"../crate-a\" }".to_string()
    } else {
        format!("{{ path = \"../crate-a\", version = \"{dep_b_to_a_req}\" }}")
    };
    fs::write(
        crate_b.join("Cargo.toml"),
        format!(
            "[package]\nname = \"xtask-fixture-dep-versions-{label}-b\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nxtask-fixture-dep-versions-{label}-a = {version_clause}\n"
        ),
    )
    .unwrap();
    fs::write(crate_b.join("src/lib.rs"), "pub fn b() {}\n").unwrap();
}

fn run_check_dep_versions(dir: &Path, extra_args: &[&str]) -> std::process::Output {
    Command::new(xtask_bin())
        .arg("check-dep-versions")
        .args(extra_args)
        .current_dir(dir)
        .output()
        .expect("xtask バイナリの起動に失敗した")
}

#[test]
fn unknown_flag_exits_two() {
    let output = Command::new(xtask_bin())
        .args(["check-dep-versions", "--bogus"])
        .output()
        .expect("xtask バイナリの起動に失敗した");
    assert_eq!(
        output.status.code(),
        Some(2),
        "未知の引数は usage エラー（終了コード 2）契約"
    );
}

#[test]
fn matching_version_passes() {
    let scratch = ScratchDir(unique_fixture_dir("match"));
    init_fixture_workspace(&scratch.0, "match", "0.2.0", "0.2.0");

    let output = run_check_dep_versions(&scratch.0, &[]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "req が依存先の現行 version と一致する場合は PASS 契約。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("req=^0.2.0")
            && stdout.contains("actual=0.2.0")
            && stdout.contains("result=PASS"),
        "1 行サマリに result=PASS が含まれる契約: {stdout}"
    );
}

#[test]
fn stale_version_requirement_fails() {
    let scratch = ScratchDir(unique_fixture_dir("stale"));
    init_fixture_workspace(&scratch.0, "stale", "0.2.1", "0.2.0");

    let output = run_check_dep_versions(&scratch.0, &[]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "依存先が互換バンプ（0.2.0→0.2.1）されたのに req が追随していない場合は FAIL 契約。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("req=^0.2.0")
            && stdout.contains("actual=0.2.1")
            && stdout.contains("result=FAIL"),
        "1 行サマリに result=FAIL が含まれる契約: {stdout}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("check-dep-versions --fix"),
        "是正コマンドの案内が stderr に含まれる契約: {stderr}"
    );
}

#[test]
fn missing_version_on_publishable_normal_dep_fails() {
    let scratch = ScratchDir(unique_fixture_dir("missing-version"));
    init_fixture_workspace(&scratch.0, "missing-version", "0.2.0", "*");

    let output = run_check_dep_versions(&scratch.0, &[]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "publish 対象クレートの normal path 依存に version 宣言がない場合は FAIL 契約（ルール 2）。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("req=*") && stdout.contains("result=FAIL"),
        "1 行サマリに result=FAIL が含まれる契約: {stdout}"
    );
}

#[test]
fn missing_version_on_non_publishable_dependent_passes() {
    let scratch = ScratchDir(unique_fixture_dir("missing-version-non-publish"));
    init_fixture_workspace(&scratch.0, "missing-version-non-publish", "0.2.0", "*");
    // crate-b を publish = false にすることでルール 2 の対象外にする。
    let crate_b_manifest = scratch.0.join("crate-b/Cargo.toml");
    let content = fs::read_to_string(&crate_b_manifest).unwrap();
    let content = content.replace(
        "edition = \"2021\"\n",
        "edition = \"2021\"\npublish = false\n",
    );
    fs::write(&crate_b_manifest, content).unwrap();

    let output = run_check_dep_versions(&scratch.0, &[]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "publish = false の依存元は version 欠落でもルール 2 対象外で PASS 契約。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn fix_rewrites_stale_version_and_reverifies_pass() {
    let scratch = ScratchDir(unique_fixture_dir("fix"));
    init_fixture_workspace(&scratch.0, "fix", "0.2.1", "0.2.0");

    let output = run_check_dep_versions(&scratch.0, &["--fix"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "--fix はルール 1 の FAIL を書き換えて PASS 化する契約。stdout: {} stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let crate_b_manifest = scratch.0.join("crate-b/Cargo.toml");
    let content = fs::read_to_string(&crate_b_manifest).unwrap();
    assert!(
        content.contains("version = \"0.2.1\""),
        "--fix 後は依存先の現行 version へ書き換えられている契約: {content}"
    );
    assert!(
        !content.contains("version = \"0.2.0\""),
        "--fix 後は旧 version 文字列が残っていない契約: {content}"
    );

    // 書き換え後に再実行しても PASS のままであること（べき等性）。
    let output2 = run_check_dep_versions(&scratch.0, &[]);
    assert_eq!(
        output2.status.code(),
        Some(0),
        "--fix 適用後、通常実行でも PASS のままである契約"
    );
}

#[test]
fn fix_does_not_touch_files_when_requirement_form_is_unsupported() {
    let scratch = ScratchDir(unique_fixture_dir("fix-unsupported"));
    init_fixture_workspace(&scratch.0, "fix-unsupported", "0.2.1", "0.2.0");

    // req を `=` ピン（サポート外の表記）へ手動で書き換える。
    let crate_b_manifest = scratch.0.join("crate-b/Cargo.toml");
    let content = fs::read_to_string(&crate_b_manifest).unwrap();
    let content = content.replace("version = \"0.2.0\"", "version = \"=0.2.0\"");
    fs::write(&crate_b_manifest, &content).unwrap();

    let output = run_check_dep_versions(&scratch.0, &["--fix"]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "サポート外の req 表記は --fix でも書き換えられず FAIL のまま残る契約。stdout: {} stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let content_after = fs::read_to_string(&crate_b_manifest).unwrap();
    assert_eq!(
        content, content_after,
        "書き換え位置を一意特定できない場合はファイルを一切書き換えない契約（部分書き込み禁止）"
    );
}
