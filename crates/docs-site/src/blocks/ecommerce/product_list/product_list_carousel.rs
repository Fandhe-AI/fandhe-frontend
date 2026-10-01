//! `product-list-carousel` block（イシュー #3063。親トラッキング #3024
//! 「Blocks EC」配下、対応表 ID R0626（主参照・基準形）/ R0627（枠付き
//! カード）/ R0628（狭幅カルーセル・広幅グリッド）/ R0629（中央寄せ、
//! Demo 化せず末尾の原稿「原案差分メモ」でのみ扱う）/ R1165（横スクロール +
//! 色見本）の 5 件を構造の参照元とする合成例。取得手段・ファイル名・内部
//! コンポーネント識別子は記載しない（`category_carousel` モジュール doc
//! 「取得手段…」節と同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! イシュー本文指定の `heading` / `carousel` / `card` / `image` / `link` /
//! `button` / `color-swatch` の 7 部品に加え、[`chevron`]（前後トリガーが
//! 空ボタンにならないようにする目的、`category_carousel`/`gallery_carousel`
//! と同じ判断）用の `icon`、価格・形ラベルの文言用の `text` を合成し、計 9
//! 部品になる（[`BLOCK`] の `parts` と一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 3 形の差分（集約元の対応表 ID を並記する理由）
//!
//! 静的な docs サイトのため、実際の横送り・ドラッグ挙動は示せない
//! （`crate::blocks` モジュール doc「静的表示」節）。見出し行（[`header`]）
//! は全形共通とし、その下のカード一覧の表現だけを 3 形並べて見た目の
//! 差分で挙動の違いを読み取れるようにする:
//!
//! - **A 基準形（R0626）**: `carousel` の viewport（`item-group`）を単独の
//!   横一列に置き、その**下段**（`category_carousel` とは逆の配置）へ
//!   `prev-trigger` / `indicator-group` / `next-trigger` を横並びでまとめた
//!   `control` 行を添える。表示枚数は既定 `basis: 50%`（2 枚）、
//!   `>= 64rem` で `basis: 25%`（4 枚）へ切り替える。前後トリガー・
//!   indicator は無 JS のため常時 `disabled` であり操作できないため、
//!   viewport 自体を `overflow-x: auto`（スクロール可能）+
//!   `scroll-snap-type: x mandatory` にして、先頭以外の商品へもネイティブ
//!   の横スクロールで到達できるようにする（`category_carousel` と同じ
//!   フォールバック設計）。
//! - **B 狭幅カルーセル・広幅グリッド（R0628 + R0627）**: 枠付きカード
//!   （`CardVariant::Outline`。既定 variant と同じ値だが意図を明示する
//!   ため明示指定する、対応表 ID R0627 の統合、末尾「原案差分メモ」参照）
//!   を 6 件用いる。`< 64rem` は A と同じ横スクロール可能なカルーセルだが、
//!   `>= 64rem` では [`LAYOUT_CSS`] が `item-group` を `display: grid`
//!   （3 列）へ切り替え、`control` 行（前後トリガー・indicator-group）を
//!   `display: none` にする。グリッド下には、無 JS では動作しない
//!   `button::button`（`disabled: true`）の CTA を添える
//!   （`category_carousel` 基準形の CTA と同じ判断）。
//! - **C 横スクロール + 色見本（R1165）**: `carousel` 部品は使わない。素の
//!   `div`（`overflow-x: auto; scroll-snap-type: x mandatory`）へカード
//!   5 件を `flex: 0 0 <幅>` で並べ、前後ボタン・ドットは置かずスクロール
//!   バーだけで送る。各カードに [`color_swatch::color_swatch`]（`Size::Sm`、
//!   `aria-hidden="true"`、装飾）を 3 色分並べ、同じ情報を可視テキスト
//!   （例「全 3 色」）でも示す（モジュール doc「色見本を
//!   可視テキストと併記する理由」節参照）。`>= 64rem` では
//!   `display: grid`（5 列）へ切り替える。領域には `role="region"` +
//!   `aria-label` を付け、ランドマーク名を与える（スクロール可能なことを
//!   支援技術へ伝える唯一の手段、`category_carousel` と同型）。
//!
//! # 静的表示の不変条件
//!
//! A（`item`/`indicator` とも index 0 のみが
//! `data-current`/`data-inview`/`aria-current` を持つ、`category_carousel`
//! と同じ）と異なり、B は `>= 64rem` で全商品を同時表示するグリッドへ
//! 切り替わるため、`item`/`indicator` とも全件を `data-current`/
//! `data-inview`/`aria-current` 付きで出力する（PR #3512 codex 指摘。
//! `product_carousel` の doc コメント参照）。「表示中」の属性は画面幅で
//! 出し分けない（無 JS の静的 SSR のため）。`id=`/`aria-labelledby` は
//! 出力しない（[`carousel::root`] の `label` 引数が `aria-label` を直接
//! 出力するため、複数インスタンスを 1 ページに置いても id 重複が
//! 起きない）。
//!
//! # 無 JS のため全操作要素を常時無効化する
//!
//! `category_carousel`/`gallery_carousel` と同じ理由（動作しない
//! インタラクション要素をクリック可能に見せない）で、A/B の
//! `prev-trigger`/`next-trigger`/`indicator`、B の CTA `button` は
//! いずれも常時 `disabled` の状態で描画する。C はそもそも操作ボタンを
//! 持たないため対象外（支援技術からはネイティブのスクロール領域として
//! 常に操作可能）。
//!
//! # `alt` を空文字列にする理由
//!
//! 商品名はカード内の商品名リンク（可視テキスト）として既に存在するため、
//! 画像自体は装飾として扱い `alt` を空文字列にする
//! （`category_carousel::category_tile` の背景画像と同じ判断）。
//!
//! # 商品名リンクのみを操作要素にする（`link-overlay` を使わない理由）
//!
//! イシュー本文が指定する使用部品に `link-overlay` を含まないため、
//! カード全体をクリック領域化する `link_overlay` は使わない。カード内の
//! 主たる操作要素は商品名の `link::root` 1 箇所のみとする。
//!
//! # 色見本を可視テキストと併記する理由
//!
//! `color_swatch::color_swatch` は `aria-hidden="true"` を付けた装飾として
//! 置く（`product_overview_gallery_split` の色見本と同じ判断）ため、色数の
//! 情報自体は `text::text` の可視テキスト（例「全 3 色」）で別途示し、
//! 支援技術からも色数が読み取れるようにする。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。画像は [`dummy_assets`] のビルド時生成 SVG（相対パス）のみを
//! 使い、`data:` URI・外部 URL は使わない。商品名・価格・色はすべて架空の
//! もの（実在の企業名・人名・PII・商標を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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

/// 商品カード 1 件（`card` > 画像 + `card::body`（商品名リンク + 価格））。
/// 画像は `card::root` 直下に置いて全幅ベタ塗りにし、テキスト側だけ
/// `card::body` の既定 padding（`--fandhe-card-padding`）で内側の余白
/// （content inset）を確保する。画像をそのまま `card::root` 直下へ置いて
/// 名称・価格を無 padding の `div` にしていた旧実装は、両者がカードの
/// 外枠（`CardVariant::Outline` のボーダー／`overflow: hidden` のクリップ
/// 境界）へ接触する不具合があったため是正した（PR #3512 Bugbot 指摘）。
/// 商品名・価格・色見本を `card::body` でラップしたことで、これらは
/// `card::root` の flex gap の対象外（`card::root` の直接の子は画像 +
/// `card::body` の 2 件のみ）になる。[`LAYOUT_CSS`] は `card::root` 側の
/// gap を撤去し（画像下の余白は `card::body` の padding のみに一本化し
/// 二重取りしない）、`card::body` 側へ gap を付け替えて商品名・価格
/// （・色見本）の行間を確保する（PR #3512 Bugbot 指摘の是正）。
/// `outline` が `true` のとき `CardVariant::Outline` を明示する（B 形、
/// モジュール doc 「枠付きカード」節参照。既定も `Outline` のため見た目は
/// A/C と同じだが意図を明示する）。`swatches` が `Some` のときのみ色見本
/// 行を末尾へ足す（C 形専用）。
fn product_card(
    name: &'static str,
    price: &'static str,
    src: &'static str,
    outline: bool,
    swatches: Option<&[(u8, u8, u8); 3]>,
) -> Node {
    let mut body_children: Vec<Node> = vec![
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
        body_children.push(div(
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
        vec![
            image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    aspect_ratio: AspectRatio::Square,
                    ..ImageProps::new(src, "")
                },
                vec![("data-blocks-product-list-carousel-image", "")],
            ),
            card::body(
                vec![("data-blocks-product-list-carousel-body", "")],
                body_children,
            ),
        ],
    )
}

/// A/B 共通のカルーセル本体（viewport + 下段 control 行）。`outline` は
/// B 形のみ `true`（商品カードを枠付きにする）。`extra_root_attr` は B 形
/// のみが持つグリッド切り替え用フックであり、`Some` のときは
/// `>= 64rem` で [`LAYOUT_CSS`] が `item-group` を grid へ切り替えて
/// 全商品を同時表示する（モジュール doc「3 形の差分」節）。
///
/// `grid_mode`（`extra_root_attr.is_some()`）のときは item/indicator を
/// 全件 `current` 扱いにする（`carousel::item`/`indicator` の `current` は
/// 「現在ビューポートに入っている」ことを表す `data-inview` 相当の意味
/// （`crates/headless-ui/src/carousel.rs` の `item` doc 参照）であり、
/// grid 表示では文字どおり全件が同時に視界へ入るため、index 0 のみを
/// `data-current`/`data-inview`/`aria-current` 付きにするのは支援技術への
/// 通知と実際の表示（全件表示・制御行非表示）が矛盾する（PR #3512 codex
/// 指摘）。grid へ切り替わらない A 形は従来どおり index 0 のみを
/// `current` とする（モジュール doc「静的表示の不変条件」節）。
fn product_carousel(
    label: &'static str,
    outline: bool,
    extra_root_attr: Option<(&'static str, &'static str)>,
) -> Node {
    let grid_mode = extra_root_attr.is_some();
    let items: Vec<Node> = PRODUCTS
        .iter()
        .enumerate()
        .map(|(i, (name, price, src, _))| {
            carousel::item(
                Orientation::Horizontal,
                i,
                PRODUCTS.len(),
                i == 0 || grid_mode,
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
                i == 0 || grid_mode,
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/product-list-carousel/",
    title: "product-list-carousel",
    category: BlockCategory::ProductList,
    rust_source: "crates/docs-site/src/blocks/ecommerce/product_list/product_list_carousel.rs",
    demo_class: "blocks-product-list-carousel",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Carousel",
            path: "/themes/carousel/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
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
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Color Swatch",
            path: "/themes/color-swatch/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `product_list_carousel` 固有のレイアウト規則（モジュール doc「3 形の
/// 差分」節参照）。`blocks.css` は全 block の CSS を連結するため、兄弟
/// block（他の carousel 系 block）へ波及しないよう
/// `.blocks-product-list-carousel-*` クラス・
/// `data-blocks-product-list-carousel-*` 属性でスコープする
/// （`category_carousel` と同じ判断）。
const LAYOUT_CSS: &str = "\
.blocks-product-list-carousel-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n  width: 100%;\n}\n\
.blocks-product-list-carousel-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-product-list-carousel-header {\n  display: flex;\n  flex-wrap: wrap;\n  align-items: baseline;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-product-list-carousel-viewport {\n  min-width: 0;\n  overflow-x: auto;\n  overflow-y: hidden;\n  scroll-snap-type: x mandatory;\n}\n\
[data-scope=\"carousel\"][data-part=\"item\"][data-blocks-product-list-carousel-tile] {\n  box-sizing: border-box;\n  padding-inline: var(--fandhe-space-2);\n  scroll-snap-align: start;\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-product-list-carousel-card] {\n  overflow: hidden;\n  display: flex;\n  flex-direction: column;\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-product-list-carousel-image] {\n  display: block;\n  width: 100%;\n  border-radius: var(--fandhe-radius-md, 0.375rem);\n}\n\
[data-scope=\"card\"][data-part=\"body\"][data-blocks-product-list-carousel-body] {\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"heading\"][data-blocks-product-list-carousel-name] {\n  font-size: 1rem;\n}\n\
.blocks-product-list-carousel-price {\n  font-weight: 600;\n}\n\
.blocks-product-list-carousel-swatch-row {\n  display: flex;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"carousel\"][data-part=\"control\"][data-blocks-product-list-carousel-control] {\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  gap: var(--fandhe-space-4);\n}\n\
[data-scope=\"carousel\"][data-part=\"indicator\"][data-blocks-product-list-carousel-indicator]:disabled {\n  opacity: 0.5;\n  cursor: not-allowed;\n}\n\
[data-scope=\"carousel\"][data-part=\"root\"][data-blocks-product-list-carousel-root] {\n  --fandhe-carousel-item-basis: 50%;\n}\n\
@media (min-width: 64rem) {\n  [data-scope=\"carousel\"][data-part=\"root\"][data-blocks-product-list-carousel-root] {\n    --fandhe-carousel-item-basis: 25%;\n  }\n}\n\
[data-blocks-product-list-carousel-cta] {\n  align-self: flex-start;\n  margin-top: var(--fandhe-space-2);\n}\n\
@media (min-width: 64rem) {\n  [data-scope=\"carousel\"][data-part=\"root\"][data-blocks-product-list-carousel-grid-mode] [data-scope=\"carousel\"][data-part=\"item-group\"] {\n    display: grid;\n    grid-template-columns: repeat(3, 1fr);\n    transform: none;\n  }\n  [data-scope=\"carousel\"][data-part=\"root\"][data-blocks-product-list-carousel-grid-mode] [data-scope=\"carousel\"][data-part=\"control\"] {\n    display: none;\n  }\n}\n\
.blocks-product-list-carousel-scroll {\n  display: flex;\n  gap: var(--fandhe-space-4);\n  overflow-x: auto;\n  scroll-snap-type: x mandatory;\n  padding-block: var(--fandhe-space-1);\n}\n\
.blocks-product-list-carousel-scroll-item {\n  flex: 0 0 60%;\n  scroll-snap-align: start;\n}\n\
@media (min-width: 64rem) {\n  .blocks-product-list-carousel-scroll {\n    display: grid;\n    grid-template-columns: repeat(5, 1fr);\n    overflow: visible;\n  }\n  .blocks-product-list-carousel-scroll-item {\n    flex: initial;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, BLOCK, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo は呼び出しごとに同一の `Node` を返す純関数であること
    /// （`crate::blocks` モジュール doc「静的表示」節）。
    #[test]
    fn demo_is_deterministic() {
        assert_eq!(render(&demo()), render(&demo()));
    }

    /// Demo が期待する 9 種の部品を出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"carousel\"",
            "data-scope=\"card\"",
            "data-scope=\"image\"",
            "data-scope=\"link\"",
            "data-scope=\"button\"",
            "data-scope=\"color-swatch\"",
            "data-scope=\"text\"",
            "data-scope=\"icon\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// carousel の WAI-ARIA carousel パターン（`aria-roledescription`）が
    /// A/B の 2 インスタンス分出力されること。
    #[test]
    fn demo_wires_two_carousel_instances() {
        let html = render(&demo());
        assert_eq!(html.matches("aria-roledescription=\"carousel\"").count(), 2);
    }

    /// A は index 0 のみ選択済み（静的表示の不変条件、`category_carousel`
    /// と同じ判断）。B は全件同時表示のグリッドへ切り替わるため全件
    /// `current` 扱い（モジュール doc「静的表示の不変条件」節、PR #3512
    /// codex 指摘）。内訳: `data-current` は A（item 1 + indicator 1）+
    /// B（item 6 + indicator 6）= 14 件、`aria-current="true"` は
    /// indicator のみ A 1 + B 6 = 7 件。
    #[test]
    fn demo_selects_first_product_by_default() {
        let html = render(&demo());
        assert_eq!(html.matches("data-current").count(), 14);
        assert_eq!(html.matches("aria-current=\"true\"").count(), 7);
    }

    /// `data-inview` は item のみが出力する。A は index 0 の 1 件、B は
    /// 全件表示のため 6 件の合計 7 件。
    #[test]
    fn demo_marks_only_first_product_inview() {
        let html = render(&demo());
        assert_eq!(html.matches("data-inview").count(), 7);
    }

    /// 無 JS のため A/B の前後トリガー・indicator・B の CTA button が
    /// すべて無効化されていること（モジュール doc「無 JS のため全操作要素を
    /// 常時無効化する」節）。内訳: prev 2 + next 2 + indicator
    /// （2 インスタンス × 6 件 = 12）+ CTA button 1 の合計 17 件。
    #[test]
    fn demo_disables_all_interactive_controls() {
        let html = render(&demo());
        assert_eq!(html.matches("aria-label=\"前の商品\"").count(), 2);
        assert_eq!(html.matches("aria-label=\"次の商品\"").count(), 2);
        assert_eq!(
            html.matches("data-scope=\"carousel\" data-part=\"indicator\"")
                .count(),
            12
        );
        assert_eq!(html.matches(" disabled=\"\"").count(), 17);
    }

    /// 商品カードが A(6) + B(6) + C(5) = 17 件出力されること。
    #[test]
    fn demo_renders_seventeen_product_cards() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-product-list-carousel-card")
                .count(),
            17
        );
    }

    /// C 形の色見本が 5 件 × 3 色 = 15 件出力され、すべて `aria-hidden`
    /// であること（モジュール doc「色見本を可視テキストと併記する理由」
    /// 節）。
    #[test]
    fn demo_renders_fifteen_color_swatches_all_aria_hidden() {
        let html = render(&demo());
        assert_eq!(html.matches("data-scope=\"color-swatch\"").count(), 15);
        assert_eq!(
            html.matches("data-scope=\"color-swatch\"")
                .zip(html.matches("aria-hidden=\"true\""))
                .count(),
            15
        );
    }

    /// 非対話・安全性の不変条件。
    #[test]
    fn demo_never_contains_forbidden_markup() {
        let html = render(&demo());
        for absent in [
            "<form",
            "href=\"#\"",
            "src=\"data:",
            "id=\"",
            "aria-labelledby",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// C 形の横スクロール領域が `role="region"` + `aria-label` を持つこと。
    #[test]
    fn demo_scroll_region_has_accessible_name() {
        let html = render(&demo());
        assert!(html.contains("role=\"region\" aria-label=\"商品一覧（横スクロール）\""));
    }

    /// [`LAYOUT_CSS`] が per-view basis・64rem ブレークポイント・B 形の
    /// グリッド切り替え・C 形の横スクロールを宣言すること。
    #[test]
    fn layout_css_declares_breakpoints_and_scroll_rules() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("--fandhe-carousel-item-basis: 50%"));
        assert!(LAYOUT_CSS.contains("--fandhe-carousel-item-basis: 25%"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: repeat(3, 1fr)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: repeat(5, 1fr)"));
        assert!(LAYOUT_CSS.contains("overflow-x: auto"));
        assert!(LAYOUT_CSS.contains("scroll-snap-type: x mandatory"));
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// レイアウト用ルート class（`.blocks-product-list-carousel-layout`）が
    /// `demo_class`（`blocks-product-list-carousel`）と異なること（既存
    /// block と同じ Bugbot 教訓）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        assert_ne!(BLOCK.demo_class, "blocks-product-list-carousel-layout");
    }
}
