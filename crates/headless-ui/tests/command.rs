//! Command（イシュー #2068）の統合テスト。
//!
//! `crates/headless-ui/src/command.rs` の inline unit tests がパーツ単体の
//! 属性出力を固定するのに対し、本ファイルは
//! `dialog(root(input, list(group(group_heading, item(shortcut)),
//! separator), empty))` という全体の組み立てにおける data-*/ARIA 対応・
//! dispatch 統合・SSR/hydration 両経路を、クレート外部から（公開 API の
//! みを使って）固定する（`crates/headless-ui/tests/combobox.rs` と同じ
//! 粒度）。
//!
//! R1〜R4（`crates/docs-site/tests/combobox_aria_association.rs` と同一
//! 規則の意図的な重複実装、同ファイル冒頭コメント参照）を本ファイル内へも
//! 複製し、docs-site 側の契約を先取りして検証する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::command::{self, Command};
use fandhe_frontend_headless_ui::OpenState;
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

/// `dialog > root(empty=false) > input + list(group(group-heading, item ×2
/// + shortcut), separator) + empty` の全体組み立て。1 件目の item を選択済み
/// （`activedescendant` の参照先）として組み立てる。
fn full_assembly() -> fandhe_frontend_core::Node {
    let input = command::input(
        OpenState::Open,
        "cal",
        "command-list-1",
        Some("command-item-calendar"),
        vec![("id", "command-input-1")],
    );

    let item_calendar = command::item(
        true,
        false,
        "calendar",
        Some("command-item-calendar"),
        vec![],
        vec![
            text("Calendar"),
            command::shortcut(vec![], vec![text("⌘C")]),
        ],
    );
    let item_search = command::item(
        false,
        false,
        "search",
        Some("command-item-search"),
        vec![],
        vec![text("Search Emoji")],
    );
    let group_heading = command::group_heading(
        Some("command-group-heading-1"),
        vec![],
        vec![text("Suggestions")],
    );
    let group = command::group(
        Some("command-group-heading-1"),
        vec![],
        vec![group_heading, item_calendar, item_search],
    );
    let separator = command::separator(vec![], vec![]);

    let list = command::list(
        "command-list-1",
        "Suggestions",
        false,
        vec![],
        vec![group, separator],
    );

    let empty = command::empty(false, vec![], vec![text("No results found.")]);

    let root = command::root(OpenState::Open, false, vec![], vec![input, list, empty]);

    command::dialog(OpenState::Open, "Command Menu", vec![], vec![root])
}

#[test]
fn full_assembly_wires_aria_controls_labelledby_and_all_parts_appear() {
    let html = render(&full_assembly());

    // 10 anatomy パーツすべてが出現する。
    for part in [
        "root",
        "input",
        "list",
        "empty",
        "group",
        "group-heading",
        "item",
        "shortcut",
        "separator",
        "dialog",
    ] {
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "missing part {part}: {html}"
        );
    }

    // ARIA 関連付け。
    assert!(html.contains(r#"aria-controls="command-list-1""#));
    assert!(html.contains(r#"aria-activedescendant="command-item-calendar""#));
    assert!(html.contains(r#"id="command-list-1""#));
    assert!(html.contains(r#"aria-label="Suggestions""#));
    assert!(html.contains(r#"aria-labelledby="command-group-heading-1""#));
    assert!(html.contains(r#"id="command-group-heading-1""#));
    assert!(html.contains(r#"id="command-item-calendar""#));
    assert!(html.contains(r#"data-value="calendar""#));
    assert!(html.contains(r#"role="dialog""#));
    assert!(html.contains(r#"aria-label="Command Menu""#));
}

// --- R1〜R4 + listbox 命名契約（`tests/combobox_aria_association.rs` と
// 同一規則の意図的な重複実装） ---

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!(" {name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];

fn is_void_tag(tag: &str) -> bool {
    let name = tag.split(|c: char| c.is_whitespace()).next().unwrap_or("");
    VOID_TAGS.contains(&name)
}

fn open_tags(html: &str) -> Vec<&str> {
    let mut tags = Vec::new();
    let mut rest = html;
    while let Some(lt) = rest.find('<') {
        let after_lt = &rest[lt + 1..];
        if after_lt.starts_with('/') {
            match after_lt.find('>') {
                Some(gt) => {
                    rest = &after_lt[gt + 1..];
                    continue;
                }
                None => break,
            }
        }
        match after_lt.find('>') {
            Some(gt) => {
                let tag = &after_lt[..gt];
                tags.push(tag);
                let _ = is_void_tag(tag);
                rest = &after_lt[gt + 1..];
            }
            None => break,
        }
    }
    tags
}

/// combobox の ARIA 関連付け規則 R1〜R4
/// （`crates/docs-site/tests/combobox_aria_association.rs` と同一規則）。
fn verify_combobox_aria_association(html: &str) -> Result<(), String> {
    let tags = open_tags(html);

    let mut id_role: std::collections::HashMap<&str, Option<&str>> =
        std::collections::HashMap::new();
    for tag in &tags {
        if let Some(id) = attr(tag, "id") {
            id_role.insert(id, attr(tag, "role"));
        }
    }

    for tag in &tags {
        if attr(tag, "role") != Some("combobox") {
            continue;
        }
        let expanded = attr(tag, "aria-expanded") == Some("true");
        let controls = attr(tag, "aria-controls");
        let activedescendant = attr(tag, "aria-activedescendant");

        if expanded && controls.is_none() {
            return Err(format!(
                "R1 violation: role=\"combobox\" element with aria-expanded=\"true\" lacks aria-controls: <{tag}>"
            ));
        }

        if let Some(target) = controls {
            match id_role.get(target) {
                Some(Some("listbox")) => {}
                Some(_) => {
                    return Err(format!(
                        "R2 violation: aria-controls=\"{target}\" target lacks role=\"listbox\""
                    ));
                }
                None => {
                    return Err(format!(
                        "R2 violation: aria-controls=\"{target}\" target id does not exist (dangling IDREF)"
                    ));
                }
            }
        }

        if let Some(target) = activedescendant {
            if !id_role.contains_key(target) {
                return Err(format!(
                    "R4 violation: aria-activedescendant=\"{target}\" target id does not exist (dangling IDREF)"
                ));
            }
        }
    }

    Ok(())
}

/// listbox のアクセシブルネーム経路の契約
/// （`crates/docs-site/tests/combobox_aria_association.rs` と同一規則）。
fn verify_listbox_has_accessible_name(html: &str) -> Result<(), String> {
    let tags = open_tags(html);
    let ids: std::collections::HashSet<&str> =
        tags.iter().filter_map(|tag| attr(tag, "id")).collect();

    for tag in &tags {
        if attr(tag, "role") != Some("listbox") {
            continue;
        }
        let labelledby = attr(tag, "aria-labelledby");
        let label = attr(tag, "aria-label");
        match (labelledby, label) {
            (Some(target), _) if ids.contains(target) => {}
            (_, Some(_)) => {}
            (Some(target), None) => {
                return Err(format!(
                    "naming violation: aria-labelledby=\"{target}\" target id does not exist (dangling IDREF) and no aria-label fallback: <{tag}>"
                ));
            }
            (None, None) => {
                return Err(format!(
                    "naming violation: role=\"listbox\" element has neither aria-labelledby nor aria-label: <{tag}>"
                ));
            }
        }
    }
    Ok(())
}

#[test]
fn verify_command_aria_association_ok_for_compliant_full_assembly() {
    let html = render(&full_assembly());
    assert_eq!(verify_combobox_aria_association(&html), Ok(()));
    assert_eq!(verify_listbox_has_accessible_name(&html), Ok(()));
}

#[test]
fn verify_command_aria_association_ok_when_input_and_list_share_id() {
    // `aria-controls` を必須引数にしているため、通常経路では R1 違反を
    // 作れない（型で強制されている裏付け）。`input`/`list` を同じ `id` で
    // ペアにして R2 が満たされることを固定する。
    let input = command::input(OpenState::Open, "", "list-x", None, vec![]);
    let list = command::list("list-x", "Suggestions", false, vec![], vec![]);
    let html = render(&fandhe_frontend_core::el("div", vec![], vec![input, list]));
    assert!(html.contains(r#"aria-controls="list-x""#));
    assert_eq!(verify_combobox_aria_association(&html), Ok(()));
}

#[test]
fn verify_command_aria_association_detects_dangling_controls_without_matching_list() {
    // list を伴わずに input 単体を描画すると R2（dangling IDREF）で
    // 違反になることを固定する（`verify_combobox_aria_association` 自体の
    // 検知力の裏付け）。
    let html = render(&command::input(OpenState::Open, "", "list-x", None, vec![]));
    assert!(verify_combobox_aria_association(&html).is_err());
}

// --- dispatch/state machine 統合 ---

#[test]
fn command_dispatch_open_input_select_integration() {
    let mut c = Command::default();
    dispatch(&mut c, "open", "");
    assert!(c.is_open());
    dispatch(&mut c, "input", "cal");
    assert_eq!(c.query(), "cal");
    assert!(c.is_open(), "input は dialog を開閉しない");
    dispatch(&mut c, "select", "calendar");
    assert_eq!(c.selected(), Some("calendar"));
    assert!(c.is_open(), "select は dialog を閉じない");
    dispatch(&mut c, "close", "");
    assert!(!c.is_open());
}

#[test]
fn command_dispatch_unknown_action_is_noop() {
    let mut c = Command::default();
    dispatch(&mut c, "bogus", "payload");
    assert!(!c.is_open());
    assert_eq!(c.query(), "");
    assert_eq!(c.selected(), None);
}

// --- hydration ---

#[test]
fn command_hydration_round_trip_via_public_api() {
    let mut c = Command::default();
    c.update(fandhe_frontend_headless_ui::CommandAction::Open);
    c.update(fandhe_frontend_headless_ui::CommandAction::Input(
        "cal".to_string(),
    ));
    c.update(fandhe_frontend_headless_ui::CommandAction::Select(
        "calendar".to_string(),
    ));

    let rendered = render(&render_for_hydration(&c));
    assert!(rendered.contains("data-hydrate-state"));
    assert!(rendered.contains("data-hydrate-input"));
    assert!(rendered.contains("data-hydrate-selected"));

    let restored = Command::from_hydration_attrs(&c.hydration_attrs()).unwrap();
    assert_eq!(c, restored);
}

#[test]
fn command_from_hydration_attrs_invalid_value_does_not_panic() {
    let attrs = vec![("data-hydrate-state".to_string(), "bogus".to_string())];
    let err = Command::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

// --- data-empty の SSR 決定性 ---

#[test]
fn command_data_empty_is_deterministic_for_same_query() {
    let items = [("calendar", "Calendar"), ("search", "Search Emoji")];
    let mut c = Command::default();
    c.update(fandhe_frontend_headless_ui::CommandAction::Input(
        "zzz".to_string(),
    ));
    let empty1 = c.is_empty(&items);
    let empty2 = c.is_empty(&items);
    assert_eq!(empty1, empty2);
    assert!(empty1);

    let html1 = render(&c.root(empty1, vec![], vec![]));
    let html2 = render(&c.root(empty2, vec![], vec![]));
    assert_eq!(html1, html2);
    assert!(html1.contains("data-empty"));
}
