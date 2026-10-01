# product-list-rich-cards

`fandhe-frontend-pre-styled-ui` の `image` / `badge` / `card` / `link` /
`text` / `rating-group` / `color-swatch` / `button` / `icon` 部品を合成した、
評価・色見本・お気に入り/カート追加ボタン付きの商品一覧カードの実例です。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0208、集約元は R0209・R0210・R0211・R0212・R1167。出典の固有名・ファイル名
は記載しません）。本 block は親イシュー #3024 配下のイシュー #3064 にあたる、
Ecommerce / Product List カテゴリの block です。

バリアント A「枠付きリッチカード」は 3 枚のカードを基本 1 列、`24rem` 以上
で 2 列、`40rem` 以上で 3 列に並べます。各カードは画像の角へバッジと
お気に入りボタンを重ね、名称（リンク）・説明・評価（件数込み）・色見本
（色数込み）・価格・カート追加ボタンを縦に並べます。バリアント B「密な
配置・セール価格」は 4 枚のカードを基本 2 列、`40rem` 以上で 4 列に詰めて
並べ、一部の商品はセール価格（現在価格 + 取り消し線の元値）で表示します。
いずれも `display: none` は使わず、狭幅でも全操作要素へ到達できます。
色見本と価格の行は折り返しを許可し、どの列数でもカード外へはみ出しません。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。お気に入りボタンは 7 枚中ちょうど 1 枚のみ
`aria-pressed="true"` の「お気に入り済み」で固定表示し、残りは
`aria-pressed="false"` のまま固定します。無 JS のため押しても表示が
変わらないボタンを操作可能なまま残さないため、お気に入りボタン・カート
追加ボタンのいずれも `type="button"` のまま送信先を持たず `disabled` で
固定表示します。商品名の
リンクは `href="#"` を避け、実在する相対パス（`../`）へ向けています。商品名
・価格・評価件数・色・バッジ文言・説明文はすべて独自に書いた架空のもので
あり、実企業名・実クレデンシャル・PII・有料アセット名を含みません。色の
RGB 値も任意に選んだもので実在ブランドカラーを模したものではありません。

## Rust コード

```rust
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
```

## 原案差分メモ

- 主参照は R0208（枠付き・バッジ + 評価 + 説明）で、バリアント A の主構成
  です。
- R0209（密な配置・セール価格 + 色見本）はバリアント B として採用しました。
- R0210（色見本 + 評価 + クイックビュー）は色見本・評価をバリアント A へ
  取り込みましたが、クイックビューは無 JS では開閉を配線できないため
  省略しました。
- R0211（お気に入り + オプション選択ボタン）はお気に入りをバリアント A へ
  取り込み、オプション選択ボタンはカート追加ボタンで代替しました。
- R0212（1→3 列・バッジ + お気に入り）はバリアント A の列数・バッジ・
  お気に入り配置として取り込みました。
- R1167（画像上に価格と追加リンク）は画像への重ね表示を行うと CSS が
  膨らむため採用せず、バリアント B で価格を画像の下に置く最小構成に
  しました。
- 使用部品に `icon`（お気に入りのハートアイコン表示用）を追加しています。
  視認性を優先し、お気に入りを文字だけのボタンにする案は採りませんでした。
- `_/blocks-intake/` の参照ファイルは本イシュー着手時点で worktree に
  存在しないため、対応表 ID のみを記して実装しました。
- ブラウザでの実機確認（`40rem`/`64rem` 前後のコンテナ幅切替・ライト/ダーク
  両テーマ）はサンドボックス制約により未実施です。cargo test による出力
  検証のみで代替しました。
