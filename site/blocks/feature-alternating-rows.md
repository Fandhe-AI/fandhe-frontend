# feature-alternating-rows

`badge` / `heading` / `text` / `image` / `separator` の 5 部品を合成した、
中央寄せのセクション見出しの下に「テキスト列 + 横長画像列」の行を積み、
偶数行で左右を入れ替える feature セクションです。

各行の直前には罫線を引き、行ごとにテキストと横長・角丸の画像を組み合わ
せます。DOM 順は常にテキスト → 画像で、幅 lg（64rem）未満ではどの行も
テキストの下に画像が来ます。lg 以上では 12 列 grid の配置だけで左右を
入れ替え、DOM 順自体は変えません。文言はすべて架空のもので、データ取得・
送信は行わない静的な表示例です。`<form>` は使用しません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 行データ 1 件分（見出し + 説明 + 画像。どの画像を使うかは行ごとに
/// 変える、`content-columns-screenshot` の複数インスタンス方針と同型）。
struct Row {
    title: &'static str,
    body: &'static str,
    src: &'static str,
}

/// 3 行分の架空データ（左右交互は [`demo`] 側で添字の偶奇から導出する）。
const ROWS: [Row; 3] = [
    Row {
        title: "決定的なビルド出力",
        body: "同じ入力からは常に同じ静的ファイルを生成し、差分レビューを容易にします。",
        src: dummy_assets::PRODUCT_SRC,
    },
    Row {
        title: "型で表現する構造",
        body: "スロットと props は Rust の型で表現され、不整合はコンパイル時に検出されます。",
        src: dummy_assets::SCREENSHOT_SRC,
    },
    Row {
        title: "静的な表示のみ",
        body: "JS ハイドレーションを行わない、決定的な静的表示専用の合成例です。",
        src: dummy_assets::BACKGROUND_SRC,
    },
];

/// 中央寄せのヘッダー（eyebrow badge + 見出し + リード文）。
fn header() -> Node {
    div(
        vec![("class", "blocks-feature-alternating-rows-header")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-feature-alternating-rows-eyebrow", "")],
                vec![text("導入ガイド")],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("特長を交互のレイアウトで紹介する")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-alternating-rows-lead", "")],
                vec![text(
                    "各行はテキストと横長画像の組で構成し、行ごとに左右を入れ替えます。",
                )],
            ),
        ],
    )
}

/// テキスト列（行見出し + 説明）。
fn row_text(row: &Row) -> Node {
    div(
        vec![("class", "blocks-feature-alternating-rows-text")],
        vec![
            heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(row.title)],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-alternating-rows-desc", "")],
                vec![text(row.body)],
            ),
        ],
    )
}

/// 画像列（横長・角丸、装飾扱いの `alt=""`）。
fn row_media(row: &Row) -> Node {
    div(
        vec![("class", "blocks-feature-alternating-rows-media")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Video,
                shape: ImageShape::Rounded,
                ..ImageProps::new(row.src, "")
            },
            vec![("data-blocks-feature-alternating-rows-image", "")],
        )],
    )
}

/// 行 1 件（上に罫線 + テキスト → 画像。`reverse` は 2 行目・4 行目…
/// （添字が奇数、1 始まりで偶数行目）にのみ立てる）。
fn feature_row(index: usize, row: &Row) -> Vec<Node> {
    let rule = separator(
        &SeparatorProps::default(),
        vec![("data-blocks-feature-alternating-rows-rule", "")],
    );

    let mut row_attrs = vec![("class", "blocks-feature-alternating-rows-row")];
    if index % 2 == 1 {
        row_attrs.push(("data-blocks-feature-alternating-rows-reverse", ""));
    }

    let row_node = div(row_attrs, vec![row_text(row), row_media(row)]);

    vec![rule, row_node]
}

/// `feature-alternating-rows` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（他 block と同じ状態を持たない設計）。
pub fn demo() -> Node {
    let mut rows_children: Vec<Node> = Vec::new();
    for (index, row) in ROWS.iter().enumerate() {
        rows_children.extend(feature_row(index, row));
    }

    div(
        vec![("class", "blocks-feature-alternating-rows-layout")],
        vec![
            header(),
            div(
                vec![("class", "blocks-feature-alternating-rows-rows")],
                rows_children,
            ),
        ],
    )
}
```

## 原案差分メモ

参照（対応表 ID R1159・R1156。出典の固有名・ファイル名は記載しません）
からの意図的な差分は次のとおりです。

- R1156 の行は左右固定ですが、本 block では基準形 R1159 に合わせて左右
  交互に統一しました。
- R1156 の左寄せ eyebrow 見出しは、中央寄せ見出し + `badge` の eyebrow へ
  まとめました。
- R1156 の「行群の上に 1 本だけ引く罫線」は、各行の直前に置く `separator`
  へ一般化しました（行の間を罫線で区切る形）。
- 参照元の `column-reverse`（狭い幅で画像が上）は採らず、DOM 順を
  テキスト → 画像にして lg 未満では画像が下に来るようにしました。
- xl での 4/8 列分割は持ち込まず、5/7 列に固定しました。
- 見出しを 1 段下げて `h3`、行見出しは `h4` にしました（ページ側が
  `## Demo` として `h2` を出すため）。
- 画像は `dummy_assets` のプレースホルダー + `alt=""`（装飾扱い）、
  `AspectRatio::Video`（16:9）+ `Rounded` で横長・角丸を揃えました。
- 文言・配色・余白は独自のもの、または既存のテーマトークンにそのまま
  従います。
