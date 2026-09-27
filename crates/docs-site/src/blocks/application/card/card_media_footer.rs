//! `card-media-footer` block（イシュー #2900。親 #2892「Phase 3: Blocks
//! アプリケーション A」、ルート #2730「Blocks 目的別パーツ拡充ツリー」配下。
//! Application / Card カテゴリの block の 1 つ、
//! `crate::blocks::application::card` に登録する）。
//!
//! カバー画像・タグ・題名・抜粋・下端の著者/メンバー情報で構成する
//! 「メディア付きカード」の合成例。集約元 2 件の差分（代表構成と、画像に
//! 操作バーを重ねたサムネイル）は静的な 2 インスタンスで読み取れるように
//! する（詳細は `site/blocks/card-media-footer.md` の「原案差分メモ」節）。
//!
//! # 使用部品
//!
//! `card` / `image` / `badge` / `heading` / `text` / `avatar` / `button` /
//! `menu` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新規 UI 部品は追加しない。
//!
//! # 2 インスタンス + グリッドで示す構成
//!
//! - **単体（Single）**: カード a は代表構成（著者アバター + 日付）、
//!   カード b は画像上に操作バー（プレビュー/保存ボタン + `menu`）を重ねた
//!   状態、下端はメンバー群（[`avatar::group`]）
//! - **グリッド（Grid）**: 3 枚を並べ、著者形とメンバー群形を混在させる
//!   （操作バーはここには置かない。`menu` の id 重複を避けるため）
//!
//! # ホバー表示化の CSS を実装しない理由
//!
//! 集約元（原稿の「原案差分メモ」節が参照する 2 案）の一方はサムネイル上に
//! 操作バーを `:hover`/`:focus-within` で出す構成だが、本 Demo は無 JS の
//! docs サイトでキーボード操作性を優先し、操作バーを常時表示状態で固定する
//! （`cta_banner_magnetic`/`cursor_hover_cards` と同様、focus 可能だが不可視
//! な要素を作らないための判断）。実利用時のホバー表示化は原稿側で案内する。
//!
//! # `card::title`/`card::description` を使わない理由
//!
//! `blog_grid_image` と同型に、題名は [`heading::heading`] を直接使い
//! `HeadingLevel::H3` を明示する（Demo 内に導入見出しを置かないため、
//! ページ全体の見出し階層はカード題名が最上位になる）。
//!
//! # `data-*` フックを選ぶ理由（`drop_class_attr` の契約）
//!
//! `card::root`/`heading::heading`/`text::text`/`badge::badge`/
//! `avatar::root` はいずれも `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、これらの CSS フックは `class`
//! ではなく `data-blocks-card-media-footer-*` 属性で渡す（`crate::blocks`
//! モジュール doc「CSS フックが `class` と `[data-*]` で混在する理由」節と
//! 同じ判断軸）。`card::cover`/`card::body`/`card::footer` は variant を
//! 持たず `attrs` をそのまま連結するパーツのため、従来どおり `class` が
//! そのまま効く。
//!
//! # `menu` の id をページ内で 1 つに限る理由
//!
//! `demo_output_has_no_dangling_aria_references_or_duplicate_ids`
//! （`crates/docs-site/tests/blocks_contract.rs`）が id 重複を fail-closed に
//! 検知するため、`menu::trigger`/`menu::content` の id
//! （`blocks-card-media-footer-menu`）はカード b の 1 箇所にのみ出力し、
//! グリッド側には操作バーを置かない。
//!
//! # `ImageStatus::Loaded`/`ImageStatus::Error` を使う理由
//!
//! `avatar::image` の既定 [`ImageStatus`] は `Loading`（JS なし環境では
//! 非表示のまま）のため、著者アバターは `ImageStatus::Loaded` を明示して
//! 画像を表示状態にする。メンバー群の「+2」表示は画像を持たないため
//! `avatar::fallback` を `ImageStatus::Error`（画像なし側が可視）で使う
//! （`dashboard_01::user_menu` と同型の判断）。
//!
//! # ダミー素材・`<form>` を持たないこと
//!
//! 画像・人名は `crate::blocks::dummy_assets` の共通ヘルパを使う。
//! 題名・抜粋・タグは実在の企業・製品を連想させない独自の日本語文である。
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力せず、送信処理・データ整形・永続化を一切持たない静的な合成例である。
//! ボタンはすべて `type="button"`（`button::button` の既定契約）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
/// `avatar::group` へ包むだけでは重なり表示にならず、各 `avatar::root` へ
/// 個別に `stacked: true` を渡す必要がある（`AvatarProps::default()` は
/// `stacked: false`、Bugbot 指摘・レビュー是正）。
fn members_footer() -> Node {
    let stacked_props = AvatarProps {
        stacked: true,
        ..AvatarProps::default()
    };
    let member_avatar = |index: usize| -> Node {
        let name = dummy_assets::PERSON_NAMES[index % dummy_assets::PERSON_NAMES.len()];
        avatar::root(
            &stacked_props,
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
                        &stacked_props,
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
/// 理由」節参照）。無 JS のため押しても何も起きず、`menu` も開けないので、
/// いずれも `disabled: true`（menu は `trigger` 第 2 引数）を固定し
/// `[data-disabled]` を [`LAYOUT_CSS`] で `opacity: 1; cursor: default;` へ
/// 中和する（`app_shell_stacked` と同型の判断、レビュー是正）。
fn media_actions() -> Node {
    let trigger = menu::trigger(
        OpenState::Closed,
        true,
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
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-card-media-footer-preview", "")],
                vec![core_text("プレビュー")],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Subtle,
                    size: Size::Sm,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-card-media-footer-save", "")],
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/card-media-footer/",
    title: "card-media-footer",
    category: BlockCategory::Card,
    rust_source: "crates/docs-site/src/blocks/application/card/card_media_footer.rs",
    demo_class: "blocks-card-media-footer",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Menu",
            path: "/themes/menu/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `card_media_footer` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節。`blog_grid_image` 等と同型で
/// `super::blocks` 経由で `crate::blocks::stylesheet` へ連結される）。
const LAYOUT_CSS: &str = "\
.blocks-card-media-footer {\n  display: flex;\n  flex-direction: column;\n  gap: 1.5rem;\n}\n\
.blocks-card-media-footer-single {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: 1.5rem;\n  max-width: 48rem;\n}\n\
@media (max-width: 39.99rem) {\n  .blocks-card-media-footer-single {\n    grid-template-columns: 1fr;\n  }\n}\n\
.blocks-card-media-footer-grid {\n  display: grid;\n  grid-template-columns: repeat(3, minmax(0, 1fr));\n  gap: 1.5rem;\n}\n\
@media (max-width: 63.99rem) {\n  .blocks-card-media-footer-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n\
@media (max-width: 39.99rem) {\n  .blocks-card-media-footer-grid {\n    grid-template-columns: 1fr;\n  }\n}\n\
[data-blocks-card-media-footer-card] {\n  height: 100%;\n  display: flex;\n  flex-direction: column;\n}\n\
.blocks-card-media-footer-media {\n  position: relative;\n}\n\
.blocks-card-media-footer-body {\n  display: flex;\n  flex-direction: column;\n  gap: 0.5rem;\n}\n\
.blocks-card-media-footer-footer {\n  margin-top: auto;\n}\n\
.blocks-card-media-footer-author {\n  display: flex;\n  align-items: center;\n  gap: 0.5rem;\n}\n\
.blocks-card-media-footer-author-text {\n  display: flex;\n  flex-direction: column;\n}\n\
[data-blocks-card-media-footer-members] {\n  flex-shrink: 0;\n}\n\
.blocks-card-media-footer-actions {\n  position: absolute;\n  inset-inline: var(--fandhe-space-3);\n  bottom: var(--fandhe-space-3);\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  padding: var(--fandhe-space-2);\n  border-radius: var(--fandhe-radius-md);\n  background-color: var(--fandhe-color-bg-canvas);\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-card-media-footer-preview][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-card-media-footer-save][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-scope=\"menu\"][data-part=\"trigger\"]#blocks-card-media-footer-menu[data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        assert!(html.contains(r#"data-scope="card""#));
        assert!(html.contains(r#"data-scope="image""#) || html.contains("<img"));
        assert!(html.contains(r#"data-scope="badge""#));
        assert!(html.contains("<h3"));
        assert!(html.contains(r#"data-scope="text""#) || html.contains("<p"));
        assert!(html.contains(r#"data-scope="avatar""#));
        assert!(html.contains(r#"data-scope="button""#));
        assert!(html.contains(r#"data-scope="menu""#));
    }

    #[test]
    fn demo_counts_and_safety() {
        let html = render(&demo());
        let card_count = html
            .matches(r#"data-blocks-card-media-footer-card"#)
            .count();
        assert_eq!(card_count, 5, "単体 2 枚 + グリッド 3 枚の合計 5 枚のはず");
        let toolbar_count = html.matches(r#"role="toolbar""#).count();
        assert_eq!(toolbar_count, 1, "操作バーはカード b の 1 つだけのはず");
        assert!(!html.contains("<form"));
        assert!(!html.contains("data:"));
        for button_tag in html.match_indices("<button").map(|(i, _)| i) {
            let snippet = &html[button_tag..(button_tag + 200).min(html.len())];
            assert!(
                snippet.contains(r#"type="button""#),
                "すべての button は type=\"button\" を持つはず: {snippet}"
            );
        }
    }

    #[test]
    fn layout_css_uses_tokens_and_collapses_to_one_column() {
        assert!(LAYOUT_CSS.contains("39.99rem"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: 1fr;"));
        assert!(!LAYOUT_CSS.contains("</style"));
        assert!(!LAYOUT_CSS.contains('<'));
    }
}
