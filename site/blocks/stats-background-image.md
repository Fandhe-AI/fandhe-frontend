# stats-background-image

`badge` / `heading` / `text` / `stat` / `image` の合成例（既存部品のみで組
んだ、背景画像の上に数値指標を並べるセクションです）。Blocks セクションは
新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせ
た実例集であることに注意してください（主参照は対応表 ID R0706。出典の
固有名・ファイル名は記載しません）。

Demo 領域いっぱいに背景画像を敷き、その上に暗幕（半透明のスクリム）を
重ねて、タグライン・見出し・説明文・数値指標 4 件を明るい文字で中央寄せ
なしのまま表示します。数値指標は `48rem` 未満で 2 列、`48rem` 以上で 4 列
のグリッドに並びます。背景画像は装飾扱いとして `alt=""` と
`aria-hidden="true"` を持ち、支援技術からは読み上げられません。本 Demo は
静的な表示例であり、`<form>` 要素を一切持たず、送信処理・データ取得を
行いません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps};
use fandhe_frontend_pre_styled_ui::Size;

const ROOT_CLASS: &str = "blocks-stats-background-image-root";
const BACKDROP_CLASS: &str = "blocks-stats-background-image-backdrop";
const SCRIM_CLASS: &str = "blocks-stats-background-image-scrim";
const CONTENT_CLASS: &str = "blocks-stats-background-image-content";
const GRID_CLASS: &str = "blocks-stats-background-image-grid";

const IMAGE_ATTR: &str = "data-blocks-stats-background-image-image";
const TAGLINE_ATTR: &str = "data-blocks-stats-background-image-tagline";
const TITLE_ATTR: &str = "data-blocks-stats-background-image-title";
const DESCRIPTION_ATTR: &str = "data-blocks-stats-background-image-description";
const STAT_ATTR: &str = "data-blocks-stats-background-image-stat";
const STAT_LABEL_ATTR: &str = "data-blocks-stats-background-image-stat-label";

/// 数値指標 1 件分（`stat::root` + `label`/`value_text`、`content_image_tiles`
/// と同じ合成方法）。
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

/// `stats-background-image` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    let backdrop = div(
        vec![("class", BACKDROP_CLASS), ("aria-hidden", "true")],
        vec![
            image::image(
                &ImageProps::new(dummy_assets::BACKGROUND_SRC, ""),
                vec![(IMAGE_ATTR, "")],
            ),
            div(vec![("class", SCRIM_CLASS)], vec![]),
        ],
    );

    let tagline = badge::badge(
        &BadgeProps::default(),
        vec![(TAGLINE_ATTR, "")],
        vec![text("Trusted by teams everywhere")],
    );

    let title = heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl3,
            ..HeadingProps::default()
        },
        vec![(TITLE_ATTR, "")],
        vec![text("Built for teams that never stop shipping")],
    );

    let description = styled_text::text(
        &TextProps::default(),
        vec![(DESCRIPTION_ATTR, "")],
        vec![text(
            "A snapshot of how teams rely on our platform every day.",
        )],
    );

    let grid = div(
        vec![("class", GRID_CLASS)],
        vec![
            stat_item("Active projects", "12k+"),
            stat_item("Uptime", "99.9%"),
            stat_item("Countries", "40"),
            stat_item("Avg. response", "2h"),
        ],
    );

    let content = div(
        vec![("class", CONTENT_CLASS)],
        vec![tagline, title, description, grid],
    );

    div(vec![("class", ROOT_CLASS)], vec![backdrop, content])
}
```

## 原案差分メモ

参照（対応表 ID R0706、基準形）からの意図的な差分は次のとおりです。

- 参照元は背景画像の上へ白文字を直書きしていますが、本リポジトリの
  規約（トークンのみを使い、参照素材の配色を持ち込まない）を優先し、
  暗幕を `--fandhe-color-fg` ベース、文字を `--fandhe-color-bg` ベースに
  した反転ペア（`color-mix()`）へ変更しました。ライトテーマでは暗い暗幕・
  明るい文字（要件どおり）になり、ダークテーマでは明暗が入れ替わります
  が、テーマが保証する fg/bg のコントラストは両モードで保たれます。
- 薄い背景画像の変種（対応表 ID R1301）は Demo に別インスタンスとして
  持ち込んでいません。この Demo のスクリム（`.blocks-stats-background-
  image-scrim` の `color-mix()` 比率）を下げれば、装飾ブラーを追加せずに
  同じ効果を再現できます。装飾ブラーは参照元にあっても取り込みません。
- 参照元は全画面の高さで表示していますが、本 block は Blocks セクションの
  Demo 枠内に収める必要があるため、`min-height` による枠内表示へ変更して
  います。
