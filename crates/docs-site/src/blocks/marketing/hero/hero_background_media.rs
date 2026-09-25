//! `hero-background-media` block（イシュー #2781。`crate::blocks::marketing::
//! hero` カテゴリ 5 件目）。
//!
//! # 出典に関する注記
//!
//! 主参照は中央寄せの背景メディア hero（R0551 基準形）。「下寄せ + 見出しと
//! 本文を lg 以上で 2 列」（R0547）・「動画背景 + 下寄せ見出し」（R0133）・
//! 「動画背景の静的プレースホルダ化」（R0546）は形 B（下寄せ 2 列）へ
//! 集約する。動画背景は無 JS の docs サイトでは再生できないため、
//! いずれの形も静止画プレースホルダで代替する（対応表 ID のみを記す。
//! 取得手段・ファイル名・内部コンポーネント識別子は記載しない）。
//!
//! # 使用部品
//!
//! `badge`（eyebrow）/ `heading`（見出し）/ `text`（リード文）/ `button`
//! （CTA 2 個）/ `image`（背景画像）の 5 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約）。
//!
//! # 2 形を 1 つの Demo に並記する
//!
//! [`super::super::cta::cta_split_image`] と同型に、[`variant_label`] で
//! 見出しを付けながら形 A（中央寄せ）・形 B（下寄せ 2 列）を [`demo`] 1 つの
//! 中へ縦に並べる。
//!
//! # 背景画像は共通ダミー素材・詳細度対策
//!
//! `crate::blocks::dummy_assets::BACKGROUND_SRC`（ビルド時生成のモノトーン
//! 背景タイル SVG）を両形で使い回す（動画背景の静的プレースホルダとしても
//! 兼用する）。`image::image` の recipe が持つ base 宣言
//! `[data-scope="image"][data-part="root"] { height: auto; ... }`
//! （詳細度 0,2,0）に確実に勝つため、`[`super::super::error_page::
//! error_page_background_image`] と同じく base と同じ 2 属性セレクタへ
//! 前置した本 block 固有フックで `height: 100%` を上書きする（PR #3207 の
//! 教訓）。
//!
//! # 暗幕は `color-mix` + トークンで作る（白文字を直書きしない）
//!
//! 背景画像の上へ `--fandhe-color-fg` を `color-mix()` で半透明化した
//! スクリムを重ね、コンテンツは `--fandhe-color-bg` トークンで描く。
//! ライトテーマでは前景色（既定は暗色）の幕の上に背景色（既定は明色）の
//! 文字が乗り「暗幕 + 明るい文字」になるが、ダークテーマでは前景/背景の
//! 意味が反転するため「明るい幕 + 暗い文字」に反転する。色リテラル
//! （`#`/`white` 等）は使わず、可読性の確保は完全にトークンへ委ねる
//! 設計上の既知の挙動であり、原稿の差分メモにも明記する。
//!
//! # secondary CTA の反転色も詳細度対策が要る
//!
//! `[data-blocks-hero-background-media-cta-secondary]` 単体（属性セレクタ
//! 1 個、詳細度 0,1,0）では `button::button` の Outline variant セレクタ
//! `[data-scope="button"][data-part="root"].fd-button--variant-outline`
//! （属性 2 個 + クラス 1 個、詳細度 0,3,0）に確実に負け、反転スクリム上で
//! secondary ボタンが `--fandhe-palette`（既定 accent）のまま残ってしまう。
//! 上記「背景画像は共通ダミー素材」節と同じ理由・同じ解法（base と同じ
//! セレクタへ前置する）で、こちらは `[data-scope="button"][data-part=
//! "root"]` を前置して詳細度 0,3,0 に揃える（同値のため `!important` は
//! 使わない。CSS 出力順は showcase の `pre-styled-ui.css` → blocks の
//! `assets/blocks.css` の順で `<link>` するため（`crate::build` 参照）、
//! 同値セレクタは後勝ちで確実に本 block 側が勝つ。同型の判断は
//! `fandhe_frontend_pre_styled_ui::marquee` の「0,2,0 でソース順末尾に出る
//! ため `!important` なしで確実に後勝ちする」注記と同じ）。
//!
//! # bottom-split の `split-side` にも gap が要る
//!
//! `text::text` の recipe は既定で `margin: 0` を持つため、`.blocks-hero-
//! background-media-split-side`（lead 文 + CTA 群を包む div）に gap を
//! 与えないと両者が密着する。centered バリアントの
//! `.blocks-hero-background-media-content` は既に `gap: 1rem` を持つため、
//! 対称性のため `.blocks-hero-background-media-split-side` にも同じ
//! `display: grid; gap: 1rem;` を与える。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge`/`heading::heading`/`text::text`/`button::button`/
//! `image::image` はいずれも `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、本 block 固有のフックは
//! `data-blocks-hero-background-media-*` の `data-*` 属性で渡す
//! （`crate::blocks` モジュール doc と同じ判断軸）。素の `div` には
//! `class` がそのまま効くため、レイアウト・背景レイヤーは
//! `.blocks-hero-background-media-*` クラスセレクタで扱う。
//!
//! # アニメーションを持たない
//!
//! 要件どおり、背景・暗幕・コンテンツのいずれも `animation`/`transition`
//! を持たない静的な合成である。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。CTA ボタンは `button::button` の既定 `type="button"` の
//! まま用いる。文言はすべて架空のものであり、実企業名・実サービス名・
//! 実クレデンシャル・PII を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

const IMAGE_ATTR: &str = "data-blocks-hero-background-media-image";
const EYEBROW_ATTR: &str = "data-blocks-hero-background-media-eyebrow";
const TITLE_ATTR: &str = "data-blocks-hero-background-media-title";
const LEAD_ATTR: &str = "data-blocks-hero-background-media-lead";
const CTA_PRIMARY_ATTR: &str = "data-blocks-hero-background-media-cta-primary";
const CTA_SECONDARY_ATTR: &str = "data-blocks-hero-background-media-cta-secondary";
const VARIANT_ATTR: &str = "data-blocks-hero-background-media-variant";

/// 各形の直前に置く短い形ラベル（`cta_split_image::variant_label` と同型）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// 背景画像 + 暗幕の 2 層（`aria-hidden` で装飾扱い、両形で共通）。
fn backdrop() -> Node {
    div(
        vec![
            ("class", "blocks-hero-background-media-backdrop"),
            ("aria-hidden", "true"),
        ],
        vec![
            image::image(
                &ImageProps::new(dummy_assets::BACKGROUND_SRC, ""),
                vec![(IMAGE_ATTR, "")],
            ),
            div(
                vec![("class", "blocks-hero-background-media-scrim")],
                vec![],
            ),
        ],
    )
}

/// 形 A（R0551 基準形）: 中央寄せの eyebrow badge + 見出し + リード文 + CTA。
fn variant_centered() -> Node {
    let content = div(
        vec![("class", "blocks-hero-background-media-content")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![(EYEBROW_ATTR, "")],
                vec![text("Now in preview")],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl4,
                    ..HeadingProps::default()
                },
                vec![(TITLE_ATTR, "")],
                vec![text("Build the page, keep the platform")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    ..TextProps::default()
                },
                vec![(LEAD_ATTR, "")],
                vec![text(
                    "Compose SSR, CSR and static output from one plain-HTML core.",
                )],
            ),
            div(
                vec![("class", "blocks-hero-background-media-actions")],
                vec![
                    button::button(
                        &ButtonProps::default(),
                        vec![(CTA_PRIMARY_ATTR, "")],
                        vec![text("Get started")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![(CTA_SECONDARY_ATTR, "")],
                        vec![text("See the demo")],
                    ),
                ],
            ),
        ],
    );
    div(
        vec![
            ("class", "blocks-hero-background-media-root"),
            (VARIANT_ATTR, "centered"),
        ],
        vec![backdrop(), content],
    )
}

/// 形 B（R0547 + R0133/R0546 集約）: 下寄せ、lg 以上で見出し/本文+CTA の
/// 2 列。動画背景は本 block と同じ静止画プレースホルダで代替する。
fn variant_bottom_split() -> Node {
    let content = div(
        vec![("class", "blocks-hero-background-media-content")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![(TITLE_ATTR, "")],
                vec![text("A platform that fades into the background")],
            ),
            div(
                vec![("class", "blocks-hero-background-media-split-side")],
                vec![
                    styled_text::text(
                        &TextProps::default(),
                        vec![(LEAD_ATTR, "")],
                        vec![text(
                            "Ship an interactive experience without shipping a framework.",
                        )],
                    ),
                    div(
                        vec![("class", "blocks-hero-background-media-actions")],
                        vec![
                            button::button(
                                &ButtonProps::default(),
                                vec![(CTA_PRIMARY_ATTR, "")],
                                vec![text("Get started")],
                            ),
                            button::button(
                                &ButtonProps {
                                    variant: ButtonVariant::Outline,
                                    ..ButtonProps::default()
                                },
                                vec![(CTA_SECONDARY_ATTR, "")],
                                vec![text("See the demo")],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    );
    div(
        vec![
            ("class", "blocks-hero-background-media-root"),
            (VARIANT_ATTR, "bottom-split"),
        ],
        vec![backdrop(), content],
    )
}

pub fn demo() -> Node {
    div(
        vec![("class", "blocks-hero-background-media-layout")],
        vec![
            variant_label("中央寄せ（R0551 基準形）"),
            variant_centered(),
            variant_label(
                "下寄せ + lg 以上で 2 列（R0547 + R0133/R0546 集約、動画背景は静止画で代替）",
            ),
            variant_bottom_split(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/hero-background-media/",
    title: "hero-background-media",
    category: BlockCategory::Hero,
    rust_source: "crates/docs-site/src/blocks/marketing/hero/hero_background_media.rs",
    demo_class: "blocks-hero-background-media",
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

/// `hero_background_media` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節、他 block と同型）。
///
/// 生の色リテラル（`#fff`/`white` 等）は使わず、可読性の確保はすべて
/// `--fandhe-color-*` トークンと `color-mix()` で行う（モジュール冒頭
/// 「暗幕は `color-mix` + トークンで作る」節）。ブレークポイントは
/// [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]（1024px =
/// 64rem）と一致するリテラル値を直書きする（CSS custom property は
/// `@media` 条件式の中では解決できないため、既存 block と同じ判断）。
const LAYOUT_CSS: &str = "\
.blocks-hero-background-media-layout {\n  display: grid;\n  gap: 2rem;\n}\n\
.blocks-hero-background-media-root {\n  position: relative;\n  isolation: isolate;\n  overflow: hidden;\n  display: grid;\n  min-height: 26rem;\n  padding: var(--fandhe-space-16) var(--fandhe-space-6);\n  border-radius: var(--fandhe-radius-lg);\n  color: var(--fandhe-color-bg);\n}\n\
.blocks-hero-background-media-backdrop {\n  position: absolute;\n  inset: 0;\n  z-index: -1;\n  overflow: hidden;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-hero-background-media-image] {\n  width: 100%;\n  height: 100%;\n  display: block;\n}\n\
.blocks-hero-background-media-scrim {\n  position: absolute;\n  inset: 0;\n  background: color-mix(in srgb, var(--fandhe-color-fg) 64%, transparent);\n}\n\
[data-blocks-hero-background-media-title],\n[data-blocks-hero-background-media-lead] {\n  color: inherit;\n}\n\
[data-scope=\"button\"][data-part=\"root\"][data-blocks-hero-background-media-cta-secondary] {\n  color: inherit;\n  border-color: currentColor;\n}\n\
[data-blocks-hero-background-media-variant=\"centered\"] .blocks-hero-background-media-content {\n  text-align: center;\n  align-items: center;\n  max-width: 48rem;\n  margin-inline: auto;\n  display: grid;\n  gap: 1rem;\n}\n\
[data-blocks-hero-background-media-variant=\"bottom-split\"] {\n  align-content: end;\n}\n\
[data-blocks-hero-background-media-variant=\"bottom-split\"] .blocks-hero-background-media-content {\n  display: grid;\n  gap: 1.25rem;\n}\n\
.blocks-hero-background-media-split-side {\n  display: grid;\n  gap: 1rem;\n}\n\
.blocks-hero-background-media-actions {\n  display: flex;\n  gap: 0.75rem;\n  flex-wrap: wrap;\n}\n\
[data-blocks-hero-background-media-variant=\"centered\"] .blocks-hero-background-media-actions {\n  justify-content: center;\n}\n\
@media (min-width: 64rem) {\n  [data-blocks-hero-background-media-variant=\"bottom-split\"] .blocks-hero-background-media-content {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    align-items: end;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// [`demo`] が 2 形（centered/bottom-split）を静的並記し、`aria-hidden`
    /// が backdrop のみに付き、`<form>`・`href="#"`・`data:` URI を含まない
    /// こと（`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_renders_expected_markup_and_avoids_disallowed_patterns() {
        let html = render(&demo());
        assert_eq!(html.matches(VARIANT_ATTR).count(), 2);
        assert!(html.contains(r#"data-blocks-hero-background-media-variant="centered""#));
        assert!(html.contains(r#"data-blocks-hero-background-media-variant="bottom-split""#));
        assert_eq!(html.matches(r#"aria-hidden="true""#).count(), 2);
        assert!(html.contains(r#"src="../../assets/blocks-demo-background.svg""#));
        assert!(html.contains(r#"alt="""#));
        for hook in [
            IMAGE_ATTR,
            EYEBROW_ATTR,
            TITLE_ATTR,
            LEAD_ATTR,
            CTA_PRIMARY_ATTR,
            CTA_SECONDARY_ATTR,
        ] {
            assert!(
                html.contains(hook),
                "demo should render the {hook} attribute"
            );
        }
        for text_fragment in [
            "Build the page, keep the platform",
            "A platform that fades into the background",
            "Get started",
            "See the demo",
        ] {
            assert!(
                html.contains(text_fragment),
                "demo should contain {text_fragment}"
            );
        }
        for absent in ["<form", "href=\"#\"", "src=\"data:", "<script", "id=\""] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が全セレクタを宣言し、色リテラルではなく
    /// トークン参照 + `color-mix()` で暗幕を作り、`animation`/`transition`
    /// を一切持たないこと。
    #[test]
    fn layout_css_declares_all_selectors_and_uses_color_tokens_not_literals() {
        for selector in [
            ".blocks-hero-background-media-layout {",
            ".blocks-hero-background-media-root {",
            ".blocks-hero-background-media-backdrop {",
            "[data-scope=\"image\"][data-part=\"root\"][data-blocks-hero-background-media-image] {",
            ".blocks-hero-background-media-scrim {",
            "[data-scope=\"button\"][data-part=\"root\"][data-blocks-hero-background-media-cta-secondary] {",
            "[data-blocks-hero-background-media-variant=\"centered\"] .blocks-hero-background-media-content {",
            "[data-blocks-hero-background-media-variant=\"bottom-split\"] {",
            ".blocks-hero-background-media-split-side {",
            "@media (min-width: 64rem) {",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(LAYOUT_CSS.contains("color-mix("));
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-fg)"));
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-bg)"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(!LAYOUT_CSS.contains("animation"));
        assert!(!LAYOUT_CSS.contains("transition"));
    }
}
