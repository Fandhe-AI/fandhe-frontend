# product-overview-featured-split

`fandhe-frontend-pre-styled-ui` の `breadcrumb` / `heading` / `text` /
`image` / `radio-card` / `rating-group` / `button` / `icon` 部品を合成
した、単一の大きな商品画像と特徴説明を持つ商品詳細の実例です。Blocks
セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照・集約元
ともに対応表 ID R1177 の 1 件のみ。出典の固有名・ファイル名は記載しま
せん）。

広い画面では左に商品情報（パンくず・商品名・価格・評価・説明・サイズ
選択・在庫表示・カート追加ボタン・保証の補足）、右に単一の大きな商品
画像を 2 カラムで並べます。`48rem` 未満の狭い画面では 1 列になり、商品
画像を情報の間（パンくず〜説明の下・サイズ選択より上）へ挟み込みます
（CSS Grid の `grid-template-areas` のみで実現しており、`display: none`
も DOM の重複も使いません。DOM 順は常に「商品情報（上）→ 画像 →
商品情報（下）」で、狭い画面の視覚順と一致します）。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。カート追加ボタンは `type="button"` のまま
送信先を持たず、押しても何も起きないボタンを操作可能なまま残さないため
`disabled` で固定表示します。サイズの選択欄はネイティブ `disabled` の
まま固定表示し、現在の選択を本文テキストで明文化しています。在庫表示・
保証の補足に添えるアイコンは独自の単純な幾何形状（チェック線・盾形）の
装飾であり、実在のアイコンセット・ブランドの意匠は持ち込んでいません。
パンくずのリンクは `href="#"` を避け、実在する相対パスへ向けています。
商品名・価格・評価件数・サイズ・説明文・在庫・保証の文言はすべて独自に
書いた架空のものであり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{self, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::radio_card::{self, Orientation as RadioCardOrientation};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 評価ラベル（`rating_group::label` の `id`）。他 block との重複を避ける
/// ため本 block 名を含む（モジュール doc「id を block 固有の定数にする
/// 理由」節参照）。
const RATING_LABEL_ID: &str = "blocks-product-overview-featured-split-rating-label";
/// サイズ選択見出し（`radio_card::label` の `id`）。
const SIZE_LABEL_ID: &str = "blocks-product-overview-featured-split-size-label";
/// サイズ選択 radio card のネイティブ `name`（フォーム未送信のため排他
/// 選択の実効はないが、[`radio_card::item_hidden_input`] の契約上必須）。
const SIZE_NAME: &str = "blocks-product-overview-featured-split-size";

/// パンくず。`href="#"` は使わず実在する相対パスへ向ける
/// （モジュール doc「`<form>` を持たない」節参照）。
fn product_breadcrumb() -> Node {
    breadcrumb::root(
        Size::Sm,
        BreadcrumbVariant::default(),
        Some("パンくず"),
        vec![],
        vec![breadcrumb::list(
            vec![],
            vec![
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../../", vec![], vec![text("ホーム")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../", vec![], vec![text("Blocks")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::current_link(
                        vec![],
                        vec![text("保温ステンレスボトル")],
                    )],
                ),
            ],
        )],
    )
}

/// 評価行（`rating-group`、readonly。現在の評価をラベルで明文化する、
/// `product_overview_gallery_split.rs::rating_row` と同型の判断）。
fn rating_row() -> Node {
    let rating_props = RatingGroupProps {
        disabled: false,
        readonly: true,
        required: false,
    };
    let rating_group_state = RatingGroup::new(5, Some(4), true);
    let rating_label = rating_group::label(
        &rating_props,
        Some(RATING_LABEL_ID),
        vec![],
        vec![text("評価 4.0（52 件）")],
    );
    let rating_items: Vec<Node> = (1..=rating_group_state.count())
        .map(|i| {
            rating_group::item(
                i,
                RatingItemFlags {
                    checked: rating_group_state.is_checked(i),
                    highlighted: rating_group_state.is_highlighted(i),
                    disabled: false,
                    readonly: true,
                },
                &format!("{i} star{}", if i == 1 { "" } else { "s" }),
                vec![],
                vec![],
            )
        })
        .collect();
    let rating_control =
        rating_group::control(&rating_props, Some(RATING_LABEL_ID), vec![], rating_items);
    div(
        vec![("class", "blocks-product-overview-featured-split-rating")],
        vec![rating_group::root(
            Size::Sm,
            ColorPalette::Accent,
            &rating_props,
            vec![],
            vec![rating_label, rating_control],
        )],
    )
}

/// 価格（素の `<p>`。モジュール doc「価格を素の `<p>` で組み立てる理由」
/// 節参照）。
fn price_line() -> Node {
    el(
        "p",
        vec![("class", "blocks-product-overview-featured-split-price")],
        vec![text("¥3,980")],
    )
}

/// サイズ選択の radio card 1 件（説明文付き）。ネイティブ disabled の
/// まま用いる（モジュール doc「サイズ選択を radio card + ネイティブ
/// disabled にする理由」節参照）。
fn size_item(checked: bool, value: &'static str, label: &str, description: &str) -> Node {
    radio_card::item(
        checked,
        true,
        value,
        vec![],
        vec![
            radio_card::item_hidden_input(checked, true, Some(SIZE_NAME), value, vec![]),
            radio_card::item_control(
                checked,
                true,
                vec![],
                vec![
                    radio_card::item_indicator(checked, true, false, vec![]),
                    radio_card::item_content(
                        vec![],
                        vec![
                            radio_card::item_text(vec![], vec![text(label)]),
                            radio_card::item_description(vec![], vec![text(description)]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// サイズ選択欄（2 択、説明文付き。350 mL を選択済みで固定）。
fn size_options() -> Node {
    let items = vec![
        size_item(true, "350ml", "350 mL", "通勤バッグに収まる軽量サイズ"),
        size_item(
            false,
            "500ml",
            "500 mL",
            "一日の水分補給に十分な大容量サイズ",
        ),
    ];
    div(
        vec![("class", "blocks-product-overview-featured-split-option")],
        vec![
            radio_card::label(Some(SIZE_LABEL_ID), vec![], vec![text("サイズ")]),
            radio_card::root(
                Size::Sm,
                ColorPalette::Accent,
                true,
                None::<RadioCardOrientation>,
                Some(SIZE_LABEL_ID),
                vec![("aria-disabled", "true")],
                items,
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("現在の選択: 350 mL")],
            ),
        ],
    )
}

/// 在庫行・保証行で共有するチェック線アイコン（独自の単純な幾何形状。
/// 実在アイコンセット・ブランドの意匠は持ち込まない）。装飾用途のため
/// `aria-hidden` を付与する（[`IconProps::label`] を `None` のまま使う）。
fn check_icon() -> Node {
    icon::icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![("class", "blocks-product-overview-featured-split-icon-stock")],
        vec![el("path", vec![("d", "M4 12l5 5L20 6")], vec![])],
    )
}

/// 在庫行・保証行で共有する盾形アイコン（独自の単純な幾何形状）。
fn shield_icon() -> Node {
    icon::icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![(
            "class",
            "blocks-product-overview-featured-split-icon-warranty",
        )],
        vec![el(
            "path",
            vec![("d", "M12 3l7 3v6c0 5-3.5 8-7 9-3.5-1-7-4-7-9V6z")],
            vec![],
        )],
    )
}

/// 在庫表示行（icon + text）。
fn stock_row() -> Node {
    div(
        vec![("class", "blocks-product-overview-featured-split-stock")],
        vec![
            check_icon(),
            styled_text::text(
                &TextProps::default(),
                vec![],
                vec![text("在庫あり・2〜3 営業日で発送")],
            ),
        ],
    )
}

/// カート追加ボタン（全幅、既定 `type="button"`。モジュール doc「購入
/// ボタンを disabled の静的表示にする理由」節参照）。
fn add_to_cart_button() -> Node {
    button::button(
        &ButtonProps {
            size: Size::Lg,
            disabled: true,
            ..ButtonProps::default()
        },
        vec![("data-blocks-product-overview-featured-split-add", "")],
        vec![text("カートに追加")],
    )
}

/// 保証の補足行（icon + text、Muted・Sm）。
fn warranty_row() -> Node {
    div(
        vec![("class", "blocks-product-overview-featured-split-warranty")],
        vec![
            shield_icon(),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("購入から 1 年間の品質保証付き")],
            ),
        ],
    )
}

/// 左上（summary 領域）: パンくず → 商品名 → 価格 → 評価 → 説明。
fn summary() -> Node {
    div(
        vec![("class", "blocks-product-overview-featured-split-summary")],
        vec![
            product_breadcrumb(),
            heading::heading(
                HeadingLevel::H2,
                &HeadingProps {
                    size: HeadingSize::Xl,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("保温ステンレスボトル")],
            ),
            price_line(),
            rating_row(),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "二重構造の真空断熱で保温・保冷が長時間続くステンレスボトルです。シンプルな見た目で普段使いしやすく、オフィスにもアウトドアにも馴染みます。",
                )],
            ),
        ],
    )
}

/// 右（media 領域）: 単一の大きな商品画像。
fn media() -> Node {
    div(
        vec![("class", "blocks-product-overview-featured-split-media")],
        vec![image::image(
            &ImageProps {
                aspect_ratio: AspectRatio::Square,
                ..ImageProps::new(
                    dummy_assets::PRODUCT_SRC,
                    "保温ステンレスボトル の商品画像（プレースホルダー）",
                )
            },
            vec![("data-blocks-product-overview-featured-split-image", "")],
        )],
    )
}

/// 左下（details 領域）: サイズ選択 → 在庫表示 → カート追加ボタン →
/// 保証の補足。
fn details() -> Node {
    div(
        vec![("class", "blocks-product-overview-featured-split-details")],
        vec![
            size_options(),
            stock_row(),
            add_to_cart_button(),
            warranty_row(),
        ],
    )
}

/// `product-overview-featured-split` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数（`crate::blocks` モジュール doc「静的表示」節）。
/// DOM 順は `summary → media → details`（モジュール doc「レイアウト」
/// 節参照）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-product-overview-featured-split-layout")],
        vec![summary(), media(), details()],
    )
}
```

## 原案差分メモ

- 主参照・集約元ともに対応表 ID R1177 の 1 件のみのため、状態違い
  （在庫切れ版等）の併記は行っていません。
- `48rem` のブレークポイントは CSS メディアクエリのため、同一ページ内で
  狭い幅のレイアウトを静的に再現することはできません。実機確認は
  サンドボックス制約により未実施で、`cargo test` による出力検証のみで
  代替しました。
- 狭い幅での画像の挟み込みは `display: none` や DOM の重複ではなく、
  CSS Grid の `grid-template-areas` の切り替えのみで実現しています
  （DOM 順は常に `summary → media → details`）。
- `_/blocks-intake/` の参照ファイルは本イシュー着手時点で worktree に
  存在しないため、対応表 ID のみを記して実装しました。

関連情報: [Breadcrumb](../themes/breadcrumb.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Image](../themes/image.md) /
[Radio Card](../themes/radio-card.md) / [Rating Group](../themes/rating-group.md) /
[Button](../themes/button.md) / [Icon](../themes/icon.md)
