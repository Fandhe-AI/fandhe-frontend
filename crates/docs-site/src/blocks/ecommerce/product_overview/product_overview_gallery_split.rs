//! `product-overview-gallery-split` block（イシュー #3068。親トラッキング
//! #3067「商品詳細（サムネイル付きギャラリー + 購入パネル）」の前半、
//! 骨格と主要領域を担う）。対応表 ID R0611（主参照）・R0613/R0614/R0615/
//! R0616/R0617/R1176（集約元）を構造の参照元とする合成例。取得手段・
//! ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。Ecommerce / Product Overview カテゴリ最初の block
//! （雛形は `mod.rs` を参照）。
//!
//! # 使用部品
//!
//! `image` / `carousel` / `breadcrumb` / `heading` / `text` /
//! `rating-group` / `radio-card` / `color-swatch` / `button` / `accordion`
//! の 10 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! `progress`（在庫僅少警告）・`dialog`（サイズガイド）・`tabs`
//! （タブ型詳細）は後半 #3069 の担当のため本 Demo には含めない。
//!
//! # レイアウト（骨格）
//!
//! `>= 48rem` で 2 カラム（左: ギャラリー、右: 購入パネル）、未満では
//! ギャラリー上・購入パネル下の縦積みにフォールバックする（[`LAYOUT_CSS`]）。
//! `display: none` は使わない（狭幅でも全操作要素へ到達可能なまま積む）。
//!
//! # ギャラリー領域
//!
//! メイン画像（正方形・`object-fit: cover`）の下へ、横並びのサムネイル
//! `carousel` を置く。前後トリガーは出さない（無 JS のデモでスライド送り
//! を実際には配線できず、`gallery_split_carousel.rs` のように
//! `disabled: true` を伴う代替も検討したが、本 block は購入パネル側に
//! 既に無効化ボタン・チェックボックスを多数持つため、ギャラリー側は視覚
//! ノイズを避けて「index 0 が選択済み」の静的表示のみとする最小構成を
//! 採る）。
//!
//! # 購入パネル領域
//!
//! パンくず（`breadcrumb`）→ 商品名（`heading` H2）→ 評価
//! （`rating-group`、readonly）→ 価格（素の `<p>`、理由は下記）→ 色選択・
//! サイズ選択（`radio-card` ×2 グループ）→ カート追加ボタン
//! （`button`、全幅）→ 詳細アコーディオン（`accordion`）の順に縦積みする。
//!
//! # 価格を素の `<p>` で組み立てる理由
//!
//! `fandhe_frontend_pre_styled_ui::text::text` は呼び出し側が渡した
//! `class` 属性を `drop_class_attr` で無条件に除去するため、
//! `blocks-product-overview-gallery-split-price` クラスを渡しても
//! [`LAYOUT_CSS`] のフォントサイズ規則が適用されない
//! （`page_heading_avatar.rs`「メタ行を素の `<p>` で組み立てる理由」節と
//! 同型の判断）。
//!
//! # 色・サイズ選択を radio card + ネイティブ disabled にする理由
//!
//! `pricing_single_split.rs::billing_toggle` と同じ判断で、無 JS の docs
//! サイトでは選択の切り替えを実配線できないため `radio_card::root` へ
//! `aria-disabled="true"` を明示し、各 item もネイティブ `disabled` で
//! 固定する。現在の選択は radio の checked 状態に依存しない
//! [`styled_text::text`] の文で明文化する（ネイティブ disabled な radio は
//! 支援技術のフォームモード走査から除外され得るため）。
//!
//! # 色見本を `color_swatch` で表現する理由
//!
//! 色選択の各 `radio_card::item` は `item_indicator` の隣に
//! `color_swatch::color_swatch`（`aria-hidden`、装飾）を置き、選択肢名
//! （`item_text`）と併記する。色自体は視覚的な補助であり、可視テキスト
//! （色名）がアクセシブルネームを担う。
//!
//! # 詳細アコーディオンを全件 open + disabled で固定する理由
//!
//! `faq_split_accordion.rs`・`careers_split_accordion.rs`・
//! `feature_accordion_image.rs` と同じ判断で、無 JS の docs サイトでは
//! `item_trigger` が `disabled: false` のフォーカス可能な `<button>` として
//! 出力されるため、閉じた項目を残すと本文が事実上到達不能になる
//! （`docs/policy/intentional-non-adoption.md` の UI 部品責務境界にある
//! 「アクセシビリティ（WAI-ARIA・キーボード操作）」に反する）。3 項目
//! （説明・素材と手入れ・配送と返品）をすべて `OpenState::Open` +
//! `AccordionProps { disabled: true, .. }` で固定する。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。カート追加ボタンは [`button::button`] の既定 `type="button"`
//! のまま送信先を持たない。パンくずのリンクは `href="#"` を避け、実在する
//! 相対パス（`../`）へ向ける（`sidebar_03.rs` と同じ判断）。
//!
//! # ダミー素材について
//!
//! 画像は [`dummy_assets`] のビルド時生成 SVG（`PRODUCT_SRC` を主画像に、
//! サムネイルは既存 4 種を巡回利用）のみを使い、`data:` URI・外部 URL は
//! 使わない。商品名・価格・評価件数・色名・サイズ・説明文はすべて独自に
//! 書いた架空の文言であり、実在の企業名・人物・PII を含まない。
//!
//! # 集約元差分・スコープ外（#3069 へ送る）
//!
//! 集約元 R0613（単一画像 + 枚数表示・丸色見本）・R0614（タブ詳細）・
//! R0615（段落列 + 共有行）・R0616（縦サムネ列）・R0617（評価なし +
//! サイズガイド統合）・R1176（サムネ切替タブ + 開閉式詳細）の併記、在庫
//! 僅少警告（`progress`）、サイズガイド（`dialog`）、`tabs` 型詳細、
//! 共有リンク行は後半 #3069 の担当とし、本 block には含めない
//! （`site/blocks/product-overview-gallery-split.md` の「原案差分メモ」節
//! 参照）。`_/blocks-intake/` の参照ファイルは本イシュー着手時点で本
//! worktree に存在しないため、対応表 ID のみを記す（`page_heading_avatar.rs`
//! と同じ扱い）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
/// 4 枚とも同一のプレースホルダー画像であり異なる商品写真ではないため、
/// `alt` は連番を付けず全枚同一の文言にする（レビュー指摘対応、Codex:
/// 「商品画像1」〜「商品画像4」という連番の `alt` は 4 枚が異なる写真で
/// あることを含意し、実態〔同一画像の使い回し〕と食い違い誤解を招く）。
///
/// `carousel::item` の `current` 引数は `index == 0` の 1 枚のみ `true` を
/// 渡す（レビュー指摘対応、Codex）。`data-current` はカルーセルの
/// headless 契約上「現在選択中の 1 件」を示す属性であり
/// （`fandhe_frontend_headless_ui::carousel::item` 参照）、
/// [`LAYOUT_CSS`] の上書きで 4 枚全てが横並びに常時可視であることは
/// 「選択状態」とは別の関心である。全枚を `true` にすると「4 件とも
/// 選択中」という誤った状態表現になるため、初期選択（メイン画像が
/// 表示する 1 枚目）のみを `current` とし、可視性の表現は行わない
/// （styled carousel に `data-current`/`data-inview` 向けの視覚強調規則が
/// ないため見た目には影響しない）。
fn thumbnail(index: usize, count: usize) -> Node {
    let src = dummy_assets::PRODUCT_SRC;
    let alt = "ワイヤレスヘッドホン 商品画像（プレースホルダー）";
    carousel::item(
        Orientation::Horizontal,
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/product-overview-gallery-split/",
    title: "product-overview-gallery-split",
    category: BlockCategory::ProductOverview,
    rust_source:
        "crates/docs-site/src/blocks/ecommerce/product_overview/product_overview_gallery_split.rs",
    demo_class: "blocks-product-overview-gallery-split",
    parts: &[
        Part {
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Carousel",
            path: "/themes/carousel/",
        },
        Part {
            label: "Breadcrumb",
            path: "/themes/breadcrumb/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
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
            label: "Radio Card",
            path: "/themes/radio-card/",
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
            label: "Accordion",
            path: "/themes/accordion/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `product_overview_gallery_split` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「block 固有 CSS の置き場」節と同型）。`48rem` 以上で
/// 2 カラム、未満は縦積み（モジュール doc「レイアウト（骨格）」節参照）。
/// `display: none` は使わない。
///
/// # サムネイル `carousel` の item 上書き（レビュー指摘対応）
///
/// styled carousel の既定 `item` 規則（`[data-scope="carousel"]
/// [data-part="item"]`、詳細度 (0,2,0)）は `flex: 0 0 100%`（1 枚だけを
/// 表示するスライドショー前提）を持つため、無上書きのままだと 4 枚の
/// サムネイルがそれぞれ container 全幅を占め、1 枚目だけが実質的に見え、
/// 残り 3 枚は `root` の `overflow: hidden` を含む既定挙動の外側へ追い
/// やられる（前後トリガーを出さない本 Demo では index を変えられず到達
/// 不能になる）。`.blocks-product-overview-gallery-split-thumbs`
/// （`item-group` へ付与）を祖先にした子孫セレクタ（詳細度 (0,3,0)）で
/// `item` を `flex: 0 0 auto` へ上書きし、[`thumbnail`] が固定する
/// `4rem` 角のサムネイル画像がそのままの寸法で横並びになるようにする
/// （`.thumbs` 側の既存 `overflow-x: auto` がそのまま横スクロール領域を
/// 担う）。`root` の `overflow: hidden` 自体は本 block では無害
/// （item 群は自身の `item-group` 内でスクロールし `root` の外へは
/// はみ出さない）だが、指摘の趣旨に沿い `overflow: visible` へ明示的に
/// 中和し「他パーツを隠す既定値のまま残さない」ことを機械的に固定する。
///
/// # 色・サイズ選択 radio card と詳細アコーディオンの disabled 減光の中和
///
/// `pricing_single_split.rs`「支払周期 radio card をネイティブ disabled に
/// する理由」節と同じ判断: ネイティブ disabled はアクセシビリティ上の
/// 理由（無 JS で選択を実配線できない）で付けているだけで、選択自体は
/// 「無効化された機能」ではないため、styled radio-card の既定
/// `disabled_declarations()`（`opacity: 0.5` + `cursor: not-allowed`）を
/// `.blocks-product-overview-gallery-split-option` 祖先の子孫セレクタ
/// （詳細度 (0,4,0)、`item` disabled 規則の詳細度 (0,3,0) より高い）で
/// 中和する。詳細アコーディオンも同型: `AccordionProps { disabled: true,
/// .. }` は「本文を閉じさせない」ためのフォーカス制御目的であり、
/// styled accordion の `item-trigger` disabled 規則
/// （`[data-scope="accordion"][data-part="item-trigger"][data-disabled]`、
/// 詳細度 (0,3,0)）を `[data-blocks-product-overview-gallery-split-details]`
/// 祖先の子孫セレクタ（詳細度 (0,4,0)）で中和する。
const LAYOUT_CSS: &str = "\
.blocks-product-overview-gallery-split-layout {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-product-overview-gallery-split-gallery {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n}\n\
.blocks-product-overview-gallery-split-thumbs {\n  display: flex;\n  flex-wrap: nowrap;\n  overflow-x: auto;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-product-overview-gallery-split-thumbs [data-scope=\"carousel\"][data-part=\"item\"] {\n  flex: 0 0 auto;\n  overflow: visible;\n}\n\
.blocks-product-overview-gallery-split-gallery [data-scope=\"carousel\"][data-part=\"root\"] {\n  overflow: visible;\n}\n\
img[data-scope=\"image\"][data-blocks-product-overview-gallery-split-thumb] {\n  width: 4rem;\n  height: 4rem;\n  flex-shrink: 0;\n}\n\
img[data-scope=\"image\"][data-blocks-product-overview-gallery-split-main-image] {\n  width: 100%;\n}\n\
.blocks-product-overview-gallery-split-panel {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  min-width: 0;\n}\n\
.blocks-product-overview-gallery-split-price {\n  margin: 0;\n  font-size: var(--fandhe-font-size-xl);\n  font-weight: var(--fandhe-font-weight-bold);\n}\n\
.blocks-product-overview-gallery-split-option {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-product-overview-gallery-split-option [data-scope=\"radio-card\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-blocks-product-overview-gallery-split-details] [data-scope=\"accordion\"][data-part=\"item-trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
@media (min-width: 48rem) {\n  .blocks-product-overview-gallery-split-layout {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);\n    align-items: start;\n  }\n}\n";

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
            "data-scope=\"carousel\"",
            "data-scope=\"breadcrumb\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"rating-group\"",
            "data-scope=\"radio-card\"",
            "data-scope=\"color-swatch\"",
            "data-scope=\"button\"",
            "data-scope=\"accordion\"",
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
    fn demo_has_no_form() {
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
    fn layout_css_is_safe_and_has_two_column_breakpoint() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(!LAYOUT_CSS.contains("display: none"));
    }

    #[test]
    fn thumb_selector_specificity_beats_image_recipe_base() {
        assert!(LAYOUT_CSS.contains(
            "img[data-scope=\"image\"][data-blocks-product-overview-gallery-split-thumb]"
        ));
    }

    /// レビュー指摘対応（Cursor Bugbot、Medium）: `PRODUCT_SRC` は
    /// 160×160 の SVG のため、pre-styled-ui `image` 部品の base 規則
    /// （`max-width: 100%; height: auto`）だけでは縮小方向の制約にしか
    /// ならず、ギャラリー列幅が 160px を超える場合にメイン画像が実寸の
    /// 小さいタイルのまま表示される。`width: 100%` を明示して列幅へ追従
    /// させる（サムネイルと異なり固定 px にはしない。メイン画像は列幅に
    /// 応じて伸縮すべき領域のため）。
    #[test]
    fn main_image_selector_stretches_to_column_width() {
        assert!(LAYOUT_CSS.contains(
            "img[data-scope=\"image\"][data-blocks-product-overview-gallery-split-main-image] {\n  width: 100%;\n}"
        ));
    }

    /// レビュー指摘対応（Cursor Bugbot、High）: サムネイル carousel の
    /// `item` を `flex: 0 0 auto` へ、`root` の `overflow` を `visible` へ
    /// 上書きし、既定のスライドショー挙動（1 枚だけ表示・残りが到達不能）
    /// を残さないこと。
    #[test]
    fn thumbs_carousel_item_and_root_are_overridden() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-product-overview-gallery-split-thumbs [data-scope=\"carousel\"][data-part=\"item\"] {\n  flex: 0 0 auto;"
        ));
        assert!(LAYOUT_CSS.contains(
            ".blocks-product-overview-gallery-split-gallery [data-scope=\"carousel\"][data-part=\"root\"] {\n  overflow: visible;"
        ));
    }

    /// レビュー指摘対応（Codex）: `data-current`/`data-inview` は
    /// 「現在選択中の 1 件」を示す headless carousel の契約であり、
    /// [`LAYOUT_CSS`] の上書きでサムネイル 4 枚が常時可視であることとは
    /// 別の関心のため、初期選択の 1 枚（index 0）だけに付き、4 枚全てには
    /// 付かないこと（`thumbnail` 関数 doc 参照）。
    #[test]
    fn only_first_thumbnail_is_marked_current() {
        let html = demo_html();
        // data-inview はサムネイル carousel::item にしか出ないため単独で
        // 検証できる。data-current はパンくずの current_link（末尾項目）
        // でも 1 件出るため、サムネイル分と合わせた合計 2 件で検証する。
        assert_eq!(html.matches("data-inview").count(), 1, "html={html}");
        assert_eq!(html.matches("data-current").count(), 2, "html={html}");
    }

    /// レビュー指摘対応（Codex）: 4 枚のサムネイルは同一のプレースホルダー
    /// 画像を使い回しており、異なる商品写真ではない。`alt` に連番を付けて
    /// 「商品画像1」〜「商品画像4」のように異なる写真を装うと実態と食い
    /// 違うため、4 枚とも同一の `alt` を持つこと。
    #[test]
    fn thumbnail_alt_text_does_not_imply_distinct_photos() {
        let html = demo_html();
        assert_eq!(
            html.matches("ワイヤレスヘッドホン 商品画像（プレースホルダー）")
                .count(),
            4,
            "html={html}"
        );
    }

    /// レビュー指摘対応（Cursor Bugbot、Medium）: 固定 disabled の
    /// radio card・アコーディオンが `disabled_declarations()` の
    /// `opacity: 0.5` で薄く表示され続けないよう中和すること
    /// （`pricing_single_split.rs` と同型の判断）。
    #[test]
    fn disabled_dimming_is_neutralized_for_options_and_details() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-product-overview-gallery-split-option [data-scope=\"radio-card\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-product-overview-gallery-split-details] [data-scope=\"accordion\"][data-part=\"item-trigger\"][data-disabled] {\n  opacity: 1;"
        ));
    }

    #[test]
    fn accordion_items_are_open_and_disabled() {
        let html = demo_html();
        // アコーディオン 3 項目すべてが disabled のトリガー（ネイティブ
        // disabled 属性 + aria-disabled + data-disabled）を持つこと。
        assert!(html.matches("aria-disabled=\"true\"").count() >= 3);
        assert!(html.matches("data-state=\"open\"").count() >= 3);
    }

    #[test]
    fn radio_cards_declare_checked_state() {
        let html = demo_html();
        // 色・サイズ選択それぞれ 1 件ずつ checked（`item` パーツ単位で計
        // 2 件。`item`/`item-control`/`item-indicator` の 3 パーツが同じ
        // checked 状態を反映するため、`data-state="checked"` の総出現数
        // ではなく `data-part="item"` に絞って数える）。
        assert_eq!(
            html.matches("data-part=\"item\" data-state=\"checked\"")
                .count(),
            2
        );
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-product-overview-gallery-split-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-product-overview-gallery-split-layout"
        );
    }
}
