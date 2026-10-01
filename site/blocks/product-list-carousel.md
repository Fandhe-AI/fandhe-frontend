# product-list-carousel

商品一覧ブロックです。見出し行（左にセクション見出し、右に一覧ページへの
リンク）の下へ、商品カード（画像 + 商品名リンク + 価格）を横一列に
並べます。`heading` / `carousel` / `card` / `image` / `link` / `button` /
`color-swatch` の 7 部品に加え、前後トリガーが空ボタンにならないよう
`icon`、価格・形ラベルの文言に `text` も合成します。Blocks は既存部品の
合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0626 です。枠付きカードは R0627、狭幅カルーセル・
広幅グリッドへの切り替えは R0628、横スクロール + 色見本版は R1165 を
参照しています。

3 形を並べて掲載しています。**基準形**は 2〜4 枚同時表示のカルーセルで、
viewport の下段に前後ボタン・ドット（indicator）をまとめた行を添えます。
**狭幅カルーセル・広幅グリッド**形は、枠付きカードを使い、広い画面幅
ではカルーセルをやめて 3 列グリッド表示へ切り替え、下に「もっと見る」
ボタンを添えます。**横スクロール + 色見本**形はカルーセル部品を使わず、
前後ボタン・ドットを持たないスクロール領域だけでカードを送り、各カードに
色見本（装飾）と色数の可視テキストを添え、広い画面幅では 5 列グリッドへ
切り替えます。

本 Demo は無 JS の静的表示のみのため、基準形・狭幅カルーセル形の前後
ボタン・ドット・「もっと見る」ボタンはいずれも常時無効状態（disabled）
で描画しています。横スクロール形はボタンを持たず、`role="region"` +
`aria-label` を付けたスクロール領域だけで操作可能です。

商品名・価格・色・画像はすべて架空のデータであり、実在の企業・ブランド・
PII・実クレデンシャルは含みません。画像はビルド時生成の同梱
プレースホルダー SVG です。

本 Demo は `<form>` を含みません。送信処理・状態管理は一切持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::carousel::{self, Orientation};
use fandhe_frontend_pre_styled_ui::color_swatch::{self, Color, ColorSwatchProps, Rgb};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// リンク先の固定外部 URL（`category_carousel::REPO` と同じ方針。
/// `Block::demo` は `base_path` を受け取れないため、サイト内リンクではなく
/// 固定の外部 URL を使う）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// [`PRODUCTS`] の 1 件（商品名・価格・画像 `src`・色見本 3 色）。
/// `clippy::type_complexity` 回避のための別名（タプル自体の意味は
/// モジュール doc「架空の商品 6 件」節参照）。
type ProductEntry = (&'static str, &'static str, &'static str, [(u8, u8, u8); 3]);

/// 架空の商品 6 件（実在の企業・商標とは無関係）。`src` は [`dummy_assets`]
/// の 4 種（`AVATAR_SRC` を除く）を循環させる。`swatches` は C 形でのみ
/// 使う色見本の RGB 3 色（架空の色で、実在ブランドカラーを模したもので
/// はない）。
const PRODUCTS: [ProductEntry; 6] = [
    (
        "リネンシャツ",
        "¥6,900",
        dummy_assets::PRODUCT_SRC,
        [(0xE8, 0xE2, 0xD6), (0x3A, 0x3A, 0x3A), (0x6B, 0x7F, 0x6B)],
    ),
    (
        "セラミックマグ",
        "¥2,400",
        dummy_assets::BACKGROUND_SRC,
        [(0xFF, 0xFF, 0xFF), (0x2B, 0x2B, 0x2B), (0xC9, 0xA0, 0x6B)],
    ),
    (
        "ウールマフラー",
        "¥8,200",
        dummy_assets::SCREENSHOT_SRC,
        [(0x8B, 0x2E, 0x2E), (0x2E, 0x3A, 0x59), (0x4A, 0x4A, 0x4A)],
    ),
    (
        "レザートート",
        "¥15,800",
        dummy_assets::LOGO_SRC,
        [(0x5C, 0x3A, 0x21), (0x1A, 0x1A, 0x1A), (0xB0, 0x8A, 0x5E)],
    ),
    (
        "ニットキャップ",
        "¥3,600",
        dummy_assets::PRODUCT_SRC,
        [(0x4A, 0x4A, 0x4A), (0xD6, 0xC7, 0xB0), (0x2E, 0x4A, 0x6B)],
    ),
    (
        "コットンソックス",
        "¥1,200",
        dummy_assets::BACKGROUND_SRC,
        [(0xFF, 0xFF, 0xFF), (0x8A, 0x8A, 0x8A), (0x2E, 0x4A, 0x2E)],
    ),
];

/// 各形の直前に置く短い形ラベル（`category_carousel::variant_label` と
/// 同型）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// 自作の幾何アイコン（線画。`category_carousel::chevron` と同型）。
fn chevron(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

fn chevron_left() -> Node {
    chevron("M15 18l-6-6 6-6")
}

fn chevron_right() -> Node {
    chevron("M9 18l6-6-6-6")
}

/// 見出し行。左にセクション見出し、右に一覧ページへのリンクを置き、狭幅
/// では折り返す（`category_carousel::header` と同型）。
fn header(title: &'static str, link_label: &'static str) -> Node {
    div(
        vec![("class", "blocks-product-list-carousel-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text(title)],
            ),
            link::root(REPO, &LinkProps::default(), vec![], vec![text(link_label)]),
        ],
    )
}

/// 商品カード 1 件（`card` > 画像 + 商品名リンク + 価格）。`outline` が
/// `true` のとき `CardVariant::Outline` を明示する（B 形、モジュール doc
/// 「枠付きカード」節参照。既定も `Outline` のため見た目は A/C と同じだが
/// 意図を明示する）。`swatches` が `Some` のときのみ色見本行を末尾へ足す
/// （C 形専用）。
fn product_card(
    name: &'static str,
    price: &'static str,
    src: &'static str,
    outline: bool,
    swatches: Option<&[(u8, u8, u8); 3]>,
) -> Node {
    let mut children: Vec<Node> = vec![
        image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Square,
                ..ImageProps::new(src, "")
            },
            vec![("data-blocks-product-list-carousel-image", "")],
        ),
        heading(
            HeadingLevel::H4,
            &HeadingProps::default(),
            vec![("data-blocks-product-list-carousel-name", "")],
            vec![link::root(
                REPO,
                &LinkProps::default(),
                vec![],
                vec![text(name)],
            )],
        ),
        div(
            vec![("class", "blocks-product-list-carousel-price")],
            vec![text(price)],
        ),
    ];
    if let Some(colors) = swatches {
        let swatch_nodes: Vec<Node> = colors
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
        children.push(div(
            vec![("class", "blocks-product-list-carousel-swatches")],
            vec![
                div(
                    vec![("class", "blocks-product-list-carousel-swatch-row")],
                    swatch_nodes,
                ),
                styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text(format!("全 {} 色", colors.len()))],
                ),
            ],
        ));
    }
    card::root(
        if outline {
            CardProps {
                variant: CardVariant::Outline,
                ..CardProps::default()
            }
        } else {
            CardProps::default()
        },
        vec![("data-blocks-product-list-carousel-card", "")],
        children,
    )
}

/// A/B 共通のカルーセル本体（viewport + 下段 control 行）。`outline` は
/// B 形のみ `true`（商品カードを枠付きにする）。`extra_root_attr` は B 形
/// のみが持つグリッド切り替え用フック。
fn product_carousel(
    label: &'static str,
    outline: bool,
    extra_root_attr: Option<(&'static str, &'static str)>,
) -> Node {
    let items: Vec<Node> = PRODUCTS
        .iter()
        .enumerate()
        .map(|(i, (name, price, src, _))| {
            carousel::item(
                Orientation::Horizontal,
                i,
                PRODUCTS.len(),
                i == 0,
                vec![("data-blocks-product-list-carousel-tile", "")],
                vec![product_card(name, price, src, outline, None)],
            )
        })
        .collect();
    let indicators: Vec<Node> = (0..PRODUCTS.len())
        .map(|i| {
            carousel::indicator(
                Orientation::Horizontal,
                i,
                i == 0,
                vec![
                    ("disabled", ""),
                    ("data-blocks-product-list-carousel-indicator", ""),
                ],
            )
        })
        .collect();

    let mut root_attrs: Vec<(&str, &str)> = vec![("data-blocks-product-list-carousel-root", "")];
    if let Some(attr) = extra_root_attr {
        root_attrs.push(attr);
    }

    carousel::root(
        Size::Md,
        Orientation::Horizontal,
        label,
        root_attrs,
        vec![
            div(
                vec![("class", "blocks-product-list-carousel-viewport")],
                vec![carousel::item_group(Orientation::Horizontal, vec![], items)],
            ),
            carousel::control(
                Orientation::Horizontal,
                vec![("data-blocks-product-list-carousel-control", "")],
                vec![
                    carousel::prev_trigger(
                        Orientation::Horizontal,
                        true,
                        "前の商品",
                        vec![],
                        vec![chevron_left()],
                    ),
                    carousel::indicator_group(
                        Orientation::Horizontal,
                        vec![("data-blocks-product-list-carousel-indicators", "")],
                        indicators,
                    ),
                    carousel::next_trigger(
                        Orientation::Horizontal,
                        true,
                        "次の商品",
                        vec![],
                        vec![chevron_right()],
                    ),
                ],
            ),
        ],
    )
}

/// A 基準形（対応表 ID R0626）。
fn variant_a() -> Node {
    div(
        vec![("class", "blocks-product-list-carousel-section")],
        vec![
            header("おすすめ商品", "すべての商品を見る"),
            product_carousel("商品一覧（2〜4 枚表示）", false, None),
        ],
    )
}

/// B 狭幅カルーセル・広幅グリッド（対応表 ID R0628 + R0627）。
fn variant_b() -> Node {
    div(
        vec![("class", "blocks-product-list-carousel-section")],
        vec![
            header("おすすめ商品", "すべての商品を見る"),
            product_carousel(
                "商品一覧（広い画面ではグリッド表示）",
                true,
                Some(("data-blocks-product-list-carousel-grid-mode", "")),
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-product-list-carousel-cta", "")],
                vec![text("商品をもっと見る")],
            ),
        ],
    )
}

/// C 横スクロール + 色見本（対応表 ID R1165）。`carousel` 部品を使わず、
/// 前後ボタン・ドットを持たないスクロール領域のみで送る。
fn variant_c() -> Node {
    let tiles: Vec<Node> = PRODUCTS
        .iter()
        .take(5)
        .map(|(name, price, src, colors)| {
            div(
                vec![("class", "blocks-product-list-carousel-scroll-item")],
                vec![product_card(name, price, src, false, Some(colors))],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-product-list-carousel-section")],
        vec![
            header("おすすめ商品", "すべての商品を見る"),
            div(
                vec![
                    ("class", "blocks-product-list-carousel-scroll"),
                    ("role", "region"),
                    ("aria-label", "商品一覧（横スクロール）"),
                ],
                tiles,
            ),
        ],
    )
}

/// `product-list-carousel` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（`crate::blocks` モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-product-list-carousel-layout")],
        vec![
            variant_label("基準形（対応表 ID R0626。既定 2 枚・lg 以上で 4 枚表示）"),
            variant_a(),
            variant_label("狭幅カルーセル・広幅グリッド（対応表 ID R0628/R0627）"),
            variant_b(),
            variant_label("横スクロール + 色見本（対応表 ID R1165。lg 以上で 5 列グリッド）"),
            variant_c(),
        ],
    )
}
```

## 原案差分メモ

- 集約元 5 件（R0626 / R0627 / R0628 / R0629 / R1165）のうち、構造の
  差分として Demo 化したのは 3 形です。R0627（枠付きカード）は独立した
  Demo を設けず、狭幅カルーセル・広幅グリッド形（R0628）へ統合しました。
  `card::root` の `variant: CardVariant::Outline` 明示が唯一の差分であり、
  独立インスタンスを増やすより B 形の属性として表現する方が合成例として
  簡潔だと判断しました。
- R0629（中央寄せカード・中央見出し）は Demo に 4 形目として追加しません
  でした。見出し・カード本文の `text-align` を変えるだけの差分であり、
  構造上の違いが小さいため本メモでの言及に留めます。中央寄せにする場合は
  `blocks-product-list-carousel-header`/`-name`/`-price` の各クラスへ
  `text-align: center` を追加するだけで再現できます。
- R1165 の参照元は `carousel` 部品に相当する送り UI を持たず、ネイティブ
  スクロールのみで商品を送ります。無 JS の静的表示では実際のドラッグ送り
  を示せないため、横スクロール形は `carousel` 部品を使わず
  `overflow-x: auto` 領域とし、前後ボタン・ドットは置きません。
- 色見本（`color-swatch`）は R1165 のみが持つ要素のため、横スクロール
  形（色見本併記）のみに添えています。装飾として `aria-hidden="true"` を付け、
  色数は別途「全 N 色」の可視テキストで示します（モジュール doc「色見本
  を可視テキストと併記する理由」節参照）。
- イシュー本文が指定する 7 部品に加え、前後トリガー（`prev-trigger`/
  `next-trigger`）が空ボタンにならないよう `icon`、価格・形ラベルの文言に
  `text` を合成しています（`category-carousel`/`gallery-carousel` の
  前例と同じ判断）。
- 基準形の表示枚数は参照元に合わせ既定 2 枚・広い画面幅で 4 枚としました。
  狭幅カルーセル・広幅グリッド形は広い画面幅で 3 列、横スクロール形は
  5 列へ切り替えます（列数の違いはそれぞれの参照元をそのまま踏襲）。
- ドット（indicator）・前後ボタンは、`category-carousel` では viewport の
  両脇に配置していましたが、本 block では viewport 下段へまとめて配置
  しています（イシュー本文の記述に合わせた配置の違いであり、部品構成
  自体は同じです）。
