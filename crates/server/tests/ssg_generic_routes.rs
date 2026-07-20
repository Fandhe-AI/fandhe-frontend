//! `fandhe_frontend_server::ssg::generate_pages`（汎用ルート SSG API、イシュー #463）の
//! integration test。
//!
//! `generate`/`generate_with`（固定ルート表）とは別系統の任意パス書き出し
//! 経路であるため、既存 `ssr_ssg_parity.rs`（REQ-6 の SSR/SSG バイト完全
//! 一致テスト）とは独立に検証する。ここでは以下を担保する:
//!
//! - 任意階層パスの書き出し（受け入れ条件 1）
//! - `..`・絶対パス欠如・空セグメントの拒否と fail-closed（受け入れ条件 2）
//! - 既存 `generate()`/`generate_with()` 出力との非干渉（受け入れ条件 3）
//! - `render()` 経由の既定エスケープ（XSS 回帰、REQ-1）

use fandhe_frontend_core::{el, text};
use fandhe_frontend_server::ssg::{generate, generate_pages, SsgError};
use std::fs;

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/support/temp_dir.rs"
));

fn page(title: &str) -> fandhe_frontend_core::Node {
    el("html", vec![], vec![el("body", vec![], vec![text(title)])])
}

#[test]
fn generate_pages_writes_multi_level_path() {
    let dir = TempDir::new("multi-level");
    let node = page("Guide Foo");
    let written = generate_pages(&[("/guide/foo/".to_string(), node)], &dir.0)
        .expect("multi-level page should be written");

    assert_eq!(written.len(), 1);
    let expected_path = dir.0.join("guide/foo/index.html");
    assert_eq!(written[0], expected_path);

    let body = fs::read_to_string(&expected_path).unwrap();
    let expected_body = format!(
        "<!DOCTYPE html>\n{}",
        fandhe_frontend_core::render(&page("Guide Foo"))
    );
    assert_eq!(body, expected_body);
}

#[test]
fn generate_pages_normalizes_root_and_trailing_slash_variants() {
    let dir = TempDir::new("normalize");
    let written = generate_pages(
        &[
            ("/".to_string(), page("Home")),
            ("/about".to_string(), page("About")),
        ],
        &dir.0,
    )
    .expect("root and no-trailing-slash paths should be written");

    assert_eq!(written.len(), 2);
    assert!(dir.0.join("index.html").exists());
    assert!(dir.0.join("about/index.html").exists());
}

#[test]
fn generate_pages_rejects_unsafe_paths_and_writes_nothing() {
    let cases = [
        "guide/foo",       // 先頭 / なし
        "/../etc/passwd",  // .. トラバーサル
        "//",              // 空セグメント
        "/guide/foo\\bar", // バックスラッシュ
    ];

    for input in cases {
        let dir = TempDir::new("unsafe-path");
        let err = generate_pages(&[(input.to_string(), page("X"))], &dir.0)
            .expect_err(&format!("{input:?} should be rejected"));
        assert!(
            matches!(err, SsgError::UnsafePagePath(_)),
            "expected UnsafePagePath for {input:?}, got {err:?}"
        );
        assert!(
            fs::read_dir(&dir.0)
                .map(|mut entries| entries.next().is_none())
                .unwrap_or(true),
            "no file should be written when {input:?} is rejected"
        );
    }
}

#[test]
fn generate_pages_rejects_duplicate_output_paths_and_writes_nothing() {
    let dir = TempDir::new("duplicate");
    let err = generate_pages(
        &[
            ("/a".to_string(), page("A1")),
            ("/a/".to_string(), page("A2")),
        ],
        &dir.0,
    )
    .expect_err("/a and /a/ should collide on the same output file");

    assert!(
        matches!(err, SsgError::DuplicatePagePath(_)),
        "expected DuplicatePagePath, got {err:?}"
    );
    assert!(
        fs::read_dir(&dir.0)
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(true),
        "no file should be written when paths collide"
    );
}

/// XSS 回帰（REQ-1）: `text()` に渡した `<script>` はエスケープ済みで
/// 出力され、生の `<script>` タグとしては現れない。
#[test]
fn generate_pages_escapes_text_content() {
    let dir = TempDir::new("xss");
    let node = el("div", vec![], vec![text("<script>alert(1)</script>")]);
    let written =
        generate_pages(&[("/xss".to_string(), node)], &dir.0).expect("page should be written");

    let body = fs::read_to_string(&written[0]).unwrap();
    assert!(!body.contains("<script>alert"));
    assert!(body.contains("&lt;script&gt;"));
}

/// 受け入れ条件 3: 同一 `out_dir` に既存 `generate()`（固定ルート表）を
/// 実行した後で `generate_pages()`（任意パス）を実行しても、既存出力の
/// 内容が変化しない（非干渉）。
#[test]
fn generate_pages_does_not_disturb_existing_generate_output() {
    let dir = TempDir::new("non-interference");
    generate(&dir.0).expect("generate should succeed");
    let index_before = fs::read_to_string(dir.0.join("index.html")).unwrap();

    generate_pages(&[("/guide/foo/".to_string(), page("Guide"))], &dir.0)
        .expect("generate_pages should succeed alongside existing output");

    let index_after = fs::read_to_string(dir.0.join("index.html")).unwrap();
    assert_eq!(index_before, index_after);
    assert!(dir.0.join("guide/foo/index.html").exists());
}
