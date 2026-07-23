//! `examples/ssg-blog` の integration test（イシュー #501）。
//!
//! `src/main.rs` はバイナリクレートのため本ファイルからは内部関数を `use`
//! できない。`examples/ssr-routing/tests/routing.rs` と同じ二本立てで検証する
//! （依存クレートの公開 API のみを使う点で利用者向けサンプルとしての実演性は
//! 変わらない）:
//!
//! 1. ライブラリ直接検証: `fandhe_frontend_server::ssg::generate_pages` を
//!    利用者と同じ形（`(String, Node)` 列）で直接呼び、既定エスケープ・
//!    fail-closed 検証・重複拒否の回帰を固定する。
//! 2. CLI ブラックボックス検証: ビルド済みバイナリを
//!    `env!("CARGO_BIN_EXE_fandhe-frontend-example-ssg-blog")` でサブプロセス
//!    起動し、`dist/` の生成結果を確認する。

use fandhe_frontend_core::{el, text};
use fandhe_frontend_server::ssg::{generate_pages, SsgError};
use std::path::PathBuf;
use std::process::Command;

/// テスト専用の一時ディレクトリ。`Drop` でベストエフォート削除する
/// （`crates/server/tests/support/temp_dir.rs` と同じ方針。`tempfile` 等の
/// 外部クレートを追加しない、REQ-3）。
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        // cargo が `CARGO_TARGET_TMPDIR` を設定するのはテストバイナリの
        // コンパイル時のみ（Cargo Book）であり、実行時 `std::env::var` 参照は
        // 常に失敗する。既定はコンパイル時に確定する
        // `env!("CARGO_TARGET_TMPDIR")`（`<target>/tmp` 配下。本サンプルは
        // root workspace から意図的に切り離された独立 `[workspace]` のため、
        // ここでの `<target>` は `examples/ssg-blog/target`）を使い、`/tmp`
        // へは一切フォールバックしない（イシュー #637/#658）。実行時 env
        // による明示上書きは引き続き許容する。
        let root = std::env::var("CARGO_TARGET_TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
        let _ = std::fs::create_dir_all(&root);
        let path = root.join(format!(
            "fandhe-frontend-example-ssg-blog-test-{tag}-{}-{unique}",
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

/// 正規化後の出力先重複（`/a` と `/a/` はいずれも `a/index.html`）は
/// `SsgError::DuplicatePagePath` として拒否されることを固定する。
#[test]
fn generate_pages_rejects_duplicate_normalized_paths() {
    let tmp = TempDir::new("duplicate-path");
    let pages = vec![
        ("/a".to_string(), el("html", vec![], vec![])),
        ("/a/".to_string(), el("html", vec![], vec![])),
    ];

    let result = generate_pages(&pages, &tmp.0);
    assert!(
        matches!(result, Err(SsgError::DuplicatePagePath(_))),
        "expected DuplicatePagePath, got {result:?}"
    );
}

// --- 2. CLI ブラックボックス検証 ---

/// `src/main.rs` のバイナリを一意な一時ディレクトリを `current_dir` として
/// 起動し、生成された `dist/` を含むディレクトリのパスを返す。
fn run_cli_in_scratch_dir(tag: &str) -> TempDir {
    let scratch = TempDir::new(tag);
    std::fs::create_dir_all(&scratch.0).expect("failed to create scratch dir");

    let output = Command::new(env!("CARGO_BIN_EXE_fandhe-frontend-example-ssg-blog"))
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

/// 受け入れ条件 1: `cargo run` で `dist/` に静的サイトが生成されることを、
/// 生成ファイル集合の固定（過不足なし）で断定する。
#[test]
fn cli_generates_expected_dist_files() {
    let scratch = run_cli_in_scratch_dir("dist-files");
    let dist = scratch.0.join("dist");

    assert!(dist.join("index.html").is_file());
    for slug in ["hello-ssg", "default-escaping", "view-transitions"] {
        assert!(
            dist.join("posts").join(slug).join("index.html").is_file(),
            "dist/posts/{slug}/index.html should exist"
        );
    }

    // 過不足なし: index.html 1 件 + posts/<slug>/index.html 3 件 = 4 ファイル。
    let mut file_count = 0usize;
    fn count_files(dir: &std::path::Path, count: &mut usize) {
        for entry in std::fs::read_dir(dir).expect("dist dir should be readable") {
            let entry = entry.expect("dir entry should be readable");
            let path = entry.path();
            if path.is_dir() {
                count_files(&path, count);
            } else {
                *count += 1;
            }
        }
    }
    count_files(&dist, &mut file_count);
    assert_eq!(file_count, 4, "dist/ should contain exactly 4 files");
}

/// 全ページに `@view-transition { navigation: auto; }` が含まれることを
/// 固定する（layout() が全ページ共通で出力する契約）。
#[test]
fn cli_output_includes_view_transition_style() {
    let scratch = run_cli_in_scratch_dir("view-transition");
    let dist = scratch.0.join("dist");

    for rel in [
        "index.html",
        "posts/hello-ssg/index.html",
        "posts/default-escaping/index.html",
        "posts/view-transitions/index.html",
    ] {
        let body = std::fs::read_to_string(dist.join(rel))
            .unwrap_or_else(|e| panic!("{rel} should be readable: {e}"));
        assert!(
            body.contains("<style>@view-transition { navigation: auto; }</style>"),
            "{rel} should include the view-transition style"
        );
        assert!(body.starts_with("<!DOCTYPE html>"));
    }
}

/// 既定エスケープ回帰（REQ-1）: XSS ペイロードタイトルを持つ記事の出力に
/// `&lt;script&gt;` が含まれ、生の `<script>` は含まれないことを CLI 経由で
/// 固定する。`layout()` は `<script>` タグを一切出力しないため、後者の断定は
/// 「タイトルの実体参照化」のみを検出する。
#[test]
fn cli_escapes_xss_payload_title() {
    let scratch = run_cli_in_scratch_dir("xss-title");
    let body = std::fs::read_to_string(scratch.0.join("dist/posts/default-escaping/index.html"))
        .expect("default-escaping post should be readable");

    assert!(!body.contains("<script>alert"));
    assert!(body.contains("&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"));
}
