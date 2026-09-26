# stats-with-image

`badge` / `heading` / `text` / `stat` / `image` の合成例（既存部品のみで組
んだ、画像と数値指標を 2 列で組み合わせるセクションです）。Blocks セクション
は新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせ
た実例集であることに注意してください（主参照は対応表 ID R0702。出典の
固有名・ファイル名は記載しません）。

画像と、見出し・説明文・数値指標 2〜4 件を持つコピー列を 2 列で組み合わせ
ます。`64rem` 未満では画像が上に、コピー列が下に来ます。`64rem` 以上では
3 形の並びが変わります: 基準形（R0702）は画像が右、反転形（R0704）は画像が
左のまま、上部全幅形（R1302）は画像がセルいっぱいの高さで左半分を占めます。
画像は装飾扱いとして `alt=""` を持ち、支援技術からは読み上げられません。
本 Demo は静的な表示例であり、`<form>` 要素を一切持たず、送信処理・データ
取得を行いません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

const IMAGE_ATTR: &str = "data-blocks-stats-with-image-image";
const STAT_ATTR: &str = "data-blocks-stats-with-image-stat";
const STAT_LABEL_ATTR: &str = "data-blocks-stats-with-image-stat-label";
const REVERSE_ATTR: &str = "data-blocks-stats-with-image-reverse";
const BLEED_ATTR: &str = "data-blocks-stats-with-image-bleed";

/// 各形の直前に置く短い形ラベル（`styled_text::text` の `Sm`/`Muted`、
/// `hero_split_image::variant_label` と同型）。
fn variant_label(label: &'static str) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(label)],
    )
}

/// 数値指標 1 件分（`stat::root` + `label`/`value_text`、
/// `stats_background_image::stat_item` と同じ合成方法）。
fn stat_item(label: &str, value: &str) -> Node {
    stat::root(
        Size::Lg,
        vec![(STAT_ATTR, "")],
        vec![
            stat::label(vec![(STAT_LABEL_ATTR, "")], vec![text(label)]),
            stat::value_text(vec![], vec![text(value)]),
        ],
    )
}

/// コピー列（badge → 見出し → リード文 → 指標グリッド）。3 形共通の構成。
fn copy(
    tagline: &'static str,
    title: &'static str,
    description: &'static str,
    stats: Vec<Node>,
) -> Node {
    div(
        vec![("class", "blocks-stats-with-image-copy")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text(tagline)]),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(description)],
            ),
            div(vec![("class", "blocks-stats-with-image-grid")], stats),
        ],
    )
}

/// 形 A（R0702 基準形）: コピー列（指標 2 件）+ 画像。`lg` 以上で画像を右へ
/// 置く（`grid-column` のみで見た目を入れ替え、DOM 順は「画像 →
/// コピー列」のまま）。
fn variant_basic() -> Node {
    let media = image::image(
        &ImageProps::new(dummy_assets::PRODUCT_SRC, ""),
        vec![(IMAGE_ATTR, "")],
    );
    let content = copy(
        "Growth at a glance",
        "The numbers behind every release",
        "A quick look at how teams rely on our platform day to day.",
        vec![
            stat_item("Customers", "8,000+"),
            stat_item("Uptime", "99.95%"),
        ],
    );
    div(
        vec![("class", "blocks-stats-with-image-row")],
        vec![media, content],
    )
}

/// 形 B（R0704 反転）: 画像を左に置く。指標は 4 件（2×2 グリッド）。DOM 順
/// どおり画像が左のままになる。
fn variant_reverse() -> Node {
    let media = image::image(
        &ImageProps::new(dummy_assets::PRODUCT_SRC, ""),
        vec![(IMAGE_ATTR, "")],
    );
    let content = copy(
        "Built to scale",
        "One dashboard, every metric that matters",
        "From onboarding to renewal, the same view keeps every team aligned.",
        vec![
            stat_item("Active teams", "1,200+"),
            stat_item("Avg. setup time", "6m"),
            stat_item("Countries", "40"),
            stat_item("Support rating", "4.9/5"),
        ],
    );
    div(
        vec![("class", "blocks-stats-with-image-row"), (REVERSE_ATTR, "")],
        vec![media, content],
    )
}

/// 形 C（R1302 上部全幅 → `lg` で左半分）: `lg` 未満は画像が横幅いっぱいで
/// 上、`lg` 以上は画像がセルいっぱいの高さで左半分を占める。指標は 3 件。
fn variant_bleed() -> Node {
    let media = image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio: AspectRatio::Video,
            ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
        },
        vec![(IMAGE_ATTR, "")],
    );
    let content = copy(
        "Always in sync",
        "See the impact before you ship it",
        "Preview usage trends alongside the change that produced them.",
        vec![
            stat_item("Requests / day", "2.4M"),
            stat_item("p95 latency", "180ms"),
            stat_item("Error budget", "99.9%"),
        ],
    );
    div(
        vec![("class", "blocks-stats-with-image-row"), (BLEED_ATTR, "")],
        vec![media, content],
    )
}

/// `stats-with-image` の Demo 本体。呼び出しごとに同一の `Node` を返す純
/// 関数（モジュール doc「3 形を 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-stats-with-image-layout")],
        vec![
            variant_label("Image right (R0702)"),
            variant_basic(),
            variant_label("Image left (R0704)"),
            variant_reverse(),
            variant_label("Full-width image → left half on lg (R1302)"),
            variant_bleed(),
        ],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0702/R0704/R1302）からの意図的な差分は次のとおりです。

- 3 形とも DOM 順を「画像 → コピー列」に固定しています。`64rem` 未満の
  1 列表示で画像が上に来る要件を、絶対配置や JS による並べ替えなしに
  満たすためです（`grid-column` の指定のみで見た目上の左右を切り替えます）。
- 画像はダミー素材（ビルド時生成の SVG）を使い、実企業名・実サービス名・
  PII を含まない架空の文言のみで数値指標を構成しています。
- 配色は本リポジトリの規約（トークンのみを使い、参照素材の配色を持ち込
  まない）に従い、`--fandhe-*` トークンのみで表現しています。
- 3 形はいずれも Blocks セクションの Demo 枠内へ収まるよう、画像は
  `100%` 幅・自動高さ（上部全幅形のみ `lg` 以上で `height: 100%`）で表示
  しています。
