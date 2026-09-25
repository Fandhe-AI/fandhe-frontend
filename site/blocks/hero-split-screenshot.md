# hero-split-screenshot

`badge` / `heading` / `text` / `button` / `image` / `code` / `link` の
7 部品を合成した、テキスト列（eyebrow badge・見出し・リード文・CTA 2 個）
とアプリ画面画像を列幅より大きく置いて右端へはみ出させる分割ヒーローです。

3 通りの見せ方（基準形・余白付き枠・タブ付きコード枠）を 1 つの Demo に
縦に並べています。lg（64rem）未満では画像・コード枠がテキストの下に来る
1 列表示になり、lg 以上でのみ 2 列のグリッド表示になります。画像は各
セクションの境界で切り取られる形ではみ出します。文言はすべて架空のもので、
データ取得・送信は行わない静的な表示例です。`<form>` は使用しません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps};
use fandhe_frontend_pre_styled_ui::code::{self, CodeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps, LinkVariant};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// `link::root` の href（固定の公開リポジトリ URL。`contact_split_info` 等
/// 他 block と同じ判断）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// テキスト列（eyebrow badge + 見出し + リード文 + CTA 2 個）。
fn copy_column(eyebrow: &str, title: &str, lead: &str) -> Node {
    div(
        vec![("class", "blocks-hero-split-screenshot-text")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-hero-split-screenshot-eyebrow", "")],
                vec![text(eyebrow)],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl4,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text(title)],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-hero-split-screenshot-lead", "")],
                vec![text(lead)],
            ),
            div(
                vec![("class", "blocks-hero-split-screenshot-actions")],
                vec![
                    button::button(&ButtonProps::default(), vec![], vec![text("今すぐ始める")]),
                    link::root(
                        REPO,
                        &LinkProps {
                            variant: LinkVariant::Underline,
                            palette: ColorPalette::Neutral,
                            ..LinkProps::default()
                        },
                        vec![],
                        vec![text("ドキュメントを見る")],
                    ),
                ],
            ),
        ],
    )
}

/// アプリ画面のプレースホルダー画像（列幅より大きい固定幅で、
/// [`LAYOUT_CSS`] 側がセクション境界での切り取りを行う）。装飾扱いのため
/// `alt=""`。
fn screenshot() -> Node {
    image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio: AspectRatio::Auto,
            shape: ImageShape::Rounded,
            ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
        },
        vec![("data-blocks-hero-split-screenshot-image", "")],
    )
}

/// A: 基準形（テキスト列 → 画像列。画像は右へはみ出す）。
fn section_a() -> Node {
    div(
        vec![("class", "blocks-hero-split-screenshot-section")],
        vec![
            copy_column(
                "プラットフォーム",
                "画面のまま特長を伝える",
                "実際の操作画面をそのまま見せながら、主要な特長を紹介します。",
            ),
            div(
                vec![("class", "blocks-hero-split-screenshot-media")],
                vec![screenshot()],
            ),
        ],
    )
}

/// B: 余白付き枠（画像を枠で包み、枠ごと右へはみ出す）。
fn section_b() -> Node {
    div(
        vec![("class", "blocks-hero-split-screenshot-section")],
        vec![
            copy_column(
                "見せ方の一例",
                "余白を持たせて画面を見せる",
                "画像の周囲に余白と枠を持たせることで、より落ち着いた印象を与えます。",
            ),
            div(
                vec![("class", "blocks-hero-split-screenshot-media")],
                vec![div(
                    vec![("class", "blocks-hero-split-screenshot-frame")],
                    vec![screenshot()],
                )],
            ),
        ],
    )
}

/// C: タブ付きコード枠（テキスト列 → 開発者向けのコード表示枠）。
fn section_c() -> Node {
    let snippet = "let app = App::new();\napp.mount(\"#root\");\napp.run();\n";
    div(
        vec![("class", "blocks-hero-split-screenshot-section")],
        vec![
            copy_column(
                "開発者向け",
                "コードで組み込む",
                "既存のアプリへ数行を追加するだけで導入できます。",
            ),
            div(
                vec![("class", "blocks-hero-split-screenshot-media")],
                vec![div(
                    vec![("class", "blocks-hero-split-screenshot-code-frame")],
                    vec![
                        div(
                            vec![("class", "blocks-hero-split-screenshot-tabs")],
                            vec![
                                span(
                                    vec![("data-blocks-hero-split-screenshot-tab-active", "")],
                                    vec![text("main.rs")],
                                ),
                                span(vec![], vec![text("Cargo.toml")]),
                            ],
                        ),
                        el(
                            "pre",
                            vec![("class", "blocks-hero-split-screenshot-pre")],
                            vec![code::code(
                                &CodeProps::default(),
                                vec![("data-blocks-hero-split-screenshot-code", "")],
                                vec![text(snippet)],
                            )],
                        ),
                    ],
                )],
            ),
        ],
    )
}

/// `hero-split-screenshot` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ状態を持たない設計）。ルート class は
/// `demo_class`（`blocks-hero-split-screenshot`）とは別名にする。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-hero-split-screenshot-layout")],
        vec![section_a(), section_b(), section_c()],
    )
}
```

## 原案差分メモ

- 3 通り（基準形・余白付き枠・タブ付きコード枠）を 1 つの Demo に縦に
  並べています。基準形は集約元の基本形、余白付き枠は画像に padding・
  border・角丸を持たせた見せ方、タブ付きコード枠は開発者向けの実装例
  相当です。
- タブ列は `tabs` 部品を使わず素の `span` 2 つによる静的表示です。
  `tabs` は使用部品一覧に含まれず、JS ハイドレーションを行わない docs
  サイトでは操作もできないため、`role="tab"` 等の操作可能を示す ARIA は
  付けていません。
- 参照元がよく使う負の margin によるはみ出しは採らず、各セクションの
  ルートへ `overflow: hidden` を付けて画像をセクションの境界で切り取る
  方式にしました（Demo 枠が横スクロールする副作用を避けるため）。
- 見出しは 1 段下げて `h3` にしています（ページ側が `## Demo` として
  `h2` を出すため）。
- 画像は `alt=""`（装飾扱い）で出力し、[`dummy_assets::SCREENSHOT_SRC`]
  を使います。
- CTA の 2 つ目は `link` で表現し、href は固定の公開リポジトリ URL のみ
  使います。
- コードスニペットは架空の Rust 風コードで、トークン・URL・メール
  アドレスに見える文字列は含みません。
- 文言・配色は独自のもの、または既存のテーマトークンにそのまま従います。
