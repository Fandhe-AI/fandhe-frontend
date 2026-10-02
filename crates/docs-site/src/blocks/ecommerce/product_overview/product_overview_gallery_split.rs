//! `product-overview-gallery-split` block（前半骨格はイシュー #3068、残り
//! 領域・状態表示・原稿の仕上げは #3069。親トラッキング #3067「商品詳細
//! （サムネイル付きギャラリー + 購入パネル）」配下）。対応表 ID R0611
//! （主参照）・R0613/R0614/R0615/R0616/R0617/R1176（集約元）を構造の参照元
//! とする合成例。取得手段・ファイル名・内部コンポーネント識別子は記載
//! しない（`docs/design/motion-reference-adoption-policy.md` §9 と同じ
//! ライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `image` / `carousel` / `breadcrumb` / `heading` / `text` /
//! `rating-group` / `radio-card` / `color-swatch` / `button` / `accordion` /
//! `progress` / `dialog` / `link` の 13 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。実物の `tabs::tabs` は使わない
//! （下記「実物の `tabs::tabs` を使わない」節参照）ため `BLOCK::parts` に
//! `Tabs` は含めない。
//!
//! # レイアウト（骨格）
//!
//! `>= 48rem` で 2 カラム（左: ギャラリー、右: 購入パネル）、未満では
//! ギャラリー上・購入パネル下の縦積みにフォールバックする（[`LAYOUT_CSS`]）。
//! `display: none` は使わない（狭幅でも全操作要素へ到達可能なまま積む）。
//!
//! # 状態違いの並記（版 A/B/C と集約元の対応）
//!
//! [`demo`] は `ai_chat_code_preview`/`product_overview_tabs_below` と同型に
//! [`caption`] 付きで 3 版を縦に並べる（版は本 block 固有の構成差。
//! `_/blocks-intake/` の参照ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、対応表 ID のみを記す）。
//!
//! 1. **代表構成**（[`variant_a`]、対応表 ID R0611）: 横並びサムネイル +
//!    評価あり。価格の直下に在庫僅少警告（`progress`、残り 3 点）、サイズ
//!    欄の横に disabled の「サイズガイド」ボタンを置き、ボタン直下へ
//!    「『サイズガイド』を開いた場合のプレビュー」キャプション付きで
//!    非モーダルな静的開状態の `dialog` を併記する。
//! 2. **縦サムネ列・タブ型詳細・評価なし**（[`variant_b`]、対応表 ID
//!    R0616・R0614・R0617）: サムネイル `carousel` を `Orientation::Vertical`
//!    にしてメイン画像の左へ縦 1 列で置く。評価行は出さない
//!    （R0617「評価なしサイズガイド統合」）。詳細はアコーディオンの代わりに
//!    [`static_tab_list`] による非対話タブ列（説明/素材と手入れ/配送と
//!    返品/サイズガイド）で示し、選択中「説明」パネルを直下に描き、残り
//!    3 タブは [`panel_preview`] で常時可視の併記にする（R0614「タブ
//!    詳細」）。
//! 3. **単一画像・枚数表示・段落詳細・共有行**（[`variant_c`]、対応表 ID
//!    R0613・R0615）: サムネイル列を持たない単一メイン画像に「1 / 4」の
//!    枚数表示（R0613）を添える。色見本は [`SwatchShape::Circle`]
//!    （R0613）。詳細は見出し（`h3`）付き段落列（R0615「段落列」）。価格の
//!    近くに共有リンク行（R0615「共有行」）を置く。
//!
//! 対応表 ID R1176（サムネ切替タブ + 開閉式の詳細）は Demo に加えず、下記
//! 「R1176（サムネ切替タブ + 開閉式詳細）を Demo に加えない理由」節で扱う。
//! 狭幅版（`< 48rem`）の並記も行わない: 2 カラム切り替えは `LAYOUT_CSS` の
//! `@media (min-width: 48rem)` viewport クエリであり、`product_overview_
//! tabs_below` と同じ理由でページ内の並記では再現できないため。
//!
//! # R1176（サムネ切替タブ + 開閉式詳細）を Demo に加えない理由
//!
//! サムネ切替は版 A・版 B の `carousel` と同型の構造であり、開閉式の詳細は
//! 無 JS の docs サイトでは全件 open 固定になる（モジュール doc「詳細
//! アコーディオンを全件 open + disabled で固定する理由」節）ため版 A の
//! アコーディオンと見分けがつかない静的表示になる。新たな状態を Demo へ
//! 追加する価値がないため、原稿の「原案差分メモ」節でのみ言及する。
//!
//! # 実物の `tabs::tabs` を使わない（`product_overview_tabs_below`/
//! `faq_tabbed_accordion` と同じ判断）
//!
//! docs サイトは無 JS のため、実物の `tabs::tabs` を置くと「操作できる
//! ように見えて切り替わらない」`<button role="tab">` と `hidden` パネルが
//! 残る（既存 block が是正済みの経路）。本 block も [`static_tab_list`] で
//! `role`/`tabindex`/`<button>` を持たない非対話の `div` 列を置き、選択中
//! パネルはタブ列とは別に可視見出し（`h3`）を持つ節として直下に描画する。
//! 残りのタブは [`panel_preview`] が「『〜』タブを選択した場合のプレビュー」
//! キャプション付きで常時可視・縦積みに併記する（`hidden` は一切使わない）。
//! [`BLOCK::parts`] に `Tabs` は含めない（実際に呼ばない部品を掲げない、
//! 既存 block と同じ判断）。
//!
//! # サイズガイドは静的な開状態・非モーダル（`cart_dialog`/
//! `contact_dialog_form`/`auth_tabs_card` と同型）
//!
//! `trigger`/`close_trigger` は置かない（押しても開閉が切り替わらない
//! ボタンを操作可能なまま残さないための判断。`auth_tabs_card` と同じ）。
//! `aria-modal` は false にする（静的なデモは閉じる機構を実際には持たず、
//! ダイアログの外側に説明・ギャラリー・購入パネルがあるため、支援技術が
//! 外側を無視しないよう表示の実態と一致させる、`cart_dialog` と同じ
//! 判断）。`backdrop` は出さない（デモ枠内で単独の静的プレビューとして
//! 十分なため、`positioner`/`content` のみを中和して `.blocks-product-
//! overview-gallery-split-demo` の通常フロー内に収める。[`LAYOUT_CSS`]
//! 「サイズガイド dialog の中和」節参照）。本文はサイズ表を素の段落
//! （`styled_text::text`）で示し、`table` 部品は増やさない。
//!
//! # 在庫僅少の `progress`（`order_tracking_progress`/
//! `settings_page_tabs` と同型）
//!
//! `progress::root` は `aria-label` を自動では配線しないため
//! （`order_tracking_progress.rs` と同じ判断）、本 block でも
//! `("aria-label", "在庫残量")` を明示する。値は固定（在庫 3 / 20）で、
//! 可視テキストでも「残り 3 点」と明文化する（進捗バーだけに依存しない）。
//!
//! # 共有リンクは架空の固定 href（`product_overview_tabs_below` と同型）
//!
//! 実在 SNS サービス名を持ち込まず「共有 A」「共有 B」の架空ラベルにし、
//! href は本リポジトリの固定 URL（[`REPO`] 定数）にする。`href="#"` は
//! 使わない。
//!
//! # ギャラリー領域
//!
//! メイン画像（正方形・`object-fit: cover`）の下へ、横並びのサムネイル
//! `carousel` を置く（版 A）。前後トリガーは出さない（無 JS のデモでスライド
//! 送りを実際には配線できないため、index 0 が選択済みの静的表示のみとする
//! 最小構成を採る）。版 B は同じ `carousel` 構造を `Orientation::Vertical`
//! でメイン画像の左へ縦 1 列に置き、版 C はサムネイル自体を持たず単一
//! メイン画像 + 枚数表示にする。
//!
//! # 購入パネル領域
//!
//! パンくず（`breadcrumb`）→ 商品名（`heading` H2）→ 評価（`rating-group`、
//! readonly、版 B は省略）→ 価格（素の `<p>`、理由は下記）→ 在庫僅少警告
//! （版 A のみ）→ 共有リンク行（版 C のみ）→ 色選択・サイズ選択
//! （`radio-card` ×2 グループ、版 A はサイズ欄の横にサイズガイドボタンを
//! 追加）→ カート追加ボタン（`button`、全幅）→ 詳細（版ごとにアコーディオン
//! /静的タブ/段落列のいずれか）の順に縦積みする。
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
//! # 詳細を 3 通りの表現で示す理由
//!
//! 「状態違いの並記」節参照。いずれの表現も無 JS で到達不能な本文を残さ
//! ない（`docs/policy/intentional-non-adoption.md` の UI 部品責務境界）。
//! 版 A（アコーディオン）は全件 `OpenState::Open` + `disabled: true` で固定
//! し、トリガーは `h3` で包む（`faq_accordion_centered.rs`/
//! `changelog_accordion.rs` と同じ判断）。版 B（静的タブ）は選択中パネルを
//! タブ列の外に可視見出しとして描く。版 C（段落列）はそもそも開閉状態を
//! 持たない。
//!
//! # id を版接尾辞で一意化する理由
//!
//! `ai_chat_code_preview.rs` と同じ判断で、評価ラベル・色/サイズ選択
//! ラベル・radio の `name`・詳細トリガー/コンテンツの各 id を
//! `format!("{{BASE}}-{{suffix}}")`（`suffix` は `"a"`/`"b"`/`"c"`）で版ごとに
//! 一意化し、`blocks_contract.rs::demo_output_has_no_dangling_aria_
//! references_or_duplicate_ids` の id 非重複検証を満たす。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。カート追加ボタンは [`button::button`] の既定 `type="button"`
//! のまま送信先を持たない。パンくずのリンクは `href="#"` を避け、実在する
//! 相対パス（`../`）へ向ける。
//!
//! # 購入ボタン・サイズガイドボタンは disabled の静的表示
//!
//! 無 JS のため押しても何も起きないボタンを操作可能なまま残さない
//! （`product_overview_tabs_below`・`feature_tabs_panel` の CTA と同じ
//! 判断）。`ButtonProps { disabled: true, .. }` で固定する。
//!
//! # ダミー素材について
//!
//! 画像は [`dummy_assets`] のビルド時生成 SVG（`PRODUCT_SRC` のみを使い、
//! 外部 URL・`data:` URI は使わない）。商品名・価格・評価件数・色名・
//! サイズ・説明文・サイズ表はすべて独自に書いた架空の文言であり、実在の
//! 企業名・人物・PII を含まない。
//!
//! # ブラウザでの実機確認
//!
//! 本 worktree にはブラウザ実行環境がないため、狭幅 ⇔ 広幅の切り替え・
//! 縦サムネ列・dialog がデモ枠からはみ出さないこと・ライト/ダーク両テーマ
//! の実機確認は未実施である（`cargo test -p fandhe-frontend-docs-site` の
//! レンダリング契約テストと `LAYOUT_CSS` の静的検証のみで確認した）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
        Part {
            label: "Breadcrumb",
            path: "/themes/breadcrumb/",
        },
        Part {
            label: "Progress",
            path: "/themes/progress/",
        },
        Part {
            label: "Dialog",
            path: "/themes/dialog/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `product_overview_gallery_split` 固有のレイアウト規則（`crate::blocks`
/// モジュール doc「CSS の置き場」節と同型）。`48rem` 以上で
/// 2 カラム、未満は縦積み（モジュール doc「レイアウト（骨格）」節参照）。
/// `display: none` は使わない。
///
/// # サムネイル `carousel` の item 上書き（横・縦共通）
///
/// styled carousel の既定 `item` 規則（`[data-scope="carousel"]
/// [data-part="item"]`、詳細度 (0,2,0)）は `flex: 0 0 100%`（1 枚だけを
/// 表示するスライドショー前提）を持つため、無上書きのままだと複数枚の
/// サムネイルがそれぞれ container 全幅を占め、残りが実質到達不能になる。
/// `.blocks-product-overview-gallery-split-thumbs`（`item-group` へ付与）を
/// 祖先にした子孫セレクタで `item` を `flex: 0 0 auto` へ上書きし、
/// `root` の `overflow: hidden` を `visible` へ中和する。縦サムネ版
/// （`--vertical` 修飾クラス）は `.thumbs` の `flex-direction` を
/// `column` にするだけで、既存の `item`/`root` 上書きをそのまま使い回せる。
///
/// # 縦サムネ列の `item-group` クリッパー `overflow: hidden` の中和
///
/// `carousel.rs` の既定 `item-group[data-orientation="vertical"]` 規則
/// （詳細度 (0,3,0)、`height: 20rem` + `overflow: hidden` の静止クリッパー）
/// は `.thumbs` 自身に付けた `overflow-y: auto`（詳細度 (0,1,0)）より強く
/// カスケードで勝ち、縦サムネ列を固定高の単一枚スライドショー表示に
/// してしまう（Cursor Bugbot 指摘 是正）。`.thumbs` クラスと `item-group`
/// の属性セレクタを同一要素へ合成した詳細度 (0,4,0) の規則で
/// `overflow-y`/`overflow-x` を上書きし、縦スクロール可能なサムネ列へ戻す
/// （`height: 20rem` の固定枠自体は維持する）。
///
/// # サイズガイド dialog の中和
///
/// `cart_dialog.rs`「固定オーバーレイのデモ枠内中和」節と同型の判断だが、
/// 本 block は `backdrop` を出力しないため `positioner` を
/// `position: static` へ、`content` の `max-height` を `none` へ中和する
/// だけで `.blocks-product-overview-gallery-split-demo` の通常フロー内に
/// 収まる（`[data-blocks-product-overview-gallery-split-size-guide]` を
/// スコープ祖先にした属性セレクタで中和する）。
///
/// # カート追加ボタンの全幅化・disabled 減光の中和
///
/// `add_to_cart_button` は `data-blocks-product-overview-gallery-split-add`
/// を公開し、styled `button` の `root`（`display: inline-flex`）へ
/// `width: 100%` を明示する。色・サイズ選択 radio card の
/// `disabled_declarations()`（`opacity: 0.5`）・アコーディオンの
/// `item-trigger` disabled 規則も同様に中和する（`pricing_single_split.rs`
/// と同じ判断）。
const LAYOUT_CSS: &str = "\
.blocks-product-overview-gallery-split-demo {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-product-overview-gallery-split-caption {\n  margin: 0;\n  font-size: var(--fandhe-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-product-overview-gallery-split-layout {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-product-overview-gallery-split-gallery {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n}\n\
.blocks-product-overview-gallery-split-gallery--vertical {\n  display: grid;\n  grid-template-columns: 4rem minmax(0, 1fr);\n  gap: var(--fandhe-space-3);\n  align-items: start;\n}\n\
.blocks-product-overview-gallery-split-gallery--vertical [data-blocks-product-overview-gallery-split-main-image] {\n  grid-column: 2;\n  grid-row: 1;\n}\n\
.blocks-product-overview-gallery-split-gallery--vertical [data-scope=\"carousel\"][data-part=\"root\"] {\n  grid-column: 1;\n  grid-row: 1;\n}\n\
.blocks-product-overview-gallery-split-gallery--vertical .blocks-product-overview-gallery-split-thumbs {\n  flex-direction: column;\n}\n\
.blocks-product-overview-gallery-split-thumbs {\n  display: flex;\n  flex-wrap: nowrap;\n  overflow-x: auto;\n  overflow-y: auto;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-product-overview-gallery-split-thumbs [data-scope=\"carousel\"][data-part=\"item\"] {\n  flex: 0 0 auto;\n  overflow: visible;\n}\n\
.blocks-product-overview-gallery-split-thumbs[data-scope=\"carousel\"][data-part=\"item-group\"][data-orientation=\"vertical\"] {\n  overflow-y: auto;\n  overflow-x: hidden;\n}\n\
.blocks-product-overview-gallery-split-gallery [data-scope=\"carousel\"][data-part=\"root\"] {\n  overflow: visible;\n}\n\
img[data-scope=\"image\"][data-blocks-product-overview-gallery-split-thumb] {\n  width: 4rem;\n  height: 4rem;\n  flex-shrink: 0;\n}\n\
img[data-scope=\"image\"][data-blocks-product-overview-gallery-split-main-image] {\n  width: 100%;\n}\n\
.blocks-product-overview-gallery-split-counter {\n  margin: 0;\n  font-size: var(--fandhe-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
.blocks-product-overview-gallery-split-panel {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  min-width: 0;\n}\n\
.blocks-product-overview-gallery-split-price {\n  margin: 0;\n  font-size: var(--fandhe-font-size-xl);\n  font-weight: var(--fandhe-font-weight-bold);\n}\n\
.blocks-product-overview-gallery-split-stock {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
[data-blocks-product-overview-gallery-split-stock-bar] {\n  width: 100%;\n}\n\
.blocks-product-overview-gallery-split-share {\n  display: flex;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-product-overview-gallery-split-option {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-product-overview-gallery-split-option-heading {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-product-overview-gallery-split-option [data-scope=\"radio-card\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-blocks-product-overview-gallery-split-details] [data-scope=\"accordion\"][data-part=\"item-trigger\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
[data-blocks-product-overview-gallery-split-add] {\n  width: 100%;\n}\n\
.blocks-product-overview-gallery-split-detail-heading {\n  margin: 0;\n  font-size: inherit;\n  font-weight: inherit;\n}\n\
.blocks-product-overview-gallery-split-tablist {\n  display: flex;\n  flex-wrap: nowrap;\n  overflow-x: auto;\n  overflow-y: hidden;\n  gap: var(--fandhe-space-2);\n  border-bottom: 1px solid var(--fandhe-color-border);\n  padding-bottom: 1px;\n}\n\
.blocks-product-overview-gallery-split-tab {\n  display: inline-flex;\n  align-items: center;\n  flex-shrink: 0;\n  padding: var(--fandhe-space-2) var(--fandhe-space-4);\n  font-size: var(--fandhe-font-size-sm);\n  font-weight: var(--fandhe-font-weight-medium);\n  white-space: nowrap;\n  color: var(--fandhe-color-fg-muted);\n  border-bottom: 2px solid transparent;\n  margin-bottom: -2px;\n  cursor: default;\n}\n\
.blocks-product-overview-gallery-split-tab[data-state=\"active\"] {\n  color: var(--fandhe-color-fg);\n  border-bottom-color: var(--fandhe-color-accent);\n}\n\
.blocks-product-overview-gallery-split-tab-selected {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  padding-top: var(--fandhe-space-4);\n}\n\
.blocks-product-overview-gallery-split-preview {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  padding-top: var(--fandhe-space-4);\n  border-top: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-product-overview-gallery-split-paragraphs {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-product-overview-gallery-split-size-guide] [data-scope=\"dialog\"][data-part=\"positioner\"] {\n  position: static;\n  inset: auto;\n  z-index: auto;\n  display: block;\n  padding: 0;\n}\n\
[data-blocks-product-overview-gallery-split-size-guide] [data-scope=\"dialog\"][data-part=\"content\"] {\n  max-height: none;\n}\n\
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
            "data-scope=\"progress\"",
            "data-scope=\"dialog\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert!(!html.contains("data-scope=\"tabs\""));
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

    #[test]
    fn main_image_selector_stretches_to_column_width() {
        assert!(LAYOUT_CSS.contains(
            "img[data-scope=\"image\"][data-blocks-product-overview-gallery-split-main-image] {\n  width: 100%;\n}"
        ));
    }

    #[test]
    fn thumbs_carousel_item_and_root_are_overridden() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-product-overview-gallery-split-thumbs [data-scope=\"carousel\"][data-part=\"item\"] {\n  flex: 0 0 auto;"
        ));
        assert!(LAYOUT_CSS.contains(
            ".blocks-product-overview-gallery-split-gallery [data-scope=\"carousel\"][data-part=\"root\"] {\n  overflow: visible;"
        ));
    }

    /// Cursor Bugbot 指摘 是正の回帰: 縦サムネ列の `item-group` クリッパー
    /// （`carousel.rs` 既定の `height: 20rem` + `overflow: hidden`、詳細度
    /// (0,3,0)）に勝つ詳細度 (0,4,0) の上書きが存在し、縦スクロールを
    /// 復元することを固定する。
    #[test]
    fn vertical_thumbs_item_group_overflow_is_overridden() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-product-overview-gallery-split-thumbs[data-scope=\"carousel\"][data-part=\"item-group\"][data-orientation=\"vertical\"] {\n  overflow-y: auto;\n  overflow-x: hidden;\n}"
        ));
    }

    /// 横並び（版 A）・縦並び（版 B）の 2 carousel で `data-inview` が
    /// それぞれ 1 件ずつ、計 2 件。`data-current` はパンくずの
    /// `current_link`（3 版 × 1 件 = 3 件）とサムネイル分（2 件）を合わせた
    /// 計 5 件。
    #[test]
    fn only_first_thumbnail_is_marked_current() {
        let html = demo_html();
        assert_eq!(html.matches("data-inview").count(), 2, "html={html}");
        assert_eq!(html.matches("data-current").count(), 5, "html={html}");
    }

    /// 横・縦サムネイル（4 枚 × 2 = 8 枚）は同一のプレースホルダー画像を
    /// 使い回しており、異なる商品写真ではない。
    #[test]
    fn thumbnail_alt_text_does_not_imply_distinct_photos() {
        let html = demo_html();
        assert_eq!(
            html.matches("ワイヤレスヘッドホン 商品画像（プレースホルダー）")
                .count(),
            8,
            "html={html}"
        );
    }

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
    fn add_to_cart_button_is_full_width() {
        assert!(LAYOUT_CSS
            .contains("[data-blocks-product-overview-gallery-split-add] {\n  width: 100%;\n}"));
    }

    /// 版 A のアコーディオン 3 項目すべてが disabled のトリガー（ネイティブ
    /// disabled 属性 + aria-disabled + data-disabled）を持つこと。
    #[test]
    fn accordion_items_are_open_and_disabled() {
        let html = demo_html();
        assert!(html.matches("aria-disabled=\"true\"").count() >= 3);
        assert!(html.matches("data-state=\"open\"").count() >= 3);
    }

    #[test]
    fn accordion_triggers_are_wrapped_in_h3() {
        let html = demo_html();
        let open = "<h3 class=\"blocks-product-overview-gallery-split-detail-heading\"><button";
        assert_eq!(html.matches(open).count(), 3);
        assert!(LAYOUT_CSS.contains(
            ".blocks-product-overview-gallery-split-detail-heading {\n  margin: 0;\n  font-size: inherit;\n  font-weight: inherit;\n}"
        ));
    }

    /// 3 版 × 2 件（色・サイズ）= 6 件の checked radio card。
    #[test]
    fn radio_cards_declare_checked_state() {
        let html = demo_html();
        assert_eq!(
            html.matches("data-part=\"item\" data-state=\"checked\"")
                .count(),
            6
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

    /// 実物の `tabs::tabs` を使わないため `role="tab"`/`hidden` パネルが
    /// 一切現れないこと（モジュール doc「実物の `tabs::tabs` を使わない」
    /// 節）。
    #[test]
    fn demo_has_no_real_tabs_or_hidden_panels() {
        let html = demo_html();
        assert!(!html.contains(r#"role="tab""#));
        assert!(!html.contains(r#"role="tabpanel""#));
        assert!(!html.contains("hidden=\"\""));
        assert!(html.contains("class=\"blocks-product-overview-gallery-split-tablist\""));
    }

    /// 在庫僅少 `progress` が `aria-label` を明示すること（モジュール doc
    /// 「在庫僅少の `progress`」節）。
    #[test]
    fn stock_progress_has_aria_label() {
        let html = demo_html();
        assert!(html.contains(r#"aria-label="在庫残量""#));
        assert!(html.contains("残り 3 点"));
    }

    /// サイズガイド dialog が非モーダル・trigger/close-trigger を持たない
    /// こと（モジュール doc「サイズガイドは静的な開状態・非モーダル」節）。
    #[test]
    fn size_guide_dialog_is_non_modal_without_triggers() {
        let html = demo_html();
        assert!(html.contains(r#"aria-modal="false""#));
        assert!(!html.contains("close-trigger"));
        assert!(!html.contains(r#"data-part="trigger""#));
    }

    /// 共有リンクが `href="#"` を使わず固定 URL を参照すること。
    #[test]
    fn share_links_use_fixed_href() {
        let html = demo_html();
        assert_eq!(html.matches(super::REPO).count(), 2);
        assert!(html.contains("共有 A"));
        assert!(html.contains("共有 B"));
    }

    /// サイズガイド dialog の固定オーバーレイ中和規則が [`LAYOUT_CSS`] に
    /// 含まれること。
    #[test]
    fn layout_css_neutralizes_fixed_overlay() {
        assert!(LAYOUT_CSS.contains(
            "[data-blocks-product-overview-gallery-split-size-guide] [data-scope=\"dialog\"][data-part=\"positioner\"] {\n  position: static;"
        ));
    }

    /// id・`name` が版接尾辞で一意化され衝突しないこと（評価ラベル・色/
    /// サイズラベルの計 3 版分の代表サンプル）。
    #[test]
    fn ids_are_unique_per_variant() {
        let html = demo_html();
        for suffix in ["a", "c"] {
            let id = format!("{}-{suffix}", super::RATING_LABEL_ID);
            assert_eq!(html.matches(&format!("id=\"{id}\"")).count(), 1);
        }
        for suffix in ["a", "b", "c"] {
            let color_id = format!("{}-{suffix}", super::COLOR_LABEL_ID);
            let size_id = format!("{}-{suffix}", super::SIZE_LABEL_ID);
            assert_eq!(html.matches(&format!("id=\"{color_id}\"")).count(), 1);
            assert_eq!(html.matches(&format!("id=\"{size_id}\"")).count(), 1);
        }
    }

    /// 枚数表示（版 C）が「1 / 4」を出力すること。
    #[test]
    fn counter_variant_shows_page_count() {
        let html = demo_html();
        assert!(html.contains("1 / 4"));
    }

    /// 版 B は評価行（`rating-group`）を一切出力しないこと（モジュール doc
    /// 「状態違いの並記」節）。
    #[test]
    fn variant_b_has_no_rating_group() {
        let html = fandhe_frontend_core::render(&super::variant_b());
        assert!(!html.contains("data-scope=\"rating-group\""));
    }
}
