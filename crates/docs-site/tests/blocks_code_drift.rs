//! `crate::blocks` 配下の手書き Rust 実装と `site/blocks/<kebab>.md` の
//! ```rust フェンス本文とのドリフト検知（イシュー #2088 §2.5）。
//!
//! 各 block 実装ファイル（[`fandhe_frontend_docs_site::blocks::Block::rust_source`]）
//! は `// blocks-code:begin` / `// blocks-code:end` の行マーカーで
//! `use` 宣言 + `pub fn demo() -> Node` を囲む。対応する Markdown 原稿の
//! 最初の ```rust フェンス本文が、このマーカー内の行（両端マーカー行を
//! 除く、末尾空白のみ trim 許容）と行単位で完全一致することを固定する。
//! `include_str!` のような自動同期は使わない設計（設計文書 §5）であるため、
//! 本テストが唯一の機械的な整合性保証になる。

use std::path::{Path, PathBuf};

use fandhe_frontend_docs_site::blocks;
use fandhe_frontend_docs_site::nav::{parse_nav, Nav};

const BEGIN_MARKER: &str = "// blocks-code:begin";
const END_MARKER: &str = "// blocks-code:end";

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

/// マーカー間の行（両端マーカー行を除く）を抽出する。マーカーが片方でも
/// 欠落していれば `None`（fail-closed。呼び出し元が panic させる）。
fn extract_marked_lines(source: &str) -> Option<Vec<&str>> {
    let lines: Vec<&str> = source.lines().collect();
    let begin = lines.iter().position(|l| l.trim() == BEGIN_MARKER)?;
    let end = lines.iter().position(|l| l.trim() == END_MARKER)?;
    if end <= begin {
        return None;
    }
    Some(lines[begin + 1..end].to_vec())
}

/// Markdown 本文中、最初の ```rust フェンスの本文行（フェンス行自体は
/// 含まない）を抽出する。フェンスが無ければ `None`。
fn extract_first_rust_fence(markdown: &str) -> Option<Vec<&str>> {
    let lines: Vec<&str> = markdown.lines().collect();
    let start = lines.iter().position(|l| l.trim() == "```rust")?;
    let end = lines[start + 1..]
        .iter()
        .position(|l| l.trim() == "```")
        .map(|i| start + 1 + i)?;
    Some(lines[start + 1..end].to_vec())
}

/// 末尾空白のみ trim して比較する（実装計画 §2.5「末尾空白のみ trim 許容」）。
fn lines_match_ignoring_trailing_whitespace(a: &[&str], b: &[&str]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b.iter())
            .all(|(x, y)| x.trim_end() == y.trim_end())
}

#[test]
fn every_registered_block_rust_source_matches_its_manuscript_fence() {
    let nav = load_nav();
    let section = nav
        .sections
        .iter()
        .find(|s| s.title == "Blocks")
        .expect("Blocks section should be registered");

    for block in blocks::BLOCKS {
        let rust_path = repo_root().join(block.rust_source);
        let rust_source = std::fs::read_to_string(&rust_path)
            .unwrap_or_else(|e| panic!("read {rust_path:?}: {e}"));
        let marked_lines = extract_marked_lines(&rust_source).unwrap_or_else(|| {
            panic!(
                "{:?} should contain {BEGIN_MARKER}/{END_MARKER} markers",
                rust_path
            )
        });

        let page = section
            .all_pages()
            .find(|p| p.path == block.path)
            .unwrap_or_else(|| panic!("nav.toml should declare a page for {}", block.path));
        let md_path = repo_root().join(&page.source);
        let md_source =
            std::fs::read_to_string(&md_path).unwrap_or_else(|e| panic!("read {md_path:?}: {e}"));
        let fence_lines = extract_first_rust_fence(&md_source)
            .unwrap_or_else(|| panic!("{:?} should contain a ```rust fence", md_path));

        assert!(
            lines_match_ignoring_trailing_whitespace(&marked_lines, &fence_lines),
            "block {} 側の drift 検知に失敗しました\n--- {:?} (marker 内) ---\n{}\n--- {:?} (```rust fence) ---\n{}",
            block.path,
            rust_path,
            marked_lines.join("\n"),
            md_path,
            fence_lines.join("\n"),
        );
    }
}
