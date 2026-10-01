# product-overview-gallery-split

`fandhe-frontend-pre-styled-ui` の `image` / `carousel` / `breadcrumb` /
`heading` / `text` / `rating-group` / `radio-card` / `color-swatch` /
`button` / `accordion` / `progress` / `dialog` / `link` 部品を合成した、
サムネイル付きギャラリーと購入パネルを持つ商品詳細の実例です。Blocks
セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0611、集約元は R0613・R0614・R0615・R0616・R0617・R1176。出典の固有名・
ファイル名は記載しません）。本 block は親イシュー #3067「商品詳細
（サムネイル付きギャラリー + 購入パネル）」の実装（#3068/#3069）です。
実物の `tabs::tabs` 部品は無 JS の docs サイトでは操作できないため使わず、
非対話の静的タブ列で代用しています（詳しくは下記参照）。

広い画面では左にギャラリー、右に購入パネル（パンくず・商品名・評価・
価格・カラー選択・サイズ選択・カート追加ボタン・詳細情報）を 2 カラムで
並べます。`48rem` 未満の狭い画面ではギャラリーの下へ購入パネルを縦に
積みます（`display: none` は使わないため、狭幅でもすべての操作要素へ
到達できます）。

Demo は構成違いの 3 版を縦に並べています。

1. **代表構成**: メイン画像 + 横並びサムネイルカルーセル、評価あり。
   価格の直下に在庫僅少警告（`progress`、残り 3 点）、サイズ選択の横に
   disabled の「サイズガイド」ボタンを置き、その下へダイアログ
   （非モーダル・静的な開状態）のプレビューを併記しています。
2. **縦サムネ列・タブ型詳細・評価なし**: サムネイルカルーセルをメイン
   画像の左に縦 1 列で置き、評価行は出しません。詳細情報は非対話の静的
   タブ列（説明/素材と手入れ/配送と返品/サイズガイド）で示し、選択中の
   「説明」パネルを直下に描画、残り 3 タブは「『〜』タブを選択した場合の
   プレビュー」として常時可視で併記しています。
3. **単一画像・枚数表示・段落詳細・共有行**: サムネイル列を持たない単一
   メイン画像に「1 / 4」の枚数表示を添え、色見本は丸形にしています。
   詳細情報は見出し付きの段落列で示し、価格の近くに共有リンク行（架空の
   ラベル、固定 href）を置いています。

いずれの版も静的な表示例であり、`<form>` 要素は一切持たず、データの
取得・送信・状態管理を行いません。カート追加ボタン・サイズガイドボタンは
`type="button"` のまま送信先を持たず、押しても何も起きないボタンを操作
可能なまま残さないため `disabled` で固定表示します。カラー・サイズの
選択欄はネイティブ `disabled` のまま固定表示し、現在の選択を本文テキスト
で明文化しています。詳細情報アコーディオン（代表構成）は無 JS のため
開閉できず、3 項目すべてを開いた状態のまま固定しています（閉じた項目を
残すと本文が到達不能になるため）。各項目のトリガーは見出し（`h3`）で
包み、支援技術の見出し移動で項目間を移動できるようにしています。
サイズガイドのダイアログは `trigger`/`close_trigger` を持たず、
`aria-modal="false"` の非モーダルな静的開状態のみを示します。パンくずの
リンクは `href="#"` を避け、実在する相対パスへ向けています。商品名・
価格・評価件数・カラー名・サイズ・説明文・サイズ表はすべて独自に書いた
架空のものであり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, h3, p, span, text, Node};
use fandhe_frontend_pre_styled_ui::accordion::{
    self, item, item_content, item_indicator, item_trigger, AccordionProps, OpenState,
};
use fandhe_frontend_pre_styled_ui::breadcrumb::{self, BreadcrumbVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::carousel::{self, Orientation};
use fandhe_frontend_pre_styled_ui::color_swatch::{
    self, Color, ColorSwatchProps, Rgb, SwatchShape,
};
use fandhe_frontend_pre_styled_ui::dialog::{
    self, ContentIds, DialogRole, OpenState as DialogOpenState,
};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::progress::{self, ProgressProps};
use fandhe_frontend_pre_styled_ui::radio_card::{self, Orientation as RadioCardOrientation};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 評価ラベル（`rating_group::label` の `id`）の接頭辞。版接尾辞を付けて
/// 一意化する（モジュール doc「id を版接尾辞で一意化する理由」節）。
const RATING_LABEL_ID: &str = "blocks-product-overview-gallery-split-rating-label";
/// 色選択見出し（`radio_card::label` の `id`）の接頭辞。
const COLOR_LABEL_ID: &str = "blocks-product-overview-gallery-split-color-label";
/// サイズ選択見出し（`radio_card::label` の `id`）の接頭辞。
const SIZE_LABEL_ID: &str = "blocks-product-overview-gallery-split-size-label";
/// 色選択 radio card のネイティブ `name` の接頭辞。
const COLOR_NAME: &str = "blocks-product-overview-gallery-split-color";
/// サイズ選択 radio card のネイティブ `name` の接頭辞。
const SIZE_NAME: &str = "blocks-product-overview-gallery-split-size";
/// 本リポジトリの固定 URL（共有リンクの href、モジュール doc「共有リンクは
/// 架空の固定 href」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// パンくず。`href="#"` は使わず実在する相対パスへ向ける
/// （モジュール doc「`<form>` を持たない」節参照）。中間項目は実際の遷移先
/// （`/blocks/`）と一致する「Blocks」をラベルにする。全版で共通のため id は
/// 持たず版接尾辞は不要。
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
/// `orientation` は親 `carousel::root`/`item_group` と揃える（版 A は
/// 横並び、版 B は縦並び）。すべて単一商品の画像であるため
/// [`dummy_assets::PRODUCT_SRC`] のみを使う。4 枚とも同一のプレースホルダー
/// 画像であり異なる商品写真ではないため、`alt` は連番を付けず全枚同一の
/// 文言にする。`current` は index 0 の 1 枚のみ true（headless carousel の
/// 「現在選択中の 1 件」契約。[`LAYOUT_CSS`] の可視性上書きとは別の関心）。
fn thumbnail(orientation: Orientation, index: usize, count: usize) -> Node {
    let src = dummy_assets::PRODUCT_SRC;
    let alt = "ワイヤレスヘッドホン 商品画像（プレースホルダー）";
    carousel::item(
        orientation,
        index,
        count,
        index == 0,
        vec![],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Square,
                shape: ImageShape::Rounded,
                ..ImageProps::new(src, alt)
            },
            vec![("data-blocks-product-overview-gallery-split-thumb", "")],
        )],
    )
}

/// メイン画像（正方形・`object-fit: cover`、全版共通）。
fn main_image() -> Node {
    image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio: AspectRatio::Square,
            shape: ImageShape::Rounded,
            ..ImageProps::new(dummy_assets::PRODUCT_SRC, "ワイヤレスヘッドホン 本体")
        },
        vec![("data-blocks-product-overview-gallery-split-main-image", "")],
    )
}

/// サムネイル `carousel`（`orientation` に応じて横並び・縦並びを切り替える、
/// モジュール doc「ギャラリー領域」節参照）。前後トリガーは出さない。
fn thumbnail_carousel(orientation: Orientation) -> Node {
    const COUNT: usize = 4;
    let thumbs: Vec<Node> = (0..COUNT)
        .map(|i| thumbnail(orientation, i, COUNT))
        .collect();
    carousel::root(
        Size::Sm,
        orientation,
        "商品画像のサムネイル",
        vec![],
        vec![carousel::control(
            orientation,
            vec![],
            vec![carousel::item_group(
                orientation,
                vec![("class", "blocks-product-overview-gallery-split-thumbs")],
                thumbs,
            )],
        )],
    )
}

/// 版 A・B 共通: 左列（ギャラリー）= メイン画像 + サムネイル `carousel`。
/// `orientation` が `Vertical` のとき縦サムネ列の修飾クラスを付ける
/// （[`LAYOUT_CSS`] 参照）。
fn gallery_with_thumbs(orientation: Orientation) -> Node {
    let class = match orientation {
        Orientation::Vertical => {
            "blocks-product-overview-gallery-split-gallery blocks-product-overview-gallery-split-gallery--vertical"
        }
        Orientation::Horizontal => "blocks-product-overview-gallery-split-gallery",
    };
    div(
        vec![("class", class)],
        vec![main_image(), thumbnail_carousel(orientation)],
    )
}

/// 版 C: 左列（ギャラリー）= 単一メイン画像 + 枚数表示（「1 / 4」）。
/// サムネイル `carousel` 自体を持たない（モジュール doc「ギャラリー領域」
/// 節参照）。
fn gallery_with_counter() -> Node {
    div(
        vec![("class", "blocks-product-overview-gallery-split-gallery")],
        vec![
            main_image(),
            p(
                vec![("class", "blocks-product-overview-gallery-split-counter")],
                vec![text("1 / 4")],
            ),
        ],
    )
}

/// 評価行（`rating-group`、readonly。版 B は呼び出さず `rating_group::root`
/// 自体を出力しない、モジュール doc「状態違いの並記」節参照）。`suffix` で
/// ラベル id を版ごとに一意化する。
fn rating_row(suffix: &str) -> Node {
    let label_id = format!("{RATING_LABEL_ID}-{suffix}");
    let rating_props = RatingGroupProps {
        disabled: false,
        readonly: true,
        required: false,
    };
    let rating_group_state = RatingGroup::new(5, Some(4), true);
    let rating_label = rating_group::label(
        &rating_props,
        Some(label_id.as_str()),
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
        rating_group::control(&rating_props, Some(label_id.as_str()), vec![], rating_items);
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

/// 在庫僅少警告（版 A のみ、モジュール doc「在庫僅少の `progress`」節
/// 参照）。可視テキスト「残り 3 点」+ `progress`（在庫 3 / 20）。
fn stock_notice() -> Node {
    let state = Progress::new(0.0, 20.0, Some(3.0), Orientation::Horizontal);
    div(
        vec![("class", "blocks-product-overview-gallery-split-stock")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("残り 3 点")],
            ),
            progress::root(
                &state,
                &ProgressProps::default(),
                Some("残り 3 点"),
                vec![
                    ("data-blocks-product-overview-gallery-split-stock-bar", ""),
                    ("aria-label", "在庫残量"),
                ],
                vec![state.track(vec![], vec![progress::range(&state, vec![])])],
            ),
        ],
    )
}

/// 共有リンク 1 件（架空ラベル・固定 href、モジュール doc「共有リンクは
/// 架空の固定 href」節参照）。
fn share_link(label: &str) -> Node {
    link::root(REPO, &LinkProps::default(), vec![], vec![text(label)])
}

/// 共有リンク行（版 C のみ）。
fn share_row() -> Node {
    div(
        vec![("class", "blocks-product-overview-gallery-split-share")],
        vec![share_link("共有 A"), share_link("共有 B")],
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
/// 色見本を item-content の先頭へ置く（色選択専用、`shape` は版 C のみ
/// `SwatchShape::Circle`）。
fn option_item(
    name: &str,
    checked: bool,
    value: &'static str,
    label: &str,
    swatch: Option<(Color, SwatchShape)>,
) -> Node {
    let mut content_children: Vec<Node> = Vec::new();
    if let Some((color, shape)) = swatch {
        content_children.push(color_swatch::color_swatch(
            &ColorSwatchProps {
                value: color,
                size: Size::Sm,
                shape,
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
/// `trailing` が `Some` のとき見出し行の末尾へ追加要素（サイズガイド
/// ボタン、版 A のみ）を置く。
fn option_group(
    class: &'static str,
    label_id: &str,
    label_text: &str,
    items: Vec<Node>,
    selected_summary: &str,
    trailing: Option<Node>,
) -> Node {
    let mut heading_row_children = vec![radio_card::label(
        Some(label_id),
        vec![],
        vec![text(label_text)],
    )];
    let heading_row = if let Some(trailing_node) = trailing {
        heading_row_children.push(trailing_node);
        div(
            vec![(
                "class",
                "blocks-product-overview-gallery-split-option-heading",
            )],
            heading_row_children,
        )
    } else {
        heading_row_children.remove(0)
    };
    div(
        vec![("class", class)],
        vec![
            heading_row,
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

/// サイズガイドボタン（disabled の静的表示、版 A のみ。モジュール doc
/// 「サイズガイドは静的な開状態・非モーダル」節参照）。
fn size_guide_button() -> Node {
    button::button(
        &ButtonProps {
            variant: ButtonVariant::Outline,
            size: Size::Sm,
            disabled: true,
            ..ButtonProps::default()
        },
        vec![(
            "data-blocks-product-overview-gallery-split-size-guide-trigger",
            "",
        )],
        vec![text("サイズガイド")],
    )
}

/// 色選択欄（3 択、2 番目を選択済みで固定）。`suffix` で id・`name` を
/// 一意化し、`shape` で色見本の形状を切り替える（版 C は Circle）。
fn color_options(suffix: &str, shape: SwatchShape) -> Node {
    let label_id = format!("{COLOR_LABEL_ID}-{suffix}");
    let name = format!("{COLOR_NAME}-{suffix}");
    let items = vec![
        option_item(
            &name,
            false,
            "charcoal",
            "チャコール",
            Some((swatch_color((0x33, 0x33, 0x36)), shape)),
        ),
        option_item(
            &name,
            true,
            "ivory",
            "アイボリー",
            Some((swatch_color((0xf1, 0xea, 0xdd)), shape)),
        ),
        option_item(
            &name,
            false,
            "forest",
            "フォレストグリーン",
            Some((swatch_color((0x2f, 0x4f, 0x3c)), shape)),
        ),
    ];
    option_group(
        "blocks-product-overview-gallery-split-option",
        &label_id,
        "カラー",
        items,
        "現在の選択: アイボリー",
        None,
    )
}

/// サイズ選択欄（4 択、M を選択済みで固定）。`with_guide_button` が true の
/// とき見出し行の末尾へ [`size_guide_button`] を追加する（版 A のみ）。
fn size_options(suffix: &str, with_guide_button: bool) -> Node {
    let label_id = format!("{SIZE_LABEL_ID}-{suffix}");
    let name = format!("{SIZE_NAME}-{suffix}");
    let items = vec![
        option_item(&name, false, "s", "S", None),
        option_item(&name, true, "m", "M", None),
        option_item(&name, false, "l", "L", None),
        option_item(&name, false, "xl", "XL", None),
    ];
    let trailing = if with_guide_button {
        Some(size_guide_button())
    } else {
        None
    };
    option_group(
        "blocks-product-overview-gallery-split-option",
        &label_id,
        "サイズ",
        items,
        "現在の選択: M",
        trailing,
    )
}

/// カート追加ボタン（全幅、既定 `type="button"`。モジュール doc「`<form>`
/// を持たない」節・「購入ボタン・サイズガイドボタンは disabled の静的
/// 表示」節参照）。版をまたいで同じ `data-*` フックを共有するが、`id` は
/// 持たないため `blocks_contract.rs` の id 非重複検証には抵触しない。
fn add_to_cart_button() -> Node {
    button::button(
        &ButtonProps {
            size: Size::Lg,
            disabled: true,
            ..ButtonProps::default()
        },
        vec![("data-blocks-product-overview-gallery-split-add", "")],
        vec![text("カートに追加")],
    )
}

/// 詳細アコーディオン 1 項目（トリガー + 本文）。全件 `OpenState::Open` +
/// `disabled: true` で固定する。トリガーは `h3` で包む。`suffix` で
/// トリガー/コンテンツ id を一意化する。
fn detail_item(suffix: &str, index: usize, title: &str, body: &str) -> Node {
    let state = OpenState::Open;
    let props = AccordionProps {
        disabled: true,
        ..AccordionProps::default()
    };
    let trigger_id =
        format!("blocks-product-overview-gallery-split-detail-{suffix}-{index}-trigger");
    let content_id =
        format!("blocks-product-overview-gallery-split-detail-{suffix}-{index}-content");

    item(
        state,
        false,
        &props,
        vec![],
        vec![
            el(
                "h3",
                vec![(
                    "class",
                    "blocks-product-overview-gallery-split-detail-heading",
                )],
                vec![item_trigger(
                    state,
                    false,
                    &props,
                    title,
                    Some(trigger_id.as_str()),
                    Some(content_id.as_str()),
                    vec![],
                    vec![
                        span(vec![], vec![text(title)]),
                        item_indicator(state, false, &props, vec![], vec![text("\u{25be}")]),
                    ],
                )],
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

/// 詳細項目の生データ（説明・素材と手入れ・配送と返品。版 A/B/C のいずれ
/// でも共通の文言を使う）。
const DETAILS: [(&str, &str); 3] = [
    (
        "説明",
        "密閉型のワイヤレスヘッドホンです。長時間の装着でも疲れにくい軽量設計で、通勤・在宅ワークの双方に向いています。",
    ),
    (
        "素材と手入れ",
        "イヤーパッドは合成皮革を使用しています。乾いた柔らかい布で軽く拭き取ってください。",
    ),
    (
        "配送と返品",
        "通常 3〜5 営業日でお届けします。未使用品に限り到着後 14 日以内の返品を承ります。",
    ),
];

/// 版 A: 詳細情報アコーディオン（説明・素材と手入れ・配送と返品の 3
/// 項目、全件 open + disabled 固定）。
fn details_accordion(suffix: &str) -> Node {
    let items = DETAILS
        .iter()
        .enumerate()
        .map(|(index, (title, body))| detail_item(suffix, index, title, body))
        .collect();
    accordion::root(
        Size::Md,
        &AccordionProps::default(),
        vec![("data-blocks-product-overview-gallery-split-details", "")],
        items,
    )
}

/// カテゴリラベルのみを装飾として示す非対話タブ列（モジュール doc「実物の
/// `tabs::tabs` を使わない」節参照）。`role`/`tabindex`/`<button>` を一切
/// 持たず、`selected` に一致するラベルだけ `data-state="active"` にする。
/// 各ラベルは `aria-hidden` で支援技術のツリーから除外する
/// （`product_overview_tabs_below::static_tab_list` と同型）。
fn static_tab_list(selected: &str) -> Node {
    let labels = [
        ("description", "説明"),
        ("care", "素材と手入れ"),
        ("shipping", "配送と返品"),
        ("size-guide", "サイズガイド"),
    ];
    div(
        vec![("class", "blocks-product-overview-gallery-split-tablist")],
        labels
            .iter()
            .map(|(value, label)| {
                let state = if *value == selected {
                    "active"
                } else {
                    "inactive"
                };
                div(
                    vec![
                        ("class", "blocks-product-overview-gallery-split-tab"),
                        ("data-state", state),
                        ("aria-hidden", "true"),
                    ],
                    vec![text(*label)],
                )
            })
            .collect(),
    )
}

/// 非選択タブのプレビュー併記（`product_overview_tabs_below::
/// panel_preview`/`faq_tabbed_accordion::category_preview` と同型の
/// 「『〜』タブを選択した場合のプレビュー」キャプション付き常時可視節、
/// モジュール doc「実物の `tabs::tabs` を使わない」節参照）。
fn panel_preview(label: &str, body: Node) -> Node {
    div(
        vec![("class", "blocks-product-overview-gallery-split-preview")],
        vec![
            h3(
                vec![],
                vec![text(format!("「{label}」タブを選択した場合のプレビュー"))],
            ),
            body,
        ],
    )
}

/// サイズガイド dialog のプレビュー併記（モジュール doc「代表構成」節
/// 「『サイズガイド』を開いた場合のプレビュー」参照）。[`panel_preview`]
/// とは見出し文言を分離する: 版 A のサイズガイドはタブ選択ではなく静的な
/// `dialog` の開状態（モジュール doc「サイズガイドは静的な開状態・
/// 非モーダル」節参照）であり、タブ用文言を流用すると実際の操作と見出しが
/// 不一致になるため（Codex/Cursor Bugbot 指摘 是正）。
fn dialog_preview(label: &str, body: Node) -> Node {
    div(
        vec![("class", "blocks-product-overview-gallery-split-preview")],
        vec![
            h3(
                vec![],
                vec![text(format!("「{label}」を開いた場合のプレビュー"))],
            ),
            body,
        ],
    )
}

/// 版 B: 静的タブ列 + 選択中「説明」パネル + 残り 3 タブのプレビュー併記
/// （モジュール doc「実物の `tabs::tabs` を使わない」節参照）。
fn details_tabs() -> Node {
    div(
        vec![("class", "blocks-product-overview-gallery-split-tabs")],
        vec![
            static_tab_list("description"),
            div(
                vec![(
                    "class",
                    "blocks-product-overview-gallery-split-tab-selected",
                )],
                vec![
                    h3(vec![], vec![text(DETAILS[0].0)]),
                    styled_text::text(
                        &TextProps::default(),
                        vec![],
                        vec![text(DETAILS[0].1)],
                    ),
                ],
            ),
            panel_preview(
                DETAILS[1].0,
                styled_text::text(&TextProps::default(), vec![], vec![text(DETAILS[1].1)]),
            ),
            panel_preview(
                DETAILS[2].0,
                styled_text::text(&TextProps::default(), vec![], vec![text(DETAILS[2].1)]),
            ),
            panel_preview(
                "サイズガイド",
                styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text(
                        "サイズ表（S〜XL の頭囲目安）を表示します。実際の表は代表構成（版 A）のサイズガイドを参照してください。",
                    )],
                ),
            ),
        ],
    )
}

/// 版 C: 見出し（`h3`）付き段落列（開閉状態を持たない、モジュール doc
/// 「詳細を 3 通りの表現で示す理由」節参照）。
fn details_paragraphs() -> Node {
    div(
        vec![("class", "blocks-product-overview-gallery-split-paragraphs")],
        DETAILS
            .iter()
            .map(|(title, body)| {
                div(
                    vec![],
                    vec![
                        h3(vec![], vec![text(*title)]),
                        styled_text::text(&TextProps::default(), vec![], vec![text(*body)]),
                    ],
                )
            })
            .collect(),
    )
}

/// サイズガイド dialog 本文（サイズ表を素の段落で示す、モジュール doc
/// 「サイズガイドは静的な開状態・非モーダル」節参照。`table` 部品は増やさ
/// ない）。
fn size_guide_body() -> Node {
    const ROWS: [(&str, &str); 4] = [
        ("S", "頭囲 52〜54 cm 目安"),
        ("M", "頭囲 54〜57 cm 目安"),
        ("L", "頭囲 57〜60 cm 目安"),
        ("XL", "頭囲 60〜63 cm 目安"),
    ];
    div(
        vec![],
        ROWS.iter()
            .map(|(label, desc)| p(vec![], vec![text(format!("{label}: {desc}"))]))
            .collect(),
    )
}

/// サイズガイド dialog（版 A のみ。モジュール doc「サイズガイドは静的な
/// 開状態・非モーダル」節参照）。`trigger`/`close_trigger` は置かず、
/// `backdrop` も出さない（[`LAYOUT_CSS`] 「サイズガイド dialog の中和」
/// 節参照）。
fn size_guide_dialog() -> Node {
    const CONTENT_ID: &str = "blocks-product-overview-gallery-split-size-guide-content";
    const TITLE_ID: &str = "blocks-product-overview-gallery-split-size-guide-title";
    dialog::root(
        Size::Md,
        DialogOpenState::Open,
        vec![("data-blocks-product-overview-gallery-split-size-guide", "")],
        vec![dialog::positioner(
            DialogOpenState::Open,
            vec![],
            vec![dialog::content(
                DialogOpenState::Open,
                DialogRole::Dialog,
                false,
                ContentIds {
                    id: Some(CONTENT_ID),
                    labelledby: Some(TITLE_ID),
                    describedby: None,
                },
                vec![],
                vec![
                    dialog::title(Some(TITLE_ID), vec![], vec![text("サイズガイド")]),
                    dialog::body(vec![], vec![size_guide_body()]),
                ],
            )],
        )],
    )
}

/// 版キャプション（`product_overview_tabs_below::caption` と同型、素の
/// `<p>` で `heading` 部品を使わない）。
fn caption(label: &str) -> Node {
    p(
        vec![("class", "blocks-product-overview-gallery-split-caption")],
        vec![text(label)],
    )
}

/// 版 A（代表構成、対応表 ID R0611）: 横並びサムネイル + 評価あり + 在庫
/// 僅少警告 + サイズガイドボタン + アコーディオン詳細。
fn variant_a() -> Node {
    div(
        vec![("class", "blocks-product-overview-gallery-split-layout")],
        vec![
            gallery_with_thumbs(Orientation::Horizontal),
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
                    rating_row("a"),
                    price_line(),
                    stock_notice(),
                    color_options("a", SwatchShape::Rounded),
                    size_options("a", true),
                    add_to_cart_button(),
                    details_accordion("a"),
                ],
            ),
        ],
    )
}

/// 版 B（縦サムネ列・タブ型詳細・評価なし、対応表 ID R0616・R0614・
/// R0617）: 縦サムネイル carousel + 評価行なし + 静的タブ型詳細。
fn variant_b() -> Node {
    div(
        vec![("class", "blocks-product-overview-gallery-split-layout")],
        vec![
            gallery_with_thumbs(Orientation::Vertical),
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
                    price_line(),
                    color_options("b", SwatchShape::Rounded),
                    size_options("b", false),
                    add_to_cart_button(),
                    details_tabs(),
                ],
            ),
        ],
    )
}

/// 版 C（単一画像・枚数表示・段落詳細・共有行、対応表 ID R0613・
/// R0615）: 単一メイン画像 + 枚数表示 + 丸色見本 + 共有行 + 段落詳細。
fn variant_c() -> Node {
    div(
        vec![("class", "blocks-product-overview-gallery-split-layout")],
        vec![
            gallery_with_counter(),
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
                    rating_row("c"),
                    price_line(),
                    share_row(),
                    color_options("c", SwatchShape::Circle),
                    size_options("c", false),
                    add_to_cart_button(),
                    details_paragraphs(),
                ],
            ),
        ],
    )
}

/// `product-overview-gallery-split` の Demo 本体。3 版（代表構成・縦サムネ
/// 列/タブ型詳細/評価なし・単一画像/枚数表示/段落詳細/共有行）をキャプ
/// ション付きで縦に並べ、版 A の下へサイズガイド dialog のプレビューを
/// 併記する（モジュール doc「状態違いの並記」節参照）。呼び出しごとに
/// 同一の `Node` を返す純関数（`crate::blocks` モジュール doc「静的表示」
/// 節）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-product-overview-gallery-split-demo")],
        vec![
            caption("代表構成"),
            fandhe_frontend_core::section(vec![], vec![variant_a()]),
            dialog_preview("サイズガイド", size_guide_dialog()),
            caption("縦サムネ列・タブ型詳細・評価なし"),
            fandhe_frontend_core::section(vec![], vec![variant_b()]),
            caption("単一画像・枚数表示・段落詳細・共有行"),
            fandhe_frontend_core::section(vec![], vec![variant_c()]),
        ],
    )
}
```

## 原案差分メモ

- 主参照は R0611（メイン画像 + サムネイルカルーセル + 購入パネル、
  代表構成＝版 A）です。集約元のうち次の 5 件は Demo の 3 版へ振り分けて
  表現しています。
  - R0613（単一画像 + 枚数表示・丸色見本）・R0615（段落列 + 共有行）:
    版 C「単一画像・枚数表示・段落詳細・共有行」が担う
  - R0614（タブ詳細）・R0617（評価なしサイズガイド統合）: 版 B
    「縦サムネ列・タブ型詳細・評価なし」が担う（サムネ縦列は R0616）
  - R0616（縦サムネ列）: 版 B のギャラリーが担う
- R1176（サムネ切替タブ + 開閉式詳細）は Demo に加えていません。サムネ
  切替は版 A・版 B の `carousel` と同型の構造であり、開閉式の詳細は無 JS
  では全件 open 固定になるため（詳細アコーディオンを全件 open + disabled
  で固定する理由と同じ）、版 A のアコーディオンと見分けがつかない静的
  表示になり、新たな状態を Demo へ追加する価値がないと判断しました。
- 狭幅版（`< 48rem`）の並記も行っていません。2 カラム切り替えは
  `@media (min-width: 48rem)` の viewport クエリであり、ページ内の並記
  （リサイズなしでの視覚確認）では再現できないためです。
- 在庫僅少警告（`progress`）・サイズガイド（`dialog`）・共有リンク行
  （`link`）は本 PR（#3069）で実装しました。`BLOCK::parts` へ 3 部品を
  追加しています。
- 実物の `tabs::tabs` 部品は使っていません。docs サイトは無 JS のため、
  実物の `tabs::tabs` を置くと「操作できるように見えて切り替わらない」
  `<button role="tab">` と `hidden` パネルが残ります（既存 block が
  是正済みの経路と同じ判断）。本 block も `role`/`tabindex`/`<button>` を
  持たない非対話の `div` 列（静的タブ列）を置き、選択中パネルは別の
  可視見出し（`h3`）付き節として直下に描画し、残りのタブは「『〜』タブを
  選択した場合のプレビュー」として常時可視・縦積みに併記しています
  （`hidden` は一切使いません）。このため `BLOCK::parts` に `Tabs` は
  含めていません（実際に呼ばない部品を掲げない、既存 block と同じ判断）。
- サイズガイドのダイアログは `trigger`/`close_trigger` を置かず、
  `aria-modal="false"` にしています。静的なデモは閉じる機構を実際には
  持たず、ダイアログの外側に説明・ギャラリー・購入パネルがあるため、
  支援技術が外側を無視しないよう表示の実態と一致させる判断です
  （`cart_dialog.rs`/`contact_dialog_form.rs`/`auth_tabs_card.rs` と同じ）。
  `backdrop` は出していません（デモ枠内の単独プレビューとして
  `positioner`/`content` の中和のみで十分なため）。
- 在庫僅少の `progress` は `aria-label="在庫残量"` を明示しています
  （`progress::root` は自動では `aria-label` を配線しないため、
  `order_tracking_progress.rs`/`settings_page_tabs.rs` と同じ判断）。可視
  テキストでも「残り 3 点」と明文化し、進捗バーだけに依存しないように
  しています。値は固定（在庫 3 / 20）です。
- 共有リンクは実在 SNS サービス名を持ち込まず、「共有 A」「共有 B」の
  架空ラベルと本リポジトリの固定 URL（`REPO` 定数）にしています
  （`product_overview_tabs_below.rs` と同型の判断）。`href="#"` は使って
  いません。
- 色・サイズの選択は全版共通で `radio_card` をネイティブ `disabled` の
  まま用い、現在の選択を本文テキストで明文化しています
  （`pricing_single_split.rs` の支払周期選択と同型の判断）。色見本は
  `color_swatch`（`aria-hidden`）を選択肢名と併記する装飾として使い、
  版 C のみ `SwatchShape::Circle` にしています。
- 評価ラベル・色/サイズ選択ラベル・radio の `name`・詳細トリガー/
  コンテンツの各 id は `ai_chat_code_preview.rs` と同じ判断で版接尾辞
  （`a`/`b`/`c`）を付けて一意化しています。
- ギャラリーの前後トリガー（prev/next）は出していません。無 JS のため
  実際にはスライド送りを配線できず、購入パネル側に既に無効化ボタン・
  チェックボックスを多数持つため、ギャラリー側は視覚ノイズを避けて
  「1 枚目が選択済み」の静的表示のみとしています。
- 詳細情報アコーディオン（版 A）は 3 項目（説明・素材と手入れ・配送と
  返品）をすべて開いた状態 + disabled で固定しています
  （`faq_split_accordion.rs` 等の前例と同じ判断。無 JS では閉じた項目が
  到達不能になるため）。
- `_/blocks-intake/` の参照ファイルは本イシュー着手時点で worktree に
  存在しないため、対応表 ID のみを記して実装しました。
- ブラウザでの実機確認（`48rem` 前後のコンテナ幅切替・縦サムネ列・
  サイズガイド dialog がデモ枠からはみ出さないこと・ライト/ダーク両
  テーマ）はサンドボックス制約により未実施です。`cargo test -p
  fandhe-frontend-docs-site` の出力検証・`LAYOUT_CSS` の静的検証のみで
  代替しました。
