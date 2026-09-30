# settings-export-data

エクスポート対象のチェックボックス一覧・ファイル形式の選択欄・実行ボタンを
上段に置き、過去のエクスポート履歴テーブル（日時・形式・状態・ダウンロード）
を下段に並べたデータエクスポート設定ブロックです。`checkbox` /
`native-select` / `field` / `button` / `table` / `badge` / `heading` の 7 部品を
合成します。Blocks は既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0243（代表構成、集約元 1 件）です。エクスポート対象は
初期状態でユーザー・プロジェクト・タスク・コメントの 4 件を選択済み、添付
ファイル・監査ログの 2 件を未選択で固定表示します。履歴は完了 3 件・処理中
1 件を並記し、状態はバッジで示します。処理中の行はダウンロード対象が存在
しないため、ダウンロードボタンをネイティブ `disabled` にしています。無 JS
のため見た目の `data-state` は初期状態から変化せず、エクスポート対象の
チェックボックスもネイティブ `disabled` で固定し、クリック後に選択状態と
チェック表示が食い違わないようにしています。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。エクスポート
実行ボタン・ダウンロードボタンはいずれも既定 `type="button"` で、送信先・
実処理を一切持ちません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::checkbox::{self, CheckboxProps, CheckedState};
use fandhe_frontend_pre_styled_ui::field::{
    self, FieldIds, FieldOrientation, FieldProps, FieldRootProps,
};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::native_select::{self, NativeSelectProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::{ColorPalette, Size};

/// 一意な id を組み立てる（`blocks-settings-export-data-` 接頭辞を共通化し、
/// フィールド追加時の綴り間違いを防ぐ）。
fn field_id(suffix: &str) -> String {
    format!("blocks-settings-export-data-{suffix}")
}

/// エクスポート対象チェックボックス 1 件を組み立てる。ネイティブ `disabled`
/// で操作不能にする理由はモジュール冒頭「checkbox をネイティブ `disabled`
/// にする理由」節参照（`onboarding_checklist.rs`/`form_layout_stacked.rs`
/// と同型の判断）。
fn export_target_checkbox(value: &'static str, label_text: &'static str, checked: bool) -> Node {
    let props = CheckboxProps {
        checked: if checked {
            CheckedState::Checked
        } else {
            CheckedState::Unchecked
        },
        disabled: true,
        ..CheckboxProps::default()
    };
    checkbox::root(
        Size::Md,
        ColorPalette::Accent,
        &props,
        vec![("data-blocks-settings-export-data-checkbox", "")],
        vec![
            checkbox::hidden_input(&props, "export-target", value, vec![]),
            checkbox::control(
                &props,
                vec![],
                vec![checkbox::indicator(&props, vec![], vec![])],
            ),
            checkbox::label(&props, vec![], vec![text(label_text)]),
        ],
    )
}

/// エクスポート対象一覧（2 列、狭幅では 1 列。[`LAYOUT_CSS`] の
/// `@container` 参照）。
fn export_targets() -> Node {
    let heading_id = "blocks-settings-export-data-targets-heading";
    div(
        vec![("class", "blocks-settings-export-data-section")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Lg,
                    weight: HeadingWeight::Bold,
                },
                vec![("id", heading_id)],
                vec![text("エクスポート対象")],
            ),
            el(
                "p",
                vec![("class", "blocks-settings-export-data-hint")],
                vec![text(
                    "選択した項目のみが出力されます。監査ログは既定で除外されています。",
                )],
            ),
            div(
                vec![
                    ("class", "blocks-settings-export-data-targets"),
                    ("role", "group"),
                    ("aria-labelledby", heading_id),
                ],
                vec![
                    export_target_checkbox("users", "ユーザー", true),
                    export_target_checkbox("projects", "プロジェクト", true),
                    export_target_checkbox("tasks", "タスク", true),
                    export_target_checkbox("comments", "コメント", true),
                    export_target_checkbox("attachments", "添付ファイル", false),
                    export_target_checkbox("audit-logs", "監査ログ", false),
                ],
            ),
        ],
    )
}

/// ファイル形式選択欄（`field` + `native_select`）。
fn format_field() -> Node {
    let id = field_id("format");
    let props = FieldProps {
        id: id.as_str(),
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    field::root(
        &FieldRootProps {
            orientation: FieldOrientation::Vertical,
        },
        &props,
        vec![("data-blocks-settings-export-data-field", "")],
        vec![
            field::label(&props, vec![], vec![text("ファイル形式")]),
            native_select::native_select(
                &NativeSelectProps::default(),
                &props,
                vec![],
                vec![
                    el(
                        "option",
                        vec![("value", "csv"), ("selected", "")],
                        vec![text("CSV")],
                    ),
                    el("option", vec![("value", "json")], vec![text("JSON")]),
                    el("option", vec![("value", "xlsx")], vec![text("XLSX")]),
                ],
            ),
        ],
    )
}

/// エクスポート設定セクション（対象一覧 + 形式選択 + 実行ボタン）。
fn export_settings_section() -> Node {
    div(
        vec![("class", "blocks-settings-export-data-section")],
        vec![
            export_targets(),
            format_field(),
            div(
                vec![("class", "blocks-settings-export-data-actions")],
                vec![button(
                    &ButtonProps {
                        variant: ButtonVariant::Solid,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("エクスポートを開始")],
                )],
            ),
        ],
    )
}

/// エクスポート履歴の 1 行（架空データ）。
struct HistoryRow {
    date: &'static str,
    format: &'static str,
    status_label: &'static str,
    status_palette: ColorPalette,
    ready: bool,
}

/// エクスポート履歴データ（架空。日時は ISO 風の固定表記、実在の日時・
/// 人物・企業を意味しない）。
const HISTORY_ROWS: &[HistoryRow] = &[
    HistoryRow {
        date: "2026-09-28 09:12",
        format: "CSV",
        status_label: "完了",
        status_palette: ColorPalette::Success,
        ready: true,
    },
    HistoryRow {
        date: "2026-09-25 18:40",
        format: "JSON",
        status_label: "完了",
        status_palette: ColorPalette::Success,
        ready: true,
    },
    HistoryRow {
        date: "2026-09-20 07:03",
        format: "XLSX",
        status_label: "完了",
        status_palette: ColorPalette::Success,
        ready: true,
    },
    HistoryRow {
        date: "2026-09-30 11:55",
        format: "CSV",
        status_label: "処理中",
        status_palette: ColorPalette::Info,
        ready: false,
    },
];

/// 履歴テーブルの 1 行。処理中の行はダウンロード対象が存在しないため
/// ボタンをネイティブ `disabled` にする（モジュール doc「ダウンロード列に
/// リンクではなくボタンを使う理由」節）。ダウンロードボタンのアクセシブル
/// 名は全行「ダウンロード」で同一だとスクリーンリーダーで対象履歴を識別
/// できないため（Codex 指摘対応）、`aria-label` で `row.date`/`row.format`
/// を含めた行固有の名前を付与する。
fn history_row(row: &HistoryRow) -> Node {
    let download_label = format!("{}（{}）をダウンロード", row.date, row.format);
    table::row(
        vec![],
        vec![
            table::cell(vec![], vec![text(row.date)]),
            table::cell(vec![], vec![text(row.format)]),
            table::cell(
                vec![],
                vec![badge(
                    &BadgeProps {
                        variant: BadgeVariant::Subtle,
                        palette: row.status_palette,
                        ..BadgeProps::default()
                    },
                    vec![],
                    vec![text(row.status_label)],
                )],
            ),
            table::cell(
                vec![],
                vec![button(
                    &ButtonProps {
                        variant: ButtonVariant::Outline,
                        size: Size::Sm,
                        disabled: !row.ready,
                        ..ButtonProps::default()
                    },
                    vec![("aria-label", download_label.as_str())],
                    vec![text("ダウンロード")],
                )],
            ),
        ],
    )
}

/// エクスポート履歴セクション（`table::scroll_area` + `table::root`）。
/// スクロール領域はキーボード操作・スクリーンリーダー双方に対応させる
/// （Cursor Bugbot 指摘対応）: `role="region"` + `aria-labelledby` で見出しへ
/// 意味づけし、`tabindex="0"` でキーボードフォーカス・矢印キースクロールを
/// 可能にする。テーブル自体のアクセシブル名は `table::caption`（視覚的には
/// [`LAYOUT_CSS`] の `.blocks-settings-export-data-sr-only` で clip）で補う
/// （新しい UI 部品は追加しない契約のため `visually_hidden` 部品は使わず、
/// 既存 `table::caption` パーツ + block 固有 CSS で実現する）。
fn history_section() -> Node {
    let heading_id = "blocks-settings-export-data-history-heading";
    div(
        vec![("class", "blocks-settings-export-data-section")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Lg,
                    weight: HeadingWeight::Bold,
                },
                vec![("id", heading_id)],
                vec![text("エクスポート履歴")],
            ),
            table::scroll_area(
                vec![
                    ("role", "region"),
                    ("aria-labelledby", heading_id),
                    ("tabindex", "0"),
                ],
                vec![table::root(
                    TableProps::default(),
                    vec![("data-blocks-settings-export-data-table", "")],
                    vec![
                        table::caption(
                            vec![("class", "blocks-settings-export-data-sr-only")],
                            vec![text("エクスポート履歴の一覧")],
                        ),
                        table::header(
                            vec![],
                            vec![table::row(
                                vec![],
                                vec![
                                    table::column_header(vec![], vec![text("日時")]),
                                    table::column_header(vec![], vec![text("形式")]),
                                    table::column_header(vec![], vec![text("状態")]),
                                    table::column_header(vec![], vec![text("ダウンロード")]),
                                ],
                            )],
                        ),
                        table::body(vec![], HISTORY_ROWS.iter().map(history_row).collect()),
                    ],
                )],
            ),
        ],
    )
}

/// `settings-export-data` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-settings-export-data-stack")],
        vec![export_settings_section(), history_section()],
    )
}
```

## 集約元との差分メモ

- 集約元は R0243 の 1 件のみで、統合すべき別 variant はありません。
- 参照ファイル（`_/blocks-intake/`）は本イシュー着手時点で本 worktree に
  存在しないため、レイアウト仕様（イシュー本文）の文面から独自に構成し、
  本原稿・実装モジュール doc には対応表 ID のみを記載しています。
- 狭幅（コンテナ幅 36rem 未満）ではエクスポート対象一覧が 2 列から 1 列へ
  切り替わります（`@container` によるコンテナクエリ判定）。
- 履歴の日時・件数はすべて架空のデータであり、実在の人物・企業・PII は
  含みません。

関連情報: [Checkbox](../themes/checkbox.md) /
[Native Select](../themes/native-select.md) / [Field](../themes/field.md) /
[Button](../themes/button.md) / [Table](../themes/table.md) /
[Badge](../themes/badge.md) / [Heading](../themes/heading.md)
