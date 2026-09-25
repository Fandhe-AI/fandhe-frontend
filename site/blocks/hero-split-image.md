# hero-split-image

`fandhe-frontend-pre-styled-ui` の `badge` / `heading` / `text` / `button` /
`image` / `avatar` / `icon` 部品を合成した、左テキスト・右画像の最も基本的な
分割ヒーローです。Blocks セクションは新規部品を追加するものではなく、既存の
Themes/Primitives 部品を組み合わせた実例集であることに注意してください
（主参照は対応表 ID R0129、R1010 の 7:5 分割を採用。R0535・R0544・R0536・
R0548・R0125・R0543 を構造として集約。出典の固有名・ファイル名は記載しま
せん）。

Demo は次の 4 形を並記しています。

- **基準形**（R0129/R1010）: badge → 見出し → リード文 → CTA ボタン 2 個
  （コピー列）+ 正方形画像。
- **チェック付き箇条書き**（R0535/R0544 集約）: リード文と CTA の間に
  チェック付き箇条書き 3 件 + 縦長画像。
- **社会的証明**（R0536）: 基準形に加え CTA の下へ重なりアバター 3 枚 +
  `+N` の残数表示 + 短い補足テキスト。
- **反転 + 全面画像**（R0548/R0125/R0543 集約）: 幅 `64rem`（`lg`）以上でのみ
  画像を左側へ入れ替えます（DOM 順はどの形も「コピー列 → 画像」で固定し、
  `grid-column` の指定のみで見た目の左右を入れ替えます）。

いずれの形も `64rem` 未満では 1 列（コピー列が上、画像が下）へ折り返し、
`64rem` 以上で左右 2 列になります。

本 Demo は静的な表示例であり、`<form>` 要素を一切持たず、データの取得・
送信・状態管理を行いません。CTA ボタンは `type="button"` のまま送信先を
持たず、値は送信されません。文言はすべて独自に書いた架空のものであり、
実企業名・実クレデンシャル・PII を含みません。画像はビルド時生成の
プレースホルダー SVG（`alt=""`）です。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, li, text, ul, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 自作の幾何アイコン（線画。モジュール doc「箇条書きのアイコン」節
/// 参照）。`path` へ `fill="none"` + `stroke="currentColor"` を明示し、
/// `icon` の `<svg>` 側が固定で持つ `fill="currentColor"`（塗り面）を
/// 上書きして線画（ストローク）として描画する。
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

/// チェックマークの幾何アイコン。
fn check_icon() -> Node {
    geo_icon("M5 13l4 4L19 7")
}

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

/// CTA ボタン 2 個（Solid + Outline）。
fn actions(primary: &'static str, secondary: &'static str) -> Node {
    div(
        vec![("class", "blocks-hero-split-image-actions")],
        vec![
            button::button(
                &ButtonProps {
                    size: Size::Lg,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text(primary)],
            ),
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    size: Size::Lg,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text(secondary)],
            ),
        ],
    )
}

/// 右列（または反転時は左列）に置く画像。`data-blocks-hero-split-image-
/// image` フックへ `aspect_ratio`/`shape` を差し替えて渡す（形ごとに
/// 正方形/縦長/全面高さを使い分ける、モジュール doc §「詳細度」節参照）。
fn media(aspect_ratio: AspectRatio, shape: ImageShape) -> Node {
    image::image(
        &ImageProps {
            fit: ImageFit::Cover,
            aspect_ratio,
            shape,
            ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
        },
        vec![("data-blocks-hero-split-image-image", "")],
    )
}

/// 形 A（R0129 基準形 + R1010 の 7:5 分割）: badge → 見出し → リード文 →
/// CTA 2 個（コピー列）+ 正方形画像。
fn variant_basic() -> Node {
    let copy = div(
        vec![("class", "blocks-hero-split-image-copy")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text("新規リリース")]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("公開までの導線を、迷わず一本化する")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "テンプレートと雛形を組み合わせ、初期構築から公開までの手間を大きく減らします。",
                )],
            ),
            actions("今すぐ試す", "詳しく見る"),
        ],
    );
    div(
        vec![("class", "blocks-hero-split-image-row")],
        vec![copy, media(AspectRatio::Square, ImageShape::Rounded)],
    )
}

/// チェック付き項目 1 件分（チェックマーク + 短い文言）。
fn checklist_item(label: &'static str) -> Node {
    li(
        vec![("class", "blocks-hero-split-image-checklist-item")],
        vec![check_icon(), text(label)],
    )
}

/// チェック付き項目 3 件（架空文言）。
const CHECKLIST_ITEMS: [&str; 3] = [
    "既定エスケープ済みの HTML 出力",
    "外部依存ゼロの描画コア",
    "単一実行ファイルでの配布",
];

/// 形 B（R0535/R0544 集約）: リード文と CTA の間にチェック付き箇条書き 3
/// 件 + 縦長画像。
fn variant_checklist() -> Node {
    let copy = div(
        vec![("class", "blocks-hero-split-image-copy")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text("導入ガイド")]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("既存のチームでもそのまま採用できる")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "導入前に確認したいポイントを、あらかじめまとめました。",
                )],
            ),
            ul(
                vec![("class", "blocks-hero-split-image-checklist")],
                CHECKLIST_ITEMS
                    .iter()
                    .map(|label| checklist_item(label))
                    .collect(),
            ),
            actions("導入を始める", "資料を見る"),
        ],
    );
    div(
        vec![("class", "blocks-hero-split-image-row")],
        vec![copy, media(AspectRatio::Portrait, ImageShape::Rounded)],
    )
}

/// 重なりアバター 3 枚（社会的証明の装飾）。[`dummy_assets::AVATAR_SRC`]
/// （ビルド時生成 SVG、`data:` URI ではない）を画像に使う
/// （`super::banner_announcement_pill::avatar_stack` と同型）。
fn avatar_trio() -> Vec<Node> {
    ["HF", "EV", "KB"]
        .iter()
        .map(|initials| {
            avatar::root(
                &AvatarProps {
                    size: Size::Sm,
                    stacked: true,
                    ..AvatarProps::default()
                },
                vec![("data-blocks-hero-split-image-avatar", "")],
                vec![
                    avatar::image(ImageStatus::Loaded, dummy_assets::AVATAR_SRC, "", vec![]),
                    avatar::fallback(ImageStatus::Loaded, vec![], vec![text(*initials)]),
                ],
            )
        })
        .collect()
}

/// `+N` の残数表示（画像を持たない fallback のみの avatar、`<img>` を
/// 増やさないための選択、モジュール doc「社会的証明」節参照）。
fn avatar_overflow(label: &'static str) -> Node {
    avatar::root(
        &AvatarProps {
            size: Size::Sm,
            stacked: true,
            ..AvatarProps::default()
        },
        vec![("data-blocks-hero-split-image-avatar-overflow", "")],
        vec![avatar::fallback(
            ImageStatus::Error,
            vec![],
            vec![text(label)],
        )],
    )
}

/// 形 C（R0536）: A に加え CTA 下へ重なりアバター 3 枚 + `+N` + 短い補足
/// テキスト（社会的証明）。
fn variant_social_proof() -> Node {
    let mut avatars = avatar_trio();
    avatars.push(avatar_overflow("+12"));
    let proof = div(
        vec![("class", "blocks-hero-split-image-proof")],
        vec![
            avatar::group(vec![("aria-hidden", "true")], avatars),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("すでに 500 チーム以上が利用しています")],
            ),
        ],
    );
    let copy = div(
        vec![("class", "blocks-hero-split-image-copy")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text("導入実績")]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("チームの合言葉を、そのまま画面へ")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "配色トークンを差し替えるだけで、複数ブランドの画面を作り分けられます。",
                )],
            ),
            actions("始める", "事例を見る"),
            proof,
        ],
    );
    div(
        vec![("class", "blocks-hero-split-image-row")],
        vec![copy, media(AspectRatio::Square, ImageShape::Rounded)],
    )
}

/// 形 D（R0548/R0125/R0543 集約）: `lg` 以上で画像を左へ反転（DOM 順は
/// 「コピー列 → 画像」のまま、`[data-blocks-hero-split-image-reverse]` +
/// `grid-column` のみで見た目を入れ替える、モジュール doc「DOM 順を固定し」
/// 節参照）。画像はセルいっぱいに広がる（全面画像）。
fn variant_reverse_full() -> Node {
    let copy = div(
        vec![("class", "blocks-hero-split-image-copy")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text("新しい体験")]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("画像を主役にした、もう一つの見せ方")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "同じ構成のまま、画像とテキストの左右だけを入れ替えて印象を変えられます。",
                )],
            ),
            actions("詳しく見る", "デモを試す"),
        ],
    );
    let media = div(
        vec![("class", "blocks-hero-split-image-reverse-media")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                ..ImageProps::new(dummy_assets::SCREENSHOT_SRC, "")
            },
            vec![("data-blocks-hero-split-image-reverse-image", "")],
        )],
    );
    div(
        vec![
            ("class", "blocks-hero-split-image-row"),
            ("data-blocks-hero-split-image-reverse", ""),
        ],
        vec![copy, media],
    )
}

/// `hero-split-image` の Demo 本体。呼び出しごとに同一の `Node` を返す純
/// 関数（モジュール doc「4 形を 1 つの Demo に並記する」節）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-hero-split-image-layout")],
        vec![
            variant_label("基準形（R0129/R1010）"),
            variant_basic(),
            variant_label("チェック付き箇条書き + 縦長画像（R0535/R0544 集約）"),
            variant_checklist(),
            variant_label("社会的証明（R0536）"),
            variant_social_proof(),
            variant_label("反転 + 全面画像（R0548/R0125/R0543 集約）"),
            variant_reverse_full(),
        ],
    )
}
```

## 原案差分メモ

参照（対応表 ID R0129 が主参照、R1010 の 7:5 分割を採用。R0535/R0544/R0536/
R0548/R0125/R0543 を構造として集約。出典の固有名・ファイル名は記載しま
せん）からの意図的な差分は次のとおりです。

- 見出しは参照側の `h2` 相当ではなく `h3`（ページ側が `## Demo` として
  `h2` を出すため）にしています。eyebrow は `badge` で表現しました。
- チェック付き項目のアイコンは、lucide 等の著作物を複製しない自作の
  幾何チェックマーク（折れ線）にしています。
- 画像はすべてビルド時生成のプレースホルダー SVG（`alt=""`）で、実写風の
  合成は持ち込んでいません。
- R0124（台形切り抜き）・R1011（斜め切り抜き）の装飾的な画像トリミングは
  採用していません。
- R0533（モバイル中央寄せ）・R1009（端末モックアップ枠）・R1013（画像
  ずらし配置）は個別の形として起こさず、本メモへの言及に留めています
  （4 形で構造差を十分示せるため）。
- 反転（形 D）は絶対配置や DOM 順の並べ替えではなく、`grid-column` の
  指定のみで実現しています（DOM 順は常に「コピー列 → 画像」で固定）。
- 社会的証明の `+N` 残数表示は、画像を追加せず fallback のみの avatar で
  表現しています。
- 文言（badge・見出し・本文・チェック項目・補足テキスト）はすべて独自に
  書き直した架空のものです。
- `id` や `aria-labelledby` は出力しません（宙に浮いた ARIA 参照・id 重複
  を構造的に避けるため）。
- ブレークポイントは `64rem`（`lg`）に固定しています。
