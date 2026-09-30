# settings-notification-matrix

通知の種類（行）× 配信経路（列）の交点を checkbox で表す設定表です。
`checkbox` / `table` / `field` / `button` / `heading` の 5 部品を合成します。
Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0258（単一構成、集約すべき別 version はありません）。
通知種別・配信経路の名称・説明文はすべて架空のもので、実在の人物・企業・
PII は含みません。

幅広（コンテナ幅 40rem 以上）では表形式、狭幅では配信経路ごとの縦並びへ
切り替わります（`@container` によるコンテナクエリ判定）。両方の表示は
同一の初期状態定数から生成されるため、表示方式によるチェック状態のずれは
ありません。checkbox はすべてネイティブ `disabled` で固定した静的表示です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::field::{self, FieldIds, FieldProps, FieldRootProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 配信経路（列）。3 経路固定。
const CHANNELS: [&str; 3] = ["メール", "チャット", "モバイル"];

/// 通知の種類（行）。`(名前, 説明)`。4 種固定。
const NOTIFICATION_TYPES: [(&str, &str); 4] = [
    ("コメント", "投稿へのコメントを通知します"),
    ("メンション", "自分へのメンションを通知します"),
    ("週次ダイジェスト", "1 週間の活動をまとめて通知します"),
    ("セキュリティ通知", "ログイン試行等の重要な通知です（必須）"),
];

/// 初期状態（`[種別][経路]`）。幅広表・狭幅縦並びの両方がこの唯一の定数
/// から生成されるため、表示方式による状態のずれが構造的に生じない。
/// セキュリティ通知（末尾行）は全経路 `true` にして「必須通知」を表す。
const INITIAL_STATE: [[bool; 3]; 4] = [
    [true, false, true],
    [true, true, false],
    [false, false, false],
    [true, true, true],
];

/// checkbox 1 件の `id`（`hidden_input`）を組み立てる。`suffix` は幅広/狭幅
/// の重複回避用（モジュール doc「id の一意化」節参照）。
fn checkbox_id(type_idx: usize, channel_idx: usize, suffix: &str) -> String {
    format!("blocks-settings-notification-matrix-{type_idx}-{channel_idx}-{suffix}")
}

/// 幅広表セル用の checkbox。`checkbox::label` に経路名を入れ、
/// `LAYOUT_CSS` の視覚的隠蔽で見た目は列見出しに委ねる
/// （モジュール doc「表の checkbox は列見出しをアクセシブルネームとして
/// 保つ」節参照）。
fn wide_checkbox(type_idx: usize, channel_idx: usize) -> Node {
    let props = CheckboxProps {
        checked: if INITIAL_STATE[type_idx][channel_idx] {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        },
        disabled: true,
        ..CheckboxProps::default()
    };
    let id = checkbox_id(type_idx, channel_idx, "wide");
    let name = "notification-matrix";
    checkbox::root(
        Size::Md,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-settings-notification-matrix-checkbox", "")],
        vec![
            checkbox::hidden_input(&props, name, "on", vec![("id", id.as_str())]),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
            checkbox::label(
                &props,
                vec![("data-blocks-settings-notification-matrix-cell-label", "")],
                vec![text(CHANNELS[channel_idx])],
            ),
        ],
    )
}

/// 幅広表本体（`table::scroll_area` + `table::root`）。
fn wide_table() -> Node {
    let mut header_cells: Vec<Node> = vec![table::column_header(vec![], vec![text("通知")])];
    header_cells.extend(
        CHANNELS
            .iter()
            .map(|channel| table::column_header(vec![], vec![text(*channel)])),
    );

    let body_rows: Vec<Node> = NOTIFICATION_TYPES
        .iter()
        .enumerate()
        .map(|(type_idx, (name, desc))| {
            let mut cells: Vec<Node> = vec![table::row_header(
                vec![],
                vec![
                    div(
                        vec![("class", "blocks-settings-notification-matrix-type-name")],
                        vec![text(*name)],
                    ),
                    div(
                        vec![("class", "blocks-settings-notification-matrix-type-desc")],
                        vec![text(*desc)],
                    ),
                ],
            )];
            cells.extend((0..CHANNELS.len()).map(|channel_idx| {
                table::cell(
                    vec![("class", "blocks-settings-notification-matrix-cell")],
                    vec![wide_checkbox(type_idx, channel_idx)],
                )
            }));
            table::row(vec![], cells)
        })
        .collect();

    table::scroll_area(
        vec![
            ("data-blocks-settings-notification-matrix-scroll", ""),
            ("role", "region"),
            ("aria-label", "通知設定表"),
            ("tabindex", "0"),
        ],
        vec![table::root(
            TableProps {
                variant: TableVariant::Outline,
                ..TableProps::default()
            },
            vec![("data-blocks-settings-notification-matrix-table", "")],
            vec![
                table::caption(vec![], vec![text("通知の種類と配信経路")]),
                table::header(vec![], vec![table::row(vec![], header_cells)]),
                table::body(vec![], body_rows),
            ],
        )],
    )
}

/// 狭幅縦並び 1 行（`field::root` + checkbox + `field::helper_text`）。
/// `field` の `id` に checkbox の `id` を流用し、`helper_text` の派生 id
/// （`"{id}-helper-text"`）を `hidden_input` の `aria-describedby` へ渡す
/// （`field::helper_text` doc の合成則と同じ結線）。
fn narrow_row(type_idx: usize, channel_idx: usize) -> Node {
    let (name, desc) = NOTIFICATION_TYPES[type_idx];
    let id = checkbox_id(type_idx, channel_idx, "narrow");
    let field_props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: true,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: true,
    };
    let helper_id = format!("{id}-helper-text");
    let checkbox_props = CheckboxProps {
        checked: if INITIAL_STATE[type_idx][channel_idx] {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        },
        disabled: true,
        ..CheckboxProps::default()
    };
    field::root(
        &FieldRootProps::default(),
        &field_props,
        vec![("class", "blocks-settings-notification-matrix-narrow-row")],
        vec![
            checkbox::root(
                Size::Md,
                ColorPalette::Accent,
                &checkbox_props,
                vec![("data-blocks-settings-notification-matrix-checkbox", "")],
                vec![
                    checkbox::hidden_input(
                        &checkbox_props,
                        "notification-matrix",
                        "on",
                        vec![
                            ("id", id.as_str()),
                            ("aria-describedby", helper_id.as_str()),
                        ],
                    ),
                    checkbox::control(
                        &checkbox_props,
                        vec![],
                        vec![checkbox::indicator(&checkbox_props, vec![], vec![])],
                    ),
                    checkbox::label(&checkbox_props, vec![], vec![text(name)]),
                ],
            ),
            field::helper_text(&field_props, vec![], vec![text(desc)]),
        ],
    )
}

/// 狭幅縦並び全体。経路ごとに見出し（H4）+ 通知種別 4 行を並べる。
fn narrow_view() -> Node {
    let sections: Vec<Node> = (0..CHANNELS.len())
        .map(|channel_idx| {
            div(
                vec![(
                    "class",
                    "blocks-settings-notification-matrix-channel-section",
                )],
                std::iter::once(heading(
                    HeadingLevel::H4,
                    &HeadingProps::default(),
                    vec![],
                    vec![text(CHANNELS[channel_idx])],
                ))
                .chain(
                    (0..NOTIFICATION_TYPES.len()).map(|type_idx| narrow_row(type_idx, channel_idx)),
                )
                .collect(),
            )
        })
        .collect();
    div(
        vec![("class", "blocks-settings-notification-matrix-narrow")],
        sections,
    )
}

/// 保存ボタン。`<form>` を持たないため `button::button` の既定
/// `type="button"` のまま用いる（送信処理は持たない）。
fn actions() -> Node {
    div(
        vec![("class", "blocks-settings-notification-matrix-actions")],
        vec![button::button(
            &ButtonProps::default(),
            vec![],
            vec![text("保存")],
        )],
    )
}

/// `settings-notification-matrix` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-notification-matrix-stack")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("通知設定")],
            ),
            div(
                vec![("data-blocks-settings-notification-matrix-view", "table")],
                vec![wide_table()],
            ),
            narrow_view(),
            actions(),
        ],
    )
}
```

## 原案差分メモ

- 主参照は R0258 のみで、集約する別 version はありません（版違いなし）。
- 幅広の表と狭幅の縦並びは、同じ初期状態定数（種別 × 経路の
  4 行 × 3 列）から生成する 2 通りの DOM を並記し、`@container`
  （コンテナ幅 40rem 判定）で表示側を切り替えています。
- checkbox はすべて `disabled: true` のネイティブ固定です。無 JS の SSR
  では checkbox の見た目（`data-state`）は初期描画のまま変化しないため、
  操作可能に見えて実は無反応という不整合を避けるための判断です
  （`form-layout-stacked` と同型）。
- 幅広表の checkbox ラベルは配信経路名を視覚的に隠しています。見た目は
  列見出し（`<th scope="col">`）に委ね、アクセシブルネームのみラベルが
  担います。行の文脈は `<th scope="row">`（行見出し）が担います。

関連情報: [Checkbox](../themes/checkbox.md) / [Table](../themes/table.md) /
[Field](../themes/field.md) / [Button](../themes/button.md) /
[Heading](../themes/heading.md)
