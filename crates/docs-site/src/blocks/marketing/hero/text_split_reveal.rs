//! `text-split-reveal` block（イシュー #2546。親 #2530「Phase 7: Motion+
//! 部品化」配下、Motion+ の hero sections に相当する合成例）。
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
//! `heading`（大見出し、文字単位 reveal）/ `text`（リード文、単語単位
//! reveal）/ `button`（CTA）の 3 部品を合成する（[`BLOCK`] の `parts` に
//! 一致させる契約）。`text_reveal::chars`/`words` 自体は単体の Themes
//! ページを持たないため `parts` には列挙しない（`text_reveal`/`cursor`
//! を `parts` に列挙しない先例と同じ判断）。
//!
//! # reveal は SSR のみで完結する（JS 不要）
//!
//! [`fandhe_frontend_pre_styled_ui::text_reveal::chars`]/[`words`] は
//! 分割 + stagger + `@keyframes` アニメーションのみで完結する A 群
//! （`text_reveal` モジュール doc「3 種の役割分担」節）であり、[`super::
//! hero_terminal`] の typewriter（C 群、JS 必須）と異なり docs-site 上でも
//! そのまま動く。
//!
//! # `TEXT_REVEAL_CSS` を Blocks が初めて `push_css` する
//!
//! `crates/pre-styled-ui` の `motion` feature は本クレートの
//! `Cargo.toml` で既に有効（イシュー #2524、`bento_staggered` モジュール
//! doc参照）だが、[`fandhe_frontend_pre_styled_ui::text_reveal::
//! TEXT_REVEAL_CSS`] 自体はどの block も `push_css` していなかった。
//! [`super::stylesheet`] が本 block の追加にあわせて 1 回だけ push する
//! （`motion::KEYFRAMES_CSS` を `bento_staggered` が初めて push したのと
//! 同型の経緯）。
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
use fandhe_frontend_pre_styled_ui::text::{self, TextProps};
use fandhe_frontend_pre_styled_ui::text_reveal;

pub fn demo() -> Node {
    let title = heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl4,
            ..HeadingProps::default()
        },
        vec![],
        vec![text_reveal::chars("Built for clarity")],
    );

    let lead = div(
        vec![("class", "blocks-text-split-reveal-lead")],
        vec![text::text(
            &TextProps::default(),
            vec![],
            vec![text_reveal::words("Words fade in one by one, in order.")],
        )],
    );

    let cta = button::button(&ButtonProps::default(), vec![], vec![text("Try it out")]);

    div(
        vec![("class", "blocks-text-split-reveal-inner")],
        vec![title, lead, cta],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/text-split-reveal/",
    title: "text-split-reveal",
    category: BlockCategory::Hero,
    rust_source: "crates/docs-site/src/blocks/marketing/hero/text_split_reveal.rs",
    demo_class: "blocks-text-split-reveal",
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
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `text_split_reveal` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節、他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される。reveal 自体のアニメーションは
/// [`fandhe_frontend_pre_styled_ui::text_reveal::TEXT_REVEAL_CSS`] が
/// 持つため、本定数は配置のみを担う）。
const LAYOUT_CSS: &str = "\
.blocks-text-split-reveal-inner {\n  max-width: 40rem;\n  margin-inline: auto;\n  text-align: center;\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  gap: 1rem;\n  padding-block: 2rem;\n}\n\
.blocks-text-split-reveal-lead {\n  max-width: 32rem;\n}\n";

#[cfg(test)]
mod tests {
    use super::demo;
    use fandhe_frontend_core::render;

    /// [`demo`] が [`fandhe_frontend_pre_styled_ui::text_reveal::
    /// TEXT_REVEAL_ATTR`] の `"chars"`/`"words"` を各 1 回含むこと
    /// （モジュール doc「使用部品」節の契約固定）。
    #[test]
    fn demo_contains_one_chars_reveal_and_one_words_reveal() {
        let html = render(&demo());
        assert_eq!(html.matches("data-fandhe-text-reveal=\"chars\"").count(), 1);
        assert_eq!(html.matches("data-fandhe-text-reveal=\"words\"").count(), 1);
    }

    /// SR 用の分割前テキストレイヤーと `aria-hidden` の分割済み表示レイヤーが
    /// 見出し・リード文の双方に存在すること（`text_reveal` モジュール doc
    /// 「マークアップ契約」節の 2 層契約が Demo でも保たれていることの固定）。
    #[test]
    fn demo_has_sr_layer_and_aria_hidden_units_layer_for_both_reveals() {
        let html = render(&demo());
        assert_eq!(html.matches("fd-text-reveal__sr").count(), 2);
        assert_eq!(
            html.matches("class=\"fd-text-reveal__units\" aria-hidden=\"true\"")
                .count(),
            2
        );
    }
}
