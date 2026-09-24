# bento-asymmetric-rows

`fandhe-frontend-pre-styled-ui` の `badge` / `heading` / `text` / `card` /
`image` を合成した、幅の異なるカードを 2 行に並べる bento グリッドの合成例
です。Blocks セクションは新規部品を追加するものではなく、既存の Themes/
Primitives 部品を組み合わせた実例集であることに注意してください。

見出しエリアの下に、幅 lg（64rem）以上では 6 列グリッドで 1 行目 4+2・
2 行目 2+4 の幅違いのカードを並べます。幅 md（48rem）以上 lg 未満では
2 列、md 未満では 1 列に切り替わります。各カードは画像を上、見出しと説明
文を下に置く構成です。文言はすべて架空のもので、データ取得・送信は行わない
静的な表示例です。`<form>` は使用しません。

本ページは基準形（対応表 ID R0769）のみを実装したものです。集約元に含まれる
残りのバリエーション（3+3・2+2+2 のグリッド、3 列ジグザグ配置、枠なしの
淡色カード、下段の小さな feature 一覧等）は後続の対応で追加予定です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// セルの幅区分（[`LAYOUT_CSS`] の `grid-column: span` を切り替える唯一の
/// 軸）。`props::*` へは昇格せず、本 block ローカルの列挙型とする
/// （`alert::Severity`/`progress::ProgressShape` と同じ判断軸）。
#[derive(Clone, Copy)]
enum CellWidth {
    /// 6 列中 4 列分（lg 以上）。
    Wide,
    /// 6 列中 2 列分（lg 以上）。
    Narrow,
}

impl CellWidth {
    /// [`LAYOUT_CSS`] の `[data-blocks-bento-asymmetric-rows-cell="..."]`
    /// セレクタと一致させる値。
    const fn value(self) -> &'static str {
        match self {
            CellWidth::Wide => "wide",
            CellWidth::Narrow => "narrow",
        }
    }
}

/// 1 枚分のセルデータ（架空の SaaS 機能名 + 1 行説明 + 幅区分）。
struct Cell {
    title: &'static str,
    description: &'static str,
    width: CellWidth,
}

/// 4 セル（1 行目 4+2、2 行目 2+4）。並び順がグリッドの既定の自動配置
/// （`grid-auto-flow: row`）と組み合わさって基準形（R0769）の配置を作る
/// 不変条件を、本ファイル末尾の `#[cfg(test)]` が固定する。
const CELLS: [Cell; 4] = [
    Cell {
        title: "Unified Workspace",
        description: "散らばっていたツールを 1 つの画面に集約し、切り替えの手間を無くします。",
        width: CellWidth::Wide,
    },
    Cell {
        title: "Instant Handoff",
        description: "担当者の引き継ぎをワンクリックで完了します。",
        width: CellWidth::Narrow,
    },
    Cell {
        title: "Version History",
        description: "変更履歴を自動保存し、いつでも巻き戻せます。",
        width: CellWidth::Narrow,
    },
    Cell {
        title: "Cross-team Reporting",
        description: "部門をまたいだ進捗を 1 枚のレポートにまとめ、共有の手間を減らします。",
        width: CellWidth::Wide,
    },
];

/// 見出しエリア（eyebrow badge + `<h3>` + リード文）。
fn header() -> Node {
    div(
        vec![("class", "blocks-bento-asymmetric-rows-header")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-bento-asymmetric-rows-eyebrow", "")],
                vec![text("Platform")],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-bento-asymmetric-rows-title", "")],
                vec![text("チームの仕事をひとつの流れにまとめる")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Md,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-bento-asymmetric-rows-lead", "")],
                vec![text(
                    "分断されがちな作業を、幅の異なるカードで用途ごとに見渡せるようにしました。",
                )],
            ),
        ],
    )
}

/// 1 枚分の bento セル（`card`。画像を上、見出しと説明を下に置く）。
fn cell(item: &Cell) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-bento-asymmetric-rows-cell", item.width.value())],
        vec![
            card::cover(
                vec![("class", "blocks-bento-asymmetric-rows-cover")],
                vec![image::image(
                    &ImageProps {
                        aspect_ratio: AspectRatio::Landscape,
                        ..ImageProps::new(
                            dummy_assets::SCREENSHOT_SRC,
                            "機能のプレースホルダー画像",
                        )
                    },
                    vec![],
                )],
            ),
            card::body(
                vec![],
                vec![
                    heading::heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(item.title)],
                    ),
                    card::description(vec![], vec![text(item.description)]),
                ],
            ),
        ],
    )
}

/// `bento-asymmetric-rows` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。見出しエリアの下に 6 列（lg 以上）/2 列（md 以上）/1 列
/// （md 未満）で切り替わるグリッドを置く。
pub fn demo() -> Node {
    let cells: Vec<Node> = CELLS.iter().map(cell).collect();
    div(
        vec![("class", "blocks-bento-asymmetric-rows")],
        vec![
            header(),
            div(vec![("class", "blocks-bento-asymmetric-rows-grid")], cells),
        ],
    )
}
```

## 原案差分メモ

本ページは基準形（対応表 ID R0769）のみを実装しています。出典の固有名・
ファイル名・取得手段は記載しません。

- 残りの集約元バリエーション（R0108 / R0411 / R0412 / R0415 / R0770）は
  スコープを分割し、後続の対応で追加する予定です。
- 見出しは `h2` ではなく `h3`/`h4` を使用しました（ページ側が
  `## Demo` として `h2` を出すため）。
- 文言（見出し・リード文・カード見出し・説明文）はすべて独自に書き直し
  ました。
- 画像はビルド時生成のプレースホルダー SVG（スクリーンショット枠）に
  置き換えました。
- 配色・余白・角丸は独自実装せず、既存のテーマトークンにそのまま従います。

関連情報: [Badge](../themes/badge.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Card](../themes/card.md) /
[Image](../themes/image.md)
