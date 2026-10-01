# promo-image-tiles

`heading` / `text` / `link` / `image` の 4 部品だけで組み立てた、画像タイル付きのプロモーションです。新しい UI 部品は追加していません。

片側に見出し・リード文・CTA（外部リンク）、もう片側には画像タイルを 3 列で上下にずらして並べたコラージュを配置します。`base` 形（見出し・リード文・CTA リンク・タイル 7 枚）と `dark`（暗色の帯の上に見出し・リード文・外部リンク・タイル 6 枚を置く形）の 2 つを縦に並記し、差分が読み取れるようにしています。

タイルのコラージュには `aria-hidden="true"` を付け、装飾画像（`alt=""`）をまとめて支援技術から隠します。sm（640px）未満ではコラージュ自体を非表示にし、sm 以上で表示、lg（1024px）以上でテキストと横並びに切り替えます。`<form>` は使わず送信処理・データ取得を持たない静的な合成例です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{
    self as styled_heading, HeadingLevel, HeadingProps, HeadingSize,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 固定の外部リンク先（モジュール doc「リンク先の方針」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// `base`（R1201 相当）の列ごとのタイル枚数配分（合計 7 枚）。
const BASE_COLUMNS: [usize; 3] = [2, 3, 2];

/// `dark`（R1197 相当）の列ごとのタイル枚数配分（合計 6 枚）。
const DARK_COLUMNS: [usize; 3] = [2, 2, 2];

/// `base` 形のコピー列（見出し → リード文 → CTA リンク 1 個）。
fn copy_base() -> Node {
    div(
        vec![("class", "blocks-promo-image-tiles-copy")],
        vec![
            styled_heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-promo-image-tiles-title", "")],
                vec![text("季節の新作、まとめてチェック")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-promo-image-tiles-lead", "")],
                vec![text(
                    "入荷したばかりのアイテムを一覧できる特集ページをご用意しました。",
                )],
            ),
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![("data-blocks-promo-image-tiles-cta", "")],
                vec![text("特集を見る")],
            ),
        ],
    )
}

/// `dark` 形のコピー列（見出し → リード文 → 外部リンク）。
fn copy_dark() -> Node {
    div(
        vec![("class", "blocks-promo-image-tiles-copy")],
        vec![
            styled_heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-promo-image-tiles-title", "")],
                vec![text("限定アイテム、数量限り")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Plain,
                    ..TextProps::default()
                },
                vec![("data-blocks-promo-image-tiles-lead", "")],
                vec![text(
                    "今季限定の生産本数で仕立てたアイテムを取り扱っています。",
                )],
            ),
            link::root(
                REPO,
                &LinkProps {
                    external: true,
                    ..LinkProps::default()
                },
                vec![("data-blocks-promo-image-tiles-link", "")],
                vec![text("もっと見る →")],
            ),
        ],
    )
}

/// タイル 1 枚分（角丸・影はラッパ `div` 側で付ける、`hero_image_tiles`
/// と同型）。
fn tile() -> Node {
    div(
        vec![("class", "blocks-promo-image-tiles-tile")],
        vec![image::image(
            &ImageProps {
                aspect_ratio: AspectRatio::Portrait,
                ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
            },
            vec![("data-blocks-promo-image-tiles-image", "")],
        )],
    )
}

/// 列 1 本分（`count` 枚のタイルを縦に並べる）。
fn column(index: usize, count: usize) -> Node {
    let column_number = (index + 1).to_string();
    div(
        vec![
            ("class", "blocks-promo-image-tiles-column"),
            (
                "data-blocks-promo-image-tiles-column",
                column_number.as_str(),
            ),
        ],
        (0..count).map(|_| tile()).collect(),
    )
}

/// タイルのコラージュ（3 列、枚数配分は `columns`）。`aria-hidden` で
/// 支援技術から隠す（モジュール doc「タイルは `aria-hidden` + `alt=""`」
/// 節）。
fn collage(columns: &[usize]) -> Node {
    div(
        vec![
            ("class", "blocks-promo-image-tiles-collage"),
            ("aria-hidden", "true"),
        ],
        columns
            .iter()
            .enumerate()
            .map(|(i, &count)| column(i, count))
            .collect(),
    )
}

/// `base` 形（R1201 相当）1 件分。
fn instance_base() -> Node {
    div(
        vec![
            ("class", "blocks-promo-image-tiles-instance"),
            ("data-blocks-promo-image-tiles-tone", "base"),
        ],
        vec![copy_base(), collage(&BASE_COLUMNS)],
    )
}

/// `dark` 形（R1197 相当）1 件分。暗色帯は `data-blocks-promo-image-
/// tiles-tone="dark"` で判定する（モジュール doc「暗色帯」節）。
fn instance_dark() -> Node {
    div(
        vec![
            ("class", "blocks-promo-image-tiles-instance"),
            ("data-blocks-promo-image-tiles-tone", "dark"),
        ],
        vec![copy_dark(), collage(&DARK_COLUMNS)],
    )
}

/// `promo-image-tiles` の Demo 本体。`base`/`dark` の 2 形を縦に積んで
/// 並記する（モジュール doc「2 つの形を縦に並べて差分を示す」節）。
/// 呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-promo-image-tiles-stack")],
        vec![instance_base(), instance_dark()],
    )
}
```

## 原案差分メモ

参照（対応表 ID R1197/R1201）はローカル取り込み前の一時ディレクトリ
（`_/blocks-intake/`）を出典としていますが、本実装セッションからは
参照素材そのものを読めない状態でした。そのため以下はイシュー本文の
レイアウト仕様のみから独自に設計したものであり、参照元の文言・配色・
装飾は持ち込んでいません。

- 主参照は R1201（基準形: 見出し・リード文・CTA リンク・タイル 7 枚）、
  集約元は R1197（暗色帯の上に見出し・リード文・外部リンク・タイル
  6 枚を置く形）です。両者の差分を 1 つの Demo に並記して示しています。
  当初 `base` 形の CTA は操作不能な `button::button` でしたが、
  レビュー指摘（PR #3528）を受けて `dark` 形と同じ `link::root`
  （固定の外部リンク）へ統一しています。
- 文言はすべて独自に書き起こしたもので、実在ブランドの文言・配色は
  含みません。
- 暗色帯の配色はテーマトークン（`--fandhe-color-fg`/`--fandhe-color-bg`）
  の反転のみで作り、色リテラルは使っていません。
- sm（640px）未満ではタイルのコラージュを `display: none` で非表示に
  し、sm 以上で表示、lg（1024px）以上でテキストと横並びへ切り替えます。
  列配分・オフセット量（`--fandhe-space-*` トークン）は参照素材が読めない
  ため、イシューの「3 列で上下にずらす」という記述から違和感のない値を
  選んで実装しました。素材が閲覧できる環境での見た目の微調整はスコープ
  外としています。
- タイルは `aria-hidden="true"` + `alt=""`（装飾扱い）にし、画像は
  `dummy_assets` のプレースホルダーを使っています。
- `id`・`href="#"`・`<form>` は出力しません。
