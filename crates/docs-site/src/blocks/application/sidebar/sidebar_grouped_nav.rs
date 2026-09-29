//! `sidebar-grouped-nav` block（イシュー #2940。親トラッキング #2892
//! 「Blocks アプリケーション A（phase:3）」配下）。縦長サイドバーへ
//! 「ロゴ / 検索欄 → 件数バッジ付きメインナビ → 見出し付き第 2 グループ
//! （チーム）→ 最下部固定のプロフィール行 / ユーザーメニュー」を並べた
//! 目的別パーツの合成例。
//!
//! # 使用部品
//!
//! `sidebar` / `avatar` / `icon` / `input-group` / `input` / `field`
//! （検索欄のアクセシブルラベル付け、`visually_hidden` と組み合わせる）/
//! `menu`（footer のユーザーメニュー）を合成する（[`BLOCK`] の `parts` に
//! 一致させる契約）。件数バッジは `sidebar::menu_badge` が単独で件数表示用
//! の装飾を持つため（`crate::blocks::application::app_shell::
//! app_shell_sidebar` の先例、Bugbot 指摘 PR #3324 参照）、汎用 `badge`
//! 部品は追加で使わない。
//!
//! # 無 JS のため 2 インスタンスを静的に並記する
//!
//! - インスタンス A: ロゴ header → メインナビ（見出しなし、件数バッジ・
//!   現在地強調あり）→ 見出し付き `Teams` グループ → ユーザーメニュー
//!   footer（`sidebar_07::user_menu` と同型）
//! - インスタンス B: 検索欄 header → 見出し付き `Workspace` グループ →
//!   見出し付き `Teams` グループ → プロフィール行 footer（メニューでは
//!   なく `menu_button` 1 個のみ、「プロフィール行」パターンの実演）
//!
//! 配色違いの集約元はテーマ面色（`--fandhe-color-bg-muted` 等）だけで
//! 吸収できるため Demo を増やさない（`docs/design/
//! docs-site-blocks-section.md` §2 の設計方針、詳細は後述の「原案差分
//! メモ」節）。id は `suffix`（`"logo"`/`"search"`）で一意化する
//! （`sidebar_07` と同型、`demo_output_has_no_dangling_aria_references_
//! or_duplicate_ids` 契約）。
//!
//! # `<form>` を使わない・全データが架空
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。検索欄は `<input type="search">` のみで送信先を持たない。
//! 人名・チーム名・役職・メールアドレスはすべて架空のものであり、実企業
//! 名・実在人物・実クレデンシャル・PII を含まない。ナビ項目はすべて
//! `href: None`（`<button type="button">`）とし、`href="#"` の死リンクを
//! 出さない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `sidebar` の全パーツ・`avatar::root`・`menu::root`・`field::root`・
//! `input_group::root` は呼び出し側 `attrs` の `class` を
//! `drop_class_attr` により黙って除去する契約を持つ（`crate::blocks::mod`
//! モジュール doc「CSS フック」節参照）。これらへの Demo 固有 CSS フックは
//! `data-blocks-sidebar-grouped-nav-*` 属性で渡す。
//!
//! # アイコンは自作の単純幾何図形（著作物を複製しない）
//!
//! lucide 等の実アイコンセット由来の path データは使わず、`sidebar_03`/
//! `sidebar_07` と同程度の単純な矩形図形を自作する。

use crate::blocks::dummy_assets::{COMPANY_NAMES, JOB_TITLES, PERSON_NAMES};
use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::sidebar;
use fandhe_frontend_pre_styled_ui::sidebar::{
    Sidebar, SidebarMenuButtonProps, SidebarMenuButtonSize, SidebarProps, SidebarState,
};
use fandhe_frontend_pre_styled_ui::{visually_hidden, Size};

/// 自作の単純な矩形アイコン（`d` は呼び出し側が座標を選ぶ、モジュール doc
/// 「アイコンは自作の単純幾何図形」参照）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// header のロゴ行（`sidebar_03::brand_header` と同型の `size="lg"`
/// `menu_button` 1 個。角丸の濃色アイコン枠 + 架空社名）。
fn brand_header() -> Node {
    let icon_box = div(
        vec![("data-blocks-sidebar-grouped-nav-brand-icon", "")],
        vec![geo_icon("M4 4h16v16H4z")],
    );
    let button = sidebar::menu_button(
        &SidebarMenuButtonProps {
            href: None,
            size: SidebarMenuButtonSize::Lg,
            ..Default::default()
        },
        Some(icon_box),
        vec![],
        vec![text(COMPANY_NAMES[0])],
    );
    sidebar::header(
        vec![],
        vec![sidebar::menu(
            vec![],
            vec![sidebar::menu_item(vec![], vec![button])],
        )],
    )
}

/// header の検索欄（`navbar_with_search::search_group` と同型。可視ラベル
/// の代わりに `visually_hidden` + `<label for>`、`<form>` は使わない）。
fn search_header(suffix: &str) -> Node {
    let field_id = format!("blocks-sidebar-grouped-nav-query-{suffix}");
    let query_field = FieldProps {
        id: &field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    let search_group = field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &query_field,
        vec![],
        vec![
            visually_hidden::root(
                vec![],
                vec![field::label(&query_field, vec![], vec![text("検索")])],
            ),
            input_group::root(
                &group_props,
                vec![],
                vec![
                    input_group::addon(
                        InputGroupAlign::InlineStart,
                        &group_props,
                        vec![],
                        vec![geo_icon("M4 10a6 6 0 1 1 10.39 4.03l3.29 3.29-1.41 1.41-3.29-3.29A6 6 0 0 1 4 10z")],
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
    );
    sidebar::header(
        vec![("data-blocks-sidebar-grouped-nav-search-header", "")],
        vec![search_group],
    )
}

/// メインナビ 1 行（icon + ラベル + 任意の未読件数バッジ）。
/// `sidebar::menu_badge` は `menu_item` 直下で `menu_button` の兄弟として
/// 絶対配置される組み込みパーツで、ラベルと重ならず折り返しもしない
/// （`app_shell_sidebar::nav_item` と同型、モジュール doc「使用部品」節
/// 参照）。
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
        current_page_label(label, active),
    );
    let mut children = vec![button];
    if let Some(count) = count {
        children.push(sidebar::menu_badge(vec![], vec![text(count)]));
    }
    sidebar::menu_item(vec![], children)
}

/// メインナビ群（`label` が `Some` なら見出し付きグループとして組む。
/// インスタンス A は見出しなし、インスタンス B は `Workspace` 見出し付き）。
///
/// アイコンの `path` はすべて `z` で閉じた矩形の組み合わせにする
/// （Cursor Bugbot 指摘 threadId PRRT_kwDOTarxgc6m-xIf: `geo_icon` の
/// `icon()` は `<svg fill="currentColor">` のみで `stroke` を持たない
/// ため、`M4 6h16` のような閉じていない直線パスは面積 0 で描画されない。
/// 「開いた直線の集合」ではなく必ず閉じた矩形の集合として設計する）。
fn main_group(suffix: &str, label: Option<&'static str>, active_label: &'static str) -> Node {
    let items = vec![
        nav_item(
            "M4 4h16v16H4z",
            "Dashboard",
            active_label == "Dashboard",
            None,
        ),
        nav_item(
            "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
            "Projects",
            active_label == "Projects",
            None,
        ),
        nav_item(
            "M4 5h16v3H4zM4 11h16v3H4zM4 17h10v3H4z",
            "Inbox",
            active_label == "Inbox",
            Some("12"),
        ),
        nav_item(
            "M5 6h14v14H5zM4 8h16v2H4zM6 3h2v4H6zM16 3h2v4H16z",
            "Calendar",
            active_label == "Calendar",
            None,
        ),
        nav_item(
            "M4 15h4v5H4zM10 10h4v10H10zM16 5h4v15H16z",
            "Reports",
            active_label == "Reports",
            None,
        ),
    ];
    let menu = sidebar::menu(vec![], items);
    match label {
        Some(label) => {
            let label_id = format!("blocks-sidebar-grouped-nav-main-label-{suffix}");
            sidebar::group(
                Some(label_id.as_str()),
                vec![],
                vec![
                    sidebar::group_label(Some(label_id.as_str()), vec![], vec![text(label)]),
                    sidebar::group_content(vec![], vec![menu]),
                ],
            )
        }
        None => sidebar::group(
            None,
            vec![],
            vec![sidebar::group_content(vec![], vec![menu])],
        ),
    }
}

/// `Teams` グループ 1 行（頭文字 1 文字の小さな枠 + チーム名）。
fn team_item(initial: &'static str, label: &'static str, active: bool) -> Node {
    let initial_box = span(
        vec![("data-blocks-sidebar-grouped-nav-initial", "")],
        vec![text(initial)],
    );
    sidebar::menu_item(
        vec![],
        vec![sidebar::menu_button(
            &SidebarMenuButtonProps {
                href: None,
                active,
                ..Default::default()
            },
            Some(initial_box),
            vec![],
            current_page_label(label, active),
        )],
    )
}

/// `nav_item`/`team_item` 共通のラベル組み立て。`sidebar::menu_button` は
/// `href: None`（`<button>`）のとき `aria-current` を付与しない
/// （`fandhe_frontend_headless_ui::sidebar::menu_button` の契約、モジュール
/// doc「`<form>` を使わない・全データが架空」節参照。ナビ項目はすべて
/// `href="#"` の死リンクを避けるため `href: None` に固定しており、リンク化
/// では解決できない）。本 Demo は無 JS で実ページ遷移を持たないため、
/// `active` のときのみ visually-hidden な "(current)" をラベル末尾へ加え、
/// 現在地の強調を視覚だけでなく支援技術（スクリーンリーダー）にも伝える
/// （codex P2 指摘 threadId PRRT_kwDOTarxgc6m_DYn 対応）。
fn current_page_label(label: &'static str, active: bool) -> Vec<Node> {
    let mut children = vec![text(label)];
    if active {
        children.push(visually_hidden::root(vec![], vec![text(" (current)")]));
    }
    children
}

/// `Teams` グループ（見出し付き、3 チーム）。`active_label` に一致する
/// 行のみ現在地として強調する。
fn teams_group(suffix: &str, active_label: Option<&'static str>) -> Node {
    let label_id = format!("blocks-sidebar-grouped-nav-teams-label-{suffix}");
    let items = vec![
        team_item("P", "Platform", active_label == Some("Platform")),
        team_item("G", "Growth", active_label == Some("Growth")),
        team_item(
            "D",
            "Design Systems",
            active_label == Some("Design Systems"),
        ),
    ];
    sidebar::group(
        Some(label_id.as_str()),
        vec![],
        vec![
            sidebar::group_label(Some(label_id.as_str()), vec![], vec![text("Teams")]),
            sidebar::group_content(vec![], vec![sidebar::menu(vec![], items)]),
        ],
    )
}

/// footer のユーザーメニュー行（`sidebar_07::user_menu` と同型。avatar
/// fallback + 名前 + メール + chevron を内包した閉じた `menu`）。
fn user_menu_footer(suffix: &str) -> Node {
    let content_id = format!("blocks-sidebar-grouped-nav-user-menu-{suffix}");
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id.as_str()),
        vec![
            ("aria-label", "Open user menu"),
            ("data-blocks-sidebar-grouped-nav-user-trigger", ""),
        ],
        vec![
            avatar::root(
                &AvatarProps::default(),
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text("HF")],
                )],
            ),
            span(
                vec![("data-blocks-sidebar-grouped-nav-label", "")],
                vec![
                    span(vec![], vec![text(PERSON_NAMES[0])]),
                    span(vec![], vec![text("haruto@example.com")]),
                ],
            ),
            span(
                vec![("data-blocks-sidebar-grouped-nav-chevron", "")],
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
            menu::item("account", false, false, vec![], vec![text("Account")]),
            menu::item("billing", false, false, vec![], vec![text("Billing")]),
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

/// footer のプロフィール行（メニューではなく `menu_button` 1 個のみ。
/// avatar fallback + 名前 + 役職の 2 行ラベル。「プロフィール行」パターン
/// の実演、モジュール doc「無 JS のため 2 インスタンスを静的に並記する」
/// 節参照）。名前・役職は `user_menu_footer` と同じ
/// `data-blocks-sidebar-grouped-nav-label`（`flex-direction: column`）で
/// 包み縦積みにする（codex P1・Cursor Bugbot 指摘 threadId
/// PRRT_kwDOTarxgc6m-xIk: 2 つの `span` を横並びのまま `menu_button` の
/// `nowrap` ラベル領域に直接渡すと 1 行に収まってしまうため）。
fn profile_footer() -> Node {
    let avatar_box = avatar::root(
        &AvatarProps::default(),
        vec![],
        vec![avatar::fallback(
            ImageStatus::Error,
            vec![],
            vec![text("EV")],
        )],
    );
    let label = span(
        vec![("data-blocks-sidebar-grouped-nav-label", "")],
        vec![
            span(vec![], vec![text(PERSON_NAMES[1])]),
            span(vec![], vec![text(JOB_TITLES[0])]),
        ],
    );
    let button = sidebar::menu_button(
        &SidebarMenuButtonProps {
            href: None,
            size: SidebarMenuButtonSize::Lg,
            ..Default::default()
        },
        Some(avatar_box),
        vec![("data-blocks-sidebar-grouped-nav-profile", "")],
        vec![label],
    );
    sidebar::footer(
        vec![],
        vec![sidebar::menu(
            vec![],
            vec![sidebar::menu_item(vec![], vec![button])],
        )],
    )
}

/// 左サイドバーの `root`（`provider` の直接の子として置く、`sidebar_03::
/// app_sidebar` と同型）。`nav_label`/`rail_label` はインスタンスごとに
/// 一意な文言を渡す（`demo_output_has_no_dangling_aria_references_or_
/// duplicate_ids` 契約・Cursor Bugbot 指摘 threadId PRRT_kwDOTarxgc6m-xIq
/// 参照。2 つの `nav` landmark・rail トグルが同一アクセシブル名になると
/// スクリーンリーダーで区別できないため、インスタンスごとに区別できる
/// 文言にする）。
fn app_sidebar(
    state: &Sidebar,
    props: &SidebarProps,
    // `(root_id, nav_label, rail_label)`。clippy `too_many_arguments`
    // 回避のため 1 引数へまとめる。
    ids: (&str, &str, &str),
    header: Node,
    content: Node,
    footer: Node,
) -> Node {
    let (root_id, nav_label, rail_label) = ids;
    sidebar::root(
        state,
        props,
        nav_label,
        Some(root_id),
        vec![],
        vec![
            header,
            content,
            footer,
            sidebar::rail(state, rail_label, vec![], vec![]),
        ],
    )
}

/// inset 側の最小プレースホルダー（見出し行 + プレースホルダー 2 枚。
/// サイドバー本体が主題のため簡素にする、モジュール doc参照）。
fn inset_area() -> Node {
    let heading = div(
        vec![("data-blocks-sidebar-grouped-nav-inset-heading", "")],
        vec![text("Dashboard")],
    );
    let grid = div(
        vec![("data-blocks-sidebar-grouped-nav-grid", "")],
        vec![
            div(
                vec![("data-blocks-sidebar-grouped-nav-placeholder", "")],
                vec![],
            ),
            div(
                vec![("data-blocks-sidebar-grouped-nav-placeholder", "")],
                vec![],
            ),
        ],
    );
    sidebar::inset(vec![], vec![heading, grid])
}

/// `sidebar-grouped-nav` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「無 JS のため 2 インスタンスを静的に並記
/// する」参照）。
pub fn demo() -> Node {
    // インスタンス A: ロゴ header・見出しなしメインナビ・見出し付き
    // `Teams` グループ・ユーザーメニュー footer。
    let a_state = Sidebar::new(SidebarState::Expanded);
    let a_props = SidebarProps::default();
    let a_root_id = "blocks-sidebar-grouped-nav-root-logo";
    let a_content = sidebar::content(
        vec![],
        vec![
            main_group("logo", None, "Projects"),
            teams_group("logo", None),
        ],
    );
    let instance_a = sidebar::provider(
        &a_state,
        &a_props,
        vec![("data-blocks-sidebar-grouped-nav-instance", "logo")],
        vec![
            app_sidebar(
                &a_state,
                &a_props,
                (a_root_id, "Main navigation", "Toggle main sidebar rail"),
                brand_header(),
                a_content,
                user_menu_footer("logo"),
            ),
            inset_area(),
        ],
    );

    // インスタンス B: 検索欄 header・見出し付き `Workspace` グループ・
    // 見出し付き `Teams` グループ・プロフィール行 footer。
    let b_state = Sidebar::new(SidebarState::Expanded);
    let b_props = SidebarProps::default();
    let b_root_id = "blocks-sidebar-grouped-nav-root-search";
    let b_content = sidebar::content(
        vec![],
        vec![
            main_group("search", Some("Workspace"), "Dashboard"),
            teams_group("search", Some("Growth")),
        ],
    );
    let instance_b = sidebar::provider(
        &b_state,
        &b_props,
        vec![("data-blocks-sidebar-grouped-nav-instance", "search")],
        vec![
            app_sidebar(
                &b_state,
                &b_props,
                (
                    b_root_id,
                    "Workspace navigation",
                    "Toggle workspace sidebar rail",
                ),
                search_header("search"),
                b_content,
                profile_footer(),
            ),
            inset_area(),
        ],
    );

    div(
        vec![("data-blocks-sidebar-grouped-nav-stack", "")],
        vec![instance_a, instance_b],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/sidebar-grouped-nav/",
    title: "sidebar-grouped-nav",
    category: BlockCategory::Sidebar,
    rust_source: "crates/docs-site/src/blocks/application/sidebar/sidebar_grouped_nav.rs",
    demo_class: "blocks-sidebar-grouped-nav",
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
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// [`super::mod@self`] の `stylesheet()` が `push_css` する block 固有の
/// レイアウト CSS（`docs/design/docs-site-blocks-section.md` §10 追記節に
/// 従い、並列進行する他 block との `mod.rs::LAYOUT_CSS` 追記衝突を避け
/// 本モジュール側の定数へ分離する）。生の色・時間値は書かず `--fandhe-*`
/// トークンのみを使う。
const LAYOUT_CSS: &str = "\
.blocks-demo.blocks-sidebar-grouped-nav {\n  padding: 0;\n  overflow-x: auto;\n}\n\
[data-blocks-sidebar-grouped-nav-stack] {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: 1rem;\n}\n\
[data-blocks-sidebar-grouped-nav-instance][data-scope=\"sidebar\"][data-part=\"provider\"] {\n  min-height: 36rem;\n  height: auto;\n  min-width: 20rem;\n}\n\
[data-blocks-sidebar-grouped-nav-brand-icon] {\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  width: 2rem;\n  height: 2rem;\n  flex-shrink: 0;\n  border-radius: var(--fandhe-radius-md);\n  background: var(--fandhe-color-fg);\n  color: var(--fandhe-color-bg);\n}\n\
[data-blocks-sidebar-grouped-nav-search-header] {\n  padding: 0.75rem;\n}\n\
[data-blocks-sidebar-grouped-nav-initial] {\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  width: 1.25rem;\n  height: 1.25rem;\n  flex-shrink: 0;\n  border-radius: var(--fandhe-radius-sm);\n  background: var(--fandhe-color-bg-muted);\n  font-size: var(--fandhe-font-font-size-xs, 0.75rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n}\n\
[data-blocks-sidebar-grouped-nav-user-trigger] {\n  display: flex;\n  align-items: center;\n  gap: 0.5rem;\n  width: 100%;\n  text-align: start;\n}\n\
[data-blocks-sidebar-grouped-nav-profile] {\n  width: 100%;\n  text-align: start;\n}\n\
[data-blocks-sidebar-grouped-nav-label] {\n  display: flex;\n  flex-direction: column;\n  overflow: hidden;\n  line-height: 1.2;\n}\n\
[data-blocks-sidebar-grouped-nav-chevron] {\n  margin-inline-start: auto;\n}\n\
[data-blocks-sidebar-grouped-nav-inset-heading] {\n  padding: 1rem 1.5rem;\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-sidebar-grouped-nav-grid] {\n  display: grid;\n  gap: 1rem;\n  padding: 1.5rem;\n}\n\
[data-blocks-sidebar-grouped-nav-placeholder] {\n  min-height: 6rem;\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg-muted);\n}\n";
