//! `testimonial-background-image` block（イシュー #2883。親トラッキング
//! #2731「Blocks 目的別パーツ拡充ツリー」配下、`crate::blocks::
//! marketing::testimonial` カテゴリ 2 件目の block）。
//!
//! # 出典に関する注記
//!
//! 参照元は「背景画像の上に暗幕を重ね、中央のパネルにロゴ・引用文・
//! 著者を置く」構造のみを参照し、Rust/CSS で独自に再実装する（対応表
//! ID R1362、基準形）。集約元は同一 1 件のみで差分はない
//! （`site/blocks/testimonial-background-image.md` の「原案差分メモ」
//! 節参照）。
//!
//! # 使用部品
//!
//! `blockquote`（引用文・出典）/ `image`（背景画像）/ `icon`（ロゴ）の
//! 3 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `blockquote::root`/`image::image`/`icon::icon` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、本 block 固有のフックは `data-blocks-testimonial-
//! background-image-*` の `data-*` 属性で渡す（`stats_background_image`
//! と同じ判断軸）。素の `div` は `class` をそのまま透過するため、それらの
//! みを `class` でフックする。
//!
//! # 背景画像は共通ダミー素材
//!
//! `crate::blocks::dummy_assets::BACKGROUND_SRC`（ビルド時生成のモノトーン
//! 背景タイル SVG）を使う。`data:` URI は `is_safe_url`（REQ-1）が拒否する
//! ため使わない（イシュー #1562 の教訓）。
//!
//! # 暗幕と文字色の設計判断
//!
//! `stats_background_image` と同じ反転ペアを採用する: 暗幕を
//! `--fandhe-color-fg` ベース、文字を `--fandhe-color-bg` ベースにする。
//! パネル面はさらに一段暗い半透明の塗りにし、暗幕単独より読みやすくする。
//! 色リテラル（`#…`/`white`/`black`）は使わず、`--fandhe-*` トークンと
//! `color-mix()` のみで表現する。
//!
//! # 背景画像フックの詳細度
//!
//! `image::image` の recipe base 宣言（`[data-scope="image"]
//! [data-part="root"] { height: auto; ... }`、詳細度 0,2,0）に勝たせる
//! ため、本 block 固有フックも同じ 2 属性セレクタへ前置し、ソース順
//! （`blocks.css` は `pre-styled-ui.css` より後に読み込まれる）で
//! 後勝ちさせる（`stats_background_image`/`error_page_background_image`
//! と同じ教訓）。
//!
//! # blockquote の上書き詳細度
//!
//! `[data-scope="blockquote"][data-part="root"]`（base 宣言、詳細度
//! 0,2,0）と variant 宣言（`.blocks-*` クラス、詳細度 0,1,0 合算）に
//! 勝たせるため、パネル祖先クラスを含む 4 セレクタ（詳細度 0,4,0）で
//! 左罫線の除去と `--fandhe-blockquote-caption-fg` の上書きを行う。
//!
//! # レスポンシブ（`48rem` のリテラルを直書きする理由）
//!
//! 他の Blocks（`stats_background_image` 等）と同じく、`pre-styled-ui`
//! の breakpoint トークンは `SlotRecipe` 経由の変数生成専用であり、
//! docs-site の生 CSS（`format!`/文字列 const で組み立てる）へ直接
//! 参照する経路を持たないため、リテラル値を直書きする（意図的な差分、
//! 既存 Blocks 全件と同じ判断）。
//!
//! # `<form>` を持たない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。静的表示のみで送信処理・データ取得は一切持たない。文言は
//! すべて `crate::blocks::dummy_assets` の架空データであり、実企業名・
//! 実サービス名・実クレデンシャル・PII を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::blockquote::{self, BlockquoteVariant};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::ColorPalette;
use fandhe_frontend_pre_styled_ui::Size;

const ROOT_CLASS: &str = "blocks-testimonial-background-image-root";
const BACKDROP_CLASS: &str = "blocks-testimonial-background-image-backdrop";
const SCRIM_CLASS: &str = "blocks-testimonial-background-image-scrim";
const PANEL_CLASS: &str = "blocks-testimonial-background-image-panel";
const AUTHOR_CLASS: &str = "blocks-testimonial-background-image-author";

const IMAGE_ATTR: &str = "data-blocks-testimonial-background-image-image";
const LOGO_ATTR: &str = "data-blocks-testimonial-background-image-logo";
const QUOTE_ATTR: &str = "data-blocks-testimonial-background-image-quote";
const NAME_ATTR: &str = "data-blocks-testimonial-background-image-name";
const ROLE_ATTR: &str = "data-blocks-testimonial-background-image-role";

/// 抽象図形ロゴ（六角形の輪郭。実在の企業ロゴを模さない）。
fn logo_icon(company: &str) -> Node {
    icon(
        &IconProps {
            size: Size::Xl,
            label: Some(company),
            ..IconProps::default()
        },
        vec![(LOGO_ATTR, "")],
        vec![el(
            "path",
            vec![("d", "M12 2L21 7V17L12 22L3 17V7Z")],
            vec![],
        )],
    )
}

/// `testimonial-background-image` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数。
pub fn demo() -> Node {
    let backdrop = div(
        vec![("class", BACKDROP_CLASS), ("aria-hidden", "true")],
        vec![
            image::image(
                &ImageProps::new(dummy_assets::BACKGROUND_SRC, ""),
                vec![(IMAGE_ATTR, "")],
            ),
            div(vec![("class", SCRIM_CLASS)], vec![]),
        ],
    );

    let quote = blockquote::root(
        BlockquoteVariant::Plain,
        ColorPalette::default(),
        vec![(QUOTE_ATTR, "")],
        vec![
            blockquote::content(vec![], vec![text(dummy_assets::TESTIMONIAL_QUOTES[0])]),
            blockquote::caption(
                vec![],
                vec![div(
                    vec![("class", AUTHOR_CLASS)],
                    vec![
                        div(
                            vec![(NAME_ATTR, "")],
                            vec![text(dummy_assets::PERSON_NAMES[0])],
                        ),
                        div(
                            vec![(ROLE_ATTR, "")],
                            vec![text(dummy_assets::JOB_TITLES[0])],
                        ),
                    ],
                )],
            ),
        ],
    );

    let panel = div(
        vec![("class", PANEL_CLASS)],
        vec![logo_icon(dummy_assets::COMPANY_NAMES[0]), quote],
    );

    div(vec![("class", ROOT_CLASS)], vec![backdrop, panel])
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/testimonial-background-image/",
    title: "testimonial-background-image",
    category: BlockCategory::Testimonial,
    rust_source:
        "crates/docs-site/src/blocks/marketing/testimonial/testimonial_background_image.rs",
    demo_class: "blocks-testimonial-background-image",
    parts: &[
        Part {
            label: "Blockquote",
            path: "/themes/blockquote/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `testimonial_background_image` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「CSS の置き場」節、他 block と同型で `pub(super)` では
/// なく本ファイル内 `const` として [`super::blocks`] から `BLOCK.layout_css`
/// 経由で連結される）。
///
/// 生の色リテラル（`#fff`/`white`/`black` 等）は使わず、可読性の確保は
/// すべて `--fandhe-color-*` トークンと `color-mix()` で行う（モジュール
/// 冒頭「暗幕と文字色の設計判断」節）。
const LAYOUT_CSS: &str = "\
.blocks-testimonial-background-image {\n  padding: 0;\n  overflow: hidden;\n}\n\
.blocks-testimonial-background-image-root {\n  position: relative;\n  isolation: isolate;\n  overflow: hidden;\n  display: grid;\n  place-items: center;\n  min-height: 24rem;\n  padding: var(--fandhe-space-4) 0;\n  color: var(--fandhe-color-bg);\n}\n\
.blocks-testimonial-background-image-backdrop {\n  position: absolute;\n  inset: 0;\n  z-index: -1;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-testimonial-background-image-image] {\n  width: 100%;\n  height: 100%;\n  display: block;\n}\n\
.blocks-testimonial-background-image-scrim {\n  position: absolute;\n  inset: 0;\n  background: color-mix(in srgb, var(--fandhe-color-fg) 70%, transparent);\n}\n\
.blocks-testimonial-background-image-panel {\n  width: 100%;\n  box-sizing: border-box;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  padding: var(--fandhe-space-6) var(--fandhe-space-4);\n  background: color-mix(in srgb, var(--fandhe-color-fg) 40%, transparent);\n}\n\
.blocks-testimonial-background-image-panel [data-scope=\"blockquote\"][data-part=\"root\"][data-blocks-testimonial-background-image-quote] {\n  border-inline-start: none;\n  padding-inline-start: 0;\n  --fandhe-blockquote-caption-fg: color-mix(in srgb, var(--fandhe-color-bg) 78%, transparent);\n}\n\
.blocks-testimonial-background-image-panel [data-scope=\"blockquote\"][data-part=\"content\"] {\n  font-size: var(--fandhe-font-font-size-xl);\n  line-height: 1.6;\n}\n\
.blocks-testimonial-background-image-author {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
[data-blocks-testimonial-background-image-name] {\n  font-weight: var(--fandhe-font-font-weight-medium);\n  color: var(--fandhe-color-bg);\n}\n\
@media (min-width: 48rem) {\n  .blocks-testimonial-background-image-root {\n    padding: var(--fandhe-space-16) var(--fandhe-space-6);\n  }\n  .blocks-testimonial-background-image-panel {\n    max-width: 40rem;\n    padding: var(--fandhe-space-10);\n    border-radius: var(--fandhe-radius-lg);\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// [`demo`] が背景画像・暗幕・ロゴ・引用文・著者名・役職を正しい
    /// 属性・文言で出力すること（`crate::blocks` モジュール doc の
    /// 不変条件）。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        assert!(html.contains(r#"data-scope="blockquote""#));
        assert!(html.contains(r#"data-scope="image""#));
        assert!(html.contains(r#"data-scope="icon""#));
        assert!(html.contains(r#"src="../../assets/blocks-demo-background.svg""#));
        assert!(html.contains(r#"alt="""#));
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(html.contains(r#"role="img""#));
        assert!(html.contains("aria-label="));
        assert!(html.contains("<blockquote"));
        assert!(html.contains("<figcaption"));
        for text_fragment in [
            dummy_assets::TESTIMONIAL_QUOTES[0],
            dummy_assets::PERSON_NAMES[0],
            dummy_assets::JOB_TITLES[0],
            dummy_assets::COMPANY_NAMES[0],
        ] {
            assert!(
                html.contains(text_fragment),
                "demo should contain {text_fragment}"
            );
        }
    }

    /// [`demo`] が `<form>`・不正リンク・`data:` URI・`<script` のいずれも
    /// 含まないこと（`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in [
            "<form",
            "type=\"submit\"",
            "href=\"#\"",
            "src=\"data:",
            "<script",
            "id=\"",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が全セレクタを宣言し、色リテラルではなく
    /// `color-mix()` + トークン参照で可読性を確保していること・`48rem`
    /// ブレークポイントを持つこと。
    #[test]
    fn layout_css_uses_tokens_and_breakpoint() {
        for selector in [
            ".blocks-testimonial-background-image {",
            ".blocks-testimonial-background-image-root {",
            ".blocks-testimonial-background-image-backdrop {",
            "[data-scope=\"image\"][data-part=\"root\"][data-blocks-testimonial-background-image-image] {",
            ".blocks-testimonial-background-image-scrim {",
            ".blocks-testimonial-background-image-panel {",
            ".blocks-testimonial-background-image-author {",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("color-mix("));
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-fg)"));
        assert!(LAYOUT_CSS.contains("var(--fandhe-color-bg)"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(!LAYOUT_CSS.contains("black"));
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// ルート class が出力に現れ、かつ `BLOCK.demo_class` とは異なること
    /// （`stats_background_image` と同じ教訓: 同一名だと `.blocks-demo`
    /// 側の共通ラッパクラスと衝突する）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains(ROOT_CLASS));
        assert_ne!(ROOT_CLASS, BLOCK.demo_class);
    }
}
