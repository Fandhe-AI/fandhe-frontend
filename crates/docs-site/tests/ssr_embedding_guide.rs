//! SSR 組み込みガイド（`docs/guides/ssr-embedding.md`、イシュー #3823）の
//! ドリフト検知テスト。
//!
//! ガイドの nav 登録・「執筆中」文言の除去・embedding-guide からの導線・
//! `Document` rustdoc の doctest との逐語一致を固定する。実サイトのビルドは使わず、
//! `CARGO_MANIFEST_DIR` 起点の固定パスだけを読む軽量テスト。

use std::fs;
use std::path::PathBuf;

use fandhe_frontend_docs_site::nav::parse_nav;
use fandhe_frontend_docs_site::section_index::GUIDES;

const GUIDE_SOURCE: &str = "docs/guides/ssr-embedding.md";
const GUIDE_PATH: &str = "/guides/ssr-embedding/";

fn read(rel: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("{rel} must be readable: {e}"))
}

/// ガイドが Guides セクションからナビ登録されていること。
#[test]
fn guide_is_registered_under_guides() {
    let nav = parse_nav(&read("site/nav.toml")).expect("nav.toml must parse");
    let guides = nav
        .sections
        .iter()
        .find(|s| s.title == "Guides")
        .expect("Guides section must exist");
    assert!(guides
        .all_pages()
        .any(|p| p.source == GUIDE_SOURCE && p.path == GUIDE_PATH));
}

/// 本文・索引カードに「執筆中」が残っていないこと。
#[test]
fn guide_is_no_longer_a_stub() {
    assert!(!read(GUIDE_SOURCE).contains("執筆中"));
    let card = GUIDES
        .iter()
        .find(|c| c.path == GUIDE_PATH)
        .expect("guide card must exist");
    assert!(!card.description.contains("執筆中"));
}

/// embedding-guide から新ガイドへ辿れること。
#[test]
fn embedding_guide_links_to_ssr_guide() {
    assert!(read("docs/guides/embedding-guide.md").contains("(./ssr-embedding.md)"));
}

/// `Document` rustdoc の doctest（`Document::new` を含む 2 本）が、ガイドに逐語で載っていること。
#[test]
fn guide_embeds_document_doctests_verbatim() {
    let src = read("crates/app/src/document.rs");
    let mut blocks: Vec<String> = Vec::new();
    let mut cur: Option<Vec<String>> = None;
    for line in src.lines() {
        let Some(rest) = line.strip_prefix("///") else {
            continue;
        };
        let rest = rest.strip_prefix(' ').unwrap_or(rest);
        if rest.trim_end() == "```" {
            match cur.take() {
                Some(b) => blocks.push(b.join("\n")),
                None => cur = Some(Vec::new()),
            }
        } else if let Some(b) = cur.as_mut() {
            b.push(rest.to_string());
        }
    }
    let targets: Vec<&String> = blocks
        .iter()
        .filter(|b| b.contains("Document::new"))
        .collect();
    assert!(
        targets.len() >= 2,
        "doctest extraction failed: {}",
        targets.len()
    );
    let guide = read(GUIDE_SOURCE);
    for b in targets {
        assert!(
            guide.contains(b.as_str()),
            "doctest not embedded verbatim:\n{b}"
        );
    }
}
