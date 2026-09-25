# hero-marquee-strip

`badge`（eyebrow）/ `heading`（見出し）/ `text`（リード文・キャプション）/
`button`（CTA 2 個）/ `marquee`（ロゴ列）/ `image`（ロゴ画像）の合成例です
（既存部品のみ、新規部品なし）。CTA の下にロゴ列が横へ流れる帯を配置します。
ロゴ列の複製列は `aria-hidden`/`inert` で支援技術・キーボード操作の両方から
除外され、両端はフェードします。`prefers-reduced-motion: reduce` では
アニメーションが止まり折り返した静止列になり、`:hover`/`:focus-within` では
一時停止します（`marquee` 部品の既定契約）。`<form>` は持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::marquee::{self, MarqueeProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize};

const STRIP_ATTR: &str = "data-blocks-hero-marquee-strip-strip";
const LOGO_ATTR: &str = "data-blocks-hero-marquee-strip-logo";
const CAPTION_ATTR: &str = "data-blocks-hero-marquee-strip-caption";
const LOGO_COUNT: usize = 8;

pub fn demo() -> Node {
    let eyebrow = badge::badge(&BadgeProps::default(), vec![], vec![text("New")]);

    let title = heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl4,
            ..HeadingProps::default()
        },
        vec![],
        vec![text("Build on a platform teams already trust")],
    );

    let lead = styled_text::text(
        &TextProps {
            size: TextSize::Lg,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "Ship faster with the workflows your team already knows.",
        )],
    );

    let actions = div(
        vec![("class", "blocks-hero-marquee-strip-actions")],
        vec![
            button::button(&ButtonProps::default(), vec![], vec![text("Get started")]),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("Talk to sales")],
            ),
        ],
    );

    let caption = styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            ..TextProps::default()
        },
        vec![(CAPTION_ATTR, "")],
        vec![text("Trusted by teams of every size")],
    );

    let logos = (0..LOGO_COUNT)
        .map(|i| {
            let name = dummy_assets::COMPANY_NAMES[i % dummy_assets::COMPANY_NAMES.len()];
            marquee::item(
                vec![],
                vec![image::image(
                    &ImageProps {
                        fit: ImageFit::Contain,
                        ..ImageProps::new(dummy_assets::LOGO_SRC, name)
                    },
                    vec![(LOGO_ATTR, "")],
                )],
            )
        })
        .collect();
    let strip = marquee::marquee(
        &MarqueeProps {
            label: Some("Teams using the platform"),
            ..MarqueeProps::default()
        },
        vec![(STRIP_ATTR, "")],
        logos,
    );

    div(
        vec![("class", "blocks-hero-marquee-strip-inner")],
        vec![eyebrow, title, lead, actions, caption, strip],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0538 を基準形とし、R0539・R0541 との差分を含みます。
出典の固有名・ファイル名は記載しません）からの意図的な差分は次のとおりです。

- R0538（基準形）の構成をそのまま採用しました。eyebrow badge → 見出し →
  リード文 → CTA 2 個 → キャプション → ロゴ帯の順です。
- R0539（CTA 文言違い）は文言差のみで構造が同一のため、Demo を分けず
  独自の CTA 文言（`Get started` / `Talk to sales`）を採用しました。
- R0541（画像 8 枚）は `marquee::item` 内の子要素をロゴから画像へ差し替える
  だけの同一構造のため、Demo は併記せずロゴ帯 1 種類（8 枚）のみを示します。
- ロゴはすべて `dummy_assets::LOGO_SRC`（ビルド時生成のプレースホルダー
  SVG）を使い回し、`alt` にのみ架空社名（`dummy_assets::COMPANY_NAMES`、
  循環参照）を割り当てています。
- 見出しを 1 段下げて `h3` にしました（ページ側が `## Demo` として `h2`
  を出すため）。
- 配色・間隔はテーマトークン（`--fandhe-space-*`）準拠です。

関連情報: [marquee](../themes/marquee.md) / [image](../themes/image.md)
