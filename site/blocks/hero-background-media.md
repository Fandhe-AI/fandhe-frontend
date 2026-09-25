# hero-background-media

背景画像全面のヒーロー。`badge` / `heading` / `text` / `button` / `image`
の合成例（既存部品のみ、新規部品なし）。中央寄せ（基準形）と下寄せ 2 列の
2 形を並記します。背景は装飾扱いの `aria-hidden`、暗幕はテーマの色
トークンを `color-mix()` で半透明化したものです。アニメーションは持たず
`<form>` も持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

const IMAGE_ATTR: &str = "data-blocks-hero-background-media-image";
const EYEBROW_ATTR: &str = "data-blocks-hero-background-media-eyebrow";
const TITLE_ATTR: &str = "data-blocks-hero-background-media-title";
const LEAD_ATTR: &str = "data-blocks-hero-background-media-lead";
const CTA_PRIMARY_ATTR: &str = "data-blocks-hero-background-media-cta-primary";
const CTA_SECONDARY_ATTR: &str = "data-blocks-hero-background-media-cta-secondary";
const VARIANT_ATTR: &str = "data-blocks-hero-background-media-variant";

/// 各形の直前に置く短い形ラベル（`cta_split_image::variant_label` と同型）。
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

/// 背景画像 + 暗幕の 2 層（`aria-hidden` で装飾扱い、両形で共通）。
fn backdrop() -> Node {
    div(
        vec![
            ("class", "blocks-hero-background-media-backdrop"),
            ("aria-hidden", "true"),
        ],
        vec![
            image::image(
                &ImageProps::new(dummy_assets::BACKGROUND_SRC, ""),
                vec![(IMAGE_ATTR, "")],
            ),
            div(
                vec![("class", "blocks-hero-background-media-scrim")],
                vec![],
            ),
        ],
    )
}

/// 形 A（R0551 基準形）: 中央寄せの eyebrow badge + 見出し + リード文 + CTA。
fn variant_centered() -> Node {
    let content = div(
        vec![("class", "blocks-hero-background-media-content")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![(EYEBROW_ATTR, "")],
                vec![text("Now in preview")],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl4,
                    ..HeadingProps::default()
                },
                vec![(TITLE_ATTR, "")],
                vec![text("Build the page, keep the platform")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    ..TextProps::default()
                },
                vec![(LEAD_ATTR, "")],
                vec![text(
                    "Compose SSR, CSR and static output from one plain-HTML core.",
                )],
            ),
            div(
                vec![("class", "blocks-hero-background-media-actions")],
                vec![
                    button::button(
                        &ButtonProps::default(),
                        vec![(CTA_PRIMARY_ATTR, "")],
                        vec![text("Get started")],
                    ),
                    button::button(
                        &ButtonProps {
                            variant: ButtonVariant::Outline,
                            ..ButtonProps::default()
                        },
                        vec![(CTA_SECONDARY_ATTR, "")],
                        vec![text("See the demo")],
                    ),
                ],
            ),
        ],
    );
    div(
        vec![
            ("class", "blocks-hero-background-media-root"),
            (VARIANT_ATTR, "centered"),
        ],
        vec![backdrop(), content],
    )
}

/// 形 B（R0547 + R0133/R0546 集約）: 下寄せ、lg 以上で見出し/本文+CTA の
/// 2 列。動画背景は本 block と同じ静止画プレースホルダで代替する。
fn variant_bottom_split() -> Node {
    let content = div(
        vec![("class", "blocks-hero-background-media-content")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![(TITLE_ATTR, "")],
                vec![text("A platform that fades into the background")],
            ),
            div(
                vec![("class", "blocks-hero-background-media-split-side")],
                vec![
                    styled_text::text(
                        &TextProps::default(),
                        vec![(LEAD_ATTR, "")],
                        vec![text(
                            "Ship an interactive experience without shipping a framework.",
                        )],
                    ),
                    div(
                        vec![("class", "blocks-hero-background-media-actions")],
                        vec![
                            button::button(
                                &ButtonProps::default(),
                                vec![(CTA_PRIMARY_ATTR, "")],
                                vec![text("Get started")],
                            ),
                            button::button(
                                &ButtonProps {
                                    variant: ButtonVariant::Outline,
                                    ..ButtonProps::default()
                                },
                                vec![(CTA_SECONDARY_ATTR, "")],
                                vec![text("See the demo")],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    );
    div(
        vec![
            ("class", "blocks-hero-background-media-root"),
            (VARIANT_ATTR, "bottom-split"),
        ],
        vec![backdrop(), content],
    )
}

pub fn demo() -> Node {
    div(
        vec![("class", "blocks-hero-background-media-layout")],
        vec![
            variant_label("中央寄せ（R0551 基準形）"),
            variant_centered(),
            variant_label(
                "下寄せ + lg 以上で 2 列（R0547 + R0133/R0546 集約、動画背景は静止画で代替）",
            ),
            variant_bottom_split(),
        ],
    )
}
```

## 原案差分メモ

- 主参照は中央寄せの背景メディア hero（基準形）。下寄せ + 見出しと本文を
  lg 以上で 2 列にする形・動画背景 + 下寄せ見出しの形は、下寄せ 2 列の
  1 形へ集約した。
- 動画背景は無 JS の docs サイトでは再生できないため、両形とも静止画
  プレースホルダ（背景タイル SVG）で代替する。
- 全画面高ではなく `min-height` を使い、Demo 枠の中に収まる高さへ変更した。
- 暗幕は色リテラルではなく `--fandhe-color-fg` トークンを `color-mix()`
  で半透明化して作る。ライトテーマでは暗い幕の上に明るい文字が乗るが、
  ダークテーマでは前景/背景の意味が反転するため、明るい幕の上に暗い
  文字が乗る形へ反転する（意図した挙動）。
- 文言はすべて架空のものに差し替えた。
