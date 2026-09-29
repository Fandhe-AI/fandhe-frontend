# profile-header-follow

アバター・氏名・認証バッジ・所属・自己紹介・フォロワー数を束ねるプロフィール
見出しと、フォロー操作ボタン群（フォロー・メッセージ）を合成したブロックです。
`avatar` / `badge` / `heading` / `text` / `button` / `stat` の 6 部品を合成し
ます。Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

Demo はアバターの配置を反転させた 2 インスタンス（`avatar-start`: アバター
左・`avatar-end`: アバター右）を並記します。フォロー状態はいずれも「未フォ
ロー」で固定です（無 JS の静的表示のため）。氏名・役職・社名・自己紹介・
数値はすべて架空のデータであり、実在の人物・企業・PII は含みません。
アバター画像はビルド時生成の同梱プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// プロフィール 1 件分の架空データ。
struct ProfileEntry {
    name: &'static str,
    title: &'static str,
    company: &'static str,
    bio: &'static str,
    followers: &'static str,
    following: &'static str,
}

/// 代表版（架空、1 件）。両インスタンスで同一データを使い、レイアウト差分
/// （アバター位置）のみを見せる。
const PROFILE: ProfileEntry = ProfileEntry {
    name: dummy_assets::PERSON_NAMES[0],
    title: dummy_assets::JOB_TITLES[0],
    company: dummy_assets::COMPANY_NAMES[0],
    bio: "新しい体験を形にする仕事をしています。休日は写真を撮りながら街を歩くのが好きです。",
    followers: "1,280",
    following: "312",
};

/// アバター（`AvatarProps::size` は `Xl`。氏名は隣接の見出しが伝えるため
/// `alt=""` とする、`content_article_toc` と同型の判断）。
fn profile_avatar() -> Node {
    avatar::root(
        &AvatarProps {
            size: Size::Xl,
            ..AvatarProps::default()
        },
        vec![("data-blocks-profile-header-follow-avatar", "")],
        vec![
            avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, "", vec![]),
            avatar::fallback(
                ImageStatus::Loaded,
                vec![],
                vec![text(PROFILE.name.chars().take(1).collect::<String>())],
            ),
        ],
    )
}

/// 名前行（氏名 + 認証バッジ）。
fn name_row() -> Node {
    div(
        vec![("class", "blocks-profile-header-follow-name-row")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Lg,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-profile-header-follow-name", "")],
                vec![text(PROFILE.name)],
            ),
            badge(
                &BadgeProps {
                    size: Size::Sm,
                    ..BadgeProps::default()
                },
                vec![("data-blocks-profile-header-follow-verified", "")],
                vec![text("認証済み")],
            ),
        ],
    )
}

/// 所属行（役職・社名）。
fn affiliation_row() -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-profile-header-follow-affiliation", "")],
        vec![text(format!("{} · {}", PROFILE.title, PROFILE.company))],
    )
}

/// 自己紹介文。
fn bio_row() -> Node {
    styled_text::text(
        &TextProps::default(),
        vec![("data-blocks-profile-header-follow-bio", "")],
        vec![text(PROFILE.bio)],
    )
}

/// 1 件分の数値指標（フォロワー数・フォロー中数）。
fn stat_entry(value: &'static str, label: &'static str) -> Node {
    stat::root(
        Size::Sm,
        vec![("data-blocks-profile-header-follow-stat", "")],
        vec![
            stat::label(vec![], vec![text(label)]),
            stat::value_text(vec![], vec![text(value)]),
        ],
    )
}

/// フォロワー数・フォロー中数の並び。
fn stats_row() -> Node {
    div(
        vec![("class", "blocks-profile-header-follow-stats")],
        vec![
            stat_entry(PROFILE.followers, "フォロワー"),
            stat_entry(PROFILE.following, "フォロー中"),
        ],
    )
}

/// 操作ボタン群（フォロー・メッセージ）。両インスタンスとも
/// 「未フォロー」固定（モジュール doc「2 インスタンス」節参照）。
fn actions_row() -> Node {
    div(
        vec![("class", "blocks-profile-header-follow-actions")],
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
    )
}

/// 1 インスタンス分のプロフィール見出し（`variant` はアバター位置を
/// 切り替える CSS フック）。
fn profile_header(variant: &'static str) -> Node {
    div(
        vec![
            ("class", "blocks-profile-header-follow-header"),
            ("data-blocks-profile-header-follow-variant", variant),
        ],
        vec![
            profile_avatar(),
            div(
                vec![("class", "blocks-profile-header-follow-body")],
                vec![name_row(), affiliation_row(), bio_row(), stats_row()],
            ),
            actions_row(),
        ],
    )
}

/// `profile-header-follow` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。`avatar-start`（アバター左）・`avatar-end`（アバター右）の
/// 2 インスタンスを縦に並べる。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-profile-header-follow-layout")],
        vec![profile_header("avatar-start"), profile_header("avatar-end")],
    )
}
```

## 原案差分メモ

- Issue 本文が示すレイアウト仕様（アバター + 氏名/認証バッジ/所属/自己紹介/
  フォロワー数 + 操作ボタン群、狭幅は縦積み・広幅は横並び、アバター右配置版
  の並記）を、既存部品（Avatar / Badge / Heading / Text / Button / Stat）の
  合成のみで再現しています。
- フォロワー数・フォロー中数は `stat` 部品（`<dl>`/`<dt>`/`<dd>`）で表現し、
  独自の数値整形ロジックは実装しません（`.claude/rules/coding-rust.md`
  「UI 部品の責務境界」節）。
- アバター右配置版（`avatar-end`）は CSS の `grid-template-columns`/`order`
  切り替えのみで実現し、DOM 構造・データは左配置版と同一です。

関連情報: [Avatar](../themes/avatar.md) / [Badge](../themes/badge.md) /
[Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Button](../themes/button.md) / [Stat](../themes/stat.md)
