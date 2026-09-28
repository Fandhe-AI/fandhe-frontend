# profile-detail-datalist

アバター・氏名・役職・操作ボタン群と、連絡先・経歴の定義リスト（data-list）を
組み合わせたプロフィール詳細ブロックです。`avatar` / `heading` / `text` /
`button` / `data-list` / `badge` / `icon` の 7 部品を合成します。Blocks は
既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0219（代表構成: 操作ボタン 2 個 + 縦積みの定義リスト）で、
R0221（操作ボタン 4 個 + 横並びの定義リスト）を集約しています。氏名・役職・
社名・連絡先はすべて架空のデータであり、実在の人物・企業・PII は含みません。
アバター画像はビルド時生成の同梱プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

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

/// 電話アイコン（受話器: 角丸の折れ線）。
fn phone_icon() -> Node {
    geo_icon("M6 4c-1 0-2 1-2 2 0 8 6 14 14 14 1 0 2-1 2-2v-3l-4-1-2 2c-2-1-4-3-5-5l2-2-1-4z")
}

/// メールアイコン（封筒 + V 字の折り返し線）。
fn mail_icon() -> Node {
    geo_icon("M3 6h18v12H3z M3 7l9 6 9-6")
}

/// カレンダーアイコン（角丸長方形 + 上端の 2 本のタブ）。
fn calendar_icon() -> Node {
    geo_icon("M4 5h16v16H4z M4 9h16 M8 3v4 M16 3v4")
}

/// 共有アイコン（3 節点 + 接続線）。
fn share_icon() -> Node {
    geo_icon(
        "M6 12a2 2 0 1 0 0-4 2 2 0 0 0 0 4z M18 6a2 2 0 1 0 0-4 2 2 0 0 0 0 4z \
         M18 22a2 2 0 1 0 0-4 2 2 0 0 0 0 4z M8 11l8-5 M8 13l8 5",
    )
}

/// アイコン + テキストラベルの操作ボタン（アイコンは装飾用途、
/// アクセシブルネームは可視テキストが担う）。
fn action_button(variant: ButtonVariant, icon_node: Node, label: &'static str) -> Node {
    button(
        &ButtonProps {
            variant,
            size: Size::Sm,
            ..ButtonProps::default()
        },
        vec![],
        vec![icon_node, text(label)],
    )
}

/// アバター・氏名・役職・ステータスバッジ・操作ボタン群を束ねる見出し行。
/// `actions` は呼び出し側で組んだボタン列（版 A/B で個数が異なる）。
fn profile_header(
    name: &'static str,
    title: &'static str,
    company: &'static str,
    actions: Vec<Node>,
) -> Node {
    div(
        vec![("class", "blocks-profile-detail-datalist-header")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Xl,
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
                vec![("class", "blocks-profile-detail-datalist-identity")],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(name)],
                    ),
                    div(
                        vec![("class", "blocks-profile-detail-datalist-identity-meta")],
                        vec![
                            styled_text::text(
                                &TextProps {
                                    variant: TextVariant::Muted,
                                    size: TextSize::Sm,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(format!("{title} · {company}"))],
                            ),
                            badge(
                                &BadgeProps {
                                    variant: BadgeVariant::Subtle,
                                    ..BadgeProps::default()
                                },
                                vec![],
                                vec![text("在籍中")],
                            ),
                        ],
                    ),
                ],
            ),
            div(
                vec![("class", "blocks-profile-detail-datalist-actions")],
                actions,
            ),
        ],
    )
}

/// ラベル・値の 1 行（`data_list::item` + `item-label` + `item-value`）を
/// 組み立てる。
fn row(label: &'static str, value: &'static str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 指定 orientation の `data_list::root`（CSS フック
/// `data-blocks-profile-detail-datalist-list` 付き）を組む。
fn list(orientation: DataListOrientation, rows: Vec<Node>) -> Node {
    data_list::root(
        DataListProps {
            orientation,
            ..DataListProps::default()
        },
        vec![("data-blocks-profile-detail-datalist-list", "")],
        rows,
    )
}

/// 見出し + 定義リストの 1 セクション。
fn section(title: &'static str, list_node: Node) -> Node {
    div(
        vec![("class", "blocks-profile-detail-datalist-section")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(title)],
            ),
            list_node,
        ],
    )
}

/// A: 代表構成（R0219）。操作ボタン 2 個 + 縦積みの定義リスト 2 セクション。
fn version_representative() -> Node {
    let name = dummy_assets::PERSON_NAMES[0];
    let header = profile_header(
        name,
        dummy_assets::JOB_TITLES[0],
        dummy_assets::COMPANY_NAMES[0],
        vec![
            action_button(ButtonVariant::Outline, phone_icon(), "通話"),
            action_button(ButtonVariant::Outline, mail_icon(), "メッセージ"),
        ],
    );
    let contact = section(
        "連絡先",
        list(
            DataListOrientation::Vertical,
            vec![
                row("メール", "haruto.fujimaki@example.com"),
                row("電話", "090-1234-5678"),
                row("所在地", "東京都渋谷区"),
                row("所属", "プロダクト開発部"),
            ],
        ),
    );
    let history = section(
        "経歴",
        list(
            DataListOrientation::Vertical,
            vec![
                row("入社", "2021-04-01"),
                row("前職", "Verdant Foundry"),
                row("資格", "情報処理安全確保支援士"),
            ],
        ),
    );
    div(
        vec![("class", "blocks-profile-detail-datalist-section")],
        vec![header, contact, history],
    )
}

/// B: 操作 4 個 + 横並び定義リスト（R0221）。連絡先・経歴を 1 本の
/// 横並びリストへ統合する。
fn version_four_actions_horizontal() -> Node {
    let name = dummy_assets::PERSON_NAMES[1];
    let header = profile_header(
        name,
        dummy_assets::JOB_TITLES[1],
        dummy_assets::COMPANY_NAMES[1],
        vec![
            action_button(ButtonVariant::Outline, phone_icon(), "通話"),
            action_button(ButtonVariant::Outline, mail_icon(), "メッセージ"),
            action_button(ButtonVariant::Ghost, calendar_icon(), "予定を追加"),
            action_button(ButtonVariant::Ghost, share_icon(), "共有"),
        ],
    );
    let rows = list(
        DataListOrientation::Horizontal,
        vec![
            row("メール", "elena.vasquez@example.com"),
            row("電話", "080-2468-1357"),
            row("所在地", "大阪府大阪市"),
            row("所属", "エンジニアリング部"),
            row("入社", "2019-07-01"),
            row("資格", "PMP"),
        ],
    );
    div(
        vec![("class", "blocks-profile-detail-datalist-section")],
        vec![header, rows],
    )
}

/// `profile-detail-datalist` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-profile-detail-datalist-stack")],
        vec![version_representative(), version_four_actions_horizontal()],
    )
}
```

## 原案差分メモ

- **版 A（代表構成、R0219）**: 操作ボタン 2 個（通話・メッセージ）と、縦積み
  （Vertical）の定義リスト 2 セクション（連絡先・経歴）で構成します。
- **版 B（操作 4 個 + 横並び定義リスト、R0221）**: 操作ボタン 4 個（通話・
  メッセージ・予定を追加・共有）と、横並び（Horizontal）の定義リスト 1 本
  （連絡先・経歴を統合）で構成します。
- 定義リストの行間罫線・狭幅時の縦積みは `data-list` 部品自体の機能ではなく、
  本 block の CSS が付与しています（`data-list` は区切り線ユーティリティを
  意図的に持たないため）。
- 狭幅（コンテナ幅 36rem 未満）では操作ボタン群が下段へ折り返し、版 B の
  横並び定義リストが縦積みへ切り替わります（`@container` によるコンテナ
  クエリ判定）。
- 操作ボタンのアイコンは自作の線画（SVG path）で、参照元由来のアイコン
  セットではありません。

関連情報: [Avatar](../themes/avatar.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Button](../themes/button.md) /
[Data List](../themes/data-list.md) / [Badge](../themes/badge.md) /
[Icon](../themes/icon.md)
