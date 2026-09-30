# product-overview-image-grid

上段に商品画像グリッド（段差配置）、下段に購入パネル（商品名・価格・評価・
色/サイズ選択・カート追加ボタン・説明）を持つ商品詳細画面のブロックです。
`breadcrumb` / `image` / `heading` / `text` / `rating-group` / `radio-card` /
`color-swatch` / `button` の 8 部品を合成します。Blocks は既存部品の
合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R1178（集約元 R1175: 段差配置 + 右に購入パネル / R0612:
2 列グリッド・左右反転）です。商品名・価格・レビュー件数・色/サイズ・
説明はすべて架空のデータであり、実在のブランド・商品・PII は含みません。
商品画像はビルド時生成の同梱プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。色・サイズの
選択は `disabled` の radio card による初期選択のみの静的表示です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::color_swatch::{self, Color, ColorSwatchProps, Rgb};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::radio_card::{self, Orientation};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::Size;

/// 評価ラベル（`rating_group::label` の `id`）。Demo は 1 ページ 1 block
/// のため重複しない。
const RATING_LABEL_ID: &str = "blocks-product-overview-image-grid-rating-label";
/// 色選択グループの見出し `id`（`radio_card::root` の `labelled_by`）。
const COLOR_LABEL_ID: &str = "blocks-product-overview-image-grid-color-label";
/// サイズ選択グループの見出し `id`（`radio_card::root` の `labelled_by`）。
const SIZE_LABEL_ID: &str = "blocks-product-overview-image-grid-size-label";

/// 架空の商品名。実在のブランド・商品を含まない。
const PRODUCT_NAME: &str = "プレミアム ワイヤレスヘッドホン";
/// 架空の価格表示（円建て固定）。
const PRICE_DISPLAY: &str = "¥32,800";
/// 架空の説明文（2〜3 文）。
const DESCRIPTION: &[&str] = &[
    "長時間の使用でも疲れにくい軽量設計と、周囲の音を抑えるノイズ\
キャンセリング機能を両立したワイヤレスヘッドホンです。",
    "満充電で最大 30 時間再生でき、外出先でも安心してお使いいただけます。",
];

/// 架空の色選択肢（値, 表示名, RGB）。1 件目（チャコール）を初期選択に
/// 固定する。実在のブランド固有色は使わない。
const COLOR_OPTIONS: &[(&str, &str, (u8, u8, u8))] = &[
    ("charcoal", "チャコール", (0x2b, 0x2b, 0x2b)),
    ("silver", "シルバー", (0xc4, 0xc4, 0xc4)),
    ("navy", "ネイビー", (0x1f, 0x3a, 0x5f)),
];

/// 架空のサイズ選択肢。2 件目（M）を初期選択に固定する。
const SIZE_OPTIONS: &[&str] = &["S", "M", "L", "XL"];

/// 画像グリッドの 1 枚（`index` は 0 始まり、`total` は総枚数）。3 列
/// グリッドに対する明示配置を [`LAYOUT_CSS`] の段差配置（2×2 + 下段 1 枚
/// 全幅）のフックにする: 1 枚目（`index == 0`）は
/// `data-blocks-product-overview-image-grid-tile="hero"`（2 列×2 行）、
/// 最後の 1 枚（`index == total - 1`）は
/// `data-blocks-product-overview-image-grid-tile="wide"`（3 列全幅の下段）。
/// 中間の画像は `grid-auto-flow` の暗黙配置に任せ、hero が占有した後の
/// 残り 2 セル（1・2 行目の 3 列目）へ収まる。この 3 分類により、3 列
/// グリッド上で「1 枚目が 2×2」「最後の 1 枚が意図せず単独の行になる」
/// 不整合（PR #3468 レビュー P1 指摘）を、全画像の配置を明示することで
/// 解消する。
fn gallery_tile(index: usize, total: usize) -> Node {
    // 4 枚すべて同一の `dummy_assets::PRODUCT_SRC`（同一プレースホルダー画像）
    // を参照しているため、「画像 1」〜「画像 4」のように異なる商品写真で
    // あるかのような alt を付けるとスクリーンリーダーで同一画像が異なる
    // 写真として読み上げられてしまう（PR #3468 レビュー P2 指摘）。実際の
    // 内容と一致する alt を持てるのは代表画像（1 枚目）のみとし、残り
    // 3 枚は代表画像の重複描画（装飾目的）として空 alt にする。
    let alt = if index == 0 {
        PRODUCT_NAME.to_string()
    } else {
        String::new()
    };
    let mut props = ImageProps::new(dummy_assets::PRODUCT_SRC, &alt);
    props.aspect_ratio = AspectRatio::Square;
    props.shape = ImageShape::Rounded;
    let attrs = if index == 0 {
        vec![("data-blocks-product-overview-image-grid-tile", "hero")]
    } else if index == total - 1 {
        vec![("data-blocks-product-overview-image-grid-tile", "wide")]
    } else {
        vec![]
    };
    image(&props, attrs)
}

/// 上段の画像グリッド（4 枚、1 枚目を 2×2 で大きく・最後の 1 枚を 3 列
/// 全幅の下段に明示配置 = 段差配置。主参照 R1178、集約元 R1175 の段差
/// 配置の要素のみを骨格に取り込む）。
fn gallery() -> Node {
    const TOTAL: usize = 4;
    let tiles = (0..TOTAL).map(|i| gallery_tile(i, TOTAL)).collect();
    div(
        vec![("class", "blocks-product-overview-image-grid-gallery")],
        tiles,
    )
}

/// パンくず（Home → Blocks → 現在の商品名。`app_shell_stacked` と同型の
/// 相対パス構成: ページは `/blocks/product-overview-image-grid/` に生成
/// されるため `../../` はサイトルート、`../` は `/blocks/` を指す。2 階層目
/// のラベルは実際の遷移先 `/blocks/` に合わせて「Blocks」とする
/// （「Shop」ラベルで `/blocks/` 索引へ遷移していたラベルと遷移先の不一致
/// 〔PR #3468 レビュー P2 指摘〕の是正）。
fn breadcrumb_nav() -> Node {
    breadcrumb::root(
        Size::Sm,
        BreadcrumbVariant::default(),
        Some("Breadcrumb"),
        vec![],
        vec![breadcrumb::list(
            vec![],
            vec![
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../../", vec![], vec![text("Home")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::link("../", vec![], vec![text("Blocks")])],
                ),
                breadcrumb::separator(vec![], vec![text("/")]),
                breadcrumb::item(
                    vec![],
                    vec![breadcrumb::current_link(vec![], vec![text(PRODUCT_NAME)])],
                ),
            ],
        )],
    )
}

/// 評価（readonly の 5 段 `rating_group`）+ レビュー件数（可視テキスト）の
/// 行。評価そのものは初期状態固定の静的表示、レビュー件数は実在する遷移先
/// を持たないためリンクにしない（上記モジュール doc「レビュー件数は
/// リンクにしない」節参照）。
fn rating_row() -> Node {
    let props = RatingGroupProps {
        disabled: false,
        readonly: true,
        required: false,
    };
    let state = RatingGroup::new(5, Some(4), true);
    let label = rating_group::label(
        &props,
        Some(RATING_LABEL_ID),
        vec![],
        vec![text("評価 4.0")],
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
    let control = rating_group::control(&props, Some(RATING_LABEL_ID), vec![], items);
    let rating = rating_group::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-product-overview-image-grid-rating", "")],
        vec![label, control],
    );
    // レビュー詳細ページ（Ecommerce / Reviews カテゴリ）は本イシュー時点で
    // block 未登録のため実在しない。以前はリンクとして実装していたが、
    // 遷移先を持たない `link` は自己参照（`href="./"`）になり遷移として
    // 機能しない指摘（PR #3468 レビュー P2）を受け、実在しないリンク先を
    // 作らず可視テキストとして表示する（`crate::blocks` 冒頭 doc
    // 「モジュール doc」不変条件の `href="#"` dead link 禁止とも整合）。
    // 実在の Reviews block が追加され次第、`link` へ差し替える（#3072）。
    let review_count =
        styled_text::text(&TextProps::default(), vec![], vec![text("レビュー 128 件")]);
    div(
        vec![("class", "blocks-product-overview-image-grid-rating-row")],
        vec![rating, review_count],
    )
}

/// 色選択肢 1 件（`radio_card::item` + `color_swatch` + ラベルテキスト）。
/// `card_form_footer` の支払方法 radio card と同じく、常に `disabled:
/// true` で静的表示にする。
fn color_item(checked: bool, value: &'static str, label: &'static str, rgb: (u8, u8, u8)) -> Node {
    let (r, g, b) = rgb;
    let swatch_props = ColorSwatchProps {
        value: Color::from_rgb(Rgb::new(r, g, b)),
        ..ColorSwatchProps::default()
    };
    radio_card::item(
        checked,
        true,
        value,
        vec![],
        vec![
            radio_card::item_hidden_input(checked, true, Some("color"), value, vec![]),
            radio_card::item_control(
                checked,
                true,
                vec![],
                vec![
                    radio_card::item_indicator(checked, true, false, vec![]),
                    radio_card::item_content(
                        vec![],
                        vec![
                            color_swatch::color_swatch(&swatch_props, vec![], vec![]),
                            radio_card::item_text(vec![], vec![text(label)]),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// 色選択欄（見出し + radio card 3 択、初期選択: チャコール）。
fn color_options() -> Node {
    div(
        vec![("class", "blocks-product-overview-image-grid-options")],
        vec![
            radio_card::label(Some(COLOR_LABEL_ID), vec![], vec![text("カラー")]),
            radio_card::root(
                Size::Sm,
                ColorPalette::Accent,
                true,
                Some(Orientation::Horizontal),
                Some(COLOR_LABEL_ID),
                vec![
                    ("aria-disabled", "true"),
                    ("data-blocks-product-overview-image-grid-color", ""),
                ],
                COLOR_OPTIONS
                    .iter()
                    .enumerate()
                    .map(|(i, (value, label, rgb))| color_item(i == 0, value, label, *rgb))
                    .collect(),
            ),
        ],
    )
}

/// サイズ選択肢 1 件（`radio_card::item`。色スウォッチは持たない）。
fn size_item(checked: bool, value: &'static str) -> Node {
    radio_card::item(
        checked,
        true,
        value,
        vec![],
        vec![
            radio_card::item_hidden_input(checked, true, Some("size"), value, vec![]),
            radio_card::item_control(
                checked,
                true,
                vec![],
                vec![
                    radio_card::item_indicator(checked, true, false, vec![]),
                    radio_card::item_content(
                        vec![],
                        vec![radio_card::item_text(vec![], vec![text(value)])],
                    ),
                ],
            ),
        ],
    )
}

/// サイズ選択欄（見出し + radio card 4 択、初期選択: M）。
fn size_options() -> Node {
    div(
        vec![("class", "blocks-product-overview-image-grid-options")],
        vec![
            radio_card::label(Some(SIZE_LABEL_ID), vec![], vec![text("サイズ")]),
            radio_card::root(
                Size::Sm,
                ColorPalette::Accent,
                true,
                Some(Orientation::Horizontal),
                Some(SIZE_LABEL_ID),
                vec![
                    ("aria-disabled", "true"),
                    ("data-blocks-product-overview-image-grid-size", ""),
                ],
                SIZE_OPTIONS
                    .iter()
                    .map(|value| size_item(*value == "M", value))
                    .collect(),
            ),
        ],
    )
}

/// 下段の購入パネル（商品名・価格・評価・色/サイズ選択・カート追加
/// ボタン・説明。主参照 R1178）。
fn purchase_panel() -> Node {
    div(
        vec![("class", "blocks-product-overview-image-grid-panel")],
        vec![
            heading(
                HeadingLevel::H2,
                &HeadingProps::default(),
                vec![],
                vec![text(PRODUCT_NAME)],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    weight: TextWeight::Bold,
                    ..TextProps::default()
                },
                vec![],
                vec![text(PRICE_DISPLAY)],
            ),
            rating_row(),
            color_options(),
            size_options(),
            button(
                &ButtonProps {
                    size: Size::Lg,
                    palette: ColorPalette::Accent,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-product-overview-image-grid-cta", "")],
                vec![text("カートに追加")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(DESCRIPTION.join(""))],
            ),
        ],
    )
}

/// `product-overview-image-grid` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-product-overview-image-grid-stack")],
        vec![breadcrumb_nav(), gallery(), purchase_panel()],
    )
}
```

## 原案差分メモ

- 上段は画像グリッド（1 枚目を 2×2 で大きく表示する段差配置）、下段は
  商品名・価格・評価・色/サイズ選択・カート追加ボタン・説明を並べた購入
  パネルです（対応表 ID R1178）。
- コンテナ幅（Demo 枠の幅、ビューポート幅ではありません）が 40rem 未満に
  なると `@container` によって画像グリッドが 1 列積みへ切り替わり、購入
  パネルはその下（DOM 順のまま）に表示されます。
- 色・サイズの選択は `radio_card` を `disabled` にした静的表示で、初期
  選択（色: チャコール、サイズ: M）のみを固定表示します。
- 集約元 R1175（右に購入パネルを配置する構成）・R0612（2 列グリッド・
  左右反転）の並記、色/サイズの選択状態違い（在庫切れ・別配色等）の並記は、
  本 PR（骨格・主要領域）の後続 #3072 で追加予定です。

関連情報: [Breadcrumb](../themes/breadcrumb.md) / [Image](../themes/image.md) /
[Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Rating Group](../themes/rating-group.md) / [Radio Card](../themes/radio-card.md) /
[Color Swatch](../themes/color-swatch.md) / [Button](../themes/button.md)
