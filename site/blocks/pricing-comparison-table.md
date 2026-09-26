# pricing-comparison-table

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `table` / `button` /
`icon` / `card` の 6 部品を合成した、カテゴリ見出し行付きのプラン比較表です。
主参照は対応表 ID R0601 です。Blocks セクションは新規部品を追加するもので
はなく、既存の Themes 部品を組み合わせた実例集であることに注意してください。

幅の広い画面ではプラン列 × 機能行の比較表（カテゴリごとに見出し行で区切り
ます）を表示し、48rem 未満の狭い画面ではプランごとのカードへ切り替わります。
本 Demo は静的な表示例です。`<form>` は出力せず、送信・決済処理を持ちません。
プラン名・機能名・価格はすべて架空のものであり、実在の企業名・PII は含み
ません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 表の列数（機能名列 + プラン数）を文字列表記した定数。プラン数が
/// [`PLANS`] 固定のため、`colspan` 属性値としてリテラルで持つ（ファイル内
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

/// 比較表の列見出しに置くプラン 1 件分（架空データ）。
struct Plan {
    name: &'static str,
    price: &'static str,
    cta_label: &'static str,
    /// 強調表示するプランかどうか（`accent-subtle` 背景、[`LAYOUT_CSS`]）。
    featured: bool,
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

/// プラン列 3 件。価格は `crate::blocks::dummy_assets::SAMPLE_PRICE_TIERS`
/// （Starter $9 / Growth $29 / Scale $79）と同じ値を直書きする（`const` の
/// 配列添字アクセスより明快なため）。
const PLANS: [Plan; 3] = [
    Plan {
        name: "Starter",
        price: "$9",
        cta_label: "Starter を選ぶ",
        featured: false,
    },
    Plan {
        name: "Growth",
        price: "$29",
        cta_label: "Growth を選ぶ",
        featured: true,
    },
    Plan {
        name: "Scale",
        price: "$79",
        cta_label: "Scale を選ぶ",
        featured: false,
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
fn feature_value_icon(value: &FeatureValue) -> Node {
    match value {
        FeatureValue::Included => icon(
            &IconProps {
                size: Size::Sm,
                label: Some("含まれる"),
                ..IconProps::default()
            },
            vec![("data-blocks-pricing-comparison-table-value", "included")],
            vec![
                el(
                    "circle",
                    vec![
                        ("cx", "12"),
                        ("cy", "12"),
                        ("r", "9"),
                        ("fill", "none"),
                        ("stroke", "currentColor"),
                        ("stroke-width", "1.5"),
                    ],
                    vec![],
                ),
                el(
                    "path",
                    vec![
                        ("d", "M8 12.5l2.5 2.5L16 9"),
                        ("fill", "none"),
                        ("stroke", "currentColor"),
                        ("stroke-width", "1.5"),
                        ("stroke-linecap", "round"),
                        ("stroke-linejoin", "round"),
                    ],
                    vec![],
                ),
            ],
        ),
        FeatureValue::Excluded => icon(
            &IconProps {
                size: Size::Sm,
                label: Some("含まれない"),
                ..IconProps::default()
            },
            vec![("data-blocks-pricing-comparison-table-value", "excluded")],
            vec![
                el(
                    "path",
                    vec![
                        ("d", "M8 8l8 8"),
                        ("stroke", "currentColor"),
                        ("stroke-width", "1.5"),
                        ("stroke-linecap", "round"),
                    ],
                    vec![],
                ),
                el(
                    "path",
                    vec![
                        ("d", "M16 8l-8 8"),
                        ("stroke", "currentColor"),
                        ("stroke-width", "1.5"),
                        ("stroke-linecap", "round"),
                    ],
                    vec![],
                ),
            ],
        ),
        FeatureValue::Text(_) => span(vec![], vec![]),
    }
}

/// プラン 1 列分の列見出し（`<th scope="col">`）。プラン名・価格・CTA を
/// 縦に並べる（親 issue のレイアウト仕様）。
fn plan_column_header(plan: &Plan) -> Node {
    let col = if plan.featured {
        "featured"
    } else {
        "standard"
    };
    let cta_variant = if plan.featured {
        ButtonVariant::Solid
    } else {
        ButtonVariant::Outline
    };
    table::column_header(
        vec![("data-blocks-pricing-comparison-table-col", col)],
        vec![div(
            vec![("class", "blocks-pricing-comparison-table-plan-head")],
            vec![
                span(vec![], vec![text(plan.name)]),
                styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![("data-blocks-pricing-comparison-table-price", "")],
                    vec![text(plan.price)],
                ),
                button::button(
                    &ButtonProps {
                        variant: cta_variant,
                        ..ButtonProps::default()
                    },
                    vec![("data-blocks-pricing-comparison-table-cta", "")],
                    vec![text(plan.cta_label)],
                ),
            ],
        )],
    )
}

/// カテゴリ見出し行（`colspan` でプラン列をまたぐ 1 セル行）。
fn category_row(category: &Category) -> Node {
    table::row(
        vec![],
        vec![table::row_header(
            vec![
                ("colspan", COL_COUNT_STR),
                ("data-blocks-pricing-comparison-table-category", ""),
            ],
            vec![text(category.label)],
        )],
    )
}

/// 機能比較 1 行分（`table::row`）。
fn feature_table_row(row: &FeatureRow) -> Node {
    let mut cells: Vec<Node> = vec![table::row_header(
        vec![("data-blocks-pricing-comparison-table-feature", "")],
        vec![text(row.label)],
    )];
    for (value, plan) in row.values.iter().zip(PLANS.iter()) {
        let col = if plan.featured {
            "featured"
        } else {
            "standard"
        };
        let value_node = match value {
            FeatureValue::Text(label) => span(
                vec![("data-blocks-pricing-comparison-table-value", "text")],
                vec![text(*label)],
            ),
            included_or_excluded => feature_value_icon(included_or_excluded),
        };
        cells.push(table::cell(
            vec![("data-blocks-pricing-comparison-table-col", col)],
            vec![value_node],
        ));
    }
    table::row(vec![], cells)
}

/// 幅広表示（R0601）: カテゴリ見出し行付きの比較表。
fn table_view() -> Node {
    let mut header_cells: Vec<Node> = vec![table::column_header(vec![], vec![text("機能")])];
    header_cells.extend(PLANS.iter().map(plan_column_header));

    let bodies: Vec<Node> = CATEGORIES
        .iter()
        .map(|category| {
            let mut rows = vec![category_row(category)];
            rows.extend(category.rows.iter().map(feature_table_row));
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
        vec![],
        table_children,
    );

    div(
        vec![("data-blocks-pricing-comparison-table-view", "table")],
        vec![table::scroll_area(
            vec![
                ("role", "region"),
                ("aria-label", "プラン比較表"),
                ("tabindex", "0"),
            ],
            vec![table_node],
        )],
    )
}

/// 狭幅表示（R1150）: プランごとのカード。カテゴリごとに小見出し（`H4`）+
/// 機能一覧を並べる。
fn plan_card(plan: &Plan) -> Node {
    let cta_variant = if plan.featured {
        ButtonVariant::Solid
    } else {
        ButtonVariant::Outline
    };

    let mut body_children: Vec<Node> = Vec::new();
    for (i, category) in CATEGORIES.iter().enumerate() {
        body_children.push(heading(
            HeadingLevel::H4,
            &HeadingProps {
                size: HeadingSize::Sm,
                weight: HeadingWeight::Bold,
            },
            vec![],
            vec![text(category.label)],
        ));
        let items: Vec<Node> = category
            .rows
            .iter()
            .map(|row| {
                let value = &row.values[i];
                let value_node = match value {
                    FeatureValue::Text(label) => span(
                        vec![("data-blocks-pricing-comparison-table-value", "text")],
                        vec![text(format!("{}: {label}", row.label))],
                    ),
                    included_or_excluded => div(
                        vec![("class", "blocks-pricing-comparison-table-card-item")],
                        vec![feature_value_icon(included_or_excluded), text(row.label)],
                    ),
                };
                el("li", vec![], vec![value_node])
            })
            .collect();
        body_children.push(el("ul", vec![], items));
    }

    card::root(
        CardProps {
            variant: CardVariant::Outline,
            ..CardProps::default()
        },
        vec![("data-blocks-pricing-comparison-table-card", "")],
        vec![
            card::header(
                vec![],
                vec![
                    card::title(vec![], vec![text(plan.name)]),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![("data-blocks-pricing-comparison-table-price", "")],
                        vec![text(plan.price)],
                    ),
                ],
            ),
            card::body(vec![], body_children),
            card::footer(
                vec![],
                vec![button::button(
                    &ButtonProps {
                        variant: cta_variant,
                        ..ButtonProps::default()
                    },
                    vec![("data-blocks-pricing-comparison-table-cta", "")],
                    vec![text(plan.cta_label)],
                )],
            ),
        ],
    )
}

/// 狭幅表示（R1150）本体。プラン数分のカードを並べる。
fn cards_view() -> Node {
    div(
        vec![("data-blocks-pricing-comparison-table-view", "cards")],
        PLANS.iter().map(plan_card).collect(),
    )
}

/// `pricing-comparison-table` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。intro 領域 + 幅広表 + 狭幅カードの 3 領域を縦に並べる（表示
/// 切替は [`LAYOUT_CSS`] の `@media` が担う）。
pub fn demo() -> Node {
    let intro = div(
        vec![("class", "blocks-pricing-comparison-table-intro")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("プランを比較する")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-pricing-comparison-table-lead", "")],
                vec![text("機能ごとの違いを一目で確認できます。")],
            ),
        ],
    );

    div(
        vec![("class", "blocks-pricing-comparison-table-layout")],
        vec![intro, table_view(), cards_view()],
    )
}
```
