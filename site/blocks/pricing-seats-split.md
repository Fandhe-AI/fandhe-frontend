# pricing-seats-split

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `number-input` /
`switch` / `card` / `button` / `list` / `icon` を合成した、座席数に応じて
プラン価格が変わる料金プランの合成例です。Blocks セクションは新規部品を
追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集
であることに注意してください。

本 Demo は「年払い（初期状態）」「月払い」の 2 状態を並記する静的な例
です。docs サイトは JS ハイドレーションを一切行わないため、座席数入力の
増減ボタンは `disabled`、入力自体と年払い switch は `readonly` にして
います。座席数は両状態とも「5」に固定し、課金周期だけを変えることで
価格差を比較できるようにしています。価格は座席数と課金周期から決定的に
計算した固定値であり、**入力を操作して価格をリアルタイムに更新する処理
（ライブ連動）は実装していません**。それを組み込む場合は、
`fandhe-frontend-wasm-full` のハイドレーション配線を購読して価格表示を
書き換える処理を、利用者自身の Rust コードで実装してください
（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界: UI コン
ポーネント層はバリデーション・データ整形等のアプリケーションロジックを
内包しません）。

各プランカードには機能リストを添えています。項目はいずれも「このプラン
に含まれる機能」であり可否情報を持たないため、チェックマークは装飾
（`list::indicator`、常に `aria-hidden`）として扱い、項目本文だけを意味
のある情報として伝えます。推奨プラン（Growth）は輪郭を強調したカード +
見出し先頭の平文「おすすめ」で強調しています。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::number_input::{self, NumberInputFlags};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// Demo が並記する状態 1 件分（架空）。座席数入力の `id`（`input_id`）を
/// 状態ごとに変えることで、2 インスタンス並記時の id 重複・宙に浮いた
/// `aria-controls`/`for` 参照を避ける（モジュール doc「状態の並記と `id`
/// の引数化」節）。
struct DemoState {
    /// 状態のラベル（並記時の見出し）。
    label: &'static str,
    /// 座席数（両状態で揃え、周期のみ変える）。
    seats: u32,
    /// 年払いかどうか。
    annual: bool,
    /// 座席数入力の `id`（状態ごとに固有）。
    input_id: &'static str,
}

/// 2 状態（年払い・月払い）を並記する。座席数は両状態とも 5 に固定。
const STATES: [DemoState; 2] = [
    DemoState {
        label: "年払い（初期状態）",
        seats: 5,
        annual: true,
        input_id: "blocks-pricing-seats-split-seats-annual",
    },
    DemoState {
        label: "月払い",
        seats: 5,
        annual: false,
        input_id: "blocks-pricing-seats-split-seats-monthly",
    },
];

/// プラン 1 件分（架空、実在の製品・企業とは無関係）。
struct Plan {
    name: &'static str,
    description: &'static str,
    /// 1 座席あたりの月額（USD、整数）。
    per_seat_usd: u32,
    cta: &'static str,
    /// 含まれる機能（架空、実在の製品名を含まない）。
    features: &'static [&'static str],
    /// 推奨プランかどうか（強調表示、モジュール doc「推奨強調に badge を
    /// 使わない理由」節）。
    featured: bool,
}

/// 3 プラン（`crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS` と同じ
/// 名称・価格水準に揃える）。
const PLANS: [Plan; 3] = [
    Plan {
        name: "Starter",
        description: "小さなチームがまず試すための最小構成です。",
        per_seat_usd: 9,
        cta: "Starter を選ぶ",
        features: &[
            "プロジェクト数 3 件まで",
            "コミュニティサポート",
            "基本レポート",
        ],
        featured: false,
    },
    Plan {
        name: "Growth",
        description: "成長中のチーム向けに機能を拡張した構成です。",
        per_seat_usd: 29,
        cta: "Growth を選ぶ",
        features: &["プロジェクト数無制限", "優先サポート", "高度なレポート"],
        featured: true,
    },
    Plan {
        name: "Scale",
        description: "大規模なチーム向けの上位構成です。",
        per_seat_usd: 79,
        cta: "Scale を選ぶ",
        features: &["専任担当者", "SLA 保証", "監査ログ"],
        featured: false,
    },
];

/// 座席数・課金周期から月額合計を求める決定的な純関数。年払いは 20% 引き
/// （整数演算のみ、四捨五入は行わない）。外部入力・ユーザー入力は受け
/// 取らない。
#[must_use]
fn monthly_total(per_seat_usd: u32, seats: u32, annual: bool) -> u32 {
    if annual {
        per_seat_usd * seats * 8 / 10
    } else {
        per_seat_usd * seats
    }
}

/// 機能リストのチェックマーク（装飾。モジュール doc「機能リストの
/// チェックが装飾扱いである理由」節参照）。参照元の形状は持ち込まない
/// 独自図形。
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

/// 左パネル（見出し・座席数入力・年払い switch）。
fn controls(state: &DemoState) -> Node {
    let number_flags = NumberInputFlags {
        readonly: true,
        ..NumberInputFlags::default()
    };
    let seats_value = state.seats.to_string();

    let seats_input = number_input::root(
        Size::Md,
        false,
        false,
        true,
        vec![("data-blocks-pricing-seats-split-seats", "")],
        vec![
            number_input::label(
                number_flags,
                Some(state.input_id),
                vec![],
                vec![text("座席数")],
            ),
            number_input::control(
                number_flags,
                vec![],
                vec![
                    number_input::decrement_trigger(
                        Some(state.input_id),
                        true,
                        vec![],
                        vec![text("-")],
                    ),
                    number_input::input(
                        "seats",
                        Some(state.input_id),
                        Some(&seats_value),
                        "1",
                        "100",
                        number_flags,
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some(state.input_id),
                        true,
                        vec![],
                        vec![text("+")],
                    ),
                ],
            ),
        ],
    );

    let switch_props = SwitchProps {
        readonly: true,
        // `readonly` は `data-readonly` を出すのみで native checkbox の
        // クリック/Space による切り替え自体は止めない（headless-ui
        // `SwitchProps::readonly` doc「native トグル操作自体を抑止する
        // 配線は持たない」節）。座席数入力の増減トリガーと同じ判断で
        // `disabled: true` も付与し、[`switch::hidden_input`] へ native
        // `disabled` を出力して操作を実際に抑止する。
        disabled: true,
        ..SwitchProps::default()
    };
    let annual_switch = switch::root(
        Size::Md,
        ColorPalette::Accent,
        state.annual,
        &switch_props,
        vec![("data-blocks-pricing-seats-split-annual", "")],
        vec![
            switch::label(
                state.annual,
                &switch_props,
                vec![],
                vec![text("年払い（20% お得）")],
            ),
            switch::hidden_input("billing-annual", "on", state.annual, &switch_props, vec![]),
            switch::control(
                state.annual,
                &switch_props,
                vec![],
                vec![switch::thumb(state.annual, &switch_props, vec![], vec![])],
            ),
        ],
    );

    let lead_text = if state.annual {
        "座席数 5・年払いの場合の月額例です（本 Demo は静的表示のため入力と連動しません）。"
    } else {
        "座席数 5・月払いの場合の月額例です（本 Demo は静的表示のため入力と連動しません）。"
    };
    let annual_status_text = if state.annual {
        "現在、年払いを選択中です（本 Demo は固定表示のため切替できません）。"
    } else {
        "現在、月払いを選択中です（本 Demo は固定表示のため切替できません）。"
    };

    div(
        vec![("class", "blocks-pricing-seats-split-controls")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("座席数に合わせてプランを選ぶ")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-pricing-seats-split-lead", "")],
                vec![text(lead_text)],
            ),
            seats_input,
            annual_switch,
            // `disabled: true` の checkbox は支援技術が checked 状態を
            // 安定して読み上げない（ブラウザ・スクリーンリーダーの
            // 組み合わせにより無視され得る）ため、選択中の周期を読める
            // 平文でも明示する（Codex レビュー指摘、イシュー #2865
            // PR #3278）。
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-pricing-seats-split-annual-status", "")],
                vec![text(annual_status_text)],
            ),
        ],
    )
}

/// プランカード 1 枚。
fn plan_card(plan: &Plan, state: &DemoState) -> Node {
    let total = monthly_total(plan.per_seat_usd, state.seats, state.annual);
    let regular_total = plan.per_seat_usd * state.seats;

    let (variant, card_state) = if plan.featured {
        (CardVariant::Elevated, "featured")
    } else {
        (CardVariant::Outline, "default")
    };
    let button_variant = if plan.featured {
        ButtonVariant::Solid
    } else {
        ButtonVariant::Outline
    };

    let mut header_children: Vec<Node> = Vec::new();
    if plan.featured {
        header_children.push(styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![("data-blocks-pricing-seats-split-recommended", "")],
            vec![text("おすすめ")],
        ));
    }
    header_children.push(heading(
        HeadingLevel::H4,
        &HeadingProps {
            size: HeadingSize::Lg,
            weight: HeadingWeight::Semibold,
        },
        vec![],
        vec![text(plan.name)],
    ));
    header_children.push(card::description(vec![], vec![text(plan.description)]));

    let mut price_row_children = vec![
        span(
            vec![("class", "blocks-pricing-seats-split-price")],
            vec![text(format!("${total}"))],
        ),
        span(
            vec![("class", "blocks-pricing-seats-split-price-period")],
            vec![text(format!(
                " / 月（{} 席・{}）",
                state.seats,
                if state.annual {
                    "年払い"
                } else {
                    "月払い"
                }
            ))],
        ),
    ];
    if state.annual {
        price_row_children.push(span(
            vec![("class", "blocks-pricing-seats-split-price-regular")],
            vec![
                text("通常 "),
                el("s", vec![], vec![text(format!("${regular_total}"))]),
            ],
        ));
    }

    let features_list = list::root(
        ListType::Unordered,
        ListVariant::Plain,
        vec![("data-blocks-pricing-seats-split-features", "")],
        plan.features
            .iter()
            .map(|feature| {
                list::item(
                    vec![("class", "blocks-pricing-seats-split-feature")],
                    vec![
                        list::indicator(vec![], vec![check_icon()]),
                        span(vec![], vec![text(*feature)]),
                    ],
                )
            })
            .collect(),
    );

    card::root(
        CardProps {
            variant,
            ..CardProps::default()
        },
        vec![("data-blocks-pricing-seats-split-card", card_state)],
        vec![
            card::header(
                vec![("class", "blocks-pricing-seats-split-card-header")],
                header_children,
            ),
            card::body(
                vec![],
                vec![
                    div(
                        vec![("class", "blocks-pricing-seats-split-price-row")],
                        price_row_children,
                    ),
                    features_list,
                ],
            ),
            card::footer(
                vec![],
                vec![button::button(
                    &ButtonProps {
                        variant: button_variant,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text(plan.cta)],
                )],
            ),
        ],
    )
}

/// 状態 1 件分（ラベル + 2 カラム本体）。
fn state_section(state: &DemoState) -> Node {
    div(
        vec![("class", "blocks-pricing-seats-split-state")],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-pricing-seats-split-state-label", "")],
                vec![text(state.label)],
            ),
            div(
                vec![("class", "blocks-pricing-seats-split-split")],
                vec![
                    controls(state),
                    div(
                        vec![("class", "blocks-pricing-seats-split-cards")],
                        PLANS.iter().map(|plan| plan_card(plan, state)).collect(),
                    ),
                ],
            ),
        ],
    )
}

/// `pricing-seats-split` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。2 状態（年払い・月払い）を並記する（モジュール doc「静的表示」
/// 節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-pricing-seats-split-layout")],
        STATES.iter().map(state_section).collect(),
    )
}
```

## 原案差分メモ

- 見出しレベルは `## Demo` が `h2` を出すため、パネル見出しに `h3`、プラン
  名に `h4` を使っています
- 座席数入力・年払い switch は無 JS のため `readonly` にし、増減ボタンは
  操作不能を明示するため `disabled` にしています
- 価格は座席数・課金周期からの決定的な固定計算で、実行時の入力連動は
  行いません
- 割引前価格の取り消し線には、視覚に依存しない読み上げのため平文の
  「通常」を添えています
- 状態切替（年払い/月払い）は実行時の JS 切替ではなく、2 状態を静的に
  並記する形で表現しています。座席数は両状態とも 5 に固定しています
- 推奨プランの強調は badge ではなく、輪郭を強調したカード + accent 枠 +
  見出し先頭の平文「おすすめ」で表しています（使用部品に `badge` を
  含めないため）
- 機能リストのチェックマークは、可否情報を持たない装飾として
  `aria-hidden` を付けています（可否が変わらない項目のため）

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Number Input](../themes/number-input.md) / [Switch](../themes/switch.md) /
[Card](../themes/card.md) / [Button](../themes/button.md) /
[List](../themes/list.md) / [Icon](../themes/icon.md)
