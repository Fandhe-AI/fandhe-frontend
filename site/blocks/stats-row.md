# stats-row

`fandhe-frontend-pre-styled-ui` の `badge` / `heading` / `text` / `stat` /
`separator` / `image` / `icon` の 7 部品を合成した、数値指標を横一列に
並べるマーケティングセクションの合成例です。Blocks セクションは新規部品を
追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集
であることに注意してください。

見せ方の違う 4 インスタンスを併記します。

1. アイブロウ badge → 見出し → リード文 → 4 指標を中央寄せで横一列に
   並べる基準形。
2. 見出しの下を 2 列本文にしたあと横罫線を挟み、指標ごとに上罫線を付けた
   4 指標行を続ける形。
3. 見出しなしで、1 つの枠の中に 3 指標を縦罫線で区切って並べる形。評価
   指標には星アイコンを添えます。
4. 見出し・枠なしで、値とラベルを 1 行文にした指標をロゴの上に重ねる
   4 列グリッド。

レスポンシブは共通で、md 未満（`< 48rem`）で 2 列、sm 未満（`< 40rem`）で
1 列に切り替わります。インスタンス 3 の縦罫線は md 未満で非表示にし、
代わりに指標が折り返して並びます。

`stat` の DOM 構造（`label` → `value_text` の順、`<dl>`/`<dt>`/`<dd>`）は
全インスタンスで変えていません。「値が上・ラベルが下」や「値 ラベルの
1 行文」といった見た目の違いは CSS の `flex-direction`（`column-reverse`/
`row-reverse`）のみで表現しており、スクリーンリーダーの読み上げ順は
常に「ラベル → 値」のままです。

いずれのインスタンスも JS を使わない静的な表示です。`<form>` は含まず、
ロゴはビルド時生成のプレースホルダー SVG、指標の数値・社名はすべて
架空のものです。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    self as styled_heading, HeadingLevel, HeadingProps, HeadingSize,
};
use fandhe_frontend_pre_styled_ui::icon::{self, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::{Orientation, Size};

/// インスタンス間のキャプション（`logo_cloud_marquee` と同じ「短い前置き
/// テキスト」形式）。
fn caption(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            size: TextSize::Sm,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// 縦積み表示の指標 1 件（`value` が視覚的に上、`label` が下）。
/// `topline` が `true` のときは指標ごとの上罫線を追加する（インスタンス 2）。
fn stat_vertical(value: &'static str, label: &'static str, topline: bool) -> Node {
    let mut attrs: Vec<(&str, &str)> = vec![("data-blocks-stats-row-stat", "")];
    if topline {
        attrs.push(("data-blocks-stats-row-stat-topline", ""));
    }
    stat::root(
        Size::Lg,
        attrs,
        vec![
            stat::label(vec![], vec![text(label)]),
            stat::value_text(vec![], vec![text(value)]),
        ],
    )
}

/// 1 行文表示の指標 1 件（`value ラベル` の横並び、インスタンス 4）。
fn stat_inline(value: &'static str, label: &'static str) -> Node {
    stat::root(
        Size::Md,
        vec![("data-blocks-stats-row-stat-inline", "")],
        vec![
            stat::label(vec![], vec![text(label)]),
            stat::value_text(vec![], vec![text(value)]),
        ],
    )
}

/// 装飾用の星形アイコン（評価指標に添える。参照元のアイコンは使わない
/// 自作の幾何 path）。
fn star_icon() -> Node {
    icon::icon(
        &IconProps {
            size: Size::Sm,
            ..IconProps::default()
        },
        vec![],
        vec![el(
            "path",
            vec![(
                "d",
                "M12 2l2.9 6.6 7.1.6-5.4 4.7 1.7 7-6.3-3.9-6.3 3.9 1.7-7-5.4-4.7 7.1-.6z",
            )],
            vec![],
        )],
    )
}

/// インスタンス 1（基準形）: アイブロウ badge → 見出し → リード文 → 4 指標。
fn instance_basic() -> Node {
    let eyebrow = badge::badge(&BadgeProps::default(), vec![], vec![text("導入実績")]);
    let title = styled_heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl3,
            ..HeadingProps::default()
        },
        vec![],
        vec![text("数字で見る導入効果")],
    );
    let lead = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "多くのチームが日々のワークフローで実感している成果を、4 つの指標でまとめました。",
        )],
    );
    let header = div(
        vec![("class", "blocks-stats-row-header")],
        vec![eyebrow, title, lead],
    );
    let grid = div(
        vec![("class", "blocks-stats-row-grid-4")],
        vec![
            stat_vertical("1,200+", "導入チーム", false),
            stat_vertical("99.9%", "稼働率", false),
            stat_vertical("120ms", "平均応答", false),
            stat_vertical("4.8", "満足度スコア", false),
        ],
    );
    div(
        vec![("class", "blocks-stats-row-instance")],
        vec![header, grid],
    )
}

/// インスタンス 2: 見出し → 2 列本文 → 横罫線 → 指標ごと上罫線の 4 指標行。
fn instance_two_col() -> Node {
    let title = styled_heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl2,
            ..HeadingProps::default()
        },
        vec![],
        vec![text("導入後のサポート体制")],
    );
    let col_a = styled_text::text(
        &TextProps::default(),
        vec![],
        vec![text(
            "要件整理から本番稼働まで、専任チームが平均 3 週間で伴走します。",
        )],
    );
    let col_b = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "稼働後も定例ミーティングで状況を確認し、継続的な改善を提案します。",
        )],
    );
    let two_col = div(
        vec![("class", "blocks-stats-row-two-col")],
        vec![col_a, col_b],
    );
    let rule = separator::separator(&SeparatorProps::default(), vec![]);
    let grid = div(
        vec![("class", "blocks-stats-row-grid-4")],
        vec![
            stat_vertical("32", "導入業種数", true),
            stat_vertical("18", "対応言語数", true),
            stat_vertical("6", "リージョン数", true),
            stat_vertical("24/7", "サポート体制", true),
        ],
    );
    div(
        vec![("class", "blocks-stats-row-instance")],
        vec![title, two_col, rule, grid],
    )
}

/// インスタンス 3（見出しなし）: 枠 + 縦罫線区切りの 3 指標。評価指標には
/// 星アイコンを添える。
fn instance_bordered() -> Node {
    let rating_label = stat::label(vec![], vec![text("平均評価")]);
    let rating_value = stat::value_text(vec![], vec![text("4.9"), star_icon()]);
    let rating = stat::root(
        Size::Lg,
        vec![("data-blocks-stats-row-stat", "")],
        vec![rating_label, rating_value],
    );
    let vsep = || {
        separator::separator(
            &SeparatorProps {
                orientation: Orientation::Vertical,
                ..SeparatorProps::default()
            },
            vec![],
        )
    };
    let row = div(
        vec![("class", "blocks-stats-row-panel-row")],
        vec![
            stat_vertical("1,024", "導入プロジェクト", false),
            vsep(),
            rating,
            vsep(),
            stat_vertical("87", "導入国・地域数", false),
        ],
    );
    div(vec![("class", "blocks-stats-row-panel")], vec![row])
}

/// ロゴ 1 件分のセル（値+ラベルの 1 行文 + 下にロゴ）。
fn logo_cell(value: &'static str, label: &'static str) -> Node {
    let row = stat_inline(value, label);
    let logo = image::image(
        &ImageProps::new(dummy_assets::LOGO_SRC, ""),
        vec![("data-blocks-stats-row-logo", "")],
    );
    div(
        vec![("class", "blocks-stats-row-logo-cell")],
        vec![row, logo],
    )
}

/// インスタンス 4（見出し・枠なし）: 1 行文の指標 + ロゴの 4 列グリッド。
fn instance_logo_grid() -> Node {
    let grid = div(
        vec![("class", "blocks-stats-row-logo-grid")],
        vec![
            logo_cell("2.4x", "ROI 改善"),
            logo_cell("40%", "運用工数削減"),
            logo_cell("15 分", "導入所要時間"),
            logo_cell("98%", "継続利用率"),
        ],
    );
    div(vec![("class", "blocks-stats-row-instance")], vec![grid])
}

/// `stats-row` の Demo 本体。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-stats-row-stack")],
        vec![
            caption("基準形（アイブロウ + 見出し + 4 指標）"),
            instance_basic(),
            caption("見出し + 2 列本文 → 上罫線付き指標行"),
            instance_two_col(),
            caption("枠 + 縦罫線区切り + 星アイコン（見出しなし）"),
            instance_bordered(),
            caption("1 行文の指標 + ロゴ（見出し・枠なし）"),
            instance_logo_grid(),
        ],
    )
}
```

## 原案差分メモ

- **R0336（主参照・基準形）**: アイブロウ badge → 見出し → リード文 → 4
  指標の横一列として、インスタンス 1 にそのまま採用しました。
- **R0703**: 指標 3 件の基準形はインスタンス 3（枠 + 縦罫線区切り）として
  読み替えています。
- **R0335 / R1311**: 指標間の縦罫線区切りはインスタンス 3 の `separator`
  （`Orientation::Vertical`）で表現しています。
- **R0342**: 指標ごとの上罫線はインスタンス 2 の
  `data-blocks-stats-row-stat-topline` で表現し、指標行の直前に横向きの
  `separator` を 1 本挟んでいます。
- **R1305**: 見出しの下を 2 列本文にしてから指標行を続ける形は
  インスタンス 2 にそのまま採用しました。
- **R0340 / R1308**: 値とラベルの 1 行文 + 指標の下のロゴは、見出し・枠
  なしの 4 列グリッド（インスタンス 4）として実装しました。`image` は
  `<dl>` の外（セルの `div` 側）に置いており、`stat` の内容モデル
  （`<dt>`/`<dd>` のみ）を崩していません。
- **R0707 / R1299**: 評価指標への星付与はインスタンス 3 の 1 件のみに
  採用し、自作の幾何アイコン（星形の `path`）で表現しています。参照元の
  アイコンそのものは使っていません。
- 参照元の文言・配色・装飾・アイコン・内部識別子は持ち込んでいません。
  指標の数値・社名・ロゴはすべて架空のものです。
