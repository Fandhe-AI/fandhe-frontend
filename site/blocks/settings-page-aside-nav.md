# settings-page-aside-nav

上段にアプリのナビバー、その下にページ見出し（アバター + 氏名/メール +
プランバッジ）、下段を「左ナビ + 本文」の 2 カラムへ分割する、設定ページの
骨格と主要領域です。`navigation-menu` / `nav-list` / `avatar` / `heading` /
`text` / `badge` / `card` / `field` / `input` / `input-group` /
`native-select` / `toggle-group` / `table` / `button` の 14 部品を合成しま
す。Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0656（代表構成）です。R0657（区切り線 3 セクション版）・
R0661（通知の配信先トグル）・R0662（積み上げ 4 カード + 削除）は本 PR の
対象外とし、後続イシューで追加します（`_/blocks-intake/` の対応ファイルは
本 worktree に存在しないため、対応表 ID のみを記載します）。

左ナビは「プロフィール」を現在項目（`aria-current="page"`）とし、本文には
プロフィールカード（表示名・ユーザー名・タイムゾーン・アバター変更）と
プランカード（請求周期の切替 + 利用状況テーブル）を縦に並べています。
デモ枠の幅が `48rem` 未満では、左ナビが本文の上へ横並び（折り返し）で移り
ます（コンテナクエリ判定、ページのビューポート幅では判定しません）。請求
周期の切替は無 JS では選択状態が変わらないため、操作不能を明示するため
disabled 表示にしています。本 Demo は無 JS の静的表示のみであり、`<form>`
を含みません。氏名・メールアドレスはすべて独自に書いた架空のものであり、
実在の人物・企業・クレデンシャル・PII を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::nav_list;
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::toggle_group::{self, ToggleGroupProps, ToggleGroupVariant};
use fandhe_frontend_pre_styled_ui::ColorPalette;

/// 実在の GitHub リポジトリへの固定外部 URL（`href="#"` 等の非実在リンクを
/// 避けるための方針、`navbar_two_row`/`footer_inline_nav` と同型）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// フィールド id の共通接頭辞を付ける小さなヘルパ（綴り間違い防止、
/// `contact_centered_form::field_id` と同型）。
fn field_id(suffix: &str) -> String {
    format!("settings-page-aside-nav-{suffix}")
}

/// 縦積み（label 上・control 下）の共通 orientation。
fn vertical() -> FieldRootProps {
    FieldRootProps {
        orientation: FieldOrientation::Vertical,
    }
}

/// ユーザー名 `input_group` の共通 props（無効化・不正値なし）。
/// [`InputGroupProps`] は `Default` を実装しないため、明示構築する
/// （`hero_install_command::instance_b` と同型の判断）。
fn username_group_props() -> InputGroupProps {
    InputGroupProps {
        disabled: false,
        invalid: false,
    }
}

/// アプリのナビバー（`navigation_menu`、trigger を持たないリンク項目
/// 4 件のみ。モジュール doc「構成」節 1.）。
fn app_navbar() -> Node {
    let props = NavigationMenuProps::default();
    let items: Vec<(&str, &str)> = vec![
        ("home", "ホーム"),
        ("projects", "プロジェクト"),
        ("reports", "レポート"),
        ("settings", "設定"),
    ];
    let children: Vec<Node> = items
        .into_iter()
        .map(|(value, label)| {
            navigation_menu::item(
                OpenState::Closed,
                false,
                &props,
                value,
                vec![],
                vec![navigation_menu::link(
                    REPO,
                    false,
                    vec![],
                    vec![text(label)],
                )],
            )
        })
        .collect();
    navigation_menu::root(
        &props,
        "アプリケーション",
        vec![("class", "blocks-settings-page-aside-nav-navbar")],
        vec![navigation_menu::list(&props, vec![], children)],
    )
}

/// ページ見出し（アバター + 氏名/メール + プランバッジ、モジュール doc
/// 「構成」節 2.）。
fn page_heading(name: &'static str, email: &'static str) -> Node {
    div(
        vec![("class", "blocks-settings-page-aside-nav-heading")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Lg,
                    ..AvatarProps::default()
                },
                vec![],
                vec![
                    avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, name, vec![]),
                    avatar::fallback(
                        ImageStatus::Loaded,
                        vec![],
                        vec![text(name.chars().take(1).collect::<String>())],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-settings-page-aside-nav-identity")],
                vec![
                    heading(
                        HeadingLevel::H2,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(name)],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(email)],
                    ),
                ],
            ),
            badge(
                &BadgeProps {
                    variant: BadgeVariant::Subtle,
                    ..BadgeProps::default()
                },
                vec![],
                vec![text("Pro プラン")],
            ),
        ],
    )
}

/// 左ナビ（設定項目、モジュール doc「構成」節 3.）。「プロフィール」だけ
/// 現在項目（`aria-current="page"`）とする
/// （`footer_inline_nav::primary_nav` と同型のリンク列組み立て）。
fn aside_nav() -> Node {
    let items: Vec<(&str, bool)> = vec![
        ("プロフィール", true),
        ("アカウント", false),
        ("プラン", false),
        ("通知", false),
        ("危険な操作", false),
    ];
    let children: Vec<Node> = items
        .into_iter()
        .map(|(label, current)| {
            nav_list::item(
                vec![],
                vec![nav_list::link(REPO, current, vec![], vec![text(label)])],
            )
        })
        .collect();
    nav_list::root(
        "設定項目",
        vec![("data-blocks-settings-page-aside-nav-aside", "")],
        vec![nav_list::list(vec![], children)],
    )
}

/// プロフィールカード（表示名・ユーザー名・タイムゾーン・アバター変更、
/// モジュール doc「構成」節 4.）。
fn profile_card() -> Node {
    let display_name_id = field_id("display-name");
    let username_id = field_id("username");
    let timezone_id = field_id("timezone");
    let display_name = FieldProps {
        id: &display_name_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let username = FieldProps {
        id: &username_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: true,
        readonly: false,
        has_helper_text: false,
    };
    let timezone = FieldProps {
        id: &timezone_id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };

    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-page-aside-nav-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("プロフィール")]),
                    card::description(
                        vec![],
                        vec![text("表示名・ユーザー名・タイムゾーンを設定します。")],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![div(
                    vec![("class", "blocks-settings-page-aside-nav-fields")],
                    vec![
                        field::root(
                            &vertical(),
                            &display_name,
                            vec![],
                            vec![
                                field::label(&display_name, vec![], vec![text("表示名")]),
                                input::input(
                                    &InputProps::default(),
                                    &display_name,
                                    vec![("type", "text"), ("value", "Kwame Boateng")],
                                ),
                            ],
                        ),
                        field::root(
                            &vertical(),
                            &username,
                            vec![],
                            vec![
                                field::label(&username, vec![], vec![text("ユーザー名")]),
                                input_group::root(
                                    &username_group_props(),
                                    vec![],
                                    vec![
                                        input_group::addon(
                                            InputGroupAlign::InlineStart,
                                            &username_group_props(),
                                            vec![],
                                            vec![input_group::text(vec![], vec![text("@")])],
                                        ),
                                        input::input(
                                            &InputProps::default(),
                                            &username,
                                            vec![("type", "text"), ("value", "kwame-boateng")],
                                        ),
                                    ],
                                ),
                            ],
                        ),
                        field::root(
                            &vertical(),
                            &timezone,
                            vec![],
                            vec![
                                field::label(&timezone, vec![], vec![text("タイムゾーン")]),
                                native_select::native_select(
                                    &NativeSelectProps::default(),
                                    &timezone,
                                    vec![],
                                    vec![
                                        select_option("Asia/Tokyo", "Asia/Tokyo（UTC+9）"),
                                        select_option("Europe/London", "Europe/London（UTC+0）"),
                                        select_option(
                                            "America/New_York",
                                            "America/New_York（UTC-5）",
                                        ),
                                    ],
                                ),
                            ],
                        ),
                        div(
                            vec![("class", "blocks-settings-page-aside-nav-avatar-row")],
                            vec![
                                avatar::root(
                                    &AvatarProps {
                                        size: Size::Sm,
                                        ..AvatarProps::default()
                                    },
                                    vec![],
                                    vec![
                                        avatar::image(
                                            ImageStatus::Loaded,
                                            dummy_assets::AVATAR_SRC,
                                            "Kwame Boateng",
                                            vec![],
                                        ),
                                        avatar::fallback(
                                            ImageStatus::Loaded,
                                            vec![],
                                            vec![text("K")],
                                        ),
                                    ],
                                ),
                                button(
                                    &ButtonProps {
                                        variant: ButtonVariant::Outline,
                                        size: Size::Sm,
                                        ..ButtonProps::default()
                                    },
                                    vec![],
                                    vec![text("画像を変更")],
                                ),
                            ],
                        ),
                    ],
                )],
            ),
            card::footer(
                vec![("class", "blocks-settings-page-aside-nav-actions")],
                vec![
                    button(&ButtonProps::default(), vec![], vec![text("保存")]),
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("キャンセル")],
                    ),
                ],
            ),
        ],
    )
}

/// `<option>` 1 件（`native_select` の子 `Node`）。
fn select_option(value: &str, label: &str) -> Node {
    fandhe_frontend_core::el("option", vec![("value", value)], vec![text(label)])
}

/// プランカード（請求周期 toggle-group + 利用状況 table、モジュール doc
/// 「構成」節 4.）。
fn plan_card() -> Node {
    let toggle_props = ToggleGroupProps {
        disabled: true,
        ..ToggleGroupProps::default()
    };
    let toggle = toggle_group::root_with_props(
        Size::Sm,
        ToggleGroupVariant::Outline,
        ColorPalette::Accent,
        &toggle_props,
        None,
        vec![("aria-label", "請求周期")],
        vec![
            toggle_group::item(
                &toggle_props,
                true,
                false,
                false,
                "monthly",
                vec![],
                vec![text("月払い")],
            ),
            toggle_group::item(
                &toggle_props,
                false,
                false,
                false,
                "yearly",
                vec![],
                vec![text("年払い")],
            ),
        ],
    );

    let usage_table = table::root(
        TableProps {
            variant: TableVariant::Outline,
            ..TableProps::default()
        },
        vec![("data-blocks-settings-page-aside-nav-table", "")],
        vec![
            table::caption(vec![], vec![text("利用状況")]),
            table::header(
                vec![],
                vec![table::row(
                    vec![],
                    vec![
                        table::column_header(vec![], vec![text("項目")]),
                        table::column_header(vec![], vec![text("現在")]),
                        table::column_header(vec![], vec![text("上限")]),
                    ],
                )],
            ),
            table::body(
                vec![],
                vec![
                    usage_row("プロジェクト数", "12", "50"),
                    usage_row("メンバー数", "6", "20"),
                    usage_row("ストレージ", "18 GB", "100 GB"),
                ],
            ),
        ],
    );

    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-page-aside-nav-card", "")],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    card::title(vec![], vec![text("プラン")]),
                    card::action(
                        vec![],
                        vec![badge(
                            &BadgeProps {
                                variant: BadgeVariant::Outline,
                                ..BadgeProps::default()
                            },
                            vec![],
                            vec![text("現在: Pro")],
                        )],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![div(
                    vec![("class", "blocks-settings-page-aside-nav-fields")],
                    vec![toggle, usage_table],
                )],
            ),
            card::footer(
                vec![("class", "blocks-settings-page-aside-nav-actions")],
                vec![button(
                    &ButtonProps::default(),
                    vec![],
                    vec![text("プランを変更")],
                )],
            ),
        ],
    )
}

/// 利用状況 table の 1 行（項目名 + 現在値 + 上限値）。
fn usage_row(label: &'static str, current: &'static str, limit: &'static str) -> Node {
    table::row(
        vec![],
        vec![
            table::row_header(vec![], vec![text(label)]),
            table::cell(vec![], vec![text(current)]),
            table::cell(vec![], vec![text(limit)]),
        ],
    )
}

/// `settings-page-aside-nav` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-page-aside-nav-stack")],
        vec![
            app_navbar(),
            page_heading("Kwame Boateng", "kwame.boateng@example.com"),
            div(
                vec![("class", "blocks-settings-page-aside-nav-layout")],
                vec![
                    aside_nav(),
                    div(
                        vec![("class", "blocks-settings-page-aside-nav-main")],
                        vec![profile_card(), plan_card()],
                    ),
                ],
            ),
        ],
    )
}
```

## 原案差分メモ

- 主参照は R0656（骨格の代表構成）です。R0657（区切り線 3 セクション版）・
  R0661（通知の配信先トグル）・R0662（積み上げ 4 カード + 削除）は本 PR の
  対象外とし、後続イシューで骨格・主要領域に追記する形で扱います。
- 左ナビ（`nav-list`）と本文の 2 カラムは `48rem` 未満のコンテナ幅で 1 列
  へ切り替わり、左ナビ自体も縦積みから横並び（折り返し）へ変わります。
  これは本 block 側の CSS が担っています（`nav-list`/`navigation-menu`
  部品自体の機能ではありません）。
- 請求周期（月払い/年払い）の `toggle-group` は無 JS では押しても選択状態
  が変わらないため、`disabled` 表示にして操作不能であることを明示して
  います。タイムゾーンの `native-select` はネイティブ `<select>` として
  通常どおり機能するため `disabled` にしていません。
- 実データ取得・保存・プラン変更・アバターアップロードの各処理は行わず、
  静的な初期状態のみを示します。氏名・メールアドレスは独自の架空データ
  です。
- ブラウザでの実機確認（`48rem` 前後の幅切替・ライト/ダーク両テーマ）は
  未実施です。`cargo test` による出力検証のみで代替しました。

関連情報: [Navigation Menu](../themes/navigation-menu.md) /
[Nav List](../themes/nav-list.md) / [Avatar](../themes/avatar.md) /
[Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Badge](../themes/badge.md) / [Card](../themes/card.md) /
[Field](../themes/field.md) / [Input](../themes/input.md) /
[Input Group](../themes/input-group.md) /
[Native Select](../themes/native-select.md) /
[Toggle Group](../themes/toggle-group.md) / [Table](../themes/table.md) /
[Button](../themes/button.md)
