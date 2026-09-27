//! `team-bio-rows` block（イシュー #2880。親トラッキング #2807）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `image` / `separator` / `link` / `icon` の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は
//! 追加しない。
//!
//! # 2 variant 併記（無 JS での静的表示）
//!
//! docs サイトは JS ハイドレーションを行わない設計（CLAUDE.md）のため、
//! 「見出し左・区切り線付き縦リスト」（`variant="side"`）と「見出し上・
//! 2 列グリッド」（`variant="top"`）を 1 ページに縦へ併記する（片方を
//! 選んで JS で切り替える機構は持たない）。狭幅ではどちらの variant も
//! 各メンバーの写真を本文の上に縦積みにする（[`LAYOUT_CSS`] 参照）。
//! 参照元の文言・配色・アイコンは持ち込まず、デモ文言・アイコン形状は
//! 独自に用意する。
//!
//! # CSS フックに data 属性を使う理由
//!
//! `image::image` / `heading::heading` / `text::text` / `separator::separator` /
//! `link::root` / `icon::icon` はいずれも `drop_class_attr` により呼び出し
//! 側 `attrs` の `class` を黙って除去するため、Demo 固有のスタイルフックは
//! `data-blocks-team-bio-rows-*` 属性で渡す（`header_simple_bar` と同じ
//! 判断）。素の `div`/`ul`/`li` には `class` をそのまま使う。
//!
//! # `href` の方針
//!
//! `href` は実在する自リポジトリ・組織の URL に限る（`href="#"` は
//! `linkcheck` が拒否する死リンクのため使わない）。
//!
//! # SNS リンクのアクセシブルネーム（WCAG 2.4.4）
//!
//! 架空メンバーは実在の SNS アカウントを持たないため、全メンバー共通で
//! 同じ自リポジトリ・自組織 URL へ揃える（`team-avatar-grid` と同じ
//! 判断）。アクセシブルネームにメンバー名を含めると「メンバーごとの
//! 個別リンク」であるかのように誤誘導するため、遷移先と食い違わない
//! 固定文字列（[`REPO_LABEL`]/[`ORG_LABEL`]）を全メンバー共通で使う。
//!
//! # 画像 `alt=""` の理由
//!
//! [`dummy_assets::AVATAR_SRC`] は全メンバー共通のプレースホルダー画像
//! であり、氏名は画像のすぐ下に見出し（[`HeadingLevel::H4`]）として
//! 常に出力される。氏名を `alt` に入れると実際の画像内容と食い違う
//! （同じプレースホルダーなのに個別の写真であるかのように誤誘導する）
//! うえ、隣接する氏名見出しと重複して読み上げられるため、装飾画像扱い
//! （`alt=""`）にする（`team-avatar-grid` と同じ判断）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 実在の自リポジトリ URL（`href` の方針、モジュール doc 参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
/// 実在の自組織 URL。
const ORG: &str = "https://github.com/Fandhe-AI";
/// [`REPO`] の accessible name（遷移先と食い違わない固定文字列、モジュール
/// doc「SNS リンクのアクセシブルネーム」節参照。全メンバー共通）。
const REPO_LABEL: &str = "fandhe-frontend の GitHub リポジトリ";
/// [`ORG`] の accessible name（同上）。
const ORG_LABEL: &str = "Fandhe-AI の GitHub 組織ページ";

/// メンバーの紹介文（架空の日本語、検索インデックス容量対策で短くする）。
const BIOS: &[&str] = &[
    "小さく試して確かめてから広げる進め方を、チーム全体に広めています。",
    "利用者からのフィードバックを設計へ素早く反映する仕組みを作ります。",
    "運用で見えた課題を、次の開発計画へ落とし込む役割を担っています。",
    "数字の裏側にある背景を掘り下げ、次の一手を提案しています。",
];

/// SNS アイコン用の装飾幾何アイコン（`label` は呼び出し側が指定する）。
fn geo_icon(size: Size, label: &str, path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size,
            label: Some(label),
            ..IconProps::default()
        },
        vec![],
        vec![fandhe_frontend_core::el(
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

/// メンバー共通の SNS リンク行（実在の自リポジトリ・自組織 URL のみ。
/// アクセシブルネームが遷移先と食い違わないよう全メンバー共通の固定文字列
/// を使う、モジュール doc「SNS リンクのアクセシブルネーム」節参照）。
fn social_links() -> Node {
    ul(
        vec![("class", "blocks-team-bio-rows-social")],
        vec![
            li(
                vec![],
                vec![link::root(
                    REPO,
                    &LinkProps {
                        external: true,
                        ..LinkProps::default()
                    },
                    vec![],
                    vec![geo_icon(Size::Sm, REPO_LABEL, "M4 4h16v6H4zM4 14h16v6H4z")],
                )],
            ),
            li(
                vec![],
                vec![link::root(
                    ORG,
                    &LinkProps {
                        external: true,
                        ..LinkProps::default()
                    },
                    vec![],
                    vec![geo_icon(
                        Size::Sm,
                        ORG_LABEL,
                        "M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5",
                    )],
                )],
            ),
        ],
    )
}

/// メンバー 1 名分（縦長の写真 + 氏名・役職・紹介文・SNS リンク）。
fn member(index: usize) -> Node {
    let name = dummy_assets::PERSON_NAMES[index];
    let role = dummy_assets::JOB_TITLES[index % dummy_assets::JOB_TITLES.len()];
    let bio = BIOS[index % BIOS.len()];
    div(
        vec![("data-blocks-team-bio-rows-member", "")],
        vec![
            image::image(
                &ImageProps {
                    aspect_ratio: AspectRatio::Portrait,
                    ..ImageProps::new(dummy_assets::AVATAR_SRC, "")
                },
                vec![("data-blocks-team-bio-rows-photo", "")],
            ),
            div(
                vec![("class", "blocks-team-bio-rows-body")],
                vec![
                    heading::heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(name)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(role)],
                    ),
                    styled_text::text(&TextProps::default(), vec![], vec![text(bio)]),
                    social_links(),
                ],
            ),
        ],
    )
}

/// 見出し左・区切り線付き縦リスト（主参照 R1354、`variant="side"`）。
fn instance_side() -> Node {
    let members: Vec<Node> = (0..3)
        .map(|i| {
            let item = member(i);
            if i == 0 {
                li(vec![], vec![item])
            } else {
                li(
                    vec![],
                    vec![
                        separator::separator(
                            &SeparatorProps::default(),
                            vec![("data-blocks-team-bio-rows-separator", "")],
                        ),
                        item,
                    ],
                )
            }
        })
        .collect();
    div(
        vec![
            ("class", "blocks-team-bio-rows-layout"),
            ("data-blocks-team-bio-rows-variant", "side"),
        ],
        vec![
            div(
                vec![("class", "blocks-team-bio-rows-intro")],
                vec![
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl2,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("チーム紹介")],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("日々の運用を支えるメンバーです。")],
                    ),
                ],
            ),
            ul(vec![("class", "blocks-team-bio-rows-list")], members),
        ],
    )
}

/// 見出し上・2 列グリッド（集約元 R1355、`variant="top"`）。
fn instance_top() -> Node {
    let members: Vec<Node> = (0..4).map(|i| li(vec![], vec![member(i)])).collect();
    div(
        vec![
            ("class", "blocks-team-bio-rows-layout"),
            ("data-blocks-team-bio-rows-variant", "top"),
        ],
        vec![
            div(
                vec![("class", "blocks-team-bio-rows-intro")],
                vec![
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl2,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text("メンバー")],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("それぞれの持ち場で運用を進めています。")],
                    ),
                ],
            ),
            ul(vec![("class", "blocks-team-bio-rows-grid")], members),
        ],
    )
}

/// caption（並記された各 variant の見出し）。
fn caption(label: &str) -> Node {
    fandhe_frontend_core::p(
        vec![("class", "blocks-team-bio-rows-caption")],
        vec![text(label)],
    )
}

/// `team-bio-rows` の Demo 本体（2 variant 併記）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-team-bio-rows-stack")],
        vec![
            caption("見出し左・区切り線付き縦リスト"),
            instance_side(),
            caption("見出し上・2 列グリッド"),
            instance_top(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/team-bio-rows/",
    title: "team-bio-rows",
    category: BlockCategory::Team,
    rust_source: "crates/docs-site/src/blocks/marketing/team/team_bio_rows.rs",
    demo_class: "blocks-team-bio-rows",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
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

/// `team_bio_rows` 固有のレイアウト規則（`--fandhe-*` トークンのみ使用）。
/// セレクタは `.blocks-team-bio-rows-*` と `[data-blocks-team-bio-rows-*]`
/// に限定する。既定（狭幅）は写真を本文の上に縦積みにし、40rem 以上で
/// 横並びへ切り替える。`side` variant は 48rem 以上で見出し左列・
/// メンバー右列の 2 列グリッドに、`top` variant は 64rem 以上でメンバーを
/// 2 列グリッドにする。
const LAYOUT_CSS: &str = "\
.blocks-team-bio-rows-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-team-bio-rows-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-team-bio-rows-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-team-bio-rows-intro {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-team-bio-rows-list {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-team-bio-rows-grid {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-team-bio-rows-member] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-team-bio-rows-photo] {\n  max-inline-size: 8rem;\n}\n\
.blocks-team-bio-rows-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-team-bio-rows-social {\n  list-style: none;\n  margin: var(--fandhe-space-1) 0 0;\n  padding: 0;\n  display: flex;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"separator\"][data-part=\"root\"][data-blocks-team-bio-rows-separator] {\n  margin-block: var(--fandhe-space-4) 0;\n}\n\
@media (min-width: 40rem) {\n  \
[data-blocks-team-bio-rows-member] {\n    flex-direction: row;\n    align-items: flex-start;\n  }\n  \
[data-scope=\"image\"][data-part=\"root\"][data-blocks-team-bio-rows-photo] {\n    flex: 0 0 8rem;\n  }\n\
}\n\
@media (min-width: 48rem) {\n  \
.blocks-team-bio-rows-layout[data-blocks-team-bio-rows-variant=\"side\"] {\n    flex-direction: row;\n    align-items: flex-start;\n  }\n  \
.blocks-team-bio-rows-layout[data-blocks-team-bio-rows-variant=\"side\"] .blocks-team-bio-rows-intro {\n    flex: 0 0 16rem;\n  }\n  \
.blocks-team-bio-rows-layout[data-blocks-team-bio-rows-variant=\"side\"] .blocks-team-bio-rows-list {\n    flex: 1;\n  }\n\
}\n\
@media (min-width: 64rem) {\n  \
.blocks-team-bio-rows-layout[data-blocks-team-bio-rows-variant=\"top\"] .blocks-team-bio-rows-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 6 部品を持ち、非対話制約（`<form>` なし・
    /// `href="#"` なし・`data:` src なし）を満たすこと。
    #[test]
    fn demo_composes_expected_parts_and_has_no_form() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"image\"",
            "data-scope=\"separator\"",
            "data-scope=\"link\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 2 variant がそれぞれ 1 回ずつ出て、caption も 2 件あること。
    #[test]
    fn demo_renders_both_variants() {
        let html = render(&demo());
        assert!(html.contains("data-blocks-team-bio-rows-variant=\"side\""));
        assert!(html.contains("data-blocks-team-bio-rows-variant=\"top\""));
        assert_eq!(html.matches("blocks-team-bio-rows-caption").count(), 2);
    }

    /// portrait の写真が side（3 件）+ top（4 件）= 7 件出ること。
    #[test]
    fn demo_renders_seven_portrait_photos() {
        let html = render(&demo());
        assert_eq!(html.matches("data-blocks-team-bio-rows-photo").count(), 7);
        assert_eq!(html.matches("aspect-ratio-portrait").count(), 7);
    }

    /// side variant の区切り線がメンバー数 3 − 1 = 2 本であること。
    #[test]
    fn side_variant_has_separators_between_members() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-team-bio-rows-separator").count(),
            2
        );
    }

    /// SNS リンクが `target`/`rel`（external）を持ち、icon に `aria-label`
    /// があること。ラベルは全メンバー共通の固定文字列（メンバー名を含まず
    /// 遷移先と食い違わない、WCAG 2.4.4）。
    #[test]
    fn social_links_are_external_with_labeled_icons() {
        let html = render(&demo());
        assert!(html.contains("target=\"_blank\""));
        assert!(html.contains("rel=\"noopener noreferrer\""));
        assert!(html.contains(super::REPO_LABEL));
        assert!(html.contains(super::ORG_LABEL));
    }

    /// [`LAYOUT_CSS`] が狭幅の縦積みと各 `@media (min-width: …)` を持つこと。
    #[test]
    fn layout_css_has_responsive_rules() {
        assert!(LAYOUT_CSS.contains("flex-direction: column;"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-team-bio-rows-layout"));
        assert_ne!(super::BLOCK.demo_class, "blocks-team-bio-rows-layout");
    }
}
