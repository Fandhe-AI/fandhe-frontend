//! `fandhe-frontend-headless-ui` の Sidebar（[`sidebar`] モジュール、イシュー
//! #2072）の公開 API 経由の統合テスト。
//!
//! `crates/headless-ui/src/sidebar.rs` 内の `#[cfg(test)]` ユニットテストが
//! パート単位の網羅を担うのに対し、本ファイルは公開 API
//! （`lib.rs` の再エクスポート `pub mod sidebar;`）のみを経由し、22 パート
//! 全てを合成した実利用形態（`fandhe-frontend-pre-styled-ui` が想定する
//! 構成、#2073）を固定する回帰テスト（`tests/item.rs`/`tests/command.rs` と
//! 同型の位置付け）。

use fandhe_frontend_core::{el, render, text};
use fandhe_frontend_headless_ui::collapsible;
use fandhe_frontend_headless_ui::sidebar::{
    self, Sidebar, SidebarMenuButtonProps, SidebarMenuSubButtonProps, SidebarProps, SidebarState,
};
use fandhe_frontend_headless_ui::state::OpenState;

/// 全 22 パートを合成し、`nav` が文書中に 1 個のみ・`inset` が `div`
/// （`main` を出さない）・`role="group"` の `aria-labelledby` 対応・
/// `menu-sub` の開閉が `collapsible` の合成で表現できることを固定する。
#[test]
fn all_twenty_two_parts_compose_into_a_single_nav_landmark() {
    let sidebar_state = Sidebar::new(SidebarState::Expanded);
    let props = SidebarProps::default();

    let sub_disclosure = OpenState::Open;
    let menu_sub = collapsible::content(
        sub_disclosure,
        false,
        Some("projects-sub"),
        vec![],
        vec![sidebar::menu_sub(
            vec![],
            vec![sidebar::menu_sub_item(
                vec![],
                vec![sidebar::menu_sub_button(
                    &SidebarMenuSubButtonProps {
                        href: Some("/projects/alpha"),
                        active: true,
                        ..Default::default()
                    },
                    vec![],
                    vec![text("Alpha")],
                )],
            )],
        )],
    );
    let projects_item = el(
        "li",
        vec![],
        vec![
            collapsible::trigger(
                sub_disclosure,
                false,
                Some("projects-sub"),
                vec![],
                vec![text("Projects")],
            ),
            menu_sub,
        ],
    );

    let menu = sidebar::menu(
        vec![],
        vec![
            sidebar::menu_item(
                vec![],
                vec![sidebar::menu_button(
                    &SidebarMenuButtonProps {
                        href: Some("/dashboard"),
                        active: true,
                        describedby: Some("dashboard-tip"),
                        ..Default::default()
                    },
                    vec![],
                    vec![
                        text("Dashboard"),
                        sidebar::menu_action("Pin", vec![], vec![]),
                        sidebar::menu_badge(vec![], vec![text("3")]),
                    ],
                )],
            ),
            projects_item,
        ],
    );

    let group = sidebar::group(
        Some("platform-group-label"),
        vec![],
        vec![
            sidebar::group_label(Some("platform-group-label"), vec![], vec![text("Platform")]),
            sidebar::group_action("Add", vec![], vec![]),
            sidebar::group_content(vec![], vec![menu]),
        ],
    );

    let root = sidebar::root(
        &sidebar_state,
        &props,
        "Main navigation",
        Some("app-sidebar"),
        vec![],
        vec![
            sidebar::header(vec![], vec![text("Acme Inc")]),
            sidebar::content(
                vec![],
                vec![
                    sidebar::input(vec![("type", "search"), ("aria-label", "Search")]),
                    sidebar::separator(vec![], vec![]),
                    group,
                ],
            ),
            sidebar::footer(vec![], vec![text("Ada Lovelace")]),
        ],
    );

    let provider = sidebar::provider(
        &sidebar_state,
        &props,
        vec![],
        vec![
            root,
            sidebar::rail(&sidebar_state, "Toggle sidebar rail", vec![], vec![]),
            sidebar::trigger(
                &sidebar_state,
                "Toggle sidebar",
                Some("app-sidebar"),
                vec![],
                vec![],
            ),
        ],
    );

    let page = el(
        "div",
        vec![],
        vec![provider, sidebar::inset(vec![], vec![text("Page content")])],
    );

    let html = render(&page);

    // 全パートが出現すること。
    for part in [
        "provider",
        "root",
        "header",
        "content",
        "footer",
        "separator",
        "input",
        "group",
        "group-label",
        "group-content",
        "group-action",
        "menu",
        "menu-item",
        "menu-button",
        "menu-action",
        "menu-badge",
        "menu-sub",
        "menu-sub-item",
        "menu-sub-button",
        "rail",
        "trigger",
        "inset",
    ] {
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "missing part {part} in {html}"
        );
    }

    // `nav` ランドマークは文書中に 1 個のみ。
    assert_eq!(html.matches("<nav").count(), 1);

    // `inset` は `div`（`main` を出さない）。
    assert!(!html.contains("<main"));

    // `role="group"` は `aria-labelledby` と対で出力される。
    assert!(html.contains(r#"role="group""#));
    assert!(html.contains(r#"aria-labelledby="platform-group-label""#));

    // `menu-sub` の開閉は `collapsible` の合成で表現され、open のとき
    // `hidden` は付かない。
    assert!(html.contains(r#"aria-controls="projects-sub""#));
    assert!(!html.contains("hidden"));

    // sidebar 自身は `data-scope="collapsible"` を内包しない出力の対象外
    // パーツ（menu-sub 等）に collapsible のロールを混入させない。
    assert!(html.contains(r#"data-scope="collapsible""#));
    assert!(html.contains(r#"data-scope="sidebar""#));
}

/// 状態機械を持たないパート（`header`/`content`/`footer` 等）は
/// `data-hydrate-*` を一切出力しない。
#[test]
fn stateless_parts_have_no_hydrate_attrs() {
    let html = render(&sidebar::header(vec![], vec![]));
    assert!(!html.contains("data-hydrate-"));
}

#[test]
fn menu_sub_closed_via_collapsible_hides_content() {
    let closed = collapsible::content(
        OpenState::Closed,
        false,
        None,
        vec![],
        vec![sidebar::menu_sub(vec![], vec![])],
    );
    let html = render(&closed);
    assert!(html.contains("hidden"));
}
