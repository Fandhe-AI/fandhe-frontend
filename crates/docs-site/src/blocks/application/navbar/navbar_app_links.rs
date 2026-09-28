//! `navbar-app-links` block（イシュー #2926。親トラッキング #2892
//! 「Blocks アプリケーション A（phase:3）」配下）。対応表 ID R1087（主参照）
//! を軸に、集約元 8 件（R1084/R1085/R1086/R1087/R1088/R1089/R0578/R0579）の
//! 差分を、本 Demo の 3 インスタンス並記と原稿の「集約元との差分メモ」節
//! （取得手段・ファイル名は記載しない、`motion-reference-adoption-policy.md`
//! §9 と同じライセンス上の転記制限）で読み取れるようにする。
//!
//! # 構成（1 段のアプリ用ナビバー）
//!
//! 左にロゴとメインナビのリンク群、右に主操作ボタン（任意）・通知ボタン・
//! アバターのメニューを置く。狭幅ではリンク群をハンバーガーで開く常時展開の
//! 縦パネルへ畳む（狭幅到達性は
//! [`super::super::app_shell::app_shell_stacked`] と同じ確定パターン）。
//!
//! # 使用部品
//!
//! `navigation-menu` / `button` / `avatar` / `menu` / `badge` / `icon` の
//! 6 部品に加え、`collapsible` を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約、`blocks_nav.rs`/`blocks_contract.rs` が検証する）。`collapsible` は
//! イシュー本文の指定にないが使用する: ハンバーガー + 常時展開パネルの
//! 組み合わせは `header_simple_bar`/`app_shell_stacked` で確立済みのパターン
//! であり、`aria-expanded`/`aria-controls` を部品が正しく出すため、button に
//! ARIA 属性を手で付けて同じものを再実装する必要がない。
//!
//! # 3 インスタンスの並記
//!
//! | variant | 現在地の表現 | 右側のアクション | 対応 ID |
//! |---|---|---|---|
//! | `pill` | ピル型（recipe の accent 背景 + 角丸を追加） | 主操作ボタン + 通知 + アバターメニュー、受信箱に件数バッジ | R1087（代表）+ R1085/R1089（主操作ボタン付き）+ R1084（ピル型）+ R0578（件数バッジ） |
//! | `underline` | 下線型（block CSS で accent 背景を打ち消し `border-block-end` で表す） | 通知 + アバターメニューのみ、件数バッジなし | R0579（現在地リンクの見せ方の違い）+ R1086/R1088（ナビのみ） |
//! | `narrow` | ピル型（`pill` と同じ構成） | 通知 + アバターメニューのみ | 狭幅でハンバーガーパネルを常時展開した状態を示す |
//!
//! 暗色系（R1084/R1085/R1086）と明色系（R1087/R1088/R1089）の違いは
//! トークン体系のテーマ切り替えで吸収される配色差にすぎないため Demo には
//! 並べず、原稿の「集約元との差分メモ」節に書く（参照元の配色は持ち込まない）。
//!
//! # `narrow` は Demo 枠を狭めて container query を発火させる
//!
//! `narrow` インスタンスのみラッパーへ
//! `data-blocks-navbar-app-links-frame="narrow"` を付け、[`LAYOUT_CSS`] が
//! `max-inline-size` で約 22rem に絞る。閲覧者の画面幅に関係なく
//! container query が狭幅側に倒れ、ハンバーガーと常時展開パネルが見える。
//!
//! # id と ARIA の一意性
//!
//! `menu` の `content_id` とハンバーガーパネルの id は variant ごとに
//! `blocks-navbar-app-links-user-menu-{variant}` / `nal-panel-{variant}` と
//! 一意にする。ナビの `aria-label` も `メインナビゲーション（{variant}）` で
//! 一意にする。パネルへはナビ（id を持たない）のみを clone し、`menu` は
//! バー内へ常時表示のまま clone しない（`id` を持つ trigger の重複を避ける、
//! `app_shell_stacked` と同じ判断）。
//!
//! # 無 JS のため全アクションを disabled 固定
//!
//! ハンバーガー・通知・主操作・プロフィール trigger はいずれも
//! `disabled: true` で押しても何も起きないことを明示し、[`LAYOUT_CSS`] の
//! `[data-disabled]` 複合セレクタで `opacity: 1; cursor: default;` に
//! 中和して通常状態と同じ見た目に保つ。
//!
//! # `<form>`/`href="#"`/`data:` を持たない
//!
//! `crate::blocks` モジュール doc「セキュリティ不変条件」節に従う静的な
//! 合成例。`href` は自リポジトリ・自組織の実在 URL とサイト内相対パス
//! （現在地リンクの `./`）に限る。文言はすべて架空の日本語。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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

    let mut frame_attrs = vec![("data-blocks-navbar-app-links-shell", "")];
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
                    ("data-blocks-navbar-app-links-current-style", current_style),
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/navbar-app-links/",
    title: "navbar-app-links",
    category: BlockCategory::Navbar,
    rust_source: "crates/docs-site/src/blocks/application/navbar/navbar_app_links.rs",
    demo_class: "blocks-navbar-app-links",
    parts: &[
        Part {
            label: "Navigation Menu",
            path: "/themes/navigation-menu/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
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
            label: "Badge",
            path: "/themes/badge/",
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

/// `navbar_app_links` 固有のレイアウト規則（`--fandhe-*` トークンのみ
/// 使用）。セレクタは `.blocks-navbar-app-links-*` /
/// `[data-blocks-navbar-app-links-*]`、および styled 部品の
/// `[data-scope][data-part]` セレクタとの複合セレクタのみを用い、他 block
/// や部品の素のセレクタへは影響させない。
const LAYOUT_CSS: &str = "\
.blocks-navbar-app-links-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-navbar-app-links-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-navbar-app-links-shell] {\n  container-type: inline-size;\n  container-name: blocks-navbar-app-links;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md);\n  overflow: hidden;\n}\n\
[data-blocks-navbar-app-links-shell][data-blocks-navbar-app-links-frame=\"narrow\"] {\n  max-inline-size: 22rem;\n}\n\
[data-blocks-navbar-app-links-root] {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-2) var(--fandhe-space-4);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  background: var(--fandhe-color-bg-subtle);\n}\n\
[data-blocks-navbar-app-links-logo] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  white-space: nowrap;\n}\n\
[data-blocks-navbar-app-links-nav-wrap] {\n  display: none;\n}\n\
[data-blocks-navbar-app-links-actions] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  margin-inline-start: auto;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-navbar-app-links-cta][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-navbar-app-links-notify][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"menu\"][data-part=\"trigger\"][data-blocks-navbar-app-links-user-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-navbar-app-links-toggle] {\n  display: inline-flex;\n}\n\
[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-navbar-app-links-toggle][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"collapsible\"][data-part=\"content\"][data-blocks-navbar-app-links-panel] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  padding: 0 var(--fandhe-space-4) var(--fandhe-space-3);\n}\n\
[data-blocks-navbar-app-links-root][data-blocks-navbar-app-links-current-style=\"pill\"] [data-scope=\"navigation-menu\"][data-part=\"link\"] {\n  border-radius: var(--fandhe-radius-full);\n}\n\
[data-blocks-navbar-app-links-root][data-blocks-navbar-app-links-current-style=\"underline\"] [data-scope=\"navigation-menu\"][data-part=\"link\"][data-current] {\n  background: transparent;\n  color: var(--fandhe-color-fg);\n  border-radius: 0;\n  border-block-end: 2px solid var(--fandhe-color-accent);\n}\n\
@container blocks-navbar-app-links (min-width: 48rem) {\n  \
[data-blocks-navbar-app-links-nav-wrap] {\n    display: block;\n  }\n  \
[data-scope=\"collapsible\"][data-part=\"trigger\"][data-blocks-navbar-app-links-toggle] {\n    display: none;\n  }\n  \
[data-scope=\"collapsible\"][data-part=\"content\"][data-blocks-navbar-app-links-panel] {\n    display: none;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, ORG, REPO};
    use fandhe_frontend_core::render;

    /// 7 部品の `data-scope` が揃い、`type="button"` があり、`<form>`・
    /// `href="#"`・`data:` src を持たないこと。
    #[test]
    fn demo_composes_expected_parts_and_has_no_form() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"navigation-menu\"",
            "data-scope=\"button\"",
            "data-scope=\"avatar\"",
            "data-scope=\"menu\"",
            "data-scope=\"badge\"",
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

    /// 3 variant がそれぞれ 1 回ずつ描画され、caption が 3 件あること。
    #[test]
    fn demo_renders_all_three_variants() {
        let html = render(&demo());
        for variant in ["pill", "underline", "narrow"] {
            assert!(
                html.contains(&format!(
                    "data-blocks-navbar-app-links-variant=\"{variant}\""
                )),
                "variant {variant} should render"
            );
        }
        assert_eq!(html.matches("blocks-navbar-app-links-caption").count(), 3);
        assert_eq!(
            html.matches("data-blocks-navbar-app-links-shell").count(),
            3
        );
    }

    /// `pill`/`narrow` は現在地リンクがピル型、`underline` は下線型で
    /// 現れること（`data-blocks-navbar-app-links-current-style`）。
    #[test]
    fn current_style_is_pill_for_two_instances_and_underline_for_one() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-navbar-app-links-current-style=\"pill\"")
                .count(),
            2
        );
        assert_eq!(
            html.matches("data-blocks-navbar-app-links-current-style=\"underline\"")
                .count(),
            1
        );
    }

    /// `pill`/`narrow` のみ主操作ボタンと受信箱の件数バッジを持ち、
    /// `underline` は持たないこと（バッジはナビの一部のため、デスクトップ
    /// 用とパネル内 clone の 2 回ずつ出現する。主操作ボタンはアクション行に
    /// 1 回だけ）。
    #[test]
    fn cta_and_badge_are_scoped_to_pill_and_narrow() {
        let html = render(&demo());
        assert_eq!(html.matches("data-blocks-navbar-app-links-cta").count(), 2);
        assert_eq!(
            html.matches("data-blocks-navbar-app-links-count").count(),
            4
        );
    }

    /// ハンバーガーが variant 数だけあり、`aria-label` を持ち、
    /// `aria-controls`/`id` が対応し、パネルに `hidden` が付かないこと。
    #[test]
    fn hamburgers_are_labeled_and_control_an_always_open_panel() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-navbar-app-links-toggle").count(),
            3
        );
        assert_eq!(html.matches(r#"aria-label="Open main menu""#).count(), 3);
        for variant in ["pill", "underline", "narrow"] {
            let panel_id = format!("nal-panel-{variant}");
            assert!(html.contains(&format!(r#"aria-controls="{panel_id}""#)));
            assert!(html.contains(&format!(r#"id="{panel_id}""#)));
        }
        assert_eq!(
            html.matches("data-blocks-navbar-app-links-panel").count(),
            3
        );
        for panel_attr_pos in html.match_indices("data-blocks-navbar-app-links-panel") {
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

    /// 通知・プロフィール trigger が押しても何も起きないよう無効化されて
    /// いること（`disabled`/`data-disabled` を持つ）。
    #[test]
    fn action_controls_are_disabled() {
        let html = render(&demo());
        for hook in [
            "data-blocks-navbar-app-links-notify",
            "data-blocks-navbar-app-links-user-trigger",
        ] {
            assert_eq!(
                html.matches(hook).count(),
                3,
                "hook {hook} should render once per variant"
            );
        }
        for pos in html.match_indices("data-blocks-navbar-app-links-user-trigger") {
            let tag_end = html[pos.0..]
                .find('>')
                .map(|rel| pos.0 + rel)
                .expect("user-trigger tag should close");
            let tag_start = html[..pos.0]
                .rfind("<button")
                .expect("user-trigger should be a button");
            assert!(html[tag_start..tag_end].contains("data-disabled"));
        }
    }

    /// ナビの `aria-label` が variant ごとに一意であること（デスクトップ
    /// 用とパネル内 clone の 2 回ずつ出現）。
    #[test]
    fn nav_aria_label_is_unique_per_variant() {
        let html = render(&demo());
        for variant in ["pill", "underline", "narrow"] {
            let label = format!("メインナビゲーション（{variant}）");
            assert_eq!(
                html.matches(&format!("aria-label=\"{label}\"")).count(),
                2,
                "label={label}"
            );
        }
    }

    /// `narrow` インスタンスのみ狭幅固定属性を持つこと。
    #[test]
    fn only_narrow_instance_has_the_frame_attribute() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-navbar-app-links-frame=\"narrow\"")
                .count(),
            1
        );
    }

    /// 現在地リンク（`aria-current="page"`）が `./` を指し、他リンクは
    /// 実在の自リポジトリ・自組織 URL を指すこと（現在地リンクはナビの
    /// 一部のため、デスクトップ用とパネル内 clone の 2 回ずつ、3 variant
    /// 分で計 6 回出現する）。
    #[test]
    fn current_link_points_to_self_and_others_to_real_urls() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"aria-current="page""#).count(), 6);
        assert!(html.contains(r#"href="./""#));
        assert!(html.contains(&format!(r#"href="{REPO}""#)));
        assert!(html.contains(&format!(r#"href="{ORG}""#)));
    }

    /// [`LAYOUT_CSS`] が container query・disabled 中和・ピル/下線 CSS を
    /// 満たすこと。
    #[test]
    fn layout_css_has_container_query_and_style_rules() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-navbar-app-links (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("border-radius: var(--fandhe-radius-full);"));
        assert!(LAYOUT_CSS.contains("border-block-end: 2px solid var(--fandhe-color-accent);"));
        assert!(LAYOUT_CSS.contains("max-inline-size: 22rem;"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-navbar-app-links-stack\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-navbar-app-links-stack");
    }
}
