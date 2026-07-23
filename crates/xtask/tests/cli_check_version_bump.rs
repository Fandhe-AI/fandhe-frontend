//! `xtask check-version-bump` の CLI 契約に対する回帰テスト（イシュー #638）。
//!
//! `.github/workflows/ci.yml` の `version-bump-guard` ジョブは本テストが
//! 固定する契約（終了コード・1 行サマリ書式・免除宣言の扱い）に依拠する。
//!
//! 契約（`xtask/src/main.rs` の `run_check_version_bump` /
//! `check_version_bump::format_report` 参照）:
//! - 終了コード 0: 変更なし・バンプ済み・未公開・免除のいずれか（PASS/EXEMPT）
//! - 終了コード 1: 公開済みバージョンのまま実体変更（FAIL）・環境エラー
//!   （fail-closed。CI はこれを失敗として扱う）
//! - 終了コード 2: 引数不備（`--base-ref` 未指定・不明な引数）
//! - stdout の 1 行サマリ書式は
//!   `version-bump-check: crate=<name> version=<v> published=<yes|no> result=<PASS|FAIL|EXEMPT>`
//!
//! ネットワーク照会（curl）を伴うシナリオ（バンプ漏れ検知・免除宣言）は、
//! `std::net::TcpListener` によるローカル擬似 sparse index サーバーへ
//! `--index-base-url` で向けることで、実 crates.io への到達性なしに
//! 決定的に検証する。curl 自体が runner に存在しない場合はこれらの
//! シナリオを実行できないため、`negative_cases.rs` の
//! `cargo_deny_available()` と同じ環境判定パターン（[`curl_available`]）で
//! スキップし、環境エラー分岐（`environment error: ` プレフィックス）の
//! 検証のみ行う。

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

/// `xtask` バイナリの起動先。子プロセスとしてビルド済みバイナリ
/// （`CARGO_BIN_EXE_xtask`）を起動する。
fn xtask_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xtask")
}

/// runner に curl が存在するかどうか。存在しない場合、ネットワーク照会を
/// 伴うシナリオはスキップし、環境エラー分岐のみ検証する
/// （`crates/cli/tests/support/mod.rs::cargo_deny_available` と同じ方針）。
fn curl_available() -> bool {
    Command::new("curl")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// テストごとに衝突しない一時ディレクトリを `<target>/tmp` 配下に作る
/// （`crates/cli/tests/support/mod.rs::scratch_root` と同一方針）。
///
/// cargo が `CARGO_TARGET_TMPDIR` を設定するのはテストバイナリの
/// **コンパイル時のみ**（Cargo Book）であり、実行時の `std::env::var`
/// 参照は常に失敗する。既定はコンパイル時に確定する
/// `env!("CARGO_TARGET_TMPDIR")` を使い、`/tmp` へは一切フォールバック
/// しない（イシュー #637 の事実誤認の再発防止、#658）。実行時 env による
/// 明示上書き（特殊なテスト実行環境向け）は引き続き許容する。
fn scratch_root() -> PathBuf {
    let root = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
    let _ = std::fs::create_dir_all(&root);
    root
}

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_fixture_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = FIXTURE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = scratch_root().join(format!(
        "xtask-check-version-bump-{label}-{}-{nanos}-{n}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("フィクスチャディレクトリの作成に失敗した");
    dir
}

/// スコープを抜けるタイミングで自身を削除するガード
/// （`crates/cli/tests/support/mod.rs::ScratchProject` と同一方針）。
struct ScratchDir(PathBuf);

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run_git(dir: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("git {args:?} の起動に失敗した: {e}"));
    assert!(
        output.status.success(),
        "git {args:?} が失敗した: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_stdout(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("git {args:?} の起動に失敗した: {e}"));
    assert!(
        output.status.success(),
        "git {args:?} が失敗した: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// 単一クレート `crate-a`（`version`）のみからなる最小 workspace フィクスチャを
/// 書き出し、git リポジトリとして初期化して最初のコミットを作る。
/// 既存クレートと名前が衝突しないフィクスチャ専用名を使う
/// （`ci.md`: 共有 `CARGO_TARGET_DIR` 対策）。
fn init_fixture_repo(dir: &Path, version: &str) {
    fs::write(
        dir.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crate-a\"]\nresolver = \"2\"\n",
    )
    .unwrap();
    let crate_dir = dir.join("crate-a");
    fs::create_dir_all(crate_dir.join("src")).unwrap();
    fs::write(
        crate_dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"xtask-fixture-crate-a\"\nversion = \"{version}\"\nedition = \"2021\"\n"
        ),
    )
    .unwrap();
    fs::write(
        crate_dir.join("src/lib.rs"),
        "pub fn greet() -> &'static str {\n    \"hello\"\n}\n",
    )
    .unwrap();

    run_git(dir, &["init", "-q"]);
    run_git(dir, &["config", "user.email", "fixture@example.invalid"]);
    run_git(dir, &["config", "user.name", "fixture"]);
    run_git(dir, &["add", "."]);
    run_git(dir, &["commit", "-q", "-m", "base"]);
}

/// フィクスチャ内の `crate-a` の `src/lib.rs` を書き換え、必要なら
/// `Cargo.toml` の version も更新したうえで 2 つ目のコミットを作る。
fn commit_source_change(dir: &Path, new_version: Option<&str>) {
    let crate_dir = dir.join("crate-a");
    fs::write(
        crate_dir.join("src/lib.rs"),
        "pub fn greet() -> &'static str {\n    \"hello, breaking change\"\n}\n",
    )
    .unwrap();
    if let Some(v) = new_version {
        fs::write(
            crate_dir.join("Cargo.toml"),
            format!(
                "[package]\nname = \"xtask-fixture-crate-a\"\nversion = \"{v}\"\nedition = \"2021\"\n"
            ),
        )
        .unwrap();
    }
    run_git(dir, &["add", "."]);
    run_git(dir, &["commit", "-q", "-m", "change source"]);
}

/// ローカル擬似 sparse index サーバー。単一パスへの単一リクエストのみを
/// 処理し、指定した HTTP ステータス・ボディを返して終了する
/// （テストごとに使い捨て、外部依存なしで `std::net` のみを使う）。
struct FakeIndexServer {
    base_url: String,
    handle: Option<thread::JoinHandle<()>>,
}

impl FakeIndexServer {
    fn start(status: &'static str, body: &'static str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("ローカルリスナーの起動に失敗した");
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let _ = handle_one_request(&mut stream, status, body);
            }
        });
        FakeIndexServer {
            base_url: format!("http://127.0.0.1:{port}"),
            handle: Some(handle),
        }
    }
}

impl Drop for FakeIndexServer {
    fn drop(&mut self) {
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

fn handle_one_request(stream: &mut TcpStream, status: &str, body: &str) -> std::io::Result<()> {
    // リクエスト内容は関心の対象外（パスに関わらず同一応答を返す）。
    // ヘッダ終端（"\r\n\r\n"）までを読み捨てる簡易実装で十分（テスト専用）。
    let mut buf = [0u8; 4096];
    let _ = stream.read(&mut buf)?;
    let reason = if status == "404" { "Not Found" } else { "OK" };
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{body}",
        len = body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

fn run_xtask_check_version_bump(dir: &Path, extra_args: &[&str]) -> std::process::Output {
    let head = git_stdout(dir, &["rev-parse", "HEAD"]);
    let base = git_stdout(dir, &["rev-parse", &format!("{head}~1")]);
    let mut args = vec!["check-version-bump", "--base-ref", &base];
    args.extend_from_slice(extra_args);
    Command::new(xtask_bin())
        .args(&args)
        .current_dir(dir)
        .output()
        .expect("xtask バイナリの起動に失敗した")
}

#[test]
fn missing_base_ref_exits_two() {
    let output = Command::new(xtask_bin())
        .arg("check-version-bump")
        .output()
        .expect("xtask バイナリの起動に失敗した");
    assert_eq!(
        output.status.code(),
        Some(2),
        "`--base-ref` 未指定は usage エラー（終了コード 2）契約"
    );
}

#[test]
fn unknown_flag_exits_two() {
    let output = Command::new(xtask_bin())
        .args(["check-version-bump", "--base-ref", "HEAD", "--bogus"])
        .output()
        .expect("xtask バイナリの起動に失敗した");
    assert_eq!(
        output.status.code(),
        Some(2),
        "未知の引数は usage エラー（終了コード 2）契約"
    );
}

#[test]
fn no_source_change_exits_zero_without_network() {
    let dir = ScratchDir(unique_fixture_dir("no-change"));
    init_fixture_repo(&dir.0, "0.1.0");
    // ドキュメントのみの変更（検知対象外）を 2 つ目のコミットとして積む。
    fs::write(dir.0.join("README.md"), "docs only\n").unwrap();
    run_git(&dir.0, &["add", "."]);
    run_git(&dir.0, &["commit", "-q", "-m", "docs only"]);

    let output = run_xtask_check_version_bump(&dir.0, &[]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "src/Cargo.toml/build.rs に触れない変更は判定不要で PASS 契約。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("no published crate sources changed"),
        "変更クレートなしの旨のメッセージが出力される契約: {stdout}"
    );
}

#[test]
fn source_change_without_bump_fails_when_already_published() {
    if !curl_available() {
        eprintln!("curl が利用できない環境のためネットワーク照会シナリオをスキップする");
        return;
    }
    let dir = ScratchDir(unique_fixture_dir("fail"));
    init_fixture_repo(&dir.0, "0.1.0");
    commit_source_change(&dir.0, None);

    let server = FakeIndexServer::start(
        "200",
        "{\"name\":\"xtask-fixture-crate-a\",\"vers\":\"0.1.0\"}\n",
    );
    let output = run_xtask_check_version_bump(&dir.0, &["--index-base-url", &server.base_url]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "公開済みバージョンのまま src が変更された場合は FAIL（終了コード 1）契約。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("crate=xtask-fixture-crate-a")
            && stdout.contains("version=0.1.0")
            && stdout.contains("result=FAIL"),
        "1 行サマリに result=FAIL が含まれる契約: {stdout}"
    );
}

#[test]
fn source_change_with_bump_passes() {
    if !curl_available() {
        eprintln!("curl が利用できない環境のためネットワーク照会シナリオをスキップする");
        return;
    }
    let dir = ScratchDir(unique_fixture_dir("bumped"));
    init_fixture_repo(&dir.0, "0.1.0");
    commit_source_change(&dir.0, Some("0.2.0"));

    let server = FakeIndexServer::start(
        "200",
        "{\"name\":\"xtask-fixture-crate-a\",\"vers\":\"0.1.0\"}\n",
    );
    let output = run_xtask_check_version_bump(&dir.0, &["--index-base-url", &server.base_url]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "0.1.0 は既公開だが Cargo.toml は 0.2.0 へバンプ済みのため PASS 契約。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("version=0.2.0") && stdout.contains("result=PASS"),
        "1 行サマリに result=PASS が含まれる契約: {stdout}"
    );
}

#[test]
fn empty_200_response_is_environment_error_not_a_silent_pass() {
    // イシュー #638 PR #647 レビュー指摘: HTTP 200 だが body が空/パース不能
    // （sparse index の異常応答）の場合、`Published([])` として PASS 扱い
    // してしまうと fail-open になる。fail-closed（終了コード 1・
    // "environment error: " プレフィックス）であることを固定する。
    if !curl_available() {
        eprintln!("curl が利用できない環境のためネットワーク照会シナリオをスキップする");
        return;
    }
    let dir = ScratchDir(unique_fixture_dir("empty-index-body"));
    init_fixture_repo(&dir.0, "0.1.0");
    commit_source_change(&dir.0, None);

    let server = FakeIndexServer::start("200", "");
    let output = run_xtask_check_version_bump(&dir.0, &["--index-base-url", &server.base_url]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "空 body の 200 応答は fail-closed（終了コード 1）契約。stdout: {} stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("environment error: "),
        "空 body の 200 応答は environment error として区別される契約: {stderr}"
    );
}

#[test]
fn unpublished_crate_passes_on_404() {
    if !curl_available() {
        eprintln!("curl が利用できない環境のためネットワーク照会シナリオをスキップする");
        return;
    }
    let dir = ScratchDir(unique_fixture_dir("unpublished"));
    init_fixture_repo(&dir.0, "0.1.0");
    commit_source_change(&dir.0, None);

    let server = FakeIndexServer::start("404", "");
    let output = run_xtask_check_version_bump(&dir.0, &["--index-base-url", &server.base_url]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "未公開クレート（404）は判定対象外で PASS 契約。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("published=no") && stdout.contains("result=PASS"),
        "未公開は published=no / result=PASS の契約: {stdout}"
    );
}

#[test]
fn exempt_declaration_with_exact_crate_name_passes() {
    if !curl_available() {
        eprintln!("curl が利用できない環境のためネットワーク照会シナリオをスキップする");
        return;
    }
    let dir = ScratchDir(unique_fixture_dir("exempt"));
    init_fixture_repo(&dir.0, "0.1.0");
    commit_source_change(&dir.0, None);

    let pr_body_path = dir.0.join("pr-body.txt");
    fs::write(
        &pr_body_path,
        "some intro\nversion-bump-exempt: xtask-fixture-crate-a docs-only rationale\n",
    )
    .unwrap();

    let server = FakeIndexServer::start(
        "200",
        "{\"name\":\"xtask-fixture-crate-a\",\"vers\":\"0.1.0\"}\n",
    );
    let output = run_xtask_check_version_bump(
        &dir.0,
        &[
            "--index-base-url",
            &server.base_url,
            "--pr-body-file",
            pr_body_path.to_str().unwrap(),
        ],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "完全一致する免除宣言がある場合は EXEMPT として PASS 契約。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("result=EXEMPT"),
        "1 行サマリに result=EXEMPT が含まれる契約: {stdout}"
    );
}

#[test]
fn exempt_declaration_with_wrong_crate_name_still_fails() {
    if !curl_available() {
        eprintln!("curl が利用できない環境のためネットワーク照会シナリオをスキップする");
        return;
    }
    let dir = ScratchDir(unique_fixture_dir("exempt-mismatch"));
    init_fixture_repo(&dir.0, "0.1.0");
    commit_source_change(&dir.0, None);

    let pr_body_path = dir.0.join("pr-body.txt");
    fs::write(&pr_body_path, "version-bump-exempt: some-other-crate\n").unwrap();

    let server = FakeIndexServer::start(
        "200",
        "{\"name\":\"xtask-fixture-crate-a\",\"vers\":\"0.1.0\"}\n",
    );
    let output = run_xtask_check_version_bump(
        &dir.0,
        &[
            "--index-base-url",
            &server.base_url,
            "--pr-body-file",
            pr_body_path.to_str().unwrap(),
        ],
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "クレート名が一致しない免除宣言は無効で FAIL のまま（包括免除を許さない）契約。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("result=FAIL"),
        "1 行サマリに result=FAIL が含まれる契約: {stdout}"
    );
}

#[test]
fn missing_pr_body_file_is_a_regular_failure_not_a_panic() {
    let dir = ScratchDir(unique_fixture_dir("missing-pr-body"));
    init_fixture_repo(&dir.0, "0.1.0");
    commit_source_change(&dir.0, None);

    let output = run_xtask_check_version_bump(&dir.0, &["--pr-body-file", "does-not-exist.txt"]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "存在しない --pr-body-file の指定は fail-closed（終了コード 1）で処理される契約。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
