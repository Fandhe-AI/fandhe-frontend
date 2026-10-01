//! `product-list-rich-cards` block（イシュー #3064。親トラッキング #3024。
//! 対応表 ID R0208（主参照）・R0209/R0210/R0211/R0212/R1167（集約元）を
//! 構造の参照元とする合成例。
//! 取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。
//!
//! # 使用部品
//!
//! `image` / `badge` / `card` / `link` / `text` / `rating-group` /
//! `color-swatch` / `button` / `icon` の 9 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。
//!
//! # レイアウト（2 バリアント）
//!
//! - **バリアント A（枠付きリッチカード）**: 3 枚。基本 1 列、コンテナ幅
//!   `>= 24rem` で 2 列、`>= 40rem` で 3 列。
//! - **バリアント B（密な配置・セール価格）**: 4 枚。基本 2 列、
//!   コンテナ幅 `>= 40rem` で 4 列。
//!
//! `display: none` は使わない（狭幅でも全操作要素へ到達可能なまま積む）。
//!
//! # 列数は `@container` で切り替える
//!
//! Demo 枠（`.docs-content`、最大 46rem。`.blocks-demo` の左右 padding
//! `1.5rem`×2 を差し引くと本コンテナの実効上限は約 43rem）の幅はビュー
//! ポート幅と一致しないため、`@media (min-width: ...)` ではなくコンテナ
//! クエリで判定する（`product_list_bordered_grid`/
//! `product_overview_image_grid` と同型）。境界値は実効上限 43rem 以下
//! （`24rem`/`40rem`）に収め、Demo 内で実際に切り替わる値を選ぶ
//! （`64rem` は Demo 幅のいかなる状態でも到達不能なため採らない）。
//! コンテナクエリは自分自身のサイズを基準に自分自身を再スタイルできない
//! （コンテナは子孫にのみ適用される）ため、`container-type`/
//! `container-name` は祖先の `.blocks-product-list-rich-cards-layout` へ
//! 宣言し、`@container` では名前付きコンテナを介して子孫の
//! `.blocks-product-list-rich-cards-grid-a`/`-grid-b` を判定対象にする。
//!
//! # カード共通構成
//!
//! 画像（角にバッジ・お気に入りボタンを重ねる） → 商品名（リンク） →
//! 説明 → 評価（readonly `rating-group`、件数込みラベル） → 色見本
//! （`color-swatch` ×N、`aria-hidden` + 可視テキストで色数を明文化） →
//! 価格（セール時は現在価格 + 取り消し線の元値） → カート追加ボタン
//! （全幅、`disabled` 固定）の順。
//!
//! # お気に入りボタンを 1 枚だけ「お気に入り済み」で固定する理由
//!
//! `feed_upvote_cards.rs::vote_column` と同じ判断で、無 JS の静的表示の
//! ため実際の切替は行わない。7 枚中 1 枚のみ `aria-pressed="true"` +
//! ハート塗りつぶしアイコンに固定し、残りは `aria-pressed="false"` +
//! 輪郭アイコンに固定する。`product_overview_gallery_split.rs`「購入ボタン
//! は disabled の静的表示」節と同じ判断で `disabled` を付け、無 JS のため
//! 押しても表示が変わらないボタンを操作可能なまま残さない
//! （`aria-pressed` 自体は状態表示として `disabled` と併存させる）。
//! ラベルは `format!("「{name}」のお気に入り")` で商品ごとに一意にし、
//! `aria-pressed` の状態に依存しない文言にする（`favorite: true` でも
//! 「追加」のまま固定され矛盾した案内になることを避けるため）。
//!
//! # カート追加ボタンは disabled の静的表示
//!
//! `product_overview_gallery_split.rs`「購入ボタンは disabled の静的表示」
//! 節と同じ判断。無 JS のため押しても何も起きないボタンを操作可能なまま
//! 残さない。
//!
//! # 評価を `rating-group`（readonly）+ 件数込みラベルで表現する理由
//!
//! `product_overview_gallery_split.rs::rating_row` と同型。現在の評価と
//! 件数をラベルで明文化し、ラベル `id` はカードごとに一意にする
//! （`crates/docs-site/tests/blocks_contract.rs` の重複 id 検査対応）。
//!
//! # 色見本を `color_swatch`（`aria-hidden`）+ 可視テキストで表現する理由
//!
//! `product_overview_gallery_split.rs`「色見本を `color_swatch` で表現する
//! 理由」節と同じ判断。色自体は視覚的な補助であり、可視テキスト
//! （「全 N 色」）が色数をアクセシブルに伝える。
//!
//! # セール価格を現在価格 + 取り消し線の元値で表現する理由
//!
//! `<del>` は取り消された内容を示すネイティブ要素であり、追加の
//! `aria-*` なしで「元の価格」であることが支援技術にも伝わる。価格行は
//! 素の `<p>`（`fandhe_frontend_pre_styled_ui::text::text` が呼び出し側の
//! `class` を `drop_class_attr` で除去するため、
//! `product_overview_gallery_split.rs`「価格を素の `<p>` で組み立てる
//! 理由」節と同じ判断）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。リンクは実在する相対パス（`../`）へ向け、`href="#"` は使わない。
//!
//! # ダミー素材について
//!
//! 画像は [`dummy_assets::PRODUCT_SRC`] のみを使い、`data:` URI・外部 URL
//! は使わない。商品名・価格・評価件数・色・バッジ文言・説明文はすべて
//! 独自に書いた架空のものであり、実在の企業名・人物・PII・有料アセット名
//! を含まない。色の RGB 値も任意に選んだもので実在ブランドカラーを模した
//! ものではない。
//!
//! # 集約元差分・スコープ外
//!
//! R0209（密な配置・セール価格 + 色見本）はバリアント B の主構成とした。
//! R0210（色見本 + 評価 + クイックビュー）はクイックビューを除き色見本・
//! 評価をバリアント A へ取り込んだ（クイックビューは無 JS で開閉できない
//! ため省略）。R0211（お気に入り + オプション選択ボタン）はお気に入りを
//! 採用し、オプション選択ボタンはカート追加ボタンで代替した。R0212
//! （1→3 列・バッジ + お気に入り）はバリアント A の列数・バッジ・
//! お気に入り配置へ取り込んだ。R1167（画像上の価格・追加リンク）は画像上
//! への重ね表示を行わず、バリアント B で画像下に価格を置く最小構成とした
//! （重ね表示は CSS が膨らむため）。`_/blocks-intake/` の参照ファイルは
//! 本イシュー着手時点で worktree に存在しないため、対応表 ID のみを記して
//! 実装した（`site/blocks/product-list-rich-cards.md` の「原案差分メモ」
//! 節参照）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardVariant};
use fandhe_frontend_pre_styled_ui::color_swatch::{self, Color, ColorSwatchProps, Rgb};
use fandhe_frontend_pre_styled_ui::icon::{self, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 商品 1 件分のダミーデータ（架空、実在の企業・人物・ブランドとは無関係）。
struct Product {
    name: &'static str,
    description: &'static str,
    price: &'static str,
    /// セール時のみ `Some`（取り消し線で表示する元値）。
    original_price: Option<&'static str>,
    /// 評価（1〜5）。
    rating: u8,
    review_count: u32,
    /// 色見本の RGB（任意に選んだ架空の色）。
    colors: &'static [(u8, u8, u8)],
    badge: Option<&'static str>,
    /// `true` の商品のみお気に入り済みで固定表示する（モジュール doc
    /// 「お気に入りボタンを 1 枚だけ『お気に入り済み』で固定する理由」節）。
    favorite: bool,
}

/// バリアント A（枠付きリッチカード）3 件。
const VARIANT_A: [Product; 3] = [
    Product {
        name: "ブレンドウールニット",
        description: "肌触りの良いウール混の定番ニット。",
        price: "¥8,800",
        original_price: None,
        rating: 4,
        review_count: 36,
        colors: &[(0x2b, 0x3a, 0x4a), (0xb0, 0x8d, 0x5a), (0x3f, 0x5a, 0x3f)],
        badge: Some("新着"),
        favorite: true,
        // ↑ 7 枚中ちょうど 1 枚のみ true（モジュール doc参照）。
    },
    Product {
        name: "リネンワイドパンツ",
        description: "通気性の良いリネン素材のワイドシルエット。",
        price: "¥6,400",
        original_price: None,
        rating: 5,
        review_count: 12,
        colors: &[(0xe8, 0xe2, 0xd6), (0x4a, 0x4a, 0x4a)],
        badge: None,
        favorite: false,
    },
    Product {
        name: "キャンバストートバッグ",
        description: "厚手キャンバス地の大容量トート。",
        price: "¥4,200",
        original_price: Some("¥5,800"),
        rating: 4,
        review_count: 58,
        colors: &[(0xc9, 0xb8, 0x9a)],
        badge: Some("セール"),
        favorite: false,
    },
];

/// バリアント B（密な配置・セール価格）4 件。
const VARIANT_B: [Product; 4] = [
    Product {
        name: "コットンシャツ",
        description: "さらりとした肌触りの定番シャツ。",
        price: "¥3,900",
        original_price: Some("¥5,200"),
        rating: 4,
        review_count: 21,
        colors: &[(0xff, 0xff, 0xff), (0x7a, 0x8a, 0x99)],
        badge: None,
        favorite: false,
    },
    Product {
        name: "デニムジャケット",
        description: "経年変化を楽しめるリジッドデニム。",
        price: "¥9,800",
        original_price: None,
        rating: 5,
        review_count: 9,
        colors: &[(0x3a, 0x4a, 0x6a)],
        badge: None,
        favorite: false,
    },
    Product {
        name: "レザースニーカー",
        description: "型崩れしにくい本革アッパー。",
        price: "¥11,200",
        original_price: Some("¥14,000"),
        rating: 4,
        review_count: 44,
        colors: &[(0xff, 0xff, 0xff), (0x1a, 0x1a, 0x1a), (0x8a, 0x5a, 0x3a)],
        badge: Some("セール"),
        favorite: false,
    },
    Product {
        name: "ウールマフラー",
        description: "軽くて暖かいウール 100% のマフラー。",
        price: "¥3,200",
        original_price: None,
        rating: 3,
        review_count: 5,
        colors: &[(0x6a, 0x2a, 0x2a)],
        badge: None,
        favorite: false,
    },
];

/// お気に入りの装飾アイコン（ハート形。独自の汎用形状で、特定の
/// 参照元・ライブラリのアイコンを持ち込まない）。`favorite` に応じて
/// 塗りつぶし/輪郭を切り替える。
fn heart_icon(favorite: bool) -> Node {
    let fill = if favorite { "currentColor" } else { "none" };
    icon::icon(
        &IconProps {
            size: button::icon_size_for(Size::Sm),
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                (
                    "d",
                    "M12 21s-6.7-4.35-9.3-8.1C.8 10.1 1.3 6.9 4 5.3c2-1.2 4.4-.6 5.8 1.1L12 8.4l2.2-2c1.4-1.7 3.8-2.3 5.8-1.1 2.7 1.6 3.2 4.8 1.3 7.6C18.7 16.65 12 21 12 21z",
                ),
                ("fill", fill),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 画像の角へ重ねるバッジ・お気に入りボタン付きの cover 領域。
fn cover(product: &Product, label: &str) -> Node {
    let mut children = vec![image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio: AspectRatio::Square,
            ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
        },
        vec![],
    )];
    if let Some(badge_label) = product.badge {
        let variant = if badge_label == "セール" {
            ColorPalette::Danger
        } else {
            ColorPalette::Accent
        };
        children.push(badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Solid,
                palette: variant,
                ..BadgeProps::default()
            },
            vec![("data-blocks-product-list-rich-cards-badge", "")],
            vec![text(badge_label)],
        ));
    }
    let (fav_variant, pressed) = if product.favorite {
        (ButtonVariant::Solid, "true")
    } else {
        (ButtonVariant::Outline, "false")
    };
    children.push(button::icon_button(
        &ButtonProps {
            variant: fav_variant,
            size: Size::Sm,
            disabled: true,
            ..ButtonProps::default()
        },
        label,
        vec![
            ("aria-pressed", pressed),
            ("data-blocks-product-list-rich-cards-favorite", ""),
        ],
        vec![heart_icon(product.favorite)],
    ));
    card::cover(
        vec![("data-blocks-product-list-rich-cards-cover", "")],
        children,
    )
}

/// 評価行（readonly `rating-group`。件数込みラベルで明文化する）。
fn rating_row(product: &Product, label_id: &str) -> Node {
    let props = RatingGroupProps {
        disabled: false,
        readonly: true,
        required: false,
    };
    let state = RatingGroup::new(5, Some(u32::from(product.rating)), true);
    let label_text = format!("評価 {}.0（{} 件）", product.rating, product.review_count);
    let label = rating_group::label(&props, Some(label_id), vec![], vec![text(label_text)]);
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
    rating_group::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![],
        vec![label, control],
    )
}

/// 色見本行（装飾の `color_swatch` + 可視テキストで色数を明文化）。
fn swatches_row(colors: &'static [(u8, u8, u8)]) -> Node {
    let mut children: Vec<Node> = colors
        .iter()
        .map(|&(r, g, b)| {
            color_swatch::color_swatch(
                &ColorSwatchProps {
                    value: Color::from_rgb(Rgb::new(r, g, b)),
                    size: Size::Sm,
                    ..ColorSwatchProps::default()
                },
                vec![("aria-hidden", "true")],
                vec![],
            )
        })
        .collect();
    children.push(styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(format!("全 {} 色", colors.len()))],
    ));
    div(
        vec![("class", "blocks-product-list-rich-cards-swatches")],
        children,
    )
}

/// 価格行（素の `<p>`。モジュール doc「セール価格を現在価格 + 取り消し線の
/// 元値で表現する理由」節参照）。
fn price_line(product: &Product) -> Node {
    let children = if let Some(original) = product.original_price {
        vec![
            el("span", vec![], vec![text(product.price)]),
            el("del", vec![], vec![text(format!("通常 {original}"))]),
        ]
    } else {
        vec![text(product.price)]
    };
    el(
        "p",
        vec![("class", "blocks-product-list-rich-cards-price")],
        children,
    )
}

/// 商品カード 1 枚（cover + body + footer）。
fn product_card(
    product: &Product,
    group: &'static str,
    index: usize,
    variant: CardVariant,
) -> Node {
    let rating_label_id = format!("blocks-product-list-rich-cards-{group}-{index}-rating-label");
    let favorite_label = format!("「{}」のお気に入り", product.name);
    card::root(
        variant,
        vec![("data-blocks-product-list-rich-cards-card", "")],
        vec![
            cover(product, &favorite_label),
            card::body(
                vec![],
                vec![
                    card::title(
                        vec![],
                        vec![link::root(
                            "../",
                            &LinkProps::default(),
                            vec![],
                            vec![text(product.name)],
                        )],
                    ),
                    card::description(vec![], vec![text(product.description)]),
                    rating_row(product, &rating_label_id),
                    swatches_row(product.colors),
                    price_line(product),
                ],
            ),
            card::footer(
                vec![],
                vec![button::button(
                    &ButtonProps {
                        disabled: true,
                        ..ButtonProps::default()
                    },
                    vec![("data-blocks-product-list-rich-cards-add", "")],
                    vec![text("カートに追加")],
                )],
            ),
        ],
    )
}

/// バリアント A（枠付きリッチカード、3 枚、1→2→3 列）。
fn variant_a() -> Node {
    div(
        vec![("class", "blocks-product-list-rich-cards-grid-a")],
        VARIANT_A
            .iter()
            .enumerate()
            .map(|(i, p)| product_card(p, "a", i, CardVariant::Outline))
            .collect(),
    )
}

/// バリアント B（密な配置・セール価格、4 枚、2→4 列）。
fn variant_b() -> Node {
    div(
        vec![("class", "blocks-product-list-rich-cards-grid-b")],
        VARIANT_B
            .iter()
            .enumerate()
            .map(|(i, p)| product_card(p, "b", i, CardVariant::Subtle))
            .collect(),
    )
}

/// `product-list-rich-cards` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（`crate::blocks` モジュール doc「静的表示」節）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-product-list-rich-cards-layout")],
        vec![
            styled_text::text(
                &TextProps::default(),
                vec![],
                vec![text("枠付きリッチカード")],
            ),
            variant_a(),
            styled_text::text(
                &TextProps::default(),
                vec![],
                vec![text("密な配置・セール価格")],
            ),
            variant_b(),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/product-list-rich-cards/",
    title: "product-list-rich-cards",
    category: BlockCategory::ProductList,
    rust_source: "crates/docs-site/src/blocks/ecommerce/product_list/product_list_rich_cards.rs",
    demo_class: "blocks-product-list-rich-cards",
    parts: &[
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Rating Group",
            path: "/themes/rating-group/",
        },
        Part {
            label: "Color Swatch",
            path: "/themes/color-swatch/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `product_list_rich_cards` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「block 固有 CSS の置き場」節と同型）。モジュール doc
/// 「列数は `@container` で切り替える」節のとおり、`24rem`/`40rem` の
/// コンテナ幅境界（Demo 枠の実効上限約 43rem 以下）で列数を切り替える。
/// `display: none` は使わない。
const LAYOUT_CSS: &str = "\
.blocks-product-list-rich-cards-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-product-list-rich-cards;\n}\n\
.blocks-product-list-rich-cards-grid-a {\n  display: grid;\n  gap: var(--fandhe-space-6) var(--fandhe-space-4);\n  grid-template-columns: minmax(0, 1fr);\n}\n\
@container blocks-product-list-rich-cards (min-width: 24rem) {\n  .blocks-product-list-rich-cards-grid-a {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n\
@container blocks-product-list-rich-cards (min-width: 40rem) {\n  .blocks-product-list-rich-cards-grid-a {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n}\n\
.blocks-product-list-rich-cards-grid-b {\n  display: grid;\n  gap: var(--fandhe-space-4) var(--fandhe-space-3);\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n}\n\
@container blocks-product-list-rich-cards (min-width: 40rem) {\n  .blocks-product-list-rich-cards-grid-b {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n}\n\
[data-blocks-product-list-rich-cards-cover] {\n  position: relative;\n}\n\
[data-blocks-product-list-rich-cards-badge] {\n  position: absolute;\n  inset-block-start: var(--fandhe-space-2);\n  inset-inline-start: var(--fandhe-space-2);\n}\n\
[data-blocks-product-list-rich-cards-favorite] {\n  position: absolute;\n  inset-block-start: var(--fandhe-space-2);\n  inset-inline-end: var(--fandhe-space-2);\n}\n\
.blocks-product-list-rich-cards-swatches {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-product-list-rich-cards-price {\n  margin: 0;\n  font-weight: var(--fandhe-font-weight-bold);\n  display: flex;\n  align-items: baseline;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-product-list-rich-cards-price del {\n  font-weight: var(--fandhe-font-weight-normal);\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-size-sm);\n}\n\
[data-blocks-product-list-rich-cards-add] {\n  width: 100%;\n}\n\
";

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
            "data-scope=\"image\"",
            "data-scope=\"badge\"",
            "data-scope=\"card\"",
            "data-scope=\"link\"",
            "data-scope=\"text\"",
            "data-scope=\"rating-group\"",
            "data-scope=\"color-swatch\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
    }

    #[test]
    fn all_buttons_are_type_button() {
        let html = demo_html();
        let count_open = html.matches("<button").count();
        let count_typed = html.matches("type=\"button\"").count();
        assert!(count_open > 0);
        assert!(count_typed >= count_open, "html={html}");
    }

    #[test]
    fn demo_has_no_form_or_unsafe_href() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
    }

    #[test]
    fn demo_is_deterministic() {
        assert_eq!(demo_html(), demo_html());
    }

    #[test]
    fn uses_shared_dummy_assets() {
        let html = demo_html();
        assert!(html.contains(super::dummy_assets::PRODUCT_SRC));
    }

    #[test]
    fn exactly_one_favorite_button_is_pressed() {
        let html = demo_html();
        assert_eq!(html.matches(r#"aria-pressed="true""#).count(), 1);
        assert_eq!(html.matches(r#"aria-pressed="false""#).count(), 6);
    }

    #[test]
    fn at_least_one_sale_price_is_struck_through() {
        let html = demo_html();
        assert!(html.matches("<del").count() >= 1);
    }

    #[test]
    fn rating_labels_have_unique_ids() {
        let html = demo_html();
        let ids: Vec<&str> = html
            .split("id=\"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .collect();
        let unique: std::collections::BTreeSet<&str> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "ids should all be unique: {ids:?}");
    }

    #[test]
    fn layout_css_is_safe_and_has_breakpoints() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("container-type: inline-size"));
        assert!(LAYOUT_CSS.contains("@container blocks-product-list-rich-cards (min-width: 24rem)"));
        assert!(LAYOUT_CSS.contains("@container blocks-product-list-rich-cards (min-width: 40rem)"));
        assert!(!LAYOUT_CSS.contains("display: none"));
    }

    #[test]
    fn demo_class_differs_from_layout_root_class() {
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-product-list-rich-cards-layout"
        );
    }
}
