# pricing-seats-split

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `number-input` /
`switch` / `card` / `button` を合成した、座席数に応じてプラン価格が変わる
料金プランの合成例です。Blocks セクションは新規部品を追加するものでは
なく、既存の Themes/Primitives 部品を組み合わせた実例集であることに注意
してください。

本 Demo は座席数「5」・年払い ON の 1 状態を固定表示する静的な例です。
docs サイトは JS ハイドレーションを一切行わないため、座席数入力の増減
ボタンは `disabled`、入力自体と年払い switch は `readonly` にしています。
価格は座席数と課金周期から決定的に計算した固定値であり、**入力を操作
して価格をリアルタイムに更新する処理（ライブ連動）は実装していません**。
それを組み込む場合は、`fandhe-frontend-wasm-full` のハイドレーション配線
を購読して価格表示を書き換える処理を、利用者自身の Rust コードで実装し
てください（`docs/policy/intentional-non-adoption.md` §3.25 の責務境界:
UI コンポーネント層はバリデーション・データ整形等のアプリケーションロ
ジックを内包しません）。

機能リスト・推奨プランの強調・月払い状態の並記は続く issue で追加予定
です。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::number_input::{self, NumberInputFlags};
use fandhe_frontend_pre_styled_ui::switch::{self, SwitchProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 座席数入力の固定表示値（架空。本 issue では 1 状態のみ、#2866 で状態
/// 違いを並記する際に引数化する）。
const SEATS: u32 = 5;

/// プラン 1 件分（架空、実在の製品・企業とは無関係）。
struct Plan {
    name: &'static str,
    description: &'static str,
    /// 1 座席あたりの月額（USD、整数）。
    per_seat_usd: u32,
    cta: &'static str,
}

/// 3 プラン（`crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS` と同じ
/// 名称・価格水準に揃える）。
const PLANS: [Plan; 3] = [
    Plan {
        name: "Starter",
        description: "小さなチームがまず試すための最小構成です。",
        per_seat_usd: 9,
        cta: "Starter を選ぶ",
    },
    Plan {
        name: "Growth",
        description: "成長中のチーム向けに機能を拡張した構成です。",
        per_seat_usd: 29,
        cta: "Growth を選ぶ",
    },
    Plan {
        name: "Scale",
        description: "大規模なチーム向けの上位構成です。",
        per_seat_usd: 79,
        cta: "Scale を選ぶ",
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

/// 左パネル（見出し・座席数入力・年払い switch）。
fn controls() -> Node {
    let number_flags = NumberInputFlags {
        readonly: true,
        ..NumberInputFlags::default()
    };
    let seats_value = SEATS.to_string();

    let seats_input = number_input::root(
        Size::Md,
        false,
        false,
        true,
        vec![("data-blocks-pricing-seats-split-seats", "")],
        vec![
            number_input::label(
                number_flags,
                Some("blocks-pricing-seats-split-seats"),
                vec![],
                vec![text("座席数")],
            ),
            number_input::control(
                number_flags,
                vec![],
                vec![
                    number_input::decrement_trigger(
                        Some("blocks-pricing-seats-split-seats"),
                        true,
                        vec![],
                        vec![text("-")],
                    ),
                    number_input::input(
                        "seats",
                        Some("blocks-pricing-seats-split-seats"),
                        Some(&seats_value),
                        "1",
                        "100",
                        number_flags,
                        vec![],
                    ),
                    number_input::increment_trigger(
                        Some("blocks-pricing-seats-split-seats"),
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
        ..SwitchProps::default()
    };
    let annual_switch = switch::root(
        Size::Md,
        ColorPalette::Accent,
        true,
        &switch_props,
        vec![("data-blocks-pricing-seats-split-annual", "")],
        vec![
            switch::label(
                true,
                &switch_props,
                vec![],
                vec![text("年払い（20% お得）")],
            ),
            switch::hidden_input("billing-annual", "on", true, &switch_props, vec![]),
            switch::control(
                true,
                &switch_props,
                vec![],
                vec![switch::thumb(true, &switch_props, vec![], vec![])],
            ),
        ],
    );

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
                vec![("class", "blocks-pricing-seats-split-lead")],
                vec![text(
                    "座席数と課金周期を指定すると、各プランの月額が自動で更新されます。",
                )],
            ),
            seats_input,
            annual_switch,
        ],
    )
}

/// プランカード 1 枚。
fn plan_card(plan: &Plan) -> Node {
    let total = monthly_total(plan.per_seat_usd, SEATS, true);
    let regular_total = plan.per_seat_usd * SEATS;

    card::root(
        CardProps::default(),
        vec![("data-blocks-pricing-seats-split-card", "")],
        vec![
            card::header(
                vec![("class", "blocks-pricing-seats-split-card-header")],
                vec![
                    heading(
                        HeadingLevel::H4,
                        &HeadingProps {
                            size: HeadingSize::Lg,
                            weight: HeadingWeight::Semibold,
                        },
                        vec![],
                        vec![text(plan.name)],
                    ),
                    card::description(vec![], vec![text(plan.description)]),
                ],
            ),
            card::body(
                vec![("class", "blocks-pricing-seats-split-price-row")],
                vec![
                    span(
                        vec![("class", "blocks-pricing-seats-split-price")],
                        vec![text(format!("${total}"))],
                    ),
                    span(
                        vec![("class", "blocks-pricing-seats-split-price-period")],
                        vec![text(format!(" / 月（{SEATS} 席・年払い）"))],
                    ),
                    span(
                        vec![("class", "blocks-pricing-seats-split-price-regular")],
                        vec![
                            text("通常 "),
                            el("s", vec![], vec![text(format!("${regular_total}"))]),
                        ],
                    ),
                ],
            ),
            card::footer(
                vec![],
                vec![button::button(
                    &ButtonProps::default(),
                    vec![],
                    vec![text(plan.cta)],
                )],
            ),
        ],
    )
}

/// `pricing-seats-split` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-pricing-seats-split-layout")],
        vec![
            controls(),
            div(
                vec![("class", "blocks-pricing-seats-split-cards")],
                PLANS.iter().map(plan_card).collect(),
            ),
        ],
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
- 機能リスト（`list` + `icon`）・推奨プランの強調・月払い状態の並記は
  続く issue の担当です

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Number Input](../themes/number-input.md) / [Switch](../themes/switch.md) /
[Card](../themes/card.md) / [Button](../themes/button.md)
