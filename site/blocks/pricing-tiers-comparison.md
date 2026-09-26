# pricing-tiers-comparison

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `badge` / `card` /
`button` / `list` / `icon` / `table` 部品を合成した、カードと表の 2 段構成
の料金セクションの合成例です。Blocks セクションは新規部品を追加するもの
ではなく、既存の Themes/Primitives 部品を組み合わせた実例集であることに
注意してください。

上段にプランカード 3 枚、下段に同じプランの詳細比較表（カテゴリ見出し行
付き）を配置します。幅広（48rem 以上）では比較表を表示し、狭幅ではその
代わりに「プランごとの機能一覧」を表示します（無 JS の静的な表示切替）。

本イシューは規模が大きいため 2 分割します。**前半（本イシュー #2873）は
骨格と主要領域（カード・比較表・狭幅一覧）まで**を実装し、**後半
（#2874）でカードと表の間の顧客ロゴ列・暗色帯 + 周期切替のバリエーション・
状態の並記・差分メモの最終版**を仕上げます。

CTA ボタンは `type="button"` のまま送信先を持たない静的な合成例であり、
`<form>` は出力しません。プラン名・価格・機能・文言はすべて架空のもので
す。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 表の列数（機能名列 + プラン数）を文字列表記した定数。プラン数が
/// [`PLANS`] 固定のため `colspan` 属性値としてリテラルで持つ（ファイル内
/// ユニットテストで `PLANS.len() + 1` との一致を固定）。
const COL_COUNT_STR: &str = "4";

/// 機能可否・値の表現。
enum FeatureValue {
    /// 含まれる（チェックアイコン）。
    Included,
    /// 含まれない（バツアイコン）。
    Excluded,
    /// 文字表示（例: 上限件数）。
    Text(&'static str),
}

/// 料金プラン 1 件分の静的データ（架空）。
struct Plan {
    name: &'static str,
    price: &'static str,
    description: &'static str,
    cta: &'static str,
    recommended: bool,
}

/// 機能比較 1 行分。`values` の長さは [`PLANS`] の件数と一致させる
/// （ファイル内ユニットテストで固定する不変条件）。
struct FeatureRow {
    label: &'static str,
    values: &'static [FeatureValue],
}

/// カテゴリ見出し行 1 件分（機能行のグルーピング単位、`table::body` 1 個に
/// 対応する）。
struct Category {
    label: &'static str,
    rows: &'static [FeatureRow],
}

/// プラン 3 件（中央 Growth を推奨プランとする）。価格は
/// `crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS` と同じ値を直書きする。
const PLANS: [Plan; 3] = [
    Plan {
        name: "Starter",
        price: "$9",
        description: "個人・小規模プロジェクト向け",
        cta: "Starter を選ぶ",
        recommended: false,
    },
    Plan {
        name: "Growth",
        price: "$29",
        description: "成長中のチーム向け",
        cta: "Growth を選ぶ",
        recommended: true,
    },
    Plan {
        name: "Scale",
        price: "$79",
        description: "大規模組織向け",
        cta: "Scale を選ぶ",
        recommended: false,
    },
];

/// カテゴリ 2 件・機能行合計 5 件（架空データ）。
const CATEGORIES: [Category; 2] = [
    Category {
        label: "基本機能",
        rows: &[
            FeatureRow {
                label: "プロジェクト数",
                values: &[
                    FeatureValue::Text("3 件"),
                    FeatureValue::Text("無制限"),
                    FeatureValue::Text("無制限"),
                ],
            },
            FeatureRow {
                label: "ストレージ容量",
                values: &[
                    FeatureValue::Text("5GB"),
                    FeatureValue::Text("100GB"),
                    FeatureValue::Text("1TB"),
                ],
            },
            FeatureRow {
                label: "カスタムドメイン",
                values: &[
                    FeatureValue::Excluded,
                    FeatureValue::Included,
                    FeatureValue::Included,
                ],
            },
        ],
    },
    Category {
        label: "サポート",
        rows: &[
            FeatureRow {
                label: "優先サポート窓口",
                values: &[
                    FeatureValue::Excluded,
                    FeatureValue::Included,
                    FeatureValue::Included,
                ],
            },
            FeatureRow {
                label: "専任担当者",
                values: &[
                    FeatureValue::Excluded,
                    FeatureValue::Excluded,
                    FeatureValue::Included,
                ],
            },
        ],
    },
];

/// 可否を表す自作の抽象チェック/バツ図形。意味を持つアイコンのため
/// `label` を指定する（参照元のアイコン形状・内部識別子は持ち込まない）。
fn value_node(value: &FeatureValue) -> Node {
    match value {
        FeatureValue::Included => icon(
            &IconProps {
                size: Size::Sm,
                label: Some("含まれる"),
                ..IconProps::default()
            },
            vec![("data-blocks-pricing-tiers-comparison-value", "included")],
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
        ),
        FeatureValue::Excluded => icon(
            &IconProps {
                size: Size::Sm,
                label: Some("含まれない"),
                ..IconProps::default()
            },
            vec![("data-blocks-pricing-tiers-comparison-value", "excluded")],
            vec![
                el(
                    "path",
                    vec![
                        ("d", "M7 7l10 10"),
                        ("stroke", "currentColor"),
                        ("stroke-width", "1.5"),
                        ("stroke-linecap", "round"),
                    ],
                    vec![],
                ),
                el(
                    "path",
                    vec![
                        ("d", "M17 7l-10 10"),
                        ("stroke", "currentColor"),
                        ("stroke-width", "1.5"),
                        ("stroke-linecap", "round"),
                    ],
                    vec![],
                ),
            ],
        ),
        FeatureValue::Text(label) => span(
            vec![("data-blocks-pricing-tiers-comparison-value", "text")],
            vec![text(*label)],
        ),
    }
}

/// header 領域（見出し・リード文）。
fn header() -> Node {
    div(
        vec![("data-blocks-pricing-tiers-comparison-header", "")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("あなたに合うプランを選ぶ")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "3 つのプランを機能ごとに比較して、必要な範囲を見極められます。",
                )],
            ),
        ],
    )
}

/// プランカード 1 件。推奨プラン（Growth）には badge と強調ボーダーを
/// 付ける。
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

    let mut attrs = vec![("data-blocks-pricing-tiers-comparison-card", "")];
    if plan.recommended {
        attrs.push(("data-blocks-pricing-tiers-comparison-recommended", ""));
    }

    card::root(
        CardProps {
            variant: CardVariant::Outline,
            size: Size::Md,
        },
        attrs,
        vec![
            card::header(
                vec![],
                vec![
                    div(
                        vec![("class", "blocks-pricing-tiers-comparison-tier-heading")],
                        heading_children,
                    ),
                    card::description(vec![], vec![text(plan.description)]),
                ],
            ),
            card::body(
                vec![],
                vec![div(
                    vec![("class", "blocks-pricing-tiers-comparison-price")],
                    vec![
                        text(plan.price),
                        span(
                            vec![("class", "blocks-pricing-tiers-comparison-price-period")],
                            vec![text(" / 月")],
                        ),
                    ],
                )],
            ),
            card::footer(
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
            ),
        ],
    )
}

/// 幅広の比較表（`>= 48rem` でのみ表示、[`LAYOUT_CSS`] 参照）。
fn comparison_table() -> Node {
    let mut header_cells = vec![table::column_header(vec![], vec![text("機能")])];
    header_cells.extend(
        PLANS
            .iter()
            .map(|plan| table::column_header(vec![], vec![text(plan.name)])),
    );

    let bodies: Vec<Node> = CATEGORIES
        .iter()
        .map(|category| {
            let mut rows = vec![table::row(
                vec![],
                vec![table::row_header(
                    vec![
                        ("colspan", COL_COUNT_STR),
                        ("data-blocks-pricing-tiers-comparison-category", ""),
                    ],
                    vec![text(category.label)],
                )],
            )];
            rows.extend(category.rows.iter().map(|row| {
                let mut cells = vec![table::row_header(
                    vec![("data-blocks-pricing-tiers-comparison-feature", "")],
                    vec![text(row.label)],
                )];
                cells.extend(
                    row.values
                        .iter()
                        .map(|value| table::cell(vec![], vec![value_node(value)])),
                );
                table::row(vec![], cells)
            }));
            table::body(vec![], rows)
        })
        .collect();

    let mut table_children = vec![
        table::caption(vec![], vec![text("プラン別の機能比較")]),
        table::header(vec![], vec![table::row(vec![], header_cells)]),
    ];
    table_children.extend(bodies);

    let table_node = table::root(
        TableProps {
            variant: TableVariant::Outline,
            ..TableProps::default()
        },
        vec![("data-blocks-pricing-tiers-comparison-table", "")],
        table_children,
    );

    div(
        vec![("data-blocks-pricing-tiers-comparison-view", "table")],
        vec![table::scroll_area(
            vec![
                ("role", "region"),
                ("aria-label", "プラン比較表"),
                ("tabindex", "0"),
                ("data-blocks-pricing-tiers-comparison-scroll", ""),
            ],
            vec![table_node],
        )],
    )
}

/// プラン 1 件分の狭幅一覧（プラン名見出し + 機能一覧）。可否アイコンは
/// 情報を運ぶため `list::indicator`（装飾専用）には収めない
/// （モジュール doc「機能一覧の可否アイコンが意味を持つ理由」節参照）。
fn plan_list(plan_index: usize, plan: &Plan) -> Node {
    let items: Vec<Node> = CATEGORIES
        .iter()
        .flat_map(|category| category.rows.iter())
        .map(|row| {
            let value = &row.values[plan_index];
            let item_children = match value {
                FeatureValue::Text(v) => {
                    vec![span(vec![], vec![text(format!("{}: {v}", row.label))])]
                }
                included_or_excluded => {
                    vec![value_node(included_or_excluded), text(row.label)]
                }
            };
            list::item(
                vec![("class", "blocks-pricing-tiers-comparison-list-item")],
                item_children,
            )
        })
        .collect();

    div(
        vec![("class", "blocks-pricing-tiers-comparison-plan-list")],
        vec![
            heading::heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(plan.name)],
            ),
            list::root(
                ListType::Unordered,
                ListVariant::Plain,
                vec![("data-blocks-pricing-tiers-comparison-features", "")],
                items,
            ),
        ],
    )
}

/// 狭幅（`< 48rem`）で比較表の代わりに表示するプランごとの一覧
/// （[`LAYOUT_CSS`] 参照）。
fn narrow_lists() -> Node {
    div(
        vec![("data-blocks-pricing-tiers-comparison-view", "list")],
        PLANS
            .iter()
            .enumerate()
            .map(|(i, plan)| plan_list(i, plan))
            .collect(),
    )
}

/// `pricing-tiers-comparison` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。header → プランカード 3 枚 → 幅広比較表 → 狭幅一覧 の順に
/// 並べる（表/一覧の表示切替は [`LAYOUT_CSS`] の `@media` が担う）。
pub fn demo() -> Node {
    let cards = div(
        vec![("data-blocks-pricing-tiers-comparison-grid", "")],
        PLANS.iter().map(plan_card).collect(),
    );

    div(
        vec![("class", "blocks-pricing-tiers-comparison-layout")],
        vec![header(), cards, comparison_table(), narrow_lists()],
    )
}
```

## 差分メモ

- **既存の `pricing-comparison-table` との主な差**: `pricing-comparison-table`
  はカテゴリ見出し行付きの幅広表と狭幅カード・select 2 列表を持つのに
  対し、本 block はカードと表の 2 段構成で、狭幅では表の代わりに
  「プランごとの機能一覧」を表示する点が異なります。
- カードと表の間の顧客ロゴ列、暗色帯 + 周期切替（`tabs`）のバリエーション
  と状態の並記は後半（#2874）で追加します。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Badge](../themes/badge.md) / [Card](../themes/card.md) /
[Button](../themes/button.md) / [List](../themes/list.md) /
[Icon](../themes/icon.md) / [Table](../themes/table.md)
