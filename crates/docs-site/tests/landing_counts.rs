//! トップの数値指標が `site/nav.toml` とレジストリから算出され、ページ追加に
//! 追従することを固定する（イシュー #3614 受け入れ条件）。

use std::path::PathBuf;

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::nav::parse_nav;
use fandhe_frontend_docs_site::{blocks, landing, primitives_catalog, wireframes};

fn repo_file(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel)
}

fn nav_toml() -> String {
    std::fs::read_to_string(repo_file("site/nav.toml")).expect("read nav.toml")
}

fn real_counts() -> Vec<landing::LayerCount> {
    landing::layer_counts(&parse_nav(&nav_toml()).expect("parse nav"))
}

fn count_of(counts: &[landing::LayerCount], label: &str) -> usize {
    counts
        .iter()
        .find(|c| c.label == label)
        .unwrap_or_else(|| panic!("{label} missing"))
        .count
}

#[test]
fn counts_match_registries() {
    let counts = real_counts();
    assert_eq!(counts.len(), 4);
    assert_eq!(
        count_of(&counts, "Primitives"),
        primitives_catalog::entries().count()
    );
    assert_eq!(count_of(&counts, "Blocks"), blocks::all_blocks().len());
    assert_eq!(
        count_of(&counts, "Wireframes"),
        wireframes::WIREFRAMES.len()
    );
    assert!(count_of(&counts, "Themes") > 0);
    assert_eq!(Some(counts), landing::site_layer_counts());
}

#[test]
fn rendered_stats_show_the_computed_counts() {
    let html: String = landing::render("").iter().map(render).collect();
    for c in real_counts() {
        assert!(
            html.contains(&format!("{} の部品数", c.label)),
            "{}",
            c.label
        );
        assert!(
            html.contains(&format!(">{}<", c.count)),
            "{} の件数 {} が描画されない",
            c.label,
            c.count
        );
    }
}

#[test]
fn adding_a_page_increases_the_count() {
    let toml = nav_toml();
    let before = landing::layer_counts(&parse_nav(&toml).expect("parse"));
    let marker = "index_path = \"/blocks/\"";
    assert!(toml.contains(marker));
    let extra = "\n[[section.page]]\ntitle = \"Extra\"\nsource = \"site/blocks.md\"\npath = \"/blocks/zz-extra-block/\"\n";
    // `/blocks/` セクションの末尾（次の `[[section]]` の直前）へ 1 ページ足す。
    let start = toml.find(marker).unwrap();
    let insert_at = toml[start..]
        .find("\n[[section]]")
        .map_or(toml.len(), |i| start + i);
    let mut modified = toml.clone();
    modified.insert_str(insert_at, extra);
    let after = landing::layer_counts(&parse_nav(&modified).expect("parse modified"));
    assert_eq!(count_of(&after, "Blocks"), count_of(&before, "Blocks") + 1);
    assert_eq!(count_of(&after, "Themes"), count_of(&before, "Themes"));
}

#[test]
fn dependency_limits_match_xtask_constants() {
    let src = std::fs::read_to_string(repo_file("crates/xtask/src/check_deps.rs")).expect("read");
    assert!(src.contains(&format!(
        "pub const MAX_PACKAGES: usize = {};",
        landing::DEP_MAX_PACKAGES
    )));
    assert!(src.contains(&format!(
        "pub const MAX_DEPTH: usize = {};",
        landing::DEP_MAX_DEPTH
    )));
}

#[test]
fn internal_links_exist_in_nav() {
    let nav = parse_nav(&nav_toml()).expect("parse");
    let paths: Vec<&str> = nav.all_pages().map(|p| p.path.as_str()).collect();
    let unique: std::collections::HashSet<_> = landing::internal_link_paths().collect();
    assert_eq!(unique.len(), landing::internal_link_paths().count());
    for p in landing::internal_link_paths() {
        assert!(paths.contains(&p), "{p} が nav に無い");
    }
}
