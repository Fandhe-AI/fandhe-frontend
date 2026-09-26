//! `stats-background-image` block（イシュー #2801。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー、phase:1」配下、`crate::blocks::
//! marketing::stats` カテゴリ最初の block）。
//!
//! # 出典に関する注記
//!
//! 参照元は構造（背景画像の上に暗幕を重ね、その面へタグライン・見出し・
//! 説明文・数値指標を明るい文字で置く基準形）のみを参照し、Rust/CSS で
//! 独自に再実装する（対応表 ID R0706）。薄い背景画像の変種（対応表 ID
//! R1301）は Demo に別インスタンスとして持ち込まず、`site/blocks/
//! stats-background-image.md` の「原案差分メモ」節で暗幕の濃さ
//! （[`LAYOUT_CSS`] の `color-mix` 比率）を変えれば再現できる旨を説明する
//! に留める。
//!
//! # 使用部品
//!
//! `badge`（タグライン）/ `heading`（見出し）/ `text`（説明文）/ `stat`
//! （数値指標 4 件）/ `image`（背景画像）の 5 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge`/`heading::heading`/`text::text`/`image::image`/
//! `stat::root`/`stat::label` はいずれも `drop_class_attr` により呼び出し
//! 側 `attrs` の `class` を黙って除去する契約を持つため、本 block 固有の
//! フックは `data-blocks-stats-background-image-*` の `data-*` 属性で渡す
//! （`crate::blocks` モジュール doc・`error_page_background_image` と同じ
//! 判断軸）。素の `div` は `class` をそのまま透過するため、それらのみ
//! `class` でフックする。
//!
//! # 背景画像は共通ダミー素材
//!
//! `crate::blocks::dummy_assets::BACKGROUND_SRC`（ビルド時生成のモノトーン
//! 背景タイル SVG）を使う。`data:` URI は `is_safe_url`（REQ-1）が拒否する
//! ため使わない（イシュー #1562 の教訓）。
//!
//! # 暗幕と文字色の設計判断（常に明るい前景トークンが無いことへの対応）
//!
//! 既定テーマには「常にライトに近い」前景トークンが無い（`fg` はライトで
//! ほぼ黒、ダークでほぼ白）。このため本 block は tooltip の反転面
//! （`crates/pre-styled-ui/src/tooltip.rs` の `background: var(--fandhe-
//! color-fg); color: var(--fandhe-color-bg)`）と同じ反転ペアを採用する:
//! 暗幕を `--fandhe-color-fg` ベース、文字を `--fandhe-color-bg` ベースに
//! する。ライトテーマでは暗い暗幕・明るい文字（要件どおり）、ダークテーマ
//! では明暗が入れ替わるが、テーマが保証する fg/bg のコントラストは両モード
//! で保たれる。色リテラル（`#…`/`white`/`black`）は使わず、`--fandhe-*`
//! トークンと `color-mix()` のみで表現する（参照元の白文字直書きを踏襲
//! しない）。
//!
//! # 背景画像フックの詳細度（`[data-scope="image"][data-part="root"]` を前置する理由）
//!
//! `image::image` の recipe は base 宣言
//! `[data-scope="image"][data-part="root"] { height: auto; ... }`
//! （詳細度 0,2,0）を持つため、本 block 固有フック単独属性セレクタ
//! （詳細度 0,1,0）のまま `height: 100%` を宣言しても base 側が勝つ
//! （`error_page_background_image` と同じ教訓、PR #3207 レビュー指摘）。
//! 是正として base と同じ 2 属性セレクタへ前置し、詳細度を base と同値に
//! したうえでソース順（`blocks.css` は `pre-styled-ui.css` より後に読み
//! 込まれる）で後勝ちさせる。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。静的表示のみで送信処理・データ取得は一切持たない。文言は
//! すべて架空のものであり、実企業名・実サービス名・実クレデンシャル・
//! PII を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps};
use fandhe_frontend_pre_styled_ui::Size;

const ROOT_CLASS: &str = "blocks-stats-background-image-root";
const BACKDROP_CLASS: &str = "blocks-stats-background-image-backdrop";
const SCRIM_CLASS: &str = "blocks-stats-background-image-scrim";
const CONTENT_CLASS: &str = "blocks-stats-background-image-content";
const GRID_CLASS: &str = "blocks-stats-background-image-grid";

const IMAGE_ATTR: &str = "data-blocks-stats-background-image-image";
const TAGLINE_ATTR: &str = "data-blocks-stats-background-image-tagline";
const TITLE_ATTR: &str = "data-blocks-stats-background-image-title";
const DESCRIPTION_ATTR: &str = "data-blocks-stats-background-image-description";
const STAT_ATTR: &str = "data-blocks-stats-background-image-stat";
const STAT_LABEL_ATTR: &str = "data-blocks-stats-background-image-stat-label";

/// 数値指標 1 件分（`stat::root` + `label`/`value_text`、`content_image_tiles`
/// と同じ合成方法）。
fn stat_item(label: &str, value: &str) -> Node {
    stat::root(
        Size::Lg,
        vec![(STAT_ATTR, "")],
        vec![
            stat::label(vec![(STAT_LABEL_ATTR, "")], vec![text(label)]),
            stat::value_text(vec![], vec![text(value)]),
        ],
    )
}

/// `stats-background-image` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    let backdrop = div(
        vec![("class", BACKDROP_CLASS), ("aria-hidden", "true")],
        vec![
            image::image(
                &ImageProps::new(dummy_assets::BACKGROUND_SRC, ""),
                vec![(IMAGE_ATTR, "")],
            ),
            div(vec![("class", SCRIM_CLASS)], vec![]),
        ],
    );

    let tagline = badge::badge(
        &BadgeProps::default(),
        vec![(TAGLINE_ATTR, "")],
        vec![text("Trusted by teams everywhere")],
    );

    let title = heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl3,
            ..HeadingProps::default()
        },
        vec![(TITLE_ATTR, "")],
        vec![text("Built for teams that never stop shipping")],
    );

    let description = styled_text::text(
        &TextProps::default(),
        vec![(DESCRIPTION_ATTR, "")],
        vec![text(
            "A snapshot of how teams rely on our platform every day.",
        )],
    );

    let grid = div(
        vec![("class", GRID_CLASS)],
        vec![
            stat_item("Active projects", "12k+"),
            stat_item("Uptime", "99.9%"),
            stat_item("Countries", "40"),
            stat_item("Avg. response", "2h"),
        ],
    );

    let content = div(
        vec![("class", CONTENT_CLASS)],
        vec![tagline, title, description, grid],
    );

    div(vec![("class", ROOT_CLASS)], vec![backdrop, content])
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/stats-background-image/",
    title: "stats-background-image",
    category: BlockCategory::Stats,
    rust_source: "crates/docs-site/src/blocks/marketing/stats/stats_background_image.rs",
    demo_class: "blocks-stats-background-image",
    parts: &[
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
            label: "Stat",
            path: "/themes/stat/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `stats_background_image` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「CSS の置き場」節、他 block と同型で `pub(super)` ではなく本ファイル内
/// `const` として [`super::blocks`] から `BLOCK.layout_css` 経由で連結される）。
///
/// 生の色リテラル（`#fff`/`white`/`black` 等）は使わず、可読性の確保はすべて
/// `--fandhe-color-*` トークンと `color-mix()` で行う（モジュール冒頭
/// 「暗幕と文字色の設計判断」節）。
const LAYOUT_CSS: &str = "\
.blocks-stats-background-image {\n  padding: 0;\n  overflow: hidden;\n}\n\
.blocks-stats-background-image-root {\n  position: relative;\n  isolation: isolate;\n  overflow: hidden;\n  min-height: 24rem;\n  padding: var(--fandhe-space-16) var(--fandhe-space-6);\n  color: var(--fandhe-color-bg);\n}\n\
.blocks-stats-background-image-backdrop {\n  position: absolute;\n  inset: 0;\n  z-index: -1;\n  overflow: hidden;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-stats-background-image-image] {\n  width: 100%;\n  height: 100%;\n  display: block;\n}\n\
.blocks-stats-background-image-scrim {\n  position: absolute;\n  inset: 0;\n  background: color-mix(in srgb, var(--fandhe-color-fg) 80%, transparent);\n}\n\
.blocks-stats-background-image-content {\n  max-width: 48rem;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-stats-background-image-description] {\n  color: color-mix(in srgb, var(--fandhe-color-bg) 78%, transparent);\n}\n\
.blocks-stats-background-image-grid {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-6);\n}\n\
@media (min-width: 48rem) {\n  .blocks-stats-background-image-grid {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n}\n\
[data-scope=\"stat\"][data-part=\"label\"][data-blocks-stats-background-image-stat-label] {\n  color: color-mix(in srgb, var(--fandhe-color-bg) 78%, transparent);\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// [`demo`] が背景画像・暗幕・タグライン・見出し・説明文・4 件の数値
    /// 指標を正しい属性・文言で出力し、`<form>`・`href="#"`・`data:` URI・
    /// `<script` のいずれも含まないこと（`crate::blocks` モジュール doc の
    /// 不変条件）。
    #[test]
    fn demo_renders_expected_markup_and_avoids_disallowed_patterns() {
        let html = render(&demo());
        assert!(html.contains(r#"src="../../assets/blocks-demo-background.svg""#));
        assert!(html.contains(r#"alt="""#));
        assert!(html.contains(r#"aria-hidden="true""#));
        for hook in [
            IMAGE_ATTR,
            TAGLINE_ATTR,
            TITLE_ATTR,
            DESCRIPTION_ATTR,
            STAT_ATTR,
            STAT_LABEL_ATTR,
        ] {
            assert!(
                html.contains(hook),
                "demo should render the {hook} attribute"
            );
        }
        assert_eq!(
            html.matches("data-scope=\"stat\" data-part=\"root\"")
                .count(),
            4,
            "demo should render exactly 4 stat items"
        );
        for text_fragment in [
            "Trusted by teams everywhere",
            "Built for teams that never stop shipping",
            "Active projects",
            "12k+",
            "Uptime",
            "99.9%",
            "Countries",
            "40",
            "Avg. response",
            "2h",
        ] {
            assert!(
                html.contains(text_fragment),
                "demo should contain {text_fragment}"
            );
        }
        for absent in ["<form", "href=\"#\"", "src=\"data:", "<script"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が全セレクタを宣言し、色リテラルではなく `color-mix()` +
    /// トークン参照で可読性を確保していること・`repeat(4` を伴う md
    /// ブレークポイントを持つこと（モジュール冒頭「暗幕と文字色の設計判断」節）。
    #[test]
    fn layout_css_declares_all_selectors_and_uses_color_tokens_not_literals() {
        for selector in [
            ".blocks-stats-background-image {",
            ".blocks-stats-background-image-root {",
            ".blocks-stats-background-image-backdrop {",
            "[data-scope=\"image\"][data-part=\"root\"][data-blocks-stats-background-image-image] {",
            ".blocks-stats-background-image-scrim {",
            ".blocks-stats-background-image-content {",
            "[data-blocks-stats-background-image-description] {",
            ".blocks-stats-background-image-grid {",
            "[data-scope=\"stat\"][data-part=\"label\"][data-blocks-stats-background-image-stat-label] {",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("repeat(4"));
        assert!(LAYOUT_CSS.contains("color-mix("));
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-fg)"));
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-bg)"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(!LAYOUT_CSS.contains("black"));
    }
}
