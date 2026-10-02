//! `product-list-bordered-grid` block。
//!
//! 罫線で仕切ったセル状の商品一覧グリッドを、既存の Themes 部品だけで
//! 合成した実例。取得元の文言・配色・装飾は持ち込まず、文言・データは
//! すべて架空のものを独自に書く（主参照は対応表 ID R1170）。
//!
//! # 使用部品
//!
//! `image` / `link` / `link-overlay` / `rating-group` / `text` /
//! `visually-hidden` の 6 部品を合成する（[`BLOCK`] の `parts` に
//! 一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。新しい UI 部品は追加しない。
//!
//! # セルは余白ではなく罫線で接する
//!
//! 外枠と区切り線を「コンテナに `border-top`/`border-left`、各セルに
//! `border-right`/`border-bottom`」の組み合わせで引く（[`LAYOUT_CSS`]
//! 参照）。`gap: 1px` に背景色を敷く方式は空セルにも背景が見えてしまう
//! ため採らない。列数は `@container`（コンテナ幅 48rem 境界）で
//! 狭幅 2 列・広幅 4 列を切り替える（Demo 枠の幅はビューポート幅と
//! 一致しないため、`product_overview_image_grid`/`cart_two_column_summary`
//! と同型のコンテナクエリ判定）。3 列段は商品 8 件に対して端数が出るため
//! 設けない。
//!
//! # `link_overlay` でセル全体をクリック領域にする
//!
//! 各セルは [`fandhe_frontend_pre_styled_ui::link_overlay::root`] で囲み、
//! 画像・商品名・評価行・価格で `root` の高さを確立する。`overlay` 自身は
//! 絶対配置で全面に広がるため可視テキストは入れず、`aria-label` に商品名
//! を渡してアクセシブルネームを与える（`category_grid_captioned` と同型）。
//! 画像は装飾扱いで `alt=""` にする。
//!
//! # `drop_class_attr` を踏まえた CSS フックの選び方
//!
//! `image::image`/`text::text`/`link::root`/`link_overlay::root`/
//! `visually_hidden::root`/`rating_group::root` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、本 block 固有のフックは `data-blocks-product-list-
//! bordered-grid-*` 属性で渡す。素の `div` には `class` がそのまま効くため
//! `.blocks-product-list-bordered-grid-*` クラスセレクタを使う。
//!
//! # `[data-scope="link-overlay"][data-part="root"]` への 3 属性セレクタ
//!
//! `link_overlay` の `root` base（属性セレクタ 2 つ）に対し、単一属性
//! セレクタの上書きは詳細度で負ける（`category_grid_captioned`/
//! `blog_grid_image` と同じ教訓）。本 block のカードフック
//! （`[data-blocks-product-list-bordered-grid-item]`）は
//! `[data-scope="link-overlay"][data-part="root"][data-blocks-...]` の
//! 3 属性セレクタで詳細度を揃える。
//!
//! # 評価のアクセシブルネーム（readonly 静的表示、重複しない id）
//!
//! `rating_group::label` を [`fandhe_frontend_pre_styled_ui::
//! visually_hidden::root`] で包み、「5 段階中 n」の読み上げ用ラベルを
//! 視覚的に隠す（label 要素自体は `control` の `aria-labelledby` 対象の
//! `id` を持ち続ける）。`id` はセルごとに一意（`blocks-product-list-
//! bordered-grid-rating-label-{index}`）にし、
//! `blocks_contract::demo_output_has_no_dangling_aria_references_or_
//! duplicate_ids` の重複 id 検査を満たす。評価は `RatingGroup::new(5,
//! Some(n), true)` の readonly 固定表示であり、状態を持たない。
//!
//! # レビュー件数はリンクにしない
//!
//! レビュー詳細ページ（Ecommerce / Reviews カテゴリ）は本イシュー時点で
//! block 未登録のため実在せず、リンク先を作ると自己参照（`href="./"`）に
//! なり遷移として機能しない（PR #3468 レビューでの既存判断、
//! `product_overview_image_grid` モジュール doc 参照）。本 block でも同じ
//! 判断を踏襲し、レビュー件数は [`fandhe_frontend_pre_styled_ui::text`]
//! の可視テキストとして表示する。実在の Reviews block が追加され次第、
//! その相対パスへの `link` へ差し替える。
//!
//! # リンク先の方針
//!
//! 他の ecommerce block と同じく、外部の絶対 URL
//! `https://github.com/Fandhe-AI/fandhe-frontend` を導入部の「すべての
//! 商品を見る」リンク・各セルの `overlay` の両方の遷移先として使う。
//! `href="#"` の死リンクは使わない。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。商品名・価格・レビュー件数はすべて架空のものであり、実
//! 企業名・実ブランド名・実クレデンシャル・PII を含まない。画像は
//! `dummy_assets::PRODUCT_SRC`（ビルド時生成のモノトーンプレースホルダー
//! SVG）を使い回す。状態を持たない静的な合成例である。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text as core_text, Node};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::visually_hidden;
use fandhe_frontend_pre_styled_ui::Size;

/// 遷移先の固定外部 URL（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 評価ラベル（`rating_group::label` の `id`）の接頭辞。セルごとに
/// `{PREFIX}-{index}` で一意にする（モジュール doc「評価のアクセシブル
/// ネーム」節参照）。
const RATING_LABEL_ID_PREFIX: &str = "blocks-product-list-bordered-grid-rating-label";

/// 1 件分の架空商品データ（実在の企業・ブランドとは無関係）。
struct Product {
    name: &'static str,
    price: &'static str,
    /// 5 段階評価の塗り数（1〜5）。
    rating: u32,
    /// 可視テキストで表示するレビュー件数。
    reviews: u32,
}

/// Demo に並べる架空の商品 8 件（2 列・4 列いずれでも端数が出ない件数）。
const PRODUCTS: [Product; 8] = [
    Product {
        name: "キャンバストートバッグ",
        price: "¥3,980",
        rating: 4,
        reviews: 56,
    },
    Product {
        name: "セラミックマグカップ",
        price: "¥1,980",
        rating: 5,
        reviews: 128,
    },
    Product {
        name: "ウールニットマフラー",
        price: "¥5,480",
        rating: 4,
        reviews: 34,
    },
    Product {
        name: "レザーカードケース",
        price: "¥4,280",
        rating: 3,
        reviews: 19,
    },
    Product {
        name: "アロマキャンドル",
        price: "¥2,480",
        rating: 5,
        reviews: 73,
    },
    Product {
        name: "ガラス製花瓶",
        price: "¥3,280",
        rating: 4,
        reviews: 41,
    },
    Product {
        name: "コットンクッションカバー",
        price: "¥2,180",
        rating: 4,
        reviews: 27,
    },
    Product {
        name: "木製コースター 4 枚セット",
        price: "¥1,680",
        rating: 5,
        reviews: 62,
    },
];

/// 評価（readonly の 5 段 `rating_group`）+ レビュー件数（可視テキスト）の
/// 行。評価ラベルは [`visually_hidden::root`] で視覚的に隠し、読み上げ
/// 専用の「5 段階中 n」を与える（モジュール doc「評価のアクセシブル
/// ネーム」節参照）。
fn rating_row(product: &Product, label_id: &str) -> Node {
    let props = RatingGroupProps {
        disabled: false,
        readonly: true,
        required: false,
    };
    let state = RatingGroup::new(5, Some(product.rating), true);
    let label = rating_group::label(
        &props,
        Some(label_id),
        vec![],
        vec![visually_hidden::root(
            vec![],
            vec![core_text(format!("5 段階中 {}", product.rating))],
        )],
    );
    let items: Vec<Node> = (1..=state.count())
        .map(|i| {
            rating_group::item(
                i,
                RatingItemFlags {
                    checked: state.is_checked(i),
                    highlighted: state.is_highlighted(i),
                    disabled: false,
                    readonly: true,
                },
                &format!("{i} star{}", if i == 1 { "" } else { "s" }),
                vec![],
                vec![],
            )
        })
        .collect();
    let control = rating_group::control(&props, Some(label_id), vec![], items);
    let rating = rating_group::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-product-list-bordered-grid-rating", "")],
        vec![label, control],
    );
    // レビュー詳細ページ（Ecommerce / Reviews カテゴリ）は本イシュー時点で
    // block 未登録のため実在しない。実在しないリンク先を作らず可視
    // テキストとして表示する（モジュール doc「レビュー件数はリンクに
    // しない」節参照）。
    let review_count = styled_text::text(
        &TextProps::default(),
        vec![],
        vec![core_text(format!("{} 件のレビュー", product.reviews))],
    );
    div(
        vec![("class", "blocks-product-list-bordered-grid-rating-row")],
        vec![rating, review_count],
    )
}

/// 商品カード 1 枚（画像 → 商品名 → 評価行 → 価格、`link_overlay` で全体を
/// クリック領域にする）を組み立てる。`label_id` は [`rating_row`] が使う
/// 評価ラベルの一意 `id`。
fn product_card(product: &Product, label_id: &str) -> Node {
    let image_props = ImageProps {
        aspect_ratio: AspectRatio::Square,
        ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
    };

    let name = styled_text::text(
        &TextProps {
            weight: TextWeight::Semibold,
            ..TextProps::default()
        },
        vec![("data-blocks-product-list-bordered-grid-name", "")],
        vec![core_text(product.name)],
    );

    let price = styled_text::text(
        &TextProps {
            weight: TextWeight::Semibold,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(product.price)],
    );

    link_overlay::root(
        vec![("data-blocks-product-list-bordered-grid-item", "")],
        vec![
            image::image(
                &image_props,
                vec![("data-blocks-product-list-bordered-grid-image", "")],
            ),
            name,
            rating_row(product, label_id),
            price,
            overlay(
                REPO,
                vec![
                    ("aria-label", product.name),
                    ("data-blocks-product-list-bordered-grid-overlay", ""),
                ],
                vec![],
            ),
        ],
    )
}

/// 導入行（見出し相当のテキスト + 「すべての商品を見る」リンク）。
fn intro() -> Node {
    div(
        vec![("class", "blocks-product-list-bordered-grid-intro")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    weight: TextWeight::Bold,
                    ..TextProps::default()
                },
                vec![],
                vec![core_text("新着アイテム")],
            ),
            link::root(
                REPO,
                &LinkProps::default(),
                vec![],
                vec![core_text("すべての商品を見る")],
            ),
        ],
    )
}

/// `product-list-bordered-grid` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。評価ラベルの `id` は商品の並び順で一意に確保する
/// （モジュール doc「評価のアクセシブルネーム」節参照）。
#[must_use]
pub fn demo() -> Node {
    let label_ids: Vec<String> = (0..PRODUCTS.len())
        .map(|i| format!("{RATING_LABEL_ID_PREFIX}-{i}"))
        .collect();
    let cards: Vec<Node> = PRODUCTS
        .iter()
        .zip(label_ids.iter())
        .map(|(product, label_id)| product_card(product, label_id))
        .collect();

    div(
        vec![("class", "blocks-product-list-bordered-grid")],
        vec![
            intro(),
            div(
                vec![("class", "blocks-product-list-bordered-grid-grid")],
                cards,
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/product-list-bordered-grid/",
    title: "product-list-bordered-grid",
    category: BlockCategory::ProductList,
    rust_source: "crates/docs-site/src/blocks/ecommerce/product_list/product_list_bordered_grid.rs",
    demo_class: "blocks-product-list-bordered-grid",
    parts: &[
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Link Overlay",
            path: "/themes/link-overlay/",
        },
        Part {
            label: "Rating Group",
            path: "/themes/rating-group/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `product_list_bordered_grid` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節、他 block と同型）。
///
/// # セル同士を罫線で接する（余白ではなく border）
///
/// コンテナに `border-top`/`border-left`、各セルに `border-right`/
/// `border-bottom` を宣言し、`gap: 0` のグリッドへ重ねることで二重線なしの
/// 格子罫線を作る（モジュール doc「セルは余白ではなく罫線で接する」節
/// 参照）。
///
/// # 列数は `@container` で切り替える
///
/// Demo 枠の幅はビューポート幅と一致しないため、コンテナクエリで判定する
/// （`product_overview_image_grid`/`cart_two_column_summary` と同型）。
/// 既定 2 列、コンテナ幅 48rem 以上で 4 列へ切り替える。3 列段は商品 8 件に
/// 対して端数が出るため設けない。コンテナクエリは自分自身のサイズを基準に
/// 自分自身を再スタイルできない（コンテナは子孫にのみ適用される）ため、
/// `container-type`/`container-name` は祖先の `.blocks-product-list-
/// bordered-grid` へ宣言し、`@container` では名前付きコンテナを介して
/// 子孫の `.blocks-product-list-bordered-grid-grid` を判定対象にする。
///
/// # `image::image` root への `width`/`display` 上書きは block 専用属性で限定する
///
/// `blocks::stylesheet()` は全 block の [`LAYOUT_CSS`] を連結して単一の
/// 共有 `assets/blocks.css` を生成する契約のため、`[data-scope="image"]
/// [data-part="root"]` を単独セレクタのまま上書きすると他 block の画像
/// 利用箇所へ波及してしまう（`category_grid_captioned` と同じ教訓）。
/// 本 block専用の `data-blocks-product-list-bordered-grid-image` 属性で
/// セレクタを限定する。
const LAYOUT_CSS: &str = "\
.blocks-product-list-bordered-grid {\n  display: flex;\n  flex-direction: column;\n  gap: 1.5rem;\n  container-type: inline-size;\n  container-name: blocks-product-list-bordered-grid;\n}\n\
.blocks-product-list-bordered-grid-intro {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  flex-wrap: wrap;\n  gap: 1rem;\n}\n\
.blocks-product-list-bordered-grid-grid {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: 0;\n  border-top: 1px solid var(--fandhe-color-border);\n  border-left: 1px solid var(--fandhe-color-border);\n}\n\
@container blocks-product-list-bordered-grid (min-width: 48rem) {\n  .blocks-product-list-bordered-grid-grid {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n}\n\
[data-scope=\"link-overlay\"][data-part=\"root\"][data-blocks-product-list-bordered-grid-item] {\n  display: grid;\n  justify-items: center;\n  text-align: center;\n  gap: var(--fandhe-space-2);\n  padding: var(--fandhe-space-4);\n  border-right: 1px solid var(--fandhe-color-border);\n  border-bottom: 1px solid var(--fandhe-color-border);\n  border-radius: 0;\n}\n\
[data-scope=\"link-overlay\"][data-part=\"root\"][data-blocks-product-list-bordered-grid-item]:hover [data-blocks-product-list-bordered-grid-name] {\n  text-decoration: underline;\n}\n\
.blocks-product-list-bordered-grid-rating-row {\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-product-list-bordered-grid-image] {\n  width: 100%;\n  display: block;\n}\n\
";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// [`demo`] が商品 8 件を `link-overlay` でリンク化し、評価・レビュー
    /// 件数・価格を表示し、禁止パターン
    /// （`<form>`/`href="#"`/`data:` URI/`<script`/`type="submit"`）を
    /// 含まないこと（`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_renders_eight_linked_products_and_avoids_disallowed_patterns() {
        let html = render(&demo());

        assert_eq!(
            html.matches("data-blocks-product-list-bordered-grid-item=\"\"")
                .count(),
            8,
            "should render exactly 8 link-overlay roots"
        );
        assert!(html.contains(r#"data-scope="link-overlay""#));
        assert_eq!(
            html.matches("data-blocks-product-list-bordered-grid-overlay=\"\"")
                .count(),
            8,
            "should render exactly 8 overlay anchors"
        );
        assert_eq!(
            html.matches(r#"data-scope="rating-group" data-part="root""#)
                .count(),
            8,
            "should render exactly 8 rating-group roots"
        );
        assert_eq!(
            html.matches(r#"data-scope="visually-hidden""#).count(),
            8,
            "should render exactly 8 visually-hidden labels"
        );
        assert!(html.contains(r#"data-scope="link""#));

        for product in &PRODUCTS {
            assert!(
                html.contains(product.name),
                "demo should contain product name {}",
                product.name
            );
            assert!(
                html.contains(product.price),
                "demo should contain product price {}",
                product.price
            );
        }

        for absent in [
            "<form",
            "href=\"#\"",
            "src=\"data:",
            "<script",
            "type=\"submit\"",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// 評価ラベルの `id` が商品数ぶん（8 件）すべて異なること
    /// （`blocks_contract::demo_output_has_no_dangling_aria_references_or_
    /// duplicate_ids` が検証する重複 id 禁止と整合）。
    #[test]
    fn rating_label_ids_are_unique_across_all_products() {
        let html = render(&demo());
        let ids: std::collections::HashSet<String> = (0..PRODUCTS.len())
            .map(|i| format!("id=\"{RATING_LABEL_ID_PREFIX}-{i}\""))
            .collect();
        assert_eq!(ids.len(), PRODUCTS.len());
        for id in &ids {
            assert_eq!(
                html.matches(id.as_str()).count(),
                1,
                "rating label id {id} should appear exactly once"
            );
        }
    }

    /// [`LAYOUT_CSS`] が主要セレクタ・コンテナクエリ境界・トークン使用の
    /// 不変条件を満たすこと。
    #[test]
    fn layout_css_declares_selectors_and_uses_tokens_not_literals() {
        for selector in [
            ".blocks-product-list-bordered-grid {",
            ".blocks-product-list-bordered-grid-intro {",
            ".blocks-product-list-bordered-grid-grid {",
            "@container blocks-product-list-bordered-grid (min-width: 48rem) {",
            "[data-scope=\"link-overlay\"][data-part=\"root\"][data-blocks-product-list-bordered-grid-item] {",
            "[data-scope=\"image\"][data-part=\"root\"][data-blocks-product-list-bordered-grid-image] {",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(LAYOUT_CSS.contains("--fandhe-color-border"));
        assert!(LAYOUT_CSS.contains("--fandhe-space-"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(!LAYOUT_CSS.contains("animation"));
        assert!(!LAYOUT_CSS.contains("transition"));
    }
}
