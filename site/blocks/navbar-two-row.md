# navbar-two-row

`fandhe-frontend-pre-styled-ui` の `input-group` / `input` /
`navigation-menu` / `tab-nav` / `button` / `avatar` / `menu` / `icon` /
`collapsible` を合成した 2 段構成のアプリ用ナビバーです。1 段目にロゴ・
中央の検索欄・通知ボタン・プロフィールメニュー、2 段目にセカンダリナビを
置きます（主参照は対応表 ID R1093。出典の固有名は記載しません）。

Demo は 2 variant を並記します: ピル型のセカンダリナビ・アクション末尾に
ハンバーガーを置く `pills`、中央寄せのタブ型セカンダリナビ・左端に
ハンバーガーを置く `tabs-center`（多階層・ドキュメントサイト向けの配置）。

広い幅（48rem 以上）では 2 段目を常時表示し、狭い幅ではハンバーガーの
常時展開パネルへ 2 段目ナビの複製を畳みます。パネルは `aria-controls` で
関連付けられ、`hidden` は付きません（常時到達可能）。

本 Demo は静的表示例です。docs サイトは JS ハイドレーションを行わないため
操作系ボタン・検索欄はすべて無効化または送信先を持ちません。`<form>` は
使わず、ボタンは `type="button"` です。文言はすべて架空のものです。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, header, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::collapsible;
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps};
use fandhe_frontend_pre_styled_ui::tab_nav;
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc 参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// 実在の自組織 URL。
const ORG: &str = "https://github.com/Fandhe-AI";

/// 装飾用の自作幾何アイコン（実在ブランドのロゴを模さない、`label: None`）。
fn geo_icon(d: &'static str) -> Node {
    icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", d)], vec![])],
    )
}

/// ロゴ（幾何図形 + ブランド名テキスト）。
fn logo() -> Node {
    span(
        vec![("data-blocks-navbar-two-row-logo", "")],
        vec![
            geo_icon("M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z"),
            span(vec![], vec![text("Fandhe Console")]),
        ],
    )
}

/// 検索アイコン（円 + 柄の 2 path、装飾のため `aria-hidden="true"`）。
fn search_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zm9 17-5.2-5.2"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 中央の検索欄（可視ラベルの代わりに `visually_hidden` + `field::label`
/// で `<label for>` の関連付けによりアクセシブル名を確保する、
/// `hero_search::search_group` と同じ判断）。`variant` は `id` の suffix。
fn search_box(variant: &'static str) -> Node {
    let field_id = format!("blocks-navbar-two-row-search-{variant}");
    let query_field = FieldProps {
        id: &field_id,
        ids: FieldIds::default(),
        disabled: true,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: true,
        invalid: false,
    };
    div(
        vec![("data-blocks-navbar-two-row-search", "")],
        vec![field::root(
            &FieldRootProps {
                orientation: FieldOrientation::Vertical,
            },
            &query_field,
            vec![],
            vec![
                visually_hidden::root(
                    vec![],
                    vec![field::label(
                        &query_field,
                        vec![],
                        vec![text("サイト内を検索")],
                    )],
                ),
                input_group::root(
                    &group_props,
                    vec![],
                    vec![
                        input_group::addon(
                            InputGroupAlign::InlineStart,
                            &group_props,
                            vec![],
                            vec![search_icon()],
                        ),
                        input::input(
                            &InputProps::default(),
                            &query_field,
                            vec![
                                ("type", "search"),
                                ("autocomplete", "off"),
                                ("placeholder", "検索"),
                            ],
                        ),
                    ],
                ),
            ],
        )],
    )
}

/// 通知ボタン（無 JS のため `disabled: true` 固定、アクセシブルネーム付き）。
fn notification_button() -> Node {
    button::icon_button(
        &ButtonProps {
            disabled: true,
            ..ButtonProps::default()
        },
        "Notifications",
        vec![("data-blocks-navbar-two-row-notify", "")],
        vec![geo_icon("M6 8a6 6 0 0 1 12 0v4l2 4H4l2-4z")],
    )
}

/// プロフィールメニュー（`menu::trigger` を `disabled: true` 固定にし、
/// 中に [`avatar`] を入れる。`content_id` は variant ごとに一意にする）。
fn profile_menu(variant: &str) -> Node {
    let content_id = format!("blocks-navbar-two-row-profile-menu-{variant}");
    let trigger = menu::trigger(
        OpenState::Closed,
        true,
        Some(content_id.as_str()),
        vec![
            ("aria-label", "アカウントメニューを開く"),
            ("data-blocks-navbar-two-row-profile-trigger", ""),
        ],
        vec![avatar::root(
            &AvatarProps::default(),
            vec![],
            vec![avatar::fallback(
                ImageStatus::Error,
                vec![],
                vec![text("YS")],
            )],
        )],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        None,
        vec![],
        vec![
            menu::item("profile", false, false, vec![], vec![text("プロフィール")]),
            menu::item("settings", false, false, vec![], vec![text("設定")]),
            menu::item("help", false, false, vec![], vec![text("ヘルプ")]),
            menu::separator(vec![], vec![]),
            menu::item("logout", false, false, vec![], vec![text("ログアウト")]),
        ],
    );
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// ハンバーガートリガー（狭い幅専用、[`mobile_panel`] を `aria-controls`
/// で指す。押しても何も起きないため `disabled: true` 固定だが、
/// `OpenState::Open` によりパネル自体は常に到達可能）。
fn hamburger(panel_id: &str) -> Node {
    collapsible::trigger(
        collapsible::OpenState::Open,
        true,
        Some(panel_id),
        vec![
            ("aria-label", "Open main menu"),
            ("data-blocks-navbar-two-row-toggle", ""),
        ],
        vec![geo_icon("M3 6h18v2H3zM3 11h18v2H3zM3 16h18v2H3z")],
    )
}

/// 常時展開のハンバーガーパネル（2 段目ナビの clone のみを持つ、
/// モジュール doc「`id`/`aria-label` の一意性」節参照）。
fn mobile_panel(panel_id: &str, secondary_nav: Node) -> Node {
    collapsible::content(
        collapsible::OpenState::Open,
        true,
        Some(panel_id),
        vec![("data-blocks-navbar-two-row-panel", "")],
        vec![secondary_nav],
    )
}

/// `pills` variant の 2 段目（`navigation-menu` のピル型リンク、
/// [`LAYOUT_CSS`] の `current-style="pill"` 複合セレクタで角丸を付ける）。
fn secondary_nav_pills(aria_label: &str) -> Node {
    let props = NavigationMenuProps::default();
    let items: &[(&str, &str, &str, bool)] = &[
        ("overview", "概要", "./", true),
        ("projects", "プロジェクト", REPO, false),
        ("reports", "レポート", REPO, false),
        ("members", "メンバー", ORG, false),
    ];
    navigation_menu::root(
        &props,
        aria_label,
        vec![("data-blocks-navbar-two-row-secondary", "")],
        vec![navigation_menu::list(
            &props,
            vec![],
            items
                .iter()
                .map(|(value, label, href, current)| {
                    navigation_menu::item(
                        navigation_menu::OpenState::Closed,
                        false,
                        &props,
                        value,
                        vec![],
                        vec![navigation_menu::link(
                            href,
                            *current,
                            vec![],
                            vec![text(*label)],
                        )],
                    )
                })
                .collect(),
        )],
    )
}

/// `tabs-center` variant の 2 段目（`tab-nav` のタブ型リンク、先頭 1 件を
/// 現在地とする。モジュール doc「2 段目に `tabs` ではなく `tab-nav` を
/// 使う判断」節参照）。
fn secondary_nav_tabs(aria_label: &str) -> Node {
    let items: &[(&str, &str, bool)] = &[
        ("./", "概要", true),
        (REPO, "プロジェクト", false),
        (REPO, "レポート", false),
        (ORG, "メンバー", false),
    ];
    tab_nav::root(
        Size::Md,
        aria_label,
        vec![("data-blocks-navbar-two-row-secondary", "")],
        items
            .iter()
            .map(|(href, label, current)| tab_nav::link(href, *current, vec![], vec![text(*label)]))
            .collect(),
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &'static str) -> Node {
    p(
        vec![("class", "blocks-navbar-two-row-caption")],
        vec![text(label)],
    )
}

/// 1 variant 分の 2 段構成ナビバー一式（1 段目 + 2 段目 + 常時展開パネル）を
/// 組み立てる。`variant` は `"pills"`/`"tabs-center"`、
/// `hamburger_at_start` は 1 段目でハンバーガーを先頭（左端）へ置くか
/// 末尾（右端）へ置くか（R0155 の多階層・ドキュメントサイト向け配置差）。
fn bar(
    variant: &'static str,
    secondary_nav_for: fn(&str) -> Node,
    hamburger_at_start: bool,
) -> Node {
    let panel_id = format!("blocks-navbar-two-row-panel-{variant}");
    let aria_label = format!("セカンダリナビゲーション（{variant}）");
    let secondary_nav = secondary_nav_for(&aria_label);
    let secondary_row = div(
        vec![("data-blocks-navbar-two-row-row-secondary", "")],
        vec![secondary_nav.clone()],
    );

    let actions = div(
        vec![("data-blocks-navbar-two-row-actions", "")],
        vec![notification_button(), profile_menu(variant)],
    );
    let search = search_box(variant);
    let toggle = hamburger(&panel_id);

    let primary_children = if hamburger_at_start {
        vec![toggle, logo(), search, actions]
    } else {
        vec![logo(), search, actions, toggle]
    };

    div(
        vec![
            ("data-blocks-navbar-two-row-shell", ""),
            ("data-blocks-navbar-two-row-variant", variant),
        ],
        vec![
            header(
                vec![("data-blocks-navbar-two-row-row-primary", "")],
                primary_children,
            ),
            secondary_row,
            mobile_panel(&panel_id, secondary_nav),
        ],
    )
}

/// `navbar-two-row` の Demo 本体。2 variant を縦に並記する純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-navbar-two-row-stack")],
        vec![
            caption("ピル型セカンダリナビ・アクション末尾のハンバーガー（pills）"),
            bar("pills", secondary_nav_pills, false),
            caption("中央寄せタブ・左端のハンバーガー（tabs-center）"),
            bar("tabs-center", secondary_nav_tabs, true),
        ],
    )
}
```

## 集約元との差分メモ

- 主参照 R1093（`pills` 相当の 2 段構成）を基準に、暗色配色のみが異なる
  集約元 R1092 は独立インスタンスとして並記していません。`LAYOUT_CSS` は
  `--fandhe-*` トークンのみ参照するため、docs サイトのテーマトグルで
  自動的に暗色へ追随します。
- R0158（2 段目にセカンダリタブを中央寄せで置く構成）は `tabs-center`
  variant に対応します。イシュー本文の部品案は `tabs` でしたが、無 JS の
  docs サイトでは `tabs`（パネル切替 UI）が実際には切り替わらないため、
  ページ遷移用のリンク集合として意味論が正しい `tab-nav` を採用しました。
- R0155（ハンバーガーを左端に置く多階層・ドキュメントサイト向け構成）は
  `tabs-center` variant のハンバーガー配置（1 段目先頭）に対応します。
  `pills` variant はハンバーガーを 1 段目末尾（アクションの右）に置く
  従来配置のままです。
- 狭幅パネルは常時展開（常に到達可能）で、2 段目ナビの複製のみを持ちます。
  操作系ボタン・検索欄はすべて無効化しています。

関連情報: [Input Group](../themes/input-group.md) / [Input](../themes/input.md) /
[Navigation Menu](../themes/navigation-menu.md) / [Tab Nav](../themes/tab-nav.md) /
[Button](../themes/button.md) / [Avatar](../themes/avatar.md) /
[Menu](../themes/menu.md) / [Icon](../themes/icon.md) /
[Collapsible](../themes/collapsible.md)
