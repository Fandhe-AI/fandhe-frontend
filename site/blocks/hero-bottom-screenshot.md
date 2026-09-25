# hero-bottom-screenshot

`fandhe-frontend-pre-styled-ui` の `badge` / `heading` / `text` / `button` /
`image` の 5 部品のみを合成した、上段に見出し・リード文・CTA、下段に横長
スクリーンショットを全幅で置く 1 列ヒーローの合成例です。Blocks セクション
は新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わ
せた実例集であることに注意してください（基準は対応表 ID R1007、集約元は
対応表 ID R0126/R0127/R0534/R0540/R0545/R0552/R0554/R1008 の 8 件です。出典
の固有名・ファイル名は記載しません）。

見せ方の差分を 6 セクションに縦並記しています。基準形（枠付きスクリーン
ショット）、上辺のみ角丸で枠なしの形、左寄せで `lg`（1024px）以上のとき
見出しと説明+CTA を左右 2 列に分ける形、動画そのものではなく静的な
「Video placeholder」プレースホルダー、大判+正方形の画像 2 枚グリッド、
淡色帯で囲んだ画像の下にロゴ 5 個 + 社名を並べたロゴ列、の 6 通りです。
`<form>` 要素は出力せず、`<video>` 要素も使いません（実際には再生できない
要素を実物のタグで偽装しないため）。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

const SECTION_VARIANT_ATTR: &str = "data-blocks-hero-bottom-screenshot-section-variant";
const IMAGE_VARIANT_ATTR: &str = "data-blocks-hero-bottom-screenshot-image-variant";

/// 見出し 1 件分（eyebrow badge + 見出し + リード文）。`align` が `"start"`
/// のとき左寄せへ切り替える（[`LAYOUT_CSS`] 側のセレクタ参照）。
fn section_header(eyebrow: Option<&str>, title: &str, lead: &str, align: &'static str) -> Node {
    let mut children: Vec<Node> = Vec::new();
    if let Some(eyebrow) = eyebrow {
        children.push(badge::badge(
            &BadgeProps::default(),
            vec![("data-blocks-hero-bottom-screenshot-eyebrow", "")],
            vec![text(eyebrow)],
        ));
    }
    children.push(heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl3,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text(title)],
    ));
    children.push(styled_text::text(
        &TextProps {
            size: TextSize::Lg,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(lead)],
    ));
    div(
        vec![
            ("class", "blocks-hero-bottom-screenshot-header"),
            ("data-blocks-hero-bottom-screenshot-align", align),
        ],
        children,
    )
}

/// CTA ボタン 2 個（Solid + Outline）の行。
fn cta_row() -> Node {
    div(
        vec![("class", "blocks-hero-bottom-screenshot-actions")],
        vec![
            button::button(&ButtonProps::default(), vec![], vec![text("Get started")]),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("View docs")],
            ),
        ],
    )
}

/// 16:9 のスクリーンショット画像（`variant` は [`LAYOUT_CSS`] 側の枠・
/// 角丸の切り替えに使う）。装飾扱いのため `alt=""`。
fn screenshot(variant: &'static str) -> Node {
    div(
        vec![
            ("class", "blocks-hero-bottom-screenshot-media"),
            (IMAGE_VARIANT_ATTR, variant),
        ],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Video,
                shape: ImageShape::Square,
                ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
            },
            vec![("data-blocks-hero-bottom-screenshot-image", "")],
        )],
    )
}

/// 動画プレースホルダー（`<video>` は使わず、破線枠 + 1 行テキストのみの
/// 静的表示。モジュール doc「`<video>` を出力しない理由」参照）。
fn video_placeholder() -> Node {
    div(
        vec![("class", "blocks-hero-bottom-screenshot-video-placeholder")],
        vec![styled_text::text(
            &TextProps {
                size: TextSize::Sm,
                variant: TextVariant::Muted,
                ..TextProps::default()
            },
            vec![],
            vec![text("Video placeholder")],
        )],
    )
}

/// 2 枚グリッド（大判の横長画像 + 正方形画像）。
fn pair_grid() -> Node {
    div(
        vec![("class", "blocks-hero-bottom-screenshot-pair-grid")],
        vec![
            div(
                vec![("class", "blocks-hero-bottom-screenshot-media")],
                vec![image::image(
                    &ImageProps {
                        fit: ImageFit::Cover,
                        aspect_ratio: AspectRatio::Landscape,
                        shape: ImageShape::Square,
                        ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
                    },
                    vec![("data-blocks-hero-bottom-screenshot-image", "")],
                )],
            ),
            div(
                vec![("class", "blocks-hero-bottom-screenshot-media")],
                vec![image::image(
                    &ImageProps {
                        fit: ImageFit::Cover,
                        aspect_ratio: AspectRatio::Square,
                        shape: ImageShape::Square,
                        ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                    },
                    vec![("data-blocks-hero-bottom-screenshot-image", "")],
                )],
            ),
        ],
    )
}

/// 淡色帯で囲んだ画像 + その下のロゴ列（ロゴ 5 個 + 社名。
/// [`dummy_assets::COMPANY_NAMES`] の先頭 5 件を使う）。
fn band_with_logos() -> Node {
    let logos: Vec<Node> = dummy_assets::COMPANY_NAMES[..5]
        .iter()
        .map(|name| {
            div(
                vec![("class", "blocks-hero-bottom-screenshot-logo-item")],
                vec![
                    image::image(
                        &ImageProps {
                            fit: ImageFit::Contain,
                            aspect_ratio: AspectRatio::Square,
                            shape: ImageShape::Square,
                            ..ImageProps::new(dummy_assets::LOGO_SRC, "")
                        },
                        vec![("data-blocks-hero-bottom-screenshot-logo", "")],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(*name)],
                    ),
                ],
            )
        })
        .collect();

    div(
        vec![("class", "blocks-hero-bottom-screenshot-band")],
        vec![
            screenshot("band"),
            div(
                vec![("class", "blocks-hero-bottom-screenshot-logo-row")],
                logos,
            ),
        ],
    )
}

/// bordered（基準形）: eyebrow + 見出し + リード文 + CTA → 枠付き画像。
fn bordered_section() -> Node {
    div(
        vec![("class", "blocks-hero-bottom-screenshot-section"), (SECTION_VARIANT_ATTR, "bordered")],
        vec![
            section_header(
                Some("New release"),
                "See it in action before you start",
                "A single screenshot below shows the real product screen, framed just like it looks in the app.",
                "center",
            ),
            cta_row(),
            screenshot("bordered"),
        ],
    )
}

/// top-rounded: 見出し + リード文（badge なし）→ 枠なし・上辺のみ角丸の画像。
fn top_rounded_section() -> Node {
    div(
        vec![("class", "blocks-hero-bottom-screenshot-section"), (SECTION_VARIANT_ATTR, "top-rounded")],
        vec![
            section_header(
                None,
                "A cleaner way to preview your work",
                "The screenshot sits flush against the section below it, rounded only where it meets the header.",
                "center",
            ),
            screenshot("top-rounded"),
        ],
    )
}

/// left-split: 左寄せ。`lg` 以上で見出しと説明+CTA を 2 列に分ける
/// → 枠付き画像。
fn left_split_section() -> Node {
    let title = heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl3,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text("Built for teams that ship every day")],
    );
    let lead = styled_text::text(
        &TextProps {
            size: TextSize::Lg,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "Give every teammate the same clear view of what changed, without digging through logs.",
        )],
    );
    let right = div(
        vec![("class", "blocks-hero-bottom-screenshot-split-right")],
        vec![lead, cta_row()],
    );
    div(
        vec![
            ("class", "blocks-hero-bottom-screenshot-section"),
            (SECTION_VARIANT_ATTR, "left-split"),
        ],
        vec![
            div(
                vec![
                    ("class", "blocks-hero-bottom-screenshot-split-row"),
                    ("data-blocks-hero-bottom-screenshot-align", "start"),
                ],
                vec![title, right],
            ),
            screenshot("bordered"),
        ],
    )
}

/// video: eyebrow なし・中央寄せ見出し → 動画プレースホルダー。
fn video_section() -> Node {
    div(
        vec![
            ("class", "blocks-hero-bottom-screenshot-section"),
            (SECTION_VARIANT_ATTR, "video"),
        ],
        vec![
            section_header(
                None,
                "Watch how it feels to use",
                "A short walkthrough of the product, right where you'd expect it.",
                "center",
            ),
            video_placeholder(),
        ],
    )
}

/// pair-grid: 中央寄せ見出し → 大判 + 正方形の画像 2 枚グリッド。
fn pair_grid_section() -> Node {
    div(
        vec![
            ("class", "blocks-hero-bottom-screenshot-section"),
            (SECTION_VARIANT_ATTR, "pair-grid"),
        ],
        vec![
            section_header(
                None,
                "Two views, one workflow",
                "See the full dashboard alongside a closer look at a single card.",
                "center",
            ),
            pair_grid(),
        ],
    )
}

/// band-logos: 中央寄せ見出し → 淡色帯で囲んだ画像 + ロゴ列。
fn band_logos_section() -> Node {
    div(
        vec![
            ("class", "blocks-hero-bottom-screenshot-section"),
            (SECTION_VARIANT_ATTR, "band-logos"),
        ],
        vec![
            section_header(
                None,
                "Trusted by teams around the world",
                "Join teams already using the product every day.",
                "center",
            ),
            band_with_logos(),
        ],
    )
}

/// `hero-bottom-screenshot` の Demo 本体（6 セクション縦並記）。呼び出し
/// ごとに同一の `Node` を返す純関数。ルート class は `demo_class`
/// （`blocks-hero-bottom-screenshot`）とは別名にする。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-hero-bottom-screenshot-layout")],
        vec![
            bordered_section(),
            top_rounded_section(),
            left_split_section(),
            video_section(),
            pair_grid_section(),
            band_logos_section(),
        ],
    )
}
```

## 原案差分メモ

参照（基準は対応表 ID R1007、集約元は対応表 ID R0126/R0127/R0534/R0540/
R0545/R0552/R0554/R1008。出典の固有名・ファイル名は記載しません）から取り
込んだのは構造（領域の配置と部品構成）のみであり、次の点を独自に設計・
変更しています。

- 「淡色帯」（R0545）と「ロゴ列」（R0534）は別々の block に分けず、1 つの
  セクションへ統合しました（両方とも「下段の周辺装飾」であるため）。
- 動画（R0554）は `<video>` 要素で偽装せず、破線枠 + 「Video placeholder」
  という 1 行テキストのみの静的なプレースホルダーにしました。
- 上辺のみ角丸（R0126/R1008 集約）は枠付き基準形とは別のセクションとして
  独立させ、`border-radius` の上辺のみを丸める CSS の差分が見た目で読み
  取れるようにしました。
- 文言はすべて独自に書いた架空のものです（実企業名・実クレデンシャル・
  PII は含みません）。ロゴ列の社名は Blocks 共通のダミー素材ヘルパの値を
  そのまま使っています。
- 画像はすべて Blocks 共通のダミー素材ヘルパが生成するプレースホルダー
  SVG（`data:` URI ではなくビルド時生成の相対パスアセット）を使い、装飾
  扱いの `alt=""` で出力しています。
- 画面幅は `lg`（1024px）以上で左右 2 列（left-split）、`md`（768px）
  以上で 2 枚グリッドを `2fr 1fr` に切り替えています。

関連情報: [Badge](../themes/badge.md) / [Heading](../themes/heading.md) /
[Text](../themes/text.md) / [Button](../themes/button.md) /
[Image](../themes/image.md)
