//! `promo-collection-cards` block（イシュー #3078。親 #3077 を骨格 + 主要
//! 領域とカード hover/focus 等の仕上げへ 2 分割した前半。Ecommerce / Promo
//! カテゴリ最初の block であり、本ファイル追加に伴い雛形 `promo.rs` を
//! `promo/mod.rs` へディレクトリ化して卒業する、`docs/design/docs-site-
//! blocks-section.md` §18 参照）。仕上げ（カード hover/focus の視覚状態・
//! 補助行・secondary CTA・原稿の差分メモ）は兄弟 #3079 へ委ねる。
//!
//! # 出典に関する注記
//!
//! 主参照は対応表 ID R1199（集約元も R1199 の 1 件のみ、取得手段・
//! ファイル名は記載しない）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `button` / `card` / `image` / `link-overlay` の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # 暗幕は `color-mix` + トークンで作る（`hero_background_media` と同型）
//!
//! ヒーロー部の背景画像へ `--fandhe-color-fg` を `color-mix()` で半透明化
//! したスクリムを重ね、見出し・リード文は `--fandhe-color-bg` トークンで
//! 描く。色リテラル（`#`/`white` 等）は使わず、ライトテーマでは
//! 「暗幕 + 明るい文字」、ダークテーマでは前景/背景の意味が反転し
//! 「明るい幕 + 暗い文字」になる既知の挙動である（原稿の差分メモは
//! 分割後半 #3079 に委ねる）。
//!
//! # コレクションカードをヒーロー下端へ重ねる
//!
//! カードグリッドへ負の `margin-top` を与え、ヒーロー部の下端へ食い込ま
//! せる（`page_heading_cover` のアバター重なりと同型の判断）。
//!
//! # sm 未満では縦積み（`blog_overlay_cards` と同じ 1 列 → N 列反転）
//!
//! グリッドは既定 1 列（縦積み）、`40rem`（sm）以上で 3 列へ切り替える。
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できないため、
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Sm`（640px = 40rem）
//! と一致するリテラル値を [`LAYOUT_CSS`] へ直書きする（既存 block と同じ
//! 判断）。
//!
//! # カード全体をリンク化（`blog_overlay_cards` と同型）
//!
//! 各カードは `card::root` の内側に `link_overlay::root` を置き、画像 +
//! コレクション名を通常フローで積み、`link_overlay::overlay` でカード
//! 全体をクリック可能にする。`card::root` の `overflow: hidden`（画像の
//! 角丸クリップ用）により `link_overlay::overlay` 既定の
//! `FocusRingOffset::Outside` がカード境界でクリップされるため、
//! `blog_overlay_cards` と同じ是正（`outline-offset` を内側へ上書き）を
//! 適用する。
//!
//! # `drop_class_attr` を踏まえた CSS フックの選び方
//!
//! `heading::heading` / `styled_text::text` / `button::button` /
//! `card::root` / `image::image` / `link_overlay::root` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、本 block 固有のフックは `data-blocks-promo-collection-
//! cards-*` 属性で渡す。素の `div`/`section` と `card::body` には `class`
//! がそのまま効くため `.blocks-promo-collection-cards-*` クラスセレクタを
//! 使う。
//!
//! # リンク先の方針
//!
//! `blog_overlay_cards` 等の前例と同じく、外部の絶対 URL
//! `https://github.com/Fandhe-AI/fandhe-frontend` をカードのリンク先として
//! 使う。`href="#"` の死リンクは使わない。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。CTA ボタンは `button::button` の既定 `type="button"` の
//! まま用いる。コレクション名・文言はすべて架空のものであり、実企業名・
//! 実サービス名・実クレデンシャル・PII を含まない。画像は
//! `dummy_assets::BACKGROUND_SRC`/`PRODUCT_SRC`（ビルド時生成のモノトーン
//! プレースホルダー SVG）を使い回す。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, section, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};

/// リンク先の固定外部 URL（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// コレクションカード 3 件分の名前（架空、実在の企業・ブランドとは無関係）。
const COLLECTIONS: [&str; 3] = ["新作コレクション", "定番コレクション", "限定コレクション"];

/// ヒーロー部（背景画像 + 暗幕 + 見出し・リード文・CTA）。
fn hero() -> Node {
    let backdrop = div(
        vec![
            ("class", "blocks-promo-collection-cards-backdrop"),
            ("aria-hidden", "true"),
        ],
        vec![
            image::image(
                &ImageProps::new(dummy_assets::BACKGROUND_SRC, ""),
                vec![("data-blocks-promo-collection-cards-image", "")],
            ),
            div(
                vec![("class", "blocks-promo-collection-cards-scrim")],
                vec![],
            ),
        ],
    );
    let content = div(
        vec![("class", "blocks-promo-collection-cards-content")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-promo-collection-cards-title", "")],
                vec![text("新作コレクション、到着")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    ..TextProps::default()
                },
                vec![("data-blocks-promo-collection-cards-lead", "")],
                vec![text(
                    "季節ごとに入れ替わる 3 つのコレクションから選べます。",
                )],
            ),
            button::button(
                &ButtonProps::default(),
                vec![("data-blocks-promo-collection-cards-cta", "")],
                vec![text("すべて見る")],
            ),
        ],
    );
    section(
        vec![("class", "blocks-promo-collection-cards-hero")],
        vec![backdrop, content],
    )
}

/// コレクションカード 1 件（画像 + 名前、カード全体をリンク化）。
fn collection_card(name: &'static str) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-promo-collection-cards-card", "")],
        vec![link_overlay::root(
            vec![("data-blocks-promo-collection-cards-link", "")],
            vec![
                image::image(
                    &ImageProps {
                        fit: ImageFit::Cover,
                        ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                    },
                    vec![("data-blocks-promo-collection-cards-card-image", "")],
                ),
                card::body(
                    vec![("class", "blocks-promo-collection-cards-card-body")],
                    vec![heading::heading(
                        HeadingLevel::H4,
                        &HeadingProps {
                            size: HeadingSize::Md,
                            ..HeadingProps::default()
                        },
                        vec![("data-blocks-promo-collection-cards-card-name", "")],
                        vec![text(name)],
                    )],
                ),
                overlay(
                    REPO,
                    vec![
                        ("aria-label", name),
                        ("data-blocks-promo-collection-cards-overlay", ""),
                    ],
                    vec![],
                ),
            ],
        )],
    )
}

/// `promo-collection-cards` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール冒頭「コレクションカードをヒーロー下端へ重ねる」
/// 節参照）。
pub fn demo() -> Node {
    let cards: Vec<Node> = COLLECTIONS
        .iter()
        .map(|name| collection_card(name))
        .collect();
    div(
        vec![("class", "blocks-promo-collection-cards-layout")],
        vec![
            hero(),
            div(vec![("class", "blocks-promo-collection-cards-grid")], cards),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/promo-collection-cards/",
    title: "promo-collection-cards",
    category: BlockCategory::Promo,
    rust_source: "crates/docs-site/src/blocks/ecommerce/promo/promo_collection_cards.rs",
    demo_class: "blocks-promo-collection-cards",
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
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Link Overlay",
            path: "/themes/link-overlay/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `promo_collection_cards` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節、他 block と同型）。
///
/// 生の色リテラルは使わず、可読性の確保はすべて `--fandhe-color-*`
/// トークンと `color-mix()` で行う（モジュール冒頭「暗幕は `color-mix` +
/// トークンで作る」節）。ブレークポイントは
/// [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Sm`]（640px =
/// 40rem）と一致するリテラル値を直書きする。
const LAYOUT_CSS: &str = "\
.blocks-promo-collection-cards-layout {\n  display: grid;\n}\n\
.blocks-promo-collection-cards-hero {\n  position: relative;\n  isolation: isolate;\n  overflow: hidden;\n  display: grid;\n  padding: var(--fandhe-space-16) var(--fandhe-space-6) calc(var(--fandhe-space-16) + 4rem);\n  border-radius: var(--fandhe-radius-lg);\n  color: var(--fandhe-color-bg);\n}\n\
.blocks-promo-collection-cards-backdrop {\n  position: absolute;\n  inset: 0;\n  z-index: -1;\n  overflow: hidden;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-promo-collection-cards-image] {\n  width: 100%;\n  height: 100%;\n  display: block;\n}\n\
.blocks-promo-collection-cards-scrim {\n  position: absolute;\n  inset: 0;\n  background: color-mix(in srgb, var(--fandhe-color-fg) 64%, transparent);\n}\n\
.blocks-promo-collection-cards-content {\n  display: grid;\n  gap: var(--fandhe-space-4);\n  max-width: 40rem;\n}\n\
[data-scope=\"heading\"][data-part=\"root\"][data-blocks-promo-collection-cards-title],\n[data-scope=\"text\"][data-part=\"root\"][data-blocks-promo-collection-cards-lead] {\n  color: inherit;\n}\n\
.blocks-promo-collection-cards-grid {\n  position: relative;\n  z-index: 1;\n  display: grid;\n  gap: var(--fandhe-space-4);\n  margin-top: -4rem;\n  padding-inline: var(--fandhe-space-6);\n}\n\
@media (min-width: 40rem) {\n  .blocks-promo-collection-cards-grid {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-promo-collection-cards-card] {\n  overflow: hidden;\n  padding: 0;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-promo-collection-cards-card-image] {\n  width: 100%;\n  display: block;\n}\n\
.blocks-promo-collection-cards-card-body {\n  padding: var(--fandhe-space-4);\n}\n\
[data-scope=\"link-overlay\"][data-part=\"overlay\"][data-blocks-promo-collection-cards-overlay]:focus-visible {\n  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// [`demo`] がヒーロー + カード 3 枚を構成し、各カードが `link-overlay`
    /// でリンク化され、禁止パターン（`<form>`/`href="#"`/`data:` URI 等）を
    /// 含まないこと（`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_renders_hero_and_three_linked_cards_and_avoids_disallowed_patterns() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-promo-collection-cards-link=\"\"")
                .count(),
            3,
            "should render exactly 3 link-overlay roots (one per card)"
        );
        assert!(html.contains(r#"data-scope="link-overlay""#));
        assert_eq!(
            html.matches("data-blocks-promo-collection-cards-card=\"\"")
                .count(),
            3,
            "should render exactly 3 collection cards"
        );
        assert!(html.contains(dummy_assets::BACKGROUND_SRC));
        assert!(html.contains(dummy_assets::PRODUCT_SRC));
        for hook in [
            "data-blocks-promo-collection-cards-image",
            "data-blocks-promo-collection-cards-title",
            "data-blocks-promo-collection-cards-lead",
            "data-blocks-promo-collection-cards-cta",
            "data-blocks-promo-collection-cards-link",
            "data-blocks-promo-collection-cards-card-image",
            "data-blocks-promo-collection-cards-card-name",
            "data-blocks-promo-collection-cards-overlay",
        ] {
            assert!(
                html.contains(hook),
                "demo should render the {hook} attribute"
            );
        }
        for name in COLLECTIONS {
            assert!(
                html.contains(name),
                "demo should contain collection name {name}"
            );
        }
        for absent in ["<form", "href=\"#\"", "src=\"data:", "<script", "id=\""] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が全主要セレクタを宣言し、`sm`（40rem）境界で 3 列へ
    /// 切り替わり、重なり配置・フォーカスリング内側化・トークン使用の
    /// 不変条件を満たすこと。
    #[test]
    fn layout_css_declares_all_selectors_and_uses_tokens_not_literals() {
        for selector in [
            ".blocks-promo-collection-cards-layout {",
            ".blocks-promo-collection-cards-hero {",
            ".blocks-promo-collection-cards-backdrop {",
            ".blocks-promo-collection-cards-scrim {",
            ".blocks-promo-collection-cards-content {",
            ".blocks-promo-collection-cards-grid {",
            "@media (min-width: 40rem) {",
            "[data-scope=\"card\"][data-part=\"root\"][data-blocks-promo-collection-cards-card] {",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(LAYOUT_CSS.contains("margin-top: -"));
        assert!(LAYOUT_CSS.contains("color-mix("));
        assert!(LAYOUT_CSS.contains("outline-offset: calc(-1"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(!LAYOUT_CSS.contains("animation"));
        assert!(!LAYOUT_CSS.contains("transition"));
    }
}
