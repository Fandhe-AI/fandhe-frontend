# pricing-slider-tiers

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `button` / `slider` /
`card` / `badge` / `list` / `icon` 部品を合成した、利用量スライダーとプラン
カード 3 枚を組み合わせた料金セクションの合成例です（対応表 ID R0206）。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください。

本 Demo は静的な表示例です。docs サイトは JS ハイドレーションを一切行わ
ないため、`slider` は「50 千件/月（初期状態）」「200 千件/月」の 2 状態を
固定値で並記し、それぞれに対応する固定価格を各プランカードへ表示するのみ
です。**実運用でスライダーの値をリアルタイムにカード価格へ反映する処理
（ライブ連動）は実装していません**。それを組み込む場合は、
`fandhe-frontend-wasm-full` のハイドレーション配線を購読して価格表示を
書き換える処理を、利用者自身の Rust コードで実装してください
（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コンポー
ネント層はバリデーション・データ整形等のアプリケーションロジックを内包
しません）。

各プランカードには機能一覧（`list`+`icon`）を表示します。一覧の各項目は
「このプランに含まれる機能」であり可否の情報を運ばないため、チェック
アイコンは常に装飾（`aria-hidden`）として扱い、支援技術へは項目本文の
プレーンテキストのみを伝えます。推奨プランは帯・badge・強調枠の 3 点で
強調します。

CTA ボタンは `type="button"` のまま送信先を持たない静的な合成例であり、
`<form>` は出力しません。プラン名・価格・機能・文言はすべて架空のもので
す。

## Rust コード

```rust
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
```

## 差分メモ

- **既存の `pricing-usage-slider` との主な差**: `pricing-usage-slider` は
  slider + stat（単一の想定コスト表示）のみで、複数プランの価格を並べる
  構成や機能一覧を持ちません（R0206）。本 block は同じ利用量 slider の
  骨格を流用しつつ、slider の値に対応する固定価格をプランカード 3 枚
  （`card`+`badge`+`list`+`icon`）へ展開する点が異なります。
- slider を別の位置に置いたときの価格差は、JS 連動ではなく 2 状態
  （50 千件/月・200 千件/月）の並記で示します。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Button](../themes/button.md) / [Slider](../themes/slider.md) /
[Card](../themes/card.md) / [Badge](../themes/badge.md) /
[List](../themes/list.md) / [Icon](../themes/icon.md)
