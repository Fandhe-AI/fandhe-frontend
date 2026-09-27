//! `card-meta-cta` block（イシュー #2901。親 #2892「Blocks アプリケーション
//! A」配下、対応表 ID R0024 を主参照とする合成例。R0025（商品版: 数量入力
//! ＋評価）・R0028（料金プラン版: 機能一覧）を集約元とする、取得手段・
//! ファイル名・内部コンポーネント識別子は記載しない契約〔`careers_card_grid`
//! モジュール doc冒頭と同型〕）。
//!
//! # 使用部品
//!
//! `card` / `badge` / `heading` / `text` / `list` / `icon` / `button` /
//! `rating-group` / `number-input` / `separator` の 10 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 骨格の共有（求人・料金プラン・商品の 3 例並記）
//!
//! 題名・分類・説明・要点リスト・下端 CTA という同じ骨格を持つ 3 つの
//! カード（求人・料金プラン・商品）を [`meta_cta_card`] 1 つで共有し、
//! それぞれの差分（要点の中身・価格行・商品版のみ持つ数量入力/評価）は
//! 呼び出し側（[`job_card`]/[`plan_card`]/[`product_card`]）で組み立てて
//! `extras` として渡す（`pricing_seats_split`/`hero_social_proof` の要点
//! ロジックを流用しつつ 1 つの共有関数へ集約する設計）。
//!
//! # readonly 静的表示（無 JS）
//!
//! docs サイトは JS ハイドレーションを一切行わないため、商品カードの
//! 数量入力・評価はいずれも `readonly` の固定値表示に留める
//! （`pricing_seats_split`/`hero_social_proof` と同じ判断、
//! `docs/policy/intentional-non-adoption.md` §3.25 の責務境界）。
//!
//! # 装飾アイコンと a11y
//!
//! 要点リストの行頭アイコン（位置ピン・時計・カレンダー・チェック・仕様
//! タグ）は `careers_card_grid::geo_icon` と同型の自作幾何アイコンで、
//! `list::indicator` が常に `aria-hidden="true"` を固定するため
//! （`crate::list::indicator` 契約）追加の `aria-hidden` 指定は不要。
//! 意味は隣接する可視テキスト（要点の本文）が担う。
//!
//! # `id` を block 固有定数にする理由
//!
//! 商品カードの評価ラベル（`rating_group::label` の `id`）と数量入力の
//! `id`（`number_input::label`/`input` が共有）は、`crates/docs-site/tests/
//! blocks_contract.rs::demo_output_has_no_dangling_aria_references_or_
//! duplicate_ids` が「id の非重複」「`aria-labelledby` 参照先の存在」を
//! 全 block 横断で検証するため、block 固有の接頭辞付き定数
//! （[`RATING_LABEL_ID`]/[`QUANTITY_INPUT_ID`]）にする。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `card::root`/`button::button` は `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、カード識別・CTA
//! 全幅化の CSS フックは `data-*` 属性で渡す（[`LAYOUT_CSS`] 参照）。
//! `card::header`/`card::body`/`card::footer` と素の `div` には `class` が
//! そのまま効くため、レイアウト用のラッパはクラスセレクタを使う。
//!
//! # `<form>` を持たない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節に従い、本 Demo
//! はフォーム・送信処理・状態機械を持たない静的な合成例である。文言・
//! 部署名・商品名・価格はすべて架空のもの（実企業・PII・クレデンシャルを
//! 含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::number_input::{self, NumberInputFlags};
use fandhe_frontend_pre_styled_ui::rating_group::{
    self, RatingGroup, RatingGroupProps, RatingItemFlags,
};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::Size;

/// 評価ラベル（`rating_group::label` の `id`）。3 枚のカードを同一ページへ
/// 並記するため固定文字列にする（block 内では商品カード 1 枚のみが評価を
/// 持つため重複しない）。
const RATING_LABEL_ID: &str = "blocks-card-meta-cta-rating-label";
/// 数量入力の `id`（`number_input::label`/`input` が共有）。
const QUANTITY_INPUT_ID: &str = "blocks-card-meta-cta-quantity";

/// 装飾用の自作幾何アイコン（lucide 等の既存アイコンセットの path を
/// 複製しないための単純図形、`careers_card_grid::geo_icon` と同型）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
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

/// 位置ピンの幾何アイコン（求人カードの勤務地行）。
fn location_icon() -> Node {
    geo_icon("M12 21s-7-6.5-7-11a7 7 0 0114 0c0 4.5-7 11-7 11z M12 12a2 2 0 100-4 2 2 0 000 4z")
}

/// 時計の幾何アイコン（求人カードの雇用形態行）。
fn clock_icon() -> Node {
    geo_icon("M12 3a9 9 0 100 18 9 9 0 000-18z M12 7v5l4 2")
}

/// カレンダーの幾何アイコン（求人カードの募集期間行）。
fn calendar_icon() -> Node {
    geo_icon("M4 5h16v15H4z M4 9h16 M8 3v4 M16 3v4")
}

/// チェックの幾何アイコン（料金プランカードの機能一覧、装飾用）。
fn check_icon() -> Node {
    geo_icon("M4 12l5 5L20 6")
}

/// 仕様タグの幾何アイコン（商品カードの仕様一覧、装飾用）。
fn spec_icon() -> Node {
    geo_icon("M4 7h16 M4 12h10 M4 17h13")
}

/// 要点リストの 1 行（アイコン + 本文テキスト。3 例で共用する）。
fn meta_item(icon_node: Node, label: &str) -> Node {
    list::item(
        vec![],
        vec![list::indicator(vec![], vec![icon_node]), text(label)],
    )
}

/// 3 例が共有するカード骨格（分類 badge + 題名 + 説明 + 要点リスト +
/// 全幅 CTA ボタン）。`pre_list_extras` は説明と要点リストの間に挿入する
/// 追加ノード（料金プラン・商品カードの価格行）。`extras` は要点リストと
/// CTA の間に挿入する追加ノード（商品カードの評価・数量入力）。
fn meta_cta_card(
    badge_label: &str,
    title: &str,
    description: &str,
    pre_list_extras: Vec<Node>,
    items: Vec<(Node, &str)>,
    extras: Vec<Node>,
    cta_label: &str,
) -> Node {
    let list_items: Vec<Node> = items
        .into_iter()
        .map(|(icon_node, label)| meta_item(icon_node, label))
        .collect();

    let mut body_children = vec![styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(description)],
    )];
    body_children.extend(pre_list_extras);
    body_children.push(separator::separator(&SeparatorProps::default(), vec![]));
    body_children.push(list::root(
        ListType::default(),
        ListVariant::Plain,
        vec![],
        list_items,
    ));
    body_children.extend(extras);

    card::root(
        CardProps::default(),
        vec![("data-blocks-card-meta-cta-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    badge::badge(&BadgeProps::default(), vec![], vec![text(badge_label)]),
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Lg,
                            weight: HeadingWeight::Semibold,
                        },
                        vec![],
                        vec![text(title)],
                    ),
                ],
            ),
            card::body(vec![("class", "blocks-card-meta-cta-body")], body_children),
            card::footer(
                vec![("class", "blocks-card-meta-cta-footer")],
                vec![button(
                    &ButtonProps::default(),
                    vec![("data-blocks-card-meta-cta-cta", "")],
                    vec![text(cta_label)],
                )],
            ),
        ],
    )
}

/// 求人カード（代表構成、対応表 ID R0024）。
fn job_card() -> Node {
    meta_cta_card(
        "エンジニアリング",
        "フロントエンドエンジニア",
        "描画コアと UI コンポーネント層の設計・実装を担当します。",
        vec![],
        vec![
            (location_icon(), "勤務地：リモート"),
            (clock_icon(), "雇用形態：正社員"),
            (calendar_icon(), "募集期間：通年"),
        ],
        vec![],
        "応募する",
    )
}

/// 料金プランカード（集約元 R0028: 機能一覧）。プラン名・価格は
/// `crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS` の 2 番目
/// （"Growth"/"$29"）を使う。
fn plan_card() -> Node {
    let price_row = styled_text::text(
        &TextProps {
            size: TextSize::Xl2,
            weight: TextWeight::Bold,
            ..TextProps::default()
        },
        vec![],
        vec![
            text(crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS[1].1),
            text(" / 月"),
        ],
    );
    // 価格行は要点リストの前（description の直後）に差し込むため、
    // meta_cta_card の pre_list_extras 経由で渡す（要点リストは機能一覧の
    // みに限定するため items には含めない）。
    let items = vec![
        (check_icon(), "エディタ統合"),
        (check_icon(), "無制限プロジェクト"),
        (check_icon(), "優先サポート"),
    ];
    meta_cta_card(
        "おすすめ",
        crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS[1].0,
        "チームでの本格運用に必要な機能をまとめたプランです。",
        vec![price_row],
        items,
        vec![],
        "このプランを選ぶ",
    )
}

/// 商品カード（集約元 R0025: 数量入力＋評価。readonly の静的表示、
/// モジュール doc「readonly 静的表示」節参照）。
fn product_card() -> Node {
    let price_row = styled_text::text(
        &TextProps {
            size: TextSize::Xl2,
            weight: TextWeight::Bold,
            ..TextProps::default()
        },
        vec![],
        vec![text("$48")],
    );

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
        vec![text("評価 4.0（128 件）")],
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
    let rating = rating_group::root(
        Size::Sm,
        ColorPalette::Accent,
        &rating_props,
        vec![("data-blocks-card-meta-cta-rating", "")],
        vec![rating_label, rating_control],
    );

    let quantity_flags = NumberInputFlags {
        readonly: true,
        ..NumberInputFlags::default()
    };
    let quantity_input = number_input::root(
        Size::Md,
        false,
        false,
        true,
        vec![("data-blocks-card-meta-cta-quantity", "")],
        vec![
            number_input::label(
                quantity_flags,
                Some(QUANTITY_INPUT_ID),
                vec![],
                vec![text("数量")],
            ),
            number_input::control(
                quantity_flags,
                vec![],
                vec![
                    number_input::decrement_trigger(
                        Some(QUANTITY_INPUT_ID),
                        true,
                        vec![],
                        vec![text("-")],
                    ),
                    number_input::input(
                        "quantity",
                        Some(QUANTITY_INPUT_ID),
                        Some("1"),
                        "1",
                        "10",
                        quantity_flags,
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some(QUANTITY_INPUT_ID),
                        true,
                        vec![],
                        vec![text("+")],
                    ),
                ],
            ),
        ],
    );

    meta_cta_card(
        "新着",
        "デスクマット Pro",
        "手首の負担を抑える傾斜構造の作業用デスクマットです。",
        vec![price_row],
        vec![
            (spec_icon(), "サイズ：90 × 40 cm"),
            (spec_icon(), "素材：撥水コーティング表面"),
        ],
        vec![rating, quantity_input],
        "カートに追加",
    )
}

/// `card-meta-cta` の Demo 本体（求人・料金プラン・商品の 3 例を横並びに
/// する。狭い幅では 1 列へ積む、[`LAYOUT_CSS`] 参照）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-card-meta-cta-layout")],
        vec![job_card(), plan_card(), product_card()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/card-meta-cta/",
    title: "card-meta-cta",
    category: BlockCategory::Card,
    rust_source: "crates/docs-site/src/blocks/application/card/card_meta_cta.rs",
    demo_class: "blocks-card-meta-cta",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
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
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Rating Group",
            path: "/themes/rating-group/",
        },
        Part {
            label: "Number Input",
            path: "/themes/number-input/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `card_meta_cta` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節と同型）。
///
/// ルート grid class（`-layout`）は [`Block::demo_class`]
/// （`blocks-card-meta-cta`）と意図的に別名にする（`careers_card_grid`
/// と同じ Bugbot 教訓の回避）。
const LAYOUT_CSS: &str = "\
.blocks-card-meta-cta {\n  padding: 3rem 1.5rem;\n}\n\
.blocks-card-meta-cta-layout {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-4);\n  align-items: stretch;\n}\n\
@media (min-width: 48rem) {\n  .blocks-card-meta-cta-layout {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n}\n\
[data-blocks-card-meta-cta-card] {\n  display: flex;\n  flex-direction: column;\n  height: 100%;\n}\n\
.blocks-card-meta-cta-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  flex: 1;\n}\n\
.blocks-card-meta-cta-footer {\n  margin-top: auto;\n}\n\
[data-blocks-card-meta-cta-cta] {\n  width: 100%;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, QUANTITY_INPUT_ID, RATING_LABEL_ID};
    use fandhe_frontend_core::render;

    /// Demo が [`crate::blocks::Block::parts`] と一致する 10 部品すべてを
    /// 出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"list\"",
            "data-scope=\"icon\"",
            "data-scope=\"button\"",
            "data-scope=\"rating-group\"",
            "data-scope=\"number-input\"",
            "data-scope=\"separator\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// カードが 3 枚（求人・料金プラン・商品）であること。
    #[test]
    fn demo_renders_three_cards() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-card-meta-cta-card=\"\"").count(),
            3
        );
    }

    /// `<form>`・暗黙 submit・script・data URI・`href="#"` を含まないこと
    /// （`crate::blocks` モジュール doc「`<form>` を使わない」節・
    /// `blocks_contract.rs` の横断検査を個別にも固定する）。
    #[test]
    fn demo_has_no_form_or_dangerous_markup() {
        let html = render(&demo());
        assert!(!html.contains("<form"));
        assert!(!html.contains("type=\"submit\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("href=\"#\""));
        assert!(html.contains("type=\"button\""));
    }

    /// 評価ラベル・数量入力の `id` がそれぞれ 1 回ずつだけ出現すること
    /// （宙に浮いた ARIA 参照・id 重複の回帰、モジュール doc「`id` を
    /// block 固有定数にする理由」節）。
    #[test]
    fn rating_and_quantity_ids_appear_exactly_once() {
        let html = render(&demo());
        assert_eq!(
            html.matches(&format!("id=\"{RATING_LABEL_ID}\"")).count(),
            1
        );
        assert_eq!(
            html.matches(&format!("id=\"{QUANTITY_INPUT_ID}\"")).count(),
            1
        );
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件・`<` 非混入を持つこと。
    #[test]
    fn layout_css_declares_md_breakpoint_and_three_columns() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("repeat(3"));
    }

    /// ルート grid class（`-layout`）が `demo_class` と別名であること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-card-meta-cta-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-card-meta-cta-layout");
    }
}
