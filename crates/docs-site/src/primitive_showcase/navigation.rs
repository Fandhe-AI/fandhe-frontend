//! Primitives Demo — Navigation（11 件、原稿は #1028）。
//! 執筆規約は `crate::primitive_showcase` モジュール doc 参照。
//!
//! `menu`/`menubar` は項目数が多いため、`trigger_item`/`context_trigger`/
//! `checkbox_item`/`radio_item_group`/`radio_item`/menubar の `sub_trigger`/
//! `sub_content` は本デモでは未網羅とし、
//! `tests/primitive_showcase.rs::KNOWN_UNCOVERED` に登録する。

use fandhe_frontend_core::{text, Node};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui as hui;
use hui::action_bar;
use hui::breadcrumb::{self, BreadcrumbItem};
use hui::data_attrs::Orientation;
use hui::link;
use hui::link_overlay;
use hui::menu;
use hui::menubar;
use hui::nav_list;
use hui::navigation_menu;
use hui::pagination::{self, ItemMode};
use hui::tabs::{tabs, ActivationMode, TabItem, TabsProps};
use hui::toolbar;
use hui::OpenState;

use super::demo_page;

pub(super) fn action_bar_section() -> Node {
    let state = OpenState::Open;
    let body = vec![action_bar::root(
        state,
        vec![],
        vec![action_bar::positioner(
            state,
            vec![],
            vec![action_bar::content(
                state,
                "Selection actions",
                vec![],
                vec![
                    action_bar::selection_trigger(vec![], vec![text("3 selected")]),
                    action_bar::separator(vec![], vec![]),
                    action_bar::close_trigger(vec![], vec![text("Cancel")]),
                ],
            )],
        )],
    )];
    demo_page("Action Bar", body)
}

pub(super) fn breadcrumb_section() -> Node {
    let items = [
        BreadcrumbItem {
            label: "Home",
            href: "https://example.com/",
        },
        BreadcrumbItem {
            label: "Docs",
            href: "https://example.com/docs/",
        },
        BreadcrumbItem {
            label: "Primitives",
            href: "https://example.com/primitives/",
        },
    ];
    let body = vec![breadcrumb::breadcrumb(None, &items, || vec![text("/")])];
    demo_page("Breadcrumb", body)
}

pub(super) fn link_section() -> Node {
    let body = vec![link::root(
        "https://example.com",
        true,
        false,
        vec![],
        vec![text("External link")],
    )];
    demo_page("Link", body)
}

pub(super) fn link_overlay_section() -> Node {
    let body = vec![link_overlay::root(
        vec![],
        vec![
            text("Card body content"),
            link_overlay::overlay(
                "https://example.com/article",
                vec![],
                vec![text("Read article")],
            ),
        ],
    )];
    demo_page("Link Overlay", body)
}

pub(super) fn menu_section() -> Node {
    let state = OpenState::Open;
    let body = vec![menu::root(
        state,
        vec![],
        vec![
            menu::trigger(
                state,
                false,
                Some("menu-content"),
                vec![],
                vec![
                    text("Actions"),
                    menu::indicator(state, vec![], vec![text("▾")]),
                ],
            ),
            menu::positioner(
                state,
                vec![],
                vec![menu::content(
                    state,
                    Some("menu-content"),
                    None,
                    vec![],
                    vec![
                        menu::arrow(vec![], vec![menu::arrow_tip(vec![], vec![])]),
                        menu::item_group(
                            Some("menu-group-label"),
                            vec![],
                            vec![
                                menu::item_group_label(
                                    Some("menu-group-label"),
                                    vec![],
                                    vec![text("Edit")],
                                ),
                                menu::item("rename", false, true, vec![], vec![text("Rename")]),
                                menu::item("delete", false, false, vec![], vec![text("Delete")]),
                            ],
                        ),
                        menu::separator(vec![], vec![]),
                    ],
                )],
            ),
        ],
    )];
    demo_page("Menu", body)
}

pub(super) fn menubar_section() -> Node {
    let orientation = Orientation::Horizontal;
    let state = OpenState::Open;
    let body = vec![menubar::root(
        orientation,
        "Main menu",
        vec![],
        vec![menubar::menu(
            state,
            vec![],
            vec![
                menubar::trigger(
                    true,
                    state,
                    false,
                    false,
                    0,
                    Some("mb-content"),
                    vec![],
                    vec![text("File")],
                ),
                menubar::positioner(
                    state,
                    vec![],
                    vec![menubar::content(
                        state,
                        Some("mb-content"),
                        None,
                        vec![],
                        vec![
                            menubar::item_group(
                                None,
                                vec![],
                                vec![
                                    menubar::item_group_label(None, vec![], vec![text("File")]),
                                    menubar::item("new", false, false, vec![], vec![text("New")]),
                                ],
                            ),
                            menubar::separator(vec![], vec![]),
                        ],
                    )],
                ),
            ],
        )],
    )];
    demo_page("Menubar", body)
}

pub(super) fn nav_list_section() -> Node {
    let body = vec![nav_list::root(
        "Sidebar",
        vec![],
        vec![
            nav_list::heading(vec![], vec![text("Guides")]),
            nav_list::list(
                vec![],
                vec![
                    nav_list::item(
                        vec![],
                        vec![nav_list::link(
                            "https://example.com/guides/getting-started/",
                            true,
                            vec![],
                            vec![text("Getting Started")],
                        )],
                    ),
                    nav_list::item(
                        vec![],
                        vec![nav_list::link(
                            "https://example.com/guides/deployment/",
                            false,
                            vec![],
                            vec![text("Deployment")],
                        )],
                    ),
                ],
            ),
        ],
    )];
    demo_page("Nav List", body)
}

pub(super) fn navigation_menu_section() -> Node {
    let open = OpenState::Open;
    let closed = OpenState::Closed;
    let body = vec![navigation_menu::root(
        "Main",
        vec![],
        vec![navigation_menu::list(
            vec![],
            vec![
                navigation_menu::item(
                    open,
                    false,
                    vec![],
                    vec![
                        navigation_menu::trigger(
                            open,
                            false,
                            "products",
                            Some("nm-trigger-0"),
                            Some("nm-content-0"),
                            vec![],
                            vec![text("Products")],
                        ),
                        navigation_menu::content(
                            open,
                            Some("nm-content-0"),
                            Some("nm-trigger-0"),
                            vec![],
                            vec![navigation_menu::link(
                                "https://example.com/products/core/",
                                false,
                                vec![],
                                vec![text("Core")],
                            )],
                        ),
                    ],
                ),
                navigation_menu::item(
                    closed,
                    false,
                    vec![],
                    vec![navigation_menu::trigger(
                        closed,
                        false,
                        "docs",
                        None,
                        None,
                        vec![],
                        vec![text("Docs")],
                    )],
                ),
            ],
        )],
    )];
    demo_page("Navigation Menu", body)
}

pub(super) fn pagination_section() -> Node {
    let body = vec![pagination::root(
        "Pagination",
        vec![],
        vec![
            pagination::prev_trigger(ItemMode::Button, false, vec![], vec![text("Prev")]),
            pagination::item(ItemMode::Button, false, false, vec![], vec![text("1")]),
            pagination::item(ItemMode::Button, true, false, vec![], vec![text("2")]),
            pagination::ellipsis(vec![], vec![text("…")]),
            pagination::item(
                ItemMode::Link {
                    href: "https://example.com/?page=9",
                },
                false,
                false,
                vec![],
                vec![text("9")],
            ),
            pagination::next_trigger(ItemMode::Button, false, vec![], vec![text("Next")]),
        ],
    )];
    demo_page("Pagination", body)
}

pub(super) fn tabs_section() -> Node {
    let props = TabsProps {
        id: "primitives-tabs",
        selected: "overview",
        orientation: Orientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: true,
    };
    let items = vec![
        TabItem {
            value: "overview",
            trigger: vec![text("Overview")],
            content: vec![text("Overview panel content.")],
            disabled: false,
        },
        TabItem {
            value: "settings",
            trigger: vec![text("Settings")],
            content: vec![text("Settings panel content.")],
            disabled: false,
        },
    ];
    let body = vec![tabs(&props, items)];
    demo_page("Tabs", body)
}

pub(super) fn toolbar_section() -> Node {
    let orientation = Orientation::Horizontal;
    let body = vec![toolbar::root(
        orientation,
        "Formatting",
        vec![],
        vec![
            toolbar::button(true, false, vec![], vec![text("Bold")]),
            toolbar::link(
                false,
                "https://example.com/help",
                false,
                vec![],
                vec![text("Help")],
            ),
            toolbar::separator(orientation, vec![], vec![]),
            toolbar::toggle_group(
                vec![],
                vec![toolbar::toggle_item(
                    true,
                    false,
                    false,
                    "align-left",
                    vec![],
                    vec![text("Left")],
                )],
            ),
        ],
    )];
    demo_page("Toolbar", body)
}
