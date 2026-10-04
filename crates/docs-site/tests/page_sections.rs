//! 汎用生成節フック（`page_sections`、イシュー #3598）の契約テスト。
//!
//! 空の登録表で出力が変わらないこと（AC1）・登録ページだけが変わること
//! （AC2）・`/blocks/` 索引との合成順（AC3 補強）・登録表の fail-closed 検証
//! を固定する。実サイトのフルビルドは行わず、フィクスチャサイト
//! （`tests/fixtures/site-ok`）と実サイトの Markdown 描画のみを使う。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_docs_site::blocks;
use fandhe_frontend_docs_site::build::{build_site_with, BuildError};
use fandhe_frontend_docs_site::layout::PageLayout;
use fandhe_frontend_docs_site::markdown::render_markdown;
use fandhe_frontend_docs_site::nav::{parse_nav, Nav};
use fandhe_frontend_docs_site::page_sections::{
    insert_generated_sections_with, stylesheets_for_path_in, validate, PageSection,
    PageSectionError, PageStylesheet, Placement, Registry, EMPTY_REGISTRY, PAGE_SECTIONS,
    PAGE_STYLESHEETS, REGISTRY,
};
use fandhe_frontend_docs_site::wireframes;
use fandhe_frontend_pre_styled_ui::{StyleSheet, StylesheetError};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scratch(tag: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "docs-site-page-sections-{tag}-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir_all(&path).expect("create scratch dir");
    path
}

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/site-ok")
}

fn real_nav() -> Nav {
    let input = std::fs::read_to_string(repo_root().join("site/nav.toml")).expect("read nav");
    parse_nav(&input).expect("parse nav")
}

const TARGET: &str = "/guide/quickstart/";
const SHEET: &str = "assets/page-sections-test.css";

fn marker(_nav: &Nav, _path: &str) -> Vec<Node> {
    vec![div(
        vec![("class", "docs-page-sections-test")],
        vec![p(vec![], vec![text("generated")])],
    )]
}

fn sheet_css() -> Result<StyleSheet, StylesheetError> {
    let mut s = StyleSheet::new();
    s.push_css(".docs-page-sections-test { margin: 0; }\n")?;
    Ok(s)
}

static SHEETS: [PageStylesheet; 1] = [PageStylesheet {
    rel_path: SHEET,
    build: sheet_css,
}];

fn registry_for(path: &'static str, placement: Placement) -> Registry {
    let sections: &'static [PageSection] = Box::leak(Box::new([PageSection {
        path,
        placement,
        render: marker,
        stylesheets: &[SHEET],
        layout: PageLayout::Docs,
        optional_menu: false,
    }]));
    Registry {
        sections,
        stylesheets: &SHEETS,
    }
}

const EMPTY: Registry = EMPTY_REGISTRY;

fn read_tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(dir: &Path, root: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).expect("read_dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                walk(&path, root, out);
            } else {
                let rel = path.strip_prefix(root).expect("prefix");
                out.insert(
                    rel.to_string_lossy().into_owned(),
                    std::fs::read(&path).expect("read"),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

/// 実サイトの Markdown 描画 + blocks/wireframes の挿入後のノード列。
fn pre_hook_blocks(nav: &Nav, page: &fandhe_frontend_docs_site::nav::Page) -> Vec<Node> {
    let md = std::fs::read_to_string(repo_root().join(&page.source)).expect("read md");
    let b =
        blocks::insert_generated_sections(&page.path, &nav.site.base_path, render_markdown(&md));
    wireframes::insert_generated_sections(&page.path, &nav.site.base_path, b)
}

#[test]
fn production_registry_matches_the_expected_table() {
    let got: Vec<_> = PAGE_SECTIONS
        .iter()
        .map(|s| (s.path, s.placement, s.layout))
        .collect();
    assert_eq!(
        got,
        [
            ("/api/", Placement::BeforeFirstH2, PageLayout::Docs),
            ("/assets/", Placement::Append, PageLayout::Landing),
            ("/blocks/", Placement::Append, PageLayout::Docs),
            ("/examples/", Placement::BeforeFirstH2, PageLayout::Docs),
            ("/guides/", Placement::Append, PageLayout::Docs),
            ("/primitives/", Placement::BeforeFirstH2, PageLayout::Docs),
            ("/themes/", Placement::BeforeFirstH2, PageLayout::Docs),
            ("/wireframes/", Placement::BeforeFirstH2, PageLayout::Docs),
            ("/", Placement::Prepend, PageLayout::Landing),
        ],
        "本番登録表の期待表（登録を増やすときは本表へ明示的に追加する）"
    );
    for s in PAGE_SECTIONS {
        for rel in s.stylesheets {
            assert!(PAGE_STYLESHEETS.iter().any(|p| p.rel_path == *rel), "{rel}");
        }
    }
    assert_eq!(validate(&REGISTRY, &real_nav()), Ok(()));
}

/// イシュー #3700: 本番登録表は全メニュー集約ページを Landing 骨格・
/// `menu_index::render` で登録している（登録漏れは Docs 骨格・カード無しへ
/// 黙って退行するため）。
#[test]
fn production_registry_covers_every_menu_index_page() {
    let nav = real_nav();
    assert!(!nav.menus.is_empty());
    for menu in &nav.menus {
        assert_eq!(
            fandhe_frontend_docs_site::page_sections::layout_for_path_in(
                &REGISTRY,
                &menu.index_path
            ),
            PageLayout::Landing,
            "{}",
            menu.index_path
        );
        let nodes = insert_generated_sections_with(&REGISTRY, &nav, &menu.index_path, Vec::new());
        let html: String = nodes.iter().map(fandhe_frontend_core::render).collect();
        assert_eq!(
            html.matches("docs-index-card-link").count(),
            menu.items.len(),
            "{}",
            menu.index_path
        );
    }
}

/// AC1: 空の登録表では全ページのノード列が変わらず、追加 CSS も配線されない。
#[test]
fn empty_registry_is_identity_for_every_real_page() {
    let nav = real_nav();
    for page in nav.all_pages() {
        let before = pre_hook_blocks(&nav, page);
        let after = insert_generated_sections_with(&EMPTY, &nav, &page.path, before.clone());
        assert_eq!(before, after, "page {} changed", page.path);
        assert!(stylesheets_for_path_in(&EMPTY, &page.path).is_empty());
    }
}

/// AC2: 登録ページの index.html と合成 CSS の追加だけが差分になる。
#[test]
fn registered_page_alone_changes_in_built_output() {
    let base_out = scratch("base");
    let hook_out = scratch("hook");
    build_site_with(&fixture_root(), &base_out, &EMPTY).expect("baseline build");
    let reg = registry_for(TARGET, Placement::BeforeFirstH2);
    build_site_with(&fixture_root(), &hook_out, &reg).expect("hook build");

    let before = read_tree(&base_out);
    let after = read_tree(&hook_out);
    let target_file = "guide/quickstart/index.html".to_string();
    let css_file = SHEET.to_string();

    for (name, bytes) in &before {
        // 生成節は検索インデックスにも載る設計（モジュール doc 参照）。
        // 対象ページを含むセクションのファイルは差分になり得る。
        if *name == target_file || name == "assets/search-index/guide.json" {
            continue;
        }
        assert_eq!(after.get(name), Some(bytes), "unexpected change in {name}");
    }
    let added: Vec<&String> = after.keys().filter(|k| !before.contains_key(*k)).collect();
    assert_eq!(added, vec![&css_file]);

    let html = String::from_utf8(after[&target_file].clone()).expect("utf8");
    assert!(html.contains("docs-page-sections-test"));
    assert!(html.contains(r#"href="/fixture-base/assets/page-sections-test.css""#));
    // 位置: 最初の h2（Section Heading）の直前
    let marker_at = html.find("docs-page-sections-test\"").expect("marker");
    let h2_at = html.find("id=\"section-heading\"").expect("h2");
    assert!(marker_at < h2_at);
    let home = String::from_utf8(after["index.html"].clone()).expect("utf8");
    assert!(!home.contains("docs-page-sections-test"));
    assert!(!home.contains("page-sections-test.css"));

    let _ = std::fs::remove_dir_all(&base_out);
    let _ = std::fs::remove_dir_all(&hook_out);
}

/// `/blocks/` 索引の生成は汎用フックが単独で担う（#3618）。フック適用前の
/// ノード列にはカテゴリグリッドがなく、本番 `REGISTRY` で挿入するとちょうど 1 回入る。
#[test]
fn blocks_index_grid_comes_only_from_the_production_hook() {
    use fandhe_frontend_core::render;
    let nav = real_nav();
    let page = nav
        .all_pages()
        .find(|p| p.path == blocks::INDEX_PATH)
        .expect("blocks index page");
    let existing = pre_hook_blocks(&nav, page);
    let before: String = existing.iter().map(render).collect();
    assert!(!before.contains("docs-category-"));
    let out = insert_generated_sections_with(&REGISTRY, &nav, &page.path, existing.clone());
    let after: String = out.iter().map(render).collect();
    assert_eq!(
        after.matches("class=\"docs-category-grid\"").count(),
        blocks::BlockSection::ALL
            .iter()
            .filter(|s| blocks::all_blocks()
                .iter()
                .any(|b| b.category.section() == **s))
            .count()
    );
    assert_eq!(out[..existing.len()], existing[..]);
}

#[test]
fn validate_rejects_registrations_on_wireframe_pages() {
    let nav = real_nav();
    let path = wireframes::WIREFRAMES
        .first()
        .map(|w| w.path)
        .expect("a wireframe page");
    assert_eq!(
        validate(&registry_for(path, Placement::Append), &nav),
        Err(PageSectionError::ConflictsWithGeneratedPage(path.into()))
    );
}

fn assert_build_rejects(reg: &Registry, tag: &str) {
    let out = scratch(tag).join("out");
    let err = build_site_with(&fixture_root(), &out, reg).expect_err("must fail closed");
    assert!(matches!(err, BuildError::PageSection(_)), "{err}");
    assert!(!out.exists(), "out_dir must not be written");
}

#[test]
fn build_fails_closed_before_writing_on_invalid_registry() {
    // 存在しないページ
    assert_build_rejects(&registry_for("/nope/", Placement::Append), "unknown");
    // 重複
    let two: &'static [PageSection] = Box::leak(Box::new([
        registry_for(TARGET, Placement::Append).sections[0],
        registry_for(TARGET, Placement::Prepend).sections[0],
    ]));
    assert_build_rejects(
        &Registry {
            sections: two,
            stylesheets: &SHEETS,
        },
        "dup",
    );
    // 未登録の stylesheet 参照
    assert_build_rejects(
        &Registry {
            sections: registry_for(TARGET, Placement::Append).sections,
            stylesheets: &[],
        },
        "unknown-sheet",
    );
    // 予約名と衝突する stylesheet
    let reserved: &'static [PageStylesheet] = Box::leak(Box::new([PageStylesheet {
        rel_path: "assets/site.css",
        build: sheet_css,
    }]));
    assert_build_rejects(
        &Registry {
            sections: &[],
            stylesheets: reserved,
        },
        "reserved-sheet",
    );
}

/// 静的アセットが登録 CSS と同名のとき、生成物のすり替えを拒否する。
#[test]
fn static_asset_with_registered_stylesheet_name_is_rejected() {
    let root = scratch("static-collision");
    fn copy(src: &Path, dst: &Path) {
        std::fs::create_dir_all(dst).expect("mkdir");
        for e in std::fs::read_dir(src).expect("read_dir") {
            let e = e.expect("entry");
            let to = dst.join(e.file_name());
            if e.path().is_dir() {
                copy(&e.path(), &to);
            } else {
                std::fs::copy(e.path(), &to).expect("copy");
            }
        }
    }
    copy(&fixture_root(), &root);
    std::fs::create_dir_all(root.join("site/assets")).expect("assets dir");
    std::fs::write(root.join("site/assets/page-sections-test.css"), "a{}").expect("write");
    let out = scratch("static-collision-out").join("out");
    let reg = registry_for(TARGET, Placement::Append);
    let err = build_site_with(&root, &out, &reg).expect_err("must reject");
    assert!(matches!(err, BuildError::ReservedAssetName(_)), "{err}");
    assert!(!out.exists());
    let _ = std::fs::remove_dir_all(&root);
}
