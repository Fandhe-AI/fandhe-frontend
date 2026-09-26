//! `pricing-slider-tiers` block（イシュー #2868「Marketing / Pricing の
//! `pricing-slider-tiers`（利用量スライダー＋プランカード 3 枚）」配下。
//! 前半 #2869「骨格と主要領域」に続き、後半 #2870「残り領域・状態表示・
//! 原稿」がプランカードの機能一覧・slider の別位置に対応する価格の状態
//! 併記・原稿を仕上げる（参照は対応表 ID R0206 のみ）。
//!
//! # 使用部品
//!
//! `heading`（見出し）+ `text`（リード文・状態ラベル・選択中の利用量の
//! 併記）+ `button`（CTA 2 個）+ `slider`（利用量、`pricing_usage_slider`
//! と同型）+ `card`（プランカード 3 枚 × 2 状態）+ `badge`（推奨プランの
//! タグ）+ `list`（機能一覧）+ `icon`（機能一覧のチェック装飾）の 8 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約）。
//!
//! # 状態の並記（無 JS、2 状態を固定表示）
//!
//! [`pricing_usage_slider`](super::pricing_usage_slider) と同じ判断で、
//! docs サイトは JS ハイドレーションを一切行わないため slider は固定値で
//! 描画される。本 block はさらに「slider を別の位置に置いたときの価格」を
//! 示すため、[`DemoState`] として 2 状態（50 千件/月・200 千件/月）を
//! [`STATES`] へ並記し、[`demo`] は header を 1 回だけ出したあと状態ごとに
//! slider + カード 3 枚を縦に繰り返す（`pricing_seats_split` 等が集約元
//! との差分を並記で示す先例と同型）。「slider の値を実行時にカード価格へ
//! 反映する」ライブ連動そのものは実装しない
//! （`docs/policy/intentional-non-adoption.md` §3.25 の責務境界）。
//!
//! # 状態の並記と `id` の引数化
//!
//! [`DemoState`] が slider ラベルの `id`（`label_id`）を保持し、
//! [`usage_slider`] へ引数として渡す。2 インスタンスを同一ページへ並記
//! するため `id`/`aria-labelledby` を固定文字列にすると重複・宙に浮いた
//! 参照になる（`crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` の対象）。
//!
//! # 機能リストのチェックが装飾扱いである理由（a11y）
//!
//! 本 block の機能リストは「このプランに含まれる機能一覧」であり、各項目は
//! 常に「含まれる」で情報の可否を運ばない（可否という情報がチェックマーク
//! そのものに宿らない）。そのためチェックアイコンは
//! [`fandhe_frontend_pre_styled_ui::list::indicator`]（常に
//! `aria-hidden="true"` の装飾用パーツ）へ収め、項目本文（プレーンテキスト）
//! だけを意味のある情報として支援技術へ伝える（`pricing_seats_split` と
//! 同じ判断）。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。CTA ボタンは既定 `type="button"` のまま送信先を持たない。
//! プラン名・価格・機能・文言はすべて架空のものであり、実企業名・実クレデ
//! ンシャル・PII を含まない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `card::root`/`slider::root`/`button::button`/`badge::badge`/
//! `heading::heading`/`list::root`/`icon::icon` は `drop_class_attr` により
//! 呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、Demo 固有
//! スタイルは `data-blocks-pricing-slider-tiers-*` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで対応する（`crate::blocks` モジュール
//! doc「CSS フックが `class` と `[data-*]` で混在する理由」節参照）。一方
//! `card::header`/`body`/`footer`（variant を持たず `attrs` をそのまま連結
//! する）と素の `div`/`span`・`list::item` には `class` がそのまま効くため、
//! それらは従来どおりクラスセレクタを使う。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::slider::Slider;
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::Orientation;
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::slider::{self, SliderProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 料金プラン 1 件分の静的データ（架空のプラン名・機能・説明）。価格は
/// slider の位置ごとに変わるため [`DemoState::prices`] 側が持つ。
struct Plan {
    name: &'static str,
    description: &'static str,
    cta: &'static str,
    /// 含まれる機能（架空、実在の製品名を含まない）。
    features: &'static [&'static str],
    recommended: bool,
}

/// 3 段のプラン定義。中央（index 1）を `recommended: true` とする。
const PLANS: &[Plan] = &[
    Plan {
        name: "Starter",
        description: "個人・小規模プロジェクト向け",
        cta: "Get started",
        features: &[
            "プロジェクト数 3 件まで",
            "コミュニティサポート",
            "基本レポート",
        ],
        recommended: false,
    },
    Plan {
        name: "Growth",
        description: "成長中のチーム向け",
        cta: "Get started",
        features: &["プロジェクト数無制限", "優先サポート", "高度なレポート"],
        recommended: true,
    },
    Plan {
        name: "Enterprise",
        description: "大規模組織向け",
        cta: "Contact sales",
        features: &["専任担当者", "SLA 保証", "監査ログ"],
        recommended: false,
    },
];

/// Demo が並記する状態 1 件分（架空）。slider を別の位置に置いたときの
/// 価格差を示す。`label_id`（slider ラベルの `id`）を状態ごとに変えることで
/// 2 インスタンス並記時の id 重複・宙に浮いた `aria-labelledby` 参照を避ける
/// （モジュール doc「状態の並記と `id` の引数化」節）。
struct DemoState {
    /// 状態のラベル（並記時の見出し）。
    label: &'static str,
    /// slider の値（千件）。
    units: f64,
    /// slider の `aria-valuetext`。
    valuetext: &'static str,
    /// slider ラベルの `id`（状態ごとに固有）。
    label_id: &'static str,
    /// 選択中の利用量の併記テキスト。
    caption: &'static str,
    /// 3 プラン分の価格（[`PLANS`] と同じ順序）。
    prices: [&'static str; 3],
}

/// 2 状態（50 千件/月・200 千件/月）を並記する。
const STATES: [DemoState; 2] = [
    DemoState {
        label: "50 千件/月（初期状態）",
        units: 50.0,
        valuetext: "50 千件",
        label_id: "blocks-pricing-slider-tiers-label-50",
        caption: "50 千件/月の料金を表示中",
        prices: ["$29", "$79", "$249"],
    },
    DemoState {
        label: "200 千件/月",
        units: 200.0,
        valuetext: "200 千件",
        label_id: "blocks-pricing-slider-tiers-label-200",
        caption: "200 千件/月の料金を表示中",
        prices: ["$59", "$149", "$449"],
    },
];

/// 機能リストのチェックマーク（装飾。モジュール doc「機能リストのチェック
/// が装飾扱いである理由」節参照）。参照元の形状は持ち込まない独自図形。
fn check_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M5 12.5l4.5 4.5L19 7"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "1.5"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// header 領域（見出し・リード文・CTA 2 個）を組み立てる。状態間で共有
/// するため [`demo`] から 1 回だけ呼ばれる。
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

/// 利用量 slider 領域（`pricing_usage_slider` と同型の構成）。状態ごとに
/// 値・`aria-valuetext`・ラベル `id` を切り替える。
fn usage_slider(state: &DemoState) -> Node {
    let props = SliderProps::default();
    let slider_state = Slider::new(0.0, 500.0, 10.0, state.units, Orientation::Horizontal);
    let units_value = format!("{}", state.units);

    let slider_node = slider::root(
        Size::Md,
        fandhe_frontend_pre_styled_ui::ColorPalette::Accent,
        &slider_state,
        &props,
        vec![("data-blocks-pricing-slider-tiers-slider", "")],
        vec![
            slider::label(
                &props,
                vec![("id", state.label_id)],
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
                        vec![slider::range(&slider_state, &props, vec![])],
                    ),
                    slider::thumb_styled(
                        &slider_state,
                        Some(state.valuetext),
                        &props,
                        vec![("aria-labelledby", state.label_id)],
                    ),
                    slider::marker_group(
                        vec![],
                        vec![
                            slider::marker(&slider_state, 10.0, false, vec![], vec![]),
                            slider::marker(&slider_state, 50.0, false, vec![], vec![]),
                            slider::marker(&slider_state, 100.0, false, vec![], vec![]),
                            slider::marker(&slider_state, 500.0, false, vec![], vec![]),
                        ],
                    ),
                ],
            ),
            slider::hidden_input("usage-units", units_value.as_str(), false, vec![]),
        ],
    );

    let selection_caption = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(state.caption)],
    );

    div(
        vec![("data-blocks-pricing-slider-tiers-usage", "")],
        vec![slider_node, selection_caption],
    )
}

/// プランカード 1 件を組み立てる。推奨プランには帯（先頭の `div`）と
/// badge、強調用の `data-blocks-pricing-slider-tiers-recommended` を付与
/// する。`price` は状態ごとの固定価格（[`DemoState::prices`]）。
fn plan_card(plan: &Plan, price: &'static str) -> Node {
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
            // 帯は装飾のみで、`aria-hidden` により badge と「おすすめ」の
            // 二重読み上げを防ぐ（a11y、任意の追加改善）。
            vec![
                ("class", "blocks-pricing-slider-tiers-band"),
                ("aria-hidden", "true"),
            ],
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

    let features_list = list::root(
        ListType::Unordered,
        ListVariant::Plain,
        vec![("data-blocks-pricing-slider-tiers-features", "")],
        plan.features
            .iter()
            .map(|feature| {
                list::item(
                    vec![("class", "blocks-pricing-slider-tiers-feature")],
                    vec![
                        list::indicator(vec![], vec![check_icon()]),
                        span(vec![], vec![text(*feature)]),
                    ],
                )
            })
            .collect(),
    );

    card_children.push(card::body(
        vec![],
        vec![
            div(
                vec![("class", "blocks-pricing-slider-tiers-price")],
                vec![
                    text(price),
                    span(
                        vec![("class", "blocks-pricing-slider-tiers-price-period")],
                        vec![text(" / 月")],
                    ),
                ],
            ),
            features_list,
        ],
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

/// 状態 1 件分（ラベル + slider + カード 3 枚）。
fn state_section(state: &DemoState) -> Node {
    let cards = div(
        vec![("data-blocks-pricing-slider-tiers-grid", "")],
        PLANS
            .iter()
            .zip(state.prices.iter())
            .map(|(plan, price)| plan_card(plan, price))
            .collect(),
    );

    div(
        vec![("class", "blocks-pricing-slider-tiers-state")],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-pricing-slider-tiers-state-label", "")],
                vec![text(state.label)],
            ),
            usage_slider(state),
            cards,
        ],
    )
}

/// `pricing-slider-tiers` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。header を 1 回だけ出したあと、2 状態（50 千件/月・200 千件/月）
/// の slider + プランカード 3 枚を縦に並べる（モジュール doc「状態の並記」
/// 節参照）。
pub fn demo() -> Node {
    let mut children = vec![header()];
    children.extend(STATES.iter().map(state_section));

    div(
        vec![("class", "blocks-pricing-slider-tiers-layout")],
        children,
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
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `pricing_slider_tiers` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` として
/// `super::stylesheet` から連結される）。既定（狭幅）は縦積み、
/// `>= 48rem`（[`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`]）
/// でカード 3 列へ切り替える。狭幅では slider も全幅にする
/// （`max-width` を media query の中へ移す）。
///
/// 推奨カードの強調ボーダー（`[data-blocks-pricing-slider-tiers-recommended]`）
/// は `card` レシピの base（`[data-scope="card"][data-part="root"]`、
/// 詳細度 0,2,0）・既定 variant（`Outline` の `border-color`、クラス
/// セレクタ併用で 0,3,0）の両方に勝つ必要があるため、`[data-scope="card"]
/// [data-part="root"]` を前置して詳細度 0,3,0 へ揃えている（単独の属性
/// セレクタ 0,1,0 のままだと variant 側に負けてボーダーが表示されない）。
///
/// 機能一覧の規則は `pricing_seats_split` と同じ詳細度で書く
/// （`[data-scope="list"][data-part="root"]`/`[data-scope="list"]
/// [data-part="item"]` を前置して `list` レシピの base に勝つ）。
const LAYOUT_CSS: &str = "\
.blocks-pricing-slider-tiers-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
[data-blocks-pricing-slider-tiers-header] {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n  max-width: 36rem;\n}\n\
[data-blocks-pricing-slider-tiers-cta-row] {\n  display: flex;\n  gap: 0.75rem;\n  flex-wrap: wrap;\n}\n\
.blocks-pricing-slider-tiers-state {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-pricing-slider-tiers-state-label] {\n  margin: 0;\n}\n\
[data-blocks-pricing-slider-tiers-usage] {\n  display: flex;\n  flex-direction: column;\n  gap: 0.75rem;\n  width: 100%;\n}\n\
[data-blocks-pricing-slider-tiers-slider] {\n  width: 100%;\n}\n\
[data-blocks-pricing-slider-tiers-grid] {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: 1.5rem;\n  align-items: stretch;\n}\n\
[data-blocks-pricing-slider-tiers-card] {\n  display: flex;\n  flex-direction: column;\n  height: 100%;\n  overflow: hidden;\n}\n\
[data-scope=\"card\"][data-part=\"root\"][data-blocks-pricing-slider-tiers-recommended] {\n  border-color: var(--fandhe-color-accent);\n  border-width: 2px;\n}\n\
.blocks-pricing-slider-tiers-band {\n  background: var(--fandhe-color-accent);\n  color: var(--fandhe-color-accent-fg);\n  text-align: center;\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n  font-weight: var(--fandhe-font-font-weight-medium, 500);\n  padding: 0.25rem 0.5rem;\n  margin: -1px -1px 0;\n}\n\
.blocks-pricing-slider-tiers-tier-heading {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: 0.5rem;\n}\n\
.blocks-pricing-slider-tiers-price {\n  display: flex;\n  align-items: baseline;\n  gap: 0.25rem;\n  font-size: var(--fandhe-font-font-size-2xl, 1.5rem);\n  font-weight: var(--fandhe-font-font-weight-bold);\n  margin-bottom: var(--fandhe-space-4);\n}\n\
.blocks-pricing-slider-tiers-price-period {\n  font-size: var(--fandhe-font-font-size-sm);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-scope=\"list\"][data-part=\"root\"][data-blocks-pricing-slider-tiers-features] {\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
[data-scope=\"list\"][data-part=\"item\"].blocks-pricing-slider-tiers-feature {\n  display: flex;\n  align-items: center;\n}\n\
@media (min-width: 48rem) {\n  [data-blocks-pricing-slider-tiers-grid] {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n  [data-blocks-pricing-slider-tiers-usage] {\n    max-width: 32rem;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    /// header/slider/card/badge/button/heading/text/list/icon の各
    /// `data-scope` と、カードの data 属性（2 状態 × 3 枚 = 6・recommended
    /// 2・帯 2・badge 2）が出力され、`<form>`・暗黙 submit・死リンク・
    /// `data:` URI を持ち込んでいないことを固定する。
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
            r#"data-scope="list""#,
            r#"data-scope="icon""#,
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }

        assert_eq!(
            html.matches("data-blocks-pricing-slider-tiers-card=\"\"")
                .count(),
            6,
            "demo output should render exactly 6 plan cards (2 states x 3 plans)"
        );
        assert_eq!(
            html.matches("data-blocks-pricing-slider-tiers-recommended=\"\"")
                .count(),
            2,
            "demo output should mark exactly 2 recommended cards (1 per state)"
        );
        assert_eq!(
            html.matches("blocks-pricing-slider-tiers-band").count(),
            2,
            "demo output should render exactly 2 recommended bands"
        );
        assert_eq!(
            html.matches(r#"data-scope="badge""#).count(),
            2,
            "demo output should render exactly 2 badges"
        );

        for absent in ["<form", "type=\"submit\"", "href=\"#\"", "src=\"data:"] {
            assert!(
                !html.contains(absent),
                "demo output should never contain {absent}"
            );
        }
    }

    /// 2 状態それぞれの slider ラベル `id` が相異なり、`aria-labelledby` の
    /// 参照先が実在すること（重複 id・宙に浮いた参照の回帰、モジュール doc
    /// 「状態の並記と `id` の引数化」節）。
    #[test]
    fn slider_thumb_is_labelled_by_visible_label() {
        let html = render(&demo());
        for state in &STATES {
            assert!(html.contains(&format!("aria-labelledby=\"{}\"", state.label_id)));
            assert!(html.contains(&format!("id=\"{}\"", state.label_id)));
        }
    }

    /// [`LAYOUT_CSS`] が狭幅では縦積み・`>= 48rem` で 3 列へ切り替わり、
    /// slider の `max-width` が media query の中に移されていること。
    #[test]
    fn layout_css_stacks_on_narrow_and_three_columns_on_md() {
        assert!(LAYOUT_CSS.contains("grid-template-columns: minmax(0, 1fr);"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: repeat(3, minmax(0, 1fr));"));
        let before_media = LAYOUT_CSS.split("@media").next().unwrap();
        assert!(!before_media.contains("max-width: 32rem;"));
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

    /// 各状態の推奨プランの価格が、その状態の slider `aria-valuetext` が
    /// 指す利用量に対応する固定表示であること（両者が同じ値を指す静的な
    /// 組であることの回帰、モジュール doc「状態の並記」節）。
    #[test]
    fn recommended_plan_price_matches_slider_value() {
        let html = render(&demo());
        for state in &STATES {
            assert!(html.contains(&format!("aria-valuetext=\"{}\"", state.valuetext)));
            assert!(html.contains(state.prices[1]));
        }
    }

    /// 機能一覧の項目が 3 プラン × 各 3 件 × 2 状態 = 18 件出ること。
    #[test]
    fn feature_list_renders_all_items_per_state() {
        let html = render(&demo());
        let total: usize = PLANS.iter().map(|plan| plan.features.len()).sum::<usize>() * 2;
        assert_eq!(total, 18);
        assert_eq!(
            html.matches("class=\"blocks-pricing-slider-tiers-feature\"")
                .count(),
            18
        );
    }
}
