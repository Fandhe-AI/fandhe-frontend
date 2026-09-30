# notification-tray-tabs

ベルボタンから開くポップオーバーの中にタブ（すべて・未読、件数バッジ付き）を
置き、タブごとの通知一覧を静的に並記した通知トレイのブロックです。`popover` /
`tabs` / `badge` / `button` / `avatar` / `menu` / `empty-state` /
`visually-hidden` / `icon` の 9 部品を合成します。Blocks は既存部品の合成例で
あり、新しい UI 部品は追加しません。

主参照は対応表 ID R0169（代表構成）で、R0168（空の状態）を Demo の 2 版並記
（空の状態版・代表構成版）で集約しています。人名は架空のデータであり、実在の
人物・企業・PII は含みません。アバター画像はビルド時生成の同梱プレースホルダー
SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。ポップオーバーは
開いた状態、操作メニューは閉じた状態で固定表示し、送信処理・送信先は一切持ち
ません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::menu;
use fandhe_frontend_pre_styled_ui::popover::{self, OpenState};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::tabs::{
    self, ActivationMode, Orientation, TabItem, TabsProps, TabsVariant,
};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// 自作の幾何アイコン（線画。モジュール doc「アイコンは自作の単純図形」
/// 節参照）。`path` へ `fill="none"` + `stroke="currentColor"` を明示し、
/// `icon` の `<svg>` 側が固定で持つ `fill="currentColor"`（塗り面）を
/// 上書きして線画（ストローク）として描画する。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
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

/// ベルアイコン（釣鐘形 + 下端の房）。
fn bell_icon() -> Node {
    geo_icon("M6 9a6 6 0 0 1 12 0v4l2 4H4l2-4z M10 20a2 2 0 0 0 4 0")
}

/// 縦 3 点アイコン（操作メニュー用）。
fn kebab_icon() -> Node {
    geo_icon("M12 5v.01 M12 12v.01 M12 19v.01")
}

/// 1 件の通知行（アバター + 本文 + 相対時刻。`unread` のとき未読ドットを
/// `data-unread` 経由の CSS 疑似要素で表示する）。
fn notification_item(
    name: &'static str,
    message: &'static str,
    when: &'static str,
    unread: bool,
) -> Node {
    let mut attrs: Vec<(&str, &str)> = vec![("class", "blocks-notification-tray-tabs-item")];
    if unread {
        attrs.push(("data-unread", ""));
    }
    div(
        attrs,
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Sm,
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
                vec![("class", "blocks-notification-tray-tabs-item-body")],
                vec![
                    text(format!("{name} が{message}")),
                    div(
                        vec![("class", "blocks-notification-tray-tabs-item-time")],
                        vec![text(when)],
                    ),
                ],
            ),
        ],
    )
}

/// 通知一覧が空のときの表示（R0168）。
fn empty_list() -> Node {
    div(
        vec![],
        vec![empty_state::root(
            &EmptyStateProps::default(),
            vec![],
            vec![empty_state::content(
                vec![],
                vec![
                    empty_state::title(vec![], vec![text("通知はありません")]),
                    empty_state::description(
                        vec![],
                        vec![text("新しい通知が届くとここに表示されます。")],
                    ),
                ],
            )],
        )],
    )
}

/// ヘッダー行の操作メニュー（`navbar_two_row::profile_menu` と同型。
/// 無 JS のため `disabled: true` 固定、`content_id` は `variant` ごとに
/// 一意にする）。
fn header_menu(variant: &str) -> Node {
    let content_id = format!("blocks-notification-tray-tabs-{variant}-menu");
    let trigger = menu::trigger(
        OpenState::Closed,
        true,
        Some(content_id.as_str()),
        vec![("aria-label", "通知の操作")],
        vec![kebab_icon()],
    );
    let content = menu::content(
        OpenState::Closed,
        Some(content_id.as_str()),
        None,
        vec![],
        vec![
            menu::item(
                "mark-all-read",
                false,
                false,
                vec![],
                vec![text("すべて既読にする")],
            ),
            menu::item("settings", false, false, vec![], vec![text("通知設定")]),
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

/// タブ 2 個（すべて・未読）+ 各 content を組む。`total`/`unread` は
/// トリガーのバッジ数値、`all_list`/`unread_list` は各タブの一覧。
#[allow(clippy::too_many_arguments)]
fn tab_group(variant: &str, total: u32, unread: u32, all_list: Node, unread_list: Node) -> Node {
    let tabs_id = format!("blocks-notification-tray-tabs-{variant}-tabs");
    let props = TabsProps {
        id: tabs_id.as_str(),
        selected: "all",
        orientation: Orientation::Horizontal,
        activation_mode: ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };
    let items = vec![
        TabItem {
            value: "all",
            trigger: vec![
                text("すべて "),
                badge::badge(
                    &BadgeProps::default(),
                    vec![],
                    vec![text(total.to_string())],
                ),
            ],
            content: vec![all_list],
            disabled: false,
        },
        TabItem {
            value: "unread",
            trigger: vec![
                text("未読 "),
                badge::badge(
                    &BadgeProps {
                        variant: BadgeVariant::Solid,
                        ..BadgeProps::default()
                    },
                    vec![],
                    vec![text(unread.to_string())],
                ),
            ],
            content: vec![unread_list],
            disabled: false,
        },
    ];
    tabs::tabs(
        TabsVariant::Line,
        Size::Sm,
        fandhe_frontend_pre_styled_ui::recipe::ColorPalette::default(),
        &props,
        items,
    )
}

/// 1 版分（`variant`: `"empty"`/`"filled"`）のトレイ全体を組み立てる。
/// `total`/`unread`/`all_list`/`unread_list` はタブ内訳（[`tab_group`]
/// へそのまま渡す）。
fn tray(variant: &str, total: u32, unread: u32, all_list: Node, unread_list: Node) -> Node {
    let content_id = format!("blocks-notification-tray-tabs-{variant}-content");
    let title_id = format!("blocks-notification-tray-tabs-{variant}-title");

    let mut bell_children = vec![bell_icon()];
    if unread > 0 {
        bell_children.push(badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Solid,
                size: Size::Sm,
                ..BadgeProps::default()
            },
            vec![("data-blocks-notification-tray-tabs-count", "")],
            vec![text(unread.to_string())],
        ));
    }
    bell_children.push(visually_hidden::root(vec![], vec![text("通知を開く")]));
    // ベルボタン自体は disabled 固定（無 JS で開閉できないため常に開状態
    // を表示する。header_menu と同じ判断）。
    let trigger = popover::trigger(
        OpenState::Open,
        true,
        Some(content_id.as_str()),
        vec![("class", "blocks-notification-tray-tabs-bell")],
        bell_children,
    );

    let header = div(
        vec![("class", "blocks-notification-tray-tabs-header")],
        vec![
            popover::title(Some(title_id.as_str()), vec![], vec![text("通知")]),
            header_menu(variant),
        ],
    );

    let footer = button(
        &ButtonProps {
            variant: ButtonVariant::Ghost,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        vec![],
        vec![text("すべての通知を見る")],
    );

    let content = popover::content(
        OpenState::Open,
        Some(content_id.as_str()),
        Some(title_id.as_str()),
        None,
        vec![("data-blocks-notification-tray-tabs-content", "")],
        vec![
            header,
            tab_group(variant, total, unread, all_list, unread_list),
            footer,
        ],
    );
    let positioner = popover::positioner(OpenState::Open, vec![], vec![content]);
    popover::root(OpenState::Open, vec![], vec![trigger, positioner])
}

/// A: 空の状態（R0168）。「すべて」「未読」いずれのタブも空表示。
fn version_empty() -> Node {
    tray("empty", 0, 0, empty_list(), empty_list())
}

/// B: 代表構成（R0169）。通知 4 件（うち未読 3 件）。
fn version_filled() -> Node {
    let names = dummy_assets::PERSON_NAMES;
    let all_list = div(
        vec![("class", "blocks-notification-tray-tabs-list")],
        vec![
            notification_item(names[0], "コメントしました", "5 分前", true),
            notification_item(
                names[1],
                "あなたをレビュアーに追加しました",
                "1 時間前",
                true,
            ),
            notification_item(names[2], "タスクを完了にしました", "3 時間前", true),
            notification_item(names[3], "ドキュメントを更新しました", "昨日", false),
        ],
    );
    let unread_list = div(
        vec![("class", "blocks-notification-tray-tabs-list")],
        vec![
            notification_item(names[0], "コメントしました", "5 分前", true),
            notification_item(
                names[1],
                "あなたをレビュアーに追加しました",
                "1 時間前",
                true,
            ),
            notification_item(names[2], "タスクを完了にしました", "3 時間前", true),
        ],
    );
    tray("filled", 4, 3, all_list, unread_list)
}

/// `notification-tray-tabs` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-notification-tray-tabs-stack")],
        vec![version_empty(), version_filled()],
    )
}
```

## 原案差分メモ

- **版 A（空の状態、R0168）**: 「すべて」「未読」いずれのタブ内も
  `empty-state` を表示します。
- **版 B（代表構成、R0169）**: 「すべて」タブに通知 4 件（うち未読 3 件）、
  「未読」タブに未読分のみを表示します。未読件数はタブのバッジと、ベル右上の
  バッジの両方で示します。
- `popover` の `positioner` は既定でオーバーレイ配置（`position: absolute`）
  ですが、本 Demo では掲示用に `position: static` へ中和し、常にフロー内へ
  表示しています。
- 狭幅（コンテナ幅 30rem 未満）ではポップオーバーの中身が枠幅いっぱいに
  広がります（`@container` によるコンテナクエリ判定）。
- タブは実物の `tabs` 部品（トリガー 2 個 + パネル切替）を使用しています。
  無 JS のため選択状態は初期値で固定表示です。
- ベルアイコン・操作メニューのアイコンはいずれも自作の線画（SVG path）で、
  参照元由来のアイコンセットではありません。

関連情報: [Popover](../themes/popover.md) / [Tabs](../themes/tabs.md) /
[Badge](../themes/badge.md) / [Button](../themes/button.md) /
[Avatar](../themes/avatar.md) / [Menu](../themes/menu.md) /
[Empty State](../themes/empty-state.md) /
[Visually Hidden](../themes/visually-hidden.md) / [Icon](../themes/icon.md)
