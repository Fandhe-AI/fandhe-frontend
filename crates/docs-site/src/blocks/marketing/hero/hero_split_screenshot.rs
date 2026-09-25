//! `hero-split-screenshot` block（イシュー #2792。親トラッキング #2731
//! 「Blocks 目的別パーツ拡充ツリー」配下）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `button` / `image` / `code` / `link` の
//! 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 3 形を 1 つの Demo へ並べる
//!
//! `feature_split_screenshot`（イシュー #2770）と同じく、集約元 3 件の差分を
//! 別々の block へ分けず、縦に並ぶ 3 つのセクションとして 1 つの Demo に
//! 収める。DOM 順はどのセクションでもテキスト列 → メディア列に固定する。
//!
//! - **A（基準形）**: テキスト列が左、画像列が右。画像は列幅を超えて右へ
//!   はみ出す。
//! - **B（余白付き枠）**: 画像を枠（padding・border・角丸）で包み、枠ごと
//!   右へはみ出す。
//! - **C（タブ付きコード枠）**: 右列は画像の代わりにタブ付きのコード表示
//!   枠。
//!
//! # lg（64rem）以上でのみ 2 列化 + はみ出しを `overflow: hidden` で切り取る
//!
//! `feature_split_screenshot` モジュール doc「負の margin を採らず
//! `overflow: hidden` で切り取る理由」節と同じ判断: 各セクションのルートへ
//! `overflow: hidden` を付け、画像を列幅より大きく（`width: 48rem`）して
//! セクション境界で切り取られる形ではみ出しを表現する（負の margin は
//! 使わない）。lg 未満では画像を `width: 100%` に留め、テキストの下へ全幅
//! で置く。`@media` の条件式内ではテーマの breakpoint トークンが解決できな
//! いため、他 block と同じ判断でリテラル `64rem` を直書きする。
//!
//! # `tabs` 部品を使わない理由（C）
//!
//! `tabs` は [`BLOCK`] の `parts` に含まれず、JS ハイドレーションを行わ
//! ない docs サイトでは操作もできない。タブ列は素の `span` 2 つで表し、
//! 選択中タブは `data-blocks-hero-split-screenshot-tab-active` 属性で静的
//! に固定する。`role="tab"`/`aria-selected`/`id` は付けない（操作できない
//! 要素に操作可能な ARIA を付けると誤った案内になるため）。
//!
//! # 見出しレベルに `H3` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため、各セクションの見出しは
//! `HeadingLevel::H3` にする（他 block と同じ判断）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge` / `heading::heading` / `image::image` / `code::code` /
//! `button::button` / `link::root` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo 固有
//! のスタイルフックは `data-blocks-hero-split-screenshot-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する。素の `div`/`span`/`pre`
//! には `class` がそのまま効くため、それらは
//! `.blocks-hero-split-screenshot-*` クラスセレクタを使う。レイアウト root
//! の class（`blocks-hero-split-screenshot-layout`）は [`Block::demo_class`]
//! （`blocks-hero-split-screenshot`）とは意図的に別名にする
//! （`feature_split_screenshot` と同じ Bugbot 教訓の回避）。
//!
//! `styled_text::text`・`image::image`・`code::code` の recipe（詳細度
//! (0,2,0)）に確実に勝つため、上書きは `[data-scope="..."][data-part=
//! "root"][data-blocks-hero-split-screenshot-*]` の 3 セレクタ構成
//! （詳細度 (0,3,0)）で行う。`code` recipe の inline 表示は本 Demo では
//! ブロック表示へ切り替える。
//!
//! # `<form>` を持たない・データ取得/送信を行わない・id を使わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言はすべて架空のもの（実企業名・実クレデンシャル・PII・
//! トークン・URL・メールアドレスを含まない）。画像は
//! [`dummy_assets::SCREENSHOT_SRC`]（ビルド時生成のプレースホルダー SVG）
//! のみを使い、装飾扱いの `alt=""` で出力する。`id` 属性は一切使わない
//! （重複 id 検知テスト対策）。CTA の 2 つ目は `link::root` で表現し、href
//! はサイト内の実在ページへの相対パスのみを使う（`linkcheck` が検証する。
//! `data:`/`javascript:` は core の `is_safe_url` が拒否するため使わない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// `link::root` の href（サイト内の実在ページへの相対パス。`linkcheck` が
/// 検証する。`error_page_split_image` 等他 block と同じ判断）。
const DOCS_HREF: &str = "../../guides/";

/// テキスト列（eyebrow badge + 見出し + リード文 + CTA 2 個）。
fn copy_column(eyebrow: &str, title: &str, lead: &str) -> Node {
    div(
        vec![("class", "blocks-hero-split-screenshot-text")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-hero-split-screenshot-eyebrow", "")],
                vec![text(eyebrow)],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl4,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-hero-split-screenshot-lead", "")],
                vec![text(lead)],
            ),
            div(
                vec![("class", "blocks-hero-split-screenshot-actions")],
                vec![
                    button::button(&ButtonProps::default(), vec![], vec![text("今すぐ始める")]),
                    link::root(
                        DOCS_HREF,
                        &LinkProps {
                            variant: LinkVariant::Underline,
                            palette: ColorPalette::Neutral,
                            ..LinkProps::default()
                        },
                        vec![],
                        vec![text("ドキュメントを見る")],
                    ),
                ],
            ),
        ],
    )
}

/// アプリ画面のプレースホルダー画像（列幅より大きい固定幅で、
/// [`LAYOUT_CSS`] 側がセクション境界での切り取りを行う）。装飾扱いのため
/// `alt=""`。
fn screenshot() -> Node {
    image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio: AspectRatio::Auto,
            shape: ImageShape::Rounded,
            ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
        },
        vec![("data-blocks-hero-split-screenshot-image", "")],
    )
}

/// A: 基準形（テキスト列 → 画像列。画像は右へはみ出す）。
fn section_a() -> Node {
    div(
        vec![("class", "blocks-hero-split-screenshot-section")],
        vec![
            copy_column(
                "プラットフォーム",
                "画面のまま特長を伝える",
                "実際の操作画面をそのまま見せながら、主要な特長を紹介します。",
            ),
            div(
                vec![("class", "blocks-hero-split-screenshot-media")],
                vec![screenshot()],
            ),
        ],
    )
}

/// B: 余白付き枠（画像を枠で包み、枠ごと右へはみ出す）。
fn section_b() -> Node {
    div(
        vec![("class", "blocks-hero-split-screenshot-section")],
        vec![
            copy_column(
                "見せ方の一例",
                "余白を持たせて画面を見せる",
                "画像の周囲に余白と枠を持たせることで、より落ち着いた印象を与えます。",
            ),
            div(
                vec![("class", "blocks-hero-split-screenshot-media")],
                vec![div(
                    vec![("class", "blocks-hero-split-screenshot-frame")],
                    vec![screenshot()],
                )],
            ),
        ],
    )
}

/// C: タブ付きコード枠（テキスト列 → 開発者向けのコード表示枠）。
fn section_c() -> Node {
    let snippet = "let app = App::new();\napp.mount(\"#root\");\napp.run();\n";
    div(
        vec![("class", "blocks-hero-split-screenshot-section")],
        vec![
            copy_column(
                "開発者向け",
                "コードで組み込む",
                "既存のアプリへ数行を追加するだけで導入できます。",
            ),
            div(
                vec![("class", "blocks-hero-split-screenshot-media")],
                vec![div(
                    vec![("class", "blocks-hero-split-screenshot-code-frame")],
                    vec![
                        div(
                            vec![("class", "blocks-hero-split-screenshot-tabs")],
                            vec![
                                span(
                                    vec![("data-blocks-hero-split-screenshot-tab-active", "")],
                                    vec![text("main.rs")],
                                ),
                                span(vec![], vec![text("Cargo.toml")]),
                            ],
                        ),
                        el(
                            "pre",
                            vec![("class", "blocks-hero-split-screenshot-pre")],
                            vec![code::code(
                                &CodeProps::default(),
                                vec![("data-blocks-hero-split-screenshot-code", "")],
                                vec![text(snippet)],
                            )],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// `hero-split-screenshot` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ状態を持たない設計）。ルート class は
/// `demo_class`（`blocks-hero-split-screenshot`）とは別名にする。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-hero-split-screenshot-layout")],
        vec![section_a(), section_b(), section_c()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/hero-split-screenshot/",
    title: "hero-split-screenshot",
    category: BlockCategory::Hero,
    rust_source: "crates/docs-site/src/blocks/marketing/hero/hero_split_screenshot.rs",
    demo_class: "blocks-hero-split-screenshot",
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
        Part {
            label: "Code",
            path: "/themes/code/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `hero_split_screenshot` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-hero-split-screenshot-*` と
/// `[data-blocks-hero-split-screenshot-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない（`feature_split_screenshot` と同じ名前空間
/// 分離）。
///
/// lg（64rem）以上での固定幅 `48rem` は `.blocks-hero-split-screenshot-
/// media` の直接の子である画像（A: 枠なし）にのみ適用する
/// （`.blocks-hero-split-screenshot-media > [data-scope="image"]...`）。
/// B（枠あり）は画像が `.blocks-hero-split-screenshot-frame` の子になり
/// この子孫セレクタに一致しないため、画像は基準ルールの `width: 100%` の
/// まま枠の内側に収まる。両者へ無条件に `width: 48rem` を当てると、
/// `box-sizing: border-box` の枠が持つ `padding`/`border` の分だけ画像が
/// 枠の内側（余白）からはみ出し、`overflow: hidden` を持つセクション側で
/// 意図しない位置で切り取られる（PR #3243 レビュー指摘）。
const LAYOUT_CSS: &str = "\
.blocks-hero-split-screenshot-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
.blocks-hero-split-screenshot-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  overflow: hidden;\n}\n\
.blocks-hero-split-screenshot-text {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  min-width: 0;\n}\n\
.blocks-hero-split-screenshot-actions {\n  display: flex;\n  align-items: center;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-hero-split-screenshot-media {\n  min-width: 0;\n  display: flex;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-hero-split-screenshot-lead] {\n  margin: 0;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-hero-split-screenshot-image] {\n  display: block;\n  width: 100%;\n  flex: none;\n  border: 1px solid var(--fandhe-color-border);\n  box-shadow: var(--fandhe-shadow-lg);\n}\n\
.blocks-hero-split-screenshot-frame {\n  width: 100%;\n  padding: var(--fandhe-space-4);\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-muted);\n}\n\
.blocks-hero-split-screenshot-frame [data-scope=\"image\"][data-part=\"root\"][data-blocks-hero-split-screenshot-image] {\n  box-shadow: none;\n}\n\
.blocks-hero-split-screenshot-code-frame {\n  width: 100%;\n  background: var(--fandhe-color-fg);\n  color: var(--fandhe-color-bg);\n  border-radius: var(--fandhe-radius-lg);\n  overflow: hidden;\n}\n\
.blocks-hero-split-screenshot-tabs {\n  display: flex;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-3) var(--fandhe-space-4);\n  border-bottom: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-hero-split-screenshot-tabs span {\n  opacity: 0.6;\n}\n\
.blocks-hero-split-screenshot-tabs [data-blocks-hero-split-screenshot-tab-active] {\n  opacity: 1;\n  text-decoration: underline;\n}\n\
.blocks-hero-split-screenshot-pre {\n  margin: 0;\n  padding: var(--fandhe-space-6);\n  overflow-x: auto;\n}\n\
[data-scope=\"code\"][data-part=\"root\"][data-blocks-hero-split-screenshot-code] {\n  display: block;\n  white-space: pre;\n  background: transparent;\n  color: inherit;\n  border: 0;\n  padding: 0;\n}\n\
@media (min-width: 64rem) {\n  \
.blocks-hero-split-screenshot-section {\n    display: grid;\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    align-items: center;\n    column-gap: var(--fandhe-space-12);\n  }\n  \
.blocks-hero-split-screenshot-media > [data-scope=\"image\"][data-part=\"root\"][data-blocks-hero-split-screenshot-image] {\n    width: 48rem;\n    max-width: none;\n    flex: none;\n  }\n  \
.blocks-hero-split-screenshot-frame {\n    width: 48rem;\n    flex: none;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, dummy_assets, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体回帰
    /// （`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複し
    /// 過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"button\"",
            "data-scope=\"image\"",
            "data-scope=\"code\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(html.matches("<img").count(), 2);
        assert_eq!(
            html.matches("data-blocks-hero-split-screenshot-tab-active")
                .count(),
            1
        );
        assert!(html.contains(dummy_assets::SCREENSHOT_SRC));
        assert_eq!(html.matches("<button").count(), 3);
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("id=\""));
        assert!(!html.contains("role=\"tab\""));
        assert!(!html.contains("type=\"submit\""));
    }

    /// [`LAYOUT_CSS`] が想定する lg グリッド切り替え・切り取りのための
    /// 宣言を持つこと。
    #[test]
    fn layout_css_declares_lg_grid_and_overflow_clip() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("repeat(2, minmax(0, 1fr))"));
        assert!(LAYOUT_CSS.contains("overflow: hidden"));
        assert!(LAYOUT_CSS.contains("max-width: none"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「CSS フックの選び方」節の Bugbot 教訓の
    /// 固定、`feature_split_screenshot` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-hero-split-screenshot-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-hero-split-screenshot-layout"
        );
    }
}
