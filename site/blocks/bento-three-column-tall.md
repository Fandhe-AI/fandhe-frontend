# bento-three-column-tall

両端のセルが縦 2 行にまたがる 3 列 bento グリッドです。見出し帯（eyebrow
badge + 見出し + リード文）の下に、`fandhe-frontend-pre-styled-ui` の
`card` / `image` / `heading` / `text` を合成した 4 枚のセルを並べます。
狭い画面（1024px 未満）では 1 列積みへ切り替わり、縦長セルも通常の高さに
戻ります。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください。

画像はビルド時生成のモノトーン抽象図形（プレースホルダー）で、機能名・
説明文はすべて架空のものです。CTA ボタン行やコード表示枠・端末風の枠への
差し替えは今後の実装で追加されます。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 1 枚分のセルデータ（架空の開発者向けプラットフォームの機能紹介、
/// 実企業名・実サービス名は含まない）。`slot` はグリッド内の配置を表す
/// 識別子で、[`LAYOUT_CSS`] の `[data-blocks-bento-three-column-tall-cell]`
/// セレクタの値と一致させる。
struct BentoCell {
    slot: &'static str,
    title: &'static str,
    description: &'static str,
    image_src: &'static str,
    aspect: AspectRatio,
}

/// セル 4 件（架空、実データなし）。1 枚目（`start`）・4 枚目（`end`）が
/// 縦長（[`AspectRatio::Portrait`]）、2・3 枚目（`center-top`/
/// `center-bottom`）は横長（[`AspectRatio::Video`]）にする
/// （モジュール doc「レイアウト」節参照）。
const CELLS: [BentoCell; 4] = [
    BentoCell {
        slot: "start",
        title: "統合ダッシュボード",
        description: "複数サービスの稼働状況を 1 画面へ集約して表示します。",
        image_src: dummy_assets::SCREENSHOT_SRC,
        aspect: AspectRatio::Portrait,
    },
    BentoCell {
        slot: "center-top",
        title: "自動デプロイ",
        description: "コミットからビルド・検証・配信までを自動化します。",
        image_src: dummy_assets::PRODUCT_SRC,
        aspect: AspectRatio::Video,
    },
    BentoCell {
        slot: "center-bottom",
        title: "チーム権限管理",
        description: "ロールごとに閲覧・操作範囲を細かく制御します。",
        image_src: dummy_assets::LOGO_SRC,
        aspect: AspectRatio::Video,
    },
    BentoCell {
        slot: "end",
        title: "利用量アラート",
        description: "しきい値を超えた利用量を検知し即座に通知します。",
        image_src: dummy_assets::BACKGROUND_SRC,
        aspect: AspectRatio::Portrait,
    },
];

/// 見出し帯（eyebrow badge + 見出し + リード文）。CTA 行は #2749 で追加する。
fn intro() -> Node {
    div(
        vec![("class", "blocks-bento-three-column-tall-intro")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![],
                vec![text("プラットフォーム機能")],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("必要な機能をひとつの基盤に")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "運用・デプロイ・権限管理をまとめて提供する、開発者向けプラットフォームの主要機能です。",
                )],
            ),
        ],
    )
}

/// 1 枚分の bento セル（`card` + カバー画像 + 見出し/説明）を組み立てる。
fn cell(item: &BentoCell) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-bento-three-column-tall-cell", item.slot)],
        vec![
            card::cover(
                vec![],
                vec![image::image(
                    &ImageProps {
                        aspect_ratio: item.aspect,
                        ..ImageProps::new(item.image_src, "")
                    },
                    vec![],
                )],
            ),
            card::body(
                vec![("class", "blocks-bento-three-column-tall-body")],
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
        ],
    )
}

/// `bento-three-column-tall` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（見出し帯 + 4 枚セルのグリッド、モジュール doc「レイアウト」
/// 節）。
pub fn demo() -> Node {
    let cells: Vec<Node> = CELLS.iter().map(cell).collect();
    div(
        vec![("class", "blocks-bento-three-column-tall")],
        vec![
            intro(),
            div(
                vec![("class", "blocks-bento-three-column-tall-grid")],
                cells,
            ),
        ],
    )
}
```

関連情報: [Badge](../themes/badge.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Card](../themes/card.md) / [Image](../themes/image.md)
