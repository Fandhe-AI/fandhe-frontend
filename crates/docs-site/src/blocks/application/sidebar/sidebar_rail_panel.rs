//! `sidebar-rail-panel` block（イシュー #2941。対応表 ID R0330 のみ参照。
//! 原本ファイルはメイン worktree にも存在しないため、イシュー本文の
//! レイアウト仕様のみから設計した、`sidebar_07`/`app_shell_sidebar` と
//! 同型の合成例）。
//!
//! # 使用部品
//!
//! `sidebar`（見出し + 検索欄付きの右側パネル。`collapsible: None` で
//! 常設）/ `icon`（レール・パネルの自作幾何アイコン）/ `avatar`（レール
//! 下部のユーザーメニュー trigger）/ `heading`（パネル見出し、選択中
//! セクション名）/ `menu`（閉じたユーザーメニュー、`sidebar_07::user_menu`
//! と同型）/ `input-group`・`input`・`field`（パネル検索欄、
//! `navbar_with_search::search_group` と同型）/ `button`（レールのアイコン
//! ボタン群）を合成する（[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # レールは素の `nav`（`sidebar` scope の外側）
//!
//! アイコンレールはセクション切替の操作ではなく現在地の掲示のみを担う
//! ため、`fandhe_frontend_core::nav` + `button::icon_button` で組む
//! （`sidebar` 部品を二重に使わない）。現在選択中のセクションのみ
//! `aria-current="page"` + `data-blocks-sidebar-rail-panel-current` を
//! 付与し、[`LAYOUT_CSS`] が背景色で強調する。
//!
//! # 狭幅ではパネルのみ隠す（コンテナクエリ、`app_shell_sidebar` と同型）
//!
//! Demo 枠（`[data-blocks-sidebar-rail-panel-frame]`）へ
//! `container-type: inline-size` を設定し、`@container` 規則だけで
//! パネル（`sidebar::root`）を非表示にする。`@container` はコンテナ自身
//! ではなく子孫にのみ適用されるため、規則は祖先セレクタを重ねず
//! `[data-scope="sidebar"][data-part="root"]` を直接選択する
//! （`app_shell_sidebar` モジュール doc「常設サイドバー ⇔ 上部バーの
//! 切替はコンテナクエリで行う」節と同型の教訓）。レールは素の `nav` の
//! ため隠れず残る。
//!
//! # 2 インスタンスを静的に並記する理由（無 JS）
//!
//! docs サイトは JS ハイドレーションを行わないため、狭幅切替を実演
//! できない。`Desktop`（幅制約なし）と `Narrow (rail only)`（フレーム幅を
//! [`LAYOUT_CSS`] で固定し常にパネル非表示）の 2 インスタンスを縦に
//! 並べる（`app_shell_sidebar`/`sidebar_07` の前例と同型）。id を持つ
//! 要素はすべて suffix（`desktop`/`narrow`）で一意化する。
//!
//! # リンクは `sidebar::menu_button { href: None }`（button）で組む
//!
//! `sidebar_03`/`sidebar_07` の前例を踏襲し、パネルのセクション内リンクは
//! 実在の遷移先を持たない静的な `<button type="button">` として組む
//! （`href="#"` は使わない）。選択・送信・永続化・認証処理は行わない。
//!
//! # `class` ではなく `data-*` で CSS フックを渡す
//!
//! `sidebar`/`menu::root`/`avatar::root`/`field::root`/`input_group::root`/
//! `button::icon_button` は呼び出し側 `attrs` の `class` を
//! `drop_class_attr` により黙って除去する契約を持つ（`sidebar_07`
//! モジュール doc「CSS フックの選び方」節と同型）。本 block も統一して
//! `data-blocks-sidebar-rail-panel-*` 属性で CSS フックを渡す。
//!
//! # アイコンは自作の単純幾何図形・文言はすべて架空
//!
//! `sidebar_07::geo_icon` と同型の自作矩形アイコンのみを使う（実アイコン
//! セット由来の path データは複製しない）。セクション名・パネル項目名・
//! 人名はすべて架空のもの（人名は [`crate::blocks::dummy_assets::
//! PERSON_NAMES`]）であり、実企業名・実在人物・実クレデンシャル・PII を
//! 含まない。`crate::blocks` モジュール doc の不変条件どおり `<form>` は
//! 出力しない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, nav, p, text, ul, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::sidebar;
use fandhe_frontend_pre_styled_ui::sidebar::{
    Sidebar, SidebarCollapsible, SidebarProps, SidebarState,
};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// レール上部の主要セクション 4 件（`icon_path`・ラベル・パネル項目）。
/// 選択中セクション（インデックス [`CURRENT_SECTION`]）のみパネルへ
/// 反映する。
const SECTIONS: &[(&str, &str, &[&str])] = &[
    ("M4 4h16v16H4z", "Home", &[]),
    (
        "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
        "Projects",
        &["Overview", "Roadmap", "Milestones", "Archive"],
    ),
    ("M5 3h14v18H5z", "Inbox", &[]),
    ("M4 4h16v4H4zM4 10h16v4H4zM4 16h16v4H4z", "Reports", &[]),
];

/// 現在選択中のセクション（`SECTIONS` のインデックス）。両インスタンス
/// 共通の静的な選択状態（無 JS のため状態は動かさない）。
const CURRENT_SECTION: usize = 1;

/// 自作の単純な矩形アイコン（`sidebar_07::geo_icon` と同型。lucide 等の
/// 著作物を複製しないためのモジュール doc「アイコンは自作の単純幾何図形」
/// 節参照）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// レールの設定歯車アイコン。
fn settings_icon() -> Node {
    geo_icon("M12 2a10 10 0 1 0 .001 20.001A10 10 0 0 0 12 2z")
}

/// パネル検索欄の虫眼鏡アイコン（`footer_cta_columns::geo_icon` と同型の
/// ストローク描画。`geo_icon`〔`fill="currentColor"` 固定〕をそのまま使うと
/// 円と柄が塗り潰されて虫眼鏡に見えなくなる〔#2941 PR レビュー指摘〕ため、
/// `fill="none"` + `stroke="currentColor"` を path 個別に上書きする）。
fn search_icon() -> Node {
    icon(
        &IconProps::default(),
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

/// レール上部 1 項目（`icon_button` + 現在地の強調表示）。
fn rail_item(icon_path: &'static str, label: &'static str, current: bool) -> Node {
    let mut attrs: Vec<(&str, &str)> = vec![("data-blocks-sidebar-rail-panel-rail-item", "")];
    if current {
        attrs.push(("aria-current", "page"));
        attrs.push(("data-blocks-sidebar-rail-panel-current", ""));
    }
    li(
        vec![],
        vec![button::icon_button(
            &ButtonProps {
                variant: ButtonVariant::Ghost,
                ..ButtonProps::default()
            },
            label,
            attrs,
            vec![geo_icon(icon_path)],
        )],
    )
}

/// footer のユーザーメニュー（閉じた `menu`、`sidebar_07::user_menu` の
/// trigger 部分のみを avatar 単体で組んだ版。`suffix` で expanded/narrow
/// インスタンス間の id 衝突を避ける）。
fn user_menu(suffix: &str) -> Node {
    let content_id = format!("blocks-sidebar-rail-panel-user-menu-{suffix}");
    let name = dummy_assets::PERSON_NAMES[0];
    let initial: String = name.chars().next().map(String::from).unwrap_or_default();
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id.as_str()),
        vec![("aria-label", "Open user menu")],
        vec![avatar::root(
            &AvatarProps::default(),
            vec![],
            vec![avatar::fallback(
                ImageStatus::Error,
                vec![],
                vec![text(initial)],
            )],
        )],
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
    menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    )
}

/// アイコンレール本体（上部: 主要セクション、下部: 設定 + ユーザー
/// メニュー）。素の `nav`（モジュール doc「レールは素の `nav`」節参照）。
///
/// `aria-label` へ `instance_caption`（"Desktop"/"Narrow (rail only)"）を
/// 含め、Desktop/Narrow 両インスタンスの `nav` ランドマークを一意化する
/// （`panel` の `sidebar::root` label と同型。両インスタンスを無 JS で
/// 静的に並記するため常に両方 DOM 上に存在し、支援技術から同名で
/// 区別不能になっていた〔#2941 PR レビュー指摘〕）。
fn rail(suffix: &str, instance_caption: &str) -> Node {
    let top_items: Vec<Node> = SECTIONS
        .iter()
        .enumerate()
        .map(|(i, (icon_path, label, _))| rail_item(icon_path, label, i == CURRENT_SECTION))
        .collect();
    let aria_label = format!("Primary sections ({instance_caption})");
    nav(
        vec![
            ("aria-label", aria_label.as_str()),
            ("data-blocks-sidebar-rail-panel-rail", ""),
        ],
        vec![
            ul(
                vec![("data-blocks-sidebar-rail-panel-rail-top", "")],
                top_items,
            ),
            div(
                vec![("data-blocks-sidebar-rail-panel-rail-bottom", "")],
                vec![
                    button::icon_button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            ..ButtonProps::default()
                        },
                        "Settings",
                        vec![],
                        vec![settings_icon()],
                    ),
                    user_menu(suffix),
                ],
            ),
        ],
    )
}

/// パネルの検索欄（`navbar_with_search::search_group` と同型、可視ラベル
/// の代わりに `visually_hidden` + `<label for>`）。
fn search_group(suffix: &str) -> Node {
    let field_id = format!("blocks-sidebar-rail-panel-search-{suffix}");
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
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &query_field,
        vec![],
        vec![
            visually_hidden::root(
                vec![],
                vec![field::label(&query_field, vec![], vec![text("Search")])],
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
                        vec![("type", "search"), ("placeholder", "Search")],
                    ),
                ],
            ),
        ],
    )
}

/// パネル本体（見出し + 検索欄 + 選択中セクションのリンク一覧）。
///
/// `sidebar::root` の label（`nav`/`aside` の `aria-label` 相当）へ
/// `instance_caption` を含め、`rail` と同じ理由で一意化する（#2941 PR
/// レビュー指摘）。
fn panel(suffix: &str, root_id: &str, instance_caption: &str) -> Node {
    let (_, label, items) = SECTIONS[CURRENT_SECTION];
    let sidebar_label = format!("Section navigation ({instance_caption})");
    let label_id = format!("blocks-sidebar-rail-panel-group-label-{suffix}");
    let state = Sidebar::new(SidebarState::Expanded);
    let props = SidebarProps {
        collapsible: SidebarCollapsible::None,
        ..SidebarProps::default()
    };
    let menu_items: Vec<Node> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            sidebar::menu_item(
                vec![],
                vec![sidebar::menu_button(
                    &fandhe_frontend_pre_styled_ui::sidebar::SidebarMenuButtonProps {
                        href: None,
                        active: i == 0,
                        ..Default::default()
                    },
                    None,
                    vec![],
                    vec![text(*item)],
                )],
            )
        })
        .collect();
    sidebar::root(
        &state,
        &props,
        sidebar_label.as_str(),
        Some(root_id),
        vec![],
        vec![
            sidebar::header(
                vec![("data-blocks-sidebar-rail-panel-header", "")],
                vec![
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: fandhe_frontend_pre_styled_ui::heading::HeadingSize::Sm,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text(label)],
                    ),
                    search_group(suffix),
                ],
            ),
            sidebar::content(
                vec![],
                vec![sidebar::group(
                    Some(label_id.as_str()),
                    vec![],
                    vec![
                        sidebar::group_label(Some(label_id.as_str()), vec![], vec![text(label)]),
                        sidebar::group_content(vec![], vec![sidebar::menu(vec![], menu_items)]),
                    ],
                )],
            ),
        ],
    )
}

/// 1 インスタンス分（レール + パネル + 本文プレースホルダ）。`narrow` は
/// `true` のとき [`LAYOUT_CSS`] がフレーム幅を固定し常にパネル非表示
/// にする。
fn instance(suffix: &str, narrow: bool, instance_caption: &str) -> Node {
    let root_id = format!("blocks-sidebar-rail-panel-root-{suffix}");
    let mut frame_attrs: Vec<(&str, &str)> = vec![("data-blocks-sidebar-rail-panel-frame", "")];
    if narrow {
        frame_attrs.push(("data-blocks-sidebar-rail-panel-narrow", ""));
    }
    div(
        frame_attrs,
        vec![
            rail(suffix, instance_caption),
            panel(suffix, &root_id, instance_caption),
            div(
                vec![("data-blocks-sidebar-rail-panel-placeholder", "")],
                vec![],
            ),
        ],
    )
}

/// `sidebar-rail-panel` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。2 インスタンスを `data-blocks-sidebar-rail-panel-stack` の下へ
/// 縦に並べる（モジュール doc「2 インスタンスを静的に並記する理由」参照）。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-sidebar-rail-panel-stack", "")],
        vec![
            p(
                vec![("data-blocks-sidebar-rail-panel-caption", "")],
                vec![text("Desktop")],
            ),
            instance("desktop", false, "Desktop"),
            p(
                vec![("data-blocks-sidebar-rail-panel-caption", "")],
                vec![text("Narrow (rail only)")],
            ),
            instance("narrow", true, "Narrow (rail only)"),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/sidebar-rail-panel/",
    title: "sidebar-rail-panel",
    category: BlockCategory::Sidebar,
    rust_source: "crates/docs-site/src/blocks/application/sidebar/sidebar_rail_panel.rs",
    demo_class: "blocks-sidebar-rail-panel",
    parts: &[
        Part {
            label: "Sidebar",
            path: "/themes/sidebar/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
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
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Field",
            path: "/themes/field/",
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
/// `@container` はコンテナ自身ではなく子孫にのみ適用されるため、狭幅規則は
/// 祖先セレクタを重ねず `[data-scope="sidebar"][data-part="root"]` を直接
/// 選択する（モジュール doc「狭幅ではパネルのみ隠す」節参照、
/// `app_shell_sidebar` PR #3324 の教訓を踏襲）。
const LAYOUT_CSS: &str = "\
.blocks-demo.blocks-sidebar-rail-panel {\n  padding: 0;\n  overflow-x: auto;\n}\n\
[data-blocks-sidebar-rail-panel-stack] {\n  display: flex;\n  flex-direction: column;\n  gap: 1rem;\n}\n\
[data-blocks-sidebar-rail-panel-caption] {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-sidebar-rail-panel-frame] {\n  display: flex;\n  container-type: inline-size;\n  container-name: blocks-sidebar-rail-panel;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  overflow: hidden;\n  min-height: 28rem;\n}\n\
[data-blocks-sidebar-rail-panel-frame][data-blocks-sidebar-rail-panel-narrow] {\n  max-inline-size: 22rem;\n}\n\
[data-blocks-sidebar-rail-panel-frame] > [data-scope=\"sidebar\"][data-part=\"root\"] {\n  min-height: 28rem;\n  height: auto;\n  flex: 1 1 auto;\n}\n\
[data-blocks-sidebar-rail-panel-rail] {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-1, 0.25rem);\n  width: 3.5rem;\n  flex: 0 0 auto;\n  padding: var(--fandhe-space-2, 0.5rem);\n  border-inline-end: 1px solid var(--fandhe-color-border);\n  background: var(--fandhe-color-bg-subtle);\n}\n\
[data-blocks-sidebar-rail-panel-rail-top] {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1, 0.25rem);\n}\n\
[data-blocks-sidebar-rail-panel-rail-bottom] {\n  margin-block-start: auto;\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-2, 0.5rem);\n}\n\
[data-scope=\"button\"][data-blocks-sidebar-rail-panel-current] {\n  background: var(--fandhe-color-bg-muted);\n  color: var(--fandhe-color-fg);\n}\n\
[data-blocks-sidebar-rail-panel-header] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2, 0.5rem);\n}\n\
[data-blocks-sidebar-rail-panel-placeholder] {\n  flex: 1;\n  margin: 1.5rem;\n  min-height: 16rem;\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg-muted);\n}\n\
@container blocks-sidebar-rail-panel (max-width: 40rem) {\n  \
[data-scope=\"sidebar\"][data-part=\"root\"] {\n    display: none;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品を実際に出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"sidebar\"",
            "data-scope=\"icon\"",
            "data-scope=\"avatar\"",
            "data-scope=\"heading\"",
            "data-scope=\"menu\"",
            "data-scope=\"input-group\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains("<input"), "demo should render a search input");
    }

    /// 2 インスタンスが id 重複なく並記されていること。
    #[test]
    fn demo_has_two_instances_with_unique_ids() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-sidebar-rail-panel-frame").count(),
            2,
            "demo should render exactly 2 frames"
        );
        assert_eq!(
            html.matches("data-blocks-sidebar-rail-panel-narrow")
                .count(),
            1,
            "demo should render exactly 1 narrow instance"
        );
        let mut ids: Vec<&str> = Vec::new();
        let mut rest = html.as_str();
        while let Some(pos) = rest.find("id=\"") {
            rest = &rest[pos + 4..];
            if let Some(end) = rest.find('"') {
                ids.push(&rest[..end]);
            }
        }
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            ids.len(),
            "demo should not render duplicate ids"
        );
    }

    /// レールの現在地強調はインスタンスごとにちょうど 1 件であること。
    #[test]
    fn rail_marks_exactly_one_current_section_per_instance() {
        let html = render(&demo());
        assert_eq!(
            html.matches("aria-current=\"page\"").count(),
            2,
            "each instance should mark exactly one current rail section"
        );
        assert_eq!(
            html.matches("data-blocks-sidebar-rail-panel-current")
                .count(),
            2
        );
    }

    /// パネル見出しが選択中セクションのラベル（Projects）と一致すること。
    #[test]
    fn panel_heading_matches_current_section_label() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-scope=\"heading\"").count(),
            2,
            "each instance should render one panel heading"
        );
        assert_eq!(
            html.matches(">Projects<").count(),
            4,
            "current section label should appear as both heading and group label per instance"
        );
    }

    /// 検索アイコンがストローク描画（`fill="none"` + `stroke="currentColor"`）
    /// であること（`fill="currentColor"` 継承のままだと虫眼鏡パスが塗り
    /// 潰された円になる、#2941 PR レビュー指摘の回帰防止）。
    #[test]
    fn search_icon_is_stroke_drawn_not_filled() {
        let html = render(&demo());
        assert_eq!(
            html.matches("fill=\"none\"").count(),
            2,
            "each instance's search icon path should override fill to none"
        );
        assert_eq!(html.matches("stroke=\"currentColor\"").count(), 2);
    }

    /// Desktop/Narrow 両インスタンスの `nav`（レール）・`sidebar::root`
    /// （パネル）のランドマークラベルが重複しないこと（#2941 PR レビュー
    /// 指摘: 両方とも無 JS で常時 DOM 上に存在するため、同名だと支援技術が
    /// 区別できない）。
    #[test]
    fn rail_and_panel_landmark_labels_are_unique_per_instance() {
        let html = render(&demo());
        assert!(html.contains("aria-label=\"Primary sections (Desktop)\""));
        assert!(html.contains("aria-label=\"Primary sections (Narrow (rail only))\""));
        assert!(html.contains("Section navigation (Desktop)"));
        assert!(html.contains("Section navigation (Narrow (rail only))"));
    }

    /// パネル見出しは Demo 本文の H2（block ページ側）の子として構造化
    /// されるよう H3 であること（#2941 PR レビュー指摘: H2 のままだと
    /// Demo 見出しの兄弟になり配下として構造化されない）。
    #[test]
    fn panel_heading_is_h3_not_h2() {
        let html = render(&demo());
        assert_eq!(html.matches("<h3").count(), 2);
        assert_eq!(html.matches("<h2").count(), 0);
    }

    /// `<form>`・死リンク（`href="#"`）・`data:` URI を出力しないこと。
    #[test]
    fn no_form_semantics_or_dead_links() {
        let html = render(&demo());
        assert!(!html.contains("<form"), "demo should not emit <form>");
        assert!(
            !html.contains("href=\"#\""),
            "demo should not emit href=\"#\""
        );
        assert!(
            !html.contains("src=\"data:"),
            "demo should not emit data: URIs"
        );
    }

    /// レール・設定・パネル内の全ボタンが `type="button"` であること
    /// （暗黙の submit を持たない）。
    #[test]
    fn all_buttons_are_type_button() {
        let html = render(&demo());
        let button_count = html.matches("<button").count();
        let typed_count = html.matches(r#"type="button""#).count();
        assert!(button_count > 0, "demo should render buttons");
        assert_eq!(
            button_count, typed_count,
            "every <button> should carry type=\"button\""
        );
    }

    /// [`LAYOUT_CSS`] が安全（`<` を含まない）で、狭幅ではパネル
    /// （`sidebar` root）のみを隠しレールは隠さないこと。
    #[test]
    fn layout_css_is_safe_and_hides_only_panel_in_narrow_container() {
        assert!(!LAYOUT_CSS.contains('<'), "LAYOUT_CSS must not contain '<'");
        assert!(LAYOUT_CSS.contains("@container blocks-sidebar-rail-panel (max-width: 40rem)"));
        let container_start = LAYOUT_CSS
            .find("@container blocks-sidebar-rail-panel")
            .expect("container query should be present");
        let container_block = &LAYOUT_CSS[container_start..];
        assert!(container_block.contains("[data-part=\"root\"]"));
        assert!(
            !container_block.contains("[data-blocks-sidebar-rail-panel-rail]"),
            "narrow container rule must not hide the rail"
        );
    }

    /// [`demo`] は呼び出しごとに同一の HTML を返す（決定的）。
    #[test]
    fn demo_is_deterministic() {
        assert_eq!(render(&demo()), render(&demo()));
    }
}
