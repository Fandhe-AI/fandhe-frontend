# category-featured-banner

1 カテゴリだけを大きく扱う横長バナーです。`heading` / `text` / `image` /
`link` の 4 部品を合成します。Blocks は既存部品の合成例であり、新しい UI
部品は追加しません。

主参照は対応表 ID R0823（全面画像 + 半透明パネル）で、R0608（画像 +
テキストの左右分割）を集約しています。広い幅では形 A は画像の上に
半透明パネルを重ね、形 B は画像とコピー列を左右 2 分割にします。狭い幅
では両形とも画像の下へコピー列が回ります。文言はすべて架空のカテゴリ
データであり、実在の企業・ブランド・PII は含みません。画像はビルド時
生成の同梱プレースホルダー SVG です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。CTA・誘導
リンクはいずれもボタン風に装飾した `link::root` で、実在する GitHub
リポジトリへ遷移します（可視テキストは遷移先と矛盾しない文言）。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// リンク先の固定外部 URL（モジュール doc「link の `href` を固定の外部
/// 絶対 URL にする・可視テキストとの整合」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// 各形の直前に置く短い形ラベル（`styled_text::text` の `Sm`/`Muted`）。
/// `copy_column` の eyebrow としても再利用するため、CSS フック用の
/// `attrs` を受け取る（反転パネル上でのコントラスト上書きに使う、
/// モジュール doc「A: パネル上の CTA・リンクの反転上書き」節参照）。
fn variant_label(label: &'static str, attrs: Vec<(&'static str, &'static str)>) -> Node {
    styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        attrs,
        vec![text(label)],
    )
}

/// 両形で共通のコピー列（小見出し・見出し・説明・CTA〔ボタン風リンク〕+
/// 誘導リンク）。`inverted` が `true` のとき（形 A のパネル上）は CTA・
/// リンクへ反転配色のフックを付与する。CTA・誘導リンクの可視テキストは
/// 遷移先（GitHub リポジトリ）と矛盾しない固定文言とする（モジュール doc
/// 「link の `href` を固定の外部絶対 URL にする・可視テキストとの整合」
/// 節参照）。
fn copy_column(
    eyebrow: &'static str,
    title: &'static str,
    body: &'static str,
    inverted: bool,
) -> Node {
    let cta_attrs = if inverted {
        vec![
            ("data-blocks-category-featured-banner-cta", ""),
            ("data-blocks-category-featured-banner-cta-inverted", ""),
        ]
    } else {
        vec![("data-blocks-category-featured-banner-cta", "")]
    };
    let link_attrs = if inverted {
        vec![("data-blocks-category-featured-banner-link", "")]
    } else {
        vec![]
    };
    let eyebrow_attrs = if inverted {
        vec![("data-blocks-category-featured-banner-eyebrow-inverted", "")]
    } else {
        vec![]
    };

    div(
        vec![("class", "blocks-category-featured-banner-copy")],
        vec![
            variant_label(eyebrow, eyebrow_attrs),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text(title)],
            ),
            styled_text::text(&TextProps::default(), vec![], vec![text(body)]),
            div(
                vec![("class", "blocks-category-featured-banner-actions")],
                vec![
                    link::root(
                        REPO,
                        &LinkProps {
                            external: true,
                            ..LinkProps::default()
                        },
                        cta_attrs,
                        vec![text("GitHub で見る")],
                    ),
                    link::root(
                        &format!("{REPO}/releases"),
                        &LinkProps {
                            external: true,
                            ..LinkProps::default()
                        },
                        link_attrs,
                        vec![text("リリースを見る")],
                    ),
                ],
            ),
        ],
    )
}

/// 形 A（R0823 主参照）: 全面画像 + 重ねた半透明パネル。広幅では画像の
/// 上にパネルが重なり、狭幅では画像の下にパネルが回る（モジュール doc
/// 「A: 絶対配置を使わず `grid-area` の重ね合わせで画像とパネルを重ねる」
/// 節）。
fn variant_overlay() -> Node {
    let panel = div(
        vec![("class", "blocks-category-featured-banner-panel")],
        vec![copy_column(
            "今季の注目",
            "アウトドア用品",
            "軽量テントから調理器具まで、週末の遠出に必要な一式をまとめました。",
            true,
        )],
    );

    div(
        vec![("class", "blocks-category-featured-banner-overlay")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    ..ImageProps::new(dummy_assets::BACKGROUND_SRC, "")
                },
                vec![("data-blocks-category-featured-banner-overlay-image", "")],
            ),
            panel,
        ],
    )
}

/// 形 B（R0608 集約）: 画像 + テキストの左右分割。狭幅では縦積み、広幅
/// （`48rem` 以上）では 2 列グリッドになる。
fn variant_split() -> Node {
    div(
        vec![("class", "blocks-category-featured-banner-split")],
        vec![
            image::image(
                &ImageProps {
                    fit: ImageFit::Cover,
                    shape: ImageShape::Rounded,
                    ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
                },
                vec![("data-blocks-category-featured-banner-split-image", "")],
            ),
            copy_column(
                "人気急上昇",
                "キッチン家電",
                "毎日の調理をすこし楽にする定番アイテムを集めました。",
                false,
            ),
        ],
    )
}

/// `category-featured-banner` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数（モジュール doc「2 形を 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-category-featured-banner-layout")],
        vec![
            variant_label("全面画像 + 半透明パネル（R0823 主参照）", vec![]),
            variant_overlay(),
            variant_label("画像 + テキストの左右分割（R0608 集約）", vec![]),
            variant_split(),
        ],
    )
}
```

## 原案差分メモ

- **形 A（主参照、R0823）**: 全面画像の上に半透明パネルを重ねます。
  `position: absolute` ではなく `display: grid` + 同一 `grid-area: 1 / 1`
  による重ね合わせを使い、狭い幅（コンテナ幅 48rem 未満）では通常フロー
  のまま画像の下へパネルが回ります（広い幅でのみ `grid` 化）。
- **形 B（集約元、R0608）**: 画像とコピー列を左右 2 分割にします。狭い
  幅では画像が上に来る縦積み、広い幅（48rem 以上）では 2 列グリッドに
  なります。
- 見出しはすべて `HeadingLevel::H3` を使用します（ページ側が `## Demo`
  として `h2` を出すため）。
- CTA は `button::button`（送信先を持てない `<button type="button">`）
  ではなく、ボタン風に装飾した `link::root` で描きます（`footer_cta_
  columns` と同じ判断）。パネル・CTA・誘導リンク・eyebrow の反転配色は
  色リテラルではなく `--fandhe-color-*` トークンと `color-mix()` のみで
  表現し、`link` の既定配色を `data-*` フックで個別に上書きしています。
- CTA は repository ルート（「GitHub で見る」）、誘導リンクは
  `/releases`（「リリースを見る」）へそれぞれ固定し、可視テキストが
  遷移先と矛盾しないようにしています。`href="#"` の死リンクは使って
  いません。
- ブレークポイントは `48rem` をリテラルで直書きしています（テーマの
  breakpoint トークンは `@media` 条件式の中では解決できないため）。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Image](../themes/image.md) / [Link](../themes/link.md)
