//! `category-grid-captioned` block（イシュー #3037。親 #3024「Phase 5:
//! Blocks EC」配下。Ecommerce / Category カテゴリ最初の block であり、
//! 本ファイル追加に伴い雛形 `category_listing.rs` を
//! `category_listing/mod.rs` へディレクトリ化して卒業する、
//! `docs/design/docs-site-blocks-section.md` §18 参照）。
//!
//! カテゴリ一覧（画像の下に名称と説明）を、既存の Themes 部品だけで合成
//! した実例。取得元の文言・配色・装飾は持ち込まず、文言・データはすべて
//! 架空のものを独自に書く。
//!
//! # 使用部品
//!
//! `heading` / `text` / `link` / `link_overlay` / `image` の 5 部品を合成
//! する（[`BLOCK`] の `parts` に一致させる契約）。`card` は使わない。
//!
//! # 2 インスタンスで示す差分（対応表 ID の集約元 5 件を 2 件へ畳み込む）
//!
//! Demo は静的な 2 インスタンスを縦に並べる（主参照は対応表 ID
//! R0824「見出しの下に説明段落」）。
//!
//! - **インスタンス A**（R0824 + R0822「3 列、名称と説明」相当）: 左寄せの
//!   見出し・説明段落・「すべてのカテゴリを見る」リンクを導入部に置き、
//!   角丸矩形画像（[`ImageShape::Rounded`]）のカードを 6 枚、2〜3 列で
//!   並べる。各カードは画像の下に名称（H4）と短い説明を持つ。
//! - **インスタンス B**（R0609「円形画像 6 枚、中央見出し」+ R0040「中央
//!   見出し、正方形 4 枚、名称のみ」相当）: 中央寄せの見出しのみを導入部に
//!   置き、真円画像（[`ImageShape::Circle`]）のカードを 6 枚、2〜6 列で
//!   並べる。各カードは名称のみで説明は持たない。
//!
//! R0607「縦長画像と下段テキスト」は Demo に並記せず、原稿の「差分メモ」
//! 節で `AspectRatio::Portrait` への差し替えとして説明する（インスタンスを
//! 3 つにすると冗長になるため）。
//!
//! # `link_overlay` の使い方（`blog_grid_image`/`promo_collection_cards`
//! と同型）
//!
//! カード全体をクリック領域にするため、各カードは
//! [`fandhe_frontend_pre_styled_ui::link_overlay::root`] で囲み、画像・
//! 名称・（インスタンス A のみ）説明で `root` の高さを確立する。`overlay`
//! 自身は絶対配置で全面に広がるため可視テキストは入れず、`aria-label` に
//! カテゴリ名を渡してアクセシブルネームを与える。画像は装飾扱いで
//! `alt=""` にする（`promo_collection_cards` と同じ判断）。
//!
//! # リンク先の方針
//!
//! `promo_collection_cards`/`blog_grid_image` 等の前例と同じく、外部の
//! 絶対 URL `https://github.com/Fandhe-AI/fandhe-frontend` を各カード・
//! 「すべてのカテゴリを見る」リンクの両方の遷移先として使う。`href="#"`
//! の死リンクは使わない。
//!
//! # `drop_class_attr` を踏まえた CSS フックの選び方
//!
//! `heading::heading`/`text::text`/`image::image`/`link::root`/
//! `link_overlay::root` はいずれも `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、本 block 固有の
//! フックは `data-blocks-category-grid-captioned-*` 属性で渡す
//! （`promo_collection_cards` モジュール doc と同じ判断軸）。素の `div`
//! には `class` がそのまま効くため `.blocks-category-grid-captioned-*`
//! クラスセレクタを使う。
//!
//! # `[data-scope="link-overlay"][data-part="root"]` への 3 属性セレクタ
//!
//! `link_overlay` の `root` base（属性セレクタ 2 つ、`border-radius:
//! inherit` 宣言を持つ）に対し、単一属性セレクタの角丸上書きは詳細度で
//! 負ける（`blog_grid_image`/`banner_floating_card` と同じ教訓）。本
//! block のカードフック（`[data-blocks-category-grid-captioned-item]`）は
//! `[data-scope="link-overlay"][data-part="root"][data-blocks-...]` の
//! 3 属性セレクタで詳細度を揃えて角丸・hover 装飾を確実に適用する。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。カテゴリ名・説明文はすべて架空のものであり、実企業名・
//! 実サービス名・実クレデンシャル・PII を含まない。画像は
//! `dummy_assets::PRODUCT_SRC`（ビルド時生成のモノトーンプレースホルダー
//! SVG）を使い回す。状態を持たない静的な合成例である。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text as core_text, Node};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 遷移先の固定外部 URL（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 1 件分のカテゴリデータ（架空、実在の企業・ブランドとは無関係）。
struct Category {
    name: &'static str,
    /// インスタンス A のみで使う短い説明（インスタンス B では未参照）。
    description: &'static str,
}

/// インスタンス A（角丸・説明あり・6 件）のカテゴリデータ。
const CATEGORIES_A: [Category; 6] = [
    Category {
        name: "キッチン用品",
        description: "毎日の調理を助ける道具をそろえました。",
    },
    Category {
        name: "文房具",
        description: "書く・貼る・まとめるの基本を一式で。",
    },
    Category {
        name: "アウトドア",
        description: "週末の外出に役立つ装備を厳選。",
    },
    Category {
        name: "インテリア",
        description: "部屋の雰囲気を整える小物たち。",
    },
    Category {
        name: "ファッション小物",
        description: "コーディネートの仕上げに添える一品。",
    },
    Category {
        name: "ガーデニング",
        description: "育てる楽しさを支える道具一式。",
    },
];

/// インスタンス B（真円・名称のみ・6 件）のカテゴリデータ。
const CATEGORIES_B: [Category; 6] = [
    Category {
        name: "キッチン",
        description: "",
    },
    Category {
        name: "ステーショナリー",
        description: "",
    },
    Category {
        name: "アウトドア用品",
        description: "",
    },
    Category {
        name: "ホーム",
        description: "",
    },
    Category {
        name: "アクセサリー",
        description: "",
    },
    Category {
        name: "ガーデン",
        description: "",
    },
];

/// カテゴリカード 1 枚を組み立てる。`rounded` が `true` のとき角丸矩形
/// 画像 + 名称 + 説明（インスタンス A）、`false` のとき真円画像 + 名称の
/// み（インスタンス B）になる。
fn category_card(category: &Category, rounded: bool) -> Node {
    let mut image_props = ImageProps::new(dummy_assets::PRODUCT_SRC, "");
    image_props.aspect_ratio = AspectRatio::Square;
    image_props.shape = if rounded {
        ImageShape::Rounded
    } else {
        ImageShape::Circle
    };

    let name = heading::heading(
        HeadingLevel::H4,
        &HeadingProps {
            size: HeadingSize::Md,
            ..HeadingProps::default()
        },
        vec![("data-blocks-category-grid-captioned-name", "")],
        vec![core_text(category.name)],
    );

    let mut children = vec![image::image(&image_props, vec![]), name];
    if rounded {
        children.push(styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![("data-blocks-category-grid-captioned-description", "")],
            vec![core_text(category.description)],
        ));
    }
    children.push(overlay(
        REPO,
        vec![
            ("aria-label", category.name),
            ("data-blocks-category-grid-captioned-overlay", ""),
        ],
        vec![],
    ));

    link_overlay::root(
        vec![("data-blocks-category-grid-captioned-item", "")],
        children,
    )
}

/// インスタンス A の導入部（左寄せ見出し・説明段落・「すべてのカテゴリを
/// 見る」リンク）。
fn intro_a() -> Node {
    div(
        vec![("class", "blocks-category-grid-captioned-intro")],
        vec![
            div(
                vec![],
                vec![
                    heading::heading(
                        HeadingLevel::H3,
                        &HeadingProps::default(),
                        vec![("data-blocks-category-grid-captioned-heading", "")],
                        vec![core_text("人気のカテゴリ")],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Md,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![core_text(
                            "よく選ばれているカテゴリから、お探しの商品を見つけられます。",
                        )],
                    ),
                ],
            ),
            link::root(
                REPO,
                &LinkProps::default(),
                vec![],
                vec![core_text("すべてのカテゴリを見る")],
            ),
        ],
    )
}

/// インスタンス B の導入部（中央寄せ見出しのみ、説明は持たない）。
fn intro_b() -> Node {
    div(
        vec![
            ("class", "blocks-category-grid-captioned-intro"),
            ("data-align", "center"),
        ],
        vec![heading::heading(
            HeadingLevel::H3,
            &HeadingProps::default(),
            vec![("data-blocks-category-grid-captioned-heading", "")],
            vec![core_text("カテゴリから探す")],
        )],
    )
}

/// 1 インスタンス分（導入部 + グリッド）を組み立てる。`rounded` が `true`
/// のとき角丸矩形画像（インスタンス A）、`false` のとき真円画像
/// （インスタンス B）になる。
fn instance(intro: Node, categories: &[Category], rounded: bool) -> Node {
    let cards: Vec<Node> = categories
        .iter()
        .map(|category| category_card(category, rounded))
        .collect();
    let grid_class = if rounded {
        "blocks-category-grid-captioned-grid"
    } else {
        "blocks-category-grid-captioned-grid blocks-category-grid-captioned-grid-wide"
    };
    div(
        vec![("class", "blocks-category-grid-captioned-instance")],
        vec![intro, div(vec![("class", grid_class)], cards)],
    )
}

/// `category-grid-captioned` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「2 インスタンスで示す差分」節参照）。
#[must_use]
pub fn demo() -> Node {
    let note_a = styled_text::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(
            "左寄せの導入部・角丸矩形画像・名称と説明の構成例。",
        )],
    );
    let note_b = styled_text::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text("中央寄せの導入部・真円画像・名称のみの構成例。")],
    );

    div(
        vec![("class", "blocks-category-grid-captioned")],
        vec![
            note_a,
            instance(intro_a(), &CATEGORIES_A, true),
            note_b,
            instance(intro_b(), &CATEGORIES_B, false),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/category-grid-captioned/",
    title: "category-grid-captioned",
    category: BlockCategory::CategoryListing,
    rust_source:
        "crates/docs-site/src/blocks/ecommerce/category_listing/category_grid_captioned.rs",
    demo_class: "blocks-category-grid-captioned",
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
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `category_grid_captioned` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節、他 block と同型）。
///
/// ブレークポイントは [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::
/// Sm`]（640px = 40rem）・[`fandhe_frontend_pre_styled_ui::recipe::
/// Breakpoint::Lg`]（1024px = 64rem）と一致するリテラル値を直書きする
/// （`@media` 条件式の中ではテーマトークンを解決できないため、既存 block
/// と同じ判断）。
const LAYOUT_CSS: &str = "\
.blocks-category-grid-captioned {\n  display: flex;\n  flex-direction: column;\n  gap: 1.5rem;\n}\n\
.blocks-category-grid-captioned-instance {\n  display: flex;\n  flex-direction: column;\n  gap: 1.5rem;\n}\n\
.blocks-category-grid-captioned-intro {\n  display: flex;\n  align-items: flex-end;\n  justify-content: space-between;\n  flex-wrap: wrap;\n  gap: 1rem;\n}\n\
.blocks-category-grid-captioned-intro[data-align=\"center\"] {\n  justify-content: center;\n  text-align: center;\n}\n\
.blocks-category-grid-captioned-grid {\n  display: grid;\n  gap: var(--fandhe-space-6) var(--fandhe-space-4);\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n}\n\
@media (min-width: 40rem) {\n  .blocks-category-grid-captioned-grid {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n}\n\
@media (min-width: 64rem) {\n  .blocks-category-grid-captioned-grid-wide {\n    grid-template-columns: repeat(6, minmax(0, 1fr));\n  }\n}\n\
[data-scope=\"link-overlay\"][data-part=\"root\"][data-blocks-category-grid-captioned-item] {\n  display: grid;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"link-overlay\"][data-part=\"root\"][data-blocks-category-grid-captioned-item]:hover [data-blocks-category-grid-captioned-name] {\n  text-decoration: underline;\n}\n\
[data-scope=\"image\"][data-part=\"root\"] {\n  width: 100%;\n  display: block;\n}\n\
";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// [`demo`] がインスタンス A・B あわせて 12 枚のカードを `link-overlay`
    /// でリンク化し、画像形状・説明の有無が仕様どおりであり、禁止パターン
    /// （`<form>`/`href="#"`/`data:` URI/`<script`/`id="`/`type="submit"`）
    /// を含まないこと（`crate::blocks` モジュール doc の不変条件）。
    #[test]
    fn demo_renders_two_instances_with_linked_items_and_avoids_disallowed_patterns() {
        let html = render(&demo());

        assert_eq!(
            html.matches("data-blocks-category-grid-captioned-item=\"\"")
                .count(),
            12,
            "should render exactly 12 link-overlay roots (6 + 6 cards)"
        );
        assert!(html.contains(r#"data-scope="link-overlay""#));
        assert_eq!(
            html.matches("data-blocks-category-grid-captioned-overlay=\"\"")
                .count(),
            12,
            "should render exactly 12 overlay anchors"
        );
        assert_eq!(
            html.matches("fd-image--shape-circle").count(),
            6,
            "should render exactly 6 circle-shaped images (instance B)"
        );
        assert_eq!(
            html.matches("fd-image--shape-rounded").count(),
            6,
            "should render exactly 6 rounded-shaped images (instance A)"
        );
        assert_eq!(
            html.matches("data-blocks-category-grid-captioned-description=\"\"")
                .count(),
            6,
            "should render exactly 6 description hooks (instance A only)"
        );
        assert!(html.contains(r#"data-scope="link""#));

        for category in CATEGORIES_A.iter().chain(CATEGORIES_B.iter()) {
            assert!(
                html.contains(category.name),
                "demo should contain category name {}",
                category.name
            );
        }
        for category in &CATEGORIES_A {
            assert!(
                html.contains(category.description),
                "demo should contain category description {}",
                category.description
            );
        }

        for absent in [
            "<form",
            "href=\"#\"",
            "src=\"data:",
            "<script",
            "id=\"",
            "type=\"submit\"",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が主要セレクタ・ブレークポイント境界・トークン使用の
    /// 不変条件を満たすこと。
    #[test]
    fn layout_css_declares_selectors_and_uses_tokens_not_literals() {
        for selector in [
            ".blocks-category-grid-captioned {",
            ".blocks-category-grid-captioned-instance {",
            ".blocks-category-grid-captioned-intro {",
            ".blocks-category-grid-captioned-intro[data-align=\"center\"] {",
            ".blocks-category-grid-captioned-grid {",
            "@media (min-width: 40rem) {",
            "@media (min-width: 64rem) {",
            "[data-scope=\"link-overlay\"][data-part=\"root\"][data-blocks-category-grid-captioned-item] {",
        ] {
            assert!(
                LAYOUT_CSS.contains(selector),
                "LAYOUT_CSS should declare a rule for {selector}"
            );
        }
        assert!(LAYOUT_CSS.contains("--fandhe-space-"));
        assert!(!LAYOUT_CSS.contains('#'));
        assert!(!LAYOUT_CSS.contains("white"));
        assert!(!LAYOUT_CSS.contains("animation"));
        assert!(!LAYOUT_CSS.contains("transition"));
    }
}
