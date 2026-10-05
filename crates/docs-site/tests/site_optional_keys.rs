//! `[site]` の任意キー（`tagline` / `copyright` / `version_badge` / `lang`、イシュー #3721）
//! の結合テスト。
//!
//! レイアウト関数（`docs_page_with_chrome`）の単体契約と、一時サイトを実際にビルドして
//! `index.html` / `404.html` / リダイレクト案内ページへ反映されることを確認する E2E の
//! 2 系統。設計の正は `docs/design/docs-site-external-use.md`。未指定時の出力が従来と
//! バイト一致することは、既定 `SiteChrome` が `docs_page_with_layout` と一致するテストと、
//! 実サイトの変更前後比較（PR 手順）で担保する。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use fandhe_frontend_core::{li, p, render, text, ul, Node};
use fandhe_frontend_docs_site::build::build_site_with;
use fandhe_frontend_docs_site::layout::{
    docs_page_with_chrome, docs_page_with_layout, PageLayout, SiteChrome, VersionBadge,
};
use fandhe_frontend_docs_site::page_sections::EMPTY_REGISTRY;

// ---- docs_page_with_chrome ----

fn sidebar() -> Node {
    ul(vec![], vec![li(vec![], vec![text("はじめに")])])
}

fn page(chrome: &SiteChrome<'_>) -> String {
    render(&docs_page_with_chrome(
        chrome,
        "T",
        "",
        sidebar(),
        p(vec![], vec![text("本文")]),
        &[],
        None,
        None,
        None,
        PageLayout::Docs,
    ))
}

#[test]
fn default_chrome_matches_docs_page_with_layout() {
    let legacy = render(&docs_page_with_layout(
        "T",
        "",
        sidebar(),
        p(vec![], vec![text("本文")]),
        &[],
        None,
        None,
        None,
        PageLayout::Docs,
    ));
    assert_eq!(page(&SiteChrome::default()), legacy);
}

#[test]
fn custom_badge_is_escaped_and_replaces_core_version() {
    let html = page(&SiteChrome {
        version_badge: VersionBadge::Custom("v9 <b>"),
        ..SiteChrome::default()
    });
    assert!(html.contains("docs-brand-version"));
    assert!(html.contains("v9 &lt;b&gt;"));
    assert!(!html.contains("v9 <b>"));
    assert!(!html.contains("core v"));
}

#[test]
fn hidden_badge_omits_the_badge_but_keeps_brand_and_actions() {
    let html = page(&SiteChrome {
        version_badge: VersionBadge::Hidden,
        ..SiteChrome::default()
    });
    assert!(!html.contains("docs-brand-version"));
    let brand = html.find("docs-brand\"").expect("brand");
    let actions = html.find("docs-header-actions").expect("actions");
    assert!(brand < actions);
}

#[test]
fn lang_is_reflected_on_the_html_element() {
    let html = page(&SiteChrome {
        lang: "en-US",
        ..SiteChrome::default()
    });
    assert!(html.starts_with("<html lang=\"en-US\">"));
}

// ---- 一時サイトのビルド ----

/// 一時ディレクトリ（`tempfile` を追加しない、REQ-3。`site_build.rs` と同方針）。
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let root = std::env::var("CARGO_TARGET_TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = root.join(format!(
            "fandhe-frontend-docs-site-optkeys-{tag}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("create temp dir");
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// 一時サイトを作る。`extra_site` は `[site]` へ足す行。リダイレクト案内ページも
/// 出力させるため `site/redirects.toml` を 1 件置く。
fn write_site(root: &Path, extra_site: &str) {
    std::fs::create_dir_all(root.join("site")).expect("create site dir");
    let nav = format!(
        "[site]\ntitle = \"Ext Docs\"\nbase_path = \"\"\n{extra_site}\n[[section]]\ntitle = \"Guide\"\nindex_path = \"/\"\n\n[[section.page]]\ntitle = \"Home\"\nsource = \"site/index.md\"\npath = \"/\"\n"
    );
    std::fs::write(root.join("site/nav.toml"), nav).expect("write nav.toml");
    std::fs::write(root.join("site/index.md"), "# Home\n\nHello.\n").expect("write index.md");
    std::fs::write(
        root.join("site/redirects.toml"),
        "[[redirect]]\nfrom = \"/old/\"\nto = \"/\"\n",
    )
    .expect("write redirects.toml");
}

fn read_html_tree(root: &Path) -> BTreeMap<String, String> {
    fn walk(base: &Path, dir: &Path, acc: &mut BTreeMap<String, String>) {
        for entry in std::fs::read_dir(dir).expect("read_dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                walk(base, &path, acc);
            } else if path.extension().is_some_and(|e| e == "html") {
                let rel = path.strip_prefix(base).expect("prefix");
                acc.insert(
                    rel.to_string_lossy().replace('\\', "/"),
                    std::fs::read_to_string(&path).expect("read"),
                );
            }
        }
    }
    let mut acc = BTreeMap::new();
    walk(root, root, &mut acc);
    acc
}

fn build(tag: &str, extra_site: &str) -> BTreeMap<String, String> {
    let temp = TempDir::new(tag);
    let root = temp.0.join("repo");
    write_site(&root, extra_site);
    let out = temp.0.join("dist");
    build_site_with(&root, &out, &EMPTY_REGISTRY).expect("site should build");
    read_html_tree(&out)
}

#[test]
fn optional_keys_apply_to_index_404_and_redirect_pages() {
    let pages = build(
        "all",
        "tagline = \"Our <b>tagline</b>\"\ncopyright = \"(c) Example Inc.\"\nversion_badge = \"v9.9\"\nlang = \"en-US\"\n",
    );
    for key in ["index.html", "404.html", "old/index.html"] {
        let html = pages.get(key).unwrap_or_else(|| panic!("{key} missing"));
        assert!(html.contains("<html lang=\"en-US\">"), "{key}");
    }
    let index = &pages["index.html"];
    assert!(index.contains(">v9.9<"));
    assert!(!index.contains("core v"));
    assert!(index.contains("Our &lt;b&gt;tagline&lt;/b&gt;"));
    assert!(!index.contains("<b>tagline"));
    assert!(index.contains("(c) Example Inc."));
    assert!(index.contains("Built with "));
    assert!(index.contains("fandhe-frontend docs-site"));
    assert!(!index.contains("Licensed under"));
    assert!(!index.contains("crates.io/crates"));
    assert!(index.contains("LICENSE-MIT") && index.contains("LICENSE-APACHE"));
    assert!(pages["404.html"].contains(">v9.9<"));
}

#[test]
fn without_optional_keys_default_chrome_is_kept() {
    let pages = build("none", "");
    for key in ["index.html", "404.html", "old/index.html"] {
        assert!(pages[key].contains("<html lang=\"ja\">"), "{key}");
    }
    let index = &pages["index.html"];
    assert!(index.contains("core v"));
    assert!(index.contains("Licensed under"));
    assert!(index.contains("crates.io/crates"));
    assert!(!index.contains("Built with "));
}

#[test]
fn lang_only_does_not_switch_attribution() {
    let pages = build("lang-only", "lang = \"en\"\n");
    let index = &pages["index.html"];
    assert!(index.contains("<html lang=\"en\">"));
    assert!(index.contains("Licensed under"));
    assert!(!index.contains("Built with "));
}

#[test]
fn empty_version_badge_hides_badge_and_keeps_attribution() {
    let pages = build("hidden", "version_badge = \"\"\n");
    let index = &pages["index.html"];
    assert!(!index.contains("docs-brand-version"));
    assert!(!index.contains("crates.io/crates"));
    assert!(index.contains("Built with "));
}

#[test]
fn binary_rejects_invalid_lang_without_writing_output() {
    let temp = TempDir::new("bad-lang");
    let root = temp.0.join("repo");
    write_site(&root, "lang = \"ja_JP\"\n");
    let out = temp.0.join("dist");
    let output = Command::new(env!("CARGO_BIN_EXE_docs-site"))
        .arg("--no-page-sections")
        .arg("--root")
        .arg(&root)
        .arg("--out")
        .arg(&out)
        .output()
        .expect("spawn docs-site binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("site.lang"), "stderr={stderr}");
    assert!(!out.exists(), "out_dir must not exist on parse failure");
}
