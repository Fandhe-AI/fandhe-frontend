//! `site/nav.toml` の Wireframes セクション・`crate::wireframes::WIREFRAMES`
//! レジストリ・`site/wireframes/*.md` 原稿ファイル集合の三方突合
//! （イシュー #2607）。
//!
//! `crates/docs-site/tests/blocks_nav.rs`（Blocks 台帳の三方突合）と同型の
//! ドリフト検知テストである。Phase 1〜8（#2608〜#2665）が部品を追加する際、
//! nav.toml・レジストリ・原稿ファイルのいずれか 1 箇所だけの更新漏れを
//! fail-closed に検知する。本イシュー時点ではレジストリ・原稿ファイルは
//! いずれも 0 件のため大半のアサーションは vacuous に通過するが、経路自体は
//! 実効化されており Phase 1 以降で即座に機能する。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use fandhe_frontend_docs_site::nav::{parse_nav, Nav};
use fandhe_frontend_docs_site::wireframes;

/// `CARGO_MANIFEST_DIR`（`crates/docs-site`）から repo_root を解決する
/// （`tests/site_nav.rs`/`tests/blocks_nav.rs` と同じ規約）。
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

/// Wireframes セクションが Blocks の直後・API Reference の直前に存在し、
/// `index_path`・group 非使用（フラット構成、設計文書 §12 §4.1）が
/// 期待どおりであること。
#[test]
fn wireframes_section_is_registered_immediately_after_blocks() {
    let nav = load_nav();
    let index = nav
        .sections
        .iter()
        .position(|s| s.title == "Wireframes")
        .expect("Wireframes section should be registered");
    assert_eq!(nav.sections[index - 1].title, "Blocks");
    assert_eq!(nav.sections[index + 1].title, "API Reference");

    let section = &nav.sections[index];
    assert_eq!(section.index_path, "/wireframes/");
    assert!(
        section.groups.is_empty(),
        "Wireframes section should use flat [[section.page]] only (no [[section.group]])"
    );
}

/// `site/nav.toml` の `/wireframes/*` ページ（索引を除く）と
/// `wireframes::WIREFRAMES` の `path` が完全一致すること（登録漏れ・孤児の
/// いずれも検知する）。
#[test]
fn nav_toml_wireframe_pages_match_the_registry_exactly() {
    let nav = load_nav();
    let section = nav
        .sections
        .iter()
        .find(|s| s.title == "Wireframes")
        .expect("Wireframes section should be registered");

    let nav_wireframe_paths: BTreeSet<&str> = section
        .all_pages()
        .map(|p| p.path.as_str())
        .filter(|path| *path != section.index_path)
        .collect();
    let registry_paths: BTreeSet<&str> = wireframes::WIREFRAMES.iter().map(|w| w.path).collect();

    assert_eq!(
        nav_wireframe_paths, registry_paths,
        "nav.toml の /wireframes/* ページ（索引除く）と wireframes::WIREFRAMES の path が一致しない"
    );
}

/// `wireframes::WIREFRAMES` の各 `path` に対応する `site/wireframes/<kebab>.md`
/// が実在すること（nav.toml の `source` フィールドとも一致させる）。
#[test]
fn every_registered_wireframe_has_a_manuscript_file() {
    let nav = load_nav();
    let section = nav
        .sections
        .iter()
        .find(|s| s.title == "Wireframes")
        .expect("Wireframes section should be registered");

    for wireframe in wireframes::WIREFRAMES {
        let page = section
            .all_pages()
            .find(|p| p.path == wireframe.path)
            .unwrap_or_else(|| panic!("nav.toml should declare a page for {}", wireframe.path));
        let source_path = repo_root().join(&page.source);
        assert!(
            source_path.is_file(),
            "manuscript file {source_path:?} for wireframe {} should exist",
            wireframe.path
        );
    }
}

/// `site/wireframes/*.md` 原稿ファイル集合と `wireframes::WIREFRAMES`/
/// `nav.toml` の三方目を締める（`blocks_nav.rs` の同名テストと同型）。
/// `site/wireframes/` に置かれたが `wireframes::WIREFRAMES`/`nav.toml` の
/// どちらにも登録されていない孤児原稿ファイルを検知する。`.gitkeep` は
/// 拡張子フィルタで無視する。
#[test]
fn site_wireframes_dir_manuscripts_match_the_registry_exactly() {
    let dir = repo_root().join("site/wireframes");
    let entries = std::fs::read_dir(&dir).expect("site/wireframes directory should be readable");

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

    let expected: BTreeSet<String> = wireframes::WIREFRAMES
        .iter()
        .map(|wireframe| {
            let kebab = wireframe
                .path
                .trim_start_matches("/wireframes/")
                .trim_end_matches('/');
            format!("{kebab}.md")
        })
        .collect();

    assert_eq!(
        on_disk, expected,
        "site/wireframes/*.md の実在ファイル集合と wireframes::WIREFRAMES から導出した期待集合が一致しない          （未登録の孤児原稿ファイル、または登録済みだがファイルが無い wireframe のいずれか）"
    );
}

/// `site/wireframes.md` が索引ページとして登録されていること。
#[test]
fn wireframes_index_page_is_registered() {
    let nav = load_nav();
    let section = nav
        .sections
        .iter()
        .find(|s| s.title == "Wireframes")
        .expect("Wireframes section should be registered");
    let index_page = section
        .pages
        .iter()
        .find(|p| p.path == "/wireframes/")
        .expect("Wireframes section should declare its index page as a direct page");
    assert_eq!(index_page.source, "site/wireframes.md");
}

/// 登録済み部品の `path` 末尾 kebab が
/// `docs/design/wireframe-ui-architecture.md` §8 の 49 kebab 集合に含まれる
/// こと（kebab 命名のドリフト検知。`media` 等の固定を機械化する）。
#[test]
fn registered_wireframe_kebabs_are_within_the_expected_49() {
    const EXPECTED_KEBABS: &[&str] = &[
        // Phase 1: レイアウト骨格
        "frame",
        "stack",
        "grid",
        "divider",
        // Phase 2: テキスト・注釈
        "text",
        "paragraph",
        "rich-text",
        "annotation",
        "link",
        "tag",
        // Phase 3: Forms A
        "button",
        "input",
        "textarea",
        "select",
        "checkbox",
        "radio",
        "switch",
        "slider",
        // Phase 4: Forms B
        "question",
        "ratings",
        "calendar",
        "file-drop",
        "stepper",
        // Phase 5: Navigation
        "nav-item",
        "menu",
        "tabs",
        "breadcrumbs",
        "pagination",
        "accordion",
        "cursor",
        // Phase 6: Overlay・Feedback
        "tooltip",
        "modal",
        "alert",
        "toast",
        "progress",
        "spinner",
        // Phase 7: Data display
        "avatar",
        "icon",
        "brand",
        "emoji",
        "counter",
        "stat",
        "list",
        "card-basic",
        // Phase 8: Media・Data
        "image",
        "media",
        "table",
        "chart",
        "map",
    ];
    assert_eq!(EXPECTED_KEBABS.len(), 49, "expected exactly 49 kebabs");

    for wireframe in wireframes::WIREFRAMES {
        let kebab = wireframe
            .path
            .trim_start_matches("/wireframes/")
            .trim_end_matches('/');
        assert!(
            EXPECTED_KEBABS.contains(&kebab),
            "registered wireframe path {} has an unexpected kebab {kebab:?} not in the §8 list",
            wireframe.path
        );
    }
}
