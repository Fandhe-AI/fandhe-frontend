//! `xtask patch-template-smoke` の CLI 契約に対する回帰テスト（イシュー #885）。
//!
//! `.github/workflows/ci.yml` の `template-app-wasm-smoke` ジョブは本テストが
//! 固定する契約（終了コード・1 行サマリ書式・`[patch.crates-io]` 注入・
//! `Cargo.lock` 削除）に依拠する（設計文書
//! `docs/ci/version-bump-publish-order-gap.md` §5）。
//!
//! 契約（`xtask/src/main.rs` の `run_patch_template_smoke` /
//! `patch_template_smoke` モジュール参照）:
//! - 終了コード 0: 全依存が crates.io で解決可能（無変更）・未公開依存への
//!   `[patch.crates-io]` フォールバック適用完了のいずれか
//! - 終了コード 1: index 到達不可・異常応答（`environment error: `
//!   プレフィックス）・既存 `[patch.crates-io]` 検出・path 依存検出・
//!   repo-root 側クレート不整合
//! - 終了コード 2: 引数不備（`--project-dir`/`--repo-root` 未指定・不明な引数）
//! - stdout の 1 行サマリ書式は
//!   `template-app-wasm-smoke: dep=<crate> version=<v> resolution=<crates-io|path-override>`
//!
//! ネットワーク照会（curl）を伴うシナリオは `cli_check_version_bump.rs` と
//! 同一パターン（`std::net::TcpListener` によるローカル擬似 sparse index +
//! `--index-base-url`）で決定的に検証する。curl 自体が runner に存在しない
//! 場合はこれらのシナリオをスキップする（`curl_available` 判定）。

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

fn curl_available() -> bool {
    Command::new("curl")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// テストごとに衝突しない一時ディレクトリを `<target>/tmp` 配下に作る
/// （`cli_check_version_bump.rs::scratch_root` と同一方針。イシュー #637）。
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
        "xtask-patch-template-smoke-{label}-{}-{nanos}-{n}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("フィクスチャディレクトリの作成に失敗した");
    dir
}

struct ScratchDir(PathBuf);

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// ローカル擬似 sparse index サーバー。クレート名ごとに固定応答を返す
/// （`cli_check_version_bump.rs::FakeIndexServer` を複数クレート対応へ拡張）。
struct FakeIndexServer {
    base_url: String,
    handle: Option<thread::JoinHandle<()>>,
}

impl FakeIndexServer {
    /// `responses`: `(crate_name, http_status, body)` の一覧。名前一致しない
    /// リクエストには 404 を返す。
    fn start(responses: Vec<(&'static str, &'static str, &'static str)>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("ローカルリスナーの起動に失敗した");
        let port = listener.local_addr().unwrap().port();
        let expected_requests = responses.len();
        let handle = thread::spawn(move || {
            for _ in 0..expected_requests {
                if let Ok((mut stream, _)) = listener.accept() {
                    let _ = handle_one_request(&mut stream, &responses);
                }
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

fn handle_one_request(
    stream: &mut TcpStream,
    responses: &[(&'static str, &'static str, &'static str)],
) -> std::io::Result<()> {
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf)?;
    let request = String::from_utf8_lossy(&buf[..n]);
    // リクエストライン（例: `GET /fa/nd/fandhe-frontend-core HTTP/1.1`）から
    // クレート名を抽出する（末尾セグメントがクレート名と一致する規約、
    // `check_version_bump::index_path` 参照）。
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("");
    let requested_name = path.rsplit('/').next().unwrap_or("");

    let (status, body) = responses
        .iter()
        .find(|(name, _, _)| *name == requested_name)
        .map(|(_, s, b)| (*s, *b))
        .unwrap_or(("404", ""));

    let reason = if status == "404" { "Not Found" } else { "OK" };
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{body}",
        len = body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

fn write_manifest(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

/// `--project-dir` 用の最小フィクスチャ（ルート Cargo.toml のみ、wasm なし）を
/// 作る。
fn init_project_fixture(project_dir: &Path, root_manifest: &str) {
    write_manifest(&project_dir.join("Cargo.toml"), root_manifest);
    write_manifest(&project_dir.join("Cargo.lock"), "# fake lock\n");
}

/// `--repo-root` 用の最小フィクスチャ（`crates/<dir>/Cargo.toml` を並べる）を
/// 作る。
fn init_repo_root_fixture(repo_root: &Path, crates: &[(&str, &str, &str)]) {
    for (name, version, dir) in crates {
        write_manifest(
            &repo_root.join("crates").join(dir).join("Cargo.toml"),
            &format!("[package]\nname = \"{name}\"\nversion = \"{version}\"\nedition = \"2021\"\n"),
        );
    }
}

fn run_patch_template_smoke(
    project_dir: &Path,
    repo_root: &Path,
    index_base_url: Option<&str>,
) -> std::process::Output {
    let mut args = vec![
        "patch-template-smoke".to_string(),
        "--project-dir".to_string(),
        project_dir.display().to_string(),
        "--repo-root".to_string(),
        repo_root.display().to_string(),
    ];
    if let Some(url) = index_base_url {
        args.push("--index-base-url".to_string());
        args.push(url.to_string());
    }
    Command::new(xtask_bin())
        .args(&args)
        .output()
        .expect("xtask バイナリの起動に失敗した")
}

#[test]
fn missing_project_dir_arg_exits_two() {
    let output = Command::new(xtask_bin())
        .args(["patch-template-smoke", "--repo-root", "."])
        .output()
        .expect("xtask バイナリの起動に失敗した");
    assert_eq!(
        output.status.code(),
        Some(2),
        "`--project-dir` 未指定は usage エラー（終了コード 2）契約"
    );
}

#[test]
fn missing_repo_root_arg_exits_two() {
    let output = Command::new(xtask_bin())
        .args(["patch-template-smoke", "--project-dir", "."])
        .output()
        .expect("xtask バイナリの起動に失敗した");
    assert_eq!(
        output.status.code(),
        Some(2),
        "`--repo-root` 未指定は usage エラー（終了コード 2）契約"
    );
}

#[test]
fn unknown_flag_exits_two() {
    let output = Command::new(xtask_bin())
        .args([
            "patch-template-smoke",
            "--project-dir",
            ".",
            "--repo-root",
            ".",
            "--bogus",
        ])
        .output()
        .expect("xtask バイナリの起動に失敗した");
    assert_eq!(
        output.status.code(),
        Some(2),
        "未知の引数は usage エラー（終了コード 2）契約"
    );
}

#[test]
fn missing_project_manifest_is_a_regular_failure() {
    let dir = ScratchDir(unique_fixture_dir("missing-manifest"));
    fs::create_dir_all(&dir.0).unwrap();
    let repo_root = dir.0.join("repo-root");
    fs::create_dir_all(&repo_root).unwrap();
    let project_dir = dir.0.join("does-not-exist");

    let output = run_patch_template_smoke(&project_dir, &repo_root, None);
    assert_eq!(
        output.status.code(),
        Some(1),
        "必須マニフェスト不在は終了コード 1 契約。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn all_deps_resolvable_on_crates_io_leaves_files_untouched() {
    if !curl_available() {
        eprintln!("curl が利用できない環境のためネットワーク照会シナリオをスキップする");
        return;
    }
    let dir = ScratchDir(unique_fixture_dir("all-resolvable"));
    let project_dir = dir.0.join("project");
    let repo_root = dir.0.join("repo-root");
    let root_manifest =
        "[package]\nname = \"fixture-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
[dependencies]\nfandhe-frontend-core = \"0.1.0\"\n";
    init_project_fixture(&project_dir, root_manifest);
    init_repo_root_fixture(&repo_root, &[("fandhe-frontend-core", "0.1.0", "core")]);

    let server = FakeIndexServer::start(vec![(
        "fandhe-frontend-core",
        "200",
        "{\"name\":\"fandhe-frontend-core\",\"vers\":\"0.1.0\"}\n",
    )]);
    let output = run_patch_template_smoke(&project_dir, &repo_root, Some(&server.base_url));
    assert_eq!(
        output.status.code(),
        Some(0),
        "全依存が crates.io で解決可能な場合は PASS 契約。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("dep=fandhe-frontend-core")
            && stdout.contains("version=0.1.0")
            && stdout.contains("resolution=crates-io"),
        "1 行サマリに resolution=crates-io が含まれる契約: {stdout}"
    );

    let manifest_after = fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
    assert_eq!(
        manifest_after, root_manifest,
        "全依存解決可能な場合はマニフェストを無変更のまま維持する契約"
    );
    assert!(
        project_dir.join("Cargo.lock").exists(),
        "全依存解決可能な場合は Cargo.lock を削除しない契約"
    );
}

#[test]
fn unresolvable_dep_gets_patch_crates_io_fallback_and_lock_removed() {
    if !curl_available() {
        eprintln!("curl が利用できない環境のためネットワーク照会シナリオをスキップする");
        return;
    }
    let dir = ScratchDir(unique_fixture_dir("unresolvable"));
    let project_dir = dir.0.join("project");
    let repo_root = dir.0.join("repo-root");
    let root_manifest =
        "[package]\nname = \"fixture-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
[dependencies]\nfandhe-frontend-core = \"0.2.0\"\n";
    init_project_fixture(&project_dir, root_manifest);
    init_repo_root_fixture(&repo_root, &[("fandhe-frontend-core", "0.2.0", "core")]);

    // index には 0.1.0 のみが存在し、要求 0.2.0 はまだ未公開（バンプ直後の
    // 想定シナリオ）。
    let server = FakeIndexServer::start(vec![(
        "fandhe-frontend-core",
        "200",
        "{\"name\":\"fandhe-frontend-core\",\"vers\":\"0.1.0\"}\n",
    )]);
    let output = run_patch_template_smoke(&project_dir, &repo_root, Some(&server.base_url));
    assert_eq!(
        output.status.code(),
        Some(0),
        "未公開依存はフォールバック適用のうえ PASS 契約。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("dep=fandhe-frontend-core")
            && stdout.contains("version=0.2.0")
            && stdout.contains("resolution=path-override"),
        "1 行サマリに resolution=path-override が含まれる契約: {stdout}"
    );

    let manifest_after = fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
    assert!(
        manifest_after.contains("[patch.crates-io]"),
        "未公開依存があれば [patch.crates-io] を注入する契約: {manifest_after}"
    );
    let expected_path = repo_root.join("crates").join("core");
    assert!(
        manifest_after.contains(&format!(
            "fandhe-frontend-core = {{ path = \"{}\" }}",
            expected_path.display()
        )),
        "[patch.crates-io] のエントリが repo-root 配下の絶対 path を指す契約: {manifest_after}"
    );
    assert!(
        !project_dir.join("Cargo.lock").exists(),
        "フォールバック発動時は Cargo.lock を削除する契約（再現性低下を許容）"
    );
}

#[test]
fn index_unreachable_is_environment_error() {
    let dir = ScratchDir(unique_fixture_dir("env-error"));
    let project_dir = dir.0.join("project");
    let repo_root = dir.0.join("repo-root");
    let root_manifest =
        "[package]\nname = \"fixture-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
[dependencies]\nfandhe-frontend-core = \"0.1.0\"\n";
    init_project_fixture(&project_dir, root_manifest);
    init_repo_root_fixture(&repo_root, &[("fandhe-frontend-core", "0.1.0", "core")]);

    // 何も listen していないポートへ向けて到達不可を発生させる。
    let output = run_patch_template_smoke(&project_dir, &repo_root, Some("http://127.0.0.1:1"));
    assert_eq!(
        output.status.code(),
        Some(1),
        "index 到達不可は終了コード 1 契約。stdout: {} stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("environment error: "),
        "index 到達不可は environment error として区別される契約: {stderr}"
    );
}

#[test]
fn existing_patch_section_is_rejected() {
    let dir = ScratchDir(unique_fixture_dir("existing-patch"));
    let project_dir = dir.0.join("project");
    let repo_root = dir.0.join("repo-root");
    let root_manifest =
        "[package]\nname = \"fixture-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
[dependencies]\nfandhe-frontend-core = \"0.1.0\"\n\n\
[patch.crates-io]\nfandhe-frontend-core = { path = \"../already-patched\" }\n";
    init_project_fixture(&project_dir, root_manifest);
    init_repo_root_fixture(&repo_root, &[("fandhe-frontend-core", "0.1.0", "core")]);

    let output = run_patch_template_smoke(&project_dir, &repo_root, None);
    assert_eq!(
        output.status.code(),
        Some(1),
        "既存の [patch.crates-io] は上書きせずエラーにする契約。stdout: {} stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn path_dependency_form_is_rejected() {
    let dir = ScratchDir(unique_fixture_dir("path-dep"));
    let project_dir = dir.0.join("project");
    let repo_root = dir.0.join("repo-root");
    let root_manifest =
        "[package]\nname = \"fixture-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
[dependencies]\nfandhe-frontend-core = { path = \"../core\" }\n";
    init_project_fixture(&project_dir, root_manifest);
    init_repo_root_fixture(&repo_root, &[("fandhe-frontend-core", "0.1.0", "core")]);

    let output = run_patch_template_smoke(&project_dir, &repo_root, None);
    assert_eq!(
        output.status.code(),
        Some(1),
        "path 依存の検出は vendor 再導入の疑いとしてエラーにする契約。stdout: {} stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn crate_not_found_in_repo_root_is_a_regular_failure() {
    if !curl_available() {
        eprintln!("curl が利用できない環境のためネットワーク照会シナリオをスキップする");
        return;
    }
    let dir = ScratchDir(unique_fixture_dir("crate-not-found"));
    let project_dir = dir.0.join("project");
    let repo_root = dir.0.join("repo-root");
    let root_manifest =
        "[package]\nname = \"fixture-app\"\nversion = \"0.2.0\"\nedition = \"2021\"\n\n\
[dependencies]\nfandhe-frontend-core = \"0.2.0\"\n";
    init_project_fixture(&project_dir, root_manifest);
    // repo-root 側に該当クレートを一切置かない。
    fs::create_dir_all(repo_root.join("crates")).unwrap();

    let server = FakeIndexServer::start(vec![(
        "fandhe-frontend-core",
        "200",
        "{\"name\":\"fandhe-frontend-core\",\"vers\":\"0.1.0\"}\n",
    )]);
    let output = run_patch_template_smoke(&project_dir, &repo_root, Some(&server.base_url));
    assert_eq!(
        output.status.code(),
        Some(1),
        "repo-root にクレートが見つからない場合は終了コード 1 契約。stdout: {} stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn wasm_manifest_is_processed_when_present() {
    if !curl_available() {
        eprintln!("curl が利用できない環境のためネットワーク照会シナリオをスキップする");
        return;
    }
    let dir = ScratchDir(unique_fixture_dir("wasm-present"));
    let project_dir = dir.0.join("project");
    let repo_root = dir.0.join("repo-root");
    let root_manifest =
        "[package]\nname = \"fixture-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
[dependencies]\nfandhe-frontend-core = \"0.1.0\"\n";
    init_project_fixture(&project_dir, root_manifest);
    let wasm_manifest =
        "[package]\nname = \"fixture-app-wasm\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
[dependencies]\nfandhe-frontend-wasm-client = \"0.3.0\"\n";
    write_manifest(&project_dir.join("wasm").join("Cargo.toml"), wasm_manifest);
    write_manifest(
        &project_dir.join("wasm").join("Cargo.lock"),
        "# fake wasm lock\n",
    );
    init_repo_root_fixture(
        &repo_root,
        &[
            ("fandhe-frontend-core", "0.1.0", "core"),
            ("fandhe-frontend-wasm-client", "0.3.0", "wasm-client"),
        ],
    );

    let server = FakeIndexServer::start(vec![
        (
            "fandhe-frontend-core",
            "200",
            "{\"name\":\"fandhe-frontend-core\",\"vers\":\"0.1.0\"}\n",
        ),
        (
            "fandhe-frontend-wasm-client",
            "200",
            "{\"name\":\"fandhe-frontend-wasm-client\",\"vers\":\"0.2.0\"}\n",
        ),
    ]);
    let output = run_patch_template_smoke(&project_dir, &repo_root, Some(&server.base_url));
    assert_eq!(
        output.status.code(),
        Some(0),
        "wasm/Cargo.toml も処理対象になる契約。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("dep=fandhe-frontend-core") && stdout.contains("resolution=crates-io"),
        "root 側 dep のサマリが含まれる契約: {stdout}"
    );
    assert!(
        stdout.contains("dep=fandhe-frontend-wasm-client")
            && stdout.contains("resolution=path-override"),
        "wasm 側 dep（未公開）のサマリが含まれる契約: {stdout}"
    );

    let wasm_manifest_after =
        fs::read_to_string(project_dir.join("wasm").join("Cargo.toml")).unwrap();
    assert!(
        wasm_manifest_after.contains("[patch.crates-io]"),
        "wasm 側マニフェストにフォールバックが注入される契約: {wasm_manifest_after}"
    );
    assert!(
        !project_dir.join("wasm").join("Cargo.lock").exists(),
        "wasm 側 Cargo.lock も削除される契約"
    );
    // root 側は未公開依存がないため無変更・lock 維持。
    let root_manifest_after = fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
    assert_eq!(root_manifest_after, root_manifest);
    assert!(project_dir.join("Cargo.lock").exists());
}
