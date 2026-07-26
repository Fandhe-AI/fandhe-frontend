//! `nav.toml` の 3 階層スキーマ（`[[section.group]]` / `[[section.group.page]]`）
//! の正常系・異常系を公開 API（`fandhe_frontend_docs_site::nav`）経由で
//! 検証する統合テスト（イシュー #939）。
//!
//! 仕様の正本は `docs/design/docs-site-component-pages.md` §6。異常系は
//! 同文書のエラー写像表に対応する `NavError` バリアント・（`Parse` へ
//! 写像される 3 件は）固定メッセージ文字列まで固定する。
//!
//! `crates/docs-site/src/nav.rs` の `#[cfg(test)] mod tests` が既存 2 階層
//! スキーマ（グループなし）の後方互換・`EmptySection` 条件是正の回帰を
//! 別途固定しているため、本ファイルはグループ機能そのものの正常系・
//! 異常系・走査順・XSS 回帰に限定する。

use std::path::PathBuf;

use fandhe_frontend_core::render;
use fandhe_frontend_docs_site::nav::{header_nav, parse_nav, sidebar, validate_sources, NavError};

/// 統合テストのスクラッチ基点。`CARGO_TARGET_TMPDIR` は cargo が統合テスト
/// バイナリの**コンパイル時のみ**設定する（Cargo Book）ため `env!` で確定し、
/// 実行時 env による明示上書きのみ許容する。`/tmp` へは一切フォールバック
/// しない（イシュー #637 の事実誤認の再発防止、#658、`tests/site_build.rs`
/// と同一パターン）。
fn scratch_root() -> PathBuf {
    let root = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
    let _ = std::fs::create_dir_all(&root);
    root
}

/// テスト専用の一時ディレクトリ。`crates/docs-site/src/nav.rs` の
/// `TempDir`・`tests/site_build.rs` の `TempDir` と同方針（外部クレート
/// `tempfile` を追加しない、REQ-3）。
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = scratch_root().join(format!(
            "fandhe-frontend-docs-site-nav-group-schema-{tag}-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("create temp dir for nav_group_schema.rs test");
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

// ---- 正常系 ----

#[test]
fn parses_three_tier_schema_with_declaration_order_preserved() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/components/button/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "button.md"
path = "/components/button/"

[[section.group.page]]
title = "Checkbox"
source = "checkbox.md"
path = "/components/checkbox/"

[[section.group]]
title = "Overlays"

[[section.group.page]]
title = "Dialog"
source = "dialog.md"
path = "/components/dialog/"

[[section.group.page]]
title = "Tooltip"
source = "tooltip.md"
path = "/components/tooltip/"
"#;
    let nav = parse_nav(input).expect("3 tier nav.toml should parse");
    assert_eq!(nav.sections.len(), 1);
    let section = &nav.sections[0];
    assert!(section.pages.is_empty());
    assert_eq!(section.groups.len(), 2);

    assert_eq!(section.groups[0].title, "Forms");
    assert_eq!(section.groups[0].pages.len(), 2);
    assert_eq!(section.groups[0].pages[0].title, "Button");
    assert_eq!(section.groups[0].pages[0].source, "button.md");
    assert_eq!(section.groups[0].pages[0].path, "/components/button/");
    assert_eq!(section.groups[0].pages[1].title, "Checkbox");

    assert_eq!(section.groups[1].title, "Overlays");
    assert_eq!(section.groups[1].pages.len(), 2);
    assert_eq!(section.groups[1].pages[0].title, "Dialog");
    assert_eq!(section.groups[1].pages[1].title, "Tooltip");
}

#[test]
fn two_tier_nav_toml_without_groups_still_parses_with_empty_groups() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Guide"
index_path = "/intro/"

[[section.page]]
title = "Intro"
source = "intro.md"
path = "/intro/"
"#;
    let nav = parse_nav(input).expect("legacy 2 tier nav.toml should still parse");
    assert_eq!(nav.sections[0].pages.len(), 1);
    assert!(nav.sections[0].groups.is_empty());
}

#[test]
fn section_can_hold_direct_pages_and_groups_at_the_same_time() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/components/"

[[section.page]]
title = "Overview"
source = "overview.md"
path = "/components/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "button.md"
path = "/components/button/"
"#;
    let nav = parse_nav(input).expect("direct pages and groups should coexist");
    let section = &nav.sections[0];
    assert_eq!(section.pages.len(), 1);
    assert_eq!(section.pages[0].title, "Overview");
    assert_eq!(section.groups.len(), 1);
    assert_eq!(section.groups[0].pages[0].title, "Button");
}

/// `[[section.group]]` の後に現れた `[[section.page]]` は直下ページとして
/// 扱い、グループへ吸着しない（宣言順に追加制約を課さない仕様、
/// `docs/design/docs-site-component-pages.md` §6 参照）。
#[test]
fn section_page_declared_after_a_group_stays_a_direct_page() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/components/button/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "button.md"
path = "/components/button/"

[[section.page]]
title = "Overview"
source = "overview.md"
path = "/components/"
"#;
    let nav = parse_nav(input).expect("late section.page should not error");
    let section = &nav.sections[0];
    assert_eq!(section.pages.len(), 1);
    assert_eq!(section.pages[0].title, "Overview");
    assert_eq!(section.groups.len(), 1);
    assert_eq!(section.groups[0].pages[0].title, "Button");

    // 走査順は「直下ページ → グループ配下ページ」(§6-2 の描画順契約)。
    let ordered: Vec<&str> = section.all_pages().map(|p| p.title.as_str()).collect();
    assert_eq!(ordered, vec!["Overview", "Button"]);
}

#[test]
fn section_with_only_a_group_and_no_direct_pages_is_not_empty_section() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/components/button/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "button.md"
path = "/components/button/"
"#;
    let nav = parse_nav(input).expect("group-only section should not be EmptySection");
    assert!(nav.sections[0].pages.is_empty());
    assert_eq!(nav.sections[0].groups.len(), 1);
}

#[test]
fn nav_all_pages_enumerates_every_page_in_document_order() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Guide"
index_path = "/intro/"

[[section.page]]
title = "Intro"
source = "intro.md"
path = "/intro/"

[[section]]
title = "Components"
index_path = "/components/button/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "button.md"
path = "/components/button/"

[[section.group]]
title = "Overlays"

[[section.group.page]]
title = "Dialog"
source = "dialog.md"
path = "/components/dialog/"
"#;
    let nav = parse_nav(input).expect("valid nav.toml should parse");
    let titles: Vec<&str> = nav.all_pages().map(|p| p.title.as_str()).collect();
    assert_eq!(titles, vec!["Intro", "Button", "Dialog"]);
}

#[test]
fn validate_sources_checks_group_pages_against_the_filesystem() {
    let temp = TempDir::new("group-sources");
    std::fs::write(temp.0.join("button.md"), b"# Button").expect("write fixture source file");
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/components/button/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "button.md"
path = "/components/button/"
"#;
    let nav = parse_nav(input).expect("valid nav.toml should parse");
    assert!(validate_sources(&nav, &temp.0).is_ok());
}

#[test]
fn validate_sources_reports_missing_group_page_source() {
    let temp = TempDir::new("group-missing-source");
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/components/button/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "does-not-exist.md"
path = "/components/button/"
"#;
    let nav = parse_nav(input).expect("structurally valid nav.toml should parse");
    match validate_sources(&nav, &temp.0) {
        Err(NavError::MissingSource(source)) => assert_eq!(source, "does-not-exist.md"),
        other => panic!("expected MissingSource, got {other:?}"),
    }
}

// ---- 異常系 ----

#[test]
fn rejects_unknown_key_in_section_group_with_exact_message() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/components/forms/"

[[section.group]]
title = "Forms"
weight = "1"
"#;
    match parse_nav(input) {
        Err(NavError::Parse { message, .. }) => {
            assert_eq!(message, "unknown key `weight` in [[section.group]]");
        }
        other => panic!("expected Parse, got {other:?}"),
    }
}

#[test]
fn rejects_unknown_key_in_section_group_page_with_exact_message() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/components/button/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "button.md"
path = "/components/button/"
weight = "1"
"#;
    match parse_nav(input) {
        Err(NavError::Parse { message, .. }) => {
            assert_eq!(message, "unknown key `weight` in [[section.group.page]]");
        }
        other => panic!("expected Parse, got {other:?}"),
    }
}

#[test]
fn rejects_missing_group_title() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/components/button/"

[[section.group]]

[[section.group.page]]
title = "Button"
source = "button.md"
path = "/components/button/"
"#;
    match parse_nav(input) {
        Err(NavError::MissingKey { context, key }) => {
            assert_eq!(context, "section.group");
            assert_eq!(key, "title");
        }
        other => panic!("expected MissingKey, got {other:?}"),
    }
}

#[test]
fn rejects_missing_group_page_path() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/components/button/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "button.md"
"#;
    match parse_nav(input) {
        Err(NavError::MissingKey { context, key }) => {
            assert_eq!(context, "section.group.page");
            assert_eq!(key, "path");
        }
        other => panic!("expected MissingKey, got {other:?}"),
    }
}

#[test]
fn rejects_empty_group_with_no_pages() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/components/forms/"

[[section.group]]
title = "Forms"
"#;
    match parse_nav(input) {
        Err(NavError::EmptyGroup(title)) => assert_eq!(title, "Forms"),
        other => panic!("expected EmptyGroup, got {other:?}"),
    }
}

#[test]
fn rejects_two_level_nested_group_with_exact_message() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/components/nested/"

[[section.group]]
title = "Forms"

[[section.group.group]]
title = "Nested"
"#;
    match parse_nav(input) {
        Err(NavError::Parse { message, .. }) => {
            assert_eq!(message, "unknown table `[[section.group.group]]`");
        }
        other => panic!("expected Parse, got {other:?}"),
    }
}

#[test]
fn rejects_duplicate_path_between_direct_page_and_group_page() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/dup/"

[[section.page]]
title = "Overview"
source = "overview.md"
path = "/dup/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "button.md"
path = "/dup/"
"#;
    match parse_nav(input) {
        Err(NavError::DuplicatePath(path)) => assert_eq!(path, "/dup/"),
        other => panic!("expected DuplicatePath, got {other:?}"),
    }
}

#[test]
fn rejects_duplicate_path_across_two_groups() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/dup/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "button.md"
path = "/dup/"

[[section.group]]
title = "Overlays"

[[section.group.page]]
title = "Dialog"
source = "dialog.md"
path = "/dup/"
"#;
    match parse_nav(input) {
        Err(NavError::DuplicatePath(path)) => assert_eq!(path, "/dup/"),
        other => panic!("expected DuplicatePath, got {other:?}"),
    }
}

#[test]
fn rejects_unsafe_source_in_group_page() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/components/button/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "../secret.md"
path = "/components/button/"
"#;
    match parse_nav(input) {
        Err(NavError::UnsafeSource(source)) => assert_eq!(source, "../secret.md"),
        other => panic!("expected UnsafeSource, got {other:?}"),
    }
}

#[test]
fn rejects_unsafe_page_path_in_group_page() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/../button/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Button"
source = "button.md"
path = "/../button/"
"#;
    assert!(matches!(parse_nav(input), Err(NavError::UnsafePagePath(_))));
}

#[test]
fn rejects_section_group_appearing_before_any_section() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section.group]]
title = "Orphan"

[[section.group.page]]
title = "Button"
source = "button.md"
path = "/button/"
"#;
    assert!(matches!(parse_nav(input), Err(NavError::Parse { .. })));
}

#[test]
fn rejects_section_group_page_appearing_before_any_section_group() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/orphan/"

[[section.group.page]]
title = "Orphan"
source = "orphan.md"
path = "/orphan/"
"#;
    match parse_nav(input) {
        Err(NavError::Parse { message, .. }) => {
            assert_eq!(
                message,
                "[[section.group.page]] appeared before any [[section.group]]"
            );
        }
        other => panic!("expected Parse, got {other:?}"),
    }
}

#[test]
fn rejects_section_with_no_direct_pages_and_no_groups() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Empty"
"#;
    match parse_nav(input) {
        Err(NavError::EmptySection(title)) => assert_eq!(title, "Empty"),
        other => panic!("expected EmptySection, got {other:?}"),
    }
}

/// `Display` 出力が行番号と理由のみを含み、入力全文・絶対パスを含まない
/// ことを確認する（`security.md` の機微情報露出防止方針、設計文書 §10 A09）。
#[test]
fn parse_error_display_contains_no_raw_input_or_absolute_paths() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "Components"
index_path = "/components/forms/"

[[section.group]]
title = "Forms"
weight = "1"
"#;
    let err = parse_nav(input).expect_err("unknown key should fail");
    let message = err.to_string();
    assert!(message.starts_with("nav.toml:"));
    assert!(!message.contains('\n'));
    assert!(!message.contains('/'));
}

// ---- XSS 回帰 ----

/// グループタイトルも `sidebar()` / `header_nav()` の既定エスケープ（REQ-1）
/// を必ず経由する。`crate::nav::tests::sidebar_escapes_title_and_attribute_content`
/// と同型の回帰テスト。
#[test]
fn sidebar_and_header_nav_escape_group_page_titles() {
    let input = r#"
[site]
title = "Docs"
base_path = ""

[[section]]
title = "<script>alert(1)</script>"
index_path = "/button/"

[[section.group]]
title = "Forms"

[[section.group.page]]
title = "Quote\"Title"
source = "button.md"
path = "/button/"
"#;
    let nav = parse_nav(input).expect("valid nav.toml should parse");

    let sidebar_html = render(&sidebar(&nav, "/button/"));
    assert!(!sidebar_html.contains("<script>"));
    assert!(sidebar_html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(sidebar_html.contains("Quote&quot;Title"));

    let header_html = render(&header_nav(&nav, "/button/"));
    assert!(!header_html.contains("<script>"));
    assert!(header_html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(header_html.contains("Quote&quot;Title"));
}
