# pricing-slider-tiers

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `button` / `slider` /
`card` / `badge` 部品を合成した、利用量スライダーとプランカード 3 枚を組み
合わせた料金セクションの合成例です（対応表 ID R0206）。Blocks セクションは
新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください。

本 Demo は静的な表示例です。docs サイトは JS ハイドレーションを一切行わ
ないため、`slider` は固定の初期値（50 千件/月）で描画され、その値に対応
する固定価格を各プランカードへ表示するのみです。**実運用でスライダーの
値をリアルタイムにカード価格へ反映する処理（ライブ連動）は実装していま
せん**。それを組み込む場合は、`fandhe-frontend-wasm-full` のハイドレーシ
ョン配線を購読して価格表示を書き換える処理を、利用者自身の Rust コードで
実装してください（`docs/policy/intentional-non-adoption.md` §3.25 の責務
境界: UI コンポーネント層はバリデーション・データ整形等のアプリケーション
ロジックを内包しません）。

CTA ボタンは `type="button"` のまま送信先を持たない静的な合成例であり、
`<form>` は出力しません。プラン名・価格・文言はすべて架空のものです。

本イシュー（#2869、親 #2868 の前半）では骨格（見出し・リード文・CTA、利用
量 slider、プランカード 3 枚、推奨カードの帯・badge による強調、狭幅で
縦積み・`>= 48rem` で 3 列になるレスポンシブ）までを実装しています。プラン
カードの機能一覧（`list`/`icon`）と、slider の別位置に対応する価格の状態
違い併記は後半 #2870 で追加します。

## Rust コード

```rust
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
```

## 差分メモ

- **既存の `pricing-usage-slider` との主な差**: `pricing-usage-slider` は
  slider + stat（単一の想定コスト表示）のみで、複数プランの価格を並べる
  構成を持ちません。本 block は同じ利用量 slider の骨格を流用しつつ、
  slider の値に対応する固定価格をプランカード 3 枚（`card`+`badge`）へ
  展開する点が異なります。
- 機能一覧（`list`+`icon`）と、slider の別位置に対応する価格の状態違い
  併記は後半 #2870 で追加します。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Button](../themes/button.md) / [Slider](../themes/slider.md) /
[Card](../themes/card.md) / [Badge](../themes/badge.md)
