//! `settings-page-tabs` block（イシュー #3007。親トラッキング #3006
//! 「Blocks に settings-page-tabs を追加する」配下。#3006 は規模 L のため
//! 骨格・主要領域・登録一式を #3007（本ファイル）、残り領域（版 B: プラン・
//! 進捗・請求書テーブル + ページ送り）・状態並記・原案差分メモの仕上げを
//! #3008 へ分割している。対応表 ID は主参照 R0660（代表構成）、集約元
//! R0658。`_/blocks-intake/` の対応ファイルはメイン worktree に存在しない
//! ため、`profile-detail-datalist`（#2937）と同じ扱いとし、issue のレイ
//! アウト仕様のみから実装する（参照元の再現は行わない）。
//!
//! # 構成（ナビバー → タブ見出し → 本文カード列）
//!
//! 上部に 1 段のアプリ用ナビバー、その下にタブ状のセクション切替を持つ
//! ページ見出し、本文に設定カードを縦に並べる。無 JS のため選択中タブ
//! （「API」）を初期状態として固定表示する。狭幅ではタブ列を横スクロール
//! させる（下記「狭幅ではタブを横スクロールする」節）。
//!
//! # 使用部品（#3007 時点）
//!
//! `navigation-menu` / `tab-nav` / `card` / `table` / `badge` / `menu` /
//! `input-group` / `input` / `button` の 9 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。版 B で追加する `progress` /
//! `checkbox` / `pagination` は #3008 で `parts` へ加える（未使用部品を
//! `parts` に含めない。単体テストが `data-scope` の実出現を検証するため）。
//! 新しい UI 部品は追加しない。
//!
//! # ナビバー（`navigation_menu` + `button`）
//!
//! [`super::super::navbar::navbar_app_links`] と同じ語彙で、ブランド文言 +
//! メインナビ（ダッシュボード / プロジェクト / 設定〔現在地〕）+ 右端に
//! 1 個の Ghost/Sm ボタンを置く最小構成にする（本 block の主眼はタブ以下の
//! 設定領域のため、ナビバー自体はハンバーガー折り畳み等を持たない単純な
//! 1 段バーとする）。
//!
//! # タブ見出し（`tab_nav`）
//!
//! `tab_nav::root` の `aria-label` は本 block 内で一意にする。「API」タブへ
//! `aria-current="page"` を固定し、無 JS のため他タブは押しても遷移しない
//! （`href` の方針はモジュール doc「`href` は現在地が `./`・他は自リポジトリ
//! の実在 URL」節参照）。
//!
//! # 狭幅ではタブを横スクロールする
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container` を使わず
//! ネイティブ CSS のみで常時 `overflow-x: auto` を宣言する（親 issue の
//! 「狭幅ではタブを横スクロール」要件はいつでも横スクロール可能であれば
//! 満たされるため、コンテナクエリでの分岐は不要）。[`LAYOUT_CSS`] が
//! `data-blocks-settings-page-tabs-tabs` 付きの `tab-nav` root へ
//! `overflow-x: auto; flex-wrap: nowrap; white-space: nowrap;` を宣言する。
//!
//! # 版 A: API 設定（`card` + `input_group` + `table` + `menu` + `badge`）
//!
//! 1 枚目のカード「API アクセス」は読み取り専用のマスク済みダミーキーを
//! 表示する `input_group`（末尾に「再生成」ボタンの addon）を持つ。2 枚目の
//! カード「発行済み API キー」は `table::scroll_area` で包んだ表（列: 名前
//! 〔`row_header`〕/ 先頭文字列 / 権限〔`badge`〕/ 最終使用 / 操作
//! 〔`menu`〕）を持つ。`menu` の `content_id` は行ごとに一意にする
//! （`blocks-settings-page-tabs-key-menu-{n}`、
//! `blocks_contract.rs::demo_output_has_no_dangling_aria_references_or_duplicate_ids`
//! の対象）。
//!
//! # `class` と `data-*` の使い分け
//!
//! `navigation_menu`/`tab_nav`/`card`/`table`/`menu`/`button`/`badge`/
//! `input_group`/`input` はいずれも `drop_class_attr` で呼び出し側
//! `class` を除去するため、これらへの CSS フックは
//! `data-blocks-settings-page-tabs-*` 属性で渡す。素の `<div>`/`<header>`
//! ラッパーのみ `class="blocks-settings-page-tabs-*"` を使う
//! （`navbar_app_links`/`profile_detail_datalist` と同型の判断）。
//!
//! # `href` は現在地が `./`・他は自リポジトリの実在 URL
//!
//! docs サイトの link check（`crate::linkcheck`）は絶対パス（`/settings` 等）
//! を実在ページとして検証するため、サイトに存在しない架空パスは使えない
//! （`navbar_app_links.rs`・`comparison_table.rs` 等の既存 block も同じ制約
//! に従う）。現在地リンク（ナビの「設定」・タブの「API」）は `href="./"`
//! （自ページを指す相対パスで常に有効）、他のリンクは `navbar_app_links.rs`
//! と同じ自リポジトリ・自組織の実在 URL（`REPO`/`ORG`）を使う。`href="#"`・
//! `data:` URI は持ち込まない。
//!
//! # アイコンは自作の線画
//!
//! `icon::icon` + `el("path", …)` の自作線画（`fill="none"` /
//! `stroke="currentColor"`）のみで構成する。装飾用途は
//! `IconProps { label: None, .. }`（`aria-hidden`）とし、可視テキストが
//! アクセシブルネームを担う。実在ブランドのロゴ・商標は持ち込まない。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。全ボタンは
//! `button::button`/`button::icon_button` の既定 `type="button"` のまま
//! 用いる。API キーは明らかなダミー値（マスク表記）で、実在サービスの鍵
//! 形式・実在の人物・企業・PII は含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, header, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::field::{FieldIds, FieldProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps};
use fandhe_frontend_pre_styled_ui::tab_nav;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::Size;

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc参照）。
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
        vec![el(
            "path",
            vec![
                ("d", d),
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

/// 再生成アイコン（円弧 + 矢印）。
fn refresh_icon() -> Node {
    geo_icon("M20 12a8 8 0 1 1-2.34-5.66 M20 4v5h-5")
}

/// 縦 3 点の行操作アイコン（`menu` トリガーの視覚的ラベル、アクセシブル
/// ネームは `aria-label` が担うため装飾用途）。
fn kebab_icon() -> Node {
    geo_icon("M12 6v.01 M12 12v.01 M12 18v.01")
}

/// ロゴ（幾何図形 + ブランド名テキスト）。`navbar_app_links.rs` の
/// `logo()` と同型。
fn logo() -> Node {
    span(
        vec![("data-blocks-settings-page-tabs-logo", "")],
        vec![
            geo_icon("M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z"),
            span(vec![], vec![text("Fandhe Console")]),
        ],
    )
}

/// 上部のアプリ用ナビバー（ダッシュボード / プロジェクト / 設定〔現在地〕
/// + 右端の Ghost/Sm ボタン 1 個）。
fn navbar() -> Node {
    let props = NavigationMenuProps::default();
    let nav_node = navigation_menu::root(
        &props,
        "メインナビゲーション",
        vec![("data-blocks-settings-page-tabs-nav", "")],
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
                        REPO,
                        false,
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
                        ORG,
                        false,
                        vec![],
                        vec![text("プロジェクト")],
                    )],
                ),
                navigation_menu::item(
                    navigation_menu::OpenState::Closed,
                    false,
                    &props,
                    "settings",
                    vec![],
                    vec![navigation_menu::link(
                        "./",
                        true,
                        vec![],
                        vec![text("設定")],
                    )],
                ),
            ],
        )],
    );
    header(
        vec![("class", "blocks-settings-page-tabs-navbar")],
        vec![
            logo(),
            nav_node,
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-settings-page-tabs-navbar-cta", "")],
                vec![text("新規プロジェクト")],
            ),
        ],
    )
}

/// タブ見出し（heading「設定」+ タブ列。「API」タブを初期状態として固定
/// 表示する）。
fn page_heading() -> Node {
    let tabs = tab_nav::root(
        Size::Md,
        "設定セクション",
        vec![("data-blocks-settings-page-tabs-tabs", "")],
        vec![
            tab_nav::link(REPO, false, vec![], vec![text("一般")]),
            tab_nav::link(ORG, false, vec![], vec![text("メンバー")]),
            tab_nav::link("./", true, vec![], vec![text("API")]),
            tab_nav::link(REPO, false, vec![], vec![text("プラン")]),
            tab_nav::link(ORG, false, vec![], vec![text("請求")]),
        ],
    );
    div(
        vec![("class", "blocks-settings-page-tabs-heading")],
        vec![
            heading(
                HeadingLevel::H2,
                &HeadingProps::default(),
                vec![],
                vec![text("設定")],
            ),
            tabs,
        ],
    )
}

/// 「API アクセス」カード（マスク済みダミーキーの読み取り専用
/// `input_group` + 「再生成」ボタン）。
fn api_access_card() -> Node {
    let field_id = "blocks-settings-page-tabs-access-key";
    let field_props = FieldProps {
        id: field_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: true,
        has_helper_text: false,
    };
    let group_props = InputGroupProps {
        disabled: false,
        invalid: false,
    };
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-page-tabs-access-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("API アクセス")]),
                    card::description(vec![], vec![text("サーバー間連携に使うアクセスキーです。")]),
                ],
            ),
            card::body(
                vec![],
                vec![input_group::root(
                    &group_props,
                    vec![("data-blocks-settings-page-tabs-access-group", "")],
                    vec![
                        input::input(
                            &InputProps::default(),
                            &field_props,
                            vec![
                                ("aria-label", "アクセスキー"),
                                ("value", "sk_live_••••••••4f2a"),
                            ],
                        ),
                        input_group::addon(
                            InputGroupAlign::InlineEnd,
                            &group_props,
                            vec![],
                            vec![button::button(
                                &ButtonProps {
                                    variant: ButtonVariant::Outline,
                                    size: Size::Sm,
                                    ..ButtonProps::default()
                                },
                                vec![],
                                vec![refresh_icon(), text("再生成")],
                            )],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// 発行済み API キー 1 行分。`content_id` は行ごとに一意な `menu` の
/// `content_id`（`aria-controls` の宙吊り防止、モジュール doc参照）。
struct ApiKeyRow {
    name: &'static str,
    prefix: &'static str,
    scope: &'static str,
    last_used: &'static str,
}

/// 発行済み API キーの操作メニュー（無 JS のため `disabled: true` 固定）。
fn key_actions_menu(content_id: &str) -> Node {
    let trigger = menu::trigger(
        OpenState::Closed,
        true,
        Some(content_id),
        vec![
            ("aria-label", "行の操作"),
            ("data-blocks-settings-page-tabs-key-trigger", ""),
        ],
        vec![kebab_icon()],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id),
        None,
        vec![],
        vec![
            menu::item("rename", false, false, vec![], vec![text("名前を変更")]),
            menu::item("revoke", false, false, vec![], vec![text("失効させる")]),
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

/// 「発行済み API キー」カード（`table::scroll_area` で包んだ表）。
fn api_keys_table_card() -> Node {
    let rows: [ApiKeyRow; 3] = [
        ApiKeyRow {
            name: "本番サーバー",
            prefix: "sk_live_",
            scope: "読み取り/書き込み",
            last_used: "2026-09-28",
        },
        ApiKeyRow {
            name: "CI パイプライン",
            prefix: "sk_ci_",
            scope: "読み取りのみ",
            last_used: "2026-09-25",
        },
        ApiKeyRow {
            name: "検証環境",
            prefix: "sk_test_",
            scope: "読み取り/書き込み",
            last_used: "未使用",
        },
    ];
    let body_rows: Vec<Node> = rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let content_id = format!("blocks-settings-page-tabs-key-menu-{i}");
            table::row(
                vec![],
                vec![
                    table::row_header(vec![], vec![text(row.name)]),
                    table::cell(vec![], vec![text(row.prefix)]),
                    table::cell(
                        vec![],
                        vec![badge::badge(
                            &BadgeProps {
                                variant: BadgeVariant::Subtle,
                                ..BadgeProps::default()
                            },
                            vec![],
                            vec![text(row.scope)],
                        )],
                    ),
                    table::cell(vec![], vec![text(row.last_used)]),
                    table::cell(vec![], vec![key_actions_menu(&content_id)]),
                ],
            )
        })
        .collect();

    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-page-tabs-keys-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("発行済み API キー")]),
                    card::action(
                        vec![],
                        vec![button::button(
                            &ButtonProps {
                                size: Size::Sm,
                                ..ButtonProps::default()
                            },
                            vec![],
                            vec![text("キーを発行")],
                        )],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![table::scroll_area(
                    vec![
                        ("role", "region"),
                        ("aria-label", "発行済み API キー一覧"),
                        ("tabindex", "0"),
                    ],
                    vec![table::root(
                        TableProps::default(),
                        vec![],
                        vec![
                            table::header(
                                vec![],
                                vec![table::row(
                                    vec![],
                                    vec![
                                        table::column_header(vec![], vec![text("名前")]),
                                        table::column_header(vec![], vec![text("先頭文字列")]),
                                        table::column_header(vec![], vec![text("権限")]),
                                        table::column_header(vec![], vec![text("最終使用")]),
                                        table::column_header(vec![], vec![text("操作")]),
                                    ],
                                )],
                            ),
                            table::body(vec![], body_rows),
                        ],
                    )],
                )],
            ),
        ],
    )
}

/// `settings-page-tabs` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（骨格 + 版 A のみ、#3008 で版 B を追加する）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-page-tabs-stack")],
        vec![
            navbar(),
            page_heading(),
            div(
                vec![("class", "blocks-settings-page-tabs-content")],
                vec![api_access_card(), api_keys_table_card()],
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/settings-page-tabs/",
    title: "settings-page-tabs",
    category: BlockCategory::Settings,
    rust_source: "crates/docs-site/src/blocks/application/settings/settings_page_tabs.rs",
    demo_class: "blocks-settings-page-tabs",
    parts: &[
        Part {
            label: "Navigation Menu",
            path: "/themes/navigation-menu/",
        },
        Part {
            label: "Tab Nav",
            path: "/themes/tab-nav/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Table",
            path: "/themes/table/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
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
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `settings_page_tabs` 固有のレイアウト規則（`--fandhe-*` トークンのみ
/// 使用）。セレクタは `.blocks-settings-page-tabs-*` /
/// `[data-blocks-settings-page-tabs-*]`、および styled 部品の
/// `[data-scope][data-part]` セレクタとの複合セレクタのみを用いる
/// （`navbar_app_links` と同型の判断）。
const LAYOUT_CSS: &str = "\
.blocks-settings-page-tabs-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-settings-page-tabs;\n}\n\
.blocks-settings-page-tabs-navbar {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n  padding-block: var(--fandhe-space-3);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
[data-blocks-settings-page-tabs-logo] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  white-space: nowrap;\n}\n\
[data-blocks-settings-page-tabs-navbar-cta] {\n  margin-inline-start: auto;\n}\n\
.blocks-settings-page-tabs-heading {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"tab-nav\"][data-part=\"root\"][data-blocks-settings-page-tabs-tabs] {\n  overflow-x: auto;\n  flex-wrap: nowrap;\n  white-space: nowrap;\n}\n\
.blocks-settings-page-tabs-content {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-settings-page-tabs-access-group] {\n  max-inline-size: 28rem;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// 9 部品の `data-scope` が全て出現し、`type="button"` があり、
    /// `<form>`・`href="#"`・`<script`・`src="data:` を含まないこと。
    #[test]
    fn demo_composes_expected_parts_and_has_no_form() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"navigation-menu\"",
            "data-scope=\"tab-nav\"",
            "data-scope=\"card\"",
            "data-scope=\"table\"",
            "data-scope=\"badge\"",
            "data-scope=\"menu\"",
            "data-scope=\"input-group\"",
            "data-scope=\"field\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"type="button""#));
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
    }

    /// `aria-current="page"` がタブ内でちょうど 1 回、ナビゲーション
    /// メニュー内でちょうど 1 回（初期状態固定）出現すること。
    #[test]
    fn exactly_one_tab_and_one_nav_item_are_current() {
        let html = render(&demo());
        assert_eq!(html.matches(r#"aria-current="page""#).count(), 2);
        assert_eq!(html.matches(r#"href="./""#).count(), 2);
    }

    /// menu の `content_id` が行数分すべて相異なり、`aria-controls` の
    /// 値と同数存在すること（宙吊り ARIA 参照・id 重複防止）。
    #[test]
    fn menu_content_ids_are_unique_and_referenced() {
        let html = render(&demo());
        for i in 0..3 {
            let content_id = format!("blocks-settings-page-tabs-key-menu-{i}");
            assert_eq!(
                html.matches(&format!(r#"id="{content_id}""#)).count(),
                1,
                "content_id {content_id} should appear exactly once"
            );
            assert!(html.contains(&format!(r#"aria-controls="{content_id}""#)));
        }
    }

    /// `LAYOUT_CSS` に `<` が無く、横スクロール・コンテナ宣言を含むこと。
    #[test]
    fn layout_css_has_no_breakout_and_has_scroll_rules() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("overflow-x: auto;"));
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-settings-page-tabs-stack\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-settings-page-tabs-stack");
    }
}
