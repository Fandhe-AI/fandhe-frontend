//! `site/nav.toml`（実マニフェスト）と既存 docs 資産のドリフト検知テスト。
//!
//! イシュー #473 の受け入れ条件（登録ページの網羅・サブセット外構文の
//! 残存によるレンダリング崩れ無し・内容不変更）のうち、機械的に検証できる
//! 部分（パース成功・ページ登録数と path・source 実在・ブロックレベルの
//! レンダリング健全性）を本テストで担保する。ジェネレータ本体（`main.rs`、
//! イシュー #470）は本テスト実行時点で fail-closed スタブのままのため、
//! `dist/` を実際に生成しての目視確認は #476（受入検証）へ引き継ぐ。
//!
//! # ページ数について（イシュー本文との数値差異）
//!
//! イシュー #473 本文は受け入れ条件を「全 11 ページ（トップ + quickstart +
//! guides 4 + api 6）」と記載するが、`1 + 1 + 4 + 6 = 12` であり列挙内容と
//! 総数表記が矛盾している（イシュー本文側の算術上の誤記と判断）。本テストは
//! 列挙されたページ集合（トップ 1 + quickstart 1 + guides 4 + api 6 =
//! 実質 12 ページ）を正としてすべて登録する。
//!
//! `site/index.md`（イシュー #472）は本テスト実行時点で未マージのことが
//! あるため、存在すれば厳格検証（他ページと同様に `render_markdown` の
//! 健全性を確認）し、存在しなければ「他 11 ページの検証」のみに縮退する。
//! #472 マージ後は自動的に全ページの厳格検証になる。

use std::path::{Path, PathBuf};

use fandhe_frontend_core::Node;
use fandhe_frontend_docs_site::markdown::render_markdown;
use fandhe_frontend_docs_site::nav::{parse_nav, validate_sources, Nav};

/// `CARGO_MANIFEST_DIR`（`crates/docs-site`）から repo_root を解決する。
/// テストフィクスチャがクレート内に閉じず repo_root 配下の実ファイルを
/// 参照するため、`crates/docs-site` の 2 階層上を repo_root とみなす
/// （`Cargo.toml` の `members = ["crates/*"]` 構成に対応）。
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo_root should resolve from CARGO_MANIFEST_DIR")
}

fn load_nav() -> Nav {
    let path = repo_root().join("site/nav.toml");
    let input = std::fs::read_to_string(&path).expect("site/nav.toml should be readable");
    parse_nav(&input).expect("site/nav.toml should conform to the fail-closed TOML subset")
}

#[test]
fn site_nav_parses_successfully() {
    let nav = load_nav();
    assert_eq!(nav.site.title, "fandhe-frontend");
    assert_eq!(nav.site.base_path, "/fandhe-frontend");
}

#[test]
fn site_nav_registers_three_sections_with_expected_titles() {
    let nav = load_nav();
    let titles: Vec<&str> = nav.sections.iter().map(|s| s.title.as_str()).collect();
    assert_eq!(titles, vec!["Getting Started", "Guides", "API Reference"]);
}

/// 受け入れ条件 1: 既存の利用者向けドキュメント（トップ + quickstart +
/// guides 4 本 + api 6 本 = 12 ページ）がサイト生成対象として登録されている
/// （イシュー本文の「全 11 ページ」表記と列挙内容の数値差異はモジュール冒頭の
/// コメント参照）。
#[test]
fn site_nav_registers_all_twelve_pages_with_expected_paths() {
    let nav = load_nav();
    let pages: Vec<(&str, &str)> = nav
        .sections
        .iter()
        .flat_map(|s| s.pages.iter())
        .map(|p| (p.source.as_str(), p.path.as_str()))
        .collect();

    assert_eq!(pages.len(), 12, "expected 12 pages, got {pages:?}");

    let expected = vec![
        ("site/index.md", "/"),
        ("docs/guides/quickstart.md", "/getting-started/quickstart/"),
        (
            "docs/guides/component-authoring.md",
            "/guides/component-authoring/",
        ),
        ("docs/guides/embedding-guide.md", "/guides/embedding-guide/"),
        (
            "docs/guides/view-transitions.md",
            "/guides/view-transitions/",
        ),
        ("docs/guides/npm-asset-build.md", "/guides/npm-asset-build/"),
        ("docs/api/component-api.md", "/api/component-api/"),
        ("docs/api/app-api.md", "/api/app-api/"),
        ("docs/api/interactive-api.md", "/api/interactive-api/"),
        ("docs/api/hydration-api.md", "/api/hydration-api/"),
        (
            "docs/api/hydration-state-format.md",
            "/api/hydration-state-format/",
        ),
        (
            "docs/api/router-path-matching.md",
            "/api/router-path-matching/",
        ),
    ];
    // "site/index.md" は #472（未マージのことがある）依存のため、期待値の
    // 先頭は上のリストに含めつつ後段の実在チェックでは条件付きにする。
    // ここでは path/source の宣言内容そのもの（10 + 1 件）を厳格検証する。
    assert_eq!(pages.len(), expected.len());
    for expected_pair in &expected {
        assert!(
            pages.contains(expected_pair),
            "nav.toml is missing expected page {expected_pair:?}"
        );
    }
}

#[test]
fn site_nav_has_no_duplicate_paths_or_sources() {
    let nav = load_nav();
    let mut seen_paths = std::collections::BTreeSet::new();
    let mut seen_sources = std::collections::BTreeSet::new();
    for section in &nav.sections {
        for page in &section.pages {
            assert!(
                seen_paths.insert(page.path.clone()),
                "duplicate page.path: {}",
                page.path
            );
            assert!(
                seen_sources.insert(page.source.clone()),
                "duplicate page.source: {}",
                page.source
            );
        }
    }
}

/// `site/index.md` を除く 11 source は #465〜#469 マージ済みの時点で必ず
/// repo_root 配下に実在する（#472 に依存しない既存資産）。
#[test]
fn site_nav_sources_other_than_site_index_exist() {
    let root = repo_root();
    let nav = load_nav();
    for section in &nav.sections {
        for page in &section.pages {
            if page.source == "site/index.md" {
                continue;
            }
            let full_path = root.join(&page.source);
            assert!(
                full_path.is_file(),
                "expected source file to exist: {}",
                page.source
            );
        }
    }
}

/// `site/index.md`（#472）が既にマージされていれば、`validate_sources` は
/// 全ページを対象に成功しなければならない。未マージであれば本テストは
/// 早期リターンし、他 11 ページ分の検証は上記テストに委ねる。#472 マージ後は
/// 本テストが自動的に全件厳格検証へ切り替わる。
#[test]
fn site_nav_validate_sources_covers_all_pages_once_site_index_exists() {
    let root = repo_root();
    if !root.join("site/index.md").is_file() {
        // #472（site/index.md 新設）が未マージ。フォールバックとして
        // 他 11 ページの実在確認は site_nav_sources_other_than_site_index_exist
        // が担保する。
        return;
    }
    let nav = load_nav();
    validate_sources(&nav, &root)
        .expect("all page.source entries should exist once site/index.md is present");
}

/// 受け入れ条件 2: サブセット外構文の残存によるレンダリング崩れがない。
///
/// `render_markdown` はブロックレベル解釈のみを担う（インライン構文は
/// #467 が別途差し替える契約、`markdown.rs` の `inline_nodes` 参照）ため、
/// 本テストはブロック構造の健全性（先頭が見出しであること・フェンス
/// コードの閉じ忘れがテキストノードに漏れ出していないこと）のみを確認する。
#[test]
fn every_existing_source_renders_without_fence_leakage_and_starts_with_a_heading() {
    let root = repo_root();
    let nav = load_nav();
    for section in &nav.sections {
        for page in &section.pages {
            let full_path = root.join(&page.source);
            if !full_path.is_file() {
                // site/index.md が未マージの場合はスキップ（#472 依存）。
                continue;
            }
            let input = std::fs::read_to_string(&full_path)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", page.source));
            let blocks = render_markdown(&input);
            assert!(
                !blocks.is_empty(),
                "{} rendered to an empty block list",
                page.source
            );
            assert!(
                is_heading(&blocks[0]),
                "{} does not start with a heading (H1 expected as the page title)",
                page.source
            );
            for block in &blocks {
                assert!(
                    !contains_unclosed_fence_marker(block),
                    "{} contains a stray ``` marker in rendered text, likely an unclosed fence",
                    page.source
                );
            }
        }
    }
}

fn is_heading(node: &Node) -> bool {
    matches!(
        node,
        Node::Element { tag, .. } if matches!(*tag, "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
    )
}

/// テキストノードに ``` （フェンス開始/終了マーカー）がそのまま残っていないか
/// 再帰的に確認する。閉じ忘れたフェンスはブロックパーサが段落テキストとして
/// フォールバックするため、マーカー文字列がテキストノードに漏れ出る形で検知できる。
fn contains_unclosed_fence_marker(node: &Node) -> bool {
    match node {
        Node::Text(s) => s.contains("```"),
        Node::RawHtml(_) => false,
        Node::Element { children, .. } => children.iter().any(contains_unclosed_fence_marker),
    }
}
