//! `product-list-simple-grid` block（イシュー #3065。親「Blocks 目的別パーツ
//! 拡充」ツリー〔Phase 5、親 #3024〕配下。商品をシンプルなグリッドに並べる、
//! shadcn/ui Blocks の product list セクション相当のレイアウトを、既存の
//! Themes 部品だけで合成した実例。取得元の文言・配色・装飾・アイコンは
//! 持ち込まず、文言・データはすべて架空のものを独自に書く。
//!
//! # 使用部品
//!
//! `heading` / `text` / `link` / `link_overlay` / `image` / `card` /
//! `visually_hidden` の 7 部品を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約）。新規 UI 部品は追加しない。
//!
//! # 3 インスタンスで示す差分・原稿メモに回した差分
//!
//! Demo は静的な 3 インスタンスを縦に並べる。
//!
//! - **インスタンス A**（[`instance_a`]、主参照。見出し + 右側に一覧
//!   リンク、正方形画像、枠なしカード、2 → 4 列）: 見出しと一覧リンクは
//!   それぞれ独立した要素として 1 つだけ置き、`@container` の
//!   `grid-template-areas` 切り替えで配置（見出しの右/グリッドの下）だけを
//!   変える。DOM を複製しない。
//! - **インスタンス B**（[`instance_b`]、見出しを視覚的に隠す・縦長画像・
//!   価格を名称の横に並べる・補足文、1 → 2 → 3 列）: 見出しは
//!   [`fandhe_frontend_pre_styled_ui::visually_hidden::root`] を子に持つ
//!   `heading` にし、`<h3>` 自体から可視テキストを消す。
//! - **インスタンス C**（[`instance_c`]、枠付きカード・カテゴリ/説明付き・
//!   hover・フォーカス時に閲覧ラベルを表示、1 → 2 → 3 列）: 画像上に
//!   装飾ラベル（`aria-hidden="true"`）を重ね、カード（`link_overlay`
//!   root）の `:hover`/`:focus-within` で可視化する。
//!
//! 4 列で価格を名称の右・見出しなしで色名付き・縦長画像 3 列と CTA・枠付き
//! カードと説明全文・3/4 列でカテゴリ・説明の有無を変えた版は Demo に含め
//! ず、原稿の「原案差分メモ」節でコードを増やさず説明する。
//!
//! # `link_overlay` の使い方（`aria-label` 方式にした理由）
//!
//! `blog_grid_image` と同じ理由（同モジュール rustdoc参照）で、カード全体を
//! [`fandhe_frontend_pre_styled_ui::link_overlay::root`] で囲み、`overlay`
//! の子は空にして `aria-label` へ商品名を渡す。可視の商品名は通常フローの
//! `heading`（H4）として別に置く。
//!
//! # `link` をカード外に置く理由
//!
//! インスタンス A の「一覧を見る」（[`fandhe_frontend_pre_styled_ui::link::root`]）
//! は `card`/`link_overlay` の外側にのみ置く。`grid_list_contact_cards`
//! モジュール doc「CSS フックの選び方」節と同じ制約（入れ子リンクの前面化
//! が `SlotRecipe` の子孫セレクタ非対応で表現できない）のため、カード内へは
//! 置かない。
//!
//! # href の方針
//!
//! `href="#"` は使わない。全商品の `href` はサイト内に実在する索引ページへ
//! の相対パス（`../../themes/` 等）を指す（`linkcheck::check_links` が
//! fail-closed に検証する）。実際の利用時は自分の商品 URL へ差し替えること
//! を原稿側の導入文で明記する。
//!
//! # `@container` で列数を切り替える理由
//!
//! `grid_list_contact_cards` モジュール doc「`@container` で列数を切り替える
//! 理由」節と同じ判断（Demo 枠の幅はビューポート幅と一致しないため
//! `@media` ではなく `@container` を使う）。`container-type: inline-size;
//! container-name: blocks-product-list-simple-grid;` は 3 インスタンスの
//! 共通祖先である外側ラッパー（`.blocks-product-list-simple-grid`）へ宣言
//! する。`@container` のスタイル規則はコンテナ自身ではなく祖先コンテナを
//! 参照する子孫要素にしか適用されない仕様のため、各インスタンス要素自身に
//! 宣言すると自分自身への規則が一切適用されない不具合になる（レビュー
//! 指摘対応）。
//!
//! # hover・フォーカス時の閲覧ラベルを `aria-hidden` にする理由
//!
//! インスタンス C の画像上に重ねる「商品を見る」ラベルは装飾であり、
//! アクセシブルネームはカード全体を覆う `link_overlay::overlay` の
//! `aria-label`（商品名）が既に担っている。ラベルを `aria-hidden="true"` に
//! することで、支援技術には商品名だけが 1 回だけ読み上げられる
//! （`page_heading_avatar` モジュール doc と同じ判断軸）。可視状態の制御は
//! カード（`link_overlay` root）自身の `:hover`/`:focus-within` で行い、
//! ラベル要素自体にはフォーカス・ホバーを持たせない。
//!
//! # `<form>`・送信処理を持たないこと
//!
//! 本 Demo は静的な表示例であり、`<form>` 要素・検証・送信処理・データ
//! 整形・永続化は一切持たない（`docs/policy/intentional-non-adoption.md`
//! §3.25 の UI 部品責務境界どおり）。
//!
//! # `data-*` フックを選ぶ理由（`drop_class_attr` の契約）
//!
//! `heading::heading`/`text::text`/`card::root`/`link_overlay::root`/
//! `link::root`/`image::image`/`visually_hidden::root` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、これらへの CSS フックは `class` ではなく
//! `data-blocks-product-list-simple-grid-*` 属性で渡す。素の `div` と
//! `card::body`/`card::cover` には variant を持たない契約のため `class` が
//! そのまま効く。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
    // 商品名は隣接する見出しが可視テキストとして既に提供するため、画像は
    // 装飾として alt を空にする（`product_list_bordered_grid` と同じ判断。
    // レビュー指摘対応: プレースホルダー画像に実商品固有の alt を与えない）。
    let mut image_props = ImageProps::new(dummy_assets::PRODUCT_SRC, "");
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
    // 商品名は隣接する見出しが可視テキストとして既に提供するため、画像は
    // 装飾として alt を空にする（`product_list_bordered_grid` と同じ判断。
    // レビュー指摘対応: プレースホルダー画像に実商品固有の alt を与えない）。
    let mut image_props = ImageProps::new(dummy_assets::PRODUCT_SRC, "");
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
    // 商品名は隣接する見出しが可視テキストとして既に提供するため、画像は
    // 装飾として alt を空にする（`product_list_bordered_grid` と同じ判断。
    // レビュー指摘対応: プレースホルダー画像に実商品固有の alt を与えない）。
    let mut image_props = ImageProps::new(dummy_assets::PRODUCT_SRC, "");
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/product-list-simple-grid/",
    title: "product-list-simple-grid",
    category: BlockCategory::ProductList,
    rust_source: "crates/docs-site/src/blocks/ecommerce/product_list/product_list_simple_grid.rs",
    demo_class: "blocks-product-list-simple-grid",
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
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Link Overlay",
            path: "/themes/link-overlay/",
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
            label: "Visually Hidden",
            path: "/themes/visually-hidden/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `product_list_simple_grid` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で [`super::blocks`]
/// 経由で [`crate::blocks::stylesheet`] へ連結される）。
///
/// `[data-scope="link-overlay"][data-part="root"]
/// [data-blocks-product-list-simple-grid-item]` の 3 属性セレクタは
/// `blog_grid_image` モジュール doc「レビュー指摘対応（PR #3156）」節と同じ
/// 判断: `link-overlay` レシピの `root` base（属性セレクタ 2 つ、詳細度
/// `(0,2,0)`）が持つ `border-radius: inherit` に対し、単一属性セレクタでは
/// 詳細度で負けて適用されないため、属性 3 つに引き上げて確実に上書きする。
const LAYOUT_CSS: &str = "\
.blocks-product-list-simple-grid {\n  container-type: inline-size;\n  container-name: blocks-product-list-simple-grid;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-product-list-simple-grid-instance-a,\n.blocks-product-list-simple-grid-instance-b,\n.blocks-product-list-simple-grid-instance-c {\n  display: grid;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-product-list-simple-grid-instance-a {\n  grid-template-columns: 1fr;\n  grid-template-areas: \"head\" \"grid\" \"more\";\n}\n\
.blocks-product-list-simple-grid-head {\n  grid-area: head;\n}\n\
.blocks-product-list-simple-grid-more {\n  grid-area: more;\n}\n\
.blocks-product-list-simple-grid-grid-a {\n  grid-area: grid;\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-product-list-simple-grid-grid-b,\n.blocks-product-list-simple-grid-grid-c {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-product-list-simple-grid-instance-b > [data-scope=\"heading\"][data-part=\"root\"] {\n  display: contents;\n}\n\
.blocks-product-list-simple-grid-card {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  height: 100%;\n}\n\
.blocks-product-list-simple-grid-name-row {\n  display: flex;\n  align-items: baseline;\n  justify-content: space-between;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-product-list-simple-grid-cover {\n  position: relative;\n}\n\
.blocks-product-list-simple-grid-label {\n  position: absolute;\n  inset: 0;\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  background: rgba(0, 0, 0, 0.45);\n  color: #fff;\n  font-size: var(--fandhe-font-font-size-sm);\n  opacity: 0;\n}\n\
[data-blocks-product-list-simple-grid-item]:hover .blocks-product-list-simple-grid-label,\n[data-blocks-product-list-simple-grid-item]:focus-within .blocks-product-list-simple-grid-label {\n  opacity: 1;\n}\n\
[data-scope=\"link-overlay\"][data-part=\"root\"][data-blocks-product-list-simple-grid-item] {\n  display: flex;\n  flex-direction: column;\n  height: 100%;\n  border-radius: var(--fandhe-radius-lg);\n}\n\
@container blocks-product-list-simple-grid (min-width: 28rem) {\n  \
.blocks-product-list-simple-grid-grid-b,\n  .blocks-product-list-simple-grid-grid-c {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n\
}\n\
@container blocks-product-list-simple-grid (min-width: 34rem) {\n  \
.blocks-product-list-simple-grid-instance-a {\n    grid-template-columns: 1fr auto;\n    grid-template-areas: \"head more\" \"grid grid\";\n    align-items: end;\n  }\n  \
.blocks-product-list-simple-grid-grid-b,\n  .blocks-product-list-simple-grid-grid-c {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n\
}\n\
@container blocks-product-list-simple-grid (min-width: 40rem) {\n  \
.blocks-product-list-simple-grid-grid-a {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"link\"",
            "data-scope=\"link-overlay\"",
            "data-scope=\"image\"",
            "data-scope=\"card\"",
            "data-scope=\"visually-hidden\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn no_form_semantics_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn overlay_anchor_count_matches_product_count() {
        let html = demo_html();
        let count = html.matches("data-part=\"overlay\"").count();
        assert_eq!(
            count, 10,
            "4 + 3 + 3 products should each have one overlay anchor"
        );
    }

    #[test]
    fn hover_label_is_decorative() {
        let html = demo_html();
        assert!(html.contains("blocks-product-list-simple-grid-label"));
        assert!(html.contains(r#"aria-hidden="true""#));
    }

    #[test]
    fn visually_hidden_heading_is_present() {
        let html = demo_html();
        assert!(html.contains("data-scope=\"visually-hidden\""));
        assert!(html.contains("季節のおすすめ雑貨"));
    }

    #[test]
    fn uses_shared_dummy_assets() {
        let html = demo_html();
        assert!(html.contains(super::dummy_assets::PRODUCT_SRC));
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }

    #[test]
    fn ids_have_no_duplicates() {
        let html = demo_html();
        let mut ids: Vec<&str> = Vec::new();
        for chunk in html.split("id=\"").skip(1) {
            if let Some(end) = chunk.find('"') {
                ids.push(&chunk[..end]);
            }
        }
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(ids.len(), sorted.len(), "id が重複している: {ids:?}");
    }

    #[test]
    fn layout_css_is_safe_and_uses_container_queries() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@container blocks-product-list-simple-grid"));
        assert!(LAYOUT_CSS.contains("grid-template-areas"));
    }

    /// レビュー指摘対応（Codex P1 / Cursor Bugbot Medium）: `@container` は
    /// コンテナ自身ではなく祖先コンテナを参照する子孫要素にしか適用されない
    /// ため、`container-name` は 3 インスタンスの共通祖先（外側ラッパー）へ
    /// 宣言し、各インスタンス要素自身には宣言しないことを固定する。
    #[test]
    fn container_name_is_declared_on_outer_wrapper_only() {
        let wrapper_rule = LAYOUT_CSS
            .split('}')
            .find(|rule| {
                rule.trim_start()
                    .starts_with(".blocks-product-list-simple-grid {")
            })
            .expect("outer wrapper rule should exist");
        assert!(wrapper_rule.contains("container-name: blocks-product-list-simple-grid"));

        let instance_rule = LAYOUT_CSS
            .split('}')
            .find(|rule| {
                rule.contains(".blocks-product-list-simple-grid-instance-a")
                    && rule.contains("display: grid")
            })
            .expect("instance rule should exist");
        assert!(
            !instance_rule.contains("container-name"),
            "インスタンス要素自身に container-name を宣言すると @container 規則が適用されない"
        );
    }

    /// レビュー指摘対応（Cursor Bugbot Low）: インスタンス B の視覚的に隠した
    /// 見出し（`<h3>`）がグリッドアイテムとして残らないよう `display: contents`
    /// にし、高さゼロの空トラックに gap が適用されないことを固定する。
    #[test]
    fn hidden_heading_does_not_reserve_a_grid_track() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-product-list-simple-grid-instance-b > [data-scope=\"heading\"][data-part=\"root\"] {\n  display: contents;\n}"
        ));
    }
}
