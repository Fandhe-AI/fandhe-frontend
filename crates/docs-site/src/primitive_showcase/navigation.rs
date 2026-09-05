//! Primitives Demo — Navigation（11 件、原稿は #1028）。
//! 執筆規約は `crate::primitive_showcase` モジュール doc 参照。
//!
//! `menu` はイシュー #1651（参照突合）で 18 anatomy パーツすべてを描画する
//! よう拡充した（`checkbox_item`/`radio_item_group`/`radio_item`/
//! `trigger_item`/`context_trigger`/`item_text`/`item_indicator` を含む）。
//! `menubar` はイシュー #1652（参照突合）で 18 anatomy パーツすべてを
//! 描画するよう拡充した（`arrow`/`arrow_tip`/`item_text`/`item_indicator`/
//! `checkbox_item`/`radio_item_group`/`radio_item`/`sub_trigger`/
//! `sub_content` を含む）。

use fandhe_frontend_core::{div, el, p, text, Node};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui as hui;
use hui::action_bar;
use hui::breadcrumb;
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

/// 7 パーツ全て（`root`/`list`/`item`/`link`/`current-link`/`separator`/
/// `ellipsis`）を Demo 上で機械導出可能にするため、[`breadcrumb::breadcrumb`]
/// 利便ビルダーではなく個別パーツを手組みする（chakra-ui の
/// breadcrumb-with-ellipsis 例と同型、イシュー #1648）。
pub(super) fn breadcrumb_section() -> Node {
    let body = vec![breadcrumb::root(
        None,
        vec![],
        vec![breadcrumb::list(
            vec![],
            vec![
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link(
                        "https://example.com/",
                        vec![],
                        vec![text("Home")],
                    )],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::ellipsis(vec![]),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link(
                        "https://example.com/primitives/",
                        vec![],
                        vec![text("Primitives")],
                    )],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::current_link(vec![], vec![text("Breadcrumb")])],
                ),
            ],
        )],
    )];
    demo_page("Breadcrumb", body)
}

/// 参考サイト（chakra-ui / Radix Themes）のデモ構成（単体 / variant 別 /
/// 文中インライン）に合わせ、通常・外部・現在ページ・文中インラインの
/// 4 変種を並べる（イシュー #1649）。href はすべて `example.com`（RFC
/// 2606）で内部リンク切れ検証を避ける（`crate::linkcheck` 対象外）。
pub(super) fn link_section() -> Node {
    let body = vec![
        div(
            vec![],
            vec![link::root(
                "https://example.com/docs",
                false,
                false,
                vec![],
                vec![text("Internal link")],
            )],
        ),
        div(
            vec![],
            vec![link::root(
                "https://example.com",
                true,
                false,
                vec![],
                vec![text("External link")],
            )],
        ),
        div(
            vec![],
            vec![link::root(
                "https://example.com/current",
                false,
                true,
                vec![],
                vec![text("Current page link")],
            )],
        ),
        p(
            vec![],
            vec![
                text("Read more in the "),
                link::root(
                    "https://example.com/guide",
                    false,
                    false,
                    vec![],
                    vec![text("inline guide")],
                ),
                text(" for details."),
            ],
        ),
    ];
    demo_page("Link", body)
}

/// 参考サイト（chakra-ui `LinkBox`/`LinkOverlay`。ark-ui の対応ページは
/// 404 で実在せず、Radix Primitives/Radix Themes にも対応部品なし）の
/// 典型構成（タイトル位置に置いた `overlay` + 説明文 + 内側の通常リンク）
/// を再現する（イシュー #1650）。`overlay` はタイトル相当の `strong` に
/// 包んで DOM 上の視覚的な見出し位置を示し、`root` 内の内側リンクは
/// 別 scope（`link::root`）ではなく素の `a`（執筆規約 1「他 scope を
/// 内包しない」）で表す。href はすべて `example.com`（RFC 2606）で内部
/// リンク切れ検証を避ける（`crate::linkcheck` 対象外）。
pub(super) fn link_overlay_section() -> Node {
    let body = vec![link_overlay::root(
        vec![],
        vec![
            p(
                vec![],
                vec![el(
                    "strong",
                    vec![],
                    vec![link_overlay::overlay(
                        "https://example.com/articles/getting-started",
                        vec![],
                        vec![text("Getting started")],
                    )],
                )],
            ),
            p(
                vec![],
                vec![text(
                    "A short summary of the article shown as normal flow content.",
                )],
            ),
            p(
                vec![],
                vec![el(
                    "a",
                    vec![("href", "https://example.com/authors/jane")],
                    vec![text("By Jane")],
                )],
            ),
        ],
    )];
    demo_page("Link Overlay", body)
}

pub(super) fn menu_section() -> Node {
    let open = OpenState::Open;
    let closed = OpenState::Closed;

    // ブロック 1（open、全機能）: root/trigger/indicator/positioner/content/
    // arrow/arrow-tip/item/item-group/item-group-label/separator/
    // checkbox-item/radio-item-group/radio-item/item-text/item-indicator/
    // trigger-item を 1 本のメニューへ集約する。
    let full_menu = menu::root(
        open,
        vec![],
        vec![
            menu::trigger(
                open,
                false,
                Some("menu-content"),
                vec![("id", "menu-trigger")],
                vec![
                    text("Actions"),
                    menu::indicator(open, vec![], vec![text("▾")]),
                ],
            ),
            menu::positioner(
                open,
                vec![],
                vec![menu::content(
                    open,
                    Some("menu-content"),
                    Some("menu-trigger"),
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
                                menu::item("delete", true, false, vec![], vec![text("Delete")]),
                            ],
                        ),
                        menu::separator(vec![], vec![]),
                        menu::checkbox_item(
                            true,
                            "bookmarks",
                            false,
                            false,
                            vec![],
                            vec![
                                menu::item_indicator(true, vec![], vec![text("✓")]),
                                menu::item_text(false, false, vec![], vec![text("Bookmarks")]),
                            ],
                        ),
                        menu::checkbox_item(
                            false,
                            "urls",
                            false,
                            false,
                            vec![],
                            vec![
                                menu::item_indicator(false, vec![], vec![text("✓")]),
                                menu::item_text(false, false, vec![], vec![text("Full URLs")]),
                            ],
                        ),
                        menu::separator(vec![], vec![]),
                        menu::radio_item_group(
                            Some("menu-radio-label"),
                            vec![],
                            vec![
                                menu::item_group_label(
                                    Some("menu-radio-label"),
                                    vec![],
                                    vec![text("Sort by")],
                                ),
                                menu::radio_item(
                                    true,
                                    "asc",
                                    false,
                                    false,
                                    vec![],
                                    vec![
                                        menu::item_indicator(true, vec![], vec![text("●")]),
                                        menu::item_text(
                                            false,
                                            false,
                                            vec![],
                                            vec![text("Ascending")],
                                        ),
                                    ],
                                ),
                                menu::radio_item(
                                    false,
                                    "desc",
                                    false,
                                    false,
                                    vec![],
                                    vec![
                                        menu::item_indicator(false, vec![], vec![text("●")]),
                                        menu::item_text(
                                            false,
                                            false,
                                            vec![],
                                            vec![text("Descending")],
                                        ),
                                    ],
                                ),
                            ],
                        ),
                        menu::separator(vec![], vec![]),
                        menu::trigger_item(
                            open,
                            false,
                            false,
                            Some("menu-sub-content"),
                            vec![],
                            vec![text("Share")],
                        ),
                        menu::positioner(
                            open,
                            vec![],
                            vec![menu::content(
                                open,
                                Some("menu-sub-content"),
                                None,
                                vec![],
                                vec![
                                    menu::item("email", false, false, vec![], vec![text("Email")]),
                                    menu::item(
                                        "message",
                                        false,
                                        false,
                                        vec![],
                                        vec![text("Message")],
                                    ),
                                ],
                            )],
                        ),
                    ],
                )],
            ),
        ],
    );

    // ブロック 2（closed、disabled）: `hidden`/`data-state="closed"`/
    // `data-disabled`/`disabled` を機械導出表へ載せる。
    let closed_menu = menu::root(
        closed,
        vec![],
        vec![
            menu::trigger(
                closed,
                true,
                Some("menu-closed-content"),
                vec![],
                vec![text("Disabled")],
            ),
            menu::indicator(closed, vec![], vec![text("▾")]),
            menu::positioner(
                closed,
                vec![],
                vec![menu::content(
                    closed,
                    Some("menu-closed-content"),
                    None,
                    vec![],
                    vec![menu::item(
                        "only",
                        false,
                        false,
                        vec![],
                        vec![text("Only item")],
                    )],
                )],
            ),
        ],
    );

    // ブロック 3（context-trigger）: 右クリックで開く Menu 自身のインスタンス。
    let context_menu = menu::root(
        closed,
        vec![],
        vec![
            menu::context_trigger(closed, vec![], vec![text("Right-click area")]),
            menu::positioner(
                closed,
                vec![],
                vec![menu::content(
                    closed,
                    Some("menu-context-content"),
                    None,
                    vec![],
                    vec![
                        menu::item("copy", false, false, vec![], vec![text("Copy")]),
                        menu::item("paste", false, false, vec![], vec![text("Paste")]),
                    ],
                )],
            ),
        ],
    );

    demo_page("Menu", vec![full_menu, closed_menu, context_menu])
}

pub(super) fn menubar_section() -> Node {
    let orientation = Orientation::Horizontal;
    let open = OpenState::Open;
    let closed = OpenState::Closed;

    // Menu 0「File」（open、全機能）: root/menu/trigger/positioner/content/
    // arrow/arrow-tip/item/item-group/item-group-label/separator/
    // checkbox-item/radio-item-group/radio-item/item-text/item-indicator/
    // sub-trigger/sub-content を 1 本のメニューへ集約する。
    let file_menu = menubar::menu(
        open,
        vec![],
        vec![
            menubar::trigger(
                true,
                open,
                false,
                false,
                0,
                Some("mb-file-content"),
                vec![("id", "mb-file-trigger")],
                vec![text("File")],
            ),
            menubar::positioner(
                open,
                vec![],
                vec![menubar::content(
                    open,
                    Some("mb-file-content"),
                    Some("mb-file-trigger"),
                    vec![],
                    vec![
                        menubar::arrow(vec![], vec![menubar::arrow_tip(vec![], vec![])]),
                        menubar::item_group(
                            Some("mb-file-group-label"),
                            vec![],
                            vec![
                                menubar::item_group_label(
                                    Some("mb-file-group-label"),
                                    vec![],
                                    vec![text("File")],
                                ),
                                menubar::item("new", false, true, vec![], vec![text("New")]),
                                menubar::item("close", true, false, vec![], vec![text("Close")]),
                            ],
                        ),
                        menubar::separator(vec![], vec![]),
                        menubar::checkbox_item(
                            true,
                            "word-wrap",
                            false,
                            false,
                            vec![],
                            vec![
                                menubar::item_indicator(true, vec![], vec![text("✓")]),
                                menubar::item_text(false, false, vec![], vec![text("Word Wrap")]),
                            ],
                        ),
                        menubar::checkbox_item(
                            false,
                            "minimap",
                            false,
                            false,
                            vec![],
                            vec![
                                menubar::item_indicator(false, vec![], vec![text("✓")]),
                                menubar::item_text(false, false, vec![], vec![text("Minimap")]),
                            ],
                        ),
                        menubar::separator(vec![], vec![]),
                        menubar::radio_item_group(
                            Some("mb-file-radio-label"),
                            vec![],
                            vec![
                                menubar::item_group_label(
                                    Some("mb-file-radio-label"),
                                    vec![],
                                    vec![text("Layout")],
                                ),
                                menubar::radio_item(
                                    true,
                                    "grid",
                                    false,
                                    false,
                                    vec![],
                                    vec![
                                        menubar::item_indicator(true, vec![], vec![text("●")]),
                                        menubar::item_text(
                                            false,
                                            false,
                                            vec![],
                                            vec![text("Grid")],
                                        ),
                                    ],
                                ),
                                menubar::radio_item(
                                    false,
                                    "list",
                                    false,
                                    false,
                                    vec![],
                                    vec![
                                        menubar::item_indicator(false, vec![], vec![text("●")]),
                                        menubar::item_text(
                                            false,
                                            false,
                                            vec![],
                                            vec![text("List")],
                                        ),
                                    ],
                                ),
                            ],
                        ),
                        menubar::separator(vec![], vec![]),
                        menubar::sub_trigger(
                            open,
                            false,
                            false,
                            Some("mb-sub-content"),
                            vec![],
                            vec![text("Export")],
                        ),
                        menubar::positioner(
                            open,
                            vec![],
                            vec![menubar::sub_content(
                                open,
                                Some("mb-sub-content"),
                                None,
                                vec![],
                                vec![
                                    menubar::item("pdf", false, false, vec![], vec![text("PDF")]),
                                    menubar::item("png", false, false, vec![], vec![text("PNG")]),
                                ],
                            )],
                        ),
                    ],
                )],
            ),
        ],
    );

    // Menu 1「Edit」（closed）: `hidden`/`data-state="closed"` を機械導出
    // 表へ載せる。
    let edit_menu = menubar::menu(
        closed,
        vec![],
        vec![
            menubar::trigger(
                false,
                closed,
                false,
                false,
                1,
                Some("mb-edit-content"),
                vec![],
                vec![text("Edit")],
            ),
            menubar::positioner(
                closed,
                vec![],
                vec![menubar::content(
                    closed,
                    Some("mb-edit-content"),
                    None,
                    vec![],
                    vec![menubar::item(
                        "undo",
                        false,
                        false,
                        vec![],
                        vec![text("Undo")],
                    )],
                )],
            ),
        ],
    );

    // Menu 2「Help」（closed + disabled）: `aria-disabled`/`data-disabled`
    // を機械導出表へ載せる。
    let help_menu = menubar::menu(
        closed,
        vec![],
        vec![menubar::trigger(
            false,
            closed,
            true,
            false,
            2,
            None,
            vec![],
            vec![text("Help")],
        )],
    );

    let body = vec![menubar::root(
        orientation,
        "Main menu",
        vec![],
        vec![file_menu, edit_menu, help_menu],
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
    // イシュー #1654: Anatomy/data-* 表の機械導出元（`primitive_showcase.rs`
    // の網羅検査）を満たすため、disabled 項目（`data-disabled`）と
    // current リンク（`data-current`）を追加し、open 項目の trigger 内に
    // `item_indicator`（`data-orientation`/`data-value`/`aria-hidden`）を
    // 併掲する。
    let props = navigation_menu::NavigationMenuProps::default();
    let body = vec![navigation_menu::root(
        &props,
        "Main",
        vec![],
        vec![navigation_menu::list(
            &props,
            vec![],
            vec![
                navigation_menu::item(
                    open,
                    false,
                    &props,
                    "products",
                    vec![],
                    vec![
                        navigation_menu::trigger(
                            open,
                            false,
                            "products",
                            Some("nm-trigger-0"),
                            Some("nm-content-0"),
                            vec![],
                            vec![
                                text("Products"),
                                navigation_menu::item_indicator(
                                    open,
                                    &props,
                                    "products",
                                    vec![],
                                    vec![text("▾")],
                                ),
                            ],
                        ),
                        navigation_menu::content(
                            open,
                            &props,
                            "products",
                            Some("nm-content-0"),
                            Some("nm-trigger-0"),
                            vec![],
                            vec![
                                navigation_menu::link(
                                    "https://example.com/products/core/",
                                    false,
                                    vec![],
                                    vec![text("Core")],
                                ),
                                navigation_menu::link(
                                    "https://example.com/products/current/",
                                    true,
                                    vec![],
                                    vec![text("Current")],
                                ),
                            ],
                        ),
                    ],
                ),
                navigation_menu::item(
                    closed,
                    false,
                    &props,
                    "docs",
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
                navigation_menu::item(
                    closed,
                    true,
                    &props,
                    "enterprise",
                    vec![],
                    vec![navigation_menu::trigger(
                        closed,
                        true,
                        "enterprise",
                        None,
                        None,
                        vec![],
                        vec![text("Enterprise")],
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
