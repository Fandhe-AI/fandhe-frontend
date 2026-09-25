//! `gallery-masonry` block（イシュー #2779。親トラッキング #2730「Blocks
//! 目的別パーツ拡充ツリー」配下、`crate::blocks::marketing::gallery`
//! カテゴリ最初の block）。
//!
//! # 使用部品
//!
//! `badge`（見出しラベル）/ `heading`（タイトル）/ `text`（説明文）/
//! `image`（ギャラリー画像 9 枚）の 4 部品を合成する（[`BLOCK`] の `parts`
//! に一致させる契約）。
//!
//! # masonry 風段組みの実装方式（JS 不使用）
//!
//! docs サイトは無 JS 前提（`crate::blocks` モジュール doc・
//! `tests/no_js_contract.rs`）のため、レイアウトライブラリではなく CSS の
//! `column-count`（multi-column）+ `break-inside: avoid` のみで段組みを
//! 表現する。既定（`< 40rem` = `sm` 未満）は 1 段、`40rem` 以上で 2 段、
//! `64rem`（`lg`）以上で 3 段へ広がるモバイルファースト設計（[`LAYOUT_CSS`]
//! 参照）。比率の異なる画像は [`fandhe_frontend_pre_styled_ui::image::AspectRatio`]
//! の variant を 9 枚へ循環的に割り当てることで表現し、独自の画像比率計算
//! ロジックは持たない。
//!
//! # 画像素材
//!
//! `crate::blocks::dummy_assets::PRODUCT_SRC`（ビルド時生成のモノトーン
//! プレースホルダー SVG）を 9 枚とも共用する。`data:` URI は `is_safe_url`
//! （REQ-1）が拒否するため使わない（イシュー #1562 の教訓、他 block と同方針）。
//! 9 枚とも同一素材で実際の情景を表さないため、`alt=""`（装飾扱い）で
//! 出力する（`content_image_tiles` と同方針。PR #3217 レビュー指摘: 異なる
//! 情景を語る `alt` を共有プレースホルダーへ付けると内容と食い違う）。
//!
//! # 画像を列幅いっぱいに広げる（`width: 100%`）
//!
//! `fandhe_frontend_pre_styled_ui::image` の base CSS は `max-width: 100%`
//! （縮小方向のみ）であり `width: 100%` は持たないため、`column-count`
//! による段組みでは画像が原寸のまま配置され可変高さタイルにならない
//! （PR #3217 レビュー指摘）。`content_image_tiles` と同型の
//! `data-blocks-gallery-masonry-image` マーカー属性を画像へ付与し、
//! [`LAYOUT_CSS`] 側で `width: 100%` を上書きする（image 部品自体の
//! 共有 CSS は変更しない）。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。文言・`alt` はすべて架空の一般名詞的な情景描写であり、実
//! 企業名・実人物名・実クレデンシャル・PII を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};

const INTRO_CLASS: &str = "blocks-gallery-masonry-intro";
const GRID_CLASS: &str = "blocks-gallery-masonry-grid";
const ITEM_CLASS: &str = "blocks-gallery-masonry-item";

/// 循環的に割り当てる [`AspectRatio`] variant（9 枚分）。比率の違いを
/// 画像部品側の指定のみで表現する（モジュール doc「masonry 風段組みの
/// 実装方式」節）。9 枚とも同一プレースホルダー画像のため `alt` は持たず
/// `alt=""` で出力する（モジュール doc「画像素材」節）。
const ASPECT_RATIOS: [AspectRatio; 9] = [
    AspectRatio::Portrait,
    AspectRatio::Landscape,
    AspectRatio::Square,
    AspectRatio::Video,
    AspectRatio::Portrait,
    AspectRatio::Landscape,
    AspectRatio::Square,
    AspectRatio::Video,
    AspectRatio::Portrait,
];

pub fn demo() -> Node {
    let intro = div(
        vec![("class", INTRO_CLASS)],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text("Gallery")]),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("最新の一枚")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    ..TextProps::default()
                },
                vec![],
                vec![text("幅に応じて 1〜3 段に流し込む段組みギャラリーです。")],
            ),
        ],
    );

    let items: Vec<Node> = ASPECT_RATIOS
        .iter()
        .map(|aspect_ratio| {
            div(
                vec![("class", ITEM_CLASS)],
                vec![image::image(
                    &ImageProps {
                        aspect_ratio: *aspect_ratio,
                        ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                    },
                    vec![("data-blocks-gallery-masonry-image", "")],
                )],
            )
        })
        .collect();

    let grid = div(vec![("class", GRID_CLASS)], items);

    div(vec![], vec![intro, grid])
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/gallery-masonry/",
    title: "gallery-masonry",
    category: BlockCategory::Gallery,
    rust_source: "crates/docs-site/src/blocks/marketing/gallery/gallery_masonry.rs",
    demo_class: "blocks-gallery-masonry",
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
            label: "Image",
            path: "/themes/image/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `gallery_masonry` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節、他 block と同型で `pub(super)` ではなく
/// 本ファイル内 `const` として [`super::blocks`] から `BLOCK.layout_css`
/// 経由で連結される）。モバイルファーストの `column-count` 拡張のため
/// `min-width` の `@media` を使う（他 block の「大きい既定から縮退」パターン
/// とは逆順、モジュール doc「masonry 風段組みの実装方式」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-gallery-masonry-intro {\n  margin: 0 0 1.5rem;\n}\n\
.blocks-gallery-masonry-intro > * + * {\n  margin-top: 0.5rem;\n}\n\
.blocks-gallery-masonry-grid {\n  column-count: 1;\n  column-gap: 1rem;\n}\n\
.blocks-gallery-masonry-item {\n  break-inside: avoid;\n  margin-bottom: 1rem;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-gallery-masonry-image] {\n  display: block;\n  width: 100%;\n}\n\
@media (min-width: 40rem) {\n  .blocks-gallery-masonry-grid { column-count: 2; }\n}\n\
@media (min-width: 64rem) {\n  .blocks-gallery-masonry-grid { column-count: 3; }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// [`demo`] が 9 枚の画像（すべて `alt=""` の装飾扱い）・イントロの
    /// badge/heading/text をすべて出力し、`<form>`・`href="#"`・
    /// `data:` URI・`<script` のいずれも含まないこと（`crate::blocks`
    /// モジュール doc の不変条件）。
    #[test]
    fn demo_renders_nine_images_and_avoids_disallowed_patterns() {
        let html = render(&demo());
        assert_eq!(
            html.matches(r#"src="../../assets/blocks-demo-product.svg""#)
                .count(),
            9,
            "demo should render 9 images sharing the dummy product asset"
        );
        assert_eq!(
            html.matches(r#"alt="""#).count(),
            9,
            "all 9 images share one placeholder asset and must be decorative (alt=\"\")"
        );
        assert!(html.contains("Gallery"));
        assert!(html.contains("最新の一枚"));
        for absent in ["<form", "href=\"#\"", "src=\"data:", "<script"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`demo`] が [`AspectRatio`] variant を循環的に割り当て、少なくとも
    /// 正方形以外の比率（`4 / 3` の landscape）を含むこと（比率の違いを
    /// 画像部品側の variant だけで表現するモジュール doc の契約）。
    #[test]
    fn demo_assigns_varying_aspect_ratios() {
        let html = render(&demo());
        assert!(html.contains("aspect-ratio"));
        assert!(
            ASPECT_RATIOS.contains(&AspectRatio::Landscape)
                && ASPECT_RATIOS.contains(&AspectRatio::Portrait)
                && ASPECT_RATIOS.contains(&AspectRatio::Square)
                && ASPECT_RATIOS.contains(&AspectRatio::Video),
            "ASPECT_RATIOS should cover multiple AspectRatio variants"
        );
    }

    /// [`LAYOUT_CSS`] が `column-count` 宣言と `sm`/`lg` 相当の `min-width`
    /// ブレークポイントを持つこと（モバイルファーストの段組み拡張）。
    #[test]
    fn layout_css_declares_column_count_and_breakpoints() {
        assert!(LAYOUT_CSS.contains(".blocks-gallery-masonry-grid {"));
        assert!(LAYOUT_CSS.contains("column-count: 1;"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("break-inside: avoid;"));
        assert!(
            LAYOUT_CSS.contains(
                "[data-scope=\"image\"][data-part=\"root\"][data-blocks-gallery-masonry-image] {\n  display: block;\n  width: 100%;\n}"
            ),
            "gallery images must be widened to fill their column (PR #3217 review)"
        );
    }
}
