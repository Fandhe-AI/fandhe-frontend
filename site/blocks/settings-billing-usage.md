# settings-billing-usage

現在プランの定義リスト・使用量の進捗バー・支払方法と請求詳細をまとめた
請求設定ブロックです。`data-list` / `progress` / `badge` / `button` /
`card` / `table` / `toggle-tip` の 7 部品を合成します。Blocks は既存部品の
合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0239（代表構成: プラン・使用量・支払方法の 3 カード）
で、R0238（残席数 + 請求履歴テーブル）を集約しています。プラン名・金額・
日付・請求先名・請求履歴はすべて架空のデータであり、実在の企業・PII・
実クレジットカード番号は含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::progress::Progress;
use fandhe_frontend_pre_styled_ui::progress::{self, Orientation, ProgressProps};
use fandhe_frontend_pre_styled_ui::recipe::{ColorPalette, Size};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::toggle_tip::{self, OpenState};

/// 使用量進捗バーの警告しきい値（%）。これ以上の消費率は
/// [`ColorPalette::Warning`] で示す（モジュール doc「警告色の進捗バー」
/// 節参照）。
const WARN_THRESHOLD: f64 = 80.0;

/// 使用量カードの補足 toggle tip の `content` id（ページ内一意、版 A のみ
/// 使用する）。
const USAGE_TIP_ID: &str = "blocks-settings-billing-usage-usage-tip";

/// ラベル・値の 1 行（`data_list::item` + `item-label` + `item-value`）を
/// 組み立てる。
fn row(label: &'static str, value: String) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 指定 orientation の `data_list::root`（CSS フック付き）を組む。
fn list(orientation: DataListOrientation, rows: Vec<Node>) -> Node {
    data_list::root(
        DataListProps {
            orientation,
            ..DataListProps::default()
        },
        vec![("data-blocks-settings-billing-usage-list", "")],
        rows,
    )
}

/// ラベル・消費率（%）・可視の値テキストから 1 本の進捗バー行を組む
/// （モジュール doc「警告色の進捗バー」「`aria-label` の明示」節参照）。
fn usage_bar(label: &'static str, percent: f64, value_text: String) -> Node {
    let p = Progress::new(0.0, 100.0, Some(percent), Orientation::Horizontal);
    let palette = if percent >= WARN_THRESHOLD {
        ColorPalette::Warning
    } else {
        ColorPalette::Accent
    };
    let mut header_children = vec![
        styled_text::text(&TextProps::default(), vec![], vec![text(label)]),
        styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(value_text)],
        ),
    ];
    if percent >= WARN_THRESHOLD {
        header_children.push(badge(
            &BadgeProps {
                variant: BadgeVariant::Outline,
                palette: ColorPalette::Warning,
                ..BadgeProps::default()
            },
            vec![],
            vec![text("上限間近")],
        ));
    }
    div(
        vec![("class", "blocks-settings-billing-usage-usage-row")],
        vec![
            div(
                vec![("class", "blocks-settings-billing-usage-usage-row-header")],
                header_children,
            ),
            progress::root(
                &p,
                &ProgressProps {
                    size: Size::Sm,
                    palette,
                    ..ProgressProps::default()
                },
                None,
                vec![("aria-label", label)],
                vec![p.track(vec![], vec![progress::range(&p, vec![])])],
            ),
        ],
    )
}

/// A: 現在プラン・使用量・支払方法の 3 `card`（R0239）。
fn version_representative() -> Node {
    let (tier_name, tier_price) = dummy_assets::SAMPLE_PRICE_TIERS[1];

    let plan_card = card::root(
        CardProps::default(),
        vec![],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    card::title(vec![], vec![text("現在のプラン")]),
                    card::action(
                        vec![],
                        vec![badge(
                            &BadgeProps {
                                variant: BadgeVariant::Solid,
                                ..BadgeProps::default()
                            },
                            vec![],
                            vec![text(tier_name)],
                        )],
                    ),
                ],
            ),
            card::body(
                vec![],
                vec![list(
                    DataListOrientation::Horizontal,
                    vec![
                        row("プラン名", format!("{tier_name}（{tier_price} / 月）")),
                        row("請求周期", "月次".to_string()),
                        row("次回請求日", "2026-10-15".to_string()),
                        row("利用シート数", "22 / 25 席".to_string()),
                    ],
                )],
            ),
            card::footer(
                vec![],
                vec![button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("プランを変更")],
                )],
            ),
        ],
    );

    let usage_card = card::root(
        CardProps::default(),
        vec![],
        vec![
            card::header(
                vec![("data-has-action", "")],
                vec![
                    card::title(vec![], vec![text("使用量")]),
                    card::action(
                        vec![],
                        vec![toggle_tip::root(
                            OpenState::Open,
                            vec![],
                            vec![
                                toggle_tip::trigger(
                                    OpenState::Open,
                                    true,
                                    Some(USAGE_TIP_ID),
                                    vec![("aria-label", "使用量の補足")],
                                    vec![text("?")],
                                ),
                                toggle_tip::positioner(
                                    OpenState::Open,
                                    vec![],
                                    vec![toggle_tip::content(
                                        OpenState::Open,
                                        Some(USAGE_TIP_ID),
                                        vec![],
                                        vec![text("集計は毎日 0:00 UTC に更新されます。")],
                                    )],
                                ),
                            ],
                        )],
                    ),
                ],
            ),
            card::body(
                vec![("class", "blocks-settings-billing-usage-usage-body")],
                vec![
                    usage_bar("API リクエスト", 82.0, "8,200 / 10,000 回".to_string()),
                    usage_bar("ストレージ", 46.0, "46 / 100 GB".to_string()),
                    usage_bar("メンバー", 50.0, "5 / 10 名".to_string()),
                ],
            ),
        ],
    );

    let billing_card = card::root(
        CardProps::default(),
        vec![],
        vec![
            card::header(
                vec![],
                vec![card::title(vec![], vec![text("支払方法と請求詳細")])],
            ),
            card::body(
                vec![],
                vec![list(
                    DataListOrientation::Vertical,
                    vec![
                        row("支払方法", "クレジットカード（末尾 4321）".to_string()),
                        row("請求先名", dummy_assets::COMPANY_NAMES[2].to_string()),
                        row("請求書送付", "メール".to_string()),
                    ],
                )],
            ),
            card::footer(
                vec![],
                vec![
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("支払方法を更新")],
                    ),
                    button(
                        &ButtonProps {
                            variant: ButtonVariant::Ghost,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("請求先を編集")],
                    ),
                ],
            ),
        ],
    );

    div(
        vec![
            ("class", "blocks-settings-billing-usage-variant"),
            ("data-variant", "a"),
        ],
        vec![
            div(
                vec![("class", "blocks-settings-billing-usage-variant-label")],
                vec![styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        size: TextSize::Sm,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text("A: 代表構成（プラン・使用量・支払方法）")],
                )],
            ),
            div(
                vec![("class", "blocks-settings-billing-usage-cards")],
                vec![plan_card, usage_card, billing_card],
            ),
        ],
    )
}

/// B: 残席数 + 請求履歴テーブル（R0238）。
fn version_seats_and_history() -> Node {
    let seats_percent = 88.0;
    let seats_card = card::root(
        CardProps::default(),
        vec![],
        vec![
            card::header(vec![], vec![card::title(vec![], vec![text("残席数")])]),
            card::body(
                vec![],
                vec![
                    usage_bar("シート使用状況", seats_percent, "22 / 25 席".to_string()),
                    div(
                        vec![("class", "blocks-settings-billing-usage-seats-footer")],
                        vec![
                            badge(
                                &BadgeProps {
                                    variant: BadgeVariant::Outline,
                                    palette: ColorPalette::Warning,
                                    ..BadgeProps::default()
                                },
                                vec![],
                                vec![text("残り 3 席")],
                            ),
                            button(&ButtonProps::default(), vec![], vec![text("席を追加")]),
                        ],
                    ),
                ],
            ),
        ],
    );

    const HISTORY_ROWS: &[(&str, &str, &str, &str, BadgeVariant)] = &[
        (
            "2026-09-15",
            "月額プラン（Growth）",
            "$29.00",
            "支払済",
            BadgeVariant::Subtle,
        ),
        (
            "2026-08-15",
            "月額プラン（Growth）",
            "$29.00",
            "支払済",
            BadgeVariant::Subtle,
        ),
        (
            "2026-07-15",
            "追加シート x3",
            "$15.00",
            "処理中",
            BadgeVariant::Outline,
        ),
        (
            "2026-06-15",
            "月額プラン（Growth）",
            "$29.00",
            "失敗",
            BadgeVariant::Surface,
        ),
    ];

    let history_table = table::root(
        TableProps::default(),
        vec![],
        vec![
            table::caption(vec![], vec![text("請求履歴")]),
            table::header(
                vec![],
                vec![table::row(
                    vec![],
                    vec![
                        table::column_header(vec![], vec![text("請求日")]),
                        table::column_header(vec![], vec![text("内容")]),
                        table::column_header(vec![("data-align", "end")], vec![text("金額")]),
                        table::column_header(vec![], vec![text("状態")]),
                    ],
                )],
            ),
            table::body(
                vec![],
                HISTORY_ROWS
                    .iter()
                    .map(|(date, desc, amount, status, variant)| {
                        table::row(
                            vec![],
                            vec![
                                table::row_header(vec![], vec![text(*date)]),
                                table::cell(vec![], vec![text(*desc)]),
                                table::cell(vec![("data-align", "end")], vec![text(*amount)]),
                                table::cell(
                                    vec![],
                                    vec![badge(
                                        &BadgeProps {
                                            variant: *variant,
                                            ..BadgeProps::default()
                                        },
                                        vec![],
                                        vec![text(*status)],
                                    )],
                                ),
                            ],
                        )
                    })
                    .collect(),
            ),
        ],
    );

    div(
        vec![
            ("class", "blocks-settings-billing-usage-variant"),
            ("data-variant", "b"),
        ],
        vec![
            div(
                vec![("class", "blocks-settings-billing-usage-variant-label")],
                vec![styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        size: TextSize::Sm,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text("B: 残席数 + 請求履歴テーブル")],
                )],
            ),
            div(
                vec![("class", "blocks-settings-billing-usage-cards")],
                vec![seats_card, history_table],
            ),
        ],
    )
}

/// `settings-billing-usage` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-billing-usage-stack")],
        vec![version_representative(), version_seats_and_history()],
    )
}
```

## 原案差分メモ

- **版 A（代表構成、R0239）**: 現在プラン（`data-list` 横並び + プラン名
  バッジ）→ 使用量（`progress` 3 本 + toggle tip 補足）→ 支払方法と請求
  詳細（`data-list` 縦積み）の 3 `card` を縦に積みます。
- **版 B（残席数 + 請求履歴テーブル、R0238）**: 残席数の `progress` 1 本を
  持つ `card` と、請求履歴の `table` の 2 ブロックで構成します。
- 消費率が 80% 以上の進捗バーは警告色（`ColorPalette::Warning`）で示し、
  「上限間近」バッジを併記します（版 A の API リクエスト・版 B の残席数）。
- 使用量カードの見出し横の toggle tip は `OpenState::Open` 固定で、トリガー
  は `disabled` にしています。無 JS の静的 Demo で常時可視の補足文にする
  ための構成であり、開閉操作自体は意味を持ちません。
- 版 A は 3 `card` を常時 1 列で縦に積みます。版 B は狭幅（コンテナ幅
  40rem 未満）で広幅 2 列グリッド配置（`card` + `table`）が 1 列へ
  切り替わります（`@container` によるコンテナクエリ判定）。

関連情報: [Data List](../themes/data-list.md) / [Progress](../themes/progress.md) /
[Badge](../themes/badge.md) / [Button](../themes/button.md) /
[Card](../themes/card.md) / [Table](../themes/table.md) /
[Toggle Tip](../themes/toggle-tip.md)
