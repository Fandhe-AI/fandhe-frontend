//! `hero-marquee-strip` block（イシュー #2787。中央寄せの見出し・リード文・
//! CTA の下に、ロゴ列が横に流れる帯を持つ hero）。
//!
//! # 出典に関する注記
//!
//! 集約元（対応表 ID 準拠、固有名・ファイル名は記載しない）3 件のうち
//! 基準形を採用し、CTA 文言違い・ロゴ枚数違いの 2 件は本 Demo の差分として
//! `site/blocks/hero-marquee-strip.md` の「原案差分メモ」節で扱う
//! （`hero_editorial_stagger`/`content_with_testimonial` と同じ判断軸）。
//!
//! # 使用部品
//!
//! `badge`（eyebrow）/ `heading`（見出し）/ `text`（リード文・キャプション）/
//! `button`（CTA 2 個）/ `marquee`（ロゴ列）/ `image`（ロゴ画像）の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # a11y・reduced-motion は `marquee` 部品側の契約に委ねる
//!
//! ロゴ列の複製列 `aria-hidden`+`inert`・両端フェード
//! （`--fandhe-marquee-fade`）・`prefers-reduced-motion: reduce` 環境での
//! 折り返し静的表示・hover/focus-within での一時停止は、いずれも
//! [`fandhe_frontend_pre_styled_ui::marquee`] が常時提供する契約であり、
//! 本 block 側の CSS では上書きしない（`crates/pre-styled-ui/src/
//! marquee.rs` モジュール doc 参照）。`decorative: false` + `label` を
//! 選んだのは、reduced-motion の静止列でも `alt` 経由でロゴ名が読める
//! ようにするため（`decorative: true` はサブツリー全体を支援技術から
//! 隠してしまうため不採用）。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。CTA ボタンは `button::button` の既定 `type="button"` の
//! まま用いる。社名・文言はすべて `crate::blocks::dummy_assets` の架空
//! データであり、実企業名・実サービス名・実クレデンシャル・PII を含まない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge`/`heading::heading`/`text::text`/`marquee::marquee`/
//! `image::image` はいずれも `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、block 固有の CSS フックは
//! `data-*` 属性で渡す（`hero_editorial_stagger` と同じ判断軸）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::marquee::{self, MarqueeProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};

const STRIP_ATTR: &str = "data-blocks-hero-marquee-strip-strip";
const LOGO_ATTR: &str = "data-blocks-hero-marquee-strip-logo";
const LOGO_COUNT: usize = 8;

pub fn demo() -> Node {
    let eyebrow = badge::badge(&BadgeProps::default(), vec![], vec![text("New")]);

    let title = heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl4,
            ..HeadingProps::default()
        },
        vec![],
        vec![text("Build on a platform teams already trust")],
    );

    let lead = styled_text::text(
        &TextProps {
            size: TextSize::Lg,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "Ship faster with the workflows your team already knows.",
        )],
    );

    let actions = div(
        vec![("class", "blocks-hero-marquee-strip-actions")],
        vec![
            button::button(&ButtonProps::default(), vec![], vec![text("Get started")]),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("Talk to sales")],
            ),
        ],
    );

    let caption = styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            ..TextProps::default()
        },
        vec![("class", "blocks-hero-marquee-strip-caption")],
        vec![text("Trusted by teams of every size")],
    );

    let logos = (0..LOGO_COUNT)
        .map(|i| {
            let name = dummy_assets::COMPANY_NAMES[i % dummy_assets::COMPANY_NAMES.len()];
            marquee::item(
                vec![],
                vec![image::image(
                    &ImageProps {
                        fit: ImageFit::Contain,
                        ..ImageProps::new(dummy_assets::LOGO_SRC, name)
                    },
                    vec![(LOGO_ATTR, "")],
                )],
            )
        })
        .collect();
    let strip = marquee::marquee(
        &MarqueeProps {
            label: Some("Teams using the platform"),
            ..MarqueeProps::default()
        },
        vec![(STRIP_ATTR, "")],
        logos,
    );

    div(
        vec![("class", "blocks-hero-marquee-strip-inner")],
        vec![eyebrow, title, lead, actions, caption, strip],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/hero-marquee-strip/",
    title: "hero-marquee-strip",
    category: BlockCategory::Hero,
    rust_source: "crates/docs-site/src/blocks/marketing/hero/hero_marquee_strip.rs",
    demo_class: "blocks-hero-marquee-strip",
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
            label: "Marquee",
            path: "/themes/marquee/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `hero_marquee_strip` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節、他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。`--fandhe-marquee-*` custom
/// property の上書きのみで [`marquee`] の a11y/reduced-motion 契約には
/// 触れない（モジュール doc「a11y・reduced-motion は `marquee` 部品側の
/// 契約に委ねる」節参照）。
const LAYOUT_CSS: &str = "\
.blocks-hero-marquee-strip-inner {\n  max-width: 48rem;\n  margin-inline: auto;\n  text-align: center;\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: 1rem;\n  padding-block: 2rem;\n}\n\
.blocks-hero-marquee-strip-actions {\n  display: flex;\n  gap: 0.75rem;\n  flex-wrap: wrap;\n  justify-content: center;\n}\n\
.blocks-hero-marquee-strip-caption {\n  margin-top: 1rem;\n}\n\
[data-blocks-hero-marquee-strip-strip] {\n  width: 100%;\n  margin-top: 0.5rem;\n  --fandhe-marquee-fade: 4rem;\n  --fandhe-marquee-gap: var(--fandhe-space-8);\n  --fandhe-marquee-duration: 30s;\n}\n\
[data-blocks-hero-marquee-strip-logo] {\n  width: 7rem;\n  height: 2.5rem;\n  opacity: 0.7;\n}\n";

#[cfg(test)]
mod tests {
    use super::LAYOUT_CSS;

    /// [`LAYOUT_CSS`] が帯フックの custom property・セレクタを実際に
    /// 参照していること（文字列ドリフト防止、`hero_editorial_stagger` と
    /// 同型）。
    #[test]
    fn layout_css_references_the_strip_hook_and_fade_property() {
        assert!(LAYOUT_CSS.contains("[data-blocks-hero-marquee-strip-strip]"));
        assert!(LAYOUT_CSS.contains("--fandhe-marquee-fade"));
    }
}
