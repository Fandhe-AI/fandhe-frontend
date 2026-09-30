# settings-page-tabs

上部にアプリ用ナビバー、その下にタブ状のセクション切替を持つページ見出し、
本文に設定カードを並べる合成例です。`navigation-menu` / `tab-nav` / `card` /
`table` / `badge` / `menu` / `input-group` / `input` / `button` の 9 部品を
合成します。Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0660（代表構成）、集約元は R0658 です。選択中タブ（API）は
無 JS のため初期状態として固定表示します。狭い幅ではタブ列を横スクロール
できます。API キーはすべて架空のマスク済みダミー値であり、実在サービスの鍵
形式・実在の人物・企業・PII は含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。

本 block は規模が大きいため 2 件（#3007/#3008）へ分割しています。本ページは
骨格・版 A（API 設定領域）を実装した #3007 時点の内容で、残りの版 B
（プラン・進捗・請求書テーブル + ページ送り）・状態並記・原案差分メモの仕上げ
は #3008 で追加します。

## Rust コード

```rust
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
```

## 原案差分メモ

- 骨格はナビバー（ダッシュボード / プロジェクト / 設定〔現在地〕+ 主操作
  ボタン 1 個）→ タブ見出し（一般 / メンバー / API〔現在地〕/ プラン / 請求）
  → 本文カード列、の順で構成します（親 issue のレイアウト要件どおり）。
- 版 A（API 設定）は「API アクセス」カード（マスク済みダミーキーの読み取り
  専用 `input_group` + 「再生成」ボタン）と「発行済み API キー」カード
  （`table::scroll_area` で包んだ表、行操作は `menu`、権限は `badge`）の
  2 枚で構成します。
- 狭い幅でのタブ横スクロールは、`tab-nav` root へ常時 `overflow-x: auto;
  flex-wrap: nowrap; white-space: nowrap;` を宣言することで実現します
  （`@container` によるコンテナクエリ分岐は不要です）。
- 版 B（プラン・進捗・請求書テーブル + ページ送り）・選択中タブ以外の状態
  並記・原案差分メモの仕上げは #3008 で追加します。

関連情報: [Navigation Menu](../themes/navigation-menu.md) /
[Tab Nav](../themes/tab-nav.md) / [Card](../themes/card.md) /
[Table](../themes/table.md) / [Badge](../themes/badge.md) /
[Menu](../themes/menu.md) / [Input Group](../themes/input-group.md) /
[Input](../themes/input.md) / [Button](../themes/button.md)
