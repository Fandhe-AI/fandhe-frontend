# logo-cloud-marquee

`fandhe-frontend-pre-styled-ui` の `heading` / `text` / `marquee` / `image` /
`card` の 5 部品を合成した、多くの参照サイトに共通するロゴクラウド
（導入企業ロゴをティッカー状に流す帯）の合成例です。Blocks セクションは
新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた
実例集であることに注意してください。

構成は 2 通りを併記します。1 つ目は帯 1 段のみ・両端フェード付き（基準形）、
2 つ目はカードの中に同じ 6 社のロゴを逆方向へ流す 2 段を重ねた構成です。
2 段目は 1 段目と同一内容の再掲のため装飾（`aria-hidden`）扱いとし、
スクリーンリーダーでの二重読み上げを防ぎます。

アニメーション・両端フェード・`prefers-reduced-motion: reduce` 時の折り返し
表示・hover/focus 時の一時停止は、いずれも `marquee` 部品自体が内蔵する
契約です。本 Demo は `--fandhe-marquee-*` の custom property でフェード幅・
間隔・速度を上書きするのみで、アニメーションに関する CSS は書きません。
ロゴはプレースホルダー SVG、社名はすべて架空のものであり、実企業名・
実データ・`<form>` は含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::card::{self, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    self as styled_heading, HeadingLevel, HeadingProps, HeadingSize,
};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::marquee::{self, MarqueeDirection, MarqueeProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 見出しエリア（見出し → リード文）。
fn header() -> Node {
    let title = styled_heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl3,
            ..HeadingProps::default()
        },
        vec![],
        vec![text("多くのチームに使われています")],
    );
    let lead = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "様々な規模のチームが日々のワークフローに組み込んでいます。",
        )],
    );
    div(
        vec![("class", "blocks-logo-cloud-marquee-header")],
        vec![title, lead],
    )
}

/// ロゴ + 社名のロックアップ 1 件（[`marquee::item`] 1 個）。
fn logo(name: &'static str) -> Node {
    let mark = image::image(
        &ImageProps::new(dummy_assets::LOGO_SRC, ""),
        vec![("data-blocks-logo-cloud-marquee-logo", "")],
    );
    let label = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            size: TextSize::Sm,
            ..TextProps::default()
        },
        vec![],
        vec![text(name)],
    );
    marquee::item(
        vec![],
        vec![div(
            vec![("class", "blocks-logo-cloud-marquee-lockup")],
            vec![mark, label],
        )],
    )
}

/// marquee 1 段分。`decorative` が `true` のときは同一 6 社の逆方向再掲
/// （二重読み上げ防止のため装飾扱い）、`false` のときはアクセシブル
/// ネームを付与する。`attr` は [`LAYOUT_CSS`] 側のトークン上書きに使う
/// data 属性名。
fn row(direction: MarqueeDirection, decorative: bool, attr: &'static str) -> Node {
    marquee::marquee(
        &MarqueeProps {
            direction,
            decorative,
            label: (!decorative).then_some("導入企業のロゴ"),
        },
        vec![(attr, "")],
        dummy_assets::COMPANY_NAMES
            .iter()
            .map(|n| logo(n))
            .collect(),
    )
}

/// `logo-cloud-marquee` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    let band_caption = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            size: TextSize::Sm,
            ..TextProps::default()
        },
        vec![],
        vec![text("1 段・両端フェード")],
    );
    let band = row(
        MarqueeDirection::Start,
        false,
        "data-blocks-logo-cloud-marquee-band",
    );

    let card_caption = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            size: TextSize::Sm,
            ..TextProps::default()
        },
        vec![],
        vec![text("カード内・逆方向 2 段")],
    );
    let card = card::root(
        CardVariant::default(),
        vec![("data-blocks-logo-cloud-marquee-card", "")],
        vec![card::body(
            vec![("class", "blocks-logo-cloud-marquee-card-body")],
            vec![
                row(
                    MarqueeDirection::Start,
                    false,
                    "data-blocks-logo-cloud-marquee-card-row",
                ),
                row(
                    MarqueeDirection::End,
                    true,
                    "data-blocks-logo-cloud-marquee-card-row",
                ),
            ],
        )],
    );

    div(
        vec![("class", "blocks-logo-cloud-marquee-stack")],
        vec![header(), band_caption, band, card_caption, card],
    )
}
```

## 原案差分メモ

- **R0148（基準形: ロゴを横一列に並べて marquee で流す）**: 1 段目の帯として
  そのまま実装しました。速度・間隔は `--fandhe-marquee-duration`/
  `--fandhe-marquee-gap`（秒指定の custom property）で与えます。参照元の
  `speed`（px/s 指定、要素幅に依存）は `marquee` 部品の契約外のため
  採用していません。
- **R0570（両端フェード）**: 個別の CSS ではなく、`marquee` 部品が既に
  持つ `--fandhe-marquee-fade`（`mask-image` によるフェード）へ統合しました。
  帯（フェード幅 `--fandhe-space-16`）とカード内 2 段（`--fandhe-space-12`）
  で異なる幅を与え、単なる複製ではないことを示しています。
- **R0147（カードの中に逆方向へ流れる 2 段）**: `card` の中へ `marquee` を
  2 個重ねて実装しました。2 段目は 1 段目と同一の 6 社を逆方向
  （`MarqueeDirection::End`）で再掲するため、二重読み上げ防止の観点から
  `decorative: true`（`aria-hidden` + `inert`）を付与しています。
- 参照元の文言・配色・実在ロゴ・内部識別子は持ち込んでいません。ロゴは
  ビルド時生成のプレースホルダー SVG、社名はすべて架空のものです。
