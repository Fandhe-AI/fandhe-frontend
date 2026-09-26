# header-floating-pill

`navigation-menu` / `button` / `icon` / `collapsible` を合成した浮遊ピル型
ヘッダーです（R0157/R0574）。無 JS のため開閉は行わず、狭幅ではデスクトップ
ナビ・CTA を隠しハンバーガートリガー + 常時展開のドロップダウンパネルを
ピルの下に表示します。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, header, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::collapsible::{self, OpenState};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps};

const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
const PANEL_ID: &str = "hfp-panel";

pub fn demo() -> Node {
    let props = NavigationMenuProps::default();
    let link_item = navigation_menu::item(
        OpenState::Closed,
        false,
        &props,
        "機能",
        vec![],
        vec![navigation_menu::link(
            REPO,
            false,
            vec![],
            vec![text("機能")],
        )],
    );
    let nav = navigation_menu::root(
        &props,
        "ナビ",
        vec![],
        vec![navigation_menu::list(&props, vec![], vec![link_item])],
    );
    let cta = button::button(
        &ButtonProps::default(),
        vec![("data-hfp-cta", "")],
        vec![text("始める")],
    );
    let mark = icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "rect",
            vec![
                ("x", "3"),
                ("y", "3"),
                ("width", "18"),
                ("height", "18"),
                ("rx", "5"),
            ],
            vec![],
        )],
    );
    let logo = span(
        vec![("class", "hfp-logo")],
        vec![mark, text("Fandhe Frontend")],
    );
    let menu_icon = icon(
        &IconProps::default(),
        vec![("stroke", "currentColor")],
        vec![
            el(
                "line",
                vec![
                    ("x1", "3"),
                    ("y1", "6"),
                    ("x2", "21"),
                    ("y2", "6"),
                    ("stroke-width", "2"),
                    ("stroke-linecap", "round"),
                ],
                vec![],
            ),
            el(
                "line",
                vec![
                    ("x1", "3"),
                    ("y1", "12"),
                    ("x2", "21"),
                    ("y2", "12"),
                    ("stroke-width", "2"),
                    ("stroke-linecap", "round"),
                ],
                vec![],
            ),
            el(
                "line",
                vec![
                    ("x1", "3"),
                    ("y1", "18"),
                    ("x2", "21"),
                    ("y2", "18"),
                    ("stroke-width", "2"),
                    ("stroke-linecap", "round"),
                ],
                vec![],
            ),
        ],
    );
    let mobile_trigger = div(
        vec![("class", "hfp-mobile-trigger")],
        vec![collapsible::root(
            OpenState::Open,
            true,
            vec![],
            vec![collapsible::trigger(
                OpenState::Open,
                true,
                Some(PANEL_ID),
                vec![("aria-label", "メニュー")],
                vec![menu_icon],
            )],
        )],
    );
    let mobile_panel = collapsible::content(
        OpenState::Open,
        true,
        Some(PANEL_ID),
        vec![("class", "hfp-mobile-panel")],
        vec![nav.clone(), cta.clone()],
    );

    div(
        vec![("class", "hfp-layout")],
        vec![
            header(
                vec![("class", "hfp-bar")],
                vec![
                    logo,
                    div(vec![("class", "hfp-nav")], vec![nav]),
                    cta,
                    mobile_trigger,
                ],
            ),
            mobile_panel,
        ],
    )
}
```

関連情報: [Navigation Menu](../themes/navigation-menu.md)
