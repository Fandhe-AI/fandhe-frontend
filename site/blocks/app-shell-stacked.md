# app-shell-stacked

`fandhe-frontend-pre-styled-ui` の `navigation-menu` / `breadcrumb` /
`avatar` / `menu` / `button` / `heading` / `separator` / `card` / `icon` /
`collapsible` 部品を合成した、縦 3 段のアプリケーションシェルです（上から
水平ナビバー・見出し帯・メイン領域）。Blocks セクションは新規部品を追加する
ものではなく、既存の Themes/Primitives 部品を組み合わせた実例集であること
に注意してください（主参照は対応表 ID R1274。出典の固有名・ファイル名は
記載しません）。

見出し帯には「ナビバーと一体化した版」「独立した白帯の版」の 2 系統があり、
違いは面色・境界線・余白のみです。Demo は次の 4 variant を並記します:
一体型見出し帯（見出し + アクションボタン）、独立白帯・パンくず付き
（`breadcrumb` + 見出し + アクション）、独立白帯・タブ風ナビ（見出しの下に
`navigation-menu` を並べ、先頭タブを `aria-current="page"` にする）、
見出し帯なし・フッター付き（メイン下部に区切り線とフッター文言）です。

狭い幅（Demo 枠の幅基準の container query、48rem 未満）ではデスクトップ用の
メインナビを隠し、ハンバーガートリガーのみを表示します。ハンバーガーは
常時展開のパネルへ `aria-controls` で関連付けられており、パネルにはナビの
複製だけを入れます（プロフィールメニューは `id` を持つため複製すると
重複 `id` になるので、通知ボタン・プロフィールメニューはバー内へ全幅で
常時表示します）。

本 Demo は静的な表示例であり、docs サイトは JS ハイドレーションを行わない
ため、押しても何も起きないボタン（通知・プロフィールメニュー・見出しの
アクション・ハンバーガートリガー）はすべて無効化して操作不能であることを
明示しています。`<form>` 要素は一切持たず、データの取得・送信・状態管理を
行いません。ボタンは `type="button"` のまま送信先を持ちません。文言は
すべて独自に書いた架空のものであり、実企業名・実クレデンシャル・PII を
含みません。実際に使うときはリンク先・文言を差し替えてください。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, header, p, section, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card;
use fandhe_frontend_pre_styled_ui::collapsible;
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc 参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// 実在の自組織 URL。
const ORG: &str = "https://github.com/Fandhe-AI";

/// メインナビ本体の項目（value, label, href）。
const NAV_ITEMS: &[(&str, &str, &str)] = &[
    ("overview", "概要", REPO),
    ("projects", "プロジェクト", REPO),
    ("members", "メンバー", ORG),
];

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
        vec![("data-blocks-app-shell-stacked-logo", "")],
        vec![
            geo_icon("M4 4h16v6H4zM4 14h16v6H4z"),
            span(vec![], vec![text("Fandhe Console")]),
        ],
    )
}

/// メインナビ本体（`aria_label` は variant ごとに一意にする。デスクトップ
/// 用と [`hamburger_panel`] への clone とで 2 回出るが、非表示側は
/// `display: none` で a11y ツリーから除外されるため実害を持たない）。
fn nav(aria_label: &str) -> Node {
    let props = NavigationMenuProps::default();
    navigation_menu::root(
        &props,
        aria_label,
        vec![("data-blocks-app-shell-stacked-nav", "")],
        vec![navigation_menu::list(
            &props,
            vec![],
            NAV_ITEMS
                .iter()
                .map(|(value, label, href)| {
                    navigation_menu::item(
                        navigation_menu::OpenState::Closed,
                        false,
                        &props,
                        value,
                        vec![],
                        vec![navigation_menu::link(
                            href,
                            false,
                            vec![],
                            vec![text(*label)],
                        )],
                    )
                })
                .collect(),
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
        vec![("data-blocks-app-shell-stacked-notify", "")],
        vec![geo_icon("M6 8a6 6 0 0 1 12 0v4l2 4H4l2-4z")],
    )
}

/// プロフィールメニュー（`menu::trigger` を `disabled: true` 固定にし、
/// 中に [`avatar`] を入れる。`content_id` は variant ごとに一意にする）。
fn user_menu(variant: &str) -> Node {
    let content_id = format!("blocks-app-shell-stacked-user-menu-{variant}");
    let trigger = menu::trigger(
        OpenState::Closed,
        true,
        Some(content_id.as_str()),
        vec![
            ("aria-label", "Open user menu"),
            ("data-blocks-app-shell-stacked-user-trigger", ""),
        ],
        vec![avatar::root(
            &AvatarProps::default(),
            vec![],
            vec![avatar::fallback(
                ImageStatus::Error,
                vec![],
                vec![text("AR")],
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
            ("data-blocks-app-shell-stacked-toggle", ""),
        ],
        vec![geo_icon("M3 6h18v2H3zM3 11h18v2H3zM3 16h18v2H3z")],
    )
}

/// 常時展開のハンバーガーパネル（ナビの clone のみを持つ、モジュール doc
/// 「パネルへ clone するのはナビだけ」節参照）。
fn hamburger_panel(panel_id: &str, nav_node: Node) -> Node {
    collapsible::content(
        collapsible::OpenState::Open,
        true,
        Some(panel_id),
        vec![("data-blocks-app-shell-stacked-panel", "")],
        vec![nav_node],
    )
}

/// 上部の水平ナビバー（ロゴ・デスクトップ用ナビ・通知・プロフィール・
/// ハンバーガー、常時展開パネルを併設する）。
fn top_bar(variant: &str) -> Node {
    let panel_id = format!("ass-panel-{variant}");
    let aria_label = format!("メインナビゲーション（{variant}）");
    let nav_node = nav(&aria_label);
    let nav_wrap = div(
        vec![("data-blocks-app-shell-stacked-nav-wrap", "")],
        vec![nav_node.clone()],
    );
    div(
        vec![("data-blocks-app-shell-stacked-bar", "")],
        vec![
            header(
                vec![
                    ("data-blocks-app-shell-stacked-root", ""),
                    ("data-blocks-app-shell-stacked-variant", variant),
                ],
                vec![
                    logo(),
                    nav_wrap,
                    div(
                        vec![("data-blocks-app-shell-stacked-actions", "")],
                        vec![notification_button(), user_menu(variant)],
                    ),
                    hamburger(&panel_id),
                ],
            ),
            hamburger_panel(&panel_id, nav_node),
        ],
    )
}

/// アクションボタン（見出し帯の右側、無 JS のため `disabled: true` 固定）。
fn heading_action() -> Node {
    button::button(
        &ButtonProps {
            disabled: true,
            ..ButtonProps::default()
        },
        vec![("data-blocks-app-shell-stacked-heading-action", "")],
        vec![text("新規作成")],
    )
}

/// D/F と同型のパンくず（Home → Blocks → 現在ページ）。
fn breadcrumb_nav() -> Node {
    breadcrumb::root(
        Size::Md,
        BreadcrumbVariant::default(),
        Some("Breadcrumb example"),
        vec![],
        vec![breadcrumb::list(
            vec![],
            vec![
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../../", vec![], vec![text("Home")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../", vec![], vec![text("Blocks")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::current_link(vec![], vec![text("プロジェクト")])],
                ),
            ],
        )],
    )
}

/// `separate-tabs` variant のタブ風ナビ（先頭を `aria-current="page"`）。
fn tabs_nav() -> Node {
    let props = NavigationMenuProps::default();
    navigation_menu::root(
        &props,
        "セクションタブ",
        vec![("data-blocks-app-shell-stacked-tabs", "")],
        vec![navigation_menu::list(
            &props,
            vec![],
            [
                ("overview", "概要", true),
                ("activity", "アクティビティ", false),
                ("settings", "設定", false),
            ]
            .into_iter()
            .map(|(value, label, current)| {
                navigation_menu::item(
                    navigation_menu::OpenState::Closed,
                    false,
                    &props,
                    value,
                    vec![],
                    vec![navigation_menu::link(
                        REPO,
                        current,
                        vec![],
                        vec![text(label)],
                    )],
                )
            })
            .collect(),
        )],
    )
}

/// 見出し帯（`heading_kind` で lead 要素を切り替える。`None` は
/// `no-heading-footer` 用で帯自体を出さない）。
fn heading_band(variant: &str, heading_kind: Option<&str>, title: &'static str) -> Option<Node> {
    // `None` は `no-heading-footer` 用で見出し帯そのものを出さない
    // （`Some("plain")` は unified 用で lead 要素なし）。
    let heading_kind = heading_kind?;
    let lead = match heading_kind {
        "breadcrumb" => Some(breadcrumb_nav()),
        _ => None,
    };
    // 上段（見出し + アクション。`breadcrumb` kind はここへパンくずも含める）
    // と下段（`tabs` kind のみのタブ風ナビ）の 2 行へ分ける。狭幅で
    // タブナビが見出しと同一行に押し込まれて overflow: hidden により
    // 切り取られるのを避けるため（Bugbot Medium/codex P1 是正）。
    let mut top_row: Vec<Node> = Vec::new();
    if let Some(lead) = lead {
        top_row.push(lead);
    }
    top_row.push(heading(
        HeadingLevel::H3,
        &HeadingProps::default(),
        vec![],
        vec![text(title)],
    ));
    top_row.push(heading_action());
    let mut children = vec![div(
        vec![("data-blocks-app-shell-stacked-heading-row", "")],
        top_row,
    )];
    if heading_kind == "tabs" {
        children.push(tabs_nav());
    }
    Some(div(
        vec![
            ("data-blocks-app-shell-stacked-heading", ""),
            ("data-blocks-app-shell-stacked-variant", variant),
        ],
        children,
    ))
}

/// メイン領域（`card` 1 枚。`no-heading-footer` のみ下部にフッターを持つ）。
fn main_section(variant: &str, with_footer: bool) -> Node {
    let mut children = vec![card::root(
        card::CardVariant::Outline,
        vec![],
        vec![
            card::header(
                vec![],
                vec![card::title(vec![], vec![text("最近のアクティビティ")])],
            ),
            card::body(
                vec![],
                vec![p(
                    vec![],
                    vec![text(
                        "ここにページ固有のコンテンツが表示されます（架空のダミー本文）。",
                    )],
                )],
            ),
        ],
    )];
    if with_footer {
        children.push(separator::separator(&SeparatorProps::default(), vec![]));
        children.push(p(
            vec![("data-blocks-app-shell-stacked-footer", "")],
            vec![text("© 2026 Fandhe Console. 架空のダミーフッターです。")],
        ));
    }
    section(
        vec![
            ("aria-label", "メインコンテンツ"),
            ("data-blocks-app-shell-stacked-main", ""),
            ("data-blocks-app-shell-stacked-variant", variant),
        ],
        children,
    )
}

/// 1 variant 分のシェル一式（バー + 見出し帯 + メイン）を組み立てる。
fn shell(
    variant: &'static str,
    heading_kind: Option<&'static str>,
    title: &'static str,
    with_footer: bool,
) -> Node {
    let mut children = vec![top_bar(variant)];
    if let Some(band) = heading_band(variant, heading_kind, title) {
        children.push(band);
    }
    children.push(main_section(variant, with_footer));
    div(
        vec![
            ("data-blocks-app-shell-stacked-shell", ""),
            ("data-blocks-app-shell-stacked-variant", variant),
        ],
        children,
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &'static str) -> Node {
    p(
        vec![("class", "blocks-app-shell-stacked-caption")],
        vec![text(label)],
    )
}

/// `app-shell-stacked` の Demo 本体。4 variant を縦に並記する純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-app-shell-stacked-stack")],
        vec![
            caption("一体型見出し帯（unified）"),
            shell("unified", Some("plain"), "プロジェクト", false),
            caption("独立した白帯・パンくず付き（separate-breadcrumb）"),
            shell(
                "separate-breadcrumb",
                Some("breadcrumb"),
                "プロジェクト",
                false,
            ),
            caption("独立した白帯・タブ風ナビ（separate-tabs）"),
            shell("separate-tabs", Some("tabs"), "プロジェクト", false),
            caption("見出し帯なし・フッター付き（no-heading-footer）"),
            shell("no-heading-footer", None, "プロジェクト", true),
        ],
    )
}
```

## 集約元との差分メモ

- 主参照 R1274（一体型見出し帯）を代表形とし、配色・余白違いのみの
  R1275/R1276/R1277/R1280/R1281 の 5 件は独立したインスタンスにせず、
  既存のテーマトークンへ統一しました。
- R0389（パンくず + アクション）は独立白帯・パンくず付き variant として、
  R0390（タブ風ナビ）は独立白帯・タブ風ナビ variant として、それぞれ実装
  しています。
- R0138（見出しなし + フッター付き骨格）は見出し帯なし・フッター付き
  variant に対応します。
- R0391（パンくず + 検索欄）は独立の variant にしていません。`input` は
  本 block が使う部品一覧に含まれておらず、無 JS デモに送信先のない検索
  入力を置くのを避けるためです。見た目としては、独立白帯・パンくず付き
  variant の右側アクションを検索欄へ差し替えたものに相当します。
- 狭い幅（Demo 枠基準の container query、48rem 未満）ではデスクトップ用の
  メインナビを隠し、常時展開のハンバーガーパネル（ナビの複製）へ委ねます。
  プロフィールメニューは `id` を持つため複製の対象にせず、バー内へ全幅で
  常時表示しています。
- 通知・プロフィールメニュー・見出しのアクション・ハンバーガートリガーは
  すべて無効化して押しても何も起きないことを明示しています。
- 文言・アイコンはすべて独自に書いた架空のものです。配色・余白・角丸は
  既存のテーマトークンに従っています。

関連情報: [Navigation Menu](../themes/navigation-menu.md) /
[Breadcrumb](../themes/breadcrumb.md) / [Avatar](../themes/avatar.md) /
[Menu](../themes/menu.md) / [Button](../themes/button.md) /
[Heading](../themes/heading.md) / [Separator](../themes/separator.md) /
[Card](../themes/card.md) / [Icon](../themes/icon.md) /
[Collapsible](../themes/collapsible.md)
