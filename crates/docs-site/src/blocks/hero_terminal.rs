//! `hero-terminal` block（イシュー #2546。親 #2530「Phase 7: Motion+
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
//! `code`（コマンド行）/ `kbd`（ヒント表示 1 個）の 2 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約）。`text_reveal::typewriter`
//! は単体の Themes ページを持たないため `parts` には列挙しない
//! （`text_reveal`/`cursor` を `parts` に列挙しない先例と同じ判断）。
//!
//! # 行の順送りは時間軸 stagger（`hero_editorial_stagger` と同型）
//!
//! ページ先頭に置かれる hero である前提のため、[`super::
//! hero_editorial_stagger`] と同じ時間軸 stagger（
//! [`fandhe_frontend_pre_styled_ui::recipe::stagger_index_style`] が書く
//! `--fandhe-motion-stagger-index` を `animation-delay: calc(...)` へ乗せ、
//! [`fandhe_frontend_pre_styled_ui::motion::FADE_IN_KEYFRAMES_NAME`] を
//! 使う）で各行をフェードインさせる。
//!
//! # 最終行の typewriter は docs-site 上では静的表示
//!
//! [`fandhe_frontend_pre_styled_ui::text_reveal::typewriter`] は
//! マークアップ（[`fandhe_frontend_pre_styled_ui::text_reveal::
//! TYPEWRITER_ATTR`] opt-in 属性）のみを供給し、実際の 1 文字ずつの文字
//! 送りは `fandhe_frontend_wasm_full::text_animation` が担う（`text_reveal`
//! モジュール doc「3 種の役割分担」節）。docs-site は JS ハイドレーション
//! を行わないため、本 Demo では最終行のテキストが最初から静的に表示される
//! （`fd-text-reveal__display` 表示レイヤーの初期値がそのまま見える、
//! `text_reveal` モジュール doc「マークアップ契約」節参照）。
//!
//! # コマンド文字列は無害な架空コマンド
//!
//! 表示するコマンドはすべて本フレームワーク自身の CLI・一般的な Rust
//! ツールチェインのみで、実在の秘密情報・URL・トークンらしき文字列を
//! 含まない。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない（入力欄自体を持たない）。

use super::{Block, BlockCategory, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, span, text, Node};
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::kbd::{self, KbdProps};
use fandhe_frontend_pre_styled_ui::recipe::stagger_index_style;
use fandhe_frontend_pre_styled_ui::text_reveal;

const LINE_ATTR: &str = "data-blocks-hero-terminal-line";

fn prompt() -> Node {
    span(
        vec![("class", "blocks-hero-terminal-prompt")],
        vec![text("$ ")],
    )
}

fn command_line(index: usize, command: &str) -> Node {
    let style = stagger_index_style(index);
    div(
        vec![(LINE_ATTR, ""), ("style", style.as_str())],
        vec![
            prompt(),
            code::code(&CodeProps::default(), vec![], vec![text(command)]),
        ],
    )
}

pub fn demo() -> Node {
    let dot = || span(vec![("class", "blocks-hero-terminal-dot")], vec![]);
    let titlebar = div(
        vec![("class", "blocks-hero-terminal-titlebar")],
        vec![dot(), dot(), dot()],
    );

    let mut lines = vec![
        command_line(0, "fw new my-app"),
        command_line(1, "cd my-app"),
        command_line(2, "cargo run"),
    ];
    let typewriter_style = stagger_index_style(3);
    lines.push(div(
        vec![(LINE_ATTR, ""), ("style", typewriter_style.as_str())],
        vec![
            prompt(),
            text_reveal::typewriter("Ready in 42ms", None),
            span(
                vec![("class", "blocks-hero-terminal-hint")],
                vec![kbd::kbd(&KbdProps::default(), vec![], vec![text("⌘ K")])],
            ),
        ],
    ));

    div(
        vec![("data-blocks-hero-terminal-panel", "")],
        vec![
            titlebar,
            div(vec![("class", "blocks-hero-terminal-body")], lines),
        ],
    )
}
// blocks-code:end

/// [`super::BLOCKS`] へ登録するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/hero-terminal/",
    title: "hero-terminal",
    category: BlockCategory::Hero,
    rust_source: "crates/docs-site/src/blocks/hero_terminal.rs",
    demo_class: "blocks-hero-terminal",
    parts: &[
        Part {
            label: "Code",
            path: "/themes/code/",
        },
        Part {
            label: "Kbd",
            path: "/themes/kbd/",
        },
    ],
    demo,
};

/// `hero_terminal` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節、他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。
pub(super) const LAYOUT_CSS: &str = "\
[data-blocks-hero-terminal-panel] {\n  background: var(--fandhe-color-fg, #0f172a);\n  color: var(--fandhe-color-bg, #e2e8f0);\n  border-radius: var(--fandhe-radius-lg, 0.75rem);\n  padding: 1rem 1.25rem 1.5rem;\n  font-family: var(--fandhe-font-font-mono, monospace);\n}\n\
.blocks-hero-terminal-titlebar {\n  display: flex;\n  gap: 0.4rem;\n  margin-bottom: 1rem;\n}\n\
.blocks-hero-terminal-dot {\n  display: inline-block;\n  width: 0.65rem;\n  height: 0.65rem;\n  border-radius: 999px;\n  background: currentColor;\n  opacity: 0.35;\n}\n\
.blocks-hero-terminal-body {\n  display: flex;\n  flex-direction: column;\n  gap: 0.5rem;\n}\n\
.blocks-hero-terminal-prompt {\n  opacity: 0.6;\n  margin-right: 0.25rem;\n}\n\
.blocks-hero-terminal-hint {\n  margin-left: 0.5rem;\n}\n\
[data-blocks-hero-terminal-line] {\n  animation-name: fd-motion-fade-in;\n  animation-duration: var(--fandhe-motion-duration-normal);\n  animation-timing-function: var(--fandhe-motion-easing-standard);\n  animation-fill-mode: both;\n  animation-delay: calc(var(--fandhe-motion-stagger-index, 0) * var(--fandhe-motion-duration-fast));\n}\n\
@media (prefers-reduced-motion: reduce) {\n  [data-blocks-hero-terminal-line] {\n    animation: none;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::LAYOUT_CSS;
    use fandhe_frontend_core::render;
    use fandhe_frontend_pre_styled_ui::motion::FADE_IN_KEYFRAMES_NAME;
    use fandhe_frontend_pre_styled_ui::recipe::STAGGER_INDEX_VAR;
    use fandhe_frontend_pre_styled_ui::text_reveal::TYPEWRITER_ATTR;

    /// [`LAYOUT_CSS`] が参照する `@keyframes` 名・stagger var 名が、
    /// `motion`/`recipe` モジュール側の定数と実際に一致していること
    /// （手書き文字列のドリフトを防ぐ、`bento_staggered` と同型）。
    #[test]
    fn layout_css_references_the_shared_keyframes_name_and_stagger_var() {
        assert!(LAYOUT_CSS.contains(FADE_IN_KEYFRAMES_NAME));
        assert!(LAYOUT_CSS.contains(STAGGER_INDEX_VAR));
    }

    /// [`super::demo`] が [`TYPEWRITER_ATTR`] をちょうど 1 回含むこと
    /// （モジュール doc「最終行の typewriter」節の契約固定）。
    #[test]
    fn demo_contains_exactly_one_typewriter_opt_in() {
        let html = render(&super::demo());
        assert_eq!(html.matches(TYPEWRITER_ATTR).count(), 1);
    }
}
