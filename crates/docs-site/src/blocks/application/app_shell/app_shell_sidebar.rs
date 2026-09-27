//! `app-shell-sidebar` block（イシュー #2894。Application / App Shell
//! カテゴリの最初の block、`crate::blocks` モジュール doc の契約を
//! `login_01`/`dashboard_01`/`sidebar_07` 等に続いて実装する）。
//!
//! # 使用部品
//!
//! `sidebar`（常設サイドバー、collapsible なし）/ `avatar`（チーム・
//! プロフィールのフォールバックイニシャル）/ `button`（狭幅バーの
//! ハンバーガー icon button）/ `icon`（自作の単純幾何アイコン）/
//! `badge`（未読件数の表示）/ `heading`（メイン領域の見出し）を合成する
//! （[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # 常設サイドバー ⇔ 上部バーの切替はコンテナクエリで行う
//!
//! `sidebar::provider`/`root` は `data-mobile` を付けるとドロワー相当の
//! `position: fixed` へ切り替わるが、これは Demo 枠から視覚的に逸脱する
//! （`sidebar_07` が同じ理由で不採用にした判断を継承）。本 block は
//! `data-mobile` を一切使わず、Demo 枠（`[data-blocks-app-shell-sidebar-
//! frame]`）へ `container-type: inline-size` を設定し、[`LAYOUT_CSS`] の
//! `@container` 規則だけでサイドバー非表示・上部バー表示を切り替える
//! （`header_flyout_menu` の `@container` 前例と同型）。
//!
//! # 3 インスタンスを静的に並記する理由（無 JS）
//!
//! docs サイトは JS ハイドレーションを行わないため、動的な開閉・幅変化を
//! 実演できない。代わりに次の 3 インスタンスを縦に並べ、狙う状態を
//! それぞれ固定表示する。
//!
//! 1. `Desktop`: 幅制約なし。実ビューポートが狭ければコンテナクエリで
//!    自然に上部バーへ切り替わる。
//! 2. `Desktop — brand surface`: 同じ構造で、面色トークン
//!    （`--fandhe-color-sidebar-*`）だけを `accent` 系へ差し替える。
//! 3. `Narrow`: フレーム幅を [`LAYOUT_CSS`] で固定し、ビューポートに
//!    関係なく常に上部バー表示（サイドバー非表示）を示す。
//!
//! 暗色（ダークモード）は専用インスタンスを設けない。`sidebar` recipe の
//! 面トークンはテーマのダークモード切り替えで自動的に追従するため、
//! 静的な Demo でも別インスタンスを要しない。
//!
//! # `class` ではなく `data-*` で CSS フックを渡す
//!
//! `sidebar` の全パーツは呼び出し側 `attrs` の `class` を `drop_class_attr`
//! で黙って除去する契約を持つ（`sidebar_07` モジュール doc「CSS フックの
//! 選び方」節と同型）。本 block も統一して `data-blocks-app-shell-sidebar-*`
//! 属性で CSS フックを渡す。
//!
//! # アイコンは自作の単純幾何図形
//!
//! `sidebar_07::geo_icon` と同型の自作矩形アイコンのみを使う（実アイコン
//! セット由来の path データは複製しない）。
//!
//! # `<form>` を使わない・全データが架空
//!
//! `crate::blocks` モジュール doc の不変条件どおり `<form>` を出力しない。
//! ブランド名・チーム名・氏名・役職はすべて架空のもの（`dummy_assets` の
//! 架空セット、または本モジュール固有の架空値）であり、実企業名・実在
//! 人物・実クレデンシャル・PII を含まない。ナビ・チーム一覧・プロフィール
//! はいずれも静的な初期状態の掲示のみで、選択・送信・永続化・認証処理は
//! 行わない。ハンバーガーボタンの `aria-label` も「静的デモであり実際には
//! 開閉しない」ことと矛盾しない文言（`"Open navigation"`）にする。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::sidebar;
use fandhe_frontend_pre_styled_ui::sidebar::{
    Sidebar, SidebarCollapsible, SidebarMenuButtonProps, SidebarProps, SidebarState,
};

/// 自作の単純な矩形アイコン（`sidebar_07::geo_icon` と同型。lucide 等の
/// 著作物を複製しないためのモジュール doc「アイコンは自作」節参照）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// ハンバーガーアイコン（3 本線）。
fn hamburger_icon() -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", "M4 6h16M4 12h16M4 18h16")], vec![])],
    )
}

/// サイドバー header（ロゴ + 架空ブランド名）。
fn brand_header() -> Node {
    sidebar::header(
        vec![],
        vec![div(
            vec![("data-blocks-app-shell-sidebar-brand", "")],
            vec![
                geo_icon("M4 4h16v16H4z"),
                span(vec![], vec![text("Northshelf Console")]),
            ],
        )],
    )
}

/// メインナビ 1 行（icon + ラベル + 任意の未読 badge）。`badge` は
/// `menu_button` の子ではなく `menu_item` 直下の兄弟として置く
/// （`showcase::sidebar_section` の `menu_action`/`menu_badge` 併記と
/// 同型の配置規約。`menu_button` の内側ラッパーへ混ぜ込むと icon
/// 折りたたみ時のラベル非表示規則が badge にも誤って波及するため）。
fn nav_item(
    icon_path: &'static str,
    label: &'static str,
    active: bool,
    count: Option<&str>,
) -> Node {
    let button = sidebar::menu_button(
        &SidebarMenuButtonProps {
            href: None,
            active,
            ..Default::default()
        },
        Some(geo_icon(icon_path)),
        vec![],
        vec![text(label)],
    );
    let mut children = vec![button];
    if let Some(count) = count {
        children.push(badge::badge(
            &BadgeProps::default(),
            vec![("data-blocks-app-shell-sidebar-nav-badge", "")],
            vec![text(count)],
        ));
    }
    sidebar::menu_item(vec![], children)
}

/// メインナビ群（label なしの `group`、架空の画面 5 件）。
fn main_nav() -> Node {
    sidebar::group(
        None,
        vec![],
        vec![sidebar::group_content(
            vec![],
            vec![sidebar::menu(
                vec![],
                vec![
                    nav_item("M4 4h16v16H4z", "Dashboard", true, None),
                    nav_item(
                        "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
                        "Inbox",
                        false,
                        Some("12"),
                    ),
                    nav_item("M5 3h14v18H5z", "Reports", false, None),
                    nav_item(
                        "M4 4h16v4H4zM4 10h16v4H4zM4 16h16v4H4z",
                        "Projects",
                        false,
                        None,
                    ),
                    nav_item(
                        "M12 2a10 10 0 1 0 .001 20.001A10 10 0 0 0 12 2z",
                        "Settings",
                        false,
                        None,
                    ),
                ],
            )],
        )],
    )
}

/// チーム 1 件（avatar フォールバックのイニシャル + 架空社名）。
fn team_item(suffix: &str, company: &'static str) -> Node {
    let initial = company
        .chars()
        .next()
        .map_or_else(|| "?".to_string(), |c| c.to_uppercase().collect::<String>());
    sidebar::menu_item(
        vec![],
        vec![sidebar::menu_button(
            &SidebarMenuButtonProps {
                href: None,
                active: false,
                ..Default::default()
            },
            Some(avatar::root(
                &AvatarProps::default(),
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text(initial)],
                )],
            )),
            vec![("data-blocks-app-shell-sidebar-team", suffix)],
            vec![text(company)],
        )],
    )
}

/// `Teams` グループ（架空社名 3 件、[`dummy_assets::COMPANY_NAMES`] から
/// 先頭 3 件を採る）。
fn teams_group(suffix: &str) -> Node {
    let label_id = format!("blocks-app-shell-sidebar-teams-label-{suffix}");
    let items: Vec<Node> = dummy_assets::COMPANY_NAMES[..3]
        .iter()
        .map(|company| team_item(suffix, company))
        .collect();
    sidebar::group(
        Some(label_id.as_str()),
        vec![],
        vec![
            sidebar::group_label(Some(label_id.as_str()), vec![], vec![text("Teams")]),
            sidebar::group_content(vec![], vec![sidebar::menu(vec![], items)]),
        ],
    )
}

/// footer のプロフィール行（avatar フォールバック + 架空氏名・役職）。
fn profile_footer() -> Node {
    let name = dummy_assets::PERSON_NAMES[0];
    let title = dummy_assets::JOB_TITLES[0];
    let initials: String = name
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .collect();
    sidebar::footer(
        vec![],
        vec![sidebar::menu(
            vec![],
            vec![sidebar::menu_item(
                vec![],
                vec![sidebar::menu_button(
                    &SidebarMenuButtonProps {
                        href: None,
                        active: false,
                        ..Default::default()
                    },
                    Some(avatar::root(
                        &AvatarProps::default(),
                        vec![],
                        vec![avatar::fallback(
                            ImageStatus::Error,
                            vec![],
                            vec![text(initials)],
                        )],
                    )),
                    vec![],
                    vec![span(
                        vec![("data-blocks-app-shell-sidebar-profile", "")],
                        vec![
                            span(vec![], vec![text(name)]),
                            span(vec![], vec![text(title)]),
                        ],
                    )],
                )],
            )],
        )],
    )
}

/// 常設サイドバー本体（header/content/footer、collapsible なし）。
fn app_sidebar(state: &Sidebar, props: &SidebarProps, suffix: &str, root_id: &str) -> Node {
    sidebar::root(
        state,
        props,
        "Main navigation",
        Some(root_id),
        vec![],
        vec![
            brand_header(),
            sidebar::content(vec![], vec![main_nav(), teams_group(suffix)]),
            profile_footer(),
        ],
    )
}

/// 狭幅時のみ表示する上部バー（ハンバーガー + 画面名 + avatar）。
fn topbar(suffix: &str) -> Node {
    let name = dummy_assets::PERSON_NAMES[0];
    let initials: String = name
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .collect();
    div(
        vec![("data-blocks-app-shell-sidebar-topbar", suffix)],
        vec![
            button::icon_button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    ..ButtonProps::default()
                },
                "Open navigation",
                vec![],
                vec![hamburger_icon()],
            ),
            span(
                vec![("data-blocks-app-shell-sidebar-screen-name", "")],
                vec![text("Dashboard")],
            ),
            avatar::root(
                &AvatarProps::default(),
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text(initials)],
                )],
            ),
        ],
    )
}

/// メイン領域（見出し + 空のコンテンツ枠）。
fn main_area() -> Node {
    div(
        vec![("data-blocks-app-shell-sidebar-main", "")],
        vec![
            heading::heading(
                HeadingLevel::H2,
                &HeadingProps::default(),
                vec![],
                vec![text("Dashboard")],
            ),
            div(
                vec![("data-blocks-app-shell-sidebar-placeholder", "")],
                vec![],
            ),
        ],
    )
}

/// 1 インスタンス分の全体（frame > provider(root + inset) の構造）。
/// `surface` は `"default"`/`"brand"`（[`LAYOUT_CSS`] のセレクタと一致
/// させる、面色トークンの差し替え）。`narrow` は `true` のとき
/// [`LAYOUT_CSS`] がフレーム幅を固定して常に上部バー表示にする。
fn shell(suffix: &str, surface: &'static str, narrow: bool) -> Node {
    let state = Sidebar::new(SidebarState::Expanded);
    let props = SidebarProps {
        collapsible: SidebarCollapsible::None,
        ..SidebarProps::default()
    };
    let root_id = format!("blocks-app-shell-sidebar-root-{suffix}");
    let provider = sidebar::provider(
        &state,
        &props,
        vec![],
        vec![
            app_sidebar(&state, &props, suffix, &root_id),
            sidebar::inset(vec![], vec![topbar(suffix), main_area()]),
        ],
    );

    let mut frame_attrs: Vec<(&str, &str)> = vec![
        ("data-blocks-app-shell-sidebar-frame", ""),
        ("data-blocks-app-shell-sidebar-surface", surface),
    ];
    if narrow {
        frame_attrs.push(("data-blocks-app-shell-sidebar-narrow", ""));
    }
    div(frame_attrs, vec![provider])
}

/// `app-shell-sidebar` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。3 インスタンスを `data-blocks-app-shell-sidebar-stack` の下へ
/// 縦に並べる（モジュール doc「3 インスタンスを静的に並記する理由」参照）。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-app-shell-sidebar-stack", "")],
        vec![
            p(
                vec![("data-blocks-app-shell-sidebar-caption", "")],
                vec![text("Desktop")],
            ),
            shell("desktop", "default", false),
            p(
                vec![("data-blocks-app-shell-sidebar-caption", "")],
                vec![text("Desktop — brand surface")],
            ),
            shell("brand", "brand", false),
            p(
                vec![("data-blocks-app-shell-sidebar-caption", "")],
                vec![text("Narrow")],
            ),
            shell("narrow", "default", true),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/app-shell-sidebar/",
    title: "app-shell-sidebar",
    category: BlockCategory::AppShell,
    rust_source: "crates/docs-site/src/blocks/application/app_shell/app_shell_sidebar.rs",
    demo_class: "blocks-app-shell-sidebar",
    parts: &[
        Part {
            label: "Sidebar",
            path: "/themes/sidebar/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// [`super::mod@self`] の `stylesheet()` が `push_css` する block 固有の
/// レイアウト CSS（`docs/design/docs-site-blocks-section.md` §10 追記節に
/// 従い、並列進行する他 block との `mod.rs::LAYOUT_CSS` 追記衝突を避け
/// 本モジュール側の定数へ分離する）。
///
/// `[data-blocks-app-shell-sidebar-frame]` へ `container-type: inline-size`
/// を設定し、`@container` 規則だけで常設サイドバー⇔上部バーを切り替える
/// （モジュール doc「常設サイドバー ⇔ 上部バーの切替はコンテナクエリで
/// 行う」節参照。`header_flyout_menu` の前例と同型）。面色（`surface`）は
/// リテラル色を書かず `--fandhe-color-accent*` トークン参照のみで
/// `--fandhe-color-sidebar-*` を上書きする。
const LAYOUT_CSS: &str = "\
.blocks-demo.blocks-app-shell-sidebar {\n  padding: 0;\n}\n\
[data-blocks-app-shell-sidebar-stack] {\n  display: flex;\n  flex-direction: column;\n  gap: 1rem;\n}\n\
[data-blocks-app-shell-sidebar-caption] {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-app-shell-sidebar-frame] {\n  container-type: inline-size;\n  container-name: blocks-app-shell-sidebar;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  overflow: hidden;\n}\n\
[data-blocks-app-shell-sidebar-frame][data-blocks-app-shell-sidebar-narrow] {\n  max-inline-size: 22rem;\n}\n\
[data-blocks-app-shell-sidebar-frame] > [data-scope=\"sidebar\"][data-part=\"provider\"] {\n  min-height: 28rem;\n  height: auto;\n}\n\
[data-blocks-app-shell-sidebar-frame][data-blocks-app-shell-sidebar-surface=\"brand\"] [data-scope=\"sidebar\"][data-part=\"root\"] {\n  --fandhe-color-sidebar-bg: var(--fandhe-color-accent);\n  --fandhe-color-sidebar-fg: var(--fandhe-color-accent-fg);\n  --fandhe-color-sidebar-accent: var(--fandhe-color-accent-emphasized);\n  --fandhe-color-sidebar-accent-fg: var(--fandhe-color-accent-fg);\n  --fandhe-color-sidebar-border: var(--fandhe-color-accent-emphasized);\n  --fandhe-color-sidebar-muted: var(--fandhe-color-accent-emphasized);\n}\n\
[data-blocks-app-shell-sidebar-brand] {\n  display: flex;\n  align-items: center;\n  gap: 0.5rem;\n  padding: var(--fandhe-space-2, 0.5rem);\n  font-weight: var(--fandhe-font-font-weight-semibold, 600);\n}\n\
[data-blocks-app-shell-sidebar-profile] {\n  display: flex;\n  flex-direction: column;\n  overflow: hidden;\n  line-height: 1.2;\n  text-align: start;\n}\n\
[data-blocks-app-shell-sidebar-topbar] {\n  display: none;\n  align-items: center;\n  gap: 0.75rem;\n  padding: 0.75rem 1rem;\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-app-shell-sidebar-screen-name] {\n  margin-inline-end: auto;\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n}\n\
[data-blocks-app-shell-sidebar-main] {\n  display: flex;\n  flex-direction: column;\n  gap: 1rem;\n  padding: 1.5rem;\n}\n\
[data-blocks-app-shell-sidebar-placeholder] {\n  min-height: 16rem;\n  border: 2px dashed var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n}\n\
@container blocks-app-shell-sidebar (max-width: 40rem) {\n  \
[data-blocks-app-shell-sidebar-frame] [data-scope=\"sidebar\"][data-part=\"root\"] {\n    display: none;\n  }\n  \
[data-blocks-app-shell-sidebar-topbar] {\n    display: flex;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 6 種の部品を実際に出力し、非対話制約
    /// （`<form>`/`data:` 不在）を満たすことの単体回帰
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_has_no_form_or_data_uri() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"sidebar\"",
            "data-scope=\"avatar\"",
            "data-scope=\"button\"",
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(!html.contains("<form"), "demo should not emit <form>");
        assert!(
            !html.contains("src=\"data:"),
            "demo should not emit data: URIs"
        );
    }

    /// 3 インスタンス（caption・frame・topbar）が並記されていること。
    #[test]
    fn demo_has_three_instances() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-app-shell-sidebar-caption")
                .count(),
            3,
            "demo should render exactly 3 captions"
        );
        assert_eq!(
            html.matches("data-blocks-app-shell-sidebar-frame").count(),
            3,
            "demo should render exactly 3 frames"
        );
        assert_eq!(
            html.matches("data-blocks-app-shell-sidebar-topbar").count(),
            3,
            "demo should render exactly 3 topbars (hidden by default via CSS)"
        );
        assert_eq!(
            html.matches("data-blocks-app-shell-sidebar-surface=\"brand\"")
                .count(),
            1,
            "demo should render exactly 1 brand-surface instance"
        );
        assert_eq!(
            html.matches("data-blocks-app-shell-sidebar-narrow").count(),
            1,
            "demo should render exactly 1 narrow instance"
        );
    }

    /// ハンバーガーボタンが `type="button"` と `aria-label` を持つこと。
    #[test]
    fn hamburger_button_is_type_button_with_aria_label() {
        let html = render(&demo());
        assert!(
            html.contains(r#"aria-label="Open navigation""#),
            "hamburger button should have an aria-label"
        );
        assert!(
            html.contains(r#"type="button""#),
            "buttons should stay type=\"button\" (no implicit form submit)"
        );
    }

    /// `LAYOUT_CSS` が `@container` の狭幅切替とブランド面色の上書きを持ち、
    /// 色リテラルを含まないこと。
    #[test]
    fn layout_css_has_container_query_and_token_only_brand_surface() {
        assert!(
            LAYOUT_CSS.contains("@container blocks-app-shell-sidebar (max-width: 40rem)"),
            "LAYOUT_CSS should define the narrow-width container query"
        );
        assert!(
            LAYOUT_CSS.contains("--fandhe-color-sidebar-bg: var(--fandhe-color-accent)"),
            "LAYOUT_CSS should override sidebar bg via a theme token, not a literal color"
        );
        assert!(
            !LAYOUT_CSS.contains('#'),
            "LAYOUT_CSS should not contain literal hex color values"
        );
    }
}
