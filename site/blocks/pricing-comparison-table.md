# pricing-comparison-table

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `table` / `button` /
`icon` / `card` / `native-select` の 7 部品を合成した、カテゴリ見出し行付き
のプラン比較表です。主参照は対応表 ID R0601 です。Blocks セクションは新規
部品を追加するものではなく、既存の Themes 部品を組み合わせた実例集で
あることに注意してください。

幅の広い画面ではプラン列 × 機能行の比較表（カテゴリごとに見出し行で区切り
ます）を表示します。48rem 未満の狭い画面では、この表の代わりに「プラン
ごとのカード」と「native-select で 1 プランを選ぶ 2 列表」の両方を縦に
並べて表示します。後者は select の選択状態が異なる 2 例（初期状態と切替後
の例）を並記した静的表示であり、無 JS のため select を操作しても表・CTA
は連動しません（連動の配線は利用者の Rust/wasm コードの責務です）。

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
use fandhe_frontend_pre_styled_ui::native_select::{self, FieldIds, FieldProps, NativeSelectProps};
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

/// カテゴリ見出し行（`colspan` でプラン列をまたぐ 1 セル行）。`colspan` を
/// 引数化し、幅広表（[`COL_COUNT_STR`]）と select 2 列表（`"2"`）の双方で
/// 共用する（最小差分の方針、モジュール doc 参照）。
fn category_row(category: &Category, colspan: &str) -> Node {
    table::row(
        vec![],
        vec![table::row_header(
            vec![
                ("colspan", colspan),
                ("data-blocks-pricing-comparison-table-category", ""),
            ],
            vec![text(category.label)],
        )],
    )
}

/// 機能 1 件分の値表現（可否アイコン or 文字列）を組み立てる。幅広表・
/// select 2 列表の双方から呼ばれる共通ヘルパ（最小差分の方針）。
fn value_node(value: &FeatureValue) -> Node {
    match value {
        FeatureValue::Text(label) => span(
            vec![("data-blocks-pricing-comparison-table-value", "text")],
            vec![text(*label)],
        ),
        included_or_excluded => feature_value_icon(included_or_excluded),
    }
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
        cells.push(table::cell(
            vec![("data-blocks-pricing-comparison-table-col", col)],
            vec![value_node(value)],
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
            let mut rows = vec![category_row(category, COL_COUNT_STR)];
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
fn plan_card(plan_index: usize, plan: &Plan) -> Node {
    let cta_variant = if plan.featured {
        ButtonVariant::Solid
    } else {
        ButtonVariant::Outline
    };

    let mut body_children: Vec<Node> = Vec::new();
    for category in CATEGORIES.iter() {
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
                let value = &row.values[plan_index];
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
        body_children.push(el(
            "ul",
            vec![("class", "blocks-pricing-comparison-table-card-list")],
            items,
        ));
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
        PLANS
            .iter()
            .enumerate()
            .map(|(i, plan)| plan_card(i, plan))
            .collect(),
    )
}

/// select 2 列表インスタンスの id（`<label for>` と select 実 `id` を同じ
/// リテラルで揃えるための固定値。モジュール doc「select のラベルと id
/// 一意性」節参照）。
const SELECT_ID_GROWTH: &str = "blocks-pricing-comparison-table-plan-select-growth";
const SELECT_ID_SCALE: &str = "blocks-pricing-comparison-table-plan-select-scale";

/// select 列見出し（`<th scope="col">`）。プラン名・価格を縦に並べる
/// （[`plan_column_header`] と異なり CTA は列見出しへ含めない。CTA は
/// [`select_view`] が表の外へ独立して配置する）。
fn select_column_header(plan: &Plan) -> Node {
    table::column_header(
        vec![],
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
        ],
    )
}

/// 狭幅表示（R0201）: native-select で 1 プランを選び、機能一覧を「機能 /
/// 値」の 2 列表で示す。`selected` は [`PLANS`] の添字（状態違いの並記は
/// [`narrow_view`] が呼び出しを 2 回行うことで表現する）。`select_id` は
/// `<label for>` と select の実 `id` を揃えるための固定リテラル
/// （[`SELECT_ID_GROWTH`]/[`SELECT_ID_SCALE`]）。無 JS のため select を
/// 操作しても表・CTA は連動しない（利用者コードの責務、モジュール doc
/// 参照）。
fn select_view(selected: usize, select_id: &'static str) -> Node {
    let plan = &PLANS[selected];
    let field = FieldProps {
        id: select_id,
        ids: FieldIds {
            control: Some(select_id),
            ..FieldIds::default()
        },
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    let options: Vec<Node> = PLANS
        .iter()
        .enumerate()
        .map(|(i, plan)| {
            let mut attrs: Vec<(&str, &str)> = vec![("value", plan.name)];
            if i == selected {
                attrs.push(("selected", ""));
            }
            el("option", attrs, vec![text(plan.name)])
        })
        .collect();

    let select_bar = div(
        vec![("class", "blocks-pricing-comparison-table-select-bar")],
        vec![
            el(
                "label",
                vec![("for", select_id)],
                vec![text("プランを選択")],
            ),
            native_select::native_select(&NativeSelectProps::default(), &field, vec![], options),
        ],
    );

    let bodies: Vec<Node> = CATEGORIES
        .iter()
        .map(|category| {
            let mut rows = vec![category_row(category, "2")];
            rows.extend(category.rows.iter().map(|row| {
                table::row(
                    vec![],
                    vec![
                        table::row_header(
                            vec![("data-blocks-pricing-comparison-table-feature", "")],
                            vec![text(row.label)],
                        ),
                        table::cell(vec![], vec![value_node(&row.values[selected])]),
                    ],
                )
            }));
            table::body(vec![], rows)
        })
        .collect();

    let mut table_children = vec![
        table::caption(vec![], vec![text(format!("{}の機能一覧", plan.name))]),
        table::header(
            vec![],
            vec![table::row(
                vec![],
                vec![
                    table::column_header(vec![], vec![text("機能")]),
                    select_column_header(plan),
                ],
            )],
        ),
    ];
    table_children.extend(bodies);

    let cta_variant = if plan.featured {
        ButtonVariant::Solid
    } else {
        ButtonVariant::Outline
    };

    div(
        vec![("data-blocks-pricing-comparison-table-view", "select")],
        vec![
            select_bar,
            table::root(
                TableProps {
                    variant: TableVariant::Outline,
                    ..TableProps::default()
                },
                vec![],
                table_children,
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
    )
}

/// 狭幅（48rem 未満）で表の代わりに置く 2 領域: 表示 A（[`cards_view`]、
/// R1150）と表示 B（[`select_view`]、R0201）。表示 B は選択状態違い
/// （Growth を選択＝初期状態 / Scale を選択）を 2 インスタンス並べる
/// （モジュール doc「狭幅表示 A/B と状態の並記」節参照）。
fn narrow_view() -> Node {
    let section_label = |label: &'static str| {
        styled_text::text(
            &TextProps {
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text(label)],
        )
    };

    div(
        vec![("class", "blocks-pricing-comparison-table-narrow")],
        vec![
            div(
                vec![],
                vec![section_label("表示 A: プラン別カード"), cards_view()],
            ),
            div(
                vec![],
                vec![
                    section_label("表示 B-1: Growth を選択（初期状態）"),
                    select_view(1, SELECT_ID_GROWTH),
                ],
            ),
            div(
                vec![],
                vec![
                    section_label("表示 B-2: Scale を選択"),
                    select_view(2, SELECT_ID_SCALE),
                ],
            ),
        ],
    )
}

/// `pricing-comparison-table` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。intro 領域 + 幅広表 + 狭幅表示（表示 A/B 状態違い並記）の
/// 3 領域を縦に並べる（表示切替は [`LAYOUT_CSS`] の `@media` が担う）。
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
        vec![intro, table_view(), narrow_view()],
    )
}
```

## 原案差分メモ

対応表 ID R0601（幅広の比較表）・R1150（狭幅ティアカード）・R0201（狭幅
select 切替）を集約する際、原案から次の差分を設けています。

- 狭幅（48rem 未満）は R1150 と R0201 を切替式にせず、両方を縦に並記しています。
- R0201 の select 状態は、初期状態（Growth 選択）と切替後の例（Scale 選択）の
  2 例を並記しています。
- select の可視ラベルは `field::label` ではなく素の `<label for>` を使い、
  `id`/`for` を明示的な固定値で一致させています。
- select の `id` はインスタンスごとに一意な値にしています。
- 可否アイコンは自作の抽象図形（チェック/バツ）で、参照元のアイコン形状は
  持ち込んでいません。
- CTA ボタンはすべて `type="button"` で、送信先や `href` を持ちません。
- 見出しレベルは `H3`/`H4` を使い、ページ本文の `H2` を飛ばさないようにして
  います。
- 配色・余白はすべて既存の `--fandhe-color-*`/`--fandhe-space-*` トークンの
  みを使っています。
- プラン名・価格・機能名・文言はすべて架空のものです。
