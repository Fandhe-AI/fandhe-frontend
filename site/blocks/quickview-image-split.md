# quickview-image-split

商品一覧から開く商品クイックビューのブロックです。ダイアログ内を左右 2
カラムに分け、左に商品画像、右に商品名・価格・評価・色選択・サイズ選択・
カート追加ボタンを置きます。`dialog` / `image` / `text` / `rating-group` /
`radio-card` / `color-swatch` / `button` / `link` の 8 部品を合成します。
Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R1181（集約元 R1182/R1183/R1184）です。商品名・価格・
評価・色名・サイズはすべて架空のデータであり、実在のブランド・商品・PII
は含みません。商品画像はビルド時生成の同梱プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、開閉・選択・カート追加の処理を
持ちません。`<form>` は含まず、操作要素はすべてネイティブ `disabled` に
しています（詳細はモジュール doc 参照）。狭い幅（コンテナ幅 40rem 未満）
では画像が上に積まれます。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::color_swatch::{
    self, Color, ColorSwatchProps, Rgb, SwatchShape,
};
use fandhe_frontend_pre_styled_ui::dialog::{self, ContentIds, DialogRole, OpenState};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::radio_card::{self, Orientation as RadioCardOrientation};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::ColorPalette;

/// [`dialog::content`] の `id`。
const CONTENT_ID: &str = "blocks-quickview-image-split-content";
/// [`dialog::title`]（商品名）の `id`（[`dialog::content`] の
/// `labelledby` と対）。
const TITLE_ID: &str = "blocks-quickview-image-split-title";
/// 評価ラベル（`rating_group::label` の `id`）。
const RATING_LABEL_ID: &str = "blocks-quickview-image-split-rating-label";
/// 色選択見出し（`radio_card::label` の `id`）。
const COLOR_LABEL_ID: &str = "blocks-quickview-image-split-color-label";
/// サイズ選択見出し（`radio_card::label` の `id`）。
const SIZE_LABEL_ID: &str = "blocks-quickview-image-split-size-label";
/// 色選択 radio card のネイティブ `name`。
const COLOR_NAME: &str = "blocks-quickview-image-split-color";
/// サイズ選択 radio card のネイティブ `name`。
const SIZE_NAME: &str = "blocks-quickview-image-split-size";
/// 詳細リンクの遷移先。同じ `/blocks/` 索引配下の商品詳細 block へ向ける
/// （モジュール doc「詳細リンク」節参照、`href="#"` は使わない）。
const PRODUCT_DETAIL_HREF: &str = "../product-overview-gallery-split/";

/// 色見本の RGB 定義（架空の色名に対応する任意の色、実在ブランドカラーを
/// 模したものではない。`product_overview_gallery_split.rs::swatch_color`
/// と同型）。
fn swatch_color(hex: (u8, u8, u8)) -> Color {
    Color::from_rgb(Rgb::new(hex.0, hex.1, hex.2))
}

/// 色・サイズ選択共通の radio card 1 件（ネイティブ disabled のまま用いる、
/// モジュール doc「色選択・サイズ選択」節参照）。`swatch` が `Some` のとき
/// のみ色見本を item-content の先頭へ置く。
fn option_item(
    name: &str,
    checked: bool,
    value: &'static str,
    label: &str,
    swatch: Option<Color>,
) -> Node {
    let mut content_children: Vec<Node> = Vec::new();
    if let Some(color) = swatch {
        content_children.push(color_swatch::color_swatch(
            &ColorSwatchProps {
                value: color,
                size: Size::Sm,
                shape: SwatchShape::Circle,
            },
            vec![("aria-hidden", "true")],
            vec![],
        ));
    }
    content_children.push(radio_card::item_text(vec![], vec![text(label)]));

    radio_card::item(
        checked,
        true,
        value,
        vec![],
        vec![
            radio_card::item_hidden_input(checked, true, Some(name), value, vec![]),
            radio_card::item_control(
                checked,
                true,
                vec![],
                vec![
                    radio_card::item_indicator(checked, true, false, vec![]),
                    radio_card::item_content(vec![], content_children),
                ],
            ),
        ],
    )
}

/// 色・サイズ選択欄 1 個（見出し + radio card 群 + 現在の選択の明文化）。
fn option_group(
    label_id: &str,
    label_text: &str,
    items: Vec<Node>,
    selected_summary: &str,
) -> Node {
    div(
        vec![("class", "blocks-quickview-image-split-option")],
        vec![
            radio_card::label(Some(label_id), vec![], vec![text(label_text)]),
            radio_card::root(
                Size::Sm,
                ColorPalette::Accent,
                true,
                None::<RadioCardOrientation>,
                Some(label_id),
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
                vec![text(selected_summary)],
            ),
        ],
    )
}

/// 色選択欄（3 択、「グレー」を選択済みで固定）。
fn color_options() -> Node {
    let items = vec![
        option_item(
            COLOR_NAME,
            true,
            "gray",
            "グレー",
            Some(swatch_color((0x6b, 0x6f, 0x76))),
        ),
        option_item(
            COLOR_NAME,
            false,
            "navy",
            "ネイビー",
            Some(swatch_color((0x1f, 0x2d, 0x4a))),
        ),
        option_item(
            COLOR_NAME,
            false,
            "sand",
            "サンド",
            Some(swatch_color((0xd8, 0xc9, 0xaa))),
        ),
    ];
    option_group(COLOR_LABEL_ID, "カラー", items, "現在の選択: グレー")
}

/// サイズ選択欄（4 択、「M」を選択済みで固定）。
fn size_options() -> Node {
    let items = vec![
        option_item(SIZE_NAME, false, "s", "S", None),
        option_item(SIZE_NAME, true, "m", "M", None),
        option_item(SIZE_NAME, false, "l", "L", None),
        option_item(SIZE_NAME, false, "xl", "XL", None),
    ];
    option_group(SIZE_LABEL_ID, "サイズ", items, "現在の選択: M")
}

/// 評価行（readonly、モジュール doc「評価」節参照）。
fn rating_row() -> Node {
    let rating_props = RatingGroupProps {
        disabled: false,
        readonly: true,
        required: false,
    };
    let state = RatingGroup::new(5, Some(4), true);
    let label = rating_group::label(
        &rating_props,
        Some(RATING_LABEL_ID),
        vec![],
        vec![text("評価 4.0（126 件）")],
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
    let control = rating_group::control(&rating_props, Some(RATING_LABEL_ID), vec![], items);
    div(
        vec![("class", "blocks-quickview-image-split-rating")],
        vec![rating_group::root(
            Size::Sm,
            ColorPalette::Accent,
            &rating_props,
            vec![],
            vec![label, control],
        )],
    )
}

/// 商品画像（正方形・`object-fit: cover`）。
fn product_image() -> Node {
    image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio: AspectRatio::Square,
            shape: ImageShape::Rounded,
            ..ImageProps::new(dummy_assets::PRODUCT_SRC, "キャンバス トートバッグ 本体")
        },
        vec![("data-blocks-quickview-image-split-image", "")],
    )
}

/// 価格（素の `<p>`。モジュール doc「価格を素の `<p>` で組み立てる理由」
/// 節参照）。
fn price_line() -> Node {
    el(
        "p",
        vec![("class", "blocks-quickview-image-split-price")],
        vec![text("¥6,600")],
    )
}

/// 詳細リンク（モジュール doc「詳細リンク」節参照）。
fn detail_link() -> Node {
    link::root(
        PRODUCT_DETAIL_HREF,
        &LinkProps::default(),
        vec![],
        vec![text("商品詳細ページの例を見る")],
    )
}

/// カート追加ボタン（全幅、`ButtonProps::disabled` で disabled 属性一式を
/// 自動付与、モジュール doc「開く・閉じる・カート追加・選択は無 JS で
/// no-op のため `disabled`」節参照）。
fn add_to_cart_button() -> Node {
    button(
        &ButtonProps {
            disabled: true,
            ..ButtonProps::default()
        },
        vec![("data-blocks-quickview-image-split-add", "")],
        vec![text("カートに追加")],
    )
}

/// `quickview-image-split` の Demo 本体。左に商品画像・右に商品情報+選択
/// パネルを置く静的開状態のダイアログ 1 件で構成する。呼び出しごとに
/// 同一の `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-quickview-image-split-stack")],
        vec![dialog::root(
            Size::Lg,
            OpenState::Open,
            vec![("data-blocks-quickview-image-split-root", "")],
            vec![
                dialog::backdrop(OpenState::Open, vec![], vec![]),
                dialog::positioner(
                    OpenState::Open,
                    vec![],
                    vec![dialog::content(
                        OpenState::Open,
                        DialogRole::Dialog,
                        false,
                        ContentIds {
                            id: Some(CONTENT_ID),
                            labelledby: Some(TITLE_ID),
                            describedby: None,
                        },
                        vec![],
                        vec![
                            dialog::close_trigger(
                                vec![
                                    ("aria-label", "閉じる"),
                                    ("disabled", ""),
                                    ("data-disabled", ""),
                                ],
                                vec![text("×")],
                            ),
                            div(
                                vec![("class", "blocks-quickview-image-split-grid")],
                                vec![
                                    product_image(),
                                    div(
                                        vec![("class", "blocks-quickview-image-split-panel")],
                                        vec![
                                            dialog::title(
                                                Some(TITLE_ID),
                                                vec![],
                                                vec![text("キャンバス トートバッグ")],
                                            ),
                                            price_line(),
                                            rating_row(),
                                            color_options(),
                                            size_options(),
                                            add_to_cart_button(),
                                            detail_link(),
                                        ],
                                    ),
                                ],
                            ),
                        ],
                    )],
                ),
            ],
        )],
    )
}
```

## 原案差分メモ

- サイズガイドへのリンクは docs サイトに実在の遷移先ページがないため
  Demo には置いていません（`href="#"` を避ける方針、モジュール doc
  「詳細リンク」節参照）。詳細リンクは実在する相対パス（商品詳細
  block のページ）へ向けています。
- R1182（集約元）は詳細リンクを持たず、サイズ選択が 8 段あります。本
  block の `size_options` の選択肢を増やすだけで同じ構造になります。
- R1183（集約元）は色選択を持たず、説明付きの大きいサイズカード
  （`radio_card::item_description`）を使う構成です。
- R1184（集約元）はサイズ選択グループを丸ごと持たない構成です。

関連情報: [Dialog](../themes/dialog.md) / [Image](../themes/image.md) /
[Text](../themes/text.md) / [Rating Group](../themes/rating-group.md) /
[Radio Card](../themes/radio-card.md) /
[Color Swatch](../themes/color-swatch.md) / [Button](../themes/button.md) /
[Link](../themes/link.md)
