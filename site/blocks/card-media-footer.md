# card-media-footer

`card` / `image` / `badge` / `heading` / `text` / `avatar` / `button` /
`menu` の 8 部品を合成した、メディア付きカード（カバー画像 + タグ・題名・
抜粋 + 下端の著者またはメンバー情報）の合成例です。Blocks セクションは
新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください。

本 Demo は静的な表示例であり、`<form>` 要素を持ちません。単体表示では
2 枚のカードを並べ、片方は下端に著者アバター + 日付、もう片方は画像上に
操作バー（プレビュー/保存ボタン + `menu`）を重ねた状態 + 下端にメンバー群
（`avatar::group`）を示します。グリッド表示は 3 枚を並べ、著者形と
メンバー群形を混在させています。操作バーの `menu` は id 重複を避けるため
ページ内 1 箇所のみに置いています。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text as core_text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::menu::{self, OpenState};
use fandhe_frontend_pre_styled_ui::text::{self as text_part, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 1 件分の架空カードデータ（実企業名・実人物は使わない）。
struct CardData {
    tag: &'static str,
    title: &'static str,
    excerpt: &'static str,
    /// [`dummy_assets::PERSON_NAMES`] への添字（著者形のときのみ使用）。
    author_index: usize,
    /// `true` のとき下端をメンバー群、`false` のとき著者 + 日付にする。
    members: bool,
    date: &'static str,
}

const CARDS_SINGLE: [CardData; 2] = [
    CardData {
        tag: "Release",
        title: "描画パイプラインの計測レポート",
        excerpt: "初回描画からハイドレーション完了までの所要時間を、\
                   構成別に比較して振り返ります。",
        author_index: 0,
        members: false,
        date: "2026-04-02",
    },
    CardData {
        tag: "Design",
        title: "余白トークンの見直し記録",
        excerpt: "既存コンポーネントへの影響範囲を洗い出しながら、\
                   段階的に置き換えた手順をまとめました。",
        author_index: 1,
        members: true,
        date: "2026-03-18",
    },
];

const CARDS_GRID: [CardData; 3] = [
    CardData {
        tag: "Guides",
        title: "初回セットアップの詰まりどころ",
        excerpt: "環境構築でよくあるつまずきと、その回避手順を短くまとめます。",
        author_index: 2,
        members: false,
        date: "2026-03-05",
    },
    CardData {
        tag: "Engineering",
        title: "差分検知の仕組みを図解する",
        excerpt: "更新対象を絞り込む判定ロジックを、具体例に沿って説明します。",
        author_index: 3,
        members: true,
        date: "2026-02-21",
    },
    CardData {
        tag: "Release",
        title: "検証環境の切り替え手順",
        excerpt: "本番相当の構成へ揃えるための、チェックリスト形式の手順書です。",
        author_index: 4,
        members: false,
        date: "2026-02-09",
    },
];

/// 下端の著者行（アバター + 氏名 + `<time datetime>` の日付）。
fn author_footer(author_index: usize, date: &str) -> Node {
    let name = dummy_assets::PERSON_NAMES[author_index % dummy_assets::PERSON_NAMES.len()];
    div(
        vec![("class", "blocks-card-media-footer-author")],
        vec![
            avatar::root(
                &AvatarProps::default(),
                vec![],
                vec![avatar::image(
                    ImageStatus::Loaded,
                    dummy_assets::AVATAR_SRC,
                    name,
                    vec![],
                )],
            ),
            div(
                vec![("class", "blocks-card-media-footer-author-text")],
                vec![
                    text_part::text(
                        &TextProps {
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![core_text(name)],
                    ),
                    text_part::text(
                        &TextProps {
                            size: TextSize::Xs,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![el("time", vec![("datetime", date)], vec![core_text(date)])],
                    ),
                ],
            ),
        ],
    )
}

/// 下端のメンバー群行（`avatar::group` に 3 名 + 「+2」fallback）。
fn members_footer() -> Node {
    let member_avatar = |index: usize| -> Node {
        let name = dummy_assets::PERSON_NAMES[index % dummy_assets::PERSON_NAMES.len()];
        avatar::root(
            &AvatarProps::default(),
            vec![],
            vec![avatar::image(
                ImageStatus::Loaded,
                dummy_assets::AVATAR_SRC,
                name,
                vec![],
            )],
        )
    };
    div(
        vec![("class", "blocks-card-media-footer-author")],
        vec![
            avatar::group(
                vec![("data-blocks-card-media-footer-members", "")],
                vec![
                    member_avatar(0),
                    member_avatar(1),
                    member_avatar(2),
                    avatar::root(
                        &AvatarProps::default(),
                        vec![],
                        vec![avatar::fallback(
                            ImageStatus::Error,
                            vec![],
                            vec![core_text("+2")],
                        )],
                    ),
                ],
            ),
            text_part::text(
                &TextProps {
                    size: TextSize::Xs,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![core_text("5 名が参加")],
            ),
        ],
    )
}

/// カード b（操作バー表示状態）専用の操作バー。プレビュー/保存ボタンと、
/// 3 項目 + 区切り線を持つ `menu` を静的な開状態ではなく閉状態のまま
/// マークアップだけを示す（モジュール doc「ホバー表示化の CSS を実装しない
/// 理由」節参照）。
fn media_actions() -> Node {
    let trigger = menu::trigger(
        OpenState::Closed,
        false,
        Some("blocks-card-media-footer-menu"),
        vec![("aria-label", "その他の操作")],
        vec![core_text("\u{2026}")],
    );
    let content = menu::content(
        OpenState::Closed,
        Some("blocks-card-media-footer-menu"),
        None,
        vec![],
        vec![
            menu::item("share", false, false, vec![], vec![core_text("共有")]),
            menu::item(
                "bookmark",
                false,
                false,
                vec![],
                vec![core_text("あとで読む")],
            ),
            menu::separator(vec![], vec![]),
            menu::item("report", false, false, vec![], vec![core_text("報告")]),
        ],
    );
    let positioner = menu::positioner(OpenState::Closed, vec![], vec![content]);
    let menu_root = menu::root(
        Size::Sm,
        OpenState::Closed,
        vec![],
        vec![trigger, positioner],
    );

    div(
        vec![
            ("class", "blocks-card-media-footer-actions"),
            ("role", "toolbar"),
            ("aria-label", "カードの操作"),
        ],
        vec![
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Subtle,
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![],
                vec![core_text("プレビュー")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Subtle,
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![],
                vec![core_text("保存")],
            ),
            menu_root,
        ],
    )
}

/// カード 1 枚を組み立てる。`with_actions` が `true` のときのみカバー画像に
/// 操作バーを重ねる（カード b のみ・グリッドには置かない）。
fn media_card(data: &CardData, with_actions: bool) -> Node {
    let alt = format!("{}のカバー画像", data.title);
    let mut image_props = ImageProps::new(dummy_assets::SCREENSHOT_SRC, &alt);
    image_props.aspect_ratio = AspectRatio::Video;
    let image_node = image::image(&image_props, vec![]);

    let mut cover_children = vec![image_node];
    if with_actions {
        cover_children.push(media_actions());
    }

    let footer = if data.members {
        members_footer()
    } else {
        author_footer(data.author_index, data.date)
    };

    card::root(
        CardProps::default(),
        vec![("data-blocks-card-media-footer-card", "")],
        vec![
            card::cover(
                vec![("class", "blocks-card-media-footer-media")],
                cover_children,
            ),
            card::body(
                vec![("class", "blocks-card-media-footer-body")],
                vec![
                    badge::badge(&BadgeProps::default(), vec![], vec![core_text(data.tag)]),
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Md,
                            weight: HeadingWeight::Semibold,
                        },
                        vec![],
                        vec![core_text(data.title)],
                    ),
                    text_part::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![core_text(data.excerpt)],
                    ),
                ],
            ),
            card::footer(
                vec![("class", "blocks-card-media-footer-footer")],
                vec![footer],
            ),
        ],
    )
}

/// `card-media-footer` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。単体 2 枚（インスタンス差分あり）とグリッド 3 枚を縦に並べる
/// 静的な表示のみを描く。
#[must_use]
pub fn demo() -> Node {
    let note_single = text_part::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(
            "左: 著者 + 日付の代表構成。右: 画像上に操作バーを重ねた状態 + メンバー群。",
        )],
    );
    let note_grid = text_part::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text("著者形とメンバー群形を混在させたグリッド配置。")],
    );

    let single_cards: Vec<Node> = CARDS_SINGLE
        .iter()
        .enumerate()
        .map(|(i, data)| media_card(data, i == 1))
        .collect();
    let grid_cards: Vec<Node> = CARDS_GRID
        .iter()
        .map(|data| media_card(data, false))
        .collect();

    div(
        vec![("class", "blocks-card-media-footer")],
        vec![
            note_single,
            div(
                vec![("class", "blocks-card-media-footer-single")],
                single_cards,
            ),
            note_grid,
            div(vec![("class", "blocks-card-media-footer-grid")], grid_cards),
        ],
    )
}
```

## 原案差分メモ

集約元は次の 2 案を参照しています。

- **代表構成**: カバー画像・タグ badge・題名・抜粋・下端の著者アバター +
  日付という定番の構成です。単体表示のカード a、グリッド表示のカードの
  一部がこれに相当します。
- **サムネイル + ホバー操作バー**: サムネイル画像にポインタを重ねると
  プレビュー/保存/メニューの操作バーが浮き上がる構成です。本来は
  `:hover`/`:focus-within` で表示・非表示を切り替えますが、本 docs サイト
  は JS ハイドレーションを行わずキーボード操作性を優先するため、単体表示の
  カード b では操作バーを常時表示状態のまま固定して示しています（focus
  可能だが不可視な要素を作らないための判断）。実利用時にホバーでの
  表示切り替えを実装する場合は、`:hover`/`:focus-within` で
  `.blocks-card-media-footer-actions` の `opacity`/`visibility` を
  切り替える CSS を追加してください。
