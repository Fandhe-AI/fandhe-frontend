//! `app-shell-sidebar-header` block（イシュー #2895）。
//!
//! 左の固定サイドバー + 右上の常時表示ヘッダーバー（検索欄・通知ボタン・
//! プロフィールメニュー）を組み合わせたアプリシェルの合成例。
//!
//! # 使用部品
//!
//! `sidebar` / `input-group` / `input` / `button` / `menu` / `avatar` /
//! `icon` の 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。
//! 新しい UI 部品は作らない。`field` は `parts` に含めない（`FieldProps`
//! 型を [`input::input`] 呼び出しに使うのみで、可視ラベルは出さず
//! `aria-label` で代える）。
//!
//! # 3 variant を 1 つの Demo に縦並記する
//!
//! | variant | 内容 |
//! |---|---|
//! | `standard` | サイドバー（展開）+ ヘッダー + 全幅メイン |
//! | `constrained` | 構成は同じで、メイン内側の最大幅を制限し中央寄せする |
//! | `narrow` | 狭い枠の中でサイドバーを Collapsed + Offcanvas で隠し、ヘッダー左端に `sidebar::trigger` を出す |
//!
//! 配色違い・タグライン付き等の集約元は本 Demo を増やさず、
//! `site/blocks/app-shell-sidebar-header.md` の差分メモへ委ねる（YAGNI）。
//!
//! narrow はビューポート幅にもリサイズにも連動しない、**状態を固定した
//! 静的な variant** である（`sidebar` の `mobile: true` は `position:
//! fixed` になり Demo 枠の外へはみ出すため採らない）。
//!
//! # 常時表示のヘッダーとスクロール
//!
//! 各 variant の provider に固定高を与え、`inset` を
//! `overflow-y: auto` のスクロールコンテナにする。ヘッダーは
//! `position: sticky; top: 0` で常時表示のまま残る。`inset` には
//! `tabindex="0"` + `role="region"` + variant ごとに一意な `aria-label`
//! を付け、キーボードでもスクロールできるようにする（`role="main"` は
//! 付けない。docs ページ側に既に `<main>` があるため）。
//!
//! # `<form>` を使わない・押しても何も起きない
//!
//! `crate::blocks` モジュール doc の不変条件どおり `<form>` を出力しない。
//! 検索 input はどこにも送信せず、通知ボタン・プロフィールメニュー・
//! サイドバーの各 `menu_button` はいずれも静的な初期状態を表示するのみで
//! 選択・送信・永続化・認証処理は行わない（`docs/policy/
//! intentional-non-adoption.md` §3.25）。ブランド名・ユーザー名・
//! メールアドレスはすべて架空（実企業名・実クレデンシャル・PII を含まない）。
//!
//! # `id`/`aria-label` は variant ごとに一意にする
//!
//! 3 variant を 1 Demo へ並記するため、サイドバー root id・検索 input
//! id・menu content id・スクロール領域の `aria-label` はすべて variant
//! 名の suffix で分ける
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` 契約、
//! `crate::blocks` モジュール doc 参照）。
//!
//! # アイコンは自作の単純幾何図形
//!
//! lucide 等の実アイコンセット由来の path は複製せず、
//! `sidebar_07::geo_icon` と同型の単純な矩形/円図形を自作する。
//!
//! # CSS フックの選び方
//!
//! `sidebar`/`input_group`/`input`/`button`/`menu`/`avatar`/`icon` の各
//! パーツは `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って
//! 除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-app-shell-sidebar-header-*` 属性で渡し、[`LAYOUT_CSS`]
//! 側も同じ属性セレクタで対応する。素の `div`/`p` には `class` が
//! そのまま効くため、配置は `.blocks-app-shell-sidebar-header-*` クラス
//! セレクタで行う。DOM 順は「ヘッダー → メイン」で固定し `order` は
//! 使わない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, p, span, text, Node};
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
use fandhe_frontend_pre_styled_ui::Size;

/// 自作の単純な矩形アイコン（モジュール doc「アイコンは自作の単純幾何
/// 図形」節。装飾用途のため `label: None`）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// header の team switcher 相当（ブランド名 + ロゴ用の幾何アイコン、
/// 静的表示のみ）。
fn brand(label: &'static str) -> Node {
    sidebar::header(
        vec![],
        vec![div(
            vec![("data-blocks-app-shell-sidebar-header-brand", "")],
            vec![geo_icon("M4 4h16v16H4z"), span(vec![], vec![text(label)])],
        )],
    )
}

/// nav グループ 1 件（`menu_button` を数件並べる。`active` で 1 件だけ
/// 選択中を示す）。
fn nav_group(items: &[(&'static str, &'static str)], active_key: &'static str) -> Node {
    let menu = sidebar::menu(
        vec![],
        items
            .iter()
            .map(|(icon_path, label)| {
                sidebar::menu_item(
                    vec![],
                    vec![sidebar::menu_button(
                        &SidebarMenuButtonProps {
                            href: None,
                            active: *label == active_key,
                            ..Default::default()
                        },
                        Some(geo_icon(icon_path)),
                        vec![],
                        vec![text(*label)],
                    )],
                )
            })
            .collect(),
    );
    sidebar::group(
        None,
        vec![],
        vec![sidebar::group_content(vec![], vec![menu])],
    )
}

/// footer の「設定」1 行（下部固定を表現する、モジュール doc参照）。
fn settings_footer() -> Node {
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
                    Some(geo_icon("M12 2a10 10 0 1 0 .001 20.001A10 10 0 0 0 12 2z")),
                    vec![],
                    vec![text("設定")],
                )],
            )],
        )],
    )
}

/// 左サイドバー本体（`provider` の直接の子として置く）。
fn app_sidebar(state: &Sidebar, props: &SidebarProps, root_id: &str) -> Node {
    sidebar::root(
        state,
        props,
        "Main navigation",
        Some(root_id),
        vec![],
        vec![
            brand("Nimbus Console"),
            sidebar::content(
                vec![],
                vec![
                    nav_group(
                        &[
                            ("M4 4h16v16H4z", "ダッシュボード"),
                            (
                                "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
                                "プロジェクト",
                            ),
                            ("M5 3h14v18H5z", "レポート"),
                        ],
                        "ダッシュボード",
                    ),
                    nav_group(
                        &[("M4 4h16v16H4z", "チーム"), ("M5 3h14v18H5z", "請求")],
                        "",
                    ),
                ],
            ),
            settings_footer(),
        ],
    )
}

/// ヘッダーの検索欄（`input_group` + `input`。可視ラベルは出さず
/// `aria-label` で代える、モジュール doc「使用部品」節）。
fn search_field(variant: &'static str) -> Node {
    let field_id = format!("blocks-app-shell-sidebar-header-search-{variant}");
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
        vec![("data-blocks-app-shell-sidebar-header-search", "")],
        vec![
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
                    ("aria-label", "アプリ内を検索"),
                    ("autocomplete", "off"),
                    ("placeholder", "検索..."),
                ],
            ),
        ],
    )
}

/// 通知ボタン（押しても何も起きない静的表示、モジュール doc参照）。
fn notify_button() -> Node {
    button::icon_button(
        &ButtonProps::default(),
        "通知",
        vec![("data-blocks-app-shell-sidebar-header-notify", "")],
        vec![geo_icon(
            "M12 3a5 5 0 0 0-5 5v3l-2 4h14l-2-4V8a5 5 0 0 0-5-5zM10 19a2 2 0 0 0 4 0",
        )],
    )
}

/// プロフィールメニュー（閉じた `menu` + avatar トリガー）。
fn profile_menu(variant: &'static str) -> Node {
    let content_id = format!("blocks-app-shell-sidebar-header-profile-menu-{variant}");
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some(content_id.as_str()),
        vec![
            ("aria-label", "プロフィールメニューを開く"),
            ("data-blocks-app-shell-sidebar-header-profile-trigger", ""),
        ],
        vec![avatar::root(
            &AvatarProps::default(),
            vec![],
            vec![avatar::fallback(
                ImageStatus::Error,
                vec![],
                vec![text("AL")],
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

/// 常時表示のヘッダーバー（`sidebar::inset` の直接の子として `sticky`
/// で置く）。`trigger` は narrow variant のみ `Some` を渡す。
fn top_bar(variant: &'static str, trigger: Option<Node>) -> Node {
    let mut children: Vec<Node> = Vec::new();
    if let Some(trigger) = trigger {
        children.push(trigger);
    }
    children.push(search_field(variant));
    children.push(div(
        vec![("data-blocks-app-shell-sidebar-header-actions", "")],
        vec![notify_button(), profile_menu(variant)],
    ));
    div(
        vec![("data-blocks-app-shell-sidebar-header-topbar", "")],
        children,
    )
}

/// メイン領域（ダミー行を並べたスクロール可能な本文）。
fn main_area(constrained: bool) -> Node {
    let rows: Vec<Node> = (1..=8)
        .map(|n| {
            div(
                vec![("data-blocks-app-shell-sidebar-header-row", "")],
                vec![text(format!("プレースホルダー行 {n}"))],
            )
        })
        .collect();
    let attr = if constrained {
        "data-blocks-app-shell-sidebar-header-main-constrained"
    } else {
        "data-blocks-app-shell-sidebar-header-main"
    };
    div(vec![(attr, "")], rows)
}

/// variant 1 件分の骨格（`provider > (root, inset(topbar, main))`）。
fn shell(
    variant: &'static str,
    state: SidebarState,
    collapsible: SidebarCollapsible,
    constrained: bool,
    show_trigger: bool,
) -> Node {
    let sidebar_state = Sidebar::new(state);
    let props = SidebarProps {
        collapsible,
        ..SidebarProps::default()
    };
    let root_id = format!("blocks-app-shell-sidebar-header-root-{variant}");
    let region_label = format!("メインコンテンツ（{variant}）");

    let trigger = if show_trigger {
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

    let inset = sidebar::inset(
        vec![
            ("tabindex", "0"),
            ("role", "region"),
            ("aria-label", region_label.as_str()),
            ("data-blocks-app-shell-sidebar-header-inset", ""),
        ],
        vec![top_bar(variant, trigger), main_area(constrained)],
    );

    sidebar::provider(
        &sidebar_state,
        &props,
        vec![
            ("data-blocks-app-shell-sidebar-header-instance", ""),
            ("data-blocks-app-shell-sidebar-header-variant", variant),
        ],
        vec![app_sidebar(&sidebar_state, &props, root_id.as_str()), inset],
    )
}

/// キャプション行。
fn caption(label: &'static str) -> Node {
    p(
        vec![("data-blocks-app-shell-sidebar-header-caption", "")],
        vec![text(label)],
    )
}

/// `app-shell-sidebar-header` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。3 variant を縦に並べる（モジュール doc「3 variant を
/// 1 つの Demo に縦並記する」参照）。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-app-shell-sidebar-header-stack", "")],
        vec![
            caption("標準"),
            shell(
                "standard",
                SidebarState::Expanded,
                SidebarCollapsible::None,
                false,
                false,
            ),
            caption("メイン幅を制限"),
            shell(
                "constrained",
                SidebarState::Expanded,
                SidebarCollapsible::None,
                true,
                false,
            ),
            caption("狭幅（サイドバー折りたたみ）"),
            shell(
                "narrow",
                SidebarState::Collapsed,
                SidebarCollapsible::Offcanvas,
                false,
                true,
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/app-shell-sidebar-header/",
    title: "app-shell-sidebar-header",
    category: BlockCategory::AppShell,
    rust_source: "crates/docs-site/src/blocks/application/app_shell/app_shell_sidebar_header.rs",
    demo_class: "blocks-app-shell-sidebar-header",
    parts: &[
        Part {
            label: "Sidebar",
            path: "/themes/sidebar/",
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
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
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
const LAYOUT_CSS: &str = "\
.blocks-demo.blocks-app-shell-sidebar-header {\n  padding: 0;\n  overflow-x: auto;\n}\n\
[data-blocks-app-shell-sidebar-header-stack] {\n  display: flex;\n  flex-direction: column;\n  gap: 1rem;\n}\n\
[data-blocks-app-shell-sidebar-header-caption] {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-app-shell-sidebar-header-instance][data-scope=\"sidebar\"][data-part=\"provider\"] {\n  min-height: 0;\n  block-size: 28rem;\n  min-width: 56rem;\n}\n\
[data-blocks-app-shell-sidebar-header-instance][data-blocks-app-shell-sidebar-header-variant=\"narrow\"][data-scope=\"sidebar\"][data-part=\"provider\"] {\n  min-width: 20rem;\n}\n\
[data-blocks-app-shell-sidebar-header-brand] {\n  display: flex;\n  align-items: center;\n  gap: 0.5rem;\n  font-weight: var(--fandhe-font-font-weight-semibold, 600);\n}\n\
[data-scope=\"sidebar\"][data-part=\"inset\"][data-blocks-app-shell-sidebar-header-inset] {\n  overflow-y: auto;\n  display: flex;\n  flex-direction: column;\n}\n\
[data-blocks-app-shell-sidebar-header-topbar] {\n  position: sticky;\n  top: 0;\n  z-index: var(--fandhe-z-index-docked);\n  display: flex;\n  align-items: center;\n  gap: 0.75rem;\n  padding: 0.75rem 1rem;\n  background: var(--fandhe-color-bg);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-app-shell-sidebar-header-search] {\n  flex: 1 1 auto;\n  max-inline-size: 24rem;\n}\n\
[data-blocks-app-shell-sidebar-header-actions] {\n  display: flex;\n  align-items: center;\n  gap: 0.5rem;\n  margin-inline-start: auto;\n}\n\
[data-blocks-app-shell-sidebar-header-main], [data-blocks-app-shell-sidebar-header-main-constrained] {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n  padding: 1rem;\n}\n\
[data-blocks-app-shell-sidebar-header-main-constrained] {\n  max-inline-size: 48rem;\n  margin-inline: auto;\n  inline-size: 100%;\n}\n\
[data-blocks-app-shell-sidebar-header-row] {\n  border-radius: var(--fandhe-radius-md);\n  background: var(--fandhe-color-bg-muted);\n  padding: 0.75rem 1rem;\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn demo_composes_all_seven_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"sidebar\"",
            "data-scope=\"input-group\"",
            "data-scope=\"field\" data-part=\"input\"",
            "data-scope=\"button\"",
            "data-scope=\"menu\"",
            "data-scope=\"avatar\"",
            "data-scope=\"icon\"",
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
    fn demo_contains_each_variant_exactly_once() {
        let html = render(&demo());
        // standard/constrained はサイドバー root の `id` にのみ現れる（1 回）。
        // narrow はさらに `sidebar::trigger` の `aria-controls` にも root_id
        // が現れるため 2 回になる（[`shell`] のドキュメント参照）。
        for (variant, expected) in [("standard", 1), ("constrained", 1), ("narrow", 2)] {
            let needle = format!("blocks-app-shell-sidebar-header-root-{variant}");
            assert_eq!(
                html.matches(&needle).count(),
                expected,
                "variant {variant} should appear exactly {expected} time(s)"
            );
        }
    }

    #[test]
    fn only_narrow_variant_shows_the_sidebar_trigger() {
        let html = render(&demo());
        // `menu::trigger`（プロフィールメニュー、variant ごとに 1 個）も
        // 同じ `data-part="trigger"` を出力するため、`data-scope="sidebar"`
        // と組み合わせて sidebar 固有の trigger のみを数える。
        assert_eq!(
            html.matches("data-scope=\"sidebar\" data-part=\"trigger\"")
                .count(),
            1,
            "sidebar trigger should appear exactly once (narrow variant only)"
        );
    }

    #[test]
    fn sidebar_footer_appears_once_per_variant() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-part=\"footer\"").count(),
            3,
            "each of the 3 variants should render its own sidebar footer"
        );
    }

    #[test]
    fn narrow_sidebar_is_collapsed_and_offcanvas() {
        let html = render(&demo());
        assert!(html.contains("data-state=\"collapsed\""));
        assert!(html.contains("data-collapsible=\"offcanvas\""));
    }

    #[test]
    fn layout_css_has_sticky_header_and_max_width_but_no_order() {
        assert!(LAYOUT_CSS.contains("position: sticky;"));
        assert!(LAYOUT_CSS.contains("max-inline-size"));
        assert!(!LAYOUT_CSS.contains("order:"));
    }

    /// narrow variant の provider は `data-blocks-app-shell-sidebar-header-
    /// variant="narrow"` を持ち、この属性を含む複合セレクタで `min-width`
    /// を上書きできること（56rem 一律適用による狭幅表示例の横スクロール
    /// 回帰防止、codex-review PR #3325 指摘）。
    #[test]
    fn narrow_variant_is_tagged_and_overrides_min_width() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-app-shell-sidebar-header-variant=\"narrow\"")
                .count(),
            1,
            "exactly one provider (the narrow variant) should carry the narrow tag"
        );
        assert!(
            LAYOUT_CSS.contains(
                "[data-blocks-app-shell-sidebar-header-variant=\"narrow\"][data-scope=\"sidebar\"][data-part=\"provider\"] {\n  min-width: 20rem;\n}"
            ),
            "narrow variant should override the 56rem min-width with a narrower one"
        );
    }
}
