//! `settings-page-sidebar` block（イシュー #3004、親 #3003。Application /
//! Settings カテゴリ）。アイコン幅へ折りたたみ可能な左サイドバー + 右上
//! パンくずのヘッダー + タブと設定カード（スイッチ行）を持つ設定ページの
//! **骨格と主要領域**を実装する（規模の大きい合成のため、前半である本
//! イシューが骨格・主要領域を、後半 #3005 が残りの領域（サイドバー footer
//! のユーザー行 + `menu`、追加カード、残りタブの内容、状態表示・原稿の
//! 仕上げ）を担う分担、`docs/design/docs-site-blocks-section.md` §19 参照）。
//!
//! # 使用部品
//!
//! `sidebar` / `breadcrumb` / `separator` / `tabs` / `card` / `switch` /
//! `button` / `icon` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約。`menu` は #3005 でサイドバー footer 追加時に加わる）。
//!
//! # 無 JS のため展開・折りたたみの 2 状態を静的に並置する
//!
//! `sidebar_07` と同じ設計判断（同モジュール doc「無 JS のため 2 状態を
//! 静的に並置する」節参照）に従い、expanded インスタンスと collapsed
//! （icon）インスタンスを縦に並べて静的に掲示する。両インスタンスとも
//! `collapsible: SidebarCollapsible::Icon` を明示する（既定は `Offcanvas`
//! のため expanded 側にも明示が必要）。
//!
//! # 狭幅ではサイドバーを隠す（`sidebar_07` との差分）
//!
//! `sidebar_07` は狭幅を `overflow-x: auto` + `min-width: 56rem` の横
//! スクロールで吸収するが、本 block はイシュー要件「狭幅ではサイドバーを
//! 隠す」に従い、`@container`（デモ枠自体の幅基準、`settings_billing_overview`
//! 等と同型）でコンテナ幅 48rem 未満のとき `provider` の直接の子である
//! `root`（サイドバー本体）を `display: none` にし、`inset` 側を全幅へ
//! 広げる。`display: none` にした `root` を `sidebar::trigger` の
//! `aria-controls` が指すが、DOM 上は実在するため参照切れにはならない
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` 契約と
//! 同型の検証を本ファイルの単体テストでも固定する）。
//!
//! # `switch` を静的固定表示にする理由
//!
//! docs サイトは無 JS のため、設定カードの `switch` 3 行はいずれも初期
//! 状態を固定した静的表示とする（[`fandhe_frontend_pre_styled_ui::switch::
//! SwitchProps::disabled`] を `true` にする、`settings_integrations_grid`
//! `card_grouped_by_category` と同型の判断）。`hidden_input` の
//! `aria-label` へ行ラベル + 状態（例:「公開プロフィール: オン」）を含め、
//! 支援技術が複数行の状態を区別できるようにする。
//!
//! # `<form>` を使わない・認証処理を持たない・全データが架空
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。ワークスペース名・ナビ項目・スイッチのラベル・説明文は
//! すべて架空のものであり、実企業名・実在人物・実クレデンシャル・PII を
//! 含まない。ナビの折りたたみ・タブ切替・スイッチ・保存ボタンはいずれも
//! 静的な初期状態を表示するのみで、選択・送信・永続化・認証処理は行わない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `sidebar` の全パーツ・`breadcrumb::root`・`separator::separator`・
//! `card::root`・`switch::root`・`button::button` は呼び出し側 `attrs` の
//! `class` を `drop_class_attr` により黙って除去する契約を持つ
//! （`crate::blocks::mod` モジュール doc「CSS フック」節参照）。これらへの
//! Demo 固有 CSS フックは `data-blocks-settings-page-sidebar-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。素の `div`/`p`・
//! `card::body`/`description` には `class` がそのまま効くため、レイアウト用
//! ラッパーはクラスセレクタを使う（`settings_item_cards` と同型の使い分け）。
//!
//! # アイコンは自作の単純幾何図形（著作物を複製しない）
//!
//! `sidebar_07::geo_icon` / `settings_item_cards::geo_icon` と同型の自作
//! 単純図形を使う。lucide 等の実アイコンセット由来の path データは使わない。
//!
//! # ダミー素材について
//!
//! ワークスペース名（「Nimbus ワークスペース」）・ナビ項目名・スイッチの
//! ラベル・説明文はすべて本ファイル内の架空値である。実在の企業・サービス
//! 名・人物・PII は含まない。
//!
//! # 参照 ID R0659（`_/blocks-intake/` 不在のため ID のみ記載）
//!
//! 集約元の対応表 ID は R0659。`_/blocks-intake/` の対応ファイルは本
//! worktree に存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`settings_integrations_list`・`settings_billing_overview` と同じ扱い）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, p, text, Node};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps, SeparatorVariant};
use fandhe_frontend_pre_styled_ui::sidebar;
use fandhe_frontend_pre_styled_ui::sidebar::{
    Sidebar, SidebarCollapsible, SidebarMenuButtonProps, SidebarProps, SidebarState,
};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::tabs::{self, ActivationMode, TabItem, TabsProps, TabsVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Orientation, Size};

/// 自作の単純な矩形・幾何アイコン（`sidebar_07::geo_icon` と同型。lucide
/// 等の著作物を複製しないためのモジュール doc「アイコンは自作」節参照）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// サイドバー header（ワークスペース名の静的表示。実際のワークスペース
/// 切替は持たず、`sidebar::menu_button` 1 行のみの表示）。
fn workspace_header() -> Node {
    sidebar::header(
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
                    Some(geo_icon("M4 4h16v16H4z")),
                    vec![("aria-label", "Nimbus ワークスペース")],
                    vec![text("Nimbus ワークスペース")],
                )],
            )],
        )],
    )
}

/// 設定ナビ 1 件分のデータ（アイコン・ラベル・現在項目か）。
struct NavItem {
    icon_path: &'static str,
    label: &'static str,
    active: bool,
}

/// 設定ナビ 5 件（現在項目は「一般」）。
const NAV_ITEMS: &[NavItem] = &[
    NavItem {
        icon_path: "M12 2a10 10 0 1 0 .001 20.001A10 10 0 0 0 12 2z",
        label: "一般",
        active: true,
    },
    NavItem {
        icon_path: "M4 20c0-4.5 3.5-7 8-7s8 2.5 8 7 M12 12a4 4 0 100-8 4 4 0 000 8z",
        label: "メンバー",
        active: false,
    },
    NavItem {
        icon_path: "M4 4h16v12H7l-3 3z",
        label: "通知",
        active: false,
    },
    NavItem {
        icon_path: "M4 6h16v12H4z M4 10h16",
        label: "請求",
        active: false,
    },
    NavItem {
        icon_path: "M12 3l7 3v6c0 4.5-3 7.5-7 9-4-1.5-7-4.5-7-9V6z",
        label: "セキュリティ",
        active: false,
    },
];

/// 設定ナビ（`sidebar::group` 1 グループ + `menu_button` 5 件、現在項目は
/// `active` を立てる）。
fn settings_nav(suffix: &str) -> Node {
    let label_id = format!("blocks-settings-page-sidebar-nav-label-{suffix}");
    let items = NAV_ITEMS
        .iter()
        .map(|item| {
            sidebar::menu_item(
                vec![],
                vec![sidebar::menu_button(
                    &SidebarMenuButtonProps {
                        href: None,
                        active: item.active,
                        ..Default::default()
                    },
                    Some(geo_icon(item.icon_path)),
                    vec![],
                    vec![text(item.label)],
                )],
            )
        })
        .collect();
    sidebar::group(
        Some(&label_id),
        vec![],
        vec![
            sidebar::group_label(Some(&label_id), vec![], vec![text("設定")]),
            sidebar::group_content(vec![], vec![sidebar::menu(vec![], items)]),
        ],
    )
}

/// 左サイドバーの `root`（`provider` の直接の子として置く契約、
/// `sidebar_07::app_sidebar` と同型）。
fn settings_sidebar(state: &Sidebar, props: &SidebarProps, root_id: &str, suffix: &str) -> Node {
    sidebar::root(
        state,
        props,
        "Settings navigation",
        Some(root_id),
        vec![],
        vec![
            workspace_header(),
            sidebar::content(vec![], vec![settings_nav(suffix)]),
            sidebar::rail(state, "Toggle sidebar rail", vec![], vec![]),
        ],
    )
}

/// inset 側ヘッダー（トリガー + 縦 separator + breadcrumb 2 階層）。
fn inset_header(state: &Sidebar, root_id: &str) -> Node {
    div(
        vec![("data-blocks-settings-page-sidebar-header", "")],
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
                                vec![text("Nimbus ワークスペース")],
                            )],
                        ),
                        breadcrumb::separator(vec![], vec![text("/")]),
                        breadcrumb::item(
                            vec![],
                            vec![breadcrumb::current_link(vec![], vec![text("設定")])],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// スイッチ行 1 件分のデータ（キー・ラベル・説明・初期状態）。
struct SwitchRow {
    key: &'static str,
    label: &'static str,
    description: &'static str,
    checked: bool,
}

/// 設定カードのスイッチ 3 行（すべて `disabled` の静的固定表示、モジュール
/// doc「`switch` を静的固定表示にする理由」節参照）。
const SWITCH_ROWS: &[SwitchRow] = &[
    SwitchRow {
        key: "public-profile",
        label: "公開プロフィール",
        description: "他のメンバーにプロフィールを公開します。",
        checked: true,
    },
    SwitchRow {
        key: "weekly-digest",
        label: "週次ダイジェスト",
        description: "毎週のアクティビティ概要をメールで受け取ります。",
        checked: false,
    },
    SwitchRow {
        key: "require-2fa",
        label: "二段階認証を必須にする",
        description: "ワークスペースの全メンバーに二段階認証を要求します。",
        checked: true,
    },
];

/// スイッチ行 1 件（`suffix` で expanded/collapsed インスタンス間の id
/// 衝突を避ける、`sidebar_07` と同型）。
fn switch_row(row: &SwitchRow, suffix: &str) -> Node {
    let switch_props = SwitchProps {
        disabled: true,
        ..SwitchProps::default()
    };
    let status_label = if row.checked { "オン" } else { "オフ" };
    let control = switch::root(
        Size::Sm,
        ColorPalette::Accent,
        row.checked,
        &switch_props,
        vec![],
        vec![
            switch::hidden_input(
                &format!("blocks-settings-page-sidebar-{}-{suffix}", row.key),
                row.key,
                row.checked,
                &switch_props,
                vec![("aria-label", &format!("{}: {status_label}", row.label))],
            ),
            switch::control(
                row.checked,
                &switch_props,
                vec![],
                vec![switch::thumb(row.checked, &switch_props, vec![], vec![])],
            ),
        ],
    );
    div(
        vec![("class", "blocks-settings-page-sidebar-row")],
        vec![
            div(
                vec![("class", "blocks-settings-page-sidebar-row-text")],
                vec![
                    p(vec![], vec![text(row.label)]),
                    p(
                        vec![("class", "blocks-settings-page-sidebar-row-description")],
                        vec![text(row.description)],
                    ),
                ],
            ),
            control,
        ],
    )
}

/// タブ「全般」内の設定カード（見出し + 説明 + スイッチ行 3 件 + フッターの
/// 保存ボタン）。
fn general_settings_card(suffix: &str) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-page-sidebar-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("一般設定")]),
                    card::description(
                        vec![],
                        vec![text("ワークスペース全体に適用される既定値です。")],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![div(
                    vec![("class", "blocks-settings-page-sidebar-rows")],
                    SWITCH_ROWS
                        .iter()
                        .map(|row| switch_row(row, suffix))
                        .collect(),
                )],
            ),
            card::footer(
                vec![],
                vec![button(
                    &ButtonProps {
                        variant: ButtonVariant::Solid,
                        ..ButtonProps::default()
                    },
                    vec![("data-blocks-settings-page-sidebar-save", "")],
                    vec![text("変更を保存")],
                )],
            ),
        ],
    )
}

/// タブ「メンバー」「通知」の暫定内容（#3005 で実データへ差し替え予定の
/// 静的なメモ、`dashboard_01::short_note` と同型）。
fn placeholder_tab_note(message: &'static str) -> Node {
    div(vec![], vec![text(message)])
}

/// inset 本体（タブ 3 件、先頭タブのみ内容あり。残りタブの内容は #3005）。
fn inset_body(suffix: &str) -> Node {
    let props = TabsProps {
        id: &format!("blocks-settings-page-sidebar-tabs-{suffix}"),
        selected: "general",
        orientation: Orientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };
    let items = vec![
        TabItem {
            value: "general",
            trigger: vec![text("全般")],
            content: vec![general_settings_card(suffix)],
            disabled: false,
        },
        TabItem {
            value: "members",
            trigger: vec![text("メンバー")],
            content: vec![placeholder_tab_note(
                "メンバー管理は後続で追加する静的な合成例です。",
            )],
            disabled: false,
        },
        TabItem {
            value: "notifications",
            trigger: vec![text("通知")],
            content: vec![placeholder_tab_note(
                "通知設定は後続で追加する静的な合成例です。",
            )],
            disabled: false,
        },
    ];
    div(
        vec![("class", "blocks-settings-page-sidebar-body")],
        vec![tabs::tabs(
            TabsVariant::Enclosed,
            Size::Md,
            ColorPalette::Accent,
            &props,
            items,
        )],
    )
}

/// inset 側（ヘッダー + タブ・設定カード本体）。
fn inset_area(state: &Sidebar, root_id: &str, suffix: &str) -> Node {
    sidebar::inset(
        vec![],
        vec![inset_header(state, root_id), inset_body(suffix)],
    )
}

/// expanded/collapsed いずれか 1 インスタンス分（`provider > root, inset`
/// の骨格、`sidebar_07::demo` と同型）。
fn instance(sidebar_state: SidebarState, suffix: &str) -> Node {
    let state = Sidebar::new(sidebar_state);
    let props = SidebarProps {
        collapsible: SidebarCollapsible::Icon,
        ..SidebarProps::default()
    };
    let root_id = format!("blocks-settings-page-sidebar-root-{suffix}");
    sidebar::provider(
        &state,
        &props,
        vec![("data-blocks-settings-page-sidebar-instance", "")],
        vec![
            settings_sidebar(&state, &props, &root_id, suffix),
            inset_area(&state, &root_id, suffix),
        ],
    )
}

/// `settings-page-sidebar` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。expanded/collapsed の 2 インスタンスを
/// `data-blocks-settings-page-sidebar-stack` の下へ縦に並べる（モジュール
/// doc「無 JS のため展開・折りたたみの 2 状態を静的に並置する」参照）。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-settings-page-sidebar-stack", "")],
        vec![
            p(
                vec![("data-blocks-settings-page-sidebar-caption", "")],
                vec![text("展開")],
            ),
            instance(SidebarState::Expanded, "expanded"),
            p(
                vec![("data-blocks-settings-page-sidebar-caption", "")],
                vec![text("折りたたみ（アイコン）")],
            ),
            instance(SidebarState::Collapsed, "collapsed"),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-page-sidebar/",
    title: "settings-page-sidebar",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_page_sidebar.rs",
    demo_class: "blocks-settings-page-sidebar",
    parts: &[
        Part {
            label: "Sidebar",
            path: "/themes/sidebar/",
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
            label: "Tabs",
            path: "/themes/tabs/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Switch",
            path: "/themes/switch/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
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
/// 狭幅（コンテナ幅 48rem 未満）では `provider` の直接の子である
/// `root`（サイドバー本体）を `display: none` にし `inset` を全幅へ広げる
/// （モジュール doc「狭幅ではサイドバーを隠す」節参照。`sidebar_07` の
/// `overflow-x: auto` + `min-width: 56rem` の横スクロール方式とは異なる）。
const LAYOUT_CSS: &str = "\
.blocks-demo.blocks-settings-page-sidebar {\n  padding: 0;\n}\n\
[data-blocks-settings-page-sidebar-stack] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  container-type: inline-size;\n  container-name: blocks-settings-page-sidebar;\n}\n\
[data-blocks-settings-page-sidebar-caption] {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-settings-page-sidebar-instance][data-scope=\"sidebar\"][data-part=\"provider\"] {\n  min-height: 28rem;\n  height: auto;\n}\n\
[data-blocks-settings-page-sidebar-header] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-settings-page-sidebar-body {\n  padding: var(--fandhe-space-4);\n}\n\
.blocks-settings-page-sidebar-rows {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-settings-page-sidebar-rows .blocks-settings-page-sidebar-row {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-settings-page-sidebar-row-text {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-settings-page-sidebar-row-text p {\n  margin: 0;\n}\n\
.blocks-settings-page-sidebar-row-description {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
@container blocks-settings-page-sidebar (max-width: 48rem) {\n  \
[data-blocks-settings-page-sidebar-instance] > [data-scope=\"sidebar\"][data-part=\"root\"] {\n    display: none;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"sidebar\"",
            "data-scope=\"breadcrumb\"",
            "data-scope=\"separator\"",
            "data-scope=\"tabs\"",
            "data-scope=\"card\"",
            "data-scope=\"switch\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn demo_has_two_sidebar_instances_with_distinct_ids() {
        let html = demo_html();
        assert_eq!(
            html.matches("id=\"blocks-settings-page-sidebar-root-expanded\"")
                .count(),
            1
        );
        assert_eq!(
            html.matches("id=\"blocks-settings-page-sidebar-root-collapsed\"")
                .count(),
            1
        );
        assert!(html.contains("data-state=\"collapsed\""));
        assert!(html.contains("id=\"blocks-settings-page-sidebar-tabs-expanded\""));
        assert!(html.contains("id=\"blocks-settings-page-sidebar-tabs-collapsed\""));
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn buttons_use_type_button_only() {
        let html = demo_html();
        let button_count = html.matches("<button").count();
        assert!(button_count > 0);
        assert_eq!(html.matches("type=\"button\"").count(), button_count);
    }

    #[test]
    fn switches_are_static_and_labelled() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-scope=\"switch\" data-part=\"hidden-input\"")
                .count(),
            6,
            "3 rows x 2 instances should render 6 switches"
        );
        for label in [
            "公開プロフィール: オン",
            "週次ダイジェスト: オフ",
            "二段階認証を必須にする: オン",
        ] {
            let aria = format!("aria-label=\"{label}\"");
            assert_eq!(
                html.matches(&aria).count(),
                2,
                "missing {aria} in both instances"
            );
        }
        let hidden_input_positions: Vec<_> = html
            .match_indices("data-scope=\"switch\" data-part=\"hidden-input\"")
            .collect();
        for (start, _) in hidden_input_positions {
            let window_end = (start + 400).min(html.len());
            assert!(
                html[start..window_end].contains("disabled"),
                "switch hidden-input should be disabled for static display"
            );
        }
    }

    #[test]
    fn layout_css_hides_sidebar_root_on_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("@container blocks-settings-page-sidebar (max-width: 48rem)"));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-settings-page-sidebar-instance] > [data-scope=\"sidebar\"][data-part=\"root\"] {\n    display: none;"
        ));
        assert!(!LAYOUT_CSS.contains("min-width: 56rem"));
    }

    #[test]
    fn row_selector_beats_docs_content_specificity() {
        assert!(LAYOUT_CSS
            .contains(".blocks-settings-page-sidebar-rows .blocks-settings-page-sidebar-row {"));
    }
}
