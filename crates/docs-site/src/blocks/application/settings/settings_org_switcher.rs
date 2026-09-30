//! `settings-org-switcher` block（イシュー #2999。親トラッキング #2951
//! 「Blocks アプリケーション B（settings / auth / …）」配下、phase:4）。
//! 主参照 R0184（組織メニュー単体）、R0185（組織 / プロジェクト横並び）を
//! 集約する。`_/blocks-intake/` の対応ファイルは本イシュー着手時点で本
//! worktree に存在しないため、原稿・本コメントには対応表 ID のみを記す
//! （`profile-detail-datalist`〔#2937〕・`list-title-meta`〔#2925〕と同じ
//! 扱い）。
//!
//! # 使用部品
//!
//! `menu` / `avatar` / `badge` / `button` / `icon` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。新しい UI 部品は追加しない。
//!
//! # 2 variant の並記
//!
//! | variant（`data-blocks-settings-org-switcher-variant`） | 構成 | 対応 ID |
//! |---|---|---|
//! | `single` | 組織メニュー 1 個（トリガー = アバター + 組織名 + メンバー数バッジ + シェブロン） | R0184（代表） |
//! | `breadcrumb` | 組織トリガー（閉） → `/` 区切り → プロジェクトトリガー（開） を横並び | R0185 |
//!
//! いずれもメニュー内は `radio_item_group`（現在の組織 / プロジェクトへ
//! `checked: true`）+ `separator` + 作成・設定の操作ボタン 2 個で構成する
//! （[`menu_shell`] へ共通化、`navbar_two_row`/`header_flyout_menu` と同じ
//! 「1 関数化して variant 間で使い回す」設計）。
//!
//! # 開いたメニューを文書フローへ固定する（`position: static`）
//!
//! headless 層 `menu::positioner` は既定 `position: absolute; top: 100%`
//! （`crates/headless-ui/src/menu.rs`）のため、Open のまま置くと
//! `.blocks-demo`（横スクロール枠）からはみ出す。
//! `settings_billing_overview` の `toggle-tip` と同じ判断で、
//! [`LAYOUT_CSS`] が本 block ルート配下の `positioner` のみを
//! `position: static` へ中和し、通常のドキュメントフローへ乗せる
//! （`data-positioned` は wasm 層のみが付与する値なしマーカーのため
//! 無 JS の本 Demo とは競合しない）。
//!
//! # 無 JS のため全操作を disabled 固定
//!
//! メニュートリガー・操作ボタンはいずれも `disabled: true` で押しても
//! 何も起きないことを明示し、[`LAYOUT_CSS`] の `[data-disabled]`
//! 複合セレクタで `opacity: 1; cursor: default;` に中和する
//! （`navbar_two_row`/`header_flyout_menu` と同じ確定パターン）。
//!
//! # `id`/`aria-label` の一意性
//!
//! `menu` の `content_id`/`item_group_label` の `id` は `{kind}-{variant}`
//! （`kind` は `"org"`/`"project"`）を suffix にして一意化する
//! （`demo_output_has_no_dangling_aria_references_or_duplicate_ids` 契約）。
//! `breadcrumb` variant の閉じた組織メニューも `content` を必ず描画する
//! （`hidden` 存在属性のみで隠す。`aria-controls` の参照先を欠落させない
//! ための確定パターン）。
//!
//! # `<form>`/`href="#"`/`data:` を持たない
//!
//! `crate::blocks` モジュール doc「セキュリティ不変条件」節に従う静的な
//! 合成例。組織名・プロジェクト名はすべて架空。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::Size;

/// 架空の組織名 3 件（[`dummy_assets::COMPANY_NAMES`] の先頭 3 件を流用）。
fn org_names() -> [&'static str; 3] {
    [
        dummy_assets::COMPANY_NAMES[0],
        dummy_assets::COMPANY_NAMES[1],
        dummy_assets::COMPANY_NAMES[2],
    ]
}
/// [`org_names`] と対にするアバター頭文字。
const ORG_INITIALS: [&str; 3] = ["LS", "VF", "QM"];
/// [`org_names`] と対にするメンバー数表示。
const ORG_MEMBERS: [&str; 3] = ["12 members", "6 members", "3 members"];
/// 架空のプロジェクト名 3 件。
const PROJECT_NAMES: [&str; 3] = ["Atlas", "Nimbus", "Voyager"];

/// 装飾用の下向きシェブロン（実在ブランドのアイコンを模さない自作幾何
/// path、`navbar_two_row::geo_icon` と同型）。
fn chevron_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M6 9l6 6 6-6"),
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

/// 頭文字 fallback のみを持つ小サイズアバター（画像アセット不要）。
fn initials_avatar(initials: &'static str) -> Node {
    avatar::root(
        &AvatarProps {
            size: Size::Sm,
            ..AvatarProps::default()
        },
        vec![],
        vec![avatar::fallback(
            ImageStatus::Error,
            vec![],
            vec![text(initials)],
        )],
    )
}

/// 組織の radio item 1 件（アバター + 名前 + メンバー数バッジ）。
fn org_radio_row(
    checked: bool,
    name: &'static str,
    initials: &'static str,
    members: &'static str,
) -> Node {
    menu::radio_item(
        checked,
        name,
        false,
        false,
        vec![],
        vec![
            initials_avatar(initials),
            span(vec![], vec![text(name)]),
            badge(&BadgeProps::default(), vec![], vec![text(members)]),
        ],
    )
}

/// プロジェクトの radio item 1 件（名前のみ）。
fn project_radio_row(checked: bool, name: &'static str) -> Node {
    menu::radio_item(
        checked,
        name,
        false,
        false,
        vec![],
        vec![span(vec![], vec![text(name)])],
    )
}

/// 作成・設定の操作行（末尾に置く 2 個の `disabled: true` ボタン）。
fn action_row(create_label: &'static str, manage_label: &'static str) -> Node {
    div(
        vec![("data-blocks-settings-org-switcher-actions", "")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-settings-org-switcher-action", "")],
                vec![text(create_label)],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-settings-org-switcher-action", "")],
                vec![text(manage_label)],
            ),
        ],
    )
}

/// 組織 / プロジェクト共通のメニュー一式（`root` + `trigger` +
/// `positioner(content)`）を組み立てる。`kind`/`variant` の組で `id` を
/// 一意にする（モジュール doc「`id`/`aria-label` の一意性」節参照）。
#[allow(clippy::too_many_arguments)]
fn menu_shell(
    kind: &'static str,
    variant: &'static str,
    state: OpenState,
    trigger_label: &'static str,
    trigger_children: Vec<Node>,
    group_label: &'static str,
    items: Vec<Node>,
    create_label: &'static str,
    manage_label: &'static str,
) -> Node {
    let content_id = format!("blocks-settings-org-switcher-{kind}-content-{variant}");
    let label_id = format!("blocks-settings-org-switcher-{kind}-label-{variant}");

    let trigger = menu::trigger(
        state,
        true,
        Some(content_id.as_str()),
        vec![
            ("aria-label", trigger_label),
            ("data-blocks-settings-org-switcher-trigger", ""),
        ],
        trigger_children,
    );

    let mut group_children = vec![menu::item_group_label(
        Some(label_id.as_str()),
        vec![],
        vec![text(group_label)],
    )];
    group_children.extend(items);

    let content = menu::content(
        state,
        Some(content_id.as_str()),
        None,
        vec![("data-blocks-settings-org-switcher-content", "")],
        vec![
            menu::radio_item_group(Some(label_id.as_str()), vec![], group_children),
            menu::separator(vec![], vec![]),
            action_row(create_label, manage_label),
        ],
    );

    let positioner = menu::positioner(
        state,
        vec![("data-blocks-settings-org-switcher-positioner", "")],
        vec![content],
    );

    menu::root(
        Size::Md,
        state,
        vec![("data-blocks-settings-org-switcher-menu", kind)],
        vec![trigger, positioner],
    )
}

/// 組織メニュー（`variant` ごとに開閉状態・現在組織インデックスを変える）。
fn org_menu(variant: &'static str, state: OpenState) -> Node {
    let names = org_names();
    let items = (0..names.len())
        .map(|i| org_radio_row(i == 0, names[i], ORG_INITIALS[i], ORG_MEMBERS[i]))
        .collect();
    let trigger_children = vec![
        initials_avatar(ORG_INITIALS[0]),
        span(
            vec![("class", "blocks-settings-org-switcher-trigger-label")],
            vec![text(names[0])],
        ),
        badge(&BadgeProps::default(), vec![], vec![text(ORG_MEMBERS[0])]),
        chevron_icon(),
    ];
    menu_shell(
        "org",
        variant,
        state,
        "組織を切り替える",
        trigger_children,
        "組織",
        items,
        "組織を作成",
        "組織の設定",
    )
}

/// プロジェクトメニュー（`breadcrumb` variant 専用）。
fn project_menu(variant: &'static str, state: OpenState) -> Node {
    let items = PROJECT_NAMES
        .iter()
        .enumerate()
        .map(|(i, name)| project_radio_row(i == 0, name))
        .collect();
    let trigger_children = vec![
        span(
            vec![("class", "blocks-settings-org-switcher-trigger-label")],
            vec![text(PROJECT_NAMES[0])],
        ),
        chevron_icon(),
    ];
    menu_shell(
        "project",
        variant,
        state,
        "プロジェクトを切り替える",
        trigger_children,
        "プロジェクト",
        items,
        "プロジェクトを作成",
        "プロジェクトの設定",
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &'static str) -> Node {
    fandhe_frontend_core::p(
        vec![("class", "blocks-settings-org-switcher-caption")],
        vec![text(label)],
    )
}

/// `settings-org-switcher` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    let single = div(
        vec![("data-blocks-settings-org-switcher-variant", "single")],
        vec![org_menu("single", OpenState::Open)],
    );

    let breadcrumb = div(
        vec![
            ("data-blocks-settings-org-switcher-variant", "breadcrumb"),
            ("data-blocks-settings-org-switcher-breadcrumb", ""),
        ],
        vec![
            org_menu("breadcrumb", OpenState::Closed),
            span(
                vec![
                    ("aria-hidden", "true"),
                    ("class", "blocks-settings-org-switcher-slash"),
                ],
                vec![text("/")],
            ),
            project_menu("breadcrumb", OpenState::Open),
        ],
    );

    div(
        vec![("class", "blocks-settings-org-switcher-stack")],
        vec![
            caption("組織メニュー単体（single）"),
            single,
            caption("組織 / プロジェクト切替（breadcrumb）"),
            breadcrumb,
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-org-switcher/",
    title: "settings-org-switcher",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_org_switcher.rs",
    demo_class: "blocks-settings-org-switcher",
    parts: &[
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
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

/// `settings_org_switcher` 固有のレイアウト規則（`--fandhe-*` トークンのみ
/// 使用）。セレクタは `.blocks-settings-org-switcher-*` /
/// `[data-blocks-settings-org-switcher-*]`、および styled 部品の
/// `[data-scope][data-part]` セレクタとの複合セレクタのみを用いる。
const LAYOUT_CSS: &str = "\
.blocks-settings-org-switcher-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-settings-org-switcher-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-settings-org-switcher-variant] {\n  container-type: inline-size;\n  container-name: blocks-settings-org-switcher;\n}\n\
[data-blocks-settings-org-switcher-trigger] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"menu\"][data-part=\"trigger\"][data-blocks-settings-org-switcher-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-settings-org-switcher-action][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"menu\"][data-part=\"positioner\"][data-blocks-settings-org-switcher-positioner] {\n  position: static;\n  margin-block-start: var(--fandhe-space-2);\n}\n\
[data-scope=\"menu\"][data-part=\"content\"][data-blocks-settings-org-switcher-content] {\n  max-inline-size: 20rem;\n}\n\
[data-blocks-settings-org-switcher-actions] {\n  display: flex;\n  justify-content: flex-end;\n  gap: var(--fandhe-space-2);\n  padding: var(--fandhe-space-2);\n}\n\
[data-blocks-settings-org-switcher-breadcrumb] {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: flex-start;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-settings-org-switcher-slash {\n  padding-block-start: var(--fandhe-space-2);\n  color: var(--fandhe-color-fg-muted);\n}\n\
@container blocks-settings-org-switcher (max-width: 28rem) {\n  \
[data-blocks-settings-org-switcher-breadcrumb] {\n    flex-direction: column;\n    align-items: flex-start;\n  }\n  \
.blocks-settings-org-switcher-slash {\n    display: none;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// 5 部品の `data-scope` が揃い、`type="button"` があり、`<form>`・
    /// `href="#"`・`data:` src を持たないこと。
    #[test]
    fn demo_composes_expected_parts_and_has_no_form() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"menu\"",
            "data-scope=\"avatar\"",
            "data-scope=\"badge\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// `role="menu"` が 3 件（single の組織 + breadcrumb の組織・
    /// プロジェクト）描画され、うち 2 件が開いた状態（`data-state="open"`）
    /// であること。
    #[test]
    fn three_menus_render_with_two_open() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"role="menu""#).count(), 3);
        assert_eq!(
            html.matches(
                "data-scope=\"menu\" data-part=\"content\" role=\"menu\" data-state=\"open\""
            )
            .count(),
            2
        );
    }

    /// 現在組織・現在プロジェクトの radio item が `aria-checked="true"` を
    /// 持つこと（single 1 + breadcrumb の組織・プロジェクト 2 = 計 3）。
    #[test]
    fn current_org_and_project_are_checked() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"aria-checked="true""#).count(), 3);
    }

    /// `<form>`/`type="button"` 以外に無効化・非表示の不変条件（no-op な
    /// トリガー・ボタン、閉じた組織メニューの `hidden` 付き content）を
    /// 満たすこと。
    #[test]
    fn closed_org_menu_content_is_rendered_with_hidden() {
        let html = render(&demo());
        let content_id = "blocks-settings-org-switcher-org-content-breadcrumb";
        let pos = html
            .find(&format!(r#"id="{content_id}""#))
            .unwrap_or_else(|| panic!("closed org content id={content_id} should render"));
        let tag_start = html[..pos]
            .rfind("<div")
            .expect("content div should have an opening tag");
        let tag_end = html[tag_start..]
            .find('>')
            .map(|rel| tag_start + rel)
            .expect("content opening tag should close");
        let tag = &html[tag_start..tag_end];
        assert!(tag.contains("hidden"), "tag={tag}");
        assert!(html.contains(&format!(r#"aria-controls="{content_id}""#)));
    }

    /// `BLOCK.parts` の `path` が全件 kebab-case の `/themes/…/` であること。
    #[test]
    fn parts_point_to_themes_pages() {
        for part in super::BLOCK.parts {
            assert!(part.path.starts_with("/themes/"));
            assert!(part.path.ends_with('/'));
        }
    }

    /// [`LAYOUT_CSS`] が positioner を `position: static` へ中和し、
    /// disabled 中和・狭幅 `@container` の縦積みを持つこと。
    #[test]
    fn layout_css_has_static_positioner_and_container_query() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("position: static;"));
        assert!(LAYOUT_CSS.contains("@container blocks-settings-org-switcher (max-width: 28rem)"));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"menu\"][data-part=\"trigger\"][data-blocks-settings-org-switcher-trigger][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-scope=\"button\"][data-part=\"root\"][data-blocks-settings-org-switcher-action][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}"
        ));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-settings-org-switcher-stack\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-settings-org-switcher-stack"
        );
    }
}
