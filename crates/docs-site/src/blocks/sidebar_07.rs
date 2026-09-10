//! `sidebar-07` block（イシュー #2090。shadcn/ui Blocks の
//! `sidebar-07`〔icon 折りたたみ可能なサイドバー〕に相当する合成例で、
//! `crate::blocks` モジュール doc の契約を `login_01`/`dashboard_01` に
//! 続いて 3 件目に実装する）。
//!
//! # 使用部品
//!
//! `sidebar`（team switcher 付き header・collapsible 付き nav-main・
//! Projects グループ・avatar + menu の footer）/ `collapsible`（nav-main の
//! 折りたたみ行）/ `menu`（team switcher・user footer の閉じたドロップ
//! ダウン）/ `avatar`（footer のユーザーアイコン）/ `breadcrumb`（inset
//! ヘッダー）/ `separator`（inset ヘッダーの縦区切り）/ `icon`（自作の
//! 単純幾何アイコン）の部品を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約）。
//!
//! # 無 JS のため 2 状態を静的に並置する
//!
//! shadcn 側の `sidebar-07` は Cmd/Ctrl+B・トリガーボタンで開閉状態を
//! 動的に切り替えるが、docs サイトは JS ハイドレーションを行わないため、
//! expanded インスタンスと collapsed（icon）インスタンスを縦に並べて
//! 静的に掲示する（`crate::blocks` モジュール doc の Blocks セクション
//! 全体の設計方針、`docs/design/docs-site-blocks-section.md` §2 参照）。
//!
//! # `asChild` 相当が無いことによる構造的差分
//!
//! shadcn の `sidebar-07` は `SidebarMenuButton` を `CollapsibleTrigger
//! asChild`（nav-main の各行）/ `DropdownMenuTrigger asChild`（team
//! switcher・user footer）で包み、1 つの `<button>` に両方の役割を
//! 持たせる。本リポジトリの `sidebar::menu_button`/`collapsible::trigger`/
//! `menu::trigger` はいずれも独立した要素を出力し、ボタンの入れ子は
//! 不正な HTML になるため同じ構造は取れない。本実装では nav-main の行は
//! `collapsible::trigger` を、team switcher・user footer は `menu::trigger`
//! を行本体とし、`sidebar::menu_button` 相当の見た目は [`LAYOUT_CSS`] で
//! 補う。この不足（`sidebar::menu_button` に collapsible/menu の trigger
//! 役割を委譲する asChild 相当の手段が無い）は Phase 4 親イシュー #2057 へ
//! コメントで送る（`.claude/rules/out-of-scope-tracking.md` 対応、新規
//! Issue は起票しない）。
//!
//! # icon 折りたたみ時に自動で隠れないものへの補完 CSS
//!
//! `sidebar::stylesheet` の `ICON_COLLAPSED` 規則が視覚的に隠すのは
//! `sidebar` 自身のパーツ（`group-label`/`menu-badge`/`menu-button` 内側
//! ラベル等）のみで、`collapsible::trigger`/`menu::trigger` のラベル・
//! chevron はこの規則の対象外である（sidebar scope に属さないため）。
//! 本モジュールはラベル・chevron に `data-blocks-sidebar-07-label`/
//! `-chevron` 属性を付け、[`LAYOUT_CSS`] が icon 折りたたみ時に前者を
//! `crate::pre_styled_ui::visually_hidden::clip_declarations`
//! （`pub(crate)` のため直接呼べず、同一の宣言列を書き写す）と同じ
//! clip 手法で視覚的に隠す（ボタンのアクセシブルネームを保つ、WCAG
//! 4.1.2）。後者（装飾用の chevron）は `display: none` で足りる。
//! collapsed インスタンスでは nav-main の collapsible を常に `Closed`
//! で組み、team switcher・user footer の menu も常に `Closed` で組む
//! （閉状態の掲示のみ、`aria-expanded="false"` が自動で付く）。
//!
//! # `<form>` を使わない・認証処理を持たない・全データが架空
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ブランド名（`Acme Inc`/`Acme Corp.`/`Evil Corp.`）・
//! ユーザー名・メールアドレスはすべて架空のものであり、実企業名・実在
//! 人物・実クレデンシャル・PII を含まない。team switcher・user footer の
//! メニュー項目・nav-main の折りたたみはいずれも静的な初期状態を表示する
//! のみで、選択・送信・永続化・認証処理は行わない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `sidebar` の全パーツ・`menu::root`・`avatar::root`・`breadcrumb::root`・
//! `separator::separator` は呼び出し側 `attrs` の `class` を
//! `drop_class_attr` により黙って除去する契約を持つ（`crate::blocks::mod`
//! モジュール doc「CSS フック」節参照）。これらへの Demo 固有 CSS フックは
//! `data-blocks-sidebar-07-*` 属性で渡し、[`LAYOUT_CSS`] 側も同じ属性
//! セレクタで対応する。素の `div`/`span`・headless 再エクスポート
//! （`collapsible::*`、`menu::trigger`/`content`/`item` 等）は `class` が
//! そのまま効くが、本 block では統一して `data-*` に揃える。
//!
//! # アイコンは自作の単純幾何図形（著作物を複製しない）
//!
//! lucide 等の実アイコンセット由来の path データは使わず、
//! `crate::showcase::sidebar_section` の先例（`M3 3h8v8H3z…`）と同程度の
//! 単純な矩形図形を自作する（モジュール doc「shadcn 側との差分メモ」節
//! 参照）。

use super::{Block, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::collapsible;
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps, SeparatorVariant};
use fandhe_frontend_pre_styled_ui::sidebar;
use fandhe_frontend_pre_styled_ui::sidebar::{
    Sidebar, SidebarCollapsible, SidebarMenuButtonProps, SidebarMenuSubButtonProps, SidebarProps,
    SidebarState,
};
use fandhe_frontend_pre_styled_ui::{Orientation, Size};

/// 自作の単純な矩形アイコン（`d` は呼び出し側が座標を選ぶ、lucide 等の
/// 著作物を複製しないためのモジュール doc「アイコンは自作」節参照）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// header の team switcher（閉じた `menu`、`Acme Inc` / `Enterprise` を
/// 表示する）。`suffix` で expanded/collapsed インスタンス間の id 衝突を
/// 避ける（モジュール doc「id はすべて suffix で分ける」相当）。
fn team_switcher(suffix: &str) -> Node {
    let content_id = format!("blocks-sidebar-07-team-menu-{suffix}");
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id.as_str()),
        vec![
            ("aria-label", "Select team"),
            ("data-blocks-sidebar-07-team-trigger", ""),
        ],
        vec![
            geo_icon("M4 4h16v16H4z"),
            span(
                vec![("data-blocks-sidebar-07-label", "")],
                vec![
                    span(vec![], vec![text("Acme Inc")]),
                    span(vec![], vec![text("Enterprise")]),
                ],
            ),
            span(
                vec![("data-blocks-sidebar-07-chevron", "")],
                vec![text("\u{2195}")],
            ),
        ],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        None,
        vec![],
        vec![
            menu::item("acme-inc", false, false, vec![], vec![text("Acme Inc")]),
            menu::item("acme-corp", false, false, vec![], vec![text("Acme Corp.")]),
            menu::item("evil-corp", false, false, vec![], vec![text("Evil Corp.")]),
            menu::separator(vec![], vec![]),
            menu::item("add-team", false, false, vec![], vec![text("Add team")]),
        ],
    );
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    let root = menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    );
    sidebar::header(
        vec![],
        vec![sidebar::menu(
            vec![],
            vec![sidebar::menu_item(vec![], vec![root])],
        )],
    )
}

/// nav-main の 1 行（`collapsible` を行本体とし、内側に icon・ラベル・
/// chevron を置く。モジュール doc「`asChild` 相当が無いことによる構造的
/// 差分」参照）。`open` は常に呼び出し元が明示する（collapsed インスタンス
/// では常に `Closed` を渡す契約）。
#[allow(clippy::too_many_arguments)]
fn nav_collapsible(
    suffix: &str,
    key: &str,
    label: &'static str,
    icon_path: &'static str,
    open: OpenState,
    active_sub: Option<&'static str>,
    subs: &[&'static str],
) -> Node {
    let content_id = format!("blocks-sidebar-07-nav-{key}-{suffix}");
    let trigger = collapsible::trigger(
        open,
        false,
        Some(content_id.as_str()),
        vec![("data-blocks-sidebar-07-nav-trigger", "")],
        vec![
            geo_icon(icon_path),
            span(
                vec![("data-blocks-sidebar-07-label", "")],
                vec![text(label)],
            ),
            collapsible::indicator(
                open,
                false,
                vec![("data-blocks-sidebar-07-chevron", "")],
                vec![text("\u{25be}")],
            ),
        ],
    );
    let sub_items: Vec<Node> = subs
        .iter()
        .map(|sub_label| {
            sidebar::menu_sub_item(
                vec![],
                vec![sidebar::menu_sub_button(
                    &SidebarMenuSubButtonProps {
                        href: None,
                        active: active_sub == Some(*sub_label),
                        ..Default::default()
                    },
                    vec![],
                    vec![text(*sub_label)],
                )],
            )
        })
        .collect();
    let content = collapsible::content(
        open,
        false,
        Some(content_id.as_str()),
        vec![],
        vec![sidebar::menu_sub(vec![], sub_items)],
    );
    sidebar::menu_item(vec![], vec![trigger, content])
}

/// `Platform` グループ（Playground〔collapsible、sub 3 件〕/ Models /
/// Documentation / Settings）。
fn platform_group(suffix: &str, nav_open: OpenState) -> Node {
    let menu = sidebar::menu(
        vec![],
        vec![
            nav_collapsible(
                suffix,
                "playground",
                "Playground",
                "M4 4h16v16H4z",
                nav_open,
                Some("History"),
                &["History", "Starred", "Settings"],
            ),
            nav_collapsible(
                suffix,
                "models",
                "Models",
                "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
                OpenState::Closed,
                None,
                &["Genesis", "Explorer", "Quantum"],
            ),
            nav_collapsible(
                suffix,
                "documentation",
                "Documentation",
                "M5 3h14v18H5z",
                OpenState::Closed,
                None,
                &["Introduction", "Get Started", "Tutorials", "Changelog"],
            ),
            nav_collapsible(
                suffix,
                "settings",
                "Settings",
                "M12 2a10 10 0 1 0 .001 20.001A10 10 0 0 0 12 2z",
                OpenState::Closed,
                None,
                &["General", "Team", "Billing", "Limits"],
            ),
        ],
    );
    sidebar::group(
        None,
        vec![],
        vec![sidebar::group_content(vec![], vec![menu])],
    )
}

/// `Projects` グループ（collapsible を持たない通常の `menu_button` 4 件）。
fn projects_group(suffix: &str) -> Node {
    let label_id = format!("blocks-sidebar-07-projects-label-{suffix}");

    fn project_item(icon_path: &'static str, label: &'static str) -> Node {
        sidebar::menu_item(
            vec![],
            vec![sidebar::menu_button(
                &SidebarMenuButtonProps {
                    href: None,
                    active: false,
                    ..Default::default()
                },
                Some(geo_icon(icon_path)),
                vec![],
                vec![text(label)],
            )],
        )
    }

    sidebar::group(
        Some(label_id.as_str()),
        vec![],
        vec![
            sidebar::group_label(Some(label_id.as_str()), vec![], vec![text("Projects")]),
            sidebar::group_content(
                vec![],
                vec![sidebar::menu(
                    vec![],
                    vec![
                        project_item("M4 4h16v16H4z", "Design Engineering"),
                        project_item(
                            "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
                            "Sales & Marketing",
                        ),
                        project_item("M5 3h14v18H5z", "Travel"),
                        project_item("M12 2a10 10 0 1 0 .001 20.001A10 10 0 0 0 12 2z", "More"),
                    ],
                )],
            ),
        ],
    )
}

/// footer のユーザー行（avatar fallback + 閉じた `menu`。
/// `dashboard_01::user_menu` と同型だが、trigger 自体が avatar・名前・
/// メール・chevron を内包する構成へ寄せる、モジュール doc参照）。
fn user_menu(suffix: &str) -> Node {
    let content_id = format!("blocks-sidebar-07-user-menu-{suffix}");
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id.as_str()),
        vec![
            ("aria-label", "Open user menu"),
            ("data-blocks-sidebar-07-user-trigger", ""),
        ],
        vec![
            avatar::root(
                &AvatarProps::default(),
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text("AL")],
                )],
            ),
            span(
                vec![("data-blocks-sidebar-07-label", "")],
                vec![
                    span(vec![], vec![text("Ada Lovelace")]),
                    span(vec![], vec![text("ada@example.com")]),
                ],
            ),
            span(
                vec![("data-blocks-sidebar-07-chevron", "")],
                vec![text("\u{2195}")],
            ),
        ],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        None,
        vec![],
        vec![
            menu::item(
                "upgrade",
                false,
                false,
                vec![],
                vec![text("Upgrade to Pro")],
            ),
            menu::separator(vec![], vec![]),
            menu::item("account", false, false, vec![], vec![text("Account")]),
            menu::item("billing", false, false, vec![], vec![text("Billing")]),
            menu::item(
                "notifications",
                false,
                false,
                vec![],
                vec![text("Notifications")],
            ),
            menu::separator(vec![], vec![]),
            menu::item("logout", false, false, vec![], vec![text("Log out")]),
        ],
    );
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    let root = menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    );
    sidebar::footer(
        vec![],
        vec![sidebar::menu(
            vec![],
            vec![sidebar::menu_item(vec![], vec![root])],
        )],
    )
}

/// 左サイドバーの `root`（`provider > root` の子結合子 CSS に合わせて
/// [`demo`] が本関数の戻り値を `provider` の直接の子として置く。`rail` は
/// `root` の直接の子として置く、`dashboard_01::sidebar_root` と同型）。
fn app_sidebar(
    state: &Sidebar,
    props: &SidebarProps,
    suffix: &str,
    root_id: &str,
    nav_open: OpenState,
) -> Node {
    sidebar::root(
        state,
        props,
        "Main navigation",
        Some(root_id),
        vec![],
        vec![
            team_switcher(suffix),
            sidebar::content(
                vec![],
                vec![platform_group(suffix, nav_open), projects_group(suffix)],
            ),
            user_menu(suffix),
            sidebar::rail(state, "Toggle sidebar rail", vec![], vec![]),
        ],
    )
}

/// inset 側（トリガー + 縦 separator + breadcrumb のヘッダー、3 枚の
/// プレースホルダーカード、大きなプレースホルダー 1 枚）。
fn inset_area(state: &Sidebar, root_id: &str) -> Node {
    let header = div(
        vec![("data-blocks-sidebar-07-header", "")],
        vec![
            sidebar::trigger(state, "Toggle sidebar", Some(root_id), vec![], vec![]),
            separator::separator(
                &SeparatorProps {
                    orientation: Orientation::Vertical,
                    variant: SeparatorVariant::Solid,
                },
                vec![],
            ),
            breadcrumb::root(
                Size::Md,
                BreadcrumbVariant::default(),
                Some("Breadcrumb"),
                vec![],
                vec![breadcrumb::list(
                    vec![],
                    vec![
                        breadcrumb::item(
                            vec![],
                            vec![breadcrumb::link(
                                "../",
                                vec![],
                                vec![text("Build Your Application")],
                            )],
                        ),
                        breadcrumb::separator(vec![], vec![text("/")]),
                        breadcrumb::item(
                            vec![],
                            vec![breadcrumb::current_link(
                                vec![],
                                vec![text("Data Fetching")],
                            )],
                        ),
                    ],
                )],
            ),
        ],
    );

    let grid = div(
        vec![("data-blocks-sidebar-07-grid", "")],
        vec![
            div(vec![("data-blocks-sidebar-07-placeholder", "")], vec![]),
            div(vec![("data-blocks-sidebar-07-placeholder", "")], vec![]),
            div(vec![("data-blocks-sidebar-07-placeholder", "")], vec![]),
        ],
    );
    let large_placeholder = div(vec![("data-blocks-sidebar-07-placeholder-lg", "")], vec![]);

    sidebar::inset(vec![], vec![header, grid, large_placeholder])
}

/// `sidebar-07` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
/// expanded/collapsed の 2 インスタンスを `data-blocks-sidebar-07-stack`
/// の下へ縦に並べる（モジュール doc「無 JS のため 2 状態を静的に並置する」
/// 参照。両インスタンスとも `collapsible: SidebarCollapsible::Icon` を
/// 明示する。既定は `Offcanvas` のため expanded 側にも明示が必要）。
pub fn demo() -> Node {
    let expanded_state = Sidebar::new(SidebarState::Expanded);
    let expanded_props = SidebarProps {
        collapsible: SidebarCollapsible::Icon,
        ..SidebarProps::default()
    };
    let expanded_root_id = "blocks-sidebar-07-root-expanded";
    let expanded = sidebar::provider(
        &expanded_state,
        &expanded_props,
        vec![("data-blocks-sidebar-07-instance", "")],
        vec![
            app_sidebar(
                &expanded_state,
                &expanded_props,
                "expanded",
                expanded_root_id,
                OpenState::Open,
            ),
            inset_area(&expanded_state, expanded_root_id),
        ],
    );

    let collapsed_state = Sidebar::new(SidebarState::Collapsed);
    let collapsed_props = SidebarProps {
        collapsible: SidebarCollapsible::Icon,
        ..SidebarProps::default()
    };
    let collapsed_root_id = "blocks-sidebar-07-root-collapsed";
    let collapsed = sidebar::provider(
        &collapsed_state,
        &collapsed_props,
        vec![("data-blocks-sidebar-07-instance", "")],
        vec![
            app_sidebar(
                &collapsed_state,
                &collapsed_props,
                "collapsed",
                collapsed_root_id,
                OpenState::Closed,
            ),
            inset_area(&collapsed_state, collapsed_root_id),
        ],
    );

    div(
        vec![("data-blocks-sidebar-07-stack", "")],
        vec![
            p(
                vec![("data-blocks-sidebar-07-caption", "")],
                vec![text("Expanded")],
            ),
            expanded,
            p(
                vec![("data-blocks-sidebar-07-caption", "")],
                vec![text("Collapsed (icon)")],
            ),
            collapsed,
        ],
    )
}
// blocks-code:end

/// [`super::BLOCKS`] へ登録するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/sidebar-07/",
    title: "sidebar-07",
    rust_source: "crates/docs-site/src/blocks/sidebar_07.rs",
    demo_class: "blocks-sidebar-07",
    parts: &[
        Part {
            label: "Sidebar",
            path: "/themes/sidebar/",
        },
        Part {
            label: "Collapsible",
            path: "/themes/collapsible/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Breadcrumb",
            path: "/themes/breadcrumb/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    demo,
};

/// [`super::mod@self`] の `stylesheet()` が `push_css` する block 固有の
/// レイアウト CSS（`docs/design/docs-site-blocks-section.md` §10 追記節に
/// 従い、並列進行する他 block との `mod.rs::LAYOUT_CSS` 追記衝突を避け
/// 本モジュール側の定数へ分離する）。
///
/// `[data-blocks-sidebar-07-label]` の icon 折りたたみ時の非表示規則は
/// `fandhe_frontend_pre_styled_ui::visually_hidden::clip_declarations`
/// （`pub(crate)` のため docs-site から直接呼べない）と同じ宣言列を書き写す
/// （モジュール doc「icon 折りたたみ時に自動で隠れないものへの補完 CSS」
/// 参照。`display: none` ではなくボタンのアクセシブルネームを保つ clip
/// 手法、WCAG 4.1.2）。
pub(super) const LAYOUT_CSS: &str = "\
.blocks-demo.blocks-sidebar-07 {\n  padding: 0;\n  overflow-x: auto;\n}\n\
[data-blocks-sidebar-07-stack] {\n  display: flex;\n  flex-direction: column;\n  gap: 1rem;\n}\n\
[data-blocks-sidebar-07-caption] {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-sidebar-07-instance][data-scope=\"sidebar\"][data-part=\"provider\"] {\n  min-height: 32rem;\n  height: auto;\n  min-width: 56rem;\n}\n\
[data-blocks-sidebar-07-team-trigger], [data-blocks-sidebar-07-user-trigger] {\n  display: flex;\n  align-items: center;\n  gap: 0.5rem;\n  width: 100%;\n  text-align: start;\n}\n\
[data-blocks-sidebar-07-nav-trigger] {\n  display: flex;\n  align-items: center;\n  gap: 0.5rem;\n  width: 100%;\n  text-align: start;\n  background: transparent;\n  border: none;\n  cursor: pointer;\n  padding: var(--fandhe-space-2, 0.5rem) var(--fandhe-space-2, 0.5rem);\n  border-radius: var(--fandhe-radius-md);\n  color: inherit;\n  font: inherit;\n}\n\
[data-blocks-sidebar-07-nav-trigger]:hover {\n  background: var(--fandhe-color-bg-muted);\n}\n\
[data-blocks-sidebar-07-label] {\n  display: flex;\n  flex-direction: column;\n  overflow: hidden;\n  line-height: 1.2;\n}\n\
[data-blocks-sidebar-07-chevron] {\n  margin-inline-start: auto;\n}\n\
[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-blocks-sidebar-07-label] {\n  position: absolute;\n  width: 1px;\n  height: 1px;\n  padding: 0;\n  margin: -1px;\n  overflow: hidden;\n  clip: rect(0, 0, 0, 0);\n  white-space: nowrap;\n  overflow-wrap: normal;\n  border-width: 0;\n}\n\
[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-blocks-sidebar-07-chevron] {\n  display: none;\n}\n\
[data-blocks-sidebar-07-header] {\n  display: flex;\n  align-items: center;\n  gap: 0.75rem;\n  padding: 1rem 1.5rem;\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-sidebar-07-grid] {\n  display: grid;\n  grid-template-columns: repeat(3, minmax(0, 1fr));\n  gap: 1rem;\n  padding: 0 1.5rem;\n  margin-top: 1.5rem;\n}\n\
[data-blocks-sidebar-07-placeholder] {\n  aspect-ratio: 16 / 9;\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg-muted);\n}\n\
[data-blocks-sidebar-07-placeholder-lg] {\n  min-height: 16rem;\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg-muted);\n  margin: 1.5rem;\n}\n";
