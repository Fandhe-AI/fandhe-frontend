# settings-page-aside-nav

上段にアプリのナビバー、その下にページ見出し（アバター + 氏名/メール +
プランバッジ）、下段を「左ナビ + 本文」の 2 カラムへ分割する、設定ページの
骨格と主要領域です。`navigation-menu` / `nav-list` / `avatar` / `heading` /
`text` / `badge` / `card` / `field` / `input` / `input-group` /
`native-select` / `toggle-group` / `table` / `button` / `checkbox` /
`separator` の 16 部品を合成します。Blocks は既存部品の合成例であり、新しい
UI 部品は追加しません。

Demo は版 A（積み上げ 4 カード）・版 B（区切り線 3 セクション）を単一
Demo 内に縦に並記しています（`_/blocks-intake/` の対応ファイルは本
worktree に存在しないため、対応表 ID のみを記載します）。

左ナビは「プロフィール」を視覚的に強調表示していますが、リンク先は実在の
GitHub 設定ページ（デモページ自身ではない）であるため `aria-current="page"`
は付けず、支援技術向けの現在地ラベルも付けません（強調は `data-*` 属性に
よる見た目のみ）。版 A の本文にはプロフィールカード（表示名・ユーザー名・
タイムゾーン・アバター変更）・プランカード（請求周期の切替 + 利用状況
テーブル）・通知カード（配信先 `checkbox` 3 件）・危険な操作カード（説明 +
削除ボタン）を縦に並べています。デモ枠の幅が `36rem` 未満では、左ナビが
本文の上へ横並び（折り返し）で移ります（コンテナクエリ判定、ページの
ビューポート幅では判定しません）。請求周期の切替・通知の配信先 checkbox は
無 JS では選択状態が変わらないため、操作不能を明示するため disabled 表示に
しています。本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。
氏名・メールアドレスはすべて独自に書いた架空のものであり、実在の人物・
企業・クレデンシャル・PII を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::input::{self, InputProps};
use fandhe_frontend_pre_styled_ui::input_group::{self, InputGroupAlign, InputGroupProps};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::nav_list;
use fandhe_frontend_pre_styled_ui::navigation_menu::{self, NavigationMenuProps, OpenState};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::toggle_group::{self, ToggleGroupProps, ToggleGroupVariant};
use fandhe_frontend_pre_styled_ui::ColorPalette;

/// 実在の GitHub リポジトリへの固定外部 URL（`href="#"` 等の非実在リンクを
/// 避けるための方針、`navbar_two_row`/`footer_inline_nav` と同型）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// アプリのナビバー 4 項目のラベルと遷移先。全項目が `REPO` 直下へ揃うと
/// ラベルと遷移先が食い違う（Codex レビュー指摘、`footer_inline_nav::
/// NAV_LINKS` と同じ是正）ため、ラベルの意味に対応する実在サブパスへ
/// 個別に張る。「設定」は当初リポジトリの `/settings`（管理権限を持つ
/// メンバーのみ閲覧できる）を指していたが、管理権限のない閲覧者には機能
/// しない（Codex レビュー指摘）ため、常にアクセス可能な GitHub 個人設定の
/// ランディングページへ変更した（`ASIDE_NAV_LINKS` の「プロフィール」と
/// 同一 URL になるが、両者は別ナビ内の別項目であり、いずれも「閲覧者
/// 自身が実際に開ける設定系ページ」という意味は保たれる）。
const NAVBAR_LINKS: &[(&str, &str, &str)] = &[
    ("home", "ホーム", REPO),
    (
        "projects",
        "プロジェクト",
        "https://github.com/Fandhe-AI/fandhe-frontend/projects",
    ),
    (
        "reports",
        "レポート",
        "https://github.com/Fandhe-AI/fandhe-frontend/pulse",
    ),
    ("settings", "設定", "https://github.com/settings/profile"),
];

/// 左ナビ（設定項目）5 件のラベルと遷移先。GitHub の実在する設定系ページ
/// （`github.com/settings/*`）へラベルの意味を合わせる。「危険な操作」は
/// 当初リポジトリの `/settings`（管理権限を持つメンバーのみ閲覧できる）を
/// 指していたが、管理権限のない閲覧者には機能しない（Codex レビュー指摘）
/// ため、GitHub 個人アカウント設定のうち誰でも自分のアカウントで開ける
/// `/settings/admin`（Account ページ、末尾に Delete account = Danger Zone
/// を含む）へ変更した。「プロフィール」の `/settings/profile` とは別の
/// 実在 URL であり、ラベルと遷移先が対応する（2 度目の Codex レビュー
/// 指摘: 「プロフィール」と同一 URL のままではラベルに対応する遷移先が
/// 無い）。現在項目（「プロフィール」）に `aria-current` を付けない理由・
/// 視覚的な現在地強調の方法は「左ナビの現在項目は `aria-current` を実在
/// 外部リンクへ付けない」節参照。
const ASIDE_NAV_LINKS: &[(&str, &str, bool)] = &[
    ("プロフィール", "https://github.com/settings/profile", true),
    ("アカウント", "https://github.com/settings/security", false),
    ("プラン", "https://github.com/settings/billing", false),
    ("通知", "https://github.com/settings/notifications", false),
    ("危険な操作", "https://github.com/settings/admin", false),
];

/// フィールド id の共通接頭辞を付ける小さなヘルパ（綴り間違い防止、
/// `contact_centered_form::field_id` と同型）。`id_prefix` は版 A/版 B の
/// `id` 衝突を避けるための追加接頭辞（モジュール doc「id の一意化」節
/// 参照）。
fn field_id(id_prefix: &str, suffix: &str) -> String {
    format!("settings-page-aside-nav-{id_prefix}-{suffix}")
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
    let children: Vec<Node> = NAVBAR_LINKS
        .iter()
        .map(|(value, label, href)| {
            navigation_menu::item(
                OpenState::Closed,
                false,
                &props,
                value,
                vec![],
                vec![navigation_menu::link(
                    href,
                    false,
                    vec![],
                    vec![text(*label)],
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
/// 現在項目とする（`footer_inline_nav::primary_nav` と同型のリンク列
/// 組み立て）が、`nav_list::link` の `current` 引数には常に `false` を渡し
/// `aria-current` は付けない。ここでの `highlighted` は「プロフィール」を
/// 視覚的に目立たせるだけの装飾フラグであり、`data-blocks-settings-page-
/// aside-nav-current` 属性（[`LAYOUT_CSS`]）のみに反映する。支援技術向け
/// の現在地ラベルは付けない（理由は「左ナビの現在項目は `aria-current` を
/// 実在外部リンクへ付けない」節参照。非表示テキストによる代替も同節の
/// 2 度目の指摘により不採用）。
fn aside_nav() -> Node {
    let children: Vec<Node> = ASIDE_NAV_LINKS
        .iter()
        .map(|(label, href, highlighted)| {
            let link_attrs = if *highlighted {
                vec![("data-blocks-settings-page-aside-nav-current", "")]
            } else {
                vec![]
            };
            nav_list::item(
                vec![],
                vec![nav_list::link(href, false, link_attrs, vec![text(*label)])],
            )
        })
        .collect();
    nav_list::root(
        "設定項目",
        vec![("data-blocks-settings-page-aside-nav-aside", "")],
        vec![nav_list::list(vec![], children)],
    )
}

/// プロフィールフィールド一式（表示名・ユーザー名・タイムゾーン・
/// アバター変更、モジュール doc「構成」節 4.）。版 A（`profile_card`）・
/// 版 B（[`variant_b_main`]）の双方から `id_prefix` を変えて呼ばれる
/// （モジュール doc「id の一意化」節参照）。
fn profile_fields(id_prefix: &str) -> Vec<Node> {
    let display_name_id = field_id(id_prefix, "display-name");
    let username_id = field_id(id_prefix, "username");
    let timezone_id = field_id(id_prefix, "timezone");
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
                        // Asia/Tokyo は夏時間を持たないため固定オフセット表記で
                        // 通年正しいが、Europe/London・America/New_York は夏時間で
                        // オフセットが変動する（London: UTC+0/+1、New_York:
                        // UTC-5/-4）ため固定オフセットを付けない
                        // （Codex レビュー指摘: 通年不正確な固定表記の是正）。
                        select_option("Asia/Tokyo", "Asia/Tokyo（UTC+9）"),
                        select_option("Europe/London", "Europe/London"),
                        select_option("America/New_York", "America/New_York"),
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
                        avatar::fallback(ImageStatus::Loaded, vec![], vec![text("K")]),
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
    ]
}

/// プロフィールカード（版 A、モジュール doc「構成」節 4.）。
fn profile_card() -> Node {
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
                    profile_fields("a"),
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

/// 版見出し（H3、`heading` 部品。`settings_integrations_list::version_title`
/// と同型。TOC 混入回避のため専用 `data-*` 属性を付ける）。
fn version_title(label: &str) -> Node {
    heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Sm,
            ..HeadingProps::default()
        },
        vec![("data-blocks-settings-page-aside-nav-version-title", "")],
        vec![text(label)],
    )
}

/// 通知配信先 checkbox 1 件（モジュール doc「通知 checkbox をネイティブ
/// disabled にする理由」節参照）。`id_prefix` は版 A/版 B の `id` 一意化用。
fn notification_checkbox(
    id_prefix: &str,
    suffix: &str,
    label: &'static str,
    checked: bool,
) -> Node {
    let props = CheckboxProps {
        checked: if checked {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        },
        disabled: true,
        ..CheckboxProps::default()
    };
    let id = format!("settings-page-aside-nav-{id_prefix}-notification-{suffix}");
    checkbox::root(
        Size::Md,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-settings-page-aside-nav-checkbox", "")],
        vec![
            checkbox::hidden_input(
                &props,
                "notification-channel",
                "on",
                vec![("id", id.as_str())],
            ),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
            checkbox::label(&props, vec![], vec![text(label)]),
        ],
    )
}

/// 通知の配信先 checkbox 3 件（メール・ブラウザ通知はチェック済み、
/// モバイルプッシュは未チェック。状態並記の意図は `checked: bool` 引数に
/// 明示する）。`role="group"` + `aria-label` で checkbox 群をひとまとまりの
/// フォームコントロール群として支援技術へ伝える（`fieldset` は本 block の
/// 使用部品外のため使わない）。
fn notification_fields(id_prefix: &str) -> Node {
    div(
        vec![
            ("class", "blocks-settings-page-aside-nav-checkboxes"),
            ("role", "group"),
            ("aria-label", "通知の配信先"),
        ],
        vec![
            notification_checkbox(id_prefix, "email", "メール", true),
            notification_checkbox(id_prefix, "browser", "ブラウザ通知", true),
            notification_checkbox(id_prefix, "push", "モバイルプッシュ", false),
        ],
    )
}

/// 通知カード（版 A、モジュール doc「構成」節 4.）。
fn notification_card() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-page-aside-nav-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text("通知")]),
                    card::description(vec![], vec![text("通知を受け取る配信先を選択します。")]),
                ],
            ),
            card::body(vec![], vec![notification_fields("a")]),
            card::footer(
                vec![("class", "blocks-settings-page-aside-nav-actions")],
                vec![button(&ButtonProps::default(), vec![], vec![text("保存")])],
            ),
        ],
    )
}

/// 危険な操作の説明 + 削除ボタン（版 A・版 B 共有。`id` を持たないため
/// `id_prefix` は不要）。削除ボタンは `ButtonVariant::Outline` +
/// `ColorPalette::Danger` で危険操作であることを視覚的に示す。
fn danger_body() -> Vec<Node> {
    vec![
        styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                size: TextSize::Sm,
                ..TextProps::default()
            },
            vec![],
            vec![text(
                "アカウントを削除すると、すべてのデータが完全に失われ元に戻せません。",
            )],
        ),
        button(
            &ButtonProps {
                variant: ButtonVariant::Outline,
                palette: ColorPalette::Danger,
                ..ButtonProps::default()
            },
            vec![("data-blocks-settings-page-aside-nav-danger", "")],
            vec![text("アカウントを削除")],
        ),
    ]
}

/// 危険な操作カード（版 A、モジュール doc「構成」節 4.）。
fn danger_card() -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-page-aside-nav-card", "")],
        vec![
            card::header(vec![], vec![card::title(vec![], vec![text("危険な操作")])]),
            card::body(
                vec![("class", "blocks-settings-page-aside-nav-fields")],
                danger_body(),
            ),
        ],
    )
}

/// 版 B（区切り線 3 セクション版）の 1 セクション（見出し + 本文）。
/// カード枠を持たない素の `div`/`heading` で組み立てる（モジュール doc
/// 「版 A / 版 B と集約元の対応」節参照）。
fn plain_section(title: &str, body: Vec<Node>) -> Node {
    let mut children = vec![heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Sm,
            ..HeadingProps::default()
        },
        vec![],
        vec![text(title)],
    )];
    children.extend(body);
    div(
        vec![("class", "blocks-settings-page-aside-nav-plain-section")],
        children,
    )
}

/// 版 B の本文一式（プロフィール / 通知 / 危険な操作の 3 セクションを
/// `separator` 2 本で区切る、モジュール doc「`separator` は版 B のみで
/// 使う」節参照）。
fn variant_b_main() -> Vec<Node> {
    vec![
        plain_section(
            "プロフィール",
            vec![div(
                vec![("class", "blocks-settings-page-aside-nav-fields")],
                profile_fields("b"),
            )],
        ),
        separator(&SeparatorProps::default(), vec![]),
        plain_section("通知", vec![notification_fields("b")]),
        separator(&SeparatorProps::default(), vec![]),
        plain_section(
            "危険な操作",
            vec![div(
                vec![("class", "blocks-settings-page-aside-nav-fields")],
                danger_body(),
            )],
        ),
    ]
}

/// `settings-page-aside-nav` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。版 A（積み上げ 4 カード）・版 B（区切り線 3 セクション）を
/// 縦に並記する（モジュール doc「版 A / 版 B と集約元の対応」節参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-page-aside-nav-stack")],
        vec![
            app_navbar(),
            page_heading("Kwame Boateng", "kwame.boateng@example.com"),
            version_title("版 A: 積み上げ 4 カード（R0656 / R0661 / R0662）"),
            div(
                vec![("class", "blocks-settings-page-aside-nav-layout")],
                vec![
                    aside_nav(),
                    div(
                        vec![("class", "blocks-settings-page-aside-nav-main")],
                        vec![
                            profile_card(),
                            plan_card(),
                            notification_card(),
                            danger_card(),
                        ],
                    ),
                ],
            ),
            version_title("版 B: 区切り線 3 セクション（R0657）"),
            div(
                vec![("class", "blocks-settings-page-aside-nav-variant-b")],
                variant_b_main(),
            ),
        ],
    )
}
```

## 原案差分メモ

対応表 ID 4 件を単一 Demo 内の版 A・版 B へ対応付けています。

- **R0656（骨格の代表構成）**: 版 A の「左ナビ + 本文」2 カラム骨格・
  ナビバー・ページ見出しに対応します。
- **R0661（通知の配信先トグル）**: 版 A の通知カードの `checkbox` 3 件
  （メール・ブラウザ通知はチェック済み、モバイルプッシュは未チェック）に
  対応します。状態の混在をそのまま静的表示することで「トグル可能な複数
  項目」であることを示します。
- **R0662（積み上げ 4 カード + 削除）**: 版 A の本文をプロフィール/プラン/
  通知/危険な操作の 4 カード積み上げへ拡張し、危険な操作カードに削除ボタン
  （`ButtonVariant::Outline` + `ColorPalette::Danger`）を置くことで対応
  します。
- **R0657（区切り線 3 セクション版）**: 版 A の直下に版 B として、カード枠
  を持たずプロフィール/通知/危険な操作の 3 セクションを `separator` 2 本で
  区切った構成を並記して対応します。左ナビは版 B では再掲しません（`nav`
  ランドマークの重複と現在項目マーカーの二重化を避けるため）。
- 左ナビ（`nav-list`）と本文の 2 カラムは `36rem` 未満のコンテナ幅で 1 列
  へ切り替わり、左ナビ自体も縦積みから横並び（折り返し）へ変わります。
  これは本 block 側の CSS が担っています（`nav-list`/`navigation-menu`
  部品自体の機能ではありません）。
- 請求周期（月払い/年払い）の `toggle-group`・通知の配信先 `checkbox` は
  無 JS では押しても選択状態が変わらないため、`disabled` 表示にして操作
  不能であることを明示しています。タイムゾーンの `native-select` は
  ネイティブ `<select>` として通常どおり機能するため `disabled` にして
  いません。
- 実データ取得・保存・プラン変更・アバターアップロード・アカウント削除の
  各処理は行わず、静的な初期状態のみを示します。氏名・メールアドレス・
  通知内容はすべて独自の架空データです。
- ブラウザでの実機確認（`36rem` 前後の幅切替・ライト/ダーク両テーマ）は
  未実施です。`cargo test` による出力検証のみで代替しました。

関連情報: [Navigation Menu](../themes/navigation-menu.md) /
[Nav List](../themes/nav-list.md) / [Avatar](../themes/avatar.md) /
[Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Badge](../themes/badge.md) / [Card](../themes/card.md) /
[Field](../themes/field.md) / [Input](../themes/input.md) /
[Input Group](../themes/input-group.md) /
[Native Select](../themes/native-select.md) /
[Toggle Group](../themes/toggle-group.md) / [Table](../themes/table.md) /
[Button](../themes/button.md) / [Checkbox](../themes/checkbox.md) /
[Separator](../themes/separator.md)
