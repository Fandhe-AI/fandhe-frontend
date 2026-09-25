# feature-image-cards

`badge` / `heading` / `text` / `card` / `image` の 5 部品を合成した、画像 →
短い見出し → 説明の順に積んだカードを 2〜4 列で並べる feature セクション
です。

2 つの静的インスタンスを縦に並べています。最初のインスタンスは中央寄せの
導入部の下へ 4:3 の画像カードを 3 枚、2 つ目のインスタンスは左寄せの導入
部の下へ正方形の画像カードを 4 枚並べます。いずれも幅 md（48rem）未満は
1 列、md 以上で 2 列、lg（64rem）以上でそれぞれ 3 列 / 4 列へ広がります。
文言・データはすべて架空のもので、データ取得・送信は行わない静的な表示
例です。`<form>` は使用しません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text as core_text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// カード 1 件分の架空データ（見出し・説明・使用画像）。
struct FeatureCard {
    title: &'static str,
    body: &'static str,
    src: &'static str,
}

/// インスタンス A（4:3 画像・3 枚）のカードデータ。
const CARDS_LANDSCAPE: [FeatureCard; 3] = [
    FeatureCard {
        title: "決定的なビルド",
        body: "同じ入力からは常に同じ静的ファイルを生成し、差分レビューを容易にします。",
        src: dummy_assets::PRODUCT_SRC,
    },
    FeatureCard {
        title: "型で表現する構造",
        body: "スロットと props は Rust の型で表現され、不整合はコンパイル時に検出されます。",
        src: dummy_assets::SCREENSHOT_SRC,
    },
    FeatureCard {
        title: "静的な表示のみ",
        body: "JS ハイドレーションを行わない、決定的な静的表示専用の合成例です。",
        src: dummy_assets::BACKGROUND_SRC,
    },
];

/// インスタンス B（正方形画像・4 枚）のカードデータ。
const CARDS_SQUARE: [FeatureCard; 4] = [
    FeatureCard {
        title: "既定エスケープ",
        body: "テキスト補間は既定でエスケープされ、迂回経路は限定されています。",
        src: dummy_assets::PRODUCT_SRC,
    },
    FeatureCard {
        title: "外部依存ゼロの描画コア",
        body: "描画コアは外部クレートに依存せず、サプライチェーンの露出面を抑えます。",
        src: dummy_assets::SCREENSHOT_SRC,
    },
    FeatureCard {
        title: "単一実行ファイル配布",
        body: "SSR/SSG のいずれも単一バイナリへまとめてデプロイできます。",
        src: dummy_assets::LOGO_SRC,
    },
    FeatureCard {
        title: "無 JS のドキュメント",
        body: "docs サイト自体は JS ハイドレーションを行わない静的表示です。",
        src: dummy_assets::BACKGROUND_SRC,
    },
];

/// 中央寄せのヘッダー（eyebrow badge + 見出し + リード文、インスタンス A）。
fn header_centered() -> Node {
    div(
        vec![("class", "blocks-feature-image-cards-header")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-feature-image-cards-eyebrow", "")],
                vec![core_text("特長")],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![core_text("画像付きカードで特長を紹介する")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-image-cards-lead", "")],
                vec![core_text(
                    "各カードは画像・見出し・説明の順に積み、画面幅に応じて列数を切り替えます。",
                )],
            ),
        ],
    )
}

/// 左寄せのヘッダー（badge + 見出し + リード文、インスタンス B）。
fn header_start() -> Node {
    div(
        vec![
            ("class", "blocks-feature-image-cards-header"),
            ("data-align", "start"),
        ],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-feature-image-cards-eyebrow", "")],
                vec![core_text("できること")],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![core_text("正方形画像で並べる 4 つの特長")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-feature-image-cards-lead", "")],
                vec![core_text(
                    "同じ縦横比の画像で揃えることで、密度の高いグリッドを組めます。",
                )],
            ),
        ],
    )
}

/// カード 1 枚分（画像 → 見出し → 説明）を組み立てる。`aspect_ratio` で
/// インスタンス A（4:3）/B（正方形）を切り替える。
fn feature_card(card: &FeatureCard, aspect_ratio: AspectRatio) -> Node {
    let image_node = image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio,
            ..ImageProps::new(card.src, "")
        },
        vec![("data-blocks-feature-image-cards-image", "")],
    );
    let title = heading::heading(
        HeadingLevel::H4,
        &HeadingProps::default(),
        vec![],
        vec![core_text(card.title)],
    );
    let body = styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-feature-image-cards-desc", "")],
        vec![core_text(card.body)],
    );

    card::root(
        CardProps::default(),
        vec![("data-blocks-feature-image-cards-card", "")],
        vec![
            card::cover(vec![], vec![image_node]),
            card::body(
                vec![("class", "blocks-feature-image-cards-body")],
                vec![title, body],
            ),
        ],
    )
}

/// 1 インスタンス分（導入部 + カードグリッド）を組み立てる。`four_columns`
/// が `true` のとき lg で 4 列（インスタンス B）、`false` のとき lg で 3 列
/// （インスタンス A）になる。
fn instance(
    header: Node,
    cards: &[FeatureCard],
    aspect_ratio: AspectRatio,
    four_columns: bool,
) -> Node {
    let mut grid_attrs = vec![("class", "blocks-feature-image-cards-grid")];
    if four_columns {
        grid_attrs.push(("data-columns", "4"));
    }
    let card_nodes: Vec<Node> = cards
        .iter()
        .map(|card| feature_card(card, aspect_ratio))
        .collect();

    div(
        vec![("class", "blocks-feature-image-cards-instance")],
        vec![header, div(grid_attrs, card_nodes)],
    )
}

/// `feature-image-cards` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数（他 block と同じ状態を持たない設計）。
#[must_use]
pub fn demo() -> Node {
    let note_a = styled_text::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(
            "中央寄せの導入部・4:3 画像・3 列 → 2 列 → 1 列の構成例。",
        )],
    );
    let note_b = styled_text::text(
        &TextProps {
            size: TextSize::Xs,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![core_text(
            "左寄せの導入部・正方形画像・4 列 → 2 列 → 1 列の構成例。",
        )],
    );

    div(
        vec![("class", "blocks-feature-image-cards-layout")],
        vec![
            note_a,
            instance(
                header_centered(),
                &CARDS_LANDSCAPE,
                AspectRatio::Landscape,
                false,
            ),
            note_b,
            instance(header_start(), &CARDS_SQUARE, AspectRatio::Square, true),
        ],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0476・R1153・R1160。出典の固有名・ファイル名は記載
しません）からの意図的な差分は次のとおりです。

- R0476 を基準形（画像 + 見出し + 説明の 3 列カード）とし、最初の
  インスタンスにしました。
- R1153 の中央寄せ見出しを最初のインスタンスへ統合しました。R1153 が持つ
  「lg で 2 列固定」と「背景色つきの帯」は持ち込まず、md 幅での 2 列表示
  で代替しています。
- R1160 の正方形画像 4 列を 2 つ目のインスタンスにしました。
- 参照元に見られる、狭い幅で画像を下に回す列反転は採らず、DOM 順を
  「画像 → 見出し → 説明」に固定しました。
- 画像は `dummy_assets` のプレースホルダーと `alt=""`（装飾扱い）にしま
  した。
- 見出しは `h3`/`h4` に下げました（ページ側が `## Demo` として `h2` を
  出すため）。
- 文言・配色・余白は独自のもの、または既存のテーマトークンにそのまま
  従います。
