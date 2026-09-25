//! `error-page-background-image` block（イシュー #2836。親トラッキング
//! #2807「Blocks 目的別パーツ拡充ツリー Phase 2、マーケティング B」配下、
//! `crate::blocks::marketing::error_page` カテゴリ最初の block）。
//!
//! # 出典に関する注記
//!
//! 参照元は構造（全面背景レイヤーの上に空状態メッセージを重ねる 2 層
//! 構成）のみを参照し、Rust/CSS で独自に再実装する。参照元は暗幕なしで
//! 白文字を直書きしているが、本 block はテーマの前景/背景トークンで
//! 可読性を確保する設計へ意図的に変更した（後述「白文字を直書きしない
//! 理由」節）。
//!
//! # 使用部品
//!
//! `empty-state`（メッセージコンテナ）/ `heading`（見出し）/ `text`
//! （エラーコード・説明文）/ `image`（背景画像）/ `link`（ホームへ戻る）の
//! 5 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `empty_state::root`/`heading::heading`/`text::text`/`image::image`/
//! `link::root` はいずれも `drop_class_attr` により呼び出し側 `attrs` の
//! `class` を黙って除去する契約を持つため、本 block 固有のフックは
//! `data-blocks-error-page-background-image-*` の `data-*` 属性で渡す
//! （`crate::blocks` モジュール doc と同じ判断軸）。`empty_state::content`/
//! `actions` と素の `div` は `class` をそのまま透過するため、それらのみ
//! `class` でフックする。
//!
//! # 背景画像は共通ダミー素材
//!
//! `crate::blocks::dummy_assets::BACKGROUND_SRC`（ビルド時生成の
//! モノトーン背景タイル SVG）を使う。`data:` URI は `is_safe_url`
//! （REQ-1）が拒否するため使わない（イシュー #1562 の教訓）。
//!
//! # 白文字を直書きしない理由（意図的な逸脱として記録）
//!
//! 参照元は背景画像の上へ白文字を直書きしているが、本リポジトリの
//! 規約（トークンのみを使い、参照素材の配色を持ち込まない）を優先し、
//! 背景画像の上へ `--fandhe-color-bg` を半透明にしたスクリムを重ね、
//! 文字は `--fandhe-color-fg`/`--fandhe-color-fg-muted` で描く設計へ変更
//! した。こうすることでライト/ダーク両テーマでテーマ側が保証する
//! コントラストがそのまま保たれる（ダークテーマでは結果として白に近い
//! 文字になる）。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。静的表示のみで遷移処理・送信処理は一切持たない。文言は
//! すべて架空のものであり、実企業名・実サービス名・実クレデンシャル・
//! PII を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::empty_state::{self, EmptyStateProps, EmptyStateVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextVariant, TextWeight,
};

const ROOT_CLASS: &str = "blocks-error-page-background-image-root";
const BACKDROP_CLASS: &str = "blocks-error-page-background-image-backdrop";
const SCRIM_CLASS: &str = "blocks-error-page-background-image-scrim";
const CONTENT_CLASS: &str = "blocks-error-page-background-image-content";
const ACTIONS_CLASS: &str = "blocks-error-page-background-image-actions";

const IMAGE_ATTR: &str = "data-blocks-error-page-background-image-image";
const MESSAGE_ATTR: &str = "data-blocks-error-page-background-image-message";
const CODE_ATTR: &str = "data-blocks-error-page-background-image-code";
const TITLE_ATTR: &str = "data-blocks-error-page-background-image-title";
const DESCRIPTION_ATTR: &str = "data-blocks-error-page-background-image-description";
const BACK_ATTR: &str = "data-blocks-error-page-background-image-back";

pub fn demo() -> Node {
    let backdrop = div(
        vec![("class", BACKDROP_CLASS)],
        vec![
            image::image(
                &ImageProps::new(dummy_assets::BACKGROUND_SRC, ""),
                vec![(IMAGE_ATTR, "")],
            ),
            div(vec![("class", SCRIM_CLASS)], vec![]),
        ],
    );

    let code = styled_text::text(
        &TextProps {
            weight: TextWeight::Semibold,
            ..TextProps::default()
        },
        vec![(CODE_ATTR, "")],
        vec![text("404")],
    );

    let title = empty_state::title(
        vec![],
        vec![heading::heading(
            HeadingLevel::H3,
            &HeadingProps {
                size: HeadingSize::Xl3,
                ..HeadingProps::default()
            },
            vec![(TITLE_ATTR, "")],
            vec![text("Page not found")],
        )],
    );

    let description = empty_state::description(
        vec![],
        vec![styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![(DESCRIPTION_ATTR, "")],
            vec![text(
                "The page you are looking for has moved or never existed.",
            )],
        )],
    );

    let actions = empty_state::actions(
        vec![("class", ACTIONS_CLASS)],
        vec![link::root(
            "../../",
            &LinkProps::default(),
            vec![(BACK_ATTR, "")],
            vec![text("← Back to home")],
        )],
    );

    let message = empty_state::root(
        &EmptyStateProps {
            variant: EmptyStateVariant::Plain,
            ..EmptyStateProps::default()
        },
        vec![(MESSAGE_ATTR, "")],
        vec![empty_state::content(
            vec![("class", CONTENT_CLASS)],
            vec![code, title, description, actions],
        )],
    );

    div(vec![("class", ROOT_CLASS)], vec![backdrop, message])
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/error-page-background-image/",
    title: "error-page-background-image",
    category: BlockCategory::ErrorPage,
    rust_source: "crates/docs-site/src/blocks/marketing/error_page/error_page_background_image.rs",
    demo_class: "blocks-error-page-background-image",
    parts: &[
        Part {
            label: "Empty State",
            path: "/themes/empty-state/",
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
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `error_page_background_image` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節、他 block と同型で
/// `pub(super)` ではなく本ファイル内 `const` として [`super::blocks`] から
/// `BLOCK.layout_css` 経由で連結される）。
///
/// 生の色リテラル（`#fff`/`white` 等）は使わず、可読性の確保はすべて
/// `--fandhe-color-*` トークンで行う（モジュール冒頭「白文字を直書きしない
/// 理由」節）。
const LAYOUT_CSS: &str = "\
.blocks-error-page-background-image {\n  padding: 0;\n  overflow: hidden;\n}\n\
.blocks-error-page-background-image-root {\n  position: relative;\n  isolation: isolate;\n  display: grid;\n  place-items: center;\n  min-height: 24rem;\n  padding: var(--fandhe-space-16) var(--fandhe-space-6);\n  text-align: center;\n}\n\
.blocks-error-page-background-image-backdrop {\n  position: absolute;\n  inset: 0;\n  z-index: -1;\n  overflow: hidden;\n}\n\
[data-blocks-error-page-background-image-image] {\n  width: 100%;\n  height: 100%;\n  display: block;\n}\n\
.blocks-error-page-background-image-scrim {\n  position: absolute;\n  inset: 0;\n  background: var(--fandhe-color-bg);\n  opacity: 0.72;\n}\n\
[data-blocks-error-page-background-image-message] {\n  max-width: 36rem;\n  width: 100%;\n  color: var(--fandhe-color-fg);\n}\n\
.blocks-error-page-background-image-content {\n  align-items: center;\n}\n\
[data-blocks-error-page-background-image-code] {\n  color: var(--fandhe-color-accent);\n  letter-spacing: 0.05em;\n}\n\
[data-blocks-error-page-background-image-description] {\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-error-page-background-image-actions {\n  justify-content: center;\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// [`demo`] が背景画像・スクリム・見出し・戻るリンクを正しい属性・
    /// 文言で出力し、`<form>`・`href="#"`・`data:` URI・`<script` の
    /// いずれも含まないこと（`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_renders_expected_markup_and_avoids_disallowed_patterns() {
        let html = render(&demo());
        assert!(html.contains(r#"src="../../assets/blocks-demo-background.svg""#));
        assert!(html.contains(r#"alt="""#));
        assert!(html.contains(r#"href="../../""#));
        for hook in [
            IMAGE_ATTR,
            MESSAGE_ATTR,
            CODE_ATTR,
            TITLE_ATTR,
            DESCRIPTION_ATTR,
            BACK_ATTR,
        ] {
            assert!(
                html.contains(hook),
                "demo should render the {hook} attribute"
            );
        }
        for text_fragment in ["404", "Page not found", "Back to home"] {
            assert!(
                html.contains(text_fragment),
                "demo should contain {text_fragment}"
            );
        }
        for absent in ["<form", "href=\"#\"", "src=\"data:", "<script"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が全セレクタを宣言し、色リテラルではなくトークン
    /// 参照（`--fandhe-color-fg`/`--fandhe-color-bg`）で可読性を確保して
    /// いること（モジュール冒頭「白文字を直書きしない理由」節）。
    #[test]
    fn layout_css_declares_all_selectors_and_uses_color_tokens_not_literals() {
        for selector in [
            ".blocks-error-page-background-image {",
            ".blocks-error-page-background-image-root {",
            ".blocks-error-page-background-image-backdrop {",
            "[data-blocks-error-page-background-image-image] {",
            ".blocks-error-page-background-image-scrim {",
            "[data-blocks-error-page-background-image-message] {",
            ".blocks-error-page-background-image-content {",
            "[data-blocks-error-page-background-image-code] {",
            "[data-blocks-error-page-background-image-description] {",
            ".blocks-error-page-background-image-actions {",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-fg)"));
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-bg)"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
    }
}
