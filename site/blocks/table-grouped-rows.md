# table-grouped-rows

地域・日付ごとにグループ見出し行を挟んで行をまとめて表示するテーブルです。
`table` / `badge` / `visually-hidden` / `heading` の 4 部品を合成します。
Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R1332（代表構成: 列見出し可視 + 地域グループ）で、R1336
（列見出しを視覚的に隠した版 + 日付グループ）を集約しています。取引先名・
担当者名・金額・日付はすべて架空のデータであり、実在の企業・人物・PII は
含みません。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets::{COMPANY_NAMES, PERSON_NAMES};
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// 1 データ行の固定内容（取引先・担当・状態・金額）。
struct Row {
    company: &'static str,
    owner: &'static str,
    status: &'static str,
    status_variant: BadgeVariant,
    amount: &'static str,
}

/// 版 A（R1332・地域別）: 3 グループ。
const GROUPS_A: &[(&str, &[Row])] = &[
    (
        "関東",
        &[
            Row {
                company: COMPANY_NAMES[0],
                owner: PERSON_NAMES[0],
                status: "完了",
                status_variant: BadgeVariant::Subtle,
                amount: "¥128,400",
            },
            Row {
                company: COMPANY_NAMES[1],
                owner: PERSON_NAMES[1],
                status: "処理中",
                status_variant: BadgeVariant::Outline,
                amount: "¥64,900",
            },
        ],
    ),
    (
        "関西",
        &[
            Row {
                company: COMPANY_NAMES[2],
                owner: PERSON_NAMES[2],
                status: "完了",
                status_variant: BadgeVariant::Subtle,
                amount: "¥212,000",
            },
            Row {
                company: COMPANY_NAMES[3],
                owner: PERSON_NAMES[3],
                status: "保留",
                status_variant: BadgeVariant::Surface,
                amount: "¥37,500",
            },
        ],
    ),
    (
        "九州",
        &[Row {
            company: COMPANY_NAMES[4],
            owner: PERSON_NAMES[4],
            status: "完了",
            status_variant: BadgeVariant::Subtle,
            amount: "¥95,200",
        }],
    ),
];

/// 版 B（R1336・日付別）: 2 グループ。
const GROUPS_B: &[(&str, &[Row])] = &[
    (
        "2026-09-24",
        &[
            Row {
                company: COMPANY_NAMES[5],
                owner: PERSON_NAMES[5],
                status: "完了",
                status_variant: BadgeVariant::Subtle,
                amount: "¥58,300",
            },
            Row {
                company: COMPANY_NAMES[0],
                owner: PERSON_NAMES[6],
                status: "処理中",
                status_variant: BadgeVariant::Outline,
                amount: "¥140,000",
            },
        ],
    ),
    (
        "2026-09-25",
        &[
            Row {
                company: COMPANY_NAMES[1],
                owner: PERSON_NAMES[7],
                status: "保留",
                status_variant: BadgeVariant::Surface,
                amount: "¥21,750",
            },
            Row {
                company: COMPANY_NAMES[2],
                owner: PERSON_NAMES[0],
                status: "完了",
                status_variant: BadgeVariant::Subtle,
                amount: "¥183,600",
            },
        ],
    ),
];

/// 列見出しラベル（共通、順序固定）。
const COLUMN_LABELS: [&str; 4] = ["取引先", "担当", "状態", "金額"];

/// 状態バッジ 1 件。
fn status_badge(label: &'static str, variant: BadgeVariant) -> Node {
    badge(
        &BadgeProps {
            variant,
            ..BadgeProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// データ行 1 件（取引先・担当・状態・金額の 4 セル）。
fn data_row(row: &Row) -> Node {
    table::row(
        vec![],
        vec![
            table::cell(vec![], vec![text(row.company)]),
            table::cell(vec![], vec![text(row.owner)]),
            table::cell(vec![], vec![status_badge(row.status, row.status_variant)]),
            table::cell(vec![("data-align", "end")], vec![text(row.amount)]),
        ],
    )
}

/// グループ見出し行（`th scope="row" colspan="4"`）。
fn group_row(label: &str) -> Node {
    table::row(
        vec![("data-blocks-table-grouped-rows-group", "")],
        vec![table::row_header(
            vec![("colspan", "4")],
            vec![text(label.to_string())],
        )],
    )
}

/// 列見出し行。`hidden` のとき各ラベルを `visually_hidden::root` で包み、
/// `th` 自体は DOM に残したまま可視領域からは [`LAYOUT_CSS`] が箱を潰す
/// （モジュール doc「版 B」節参照）。
fn column_headers(hidden: bool) -> Node {
    let cells: Vec<Node> = COLUMN_LABELS
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let align = if i == 3 {
                Some(("data-align", "end"))
            } else {
                None
            };
            let attrs: Vec<(&str, &str)> = align.into_iter().collect();
            let label_node: Node = text(*label);
            let content = if hidden {
                visually_hidden::root(vec![], vec![label_node])
            } else {
                label_node
            };
            table::column_header(attrs, vec![content])
        })
        .collect();
    table::row(vec![], cells)
}

/// グループ分けされた `table` 1 本を組み立てる。
fn grouped_table(hidden_head: bool, groups: &[(&str, &[Row])]) -> Node {
    let mut body_rows = Vec::new();
    for (label, rows) in groups {
        body_rows.push(group_row(label));
        for row in *rows {
            body_rows.push(data_row(row));
        }
    }
    let mut table_attrs: Vec<(&str, &str)> = vec![("data-blocks-table-grouped-rows-table", "")];
    if hidden_head {
        table_attrs.push(("data-blocks-table-grouped-rows-hidden-head", ""));
    }
    table::root(
        TableProps::default(),
        table_attrs,
        vec![
            table::header(vec![], vec![column_headers(hidden_head)]),
            table::body(vec![], body_rows),
        ],
    )
}

/// A: 列見出し可視 + 地域グループ（R1332・代表構成）。
fn version_regions() -> Node {
    div(
        vec![("class", "blocks-table-grouped-rows-section")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("地域別の受注一覧")],
            ),
            grouped_table(false, GROUPS_A),
        ],
    )
}

/// B: 列見出し非表示 + 日付グループ（R1336・集約元）。
fn version_dates_hidden_head() -> Node {
    div(
        vec![("class", "blocks-table-grouped-rows-section")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("日付別の入金一覧")],
            ),
            grouped_table(true, GROUPS_B),
        ],
    )
}

/// `table-grouped-rows` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-table-grouped-rows-stack")],
        vec![version_regions(), version_dates_hidden_head()],
    )
}
```

## 原案差分メモ

- **版 A（代表構成、R1332）**: 列見出し（取引先・担当・状態・金額）を可視
  のまま表示し、地域（関東・関西・九州）ごとにグループ見出し行
  （`th scope="row" colspan="4"`）で区切ります。
- **版 B（列見出し非表示、R1336）**: 列見出しの中身を `visually-hidden`
  で包み、可視領域からは block 固有 CSS が箱を潰して消します（`th` 自体は
  DOM・アクセシビリティツリーに残るため、スクリーンリーダー利用者は列見出し
  を読めます）。日付（2026-09-24・2026-09-25）ごとのグループ見出し行だけで
  区切ります。
- グループ見出し行の背景色・太字表示、ヘッダー行の見た目の潰しはいずれも
  `table` 部品自体の機能ではなく、本 block の CSS が付与しています。
- グループ見出し行は `table::row_header`（`scope="row"` 固定）で表現し
  ています。列全体へまたがる `scope="colgroup"` は現状の API では表現でき
  ません。
- 金額は右揃え（`data-align="end"`）の固定文字列、状態は `badge` で表示
  します。
- 狭幅（コンテナ幅 36rem 未満）ではテーブルが横スクロール可能なブロックへ
  切り替わります（`@container` によるコンテナクエリ判定）。

関連情報: [Table](../themes/table.md) / [Badge](../themes/badge.md) /
[Visually Hidden](../themes/visually-hidden.md) / [Heading](../themes/heading.md)
