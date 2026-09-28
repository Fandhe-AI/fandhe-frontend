# navbar-app-links

`fandhe-frontend-pre-styled-ui` の `navigation-menu` / `button` / `avatar` /
`menu` / `badge` / `icon` を合成した 1 段のアプリ用ナビバーです（左にロゴ
と主ナビ、右に主操作・通知・アバターメニュー）。`collapsible` は指定外
ですが、狭幅到達性の確立済みパターン（ハンバーガー + 常時展開パネル）を
再利用するために使います。既存の Themes/Primitives 部品の合成例です
（主参照は対応表 ID R1087。出典の固有名は記載しません）。

現在地リンクは「ピル型」「下線型」の 2 種類があり、Demo は 3 インスタンス
を並記します: ピル型・主操作ボタン・受信箱の件数バッジ付き、下線型・
ナビのみ、狭幅でハンバーガーパネルを常時展開した状態（ピル型と同じ構成）。

狭い幅（Demo 枠基準の container query、48rem 未満）ではデスクトップ用ナビ
を隠し、ハンバーガーのみ表示します。パネルは `aria-controls` で常時展開の
ナビ複製のみを持ち、`id` を持つアバターメニューはバー内へ常時表示のまま
複製しません。

本 Demo は静的表示例です。docs サイトは JS ハイドレーションを行わないため
操作系ボタンはすべて無効化しています。`<form>` は使わず、ボタンは
`type="button"` で送信先を持ちません。文言はすべて架空のものです。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, header, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::collapsible;
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps};
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
        vec![("data-blocks-navbar-app-links-logo", "")],
        vec![
            geo_icon("M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z"),
            span(vec![], vec![text("Fandhe Console")]),
        ],
    )
}

/// メインナビ本体（`aria_label` は variant ごとに一意にする。デスクトップ
/// 用と [`hamburger_panel`] への clone とで 2 回出るが、非表示側は
/// `display: none` で a11y ツリーから除外されるため実害を持たない）。
/// `with_badge` は「受信箱」リンクへ件数 [`badge`] を付けるかどうか
/// （R0578 の集約対応、対応表参照）。
fn nav(aria_label: &str, with_badge: bool) -> Node {
    let props = NavigationMenuProps::default();
    let mut inbox_children: Vec<Node> = vec![text("受信箱")];
    if with_badge {
        inbox_children.push(badge::badge(
            &BadgeProps::default(),
            vec![("data-blocks-navbar-app-links-count", "")],
            vec![text("3")],
        ));
    }
    navigation_menu::root(
        &props,
        aria_label,
        vec![("data-blocks-navbar-app-links-nav", "")],
        vec![navigation_menu::list(
            &props,
            vec![],
            vec![
                navigation_menu::item(
                    navigation_menu::OpenState::Closed,
                    false,
                    &props,
                    "dashboard",
                    vec![],
                    vec![navigation_menu::link(
                        "./",
                        true,
                        vec![],
                        vec![text("ダッシュボード")],
                    )],
                ),
                navigation_menu::item(
                    navigation_menu::OpenState::Closed,
                    false,
                    &props,
                    "projects",
                    vec![],
                    vec![navigation_menu::link(
                        REPO,
                        false,
                        vec![],
                        vec![text("プロジェクト")],
                    )],
                ),
                navigation_menu::item(
                    navigation_menu::OpenState::Closed,
                    false,
                    &props,
                    "inbox",
                    vec![],
                    vec![navigation_menu::link(ORG, false, vec![], inbox_children)],
                ),
            ],
        )],
    )
}

/// 主操作ボタン（`pill`/`narrow` のみ、無 JS のため `disabled: true` 固定）。
fn primary_action() -> Node {
    button::button(
        &ButtonProps {
            disabled: true,
            ..ButtonProps::default()
        },
        vec![("data-blocks-navbar-app-links-cta", "")],
        vec![text("新規作成")],
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
        vec![("data-blocks-navbar-app-links-notify", "")],
        vec![geo_icon("M6 8a6 6 0 0 1 12 0v4l2 4H4l2-4z")],
    )
}

/// アバターメニュー（`menu::trigger` を `disabled: true` 固定にし、中に
/// [`avatar`] を入れる。`content_id` は variant ごとに一意にする）。
fn user_menu(variant: &str) -> Node {
    let content_id = format!("blocks-navbar-app-links-user-menu-{variant}");
    let trigger = menu::trigger(
        OpenState::Closed,
        true,
        Some(content_id.as_str()),
        vec![
            ("aria-label", "Open user menu"),
            ("data-blocks-navbar-app-links-user-trigger", ""),
        ],
        vec![avatar::root(
            &AvatarProps::default(),
            vec![],
            vec![avatar::fallback(
                ImageStatus::Error,
                vec![],
                vec![text("YK")],
            )],
        )],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        None,
        vec![],
        vec![
            menu::item("account", false, false, vec![], vec![text("アカウント")]),
            menu::item("settings", false, false, vec![], vec![text("設定")]),
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

/// ハンバーガートリガー（狭い幅専用、[`hamburger_panel`] を
/// `aria-controls` で指す。押しても何も起きないため `disabled: true`
/// 固定だが、`OpenState::Open` によりパネル自体は常に到達可能）。
fn hamburger(panel_id: &str) -> Node {
    collapsible::trigger(
        collapsible::OpenState::Open,
        true,
        Some(panel_id),
        vec![
            ("aria-label", "Open main menu"),
            ("data-blocks-navbar-app-links-toggle", ""),
        ],
        vec![geo_icon("M3 6h18v2H3zM3 11h18v2H3zM3 16h18v2H3z")],
    )
}

/// 常時展開のハンバーガーパネル（ナビの clone のみを持つ、モジュール doc
/// 「id と ARIA の一意性」節参照）。
fn hamburger_panel(panel_id: &str, nav_node: Node) -> Node {
    collapsible::content(
        collapsible::OpenState::Open,
        true,
        Some(panel_id),
        vec![("data-blocks-navbar-app-links-panel", "")],
        vec![nav_node],
    )
}

/// 1 variant 分のナビバー一式（バー + 常時展開パネル）を組み立てる。
/// `current_style` は `"pill"`/`"underline"`（見せ方の切り替え、[`LAYOUT_CSS`]
/// が data 属性で分岐する）、`with_cta` は主操作ボタンの有無、`with_badge`
/// は受信箱の件数バッジの有無、`narrow` はラッパーを狭幅に固定するか。
fn bar(
    variant: &'static str,
    current_style: &'static str,
    with_cta: bool,
    with_badge: bool,
    narrow: bool,
) -> Node {
    let panel_id = format!("nal-panel-{variant}");
    let aria_label = format!("メインナビゲーション（{variant}）");
    let nav_node = nav(&aria_label, with_badge);
    let nav_wrap = div(
        vec![("data-blocks-navbar-app-links-nav-wrap", "")],
        vec![nav_node.clone()],
    );
    let mut actions: Vec<Node> = Vec::new();
    if with_cta {
        actions.push(primary_action());
    }
    actions.push(notification_button());
    actions.push(user_menu(variant));

    let mut frame_attrs = vec![
        ("data-blocks-navbar-app-links-shell", ""),
        // 狭幅ではハンバーガーパネルが root（header）の外側（兄弟）に置かれる
        // ため、current-style は root ではなくパネルとの共通祖先である shell
        // へ付ける。LAYOUT_CSS の current-style セレクタは shell を祖先に
        // 見るため、この属性位置がパネル内リンクへも下線/ピルを届かせる
        // 前提になる（イシュー #2926 レビュー指摘: root 限定だと狭幅表示で
        // 現在地スタイルがパネル内に反映されない）。
        ("data-blocks-navbar-app-links-current-style", current_style),
    ];
    if narrow {
        frame_attrs.push(("data-blocks-navbar-app-links-frame", "narrow"));
    }

    div(
        frame_attrs,
        vec![
            header(
                vec![
                    ("data-blocks-navbar-app-links-root", ""),
                    ("data-blocks-navbar-app-links-variant", variant),
                ],
                vec![
                    logo(),
                    nav_wrap,
                    div(vec![("data-blocks-navbar-app-links-actions", "")], actions),
                    hamburger(&panel_id),
                ],
            ),
            hamburger_panel(&panel_id, nav_node),
        ],
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &'static str) -> Node {
    fandhe_frontend_core::p(
        vec![("class", "blocks-navbar-app-links-caption")],
        vec![text(label)],
    )
}

/// `navbar-app-links` の Demo 本体。3 variant を縦に並記する純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-navbar-app-links-stack")],
        vec![
            caption("現在地をピル型で示す・主操作ボタン付き（pill）"),
            bar("pill", "pill", true, true, false),
            caption("現在地を下線で示す・ナビのみ（underline）"),
            bar("underline", "underline", false, false, false),
            caption("狭幅でハンバーガーパネルを常時展開した状態（narrow）"),
            bar("narrow", "pill", true, true, true),
        ],
    )
}
```

## 集約元との差分メモ

- 主参照 R1087（ピル型代表構成）を基準に、色調違いのみの集約元
  （暗色系 R1084/R1085/R1086、明色系 R1088/R1089）はテーマの明暗切り替えへ
  統一し独立インスタンスにしていません。
- R1084/R1089（主操作ボタン付き）は「ピル型」に、R1086/R1088（ナビのみ）は
  「下線型」に対応します。
- R0578（件数バッジ付きリンク）はピル型・narrow のみが持つ受信箱の件数
  `badge` に、R0579（現在地リンクの見せ方違い）はピル型・下線型の並記に
  対応します。
- ハンバーガーパネルはナビの複製のみを持ち、`id` を持つアバターメニューは
  複製せずバー内へ常時表示します。操作系ボタンはすべて無効化しています。

関連情報: [Navigation Menu](../themes/navigation-menu.md) /
[Button](../themes/button.md) / [Avatar](../themes/avatar.md) /
[Menu](../themes/menu.md) / [Badge](../themes/badge.md) /
[Icon](../themes/icon.md) / [Collapsible](../themes/collapsible.md)
