//! `app-shell-three-column` block（イシュー #2898。Application / App Shell
//! カテゴリ 3 件目、対応表 ID R1078（代表構成）を主参照とする合成例）。
//!
//! 左の固定サイドバー + メイン + 補助カラム（一覧/詳細用）の 3 列アプリ
//! シェル。集約元 R1079（補助カラムの右配置）・R1082（アイコンのみの狭幅
//! サイドバー）・R1083（狭サイドバー + 上部ヘッダー）・R0136（中身なしの
//! 2 カラム骨格）の差分は `site/blocks/app-shell-three-column.md` の
//! 「原案差分メモ」節に記す。
//!
//! # 使用部品
//!
//! `sidebar` / `avatar` / `button` / `icon` / `input-group` / `input` /
//! `menu` / `visually-hidden` の 8 部品を合成する（[`BLOCK`] の `parts` に
//! 一致させる契約）。新しい UI 部品は作らない。`field` は `parts` に含めず
//! [`input::input`] 呼び出しの `FieldProps` にのみ使う
//! （`app_shell_sidebar_header` と同じ判断）。
//!
//! # 3 variant を 1 つの Demo に縦並記する（無 JS のため実挙動は示せない）
//!
//! `app_shell_sidebar_header`（イシュー #2895）と同じ判断: 無 JS の静的
//! HTML では幅に連動した動的な折りたたみを実演できないため、同じ骨格
//! （[`shell`]）を使う 3 variant を縦に並べ、集約元の差分をページ内で
//! 読み取れるようにする（`@container` による幅連動の切り替えは採らない）。
//!
//! | variant | 内容 | 対応 ID |
//! |---|---|---|
//! | `wide` | サイドバー（展開）+ 補助カラム（一覧、右） + メイン。ヘッダーなし | R1078（代表）+ R1079（補助カラム右配置）+ R0136（骨格） |
//! | `icon` | サイドバー（アイコンのみ折りたたみ）+ ヘッダーバー（検索・通知・プロフィール）+ メイン + 補助カラム（詳細、右） | R1082（アイコンのみの狭サイドバー） |
//! | `narrow` | サイドバーを Offcanvas で隠し、ヘッダーバー（trigger のみ）+ メイン。補助カラムなし | R1083（狭サイドバー + 上部ヘッダー） |
//!
//! `wide` と `icon` を並べることで「幅広」と「アイコンのみの狭幅」の 2
//! 状態を対比でき、`narrow` が「狭い幅ではサイドバーも上部バーへ畳む」
//! 挙動の到達状態を示す。ヘッダーバー（検索・通知・プロフィール）の
//! 有無だけが `wide` と `icon` の差分であり、要件の「ヘッダー付き版は
//! 上端にヘッダーバーを足すだけ」を体現する。
//!
//! # DOM 順はサイドバー → メイン → 補助カラム（`order` を使わない）
//!
//! 補助カラムは常にメインの後ろ（視覚的には右）に置く。`grid-template-
//! areas`/`order` による視覚順の入れ替えは行わないため、読み上げ順・Tab
//! 順が視覚順と一致する（WCAG 1.3.2）。
//!
//! # `<form>` を使わない・押しても何も起きない
//!
//! `crate::blocks` モジュール doc の不変条件どおり `<form>` を出力しない。
//! 検索 input はどこにも送信せず、通知ボタン・プロフィールメニュー・
//! サイドバーの各 `menu_button` はいずれも静的な初期状態を表示するのみで
//! 選択・送信・永続化・認証処理は行わない（`docs/policy/
//! intentional-non-adoption.md` §3.25）。ブランド名・ユーザー名は架空
//! （実企業名・実クレデンシャル・PII を含まない）。
//!
//! # `id`/`aria-label` は variant ごとに一意にする
//!
//! 3 variant を 1 Demo へ並記するため、サイドバー root id・検索 input
//! id・menu content id はすべて variant 名の suffix で分ける
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` 契約、
//! `crate::blocks` モジュール doc 参照）。
//!
//! # アイコンは自作の単純幾何図形
//!
//! `sidebar_07`/`app_shell_sidebar_header` と同型の単純な矩形/円図形を
//! 自作する（実在ブランドのロゴを模さない）。
//!
//! # CSS フックの選び方
//!
//! `sidebar`/`input_group`/`input`/`button`/`menu`/`avatar`/`icon` の各
//! パーツは呼び出し側 `attrs` の `class` を `drop_class_attr` により黙って
//! 除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-app-shell-three-column-*` 属性で渡し、[`LAYOUT_CSS`] 側も
//! 同じ属性セレクタで対応する。素の `div`/`p`/`aside` には `class` が
//! そのまま効くため、配置は `.blocks-app-shell-three-column-*` クラス
//! セレクタで行う。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{aside, div, el, label, li, p, section, span, text, ul, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::field::FieldProps;
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::sidebar;
use fandhe_frontend_pre_styled_ui::sidebar::{
    Sidebar, SidebarCollapsible, SidebarMenuButtonProps, SidebarProps, SidebarState,
};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 自作の単純な矩形アイコン（装飾用途のため `label: None`）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// ブランド（ロゴ用の幾何アイコン + 社名、静的表示のみ）。
fn brand() -> Node {
    sidebar::header(
        vec![],
        vec![div(
            vec![("data-blocks-app-shell-three-column-brand", "")],
            vec![
                geo_icon("M4 4h16v16H4z"),
                span(vec![], vec![text("サンプル社")]),
            ],
        )],
    )
}

/// nav 項目一覧（1 件だけ active）。
fn nav_menu() -> Node {
    let items: &[(&str, &str)] = &[
        ("M4 4h16v16H4z", "ダッシュボード"),
        (
            "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
            "プロジェクト",
        ),
        ("M12 2 3 7v10l9 5 9-5V7z", "カレンダー"),
        ("M5 3h14v18H5z", "レポート"),
        ("M12 2a10 10 0 1 0 .001 20.001A10 10 0 0 0 12 2z", "設定"),
    ];
    sidebar::group(
        None,
        vec![],
        vec![sidebar::group_content(
            vec![],
            vec![sidebar::menu(
                vec![],
                items
                    .iter()
                    .map(|(icon_path, label_text)| {
                        sidebar::menu_item(
                            vec![],
                            vec![sidebar::menu_button(
                                &SidebarMenuButtonProps {
                                    href: None,
                                    active: *label_text == "ダッシュボード",
                                    ..Default::default()
                                },
                                Some(geo_icon(icon_path)),
                                vec![],
                                vec![text(*label_text)],
                            )],
                        )
                    })
                    .collect(),
            )],
        )],
    )
}

/// 左サイドバー本体（`provider` の直接の子として置く）。
fn app_sidebar(state: &Sidebar, props: &SidebarProps, root_id: &str, aria_label: &str) -> Node {
    sidebar::root(
        state,
        props,
        aria_label,
        Some(root_id),
        vec![],
        vec![brand(), sidebar::content(vec![], vec![nav_menu()])],
    )
}

/// ヘッダーの検索欄（`input_group` + `input` + `visually_hidden` の可視
/// ラベル代替、`icon`/`narrow` variant のみが持つ）。
fn search_field(variant: &'static str) -> Node {
    let control_id = format!("blocks-app-shell-three-column-search-{variant}-control");
    let field_id = format!("blocks-app-shell-three-column-search-{variant}");
    let field = FieldProps {
        id: &field_id,
        ids: Default::default(),
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
    input_group::root(
        &group_props,
        vec![("data-blocks-app-shell-three-column-search", "")],
        vec![
            visually_hidden::root(
                vec![],
                vec![label(
                    vec![("for", control_id.as_str())],
                    vec![text("アプリ内を検索")],
                )],
            ),
            input_group::addon(
                InputGroupAlign::InlineStart,
                &group_props,
                vec![],
                vec![geo_icon(
                    "M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zm9 17-5.2-5.2",
                )],
            ),
            input::input(
                &InputProps::default(),
                &field,
                vec![
                    ("type", "search"),
                    ("autocomplete", "off"),
                    ("placeholder", "検索..."),
                ],
            ),
        ],
    )
}

/// 通知ボタン（押しても何も起きない静的表示）。
fn notify_button() -> Node {
    button::icon_button(
        &ButtonProps::default(),
        "通知を表示",
        vec![("data-blocks-app-shell-three-column-notify", "")],
        vec![geo_icon(
            "M12 3a5 5 0 0 0-5 5v3l-2 4h14l-2-4V8a5 5 0 0 0-5-5zM10 19a2 2 0 0 0 4 0",
        )],
    )
}

/// プロフィールメニュー（閉じた `menu` + avatar トリガー）。
fn profile_menu(variant: &'static str) -> Node {
    let content_id = format!("blocks-app-shell-three-column-profile-menu-{variant}");
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id.as_str()),
        vec![
            ("aria-label", "プロフィールメニューを開く"),
            ("data-blocks-app-shell-three-column-profile-trigger", ""),
        ],
        vec![avatar::root(
            &AvatarProps::default(),
            vec![],
            vec![avatar::fallback(
                ImageStatus::Error,
                vec![],
                vec![text("サ")],
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

/// 常時表示のヘッダーバー（`icon`/`narrow` variant のみが持つ）。
/// `trigger` は `narrow` variant のみ `Some` を渡す。
fn header_bar(variant: &'static str, trigger: Option<Node>, with_search: bool) -> Node {
    let mut children: Vec<Node> = Vec::new();
    if let Some(trigger) = trigger {
        children.push(trigger);
    }
    if with_search {
        children.push(search_field(variant));
    }
    children.push(div(
        vec![("data-blocks-app-shell-three-column-actions", "")],
        vec![notify_button(), profile_menu(variant)],
    ));
    div(
        vec![("data-blocks-app-shell-three-column-topbar", "")],
        children,
    )
}

/// メイン領域（中身なしのプレースホルダー行、R0136 の骨格を兼ねる）。
fn main_area(aria_label: &str) -> Node {
    let rows: Vec<Node> = (1..=6)
        .map(|n| {
            div(
                vec![("data-blocks-app-shell-three-column-row", "")],
                vec![text(format!("プレースホルダー行 {n}"))],
            )
        })
        .collect();
    section(
        vec![
            ("tabindex", "0"),
            ("role", "region"),
            ("aria-label", aria_label),
            ("data-blocks-app-shell-three-column-main", ""),
        ],
        rows,
    )
}

/// 補助カラム（一覧/詳細用、中身なしのプレースホルダー枠）。
fn secondary_column(aria_label: &str) -> Node {
    let items = ["項目 A", "項目 B", "項目 C"]
        .iter()
        .map(|label_text| {
            li(
                vec![("class", "blocks-app-shell-three-column-side-item")],
                vec![text(*label_text)],
            )
        })
        .collect();
    aside(
        vec![
            ("aria-label", aria_label),
            ("data-blocks-app-shell-three-column-aux", ""),
        ],
        vec![ul(
            vec![("class", "blocks-app-shell-three-column-side-list")],
            items,
        )],
    )
}

/// キャプション行。
fn caption(label_text: &'static str) -> Node {
    p(
        vec![("data-blocks-app-shell-three-column-caption", "")],
        vec![text(label_text)],
    )
}

/// variant 1 件分の骨格（`provider > (root, inset(topbar?, body(main, aux?)))`）。
#[allow(clippy::too_many_arguments)]
fn shell(
    variant: &'static str,
    state: SidebarState,
    collapsible: SidebarCollapsible,
    show_header: bool,
    with_search: bool,
    with_trigger: bool,
    with_aux: bool,
) -> Node {
    let sidebar_state = Sidebar::new(state);
    let props = SidebarProps {
        collapsible,
        ..SidebarProps::default()
    };
    let root_id = format!("blocks-app-shell-three-column-root-{variant}");
    let sidebar_label = format!("アプリのナビゲーション（{variant}）");
    let main_label = format!("メインコンテンツ（{variant}）");
    let aux_label = format!("補足情報（{variant}）");

    let trigger = if with_trigger {
        Some(sidebar::trigger(
            &sidebar_state,
            "サイドバーを開く",
            Some(root_id.as_str()),
            vec![],
            vec![],
        ))
    } else {
        None
    };

    let mut body_children = vec![main_area(&main_label)];
    if with_aux {
        body_children.push(secondary_column(&aux_label));
    }
    let body = div(
        vec![("data-blocks-app-shell-three-column-body", "")],
        body_children,
    );

    let mut inset_children = Vec::new();
    if show_header {
        inset_children.push(header_bar(variant, trigger, with_search));
    }
    inset_children.push(body);

    let inset = sidebar::inset(
        vec![("data-blocks-app-shell-three-column-inset", "")],
        inset_children,
    );

    sidebar::provider(
        &sidebar_state,
        &props,
        vec![
            ("data-blocks-app-shell-three-column-instance", ""),
            ("data-blocks-app-shell-three-column-variant", variant),
        ],
        vec![
            app_sidebar(&sidebar_state, &props, root_id.as_str(), &sidebar_label),
            inset,
        ],
    )
}

/// `app-shell-three-column` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。3 variant を縦に並べる（モジュール doc「3 variant を 1 つの
/// Demo に縦並記する」参照）。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-app-shell-three-column-stack", "")],
        vec![
            caption("幅広（サイドバー展開 + 補助カラム）"),
            shell(
                "wide",
                SidebarState::Expanded,
                SidebarCollapsible::None,
                false,
                false,
                false,
                true,
            ),
            caption("アイコンのみの狭サイドバー + ヘッダー"),
            shell(
                "icon",
                SidebarState::Collapsed,
                SidebarCollapsible::Icon,
                true,
                true,
                false,
                true,
            ),
            caption("狭幅（サイドバー折りたたみ、補助カラムなし）"),
            shell(
                "narrow",
                SidebarState::Collapsed,
                SidebarCollapsible::Offcanvas,
                true,
                false,
                true,
                false,
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/app-shell-three-column/",
    title: "app-shell-three-column",
    category: BlockCategory::AppShell,
    rust_source: "crates/docs-site/src/blocks/application/app_shell/app_shell_three_column.rs",
    demo_class: "blocks-app-shell-three-column",
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
            label: "Input Group",
            path: "/themes/input-group/",
        },
        Part {
            label: "Input",
            path: "/themes/input/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// [`super::mod@self`] の `stylesheet()` が `push_css` する block 固有の
/// レイアウト CSS（並列進行する他 block との `mod.rs::LAYOUT_CSS` 追記衝突
/// を避け本モジュール側の定数へ分離する）。
const LAYOUT_CSS: &str = "\
.blocks-demo.blocks-app-shell-three-column {\n  padding: 0;\n  overflow-x: auto;\n}\n\
[data-blocks-app-shell-three-column-stack] {\n  display: flex;\n  flex-direction: column;\n  gap: 1rem;\n}\n\
[data-blocks-app-shell-three-column-caption] {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-app-shell-three-column-instance][data-scope=\"sidebar\"][data-part=\"provider\"] {\n  min-height: 0;\n  block-size: 26rem;\n  min-width: 60rem;\n}\n\
[data-blocks-app-shell-three-column-brand] {\n  display: flex;\n  align-items: center;\n  gap: 0.5rem;\n  font-weight: var(--fandhe-font-font-weight-semibold, 600);\n}\n\
[data-scope=\"sidebar\"][data-part=\"inset\"][data-blocks-app-shell-three-column-inset] {\n  overflow-y: auto;\n  display: flex;\n  flex-direction: column;\n}\n\
[data-blocks-app-shell-three-column-topbar] {\n  position: sticky;\n  top: 0;\n  z-index: var(--fandhe-z-index-docked);\n  display: flex;\n  align-items: center;\n  gap: 0.75rem;\n  padding: 0.75rem 1rem;\n  background: var(--fandhe-color-bg);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-app-shell-three-column-search] {\n  flex: 1 1 auto;\n  max-inline-size: 24rem;\n}\n\
[data-blocks-app-shell-three-column-actions] {\n  display: flex;\n  align-items: center;\n  gap: 0.5rem;\n  margin-inline-start: auto;\n}\n\
[data-blocks-app-shell-three-column-body] {\n  display: flex;\n  gap: 1rem;\n  padding: 1rem;\n  flex: 1 1 auto;\n}\n\
[data-blocks-app-shell-three-column-main] {\n  flex: 1 1 auto;\n  min-inline-size: 0;\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n}\n\
[data-blocks-app-shell-three-column-aux] {\n  flex: 0 0 14rem;\n}\n\
.blocks-app-shell-three-column-side-list {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n  gap: 0.5rem;\n}\n\
.blocks-app-shell-three-column-side-item {\n  padding: 0.5rem 0.75rem;\n  border-radius: 0.375rem;\n  background: var(--fandhe-color-bg-subtle);\n}\n\
[data-blocks-app-shell-three-column-row] {\n  border-radius: var(--fandhe-radius-md);\n  background: var(--fandhe-color-bg-muted);\n  padding: 0.75rem 1rem;\n}\n\
[data-blocks-app-shell-three-column-instance][data-blocks-app-shell-three-column-variant=\"narrow\"][data-scope=\"sidebar\"][data-part=\"provider\"] {\n  min-width: 20rem;\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn demo_composes_all_eight_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"sidebar\"",
            "data-scope=\"avatar\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
            "data-scope=\"input-group\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"menu\"",
            "data-scope=\"visually-hidden\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    #[test]
    fn demo_has_no_form_no_dead_link_no_data_uri() {
        let html = render(&demo());
        assert!(!html.contains("<form"), "demo should never contain <form>");
        assert!(
            !html.contains("href=\"#\""),
            "demo should never contain a dead href=\"#\" link"
        );
        assert!(
            !html.contains("src=\"data:"),
            "demo should never contain a data: URI"
        );
        assert!(
            html.contains("type=\"button\""),
            "buttons should be type=\"button\""
        );
    }

    #[test]
    fn demo_contains_each_variant_root_exactly_once() {
        let html = render(&demo());
        for variant in ["wide", "icon", "narrow"] {
            let needle = format!("blocks-app-shell-three-column-root-{variant}");
            assert_eq!(
                html.matches(&needle).count(),
                if variant == "narrow" { 2 } else { 1 },
                "variant {variant} root id should appear the expected number of times"
            );
        }
    }

    #[test]
    fn only_narrow_variant_shows_the_sidebar_trigger() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-scope=\"sidebar\" data-part=\"trigger\"")
                .count(),
            1,
            "sidebar trigger should appear exactly once (narrow variant only)"
        );
    }

    #[test]
    fn only_wide_variant_omits_the_topbar() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-app-shell-three-column-topbar")
                .count(),
            2,
            "icon and narrow variants should each render a topbar"
        );
    }

    #[test]
    fn aux_column_renders_for_wide_and_icon_but_not_narrow() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-app-shell-three-column-aux")
                .count(),
            2,
            "wide and icon variants should each render an aux column, narrow should not"
        );
    }

    #[test]
    fn narrow_sidebar_is_collapsed_and_offcanvas_icon_is_collapsed_and_icon() {
        let html = render(&demo());
        // `data-collapsible`/`data-state` は headless-ui の契約により
        // `provider` と `root` の双方へ出力されるため、variant 1 件あたり
        // 2 回ずつ現れる（`app_shell_sidebar_header` と同型）。
        assert_eq!(
            html.matches("data-collapsible=\"offcanvas\"").count(),
            2,
            "only narrow variant uses offcanvas collapsible (provider + root)"
        );
        assert_eq!(
            html.matches("data-collapsible=\"icon\"").count(),
            2,
            "only icon variant uses icon collapsible (provider + root)"
        );
        assert!(
            html.matches("data-state=\"collapsed\"").count() >= 4,
            "icon and narrow variants should both start collapsed (provider + root each)"
        );
    }

    #[test]
    fn search_field_label_references_the_input_control_id() {
        let html = render(&demo());
        assert!(
            html.contains("blocks-app-shell-three-column-search-icon-control"),
            "html={html}"
        );
        assert!(html.contains("for=\"blocks-app-shell-three-column-search-icon-control\""));
    }

    #[test]
    fn layout_css_has_no_order_and_scopes_selectors_to_this_block() {
        assert!(!LAYOUT_CSS.contains("{\n  order:"));
        assert!(!LAYOUT_CSS.contains(";\n  order:"));
        assert!(LAYOUT_CSS.contains("[data-blocks-app-shell-three-column-aux]"));
    }
}
