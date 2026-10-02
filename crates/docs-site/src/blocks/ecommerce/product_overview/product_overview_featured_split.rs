//! `product-overview-featured-split` block（イシュー #3066。親トラッキング
//! #3024「Blocks EC」配下）。対応表 ID R1177（主参照、集約元もこの 1 件
//! のみ）を構造の参照元とする合成例。取得手段・ファイル名・内部
//! コンポーネント識別子は記載しない（`product_overview_gallery_split.rs`
//! と同じ転記制限）。`_/blocks-intake/` の参照ファイルは本イシュー着手
//! 時点で本 worktree に存在しないため、対応表 ID のみを記す。
//!
//! # 使用部品
//!
//! `breadcrumb` / `heading` / `text` / `image` / `radio-card` /
//! `rating-group` / `button` / `icon` の 8 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。新規 UI 部品は追加しない。
//!
//! # レイアウト（単一の大きな商品画像 + 特徴説明）
//!
//! `>= 48rem` で 2 カラム（左: 商品情報、右: 単一の大きな商品画像）、
//! 未満では縦積みへフォールバックする（[`LAYOUT_CSS`]）。狭い幅では
//! CSS Grid の `grid-template-areas` だけで「パンくず〜説明 → 画像 →
//! サイズ選択〜保証」の順に画像を情報の間へ挟み込む（DOM の重複も
//! `display: none` も使わない。DOM 順と読み上げ順は常に
//! `summary → media → details` で、狭い幅の視覚順と一致する）。
//!
//! # 価格を素の `<p>` で組み立てる理由
//!
//! `product_overview_gallery_split.rs`「価格を素の `<p>` で組み立てる
//! 理由」節と同じ判断。`fandhe_frontend_pre_styled_ui::text::text` は
//! 呼び出し側が渡した `class` 属性を `drop_class_attr` で無条件に除去
//! するため、独自クラスを渡してもフォントサイズ規則が適用されない。
//!
//! # サイズ選択を radio card + ネイティブ disabled にする理由
//!
//! `product_overview_gallery_split.rs`「色・サイズ選択を radio card +
//! ネイティブ disabled にする理由」節と同じ判断。無 JS の docs サイトで
//! は選択の切り替えを実配線できないため、`radio_card::root` へ
//! `aria-disabled="true"` を明示し各 item もネイティブ `disabled` で
//! 固定する。現在の選択は radio の checked 状態に依存しない
//! [`styled_text::text`] の文で明文化する。
//!
//! # 購入ボタンを disabled の静的表示にする理由
//!
//! `product_overview_gallery_split.rs`「購入ボタンは disabled の静的
//! 表示」節と同じ判断。無 JS のため押しても何も起きないボタンを操作
//! 可能なまま残さない。
//!
//! # id を block 固有の定数にする理由
//!
//! 評価ラベル（[`RATING_LABEL_ID`]）・サイズ選択見出し
//! （[`SIZE_LABEL_ID`]）の `id` は、他 block（`product_overview_*`）と
//! 同一 docs サイト上で重複しないよう、本 block 名を含む固有値にする
//! （`crates/docs-site/tests/blocks_contract.rs` が重複 id を検知する）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリ
//! ティ不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的
//! な合成例である。カート追加ボタンは [`button::button`] の既定
//! `type="button"` のまま送信先を持たない。パンくずのリンクは
//! `href="#"` を避け、実在する相対パス（`../`・`../../`）へ向ける。
//!
//! # ダミー素材について
//!
//! 画像は [`dummy_assets::PRODUCT_SRC`]（ビルド時生成 SVG）のみを使い、
//! `data:` URI・外部 URL は使わない。商品名・価格・評価件数・サイズ・
//! 説明文・在庫・保証の文言はすべて独自に書いた架空のものであり、実在
//! の企業名・人物・PII を含まない。
//!
//! # 集約元差分・スコープ外
//!
//! 集約元が R1177 の 1 件のみのため状態違い（在庫切れ版等）の併記は
//! 行わない。`48rem` のブレークポイントは CSS メディアクエリのため、
//! 同一ページ内で狭い幅のレイアウトを静的に再現することはできない
//! （`site/blocks/product-overview-featured-split.md` の「原案差分メモ」
//! 節参照）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
///
/// `icon::icon` は呼び出し側が渡した `class` 属性を `drop_class_attr` で
/// 無条件に除去するため、独自クラスでは [`LAYOUT_CSS`] の色規則が
/// 当たらない（レビュー指摘対応）。`class` の代わりに `data-*` 属性を渡す
/// （`drop_class_attr` の除去対象ではない）ことで属性セレクタから当て、
/// `[data-scope="icon"][data-part="root"][data-blocks-*]` の形は
/// `testimonial_quote_stats.rs`・`comparison_table.rs`・
/// `auth_split_accent_panel.rs` 等の既存 icon 色付け block と同じ判断。
/// `d="M4 12l5 5L20 6"` はオープンポリライン（チェックマーク）のため、
/// `icon::icon` の既定 `fill="currentColor"` のままでは塗りつぶされて
/// 三角形に見える（レビュー指摘対応）。`fill="none"` + `stroke="currentColor"`
/// へ明示的に切り替える（`feed_upvote_cards.rs::vote_icon` と同じ判断）。
fn check_icon() -> Node {
    icon::icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![("data-blocks-product-overview-featured-split-icon-stock", "")],
        vec![el(
            "path",
            vec![
                ("d", "M4 12l5 5L20 6"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 在庫行・保証行で共有する盾形アイコン（独自の単純な幾何形状）。
/// `d` は `z` で閉じた単一パスのため、[`check_icon`] と異なり既定の
/// `fill="currentColor"` のままでよい（塗りつぶし形状として意図通り）。
fn shield_icon() -> Node {
    icon::icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/product-overview-featured-split/",
    title: "product-overview-featured-split",
    category: BlockCategory::ProductOverview,
    rust_source:
        "crates/docs-site/src/blocks/ecommerce/product_overview/product_overview_featured_split.rs",
    demo_class: "blocks-product-overview-featured-split",
    parts: &[
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
            label: "Image",
            path: "/themes/image/",
        },
        Part {
            label: "Radio Card",
            path: "/themes/radio-card/",
        },
        Part {
            label: "Rating Group",
            path: "/themes/rating-group/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `product_overview_featured_split` 固有のレイアウト規則
/// （`crate::blocks` モジュール doc「CSS の置き場」節と同型）。
/// 既定（狭い幅）は `grid-template-areas` による縦積み（summary → media →
/// details）、`48rem` 以上で 2 カラム（左列に summary/details、右列に
/// media）へ切り替える。`display: none` は使わない。
///
/// # disabled 減光の中和
///
/// `product_overview_gallery_split.rs`「色・サイズ選択 radio card と
/// 詳細アコーディオンの disabled 減光の中和」節と同じ判断。ネイティブ
/// disabled はアクセシビリティ上の理由（無 JS で選択を実配線できない）
/// で付けているだけで、選択自体は「無効化された機能」ではないため、
/// styled radio-card の既定 `disabled_declarations()`（`opacity: 0.5` +
/// `cursor: not-allowed`）を祖先の子孫セレクタ（詳細度 (0,4,0)、`item`
/// disabled 規則の詳細度 (0,3,0) より高い）で中和する。
///
/// # カート追加ボタンの全幅化
///
/// styled `button` の `root` は `display: inline-flex`（コンテンツ幅）
/// のため、`data-blocks-product-overview-featured-split-add` を明示
/// セレクタにして `width: 100%` を当てる（`product_overview_gallery_split.rs`
/// と同じ判断）。
///
/// # 画像の列幅追従
///
/// `PRODUCT_SRC` は固定サイズの SVG のため、pre-styled-ui `image` 部品の
/// base 規則（`max-width: 100%; height: auto`）だけでは縮小方向の制約
/// にしかならない。`width: 100%` を明示して列幅へ追従させる
/// （`product_overview_gallery_split.rs::main_image_selector_stretches_to_column_width`
/// と同じ判断）。
const LAYOUT_CSS: &str = "\
.blocks-product-overview-featured-split-layout {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  grid-template-areas: \"summary\" \"media\" \"details\";\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-product-overview-featured-split-summary {\n  grid-area: summary;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n}\n\
.blocks-product-overview-featured-split-media {\n  grid-area: media;\n  min-width: 0;\n}\n\
.blocks-product-overview-featured-split-details {\n  grid-area: details;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  min-width: 0;\n}\n\
img[data-scope=\"image\"][data-blocks-product-overview-featured-split-image] {\n  width: 100%;\n}\n\
.blocks-product-overview-featured-split-price {\n  margin: 0;\n  font-size: var(--fandhe-font-font-size-xl);\n  font-weight: var(--fandhe-font-font-weight-semibold);\n}\n\
.blocks-product-overview-featured-split-option {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-product-overview-featured-split-option [data-scope=\"radio-card\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;\n  cursor: default;\n}\n\
.blocks-product-overview-featured-split-stock,\n.blocks-product-overview-featured-split-warranty {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"icon\"][data-part=\"root\"][data-blocks-product-overview-featured-split-icon-stock] {\n  color: var(--fandhe-color-success);\n}\n\
[data-blocks-product-overview-featured-split-add] {\n  width: 100%;\n}\n\
@media (min-width: 48rem) {\n  .blocks-product-overview-featured-split-layout {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);\n    grid-template-areas: \"summary media\" \"details media\";\n    align-items: start;\n  }\n}\n";

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
            "data-scope=\"breadcrumb\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"image\"",
            "data-scope=\"radio-card\"",
            "data-scope=\"rating-group\"",
            "data-scope=\"button\"",
            "data-scope=\"icon\"",
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
    fn add_to_cart_button_is_disabled() {
        let html = demo_html();
        assert!(html.contains("blocks-product-overview-featured-split-add"));
        assert!(html.contains("disabled"));
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
    fn exactly_one_checked_radio_card_item() {
        let html = demo_html();
        // `item`/`item-control`/`item-indicator` の 3 パーツが同じ checked
        // 状態を反映するため、`item` パーツ単位に絞って数える
        // （`product_overview_gallery_split.rs` と同じ判断）。
        assert_eq!(
            html.matches("data-part=\"item\" data-state=\"checked\"")
                .count(),
            1
        );
    }

    #[test]
    fn item_description_is_rendered() {
        let html = demo_html();
        assert_eq!(html.matches("data-part=\"item-description\"").count(), 2);
    }

    #[test]
    fn label_ids_appear_exactly_once() {
        let html = demo_html();
        assert_eq!(
            html.matches(r#"id="blocks-product-overview-featured-split-rating-label""#)
                .count(),
            1
        );
        assert_eq!(
            html.matches(r#"id="blocks-product-overview-featured-split-size-label""#)
                .count(),
            1
        );
    }

    #[test]
    fn dom_order_is_summary_then_media_then_details() {
        let html = demo_html();
        let summary_pos = html
            .find("blocks-product-overview-featured-split-summary")
            .expect("summary class should be present");
        let media_pos = html
            .find("blocks-product-overview-featured-split-media")
            .expect("media class should be present");
        let details_pos = html
            .find("blocks-product-overview-featured-split-details")
            .expect("details class should be present");
        assert!(summary_pos < media_pos, "html={html}");
        assert!(media_pos < details_pos, "html={html}");
    }

    #[test]
    fn icons_are_decorative() {
        let html = demo_html();
        // 2 個のアイコン（在庫・保証）がいずれも装飾用途として
        // aria-hidden を伴う <svg> であること（パンくず区切りの
        // aria-hidden と混同しないよう、icon の <svg> 開始タグ単位で
        // 検証する）。
        let icon_svg_open = "<svg data-scope=\"icon\" data-part=\"root\"";
        assert_eq!(html.matches(icon_svg_open).count(), 2, "html={html}");
        for (i, _) in html.match_indices(icon_svg_open) {
            let tag_end = html[i..].find('>').expect("svg open tag should close");
            assert!(
                html[i..i + tag_end].contains(r#"aria-hidden="true""#),
                "html={html}"
            );
        }
    }

    #[test]
    fn layout_css_is_safe_and_has_two_column_breakpoint() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(!LAYOUT_CSS.contains("display: none"));
        assert!(LAYOUT_CSS.contains("grid-template-areas"));
    }

    #[test]
    fn layout_css_does_not_reference_undefined_font_size_token() {
        // レビュー指摘対応（`product_overview_gallery_split.rs` 既知の
        // 誤りを踏襲しない）: 実在しない `--fandhe-font-size-*` 系の
        // 省略形トークンではなく `--fandhe-font-font-size-*` を使うこと。
        assert!(!LAYOUT_CSS.contains("--fandhe-font-size-"));
        assert!(!LAYOUT_CSS.contains("--fandhe-font-weight-"));
    }

    #[test]
    fn disabled_dimming_is_neutralized_for_size_options() {
        assert!(LAYOUT_CSS.contains(
            ".blocks-product-overview-featured-split-option [data-scope=\"radio-card\"][data-part=\"item\"][data-disabled] {\n  opacity: 1;"
        ));
    }

    #[test]
    fn add_to_cart_button_is_full_width() {
        assert!(LAYOUT_CSS
            .contains("[data-blocks-product-overview-featured-split-add] {\n  width: 100%;\n}"));
    }

    #[test]
    fn main_image_selector_stretches_to_column_width() {
        assert!(LAYOUT_CSS.contains(
            "img[data-scope=\"image\"][data-blocks-product-overview-featured-split-image] {\n  width: 100%;\n}"
        ));
    }

    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = demo_html();
        assert!(html.contains("class=\"blocks-product-overview-featured-split-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-product-overview-featured-split-layout"
        );
    }
}
