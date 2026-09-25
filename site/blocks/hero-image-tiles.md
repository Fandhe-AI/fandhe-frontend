# hero-image-tiles

`badge` / `heading` / `text` / `button` / `image` の 5 部品だけで組み立てた、画像タイルのコラージュ付きヒーローです。新しい UI 部品は追加していません。

左側に告知バッジ・見出し・リード文・CTA ボタン 2 個を縦に並べ、右側には縦長の画像タイル 5 枚を 3 列（1・2・2 枚）に分けて配置します。各列は上下にオフセットさせ、段違いに見えるコラージュにしています。タイルの角丸・影はテーマトークン（`--fandhe-radius-xl`・`--fandhe-shadow-lg`）で付けています。

lg（1024px）未満ではテキストの下にタイル列を横並びで配置し、はみ出す分は `overflow: hidden` で隠します。breakpoint は lg の 1 段のみで、`<form>` は使わず送信処理・データ取得を持たない静的な合成例です。画像はすべて同一のプレースホルダーで実データを持たず、装飾用途として `alt=""` にしています。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    self as styled_heading, HeadingLevel, HeadingProps, HeadingSize,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 列ごとのタイル枚数配分（左から 1・2・2 枚、合計 5 枚固定）。
const COLUMNS: [usize; 3] = [1, 2, 2];

/// テキスト列（告知バッジ → 見出し → リード文 → CTA 2 個）。
fn copy() -> Node {
    let eyebrow = badge::badge(&BadgeProps::default(), vec![], vec![text("Now in preview")]);
    let title = styled_heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl4,
            ..HeadingProps::default()
        },
        vec![],
        vec![text("チームの制作物を、届くところまで速く")],
    );
    let lead = styled_text::text(
        &TextProps {
            size: TextSize::Lg,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "撮影から公開までの制作フローを一つの場所にまとめ、チーム全員が同じ進捗を見ながら進められます。",
        )],
    );
    let actions = div(
        vec![("class", "blocks-hero-image-tiles-actions")],
        vec![
            button::button(&ButtonProps::default(), vec![], vec![text("Get started")]),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("See how it works")],
            ),
        ],
    );
    div(
        vec![("class", "blocks-hero-image-tiles-copy")],
        vec![eyebrow, title, lead, actions],
    )
}

/// タイル 1 枚分。角丸・影はラッパ `div` 側で付け、`image` 自体は既定
/// shape のまま使う（モジュール doc「タイルの角丸・影」節）。
fn tile() -> Node {
    div(
        vec![("class", "blocks-hero-image-tiles-tile")],
        vec![image::image(
            &ImageProps {
                aspect_ratio: AspectRatio::Portrait,
                ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
            },
            vec![("data-blocks-hero-image-tiles-image", "")],
        )],
    )
}

/// 列 1 本分（`count` 枚のタイルを縦に並べる）。
fn column(index: usize, count: usize) -> Node {
    let column_number = (index + 1).to_string();
    div(
        vec![
            ("class", "blocks-hero-image-tiles-column"),
            (
                "data-blocks-hero-image-tiles-column",
                column_number.as_str(),
            ),
        ],
        (0..count).map(|_| tile()).collect(),
    )
}

/// 右側のコラージュ（3 列、枚数配分は [`COLUMNS`]）。
fn collage() -> Node {
    div(
        vec![("class", "blocks-hero-image-tiles-collage")],
        COLUMNS
            .iter()
            .enumerate()
            .map(|(i, &count)| column(i, count))
            .collect(),
    )
}

/// `hero-image-tiles` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-hero-image-tiles-grid")],
        vec![copy(), collage()],
    )
}
```

## 原案差分メモ

参照（対応表 ID R1012）はローカル取り込み前の一時ディレクトリ
（`_/blocks-intake/`）を出典としていますが、本実装セッションからは
参照素材そのものを読めない状態でした。そのため以下はイシュー本文の
レイアウト仕様（左テキスト・右 3 列コラージュ・列ごとの上下オフセット・
lg 未満の横並び退行）のみから独自に設計したものであり、参照元の
文言・配色・装飾は持ち込んでいません。

- 文言・見出し・リード文はすべて独自に書き起こしたもので、実在
  ブランドの文言・配色は含みません。
- タイルの角丸・影はテーマトークン（`--fandhe-radius-xl`・
  `--fandhe-shadow-lg`）で付け、色そのものはテーマ既定に委ねています
  （参照元の配色は持ち込みません）。
- 列の幅配分・オフセット量（`--fandhe-space-*` トークン）は参照素材が
  読めないため、イシューの「上下にずらす」という記述から違和感のない
  値を選んで実装しました。素材が閲覧できる環境での見た目の微調整は
  スコープ外としています。
- breakpoint は lg（1024px）の 1 段のみとし、`sm`/`md` は新設して
  いません。
- 画像は `dummy_assets` のプレースホルダーで `alt=""`（装飾扱い）に
  しました。
- `id`・`href`・`<form>` は出力しません。
