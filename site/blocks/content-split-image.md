# content-split-image

`badge` / `heading` / `text` / `image` / `icon` の 5 部品を合成した、見出し
と本文を片側の列に、画像または画面画像を反対側の列に置く 2 列コンテンツ
です。

次の 2 形を並記します。

- **sticky な画面画像（対応表 ID R0867、基準形）**: 本文をスクロールして
  いる間、画面画像が上部に留まります。
- **全高の画像（対応表 ID R0872）**: 左半分に全高の画像を張り、右に本文を
  置きます。

幅 lg（64rem）未満ではどちらも 1 列表示になり、画像が本文より上に来ま
す。文言はすべて架空のもので、データ取得・送信は行わない静的な表示例で
す。`<form>` は使用しません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextSize, TextVariant, TextWeight,
};

/// 装飾用の自作幾何アイコン（lucide 等の著作物を複製しないための単純図形、
/// `feature_expand::geo_icon` と同型の判断）。`path` へ `fill="none"` +
/// `stroke="currentColor"` を明示し、`icon` の `<svg>` 側が固定で持つ
/// `fill="currentColor"`（塗り面）を上書きして線画（ストローク）として
/// 描画する。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", path_d),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 特長リスト 1 項目分（架空の特長名 + 短い説明 + 幾何アイコン）。
struct Feature {
    icon_path_d: &'static str,
    label: &'static str,
    description: &'static str,
}

/// 形 A（sticky）の本文が持つ特長リスト（3 項目、架空文言）。
const FEATURES: [Feature; 3] = [
    Feature {
        icon_path_d: "M5 13l4 4L19 7",
        label: "決定的な出力",
        description: "同じ入力からは常に同じ HTML を生成し、差分を追跡しやすくします。",
    },
    Feature {
        icon_path_d: "M12 3l8 4.5v9L12 21l-8-4.5v-9L12 3z",
        label: "型で保証する構造",
        description: "スロットと props は Rust の型で表現され、不整合をコンパイル時に検出します。",
    },
    Feature {
        icon_path_d: "M12 8v4l3 3",
        label: "静的な表示のみ",
        description: "JS ハイドレーションを行わない docs サイト向けの、決定的な静的表示です。",
    },
];

/// 本文段落（架空文言、2 形で使い回して検索インデックスの増分を抑える）。
const PARAGRAPHS: &[&str] = &[
    "本文は node 木 API で組み立てた text ノードのみで構成し、文字列結合による HTML 生成は行いません。",
    "見出し・本文・画像はいずれも Themes の既存部品からのみ合成しています。",
    "この Demo はデータ取得や送信を行わない、静的な表示専用の合成例です。",
];

/// 各形の直前に置く短い形ラベル（`styled_text::text` の `Sm`/`Muted`）。
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

/// 本文段落 1 個（`margin: 0` 上書きフックを内蔵する。呼び出し箇所ごとに
/// 属性リテラルを重複させないため、フック名はここへ 1 箇所だけ書く）。
fn paragraph(body: &'static str) -> Node {
    styled_text::text(
        &TextProps::default(),
        vec![("data-blocks-content-split-image-paragraph", "")],
        vec![text(body)],
    )
}

/// eyebrow badge + 見出しの組（形 A・形 B で共通の頭部、
/// `content-columns-screenshot::demo` のヘッダー構成と同型）。
fn header(eyebrow: &'static str, title: &'static str) -> Node {
    div(
        vec![("class", "blocks-content-split-image-header")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-content-split-image-eyebrow", "")],
                vec![text(eyebrow)],
            ),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text(title)],
            ),
        ],
    )
}

/// 特長リスト 1 項目（アイコン + 太字ラベル + 説明）。
fn feature_item(feature: &Feature) -> Node {
    li(
        vec![("class", "blocks-content-split-image-feature")],
        vec![
            geo_icon(feature.icon_path_d),
            div(
                vec![("class", "blocks-content-split-image-feature-copy")],
                vec![
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Semibold,
                            ..TextProps::default()
                        },
                        vec![("data-blocks-content-split-image-feature-label", "")],
                        vec![text(feature.label)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![("data-blocks-content-split-image-feature-desc", "")],
                        vec![text(feature.description)],
                    ),
                ],
            ),
        ],
    )
}

/// 特長リスト（形 A の本文にのみ現れる、`ul`/`li` を素の core タグで組む）。
fn feature_list() -> Node {
    ul(
        vec![("class", "blocks-content-split-image-features")],
        FEATURES.iter().map(feature_item).collect(),
    )
}

/// 形 A（R0867 基準形）: 本文をスクロールしている間、画面画像が上部に
/// 留まる sticky 配置。DOM 順は header → 画像 → body（lg 未満の 1 列
/// 表示で画像が本文より上に来るようにするため、モジュール doc「sticky を
/// 実際に見せるためのスクロール枠」節参照）。
fn variant_sticky() -> Node {
    let header_node = header("導入ガイド", "スクロールに追従する画面画像");

    let body = div(
        vec![("class", "blocks-content-split-image-body")],
        vec![
            paragraph(PARAGRAPHS[0]),
            paragraph(PARAGRAPHS[1]),
            paragraph(PARAGRAPHS[2]),
            feature_list(),
        ],
    );

    let media = div(
        vec![("class", "blocks-content-split-image-sticky-media")],
        vec![image::image(
            &ImageProps {
                shape: ImageShape::Rounded,
                ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
            },
            vec![("data-blocks-content-split-image-sticky-image", "")],
        )],
    );

    let grid = div(
        vec![("class", "blocks-content-split-image-sticky-grid")],
        vec![header_node, media, body],
    );

    div(
        vec![
            ("class", "blocks-content-split-image-scroller"),
            ("tabindex", "0"),
            ("role", "region"),
            ("aria-label", "本文をスクロールできる領域"),
        ],
        vec![grid],
    )
}

/// 形 B（R0872）: 左半分に全高の画像を張り、右に本文を置く形。DOM 順は
/// 画像 → テキスト列（lg 未満の 1 列表示で画像が上に来るようにするため）。
fn variant_full_height() -> Node {
    let media = div(
        vec![("class", "blocks-content-split-image-full-media")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                shape: ImageShape::Rounded,
                ..ImageProps::new(dummy_assets::BACKGROUND_SRC, "")
            },
            vec![("data-blocks-content-split-image-full-image", "")],
        )],
    );

    let text_col = div(
        vec![("class", "blocks-content-split-image-text-col")],
        vec![
            header("導入ガイド", "左半分に全高の画像を敷く構成"),
            div(
                vec![("class", "blocks-content-split-image-body")],
                vec![paragraph(PARAGRAPHS[0]), paragraph(PARAGRAPHS[1])],
            ),
        ],
    );

    div(
        vec![("class", "blocks-content-split-image-full-grid")],
        vec![media, text_col],
    )
}

/// `content-split-image` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「2 形を 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-content-split-image-layout")],
        vec![
            variant_label("sticky な画面画像（R0867 基準形）"),
            variant_sticky(),
            variant_label("全高の画像（R0872）"),
            variant_full_height(),
        ],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0867・R0872。出典の固有名・ファイル名は記載しません）
からの意図的な差分は次のとおりです。

- sticky を実際に見せるため、形 A（R0867）全体を専用のスクロール枠
  （`max-height` + `overflow-y: auto`）で包みました。`.blocks-demo` 自体を
  スクロールコンテナにする方式（他 block の前例）ではなく、形 A だけを
  包む方式にしています。スクロール枠はキーボード操作者向けに
  `tabindex="0"` / `role="region"` / `aria-label` を持ちます。
- R0872 の画面左半分への絶対配置・`display: contents`・負のマージンは、
  2 列 grid（`align-items: stretch`）+ `object-fit: cover` の全高画像へ
  簡略化しました。
- 見出しを 1 段下げて `h3` にしました（ページ側が `## Demo` として `h2`
  を出すため）。小見出しの `h2` は出しません。
- eyebrow をスタイル付きの段落ではなく `badge` にしました。
- 背景の格子などの装飾 SVG は持ち込みません。
- アイコンは自作の幾何図形（線画）にしました。
- 画像は `dummy_assets` のプレースホルダーで `alt=""`（装飾扱い）に
  しました。
- 画像の固定幅によるはみ出し表現は持ち込みません。
- 文言・配色・余白は独自のもの、または既存テーマトークンにそのまま
  従います。
