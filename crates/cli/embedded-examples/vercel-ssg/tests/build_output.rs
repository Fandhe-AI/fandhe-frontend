//! `examples/vercel-ssg` の integration test（イシュー #3290）。
//!
//! `src/main.rs` はバイナリクレートのため本ファイルからは内部関数を `use`
//! できない。`examples/ssg-blog/tests/ssg_output.rs` と同じ二本立てで
//! 検証する:
//!
//! 1. ライブラリ直接検証: `fandhe_frontend_server::ssg::generate_pages` /
//!    `generate_assets` を利用者と同じ形で直接呼び、既定エスケープ・
//!    fail-closed 検証の回帰を固定する。
//! 2. CLI ブラックボックス検証: ビルド済みバイナリを
//!    `env!("CARGO_BIN_EXE_fandhe-frontend-example-vercel-ssg")` で
//!    サブプロセス起動し、`.vercel/output/` の生成結果を確認する。

use fandhe_frontend_core::{el, render, text};
use fandhe_frontend_server::ssg::{generate_assets, generate_pages, SsgError};
use std::path::PathBuf;
use std::process::Command;

/// テスト専用の一時ディレクトリ。`Drop` でベストエフォート削除する
/// （`examples/ssg-blog/tests/ssg_output.rs::TempDir` と同じ方針。`tempfile`
/// 等の外部クレートを追加しない、REQ-3）。
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        // `CARGO_TARGET_TMPDIR` はコンパイル時にのみ設定される（Cargo Book）
        // ため、実行時 `std::env::var` は使わず `env!` を既定にする
        // （examples/ssg-blog と同じ理由、イシュー #637/#658）。
        let root = std::env::var("CARGO_TARGET_TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
        let _ = std::fs::create_dir_all(&root);
        let path = root.join(format!(
            "fandhe-frontend-example-vercel-ssg-test-{tag}-{}-{unique}",
            std::process::id()
        ));
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

// --- 1. ライブラリ直接検証 ---

/// 既定エスケープ回帰（REQ-1）: `<script>` を含むタイトルが実体参照化されて
/// 出力され、生の `<script>` タグとしては現れないことを固定する。
#[test]
fn generate_pages_escapes_text_content() {
    let tmp = TempDir::new("escape");
    let node = el("html", vec![], vec![text("<script>alert('xss')</script>")]);

    let written = generate_pages(&[("/".to_string(), node)], &tmp.0)
        .expect("valid single page should generate successfully");
    assert_eq!(written.len(), 1);

    let body = std::fs::read_to_string(&written[0]).expect("generated file should be readable");
    assert!(!body.contains("<script>alert"));
    assert!(body.contains("&lt;script&gt;"));
}

/// fail-closed 回帰: 不正なページパス（`..` を含む）が 1 件でも混ざると
/// `SsgError::UnsafePagePath` を返し、他の正当なページも含めて 1 つも
/// 書き出さないことを固定する。
#[test]
fn generate_pages_rejects_unsafe_path_without_partial_writes() {
    let tmp = TempDir::new("unsafe-path");
    let pages = vec![
        ("/ok/".to_string(), el("html", vec![], vec![])),
        ("/../etc".to_string(), el("html", vec![], vec![])),
    ];

    let result = generate_pages(&pages, &tmp.0);
    assert!(
        matches!(result, Err(SsgError::UnsafePagePath(_))),
        "expected UnsafePagePath, got {result:?}"
    );
    assert!(
        !tmp.0.exists(),
        "no files should be written when any page path fails validation (fail-closed)"
    );
}

/// 404 本文（`generate_assets` 経由）も `render` を経由する限り既定
/// エスケープが適用されることを固定する（`src/main.rs::not_found_asset` の
/// 契約を単独 API 呼び出しで再現する）。
#[test]
fn rendered_not_found_body_is_escaped() {
    let node = el("html", vec![], vec![text("<script>alert('xss')</script>")]);
    let body = format!("<!DOCTYPE html>\n{}", render(&node));
    assert!(!body.contains("<script>alert"));
    assert!(body.contains("&lt;script&gt;"));
}

/// `generate_assets` の無加工書き出し contract 回帰: `config.json` のような
/// 静的コンテンツがバイト無加工で書かれることを固定する。
#[test]
fn generate_assets_writes_content_verbatim_without_escaping() {
    let tmp = TempDir::new("assets-verbatim");
    let raw = r#"{"version": 3, "routes": []}"#;
    let assets = vec![("/config.json".to_string(), raw.to_string())];

    let written =
        generate_assets(&assets, &tmp.0).expect("valid single asset should write successfully");
    assert_eq!(written.len(), 1);

    let body = std::fs::read_to_string(&written[0]).expect("written asset should be readable");
    assert_eq!(
        body, raw,
        "generate_assets must write content byte-for-byte without default escaping"
    );
}

// --- 2. CLI ブラックボックス検証 ---

/// `src/main.rs` のバイナリを一意な一時ディレクトリを `current_dir` として
/// 起動し、生成された `.vercel/output/` を含むディレクトリのパスを返す。
fn run_cli_in_scratch_dir(tag: &str) -> TempDir {
    let scratch = TempDir::new(tag);
    std::fs::create_dir_all(&scratch.0).expect("failed to create scratch dir");

    let output = Command::new(env!("CARGO_BIN_EXE_fandhe-frontend-example-vercel-ssg"))
        .current_dir(&scratch.0)
        .output()
        .expect("binary should spawn and run to completion");
    assert!(
        output.status.success(),
        "CLI should exit 0: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    scratch
}

/// 受け入れ条件 1: `cargo run` で Build Output API 形式の `.vercel/output/`
/// が生成されることを固定する（`config.json` + `static/` 以下の HTML 群）。
#[test]
fn cli_generates_expected_output_tree() {
    let scratch = run_cli_in_scratch_dir("output-tree");
    let output = scratch.0.join(".vercel/output");

    assert!(output.join("config.json").is_file());
    let static_dir = output.join("static");
    assert!(static_dir.join("index.html").is_file());
    assert!(static_dir.join("404.html").is_file());
    for slug in ["about", "default-escaping"] {
        assert!(
            static_dir
                .join("pages")
                .join(slug)
                .join("index.html")
                .is_file(),
            "static/pages/{slug}/index.html should exist"
        );
    }
}

/// 受け入れ条件 2: `config.json` の `routes` によってファイルシステムに
/// 一致しないパスへ 404 ステータスで `/404.html` を返す設定が含まれることを
/// 固定する。
#[test]
fn cli_config_json_declares_filesystem_fallback_to_404() {
    let scratch = run_cli_in_scratch_dir("config-404");
    let config = std::fs::read_to_string(scratch.0.join(".vercel/output/config.json"))
        .expect("config.json should be readable");

    assert!(config.contains("\"version\": 3"));
    assert!(config.contains("\"handle\": \"filesystem\""));
    assert!(config.contains("\"status\": 404"));
    assert!(config.contains("\"dest\": \"/404.html\""));
}

/// 古い出力の削除（OWASP A05）: 事前に置いた古いファイルが実行後に消え、
/// `.vercel/project.json`（`vercel link` 相当のダミー）は残ることを固定する。
#[test]
fn cli_removes_stale_output_but_keeps_vercel_project_files() {
    let scratch = TempDir::new("stale-cleanup");
    std::fs::create_dir_all(&scratch.0).expect("failed to create scratch dir");
    std::fs::create_dir_all(scratch.0.join(".vercel/output/static")).unwrap();
    std::fs::write(
        scratch.0.join(".vercel/output/static/stale.html"),
        "<html>stale</html>",
    )
    .unwrap();
    std::fs::write(scratch.0.join(".vercel/project.json"), "{}").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_fandhe-frontend-example-vercel-ssg"))
        .current_dir(&scratch.0)
        .output()
        .expect("binary should spawn and run to completion");
    assert!(output.status.success());

    assert!(
        !scratch.0.join(".vercel/output/static/stale.html").exists(),
        "stale output should be removed before regeneration"
    );
    assert!(
        scratch.0.join(".vercel/project.json").is_file(),
        ".vercel/project.json (vercel link output) must not be deleted"
    );
}

/// XSS 回帰（REQ-1）: `default-escaping` ページのタイトルが既定エスケープ
/// され、生の `<script>` を含まないことを CLI 経由で固定する。
#[test]
fn cli_escapes_xss_payload_title() {
    let scratch = run_cli_in_scratch_dir("xss-title");
    let body = std::fs::read_to_string(
        scratch
            .0
            .join(".vercel/output/static/pages/default-escaping/index.html"),
    )
    .expect("default-escaping page should be readable");

    assert!(!body.contains("<script>alert"));
    assert!(body.contains("&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"));
}

/// 全ページに `@view-transition { navigation: auto; }` が含まれることを
/// 固定する（`layout()` が全ページ共通で出力する契約）。
#[test]
fn cli_output_includes_view_transition_style() {
    let scratch = run_cli_in_scratch_dir("view-transition");
    let output = scratch.0.join(".vercel/output/static");

    for rel in [
        "index.html",
        "pages/about/index.html",
        "pages/default-escaping/index.html",
        "404.html",
    ] {
        let body = std::fs::read_to_string(output.join(rel))
            .unwrap_or_else(|e| panic!("{rel} should be readable: {e}"));
        assert!(
            body.contains("<style>@view-transition { navigation: auto; }</style>"),
            "{rel} should include the view-transition style"
        );
        assert!(body.starts_with("<!DOCTYPE html>"));
    }
}

#[cfg(unix)]
/// fail-closed 回帰: `.vercel/output` がシンボリックリンクの場合は非ゼロ
/// 終了し、リンク先を削除しないことを固定する（unix 限定）。
#[test]
fn cli_refuses_to_clean_symlinked_output_dir() {
    use std::os::unix::fs::symlink;

    let scratch = TempDir::new("symlink-guard");
    std::fs::create_dir_all(&scratch.0).expect("failed to create scratch dir");
    let real_target = scratch.0.join("real-target");
    std::fs::create_dir_all(&real_target).unwrap();
    std::fs::write(real_target.join("marker.txt"), "keep-me").unwrap();
    std::fs::create_dir_all(scratch.0.join(".vercel")).unwrap();
    symlink(&real_target, scratch.0.join(".vercel/output")).expect("symlink should be created");

    let output = Command::new(env!("CARGO_BIN_EXE_fandhe-frontend-example-vercel-ssg"))
        .current_dir(&scratch.0)
        .output()
        .expect("binary should spawn and run to completion");

    assert!(
        !output.status.success(),
        "CLI should refuse to run when .vercel/output is a symlink"
    );
    assert!(
        real_target.join("marker.txt").is_file(),
        "symlink target must not be deleted"
    );
}
