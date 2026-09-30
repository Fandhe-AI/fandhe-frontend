//! `product-overview-image-grid` block（イシュー #3071、親 #3070「商品詳細
//! （複数画像グリッド + 購入パネル）」の前半。Ecommerce / Product Overview
//! カテゴリの最初の block）。上段に商品画像グリッド（段差配置）、下段に
//! 購入パネル（商品名・価格・評価・色/サイズ選択・カート追加ボタン・
//! 説明）を持つ商品詳細画面の骨格と主要領域を実装する。主参照は対応表 ID
//! R1178（集約元 R1175: 段差配置 + 右に購入パネル / R0612: 2 列グリッド・
//! 左右反転）。残り領域（状態表示の並記・R1175/R0612 の差分並記・原稿の
//! 仕上げ）は後半 #3072 で追加する（本ファイル末尾のモジュール doc
//! 「後半 #3072 で追加する領域」節参照）。`_/blocks-intake/` の対応ファイル
//! は本イシュー着手時点で本 worktree に存在しないため、原稿・本コメントには
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
//! 持たない（後半 #3072 まで）ため `link` は使わず、`text` の可視テキスト
//! として表示する（下記「レビュー件数はリンクにしない」節参照）。
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
//! と同型の判断）。レイアウト用ラッパー（グリッド・パネル・行）は素の
//! `<div>` のため `class="blocks-product-overview-image-grid-*"` を使う。
//!
//! # 狭幅では画像 1 列積み・パネルがその下へ回る（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container`
//! （コンテナクエリ）で判定する（`cart_two_column_summary` と同型の
//! パターン）。[`LAYOUT_CSS`] のラッパー
//! `.blocks-product-overview-image-grid-stack` へ `container-type:
//! inline-size` を宣言し、コンテナ幅が `40rem` 未満のとき画像グリッドを
//! 1 カラムへ切り替え（1 枚目の段差 `grid-column`/`grid-row` の span・
//! 最後の 1 枚の全幅 `grid-column` もいずれも `auto` へ戻す）、購入パネル
//! はその下（DOM 順のまま）に積む。
//!
//! # 色/サイズ選択は `disabled` の静的表示（無 JS）
//!
//! docs サイトは JS ハイドレーションを行わないため、選択操作を実行時に
//! 反映できない。`card_form_footer` の支払方法 radio card と同じ判断で、
//! 色・サイズ選択はどちらも [`fandhe_frontend_pre_styled_ui::radio_card`]
//! を `disabled: true` + `aria-disabled="true"` で描き、初期選択（色:
//! チャコール、サイズ: M）のみを固定表示する（選択状態違いの並記は
//! #3072）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。カート追加
//! ボタンは [`fandhe_frontend_pre_styled_ui::button::button`] の既定
//! `type="button"` のまま用いる。
//!
//! # 評価のアクセシブルネーム
//!
//! [`fandhe_frontend_pre_styled_ui::rating_group::label`] の `id` を固定
//! 文字列にする（Demo は 1 ページ 1 block のため重複しない）。
//!
//! # レビュー件数はリンクにしない
//!
//! 当初はレビュー詳細ページへのリンクとして実装したが、レビュー詳細ページ
//! （Ecommerce / Reviews カテゴリ）は本イシュー時点で block 未登録のため
//! 実在せず、リンク先が単なる自己参照（`href="./"`）になり遷移として機能
//! しない指摘（PR #3468 レビュー）を受けて、実在しないリンク先を作らず
//! [`fandhe_frontend_pre_styled_ui::text`] の可視テキストとして表示する
//! よう改めた。実在の Reviews block が追加され次第、その相対パスへの
//! `link` へ差し替える（#3072）。
//!
//! # ダミー素材について
//!
//! 商品名・価格・レビュー件数・色/サイズ・説明はすべて本ファイル内の架空
//! データ（実在のブランド・商品・PII を含まない）。商品画像はビルド時生成
//! の同梱 SVG（[`dummy_assets::PRODUCT_SRC`]）を 4 枚使う（外部 URL・
//! `data:` URI は使わない）。`alt` は商品名を含む説明文にする。
//!
//! # 後半 #3072 で追加する領域（本 PR のスコープ外）
//!
//! 集約元 R1175（右に購入パネル配置）・R0612（2 列グリッド・左右反転）の
//! 並記、色/サイズの選択状態違い（在庫切れ・別配色等）、原稿の仕上げは、
//! 本 PR の骨格・主要領域が固まった後に別イシュー #3072 で追加する。
//! 放置ではなく `site/blocks/product-overview-image-grid.md` の
//! 「原案差分メモ」節に明記して追跡する。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
    let alt = format!("{PRODUCT_NAME} の画像 {}", index + 1);
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
const LAYOUT_CSS: &str = "\
.blocks-product-overview-image-grid-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n  container-type: inline-size;\n  container-name: blocks-product-overview-image-grid;\n}\n\
.blocks-product-overview-image-grid-gallery {\n  display: grid;\n  grid-template-columns: repeat(3, minmax(0, 1fr));\n  grid-auto-rows: minmax(0, 1fr);\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-product-overview-image-grid-tile=\"hero\"] {\n  grid-column: span 2;\n  grid-row: span 2;\n}\n\
[data-blocks-product-overview-image-grid-tile=\"wide\"] {\n  grid-column: 1 / -1;\n}\n\
.blocks-product-overview-image-grid-panel {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  max-width: 40rem;\n}\n\
.blocks-product-overview-image-grid-rating-row {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-product-overview-image-grid-options {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
@container blocks-product-overview-image-grid (max-width: 40rem) {\n  \
.blocks-product-overview-image-grid-gallery {\n    grid-template-columns: minmax(0, 1fr);\n  }\n  \
[data-blocks-product-overview-image-grid-tile=\"hero\"] {\n    grid-column: auto;\n    grid-row: auto;\n  }\n  \
[data-blocks-product-overview-image-grid-tile=\"wide\"] {\n    grid-column: auto;\n  }\n\
}\n";

#[cfg(test)]
mod tests {
    use super::{demo, COLOR_OPTIONS, LAYOUT_CSS, SIZE_OPTIONS};
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
        assert_eq!(html.matches("<img").count(), 4);
        // 段差配置: 1 枚目のみ hero、最後の 1 枚のみ wide（PR #3468
        // レビュー P1 是正: 全画像の配置を明示し暗黙配置での取り残しを防ぐ）。
        assert_eq!(
            html.matches("data-blocks-product-overview-image-grid-tile=\"hero\"")
                .count(),
            1
        );
        assert_eq!(
            html.matches("data-blocks-product-overview-image-grid-tile=\"wide\"")
                .count(),
            1
        );
        assert!(html.contains("レビュー 128 件"));
        assert!(
            !html.contains("data-scope=\"link\""),
            "review count should be plain text, not a link (PR #3468 review P2)"
        );
        let expected_radios = COLOR_OPTIONS.len() + SIZE_OPTIONS.len();
        assert_eq!(html.matches("type=\"radio\"").count(), expected_radios);
        // `data-checked=""`（rating item 等）は部分文字列として
        // `checked=""` を含むため、先頭スペース付きで hidden input の
        // `checked=""` 属性のみを数える。
        assert_eq!(html.matches(" checked=\"\"").count(), 2);
        assert_eq!(html.matches("<button").count(), 1);
    }

    #[test]
    fn demo_has_no_form_and_no_unsafe_html() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("data:"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("<script"));
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
}
