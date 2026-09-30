# product-overview-gallery-split

`fandhe-frontend-pre-styled-ui` の `image` / `carousel` / `breadcrumb` /
`heading` / `text` / `rating-group` / `radio-card` / `color-swatch` /
`button` / `accordion` 部品を合成した、サムネイル付きギャラリーと購入パネル
を持つ商品詳細の実例です。Blocks セクションは新規部品を追加するものでは
なく、既存の Themes/Primitives 部品を組み合わせた実例集であることに注意
してください（主参照は対応表 ID R0611、集約元は R0613・R0614・R0615・
R0616・R0617・R1176。出典の固有名・ファイル名は記載しません）。本 block
は親イシュー #3067「商品詳細（サムネイル付きギャラリー + 購入パネル）」
の前半（#3068）にあたり、骨格と主要領域のみを実装しています。

広い画面では左にメイン画像 + 横並びのサムネイルカルーセル、右にパンくず・
商品名・評価・価格・カラー選択・サイズ選択・カート追加ボタン・詳細情報
アコーディオンをまとめた購入パネルを 2 カラムで並べます。`48rem` 未満の
狭い画面ではギャラリーの下へ購入パネルを縦に積みます（`display: none` は
使わないため、狭幅でもすべての操作要素へ到達できます）。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。カート追加ボタンは `type="button"` のまま
送信先を持たず、カラー・サイズの選択欄はネイティブ `disabled` のまま
固定表示し、現在の選択を本文テキストで明文化しています。詳細情報
アコーディオンは無 JS のため開閉できず、3 項目すべてを開いた状態のまま
固定しています（閉じた項目を残すと本文が到達不能になるため）。パンくずの
リンクは `href="#"` を避け、実在する相対パスへ向けています。商品名・
価格・評価件数・カラー名・サイズ・説明文はすべて独自に書いた架空のもので
あり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::accordion::{
    self, item, item_content, item_indicator, item_trigger, AccordionProps, OpenState,
};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::carousel::{self, Orientation};
use fandhe_frontend_pre_styled_ui::color_swatch::{self, Color, ColorSwatchProps, Rgb};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::radio_card::{self, Orientation as RadioCardOrientation};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 評価ラベル（`rating_group::label` の `id`）。
const RATING_LABEL_ID: &str = "blocks-product-overview-gallery-split-rating-label";
/// 色選択見出し（`radio_card::label` の `id`）。
const COLOR_LABEL_ID: &str = "blocks-product-overview-gallery-split-color-label";
/// サイズ選択見出し（`radio_card::label` の `id`）。
const SIZE_LABEL_ID: &str = "blocks-product-overview-gallery-split-size-label";
/// 色選択 radio card のネイティブ `name`（フォーム未送信のため排他選択の
/// 実効はないが、[`radio_card::item_hidden_input`] の契約上必須）。
const COLOR_NAME: &str = "blocks-product-overview-gallery-split-color";
/// サイズ選択 radio card のネイティブ `name`。
const SIZE_NAME: &str = "blocks-product-overview-gallery-split-size";

/// パンくず。`href="#"` は使わず実在する相対パスへ向ける
/// （モジュール doc「`<form>` を持たない」節参照）。中間項目は実在しない
/// カテゴリ名を騙らず、実際の遷移先（`/blocks/`）と一致する「Blocks」を
/// ラベルにする（`page_heading_meta.rs`・`help_center_article_list.rs` と
/// 同じ判断、レビュー指摘対応）。
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
                        vec![text("ワイヤレスヘッドホン")],
                    )],
                ),
            ],
        )],
    )
}

/// ギャラリーのサムネイル 1 枚分（`carousel::item` + [`image::image`]）。
/// 装飾ではなくカルーセルの主要コンテンツのため `alt` を空文字列にしない
/// （`gallery_split_carousel.rs::slide` と同じ判断）。すべて単一商品の
/// 画像であるため [`dummy_assets::PRODUCT_SRC`] のみを使う
/// （`cart_two_column_summary.rs`・`order_tracking_progress.rs` と同じ
/// 判断。`BACKGROUND_SRC`/`SCREENSHOT_SRC`/`LOGO_SRC` は無関係カテゴリの
/// プレースホルダーのため商品画像としては使わない、レビュー指摘対応）。
///
/// `carousel::item` の `current` 引数は `true` を渡す（レビュー指摘対応、
/// Codex）。[`LAYOUT_CSS`] の上書き（「サムネイル `carousel` の item
/// 上書き」節参照）により 4 枚全てが常時横並びで画面に見えており、
/// 1 枚だけを表示するスライドショー（1 枚だけが実際に viewport 内）を
/// 前提にした `index == 0` 判定では `data-inview`/`data-current` が
/// 先頭 1 枚にしか付かず、実際の表示状態（4 枚とも常に可視）と
/// カルーセルの状態表現が食い違う。`item` パーツの `data-current`
/// （`data-inview` も同一 bool から生成される、
/// `fandhe_frontend_headless_ui::carousel::item` 参照）には styled
/// carousel のスタイル規則が存在しない（強調は `indicator` パーツのみ）
/// ため、全枚 `true` にしても見た目の変化はなく、可視状態の表現のみが
/// 実態に合う。
fn thumbnail(index: usize, count: usize) -> Node {
    let src = dummy_assets::PRODUCT_SRC;
    let alt = format!("商品画像{}", index + 1);
    carousel::item(
        Orientation::Horizontal,
        index,
        count,
        true,
        vec![],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Square,
                shape: ImageShape::Rounded,
                ..ImageProps::new(src, &alt)
            },
            vec![("data-blocks-product-overview-gallery-split-thumb", "")],
        )],
    )
}

/// 左列（ギャラリー）: メイン画像 + サムネイル `carousel`。前後トリガーは
/// 出さない（モジュール doc「ギャラリー領域」節参照）。
fn gallery() -> Node {
    const COUNT: usize = 4;
    let thumbs: Vec<Node> = (0..COUNT).map(|i| thumbnail(i, COUNT)).collect();

    div(
        vec![("class", "blocks-product-overview-gallery-split-gallery")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    aspect_ratio: AspectRatio::Square,
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, "ワイヤレスヘッドホン 本体")
                },
                vec![("data-blocks-product-overview-gallery-split-main-image", "")],
            ),
            carousel::root(
                Size::Sm,
                Orientation::Horizontal,
                "商品画像のサムネイル",
                vec![],
                vec![carousel::control(
                    Orientation::Horizontal,
                    vec![],
                    vec![carousel::item_group(
                        Orientation::Horizontal,
                        vec![("class", "blocks-product-overview-gallery-split-thumbs")],
                        thumbs,
                    )],
                )],
            ),
        ],
    )
}

/// 評価行（`rating-group`、readonly。現在の評価をラベルで明文化する、
/// `card_meta_cta.rs::product_card` と同型の判断）。
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
        vec![text("評価 4.0（48 件）")],
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
        vec![("class", "blocks-product-overview-gallery-split-rating")],
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
        vec![("class", "blocks-product-overview-gallery-split-price")],
        vec![text("¥24,800")],
    )
}

/// 色見本の RGB 定義（架空の色名に対応する任意の色。実在ブランドカラーを
/// 模したものではない）。
fn swatch_color(hex: (u8, u8, u8)) -> Color {
    Color::from_rgb(Rgb::new(hex.0, hex.1, hex.2))
}

/// 色選択・サイズ選択共通の radio card 1 件を組み立てる。ネイティブ
/// disabled のまま用いる（モジュール doc「色・サイズ選択を radio card +
/// ネイティブ disabled にする理由」節参照）。`swatch` が `Some` のときのみ
/// 色見本を item-content の先頭へ置く（色選択専用）。
fn option_item(
    name: &'static str,
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
                ..ColorSwatchProps::default()
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
/// ネイティブ `name` は [`option_item`] が組み立てる
/// `radio_card::item_hidden_input` へ既に渡し込み済みのため、本関数は
/// 受け取らない。
fn option_group(
    class: &'static str,
    label_id: &'static str,
    label_text: &str,
    items: Vec<Node>,
    selected_summary: &str,
) -> Node {
    div(
        vec![("class", class)],
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

/// 色選択欄（3 択、2 番目を選択済みで固定）。
fn color_options() -> Node {
    let items = vec![
        option_item(
            COLOR_NAME,
            false,
            "charcoal",
            "チャコール",
            Some(swatch_color((0x33, 0x33, 0x36))),
        ),
        option_item(
            COLOR_NAME,
            true,
            "ivory",
            "アイボリー",
            Some(swatch_color((0xf1, 0xea, 0xdd))),
        ),
        option_item(
            COLOR_NAME,
            false,
            "forest",
            "フォレストグリーン",
            Some(swatch_color((0x2f, 0x4f, 0x3c))),
        ),
    ];
    option_group(
        "blocks-product-overview-gallery-split-option",
        COLOR_LABEL_ID,
        "カラー",
        items,
        "現在の選択: アイボリー",
    )
}

/// サイズ選択欄（4 択、M を選択済みで固定）。
fn size_options() -> Node {
    let items = vec![
        option_item(SIZE_NAME, false, "s", "S", None),
        option_item(SIZE_NAME, true, "m", "M", None),
        option_item(SIZE_NAME, false, "l", "L", None),
        option_item(SIZE_NAME, false, "xl", "XL", None),
    ];
    option_group(
        "blocks-product-overview-gallery-split-option",
        SIZE_LABEL_ID,
        "サイズ",
        items,
        "現在の選択: M",
    )
}

/// カート追加ボタン（全幅、既定 `type="button"`。モジュール doc「`<form>`
/// を持たない」節参照）。
fn add_to_cart_button() -> Node {
    button::button(
        &ButtonProps {
            size: Size::Lg,
            ..ButtonProps::default()
        },
        vec![("data-blocks-product-overview-gallery-split-add", "")],
        vec![text("カートに追加")],
    )
}

/// 詳細アコーディオン 1 項目（トリガー + 本文）。全件 `OpenState::Open` +
/// `disabled: true` で固定する（モジュール doc「詳細アコーディオンを全件
/// open + disabled で固定する理由」節参照）。
fn detail_item(index: usize, title: &str, body: &str) -> Node {
    let state = OpenState::Open;
    let props = AccordionProps {
        disabled: true,
        ..AccordionProps::default()
    };
    let trigger_id = format!("blocks-product-overview-gallery-split-detail-{index}-trigger");
    let content_id = format!("blocks-product-overview-gallery-split-detail-{index}-content");

    item(
        state,
        false,
        &props,
        vec![],
        vec![
            item_trigger(
                state,
                false,
                &props,
                title,
                Some(trigger_id.as_str()),
                Some(content_id.as_str()),
                vec![],
                vec![
                    fandhe_frontend_core::span(vec![], vec![text(title)]),
                    item_indicator(state, false, &props, vec![], vec![text("\u{25be}")]),
                ],
            ),
            item_content(
                state,
                false,
                &props,
                Some(content_id.as_str()),
                Some(trigger_id.as_str()),
                vec![],
                vec![styled_text::text(
                    &TextProps::default(),
                    vec![],
                    vec![text(body)],
                )],
            ),
        ],
    )
}

/// 詳細情報アコーディオン（説明・素材と手入れ・配送と返品の 3 項目）。
fn details() -> Node {
    let items = vec![
        detail_item(
            0,
            "説明",
            "密閉型のワイヤレスヘッドホンです。長時間の装着でも疲れにくい軽量設計で、通勤・在宅ワークの双方に向いています。",
        ),
        detail_item(
            1,
            "素材と手入れ",
            "イヤーパッドは合成皮革を使用しています。乾いた柔らかい布で軽く拭き取ってください。",
        ),
        detail_item(
            2,
            "配送と返品",
            "通常 3〜5 営業日でお届けします。未使用品に限り到着後 14 日以内の返品を承ります。",
        ),
    ];
    accordion::root(
        Size::Md,
        &AccordionProps::default(),
        vec![("data-blocks-product-overview-gallery-split-details", "")],
        items,
    )
}

/// 右列（購入パネル）: パンくず → 商品名 → 評価 → 価格 → 色/サイズ選択 →
/// カート追加ボタン → 詳細アコーディオン。
fn purchase_panel() -> Node {
    div(
        vec![("class", "blocks-product-overview-gallery-split-panel")],
        vec![
            product_breadcrumb(),
            heading::heading(
                HeadingLevel::H2,
                &HeadingProps {
                    size: HeadingSize::Xl,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("ワイヤレスヘッドホン Aria")],
            ),
            rating_row(),
            price_line(),
            color_options(),
            size_options(),
            add_to_cart_button(),
            details(),
        ],
    )
}

/// `product-overview-gallery-split` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数（`crate::blocks` モジュール doc「静的表示」節）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-product-overview-gallery-split-layout")],
        vec![gallery(), purchase_panel()],
    )
}
```

## 原案差分メモ

- 主参照は R0611（メイン画像 + サムネイルカルーセル + 購入パネル）です。
  本 PR（#3068）では骨格と主要領域のみを実装し、集約元 R0613（単一画像 +
  枚数表示・丸色見本）・R0614（タブ詳細）・R0615（段落列 + 共有行）・
  R0616（縦サムネ列）・R0617（評価なし + サイズガイド統合）・R1176
  （サムネ切替タブ + 開閉式詳細）との差分併記は後半 #3069 へ持ち越して
  います。
- 在庫僅少警告（`progress`）・サイズガイド（`dialog`）・タブ型詳細
  （`tabs`）・共有リンク行は #3069 の担当のため、本 block の `parts` には
  含めていません（Demo が実際に使う部品のみを列挙する契約のため）。
- ギャラリーの前後トリガー（prev/next）は出していません。無 JS のため
  実際にはスライド送りを配線できず、購入パネル側に既に無効化ボタン・
  チェックボックスを多数持つため、ギャラリー側は視覚ノイズを避けて
  「1 枚目が選択済み」の静的表示のみとしています。
- カラー・サイズの選択は `radio_card` をネイティブ `disabled` のまま用い、
  現在の選択を本文テキストで明文化しています（`pricing_single_split.rs`
  の支払周期選択と同型の判断）。色見本は `color_swatch`（`aria-hidden`）
  を選択肢名と併記する装飾として使っています。
- 詳細情報アコーディオンは 3 項目（説明・素材と手入れ・配送と返品）を
  すべて開いた状態 + disabled で固定しています（`faq_split_accordion.rs`
  等の前例と同じ判断。無 JS では閉じた項目が到達不能になるため）。
- `_/blocks-intake/` の参照ファイルは本イシュー着手時点で worktree に
  存在しないため、対応表 ID のみを記して実装しました。
- ブラウザでの実機確認（`48rem` 前後のコンテナ幅切替・ライト/ダーク両
  テーマ）はサンドボックス制約により未実施です。cargo test による出力
  検証のみで代替しました。
