//! `nav.toml` の `[[menu]]` / `[[menu.item]]`（複数セクションを束ねる
//! メニュー、イシュー #3699）の正常系・異常系・列挙 API を公開 API
//! （`fandhe_frontend_docs_site::nav`）経由で固定する統合テスト。
//!
//! 異常系は `NavError::Parse` の行番号と固定メッセージまで検証する。
//! 後続 #3700 / #3701 / #3703 が依存する並び規則（`header_entries`）と
//! 判定（`menu_for_path`）もここで固定する。

use std::path::{Path, PathBuf};

use fandhe_frontend_docs_site::nav::{parse_nav, validate_sources, HeaderEntry, Nav, NavError};

fn scratch_root() -> PathBuf {
    let root = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
    let _ = std::fs::create_dir_all(&root);
    root
}

/// テスト専用の一時ディレクトリ（外部クレート追加なし、REQ-3）。
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = scratch_root().join(format!(
            "fandhe-frontend-docs-site-nav-menu-{tag}-{}-{unique}",
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

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repo root")
        .to_path_buf()
}

/// 4 セクション（S1 / S2 / S3 / S4）の共通ヘッダ。
const BASE: &str = r#"[site]
title = "Docs"
base_path = ""

[[section]]
title = "S1"
index_path = "/s1/"

[[section.page]]
title = "P"
source = "s1.md"
path = "/s1/"

[[section]]
title = "S2"
index_path = "/s2/"

[[section.group]]
title = "G"

[[section.group.page]]
title = "P"
source = "s2.md"
path = "/s2/"

[[section.group.page]]
title = "Q"
source = "s2q.md"
path = "/s2/q/"

[[section]]
title = "S3"
index_path = "/s3/"

[[section.page]]
title = "P"
source = "s3.md"
path = "/s3/"

[[section]]
title = "S4"
index_path = "/s4/"

[[section.page]]
title = "P"
source = "s4.md"
path = "/s4/"
"#;

const MENU_OK: &str = r#"
[[menu]]
title = "M"
index_path = "/m/"
source = "m.md"

[[menu.item]]
section = "/s3/"
description = "three"

[[menu.item]]
section = "/s2/"
description = "two"
"#;

fn with(menu: &str) -> String {
    format!("{BASE}{menu}")
}

/// 異常系: `Parse { line, message }` の line と message を固定する。
/// `line` は `input` 内で `needle` を含む最初の行（1 始まり）。
#[track_caller]
fn assert_parse_err(input: &str, needle: &str, message: &str) {
    let want_line = input
        .lines()
        .position(|l| l.contains(needle))
        .map(|i| i + 1)
        .unwrap_or_else(|| panic!("needle `{needle}` not found"));
    match parse_nav(input) {
        Err(NavError::Parse { line, message: m }) => {
            assert_eq!(m, message, "message");
            assert_eq!(line, want_line, "line for `{message}`");
        }
        other => panic!("expected Parse error `{message}`, got {other:?}"),
    }
}

/// `assert_parse_err` の「最後に一致する行」版（BASE 側に同形の行がある場合）。
#[track_caller]
fn assert_parse_err_last(input: &str, needle: &str, message: &str) {
    let lines: Vec<_> = input.lines().collect();
    let want_line = lines
        .iter()
        .rposition(|l| l.contains(needle))
        .map(|i| i + 1)
        .unwrap_or_else(|| panic!("needle `{needle}` not found"));
    match parse_nav(input) {
        Err(NavError::Parse { line, message: m }) => {
            assert_eq!(m, message, "message");
            assert_eq!(line, want_line, "line for `{message}`");
        }
        other => panic!("expected Parse error `{message}`, got {other:?}"),
    }
}

fn entry_titles(nav: &Nav) -> Vec<String> {
    nav.header_entries()
        .into_iter()
        .map(|e| match e {
            HeaderEntry::Section(s) => s.title.clone(),
            HeaderEntry::Menu(m) => m.title.clone(),
        })
        .collect()
}

// ---- 正常系 ----

#[test]
fn parses_menu_with_declaration_order_preserved() {
    let nav = parse_nav(&with(MENU_OK)).expect("parse");
    assert_eq!(nav.menus.len(), 1);
    let m = &nav.menus[0];
    assert_eq!(m.title, "M");
    assert_eq!(m.index_path, "/m/");
    assert_eq!(m.source, "m.md");
    let order: Vec<_> = m.items.iter().map(|i| i.section.as_str()).collect();
    assert_eq!(order, ["/s3/", "/s2/"]);
    // メニューはページではない（ページ数契約を変えない）。
    assert_eq!(nav.all_pages().count(), 5);
}

#[test]
fn header_entries_places_menu_at_earliest_member_position() {
    let nav = parse_nav(&with(MENU_OK)).expect("parse");
    // メンバーは S3, S2（宣言順は S2 < S3）。M は S2 の位置を占める。
    assert_eq!(entry_titles(&nav), ["S1", "M", "S4"]);
}

#[test]
fn header_entries_with_two_menus() {
    let menus = r#"
[[menu]]
title = "A"
index_path = "/a/"
source = "a.md"

[[menu.item]]
section = "/s1/"
description = "d"

[[menu.item]]
section = "/s3/"
description = "d"

[[menu]]
title = "B"
index_path = "/b/"
source = "b.md"

[[menu.item]]
section = "/s2/"
description = "d"

[[menu.item]]
section = "/s4/"
description = "d"
"#;
    let nav = parse_nav(&with(menus)).expect("parse");
    assert_eq!(entry_titles(&nav), ["A", "B"]);
}

#[test]
fn header_entries_without_menu_equals_sections() {
    let nav = parse_nav(BASE).expect("parse");
    assert!(nav.menus.is_empty());
    assert_eq!(entry_titles(&nav), ["S1", "S2", "S3", "S4"]);
    assert!(nav
        .header_entries()
        .iter()
        .all(|e| matches!(e, HeaderEntry::Section(_))));
}

#[test]
fn menu_may_precede_sections() {
    // メニュー宣言の位置は他テーブルに依存しない（検証は全行走査後）。
    let site = "[site]\ntitle = \"Docs\"\nbase_path = \"\"\n";
    let rest = BASE.trim_start_matches(site);
    let input = format!("{site}{MENU_OK}{rest}");
    let nav = parse_nav(&input).expect("parse");
    assert_eq!(entry_titles(&nav), ["S1", "M", "S4"]);
}

#[test]
fn menu_for_path_and_members() {
    let nav = parse_nav(&with(MENU_OK)).expect("parse");
    for p in ["/s2/", "/s2/q/", "/s3/", "/m/"] {
        assert_eq!(
            nav.menu_for_path(p).map(|m| m.title.as_str()),
            Some("M"),
            "{p}"
        );
    }
    for p in ["/s1/", "/s4/", "/unknown/"] {
        assert!(nav.menu_for_path(p).is_none(), "{p}");
    }
    let menu = &nav.menus[0];
    let members: Vec<_> = nav
        .menu_members(menu)
        .map(|(s, i)| (s.title.as_str(), i.description.as_str()))
        .collect();
    assert_eq!(members, [("S3", "three"), ("S2", "two")]);
}

#[test]
fn validate_sources_checks_menu_source() {
    let nav = parse_nav(&with(MENU_OK)).expect("parse");
    let dir = TempDir::new("sources");
    for f in ["s1.md", "s2.md", "s2q.md", "s3.md", "s4.md"] {
        std::fs::write(dir.0.join(f), "# x\n").expect("write");
    }
    assert_eq!(
        validate_sources(&nav, &dir.0),
        Err(NavError::MissingSource("m.md".to_string()))
    );
    std::fs::write(dir.0.join("m.md"), "# m\n").expect("write");
    assert_eq!(validate_sources(&nav, &dir.0), Ok(()));
}

// ---- 異常系 ----

#[test]
fn rejects_missing_menu_keys() {
    for (key, drop) in [
        ("title", "title = \"M\"\n"),
        ("index_path", "index_path = \"/m/\"\n"),
        ("source", "source = \"m.md\"\n"),
    ] {
        let input = with(&MENU_OK.replacen(drop, "", 1));
        assert_parse_err(
            &input,
            "[[menu]]",
            &format!("menu is missing required key `{key}`"),
        );
    }
}

#[test]
fn rejects_missing_item_keys() {
    let input = with(&MENU_OK.replacen("description = \"three\"\n", "", 1));
    assert_parse_err(
        &input,
        "[[menu.item]]",
        "menu item is missing required key `description`",
    );
    let input = with(&MENU_OK.replacen("section = \"/s3/\"\n", "", 1));
    assert_parse_err(
        &input,
        "[[menu.item]]",
        "menu item is missing required key `section`",
    );
}

#[test]
fn rejects_empty_title_and_description() {
    for t in ["", "   "] {
        let input = with(&MENU_OK.replacen("title = \"M\"", &format!("title = \"{t}\""), 1));
        assert_parse_err_last(&input, "title = \"", "menu title must not be empty");
    }
    // BASE 内の title 行にも "title = \"" があるため MENU 部の行を直接数える。
    let input = with(&MENU_OK.replacen("description = \"three\"", "description = \" \"", 1));
    assert_parse_err(
        &input,
        "description = \" \"",
        "menu item description must not be empty",
    );
    let input = with(&MENU_OK.replacen("description = \"three\"", "description = \"a\\nb\"", 1));
    assert_parse_err(
        &input,
        "description = \"a",
        "menu item description must be a single line",
    );
}

#[test]
fn rejects_menu_without_items() {
    let input = with("\n[[menu]]\ntitle = \"M\"\nindex_path = \"/m/\"\nsource = \"m.md\"\n");
    assert_parse_err(&input, "[[menu]]", "menu `M` has no items");
}

#[test]
fn rejects_bad_index_path() {
    let input = with(&MENU_OK.replacen("/m/", "m", 1));
    assert_parse_err(
        &input,
        "index_path = \"m\"",
        "menu index_path `m` is not a safe page path",
    );
}

#[test]
fn rejects_index_path_colliding_with_page_or_section_index() {
    for p in ["/s1/", "/s2/q/"] {
        let input = with(&MENU_OK.replacen("/m/", p, 1));
        // 同値の行が BASE 側にもあるため、メニュー側（最後）の行を期待する。
        let want = input
            .lines()
            .collect::<Vec<_>>()
            .iter()
            .rposition(|l| l.contains(&format!("index_path = \"{p}\"")))
            .map(|i| i + 1)
            .expect("line");
        match parse_nav(&input) {
            Err(NavError::Parse { line, message }) => {
                assert_eq!(
                    message,
                    format!("menu index_path `{p}` collides with an existing page.path")
                );
                assert_eq!(line, want);
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}

#[test]
fn rejects_duplicate_menu_index_path() {
    let second = r#"
[[menu]]
title = "N"
index_path = "/m/"
source = "n.md"

[[menu.item]]
section = "/s1/"
description = "d"
"#;
    let input = with(&format!("{MENU_OK}{second}"));
    let lines: Vec<_> = input.lines().collect();
    let want = lines
        .iter()
        .rposition(|l| l.contains("index_path = \"/m/\""))
        .map(|i| i + 1)
        .expect("line");
    match parse_nav(&input) {
        Err(NavError::Parse { line, message }) => {
            assert_eq!(message, "duplicate menu index_path `/m/`");
            assert_eq!(line, want);
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn rejects_unsafe_source() {
    for s in ["../x.md", "/abs.md", "a\\\\b.md"] {
        let input = with(&MENU_OK.replacen("m.md", s, 1));
        let needle = format!("source = \"{s}\"");
        assert_parse_err(
            &input,
            &needle,
            &format!(
                "menu source `{}` is not a safe relative path",
                s.replace("\\\\", "\\")
            ),
        );
    }
}

#[test]
fn rejects_unknown_item_section() {
    let input = with(&MENU_OK.replacen("/s3/", "/nope/", 1));
    assert_parse_err(
        &input,
        "section = \"/nope/\"",
        "menu item section `/nope/` does not match any [[section]] index_path",
    );
}

#[test]
fn rejects_duplicate_member_in_menu() {
    let input = with(&MENU_OK.replacen("section = \"/s2/\"", "section = \"/s3/\"", 1));
    let want = input
        .lines()
        .enumerate()
        .filter(|(_, l)| l.contains("section = \"/s3/\""))
        .map(|(i, _)| i + 1)
        .last()
        .expect("line");
    match parse_nav(&input) {
        Err(NavError::Parse { line, message }) => {
            assert_eq!(
                message,
                "section `/s3/` is listed more than once in menu `M`"
            );
            assert_eq!(line, want);
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn rejects_section_in_two_menus() {
    let second = r#"
[[menu]]
title = "N"
index_path = "/n/"
source = "n.md"

[[menu.item]]
section = "/s3/"
description = "d"
"#;
    let input = with(&format!("{MENU_OK}{second}"));
    let want = input
        .lines()
        .enumerate()
        .filter(|(_, l)| l.contains("section = \"/s3/\""))
        .map(|(i, _)| i + 1)
        .last()
        .expect("line");
    match parse_nav(&input) {
        Err(NavError::Parse { line, message }) => {
            assert_eq!(message, "section `/s3/` already belongs to menu `M`");
            assert_eq!(line, want);
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn rejects_unknown_keys_and_tables() {
    let input = with(&MENU_OK.replacen("source = \"m.md\"", "source = \"m.md\"\nextra = \"x\"", 1));
    assert_parse_err(&input, "extra", "unknown key `extra` in [[menu]]");
    let input = with(&MENU_OK.replacen(
        "description = \"two\"",
        "description = \"two\"\nextra = \"x\"",
        1,
    ));
    assert_parse_err(&input, "extra", "unknown key `extra` in [[menu.item]]");
    let input = with("\n[[menu]]\ntitle = \"M\"\n[[menu.group]]\n");
    assert_parse_err(&input, "[[menu.group]]", "unknown table `[[menu.group]]`");
}

#[test]
fn rejects_menu_item_before_menu() {
    let input = format!("[[menu.item]]\nsection = \"/s1/\"\ndescription = \"d\"\n{BASE}");
    assert_parse_err(
        &input,
        "[[menu.item]]",
        "[[menu.item]] appeared before any [[menu]]",
    );
}

#[test]
fn rejects_detached_menu_item_after_section() {
    // `[[menu]]` と `[[menu.item]]` の間に `[[section]]` を挟んだ離れた項目は
    // 以前のメニューへ吸着させず Parse エラーにする。
    let input = with(&format!(
        "{MENU_OK}\n[[section]]\ntitle = \"S3\"\nindex_path = \"/s3/\"\n[[menu.item]]\nsection = \"/s1/\"\ndescription = \"d\"\n"
    ));
    assert_parse_err_last(
        &input,
        "[[menu.item]]",
        "[[menu.item]] must directly follow a [[menu]] or another [[menu.item]]",
    );
}

#[test]
fn rejects_section_page_directly_after_menu() {
    let input = with(&format!(
        "{MENU_OK}\n[[section.page]]\ntitle = \"X\"\nsource = \"x.md\"\npath = \"/x/\"\n"
    ));
    assert_parse_err_last(
        &input,
        "[[section.page]]",
        "[[section.page]] must follow a [[section]] (not directly after [[menu]])",
    );
}

// ---- 実 site/nav.toml ----

#[test]
fn real_nav_declares_assets_menu() {
    let root = repo_root();
    let text = std::fs::read_to_string(root.join("site/nav.toml")).expect("read nav.toml");
    let nav = parse_nav(&text).expect("parse nav.toml");
    validate_sources(&nav, &root).expect("sources exist");

    assert_eq!(
        entry_titles(&nav),
        ["Getting Started", "Guides", "Assets", "API Reference"]
    );
    let menu = nav
        .menus
        .iter()
        .find(|m| m.title == "Assets")
        .expect("Assets");
    assert_eq!(menu.index_path, "/assets/");
    let members: Vec<_> = nav.menu_members(menu).collect();
    let titles: Vec<_> = members.iter().map(|(s, _)| s.title.as_str()).collect();
    assert_eq!(
        titles,
        ["Primitives", "Themes", "Blocks", "Wireframes", "Examples"]
    );
    assert!(members
        .iter()
        .all(|(_, i)| !i.description.trim().is_empty()));

    for p in ["/themes/", "/primitives/", "/examples/", "/assets/"] {
        assert_eq!(
            nav.menu_for_path(p).map(|m| m.title.as_str()),
            Some("Assets"),
            "{p}"
        );
    }
    let some_theme_page = nav
        .sections
        .iter()
        .find(|s| s.index_path == "/themes/")
        .and_then(|s| s.all_pages().nth(1))
        .expect("themes page");
    assert!(nav.menu_for_path(&some_theme_page.path).is_some());
    for p in ["/guides/", "/", "/api/"] {
        assert!(nav.menu_for_path(p).is_none(), "{p}");
    }
}
