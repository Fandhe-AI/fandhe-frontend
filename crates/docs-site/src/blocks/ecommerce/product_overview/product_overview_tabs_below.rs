//! `product-overview-tabs-below` block（イシュー #3074。親 #3073「Blocks
//! ecommerce/Product Overview カテゴリ新設」配下、対応表 ID R1179 を主参照
//! とする合成例。取得手段・ファイル名・内部コンポーネント識別子は記載
//! しない契約〔`faq_tabbed_accordion` モジュール doc冒頭と同型〕）。
//!
//! 全体の規模が大きいため前半（本イシュー #3074）と後半（#3075）に分割
//! されている。**本 PR の範囲は骨格（上段 2 カラム + 下段全幅タブ）と
//! 主要領域のみ**であり、「よくある質問」「利用条件」タブのパネル併記
//! （非選択タブのプレビュー）・状態違いの並記・集約元差分の扱い・原稿の
//! 「原案差分メモ」節の仕上げは後半 #3075 が担う。
//!
//! # 使用部品
//!
//! `image` / `heading` / `text` / `rating-group` / `button` / `list` /
//! `link` / `avatar` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。
//!
//! # 実物の `tabs::tabs` を使わない（`faq_tabbed_accordion`/
//! `feature_tabs_panel` と同じ判断）
//!
//! docs サイトは無 JS のため、実物の `tabs::tabs` を置くと「操作できる
//! ように見えて切り替わらない」トリガーと `hidden` パネルが残る
//! （既存 block が Codex/Bugbot 指摘を受けて是正済みの経路）。本 block も
//! [`static_tab_list`] で `role`/`tabindex`/`<button>` を持たない非対話の
//! `div` 列を置き、選択中の「レビュー」パネルはタブ列とは別に可視見出し
//! （`H3`）を持つ節として直下に描画する。[`BLOCK::parts`] に `Tabs` は
//! 含めない（実際に呼ばない部品を掲げない、`faq_tabbed_accordion`/
//! `feature_tabs_panel` と同じ判断）。「よくある質問」「利用条件」ラベルは
//! タブ列には非対話表示で出すが、対応するパネルの併記は後半 #3075 の
//! スコープ（モジュール doc冒頭節参照）。
//!
//! # 購入ボタンは disabled の静的表示
//!
//! 無 JS のため押しても何も起きないボタンを操作可能なまま残さない
//! （`feature_tabs_panel` の CTA と同じ判断）。`ButtonProps { disabled:
//! true, .. }` で固定する。
//!
//! # 共有リンクは架空の固定 href
//!
//! 実在 SNS サービス名を持ち込まず「共有 A」「共有 B」の架空ラベルにし、
//! href は本リポジトリの固定 URL（`REPO` 定数、他 block と同型）にする。
//!
//! # id を block 固有定数にする理由
//!
//! 商品の評価ラベル・レビュー各件の評価ラベルはいずれも
//! `rating_group::label` の `id` を持つため、`crates/docs-site/tests/
//! blocks_contract.rs::demo_output_has_no_dangling_aria_references_or_
//! duplicate_ids` の「id の非重複」検証のため block 固有の接頭辞付き定数
//! （[`RATING_LABEL_ID`]）とレビュー件別の動的 id（[`review_rating_label_id`]）
//! にする。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading`/`text`/`image`/`button`/`avatar`/`link::root`/`list::root`/
//! `rating_group::root` はいずれも `drop_class_attr` により呼び出し側
//! `class` を黙って除去する契約を持つため、これらへの CSS フックは
//! `data-*` 属性で渡し、素の `div` には `class` をそのまま使う
//! （`crate::blocks` モジュール doc「CSS フックが `class` と `[data-*]` で
//! 混在する理由」節参照）。
//!
//! # `<form>` を持たない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節に従い、本 Demo
//! はフォーム・送信処理・状態機械を持たない静的な合成例である。商品名・
//! レビュー本文・人名はすべて架空のもの（実企業・PII・クレデンシャルを
//! 含まない）。

use crate::blocks::dummy_assets;
use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el_owned, h3, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{image, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 商品の評価ラベル（`rating_group::label` の `id`）。
const RATING_LABEL_ID: &str = "blocks-product-overview-tabs-below-rating-label";

/// 本リポジトリの固定 URL（共有リンクの href、他 block と同型の判断）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// レビュー 1 件分のダミーデータ（人名は [`dummy_assets::PERSON_NAMES`] の
/// 索引、評価は 5 段階、本文は架空のもの）。
const REVIEWS: [(usize, u32, &str); 3] = [
    (
        0,
        5,
        "手首の負担が減り、長時間の作業がかなり楽になりました。",
    ),
    (1, 4, "質感は良いですが、もう少し大きいサイズも欲しいです。"),
    (
        2,
        5,
        "梱包も丁寧で、届いてすぐに使い始められました。おすすめです。",
    ),
];

/// レビュー `index` 件目の評価ラベル id（全件を通じて一意にする、
/// モジュール doc「id を block 固有定数にする理由」節）。
fn review_rating_label_id(index: usize) -> String {
    format!("blocks-product-overview-tabs-below-review-{index}-rating-label")
}

/// カテゴリラベルのみを装飾として示す非対話タブ列（モジュール doc「実物の
/// `tabs::tabs` を使わない」節）。`role`/`tabindex`/`<button>` を一切持たず、
/// `selected` に一致するラベルだけ `data-state="active"` にする。各ラベルは
/// `aria-hidden` で支援技術のツリーから除外する。
fn static_tab_list(selected: &str) -> Node {
    let labels = [
        ("reviews", "レビュー"),
        ("faq", "よくある質問"),
        ("terms", "利用条件"),
    ];
    el_owned(
        "div",
        vec![(
            "class".to_string(),
            "blocks-product-overview-tabs-below-tablist".to_string(),
        )],
        labels
            .iter()
            .map(|(value, label)| {
                let state = if *value == selected {
                    "active"
                } else {
                    "inactive"
                };
                el_owned(
                    "div",
                    vec![
                        (
                            "class".to_string(),
                            "blocks-product-overview-tabs-below-tab".to_string(),
                        ),
                        ("data-state".to_string(), state.to_string()),
                        ("aria-hidden".to_string(), "true".to_string()),
                    ],
                    vec![text(*label)],
                )
            })
            .collect(),
    )
}

/// 商品画像（左列）。
fn product_image() -> Node {
    image(
        &ImageProps {
            aspect_ratio: AspectRatio::Square,
            ..ImageProps::new(dummy_assets::PRODUCT_SRC, "デスクマット Pro の商品画像")
        },
        vec![("data-blocks-product-overview-tabs-below-image", "")],
    )
}

/// 商品の評価（readonly の静的表示。無 JS のため操作不能、
/// `card_meta_cta::product_card` と同じ判断）。
fn product_rating() -> Node {
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
        vec![text("評価 4.0（86 件）")],
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
    rating_group::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-product-overview-tabs-below-rating", "")],
        vec![label, control],
    )
}

/// 特徴リストの 1 行。
fn feature_item(label: &str) -> Node {
    list::item(vec![], vec![text(label)])
}

/// 共有リンク 1 件（架空ラベル、固定 href、モジュール doc「共有リンクは
/// 架空の固定 href」節）。
fn share_link(label: &str) -> Node {
    link::root(REPO, &LinkProps::default(), vec![], vec![text(label)])
}

/// 上段・情報列（商品名 + 評価 + 説明 + 購入ボタン + 特徴リスト + 共有
/// リンク）。
fn info_column() -> Node {
    div(
        vec![("class", "blocks-product-overview-tabs-below-info")],
        vec![
            heading(
                HeadingLevel::H2,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Semibold,
                },
                vec![],
                vec![text("デスクマット Pro")],
            ),
            product_rating(),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "手首の負担を抑える傾斜構造の作業用デスクマットです。撥水コーティング表面で日常使いにも適しています。",
                )],
            ),
            button(
                &ButtonProps {
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![("data-blocks-product-overview-tabs-below-cta", "")],
                vec![text("カートに追加")],
            ),
            list::root(
                ListType::default(),
                ListVariant::Plain,
                vec![("data-blocks-product-overview-tabs-below-features", "")],
                vec![
                    feature_item("サイズ：90 × 40 cm"),
                    feature_item("素材：撥水コーティング表面"),
                    feature_item("お届け目安：3〜5 営業日"),
                ],
            ),
            div(
                vec![("class", "blocks-product-overview-tabs-below-share")],
                vec![share_link("共有 A"), share_link("共有 B")],
            ),
        ],
    )
}

/// 上段（商品画像 + 情報列、狭幅では縦積み、[`LAYOUT_CSS`] 参照）。
fn top_section() -> Node {
    div(
        vec![("class", "blocks-product-overview-tabs-below-top")],
        vec![product_image(), info_column()],
    )
}

/// レビュー 1 件分（アバター + 氏名 + 評価 + 本文）。
fn review_item((index, rating, body): (usize, u32, &str)) -> Node {
    let name = dummy_assets::PERSON_NAMES[index % dummy_assets::PERSON_NAMES.len()];
    let initials: String = name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .collect();
    let label_id = review_rating_label_id(index);

    let props = RatingGroupProps {
        disabled: false,
        readonly: true,
        required: false,
    };
    let state = RatingGroup::new(5, Some(rating), true);
    let label = rating_group::label(
        &props,
        Some(label_id.as_str()),
        vec![],
        vec![text(format!("評価 {rating}.0"))],
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
    let control = rating_group::control(&props, Some(label_id.as_str()), vec![], items);
    let rating_node = rating_group::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![],
        vec![label, control],
    );

    div(
        vec![("class", "blocks-product-overview-tabs-below-review")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Sm,
                    ..AvatarProps::default()
                },
                vec![],
                vec![avatar::fallback(
                    ImageStatus::Error,
                    vec![],
                    vec![text(initials)],
                )],
            ),
            div(
                vec![("class", "blocks-product-overview-tabs-below-review-body")],
                vec![
                    div(
                        vec![("class", "blocks-product-overview-tabs-below-review-name")],
                        vec![text(name)],
                    ),
                    rating_node,
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(body)],
                    ),
                ],
            ),
        ],
    )
}

/// 下段・全幅タブ（静的タブ列 + レビュー一覧パネル、モジュール doc冒頭節
/// 参照。「よくある質問」「利用条件」パネルの併記は後半 #3075 で扱う）。
fn tabs_section() -> Node {
    div(
        vec![("class", "blocks-product-overview-tabs-below-tabs")],
        vec![
            static_tab_list("reviews"),
            div(
                vec![("class", "blocks-product-overview-tabs-below-reviews")],
                vec![
                    h3(vec![], vec![text("レビュー")]),
                    div(
                        vec![("class", "blocks-product-overview-tabs-below-review-list")],
                        REVIEWS.iter().copied().map(review_item).collect(),
                    ),
                ],
            ),
        ],
    )
}

/// `product-overview-tabs-below` の Demo 本体（上段 2 カラム + 下段全幅
/// タブ、モジュール doc冒頭節参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-product-overview-tabs-below-layout")],
        vec![top_section(), tabs_section()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/product-overview-tabs-below/",
    title: "product-overview-tabs-below",
    category: BlockCategory::ProductOverview,
    rust_source:
        "crates/docs-site/src/blocks/ecommerce/product_overview/product_overview_tabs_below.rs",
    demo_class: "blocks-product-overview-tabs-below",
    parts: &[
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
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `product_overview_tabs_below` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節と同型）。
///
/// ルート class（`-layout`）は [`Block::demo_class`] と意図的に別名にする
/// （`faq_tabbed_accordion`/`card_meta_cta` と同じ Bugbot 教訓の回避）。
/// 上段は既定 1 カラム、`min-width: 48rem` で 2 カラムへ切り替える
/// （ブレークポイントトークンは `@media` 内で解決できないため rem 直書き、
/// `contact_image_info` と同じ判断）。タブ列は下線型で `overflow-x: auto`
/// による横スクロールのみを許す。
const LAYOUT_CSS: &str = "\
.blocks-product-overview-tabs-below-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-product-overview-tabs-below-top {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-6);\n}\n\
@media (min-width: 48rem) {\n  .blocks-product-overview-tabs-below-top {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);\n  }\n}\n\
[data-blocks-product-overview-tabs-below-image] {\n  width: 100%;\n}\n\
.blocks-product-overview-tabs-below-info {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  align-items: flex-start;\n}\n\
[data-blocks-product-overview-tabs-below-cta] {\n  margin-top: var(--fandhe-space-2);\n}\n\
.blocks-product-overview-tabs-below-share {\n  display: flex;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-product-overview-tabs-below-tabs {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-product-overview-tabs-below-tablist {\n  display: flex;\n  flex-wrap: nowrap;\n  overflow-x: auto;\n  overflow-y: hidden;\n  gap: var(--fandhe-space-2);\n  border-bottom: 1px solid var(--fandhe-color-border);\n  padding-bottom: 1px;\n}\n\
.blocks-product-overview-tabs-below-tab {\n  display: inline-flex;\n  align-items: center;\n  flex-shrink: 0;\n  padding: var(--fandhe-space-2) var(--fandhe-space-4);\n  font-size: var(--fandhe-font-font-size-sm);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  white-space: nowrap;\n  color: var(--fandhe-color-fg-muted);\n  border-bottom: 2px solid transparent;\n  margin-bottom: -2px;\n  cursor: default;\n}\n\
.blocks-product-overview-tabs-below-tab[data-state=\"active\"] {\n  color: var(--fandhe-color-fg);\n  border-bottom-color: var(--fandhe-color-accent);\n}\n\
.blocks-product-overview-tabs-below-review-list {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-product-overview-tabs-below-review {\n  display: flex;\n  gap: var(--fandhe-space-3);\n  align-items: flex-start;\n}\n\
.blocks-product-overview-tabs-below-review-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
.blocks-product-overview-tabs-below-review-name {\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, RATING_LABEL_ID};
    use fandhe_frontend_core::render;

    /// Demo が [`crate::blocks::Block::parts`] と一致する 8 部品すべてを
    /// 出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"image\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"rating-group\"",
            "data-scope=\"button\"",
            "data-scope=\"list\"",
            "data-scope=\"link\"",
            "data-scope=\"avatar\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(!html.contains("<form"));
    }

    /// 実物の `tabs::tabs` を使わないため `role="tab"`/`hidden` パネルが
    /// 一切現れないこと（モジュール doc「実物の `tabs::tabs` を使わない」
    /// 節）。
    #[test]
    fn demo_has_no_real_tabs_or_hidden_panels() {
        let html = render(&demo());
        assert!(!html.contains(r#"role="tab""#));
        assert!(!html.contains(r#"role="tabpanel""#));
        assert!(!html.contains("hidden=\"\""));
        assert!(html.contains("class=\"blocks-product-overview-tabs-below-tablist\""));
    }

    /// 静的タブ列は「レビュー」のみ `data-state="active"` であること。
    #[test]
    fn static_tab_list_marks_only_reviews_as_active() {
        let html = render(&demo());
        assert_eq!(html.matches("data-state=\"active\"").count(), 1);
        assert!(html.contains("レビュー"));
        assert!(html.contains("よくある質問"));
        assert!(html.contains("利用条件"));
    }

    /// 選択中「レビュー」パネルの見出しが `aria-hidden` なタブ列の外に
    /// 可視・支援技術から到達可能な形で存在すること。
    #[test]
    fn reviews_heading_is_visible_outside_aria_hidden_tab_list() {
        let html = render(&demo());
        assert!(html.contains("<h3"));
        assert_eq!(html.matches("<h3").count(), 1);
    }

    /// 購入ボタンが `type="button"` かつ `disabled` の静的表示であること
    /// （モジュール doc「購入ボタンは disabled の静的表示」節）。
    #[test]
    fn purchase_button_is_type_button_and_disabled() {
        let html = render(&demo());
        assert!(html.contains(r#"type="button""#));
        assert!(html.contains("disabled"));
        assert!(html.contains("カートに追加"));
    }

    /// [`LAYOUT_CSS`] が上段 2 カラムのブレークポイントとタブ列の
    /// 横スクロールを宣言すること。
    #[test]
    fn layout_css_declares_two_column_top_and_scrollable_tablist() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains(".blocks-product-overview-tabs-below-tablist {"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-product-overview-tabs-below-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-product-overview-tabs-below-layout"
        );
    }

    /// 評価ラベルの id が重複しないこと（商品 1 件 + レビュー 3 件 = 4 件、
    /// モジュール doc「id を block 固有定数にする理由」節）。
    #[test]
    fn rating_label_ids_are_unique() {
        let html = render(&demo());
        assert_eq!(
            html.matches(&format!("id=\"{RATING_LABEL_ID}\"")).count(),
            1
        );
        for index in 0..3 {
            let id = super::review_rating_label_id(index);
            assert_eq!(html.matches(&format!("id=\"{id}\"")).count(), 1);
        }
    }
}
