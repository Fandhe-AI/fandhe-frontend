//! `footer-cta-columns` block（イシュー #2849。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、対応表 ID R0961 を主参照とする
//! 合成例。CTA 付きリンクカラム footer）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `link` / `separator` / `icon` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、`crates/docs-site/tests/
//! blocks_nav.rs`/`blocks_contract.rs` が検証する）。イシュー本文の部品
//! 候補には `button` も挙げられていたが、下記「CTA はリンクとして描く」
//! 節の理由により `<button>` を出力しないため `parts` へは含めない。
//!
//! # 3 段構成（イシュー本文のレイアウト仕様）
//!
//! 1. 上段: 中央寄せの CTA（小見出し・見出し・説明文・ボタン風リンク）。
//! 2. 区切り線の下の中段: ロゴと 4 列のリンク。
//! 3. 区切り線の下の下段: SNS アイコンのリンクと著作権表記。
//!
//! # CTA はリンクとして描く（`<button>` を出力しない）
//!
//! イシュー本文が「CTA のボタンは `<a>`（リンク）として描き、フォームは
//! 持たない」と明示するため、[`cta_section`] の CTA は
//! `fandhe_frontend_pre_styled_ui::link::root` をボタン風の外見（
//! [`LAYOUT_CSS`] の `[data-blocks-footer-cta-columns-cta]`）で装飾して描く。
//! `button::button` は `<button type="button">` のみを出力し href を持つ
//! 形を公開していないため、`<a>` の中へ `<button>` を入れ子にする（HTML
//! として不正）ことも、`button` を素通しで置くことも採らない。この判断で
//! `button` を [`BLOCK`] の `parts` から除外している。
//!
//! # 狭い幅ではリンクを 2 列（`grid-template-columns: repeat(2, ...)`）
//!
//! 中段グリッドは既定（`48rem` 未満）で 2 列、`48rem` 以上でロゴ 1 列 +
//! リンク 4 列へ切り替える（[`LAYOUT_CSS`]。ブレークポイントは
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`]（768px =
//! 48rem）と一致するリテラル値を直書きする。`@media` 条件式の中では
//! テーマの breakpoint トークンを解決できないため、既存 block と同じ判断）。
//!
//! # ロゴ・リンク先・SNS 行
//!
//! ロゴは [`super::super::cta::cta_centered`] と同型の自作抽象幾何 span
//! （`aria-hidden="true"`）+ 架空でないサイト名テキスト（本フレームワーク
//! 自身の名称、[`footer_newsletter`](super::footer_newsletter) と同じ
//! 判断）。リンク先はすべて `Fandhe-AI` の実在 GitHub リポジトリへの外部
//! 絶対 URL（`base_path` を受け取らない [`Block::demo`] 契約下で
//! `href="#"` を避ける唯一の実用的な手段、`footer_newsletter`/
//! `contact_split_info` と同じ判断）で、可視テキストは遷移先パスと矛盾
//! しない文言にする。SNS 行は自作幾何 SVG アイコンのみを内容に持つリンク
//! 3 件（実在 SNS ブランドのロゴは描かない）で、アイコンは装飾扱い
//! （`IconProps.label = None` で `aria-hidden`）、リンク側へ遷移先と一致
//! する相異なる `aria-label` を付与する。
//!
//! # `<form>` を使わない・状態を持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>`・
//! `data:` URI・`raw_html()`・HTML 文字列の直接組み立てのいずれも使わない
//! 静的な合成例である。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading`/`styled_text::text`/`link::root`/`separator::separator`/
//! `icon::icon` は `drop_class_attr` により呼び出し側 `attrs` の `class` を
//! 黙って除去する契約を持つため、Demo 固有スタイルは
//! `data-blocks-footer-cta-columns-*` 属性で渡し、素の `div`/`p`/`span` には
//! `.blocks-footer-cta-columns-*` クラスを使う。ルート要素の class は
//! `blocks-footer-cta-columns-layout`（[`Block::demo_class`] の
//! `blocks-footer-cta-columns` とは別名。イシュー #2809 と同じ Bugbot の
//! 教訓）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, footer, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
const REPO_SPEC: &str = "https://github.com/Fandhe-AI/fandhe-frontend-spec";

/// 自作の抽象幾何ロゴ（`cta_centered::logo_mark` と同型）。
fn logo_mark() -> Node {
    span(
        vec![
            ("class", "blocks-footer-cta-columns-logo"),
            ("aria-hidden", "true"),
        ],
        vec![],
    )
}

/// 装飾用の自作幾何アイコン（`contact_split_info::geo_icon` と同型、
/// lucide 等の既存アイコンセットの path を複製しない単純図形）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
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

/// リンク列 1 群（見出し + リンク一覧、`footer_newsletter::link_column` と
/// 同型）。
fn link_column(title: &str, links: &[(&str, &str)]) -> Node {
    let items: Vec<Node> = links
        .iter()
        .map(|(href, label)| {
            div(
                vec![("class", "blocks-footer-cta-columns-link-item")],
                vec![link::root(
                    href,
                    &LinkProps::default(),
                    vec![],
                    vec![text(*label)],
                )],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-footer-cta-columns-column")],
        std::iter::once(p(
            vec![("class", "blocks-footer-cta-columns-column-title")],
            vec![text(title)],
        ))
        .chain(items)
        .collect(),
    )
}

/// SNS 風リンク 1 件（自作幾何アイコンのみ + 遷移先と一致する `aria-label`。
/// `contact_split_info::social_link` と異なり可視テキストを持たないため、
/// 3 件とも `aria-label` を付与する）。
fn social_link(href: &str, aria_label: &str, icon_node: Node) -> Node {
    link::root(
        href,
        &LinkProps::default(),
        vec![
            ("aria-label", aria_label),
            ("data-blocks-footer-cta-columns-social-link", ""),
        ],
        vec![icon_node],
    )
}

/// 上段: 中央寄せの CTA（小見出し・見出し・説明文・ボタン風リンク）。
fn cta_section() -> Node {
    div(
        vec![("class", "blocks-footer-cta-columns-cta")],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-footer-cta-columns-eyebrow", "")],
                vec![text("コミュニティ")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("今すぐプロジェクトに参加しましょう")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "GitHub で最新のリリースを追いかけ、Issue や Pull Request で\
                     フィードバックを送りましょう。",
                )],
            ),
            link::root(
                REPO,
                &LinkProps::default(),
                vec![("data-blocks-footer-cta-columns-cta", "")],
                vec![text("GitHub で見る")],
            ),
        ],
    )
}

/// 中段: ロゴ + 4 列のリンク。
fn columns_section() -> Node {
    div(
        vec![("class", "blocks-footer-cta-columns-columns")],
        vec![
            div(
                vec![("class", "blocks-footer-cta-columns-brand")],
                vec![
                    logo_mark(),
                    p(
                        vec![("class", "blocks-footer-cta-columns-brand-name")],
                        vec![text("Fandhe Frontend")],
                    ),
                ],
            ),
            link_column(
                "Product",
                &[
                    (&format!("{REPO}#readme"), "Guide"),
                    (&format!("{REPO}/tree/main/examples"), "Examples"),
                    (&format!("{REPO}/releases"), "Releases"),
                ],
            ),
            link_column(
                "Docs",
                &[
                    (&format!("{REPO}/tree/main/docs/guides"), "Guides"),
                    (&format!("{REPO}/tree/main/docs/api"), "API Reference"),
                    (&format!("{REPO}/tree/main/docs/design"), "Design Docs"),
                ],
            ),
            link_column(
                "Community",
                &[
                    (&format!("{REPO}/issues"), "Issues"),
                    (&format!("{REPO}/pulls"), "Pull Requests"),
                    (&format!("{REPO}/blob/main/CLAUDE.md"), "CLAUDE.md"),
                ],
            ),
            link_column(
                "Specification",
                &[
                    (&format!("{REPO_SPEC}#readme"), "Spec Repository"),
                    (&format!("{REPO_SPEC}/issues"), "Spec Issues"),
                    (&format!("{REPO_SPEC}/pulls"), "Spec Pull Requests"),
                ],
            ),
        ],
    )
}

/// 下段: SNS アイコンのリンクと著作権表記。
fn bottom_row() -> Node {
    div(
        vec![("class", "blocks-footer-cta-columns-bottom")],
        vec![
            div(
                vec![("class", "blocks-footer-cta-columns-social")],
                vec![
                    social_link(
                        "https://github.com/Fandhe-AI",
                        "GitHub（Fandhe-AI）",
                        geo_icon(
                            "M9 3a6 6 0 00-2 11.6c0 .3 0 1 0 1.9 M9 3a6 6 0 012 11.6c0 .3 0 1 0 1.9 \
                             M6 17c-1.2.5-2 0-2.5-1",
                        ),
                    ),
                    social_link(
                        REPO,
                        "GitHub（fandhe-frontend）",
                        geo_icon(
                            "M4 4h8l4 4v12H4V4z M12 4v4h4 M8 12h6 M8 15h6",
                        ),
                    ),
                    social_link(
                        REPO_SPEC,
                        "GitHub（fandhe-frontend-spec）",
                        geo_icon("M12 3l7 4v10l-7 4-7-4V7l7-4z M12 3v18 M5 7l7 4 7-4"),
                    ),
                ],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-footer-cta-columns-copyright", "")],
                vec![text("© 2026 Fandhe Frontend.")],
            ),
        ],
    )
}

/// `footer-cta-columns` の Demo 本体（モジュール doc「3 段構成」節参照）。
pub fn demo() -> Node {
    footer(
        vec![("class", "blocks-footer-cta-columns-layout")],
        vec![
            cta_section(),
            separator(&SeparatorProps::default(), vec![]),
            columns_section(),
            separator(&SeparatorProps::default(), vec![]),
            bottom_row(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/footer-cta-columns/",
    title: "footer-cta-columns",
    category: BlockCategory::Footer,
    rust_source: "crates/docs-site/src/blocks/marketing/footer/footer_cta_columns.rs",
    demo_class: "blocks-footer-cta-columns",
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
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `footer_cta_columns` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。狭い幅（既定）はリンクを 2 列、
/// `48rem` 以上でロゴ 1 列 + リンク 4 列（モジュール doc「狭い幅では
/// リンクを 2 列」節参照）。
///
/// eyebrow の CSS フックは `class` ではなく `data-*` 属性
/// （`[data-blocks-footer-cta-columns-eyebrow]`）を使う。`styled_text::text`
/// は呼び出し側 `class` を `drop_class_attr` で除去するため（モジュール doc
/// 「CSS フックの選び方」節参照）、`class` セレクタでは出力に到達しない。
///
/// CTA リンク・SNS リンクの `color` は `link` recipe の base `color`
/// （`[data-scope="link"][data-part="root"]`、詳細度 (0,2,0)）と競合する
/// ため、`banner_announcement_pill`（詳細度衝突の先例）と同じ判断で
/// `[data-scope="link"][data-part="root"]` を連結した属性セレクタ 2 個
/// （詳細度 (0,3,0)）へ揃えて必ず勝たせる。他プロパティ（`display`・
/// `padding` 等）は競合しないため単一属性セレクタのままにする。
const LAYOUT_CSS: &str = "\
.blocks-footer-cta-columns-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8, 2rem);\n  padding: var(--fandhe-space-8, 2rem);\n  background: var(--fandhe-color-bg-subtle);\n  border-radius: var(--fandhe-radius-lg, 0.75rem);\n}\n\
.blocks-footer-cta-columns-cta {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  text-align: center;\n  gap: var(--fandhe-space-4, 1rem);\n  max-inline-size: 32rem;\n  margin-inline: auto;\n}\n\
[data-blocks-footer-cta-columns-eyebrow] {\n  text-transform: uppercase;\n  letter-spacing: 0.05em;\n}\n\
[data-blocks-footer-cta-columns-cta] {\n  display: inline-flex;\n  align-items: center;\n  justify-content: center;\n  padding: 0.75rem 1.5rem;\n  border-radius: var(--fandhe-radius-md, 0.375rem);\n  background: var(--fandhe-color-accent);\n  text-decoration: none;\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n}\n\
[data-scope=\"link\"][data-part=\"root\"][data-blocks-footer-cta-columns-cta] {\n  color: var(--fandhe-color-accent-fg);\n}\n\
[data-blocks-footer-cta-columns-cta]:hover {\n  opacity: 0.9;\n}\n\
.blocks-footer-cta-columns-columns {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-6, 1.5rem);\n  min-width: 0;\n}\n\
.blocks-footer-cta-columns-brand {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2, 0.5rem);\n  grid-column: 1 / -1;\n}\n\
.blocks-footer-cta-columns-logo {\n  display: block;\n  inline-size: var(--fandhe-space-8, 2rem);\n  block-size: var(--fandhe-space-8, 2rem);\n  border-radius: var(--fandhe-radius-md, 0.375rem);\n  border: 2px solid currentColor;\n  background: linear-gradient(135deg, currentColor, transparent);\n  opacity: 0.85;\n}\n\
.blocks-footer-cta-columns-brand-name {\n  margin: 0;\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n}\n\
.blocks-footer-cta-columns-column {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2, 0.5rem);\n  min-width: 0;\n}\n\
.blocks-footer-cta-columns-column-title {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n}\n\
.blocks-footer-cta-columns-link-item {\n  display: block;\n}\n\
.blocks-footer-cta-columns-bottom {\n  display: flex;\n  justify-content: space-between;\n  align-items: center;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-4, 1rem);\n}\n\
.blocks-footer-cta-columns-social {\n  display: flex;\n  gap: var(--fandhe-space-3, 0.75rem);\n}\n\
[data-scope=\"link\"][data-part=\"root\"][data-blocks-footer-cta-columns-social-link] {\n  color: var(--fandhe-color-fg-muted);\n}\n\
@media (min-width: 48rem) {\n  .blocks-footer-cta-columns-columns {\n    grid-template-columns: 1.5fr repeat(4, minmax(0, 1fr));\n  }\n  .blocks-footer-cta-columns-brand {\n    grid-column: auto;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// Demo が使用部品 5 件（`data-scope`）をすべて含み、`<form>`・死リンク
    /// （`href="#"`）・`data:` URI・`<button` を含まないこと（モジュール
    /// doc「CTA はリンクとして描く」節が言う `<button>` 非出力の固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forbidden_markup() {
        let html = render(&demo());
        for scope in ["heading", "text", "link", "separator", "icon"] {
            assert!(
                html.contains(&format!("data-scope=\"{scope}\"")),
                "demo output should contain data-scope=\"{scope}\": {html}"
            );
        }
        assert!(!html.contains("<form"), "demo should never contain <form>");
        assert!(
            !html.contains("href=\"#\""),
            "demo should never contain a dead link"
        );
        assert!(
            !html.contains("src=\"data:"),
            "demo should never contain a data: URI"
        );
        assert!(
            !html.contains("<button"),
            "CTA must be a link, not a <button>"
        );
    }

    /// CTA リンクがちょうど 1 件で、外部絶対 URL（`https://`）を持つこと。
    #[test]
    fn cta_link_is_a_single_external_absolute_url() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-footer-cta-columns-cta=\"\"")
                .count(),
            1,
            "there should be exactly one CTA link"
        );
        assert!(html.contains(&format!("href=\"{REPO}\"")));
    }

    /// 区切り線（`data-scope="separator"`）がちょうど 2 件であること
    /// （モジュール doc「3 段構成」節の 2 本の区切り線）。
    #[test]
    fn demo_has_exactly_two_separators() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-scope=\"separator\"").count(),
            2,
            "there should be exactly two separators between the 3 sections"
        );
    }

    /// リンク列のコンテナがちょうど 4 件であること（イシュー本文の
    /// 「4 列のリンク」）。
    #[test]
    fn demo_has_exactly_four_link_columns() {
        let html = render(&demo());
        assert_eq!(
            html.matches("class=\"blocks-footer-cta-columns-column\"")
                .count(),
            4,
            "there should be exactly four link columns"
        );
    }

    /// SNS リンク数と、相異なる `aria-label` の数が一致すること（可視
    /// テキストを持たないアイコンリンクの各遷移先が判別可能であることの
    /// 固定）。
    #[test]
    fn social_links_have_distinct_aria_labels() {
        let html = render(&demo());
        let social_link_count = html
            .matches("data-blocks-footer-cta-columns-social-link")
            .count();
        let mut labels = std::collections::BTreeSet::new();
        for label in [
            "GitHub（Fandhe-AI）",
            "GitHub（fandhe-frontend）",
            "GitHub（fandhe-frontend-spec）",
        ] {
            assert!(
                html.contains(&format!("aria-label=\"{label}\"")),
                "expected aria-label {label} to be present"
            );
            labels.insert(label);
        }
        assert_eq!(social_link_count, labels.len());
    }

    /// [`LAYOUT_CSS`] が狭幅 2 列（`repeat(2,`）と `@media (min-width:
    /// 48rem)` 内の 4 列指定を持つこと（モジュール doc「狭い幅では
    /// リンクを 2 列」節のドリフト防止）。
    #[test]
    fn layout_css_has_narrow_two_column_and_wide_four_column_rules() {
        assert!(LAYOUT_CSS.contains("repeat(2, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("repeat(4, minmax(0, 1fr))"));
    }

    /// ルート class が出力に現れ、[`BLOCK::demo_class`] とは異なること
    /// （イシュー #2809 と同じ Bugbot の教訓、モジュール doc「CSS フックの
    /// 選び方」節参照）。
    #[test]
    fn root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-footer-cta-columns-layout\""));
        assert_ne!(BLOCK.demo_class, "blocks-footer-cta-columns-layout");
    }
}
