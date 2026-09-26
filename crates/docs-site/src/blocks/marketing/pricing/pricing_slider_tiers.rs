//! `pricing-slider-tiers` block（イシュー #2868「Marketing / Pricing の
//! `pricing-slider-tiers`（利用量スライダー＋プランカード 3 枚）」配下、
//! 前半 #2869「骨格と主要領域」担当。参照は対応表 ID R0206 のみ）。
//!
//! # 使用部品
//!
//! `heading`（見出し）+ `text`（リード文・選択中の利用量の併記）+
//! `button`（CTA 2 個）+ `slider`（利用量、`pricing_usage_slider` と同型）+
//! `card`（プランカード 3 枚）+ `badge`（推奨プランのタグ）を合成する
//! （[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # 前半 #2869 と後半 #2870 の分担
//!
//! 本 issue（前半）は骨格（header・利用量 slider・プランカード 3 枚・
//! 推奨カードの強調・レスポンシブ）までを実装する。プランカードの機能一覧
//! （`list`+`icon`）と、slider の別位置に対応する価格の状態違い併記は
//! 後半 #2870 が追加する。このため `parts` には現時点で実際に使う 6 部品
//! のみを登録し、`list`/`icon` はまだ含めない（「使用部品」節と実際の
//! 出力を一致させるため）。
//!
//! # 静的な組であることの明示（実行時計算・ライブ連動を持たない）
//!
//! [`pricing_usage_slider`](super::pricing_usage_slider) と同じ判断で、
//! docs サイトは JS ハイドレーションを一切行わないため slider は固定値で
//! 描画され、対応する価格はその値に対応する固定表示にする。「slider の値を
//! 実行時にカード価格へ反映する」ライブ連動そのものは実装しない
//! （`docs/policy/intentional-non-adoption.md` §3.25 の責務境界）。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。CTA ボタンは既定 `type="button"` のまま送信先を持たない。
//! プラン名・価格・文言はすべて架空のものであり、実企業名・実クレデン
//! シャル・PII を含まない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `card::root`/`slider::root`/`button::button`/`badge::badge`/`heading::heading`
//! は `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有スタイルは `data-blocks-pricing-slider-tiers-*`
//! 属性で渡し、[`LAYOUT_CSS`] 側も同じ属性セレクタで対応する（`crate::blocks`
//! モジュール doc「CSS フックが `class` と `[data-*]` で混在する理由」節
//! 参照）。一方 `card::header`/`body`/`footer`（variant を持たず `attrs` を
//! そのまま連結する）と素の `div`/`span` には `class` がそのまま効くため、
//! それらは従来どおりクラスセレクタを使う。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::slider::Slider;
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::Orientation;
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::slider::{self, SliderProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 料金プラン 1 件分の静的データ（架空のプラン名・価格・説明）。価格は
/// slider の固定初期値（50 千件/月）に対応する値として選んでいる。
struct Plan {
    name: &'static str,
    description: &'static str,
    price: &'static str,
    cta: &'static str,
    recommended: bool,
}

/// 3 段のプラン定義。中央（index 1）を `recommended: true` とする。
const PLANS: &[Plan] = &[
    Plan {
        name: "Starter",
        description: "個人・小規模プロジェクト向け",
        price: "$29",
        cta: "Get started",
        recommended: false,
    },
    Plan {
        name: "Growth",
        description: "成長中のチーム向け",
        price: "$79",
        cta: "Get started",
        recommended: true,
    },
    Plan {
        name: "Enterprise",
        description: "大規模組織向け",
        price: "$249",
        cta: "Contact sales",
        recommended: false,
    },
];

/// header 領域（見出し・リード文・CTA 2 個）を組み立てる。
fn header() -> Node {
    div(
        vec![("data-blocks-pricing-slider-tiers-header", "")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("使った分だけ、必要なプランで")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "利用量スライダーで想定コストを確認し、ぴったりのプランを選べます。",
                )],
            ),
            div(
                vec![("data-blocks-pricing-slider-tiers-cta-row", "")],
                vec![
                    button::button(&ButtonProps::default(), vec![], vec![text("無料で始める")]),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("資料をダウンロード")],
                    ),
                ],
            ),
        ],
    )
}

/// 利用量 slider 領域（`pricing_usage_slider` と同型の構成。初期値は 4 段
/// プリセット目盛りの中間（50）に固定する）。
fn usage_slider() -> Node {
    let props = SliderProps::default();
    let selected_units = 50.0_f64;
    let state = Slider::new(0.0, 500.0, 10.0, selected_units, Orientation::Horizontal);

    let slider_node = slider::root(
        Size::Md,
        fandhe_frontend_pre_styled_ui::ColorPalette::Accent,
        &state,
        &props,
        vec![("data-blocks-pricing-slider-tiers-slider", "")],
        vec![
            slider::label(
                &props,
                vec![("id", "blocks-pricing-slider-tiers-label")],
                vec![text("月間リクエスト数（千件）")],
            ),
            slider::control(
                Orientation::Horizontal,
                &props,
                vec![],
                vec![
                    slider::track(
                        Orientation::Horizontal,
                        &props,
                        vec![],
                        vec![slider::range(&state, &props, vec![])],
                    ),
                    slider::thumb_styled(
                        &state,
                        Some("50 千件"),
                        &props,
                        vec![("aria-labelledby", "blocks-pricing-slider-tiers-label")],
                    ),
                    slider::marker_group(
                        vec![],
                        vec![
                            slider::marker(&state, 10.0, false, vec![], vec![]),
                            slider::marker(&state, 50.0, false, vec![], vec![]),
                            slider::marker(&state, 100.0, false, vec![], vec![]),
                            slider::marker(&state, 500.0, false, vec![], vec![]),
                        ],
                    ),
                ],
            ),
            slider::hidden_input("usage-units", "50", false, vec![]),
        ],
    );

    let selection_caption = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text("50 千件/月の料金を表示中")],
    );

    div(
        vec![("data-blocks-pricing-slider-tiers-usage", "")],
        vec![slider_node, selection_caption],
    )
}

/// プランカード 1 件を組み立てる。推奨プランには帯（先頭の `div`）と
/// badge、強調用の `data-blocks-pricing-slider-tiers-recommended` を付与
/// する。
fn plan_card(plan: &Plan) -> Node {
    let mut heading_children = vec![heading::heading(
        HeadingLevel::H4,
        &HeadingProps::default(),
        vec![],
        vec![text(plan.name)],
    )];
    if plan.recommended {
        heading_children.push(badge::badge(
            &BadgeProps {
                variant: BadgeVariant::Solid,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("おすすめ")],
        ));
    }

    let mut card_children = Vec::new();
    if plan.recommended {
        card_children.push(div(
            vec![("class", "blocks-pricing-slider-tiers-band")],
            vec![text("おすすめ")],
        ));
    }
    card_children.push(card::header(
        vec![],
        vec![
            div(
                vec![("class", "blocks-pricing-slider-tiers-tier-heading")],
                heading_children,
            ),
            card::description(vec![], vec![text(plan.description)]),
        ],
    ));
    card_children.push(card::body(
        vec![],
        vec![div(
            vec![("class", "blocks-pricing-slider-tiers-price")],
            vec![
                text(plan.price),
                span(
                    vec![("class", "blocks-pricing-slider-tiers-price-period")],
                    vec![text(" / 月")],
                ),
            ],
        )],
    ));
    card_children.push(card::footer(
        vec![],
        vec![button::button(
            &ButtonProps {
                variant: if plan.recommended {
                    ButtonVariant::Solid
                } else {
                    ButtonVariant::Outline
                },
                ..ButtonProps::default()
            },
            vec![],
            vec![text(plan.cta)],
        )],
    ));

    let mut attrs = vec![("data-blocks-pricing-slider-tiers-card", "")];
    if plan.recommended {
        attrs.push(("data-blocks-pricing-slider-tiers-recommended", ""));
    }

    card::root(
        CardProps {
            variant: CardVariant::Outline,
            size: Size::Md,
        },
        attrs,
        card_children,
    )
}

/// `pricing-slider-tiers` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。header → 利用量 slider → プランカード 3 枚の順に縦へ並べる
/// （モジュール doc「前半 #2869 と後半 #2870 の分担」節参照）。
pub fn demo() -> Node {
    let cards = div(
        vec![("data-blocks-pricing-slider-tiers-grid", "")],
        PLANS.iter().map(plan_card).collect(),
    );

    div(
        vec![("class", "blocks-pricing-slider-tiers-layout")],
        vec![header(), usage_slider(), cards],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/pricing-slider-tiers/",
    title: "pricing-slider-tiers",
    category: BlockCategory::Pricing,
    rust_source: "crates/docs-site/src/blocks/marketing/pricing/pricing_slider_tiers.rs",
    demo_class: "blocks-pricing-slider-tiers",
    parts: &[
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Slider",
            path: "/themes/slider/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `pricing_slider_tiers` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。既定（狭幅）は縦積み、
/// `>= 48rem`（[`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`]）
/// でカード 3 列へ切り替える。
const LAYOUT_CSS: &str = "\
.blocks-pricing-slider-tiers-layout {\n  display: flex;\n  flex-direction: column;\n  gap: 2rem;\n}\n\
[data-blocks-pricing-slider-tiers-header] {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n  max-width: 36rem;\n}\n\
[data-blocks-pricing-slider-tiers-cta-row] {\n  display: flex;\n  gap: 0.75rem;\n  flex-wrap: wrap;\n}\n\
[data-blocks-pricing-slider-tiers-usage] {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n  width: 100%;\n  max-width: 32rem;\n}\n\
[data-blocks-pricing-slider-tiers-slider] {\n  width: 100%;\n}\n\
[data-blocks-pricing-slider-tiers-grid] {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: 1.5rem;\n  align-items: stretch;\n}\n\
[data-blocks-pricing-slider-tiers-card] {\n  display: flex;\n  flex-direction: column;\n  height: 100%;\n  overflow: hidden;\n}\n\
[data-blocks-pricing-slider-tiers-recommended] {\n  border-color: var(--fandhe-color-accent);\n  border-width: 2px;\n}\n\
.blocks-pricing-slider-tiers-band {\n  background: var(--fandhe-color-accent);\n  color: var(--fandhe-color-accent-fg);\n  text-align: center;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  padding: 0.25rem 0.5rem;\n  margin: -1px -1px 0;\n}\n\
.blocks-pricing-slider-tiers-tier-heading {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: 0.5rem;\n}\n\
.blocks-pricing-slider-tiers-price {\n  display: flex;\n  align-items: baseline;\n  gap: 0.25rem;\n  font-size: var(--fandhe-font-font-size-2xl, 1.5rem);\n  font-weight: var(--fandhe-font-font-weight-bold);\n}\n\
.blocks-pricing-slider-tiers-price-period {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
@media (min-width: 48rem) {\n  [data-blocks-pricing-slider-tiers-grid] {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// header/slider/card/badge/button/heading/text の各 `data-scope` と、
    /// カードの data 属性（3 件・recommended 1 件・帯 1 件・badge 1 件）が
    /// 出力され、`<form>`・暗黙 submit・死リンク・`data:` URI を持ち込んで
    /// いないことを固定する。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());

        for scope in [
            r#"data-scope="heading""#,
            r#"data-scope="text""#,
            r#"data-scope="button""#,
            r#"data-scope="slider""#,
            r#"data-scope="card""#,
            r#"data-scope="badge""#,
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }

        assert_eq!(
            html.matches("data-blocks-pricing-slider-tiers-card=\"\"")
                .count(),
            3,
            "demo output should render exactly 3 plan cards"
        );
        assert_eq!(
            html.matches("data-blocks-pricing-slider-tiers-recommended=\"\"")
                .count(),
            1,
            "demo output should mark exactly 1 recommended card"
        );
        assert_eq!(
            html.matches("blocks-pricing-slider-tiers-band").count(),
            1,
            "demo output should render exactly 1 recommended band"
        );
        assert_eq!(
            html.matches(r#"data-scope="badge""#).count(),
            1,
            "demo output should render exactly 1 badge"
        );

        for absent in ["<form", "type=\"submit\"", "href=\"#\"", "src=\"data:"] {
            assert!(
                !html.contains(absent),
                "demo output should never contain {absent}"
            );
        }
    }

    /// slider thumb の `aria-labelledby` が可視ラベルの `id` を参照して
    /// おり、その `id` が実際に出力へ存在すること。
    #[test]
    fn slider_thumb_is_labelled_by_visible_label() {
        let html = render(&demo());
        assert!(html.contains("aria-labelledby=\"blocks-pricing-slider-tiers-label\""));
        assert!(html.contains("id=\"blocks-pricing-slider-tiers-label\""));
    }

    /// [`LAYOUT_CSS`] が狭幅では縦積み・`>= 48rem` で 3 列へ切り替わること。
    #[test]
    fn layout_css_stacks_on_narrow_and_three_columns_on_md() {
        assert!(LAYOUT_CSS.contains("grid-template-columns: minmax(0, 1fr);"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: repeat(3, minmax(0, 1fr));"));
    }

    /// ルート class（`blocks-pricing-slider-tiers-layout`）が
    /// [`BLOCK`] の `demo_class`（`blocks-pricing-slider-tiers`）とは
    /// 別名であること（`insert_generated_sections` が Demo ラッパへ
    /// `demo_class` を別途付与するため、ルート自身が同名を名乗ると
    /// クラス指定が重複する紛らわしさを避ける、既存 block からの教訓）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-pricing-slider-tiers-layout\""));
        assert_ne!(BLOCK.demo_class, "blocks-pricing-slider-tiers-layout");
    }

    /// 推奨プランの価格が、slider の `aria-valuetext`（50 千件）が指す
    /// 利用量に対応する固定表示であること（両者が同じ値を指す静的な組で
    /// あることの回帰、モジュール doc「静的な組であることの明示」節）。
    #[test]
    fn recommended_plan_price_matches_slider_value() {
        let html = render(&demo());
        let recommended = PLANS
            .iter()
            .find(|p| p.recommended)
            .expect("recommended plan");
        assert!(html.contains(recommended.price));
        assert!(html.contains("aria-valuetext=\"50 千件\""));
    }
}
