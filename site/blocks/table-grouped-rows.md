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

/// データ行 1 件（取引先・担当・状態・金額の 4 セル）。`headers` に
/// 「対応列見出し `id` + 所属グループ見出し `id`」を空白区切りで列挙し、
/// 各セルを両方の見出しへ明示的に関連付ける（codex-review P2 是正、モジュール
/// doc「グループ見出しとデータ行を `headers` 属性で明示的に関連付ける」節
/// 参照）。
fn data_row(row: &Row, prefix: &str, group_id: &str) -> Node {
    let headers = |col: usize| format!("{prefix}-col-{col} {group_id}");
    let h: Vec<String> = (0..4).map(headers).collect();
    table::row(
        vec![],
        vec![
            table::cell(vec![("headers", h[0].as_str())], vec![text(row.company)]),
            table::cell(vec![("headers", h[1].as_str())], vec![text(row.owner)]),
            table::cell(
                vec![("headers", h[2].as_str())],
                vec![status_badge(row.status, row.status_variant)],
            ),
            table::cell(
                vec![("data-align", "end"), ("headers", h[3].as_str())],
                vec![text(row.amount)],
            ),
        ],
    )
}

/// グループ見出し行（`th scope="row" colspan="4" id="<group_id>"`）。`id` は
/// 同一グループのデータ行 `headers` から参照される（codex-review P2 是正）。
fn group_row(label: &str, group_id: &str) -> Node {
    table::row(
        vec![("data-blocks-table-grouped-rows-group", "")],
        vec![table::row_header(
            vec![("colspan", "4"), ("id", group_id)],
            vec![text(label.to_string())],
        )],
    )
}

/// 列見出し行。各 `th` に `id="<prefix>-col-<i>"` を付け、データセルの
/// `headers` から参照できるようにする（codex-review P2 是正）。`hidden` の
/// とき各ラベルを `visually_hidden::root` で包み、`th` 自体は DOM に残した
/// まま可視領域からは [`LAYOUT_CSS`] が箱を潰す（モジュール doc「版 B」節
/// 参照）。
fn column_headers(hidden: bool, prefix: &str) -> Node {
    let cells: Vec<Node> = COLUMN_LABELS
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let id = format!("{prefix}-col-{i}");
            let mut attrs: Vec<(&str, &str)> = vec![("id", id.as_str())];
            if i == 3 {
                attrs.push(("data-align", "end"));
            }
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

/// グループ分けされた `table` 1 本を組み立てる。`heading_id` は
/// `aria-labelledby` で参照する見出し `id`（モジュール doc
/// 「`caption`/`aria-labelledby` によるテーブル命名」節参照）。`id_prefix`
/// は列見出し・グループ見出しの `id`、`headers` 属性の接頭辞（テーブルごとに
/// 一意な値を渡し、demo 全体で `id` が重複しないようにする、codex-review P2
/// 是正）。
///
/// グループごとに独立した `tbody` を発行し、グループ見出し行をその先頭行として
/// 同居させる（モジュール doc「グループ見出し行を `row_header`
/// （`scope="row"`）+ グループ単位 `tbody` で表す」節参照、codex-review P1
/// 是正）。
fn grouped_table(
    hidden_head: bool,
    heading_id: &'static str,
    id_prefix: &str,
    groups: &[(&str, &[Row])],
) -> Node {
    let mut sections: Vec<Node> = vec![table::header(
        vec![],
        vec![column_headers(hidden_head, id_prefix)],
    )];
    for (n, (label, rows)) in groups.iter().enumerate() {
        let group_id = format!("{id_prefix}-g{}", n + 1);
        let mut body_rows = vec![group_row(label, &group_id)];
        for row in *rows {
            body_rows.push(data_row(row, id_prefix, &group_id));
        }
        sections.push(table::body(vec![], body_rows));
    }
    let mut table_attrs: Vec<(&str, &str)> = vec![
        ("data-blocks-table-grouped-rows-table", ""),
        ("aria-labelledby", heading_id),
    ];
    if hidden_head {
        table_attrs.push(("data-blocks-table-grouped-rows-hidden-head", ""));
    }
    table::root(TableProps::default(), table_attrs, sections)
}

/// A: 列見出し可視 + 地域グループ（R1332・代表構成）。
fn version_regions() -> Node {
    const HEADING_ID: &str = "blocks-table-grouped-rows-regions-heading";
    div(
        vec![("class", "blocks-table-grouped-rows-section")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![("id", HEADING_ID)],
                vec![text("地域別の受注一覧")],
            ),
            grouped_table(
                false,
                HEADING_ID,
                "blocks-table-grouped-rows-regions",
                GROUPS_A,
            ),
        ],
    )
}

/// B: 列見出し非表示 + 日付グループ（R1336・集約元）。
fn version_dates_hidden_head() -> Node {
    const HEADING_ID: &str = "blocks-table-grouped-rows-dates-heading";
    div(
        vec![("class", "blocks-table-grouped-rows-section")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![("id", HEADING_ID)],
                vec![text("日付別の入金一覧")],
            ),
            grouped_table(
                true,
                HEADING_ID,
                "blocks-table-grouped-rows-dates",
                GROUPS_B,
            ),
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
  ています。行グループ全体へ及ぶ `scope="rowgroup"` は現状の API では表現
  できませんが、各データセルへ `headers` 属性（列見出し `id` + グループ
  見出し `id`）を付与して明示的に関連付けているため、支援技術は各セルが
  どの列・どのグループに属するかを判別できます。
- 金額は右揃え（`data-align="end"`）の固定文字列、状態は `badge` で表示
  します。
- 狭幅（コンテナ幅 36rem 未満）ではテーブルが横スクロール可能なブロックへ
  切り替わります（`@container` によるコンテナクエリ判定）。

関連情報: [Table](../themes/table.md) / [Badge](../themes/badge.md) /
[Visually Hidden](../themes/visually-hidden.md) / [Heading](../themes/heading.md)
