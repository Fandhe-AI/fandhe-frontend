# product-list-bordered-grid

`fandhe-frontend-pre-styled-ui` の `image` / `link` / `link-overlay` /
`rating-group` / `text` / `visually-hidden` の 6 部品を合成した、罫線で
仕切ったセル状の商品一覧グリッドの構成例です。Blocks セクションは新規
部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください（主参照は対応表 ID R1170。出典の
固有名・ファイル名は記載しません）。

各セルは画像を上に、下へ商品名・評価（星）・レビュー件数・価格を中央
寄せで積み、セル全体を 1 つのリンクにしています。セル同士は余白ではなく
罫線で接し、狭い幅では 2 列、広い幅では 4 列に並びます。データ・文言は
すべて架空のものであり、`<form>` 要素は一切持たず、無 JS の静的表示の
みを行います。

## Rust コード

```rust
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
```

## 差分メモ

- 主参照は対応表 ID R1170 です。参照元ファイルは本 worktree から参照
  できないため、Issue 本文の文章仕様（罫線区切り・中央寄せ・2〜4 列）
  に基づいて独自に再構成しています。
- レビュー件数はリンクにしていません。遷移先の Reviews block（Ecommerce
  / Reviews カテゴリ）は本 block 追加時点で未登録のため、実在しない
  リンク先（自己参照や `href="#"`）を作らず可視テキストとして表示して
  います。Reviews block が追加され次第、`link` への差し替えを検討して
  ください。
- 評価は `readonly` の静的表示で、初期状態から変化しません。評価ラベルは
  `visually-hidden` で視覚的に隠し、読み上げ専用の「5 段階中 n」を
  スクリーンリーダーへ伝えます。
- 商品数は 8 件です。2 列でも 4 列でも端数が出ない件数として選びました。
  3 列段は設けていません。
- 画像は全セルで同じ同梱 SVG プレースホルダーを使い回しています。

関連情報: [Image](../themes/image.md) / [Link](../themes/link.md) /
[Link Overlay](../themes/link-overlay.md) /
[Rating Group](../themes/rating-group.md) / [Text](../themes/text.md) /
[Visually Hidden](../themes/visually-hidden.md)
