# bento-two-column

見出し帯（eyebrow badge + 見出し + リード文）の下に、`fandhe-frontend-pre-styled-ui`
の `card` / `image` / `heading` / `text` / `badge` を合成した 2 列の bento
グリッドです。各カードは「見出しと説明」が上、「画像」が下の縦積みで、狭い
画面（768px 未満）では 1 列に積み替わります。Blocks セクションは新規部品を
追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集
であることに注意してください。

集約元には 2 つの形があります。基準形はカード 4 枚がすべて同じ幅の 2 列 ×
2 行です。もう一方の形は先頭のカードだけがグリッド全幅に広がり、768px 以上
では見出しと画像が横並びになります。Demo はこの 2 形を上下に並べて掲載し、
差分を読み取れるようにしています。

画像はビルド時生成のモノトーン抽象図形（プレースホルダー）で、機能名・説明
文はすべて架空のものです。`<form>` は使用しません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 1 枚分のカードデータ（架空の開発者向けプラットフォームの機能紹介、
/// 実企業名・実サービス名は含まない）。
struct BentoCell {
    title: &'static str,
    description: &'static str,
    image_src: &'static str,
}

/// 基準形（4 枚すべて同じ幅）で使うカード 4 件（架空、実データなし）。
/// 全幅の形（[`demo`] 内 2 段目）は本配列の先頭 2 件を再利用する。
const BASE_CELLS: [BentoCell; 4] = [
    BentoCell {
        title: "アクセス解析",
        description: "利用状況を可視化し、改善点を素早く把握します。",
        image_src: dummy_assets::SCREENSHOT_SRC,
    },
    BentoCell {
        title: "外部連携",
        description: "既存の業務ツールとシームレスに接続します。",
        image_src: dummy_assets::PRODUCT_SRC,
    },
    BentoCell {
        title: "チーム招待",
        description: "メンバーを招待し、役割ごとに権限を割り当てます。",
        image_src: dummy_assets::LOGO_SRC,
    },
    BentoCell {
        title: "バックアップ",
        description: "定期的なスナップショットでデータ損失に備えます。",
        image_src: dummy_assets::BACKGROUND_SRC,
    },
];

/// 全幅の形（2 段目）の先頭に置く強調カード。
const FEATURED_CELL: BentoCell = BentoCell {
    title: "統合ワークスペース",
    description: "複数プロジェクトの状況をひとつの画面へ集約して表示します。",
    image_src: dummy_assets::PRODUCT_SRC,
};

/// 見出し帯（eyebrow badge + 見出し + リード文）。
fn intro() -> Node {
    div(
        vec![("class", "blocks-bento-two-column-intro")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![],
                vec![text("機能ハイライト")],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("必要な機能をわかりやすく整理")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "主要な機能をカードごとにまとめた一覧です。運用に必要な要素を素早く見渡せます。",
                )],
            ),
        ],
    )
}

/// 1 枚分の bento カード（`card` + 見出し/説明 + カバー画像）を組み立てる。
/// `cell_kind` は `"base"`（基準形の等幅カード）または `"featured"`
/// （全幅に広がる強調カード）で、[`LAYOUT_CSS`] の
/// `[data-blocks-bento-two-column-cell]` セレクタの値と一致させる。
fn cell(item: &BentoCell, cell_kind: &'static str) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-bento-two-column-cell", cell_kind)],
        vec![
            card::header(
                vec![("class", "blocks-bento-two-column-text")],
                vec![
                    heading::heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(item.title)],
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
            ),
            card::cover(
                vec![("class", "blocks-bento-two-column-media")],
                vec![image::image(
                    &ImageProps {
                        aspect_ratio: AspectRatio::Video,
                        ..ImageProps::new(item.image_src, "")
                    },
                    vec![],
                )],
            ),
        ],
    )
}

/// 「基準形」または「全幅の形」1 段分（小見出し + グリッド）を組み立てる。
fn variant_section(label: &'static str, cells: Vec<Node>) -> Node {
    div(
        vec![("class", "blocks-bento-two-column-variant")],
        vec![
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(label)],
            ),
            div(vec![("class", "blocks-bento-two-column-grid")], cells),
        ],
    )
}

/// `bento-two-column` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。見出し帯 + 「基準形」（4 枚等幅）+ 「先頭カードを全幅にした形」
/// （featured 1 枚 + base 2 枚）の 2 段を上下に並べ、集約元の 2 形の差分を
/// 1 つの Demo 内で読み取れるようにする（モジュール doc「レイアウト」節）。
pub fn demo() -> Node {
    let base_cells: Vec<Node> = BASE_CELLS.iter().map(|item| cell(item, "base")).collect();
    let featured_cells: Vec<Node> = std::iter::once(cell(&FEATURED_CELL, "featured"))
        .chain(BASE_CELLS.iter().take(2).map(|item| cell(item, "base")))
        .collect();
    div(
        vec![("class", "blocks-bento-two-column")],
        vec![
            intro(),
            variant_section("基準形（4 枚が同じ幅）", base_cells),
            variant_section(
                "先頭カードを全幅にした形（48rem 以上で見出しと画像が横並び）",
                featured_cells,
            ),
        ],
    )
}
```

**原案差分メモ**

集約元の 2 形（対応表 ID は非公開のローカル対応表を参照）からの差分は
次のとおりです。

- 基準形（カード 4 枚が同じ幅）と、先頭カードのみグリッド全幅に広げた形
  （768px 以上で見出しと画像を横並びにする）を、1 つの Demo 内へ上下 2 段
  で並記しました。参照元では別々の Figma フレームに分かれていましたが、
  差分をこのページ単体で確認できるよう統合しています。
- 参照元のブレークポイント（64rem）は使わず、イシュー指定の md
  （`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md` = 768px = 48rem）
  に合わせました。
- カード内の画像は `card::cover` を末尾（説明の後）に置く構成のため、
  上端ではなく下端（全幅カードでは右端）の角丸のみを丸める CSS 上書きを
  独自に追加しています。
- 文言（見出し・説明文）・画像はすべて独自に書き直した架空のものです。
  実在の企業名・サービス名・ロゴ・配色は持ち込んでいません。
- `id`・`aria-controls`・`aria-labelledby`・`aria-describedby`・`href`・
  `<a>`・`<button>`・`<form>` はいずれも出力しません（宙に浮いた ARIA 参照・
  id 重複・不要な対話要素を構造的に避けるため）。

関連情報: [Badge](../themes/badge.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Card](../themes/card.md) / [Image](../themes/image.md)
