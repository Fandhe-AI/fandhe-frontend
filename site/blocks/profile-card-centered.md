# profile-card-centered

`card` / `avatar` / `badge` / `heading` / `text` / `button` / `link` /
`icon` の 8 部品を合成した、中央寄せのプロフィールカードの合成例です。
Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください。

avatar → 氏名（隣に認証済みを示す badge を併記）→ 肩書・所在地 → 自己紹介の
順に縦積みし、下に SNS アイコンリンクと主操作ボタンを置いた構成（代表構成）
と、主操作を全幅ボタン 1 個にして自己紹介の下へテキスト付きリンク一覧を
縦並びにした最小版の 2 インスタンスを併記します。カードは固定幅ですが
狭いコンテナではコンテナ幅に追従します。

いずれも静的な表示例であり、`<form>` 要素を持ちません。氏名・肩書・
所在地・自己紹介はすべて独自に書いた架空の文言です。SNS リンクは架空
人物のため実在アカウントを持たず、全件が本リポジトリの GitHub ページへ
遷移します。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, AvatarShape, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 外部の実在 URL（`href="#"` は使わない、他 block と同型の判断）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// [`REPO`] の accessible name（遷移先と食い違わない固定文字列、WCAG 2.4.4、
/// PR #3271 の Bugbot 教訓）。
const REPO_LABEL: &str = "fandhe-frontend の GitHub リポジトリ";

/// 架空の肩書・所在地・自己紹介（[`dummy_assets::PERSON_NAMES`]/
/// [`dummy_assets::JOB_TITLES`] と対で使う）。
const LOCATION: &str = "東京, 日本";
const BIO: &str =
    "UI コンポーネントの一貫性とアクセシビリティを軸に、プロダクト全体の体験設計を担当しています。";

/// 自作の幾何パスによる装飾/SNS 兼用アイコン（`team_avatar_grid::geo_icon`
/// と同型。`label` は呼び出し側が指定する）。
fn geo_icon(size: Size, path_d: &'static str, label: Option<&'static str>) -> Node {
    icon(
        &IconProps {
            size,
            label,
            ..IconProps::default()
        },
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

/// リンク（鎖）の幾何アイコン。`label` はアイコン単独リンク（[`social_link`]）
/// でのみ `Some`（可視テキスト付きの [`minimal_card`] リンク一覧では装飾
/// 扱いの `None`、モジュール doc「アイコン単独リンクと可視テキスト付き
/// リンクでのアクセシブル名の作り分け」参照）。
fn link_shape(size: Size, label: Option<&'static str>) -> Node {
    geo_icon(
        size,
        "M9 15l6-6 M11 6l1-1a3 3 0 114 4l-1 1 M13 18l-1 1a3 3 0 11-4-4l1-1",
        label,
    )
}

/// 吹き出し（メッセージ）の幾何アイコン（`label` の扱いは [`link_shape`] 参照）。
fn message_shape(size: Size, label: Option<&'static str>) -> Node {
    geo_icon(
        size,
        "M21 15a2 2 0 01-2 2H8l-4 4V6a2 2 0 012-2h13a2 2 0 012 2z",
        label,
    )
}

/// 地球儀の幾何アイコン（`label` の扱いは [`link_shape`] 参照）。
fn globe_shape(size: Size, label: Option<&'static str>) -> Node {
    geo_icon(
        size,
        "M12 3a9 9 0 100 18 9 9 0 000-18z M3 12h18 M12 3c2.2 2.4 3.5 5.5 3.5 9s-1.3 6.6-3.5 9c-2.2-2.4-3.5-5.5-3.5-9s1.3-6.6 3.5-9z",
        label,
    )
}

/// [`REPO`] へ遷移する SNS アイコンのみのリンク。可視テキストを持たない
/// ためアイコン自体に [`REPO_LABEL`] を与え `aria-label` を出す。
fn social_link(shape: fn(Size, Option<&'static str>) -> Node) -> Node {
    link::root(
        REPO,
        &LinkProps {
            external: true,
            ..LinkProps::default()
        },
        vec![("data-blocks-profile-card-centered-social-link", "")],
        vec![shape(Size::Sm, Some(REPO_LABEL))],
    )
}

/// SNS アイコンリンク 3 個（全件 [`REPO`] へ遷移、モジュール doc参照）。
fn social_links() -> Node {
    div(
        vec![("class", "blocks-profile-card-centered-social")],
        vec![
            social_link(link_shape),
            social_link(message_shape),
            social_link(globe_shape),
        ],
    )
}

/// 装飾画像の avatar（氏名は隣接テキストで伝わるため `alt=""`、
/// `team_avatar_grid::member_avatar` と同型の判断）。
fn avatar_node() -> Node {
    avatar::root(
        &AvatarProps {
            size: Size::Xl,
            shape: AvatarShape::Circle,
            ..AvatarProps::default()
        },
        vec![],
        vec![avatar::image(
            ImageStatus::Loaded,
            dummy_assets::AVATAR_SRC,
            "",
            vec![],
        )],
    )
}

/// 氏名見出し + 「認証済み」badge（モジュール doc「認証済みを可視テキスト
/// 付き badge で表す理由」節参照）。見出しレベルは [`section`] の
/// variant タイトル（`h3`）配下として `H4`（`team_avatar_grid` の
/// 変種見出し `H3`・メンバー氏名 `H4` と同じ階層、PR #3388 Bugbot 指摘）。
fn name_row() -> Node {
    div(
        vec![("class", "blocks-profile-card-centered-name-row")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps {
                    size: HeadingSize::Md,
                    weight: HeadingWeight::Semibold,
                },
                vec![],
                vec![text(dummy_assets::PERSON_NAMES[0])],
            ),
            badge::badge(
                &BadgeProps {
                    variant: BadgeVariant::Subtle,
                    size: Size::Sm,
                    palette: ColorPalette::Success,
                },
                vec![],
                vec![text("認証済み")],
            ),
        ],
    )
}

/// 肩書・所在地（中黒区切りの 1 行）。
fn role_location() -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(format!(
            "{} · {}",
            dummy_assets::JOB_TITLES[0],
            LOCATION
        ))],
    )
}

/// 自己紹介の 1 段落。
fn bio() -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            ..TextProps::default()
        },
        vec![],
        vec![text(BIO)],
    )
}

/// 代表構成（R0217）: avatar → 氏名/認証済み → 肩書・所在地 → 自己紹介 →
/// SNS → 主操作ボタン 2 個（Solid/Outline）の縦積み。
fn representative_card() -> Node {
    card::root(
        CardVariant::Elevated,
        vec![("data-blocks-profile-card-centered-card", "")],
        vec![card::body(
            vec![("class", "blocks-profile-card-centered-body")],
            vec![
                avatar_node(),
                name_row(),
                role_location(),
                bio(),
                social_links(),
                div(
                    vec![("class", "blocks-profile-card-centered-actions")],
                    vec![
                        button(&ButtonProps::default(), vec![], vec![text("フォローする")]),
                        button(
                            &ButtonProps {
                                variant: ButtonVariant::Outline,
                                ..ButtonProps::default()
                            },
                            vec![],
                            vec![text("メッセージ")],
                        ),
                    ],
                ),
            ],
        )],
    )
}

/// 最小版（R0224）: avatar → 氏名/認証済み → 肩書・所在地 → 全幅ボタン
/// 1 個 → 自己紹介 → リンク一覧（テキスト付き 3 件、縦並び）。
fn minimal_card() -> Node {
    // アイコンは装飾（`label: None`）とし、可視テキスト自体を遷移先
    // （GitHub リポジトリ）と一致する文言にする（モジュール doc「アイコン
    // 単独リンクと可視テキスト付きリンクでのアクセシブル名の作り分け」
    // 参照。イシュー #2936 codex レビュー P1 対応）。
    let link_list_item = |shape: fn(Size, Option<&'static str>) -> Node, label: &'static str| {
        el(
            "li",
            vec![],
            vec![link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![("class", "blocks-profile-card-centered-link-list-item")],
                vec![shape(Size::Sm, None), text(label)],
            )],
        )
    };
    card::root(
        CardVariant::Elevated,
        vec![("data-blocks-profile-card-centered-card", "")],
        vec![card::body(
            vec![("class", "blocks-profile-card-centered-body")],
            vec![
                avatar_node(),
                name_row(),
                role_location(),
                button(
                    &ButtonProps::default(),
                    vec![("data-blocks-profile-card-centered-cta-full", "")],
                    vec![text("プロフィールを見る")],
                ),
                bio(),
                el(
                    "ul",
                    vec![("class", "blocks-profile-card-centered-link-list")],
                    vec![
                        link_list_item(link_shape, "GitHub"),
                        link_list_item(message_shape, "連絡する（GitHub）"),
                        link_list_item(globe_shape, "Webサイト（GitHub）"),
                    ],
                ),
            ],
        )],
    )
}

/// 見出し付きセクションを組み立てる（`list_title_meta::section` と同型）。
fn section(label: &'static str, node: Node) -> Node {
    div(
        vec![("class", "blocks-profile-card-centered-section")],
        vec![
            el(
                "h3",
                vec![("class", "blocks-profile-card-centered-section-title")],
                vec![text(label)],
            ),
            node,
        ],
    )
}

/// `profile-card-centered` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。代表構成（R0217）を先頭に、最小版（R0224）を並記する。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-profile-card-centered-layout")],
        vec![
            section("代表構成", representative_card()),
            section("全幅ボタン + リンク縦並び（最小版）", minimal_card()),
        ],
    )
}
```

## 原案差分メモ

- 主参照は R0217（代表構成）、集約元は R0224（全幅ボタン + リンク縦並び
  の最小版）です。出典の固有名・ファイル名は記載しません。
- 認証済みの印は avatar の色ドット（`avatar::badge`）ではなく、可視
  テキスト「認証済み」を持つ `badge` にしています。色だけで状態を伝えな
  いための判断です。
- SNS リンクは架空人物のため実在アカウントを持たず、全件で本リポジトリ
  の GitHub ページへ遷移させています。アクセシブル名は遷移先と食い違わ
  ない固定文字列（「fandhe-frontend の GitHub リポジトリ」）にしています。
- 氏名・肩書・所在地・自己紹介はすべて独自に書いた架空のものです（実在
  の人物・企業・ブランドとは無関係です）。
- 実データ取得・送信・フォロー/メッセージ操作は行わず、静的な初期状態の
  みを示します。
- ブラウザでの実機確認（狭幅追従・ライト/ダーク両テーマ）はサンドボックス
  制約により未実施です。cargo test による出力検証のみで代替しました。
