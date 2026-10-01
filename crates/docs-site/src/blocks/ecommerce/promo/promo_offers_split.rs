//! `promo-offers-split` block（イシュー #3082。Ecommerce / Promo カテゴリ、
//! `promo_collection_cards` に続く本カテゴリ 2 件目）。
//!
//! # 出典に関する注記
//!
//! 主参照は対応表 ID R1200（集約元も R1200 の 1 件のみ、取得手段・
//! ファイル名は記載しない）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `button` / `image` / `link` / `separator` の
//! 6 部品を合成する（[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # オファー見出しを `heading` ではなく `text` で表す理由
//!
//! 上段のオファー帯は 3 件のリンクを横に並べる構成である。各リンク内の
//! 「短い見出し + 説明」を `heading` 要素にすると、`## Demo` 自体が
//! `h2`・下段の本見出しが `h3` の文書構造の中へ `h4` 級の見出しが 3 つ
//! 連続して現れ、文書の見出し構造が騒がしくなる。本 block はオファーの
//! 短い見出しを `TextWeight::Semibold` の `text` で表し、`heading` は
//! 下段の 1 個（H3）のみに絞る。
//!
//! # 区切り線を横・縦の 2 個で切り替える理由
//!
//! `separator` は `role="separator"` と `aria-orientation` を常時出力し、
//! 装飾専用のオプションは持たない（`separator` モジュール rustdoc）。
//! 見た目だけを CSS で回転させると `aria-orientation` が表示と食い違う
//! ため、本 block は隣り合うオファーの間へ横向き・縦向きの区切り線を
//! それぞれ 1 個ずつ出力し、`display: none` で表示/非表示を切り替える
//! （`testimonial_two_up` と同じ手段。非表示側は支援技術からも除外される
//! ため、表示中の向きと aria は常に一致する）。
//!
//! # レスポンシブ（`64rem` のリテラルを直書きする理由）
//!
//! `pre-styled-ui` の breakpoint トークン
//! （[`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]、1024px =
//! 64rem）は `SlotRecipe` 経由の変数生成専用であり、docs-site の生 CSS
//! へ直接参照する経路を持たないため、一致するリテラル値を直書きする
//! （既存 block 全件と同じ判断）。既定（狭幅）はオファー帯を縦積み +
//! 横区切り線・下段を 1 列（テキスト → 画像の順で画像が下へ回る）、
//! `64rem` 以上でオファー帯を 3 列 + 縦区切り線・下段を 2 列へ切り替える。
//!
//! # `drop_class_attr` を踏まえた CSS フックの選び方
//!
//! `heading::heading` / `styled_text::text` / `button::button` /
//! `image::image` / `link::root` / `separator::separator` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、本 block 固有のフックは `data-blocks-promo-offers-
//! split-*` 属性で渡す。素の `div`/`section` には `class` がそのまま
//! 効くため `.blocks-promo-offers-split-*` クラスセレクタを使う。
//!
//! # リンク先の方針
//!
//! 既存 block と同じく、外部の絶対 URL
//! `https://github.com/Fandhe-AI/fandhe-frontend` をオファーリンク・CTA
//! ボタンの行き先として使う。`href="#"` の死リンクは使わない。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。CTA ボタンは `button::button` の既定 `type="button"` の
//! まま用いる。状態を持たない静的な表示のみで、送信処理・データ取得は
//! 一切行わない。オファー文言・リード文はすべて架空のものであり、実企業名・
//! 実サービス名・実クレデンシャル・PII を含まない。画像は
//! `dummy_assets::PRODUCT_SRC`（ビルド時生成のモノトーンプレースホルダー
//! SVG）を使い回す。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, section, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::Orientation;

/// リンク先の固定外部 URL（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// オファー帯 3 件分の（短い見出し, 説明）の組。架空の文言であり、実在の
/// キャンペーン条件を示すものではない。
const OFFERS: [(&str, &str); 3] = [
    ("送料無料", "全国どこでも配送料がかかりません"),
    ("30 日間返品可", "気に入らなければ全額返金いたします"),
    (
        "会員限定ポイント",
        "次回以降のお買い物に使えるポイントを進呈",
    ),
];

/// オファー帯の区切り線 1 組（横向き・縦向きを両方出力し、
/// `64rem` 境界で表示側を切り替える。モジュール冒頭「区切り線を横・縦の
/// 2 個で切り替える理由」節参照）。
fn offer_divider() -> Vec<Node> {
    vec![
        separator::separator(
            &SeparatorProps {
                orientation: Orientation::Horizontal,
                ..SeparatorProps::default()
            },
            vec![("data-blocks-promo-offers-split-divider", "horizontal")],
        ),
        separator::separator(
            &SeparatorProps {
                orientation: Orientation::Vertical,
                ..SeparatorProps::default()
            },
            vec![("data-blocks-promo-offers-split-divider", "vertical")],
        ),
    ]
}

/// オファー 1 件（短い見出し + 説明をまとめたリンク 1 個）。
fn offer(title: &'static str, description: &'static str) -> Node {
    link::root(
        REPO,
        &LinkProps::default(),
        vec![("data-blocks-promo-offers-split-offer", "")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    weight: TextWeight::Semibold,
                    ..TextProps::default()
                },
                vec![("data-blocks-promo-offers-split-offer-title", "")],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-promo-offers-split-offer-desc", "")],
                vec![text(description)],
            ),
        ],
    )
}

/// 上段のオファー帯（オファー 3 件 + 区切り線 2 組）。
fn offers_strip() -> Node {
    let mut children = Vec::new();
    for (index, (title, description)) in OFFERS.iter().enumerate() {
        if index > 0 {
            children.extend(offer_divider());
        }
        children.push(offer(title, description));
    }
    div(
        vec![("class", "blocks-promo-offers-split-offers")],
        children,
    )
}

/// 下段（左テキスト・右画像の 2 列）。
fn split() -> Node {
    section(
        vec![("class", "blocks-promo-offers-split-split")],
        vec![
            div(
                vec![("class", "blocks-promo-offers-split-content")],
                vec![
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl3,
                            ..HeadingProps::default()
                        },
                        vec![("data-blocks-promo-offers-split-title", "")],
                        vec![text("季節の特典をまとめてご案内")],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Lg,
                            ..TextProps::default()
                        },
                        vec![("data-blocks-promo-offers-split-lead", "")],
                        vec![text(
                            "対象商品のご購入で、上段 3 つの特典をすべて自動的に適用します。",
                        )],
                    ),
                    button::button(
                        &ButtonProps::default(),
                        vec![("data-blocks-promo-offers-split-cta", "")],
                        vec![text("対象商品を見る")],
                    ),
                ],
            ),
            image::image(
                &ImageProps::new(dummy_assets::PRODUCT_SRC, ""),
                vec![("data-blocks-promo-offers-split-image", "")],
            ),
        ],
    )
}

/// `promo-offers-split` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（他 block と同じ契約）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-promo-offers-split-layout")],
        vec![offers_strip(), split()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/promo-offers-split/",
    title: "promo-offers-split",
    category: BlockCategory::Promo,
    rust_source: "crates/docs-site/src/blocks/ecommerce/promo/promo_offers_split.rs",
    demo_class: "blocks-promo-offers-split",
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
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `promo_offers_split` 固有のレイアウト規則（`--fandhe-*` トークンのみ
/// 使用）。ブレークポイントは
/// [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]（1024px =
/// 64rem）と一致するリテラル値を直書きする（モジュール冒頭「レスポンシブ」
/// 節参照）。
const LAYOUT_CSS: &str = "\
.blocks-promo-offers-split-layout {\n  display: grid;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-promo-offers-split-offers {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-4);\n}\n\
[data-scope=\"separator\"][data-blocks-promo-offers-split-divider=\"vertical\"] {\n  display: none;\n}\n\
[data-scope=\"link\"][data-part=\"root\"][data-blocks-promo-offers-split-offer] {\n  display: grid;\n  gap: var(--fandhe-space-1);\n  padding: var(--fandhe-space-3);\n  border-radius: var(--fandhe-radius-md);\n  color: inherit;\n}\n\
.blocks-promo-offers-split-split {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-promo-offers-split-content {\n  display: grid;\n  gap: var(--fandhe-space-4);\n  align-content: start;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-promo-offers-split-image] {\n  width: 100%;\n  display: block;\n  border-radius: var(--fandhe-radius-lg);\n}\n\
@media (min-width: 64rem) {\n  \
.blocks-promo-offers-split-offers {\n    grid-template-columns: 1fr auto 1fr auto 1fr;\n    align-items: stretch;\n  }\n  \
[data-scope=\"separator\"][data-blocks-promo-offers-split-divider=\"horizontal\"] {\n    display: none;\n  }\n  \
[data-scope=\"separator\"][data-blocks-promo-offers-split-divider=\"vertical\"] {\n    display: block;\n  }\n  \
.blocks-promo-offers-split-split {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n    align-items: center;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// [`demo`] がオファー帯 3 件 + 区切り線 4 個（うち縦向き 2 個）+
    /// 下段 2 列を構成し、禁止パターン（`<form>`/`type="submit"`/
    /// `href="#"`/`data:` URI 等）を含まないこと（`crate::blocks` モジュール
    /// doc の不変条件）。
    #[test]
    fn demo_renders_offer_strip_and_split_and_avoids_disallowed_patterns() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-promo-offers-split-offer=\"\"")
                .count(),
            3,
            "should render exactly 3 offer links"
        );
        assert_eq!(
            html.matches(r#"data-scope="separator""#).count(),
            4,
            "should render exactly 4 separators (2 pairs)"
        );
        assert_eq!(
            html.matches(r#"aria-orientation="vertical""#).count(),
            2,
            "should render exactly 2 vertical separators"
        );
        for (title, description) in OFFERS {
            assert!(
                html.contains(title),
                "demo should contain offer title {title}"
            );
            assert!(
                html.contains(description),
                "demo should contain offer description {description}"
            );
        }
        assert!(html.contains(dummy_assets::PRODUCT_SRC));
        for hook in [
            "data-blocks-promo-offers-split-offer-title",
            "data-blocks-promo-offers-split-offer-desc",
            "data-blocks-promo-offers-split-title",
            "data-blocks-promo-offers-split-lead",
            "data-blocks-promo-offers-split-cta",
            "data-blocks-promo-offers-split-image",
        ] {
            assert!(
                html.contains(hook),
                "demo should render the {hook} attribute"
            );
        }
        assert!(!html.contains("type=\"submit\""));
        for absent in ["<form", "href=\"#\"", "src=\"data:", "<script", "id=\""] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が全主要セレクタ・`64rem` ブレークポイント・区切り線
    /// 切替・トークン使用の不変条件を満たすこと。
    #[test]
    fn layout_css_declares_all_selectors_and_uses_tokens_not_literals() {
        for selector in [
            ".blocks-promo-offers-split-layout {",
            ".blocks-promo-offers-split-offers {",
            "[data-scope=\"separator\"][data-blocks-promo-offers-split-divider=\"vertical\"] {",
            "[data-scope=\"link\"][data-part=\"root\"][data-blocks-promo-offers-split-offer] {",
            ".blocks-promo-offers-split-split {",
            ".blocks-promo-offers-split-content {",
            "[data-scope=\"image\"][data-part=\"root\"][data-blocks-promo-offers-split-image] {",
            "@media (min-width: 64rem) {",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(LAYOUT_CSS.contains("data-blocks-promo-offers-split-divider=\"vertical\""));
        assert!(LAYOUT_CSS.contains("data-blocks-promo-offers-split-divider=\"horizontal\""));
        assert!(LAYOUT_CSS.contains("var(--fandhe-"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(!LAYOUT_CSS.contains("animation"));
        assert!(!LAYOUT_CSS.contains("transition"));
    }

    /// ルート class が [`demo`] の出力へ実際に現れ、かつ `BLOCK.demo_class`
    /// とは異なること（既存 Blocks と同じ教訓: 同一名だと `.blocks-demo`
    /// 側の共通ラッパクラスと衝突する）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("blocks-promo-offers-split-layout"));
        assert_ne!("blocks-promo-offers-split-layout", BLOCK.demo_class);
    }
}
