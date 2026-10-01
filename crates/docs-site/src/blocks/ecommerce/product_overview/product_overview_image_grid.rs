//! `product-overview-image-grid` block（イシュー #3071/#3072、親 #3070
//! 「商品詳細（複数画像グリッド + 購入パネル）」。Ecommerce / Product
//! Overview カテゴリ）。上段に商品画像グリッド、下段または横に購入パネル
//! （商品名・価格・評価・色/サイズ選択・カート追加ボタン・説明）を持つ
//! 商品詳細画面を、集約元 2 件の差分・選択状態違いを並記して実装する。
//! 主参照は対応表 ID R1178（集約元 R1175: 段差配置 + 右に購入パネル /
//! R0612: 2 列グリッド・左右反転）。`_/blocks-intake/` の対応ファイルは
//! 本イシュー着手時点で本 worktree に存在しないため、原稿・本コメントには
//! 対応表 ID のみを記し、レイアウトは親 issue #3070 の仕様文に従って独自に
//! 組む（`cart-two-column-summary` #3033 と同じ扱い）。
//!
//! # 使用部品
//!
//! `breadcrumb` / `image` / `heading` / `text` / `rating-group` /
//! `radio-card` / `color-swatch` / `button` の 8 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。レビュー件数は実在するレビュー領域を
//! 持たない（下記「レビュー件数はリンクにしない」節参照）ため `link` は
//! 使わず、`text` の可視テキストとして表示する。
//!
//! # `class` と `data-*` の使い分け
//!
//! [`fandhe_frontend_pre_styled_ui::breadcrumb::root`]・
//! [`fandhe_frontend_pre_styled_ui::image::image`]・
//! [`fandhe_frontend_pre_styled_ui::rating_group::root`]・
//! [`fandhe_frontend_pre_styled_ui::radio_card::root`]・
//! [`fandhe_frontend_pre_styled_ui::button::button`] はいずれも
//! `drop_class_attr` で呼び出し側 `class` を除去してから内部 variant
//! クラスと合成するため、これらへの CSS フックは `data-*` 属性で渡す
//! （`data-blocks-product-overview-image-grid-*`、`cart_two_column_summary`
//! と同型の判断）。レイアウト用ラッパー（グリッド・パネル・行・版の並記）は
//! 素の `<div>` のため `class="blocks-product-overview-image-grid-*"` を
//! 使う。
//!
//! # 狭幅では画像 1 列積み・パネルがその下へ回る（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container`
//! （コンテナクエリ）で判定する（`cart_two_column_summary` と同型の
//! パターン）。[`LAYOUT_CSS`] のラッパー
//! `.blocks-product-overview-image-grid-stack` へ `container-type:
//! inline-size` を宣言し、コンテナ幅が `40rem` 未満のとき次をすべて戻す:
//! 画像グリッドを 1 カラムへ（1 枚目の段差 `grid-column`/`grid-row` の
//! span・最後の 1 枚の全幅 `grid-column` もいずれも `auto` へ）、2 列
//! 均等グリッド（`-gallery-uniform`）も 1 カラムへ、2 カラム配置
//! （`-body-side`）も縦積みへ。しきい値は全版で `40rem` の 1 つに統一し、
//! 版ごとに別のクエリは増やさない。
//!
//! # 色/サイズ選択は `disabled` の静的表示（無 JS）
//!
//! docs サイトは JS ハイドレーションを行わないため、選択操作を実行時に
//! 反映できない。`card_form_footer` の支払方法 radio card と同じ判断で、
//! 色・サイズ選択はどちらも [`fandhe_frontend_pre_styled_ui::radio_card`]
//! を `disabled: true` + `aria-disabled="true"` で描き、版ごとに異なる
//! 初期選択（下記「3 版の並記」節の表）のみを固定表示する。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。カート追加
//! ボタンは [`fandhe_frontend_pre_styled_ui::button::button`] の既定
//! `type="button"` のまま用いる。
//!
//! # 評価のアクセシブルネーム・radio グループ名の版ごとの一意化
//!
//! [`fandhe_frontend_pre_styled_ui::rating_group::label`] の `id`、
//! [`fandhe_frontend_pre_styled_ui::radio_card::label`] の `id`、パンくずの
//! `aria-label` はいずれも [`Variant::suffix`] を連結した版固有の値にする
//! （`crates/docs-site/tests/blocks_contract.rs` の重複 id・宙ぶらりん
//! aria 参照検査、および `ai_chat_code_preview` で Bugbot が同名 landmark
//! を指摘した前例への対応）。**radio の hidden input の `name`
//! （[`fandhe_frontend_pre_styled_ui::radio_card::item_hidden_input`]）も
//! 同様に版ごとに一意にする**: `<form>` のない文書では同じ `name` の
//! radio が 1 つのグループとして扱われ、3 版それぞれの `checked` のうち
//! 最後の 1 件しか残らない（前の版の初期選択表示が消える）ため、
//! `blocks-product-overview-image-grid-color-{suffix}` のように
//! 名前空間を付ける（`product_overview_gallery_split` の `COLOR_NAME` と
//! 同じ判断）。
//!
//! # 3 版の並記（集約元差分・状態違い、イシュー #3072）
//!
//! [`demo`] は [`VARIANTS`] の 3 版をキャプション付きで縦に並べる
//! （`product_overview_tabs_below`・`ai_chat_code_preview` と同型の
//! 判断）。各版の差分は [`Variant`] 1 つで表し、`shell`/`caption` の 2 関数
//! だけで組み立てる（enum・builder は作らない）。
//!
//! | 版 | キャプション（要旨） | レイアウト | 画像グリッド | 状態 |
//! |---|---|---|---|---|
//! | `stacked` | 代表構成 | 縦積み（画像グリッド上段・パネル下段） | 段差配置（hero/wide） | 在庫あり・チャコール/M |
//! | `side` | R1175: 段差配置 + 右パネル | 2 カラム（画像が左・パネルが右） | 段差配置（hero/wide） | 在庫あり・シルバー/L |
//! | `reverse` | R0612: 2 列グリッド・左右反転・在庫切れ | 2 カラム反転（パネルが左・画像が右） | 2 列均等グリッド | 在庫切れ・ネイビー/S |
//!
//! 左右反転（`reverse`）は CSS の `grid-template-areas` のみで行い、DOM
//! 順は常に画像 → パネルに保つ。読み上げ順の意味は変わらない
//! （WCAG 1.3.2 の観点で可、視覚順の入れ替えは 2 カラム時のみ）。
//!
//! 見出し階層は `crate::blocks::insert_generated_sections` が出す
//! H1 → H2「Demo」→（本 Demo）→ H2「使用部品」の中に、版キャプション
//! （[`caption`]、`<h3>`）→ 商品名（[`purchase_panel`]、`HeadingLevel::H4`）
//! の 2 段を持つ。キャプションを H2 にすると「Demo」H2 と同列になり構造が
//! 崩れるため、キャプションは `<h3>` のまま商品名を H4 に下げている
//! （PR #3522 Codex レビューの是正）。
//!
//! # レビュー件数はリンクにしない
//!
//! 当初はレビュー詳細ページへのリンクとして実装したが、レビュー詳細ページ
//! （Ecommerce / Reviews カテゴリ）は本イシュー時点で block 未登録のため
//! 実在せず、リンク先が単なる自己参照（`href="./"`）になり遷移として機能
//! しない指摘（PR #3468 レビュー）を受けて、実在しないリンク先を作らず
//! [`fandhe_frontend_pre_styled_ui::text`] の可視テキストとして表示する
//! よう改めた。親 issue #3070 の使用部品リストには `link` も挙げられて
//! いるが、上記の理由で意図的に不採用とする（[`BLOCK`] の `parts` は
//! 8 部品のまま変えない）。実在の Reviews block が追加され次第、その
//! 相対パスへの `link` へ版ごと差し替える（本 PR のスコープ外、追跡は
//! `out-of-scope-tracking.md` の手順に従う）。
//!
//! # 下段の全幅画像は行サイズ計算から独立させる
//!
//! [`LAYOUT_CSS`] の `.blocks-product-overview-image-grid-gallery` は
//! `grid-auto-rows: minmax(0, 1fr) minmax(0, 1fr) auto;` とし、1・2 行目
//! （hero が span する 2 行）のみ `1fr` を共有させ、3 行目（`wide` タイル）
//! は `auto` にする。全行を単一の `minmax(0, 1fr)` にすると、正方形
//! `aspect_ratio` の全幅画像（3 列分の幅）が要求する高さへ他の行も
//! 揃ってしまい、意図した 2×2 の段差配置が崩れる（PR #3468 レビュー P1
//! 指摘）。`auto` にした 3 行目は `fr` の使用可能領域分配（flex fraction）
//! に加わらないため、1・2 行目のサイズ計算から切り離される。この配置は
//! `uniform_gallery` 版（2 列均等グリッド）には適用しない
//! （[`LAYOUT_CSS`] の `.blocks-product-overview-image-grid-gallery-uniform`
//! が base の `grid-auto-rows` を上書きする）。
//!
//! # ダミー素材について
//!
//! 商品名・価格・レビュー件数・色/サイズ・説明・在庫注記はすべて本ファイル
//! 内の架空データ（実在のブランド・商品・PII を含まない）。商品画像は
//! ビルド時生成の同梱 SVG（[`dummy_assets::PRODUCT_SRC`]）を各版 4 枚使う
//! （外部 URL・`data:` URI は使わない）。4 枚とも同一画像のため、内容と
//! 一致する `alt`（商品名）を持てるのは各版の代表画像（1 枚目）のみとし、
//! 残りは代表画像の重複描画として空 `alt`（装飾扱い）にする（PR #3468
//! レビュー P2 指摘の是正、詳細は [`gallery_tile`] のコメント参照）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/product-overview-image-grid/",
    title: "product-overview-image-grid",
    category: BlockCategory::ProductOverview,
    rust_source:
        "crates/docs-site/src/blocks/ecommerce/product_overview/product_overview_image_grid.rs",
    demo_class: "blocks-product-overview-image-grid",
    parts: &[
        Part {
            label: "Breadcrumb",
            path: "/themes/breadcrumb/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
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
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `product_overview_image_grid` 固有のレイアウト規則
/// （`crate::blocks::LAYOUT_CSS` doc「block 固有 CSS の置き場」節と同型）。
/// `data-blocks-product-overview-image-grid-tile` は `image` 部品が生成する
/// `<img>` 自体に付与されるため、グリッド配置の対象要素と幅制約の対象
/// 要素が同一である。`image` 部品の既定 CSS（`max-width: 100%; height: auto`）
/// は画像の縮小のみを保証し、hero（2 列×2 行）・wide（3 列全幅）のような
/// 複数トラックにまたがるグリッド領域を画像が自動的に埋めることは保証
/// しないため、両セレクタへ `width: 100%` を明示する（PR #3468 レビュー
/// P1 是正）。
///
/// `-body-side`/`-body-reverse` は `grid-template-areas` で画像グリッドと
/// 購入パネルの左右配置・反転を切り替える（モジュール doc「3 版の並記」
/// 節参照）。`body` コンテナ（ギャラリー・購入パネルの共通親）が
/// `blocks-product-overview-image-grid-stack` の flex スタックから独立する
/// ため、`gap` は `body`（既定の縦積み・`-body-side` 共通の基準値）と
/// `-body-side`（グリッド）・狭幅時の flex リセットの双方へ個別に明示する
/// （PR #3522 Bugbot 指摘の是正。`stack` 側の `gap` は継承されない）。
/// docs 本文幅（`--docs-max-content-width: 46rem`）の制約上
/// `@container` の 2 カラム化しきい値は実質発火しないため、2 カラム化は
/// 既定スタイルで行い、既存の `@container blocks-product-overview-image-grid
/// (max-width: 40rem)` 内で縦積みへ戻す（新しいクエリは足さない）。
///
/// 色・サイズ選択の radio card は無 JS で選択を実配線できないため
/// ネイティブ disabled で固定しているだけで、選択肢自体は「利用不可」では
/// ない。styled radio-card の既定 `disabled_declarations()`（`opacity: 0.5`
/// と `cursor: not-allowed`、`item` disabled 規則の詳細度 (0,3,0)）で
/// 減光されないよう、`.blocks-product-overview-image-grid-options` 祖先の
/// 子孫セレクタ（詳細度 (0,4,0)）で中和する
/// （`product_overview_gallery_split.rs`・`card_form_footer.rs` と同型、
/// PR #3468 Bugbot 指摘の是正）。
///
/// `.blocks-product-overview-image-grid-options [data-scope="radio-card"]
/// [data-part="root"]` へ `flex-wrap: wrap` を明示するのは、2 カラム配置
/// （`-body-side`、パネル幅は約 20rem）だと横並び
/// （`Orientation::Horizontal`）のサイズ選択肢 4 件・色選択肢 3 件が
/// 折り返さずパネル外へはみ出すため（PR #3522 Codex 指摘の是正）。
/// `radio_card::root` の recipe（`crates/pre-styled-ui/src/radio_card.rs`）
/// は horizontal のとき `display: flex; flex-direction: row` のみで
/// `flex-wrap` を指定しないため、block 側のセレクタ（詳細度 (0,3,0)）が
/// 上書きする。部品本体や `Orientation` の切り替えはしない。
const LAYOUT_CSS: &str = "\
.blocks-product-overview-image-grid-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-product-overview-image-grid;\n}\n\
.blocks-product-overview-image-grid-demo {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-product-overview-image-grid-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-product-overview-image-grid-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-product-overview-image-grid-body-side {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  grid-template-areas: \"gallery panel\";\n  align-items: start;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-product-overview-image-grid-body-side > .blocks-product-overview-image-grid-gallery {\n  grid-area: gallery;\n}\n\
.blocks-product-overview-image-grid-body-side > .blocks-product-overview-image-grid-panel {\n  grid-area: panel;\n}\n\
.blocks-product-overview-image-grid-body-reverse {\n  grid-template-areas: \"panel gallery\";\n}\n\
.blocks-product-overview-image-grid-gallery {\n  display: grid;\n  grid-template-columns: repeat(3, minmax(0, 1fr));\n  grid-auto-rows: minmax(0, 1fr) minmax(0, 1fr) auto;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-product-overview-image-grid-gallery-uniform {\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  grid-auto-rows: auto;\n}\n\
.blocks-product-overview-image-grid-gallery-uniform [data-scope=\"image\"] {\n  width: 100%;\n}\n\
[data-blocks-product-overview-image-grid-tile=\"hero\"] {\n  grid-column: span 2;\n  grid-row: span 2;\n  width: 100%;\n}\n\
[data-blocks-product-overview-image-grid-tile=\"wide\"] {\n  grid-column: 1 / -1;\n  width: 100%;\n}\n\
.blocks-product-overview-image-grid-panel {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  max-width: 40rem;\n}\n\
.blocks-product-overview-image-grid-rating-row {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-product-overview-image-grid-options {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-product-overview-image-grid-options [data-scope=\"radio-card\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-product-overview-image-grid-options [data-scope=\"radio-card\"][data-part=\"root\"] {\n  flex-wrap: wrap;\n}\n\
@container blocks-product-overview-image-grid (max-width: 40rem) {\n  \
.blocks-product-overview-image-grid-gallery {\n    grid-template-columns: minmax(0, 1fr);\n  }\n  \
.blocks-product-overview-image-grid-gallery-uniform {\n    grid-template-columns: minmax(0, 1fr);\n  }\n  \
[data-blocks-product-overview-image-grid-tile=\"hero\"] {\n    grid-column: auto;\n    grid-row: auto;\n  }\n  \
[data-blocks-product-overview-image-grid-tile=\"wide\"] {\n    grid-column: auto;\n  }\n  \
.blocks-product-overview-image-grid-body-side {\n    display: flex;\n    flex-direction: column;\n    gap: var(--fandhe-space-6);\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, COLOR_OPTIONS, LAYOUT_CSS, PRODUCT_NAME, SIZE_OPTIONS, VARIANTS};
    use fandhe_frontend_core::render;

    fn demo_html() -> String {
        render(&demo())
    }

    #[test]
    fn demo_composes_expected_parts() {
        let html = demo_html();
        for scope in [
            "data-scope=\"breadcrumb\"",
            "data-scope=\"image\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"rating-group\"",
            "data-scope=\"radio-card\"",
            "data-scope=\"color-swatch\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        // 3 版 × 4 枚 = 12 枚。段差配置の版（stacked/side）は各 1 件ずつ
        // hero/wide を持ち、2 列均等版（reverse）はどちらも持たない。
        assert_eq!(html.matches("<img").count(), 12);
        assert_eq!(
            html.matches("data-blocks-product-overview-image-grid-tile=\"hero\"")
                .count(),
            2
        );
        assert_eq!(
            html.matches("data-blocks-product-overview-image-grid-tile=\"wide\"")
                .count(),
            2
        );
        // 4 枚とも同一画像のため、内容と一致する alt を持てるのは各版の
        // 代表画像（1 枚目）のみ。異なる商品写真であるかのような alt
        // （「画像 1」〜「画像 4」等）を付けない（PR #3468 レビュー P2 是正）。
        assert_eq!(html.matches(&format!("alt=\"{PRODUCT_NAME}\"")).count(), 3);
        assert_eq!(html.matches("alt=\"\"").count(), 9);
        assert!(html.contains("レビュー 128 件"));
        assert!(
            !html.contains("data-scope=\"link\""),
            "review count should be plain text, not a link (PR #3468 review P2)"
        );
        let expected_radios = 3 * (COLOR_OPTIONS.len() + SIZE_OPTIONS.len());
        assert_eq!(html.matches("type=\"radio\"").count(), expected_radios);
        // `data-checked=""`（rating item 等）は部分文字列として
        // `checked=""` を含むため、先頭スペース付きで hidden input の
        // `checked=""` 属性のみを数える（版ごとに色・サイズ各 1 件 = 2、
        // 3 版で合計 6 件）。
        assert_eq!(html.matches(" checked=\"\"").count(), 6);
        assert_eq!(html.matches("<button").count(), 3);
    }

    #[test]
    fn demo_has_no_form_and_no_unsafe_html() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("data:"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("<script"));
    }

    /// カート追加ボタンが disabled の静的表示であること（無 JS・フォーム
    /// なしの Demo では押しても何も起きないため、`product_overview_tabs_below`
    /// と同様に操作不能を明示する）。
    #[test]
    fn cart_button_is_disabled() {
        let html = demo_html();
        assert!(html.contains("disabled"));
        assert!(html.contains("カートに追加"));
    }

    /// 無 JS の文書で同名 radio グループが版をまたいで衝突しないこと
    /// （モジュール doc「radio グループ名の版ごとの一意化」節の回帰防止）。
    /// 旧 `name="color"`/`name="size"` が出力されず、版ごとの `name` で
    /// ちょうど 1 件の初期選択（` checked=""`）が残ることを固定する。
    #[test]
    fn radio_names_are_unique_per_variant() {
        let html = demo_html();
        assert!(!html.contains("name=\"color\""));
        assert!(!html.contains("name=\"size\""));
        for variant in &VARIANTS {
            let color_name = format!(
                "name=\"blocks-product-overview-image-grid-color-{}\"",
                variant.suffix
            );
            let size_name = format!(
                "name=\"blocks-product-overview-image-grid-size-{}\"",
                variant.suffix
            );
            assert_eq!(html.matches(&color_name).count(), 3);
            assert_eq!(html.matches(&size_name).count(), 4);
        }
    }

    /// 3 版のキャプション・在庫状態違い・レイアウト修飾クラスがすべて
    /// 出力に現れること（モジュール doc「3 版の並記」節の表の固定）。
    #[test]
    fn variants_render_captions_and_states() {
        let html = demo_html();
        for variant in &VARIANTS {
            assert!(
                html.contains(variant.caption),
                "missing caption: {}",
                variant.caption
            );
        }
        assert_eq!(html.matches("カートに追加").count(), 2);
        // キャプション文言（「...在庫切れ（集約元 R0612）」）とボタン文言
        // （「在庫切れ」）の 2 件。
        assert_eq!(html.matches("在庫切れ").count(), 2);
        assert!(html.contains(super::OUT_OF_STOCK_NOTE));
        assert_eq!(
            html.matches("blocks-product-overview-image-grid-body-side")
                .count(),
            2
        );
        assert_eq!(
            html.matches("blocks-product-overview-image-grid-body-reverse")
                .count(),
            1
        );
        assert_eq!(
            html.matches("blocks-product-overview-image-grid-gallery-uniform")
                .count(),
            1
        );
    }

    #[test]
    fn layout_css_uses_only_namespaced_selectors_and_tokens() {
        assert!(!LAYOUT_CSS.contains('<'));
        for rule in LAYOUT_CSS.split("}\n") {
            let rule = rule.trim();
            if rule.is_empty() {
                continue;
            }
            assert!(
                rule.starts_with(".blocks-product-overview-image-grid")
                    || rule.starts_with("[data-blocks-product-overview-image-grid")
                    || rule.starts_with("@container blocks-product-overview-image-grid"),
                "unexpected rule: {rule}"
            );
        }
        assert!(LAYOUT_CSS.contains("container-type: inline-size;"));
        assert!(
            LAYOUT_CSS.contains("@container blocks-product-overview-image-grid (max-width: 40rem)")
        );
        assert!(LAYOUT_CSS.contains("[data-blocks-product-overview-image-grid-tile=\"wide\"]"));
    }

    /// side/reverse の 2 カラム配置・uniform の 2 列グリッド・狭幅での
    /// 縦積み復帰ルールが揃っていること（モジュール doc「3 版の並記」節）。
    #[test]
    fn layout_css_side_and_uniform_rules() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-product-overview-image-grid-body-side {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  grid-template-areas: \"gallery panel\";\n  align-items: start;\n  gap: var(--fandhe-space-6);\n}"
        ));
        assert!(LAYOUT_CSS.contains(
            ".blocks-product-overview-image-grid-body-reverse {\n  grid-template-areas: \"panel gallery\";\n}"
        ));
        assert!(LAYOUT_CSS.contains(
            ".blocks-product-overview-image-grid-gallery-uniform {\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  grid-auto-rows: auto;\n}"
        ));
        assert!(LAYOUT_CSS.contains(
            "@container blocks-product-overview-image-grid (max-width: 40rem) {\n  .blocks-product-overview-image-grid-gallery {\n    grid-template-columns: minmax(0, 1fr);\n  }\n  .blocks-product-overview-image-grid-gallery-uniform {\n    grid-template-columns: minmax(0, 1fr);\n  }"
        ));
        assert!(LAYOUT_CSS.contains(
            ".blocks-product-overview-image-grid-body-side {\n    display: flex;\n    flex-direction: column;\n    gap: var(--fandhe-space-6);\n  }"
        ));
    }

    /// 商品名（H2）が全版共通で見出し/領域ナビゲーションでは版を区別
    /// できない指摘（Codex レビュー、PR #3522）の回帰防止。各版の
    /// `section` が版固有のキャプション見出し `<h3>` を内部に持ち、
    /// `aria-labelledby` でそれを参照していることを固定する。
    #[test]
    fn section_has_unique_accessible_name_via_caption_heading() {
        let html = demo_html();
        for variant in &VARIANTS {
            let heading_id = format!(
                "blocks-product-overview-image-grid-caption-{}",
                variant.suffix
            );
            let labelledby_attr = format!("aria-labelledby=\"{heading_id}\"");
            let heading_id_attr = format!("id=\"{heading_id}\"");
            let heading_close = format!("{}</h3>", variant.caption);
            assert!(
                html.contains(&labelledby_attr),
                "missing labelled section for {}",
                variant.suffix
            );
            assert!(
                html.contains(&heading_id_attr),
                "missing caption heading id for {}",
                variant.suffix
            );
            assert!(
                html.contains(&heading_close),
                "missing caption heading text for {}",
                variant.suffix
            );
            // 見出し id が h3 要素自身に付与されていること（他要素への
            // 迷子参照でないこと）を確認する。`blocks_source_does_not_use_raw_html_or_build_html_strings`
            // の HTML 文字列直接組み立て検知を避けるため `format!("<...")`
            // を使わず `String::from` + 文字列結合で組み立てる。
            let h3_open_with_id =
                String::from("h3 class=\"blocks-product-overview-image-grid-caption\" ")
                    + &heading_id_attr;
            assert!(
                html.contains(&h3_open_with_id),
                "caption heading id should be on the h3 element for {}",
                variant.suffix
            );
        }
    }

    #[test]
    fn disabled_radio_cards_are_not_dimmed() {
        // 無 JS 固定のためのネイティブ disabled で選択肢が減光表示され
        // ないことを固定する（PR #3468 Bugbot 指摘の回帰防止）。
        assert!(LAYOUT_CSS.contains(
            ".blocks-product-overview-image-grid-options [data-scope=\"radio-card\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}"
        ));
    }

    /// 商品名見出しが版キャプション（`<h3>`）の下位（`<h4>`）であること
    /// （PR #3522 Codex レビューの回帰防止。見出し階層はモジュール doc
    /// 「3 版の並記」節参照）。
    #[test]
    fn product_name_heading_is_below_caption() {
        let html = demo_html();
        assert_eq!(html.matches("<h2").count(), 0);
        assert_eq!(html.matches("<h3").count(), VARIANTS.len());
        assert_eq!(html.matches("<h4").count(), VARIANTS.len());
    }

    /// 2 カラム配置時に選択肢（radio-card）が折り返すこと（はみ出し禁止、
    /// PR #3522 Codex レビューの回帰防止）。
    #[test]
    fn options_radio_cards_wrap() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-product-overview-image-grid-options [data-scope=\"radio-card\"][data-part=\"root\"] {\n  flex-wrap: wrap;\n}"
        ));
    }

    #[test]
    fn hero_and_wide_tiles_fill_their_grid_area_width() {
        // image 部品の既定 CSS（max-width: 100%; height: auto）だけでは
        // 複数トラックにまたがるグリッド領域を画像が埋めないため、両方の
        // 段差配置トラックへ width: 100% が明示されていることを固定する
        // （PR #3468 レビュー P1 是正の回帰防止）。
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-product-overview-image-grid-tile=\"hero\"] {\n  grid-column: span 2;\n  grid-row: span 2;\n  width: 100%;\n}"
        ));
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-product-overview-image-grid-tile=\"wide\"] {\n  grid-column: 1 / -1;\n  width: 100%;\n}"
        ));
    }
}
