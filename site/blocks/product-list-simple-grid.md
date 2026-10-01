# product-list-simple-grid

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `link` /
`link-overlay` / `image` / `card` / `visually-hidden` の 7 部品を合成した、
シンプルなグリッドの商品一覧です。Blocks セクションは新規部品を追加する
ものではなく、既存の Themes/Primitives 部品を組み合わせた実例集である
ことに注意してください。

商品データ・画像はすべて架空のもので、実在の企業・製品とは関係ありません。
画像は `crate::blocks::dummy_assets` が供給するプレースホルダー SVG（docs
サイト内部の素材ヘルパ）です。実際に利用する際は、自分の商品データ・
画像 URL・商品ページの URL へ置き換えてください。`<form>` 要素・検証・
送信処理・データ整形・永続化は一切持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text as core_text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay;
use fandhe_frontend_pre_styled_ui::text::{
    self as text_part, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// 1 件の架空商品データ（実企業名・PII を含まない）。インスタンスごとに
/// 使うフィールドが異なる（モジュール doc「3 インスタンスで示す差分」節）:
/// `note` はインスタンス A のみ、`category`/`description` はインスタンス
/// B・C のみで使う。
struct Product {
    name: &'static str,
    note: &'static str,
    price: &'static str,
    category: Option<&'static str>,
    description: Option<&'static str>,
    /// サイト内に実在する索引ページへの相対パス（モジュール doc「href の
    /// 方針」節参照）。
    href: &'static str,
}

/// インスタンス A（枠なし・正方形画像・2 → 4 列）用の 4 件。
const PRODUCTS_A: [Product; 4] = [
    Product {
        name: "方眼ノート B6",
        note: "色: チャコール",
        price: "¥480",
        category: None,
        description: None,
        href: "../../themes/",
    },
    Product {
        name: "真鍮クリップ 10 本組",
        note: "色: シルバー",
        price: "¥620",
        category: None,
        description: None,
        href: "../../primitives/",
    },
    Product {
        name: "帆布ペンケース",
        note: "色: オリーブ",
        price: "¥1,980",
        category: None,
        description: None,
        href: "../../examples/",
    },
    Product {
        name: "再生紙付箋セット",
        note: "色: ナチュラル",
        price: "¥360",
        category: None,
        description: None,
        href: "../../guides/",
    },
];

/// インスタンス B（縦長画像・価格を名称の横に並べる・3 件）用のデータ。
const PRODUCTS_B: [Product; 3] = [
    Product {
        name: "樫材デスクトレー",
        note: "",
        price: "¥3,200",
        category: None,
        description: Some("書類とペンをまとめて置ける、浅型の木製トレーです。"),
        href: "../../api/",
    },
    Product {
        name: "リネン巾着ポーチ",
        note: "",
        price: "¥1,450",
        category: None,
        description: Some("小物の持ち運びに使える、厚手リネン生地のポーチです。"),
        href: "../../wireframes/",
    },
    Product {
        name: "陶器製ペン立て",
        note: "",
        price: "¥2,100",
        category: None,
        description: Some("手作業で釉薬をかけた、1 点ごとに表情が異なるペン立てです。"),
        href: "../../themes/",
    },
];

/// インスタンス C（枠付きカード・カテゴリ/説明付き・hover ラベル・3 件）
/// 用のデータ。
const PRODUCTS_C: [Product; 3] = [
    Product {
        name: "刻印リングノート",
        note: "",
        price: "¥890",
        category: Some("文房具"),
        description: Some("表紙に名入れができる、リング式のノートです。"),
        href: "../../primitives/",
    },
    Product {
        name: "ウールひざ掛け",
        note: "",
        price: "¥4,600",
        category: Some("生活雑貨"),
        description: Some("オフィスでも使いやすい、落ち着いた色合いのひざ掛けです。"),
        href: "../../examples/",
    },
    Product {
        name: "ガラス製タンブラー",
        note: "",
        price: "¥1,320",
        category: Some("キッチン"),
        description: Some("手吹きガラスのゆらぎが心地よい、普段使い向けのタンブラーです。"),
        href: "../../guides/",
    },
];

/// インスタンス A の商品カード 1 件（枠なし・正方形画像）。
fn product_card_a(product: &Product) -> Node {
    let alt = format!("{}の商品画像", product.name);
    let mut image_props = ImageProps::new(dummy_assets::PRODUCT_SRC, &alt);
    image_props.aspect_ratio = AspectRatio::Square;
    let image_node = image::image(&image_props, vec![]);

    let name = heading::heading(
        HeadingLevel::H4,
        &HeadingProps {
            size: HeadingSize::Sm,
            weight: HeadingWeight::Semibold,
        },
        vec![],
        vec![core_text(product.name)],
    );
    let note = text_part::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(product.note)],
    );
    let price = text_part::text(
        &TextProps {
            size: TextSize::Sm,
            weight: TextWeight::Semibold,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(product.price)],
    );

    let content = div(
        vec![("class", "blocks-product-list-simple-grid-card")],
        vec![image_node, name, note, price],
    );

    link_overlay::root(
        vec![("data-blocks-product-list-simple-grid-item", "")],
        vec![
            content,
            link_overlay::overlay(product.href, vec![("aria-label", product.name)], vec![]),
        ],
    )
}

/// インスタンス B の商品カード 1 件（縦長画像・名称の横に価格・補足文）。
fn product_card_b(product: &Product) -> Node {
    let alt = format!("{}の商品画像", product.name);
    let mut image_props = ImageProps::new(dummy_assets::PRODUCT_SRC, &alt);
    image_props.aspect_ratio = AspectRatio::Portrait;
    image_props.shape = ImageShape::Rounded;
    let image_node = image::image(&image_props, vec![]);

    let name_row = div(
        vec![("class", "blocks-product-list-simple-grid-name-row")],
        vec![
            heading::heading(
                HeadingLevel::H4,
                &HeadingProps {
                    size: HeadingSize::Sm,
                    weight: HeadingWeight::Semibold,
                },
                vec![],
                vec![core_text(product.name)],
            ),
            text_part::text(
                &TextProps {
                    size: TextSize::Sm,
                    weight: TextWeight::Semibold,
                    ..TextProps::default()
                },
                vec![],
                vec![core_text(product.price)],
            ),
        ],
    );
    let description = text_part::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(product.description.unwrap_or_default())],
    );

    let content = div(
        vec![("class", "blocks-product-list-simple-grid-card")],
        vec![image_node, name_row, description],
    );

    link_overlay::root(
        vec![("data-blocks-product-list-simple-grid-item", "")],
        vec![
            content,
            link_overlay::overlay(product.href, vec![("aria-label", product.name)], vec![]),
        ],
    )
}

/// インスタンス C の商品カード 1 件（枠付き・hover/フォーカスで閲覧ラベル
/// 表示・カテゴリ/説明付き）。
fn product_card_c(product: &Product) -> Node {
    let alt = format!("{}の商品画像", product.name);
    let mut image_props = ImageProps::new(dummy_assets::PRODUCT_SRC, &alt);
    image_props.aspect_ratio = AspectRatio::Square;
    let image_node = image::image(&image_props, vec![]);

    // モジュール doc「hover・フォーカス時の閲覧ラベルを aria-hidden にする
    // 理由」節: アクセシブルネームは overlay の aria-label が既に担うため、
    // ラベル自体は装飾として読み上げから除外する。
    let label = div(
        vec![
            ("class", "blocks-product-list-simple-grid-label"),
            ("aria-hidden", "true"),
        ],
        vec![core_text("商品を見る")],
    );
    let cover = card::cover(
        vec![("class", "blocks-product-list-simple-grid-cover")],
        vec![image_node, label],
    );

    let mut body_children = vec![heading::heading(
        HeadingLevel::H4,
        &HeadingProps {
            size: HeadingSize::Sm,
            weight: HeadingWeight::Semibold,
        },
        vec![],
        vec![core_text(product.name)],
    )];
    if let Some(category) = product.category {
        body_children.push(text_part::text(
            &TextProps {
                size: TextSize::Xs,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![core_text(category)],
        ));
    }
    if let Some(description) = product.description {
        body_children.push(text_part::text(
            &TextProps {
                size: TextSize::Sm,
                ..TextProps::default()
            },
            vec![],
            vec![core_text(description)],
        ));
    }
    body_children.push(text_part::text(
        &TextProps {
            size: TextSize::Sm,
            weight: TextWeight::Semibold,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(product.price)],
    ));

    let card_node = card::root(
        CardProps::default(),
        vec![],
        vec![cover, card::body(vec![], body_children)],
    );

    link_overlay::root(
        vec![("data-blocks-product-list-simple-grid-item", "")],
        vec![
            card_node,
            link_overlay::overlay(product.href, vec![("aria-label", product.name)], vec![]),
        ],
    )
}

/// インスタンス A（主参照: 見出し + 右側に一覧リンク、2 → 4 列）。
fn instance_a() -> Node {
    let head = div(
        vec![("class", "blocks-product-list-simple-grid-head")],
        vec![heading::heading(
            HeadingLevel::H3,
            &HeadingProps::default(),
            vec![],
            vec![core_text("人気の文房具")],
        )],
    );
    let more = div(
        vec![("class", "blocks-product-list-simple-grid-more")],
        vec![link::root(
            "../../themes/",
            &LinkProps::default(),
            vec![],
            vec![core_text("一覧を見る")],
        )],
    );
    let cards: Vec<Node> = PRODUCTS_A.iter().map(product_card_a).collect();
    let grid = div(
        vec![("class", "blocks-product-list-simple-grid-grid-a")],
        cards,
    );

    div(
        vec![("class", "blocks-product-list-simple-grid-instance-a")],
        vec![head, more, grid],
    )
}

/// インスタンス B（見出しを視覚的に隠す、縦長画像、1 → 2 → 3 列）。
fn instance_b() -> Node {
    // モジュール doc「3 インスタンスで示す差分」節: heading タグ自体を
    // visually_hidden の子にし、見出し階層は保ったまま可視テキストを消す。
    let hidden_heading = heading::heading(
        HeadingLevel::H3,
        &HeadingProps::default(),
        vec![],
        vec![visually_hidden::root(
            vec![],
            vec![core_text("季節のおすすめ雑貨")],
        )],
    );
    let cards: Vec<Node> = PRODUCTS_B.iter().map(product_card_b).collect();
    let grid = div(
        vec![("class", "blocks-product-list-simple-grid-grid-b")],
        cards,
    );

    div(
        vec![("class", "blocks-product-list-simple-grid-instance-b")],
        vec![hidden_heading, grid],
    )
}

/// インスタンス C（枠付きカード・hover/フォーカスで閲覧ラベル、1 → 2 → 3
/// 列）。
fn instance_c() -> Node {
    let heading_node = heading::heading(
        HeadingLevel::H3,
        &HeadingProps::default(),
        vec![],
        vec![core_text("スタッフのおすすめ")],
    );
    let cards: Vec<Node> = PRODUCTS_C.iter().map(product_card_c).collect();
    let grid = div(
        vec![("class", "blocks-product-list-simple-grid-grid-c")],
        cards,
    );

    div(
        vec![("class", "blocks-product-list-simple-grid-instance-c")],
        vec![heading_node, grid],
    )
}

/// `product-list-simple-grid` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。3 インスタンス（モジュール doc「3 インスタンスで示す差分」
/// 節）を縦に並べた静的な表示のみを描く。
#[must_use]
pub fn demo() -> Node {
    let note_a = text_part::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(
            "見出しの右に一覧リンクを置き、2 列から 4 列へ広がる構成例。",
        )],
    );
    let note_b = text_part::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(
            "見出しを視覚的に隠し、縦長画像と横並びの価格を使う構成例。",
        )],
    );
    let note_c = text_part::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(
            "枠付きカードで hover・フォーカス時に閲覧ラベルを表示する構成例。",
        )],
    );

    div(
        vec![("class", "blocks-product-list-simple-grid")],
        vec![
            note_a,
            instance_a(),
            note_b,
            instance_b(),
            note_c,
            instance_c(),
        ],
    )
}
```

## 原案差分メモ

- 4 列で価格を名称の右に置く版: インスタンス A の `product_card_a` の
  レイアウトを `.blocks-product-list-simple-grid-card` の `flex-direction`
  のみ変更すれば表現できます（コードを増やさず CSS 調整で対応可能）。
- 縦長画像 3 列と CTA を添える版: インスタンス B の画像を
  `AspectRatio::Portrait` のまま列数を 3 固定にし、`instance_a` と同じ
  `.blocks-product-list-simple-grid-more` を追加すれば表現できます。
- 見出しなしで色名付きの版: インスタンス A から見出しブロックを省き、
  `note`（色名）フィールドをそのまま流用できます。
- 枠付きカードと説明全文を見せる版: インスタンス C の `description` を
  `text::text` の `size: TextSize::Md` にすれば全文表示へ切り替えられます
  （現在は要約表示として `Sm` を使用）。
- 4 列でカテゴリ・説明なしの版: インスタンス A の画像・レイアウトを流用し
  `PRODUCTS_A` を 8 件へ増やすだけで表現できます。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Link](../themes/link.md) / [Link Overlay](../themes/link-overlay.md) /
[Image](../themes/image.md) / [Card](../themes/card.md) /
[Visually Hidden](../themes/visually-hidden.md)
