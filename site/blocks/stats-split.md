# stats-split

`fandhe-frontend-pre-styled-ui` の `badge` / `heading` / `text` / `stat` /
`separator` の 5 部品のみを合成した、見出しと数値指標の 2 列の合成例です。
Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives
部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID
R0337、集約元は対応表 ID R0339・R0708・R1306 の 3 件です。出典の固有名・
ファイル名は記載しません）。

Demo は 2 インスタンスで構成しています。1 つ目（基準形）は `md`
（768px）以上で左に `badge`（タグライン）+ `heading` + `text` の見出しブ
ロック、右に `stat` を 2 列グリッドで 6 件並べ、各指標の下へ罫線を引きま
す。2 つ目（導入行 + 左罫線）は `heading` + `text` を横並びの導入行にし、
`separator`（横罫線）を挟んでから `stat` を 4 件並べ、各指標の左側へ罫線
を付けます。`md` 未満ではいずれも縦積みになります。

## Rust コード

```rust
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Orientation;

/// 数値指標 1 件分（`stat::root` + `label`/`value_text`/`value_unit`）。
/// `bordered_attr` は罫線バリアント区別用の `data-*` 属性 1 件
/// （下罫線・左罫線のいずれかを [`LAYOUT_CSS`] のセレクタで切り替える）。
fn stat_item(label: &str, value: &str, unit: &str, bordered_attr: (&str, &str)) -> Node {
    stat::root(
        Size::Md,
        vec![bordered_attr],
        vec![
            stat::label(vec![], vec![text(label)]),
            stat::value_text(
                vec![],
                vec![text(value), stat::value_unit(vec![], vec![text(unit)])],
            ),
        ],
    )
}

/// 基準形（R0337 主参照）: 左に `badge`/`heading`/`text`、右に 2 列 6 指標
/// （下罫線）。
fn base_variant() -> Node {
    let left = div(
        vec![("class", "blocks-stats-split-left")],
        vec![
            badge(&BadgeProps::default(), vec![], vec![text("実績")]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("数字で見る導入実績")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "架空のダミー指標です。導入企業数・稼働率・処理件数などの傾向を\
                     まとめて示す想定の Demo です。",
                )],
            ),
        ],
    );

    let items = [
        ("導入企業数", "1,240", "社"),
        ("稼働率", "99.9", "%"),
        ("月間処理件数", "3.6", "M件"),
        ("平均応答時間", "48", "ms"),
        ("継続利用率", "96", "%"),
        ("サポート満足度", "4.8", "/5"),
    ];
    let grid = div(
        vec![("class", "blocks-stats-split-grid")],
        items
            .iter()
            .map(|(label, value, unit)| {
                stat_item(
                    label,
                    value,
                    unit,
                    ("data-blocks-stats-split-stat-bottom", ""),
                )
            })
            .collect(),
    );

    div(
        vec![
            ("data-blocks-stats-split-row", ""),
            ("data-blocks-stats-split-variant", "base"),
        ],
        vec![left, grid],
    )
}

/// 導入行 + 左罫線（R0339）: `heading`/`text` の横並び導入行 + `separator`
/// + 4 指標（左罫線）。
fn intro_row_variant() -> Node {
    let intro = div(
        vec![("class", "blocks-stats-split-intro")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("プラットフォーム全体の状況")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("直近期間の架空サマリーです。")],
            ),
        ],
    );

    let items = [
        ("アクティブ組織", "820", "社"),
        ("新規登録", "154", "件/月"),
        ("平均処理速度", "1.2", "秒"),
        ("障害件数", "0", "件"),
    ];
    let grid = div(
        vec![("class", "blocks-stats-split-grid-intro")],
        items
            .iter()
            .map(|(label, value, unit)| {
                stat_item(
                    label,
                    value,
                    unit,
                    ("data-blocks-stats-split-stat-left", ""),
                )
            })
            .collect(),
    );

    div(
        vec![
            ("class", "blocks-stats-split-intro-row"),
            ("data-blocks-stats-split-variant", "intro-row"),
        ],
        vec![
            intro,
            separator::separator(
                &SeparatorProps {
                    orientation: Orientation::Horizontal,
                    ..SeparatorProps::default()
                },
                vec![],
            ),
            grid,
        ],
    )
}

/// `stats-split` の Demo 本体（基準形 + 導入行の 2 インスタンスを縦に
/// 並べる）。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-stats-split-layout")],
        vec![base_variant(), intro_row_variant()],
    )
}
```

## 原案差分メモ

参照（主参照は対応表 ID R0337、集約元は対応表 ID R0339・R0708・R1306。
出典の固有名・ファイル名は記載しません）から取り込んだのは構造（領域の
配置と部品構成）のみであり、次の点を独自に設計・変更しています。

- 主参照（左見出し + 右に 2 列で 6 指標）を「基準形」として実装し、指標へ
  下罫線を添えました。これは集約元の 1 つ（2×2 の 4 指標へ罫線を添える
  構成）の考え方も併せて表現したものです（`sm` 未満では 1 列 6 段へ折り
  返るため、2×2 相当の見え方も自然に共有できます）。
- 集約元の 1 つ（横並びの導入行 + 左罫線 4 指標）は「導入行 + 左罫線」と
  してそのままインスタンス化しました。
- 集約元のもう 1 つ（左に本文 2 段落 + 右に 3 指標）は Demo にインスタン
  ス化していません。基準形の左列（`badge`/`heading`/`text`）を `text` 2 段
  落へ、右列の `stat` 件数を 3 件へ差し替えるだけで同じ構造を再現できるた
  め、Demo を 2 件に留める判断をしました。
- 数値・単位・見出し・説明文はすべて独自に書いた架空のものにしました
  （実企業名・実指標・PII は含みません）。
- レイアウトの切り替えを「1 列 → `md`（768px）以上で左右 2 カラム / 横並
  び導入行」にし、導入行の指標グリッドのみ `sm`（640px）以上で 2 列の中間
  段を挟むようにしました。
- 配色は `--fandhe-color-border` 等のトークンに従わせました。
- `<form>` は出力せず、リンク・ボタンも持たない静的な表示にしました。

関連情報: [Badge](../themes/badge.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Stat](../themes/stat.md) /
[Separator](../themes/separator.md)
