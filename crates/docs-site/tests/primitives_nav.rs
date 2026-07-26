//! `site/nav.toml` の Primitives セクション・`site/primitives/*.md` 原稿群を
//! `crates/docs-site/src/primitives_catalog.rs`（イシュー #1020、63 部品の
//! 唯一の正）と三方突合するドリフト検知テスト（イシュー #1021）。
//!
//! 63 件の手書きリスト（nav.toml・原稿ファイル・本テストの期待値）を
//! 目視同期に委ねると、追加・削除・並べ替えのいずれかが片方だけに反映
//! される事故が起きうる。本テストは「台帳を唯一の正として nav.toml と
//! 原稿ファイル集合の双方を機械突合する」ことで、以後のドリフトを
//! fail-closed に検知する（計画 §2-4）。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use fandhe_frontend_docs_site::nav::{parse_nav, Nav};
use fandhe_frontend_docs_site::primitives_catalog::{self, PrimitiveCategory};

/// `CARGO_MANIFEST_DIR`（`crates/docs-site`）から repo_root を解決する
/// （`tests/site_nav.rs` と同じ規約）。
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

/// Primitives セクションが Themes の直前（index 3）に存在し、
/// `index_path`・直下ページ（索引 1 件のみ）が期待どおりであること。
#[test]
fn primitives_section_is_registered_immediately_before_themes() {
    let nav = load_nav();
    let index = nav
        .sections
        .iter()
        .position(|s| s.title == "Primitives")
        .expect("Primitives section should be registered");
    assert_eq!(
        index, 3,
        "Primitives section should be the 4th section (index 3, immediately before Themes)"
    );
    assert_eq!(nav.sections[index + 1].title, "Themes");

    let section = &nav.sections[index];
    assert_eq!(section.index_path, "/primitives/");
    assert_eq!(section.pages.len(), 1, "direct pages should be index only");
    assert_eq!(section.pages[0].source, "site/primitives.md");
    assert_eq!(section.pages[0].path, "/primitives/");
}

/// `section.groups` が台帳の 6 カテゴリと順序込みで一致すること。
#[test]
fn primitives_section_groups_match_catalog_category_order() {
    let nav = load_nav();
    let section = nav
        .sections
        .iter()
        .find(|s| s.title == "Primitives")
        .expect("Primitives section should be registered");

    let expected_titles: Vec<&str> = PrimitiveCategory::all().iter().map(|c| c.title()).collect();
    let actual_titles: Vec<&str> = section.groups.iter().map(|g| g.title.as_str()).collect();
    assert_eq!(actual_titles, expected_titles);
}

/// 各グループのページ列が台帳の宣言順・`(title, path, source)` で
/// 完全一致すること。
#[test]
fn primitives_group_pages_match_catalog_entries_exactly() {
    let nav = load_nav();
    let section = nav
        .sections
        .iter()
        .find(|s| s.title == "Primitives")
        .expect("Primitives section should be registered");

    for (group, category) in section.groups.iter().zip(PrimitiveCategory::all().iter()) {
        let expected: Vec<(String, String, String)> = primitives_catalog::entries_in(*category)
            .map(|e| {
                (
                    e.title.to_string(),
                    e.path.to_string(),
                    format!(
                        "site/primitives/{}.md",
                        primitives_catalog::kebab_of(e.module)
                    ),
                )
            })
            .collect();
        let actual: Vec<(String, String, String)> = group
            .pages
            .iter()
            .map(|p| (p.title.clone(), p.path.clone(), p.source.clone()))
            .collect();
        assert_eq!(
            actual, expected,
            "group `{}` page list does not match catalog entries for category {category:?}",
            group.title
        );
    }
}

/// `site/primitives/` 直下の `.md` ファイル集合が台帳の kebab 集合と
/// 過不足なく一致すること（孤児ファイル・欠落ファイルの双方で落ちる）。
#[test]
fn primitives_markdown_files_match_catalog_exactly() {
    let dir = repo_root().join("site/primitives");
    let observed: BTreeSet<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
        .map(|entry| entry.expect("dir entry should be readable"))
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();

    let expected: BTreeSet<String> = primitives_catalog::entries()
        .map(|e| primitives_catalog::kebab_of(e.module))
        .collect();

    let missing: Vec<&String> = expected.difference(&observed).collect();
    let orphaned: Vec<&String> = observed.difference(&expected).collect();
    assert!(
        missing.is_empty() && orphaned.is_empty(),
        "site/primitives/ drifted from the catalog: missing={missing:?} orphaned={orphaned:?}"
    );
}

/// `site/primitives/*.md`（索引 `site/primitives.md` は対象外）が `## ` 見出しを
/// 持たないこと（`tests/component_pages.rs::component_markdown_sources_have_no_h2_headings`
/// と同型の契約。節は Rust 側 `ComponentPageSpec` から供給する設計、
/// `docs/design/docs-site-component-pages.md` §7a.1）。
#[test]
fn primitives_markdown_sources_have_no_h2_headings() {
    let dir = repo_root().join("site/primitives");
    for entry in std::fs::read_dir(&dir).expect("read_dir site/primitives") {
        let entry = entry.expect("dir entry should be readable");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        assert!(
            !content.lines().any(|line| line.starts_with("## ")),
            "{} should not declare its own H2 headings (sections come from ComponentPageSpec)",
            path.display()
        );
    }
}
