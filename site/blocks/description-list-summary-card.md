# description-list-summary-card

`fandhe-frontend-pre-styled-ui` の `card` / `data-list` / `badge` / `icon` /
`link` / `visually-hidden` を合成した、アイコン行のサマリーカードです。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/
Primitives 部品を組み合わせた実例集であることに注意してください（主参照は
対応表 ID R0892。出典の固有名・ファイル名は記載しません）。

カード上段には金額と支払状態バッジを横並びで表示し、下段には担当者・
期日・支払方法をアイコン付きの行として縦に並べます。下段の行ラベルは
画面上は視覚的に隠し（`visually-hidden`）、アイコンで意味を示します。
末尾には実在の GitHub リポジトリへの案内リンクを置いています。

本 Demo は静的な表示例であり、`<form>` 要素は一切持たず、データの取得・
送信・状態管理を行いません。文言・数値はすべて独自に書いた架空のもの
であり、実企業名・実クレデンシャル・PII を含みません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::data_list::{self, DataListProps};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::recipe::{ColorPalette, Size};
use fandhe_frontend_pre_styled_ui::visually_hidden;

/// 実在の GitHub リポジトリへの固定外部 URL（`href="#"` を避ける、footer
/// 系 block と同じ判断）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 幾何図形アイコンを 1 個組み立てる小さな helper（内部専用）。装飾用途
/// のため常に `label: None`（`aria-hidden="true"`）で、隣接する
/// [`visually_hidden::root`] がアクセシブルネームを担う契約は呼び出し側
/// （[`icon_row`]）が満たす。
fn geo_icon(children: Vec<Node>) -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        children,
    )
}

/// 人物アイコン（円 = 頭部 + 弧 = 肩）。
fn person_icon() -> Node {
    geo_icon(vec![
        el(
            "circle",
            vec![
                ("cx", "12"),
                ("cy", "8"),
                ("r", "3"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
            ],
            vec![],
        ),
        el(
            "path",
            vec![
                ("d", "M5 20a7 7 0 0114 0"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
            ],
            vec![],
        ),
    ])
}

/// カレンダーアイコン（角丸矩形 + 横線）。
fn calendar_icon() -> Node {
    geo_icon(vec![
        el(
            "rect",
            vec![
                ("x", "4"),
                ("y", "5"),
                ("width", "16"),
                ("height", "15"),
                ("rx", "2"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
            ],
            vec![],
        ),
        el(
            "line",
            vec![
                ("x1", "4"),
                ("y1", "10"),
                ("x2", "20"),
                ("y2", "10"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
            ],
            vec![],
        ),
    ])
}

/// 支払方法アイコン（矩形 + 帯、クレジットカードの磁気ストライプを模す）。
fn card_icon() -> Node {
    geo_icon(vec![
        el(
            "rect",
            vec![
                ("x", "3"),
                ("y", "6"),
                ("width", "18"),
                ("height", "12"),
                ("rx", "2"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
            ],
            vec![],
        ),
        el(
            "rect",
            vec![
                ("x", "3"),
                ("y", "9"),
                ("width", "18"),
                ("height", "3"),
                ("fill", "currentColor"),
            ],
            vec![],
        ),
    ])
}

/// アイコン付きの 1 行（`dt` = アイコン + 隠しラベル、`dd` = 値）を組み立
/// てる（本モジュール doc「行ラベルの隠し方」参照）。`value` は呼び出し側
/// が組み立てた任意のノード（プレーンテキスト・`<time>` 等）。
fn icon_row(icon_node: Node, label: &'static str, value: Node) -> Node {
    data_list::item(
        vec![("data-blocks-description-list-summary-card-row", "")],
        vec![
            data_list::item_label(
                vec![],
                vec![icon_node, visually_hidden::root(vec![], vec![text(label)])],
            ),
            data_list::item_value(vec![], vec![value]),
        ],
    )
}

/// `description-list-summary-card` の Demo 本体。呼び出しごとに同一の
/// `Node` を返す純関数。
pub fn demo() -> Node {
    let summary = data_list::item(
        vec![("data-blocks-description-list-summary-card-summary", "")],
        vec![
            data_list::item_label(vec![], vec![text("金額")]),
            data_list::item_value(
                vec![("data-blocks-description-list-summary-card-amount-row", "")],
                vec![
                    span(
                        vec![("data-blocks-description-list-summary-card-amount", "")],
                        vec![text("¥128,000")],
                    ),
                    badge(
                        &BadgeProps {
                            variant: BadgeVariant::Subtle,
                            size: Size::Sm,
                            palette: ColorPalette::Success,
                            shape: None,
                        },
                        vec![],
                        vec![text("支払済み")],
                    ),
                ],
            ),
        ],
    );

    let assignee = icon_row(person_icon(), "担当者", text("山田 花子"));
    let due_date = icon_row(
        calendar_icon(),
        "期日",
        el(
            "time",
            vec![("datetime", "2026-10-31")],
            vec![text("2026年10月31日")],
        ),
    );
    let payment_method = icon_row(card_icon(), "支払方法", text("クレジットカード"));

    let card = card::root(
        CardProps::from(CardVariant::Subtle),
        vec![],
        vec![
            card::body(
                vec![],
                vec![data_list::root(
                    DataListProps::default(),
                    vec![],
                    vec![summary, assignee, due_date, payment_method],
                )],
            ),
            card::footer(
                vec![],
                vec![link::root(
                    REPO,
                    &LinkProps::default(),
                    vec![],
                    vec![
                        text("GitHub で見る"),
                        span(vec![("aria-hidden", "true")], vec![text("→")]),
                    ],
                )],
            ),
        ],
    );

    div(
        vec![("class", "blocks-description-list-summary-card-layout")],
        vec![card],
    )
}
```

## 原案差分メモ

- 集約元は主参照（対応表 ID R0892）の 1 件のみです。参照元の文言・配色・
  アイコン形状は持ち込まず、デモ文言・アイコンはすべて独自に書きました。
- 上段（金額 + 支払状態バッジ）は別々の `dl` グループに分けず、1 つの
  `dt`/`dd` に収めています。`dd` 自体が横並びレイアウトを持つため、
  追加のレイアウト CSS はバッジを右寄せする 1 行のみです。
- 下段 3 行の `dt` はアイコン（装飾、`aria-hidden`）と視覚的に隠した
  ラベル文字列を両方持ちます。ラベルを丸ごと削除せず隠すだけに留めて
  いるため、スクリーンリーダーには「担当者 山田 花子」のような対が
  そのまま伝わります。
- アイコンは実在ブランドを模さない自作の幾何図形（円+弧・矩形+横線・
  矩形+帯）です。

関連情報: [Card](../themes/card.md) / [Data List](../themes/data-list.md) /
[Badge](../themes/badge.md) / [Icon](../themes/icon.md) /
[Link](../themes/link.md) / [Visually Hidden](../themes/visually-hidden.md)
