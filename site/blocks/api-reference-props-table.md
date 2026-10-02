# api-reference-props-table

`table` / `code` / `text` / `heading` の 4 部品を合成した API プロパティ表
です。プロパティ名・型・既定値・説明の 4 列を持つ枠付きの表（`<table>`）
を 1 つ表示します。

- プロパティ名は強調色（accent）の等幅（`code`）で表示します。
- 型・既定値も等幅（`code`、既定の Neutral palette）で表示します。
- 表は横スクロール領域（`table` の scroll-area パーツ）で包み、狭い幅
  でも枠内に収まります。
- 各セルは上揃えです。
- 静的表示です。`<form>` は使用せず、データ取得・送信も行いません。
- プロパティ名・型・説明はすべて架空の UI 部品（`Toast`）のもので、
  実在の製品名ではありません。

集約元は 1 件（対応表 ID R0196、4 列構成）です。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps, TableVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::ColorPalette;

/// プロパティ表 1 行分（架空コンポーネントのプロパティ）。
struct PropRow {
    name: &'static str,
    ty: &'static str,
    default: &'static str,
    description: &'static str,
}

/// 架空コンポーネント（`Toast`）のプロパティ一覧。`<` と `&` を含む型表記
/// （`Option<&str>`）を 1 件含め、既定エスケープ経路を実演する
/// （ファイル内ユニットテストで固定する不変条件）。
const PROPS: [PropRow; 5] = [
    PropRow {
        name: "title",
        ty: "Option<&str>",
        default: "None",
        description: "通知の見出し文言。省略時は本文のみ表示します。",
    },
    PropRow {
        name: "variant",
        ty: "ToastVariant",
        default: "Info",
        description: "見た目の種別（Info / Success / Warning / Danger）。",
    },
    PropRow {
        name: "dismissible",
        ty: "bool",
        default: "true",
        description: "閉じるボタンを表示するかどうか。",
    },
    PropRow {
        name: "duration_ms",
        ty: "u32",
        default: "4000",
        description: "自動で閉じるまでの表示時間（ミリ秒）。",
    },
    PropRow {
        name: "on_dismiss",
        ty: "Option<fn()>",
        default: "None",
        description: "閉じられた際に呼び出すコールバック。",
    },
];

/// プロパティ 1 行分（`table::row`）を組み立てる。
///
/// プロパティ名は `table::row_header`（`<th scope="row">`、モジュール doc
/// 「行見出しに `table::row_header` を使う理由」節）の中に accent palette の
/// `code` を置いて強調する。型・既定値は `table::cell` の中に既定
/// （Neutral）palette の `code`、説明は `styled_text::text`（Muted）。
fn prop_row(row: &PropRow) -> Node {
    table::row(
        vec![],
        vec![
            table::row_header(
                vec![("data-blocks-api-reference-props-table-name", "")],
                vec![code::code(
                    &CodeProps {
                        palette: ColorPalette::Accent,
                        ..CodeProps::default()
                    },
                    vec![],
                    vec![text(row.name)],
                )],
            ),
            table::cell(
                vec![("data-blocks-api-reference-props-table-type", "")],
                vec![code::code(
                    &CodeProps::default(),
                    vec![],
                    vec![text(row.ty)],
                )],
            ),
            table::cell(
                vec![("data-blocks-api-reference-props-table-default", "")],
                vec![code::code(
                    &CodeProps::default(),
                    vec![],
                    vec![text(row.default)],
                )],
            ),
            table::cell(
                vec![("data-blocks-api-reference-props-table-description", "")],
                vec![styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![],
                    vec![text(row.description)],
                )],
            ),
        ],
    )
}

/// `api-reference-props-table` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ設計）。
pub fn demo() -> Node {
    let intro = div(
        vec![("class", "blocks-api-reference-props-table-intro")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("Toast props")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-api-reference-props-table-lead", "")],
                vec![text(
                    "Toast コンポーネントが受け付けるプロパティの一覧です。",
                )],
            ),
        ],
    );

    let header = table::header(
        vec![],
        vec![table::row(
            vec![],
            vec![
                table::column_header(vec![], vec![text("プロパティ")]),
                table::column_header(vec![], vec![text("型")]),
                table::column_header(vec![], vec![text("既定値")]),
                table::column_header(vec![], vec![text("説明")]),
            ],
        )],
    );

    let body = table::body(vec![], PROPS.iter().map(prop_row).collect());

    let table_node = table::root(
        TableProps {
            variant: TableVariant::Outline,
            ..TableProps::default()
        },
        vec![("data-blocks-api-reference-props-table-table", "")],
        vec![
            table::caption(vec![], vec![text("Toast コンポーネントの props 一覧")]),
            header,
            body,
        ],
    );

    let scroll = table::scroll_area(
        vec![
            ("data-blocks-api-reference-props-table-scroll", ""),
            ("role", "region"),
            ("aria-label", "Toast props 一覧表"),
            ("tabindex", "0"),
        ],
        vec![table_node],
    );

    div(
        vec![("class", "blocks-api-reference-props-table-layout")],
        vec![intro, scroll],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0196）からの意図的な差分は次のとおりです。

- 説明を独立した 4 列目にしました（既定値の下へ続ける形は採りません
  でした）。
- プロパティ名は行見出しパーツ `table::row_header`（`<th scope="row">`）
  に置きました。
- 見出しレベルは `h3` にしました（ページ側が `## Demo` として `h2` を
  出すため）。
- 配色は既存トークンのみを使い、参照元の装飾は持ち込みませんでした。
- 文言はすべて独自に書き下ろしたもので、実在の製品・サービス名では
  ありません。
