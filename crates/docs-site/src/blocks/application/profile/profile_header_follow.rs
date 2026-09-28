//! `profile-header-follow` block（イシュー #2939。Application / Profile
//! カテゴリ、[`super::profile_detail_datalist`] に続く 2 件目）。
//! アバター・氏名・認証バッジ・所属・自己紹介・フォロワー数を束ねる
//! プロフィール見出しと、フォロー操作ボタン群を合成する。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには Issue 本文のレイアウト仕様のみを
//! 根拠とする（`list-title-meta`〔イシュー #2925〕と同じ扱い）。
//!
//! # 使用部品
//!
//! `avatar` / `badge` / `heading` / `text` / `button` / `stat` の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 2 インスタンス（アバター位置の左右反転）
//!
//! Demo は `avatar-start`（アバター左・既定の縦積み → 横並び）と
//! `avatar-end`（アバター右）の 2 インスタンスを並記する。両者とも
//! フォロー状態は「未フォロー」で固定する（状態違いを混ぜるとレイアウト
//! 差分の比較が読み取りにくくなるため）。
//!
//! # 狭幅では縦積み・広幅で横並び（`@media (min-width: 40rem)`）
//!
//! [`LAYOUT_CSS`] のヘッダーラッパーは既定でアバター・本文・操作群を
//! 縦積みにし、`min-width: 40rem`（本リポの主流ブレークポイント）から
//! `grid-template-areas` でアバター・本文を横並びへ切り替える。
//! `avatar-end` インスタンスはアバターを右列へ配置する。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::avatar::root`]・
//! [`fandhe_frontend_pre_styled_ui::heading::heading`]・
//! [`fandhe_frontend_pre_styled_ui::text::text`]・
//! [`fandhe_frontend_pre_styled_ui::badge::badge`]・
//! [`fandhe_frontend_pre_styled_ui::button::button`]・
//! [`fandhe_frontend_pre_styled_ui::stat::root`] はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは
//! `data-blocks-profile-header-follow-*` 属性で渡す
//! （`list_narrow_activity`/`profile_detail_datalist` と同型の判断）。
//! レイアウト用ラッパー（見出し行・本文・操作群）は素の `<div>` のため
//! `class="blocks-profile-header-follow-*"` を使う。
//!
//! # `<form>` を使わない・`id` 属性を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。ボタンは
//! `button::button` の既定 `type="button"` のまま用いる。2 インスタンスを
//! 同一ページへ並べるが `id`/`aria-controls`/`aria-labelledby` を一切出力
//! しないため重複・宙に浮いた参照は発生しない。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、`list_narrow_activity` と同じく前者を `styled_text` へ
//! 別名 import する。
//!
//! # ダミー素材について
//!
//! 氏名・役職・社名は `crate::blocks::dummy_assets`（架空セット）を使う。
//! フォロワー数・フォロー中の数値は架空の固定値とし、実在の人物・企業・
//! PII は含まない。アバター画像はビルド時生成の同梱 SVG
//! （[`dummy_assets::AVATAR_SRC`]）を使う（外部 URL・`data:` URI は使わない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
            stat::value_text(vec![], vec![text(value)]),
            stat::label(vec![], vec![text(label)]),
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/profile-header-follow/",
    title: "profile-header-follow",
    category: BlockCategory::Profile,
    rust_source: "crates/docs-site/src/blocks/application/profile/profile_header_follow.rs",
    demo_class: "blocks-profile-header-follow",
    parts: &[
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
            label: "Stat",
            path: "/themes/stat/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `profile_header_follow` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。
///
/// # 狭幅は縦積み・`min-width: 40rem` から横並び
///
/// ヘッダーは既定 `grid-template-columns: 1fr`（縦積み）とし、
/// `@media (min-width: 40rem)` で `auto minmax(0, 1fr) auto`（アバター・
/// 本文・操作群）の横並びへ切り替える。`avatar-end` は
/// `grid-template-columns` の並びを反転してアバターを右列へ移す。
///
/// # 2 インスタンスの間隔
///
/// `.blocks-profile-header-follow-layout` は `display: grid` + `gap` で
/// 2 件のヘッダーを縦に並べる。
const LAYOUT_CSS: &str = "\
.blocks-profile-header-follow-layout {\n  display: grid;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-profile-header-follow-header {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-5);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md, 0.375rem);\n}\n\
@media (min-width: 40rem) {\n  .blocks-profile-header-follow-header {\n    grid-template-columns: auto minmax(0, 1fr) auto;\n    align-items: start;\n  }\n  [data-blocks-profile-header-follow-variant=\"avatar-end\"] {\n    grid-template-columns: minmax(0, 1fr) auto auto;\n  }\n  [data-blocks-profile-header-follow-variant=\"avatar-end\"] [data-blocks-profile-header-follow-avatar] {\n    order: 2;\n  }\n  [data-blocks-profile-header-follow-variant=\"avatar-end\"] .blocks-profile-header-follow-body {\n    order: 1;\n  }\n  [data-blocks-profile-header-follow-variant=\"avatar-end\"] .blocks-profile-header-follow-actions {\n    order: 3;\n  }\n}\n\
.blocks-profile-header-follow-body {\n  display: grid;\n  gap: var(--fandhe-space-2);\n  min-width: 0;\n}\n\
.blocks-profile-header-follow-name-row {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  flex-wrap: wrap;\n}\n\
.blocks-profile-header-follow-stats {\n  display: flex;\n  gap: var(--fandhe-space-6);\n  margin-top: var(--fandhe-space-2);\n}\n\
.blocks-profile-header-follow-actions {\n  display: flex;\n  align-items: flex-start;\n  gap: var(--fandhe-space-3);\n  flex-wrap: wrap;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 6 種の部品を出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"avatar\"",
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"button\"",
            "data-scope=\"stat\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// avatar-start・avatar-end 双方の variant フックが出力されていること。
    #[test]
    fn demo_wires_both_variant_hooks() {
        let html = render(&demo());
        assert!(html.contains(r#"data-blocks-profile-header-follow-variant="avatar-start""#));
        assert!(html.contains(r#"data-blocks-profile-header-follow-variant="avatar-end""#));
    }

    /// 非対話・安全性の不変条件（`<form>`・`id="..."` を持たず、ボタンは
    /// すべて `type="button"` であること）。
    #[test]
    fn demo_never_contains_forbidden_markup() {
        let html = render(&demo());
        assert!(!html.contains("<form"));
        assert!(!html.contains("id=\""));
        assert!(!html.contains("src=\"data:"));
        let button_count = html.matches("<button").count();
        assert_eq!(button_count, 4, "html={html}"); // 2 インスタンス × 2 ボタン
        assert_eq!(html.matches(r#"type="button""#).count(), button_count);
    }

    /// [`LAYOUT_CSS`] が 40rem ブレークポイントと avatar-end セレクタを
    /// 持つこと。
    #[test]
    fn layout_css_stacks_avatar_below_breakpoint() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("avatar-end"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`blog_list_image` と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-profile-header-follow-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-profile-header-follow-layout"
        );
    }
}
