# grid-list-contact-cards

`fandhe-frontend-pre-styled-ui` の `card` / `avatar` / `badge` / `button` /
`icon` / `list` 部品を合成した、連絡先カードのグリッドです。Blocks セクション
は新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください（主参照は対応表 ID R0973、集約元は
R0974。出典の固有名・ファイル名は記載しません）。

連絡先カードを 1〜4 列のグリッドに並べます。各カードの下端には「メール」
「電話」の 2 分割アクションを配置します。カード本体は横型（主参照、左に
名前・役職・権限バッジ、右に小さいアバター）と縦型（集約元、中央に大きい
アバター、その下に名前などを縦に積む）の 2 通りを静的インスタンスとして
並べています。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。メール・電話ボタンは `type="button"` のまま
送信先（`mailto:`/`tel:` リンク）を持ちません。氏名・役職・権限はすべて
独自に書いた架空のものであり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, AvatarShape, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 連絡先 1 件分の静的データ（架空、実企業名・PII を含まない）。
struct Contact {
    name: &'static str,
    role: &'static str,
    permission: &'static str,
    palette: ColorPalette,
}

/// `horizontal`（R0973）用の 6 件。
const CONTACTS_HORIZONTAL: [Contact; 6] = [
    Contact {
        name: "Haruto Fujimaki",
        role: "Product Designer",
        permission: "管理者",
        palette: ColorPalette::Success,
    },
    Contact {
        name: "Elena Vasquez",
        role: "Engineering Lead",
        permission: "編集者",
        palette: ColorPalette::Info,
    },
    Contact {
        name: "Kwame Boateng",
        role: "Customer Success Manager",
        permission: "閲覧者",
        palette: ColorPalette::Neutral,
    },
    Contact {
        name: "Mei Lindqvist",
        role: "Data Analyst",
        permission: "編集者",
        palette: ColorPalette::Info,
    },
    Contact {
        name: "Noor Al-Sayed",
        role: "Marketing Strategist",
        permission: "閲覧者",
        palette: ColorPalette::Neutral,
    },
    Contact {
        name: "Ola Bergström",
        role: "Operations Coordinator",
        permission: "管理者",
        palette: ColorPalette::Success,
    },
];

/// `vertical`（R0974）用の 8 件。
const CONTACTS_VERTICAL: [Contact; 8] = [
    Contact {
        name: "Priya Chandran",
        role: "Customer Success Manager",
        permission: "編集者",
        palette: ColorPalette::Info,
    },
    Contact {
        name: "Théo Marchetti",
        role: "Data Analyst",
        permission: "閲覧者",
        palette: ColorPalette::Neutral,
    },
    Contact {
        name: "Ingrid Solheim",
        role: "Product Manager",
        permission: "管理者",
        palette: ColorPalette::Success,
    },
    Contact {
        name: "Diego Alcantara",
        role: "Support Engineer",
        permission: "閲覧者",
        palette: ColorPalette::Neutral,
    },
    Contact {
        name: "Amara Okafor",
        role: "Sales Manager",
        permission: "編集者",
        palette: ColorPalette::Info,
    },
    Contact {
        name: "Lukas Hoffmann",
        role: "QA Engineer",
        permission: "閲覧者",
        palette: ColorPalette::Neutral,
    },
    Contact {
        name: "Saanvi Rao",
        role: "Content Strategist",
        permission: "編集者",
        palette: ColorPalette::Info,
    },
    Contact {
        name: "Marcus Lindberg",
        role: "Operations Analyst",
        permission: "管理者",
        palette: ColorPalette::Success,
    },
];

/// メール封筒アイコン（装飾、モジュール doc「アバター・アイコンの a11y」節）。
fn mail_icon() -> Node {
    icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M4 6h16v12H4z M4 6l8 7 8-7"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "1.5"),
                ("stroke-linejoin", "round"),
                ("stroke-linecap", "round"),
            ],
            vec![],
        )],
    )
}

/// 電話受話器アイコン（装飾）。
fn phone_icon() -> Node {
    icon(
        &IconProps {
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                (
                    "d",
                    "M5 4h3l2 5-2.5 1.5a11 11 0 0 0 5 5L14 13l5 2v3a2 2 0 0 1-2 2A15 15 0 0 1 3 6a2 2 0 0 1 2-2z",
                ),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "1.5"),
                ("stroke-linejoin", "round"),
                ("stroke-linecap", "round"),
            ],
            vec![],
        )],
    )
}

/// 権限バッジ 1 件（`Size::Sm`・`BadgeVariant::Subtle`）。
fn permission_badge(item: &Contact) -> Node {
    badge::badge(
        &BadgeProps {
            variant: BadgeVariant::Subtle,
            size: Size::Sm,
            palette: item.palette,
        },
        vec![],
        vec![text(item.permission)],
    )
}

/// メール・電話の 2 分割アクション行（横型・縦型共通、モジュール doc
/// 「`mailto:`/`tel:` リンクを使わない理由」節）。
fn actions_footer() -> Node {
    card::footer(
        vec![("class", "blocks-grid-list-contact-cards-actions")],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-grid-list-contact-cards-action", "")],
                vec![mail_icon(), text("メール")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Ghost,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-grid-list-contact-cards-action", "")],
                vec![phone_icon(), text("電話")],
            ),
        ],
    )
}

/// 横型カード 1 枚（R0973。左に名前・役職・権限バッジ、右に小さい
/// アバター）。
fn horizontal_card(item: &Contact) -> Node {
    card::root(
        CardProps::default(),
        vec![],
        vec![
            card::body(
                vec![("class", "blocks-grid-list-contact-cards-main")],
                vec![
                    div(
                        vec![("class", "blocks-grid-list-contact-cards-info")],
                        vec![
                            div(vec![], vec![text(item.name)]),
                            permission_badge(item),
                            div(
                                vec![("class", "blocks-grid-list-contact-cards-role")],
                                vec![text(item.role)],
                            ),
                        ],
                    ),
                    avatar::root(
                        &AvatarProps {
                            size: Size::Md,
                            shape: AvatarShape::Circle,
                            ..AvatarProps::default()
                        },
                        vec![("data-blocks-grid-list-contact-cards-avatar", "")],
                        vec![avatar::image(
                            ImageStatus::Loaded,
                            dummy_assets::AVATAR_SRC,
                            "",
                            vec![],
                        )],
                    ),
                ],
            ),
            actions_footer(),
        ],
    )
}

/// 縦型カード 1 枚（R0974。中央に大きいアバター、その下に名前などを縦に
/// 積む）。
fn vertical_card(item: &Contact) -> Node {
    card::root(
        CardProps::default(),
        vec![],
        vec![
            card::body(
                vec![("class", "blocks-grid-list-contact-cards-stack")],
                vec![
                    avatar::root(
                        &AvatarProps {
                            size: Size::Xl,
                            shape: AvatarShape::Circle,
                            ..AvatarProps::default()
                        },
                        vec![("data-blocks-grid-list-contact-cards-avatar", "")],
                        vec![avatar::image(
                            ImageStatus::Loaded,
                            dummy_assets::AVATAR_SRC,
                            "",
                            vec![],
                        )],
                    ),
                    div(vec![], vec![text(item.name)]),
                    div(
                        vec![("class", "blocks-grid-list-contact-cards-role")],
                        vec![text(item.role)],
                    ),
                    permission_badge(item),
                ],
            ),
            actions_footer(),
        ],
    )
}

/// 1 枚のカードを `list::item` へ包む（モジュール doc「CSS フックの選び方」
/// 節。`ListVariant::Plain` の item は `display: flex; align-items:
/// flex-start` になり item 自身は親 grid セルいっぱいに伸びるが、その
/// 唯一の子である card は主軸方向（既定 row）にサイズが content 依存の
/// まま先頭寄せに残る。`blocks-grid-list-contact-cards-item` 側は
/// `min-width: 0`（オーバーフロー対策）のみを持ち、card 側のセレクタへ
/// `flex: 1; width: 100%` を持たせてグリッドセルいっぱいへ伸長させる）。
fn card_item(card: Node) -> Node {
    list::item(
        vec![("class", "blocks-grid-list-contact-cards-item")],
        vec![card],
    )
}

/// インスタンス 1 件分（状態ラベル + グリッド）を組み立てる。
fn instance(variant: &'static str, label: &'static str, cards: Vec<Node>) -> Node {
    div(
        vec![("data-blocks-grid-list-contact-cards-variant", variant)],
        vec![
            div(
                vec![("class", "blocks-grid-list-contact-cards-label")],
                vec![text(label)],
            ),
            list::root(
                ListType::Unordered,
                ListVariant::Plain,
                vec![
                    ("data-blocks-grid-list-contact-cards-grid", ""),
                    ("data-blocks-grid-list-contact-cards-grid-variant", variant),
                ],
                cards,
            ),
        ],
    )
}

/// `grid-list-contact-cards` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。horizontal/vertical の 2 インスタンスを縦に並べる
/// （モジュール doc「horizontal / vertical の 2 インスタンスを並べる理由」
/// 節）。
pub fn demo() -> Node {
    let horizontal_cards: Vec<Node> = CONTACTS_HORIZONTAL
        .iter()
        .map(|item| card_item(horizontal_card(item)))
        .collect();
    let vertical_cards: Vec<Node> = CONTACTS_VERTICAL
        .iter()
        .map(|item| card_item(vertical_card(item)))
        .collect();

    div(
        vec![("class", "blocks-grid-list-contact-cards-layout")],
        vec![
            instance("horizontal", "横型カード（主参照）", horizontal_cards),
            instance("vertical", "縦型カード（集約元）", vertical_cards),
        ],
    )
}
```

## 原案差分メモ

- 主参照（対応表 ID R0973、横型カード）は `horizontal` インスタンスに
  対応します。左に名前・権限バッジ・役職、右に小さいアバターを配置して
  います。カードは 6 枚です。
- 集約元（対応表 ID R0974、縦型カード）は `vertical` インスタンスに対応
  します。中央に大きいアバター、その下に名前・役職・権限バッジを縦に
  積んでいます。カードは 8 枚です。
- Issue 本文がメール・電話をボタンで表現するよう指定しているため、
  `mailto:`/`tel:` の `<a>` リンクではなく `button::button`（既定
  `type="button"`）を使っています。送信先は一切持ちません。
- 列数の切り替えは `@container` を使い、Demo 枠の幅（ビューポート幅とは
  一致しない）を基準にしています。既定は 1 列、`36rem` 以上で 2 列、
  `52rem` 以上で 3 列、`vertical` インスタンスのみ `68rem` 以上でさらに
  4 列へ増やします（横型カードは情報量が多く 4 列だと窮屈になるため
  据え置いています）。
- メール・電話アイコンは自作の幾何図形（封筒・受話器を単純な線で描画）
  であり、実在ブランドのロゴ・商標は模していません。`aria-hidden="true"`
  の装飾扱いで、ボタン名（可視テキスト）が操作の意味を伝えるため情報は
  失われません。
- 氏名・役職・権限区分はすべて独自に書いた架空のものです。参照元の文言・
  配色・アイコン形状は持ち込んでいません。

関連情報: [Card](../themes/card.md) / [Avatar](../themes/avatar.md) /
[Badge](../themes/badge.md) / [Button](../themes/button.md) /
[Icon](../themes/icon.md) / [List](../themes/list.md)
