//! `testimonial-two-up` block（イシュー #2891。親トラッキング #2731
//! 「Blocks 目的別パーツ拡充ツリー」配下、`crate::blocks::marketing::
//! testimonial` カテゴリの block）。
//!
//! # 出典に関する注記
//!
//! 参照元は「2 件の推薦文を左右に並べ、列の間を縦の区切り線で分ける」
//! 構造のみを参照し、Rust/CSS で独自に再実装する（対応表 ID R1363、
//! 主参照。集約元 R0725 は同形、R0728 は上部に見出しを追加した形）。
//! 差分の詳細は `site/blocks/testimonial-two-up.md` の「原案差分メモ」
//! 節を参照する。
//!
//! # 使用部品
//!
//! `blockquote` / `avatar` / `separator` / `icon` / `heading` / `text`
//! の 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # 2 形併記（無 JS での静的表示）
//!
//! docs サイトは JS ハイドレーションを行わない設計（CLAUDE.md）のため、
//! 「見出しなし・2 件並び」（集約元 R0725 相当）と「見出し付き・2 件
//! 並び」（集約元 R0728 相当）を 1 ページに縦へ併記する
//! （`team_bio_rows`/`content_with_testimonial` と同じ判断）。
//!
//! # 区切り線を横 / 縦の 2 個で切り替える理由
//!
//! 1 個の区切り線を CSS で見た目だけ回転させると `aria-orientation` が
//! 表示と食い違う。本 block は横向き・縦向きの区切り線をそれぞれ 1 個ずつ
//! 出力し、`data-blocks-testimonial-two-up-divider` で表示/非表示を
//! 切り替える（`display: none` で非表示側は支援技術からも除外される
//! ため、表示中の向きと aria が常に一致する。`contact_split_form_info`
//! の `display: none` 切替と同じ手段）。
//!
//! # CSS フックに data 属性を使う理由
//!
//! `blockquote::root`/`avatar::root`/`separator::separator`/`icon::icon`/
//! `heading::heading`/`text::text` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去するため、Demo 固有のスタイル
//! フックは `data-blocks-testimonial-two-up-*` 属性で渡す（既存 Blocks
//! 全件と同じ判断）。素の `div`/`p` には `class` をそのまま使う。
//!
//! # 著者行の下端揃え
//!
//! 引用文の長さが列ごとに異なっても著者行が下端で揃うよう、列内の
//! blockquote を `flex: 1; display: flex; flex-direction: column;` にし、
//! caption（著者行）に `margin-top: auto` を与える（[`LAYOUT_CSS`] 参照）。
//!
//! # ロゴは架空の幾何形状
//!
//! [`logo_icon`] は列ごとに異なる抽象図形（六角形・菱形）を出力し、実在の
//! 企業ロゴを模さない（`testimonial_background_image` と同じ判断）。
//!
//! # レスポンシブ（`48rem` のリテラルを直書きする理由）
//!
//! 他の Blocks と同じく、`pre-styled-ui` の breakpoint トークンは
//! `SlotRecipe` 経由の変数生成専用であり、docs-site の生 CSS へ直接参照
//! する経路を持たないため、リテラル値を直書きする（意図的な差分、既存
//! Blocks 全件と同じ判断）。
//!
//! # `<form>` を持たない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。静的表示のみで送信処理・データ取得は一切持たない。文言は
//! すべて `crate::blocks::dummy_assets` の架空データであり、実企業名・
//! 実サービス名・実クレデンシャル・PII を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, p, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::ColorPalette;
use fandhe_frontend_pre_styled_ui::Orientation;
use fandhe_frontend_pre_styled_ui::Size;

const LOGO_ATTR: &str = "data-blocks-testimonial-two-up-logo";
const QUOTE_ATTR: &str = "data-blocks-testimonial-two-up-quote";
const AVATAR_ATTR: &str = "data-blocks-testimonial-two-up-avatar";
const DIVIDER_ATTR: &str = "data-blocks-testimonial-two-up-divider";

/// 抽象図形ロゴ（列ごとに異なる幾何形状。実在の企業ロゴを模さない）。
fn logo_icon(company: &str, path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Xl,
            label: Some(company),
            ..IconProps::default()
        },
        vec![(LOGO_ATTR, "")],
        vec![el("path", vec![("d", path_d)], vec![])],
    )
}

/// 著者行（アバター + 氏名 + 役職）。
fn author(index: usize) -> Node {
    div(
        vec![("class", "blocks-testimonial-two-up-author")],
        vec![
            avatar::root(
                &AvatarProps::default(),
                vec![(AVATAR_ATTR, "")],
                vec![avatar::image(
                    ImageStatus::Loaded,
                    dummy_assets::AVATAR_SRC,
                    "",
                    vec![],
                )],
            ),
            div(
                vec![],
                vec![
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Medium,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(dummy_assets::PERSON_NAMES[index])],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(
                            dummy_assets::JOB_TITLES[index % dummy_assets::JOB_TITLES.len()],
                        )],
                    ),
                ],
            ),
        ],
    )
}

/// 列 1 個（ロゴ → 引用文 → 著者行）。
fn column(index: usize, company: &str, logo_path_d: &'static str) -> Node {
    div(
        vec![("class", "blocks-testimonial-two-up-column")],
        vec![
            logo_icon(company, logo_path_d),
            blockquote::root(
                BlockquoteVariant::Plain,
                ColorPalette::default(),
                vec![(QUOTE_ATTR, "")],
                vec![
                    blockquote::content(
                        vec![],
                        vec![text(dummy_assets::TESTIMONIAL_QUOTES[index])],
                    ),
                    blockquote::caption(vec![], vec![author(index)]),
                ],
            ),
        ],
    )
}

/// 2 件並びの本体（列 2 個 + 区切り線 2 個）。
fn pair(first: usize, second: usize) -> Node {
    div(
        vec![("class", "blocks-testimonial-two-up-root")],
        vec![
            column(
                first,
                dummy_assets::COMPANY_NAMES[first % dummy_assets::COMPANY_NAMES.len()],
                "M12 2L21 7V17L12 22L3 17V7Z",
            ),
            separator::separator(
                &SeparatorProps {
                    orientation: Orientation::Horizontal,
                    ..SeparatorProps::default()
                },
                vec![(DIVIDER_ATTR, "horizontal")],
            ),
            separator::separator(
                &SeparatorProps {
                    orientation: Orientation::Vertical,
                    ..SeparatorProps::default()
                },
                vec![(DIVIDER_ATTR, "vertical")],
            ),
            column(
                second,
                dummy_assets::COMPANY_NAMES[second % dummy_assets::COMPANY_NAMES.len()],
                "M12 2L22 12L12 22L2 12Z",
            ),
        ],
    )
}

/// caption（並記された各形の見出し）。
fn caption(label: &str) -> Node {
    p(
        vec![("class", "blocks-testimonial-two-up-caption")],
        vec![text(label)],
    )
}

/// 見出しなし・2 件並び（主参照 R1363 / 集約元 R0725）。
fn instance_plain() -> Node {
    pair(0, 1)
}

/// 見出し付き・2 件並び（集約元 R0728）。
fn instance_with_heading() -> Node {
    div(
        vec![],
        vec![
            div(
                vec![("class", "blocks-testimonial-two-up-header")],
                vec![
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![],
                        vec![text("利用者の声")],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("実際に使い続けているお客様からいただいた感想です。")],
                    ),
                ],
            ),
            pair(2, 3),
        ],
    )
}

/// `testimonial-two-up` の Demo 本体（2 形併記）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-testimonial-two-up-stack")],
        vec![
            caption("見出しなし・2 件並び"),
            instance_plain(),
            caption("見出し付き・2 件並び"),
            instance_with_heading(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/testimonial-two-up/",
    title: "testimonial-two-up",
    category: BlockCategory::Testimonial,
    rust_source: "crates/docs-site/src/blocks/marketing/testimonial/testimonial_two_up.rs",
    demo_class: "blocks-testimonial-two-up",
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
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Blockquote",
            path: "/themes/blockquote/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `testimonial_two_up` 固有のレイアウト規則（`--fandhe-*` トークンのみ
/// 使用）。既定（狭幅）は縦積み + 横区切り線、`48rem` 以上で 2 列 + 縦
/// 区切り線へ切り替える。列内 blockquote は `flex: 1` にし著者行へ
/// `margin-top: auto` を与えることで、引用文の長さが異なっても著者行の
/// 下端を揃える（モジュール冒頭「著者行の下端揃え」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-testimonial-two-up-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-testimonial-two-up-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-testimonial-two-up-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  margin-bottom: var(--fandhe-space-6);\n}\n\
.blocks-testimonial-two-up-root {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-testimonial-two-up-column {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-testimonial-two-up-column [data-scope=\"blockquote\"][data-part=\"root\"][data-blocks-testimonial-two-up-quote] {\n  flex: 1;\n  display: flex;\n  flex-direction: column;\n  border-inline-start: none;\n  padding-inline-start: 0;\n  margin: 0;\n}\n\
.blocks-testimonial-two-up-column [data-scope=\"blockquote\"][data-part=\"root\"][data-blocks-testimonial-two-up-quote] [data-part=\"content\"] {\n  font-size: var(--fandhe-font-font-size-lg);\n}\n\
.blocks-testimonial-two-up-column [data-scope=\"blockquote\"][data-part=\"root\"][data-blocks-testimonial-two-up-quote] [data-part=\"caption\"] {\n  margin-top: auto;\n}\n\
.blocks-testimonial-two-up-author {\n  display: flex;\n  flex-direction: row;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-scope=\"separator\"][data-blocks-testimonial-two-up-divider=\"vertical\"] {\n  display: none;\n}\n\
@media (min-width: 48rem) {\n  \
.blocks-testimonial-two-up-root {\n    grid-template-columns: 1fr auto 1fr;\n    align-items: stretch;\n  }\n  \
[data-scope=\"separator\"][data-blocks-testimonial-two-up-divider=\"horizontal\"] {\n    display: none;\n  }\n  \
[data-scope=\"separator\"][data-blocks-testimonial-two-up-divider=\"vertical\"] {\n    display: block;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// [`demo`] が 6 部品を正しい属性・向き・文言で出力すること
    /// （`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"icon\"",
            "data-scope=\"blockquote\"",
            "data-scope=\"avatar\"",
            "data-scope=\"separator\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains(r#"aria-orientation="vertical""#));
        assert!(html.contains(r#"aria-orientation="horizontal""#));
        assert!(html.contains("<blockquote"));
        assert!(html.contains("<figcaption"));
        assert!(html.contains("<h3"));
        assert!(html.contains("利用者の声"));
        for text_fragment in [
            dummy_assets::TESTIMONIAL_QUOTES[0],
            dummy_assets::TESTIMONIAL_QUOTES[1],
            dummy_assets::TESTIMONIAL_QUOTES[2],
            dummy_assets::TESTIMONIAL_QUOTES[3],
            dummy_assets::PERSON_NAMES[0],
            dummy_assets::PERSON_NAMES[1],
            dummy_assets::PERSON_NAMES[2],
            dummy_assets::PERSON_NAMES[3],
        ] {
            assert!(
                html.contains(text_fragment),
                "demo should contain {text_fragment}"
            );
        }
        assert_eq!(html.matches("blocks-testimonial-two-up-caption").count(), 2);
    }

    /// [`demo`] が `<form>`・不正リンク・`data:` URI・`<script`・`id` を
    /// 一切含まないこと（`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in [
            "<form",
            "type=\"submit\"",
            "href=\"#\"",
            "src=\"data:",
            "<script",
            "id=\"",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が全セレクタ・`48rem` ブレークポイント・下端揃え・
    /// 区切り線切替・トークンを持ち、色リテラルを含まないこと。
    #[test]
    fn layout_css_uses_tokens_and_breakpoint() {
        for selector in [
            ".blocks-testimonial-two-up-stack {",
            ".blocks-testimonial-two-up-caption {",
            ".blocks-testimonial-two-up-header {",
            ".blocks-testimonial-two-up-root {",
            ".blocks-testimonial-two-up-column {",
            ".blocks-testimonial-two-up-author {",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: 1fr auto 1fr;"));
        assert!(LAYOUT_CSS.contains("margin-top: auto;"));
        assert!(LAYOUT_CSS.contains("data-blocks-testimonial-two-up-divider=\"vertical\""));
        assert!(LAYOUT_CSS.contains("data-blocks-testimonial-two-up-divider=\"horizontal\""));
        assert!(LAYOUT_CSS.contains("var(--fandhe-"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(!LAYOUT_CSS.contains("black"));
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// ルート class が [`demo`] の出力へ実際に現れ、かつ `BLOCK.demo_class`
    /// とは異なること（既存 Blocks と同じ教訓: 同一名だと `.blocks-demo`
    /// 側の共通ラッパクラスと衝突する）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("blocks-testimonial-two-up-root"));
        assert_ne!("blocks-testimonial-two-up-root", BLOCK.demo_class);
    }
}
