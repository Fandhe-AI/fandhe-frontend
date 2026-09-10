//! `site/nav.toml` の Blocks セクション・`crate::blocks::BLOCKS` レジストリ・
//! `site/blocks/*.md` 原稿ファイル集合の三方突合（イシュー #2088）。
//!
//! `crates/docs-site/tests/primitives_nav.rs`（Primitives 台帳の三方突合）と
//! 同型のドリフト検知テストである。後続イシュー #2089〜#2095 が block を
//! 追加する際、nav.toml・レジストリ・原稿ファイルのいずれか 1 箇所だけの
//! 更新漏れを fail-closed に検知する。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use fandhe_frontend_docs_site::blocks;
use fandhe_frontend_docs_site::nav::{parse_nav, Nav};

/// `CARGO_MANIFEST_DIR`（`crates/docs-site`）から repo_root を解決する
/// （`tests/site_nav.rs`/`tests/primitives_nav.rs` と同じ規約）。
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

/// Blocks セクションが Themes の直後・API Reference の直前（Primitives の
/// 次の次）に存在し、`index_path`・group 非使用（フラット構成、設計 §6）が
/// 期待どおりであること。
#[test]
fn blocks_section_is_registered_immediately_after_themes() {
    let nav = load_nav();
    let index = nav
        .sections
        .iter()
        .position(|s| s.title == "Blocks")
        .expect("Blocks section should be registered");
    assert_eq!(nav.sections[index - 1].title, "Themes");
    assert_eq!(nav.sections[index + 1].title, "API Reference");

    let section = &nav.sections[index];
    assert_eq!(section.index_path, "/blocks/");
    assert!(
        section.groups.is_empty(),
        "Blocks section should use flat [[section.page]] only (no [[section.group]])"
    );
}

/// `site/nav.toml` の `/blocks/*` ページ（索引を除く）と `blocks::BLOCKS` の
/// `path` が完全一致すること（登録漏れ・孤児のいずれも検知する）。
#[test]
fn nav_toml_block_pages_match_the_registry_exactly() {
    let nav = load_nav();
    let section = nav
        .sections
        .iter()
        .find(|s| s.title == "Blocks")
        .expect("Blocks section should be registered");

    let nav_block_paths: BTreeSet<&str> = section
        .all_pages()
        .map(|p| p.path.as_str())
        .filter(|path| *path != section.index_path)
        .collect();
    let registry_paths: BTreeSet<&str> = blocks::BLOCKS.iter().map(|b| b.path).collect();

    assert_eq!(
        nav_block_paths, registry_paths,
        "nav.toml の /blocks/* ページ（索引除く）と blocks::BLOCKS の path が一致しない"
    );
}

/// `blocks::BLOCKS` の各 `path` に対応する `site/blocks/<kebab>.md` が
/// 実在すること（nav.toml の `source` フィールドとも一致させる）。
#[test]
fn every_registered_block_has_a_manuscript_file() {
    let nav = load_nav();
    let section = nav
        .sections
        .iter()
        .find(|s| s.title == "Blocks")
        .expect("Blocks section should be registered");

    for block in blocks::BLOCKS {
        let page = section
            .all_pages()
            .find(|p| p.path == block.path)
            .unwrap_or_else(|| panic!("nav.toml should declare a page for {}", block.path));
        let source_path = repo_root().join(&page.source);
        assert!(
            source_path.is_file(),
            "manuscript file {source_path:?} for block {} should exist",
            block.path
        );
    }
}

/// `site/blocks/*.md` 原稿ファイル集合と `blocks::BLOCKS`/`nav.toml` の
/// 三方目を締める（イシュー #2088 codex-review P2 指摘）。
/// `every_registered_block_has_a_manuscript_file` は「登録済み block に
/// 対応する原稿ファイルが実在するか」の片方向しか検証しないため、
/// `site/blocks/` に置かれたが `blocks::BLOCKS`/`nav.toml` のどちらにも
/// 登録されていない孤児原稿ファイルを検知できない欠落があった。本テストは
/// `site/blocks/` ディレクトリを実際に列挙し、その集合が
/// `blocks::BLOCKS` の `path` から導出した想定ファイル名集合と完全一致する
/// ことを固定する。
#[test]
fn site_blocks_dir_manuscripts_match_the_registry_exactly() {
    let dir = repo_root().join("site/blocks");
    let entries = std::fs::read_dir(&dir).expect("site/blocks directory should be readable");

    let on_disk: BTreeSet<String> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
        .filter_map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(str::to_owned)
        })
        .collect();

    let expected: BTreeSet<String> = blocks::BLOCKS
        .iter()
        .map(|block| {
            let kebab = block
                .path
                .trim_start_matches("/blocks/")
                .trim_end_matches('/');
            format!("{kebab}.md")
        })
        .collect();

    assert_eq!(
        on_disk, expected,
        "site/blocks/*.md の実在ファイル集合と blocks::BLOCKS から導出した期待集合が一致しない          （未登録の孤児原稿ファイル、または登録済みだがファイルが無い block のいずれか）"
    );
}

/// `site/blocks.md` が索引ページとして登録され、掲載済み block（login-01）
/// への相対リンクを含むこと。
#[test]
fn blocks_index_page_links_to_the_registered_block() {
    let nav = load_nav();
    let section = nav
        .sections
        .iter()
        .find(|s| s.title == "Blocks")
        .expect("Blocks section should be registered");
    let index_page = section
        .pages
        .iter()
        .find(|p| p.path == "/blocks/")
        .expect("Blocks section should declare its index page as a direct page");
    assert_eq!(index_page.source, "site/blocks.md");

    let content = std::fs::read_to_string(repo_root().join(&index_page.source))
        .expect("site/blocks.md should be readable");
    assert!(
        content.contains("./blocks/login-01.md"),
        "site/blocks.md should link to the registered login-01 block"
    );
}
