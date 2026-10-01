# incentives-split-header

`heading` / `text` / `image` / `icon` の 4 部品を合成した、特典紹介の
導入ブロックです。上段は左に見出し・本文、右に大きな画像を置く 2 列、
下段はアイコン付きの特典項目 3 点を横に並べます。Blocks は既存部品の
合成例であり、新しい UI 部品は追加しません。

主参照は対応表 ID R1015（導入 2 列 + 特典 3 点）で、集約元は 1 件のため
Demo は 1 形で足ります。特典の文言はすべて架空のデータであり、実在の
企業名・PII は含みません。画像はビルド時生成の同梱プレースホルダー SVG
です。

本 Demo は無 JS の静的表示のみであり、`<form>` を含みません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{
    self, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageFit, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 特典項目 1 件分の架空データ（見出し・説明・自作アイコンのパス）。
struct Incentive {
    title: &'static str,
    body: &'static str,
    icon_path_d: &'static str,
}

/// 特典項目 3 件（架空、実在の企業・製品とは無関係）。自作の単純な幾何
/// パスのみを使い、lucide 等の著作物は複製しない。
const INCENTIVES: [Incentive; 3] = [
    Incentive {
        title: "送料無料",
        body: "注文金額にかかわらず、国内配送はすべて無料です。",
        icon_path_d: "M3 7h11v9H3zM14 10h4l3 3v3h-7zM6.5 19a1.5 1.5 0 100-3 1.5 1.5 0 000 3zM17.5 19a1.5 1.5 0 100-3 1.5 1.5 0 000 3z",
    },
    Incentive {
        title: "30 日以内の返品",
        body: "到着から 30 日以内であれば、理由を問わず返品できます。",
        icon_path_d: "M4 4v6h6M4.5 15a8 8 0 108-11.3",
    },
    Incentive {
        title: "サポート窓口",
        body: "注文に関するお問い合わせに、専任スタッフが対応します。",
        icon_path_d: "M12 21a9 9 0 100-18 9 9 0 000 18zM12 8v5l3 3",
    },
];

/// 装飾用の自作幾何アイコン（`fill="none"` + `stroke="currentColor"` の
/// 線画、`feature_three_column_icons::geo_icon` と同型の判断）。
fn geo_icon(path_d: &'static str) -> Node {
    icon(
        &IconProps {
            size: Size::Xl,
            ..IconProps::default()
        },
        vec![("data-blocks-incentives-split-header-icon", "")],
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

/// 左列（見出し + リード文）。
fn intro() -> Node {
    div(
        vec![("class", "blocks-incentives-split-header-intro")],
        vec![
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("安心してお買い物いただくために")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-incentives-split-header-lead", "")],
                vec![text(
                    "送料・返品・サポートのすべてで、ご注文の不安を取り除きます。",
                )],
            ),
        ],
    )
}

/// 右列（4:3 画像、装飾扱いの `alt=""`）。
fn media() -> Node {
    div(
        vec![("class", "blocks-incentives-split-header-media")],
        vec![image::image(
            &ImageProps {
                fit: ImageFit::Cover,
                aspect_ratio: AspectRatio::Landscape,
                shape: ImageShape::Rounded,
                ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
            },
            vec![("data-blocks-incentives-split-header-image", "")],
        )],
    )
}

/// 特典項目 1 件（アイコン → 見出し → 説明）。
fn incentive_item(i: &Incentive) -> Node {
    div(
        vec![("class", "blocks-incentives-split-header-item")],
        vec![
            geo_icon(i.icon_path_d),
            heading::heading(
                HeadingLevel::H4,
                &HeadingProps::default(),
                vec![],
                vec![text(i.title)],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Sm,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-incentives-split-header-item-desc", "")],
                vec![text(i.body)],
            ),
        ],
    )
}

/// `incentives-split-header` の Demo 本体。呼び出しごとに同一の `Node`
/// を返す純関数（他 block と同じ状態を持たない設計）。
#[must_use]
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-incentives-split-header-layout")],
        vec![
            div(
                vec![("class", "blocks-incentives-split-header-top")],
                vec![intro(), media()],
            ),
            div(
                vec![("class", "blocks-incentives-split-header-items")],
                INCENTIVES.iter().map(incentive_item).collect(),
            ),
        ],
    )
}
```

## 原案差分メモ

- 主参照 R1015（導入 2 列 + 特典 3 点）の集約元は 1 件のみのため、Demo は
  1 形のみで構成します。
- 文言・アイコンは独自に作成した架空のものです（送料無料・返品・
  サポート窓口）。参照元の文言・配色・装飾・アイコンは持ち込みません。
- 画像は参照元のものではなく、同梱のビルド時生成プレースホルダー SVG
  （`crate::blocks::dummy_assets`）を使用します。
- 幅 md（48rem）未満では、上段の 2 列（見出し + 画像）・下段の特典 3 点を
  いずれも 1 列へ縦積みします。
- アイコンは自作の単純な幾何パス（線画）のみを使い、lucide 等の著作物は
  複製していません。

関連情報: [Heading](../themes/heading.md) / [Text](../themes/text.md) /
[Image](../themes/image.md) / [Icon](../themes/icon.md)
