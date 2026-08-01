//! `fandhe_frontend_server::ssg::generate_assets`（パス検証付き非 HTML
//! アセット書き出し API、イシュー #1119）の integration test。
//!
//! `generate_pages`（`<path>/index.html` 固定・`Node` 経由の既定
//! エスケープ）とは異なり、`generate_assets` は任意のファイル名 +
//! 無加工の文字列コンテンツを書き出す。`ssg_generic_routes.rs`（同一系統
//! `generate_pages` の観点構成）を踏襲しつつ、本ファイル固有の契約
//! （無加工書き出し・任意ファイル名）を追加で担保する:
//!
//! - `sitemap.xml`/`robots.txt`/`404.html`/`healthz`（イシューの実例）の
//!   書き出し（正常系）
//! - `/.well-known/` 配下（RFC 8615 well-known URI、イシュー #1137）への
//!   書き出し（正常系）
//! - コンテンツが無加工（エスケープなし）でそのまま書き出される契約
//! - `..`・先頭 `/` 欠如・末尾 `/`・空セグメント・非許可文字・ドットのみの
//!   セグメント・`.git` セグメントの拒否と fail-closed
//! - 重複拒否・空入力・決定性
//! - `generate_pages`/`generate` との非干渉（両順序）

use fandhe_frontend_server::ssg::{generate, generate_assets, generate_pages, SsgError};
use std::fs;

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/support/temp_dir.rs"
));

/// 正常系: イシュー本文の 4 実例（`sitemap.xml`/`robots.txt`/`404.html`/
/// `healthz`）とネストパス（`assets/site.css`）が期待パスへバイト完全
/// 一致で書き出されること。
#[test]
fn generate_assets_writes_expected_files_for_issue_examples() {
    let dir = TempDir::new("issue-examples");
    let assets = [
        (
            "/sitemap.xml".to_string(),
            "<?xml version=\"1.0\"?><urlset></urlset>".to_string(),
        ),
        ("/robots.txt".to_string(), "User-agent: *\n".to_string()),
        (
            "/404.html".to_string(),
            "<!DOCTYPE html><html><body>Not Found</body></html>".to_string(),
        ),
        ("/healthz".to_string(), "ok".to_string()),
        (
            "/assets/site.css".to_string(),
            "body { margin: 0; }".to_string(),
        ),
    ];

    let written = generate_assets(&assets, &dir.0).expect("all assets should be written");
    assert_eq!(written.len(), assets.len());

    for (path, content) in &assets {
        let relative = path.trim_start_matches('/');
        let body = fs::read_to_string(dir.0.join(relative)).unwrap();
        assert_eq!(&body, content);
    }
}

/// 受け入れ条件 1 の直接証明: `/.well-known/` 配下（RFC 8615 well-known
/// URI）へのアセット出力が可能になること（イシュー #1137）。
#[test]
fn generate_assets_writes_well_known_assets() {
    let dir = TempDir::new("well-known");
    let assets = [
        (
            "/.well-known/security.txt".to_string(),
            "Contact: mailto:security@example.com\n".to_string(),
        ),
        (
            "/.well-known/acme-challenge/token".to_string(),
            "challenge-token-value".to_string(),
        ),
    ];

    let written = generate_assets(&assets, &dir.0).expect("well-known assets should be written");
    assert_eq!(written.len(), assets.len());

    for (path, content) in &assets {
        let relative = path.trim_start_matches('/');
        let body = fs::read_to_string(dir.0.join(relative)).unwrap();
        assert_eq!(&body, content);
    }
}

/// 無加工契約: `<`/`&` 等を含むコンテンツがエスケープされずそのまま
/// 書き出されること（`generate_pages` の XSS エスケープ回帰テストと対に
/// なる「本 API は HTML を組み立てない」契約の固定）。
#[test]
fn generate_assets_writes_content_verbatim_without_escaping() {
    let dir = TempDir::new("verbatim");
    let content = "<urlset><url><loc>https://example.com/?a=1&b=2</loc></url></urlset>";
    let written = generate_assets(&[("/sitemap.xml".to_string(), content.to_string())], &dir.0)
        .expect("asset should be written");

    let body = fs::read_to_string(&written[0]).unwrap();
    assert_eq!(body, content);
    assert!(body.contains("&b=2"), "content must not be HTML-escaped");
}

#[test]
fn generate_assets_rejects_unsafe_paths_and_writes_nothing() {
    let cases = [
        "sitemap.xml",           // 先頭 / なし
        "/../etc/passwd",        // .. トラバーサル
        "/a/../b.txt",           // .. トラバーサル
        "/.",                    // ファイル名がドットのみ
        "/..",                   // ファイル名がドットのみ（トラバーサル）
        "/...",                  // ファイル名がドットのみ
        "/healthz/",             // 末尾スラッシュ
        "//",                    // 空セグメント
        "/a/foo\\bar.txt",       // バックスラッシュ
        "/.git/config",          // .git ディレクトリ（defense-in-depth、イシュー #1137）
        "/./x.txt",              // 中間セグメントがドットのみ
        "/.well-known/../x.txt", // ドット始まりディレクトリ経由のトラバーサル
    ];

    for input in cases {
        let dir = TempDir::new("unsafe-path");
        let err = generate_assets(&[(input.to_string(), "x".to_string())], &dir.0)
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
fn generate_assets_rejects_duplicate_output_paths_and_writes_nothing() {
    let dir = TempDir::new("duplicate");
    let err = generate_assets(
        &[
            ("/robots.txt".to_string(), "a".to_string()),
            ("/robots.txt".to_string(), "b".to_string()),
        ],
        &dir.0,
    )
    .expect_err("duplicate robots.txt should be rejected");

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

#[test]
fn generate_assets_empty_input_writes_nothing() {
    let dir = TempDir::new("empty-input");
    let written = generate_assets(&[], &dir.0).expect("empty input should succeed");
    assert!(written.is_empty());
    assert!(
        !dir.0.exists()
            || fs::read_dir(&dir.0)
                .map(|mut entries| entries.next().is_none())
                .unwrap_or(true),
        "empty input should not create any file under out_dir"
    );
}

#[test]
fn generate_assets_is_deterministic_across_reruns() {
    let dir = TempDir::new("deterministic");
    let assets = [
        ("/robots.txt".to_string(), "User-agent: *\n".to_string()),
        ("/healthz".to_string(), "ok".to_string()),
    ];

    let first = generate_assets(&assets, &dir.0).expect("first run should succeed");
    let first_bytes: Vec<Vec<u8>> = first.iter().map(|p| fs::read(p).unwrap()).collect();

    let second = generate_assets(&assets, &dir.0).expect("second run should succeed");
    let second_bytes: Vec<Vec<u8>> = second.iter().map(|p| fs::read(p).unwrap()).collect();

    assert_eq!(first, second, "written file list changed across reruns");
    assert_eq!(
        first_bytes, second_bytes,
        "written bytes changed across reruns"
    );
}

/// 非干渉（正順）: `generate()`（固定ルート表）の後で `generate_assets()`
/// を実行しても既存出力が破壊されないこと。
#[test]
fn generate_assets_does_not_disturb_existing_generate_output() {
    let dir = TempDir::new("non-interference");
    generate(&dir.0).expect("generate should succeed");
    let index_before = fs::read_to_string(dir.0.join("index.html")).unwrap();

    generate_assets(
        &[("/robots.txt".to_string(), "User-agent: *\n".to_string())],
        &dir.0,
    )
    .expect("generate_assets should succeed alongside existing output");

    let index_after = fs::read_to_string(dir.0.join("index.html")).unwrap();
    assert_eq!(index_before, index_after);
    assert!(dir.0.join("robots.txt").exists());
}

/// 非干渉（逆順）: `generate_assets()` を先に実行し、その後
/// `generate_pages()` を同じ `out_dir` へ実行しても互いの出力が
/// 破壊されないこと。
#[test]
fn generate_assets_and_generate_pages_coexist_in_either_order() {
    use fandhe_frontend_core::{el, text};

    let dir = TempDir::new("coexist");
    generate_assets(
        &[("/robots.txt".to_string(), "User-agent: *\n".to_string())],
        &dir.0,
    )
    .expect("generate_assets should succeed");
    let robots_before = fs::read_to_string(dir.0.join("robots.txt")).unwrap();

    let node = el(
        "html",
        vec![],
        vec![el("body", vec![], vec![text("Guide")])],
    );
    generate_pages(&[("/guide".to_string(), node)], &dir.0)
        .expect("generate_pages should succeed after generate_assets");

    let robots_after = fs::read_to_string(dir.0.join("robots.txt")).unwrap();
    assert_eq!(robots_before, robots_after);
    assert!(dir.0.join("guide/index.html").exists());
}
