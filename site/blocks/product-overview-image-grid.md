# product-overview-image-grid

商品画像グリッドと購入パネル（商品名・価格・評価・色/サイズ選択・カート
追加ボタン・説明）を持つ商品詳細画面のブロックです。
`breadcrumb` / `image` / `heading` / `text` / `rating-group` / `radio-card` /
`color-swatch` / `button` の 8 部品を合成します。Blocks は既存部品の
合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R1178（集約元 R1175: 段差配置 + 右に購入パネル / R0612:
2 列グリッド・左右反転）です。集約元 2 件の差分と選択状態の違い（別配色・
在庫切れ）を 3 版並記で示します。商品名・価格・レビュー件数・色/サイズ・
説明・在庫注記はすべて架空のデータであり、実在のブランド・商品・PII は
含みません。商品画像はビルド時生成の同梱プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。色・サイズの
選択は `disabled` の radio card による初期選択のみの静的表示です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, h3, section, text, Node};
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

/// 版キャプション見出し（各版 `section` の `aria-labelledby` 対象）の
/// `id` の基底文字列。3 版並記のため [`Variant::suffix`] を連結して
/// 版ごとに一意にする（モジュール doc「3 版の並記」節参照。商品名
/// （H2）は全版共通のため、見出し/領域ナビゲーションで版を区別できる
/// ようこのキャプション見出しを `section` のアクセシブルネームにする、
/// Codex レビュー〔PR #3522〕の是正）。
const CAPTION_HEADING_ID: &str = "blocks-product-overview-image-grid-caption";
/// 評価ラベル（`rating_group::label` の `id`）の基底文字列。3 版並記のため
/// [`Variant::suffix`] を連結して版ごとに一意にする（重複 id 検査対応）。
const RATING_LABEL_ID: &str = "blocks-product-overview-image-grid-rating-label";
/// 色選択グループの見出し `id`（`radio_card::root` の `labelled_by`）の
/// 基底文字列。版ごとに一意にする。
const COLOR_LABEL_ID: &str = "blocks-product-overview-image-grid-color-label";
/// サイズ選択グループの見出し `id`（`radio_card::root` の `labelled_by`）の
/// 基底文字列。版ごとに一意にする。
const SIZE_LABEL_ID: &str = "blocks-product-overview-image-grid-size-label";

/// 架空の商品名。実在のブランド・商品を含まない。全版共通。
const PRODUCT_NAME: &str = "プレミアム ワイヤレスヘッドホン";
/// 架空の価格表示（円建て固定）。全版共通。
const PRICE_DISPLAY: &str = "¥32,800";
/// 架空の説明文（2〜3 文）。全版共通。
const DESCRIPTION: &[&str] = &[
    "長時間の使用でも疲れにくい軽量設計と、周囲の音を抑えるノイズ\
キャンセリング機能を両立したワイヤレスヘッドホンです。",
    "満充電で最大 30 時間再生でき、外出先でも安心してお使いいただけます。",
];
/// 在庫切れ時の架空の注記文。実在の在庫状況を示すものではない。
const OUT_OF_STOCK_NOTE: &str = "現在在庫がありません。入荷までしばらくお待ちください。";

/// 架空の色選択肢（値, 表示名, RGB）。[`Variant::color`] が初期選択を
/// 版ごとに指定する。実在のブランド固有色は使わない。
const COLOR_OPTIONS: &[(&str, &str, (u8, u8, u8))] = &[
    ("charcoal", "チャコール", (0x2b, 0x2b, 0x2b)),
    ("silver", "シルバー", (0xc4, 0xc4, 0xc4)),
    ("navy", "ネイビー", (0x1f, 0x3a, 0x5f)),
];

/// 架空のサイズ選択肢。[`Variant::size`] が初期選択を版ごとに指定する。
const SIZE_OPTIONS: &[&str] = &["S", "M", "L", "XL"];

/// 1 版分の設定（集約元差分・選択状態違いを表す。`product_overview_tabs_below::Variant`
/// と同型の判断軸。モジュール doc「3 版の並記」節参照）。
struct Variant {
    /// 版の識別子。id・radio `name`・aria-label の一意化に使う suffix。
    suffix: &'static str,
    /// Demo 上の版キャプション文言。
    caption: &'static str,
    /// true のとき画像グリッドとパネルを 2 カラムに配置する（R1175）。
    side: bool,
    /// true のとき `side` の 2 カラムを左右反転する（R0612）。`side` が
    /// false のときは無視される。
    reverse: bool,
    /// true のとき段差配置（hero/wide）ではなく 2 列均等グリッドにする
    /// （R0612 の画像グリッド形状）。
    uniform_gallery: bool,
    /// 初期選択の色（[`COLOR_OPTIONS`] の value と一致させる）。
    color: &'static str,
    /// 初期選択のサイズ（[`SIZE_OPTIONS`] の値と一致させる）。
    size: &'static str,
    /// false のとき在庫切れ表示にする（ボタン文言・在庫注記）。
    in_stock: bool,
}

/// Demo が並記する 3 版（モジュール doc「3 版の並記」節の表と対応）。
const VARIANTS: [Variant; 3] = [
    Variant {
        suffix: "stacked",
        caption: "代表構成（画像グリッド上段・購入パネル下段）",
        side: false,
        reverse: false,
        uniform_gallery: false,
        color: "charcoal",
        size: "M",
        in_stock: true,
    },
    Variant {
        suffix: "side",
        caption: "段差配置の画像 + 右に購入パネル（集約元 R1175）",
        side: true,
        reverse: false,
        uniform_gallery: false,
        color: "silver",
        size: "L",
        in_stock: true,
    },
    Variant {
        suffix: "reverse",
        caption: "2 列均等グリッド・左右反転・在庫切れ（集約元 R0612）",
        side: true,
        reverse: true,
        uniform_gallery: true,
        color: "navy",
        size: "S",
        in_stock: false,
    },
];

/// 画像グリッドの 1 枚（`index` は 0 始まり）。`tile` に `Some("hero"|"wide")`
/// を渡すと [`LAYOUT_CSS`] の段差配置（2×2 + 下段 1 枚全幅）のフックとなる
/// `data-blocks-product-overview-image-grid-tile` 属性を付与する。`None`
/// （2 列均等グリッド版）では暗黙配置に任せる。
fn gallery_tile(index: usize, tile: Option<&'static str>) -> Node {
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
    let attrs = match tile {
        Some(tag) => vec![("data-blocks-product-overview-image-grid-tile", tag)],
        None => vec![],
    };
    image(&props, attrs)
}

/// 上段の画像グリッド（4 枚）。`variant.uniform_gallery` が false の版は
/// 1 枚目を 2×2 で大きく・最後の 1 枚を 3 列全幅の下段に明示配置する
/// 段差配置（主参照 R1178、集約元 R1175 の要素）。true の版（R0612）は
/// 全画像を暗黙配置に任せる 2 列均等グリッドにし、`-gallery-uniform`
/// 修飾クラスで [`LAYOUT_CSS`] のグリッド定義を上書きする。
fn gallery(variant: &Variant) -> Node {
    const TOTAL: usize = 4;
    let tiles: Vec<Node> = (0..TOTAL)
        .map(|index| {
            let tile = if variant.uniform_gallery {
                None
            } else if index == 0 {
                Some("hero")
            } else if index == TOTAL - 1 {
                Some("wide")
            } else {
                None
            };
            gallery_tile(index, tile)
        })
        .collect();
    let class = if variant.uniform_gallery {
        "blocks-product-overview-image-grid-gallery blocks-product-overview-image-grid-gallery-uniform"
    } else {
        "blocks-product-overview-image-grid-gallery"
    };
    div(vec![("class", class)], tiles)
}

/// パンくず（Home → Blocks → 現在の商品名。`app_shell_stacked` と同型の
/// 相対パス構成: ページは `/blocks/product-overview-image-grid/` に生成
/// されるため `../../` はサイトルート、`../` は `/blocks/` を指す）。
/// `aria-label` は版ごとに一意にする（モジュール doc「評価のアクセシブル
/// ネーム・radio グループ名の版ごとの一意化」節参照、パンくずは各版が
/// 1 画面として完結するよう版ごとに置く）。
fn breadcrumb_nav(variant: &Variant) -> Node {
    let aria_label = format!("Breadcrumb ({})", variant.suffix);
    breadcrumb::root(
        Size::Sm,
        BreadcrumbVariant::default(),
        Some(&aria_label),
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
/// を持たないためリンクにしない（モジュール doc「レビュー件数は
/// リンクにしない」節参照）。
fn rating_row(variant: &Variant) -> Node {
    let label_id = format!("{RATING_LABEL_ID}-{}", variant.suffix);
    let props = RatingGroupProps {
        disabled: false,
        readonly: true,
        required: false,
    };
    let state = RatingGroup::new(5, Some(4), true);
    let label = rating_group::label(&props, Some(&label_id), vec![], vec![text("評価 4.0")]);
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
    let control = rating_group::control(&props, Some(&label_id), vec![], items);
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
    // 実在の Reviews block が追加され次第、`link` へ差し替える
    // （本 PR のスコープ外）。
    let review_count =
        styled_text::text(&TextProps::default(), vec![], vec![text("レビュー 128 件")]);
    div(
        vec![("class", "blocks-product-overview-image-grid-rating-row")],
        vec![rating, review_count],
    )
}

/// 色選択肢 1 件（`radio_card::item` + `color_swatch` + ラベルテキスト）。
/// `card_form_footer` の支払方法 radio card と同じく、常に `disabled:
/// true` で静的表示にする。`name` は版ごとに一意な hidden input グループ名
/// （モジュール doc「radio グループ名の版ごとの一意化」節参照）。
fn color_item(
    checked: bool,
    value: &'static str,
    label: &'static str,
    rgb: (u8, u8, u8),
    name: &str,
) -> Node {
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
            radio_card::item_hidden_input(checked, true, Some(name), value, vec![]),
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

/// 色選択欄（見出し + radio card 3 択）。初期選択は [`Variant::color`]。
fn color_options(variant: &Variant) -> Node {
    let label_id = format!("{COLOR_LABEL_ID}-{}", variant.suffix);
    let name = format!(
        "blocks-product-overview-image-grid-color-{}",
        variant.suffix
    );
    div(
        vec![("class", "blocks-product-overview-image-grid-options")],
        vec![
            radio_card::label(Some(&label_id), vec![], vec![text("カラー")]),
            radio_card::root(
                Size::Sm,
                ColorPalette::Accent,
                true,
                Some(Orientation::Horizontal),
                Some(&label_id),
                vec![
                    ("aria-disabled", "true"),
                    ("data-blocks-product-overview-image-grid-color", ""),
                ],
                COLOR_OPTIONS
                    .iter()
                    .map(|(value, label, rgb)| {
                        color_item(*value == variant.color, value, label, *rgb, &name)
                    })
                    .collect(),
            ),
        ],
    )
}

/// サイズ選択肢 1 件（`radio_card::item`。色スウォッチは持たない）。
/// `name` は版ごとに一意な hidden input グループ名。
fn size_item(checked: bool, value: &'static str, name: &str) -> Node {
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
                    radio_card::item_content(
                        vec![],
                        vec![radio_card::item_text(vec![], vec![text(value)])],
                    ),
                ],
            ),
        ],
    )
}

/// サイズ選択欄（見出し + radio card 4 択）。初期選択は [`Variant::size`]。
fn size_options(variant: &Variant) -> Node {
    let label_id = format!("{SIZE_LABEL_ID}-{}", variant.suffix);
    let name = format!("blocks-product-overview-image-grid-size-{}", variant.suffix);
    div(
        vec![("class", "blocks-product-overview-image-grid-options")],
        vec![
            radio_card::label(Some(&label_id), vec![], vec![text("サイズ")]),
            radio_card::root(
                Size::Sm,
                ColorPalette::Accent,
                true,
                Some(Orientation::Horizontal),
                Some(&label_id),
                vec![
                    ("aria-disabled", "true"),
                    ("data-blocks-product-overview-image-grid-size", ""),
                ],
                SIZE_OPTIONS
                    .iter()
                    .map(|value| size_item(*value == variant.size, value, &name))
                    .collect(),
            ),
        ],
    )
}

/// 下段（または横）の購入パネル（商品名・価格・評価・色/サイズ選択・
/// カート追加ボタン・説明。主参照 R1178）。`variant.in_stock` が false の
/// ときはボタン文言を「在庫切れ」にし、価格の下へ在庫注記を挿む。
fn purchase_panel(variant: &Variant) -> Node {
    let cta_label = if variant.in_stock {
        "カートに追加"
    } else {
        "在庫切れ"
    };
    let mut children = vec![
        // 見出しレベルは H4（H2 ではない）: ページの見出し階層は
        // H1 → H2「Demo」→ H3（版キャプション、[`caption`]）→ ここ。
        // 商品名を H2 にするとページ全体の「Demo」H2 と並んでしまい、
        // 階層が崩れる（Codex レビュー、PR #3522 の是正。モジュール doc
        // 「3 版の並記」節参照）。`HeadingProps.size`（既定 `Xl`）は level と
        // 独立なので見た目は変わらない。
        heading(
            HeadingLevel::H4,
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
    ];
    if !variant.in_stock {
        children.push(styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(OUT_OF_STOCK_NOTE)],
        ));
    }
    children.push(rating_row(variant));
    children.push(color_options(variant));
    children.push(size_options(variant));
    children.push(button(
        &ButtonProps {
            size: Size::Lg,
            palette: ColorPalette::Accent,
            disabled: true,
            ..ButtonProps::default()
        },
        vec![("data-blocks-product-overview-image-grid-cta", "")],
        vec![text(cta_label)],
    ));
    children.push(styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(DESCRIPTION.join(""))],
    ));
    div(
        vec![("class", "blocks-product-overview-image-grid-panel")],
        children,
    )
}

/// 版キャプション見出し（`product_overview_tabs_below::caption` と異なり、
/// 本版は素の `<h3>`（`heading` 部品は使わない。`BLOCK` の `parts` を
/// 変えないため）にする。商品名（H2）が全版共通で見出し/領域ナビゲー
/// ションでは版を区別できない指摘（Codex レビュー、PR #3522）の是正と
/// して、各版の `section` 内に置き `id` を付与し、`section` 自体の
/// `aria-labelledby` から参照させる（下記 [`shell`] 参照）。
fn caption(variant: &Variant) -> Node {
    h3(
        vec![
            ("class", "blocks-product-overview-image-grid-caption"),
            ("id", &caption_heading_id(variant)),
        ],
        vec![text(variant.caption)],
    )
}

/// [`caption`] の見出し `id`（版ごとに一意）。[`shell`] の
/// `aria-labelledby` と対で使う。
fn caption_heading_id(variant: &Variant) -> String {
    format!("{CAPTION_HEADING_ID}-{}", variant.suffix)
}

/// 1 版分の shell（キャプション見出し + パンくず + 画像グリッド・購入
/// パネルの本体）を組み立てる。`variant.side` が true のとき本体を
/// 2 カラムにし、`variant.reverse` が true のとき左右を入れ替える
/// （モジュール doc「3 版の並記」節参照）。
fn shell(variant: &Variant) -> Node {
    let mut body_class = String::from("blocks-product-overview-image-grid-body");
    if variant.side {
        body_class.push_str(" blocks-product-overview-image-grid-body-side");
    }
    if variant.reverse {
        body_class.push_str(" blocks-product-overview-image-grid-body-reverse");
    }
    let body = div(
        vec![("class", body_class.as_str())],
        vec![gallery(variant), purchase_panel(variant)],
    );
    div(
        vec![("class", "blocks-product-overview-image-grid-stack")],
        vec![breadcrumb_nav(variant), body],
    )
}

/// `product-overview-image-grid` の Demo 本体。[`VARIANTS`] の 3 版を
/// キャプション付きで縦に並べる（モジュール doc「3 版の並記」節）。
/// 呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let mut children = Vec::with_capacity(VARIANTS.len());
    for variant in &VARIANTS {
        let heading_id = caption_heading_id(variant);
        children.push(section(
            vec![("aria-labelledby", &heading_id)],
            vec![caption(variant), shell(variant)],
        ));
    }
    div(
        vec![("class", "blocks-product-overview-image-grid-demo")],
        children,
    )
}
```

## 原案差分メモ

- 画像グリッド（1 枚目を 2×2 で大きく表示する段差配置）と、商品名・価格・
  評価・色/サイズ選択・カート追加ボタン・説明を並べた購入パネル（対応表
  ID R1178）を軸に、集約元 2 件の差分と選択状態違いを 3 版並記で示します。
  - **stacked**: 代表構成。画像グリッドが上段、購入パネルが下段の縦積み。
    段差配置（hero/wide）。在庫あり、初期選択はチャコール/M。
  - **side**: 集約元 R1175（段差配置の画像 + 右に購入パネル）。画像グリッド
    が左、購入パネルが右の 2 カラム。段差配置。在庫あり、初期選択は
    シルバー/L（選択状態違いの並記）。
  - **reverse**: 集約元 R0612（2 列グリッド・左右反転）。購入パネルが左、
    画像グリッドが右の 2 カラム反転。画像は 2 列均等グリッド（段差なし）。
    在庫切れ（カート追加ボタンの文言が「在庫切れ」に変わり、価格の下へ
    在庫注記が入ります）、初期選択はネイビー/S。
- 左右反転（reverse）は CSS の `grid-template-areas` のみで行い、DOM 順は
  常に画像 → パネルのまま変えません（読み上げ順は変わりません）。
- コンテナ幅（Demo 枠の幅、ビューポート幅ではありません）が 40rem 未満に
  なると `@container` によって、2 カラム配置は縦積みへ、画像グリッドは
  段差配置・2 列均等グリッドのどちらも 1 列積みへ切り替わります。しきい値
  は 3 版共通で 40rem の 1 つです。
- 色・サイズの選択は `radio_card` を `disabled` にした静的表示です。3 版
  それぞれ異なる radio グループ名（`<form>` のない文書で同名 radio が
  1 グループに混ざらないようにするため）を持たせ、版ごとの初期選択が
  独立して表示されます。
- 親 issue の使用部品リストには `link` も挙げられていますが、レビュー
  詳細ページ（Ecommerce / Reviews カテゴリ）は本イシュー時点で block が
  未登録で実在しないため、実在しない遷移先を作らず可視テキスト
  （「レビュー 128 件」）のまま表示しています。Reviews block が追加され
  次第、各版の `link` へ差し替える想定です（本 PR のスコープ外）。
- 2 カラム配置（side/reverse）ではパネル幅が約 20rem のため、色・サイズの
  選択肢（横並び）は折り返して表示します（はみ出しません）。
- 商品名の見出しは版キャプション（`<h3>`）より下位の `<h4>` です（ページ
  全体の見出し階層が H1 → H2「Demo」→ H3「版キャプション」→ H4「商品名」
  となるようにしています）。

関連情報: [Breadcrumb](../themes/breadcrumb.md) / [Image](../themes/image.md) /
[Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Rating Group](../themes/rating-group.md) / [Radio Card](../themes/radio-card.md) /
[Color Swatch](../themes/color-swatch.md) / [Button](../themes/button.md)
