//! `promo-sale-products` block（イシュー #3084。親 #3024 配下、Ecommerce /
//! Promo カテゴリ。カテゴリ初出の `promo-collection-cards` に続く 2 件目）。
//!
//! # 出典に関する注記
//!
//! 主参照は対応表 ID R0039（集約元も R0039 の 1 件のみ、取得手段・
//! ファイル名は記載しない）。参照元の文言・配色・装飾・アイコンは
//! 持ち込まず、トークン配色と独自の架空文言で組み直している。
//!
//! # 使用部品
//!
//! `heading` / `text` / `button` / `image` / `card` / `strong` の 6 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約）。`strong` は
//! pre-styled-ui の `strong::strong`（`data-scope="strong"` の `<strong>`）を使う。
//!
//! # レイアウト（告知面 + 商品グリッドの 2 カラム）
//!
//! ルート（`.blocks-promo-sale-products-layout`）を `container-type:
//! inline-size` の container にし、その直下の子 `.blocks-promo-sale-products-columns`
//! が既定で `grid-template-columns: minmax(0,1fr) minmax(0,1.5fr)` を持ち、
//! 左に告知面、右に 2×2 の商品グリッドを置く。CSS Container Queries は
//! コンテナ自身のサイズクエリでコンテナ自身のプロパティを変更できない
//! 仕様のため、`container-name` を持つ要素と `grid-template-columns` を
//! 切り替える要素は別の要素にする必要がある（`cart_two_column_summary` の
//! `stack`/`columns` 分離と同型）。`@container`（`40rem` 未満）で
//! `.blocks-promo-sale-products-columns` と商品グリッドを 1 列に畳み、DOM 順
//! （告知 → グリッド）のまま縦積みにする（`promo_collection_cards` の sm
//! ブレークポイントと同じ判断軸だが、本 block はメディアクエリではなく
//! コンテナクエリで切り替える）。
//!
//! # 割引前価格の取り消し線に平文の「通常」を添える理由（a11y）
//!
//! `<s>`（打ち消し線）は読み上げで区別されにくいため、`pricing_seats_split`
//! と同じ判断で直前へ平文の「通常」を置き、視覚に依存しない形で割引前
//! 価格であることを伝える。セール価格は `strong` で強調する。
//!
//! # `drop_class_attr` を踏まえた CSS フックの選び方
//!
//! `heading::heading` / `styled_text::text` / `button::button` /
//! `card::root` / `image::image` はいずれも `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、本 block
//! 固有のフックは `data-blocks-promo-sale-products-*` 属性で渡す。素の
//! `div` のラッパー（レイアウト・告知面・グリッド・価格行）には `class`
//! がそのまま効くため、ルート class（`blocks-promo-sale-products-layout`）
//! は [`Block::demo_class`]（`blocks-promo-sale-products`）とは意図的に
//! 別名にする（`blog_list_image` 等と同じ Bugbot 教訓の回避）。
//!
//! # `<form>` を持たない・ボタンはリンクでも送信でもない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。CTA ボタンは `button::button` の既定 `type="button"` の
//! まま用い、`href="#"` の死リンクも作らない。商品名・価格はすべて架空の
//! ものであり、実企業名・実サービス名・実クレデンシャル・PII を含まない。
//! 画像は `dummy_assets::PRODUCT_SRC`（ビルド時生成のモノトーン
//! プレースホルダー SVG）を使い回す。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::strong;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};

/// 割引商品 4 点分の商品名（架空、実在のブランド・商品とは無関係）。
const PRODUCTS: [&str; 4] = [
    "リネンのトートバッグ",
    "陶器のマグカップ",
    "ウールのマフラー",
    "木製のコースター",
];

/// 割引商品 4 点分の元値（`PRODUCTS` と対で使う、架空の価格）。
const REGULAR_PRICES: [&str; 4] = ["¥4,800", "¥2,200", "¥6,000", "¥1,500"];

/// 割引商品 4 点分のセール価格（`PRODUCTS` と対で使う、架空の価格）。
const SALE_PRICES: [&str; 4] = ["¥3,360", "¥1,540", "¥4,200", "¥1,050"];

/// 告知面（見出し・本文・CTA 2 個）。
fn announcement() -> Node {
    div(
        vec![("class", "blocks-promo-sale-products-announcement")],
        vec![
            heading::heading(
                HeadingLevel::H2,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-promo-sale-products-title", "")],
                vec![text("季節のセール、開催中")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    ..TextProps::default()
                },
                vec![("data-blocks-promo-sale-products-lead", "")],
                vec![text(
                    "対象商品が最大 3 割引。数量限定のためお早めにご覧ください。",
                )],
            ),
            div(
                vec![("class", "blocks-promo-sale-products-actions")],
                vec![
                    button::button(
                        &ButtonProps::default(),
                        vec![("data-blocks-promo-sale-products-cta", "")],
                        vec![text("セール商品を見る")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![("data-blocks-promo-sale-products-cta-secondary", "")],
                        vec![text("カテゴリから探す")],
                    ),
                ],
            ),
        ],
    )
}

/// 割引商品カード 1 件（画像 + 商品名 + 価格行）。
fn product_card(name: &'static str, regular: &'static str, sale: &'static str) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-promo-sale-products-card", "")],
        vec![
            card::cover(
                vec![],
                vec![image::image(
                    &ImageProps {
                        aspect_ratio: AspectRatio::Square,
                        ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                    },
                    vec![("data-blocks-promo-sale-products-card-image", "")],
                )],
            ),
            card::body(
                vec![("class", "blocks-promo-sale-products-card-body")],
                vec![
                    card::title(
                        vec![("data-blocks-promo-sale-products-card-name", "")],
                        vec![text(name)],
                    ),
                    div(
                        vec![("class", "blocks-promo-sale-products-price-row")],
                        vec![
                            text("通常 "),
                            el("s", vec![], vec![text(regular)]),
                            strong::strong(
                                vec![("data-blocks-promo-sale-products-card-sale-price", "")],
                                vec![text(sale)],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// `promo-sale-products` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（告知面 + 商品グリッド 4 枚、モジュール冒頭「レイアウト」節参照）。
pub fn demo() -> Node {
    let cards: Vec<Node> = PRODUCTS
        .iter()
        .zip(REGULAR_PRICES)
        .zip(SALE_PRICES)
        .map(|((name, regular), sale)| product_card(name, regular, sale))
        .collect();
    div(
        vec![("class", "blocks-promo-sale-products-layout")],
        vec![div(
            vec![("class", "blocks-promo-sale-products-columns")],
            vec![
                announcement(),
                div(vec![("class", "blocks-promo-sale-products-grid")], cards),
            ],
        )],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/promo-sale-products/",
    title: "promo-sale-products",
    category: BlockCategory::Promo,
    rust_source: "crates/docs-site/src/blocks/ecommerce/promo/promo_sale_products.rs",
    demo_class: "blocks-promo-sale-products",
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
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Strong",
            path: "/themes/strong/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `promo_sale_products` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節、他 block と同型）。`--fandhe-*`
/// トークンのみを使い、生の色リテラルは置かない。
const LAYOUT_CSS: &str = "\
.blocks-promo-sale-products-layout {\n  container-type: inline-size;\n  container-name: blocks-promo-sale-products;\n}\n\
.blocks-promo-sale-products-columns {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr) minmax(0, 1.5fr);\n  gap: var(--fandhe-space-8);\n  align-items: start;\n}\n\
.blocks-promo-sale-products-announcement {\n  display: grid;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-promo-sale-products-actions {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-promo-sale-products-grid {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-promo-sale-products-card-body {\n  display: grid;\n  gap: var(--fandhe-space-2);\n  padding: var(--fandhe-space-4);\n}\n\
.blocks-promo-sale-products-price-row {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: baseline;\n  gap: var(--fandhe-space-2);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-promo-sale-products-card] {\n  overflow: hidden;\n  padding: 0;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-promo-sale-products-card-image] {\n  width: 100%;\n  display: block;\n}\n\
[data-blocks-promo-sale-products-card-sale-price] {\n  color: var(--fandhe-color-fg);\n}\n\
@container blocks-promo-sale-products (max-width: 40rem) {\n  .blocks-promo-sale-products-columns,\n  .blocks-promo-sale-products-grid {\n    grid-template-columns: minmax(0, 1fr);\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// [`demo`] が告知面（見出し 1・CTA 2）と商品カード 4 枚（取り消し線・
    /// 強調セール価格込み）を構成し、禁止パターン（`<form>`/`href="#"`/
    /// `data:` URI 等）を含まないこと（`crate::blocks` モジュール doc の
    /// 不変条件）。
    #[test]
    fn demo_renders_announcement_and_four_product_cards_and_avoids_disallowed_patterns() {
        let html = render(&demo());
        assert_eq!(
            html.matches("<h2").count(),
            1,
            "should render exactly 1 h2 heading"
        );
        assert_eq!(
            html.matches("<button").count(),
            2,
            "should render exactly 2 buttons"
        );
        assert_eq!(
            html.matches("<s>").count(),
            4,
            "should render exactly 4 strikethrough prices"
        );
        assert_eq!(
            html.matches("<strong").count(),
            4,
            "should render exactly 4 strong sale prices"
        );
        assert_eq!(
            html.matches("data-scope=\"strong\"").count(),
            4,
            "sale prices should use the pre-styled-ui Strong part"
        );
        assert_eq!(
            html.matches("data-blocks-promo-sale-products-card=\"\"")
                .count(),
            4,
            "should render exactly 4 product cards"
        );
        assert!(html.contains("blocks-demo-product.svg"));
        for hook in [
            "data-blocks-promo-sale-products-title",
            "data-blocks-promo-sale-products-lead",
            "data-blocks-promo-sale-products-cta",
            "data-blocks-promo-sale-products-cta-secondary",
            "data-blocks-promo-sale-products-card-image",
            "data-blocks-promo-sale-products-card-name",
            "data-blocks-promo-sale-products-card-sale-price",
        ] {
            assert!(
                html.contains(hook),
                "demo should render the {hook} attribute"
            );
        }
        // 商品名は直後のカードタイトルで読まれるため、画像は装飾扱い（空 alt）にして二重読み上げを避ける。
        assert_eq!(html.matches("alt=\"\"").count(), PRODUCTS.len());
        for name in PRODUCTS {
            assert!(
                !html.contains(&format!("alt=\"{name}\"")),
                "demo should not repeat {name} in alt text"
            );
        }
        for price in REGULAR_PRICES.iter().chain(SALE_PRICES.iter()) {
            assert!(html.contains(price), "demo should contain price {price}");
        }
        assert_eq!(
            html.matches("fd-button--variant-outline").count(),
            1,
            "should render exactly 1 outline (secondary) button"
        );
        assert!(!html.contains("type=\"submit\""));
        for absent in ["<form", "href=\"#\"", "src=\"data:", "<script"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] がコンテナクエリ・2 カラム・グリッド 2 列・トークン
    /// 使用の不変条件を満たすこと。
    #[test]
    fn layout_css_declares_container_query_and_grid_and_uses_tokens_not_literals() {
        for selector in [
            ".blocks-promo-sale-products-layout {",
            ".blocks-promo-sale-products-columns {",
            ".blocks-promo-sale-products-announcement {",
            ".blocks-promo-sale-products-actions {",
            ".blocks-promo-sale-products-grid {",
            ".blocks-promo-sale-products-card-body {",
            ".blocks-promo-sale-products-price-row {",
            "@container blocks-promo-sale-products (max-width: 40rem) {",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(LAYOUT_CSS.contains("repeat(2,"));
        assert!(LAYOUT_CSS.contains("flex-wrap: wrap;\n  align-items: baseline;"));
        // 狭いコンテナでは告知面との縦積みと同時に商品グリッドも 1 列へ畳む。
        assert!(LAYOUT_CSS.contains(
            "  .blocks-promo-sale-products-columns,\n  .blocks-promo-sale-products-grid {\n    grid-template-columns: minmax(0, 1fr);"
        ));
        assert!(LAYOUT_CSS.contains("--fandhe-space-"));
        assert!(LAYOUT_CSS.contains("--fandhe-color-fg"));
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("animation"));
        assert!(!LAYOUT_CSS.contains("transition"));
    }
}
