# table-responsive-stacked

`table` / `data-list` / `heading` / `button` / `visually-hidden` の 5 部品を
合成した、狭幅で副次列を折り畳むテーブルの合成例です。Blocks セクションは
新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください。

広幅では氏名・メール・役職・所属・操作の 5 列テーブルとして表示し、狭幅
（コンテナ幅 40rem 未満）ではメール・役職・所属の列を隠し、その値を氏名
セル内へ縦積みの定義リスト（`data-list`）として表示します。「操作」列の
見出しは視覚上は空で、スクリーンリーダー向けにのみ列名を供給します
（`visually-hidden` の使用箇所）。docs サイトは JS ハイドレーションを行わ
ないため、実際にビューポートを変えてリサイズする実演はできません。代わりに
同一構造を持つ 2 インスタンス（広幅 / `max-inline-size: 24rem` で強制的に
狭幅化）を並記し、折り畳みの違いを静的に見せます。

メール・役職・所属の値はセル本体と氏名セル内の定義リストの 2 か所に出力
しますが、コンテナクエリの条件で常にどちらか一方だけが `display: none` に
なるため、スクリーンリーダーによる二重読み上げは起きません。見出し行の
上部には「メンバー」見出し + 「メンバーを追加」ボタンのツールバーを置き
ます（基本形 `table-with-heading` と同じ配置）。

静的な表示例であり、`<form>` 要素を持ちません。氏名・役職・社名は独自に
書いた架空の人名・役職・社名セットから引用し、メールアドレスは
`example.com` ドメインの架空値です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::button::{button, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListOrientation, DataListProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::table::{self, TableProps};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// ラベル・値の 1 行（狭幅時に氏名セル内へ表示する定義リストの内訳）。
fn stacked_row(label: &'static str, value: &'static str) -> Node {
    data_list::item(
        vec![],
        vec![
            data_list::item_label(vec![], vec![text(label)]),
            data_list::item_value(vec![], vec![text(value)]),
        ],
    )
}

/// 1 行分のデータ（氏名 + 副次列 3 種）を組み立てる。
fn member_row(
    name: &'static str,
    email: &'static str,
    title: &'static str,
    company: &'static str,
) -> Node {
    table::row(
        vec![],
        vec![
            table::row_header(
                vec![],
                vec![
                    text(name),
                    data_list::root(
                        DataListProps {
                            orientation: DataListOrientation::Vertical,
                            size: Size::Sm,
                            ..DataListProps::default()
                        },
                        vec![("data-blocks-table-responsive-stacked-stacked", "")],
                        vec![
                            stacked_row("メール", email),
                            stacked_row("役職", title),
                            stacked_row("所属", company),
                        ],
                    ),
                ],
            ),
            table::cell(
                vec![("data-blocks-table-responsive-stacked-secondary", "")],
                vec![text(email)],
            ),
            table::cell(
                vec![("data-blocks-table-responsive-stacked-secondary", "")],
                vec![text(title)],
            ),
            table::cell(
                vec![("data-blocks-table-responsive-stacked-secondary", "")],
                vec![text(company)],
            ),
            table::cell(
                vec![],
                vec![button(
                    &ButtonProps {
                        variant: ButtonVariant::Ghost,
                        size: Size::Sm,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("編集")],
                )],
            ),
        ],
    )
}

/// 見出し + 追加ボタンのツールバー（基本形 `table-with-heading` と同じ
/// 位置＝テーブル上部に置く）。
fn toolbar() -> Node {
    div(
        vec![("class", "blocks-table-responsive-stacked-toolbar")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps::default(),
                vec![],
                vec![text("メンバー")],
            ),
            button(
                &ButtonProps {
                    variant: ButtonVariant::Solid,
                    size: Size::Sm,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("メンバーを追加")],
            ),
        ],
    )
}

/// テーブル本体（ツールバー + `table::root`）。状態 A/B で共有する。
fn table_section() -> Vec<Node> {
    vec![
        toolbar(),
        table::root(
            TableProps::default(),
            vec![("data-blocks-table-responsive-stacked-table", "")],
            vec![
                table::header(
                    vec![],
                    vec![table::row(
                        vec![],
                        vec![
                            table::column_header(vec![], vec![text("名前")]),
                            table::column_header(
                                vec![("data-blocks-table-responsive-stacked-secondary", "")],
                                vec![text("メール")],
                            ),
                            table::column_header(
                                vec![("data-blocks-table-responsive-stacked-secondary", "")],
                                vec![text("役職")],
                            ),
                            table::column_header(
                                vec![("data-blocks-table-responsive-stacked-secondary", "")],
                                vec![text("所属")],
                            ),
                            table::column_header(
                                vec![],
                                vec![visually_hidden::root(vec![], vec![text("操作")])],
                            ),
                        ],
                    )],
                ),
                table::body(
                    vec![],
                    vec![
                        member_row(
                            dummy_assets::PERSON_NAMES[0],
                            "haruto.fujimaki@example.com",
                            dummy_assets::JOB_TITLES[0],
                            dummy_assets::COMPANY_NAMES[0],
                        ),
                        member_row(
                            dummy_assets::PERSON_NAMES[1],
                            "elena.vasquez@example.com",
                            dummy_assets::JOB_TITLES[1],
                            dummy_assets::COMPANY_NAMES[1],
                        ),
                        member_row(
                            dummy_assets::PERSON_NAMES[2],
                            "kenji.oshiro@example.com",
                            dummy_assets::JOB_TITLES[2],
                            dummy_assets::COMPANY_NAMES[2],
                        ),
                        member_row(
                            dummy_assets::PERSON_NAMES[3],
                            "amara.okafor@example.com",
                            dummy_assets::JOB_TITLES[3],
                            dummy_assets::COMPANY_NAMES[3],
                        ),
                    ],
                ),
            ],
        ),
    ]
}

/// `table-responsive-stacked` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-table-responsive-stacked-stack")],
        vec![
            div(
                vec![("class", "blocks-table-responsive-stacked-frame")],
                table_section(),
            ),
            div(
                vec![(
                    "class",
                    "blocks-table-responsive-stacked-frame blocks-table-responsive-stacked-frame--narrow",
                )],
                table_section(),
            ),
        ],
    )
}
```

## 原案差分メモ

- 参照 ID は R1325（代表構成）のみです。集約元が 1 件のため、差分の
  すり合わせは発生していません。
- 広幅・狭幅の並記は、JS ハイドレーションを行わない docs サイトの制約
  下でリサイズ実演を代替する仕組みです。`.blocks-table-responsive-
  stacked-frame--narrow` の `max-inline-size: 24rem` で強制的に狭幅化し、
  `@container` の判定閾値（`40rem`）を下回らせています。
- 副次列の値をセル本体と定義リストの 2 か所へ出力する設計は、`display:
  none` がアクセシビリティツリーからも要素を除外する性質を利用した、
  読み上げ重複回避のための構成です（情報の重複自体は意図的）。
- ブラウザでの実機確認（コンテナ幅切替・ライト/ダーク両テーマ）は
  サンドボックス制約により未実施です。cargo test による出力検証のみで
  代替しました。

関連情報: [Table](../themes/table.md) / [Data List](../themes/data-list.md) /
[Heading](../themes/heading.md) / [Button](../themes/button.md) /
[Visually Hidden](../themes/visually-hidden.md)
