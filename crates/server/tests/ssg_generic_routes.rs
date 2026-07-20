//! `fandhe_frontend_server::ssg::generate_pages`（汎用ルート SSG API、イシュー #463）の
//! integration test。
//!
//! `generate`/`generate_with`（固定ルート表）とは別系統の任意パス書き出し
//! 経路であるため、既存 `ssr_ssg_parity.rs`（REQ-6 の SSR/SSG バイト完全
//! 一致テスト）とは独立に検証する。ここでは以下を担保する:
//!
//! - 任意階層パスの書き出し（受け入れ条件 1）
//! - `..`・絶対パス欠如・空セグメントの拒否と fail-closed（受け入れ条件 2）
//! - 既存 `generate()`/`generate_with()` 出力との非干渉（受け入れ条件 3、
//!   `generate()` → `generate_pages()` の順に加え、逆順
//!   `generate_pages()` → `generate()` でも破壊されないことを固定）
//! - `render()` 経由の既定エスケープ（XSS 回帰、REQ-1）
//! - 空入力（`Ok(vec![])` かつ書き出しなし）・決定性（同一入力の再実行で
//!   バイト列が変化しないこと）
//!
//! # 本ファイルの成立ちについて（イシュー #464）
//!
//! 本ファイルの初版は #463（`generate_pages()` 本体の追加）実装時に
//! PR #477 で同時に作成された。イシュー #464 は本ファイルの整備を
//! 独立タスクとして追跡するために起票されたが、着手時点で既に
//! 受け入れ条件（多階層パス生成・ネガティブ + fail-closed・既存 SSG
//! 非干渉の各テスト、`cargo test -p fandhe-frontend-server` 全通過）を
//! 満たしていた。#464 の作業はその上に、初版が未カバーだった観点
//! （空入力・決定性・非干渉の逆順パターン）を追加する形で行った
//! （初版のテストケースは削除・弱体化していない）。

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

/// 受け入れ条件 3（逆順）: `generate_pages()` を先に実行し、その後
/// `generate()`（固定ルート表）を同じ `out_dir` へ実行しても、
/// `generate_pages()` 側の出力が破壊されないこと。既存の
/// `generate_pages_does_not_disturb_existing_generate_output` は
/// 「先に `generate()`」の順序しか検証していないため、順序依存の
/// 非干渉漏れを別途固定する。
#[test]
fn generate_pages_output_survives_subsequent_generate_call() {
    let dir = TempDir::new("non-interference-reverse");
    generate_pages(&[("/guide/foo/".to_string(), page("Guide"))], &dir.0)
        .expect("generate_pages should succeed");
    let guide_before = fs::read_to_string(dir.0.join("guide/foo/index.html")).unwrap();

    generate(&dir.0).expect("generate should succeed after generate_pages");

    let guide_after = fs::read_to_string(dir.0.join("guide/foo/index.html")).unwrap();
    assert_eq!(guide_before, guide_after);
    assert!(dir.0.join("index.html").exists());
}

/// 空入力は成功しつつ何も書き出さないこと（`generate_pages` の空リスト
/// 契約: rustdoc 「`pages` が空なら `Ok(vec![])` を返し、何も書き出さない」
/// の回帰）。
#[test]
fn generate_pages_empty_input_writes_nothing() {
    let dir = TempDir::new("empty-input");
    let written = generate_pages(&[], &dir.0).expect("empty input should succeed");
    assert!(written.is_empty());
    assert!(
        !dir.0.exists()
            || fs::read_dir(&dir.0)
                .map(|mut entries| entries.next().is_none())
                .unwrap_or(true),
        "empty input should not create any file under out_dir"
    );
}

/// 決定性: 同一入力を 2 回書き出しても同一バイト列になること
/// （`ssr_ssg_parity.rs` の決定性検証と同じ観点を汎用 API にも展開する）。
#[test]
fn generate_pages_is_deterministic_across_reruns() {
    let dir = TempDir::new("deterministic");
    let pages = [
        ("/a".to_string(), page("A")),
        ("/b/c".to_string(), page("B-C")),
    ];

    let first = generate_pages(&pages, &dir.0).expect("first run should succeed");
    let first_bytes: Vec<Vec<u8>> = first.iter().map(|p| fs::read(p).unwrap()).collect();

    let second = generate_pages(&pages, &dir.0).expect("second run should succeed");
    let second_bytes: Vec<Vec<u8>> = second.iter().map(|p| fs::read(p).unwrap()).collect();

    assert_eq!(first, second, "written file list changed across reruns");
    assert_eq!(
        first_bytes, second_bytes,
        "written bytes changed across reruns"
    );
}
