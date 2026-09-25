# comparison-split-table

`heading` / `text` / `badge` / `button` / `table` / `icon` の 6 部品を合成した、見出しブロックと比較表を左右に並べるレイアウトです。左に見出しブロック（タグライン・見出し・説明・ボタン）、右に自社と競合を比べる表を置きます。

- 静的表示です。docs サイトは無 JS のため、状態機械・フォーム・データ取得を持たない固定描画にしています。
- 文言はすべて架空のものです。実在の企業名・製品名・競合名は使わず、中立的な「自社」「他社」ラベルにしています。
- データ取得・送信は行わず、`<form>` は使いません。ボタンは 2 個とも `type="button"` のままで、送信先を持ちません。
- `64rem` 以上（lg）では 2 カラムになります。左に見出し・説明・ボタン、右に比較表を置きます。狭い幅では見出しの下に表を続け、表は横スクロールします。
- 比較表のセルはチェック/バツのアイコンで示すもの（対応/非対応）と、テキスト値で示すもの（例: 「無制限」「10 人まで」）が混在しています。
- 機能名には補足説明を添えています。参照元はホバーで開くツールチップですが、無 JS のため機能名の下に常時表示のテキストへ置き換えています。
- アイコンは参照元の SVG をコピーせず、独自のチェックマーク・バツ印にしています。色だけで区別しないよう、アイコンの形自体も変え、`aria-label`（「対応」/「非対応」）を付けています。
- 集約元は 1 件です（対応表 ID R0437）。取り込んだのはレイアウト構造（見出しブロック + 比較表の左右配置・レスポンシブな 1 カラム/2 カラム切り替え・表の横スクロール）のみで、文言・配色・装飾・アイコン形状・内部識別子は取り込んでいません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::ColorPalette;

/// 対応セルの CSS フック（色だけに頼らずチェックマークの形でも区別する。
/// 上記モジュール doc「アイコンは独自の抽象図形のみ・アクセシブルネームを
/// 持つ」節参照）。`icon::icon` は呼び出し側 `attrs` の `class` を
/// `drop_class_attr` で除去する契約のため、`class` ではなく値なしの
/// `data-*` 属性で渡す（`class` で渡すと出力から消え CSS が一致しない）。
const YES_ATTR: &str = "data-blocks-comparison-split-table-yes";
/// 非対応セルの CSS フック（上記と対になる、バツ印用）。
const NO_ATTR: &str = "data-blocks-comparison-split-table-no";

/// チェックマークのみの自作アイコン（`comparison_feature_rows::glyph_circle`
/// と同型の対処。参照元の SVG path はコピーしない）。
fn icon_check() -> Node {
    icon(
        &IconProps {
            label: Some("対応"),
            ..IconProps::default()
        },
        vec![(YES_ATTR, "")],
        vec![el(
            "path",
            vec![
                ("d", "M20 6L9 17l-5-5"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// バツ印のみの自作アイコン（[`icon_check`] と対になる、非対応セル用）。
fn icon_cross() -> Node {
    icon(
        &IconProps {
            label: Some("非対応"),
            ..IconProps::default()
        },
        vec![(NO_ATTR, "")],
        vec![el(
            "path",
            vec![
                ("d", "M6 6l12 12M6 18L18 6"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// セル値（対応/非対応/テキスト値の混在、上記モジュール doc「レイアウト
/// 構造」節参照）。
enum CellValue {
    /// チェックマーク（[`icon_check`]、アクセシブルネーム「対応」）。
    Yes,
    /// バツ印（[`icon_cross`]、アクセシブルネーム「非対応」）。
    No,
    /// テキスト値（既定エスケープ経由で出力）。
    Text(&'static str),
}

/// 比較行 1 件分のダミーデータ（架空、実在の製品・企業とは無関係）。
struct Row {
    /// 機能名。
    name: &'static str,
    /// 補足説明（機能名の下へ常時表示、上記モジュール doc「補足説明の
    /// 表示方法」節参照）。
    note: &'static str,
    /// 自社列の値。
    ours: CellValue,
    /// 他社列の値。
    theirs: CellValue,
}

/// 比較行一覧（架空、5 行固定。検索インデックスの肥大を抑えるため件数を
/// 増やさない。Yes/No が混在する並びにする）。
const ROWS: [Row; 5] = [
    Row {
        name: "同時編集の人数",
        note: "1 ワークスペースで同時に編集できる人数の上限です。",
        ours: CellValue::Text("無制限"),
        theirs: CellValue::Text("10 人まで"),
    },
    Row {
        name: "権限管理",
        note: "メンバーごとに閲覧・編集・管理者の権限を分けられます。",
        ours: CellValue::Yes,
        theirs: CellValue::No,
    },
    Row {
        name: "監査ログ",
        note: "誰がいつ何を変更したかを記録し、いつでも確認できます。",
        ours: CellValue::Yes,
        theirs: CellValue::No,
    },
    Row {
        name: "共同編集",
        note: "複数人が同じドキュメントをリアルタイムに編集できます。",
        ours: CellValue::Yes,
        theirs: CellValue::Yes,
    },
    Row {
        name: "データの書き出し",
        note: "保存済みデータをまとめて書き出せます。",
        ours: CellValue::Text("全形式に対応"),
        theirs: CellValue::Text("CSV のみ"),
    },
];

/// 見出しブロック（タグライン `badge` + セクション見出し + リード文 +
/// アクション `button` × 2、上記モジュール doc「レイアウト構造」節参照）。
fn intro() -> Node {
    div(
        vec![("class", "blocks-comparison-split-table-intro")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text("比較")]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("選ばれる理由を表で確認")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "主要な機能を自社・他社で並べて比較できます。まずは無料でお試しください。",
                )],
            ),
            div(
                vec![("class", "blocks-comparison-split-table-actions")],
                vec![
                    button::button(&ButtonProps::default(), vec![], vec![text("無料で試す")]),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![],
                        vec![text("プランを見る")],
                    ),
                ],
            ),
        ],
    )
}

/// セル値 1 件を [`Node`] へ変換する（[`CellValue::Text`] は既定エスケープ
/// を経由する [`fandhe_frontend_core::text`] で出力する）。
fn value_node(value: &CellValue) -> Node {
    match value {
        CellValue::Yes => icon_check(),
        CellValue::No => icon_cross(),
        CellValue::Text(value) => span(
            vec![("data-blocks-comparison-split-table-value", "")],
            vec![text(*value)],
        ),
    }
}

/// 機能名セル（行見出し）を `<th scope="row">` として組み立てる
/// （上記モジュール doc「`table` の合成方法」節参照）。`table::cell`
/// （`<td>`）は呼ばず、`fandhe_frontend_pre_styled_ui::table` の CSS が
/// `[data-scope="table"][data-part="cell"]`（タグ名非依存の属性セレクタ）で
/// 出力されることを利用して、見た目は既存の `cell` パーツと同一のまま
/// `scope="row"` を持つ `<th>` を直接構築する。これによりスクリーンリーダー
/// が対応/非対応セルへ移動した際、行見出し（機能名）が自動的に読み上げ
/// られる（`table::column_header` は `scope="col"` を固定して呼び出し側の
/// `scope` 指定を除去するため使えない）。
fn feature_cell(row: &Row) -> Node {
    el(
        "th",
        vec![
            ("data-scope", "table"),
            ("data-part", "cell"),
            ("scope", "row"),
            ("data-blocks-comparison-split-table-feature", ""),
        ],
        vec![
            span(
                vec![("data-blocks-comparison-split-table-feature-name", "")],
                vec![text(row.name)],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-comparison-split-table-feature-note", "")],
                vec![text(row.note)],
            ),
        ],
    )
}

/// 比較表本体（列見出し「機能」「自社」「他社」+ [`ROWS`] の件数分の本文
/// 行。上記モジュール doc「`table` の合成方法」節参照）。
fn comparison_table() -> Node {
    let header_row = table::row(
        vec![],
        vec![
            table::column_header(vec![], vec![text("機能")]),
            table::column_header(
                vec![("data-align", "center")],
                vec![badge::badge(
                    &BadgeProps {
                        variant: BadgeVariant::Solid,
                        palette: ColorPalette::Accent,
                        ..BadgeProps::default()
                    },
                    vec![],
                    vec![text("自社")],
                )],
            ),
            table::column_header(
                vec![("data-align", "center")],
                vec![badge::badge(
                    &BadgeProps {
                        variant: BadgeVariant::Outline,
                        palette: ColorPalette::Neutral,
                        ..BadgeProps::default()
                    },
                    vec![],
                    vec![text("他社")],
                )],
            ),
        ],
    );

    let body_rows: Vec<Node> = ROWS
        .iter()
        .map(|row| {
            table::row(
                vec![("data-blocks-comparison-split-table-row", "")],
                vec![
                    feature_cell(row),
                    table::cell(vec![("data-align", "center")], vec![value_node(&row.ours)]),
                    table::cell(
                        vec![("data-align", "center")],
                        vec![value_node(&row.theirs)],
                    ),
                ],
            )
        })
        .collect();

    table::scroll_area(
        vec![
            ("data-blocks-comparison-split-table-scroll", ""),
            ("tabindex", "0"),
            ("role", "region"),
            ("aria-label", "機能比較表"),
        ],
        vec![table::root(
            TableProps::default(),
            vec![("data-blocks-comparison-split-table-table", "")],
            vec![
                table::header(vec![], vec![header_row]),
                table::body(vec![], body_rows),
            ],
        )],
    )
}

/// `comparison-split-table` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（[`crate::blocks`] モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    div(
        vec![("data-blocks-comparison-split-table-root", "")],
        vec![intro(), comparison_table()],
    )
}
```

## 原案差分メモ

- 補足説明は参照元ではホバーで開くツールチップですが、無 JS 制約のため機能名の下に常時表示の補足テキストへ置き換えました。
- アイコンは参照元の SVG path をコピーせず、独自のチェックマーク・バツ印にしています。色だけで区別しないよう形自体も変え、`aria-label`（「対応」/「非対応」）を付けています。
- 見出しレベルは、ページ側の `## Demo` に合わせてセクション見出しを `h3` にしています。
- 比較表の左上見出しは空欄にせず「機能」と表示しています。
- 自社・他社のラベルは `badge` で示し、自社側を強調配色（`Solid` + `Accent`）、他社側を中立配色（`Outline` + `Neutral`）にして視覚的に区別しています。
- ボタンは 2 個で、いずれも `type="button"` のままで送信先を持ちません。
- 狭い幅での横スクロールは `table` の scroll-area パーツと `min-width` で実現しています。
- 配色・余白は既存のテーマトークンに従い、文言はすべて独自に書きました。実在の企業名・製品名・競合名は使っていません。
