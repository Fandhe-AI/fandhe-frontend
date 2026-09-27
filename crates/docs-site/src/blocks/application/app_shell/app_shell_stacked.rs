//! `app-shell-stacked` block（イシュー #2896。親トラッキング #2892
//! 「Blocks アプリケーション A（phase:3）」配下）。対応表 ID R1274（主参照）
//! を軸に、配色・余白違いのみの集約元 5 件（R1275/R1276/R1277/R1280/
//! R1281）と骨格違いの集約元 4 件（R0138/R0389/R0390/R0391）の差分を、
//! 本 Demo の 4 variant 並記と原稿の「集約元との差分メモ」節（取得手段・
//! ファイル名は記載しない、`motion-reference-adoption-policy.md` §9 と
//! 同じライセンス上の転記制限）で読み取れるようにする。
//!
//! # 構成（縦 3 段）
//!
//! 上から (1) 水平ナビバー（ロゴ・メインナビ・通知・プロフィールメニュー）
//! (2) 見出し帯（variant により省略） (3) メイン領域、の順に積む。
//!
//! # 使用部品
//!
//! `navigation-menu` / `breadcrumb` / `avatar` / `menu` / `button` /
//! `heading` / `separator` / `card` / `icon` / `collapsible` の 10 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、`blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。狭幅でのナビ到達性は
//! [`super::super::super::marketing::header::header_simple_bar`]
//! （`collapsible` の常時展開パネル）と同じ確定パターンで担保する。
//!
//! # variant の並記
//!
//! | variant | 見出し帯 | 対応 ID |
//! |---|---|---|
//! | `unified` | ナビバーと同面色で一体化。見出し + アクションボタン 1 個 | R1274（代表）+ R1275/1276/1277/1280/1281 |
//! | `separate-breadcrumb` | 独立した白帯。breadcrumb + 見出し + アクション | R0389 |
//! | `separate-tabs` | 独立した白帯。見出し + タブ風ナビ（先頭を `aria-current="page"`） | R0390 |
//! | `no-heading-footer` | 見出し帯なし。メイン下に区切り線 + フッター文言 | R0138 |
//!
//! R0391（パンくず + 検索欄）は独立の variant にせず原稿の差分メモで扱う
//! （`input` はイシュー指定部品に含まれず、無 JS デモに送信先のない検索欄を
//! 置かないため）。
//!
//! # パネルへ clone するのはナビだけ
//!
//! プロフィールメニュー（[`user_menu`]）は `id` を持つため、
//! `collapsible` の常時展開パネルへ `Node::clone()` すると
//! `blocks_contract.rs` の重複 id 検知に抵触する。パネルにはナビのみを
//! clone し、通知ボタン・プロフィールメニューはバー内へ全幅で常時表示する
//! （`header_simple_bar` はナビ・アクション両方を clone するが、本 block の
//! アクション行は id を持つ trigger を含むため差分がある）。
//!
//! # 無 JS のため全アクションを disabled 固定
//!
//! ハンバーガー・通知・プロフィール・見出しのアクションボタンはいずれも
//! `disabled: true` で押しても何も起きないことを明示し、[`LAYOUT_CSS`] の
//! `[data-disabled]` 複合セレクタで `opacity: 1; cursor: default;` に
//! 中和して通常状態と同じ見た目に保つ（`header_simple_bar` と同じ判断）。
//!
//! # 幅の切り替えは container query
//!
//! ラッパーに `container-type: inline-size` を指定し、ビューポート幅では
//! なく Demo 枠自体の幅を基準に `@container (min-width: 48rem)` で
//! デスクトップ用ナビ⇄ハンバーガーパネルを切り替える。
//!
//! # `<form>`/`href="#"`/`data:` を持たない
//!
//! `crate::blocks` モジュール doc「セキュリティ不変条件」節に従う静的な
//! 合成例。`href` は自リポジトリ・自組織の実在 URL とサイト内相対パスに
//! 限る。文言はすべて架空の日本語。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/app-shell-stacked/",
    title: "app-shell-stacked",
    category: BlockCategory::AppShell,
    rust_source: "crates/docs-site/src/blocks/application/app_shell/app_shell_stacked.rs",
    demo_class: "blocks-app-shell-stacked",
    parts: &[
        Part {
            label: "Navigation Menu",
            path: "/themes/navigation-menu/",
        },
        Part {
            label: "Breadcrumb",
            path: "/themes/breadcrumb/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Collapsible",
            path: "/themes/collapsible/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `app_shell_stacked` 固有のレイアウト規則（`--fandhe-*` トークンのみ
/// 使用）。セレクタは `.blocks-app-shell-stacked-*` /
/// `[data-blocks-app-shell-stacked-*]`、および styled 部品の
/// `[data-scope][data-part]` セレクタとの複合セレクタのみを用い、他 block
/// や部品の素のセレクタへは影響させない。
const LAYOUT_CSS: &str = "\
.blocks-app-shell-stacked-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-app-shell-stacked-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-app-shell-stacked-shell] {\n  container-type: inline-size;\n  container-name: blocks-app-shell-stacked;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  overflow: hidden;\n}\n\
[data-blocks-app-shell-stacked-root] {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-2) var(--fandhe-space-4);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  background: var(--fandhe-color-bg-subtle);\n}\n\
[data-blocks-app-shell-stacked-logo] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  white-space: nowrap;\n}\n\
[data-blocks-app-shell-stacked-nav-wrap] {\n  display: none;\n}\n\
[data-blocks-app-shell-stacked-actions] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  margin-inline-start: auto;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-app-shell-stacked-notify][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"menu\"][data-part=\"trigger\"][data-blocks-app-shell-stacked-user-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-app-shell-stacked-toggle] {\n  display: inline-flex;\n}\n\
[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-app-shell-stacked-toggle][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"collapsible\"][data-part=\"content\"][data-blocks-app-shell-stacked-panel] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  padding: 0 var(--fandhe-space-4) var(--fandhe-space-3);\n}\n\
[data-blocks-app-shell-stacked-heading] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  padding: var(--fandhe-space-4);\n}\n\
[data-blocks-app-shell-stacked-heading-row] {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-app-shell-stacked-heading][data-blocks-app-shell-stacked-variant=\"unified\"] {\n  background: var(--fandhe-color-bg-subtle);\n}\n\
[data-blocks-app-shell-stacked-heading][data-blocks-app-shell-stacked-variant^=\"separate\"] {\n  background: var(--fandhe-color-bg);\n  border-block-end: 1px solid var(--fandhe-color-border);\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-app-shell-stacked-heading-action] {\n  margin-inline-start: auto;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-app-shell-stacked-heading-action][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-blocks-app-shell-stacked-main] {\n  padding: var(--fandhe-space-4);\n  max-inline-size: 64rem;\n  margin-inline: auto;\n}\n\
[data-blocks-app-shell-stacked-footer] {\n  margin: var(--fandhe-space-3) 0 0;\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
@container blocks-app-shell-stacked (min-width: 48rem) {\n  \
[data-blocks-app-shell-stacked-nav-wrap] {\n    display: block;\n  }\n  \
[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-app-shell-stacked-toggle] {\n    display: none;\n  }\n  \
[data-scope=\"collapsible\"][data-part=\"content\"][data-blocks-app-shell-stacked-panel] {\n    display: none;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// 10 部品の `data-scope` が揃い、`type="button"` があり、`<form>`・
    /// `href="#"`・`data:` src を持たないこと。
    #[test]
    fn demo_composes_expected_parts_and_has_no_form() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"navigation-menu\"",
            "data-scope=\"breadcrumb\"",
            "data-scope=\"avatar\"",
            "data-scope=\"menu\"",
            "data-scope=\"button\"",
            "data-scope=\"heading\"",
            "data-scope=\"separator\"",
            "data-scope=\"card\"",
            "data-scope=\"icon\"",
            "data-scope=\"collapsible\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 4 variant がそれぞれ 1 回ずつ描画され、caption が 4 件あること。
    #[test]
    fn demo_renders_all_four_variants() {
        let html = render(&demo());
        for variant in [
            "unified",
            "separate-breadcrumb",
            "separate-tabs",
            "no-heading-footer",
        ] {
            assert!(
                html.contains(&format!(
                    "data-blocks-app-shell-stacked-variant=\"{variant}\""
                )),
                "variant {variant} should render"
            );
        }
        assert_eq!(html.matches("blocks-app-shell-stacked-caption").count(), 4);
        assert_eq!(
            html.matches("data-blocks-app-shell-stacked-shell").count(),
            4
        );
    }

    /// ハンバーガーが variant 数だけあり、`aria-label` を持ち、
    /// `aria-controls`/`id` が対応し、パネルに `hidden` が付かないこと。
    #[test]
    fn hamburgers_are_labeled_and_control_an_always_open_panel() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-app-shell-stacked-toggle").count(),
            4
        );
        assert_eq!(html.matches(r#"aria-label="Open main menu""#).count(), 4);
        for variant in [
            "unified",
            "separate-breadcrumb",
            "separate-tabs",
            "no-heading-footer",
        ] {
            let panel_id = format!("ass-panel-{variant}");
            assert!(html.contains(&format!(r#"aria-controls="{panel_id}""#)));
            assert!(html.contains(&format!(r#"id="{panel_id}""#)));
        }
        assert_eq!(
            html.matches("data-blocks-app-shell-stacked-panel").count(),
            4
        );
        // パネル自身は常時展開のため `hidden` を持たないこと（`menu`
        // positioner 等、無関係な閉状態パーツが持つ `hidden` と混同しない
        // よう panel div の開始タグに限定して確認する）。
        for panel_attr_pos in html.match_indices("data-blocks-app-shell-stacked-panel") {
            let tag_start = html[..panel_attr_pos.0]
                .rfind("<div")
                .expect("panel div should have an opening tag");
            let tag_end = html[tag_start..]
                .find('>')
                .map(|rel| tag_start + rel)
                .expect("panel opening tag should close");
            let panel_tag = &html[tag_start..tag_end];
            assert!(!panel_tag.contains("hidden"), "panel_tag={panel_tag}");
        }
    }

    /// 通知・プロフィール trigger・見出しアクションボタンが押しても何も
    /// 起きないよう無効化されていること（`disabled`/`data-disabled` を
    /// 持つ）。
    #[test]
    fn action_controls_are_disabled() {
        let html = render(&demo());
        for hook in [
            "data-blocks-app-shell-stacked-notify",
            "data-blocks-app-shell-stacked-user-trigger",
        ] {
            assert_eq!(
                html.matches(hook).count(),
                4,
                "hook {hook} should render once per variant"
            );
        }
        // プロフィール trigger 自身の開始タグに `data-disabled` が
        // 付いていること（`menu::trigger` は `aria-disabled` を持たない、
        // 下記コメント参照）。
        for pos in html.match_indices("data-blocks-app-shell-stacked-user-trigger") {
            let tag_end = html[pos.0..]
                .find('>')
                .map(|rel| pos.0 + rel)
                .expect("user-trigger tag should close");
            let tag_start = html[..pos.0]
                .rfind("<button")
                .expect("user-trigger should be a button");
            assert!(html[tag_start..tag_end].contains("data-disabled"));
        }
        // unified/separate-* の 3 variant のみ見出しアクションを持つ
        // （no-heading-footer は見出し帯自体を持たないため）。
        assert_eq!(
            html.matches("data-blocks-app-shell-stacked-heading-action")
                .count(),
            3
        );
        // `button::icon_button`/`button::button`（通知 4 + 見出しアクション
        // 3）は `disabled: true` で `aria-disabled="true"` を付与するが、
        // `menu::trigger`/`collapsible::trigger`（プロフィール・ハンバー
        // ガー）は headless 層の契約上 `data-disabled` のみで
        // `aria-disabled` は付与しない（`crates/headless-ui/src/menu.rs`/
        // `collapsible.rs` 参照）。
        assert_eq!(html.matches(r#"aria-disabled="true""#).count(), 7);
    }

    /// パンくずは `separate-breadcrumb` にだけ、`aria-current="page"` の
    /// タブリンクは `separate-tabs` にだけ、フッターは
    /// `no-heading-footer` にだけ現れ、見出し帯は `no-heading-footer` に
    /// 現れないこと。
    #[test]
    fn variant_specific_heading_content_is_scoped() {
        let html = render(&demo());
        // breadcrumb は `separate-breadcrumb` の 1 インスタンスにのみ現れる
        // （`root`/`list`/`item`/`link`×2/`separator`×2/`current-link` の
        // anatomy パーツすべてが `data-scope="breadcrumb"` を持つため、
        // インスタンス数の固定は一意な `aria-label` の出現回数で行う）。
        assert!(html.contains("data-scope=\"breadcrumb\""));
        assert_eq!(html.matches("aria-label=\"Breadcrumb example\"").count(), 1);
        // `aria-current="page"` は `separate-tabs` のタブリンク（1 件）と
        // `separate-breadcrumb` の `breadcrumb::current_link`（現在ページ、
        // 1 件）の計 2 件に現れる。
        assert_eq!(html.matches(r#"aria-current="page""#).count(), 2);
        assert_eq!(
            html.matches("data-blocks-app-shell-stacked-footer").count(),
            1
        );
        // 見出し帯（`unified`/`separate-breadcrumb`/`separate-tabs` の 3
        // variant のみ）は `data-blocks-app-shell-stacked-heading` 属性を
        // 持つ div として現れ、`no-heading-footer` はこの属性を持たない。
        assert_eq!(
            html.matches("data-blocks-app-shell-stacked-heading=\"\"")
                .count(),
            3
        );
    }

    /// ナビの `aria-label` が variant ごとに一意であること（デスクトップ
    /// 用とパネル内 clone の 2 回ずつ出現）。
    #[test]
    fn nav_aria_label_is_unique_per_variant() {
        let html = render(&demo());
        for variant in [
            "unified",
            "separate-breadcrumb",
            "separate-tabs",
            "no-heading-footer",
        ] {
            let label = format!("メインナビゲーション（{variant}）");
            assert_eq!(
                html.matches(&format!("aria-label=\"{label}\"")).count(),
                2,
                "label={label}"
            );
        }
    }

    /// [`LAYOUT_CSS`] が container query・disabled 中和を満たすこと（DOM
    /// 順と視覚順は一致させ、CSS `order` プロパティは使わない設計だが、
    /// `border:` 等の無関係な宣言との部分一致を避けるため本テストでは
    /// `order` 自体は文字列検査しない）。
    #[test]
    fn layout_css_has_container_query_rules() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-app-shell-stacked (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("opacity: 1;"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-app-shell-stacked-stack\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-app-shell-stacked-stack");
    }
}
