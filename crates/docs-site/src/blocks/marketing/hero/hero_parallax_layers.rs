//! `hero-parallax-layers` block（イシュー #2546。親 #2530「Phase 7:
//! Motion+ 部品化」配下、Motion+ の hero sections に相当する合成例）。
//!
//! # 出典に関する注記
//!
//! `bento_staggered`/`testimonials_stack` と同じ系統（Motion+ 参照系、
//! #2530/#2476）からの純追加である。Motion+ ではなく着想のみを参照し、
//! Rust/CSS で独自に再実装する（`docs/design/motion-reference-adoption-
//! policy.md` §9 準拠）。
//!
//! # 使用部品
//!
//! `heading`（見出し）/ `text`（リード文）/ `button`（CTA）の 3 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約）。背景・中景・前景の
//! 3 レイヤーは抽象図形（`div` + CSS グラデーション/`border-radius`）で
//! あり、独立した部品ではないため `parts` には列挙しない
//! （`text_reveal`/`cursor` を `parts` に列挙しない先例と同じ判断）。
//!
//! # 視差移動の実体は `SlotRecipe::parallax`
//!
//! [`fandhe_frontend_pre_styled_ui::recipe::SlotRecipe::parallax`] を直接
//! 使う（`crate::showcase::parallax_demo` と同じ手法。`bento_staggered`/
//! `testimonials_stack` のような生 CSS 手書きではなく、3 レイヤーへ
//! 速度差を持たせるため `ParallaxSpeed::Slow`/`Normal`/`Fast` をそのまま
//! 割り当てる）。マークアップは `[data-scope="blocks-hero-parallax"]`
//! `[data-part="layer-back"/"layer-mid"/"layer-front"/"content"]` を直接
//! 付与する（`crate::showcase::parallax_demo` と同型、`SlotRecipe` の
//! セレクタ生成契約と一致させるため `class` ではなく `data-scope`/
//! `data-part` を使う）。
//!
//! # `data-fandhe-scroll-progress` を付与しない理由
//!
//! `SlotRecipe::parallax` は非対応ブラウザ向けに `--fandhe-motion-scroll-
//! progress` を読む `calc()` フォールバックを持つが、これは
//! `fandhe-frontend-animation::scroll_driver` の JS 計測（`data-fandhe-
//! scroll-progress` 属性を持つ要素へ書き込む）を前提とする。docs-site は
//! JS ハイドレーションを一切行わないため、属性を付与しても JS 側が動かず
//! 無意味である（`crate::showcase::parallax_demo` と同じ判断）。実アプリで
//! 非対応ブラウザ向けフォールバックが必要な場合は `wasm-full`
//! `scroll-driver` feature と当該属性を利用者側で付与する。
//!
//! # `.blocks-demo` の `overflow-x: auto` を打ち消す理由
//!
//! `bento_staggered` と同じ理由（`crate::blocks::LAYOUT_CSS` doc・
//! `bento_staggered` モジュール doc参照）。`animation-timeline: view()`
//! が `.blocks-demo` 自身をスクロールコンテナ化させないよう、本 block
//! 限定で `overflow: visible` へ打ち消す。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。CTA ボタンは `button::button` の既定 `type="button"` の
//! まま用いる。文言はすべて架空のものであり、実企業名・実サービス名・
//! 実クレデンシャル・PII を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};

fn parallax_layer(part: &'static str) -> Node {
    div(
        vec![("data-scope", "blocks-hero-parallax"), ("data-part", part)],
        vec![],
    )
}

pub fn demo() -> Node {
    let content = div(
        vec![
            ("data-scope", "blocks-hero-parallax"),
            ("data-part", "content"),
        ],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("A workspace that moves with you")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    ..TextProps::default()
                },
                vec![],
                vec![text("Layers drift at different speeds as you scroll.")],
            ),
            button::button(&ButtonProps::default(), vec![], vec![text("Explore")]),
        ],
    );

    div(
        vec![("class", "blocks-hero-parallax-stage")],
        vec![
            parallax_layer("layer-back"),
            parallax_layer("layer-mid"),
            parallax_layer("layer-front"),
            content,
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/hero-parallax-layers/",
    title: "hero-parallax-layers",
    category: BlockCategory::Hero,
    rust_source: "crates/docs-site/src/blocks/marketing/hero/hero_parallax_layers.rs",
    demo_class: "blocks-hero-parallax-layers",
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
            label: "Button",
            path: "/themes/button/",
        },
    ],
    layout_css: LayoutCss::Dynamic(layout_css),
    demo,
};

/// `hero_parallax_layers` 固有のレイアウト規則を組み立てる（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節。`testimonials_stack` と
/// 同じ `pub(super) fn` 形式を使う——[`fandhe_frontend_pre_styled_ui::
/// recipe::SlotRecipe`] の `.css()` 呼び出し結果を固定レイアウト CSS と
/// 連結して返す必要があるため `const` にできない）。
///
/// # 3 レイヤーの抽象図形（CSS グラデーションのみ、画像を使わない）
///
/// `layer-back`/`layer-mid`/`layer-front` はいずれも `position: absolute`
/// でステージ全体を覆う `div` に、`radial-gradient`/`linear-gradient` と
/// `border-radius` の円形のみで構成する（`data:` URI・外部画像は使わない、
/// `crate::blocks` モジュール doc「セキュリティ不変条件」節）。
fn layout_css() -> String {
    let recipe = fandhe_frontend_pre_styled_ui::recipe::SlotRecipe::new(
        "blocks-hero-parallax",
        &["layer-back", "layer-mid", "layer-front", "content"],
    )
    .parallax(
        "layer-back",
        fandhe_frontend_pre_styled_ui::recipe::ParallaxSpeed::Slow,
    )
    .parallax(
        "layer-mid",
        fandhe_frontend_pre_styled_ui::recipe::ParallaxSpeed::Normal,
    )
    .parallax(
        "layer-front",
        fandhe_frontend_pre_styled_ui::recipe::ParallaxSpeed::Fast,
    );

    format!(
        "\
.blocks-demo.blocks-hero-parallax-layers {{
  overflow: visible;
}}
.blocks-hero-parallax-stage {{
  position: relative;
  min-height: 22rem;
  overflow: visible;
  border-radius: var(--fandhe-radius-lg, 0.75rem);
  background: var(--fandhe-color-bg-subtle);
}}
[data-scope=\"blocks-hero-parallax\"][data-part=\"layer-back\"] {{
  position: absolute;
  inset: 0;
  background: radial-gradient(circle at 20% 30%, var(--fandhe-color-accent-subtle, #cbd5f5) 0%, transparent 60%);
}}
[data-scope=\"blocks-hero-parallax\"][data-part=\"layer-mid\"] {{
  position: absolute;
  left: 10%;
  top: 20%;
  width: 40%;
  height: 40%;
  border-radius: 999px;
  background: var(--fandhe-color-accent, #64748b);
  opacity: 0.15;
}}
[data-scope=\"blocks-hero-parallax\"][data-part=\"layer-front\"] {{
  position: absolute;
  right: 8%;
  bottom: 10%;
  width: 30%;
  height: 30%;
  border-radius: 999px;
  background: var(--fandhe-color-accent-emphasized, #334155);
  opacity: 0.25;
}}
[data-scope=\"blocks-hero-parallax\"][data-part=\"content\"] {{
  position: relative;
  z-index: 1;
  max-width: 32rem;
  margin-inline: auto;
  padding: 3rem 1.5rem;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
}}
{recipe_css}",
        recipe_css = recipe.css(),
    )
}

#[cfg(test)]
mod tests {
    use super::layout_css;

    /// [`layout_css`] が `SlotRecipe::parallax` のプログレッシブ
    /// エンハンスメント契約（ネイティブ/フォールバック/reduced-motion の
    /// 3 ブロック）を実際に含んでいること（`SlotRecipe::parallax` doc
    /// 「プログレッシブエンハンスメント契約」節参照）。
    #[test]
    fn layout_css_includes_parallax_progressive_enhancement_blocks() {
        let css = layout_css();
        assert!(css.contains("@supports (animation-timeline: view())"));
        assert!(css.contains("@media (prefers-reduced-motion: reduce)"));
        assert!(css.contains("--fandhe-motion-parallax-distance"));
        assert!(css.contains("data-part=\"layer-back\""));
        assert!(css.contains("data-part=\"layer-mid\""));
        assert!(css.contains("data-part=\"layer-front\""));
    }
}
