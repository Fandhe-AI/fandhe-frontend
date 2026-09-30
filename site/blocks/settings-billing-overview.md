# settings-billing-overview

請求関連の統計値 3 件（今月の請求額・利用シート数・次回請求日）と、現在プラン・
候補プランのカード 2 枚、サブスクリプション一覧テーブルを組み合わせた設定画面
向けブロックです。`stat` / `card` / `badge` / `button` / `table` / `toggle-tip`
の 6 部品を合成します。Blocks は既存部品の合成例であり、新しい UI 部品は
追加しません。

主参照は対応表 ID R0240（`_/blocks-intake/` の対応ファイルは本 worktree に
存在しないため、対応表 ID のみを記載）。統計値・プラン名・価格・
サブスクリプション一覧の行はすべて架空のデータであり、実在の企業・人物・
メールアドレス・クレデンシャル・PII を含みません。請求・決済処理そのもの
（送信先 URL・カード番号・請求先住所などの決済情報）も一切含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。現在プラン
カードの補足ヒントは toggle-tip を開いた状態（`OpenState::Open`）で固定表示
しています。

## Rust コード

```rust
use crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::toggle_tip::{self, OpenState};

/// 補足ヒント `content` の一意な `id`（`toggle_tip::trigger` の `controls`
/// と対で使う。ダングリング aria 参照・id 重複防止は `blocks_contract.rs`
/// が検証する）。
const PLAN_TIP_CONTENT_ID: &str = "blocks-settings-billing-overview-plan-tip";

/// 上段の統計カード 1 枚分のデータ（ラベル・値・増減方向・増減バッジ文言・
/// 補足文）。
struct StatDatum {
    label: &'static str,
    value: &'static str,
    trend_up: bool,
    trend_label: &'static str,
    help: &'static str,
}

/// 上段 3 統計（今月の請求額・利用シート数・次回請求日、いずれも架空値）。
const STATS: &[StatDatum] = &[
    StatDatum {
        label: "今月の請求額",
        value: "$29.00",
        trend_up: false,
        trend_label: "-2.1%",
        help: "先月比で減少",
    },
    StatDatum {
        label: "利用シート数",
        value: "12",
        trend_up: true,
        trend_label: "+3",
        help: "先月比で増加",
    },
    StatDatum {
        label: "次回請求日",
        value: "2026-10-15",
        trend_up: true,
        trend_label: "自動更新",
        help: "解約は前日まで可能",
    },
];

/// サブスクリプション一覧 1 行分（サービス名・プラン・請求日・金額・状態）。
struct SubscriptionRow {
    service: &'static str,
    plan: &'static str,
    billed_on: &'static str,
    amount: &'static str,
    status: &'static str,
    status_variant: BadgeVariant,
}

/// 下段テーブルの架空サブスクリプション 4 件。
const SUBSCRIPTIONS: &[SubscriptionRow] = &[
    SubscriptionRow {
        service: "Lumenbridge Systems",
        plan: "Growth",
        billed_on: "2026-09-15",
        amount: "$29.00",
        status: "有効",
        status_variant: BadgeVariant::Subtle,
    },
    SubscriptionRow {
        service: "Verdant Foundry",
        plan: "Starter",
        billed_on: "2026-09-01",
        amount: "$9.00",
        status: "有効",
        status_variant: BadgeVariant::Subtle,
    },
    SubscriptionRow {
        service: "Trellisworks Co.",
        plan: "Scale",
        billed_on: "2026-08-20",
        amount: "$79.00",
        status: "更新待ち",
        status_variant: BadgeVariant::Outline,
    },
    SubscriptionRow {
        service: "Aurelia Dynamics",
        plan: "Starter",
        billed_on: "2026-07-30",
        amount: "$9.00",
        status: "解約済み",
        status_variant: BadgeVariant::Surface,
    },
];

/// 1 枚の統計カード（`card` + `stat` + `badge` の合成、`dashboard_01` の
/// `stat_card` ヘルパと同型）。
fn stat_card(datum: &StatDatum) -> Node {
    let trend_badge = badge(
        &BadgeProps {
            variant: BadgeVariant::Outline,
            ..BadgeProps::default()
        },
        vec![],
        vec![text(datum.trend_label)],
    );

    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-billing-overview-stat-card", "")],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    card::title(
                        vec![("class", "blocks-settings-billing-overview-stat-label")],
                        vec![text(datum.label)],
                    ),
                    card::action(vec![], vec![trend_badge]),
                ],
            ),
            card::body(
                vec![],
                vec![stat::root(
                    Size::Lg,
                    vec![],
                    vec![stat::value_text(vec![], vec![text(datum.value)])],
                )],
            ),
            card::footer(
                vec![],
                vec![stat::help_text(
                    vec![],
                    vec![
                        if datum.trend_up {
                            stat::up_indicator(vec![])
                        } else {
                            stat::down_indicator(vec![])
                        },
                        text(datum.help),
                    ],
                )],
            ),
        ],
    )
}

/// 現在プランカードに添える補足ヒント（`OpenState::Open` 固定で常時可視。
/// モジュール doc「現在プランの強調は toggle-tip を `Open` 固定で常時可視
/// にする」節参照）。
fn plan_tip() -> Node {
    let state = OpenState::Open;
    toggle_tip::root(
        state,
        vec![],
        vec![
            toggle_tip::trigger(
                state,
                true,
                Some(PLAN_TIP_CONTENT_ID),
                vec![("aria-label", "現在プランの補足")],
                vec![text("\u{24d8}")],
            ),
            toggle_tip::positioner(
                state,
                vec![],
                vec![toggle_tip::content(
                    state,
                    Some(PLAN_TIP_CONTENT_ID),
                    vec![],
                    vec![text(
                        "次回請求日の前日までにダウングレード・解約手続きを完了すると、\
                         当月分の差額は請求されません。",
                    )],
                )],
            ),
        ],
    )
}

/// 現在プランカード（Growth）。ヒント常時表示・強調枠・「プランを管理」
/// ボタンを持つ。
fn current_plan_card() -> Node {
    let (name, price) = SAMPLE_PRICE_TIERS[1];
    card::root(
        CardProps::default(),
        vec![
            ("data-blocks-settings-billing-overview-plan-card", ""),
            ("data-current", ""),
        ],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    card::title(vec![], vec![text(name)]),
                    card::action(
                        vec![],
                        vec![badge(
                            &BadgeProps {
                                variant: BadgeVariant::Solid,
                                ..BadgeProps::default()
                            },
                            vec![],
                            vec![text("現在のプラン")],
                        )],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![
                    div(
                        vec![("class", "blocks-settings-billing-overview-plan-price")],
                        vec![text(format!("{price} / 月"))],
                    ),
                    card::description(
                        vec![],
                        vec![text("成長中のチーム向けに機能を拡張した構成です。")],
                    ),
                    plan_tip(),
                ],
            ),
            card::footer(
                vec![],
                vec![button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("プランを管理")],
                )],
            ),
        ],
    )
}

/// 候補プランカード（Scale）。「アップグレード」導線を持つ。
fn candidate_plan_card() -> Node {
    let (name, price) = SAMPLE_PRICE_TIERS[2];
    card::root(
        CardProps::default(),
        vec![("data-blocks-settings-billing-overview-plan-card", "")],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    card::title(vec![], vec![text(name)]),
                    card::action(
                        vec![],
                        vec![badge(
                            &BadgeProps {
                                variant: BadgeVariant::Outline,
                                ..BadgeProps::default()
                            },
                            vec![],
                            vec![text("おすすめ")],
                        )],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![
                    div(
                        vec![("class", "blocks-settings-billing-overview-plan-price")],
                        vec![text(format!("{price} / 月"))],
                    ),
                    card::description(vec![], vec![text("大規模なチーム向けの上位構成です。")]),
                ],
            ),
            card::footer(
                vec![],
                vec![button(
                    &ButtonProps {
                        variant: ButtonVariant::Solid,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("アップグレード")],
                )],
            ),
        ],
    )
}

/// サブスクリプション一覧テーブルの 1 行。
fn subscription_row(row: &SubscriptionRow) -> Node {
    table::row(
        vec![],
        vec![
            table::row_header(vec![], vec![text(row.service)]),
            table::cell(vec![], vec![text(row.plan)]),
            table::cell(vec![], vec![text(row.billed_on)]),
            table::cell(vec![("data-align", "end")], vec![text(row.amount)]),
            table::cell(
                vec![],
                vec![badge(
                    &BadgeProps {
                        variant: row.status_variant,
                        ..BadgeProps::default()
                    },
                    vec![],
                    vec![text(row.status)],
                )],
            ),
        ],
    )
}

/// `settings-billing-overview` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数。
pub fn demo() -> Node {
    let stats = div(
        vec![("class", "blocks-settings-billing-overview-stats")],
        STATS.iter().map(stat_card).collect(),
    );

    let plans = div(
        vec![("class", "blocks-settings-billing-overview-plans")],
        vec![current_plan_card(), candidate_plan_card()],
    );

    let header_row = table::row(
        vec![],
        vec![
            table::column_header(vec![], vec![text("サービス")]),
            table::column_header(vec![], vec![text("プラン")]),
            table::column_header(vec![], vec![text("請求日")]),
            table::column_header(vec![("data-align", "end")], vec![text("金額")]),
            table::column_header(vec![], vec![text("状態")]),
        ],
    );

    let table_node = table::root(
        TableProps {
            size: Size::Md,
            ..TableProps::default()
        },
        vec![],
        vec![
            table::caption(vec![], vec![text("サブスクリプション一覧")]),
            table::header(vec![], vec![header_row]),
            table::body(vec![], SUBSCRIPTIONS.iter().map(subscription_row).collect()),
        ],
    );

    div(
        vec![("class", "blocks-settings-billing-overview-stack")],
        vec![stats, plans, table_node],
    )
}
```

## 原案差分メモ

- 主参照 R0240 の集約元は 1 件のみのため、他 block にあるような複数版
  （原案差分）はありません。単一構成です。
- 現在プランカードの補足ヒントは、無 JS の静的 Demo では `Closed` 状態だと
  内容を一切閲覧できないため（`positioner`/`content` が `hidden` 存在属性を
  持つため）、`toggle-tip` を `OpenState::Open` 固定にして常時可視にしています。
- 現在プランの強調枠・狭幅（コンテナ幅 40rem 未満）での統計・プランカードの
  1 列化は、いずれも本 block 側の CSS が担っています（`stat`/`card` 部品自体
  の機能ではありません）。

関連情報: [Stat](../themes/stat.md) / [Card](../themes/card.md) /
[Badge](../themes/badge.md) / [Button](../themes/button.md) /
[Table](../themes/table.md) / [Toggle Tip](../themes/toggle-tip.md)
