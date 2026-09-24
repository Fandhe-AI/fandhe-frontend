# bento-asymmetric-rows

`fandhe-frontend-pre-styled-ui` の `badge` / `heading` / `text` / `card` /
`image` / `icon` を合成した、幅の異なるカードを 2 行に並べる bento グリッド
の合成例です。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください。

共通の見出しエリアの下に、キャプション付きで 4 つのバリエーションを縦に
並べています。幅 lg（64rem）以上ではいずれも複数列グリッドになり、幅 md
（48rem）以上 lg 未満では 2 列、md 未満では 1 列に切り替わります。各カード
は画像を上、見出しと説明文を下に置く構成です。

1. 6 列グリッドで 1 行目 4+2・2 行目 2+4 の幅違いカード（基準形）
2. 6 列グリッドで 1 行目 3+3・2 行目 2+2+2 の幅違いカード（分割形）
3. 3 列グリッドで 1 行目 2+1・2 行目 1+2 のジグザグ配置、枠なしの淡色カード
4. 6 列グリッドで 4/2/3/3 のカード配置 + 下段に 4 列の小さな feature 一覧

文言はすべて架空のもので、データ取得・送信は行わない静的な表示例です。
`<form>` は使用しません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, p, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};
use fandhe_frontend_pre_styled_ui::Size;

/// セルの幅区分（[`LAYOUT_CSS`] の `grid-column: span` を切り替える唯一の
/// 軸）。`props::*` へは昇格せず、本 block ローカルの列挙型とする
/// （`alert::Severity`/`progress::ProgressShape` と同じ判断軸）。列モード
/// （[`GridColumns`]）に対する相対幅であり、同じ属性値が列モードによって
/// 異なる `grid-column: span` へ解決される（モジュール doc「列モードと
/// `wide`/`narrow`/`half` の意味づけ」節参照）。
#[derive(Clone, Copy)]
enum CellWidth {
    /// 6 列中 4 列分・3 列中 2 列分。
    Wide,
    /// 6 列中 2 列分・3 列中 1 列分。
    Narrow,
    /// 6 列中 3 列分（3 列モードのバリエーションでは使わない）。
    Half,
}

impl CellWidth {
    /// [`LAYOUT_CSS`] の `[data-blocks-bento-asymmetric-rows-cell="..."]`
    /// セレクタと一致させる値。
    const fn value(self) -> &'static str {
        match self {
            CellWidth::Wide => "wide",
            CellWidth::Narrow => "narrow",
            CellWidth::Half => "half",
        }
    }
}

/// グリッドの列モード（[`grid`] ヘルパの引数）。`wide`/`narrow`/`half` の
/// 意味づけを列モードごとに切り替える（モジュール doc 参照）。
#[derive(Clone, Copy)]
enum GridColumns {
    /// 6 列グリッド（基準形・分割形・混在形）。
    Six,
    /// 3 列グリッド（ジグザグ形）。
    Three,
}

impl GridColumns {
    /// [`LAYOUT_CSS`] の
    /// `[data-blocks-bento-asymmetric-rows-columns="..."]` セレクタと
    /// 一致させる値。
    const fn value(self) -> &'static str {
        match self {
            GridColumns::Six => "six",
            GridColumns::Three => "three",
        }
    }
}

/// 1 枚分のセルデータ（架空の SaaS 機能名 + 1 行説明 + 幅区分）。
struct Cell {
    title: &'static str,
    description: &'static str,
    width: CellWidth,
}

/// 1 番目のバリエーション（基準形・R0769）: 1 行目 4+2、2 行目 2+4。
/// 並び順がグリッドの既定の自動配置（`grid-auto-flow: row`）と組み合わ
/// さってこの配置を作る不変条件を、本ファイル末尾の `#[cfg(test)]` が
/// 固定する。
const BASE_CELLS: [Cell; 4] = [
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

/// 2 番目のバリエーション（分割形・R0412/R0770）: 1 行目 3+3（`half` 2
/// 枚）、2 行目 2+2+2（`narrow` 3 枚）。
const SPLIT_CELLS: [Cell; 5] = [
    Cell {
        title: "Shared Roadmap",
        description: "四半期の計画をチーム全員が同じ画面で追跡します。",
        width: CellWidth::Half,
    },
    Cell {
        title: "Automated Backups",
        description: "スナップショットを自動生成し、復元の手間を省きます。",
        width: CellWidth::Half,
    },
    Cell {
        title: "Custom Fields",
        description: "案件ごとに必要な項目だけを追加できます。",
        width: CellWidth::Narrow,
    },
    Cell {
        title: "Audit Trail",
        description: "誰が何を変更したかをいつでも確認できます。",
        width: CellWidth::Narrow,
    },
    Cell {
        title: "Bulk Actions",
        description: "複数件の操作をまとめて一度に実行します。",
        width: CellWidth::Narrow,
    },
];

/// 3 番目のバリエーション（ジグザグ形・R0411/R0415）: 3 列モードで
/// 1 行目 2+1、2 行目 1+2（[`GridColumns::Three`] + [`CardVariant::Subtle`]
/// と組み合わせて使う）。
const ZIGZAG_CELLS: [Cell; 4] = [
    Cell {
        title: "Quick Filters",
        description: "条件を組み合わせて必要な情報だけを絞り込みます。",
        width: CellWidth::Wide,
    },
    Cell {
        title: "Saved Views",
        description: "よく使う表示設定を保存していつでも呼び出せます。",
        width: CellWidth::Narrow,
    },
    Cell {
        title: "Inline Comments",
        description: "作業中の項目に直接コメントを残せます。",
        width: CellWidth::Narrow,
    },
    Cell {
        title: "Export to CSV",
        description: "表示中のデータをそのままファイルへ書き出せます。",
        width: CellWidth::Wide,
    },
];

/// 4 番目のバリエーション（混在形・R0108 上段）: 4/2/3/3
/// （`wide`/`narrow`/`half`/`half`）。下段は [`FEATURES`]/[`features`]。
const MIXED_CELLS: [Cell; 4] = [
    Cell {
        title: "Realtime Sync",
        description: "変更内容をチーム全員の画面へ即座に反映します。",
        width: CellWidth::Wide,
    },
    Cell {
        title: "Access Control",
        description: "役割ごとに閲覧・編集の権限を分けられます。",
        width: CellWidth::Narrow,
    },
    Cell {
        title: "Usage Insights",
        description: "利用状況を簡単なグラフで把握できます。",
        width: CellWidth::Half,
    },
    Cell {
        title: "Integration Hub",
        description: "外部サービスとの連携をひとつの画面で管理します。",
        width: CellWidth::Half,
    },
];

/// feature 一覧の 1 項目（アイコン + タイトル + 説明文）。
struct Feature {
    title: &'static str,
    description: &'static str,
}

/// 混在形（R0108）下段の 4 列 feature 一覧データ。
const FEATURES: [Feature; 4] = [
    Feature {
        title: "Fast Setup",
        description: "数分でチームの利用を開始できます。",
    },
    Feature {
        title: "Reliable Uptime",
        description: "安定した稼働を継続的に監視しています。",
    },
    Feature {
        title: "Flexible Plans",
        description: "チーム規模に合わせて契約内容を選べます。",
    },
    Feature {
        title: "Responsive Support",
        description: "問い合わせには迅速に対応します。",
    },
];

/// feature 一覧用の自作の抽象チェックマーク図形（装飾用途のため
/// `aria-hidden`）。参照元のアイコン形状・内部識別子は持ち込まない。
fn feature_icon() -> Node {
    icon(
        &IconProps {
            size: Size::Sm,
            label: None,
            ..IconProps::default()
        },
        vec![],
        vec![
            el(
                "circle",
                vec![
                    ("cx", "12"),
                    ("cy", "12"),
                    ("r", "9"),
                    ("fill", "none"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "1.5"),
                ],
                vec![],
            ),
            el(
                "path",
                vec![
                    ("d", "M8 12.5l2.5 2.5L16 9"),
                    ("stroke", "currentColor"),
                    ("stroke-width", "1.5"),
                    ("stroke-linecap", "round"),
                    ("stroke-linejoin", "round"),
                    ("fill", "none"),
                ],
                vec![],
            ),
        ],
    )
}

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

/// 1 行のキャプション（`h2`/`h3` は使わない。右目次・折りたたみ目次が
/// 拾ってしまうため、`banner_full_width_bar`/`footer_newsletter` と同じ
/// 判断で素の `p` を使う）。
fn caption(label: &str) -> Node {
    p(
        vec![("data-blocks-bento-asymmetric-rows-caption", "")],
        vec![text(label)],
    )
}

/// 1 枚分の bento セル（`card`。画像を上、見出しと説明を下に置く）。
/// `variant` はカードの見た目（ジグザグ形のみ [`CardVariant::Subtle`]）。
fn cell(item: &Cell, variant: CardVariant) -> Node {
    card::root(
        CardProps {
            variant,
            ..CardProps::default()
        },
        vec![("data-blocks-bento-asymmetric-rows-cell", item.width.value())],
        vec![
            card::cover(
                vec![("class", "blocks-bento-asymmetric-rows-cover")],
                vec![image::image(
                    &ImageProps {
                        aspect_ratio: AspectRatio::Landscape,
                        // 全セルが同一のプレースホルダー画像を使い、内容を
                        // 伝えない同一文言の alt を持たせると支援技術で
                        // 同じ文言が繰り返し読み上げられる（WCAG 1.1.1
                        // 違反、Bugbot 指摘 #3162）。装飾用途として空の
                        // alt にする（`blog_list_image` /
                        // `bento_three_column_tall` / `login_04` の
                        // 同型プレースホルダー画像と同じ判断）。
                        ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
                    },
                    vec![],
                )],
            ),
            card::body(
                vec![("class", "blocks-bento-asymmetric-rows-body")],
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

/// 1 個の bento グリッド（`columns` に応じて `wide`/`narrow`/`half` の
/// 意味づけが変わる、モジュール doc「列モードと `wide`/`narrow`/`half`
/// の意味づけ」節参照）。`variant` は全セル共通のカード見た目。
fn grid(columns: GridColumns, variant: CardVariant, cells: &[Cell]) -> Node {
    let rendered: Vec<Node> = cells.iter().map(|item| cell(item, variant)).collect();
    div(
        vec![
            ("class", "blocks-bento-asymmetric-rows-grid"),
            ("data-blocks-bento-asymmetric-rows-columns", columns.value()),
        ],
        rendered,
    )
}

/// 混在形（4 番目のバリエーション）下段の 4 列 feature 一覧。`card` は
/// 使わず、アイコン + 太字タイトル + 説明文を縦に並べる軽い一覧にする
/// （モジュール doc「feature 一覧と `icon` の扱い」節参照）。
fn features() -> Node {
    let items: Vec<Node> = FEATURES
        .iter()
        .map(|item| {
            div(
                vec![("data-blocks-bento-asymmetric-rows-feature", "")],
                vec![
                    div(
                        vec![("class", "blocks-bento-asymmetric-rows-feature-title")],
                        vec![
                            feature_icon(),
                            styled_text::text(
                                &TextProps {
                                    size: TextSize::Sm,
                                    weight: TextWeight::Semibold,
                                    ..TextProps::default()
                                },
                                vec![],
                                vec![text(item.title)],
                            ),
                        ],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(item.description)],
                    ),
                ],
            )
        })
        .collect();
    div(
        vec![("class", "blocks-bento-asymmetric-rows-features")],
        items,
    )
}

/// `bento-asymmetric-rows` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。見出しエリアの下に、キャプション付きで 4 つのバリエー
/// ション（モジュール doc「4 バリエーションと対応表 ID の対応」節）を
/// 縦に並べる。ルートの class は [`BLOCK::demo_class`]（外側の
/// `.blocks-demo` ラッパへ付与される）と意図的に別名にする。同名だと
/// ラッパ div にも [`LAYOUT_CSS`] の `display: flex` 規則が二重適用
/// されてしまうため（`banner_full_width_bar`/`footer_newsletter` の
/// `-stack` 命名と同じ判断）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-bento-asymmetric-rows-stack")],
        vec![
            header(),
            caption("6 columns · 4+2 / 2+4"),
            grid(GridColumns::Six, CardVariant::Outline, &BASE_CELLS),
            caption("6 columns · 3+3 / 2+2+2"),
            grid(GridColumns::Six, CardVariant::Outline, &SPLIT_CELLS),
            caption("3 columns · 2+1 / 1+2 zigzag · subtle cards"),
            grid(GridColumns::Three, CardVariant::Subtle, &ZIGZAG_CELLS),
            caption("6 columns · 4/2/3/3 + 4-column feature list"),
            grid(GridColumns::Six, CardVariant::Outline, &MIXED_CELLS),
            features(),
        ],
    )
}
```

## 原案差分メモ

出典の固有名・ファイル名・取得手段は記載しません。記載してよいのは対応表
ID のみです。

- 4 つのバリエーションと対応表 ID の対応は次のとおりです。
  1. 基準形（R0769）: 6 列グリッドで 1 行目 4+2・2 行目 2+4
  2. 分割形（R0412/R0770）: 6 列グリッドで 1 行目 3+3（`half` 2 枚）・
     2 行目 2+2+2（`narrow` 3 枚）
  3. ジグザグ形（R0411/R0415）: 3 列グリッドで 1 行目 2+1・2 行目 1+2、
     枠なし淡色カード
  4. 混在形（R0108）: 6 列グリッドで 4/2/3/3 + 下段の 4 列 feature 一覧
- R0411 の枠あり版は構造が枠なし版（R0415）と同じであるため 1 インスタン
  スへ集約し、淡色カード側で見せました。
- R0108 は元の並び（本文が先・画像が後、淡色カード）を、親仕様の「画像が
  上」と枠ありカードに統一しました。
- 見出しは `h2` ではなく `h3`/`h4` を使用しました（ページ側が `## Demo`
  として `h2` を出すため）。バリエーション間の区切りにも `h2`/`h3` は使わ
  ず、右目次・折りたたみ目次に拾われない素の `p`（キャプション）にしまし
  た。
- feature 一覧のアイコンは自作の抽象チェックマーク図形にしました。
- 文言（見出し・リード文・カード見出し・説明文・feature タイトル/説明文）
  はすべて独自に書き起こしました。実企業名・実サービス名・PII は含みませ
  ん。
- 画像はビルド時生成のプレースホルダー SVG（スクリーンショット枠）に
  置き換えました。
- 配色・余白・角丸は独自実装せず、既存のテーマトークンにそのまま従います。

関連情報: [Badge](../themes/badge.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Card](../themes/card.md) /
[Image](../themes/image.md) / [Icon](../themes/icon.md)
