//! `profile-card-centered` block（イシュー #2936。親トラッキング #2892
//! 「Blocks 目的別パーツ拡充」配下。Application / Profile カテゴリ 4 件目の
//! block、カテゴリ卒業の手順は `docs/design/docs-site-blocks-section.md`
//! §18 参照）。中央寄せのプロフィールカードの合成例。対応表 ID R0217
//! （主参照・代表構成）・R0224（集約元・全幅ボタン + リンク縦並びの最小
//! 版）を構造の参照元とする。`_/blocks-intake/` の対応ファイルは本イシュー
//! 着手時点で本 worktree に存在しないため、原稿・本コメントには対応表 ID
//! のみを記す（`list-title-meta`〔イシュー #2925〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `card` / `avatar` / `badge` / `heading` / `text` / `button` / `link` /
//! `icon` の 8 部品のみを合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 認証済みを可視テキスト付き badge で表す理由
//!
//! `avatar::badge` は色ドットのみで文字情報を持たないため、色だけで状態を
//! 伝えないという `list_title_meta::deploy_status` と同型の判断により
//! 不採用とし、可視テキスト「認証済み」を持つ [`badge::badge`] を氏名の
//! 隣に置く。
//!
//! # SNS リンク先が全件 [`REPO`] である理由
//!
//! 架空人物には実在の SNS アカウントが存在しないため、全リンク先を
//! リポジトリへ統一する（`team_avatar_grid::social_link` と同じ判断）。
//!
//! # アイコン単独リンク（[`social_links`]）と可視テキスト付きリンク
//! （[`minimal_card`] のリンク一覧）でのアクセシブル名の作り分け
//!
//! アイコンのみで可視テキストを持たない [`social_link`] は、アイコン
//! 自体に [`REPO_LABEL`] を `label` として与え `role="img"` +
//! `aria-label` で遷移先と食い違わない名前を出す。一方
//! [`minimal_card`] のリンク一覧は隣に可視テキストを持つため、アイコンは
//! 装飾（`label: None`）とし、可視テキスト自体を遷移先（GitHub
//! リポジトリ）と一致する文言（「連絡する（GitHub）」「Webサイト
//! （GitHub）」）にする（イシュー #2936 codex レビュー P1 対応）。アイコン
//! にも `label` を与えて可視テキストと連結させると、アクセシブルネームが
//! 冗長・不整合になる（PR #3388 Bugbot 指摘）ため行わない。
//!
//! # `<form>` を持たない・状態は静的固定
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 状態機械を持たない静的な合成例である。ボタンは
//! `fandhe_frontend_pre_styled_ui::button::button` の既定 `type="button"`
//! のまま用いる。氏名・肩書・所在地・自己紹介はすべて架空のもの（実在の
//! 人物・企業・ブランドとは無関係）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
                    shape: None,
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
    // `link::root` は自身の variant class を組み立てるため呼び出し側の
    // `class` 属性を drop する（`drop_class_attr`、`crates/pre-styled-ui/
    // src/link.rs`）。`class` 以外の属性はそのまま素通りするため、
    // レイアウト用セレクタは `class` ではなく `data-*` 属性で `a` 自身へ
    // 付与する（イシュー #2936 レビュー指摘: (1) `li` へ `class` を
    // 逃がしても `li` の子は `a` 1 個のみで `gap` は効かない、
    // (2) アイコンとテキストの間隔が必要なのは `a` の内側なので、
    // `a` 自体を `inline-flex` にして `gap` を持たせる）。
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
                vec![("data-blocks-profile-card-centered-link-list-item", "")],
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/profile-card-centered/",
    title: "profile-card-centered",
    category: BlockCategory::Profile,
    rust_source: "crates/docs-site/src/blocks/application/profile/profile_card_centered.rs",
    demo_class: "blocks-profile-card-centered",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
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
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `profile_card_centered` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型で private 定数として
/// `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-profile-card-centered-*` と
/// `[data-blocks-profile-card-centered-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない。カードは固定 `max-width` を持たせつつ
/// `width: 100%` で狭幅コンテナに追従させる。
const LAYOUT_CSS: &str = "\
.blocks-profile-card-centered-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  align-items: center;\n}\n\
.blocks-profile-card-centered-section {\n  width: 100%;\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-profile-card-centered-section-title {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-profile-card-centered-card] {\n  width: 100%;\n  max-width: 22rem;\n}\n\
.blocks-profile-card-centered-body {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  text-align: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-profile-card-centered-name-row {\n  display: flex;\n  align-items: center;\n  flex-wrap: wrap;\n  justify-content: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-profile-card-centered-social {\n  display: flex;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-profile-card-centered-actions {\n  display: flex;\n  flex-wrap: wrap;\n  justify-content: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-profile-card-centered-cta-full] {\n  width: 100%;\n}\n\
.blocks-profile-card-centered-link-list {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  width: 100%;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-profile-card-centered-link-list-item] {\n  display: inline-flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"avatar\"",
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"button\"",
            "data-scope=\"link\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn no_form_semantics_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    #[test]
    fn all_buttons_are_type_button() {
        let html = demo_html();
        let count_open = html.matches("<button").count();
        let count_typed = html.matches("type=\"button\"").count();
        assert!(count_open > 0);
        assert!(count_typed >= count_open, "html={html}");
    }

    #[test]
    fn layout_css_is_safe() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains(".blocks-profile-card-centered-layout"));
        assert!(LAYOUT_CSS.contains("[data-blocks-profile-card-centered-cta-full]"));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-profile-card-centered-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-profile-card-centered-layout"
        );
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }
}
