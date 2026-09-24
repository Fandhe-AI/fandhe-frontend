//! `hero-editorial-stagger` block（イシュー #2546。親 #2530「Phase 7:
//! Motion+ 部品化」配下、Motion+ の hero sections に相当する合成例で、
//! `crate::blocks` モジュール doc の契約を `cursor-hover-cards` に続いて
//! 16 件目に実装する）。
//!
//! # 出典に関する注記
//!
//! `bento_staggered`/`testimonials_stack` と同じ系統（Motion+ 参照系、
//! #2530/#2476）からの純追加である。Motion+ ではなく着想のみを参照し、
//! Rust/CSS で独自に再実装する（`docs/design/motion-reference-adoption-
//! policy.md` §9 準拠。取得手段・ファイル名・内部識別子は記載しない）。
//!
//! # 使用部品
//!
//! `badge`（eyebrow）/ `heading`（見出し）/ `text`（リード文）/ `button`
//! （CTA 2 個）の 4 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # stagger は時間軸（`animation-delay`）で表現する
//!
//! `bento_staggered` の scroll-driven stagger（`animation-range`）とは
//! 異なり、本 block はページ先頭に置かれる hero である前提のため
//! ロード時に即座に発火する時間軸 stagger（
//! [`fandhe_frontend_pre_styled_ui::recipe::stagger_index_style`] が書く
//! `--fandhe-motion-stagger-index` を `animation-delay: calc(...)` へ乗せる）
//! を使う。`@supports (animation-timeline: view())` は不要（対応可否に
//! 関わらず常に動く）。
//!
//! # `animation-delay` を個別 `@media` で縮退させる理由
//!
//! `--fandhe-motion-duration-*` トークンは `prefers-reduced-motion:
//! reduce` 下で `Theme::to_css` が 0ms 化するが、[`LAYOUT_CSS`] の
//! `animation-delay` はリテラル `calc()` であり duration トークンの
//! 0 化だけでは消えない（[`super::text_split_reveal`]/`crate::pre_styled_ui::
//! text_reveal` と同じ既知の制約）。このため個別に
//! `@media (prefers-reduced-motion: reduce) { … { animation: none; } }`
//! を持つ。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。CTA ボタンは `button::button` の既定 `type="button"` の
//! まま用いる。文言はすべて架空のものであり、実企業名・実サービス名・
//! 実クレデンシャル・PII を含まない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge`/`heading::heading`/`text::text`/`button::button` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を
//! 黙って除去する契約を持つため、stagger 用の `data-*`/`style` 属性は
//! `attrs` へ直接渡す（`crate::blocks` モジュール doc「CSS フックが
//! `class` と `[data-*]` で混在する理由」節と同じ判断軸）。

use super::{Block, BlockCategory, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::recipe::stagger_index_style;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};

const ITEM_ATTR: &str = "data-blocks-hero-editorial-stagger-item";

pub fn demo() -> Node {
    let eyebrow_style = stagger_index_style(0);
    let eyebrow = badge::badge(
        &BadgeProps::default(),
        vec![(ITEM_ATTR, ""), ("style", eyebrow_style.as_str())],
        vec![text("New: workspace insights")],
    );

    let heading_style = stagger_index_style(1);
    let title = heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl4,
            ..HeadingProps::default()
        },
        vec![(ITEM_ATTR, ""), ("style", heading_style.as_str())],
        vec![text("Ship features your team can trust")],
    );

    let lead_style = stagger_index_style(2);
    let lead = styled_text::text(
        &TextProps {
            size: TextSize::Lg,
            ..TextProps::default()
        },
        vec![(ITEM_ATTR, ""), ("style", lead_style.as_str())],
        vec![text("Ship with confidence, review with ease.")],
    );

    let actions_style = stagger_index_style(3);
    let actions = div(
        vec![
            ("class", "blocks-hero-editorial-stagger-actions"),
            (ITEM_ATTR, ""),
            ("style", actions_style.as_str()),
        ],
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
    );

    div(
        vec![("class", "blocks-hero-editorial-stagger-inner")],
        vec![eyebrow, title, lead, actions],
    )
}
// blocks-code:end

/// [`super::BLOCKS`] へ登録するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/hero-editorial-stagger/",
    title: "hero-editorial-stagger",
    category: BlockCategory::Hero,
    rust_source: "crates/docs-site/src/blocks/hero_editorial_stagger.rs",
    demo_class: "blocks-hero-editorial-stagger",
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
    ],
    demo,
};

/// `hero_editorial_stagger` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節、他 block と同型で
/// `pub(super)` として `super::stylesheet` から連結される）。
pub(super) const LAYOUT_CSS: &str = "\
.blocks-hero-editorial-stagger-inner {\n  max-width: 48rem;\n  margin-inline: auto;\n  text-align: center;\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: 1rem;\n  padding-block: 2rem;\n}\n\
.blocks-hero-editorial-stagger-actions {\n  display: flex;\n  gap: 0.75rem;\n  flex-wrap: wrap;\n  justify-content: center;\n}\n\
[data-blocks-hero-editorial-stagger-item] {\n  animation-name: fd-motion-slide-from-bottom;\n  animation-duration: var(--fandhe-motion-duration-normal);\n  animation-timing-function: var(--fandhe-motion-easing-standard);\n  animation-fill-mode: both;\n  animation-delay: calc(var(--fandhe-motion-stagger-index, 0) * var(--fandhe-motion-duration-fast));\n}\n\
@media (prefers-reduced-motion: reduce) {\n  [data-blocks-hero-editorial-stagger-item] {\n    animation: none;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::LAYOUT_CSS;
    use fandhe_frontend_pre_styled_ui::motion::SLIDE_FROM_BOTTOM_KEYFRAMES_NAME;
    use fandhe_frontend_pre_styled_ui::recipe::STAGGER_INDEX_VAR;

    /// [`LAYOUT_CSS`] が参照する `@keyframes` 名・stagger var 名が、
    /// `motion`/`recipe` モジュール側の定数と実際に一致していること
    /// （手書き文字列のドリフトを防ぐ、`bento_staggered` と同型）。
    #[test]
    fn layout_css_references_the_shared_keyframes_name_and_stagger_var() {
        assert!(LAYOUT_CSS.contains(SLIDE_FROM_BOTTOM_KEYFRAMES_NAME));
        assert!(LAYOUT_CSS.contains(STAGGER_INDEX_VAR));
    }
}
