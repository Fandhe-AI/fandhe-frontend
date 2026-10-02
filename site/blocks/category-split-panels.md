# category-split-panels

表示幅を左右 2 等分したパネルを 2 枚並べる、カテゴリ 2 分割パネルです。
`heading` / `text` / `image` / `link` の 4 部品を合成します。Blocks は
既存部品の合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R0826 の 1 形です。各パネルは背景画像の上に淡い半透明の
面を重ね、見出し・説明・買い物導線のリンクを置きます。広い幅（40rem
以上）では 2 枚のパネルを左右 2 列、それ未満では上下 2 段に積みます。
文言はすべて架空のカテゴリデータであり、実在の企業・ブランド・PII は
含みません。画像はビルド時生成の同梱プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。買い物導線
リンクは `link::root` の通常のナビゲーションリンクで、実在する GitHub
リポジトリへ遷移します（2 つのリンクは可視テキスト・遷移先ともに異なり、
曖昧な重複リンクにしていません）。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps};

/// 買い物導線リンクの固定外部 URL（モジュール doc「link の `href` を
/// 固定の外部絶対 URL にする・可視テキストとの整合」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";
const RELEASES: &str = "https://github.com/Fandhe-AI/fandhe-frontend/releases";

/// 1 パネル分のデータ（見出し・説明・背景画像・買い物導線リンク）。
struct Panel {
    title: &'static str,
    body: &'static str,
    image_src: &'static str,
    href: &'static str,
    link_label: &'static str,
}

const PANELS: [Panel; 2] = [
    Panel {
        title: "アウトドア用品",
        body: "軽量テントから調理器具まで、週末の遠出に必要な一式を集めました。",
        image_src: dummy_assets::BACKGROUND_SRC,
        href: REPO,
        link_label: "GitHub で見る",
    },
    Panel {
        title: "キッチン家電",
        body: "毎日の調理をすこし楽にする定番アイテムを集めました。",
        image_src: dummy_assets::PRODUCT_SRC,
        href: RELEASES,
        link_label: "リリースを見る",
    },
];

/// 1 パネル分の構成（背景画像 + 淡い面に重ねた見出し・説明・リンク）。
/// 画像と面を同じ `grid-area: 1 / 1` で重ねる（モジュール doc「重ね合わせ
/// は `grid-area` の共有で行う」節）。
fn panel(p: &Panel) -> Node {
    div(
        vec![("class", "blocks-category-split-panels-panel")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    ..ImageProps::new(p.image_src, "")
                },
                vec![("data-blocks-category-split-panels-image", "")],
            ),
            div(
                vec![("class", "blocks-category-split-panels-surface")],
                vec![
                    heading(
                        HeadingLevel::H3,
                        &HeadingProps {
                            size: HeadingSize::Xl,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text(p.title)],
                    ),
                    styled_text::text(&TextProps::default(), vec![], vec![text(p.body)]),
                    link::root(
                        p.href,
                        &LinkProps {
                            external: true,
                            ..LinkProps::default()
                        },
                        vec![],
                        vec![text(p.link_label)],
                    ),
                ],
            ),
        ],
    )
}

/// `category-split-panels` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-category-split-panels-layout")],
        vec![div(
            vec![("class", "blocks-category-split-panels-split")],
            PANELS.iter().map(panel).collect(),
        )],
    )
}
```

## 原案差分メモ

- 見出しはすべて `HeadingLevel::H3` にしました（ページ側が `## Demo` として
  `h2` を出すため）。
- 参照元 R0826 の配色をそのまま持ち込まず、`--fandhe-color-bg` ベースの
  淡い半透明面（`color-mix(in srgb, var(--fandhe-color-bg) 88%,
  transparent)`）+ 既定の `--fandhe-color-fg` のトークンベース配色に
  置き換えました。`category-featured-banner` の形 A（暗い半透明パネル +
  明るい文字の反転ペア）とは逆の、非反転のペアです。
- 背景画像は共通ダミー素材（`dummy_assets::BACKGROUND_SRC`/
  `PRODUCT_SRC`）に置き換えました。
- 買い物導線リンクは、遷移先と矛盾しない文言の GitHub 固定リンク（repository
  ルートへの「GitHub で見る」・`/releases` への「リリースを見る」）に
  置き換えました。死リンク `href="#"` は使っていません。
- レイアウト切り替えは `@media`（ビューポート幅判定）ではなく
  `@container`（コンテナクエリ）を使っています。Demo のルートへ
  `container-type: inline-size` を宣言し、表示領域自身の実測幅を基準に
  `40rem` 以上で 2 列、それ未満では縦積みへ切り替えます。`40rem` は docs
  サイト上の Demo 枠（実測上限約 `43rem`）でも閾値へ到達する値です。CSS
  container query はコンテナ宣言要素自身ではなく子孫のみを判定対象に
  できるため、列数を変える要素（`.blocks-category-split-panels-split`）は
  `container-type`/`container-name` を持つ Demo ルートとは別の内側ラッパーに
  分離しています。
- 文言はすべて独自に書いた架空のカテゴリデータです。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Image](../themes/image.md) / [Link](../themes/link.md)
