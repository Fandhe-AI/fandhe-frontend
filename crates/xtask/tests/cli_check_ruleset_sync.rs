//! `xtask check-ruleset-sync` の CLI 契約に対する回帰テスト（イシュー #2325）。
//!
//! `.github/workflows/ci.yml` の `dep-version-check` ジョブは本テストが
//! 固定する契約（終了コード・1 行サマリ書式）に依拠する。
//!
//! 契約（`xtask/src/main.rs` の `run_check_ruleset_sync` /
//! `check_ruleset_sync::format_entry_line`/`format_strict_line` 参照）:
//! - 終了コード 0: マニフェストと live ruleset が完全一致し、
//!   `strict_required_status_checks_policy` が `false`
//! - 終了コード 1: 1 件でも `missing-in-ruleset`/`extra-in-ruleset`/
//!   `integration-id-mismatch`・`strict=true`・環境エラー（fail-closed）
//! - 終了コード 2: 引数不備（`--manifest`/`--repo`/`--branch`/
//!   `--api-base-url` の値欠落・未知の引数）
//! - stdout の 1 行サマリ書式は
//!   `ruleset-sync: context=<context> integration_id=<n> result=<PASS|FAIL:...>`
//!   と末尾 `ruleset-sync: strict=<bool> result=<PASS|FAIL>`
//!
//! ネットワーク照会（curl）を伴うシナリオは、`std::net::TcpListener` による
//! ローカル擬似 GitHub API サーバーへ `--api-base-url` で向けることで、実
//! GitHub API への到達性なしに決定的に検証する（`cli_check_version_bump.rs`
//! の `FakeIndexServer` と同一方針）。curl 自体が runner に存在しない場合は
//! これらのシナリオを実行できないため、同ファイルと同じ環境判定パターンで
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

fn xtask_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xtask")
}

/// runner に curl が存在するかどうか（`cli_check_version_bump.rs` と同一方針）。
fn curl_available() -> bool {
    Command::new("curl")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// テストごとに衝突しない一時ディレクトリを `<target>/tmp` 配下に作る
/// （`cli_check_version_bump.rs::scratch_root` と同一方針。イシュー #637/#658）。
fn scratch_root() -> PathBuf {
    let root = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
    let _ = std::fs::create_dir_all(&root);
    root
}

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_manifest_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = FIXTURE_COUNTER.fetch_add(1, Ordering::SeqCst);
    scratch_root().join(format!(
        "xtask-check-ruleset-sync-{label}-{}-{nanos}-{n}.json",
        std::process::id()
    ))
}

fn write_manifest(context_and_ids: &[(&str, i64)], strict: bool) -> PathBuf {
    let path = unique_manifest_path("manifest");
    let entries: Vec<String> = context_and_ids
        .iter()
        .map(|(c, id)| format!("{{\"context\": \"{c}\", \"integration_id\": {id}}}"))
        .collect();
    let body = format!(
        "{{\"ruleset\": \"main-protection\", \"strict_required_status_checks_policy\": {strict}, \"required_status_checks\": [{}]}}",
        entries.join(", ")
    );
    fs::write(&path, body).expect("マニフェストフィクスチャの書き込みに失敗した");
    path
}

/// ローカル擬似 GitHub API サーバー。単一パスへの単一リクエストのみを
/// 処理し、指定した HTTP ステータス・ボディを返して終了する
/// （`cli_check_version_bump.rs::FakeIndexServer` と同一方針）。
struct FakeGithubServer {
    base_url: String,
    handle: Option<thread::JoinHandle<()>>,
}

impl FakeGithubServer {
    fn start(status: &'static str, body: String) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("ローカルリスナーの起動に失敗した");
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let _ = handle_one_request(&mut stream, status, &body);
            }
        });
        FakeGithubServer {
            base_url: format!("http://127.0.0.1:{port}"),
            handle: Some(handle),
        }
    }
}

impl Drop for FakeGithubServer {
    fn drop(&mut self) {
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

fn handle_one_request(stream: &mut TcpStream, status: &str, body: &str) -> std::io::Result<()> {
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

/// live ruleset に `required_status_checks` 型 rule 1 件・
/// 指定 context 一覧を持つレスポンス本文を組み立てる。
fn live_rules_body(context_and_ids: &[(&str, i64)], strict: bool) -> String {
    let entries: Vec<String> = context_and_ids
        .iter()
        .map(|(c, id)| format!("{{\"context\": \"{c}\", \"integration_id\": {id}}}"))
        .collect();
    format!(
        "[{{\"type\": \"deletion\"}}, {{\"type\": \"required_status_checks\", \"parameters\": \
{{\"strict_required_status_checks_policy\": {strict}, \"required_status_checks\": [{}]}}}}]",
        entries.join(", ")
    )
}

fn run_xtask_check_ruleset_sync(manifest: &Path, extra_args: &[&str]) -> std::process::Output {
    let manifest_str = manifest.to_str().expect("manifest パスが UTF-8 でない");
    let mut args = vec!["check-ruleset-sync", "--manifest", manifest_str];
    args.extend_from_slice(extra_args);
    Command::new(xtask_bin())
        .args(&args)
        .output()
        .expect("xtask バイナリの起動に失敗した")
}

#[test]
fn unknown_flag_exits_two() {
    let manifest = write_manifest(&[("ci-complete", 15368)], false);
    let output = run_xtask_check_ruleset_sync(&manifest, &["--bogus"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "未知の引数は usage エラー（終了コード 2）契約"
    );
    let _ = fs::remove_file(&manifest);
}

#[test]
fn missing_manifest_value_exits_two() {
    let output = Command::new(xtask_bin())
        .args(["check-ruleset-sync", "--manifest"])
        .output()
        .expect("xtask バイナリの起動に失敗した");
    assert_eq!(
        output.status.code(),
        Some(2),
        "`--manifest` の値欠落は usage エラー（終了コード 2）契約"
    );
}

#[test]
fn missing_manifest_file_exits_one() {
    let missing = scratch_root().join("does-not-exist.json");
    let output = run_xtask_check_ruleset_sync(&missing, &[]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "マニフェストファイル不在は終了コード 1"
    );
}

#[test]
fn identical_manifest_and_live_ruleset_exits_zero() {
    if !curl_available() {
        eprintln!("curl 不在のため本シナリオをスキップする");
        return;
    }
    let manifest = write_manifest(&[("ci-complete", 15368), ("Cursor Bugbot", 1210556)], false);
    let server = FakeGithubServer::start(
        "200",
        live_rules_body(&[("ci-complete", 15368), ("Cursor Bugbot", 1210556)], false),
    );
    let output = run_xtask_check_ruleset_sync(&manifest, &["--api-base-url", &server.base_url]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code(),
        Some(0),
        "一致時は終了コード 0: stdout={stdout} stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("ruleset-sync: context=ci-complete integration_id=15368 result=PASS"),
        "1 行サマリ契約: {stdout}"
    );
    assert!(
        stdout.contains("ruleset-sync: strict=false result=PASS"),
        "strict 行契約: {stdout}"
    );
    let _ = fs::remove_file(&manifest);
}

#[test]
fn missing_in_ruleset_exits_one() {
    if !curl_available() {
        eprintln!("curl 不在のため本シナリオをスキップする");
        return;
    }
    let manifest = write_manifest(
        &[("ci-complete", 15368), ("extra-only-in-manifest", 15368)],
        false,
    );
    let server = FakeGithubServer::start("200", live_rules_body(&[("ci-complete", 15368)], false));
    let output = run_xtask_check_ruleset_sync(&manifest, &["--api-base-url", &server.base_url]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stdout.contains(
            "ruleset-sync: context=extra-only-in-manifest integration_id=15368 result=FAIL:missing-in-ruleset"
        ),
        "missing-in-ruleset 契約: {stdout}"
    );
    let _ = fs::remove_file(&manifest);
}

#[test]
fn extra_in_ruleset_exits_one() {
    if !curl_available() {
        eprintln!("curl 不在のため本シナリオをスキップする");
        return;
    }
    let manifest = write_manifest(&[("ci-complete", 15368)], false);
    let server = FakeGithubServer::start(
        "200",
        live_rules_body(
            &[("ci-complete", 15368), ("new-job-not-in-manifest", 15368)],
            false,
        ),
    );
    let output = run_xtask_check_ruleset_sync(&manifest, &["--api-base-url", &server.base_url]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stdout.contains(
            "ruleset-sync: context=new-job-not-in-manifest integration_id=15368 result=FAIL:extra-in-ruleset"
        ),
        "extra-in-ruleset 契約: {stdout}"
    );
    let _ = fs::remove_file(&manifest);
}

#[test]
fn integration_id_mismatch_exits_one() {
    if !curl_available() {
        eprintln!("curl 不在のため本シナリオをスキップする");
        return;
    }
    let manifest = write_manifest(&[("ci-complete", 15368)], false);
    let server = FakeGithubServer::start("200", live_rules_body(&[("ci-complete", 9999)], false));
    let output = run_xtask_check_ruleset_sync(&manifest, &["--api-base-url", &server.base_url]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stdout.contains(
            "ruleset-sync: context=ci-complete integration_id=9999 result=FAIL:integration-id-mismatch"
        ),
        "integration-id-mismatch 契約: {stdout}"
    );
    let _ = fs::remove_file(&manifest);
}

#[test]
fn strict_true_exits_one() {
    if !curl_available() {
        eprintln!("curl 不在のため本シナリオをスキップする");
        return;
    }
    let manifest = write_manifest(&[("ci-complete", 15368)], false);
    let server = FakeGithubServer::start("200", live_rules_body(&[("ci-complete", 15368)], true));
    let output = run_xtask_check_ruleset_sync(&manifest, &["--api-base-url", &server.base_url]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stdout.contains("ruleset-sync: strict=true result=FAIL"),
        "strict=true は FAIL 契約: {stdout}"
    );
    let _ = fs::remove_file(&manifest);
}

#[test]
fn unreachable_api_base_url_is_environment_error() {
    if !curl_available() {
        eprintln!("curl 不在のため本シナリオをスキップする");
        return;
    }
    let manifest = write_manifest(&[("ci-complete", 15368)], false);
    // 127.0.0.1 の未使用ポートへ接続を試みて即座に失敗させる（接続拒否）。
    let output = run_xtask_check_ruleset_sync(&manifest, &["--api-base-url", "http://127.0.0.1:1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr.contains("environment error: "),
        "到達不可は environment error として区別される契約: {stderr}"
    );
    let _ = fs::remove_file(&manifest);
}

#[test]
fn no_required_status_checks_rule_is_environment_error() {
    if !curl_available() {
        eprintln!("curl 不在のため本シナリオをスキップする");
        return;
    }
    let manifest = write_manifest(&[("ci-complete", 15368)], false);
    let server = FakeGithubServer::start("200", "[{\"type\": \"deletion\"}]".to_string());
    let output = run_xtask_check_ruleset_sync(&manifest, &["--api-base-url", &server.base_url]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr.contains("environment error: "),
        "required_status_checks rule 0 件は environment error として区別される契約: {stderr}"
    );
    let _ = fs::remove_file(&manifest);
}
