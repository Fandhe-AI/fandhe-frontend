# header-floating-pill

`navigation-menu` / `button` / `icon` / `collapsible` を合成した浮遊ピル型
ヘッダーです（R0157/R0574）。狭幅ではハンバーガーのみ残し閉状態固定です。

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
            vec![("width", "18"), ("height", "18"), ("rx", "5")],
            vec![],
        )],
    );
    let logo = span(
        vec![("class", "hfp-logo")],
        vec![mark.clone(), text("Fandhe Frontend")],
    );
    let mobile = div(
        vec![("class", "hfp-mobile")],
        vec![collapsible::root(
            OpenState::Closed,
            false,
            vec![],
            vec![
                collapsible::trigger(
                    OpenState::Closed,
                    false,
                    Some(PANEL_ID),
                    vec![("aria-label", "メニュー")],
                    vec![mark],
                ),
                collapsible::content(
                    OpenState::Closed,
                    false,
                    Some(PANEL_ID),
                    vec![],
                    vec![nav.clone(), cta.clone()],
                ),
            ],
        )],
    );

    div(
        vec![("class", "hfp-layout")],
        vec![header(
            vec![("class", "hfp-bar")],
            vec![
                logo,
                div(vec![("class", "hfp-nav")], vec![nav]),
                cta,
                mobile,
            ],
        )],
    )
}
```

関連情報: [Navigation Menu](../themes/navigation-menu.md)
