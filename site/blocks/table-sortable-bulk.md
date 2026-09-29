# table-sortable-bulk

`data-table` / `table` / `checkbox` / `button` / `icon` の 5 部品を合成
した、列見出しに並び替え印を付け先頭列に行選択チェックボックスを置く
データテーブルの合成例です。Blocks セクションは新規部品を追加するもの
ではなく、既存の Themes/Primitives 部品を組み合わせた実例集であること
に注意してください。

1 行以上選択された状態では、見出し行の上に一括操作ツールバー（選択件数
+ アーカイブ/削除ボタン）を重ねて表示します。docs サイトは JS ハイド
レーションを行わないため、「未選択」（0 行選択・ツールバーなし）と
「選択中」（4 行中 3 行選択・ツールバー表示中）の 2 状態を静的に併記し
ます。狭いコンテナ幅（40rem 以下）では役割・最終更新の 2 列を隠し、
ツールバーは全幅帯として残ります。

並び替え・行選択・一括操作（アーカイブ/削除）の実処理は行いません。
`<form>` 要素は持たず、ボタンはすべて `type="button"` で送信先を持たない
静的表示です。行データ（氏名・役職）は架空の人名セットから引用し、
ステータス・日付・担当は独自に書いた架空の文言です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps};
use fandhe_frontend_pre_styled_ui::data_table::{
    self, ColumnHeaderProps, ColumnProps, DataTable, DataTableProps, SortDirection,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 行選択チェックボックス（`crate::showcase::data_table_section` の
/// `row_select_checkbox` と同型）。`name` は panel・行ごとに一意にし
/// （選択中/未選択パネル双方を同じページへ静的併記するため）、
/// `demo_output_has_no_dangling_aria_references_or_duplicate_ids`
/// （`crates/docs-site/tests/blocks_contract.rs`）の id 重複検知に抵触
/// しないよう `id` 属性自体を持たない（`aria-label` のみで名前付け）。
fn row_select_checkbox(name: &str, checked: checkbox::CheckedState, label: &str) -> Node {
    let props = CheckboxProps {
        checked,
        ..CheckboxProps::default()
    };
    checkbox::root(
        Size::Sm,
        ColorPalette::Accent,
        &props,
        vec![],
        vec![
            checkbox::hidden_input(&props, name, "on", vec![("aria-label", label)]),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
        ],
    )
}

/// アーカイブボタンの自作アイコン（トレイ + 下矢印の抽象図形。参照元の
/// アイコンは持ち込まず単純な幾何図形とする、`stats_row`/
/// `error_page_popular_links` と同型の判断）。
fn archive_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![
            el(
                "path",
                vec![
                    ("d", "M3 5h18v4H3z"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linejoin", "round"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "M5 9v9a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V9"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linecap", "round"),
                    ("stroke-linejoin", "round"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "M10 13h4"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linecap", "round"),
                ],
                vec![],
            ),
        ],
    )
}

/// 削除ボタンの自作アイコン（ゴミ箱の抽象図形）。
fn delete_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![
            el(
                "path",
                vec![
                    ("d", "M4 7h16"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linecap", "round"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "M9 7V4h6v3"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linejoin", "round"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "M6 7l1 13h10l1-13"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "2"),
                    ("stroke-linejoin", "round"),
                ],
                vec![],
            ),
        ],
    )
}

/// 行 1 件分のダミーデータ（架空・`dummy_assets` 由来）。
struct RowData {
    name: &'static str,
    status: &'static str,
    role: &'static str,
    updated: (&'static str, &'static str),
    assignee: &'static str,
}

/// 4 行分の共通ダミーデータ（「未選択」「選択中」両パネルで共有する）。
/// 名前列を `aria-sort="ascending"` 表示するため、行の並びも名前の
/// 昇順（Elena, Haruto, Kwame, Mei）にする（表示順とソート表示の矛盾を
/// 防ぐ。Codex レビュー指摘、イシュー #2945 PR #3393）。
fn rows_data() -> [RowData; 4] {
    [
        RowData {
            name: dummy_assets::PERSON_NAMES[1],
            status: "Pending",
            role: dummy_assets::JOB_TITLES[1],
            updated: ("2026-09-18", "Sep 18, 2026"),
            assignee: dummy_assets::PERSON_NAMES[5],
        },
        RowData {
            name: dummy_assets::PERSON_NAMES[0],
            status: "Active",
            role: dummy_assets::JOB_TITLES[0],
            updated: ("2026-09-20", "Sep 20, 2026"),
            assignee: dummy_assets::PERSON_NAMES[4],
        },
        RowData {
            name: dummy_assets::PERSON_NAMES[2],
            status: "Active",
            role: dummy_assets::JOB_TITLES[2],
            updated: ("2026-09-15", "Sep 15, 2026"),
            assignee: dummy_assets::PERSON_NAMES[6],
        },
        RowData {
            name: dummy_assets::PERSON_NAMES[3],
            status: "Archived",
            role: dummy_assets::JOB_TITLES[3],
            updated: ("2026-09-02", "Sep 2, 2026"),
            assignee: dummy_assets::PERSON_NAMES[7],
        },
    ]
}

/// 1 パネル分（「未選択」または「選択中」）を組み立てる。
///
/// `variant` は `data-blocks-table-sortable-bulk-variant` の値
/// （`"none"`/`"selected"`）、`selected_count` は選択中行数（0 なら
/// ツールバーを出力しない）。
fn panel(variant: &'static str, selected_count: usize) -> Node {
    let rows = rows_data();
    let table_state = DataTable::new(Some(("name".to_string(), SortDirection::Ascending)), vec![]);

    let name_column = ColumnProps {
        id: "name",
        hidden: false,
    };
    let status_column = ColumnProps {
        id: "status",
        hidden: false,
    };
    let role_column = ColumnProps {
        id: "role",
        hidden: false,
    };
    let updated_column = ColumnProps {
        id: "updated",
        hidden: false,
    };
    let assignee_column = ColumnProps {
        id: "assignee",
        hidden: false,
    };

    let secondary_attr = ("data-blocks-table-sortable-bulk-secondary", "");

    let header_row = table::row(
        vec![],
        vec![
            table::column_header(
                vec![("scope", "col")],
                vec![row_select_checkbox(
                    match variant {
                        "selected" => "select-selected-all",
                        _ => "select-none-all",
                    },
                    match selected_count {
                        0 => checkbox::CheckedState::Unchecked,
                        n if n >= rows.len() => checkbox::CheckedState::Checked,
                        _ => checkbox::CheckedState::Indeterminate,
                    },
                    "Select all rows",
                )],
            ),
            table::column_header(
                data_table::column_header_attrs(&ColumnHeaderProps {
                    column: name_column,
                    sort: Some(table_state.sort_direction_of("name").unwrap()),
                }),
                vec![data_table::sort_trigger(
                    &table_state,
                    "name",
                    vec![],
                    vec![text("名前")],
                )],
            ),
            table::column_header(
                data_table::column_header_attrs(&ColumnHeaderProps {
                    column: status_column,
                    sort: Some(
                        table_state
                            .sort_direction_of("status")
                            .unwrap_or(SortDirection::None),
                    ),
                }),
                vec![data_table::sort_trigger(
                    &table_state,
                    "status",
                    vec![],
                    vec![text("ステータス")],
                )],
            ),
            table::column_header(
                {
                    let mut attrs = data_table::column_header_attrs(&ColumnHeaderProps {
                        column: role_column,
                        sort: None,
                    });
                    attrs.push(secondary_attr);
                    attrs
                },
                vec![text("役割")],
            ),
            table::column_header(
                {
                    let mut attrs = data_table::column_header_attrs(&ColumnHeaderProps {
                        column: updated_column,
                        sort: None,
                    });
                    attrs.push(secondary_attr);
                    attrs
                },
                vec![text("最終更新")],
            ),
            table::column_header(
                data_table::column_header_attrs(&ColumnHeaderProps {
                    column: assignee_column,
                    sort: None,
                }),
                vec![text("担当")],
            ),
        ],
    );

    let mut body_rows = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        let is_selected = variant == "selected" && index < selected_count;
        let row_checkbox_name = format!("select-{variant}-row-{index}");
        body_rows.push(table::row(
            data_table::row_attrs(is_selected),
            vec![
                table::cell(
                    vec![],
                    vec![row_select_checkbox(
                        &row_checkbox_name,
                        if is_selected {
                            checkbox::CheckedState::Checked
                        } else {
                            checkbox::CheckedState::Unchecked
                        },
                        &format!("Select row: {}", row.name),
                    )],
                ),
                table::cell(data_table::column_attrs(&name_column), vec![text(row.name)]),
                table::cell(
                    data_table::column_attrs(&status_column),
                    vec![text(row.status)],
                ),
                table::cell(
                    {
                        let mut attrs = data_table::column_attrs(&role_column);
                        attrs.push(secondary_attr);
                        attrs
                    },
                    vec![text(row.role)],
                ),
                table::cell(
                    {
                        let mut attrs = data_table::column_attrs(&updated_column);
                        attrs.push(secondary_attr);
                        attrs
                    },
                    vec![el(
                        "time",
                        vec![("datetime", row.updated.0)],
                        vec![text(row.updated.1)],
                    )],
                ),
                table::cell(
                    data_table::column_attrs(&assignee_column),
                    vec![text(row.assignee)],
                ),
            ],
        ));
    }

    let mut wrap_children = Vec::new();
    if selected_count > 0 {
        wrap_children.push(data_table::toolbar(
            vec![("data-blocks-table-sortable-bulk-toolbar", "")],
            vec![
                data_table::selection_count(
                    vec![],
                    vec![text(format!("{selected_count} 件選択中"))],
                ),
                button::button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        size: Size::Sm,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![archive_icon(), text("アーカイブ")],
                ),
                button::button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        size: Size::Sm,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![delete_icon(), text("削除")],
                ),
            ],
        ));
    }
    wrap_children.push(table::root(
        TableProps {
            interactive: true,
            ..TableProps::default()
        },
        vec![],
        vec![
            table::header(vec![], vec![header_row]),
            table::body(vec![], body_rows),
        ],
    ));

    data_table::root(
        DataTableProps::default(),
        vec![("data-blocks-table-sortable-bulk-variant", variant)],
        vec![div(
            vec![("class", "blocks-table-sortable-bulk-table-wrap")],
            wrap_children,
        )],
    )
}

/// `table-sortable-bulk` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。「未選択」（R1331 相当・0 行選択）と「選択中」（R1335 相当・
/// 4 行中 3 行選択）の 2 パネルを縦に併記する。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-table-sortable-bulk-layout")],
        vec![
            div(
                vec![("class", "blocks-table-sortable-bulk-panel")],
                vec![panel("none", 0)],
            ),
            div(
                vec![("class", "blocks-table-sortable-bulk-panel")],
                vec![panel("selected", 3)],
            ),
        ],
    )
}
```

## 原案差分メモ

- 「未選択」インスタンスは対応表 ID R1331（ソート可能な列見出しのみの
  構成）へ、「選択中」インスタンスは主参照 R1335（代表構成。一括操作
  ツールバーの重ね表示を含む）へ対応します。
- 狭幅での副次列非表示・ツールバーの全幅帯化は `@container` による CSS
  のみで表現し、専用の JS 配線は行いません。
- 実機ブラウザでのコンテナ幅切替・ライト/ダークテーマ両対応の目視確認は
  サンドボックス制約により未実施です。
