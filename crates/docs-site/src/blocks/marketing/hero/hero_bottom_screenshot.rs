//! `hero-bottom-screenshot` block（イシュー #2782。親トラッキング #2738
//! 「Phase 1 マーケティング A」配下、Marketing / Hero カテゴリの 5 件目。
//! 上段に見出し・リード文・CTA、下段に横長スクリーンショットを全幅で置く
//! 1 列ヒーローの合成例（対応表 ID 基準 R1007、集約元 R0126/R0127/R0534/
//! R0540/R0545/R0552/R0554/R1008。出典の固有名・ファイル名は記載しない）。
//!
//! # 使用部品
//!
//! `badge`（eyebrow）/ `heading`（見出し）/ `text`（リード文・キャプション）
//! / `button`（CTA）/ `image`（スクリーンショット・ロゴ）の 5 部品を合成
//! する（[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # 6 セクションを 1 つの Demo に並べる
//!
//! イシューが求める見せ方の差分（枠付き基準形・上辺のみ角丸・左寄せ 2 列・
//! 動画プレースホルダー・2 枚グリッド・淡色帯 + ロゴ列）を別々の block へ
//! 分けず、`feature-large-screenshot` と同じ方針で 1 Demo に縦並記する。
//!
//! - **bordered（基準形、R1007 基準・R0126 集約）**: 中央寄せの
//!   eyebrow badge + 見出し + リード文 + CTA 2 個 → 16:9 の枠付き画像。
//! - **top-rounded（R0126/R1008 集約）**: 中央寄せ（badge なし）→ 枠なし・
//!   上辺のみ角丸の画像。
//! - **left-split（R0127/R0552 集約）**: 左寄せ。`lg`（64rem）以上で見出し
//!   と説明+CTA を左右 2 列に分ける → 枠付き画像。
//! - **video（R0554 集約）**: 中央寄せ → `<video>` は使わず、破線枠 +
//!   「Video placeholder」の 1 行のみの静的プレースホルダー。
//! - **pair-grid（R0540 集約）**: 中央寄せ → 大判（横長）+ 正方形の画像
//!   2 枚グリッド。
//! - **band-logos（R0534/R0545 集約）**: 中央寄せ → 淡色帯で囲んだ画像 +
//!   その下にロゴ 5 個 + 社名のロゴ列。
//!
//! # `<video>` を出力しない理由
//!
//! 本 Demo は無 JS の静的合成例であり再生制御を持たないため、動画そのもの
//! ではなく「動画プレースホルダー」であることを明示する `div` + テキスト
//! で表現する（`crate::blocks` モジュール doc の `<form>` 不使用と同じ
//! 「実際には動作しない機能を実物のタグで偽装しない」判断軸）。
//!
//! # 画像に 16:9（`AspectRatio::Video`）を選ぶ理由
//!
//! [`dummy_assets::SCREENSHOT_SRC`] は 200×140（比率 10:7 に近い）だが、
//! イシューが明示する「16:9 の横長画像」要件を優先し `ImageFit::Cover` +
//! `AspectRatio::Video` で trim して表示する（`feature_large_screenshot`
//! が `AspectRatio::Auto` を選んだ判断とは要件が異なるため踏襲しない）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge`/`heading::heading`/`button::button`/`image::image` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って
//! 除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-hero-bottom-screenshot-*` 属性で渡す。素の `div` には
//! `class` がそのまま効くため `.blocks-hero-bottom-screenshot-*` クラス
//! セレクタを使う。レイアウト root の class
//! （`blocks-hero-bottom-screenshot-layout`）は [`Block::demo_class`]
//! （`blocks-hero-bottom-screenshot`）とは意図的に別名にする
//! （`feature-large-screenshot` と同じ Bugbot 教訓の回避）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない・id を使わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言はすべて架空のもの（実企業名・実クレデンシャル・PII を
//! 含まない）。画像は [`dummy_assets`] のビルド時生成プレースホルダー SVG
//! のみを使い、装飾扱いの `alt=""` で出力する。ボタンは既定の
//! `type="button"` のまま送信先を持たない。`id` 属性は一切使わない
//! （重複 id 検知テスト対策）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

const SECTION_VARIANT_ATTR: &str = "data-blocks-hero-bottom-screenshot-section-variant";
const IMAGE_VARIANT_ATTR: &str = "data-blocks-hero-bottom-screenshot-image-variant";

/// 見出し 1 件分（eyebrow badge + 見出し + リード文）。`align` が `"start"`
/// のとき左寄せへ切り替える（[`LAYOUT_CSS`] 側のセレクタ参照）。
fn section_header(eyebrow: Option<&str>, title: &str, lead: &str, align: &'static str) -> Node {
    let mut children: Vec<Node> = Vec::new();
    if let Some(eyebrow) = eyebrow {
        children.push(badge::badge(
            &BadgeProps::default(),
            vec![("data-blocks-hero-bottom-screenshot-eyebrow", "")],
            vec![text(eyebrow)],
        ));
    }
    children.push(heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl3,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text(title)],
    ));
    children.push(styled_text::text(
        &TextProps {
            size: TextSize::Lg,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(lead)],
    ));
    div(
        vec![
            ("class", "blocks-hero-bottom-screenshot-header"),
            ("data-blocks-hero-bottom-screenshot-align", align),
        ],
        children,
    )
}

/// CTA ボタン 2 個（Solid + Outline）の行。
fn cta_row() -> Node {
    div(
        vec![("class", "blocks-hero-bottom-screenshot-actions")],
        vec![
            button::button(&ButtonProps::default(), vec![], vec![text("Get started")]),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("View docs")],
            ),
        ],
    )
}

/// 16:9 のスクリーンショット画像（`variant` は [`LAYOUT_CSS`] 側の枠・
/// 角丸の切り替えに使う）。装飾扱いのため `alt=""`。
fn screenshot(variant: &'static str) -> Node {
    div(
        vec![
            ("class", "blocks-hero-bottom-screenshot-media"),
            (IMAGE_VARIANT_ATTR, variant),
        ],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Video,
                shape: ImageShape::Square,
                ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
            },
            vec![("data-blocks-hero-bottom-screenshot-image", "")],
        )],
    )
}

/// 動画プレースホルダー（`<video>` は使わず、破線枠 + 1 行テキストのみの
/// 静的表示。モジュール doc「`<video>` を出力しない理由」参照）。
fn video_placeholder() -> Node {
    div(
        vec![("class", "blocks-hero-bottom-screenshot-video-placeholder")],
        vec![styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text("Video placeholder")],
        )],
    )
}

/// 2 枚グリッド（大判の横長画像 + 正方形画像）。
fn pair_grid() -> Node {
    div(
        vec![("class", "blocks-hero-bottom-screenshot-pair-grid")],
        vec![
            div(
                vec![("class", "blocks-hero-bottom-screenshot-media")],
                vec![image::image(
                    &ImageProps {
                        fit: ImageFit::Cover,
                        aspect_ratio: AspectRatio::Landscape,
                        shape: ImageShape::Square,
                        ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
                    },
                    vec![("data-blocks-hero-bottom-screenshot-image", "")],
                )],
            ),
            div(
                vec![("class", "blocks-hero-bottom-screenshot-media")],
                vec![image::image(
                    &ImageProps {
                        fit: ImageFit::Cover,
                        aspect_ratio: AspectRatio::Square,
                        shape: ImageShape::Square,
                        ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                    },
                    vec![("data-blocks-hero-bottom-screenshot-image", "")],
                )],
            ),
        ],
    )
}

/// 淡色帯で囲んだ画像 + その下のロゴ列（ロゴ 5 個 + 社名。
/// [`dummy_assets::COMPANY_NAMES`] の先頭 5 件を使う）。
fn band_with_logos() -> Node {
    let logos: Vec<Node> = dummy_assets::COMPANY_NAMES[..5]
        .iter()
        .map(|name| {
            div(
                vec![("class", "blocks-hero-bottom-screenshot-logo-item")],
                vec![
                    image::image(
                        &ImageProps {
                            fit: ImageFit::Contain,
                            aspect_ratio: AspectRatio::Square,
                            shape: ImageShape::Square,
                            ..ImageProps::new(dummy_assets::LOGO_SRC, "")
                        },
                        vec![("data-blocks-hero-bottom-screenshot-logo", "")],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(*name)],
                    ),
                ],
            )
        })
        .collect();

    div(
        vec![("class", "blocks-hero-bottom-screenshot-band")],
        vec![
            screenshot("band"),
            div(
                vec![("class", "blocks-hero-bottom-screenshot-logo-row")],
                logos,
            ),
        ],
    )
}

/// bordered（基準形）: eyebrow + 見出し + リード文 + CTA → 枠付き画像。
fn bordered_section() -> Node {
    div(
        vec![("class", "blocks-hero-bottom-screenshot-section"), (SECTION_VARIANT_ATTR, "bordered")],
        vec![
            section_header(
                Some("New release"),
                "See it in action before you start",
                "A single screenshot below shows the real product screen, framed just like it looks in the app.",
                "center",
            ),
            cta_row(),
            screenshot("bordered"),
        ],
    )
}

/// top-rounded: 見出し + リード文（badge なし）→ 枠なし・上辺のみ角丸の画像。
fn top_rounded_section() -> Node {
    div(
        vec![("class", "blocks-hero-bottom-screenshot-section"), (SECTION_VARIANT_ATTR, "top-rounded")],
        vec![
            section_header(
                None,
                "A cleaner way to preview your work",
                "The screenshot sits flush against the section below it, rounded only where it meets the header.",
                "center",
            ),
            screenshot("top-rounded"),
        ],
    )
}

/// left-split: 左寄せ。`lg` 以上で見出しと説明+CTA を 2 列に分ける
/// → 枠付き画像。
fn left_split_section() -> Node {
    let title = heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl3,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text("Built for teams that ship every day")],
    );
    let lead = styled_text::text(
        &TextProps {
            size: TextSize::Lg,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "Give every teammate the same clear view of what changed, without digging through logs.",
        )],
    );
    let right = div(
        vec![("class", "blocks-hero-bottom-screenshot-split-right")],
        vec![lead, cta_row()],
    );
    div(
        vec![
            ("class", "blocks-hero-bottom-screenshot-section"),
            (SECTION_VARIANT_ATTR, "left-split"),
        ],
        vec![
            div(
                vec![
                    ("class", "blocks-hero-bottom-screenshot-split-row"),
                    ("data-blocks-hero-bottom-screenshot-align", "start"),
                ],
                vec![title, right],
            ),
            screenshot("bordered"),
        ],
    )
}

/// video: eyebrow なし・中央寄せ見出し → 動画プレースホルダー。
fn video_section() -> Node {
    div(
        vec![
            ("class", "blocks-hero-bottom-screenshot-section"),
            (SECTION_VARIANT_ATTR, "video"),
        ],
        vec![
            section_header(
                None,
                "Watch how it feels to use",
                "A short walkthrough of the product, right where you'd expect it.",
                "center",
            ),
            video_placeholder(),
        ],
    )
}

/// pair-grid: 中央寄せ見出し → 大判 + 正方形の画像 2 枚グリッド。
fn pair_grid_section() -> Node {
    div(
        vec![
            ("class", "blocks-hero-bottom-screenshot-section"),
            (SECTION_VARIANT_ATTR, "pair-grid"),
        ],
        vec![
            section_header(
                None,
                "Two views, one workflow",
                "See the full dashboard alongside a closer look at a single card.",
                "center",
            ),
            pair_grid(),
        ],
    )
}

/// band-logos: 中央寄せ見出し → 淡色帯で囲んだ画像 + ロゴ列。
fn band_logos_section() -> Node {
    div(
        vec![
            ("class", "blocks-hero-bottom-screenshot-section"),
            (SECTION_VARIANT_ATTR, "band-logos"),
        ],
        vec![
            section_header(
                None,
                "Trusted by teams around the world",
                "Join teams already using the product every day.",
                "center",
            ),
            band_with_logos(),
        ],
    )
}

/// `hero-bottom-screenshot` の Demo 本体（6 セクション縦並記）。呼び出し
/// ごとに同一の `Node` を返す純関数。ルート class は `demo_class`
/// （`blocks-hero-bottom-screenshot`）とは別名にする。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-hero-bottom-screenshot-layout")],
        vec![
            bordered_section(),
            top_rounded_section(),
            left_split_section(),
            video_section(),
            pair_grid_section(),
            band_logos_section(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/hero-bottom-screenshot/",
    title: "hero-bottom-screenshot",
    category: BlockCategory::Hero,
    rust_source: "crates/docs-site/src/blocks/marketing/hero/hero_bottom_screenshot.rs",
    demo_class: "blocks-hero-bottom-screenshot",
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
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `hero_bottom_screenshot` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-hero-bottom-screenshot-*` と
/// `[data-blocks-hero-bottom-screenshot-*]` のみを用いる。
const LAYOUT_CSS: &str = "\
.blocks-hero-bottom-screenshot-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
.blocks-hero-bottom-screenshot-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  padding-block: var(--fandhe-space-8);\n}\n\
.blocks-hero-bottom-screenshot-section + .blocks-hero-bottom-screenshot-section {\n  border-top: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-hero-bottom-screenshot-header {\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n  max-width: 48rem;\n  margin: 0 auto;\n  text-align: center;\n}\n\
[data-blocks-hero-bottom-screenshot-align=\"start\"] {\n  align-items: flex-start;\n  text-align: left;\n  max-width: none;\n  margin: 0;\n}\n\
.blocks-hero-bottom-screenshot-actions {\n  display: flex;\n  flex-wrap: wrap;\n  justify-content: center;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-hero-bottom-screenshot-align=\"start\"] + .blocks-hero-bottom-screenshot-actions,\n\
.blocks-hero-bottom-screenshot-split-right .blocks-hero-bottom-screenshot-actions {\n  justify-content: flex-start;\n}\n\
.blocks-hero-bottom-screenshot-split-row {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n  align-items: start;\n}\n\
@media (min-width: 64rem) {\n  \
.blocks-hero-bottom-screenshot-split-row {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
}\n\
.blocks-hero-bottom-screenshot-split-right {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  align-items: flex-start;\n}\n\
.blocks-hero-bottom-screenshot-media {\n  width: 100%;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-hero-bottom-screenshot-image] {\n  display: block;\n  width: 100%;\n}\n\
[data-blocks-hero-bottom-screenshot-image-variant=\"bordered\"] [data-scope=\"image\"][data-part=\"root\"][data-blocks-hero-bottom-screenshot-image] {\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  box-shadow: var(--fandhe-shadow-lg);\n}\n\
[data-blocks-hero-bottom-screenshot-image-variant=\"top-rounded\"] [data-scope=\"image\"][data-part=\"root\"][data-blocks-hero-bottom-screenshot-image] {\n  border-radius: var(--fandhe-radius-lg) var(--fandhe-radius-lg) 0 0;\n}\n\
.blocks-hero-bottom-screenshot-video-placeholder {\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  aspect-ratio: 16 / 9;\n  width: 100%;\n  border: 1px dashed var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg-subtle);\n}\n\
.blocks-hero-bottom-screenshot-pair-grid {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n}\n\
@media (min-width: 48rem) {\n  \
.blocks-hero-bottom-screenshot-pair-grid {\n    grid-template-columns: 2fr 1fr;\n  }\n\
}\n\
.blocks-hero-bottom-screenshot-band {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  padding: var(--fandhe-space-6);\n  background: var(--fandhe-color-bg-subtle);\n  border-radius: var(--fandhe-radius-xl);\n}\n\
.blocks-hero-bottom-screenshot-logo-row {\n  display: flex;\n  flex-wrap: wrap;\n  justify-content: center;\n  align-items: center;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-hero-bottom-screenshot-logo-item {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-hero-bottom-screenshot-logo] {\n  display: block;\n  width: 2rem;\n  height: 2rem;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, dummy_assets, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"button\"",
            "data-scope=\"image\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        // 画像枚数: bordered(1) + top-rounded(1) + left-split(1) +
        // pair-grid(2) + band-logos(1 + logo 5) = 11。
        assert_eq!(html.matches("<img").count(), 11);
        // ボタン: bordered(2) + left-split(2) = 4。
        assert_eq!(html.matches("<button").count(), 4);
        for variant in [
            "bordered",
            "top-rounded",
            "left-split",
            "video",
            "pair-grid",
            "band-logos",
        ] {
            assert_eq!(
                html.matches(&format!(
                    "data-blocks-hero-bottom-screenshot-section-variant=\"{variant}\""
                ))
                .count(),
                1,
                "section variant {variant} should appear exactly once"
            );
        }
        assert!(html.contains(dummy_assets::SCREENSHOT_SRC));
        assert!(html.contains(dummy_assets::PRODUCT_SRC));
        assert!(html.contains(dummy_assets::LOGO_SRC));
        assert!(!html.contains("<form"));
        assert!(!html.contains("<video"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("id=\""));
    }

    /// [`LAYOUT_CSS`] が想定する breakpoint・16:9 プレースホルダー・
    /// 上辺のみ角丸を持つこと。
    #[test]
    fn layout_css_declares_breakpoints_and_shapes() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("aspect-ratio: 16 / 9"));
        assert!(LAYOUT_CSS.contains("var(--fandhe-radius-lg) var(--fandhe-radius-lg) 0 0"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「CSS フックの選び方」節の Bugbot 教訓の
    /// 固定、`feature-large-screenshot` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-hero-bottom-screenshot-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-hero-bottom-screenshot-layout"
        );
    }
}
