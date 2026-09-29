//! `action-panel-stacked` block（イシュー #2954。Application / Action
//! Panel カテゴリ最初の block、`docs/design/docs-site-blocks-section.md`
//! §18「カテゴリの卒業」対象）。
//!
//! # 使用部品
//!
//! `card` / `heading` / `text` / `button` / `link` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 3 インスタンス並記
//!
//! カード内にタイトル・説明文・主操作を縦に積んだ骨格を [`panel`] で共有し、
//! (1) ボタン操作・影付きカード（主参照）、(2) 矢印付きリンク操作・影付き
//! カード、(3) ボタン操作・面色のみの枠（影なし、`well` 相当）の 3 通りを
//! 並べる。狭幅でも縦積みのまま、主操作は内容幅を保ち全幅化しない
//! （[`LAYOUT_CSS`] 参照）。
//!
//! # `CardVariant::Subtle` を「面色のみの枠」に使う理由
//!
//! `fandhe_frontend_pre_styled_ui::card::CardVariant` は
//! `Elevated`/`Outline`/`Subtle` の 3 種を持つ。「影付きカード」は
//! `Elevated`、「面色のみの枠（影なし）」は境界線を持つ `Outline` ではなく
//! 淡色背景の `Subtle` が対応する（`crates/pre-styled-ui/src/card.rs` の
//! variant 定義参照）。
//!
//! `Subtle` の背景は `--fandhe-color-bg-subtle` で、`.blocks-demo` 枠の
//! 背景と同一トークンのため境界線なしではデモ上でパネルが同化して見える
//! （イシュー #2954 Bugbot レビュー）。`card.rs` 共有 variant 定義は変えず、
//! 本 block 固有の `[data-blocks-action-panel-stacked-panel="button-subtle"]`
//! セレクタで境界線のみ追加し、デモ上での視認性を確保する（[`LAYOUT_CSS`]
//! 参照）。
//!
//! # リンク操作のラベルと遷移先
//!
//! 矢印付きリンク版は `href="#"` を使わず、他 block と同型の実在 URL
//! （自リポジトリ `REPO`）へ `external: true` で遷移させる。可視テキストを
//! 遷移先がわかる「GitHub で見る」にし、末尾の矢印（`→`）は
//! `aria-hidden="true"` の装飾として付与する（イシュー #2931 codex レビュー
//! の「表示文言と遷移先を一致させる」教訓を踏襲）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `card::root`/`button::button`/`link::root` は `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、パネル
//! 識別・主操作の内容幅固定の CSS フックは `data-*` 属性で渡す
//! （[`LAYOUT_CSS`] 参照）。`card::header`/`card::body`/`card::footer` と
//! 素の `div` には `class` がそのまま効く。
//!
//! # `<form>` を持たない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節に従い、本 Demo
//! はフォーム・送信処理・状態機械を持たない静的な合成例である。ボタンは
//! 既定 `type="button"` のまま送信先を持たない。文言はすべて独自に書いた
//! 架空のものであり、実企業名・PII・クレデンシャルを含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// リンク操作版の遷移先（外部の実在 URL、`href="#"` は使わない、他 block と
/// 同型の判断。モジュール doc「リンク操作のラベルと遷移先」参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 3 インスタンスが共有するカード骨格（見出し・説明・主操作を縦に積む）。
/// `variant` でカードの見せ方（影付き/面色のみ）を切り替え、`action` に
/// 呼び出し側が組み立てたボタン or リンクを渡す。
fn panel(
    variant: CardVariant,
    instance: &'static str,
    title: &str,
    description: &str,
    action: Node,
) -> Node {
    card::root(
        CardProps {
            variant,
            ..CardProps::default()
        },
        vec![("data-blocks-action-panel-stacked-panel", instance)],
        vec![
            card::header(
                vec![],
                vec![heading(
                    HeadingLevel::H3,
                    &HeadingProps {
                        size: HeadingSize::Lg,
                        weight: HeadingWeight::Semibold,
                    },
                    vec![],
                    vec![text(title)],
                )],
            ),
            card::body(
                vec![("class", "blocks-action-panel-stacked-body")],
                vec![styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text(description)],
                )],
            ),
            card::footer(
                vec![("class", "blocks-action-panel-stacked-footer")],
                vec![action],
            ),
        ],
    )
}

/// (1) 主参照: ボタン操作・影付きカード。
fn button_elevated_panel() -> Node {
    panel(
        CardVariant::Elevated,
        "button-elevated",
        "通知設定を見直す",
        "メール・アプリ内通知の頻度をまとめて見直せます。設定は後からいつでも変更できます。",
        button(
            &ButtonProps::default(),
            vec![("data-blocks-action-panel-stacked-action", "")],
            vec![text("設定を見直す")],
        ),
    )
}

/// (2) 矢印付きリンク操作・影付きカード。
fn link_elevated_panel() -> Node {
    let action = link::root(
        REPO,
        &LinkProps {
            external: true,
            ..LinkProps::default()
        },
        vec![("data-blocks-action-panel-stacked-action", "")],
        vec![
            text("GitHub で見る"),
            span(vec![("aria-hidden", "true")], vec![text(" \u{2192}")]),
        ],
    );
    panel(
        CardVariant::Elevated,
        "link-elevated",
        "リポジトリを見る",
        "ソースコードや Issue をリポジトリでまとめて確認できます。",
        action,
    )
}

/// (3) ボタン操作・面色のみの枠（影なし）。
fn button_subtle_panel() -> Node {
    panel(
        CardVariant::Subtle,
        "button-subtle",
        "ワークスペースをアーカイブする",
        "アーカイブ後もデータは保持され、必要になれば復元できます。",
        button(
            &ButtonProps::default(),
            vec![("data-blocks-action-panel-stacked-action", "")],
            vec![text("アーカイブする")],
        ),
    )
}

/// `action-panel-stacked` の Demo 本体（3 インスタンスを縦に並べる。
/// [`LAYOUT_CSS`] 参照）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-action-panel-stacked-layout")],
        vec![
            button_elevated_panel(),
            link_elevated_panel(),
            button_subtle_panel(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/action-panel-stacked/",
    title: "action-panel-stacked",
    category: BlockCategory::ActionPanel,
    rust_source: "crates/docs-site/src/blocks/application/action_panel/action_panel_stacked.rs",
    demo_class: "blocks-action-panel-stacked",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
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
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `action_panel_stacked` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型）。
///
/// ルート grid class（`-layout`）は [`Block::demo_class`]
/// （`blocks-action-panel-stacked`）と意図的に別名にする（`card_meta_cta`
/// と同じ Bugbot 教訓の回避）。狭幅でも 1 列固定のため `@media` は不要
/// （`max-width` で頭打ちにするのみ）。主操作は `width: auto` で内容幅を
/// 保ち、全幅化しない（`card_meta_cta` の CTA 全幅化とは逆の意図）。
const LAYOUT_CSS: &str = "\
.blocks-action-panel-stacked {\n  padding: 3rem 1.5rem;\n}\n\
.blocks-action-panel-stacked-layout {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-6);\n  max-width: 40rem;\n}\n\
.blocks-action-panel-stacked-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-action-panel-stacked-footer {\n  display: flex;\n  justify-content: flex-start;\n}\n\
[data-blocks-action-panel-stacked-action] {\n  width: auto;\n}\n\
[data-blocks-action-panel-stacked-panel=\"button-subtle\"] {\n  border: 1px solid var(--fandhe-color-border);\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が [`crate::blocks::Block::parts`] と一致する 5 部品すべてを
    /// 出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"button\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// パネルが 3 枚で、影付き（elevated）2 枚・面色のみ（subtle）1 枚の
    /// variant 内訳になっていること。
    #[test]
    fn demo_renders_three_panels_with_expected_variants() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-action-panel-stacked-panel=")
                .count(),
            3
        );
        assert_eq!(html.matches("fd-card--variant-elevated").count(), 2);
        assert_eq!(html.matches("fd-card--variant-subtle").count(), 1);
    }

    /// `<form>`・暗黙 submit・script・data URI・`href="#"` を含まず、
    /// `type="button"` を持つこと（`crate::blocks` モジュール doc
    /// 「`<form>` を使わない」節・`blocks_contract.rs` の横断検査を
    /// 個別にも固定する）。
    #[test]
    fn demo_has_no_form_or_dangerous_markup() {
        let html = render(&demo());
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(html.contains("type=\"button\""));
    }

    /// [`LAYOUT_CSS`] が `<` を含まず、主操作を内容幅に保つ規則
    /// （`width: 100%` を与えない）を持ち、ルート grid class が
    /// `demo_class` と別名であること。
    #[test]
    fn layout_css_keeps_actions_at_content_width() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(!LAYOUT_CSS.contains("width: 100%"));
        assert!(
            LAYOUT_CSS.contains("[data-blocks-action-panel-stacked-action] {\n  width: auto;\n}")
        );
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-action-panel-stacked-layout"
        );
    }

    /// `Subtle` パネル（背景が `.blocks-demo` と同一トークン）が境界線を
    /// 持ち、デモ上で同化しないこと（イシュー #2954 Bugbot レビュー）。
    #[test]
    fn subtle_panel_has_visible_border() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-action-panel-stacked-panel=\"button-subtle\"] {\n  border: 1px solid var(--fandhe-color-border);\n}"
        ));
    }
}
