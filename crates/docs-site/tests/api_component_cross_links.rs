//! イシュー #955（設計 `docs/design/docs-site-api-reference-split.md` §3-2）:
//! API Reference セクションの API ページと Components セクションの部品ページ
//! ( `/themes/<kebab>/`。イシュー #1017 で `/components/<kebab>/` から移行 )
//! は相互リンクで結ばれている契約を機械固定する。
//!
//! - API ページ → 部品ページ: `docs/api/headless-ui-api.md` /
//!   `docs/api/pre-styled-ui-api.md` が `../../site/themes/<kebab>.md`
//!   形式のリンクで指す先はすべて nav 登録済みの部品ページであること。
//! - 部品ページ → API ページ: 全 107 部品ページが
//!   `../../docs/api/pre-styled-ui-api.md` へのリンクを持ち、その集合は
//!   `pre-styled-ui-api.md` 側が指す 107 件と完全一致すること（イシュー
//!   #994 で Callout が加わり 102 → 103、イシュー #995 で Quote / Strong が
//!   加わり 103 → 105、イシュー #996 で Tab Nav が加わり 105 → 106、
//!   イシュー #997 で Checkbox Group が加わり 106 → 107）。
//! - headless-ui 裏付けを持つ部品ページ（nav 登録の部品ページ kebab と
//!   `crates/headless-ui/src/<snake>.rs` mod 名の共通集合）は
//!   `docs/api/headless-ui-api.md` と双方向にリンクしていること。
//!
//! いずれもリンクの「存在（相対パスの部分文字列としての出現）」だけを
//! 表明し、行全体の文言までは固定しない（原稿充填・言い回し変更で
//! テストが壊れない粒度に留める設計判断、計画 §5-2 参照）。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use fandhe_frontend_docs_site::nav::{parse_nav, Nav};

/// `CARGO_MANIFEST_DIR`（`crates/docs-site`）から repo_root を解決する。
/// `site_nav.rs` と同じ解決規則（`Cargo.toml` の `members = ["crates/*"]`
/// 構成に対応する 2 階層上）に合わせる。
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

/// nav 登録済みの部品ページ kebab 名の集合（`site/themes/<kebab>.md`
/// 由来の 104 件）を返す。
fn nav_component_kebabs() -> BTreeSet<String> {
    let nav = load_nav();
    nav.all_pages()
        .filter_map(|p| {
            p.source
                .strip_prefix("site/themes/")
                .and_then(|rest| rest.strip_suffix(".md"))
                .map(|kebab| kebab.to_string())
        })
        .collect()
}

/// `crates/headless-ui/src/<snake>.rs` の mod 名を kebab-case へ変換した
/// 集合を返す。`lib.rs` はクレート入口でありコンポーネント mod ではないため
/// 除外する。
fn headless_ui_mod_kebabs() -> BTreeSet<String> {
    let src_dir = repo_root().join("crates/headless-ui/src");
    let mut mods = BTreeSet::new();
    for entry in std::fs::read_dir(&src_dir)
        .unwrap_or_else(|e| panic!("failed to read crates/headless-ui/src/: {e}"))
    {
        let entry = entry.unwrap_or_else(|e| panic!("failed to read src/ entry: {e}"));
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("mod file stem should be valid UTF-8");
        if stem == "lib" {
            continue;
        }
        mods.insert(stem.replace('_', "-"));
    }
    mods
}

/// headless-ui 裏付けを持つ部品ページ kebab（nav 登録 104 件と headless-ui
/// mod 名の共通集合）。基盤 mod（state/anatomy 等）が将来たまたま部品ページ
/// と同名になった場合に誤って対象化しないよう、例外は理由付きの明示リスト
/// でのみ許容する（現時点では例外なし）。
fn headless_backed_component_kebabs() -> BTreeSet<String> {
    // 部品ページと同名の基盤 mod が現れた場合はここへ理由付きで追加する
    // （暗黙スキップを作らない、計画 §5-2 の方針）。現時点では該当なし。
    const EXCLUDED_FOUNDATION_MODS: &[&str] = &[];

    let nav_kebabs = nav_component_kebabs();
    let headless_kebabs = headless_ui_mod_kebabs();
    nav_kebabs
        .intersection(&headless_kebabs)
        .filter(|kebab| !EXCLUDED_FOUNDATION_MODS.contains(&kebab.as_str()))
        .cloned()
        .collect()
}

fn read_repo_file(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {relative}: {e}"))
}

/// `docs/api/*.md` 本文中の `../../site/themes/<kebab>.md` 相対リンクを
/// すべて抽出する。
fn extract_component_link_kebabs(markdown: &str) -> BTreeSet<String> {
    const PREFIX: &str = "../../site/themes/";
    const SUFFIX: &str = ".md";
    let mut found = BTreeSet::new();
    let mut rest = markdown;
    while let Some(start) = rest.find(PREFIX) {
        let after_prefix = &rest[start + PREFIX.len()..];
        if let Some(end) = after_prefix.find(SUFFIX) {
            let kebab = &after_prefix[..end];
            // kebab セグメントは英数字・ハイフンのみ（ディレクトリ区切りを
            // 含む誤マッチを避ける fail-closed なガード）。
            if !kebab.is_empty()
                && kebab
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            {
                found.insert(kebab.to_string());
            }
            rest = &after_prefix[end + SUFFIX.len()..];
        } else {
            break;
        }
    }
    found
}

/// 受け入れ条件 3（相互リンク）の片側: API ページ → 部品ページ。
/// `headless-ui-api.md` / `pre-styled-ui-api.md` 中の部品ページリンクは
/// すべて nav 登録済みの部品ページを指していること。
#[test]
fn api_links_to_component_pages_are_all_nav_registered() {
    let nav_kebabs = nav_component_kebabs();
    assert_eq!(
        nav_kebabs.len(),
        107,
        "expected 107 nav-registered component pages, got {}",
        nav_kebabs.len()
    );

    for api_source in [
        "docs/api/headless-ui-api.md",
        "docs/api/pre-styled-ui-api.md",
    ] {
        let markdown = read_repo_file(api_source);
        let linked = extract_component_link_kebabs(&markdown);
        assert!(
            !linked.is_empty(),
            "{api_source} should link to at least one component page"
        );
        for kebab in &linked {
            assert!(
                nav_kebabs.contains(kebab),
                "{api_source} links to site/themes/{kebab}.md, which is not registered in \
                 site/nav.toml"
            );
        }
    }
}

/// 受け入れ条件 3 のもう片側: 全 107 部品ページが
/// `pre-styled-ui-api.md` へ委譲リンクし、`pre-styled-ui-api.md` 側が指す
/// 部品ページ集合と完全一致すること（過不足ゼロ）。
#[test]
fn every_component_page_links_back_to_pre_styled_ui_api() {
    const LINK_FRAGMENT: &str = "../../docs/api/pre-styled-ui-api.md";

    let nav_kebabs = nav_component_kebabs();
    assert_eq!(nav_kebabs.len(), 107);

    let mut pages_missing_link = Vec::new();
    for kebab in &nav_kebabs {
        let page_source = format!("site/themes/{kebab}.md");
        let markdown = read_repo_file(&page_source);
        if !markdown.contains(LINK_FRAGMENT) {
            pages_missing_link.push(page_source);
        }
    }
    assert!(
        pages_missing_link.is_empty(),
        "component pages missing a link back to pre-styled-ui-api.md: {pages_missing_link:?}"
    );

    let api_markdown = read_repo_file("docs/api/pre-styled-ui-api.md");
    let linked_from_api = extract_component_link_kebabs(&api_markdown);
    assert_eq!(
        linked_from_api, nav_kebabs,
        "docs/api/pre-styled-ui-api.md component links must exactly match the 107 nav-registered \
         component pages (no missing, no stale entries)"
    );
}

/// 受け入れ条件 3: headless-ui 裏付けを持つ部品ページ（nav 登録の部品ページ
/// kebab と `crates/headless-ui/src/<snake>.rs` mod 名の共通集合）は
/// `docs/api/headless-ui-api.md` と双方向にリンクしていること。
#[test]
fn headless_backed_component_pages_link_bidirectionally_with_headless_ui_api() {
    const LINK_FRAGMENT: &str = "../../docs/api/headless-ui-api.md";

    let headless_backed = headless_backed_component_kebabs();
    assert_eq!(
        headless_backed.len(),
        60,
        "expected 60 headless-ui-backed component pages (nav ∩ headless-ui src mods), got {}: \
         {headless_backed:?}",
        headless_backed.len()
    );

    let api_markdown = read_repo_file("docs/api/headless-ui-api.md");
    let linked_from_api = extract_component_link_kebabs(&api_markdown);

    let mut pages_missing_link = Vec::new();
    let mut api_missing_link = Vec::new();
    for kebab in &headless_backed {
        let page_source = format!("site/themes/{kebab}.md");
        let markdown = read_repo_file(&page_source);
        if !markdown.contains(LINK_FRAGMENT) {
            pages_missing_link.push(page_source);
        }
        if !linked_from_api.contains(kebab) {
            api_missing_link.push(kebab.clone());
        }
    }
    assert!(
        pages_missing_link.is_empty(),
        "headless-ui-backed component pages missing a link back to headless-ui-api.md: \
         {pages_missing_link:?}"
    );
    assert!(
        api_missing_link.is_empty(),
        "docs/api/headless-ui-api.md is missing links to headless-ui-backed component pages: \
         {api_missing_link:?}"
    );
}
