//! `quickview-image-split` block（イシュー #3087。Ecommerce / Quickview
//! カテゴリの block）。商品一覧から開く「商品クイックビュー」ダイアログを、
//! 左に商品画像・右に商品名/価格/評価/色選択/サイズ選択/カート追加
//! ボタンを置く 2 カラム構成の静的開状態で示す。主参照は対応表 ID R1181
//! （集約元 R1182/R1183/R1184）。
//! `_/blocks-intake/` の対応ファイルは本イシュー着手時点で本 worktree に
//! 存在しないため、原稿・本コメントには対応表 ID のみを記し、レイアウトは
//! issue のレイアウト仕様に従って独自に組む
//! （`cart_dialog.rs`/`product_overview_gallery_split.rs` と同じ扱い）。
//!
//! # 使用部品
//!
//! `dialog` / `image` / `text` / `rating-group` / `radio-card` /
//! `color-swatch` / `button` / `link` の 8 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。商品名は [`dialog::title`]（`h2`）が
//! 兼ねるため、新たに `heading` 部品は呼ばない。新しい UI 部品は追加しない。
//!
//! # 開く・閉じる・カート追加・選択は無 JS で no-op のため `disabled`
//!
//! docs サイトは JS ハイドレーションを行わない設計（CLAUDE.md）のため、
//! 閉じるボタン・カート追加ボタン・色/サイズの各 radio は押しても状態が
//! 変わらない。押せない操作をキーボード・支援技術利用者に実行可能なもの
//! として提示しないため、`cart_dialog.rs` と同じ判断でネイティブ
//! `disabled` 属性 + `data-disabled`（色/サイズの root は
//! `aria-disabled="true"`）を明示する
//! （`product_overview_gallery_split.rs::option_group`/`option_item` と
//! 同型）。
//!
//! # 閉じるボタンを置く理由（`store_nav_centered_logo.rs` と同じ判断）
//!
//! issue のレイアウト仕様が右上の閉じるボタンを明示的に求めているため、
//! `contact_dialog_form.rs`（`close_trigger` を置かない判断）ではなく
//! `cart_dialog.rs`/`store_nav_centered_logo.rs` の判断を採る。
//!
//! # `aria-modal` を false にする理由
//!
//! 静的なデモは閉じる機構を実際には持たず、ダイアログの外側に説明・
//! コードがある。支援技術が外側を無視しないよう、表示の実態と一致させて
//! `aria-modal` は false にする（`cart_dialog.rs`/`contact_dialog_form.rs`/
//! `game_ui_modal.rs` と同じ判断）。
//!
//! # 価格を素の `<p>` で組み立てる理由
//!
//! `fandhe_frontend_pre_styled_ui::text::text` は呼び出し側が渡した
//! `class` 属性を `drop_class_attr` で無条件に除去するため、
//! `blocks-quickview-image-split-price` クラスを渡しても [`LAYOUT_CSS`]
//! のフォントサイズ規則が適用されない
//! （`product_overview_gallery_split.rs`「価格を素の `<p>` で組み立てる
//! 理由」節と同型の判断）。
//!
//! # 色選択・サイズ選択
//!
//! `radio_card` を 2 グループ置く（色は `item_content` 先頭へ
//! `color_swatch::color_swatch`〔`aria-hidden`、装飾〕+ `item_text`、
//! サイズは `item_text` のみ）。`name`・ラベル id は
//! `product_overview_gallery_split.rs::option_group`/`option_item` と同型に
//! 2 グループで分ける。初期選択は色 1 件・サイズ 1 件。現在の選択は radio
//! の checked 状態に依存しない [`styled_text::text`] の文で明文化する
//! （ネイティブ disabled な radio は支援技術のフォームモード走査から除外
//! され得るため）。
//!
//! # 評価
//!
//! `rating_group` を readonly で使い、件数・平均を可視テキストでも示す
//! （`hero_social_proof.rs`/`product_overview_gallery_split.rs::rating_row`
//! と同型）。
//!
//! # 詳細リンク
//!
//! 可視ラベルと遷移先を一致させる（`page_heading_avatar.rs` で是正された
//! 判断）。本 block が商品詳細ページの合成例を兼ねられるよう、詳細リンクは
//! 既存の `product-overview-gallery-split` block ページへ向ける
//! （`href="#"` は使わない）。サイズガイドは docs サイトに実在の遷移先が
//! ないため Demo には置かず、原稿の「原案差分メモ」節でのみ言及する。
//!
//! # 狭幅では画像が上に積まれる（`@container`）
//!
//! Demo 枠の幅はビューポート幅と一致しないため、`@container`
//! （コンテナクエリ）で判定する（`cart_dialog.rs` と同型のパターン）。
//! [`LAYOUT_CSS`] のラッパー `.blocks-quickview-image-split-stack` へ
//! `container-type: inline-size` を宣言し、コンテナ幅が `40rem` 未満の
//! とき 2 カラム grid を 1 カラムへ切り替える。`display: none` は使わない。
//!
//! # 固定オーバーレイのデモ枠内中和
//!
//! `dialog::backdrop`/`positioner` は本来 `position: fixed; inset: 0` の
//! ビューポート全体オーバーレイだが、Blocks の掲示は `.blocks-demo` 枠内へ
//! 収める必要がある。本 block スコープ（`.blocks-quickview-image-split`
//! 配下）に限定した属性セレクタで中和する（`cart_dialog.rs`/
//! `contact_dialog_form.rs` と同型）。
//!
//! # `<form>` を使わない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない静的表示のみで、送信処理・送信先は一切持たない。カート追加
//! ボタンは `button::button` の既定 `type="button"` のまま用いる。
//!
//! # ダミー素材について
//!
//! 商品名・価格・評価・色名・サイズは本ファイル内の架空データ（実在の
//! ブランド・商品・PII を含まない）で持つ。商品画像はビルド時生成の同梱
//! SVG（[`dummy_assets::PRODUCT_SRC`]）を使う（外部 URL・`data:` URI は
//! 使わない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/quickview-image-split/",
    title: "quickview-image-split",
    category: BlockCategory::Quickview,
    rust_source: "crates/docs-site/src/blocks/ecommerce/quickview/quickview_image_split.rs",
    demo_class: "blocks-quickview-image-split",
    parts: &[
        Part {
            label: "Dialog",
            path: "/themes/dialog/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
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
            label: "Link",
            path: "/themes/link/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `quickview_image_split` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型、`cart_dialog.rs` の固定
/// オーバーレイ中和パターンを踏襲）。
const LAYOUT_CSS: &str = "\
.blocks-quickview-image-split.blocks-demo {\n  overflow: visible;\n}\n\
.blocks-quickview-image-split-stack {\n  container-type: inline-size;\n  container-name: blocks-quickview-image-split;\n}\n\
[data-blocks-quickview-image-split-root] {\n  position: relative;\n}\n\
.blocks-quickview-image-split [data-scope=\"dialog\"] h2 {\n  border-top: none;\n  padding-top: 0;\n  letter-spacing: normal;\n}\n\
.blocks-quickview-image-split [data-scope=\"dialog\"][data-part=\"backdrop\"] {\n  position: absolute;\n  inset: 0;\n  z-index: auto;\n  border-radius: var(--fandhe-radius-lg);\n  background: var(--fandhe-color-bg-subtle);\n}\n\
.blocks-quickview-image-split [data-scope=\"dialog\"][data-part=\"positioner\"] {\n  position: relative;\n  inset: auto;\n  z-index: auto;\n  width: 100%;\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  padding: var(--fandhe-space-6);\n}\n\
.blocks-quickview-image-split [data-scope=\"dialog\"][data-part=\"content\"] {\n  position: relative;\n  max-width: 48rem;\n  width: 100%;\n}\n\
[data-blocks-quickview-image-split-image] {\n  width: 100%;\n  height: auto;\n}\n\
.blocks-quickview-image-split-grid {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-quickview-image-split-panel {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-quickview-image-split-price {\n  font-size: var(--fandhe-font-size-xl);\n  font-weight: var(--fandhe-font-weight-bold, 700);\n  margin: 0;\n}\n\
.blocks-quickview-image-split-option {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-quickview-image-split [data-scope=\"radio-card\"][data-part=\"item\"] {\n  opacity: 1;\n  cursor: default;\n}\n\
@container blocks-quickview-image-split (max-width: 40rem) {\n  \
.blocks-quickview-image-split-grid {\n    grid-template-columns: minmax(0, 1fr);\n  }\n  \
.blocks-quickview-image-split [data-scope=\"dialog\"][data-part=\"positioner\"] {\n    padding: var(--fandhe-space-2);\n  }\n\
}\n";

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
            "data-scope=\"dialog\" data-part=\"backdrop\"",
            "data-scope=\"dialog\" data-part=\"positioner\"",
            "data-scope=\"dialog\" data-part=\"content\"",
            "data-scope=\"dialog\" data-part=\"title\"",
            "data-scope=\"dialog\" data-part=\"close-trigger\"",
            "data-scope=\"image\"",
            "data-scope=\"text\"",
            "data-scope=\"rating-group\"",
            "data-scope=\"radio-card\" data-part=\"root\"",
            "data-scope=\"color-swatch\"",
            "data-scope=\"button\"",
            "data-scope=\"link\"",
        ] {
            assert!(html.contains(scope), "demo should contain {scope}");
        }
        assert_eq!(
            html.matches("data-scope=\"radio-card\" data-part=\"root\"")
                .count(),
            2
        );
        assert_eq!(html.matches("data-scope=\"color-swatch\"").count(), 3);
        assert_eq!(html.matches("data-part=\"item-hidden-input\"").count(), 7);
    }

    #[test]
    fn no_form_submit_or_dead_links() {
        let html = demo_html();
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("href=\"#\""));
        assert!(html.contains("../../assets/blocks-demo-product.svg"));
    }

    #[test]
    fn buttons_are_type_button() {
        let html = demo_html();
        let button_count = html.matches("<button").count();
        let type_button_count = html.matches("type=\"button\"").count();
        assert_eq!(button_count, type_button_count);
        // close-trigger + カート追加 = 2。
        assert_eq!(button_count, 2);
    }

    #[test]
    fn noop_controls_are_disabled() {
        let html = demo_html();
        // close-trigger + カート追加 + 色 radio 3 + サイズ radio 4 = 9。
        assert_eq!(html.matches(" disabled=\"\"").count(), 9);
        assert_eq!(
            html.matches("aria-disabled=\"true\"").count(),
            3,
            "色・サイズ両 radio-card root + カート追加ボタンに aria-disabled が必要"
        );
        assert!(html.contains("aria-label=\"閉じる\""));
    }

    #[test]
    fn dialog_is_open_static_and_non_modal() {
        let html = demo_html();
        assert!(html.contains("aria-modal=\"false\""));
        assert!(html.contains("data-state=\"open\""));
        assert!(!html.contains(" hidden>"));
        assert!(!html.contains(" hidden "));
    }

    #[test]
    fn selection_is_stated_in_text() {
        let html = demo_html();
        assert!(html.contains("現在の選択: グレー"));
        assert!(html.contains("現在の選択: M"));
    }

    #[test]
    fn labels_and_ids_are_unique_and_prefixed() {
        let html = demo_html();
        let mut seen = std::collections::HashSet::new();
        for part in html.split("id=\"").skip(1) {
            let id = part.split('"').next().unwrap();
            assert!(
                id.starts_with("blocks-quickview-image-split-"),
                "unexpected id prefix: {id}"
            );
            assert!(seen.insert(id.to_string()), "duplicate id: {id}");
        }
    }

    #[test]
    fn detail_link_points_at_existing_block_page() {
        let html = demo_html();
        assert!(html.contains("href=\"../product-overview-gallery-split/\""));
    }

    #[test]
    fn layout_css_neutralizes_overlay_and_declares_container_query() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("position: relative;\n  inset: auto;"));
        assert!(LAYOUT_CSS.contains("container-type: inline-size"));
        assert!(LAYOUT_CSS.contains("@container blocks-quickview-image-split"));
    }
}
