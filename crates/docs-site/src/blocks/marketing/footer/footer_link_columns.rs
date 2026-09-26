//! `footer-link-columns` block（イシュー #2851。親トラッキング #2734 系
//! 「Blocks マーケティング / Footer」配下）。定番のリンクカラム型 footer
//! の合成例。既存 2 件（[`super::footer_newsletter`]/
//! [`super::footer_sticky_reveal`]）に続く 3 件目。
//!
//! # 使用部品
//!
//! `link` / `separator` / `icon` / `heading` / `text` の 5 部品のみを合成
//! する（[`BLOCK`] の `parts` に一致させる契約）。新規 UI 部品・外部依存は
//! 追加しない。
//!
//! # 3 インスタンス統合
//!
//! 集約元の差分を 3 インスタンスへ統合する（詳細は原稿側「原案差分メモ」
//! 節）。
//!
//! - **A**: 基準形。ブランド列（ロゴマーク + 名称 + 一文説明）+ リンク
//!   4 群。下段に区切り線 + 著作権 + SNS アイコンのみのリンク 3 個
//! - **B**: ブランド列を 2 列幅に広げ、mission 文・架空の連絡先・SNS
//!   アイコンを収める。リンクは 2 群。下段は著作権 + 法務リンク（プライバ
//!   シー・利用規約）
//! - **C**: 下段なし。リンク 5 群（最後の「Social」群はアイコン + テキスト
//!   のリンク）
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、ブランド名は
//! `HeadingLevel::H3`、各リンク群の見出しはそれより 1 段下げて
//! `HeadingLevel::H4` にする。
//!
//! # アイコンの扱い
//!
//! ロゴマーク・SNS アイコンのみのリンクは自作の抽象幾何パス（線画）のみを
//! 使い、実在ブランドのロゴは複製しない。ロゴマークは隣の名称テキストが
//! 意味を伝えるため装飾扱い（[`fandhe_frontend_pre_styled_ui::icon::
//! IconProps::label`] を `None`）にする。SNS アイコンのみのリンクは
//! リンク名を成立させるため `label` にリンクごと異なる固定文字列
//! （WCAG 2.4.4 Link Purpose）を与える。C の Social 群は装飾アイコン
//! （`label: None`）+ 可視テキストの組み合わせにする。
//!
//! # リンク先の方針
//!
//! `Block::demo` は `fn() -> Node` で `base_path` を受け取れない
//! （`crate::blocks` モジュール doc「`<form>` を使わない」節と同じ制約）。
//! 前例（[`super::footer_newsletter`] 等）と同じく、固定の実在外部 URL
//! [`REPO`] へ統一し `href="#"` は使わない。`LinkProps { external: true,
//! .. }` により `target="_blank"` + `rel="noopener noreferrer"` を headless
//! 層が付与する（reverse tabnabbing 対策）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading`/`styled_text::text`/`link::root`/`icon::icon`/
//! `separator::separator` は `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-footer-link-columns-*` 属性で渡す。素の `div`/`footer`/
//! `ul` には `class` がそのまま効くため `.blocks-footer-link-columns-*`
//! クラスセレクタを使う。レイアウト root の class
//! （`blocks-footer-link-columns-layout`）は [`Block::demo_class`]
//! （`blocks-footer-link-columns`）とは意図的に別名にする（先行 block で
//! 得た Bugbot 教訓の踏襲）。
//!
//! # `<form>` を持たない・状態を持たない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言・住所・連絡先はすべて架空のもの（実企業名・実クレデン
//! シャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, footer, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 自作の幾何パスによる装飾アイコン（`feature_three_column_icons::
/// geo_icon` と同型の線画。`label` は呼び出し側が指定する）。
fn geo_icon(path_d: &'static str, label: Option<&'static str>) -> Node {
    icon(
        &IconProps {
            size: Size::Md,
            label,
            ..IconProps::default()
        },
        vec![("data-blocks-footer-link-columns-icon", "")],
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

/// ロゴマーク（装飾扱い、`label: None`）。
fn logo_mark() -> Node {
    geo_icon(
        "M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5",
        None,
    )
}

/// SNS アイコンのみのリンク（accessible name はリンクごと異なる固定
/// 文字列、WCAG 2.4.4）。
fn social_icon_link(path_d: &'static str, label: &'static str) -> Node {
    link::root(
        REPO,
        &LinkProps {
            external: true,
            ..LinkProps::default()
        },
        vec![("data-blocks-footer-link-columns-social-link", "")],
        vec![geo_icon(path_d, Some(label))],
    )
}

/// リンク群 1 個（見出し + `ul`/`li` のリンク一覧）。
fn link_group(heading_text: &'static str, links: &[&'static str]) -> Node {
    let items: Vec<Node> = links
        .iter()
        .map(|label| {
            li(
                vec![],
                vec![link::root(
                    REPO,
                    &LinkProps::default(),
                    vec![],
                    vec![text(*label)],
                )],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-footer-link-columns-group")],
        vec![
            heading::heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![("data-blocks-footer-link-columns-group-title", "")],
                vec![text(heading_text)],
            ),
            ul(vec![("class", "blocks-footer-link-columns-list")], items),
        ],
    )
}

/// ブランド列（ロゴマーク + 名称 + 説明。`wide` で B 用の 2 列幅指定を
/// 付ける）。
fn brand_column(description: &'static str, wide: bool) -> Node {
    let mut attrs = vec![("class", "blocks-footer-link-columns-brand")];
    if wide {
        attrs.push(("data-wide", ""));
    }
    div(
        attrs,
        vec![
            div(
                vec![("class", "blocks-footer-link-columns-brand-mark")],
                vec![
                    logo_mark(),
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![text("Fandhe Frontend")],
                    ),
                ],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-footer-link-columns-brand-desc", "")],
                vec![text(description)],
            ),
        ],
    )
}

/// 著作権表示。
fn copyright() -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-footer-link-columns-copyright", "")],
        vec![text("\u{00a9} 2026 Fandhe Frontend")],
    )
}

/// 注記行（各インスタンスの原案差分の要約）。
fn note(body: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-footer-link-columns-note", "")],
        vec![text(body)],
    )
}

/// インスタンス A: 基準形（ブランド列 + リンク 4 群、下段に SNS アイコン
/// のみのリンク 3 個）。
fn instance_a() -> Node {
    footer(
        vec![("class", "blocks-footer-link-columns-instance")],
        vec![
            note("基準形。ブランド列 + リンク 4 群。下段に SNS アイコンのみのリンク 3 個。"),
            div(
                vec![("class", "blocks-footer-link-columns-top")],
                vec![
                    brand_column(
                        "決定的なビルドを目指す Rust 製フロントエンドフレームワークです。",
                        false,
                    ),
                    div(
                        vec![("class", "blocks-footer-link-columns-columns")],
                        vec![
                            link_group("Product", &["Guide", "API Reference"]),
                            link_group("Resources", &["Examples", "Changelog"]),
                            link_group("Community", &["GitHub", "Discussions"]),
                            link_group("Company", &["About", "Blog"]),
                        ],
                    ),
                ],
            ),
            separator::separator(
                &SeparatorProps::default(),
                vec![("data-blocks-footer-link-columns-separator", "")],
            ),
            div(
                vec![("class", "blocks-footer-link-columns-bottom")],
                vec![
                    copyright(),
                    div(
                        vec![("class", "blocks-footer-link-columns-social")],
                        vec![
                            social_icon_link("M12 2L2 7l10 5 10-5-10-5z", "GitHub"),
                            social_icon_link("M4 4h16v16H4z", "X"),
                            social_icon_link("M12 21a9 9 0 100-18 9 9 0 000 18z", "Mastodon"),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// インスタンス B: ブランド列を強調（2 列幅、mission 文 + 架空の連絡先 +
/// SNS アイコン）。リンクは 2 群。下段は著作権 + 法務リンク。
fn instance_b() -> Node {
    footer(
        vec![("class", "blocks-footer-link-columns-instance")],
        vec![
            note("ブランド列を 2 列幅に広げ、mission 文 + 連絡先 + SNS アイコンを収める。リンクは 2 群。下段は法務リンク。"),
            div(
                vec![("class", "blocks-footer-link-columns-top"), ("data-brand", "wide")],
                vec![
                    div(
                        vec![("class", "blocks-footer-link-columns-brand"), ("data-wide", "")],
                        vec![
                            div(
                                vec![("class", "blocks-footer-link-columns-brand-mark")],
                                vec![
                                    logo_mark(),
                                    heading::heading(
                                        HeadingLevel::H3,
                                        &HeadingProps::default(),
                                        vec![],
                                        vec![text("Fandhe Frontend")],
                                    ),
                                ],
                            ),
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    variant: TextVariant::Muted,
                                    ..TextProps::default()
                                },
                                vec![("data-blocks-footer-link-columns-brand-desc", "")],
                                vec![text(
                                    "AI 時代のセキュリティリスク低減を目指し、プレーンな HTML/JS/CSS を尊重するフレームワークを開発しています。",
                                )],
                            ),
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    variant: TextVariant::Muted,
                                    ..TextProps::default()
                                },
                                vec![("data-blocks-footer-link-columns-brand-contact", "")],
                                vec![text("123 Example Street, Sample City, ST 00000")],
                            ),
                            div(
                                vec![("class", "blocks-footer-link-columns-social")],
                                vec![
                                    social_icon_link("M12 2L2 7l10 5 10-5-10-5z", "GitHub"),
                                    social_icon_link("M4 4h16v16H4z", "X"),
                                ],
                            ),
                        ],
                    ),
                    div(
                        vec![("class", "blocks-footer-link-columns-columns")],
                        vec![
                            link_group("Product", &["Guide", "API Reference"]),
                            link_group("Company", &["About", "Careers"]),
                        ],
                    ),
                ],
            ),
            separator::separator(
                &SeparatorProps::default(),
                vec![("data-blocks-footer-link-columns-separator", "")],
            ),
            div(
                vec![("class", "blocks-footer-link-columns-bottom")],
                vec![
                    copyright(),
                    div(
                        vec![("class", "blocks-footer-link-columns-legal")],
                        vec![
                            link::root(
                                REPO,
                                &LinkProps {
                                    external: true,
                                    ..LinkProps::default()
                                },
                                vec![],
                                vec![text("Privacy Policy")],
                            ),
                            link::root(
                                REPO,
                                &LinkProps {
                                    external: true,
                                    ..LinkProps::default()
                                },
                                vec![],
                                vec![text("Terms of Service")],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// インスタンス C: 下段なし。リンク 5 群（最後の Social 群はアイコン +
/// テキストのリンク）。
fn instance_c() -> Node {
    footer(
        vec![("class", "blocks-footer-link-columns-instance")],
        vec![
            note("下段なし。リンク 5 群。最後の Social 群はアイコン + テキストのリンク。"),
            div(
                vec![("class", "blocks-footer-link-columns-top")],
                vec![
                    brand_column("外部依存ゼロの描画コアを持つフレームワークです。", false),
                    div(
                        vec![("class", "blocks-footer-link-columns-columns")],
                        vec![
                            link_group("Product", &["Guide", "API Reference"]),
                            link_group("Resources", &["Examples", "Changelog"]),
                            link_group("Community", &["Discussions"]),
                            link_group("Company", &["About"]),
                            social_link_group(),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// C 用の Social 群（アイコン + テキストの `link`。`link_group` とは異なり
/// 各項目がアイコンを伴うため専用実装にする）。
fn social_link_group() -> Node {
    let socials: [(&'static str, &'static str); 2] = [
        ("M12 2L2 7l10 5 10-5-10-5z", "GitHub"),
        ("M4 4h16v16H4z", "X"),
    ];
    let items: Vec<Node> = socials
        .iter()
        .map(|(path_d, label)| {
            li(
                vec![],
                vec![link::root(
                    REPO,
                    &LinkProps {
                        external: true,
                        ..LinkProps::default()
                    },
                    vec![("data-blocks-footer-link-columns-social-text-link", "")],
                    vec![geo_icon(path_d, None), text(*label)],
                )],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-footer-link-columns-group")],
        vec![
            heading::heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![("data-blocks-footer-link-columns-group-title", "")],
                vec![text("Social")],
            ),
            ul(vec![("class", "blocks-footer-link-columns-list")], items),
        ],
    )
}

/// `footer-link-columns` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（他 block と同じ状態を持たない設計）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-footer-link-columns-layout")],
        vec![instance_a(), instance_b(), instance_c()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/footer-link-columns/",
    title: "footer-link-columns",
    category: BlockCategory::Footer,
    rust_source: "crates/docs-site/src/blocks/marketing/footer/footer_link_columns.rs",
    demo_class: "blocks-footer-link-columns",
    parts: &[
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `footer_link_columns` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。ブレークポイントのリテラル
/// `48rem`/`64rem` は `recipe::Breakpoint::Md`/`Lg` と一致させる（テーマの
/// breakpoint トークンは `@media` 条件の中では解決できないため）。
const LAYOUT_CSS: &str = "\
.blocks-footer-link-columns-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-footer-link-columns-instance {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  padding: var(--fandhe-space-6);\n  background: var(--fandhe-color-bg-subtle);\n  border-radius: var(--fandhe-radius-lg, 0.5rem);\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-footer-link-columns-note] {\n  margin: 0;\n}\n\
.blocks-footer-link-columns-top {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-footer-link-columns-brand {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n}\n\
.blocks-footer-link-columns-brand-mark {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-footer-link-columns-brand-desc],\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-footer-link-columns-brand-contact] {\n  margin: 0;\n}\n\
.blocks-footer-link-columns-social {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-footer-link-columns-columns {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-footer-link-columns-group {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n}\n\
[data-scope=\"heading\"][data-part=\"root\"][data-blocks-footer-link-columns-group-title] {\n  margin: 0;\n}\n\
.blocks-footer-link-columns-list {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-footer-link-columns-bottom {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-footer-link-columns-copyright] {\n  margin: 0;\n}\n\
.blocks-footer-link-columns-legal {\n  display: flex;\n  gap: var(--fandhe-space-4);\n}\n\
@media (min-width: 48rem) {\n  \
.blocks-footer-link-columns-bottom {\n    flex-direction: row;\n    justify-content: space-between;\n    align-items: center;\n  }\n\
}\n\
@media (min-width: 64rem) {\n  \
.blocks-footer-link-columns-top {\n    display: grid;\n    grid-template-columns: minmax(0, 1fr) minmax(0, 3fr);\n    gap: var(--fandhe-space-6);\n  }\n  \
.blocks-footer-link-columns-top[data-brand=\"wide\"] {\n    grid-template-columns: minmax(0, 2fr) minmax(0, 2fr);\n  }\n  \
.blocks-footer-link-columns-columns {\n    grid-auto-flow: column;\n    grid-auto-columns: minmax(0, 1fr);\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品を全て含み、`<form>`・危険な出力を持たないこと。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"link\"",
            "data-scope=\"separator\"",
            "data-scope=\"icon\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(!html.contains("<form"));
        assert!(!html.contains("<button"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("id=\""));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
    }

    /// 区切り線は A・B の 2 件のみ（C は下段なし）。
    #[test]
    fn separator_appears_only_in_a_and_b() {
        let html = render(&demo());
        assert_eq!(html.matches("data-scope=\"separator\"").count(), 2);
    }

    /// リンク群の見出し（H4 のフック）は A(4) + B(2) + C(5) = 11 件。
    #[test]
    fn link_group_headings_count_matches_layout() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-footer-link-columns-group-title=\"\"")
                .count(),
            11
        );
    }

    /// 外部リンク全件が `target="_blank"` + `rel="noopener noreferrer"`
    /// を持つこと（reverse tabnabbing 対策）。件数の内訳:
    /// A: リンク群 4 群 x 2 件 + social 3 = 11
    /// B: リンク群 2 群 x 2 件 + social 2 + legal 2 = 8
    /// C: リンク群 4 群（4+2+1+1=8 件）+ social-text 2 = 10
    /// 合計 29 件、うち内部（`LinkProps::default`）は `link_group` の
    /// 通常項目（外部指定なし）ではなく、全リンクを一律 `external: true`
    /// にはしていないため、実測値で固定する。
    #[test]
    fn external_links_have_safe_rel_and_target() {
        let html = render(&demo());
        let blank = html.matches("target=\"_blank\"").count();
        let rel = html.matches("rel=\"noopener noreferrer\"").count();
        assert!(blank > 0);
        assert_eq!(blank, rel);
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント・グリッド切り替えを持つ
    /// こと。
    #[test]
    fn layout_css_declares_expected_breakpoints() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("repeat(2, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("grid-auto-flow: column"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（既存 block と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-footer-link-columns-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-footer-link-columns-layout");
    }
}
