# gallery-image-grid

`badge` / `heading` / `text` / `image` の 4 部品だけで組み立てた画像グリッドのギャラリーです。新しい UI 部品は追加していません。

タグライン（badge）→ 見出し（heading）→ 説明（text）の下に、列数・枚数の異なる画像グリッドを 6 パターン並べています。

- 1 枚をフル幅で見せる最小形
- 正方形の画像を 2 枚並べる
- `md` 以上で 3 列に並べる基準形
- `md` 以上で 4 列に並べる
- 先頭 1 枚だけ 2 列幅にする featured 配置
- 6 枚を 2 段 × 3 列で並べる

`md`（768px）未満はいずれも 1 列に縮退します。`<form>` は使わず、送信処理・データ取得を持たない静的な合成例です。画像はすべて同一のプレースホルダーで実データを持ちません。

## Rust コード

```rust
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, p, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    self as styled_heading, HeadingLevel, HeadingProps, HeadingSize,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// 1 バリエーション分の設定（列数・比率・枚数・featured の有無）。
struct Variant {
    caption: &'static str,
    /// `md` 以上での列数。`1` はグリッド化不要（`columns` 属性を付与しない）
    /// ことを表す（R0501: 1 枚フル幅）。
    columns: u8,
    aspect: AspectRatio,
    image_count: usize,
    /// 先頭 1 枚を 2 列幅にする（R0508）。
    featured: bool,
}

const VARIANTS: [Variant; 6] = [
    Variant {
        caption: "1 枚をフル幅で見せる最小形",
        columns: 1,
        aspect: AspectRatio::Landscape,
        image_count: 1,
        featured: false,
    },
    Variant {
        caption: "正方形の画像を 2 枚並べる",
        columns: 2,
        aspect: AspectRatio::Square,
        image_count: 2,
        featured: false,
    },
    Variant {
        caption: "md 以上で 3 列に並べる基準形",
        columns: 3,
        aspect: AspectRatio::Square,
        image_count: 3,
        featured: false,
    },
    Variant {
        caption: "md 以上で 4 列に並べる",
        columns: 4,
        aspect: AspectRatio::Square,
        image_count: 4,
        featured: false,
    },
    Variant {
        caption: "先頭 1 枚だけ 2 列幅にする featured 配置",
        columns: 3,
        aspect: AspectRatio::Square,
        image_count: 5,
        featured: true,
    },
    Variant {
        caption: "6 枚を 2 段 × 3 列で並べる",
        columns: 3,
        aspect: AspectRatio::Square,
        image_count: 6,
        featured: false,
    },
];

/// 見出しエリア（タグライン → 見出し → 説明）。
fn header() -> Node {
    let eyebrow = badge::badge(&BadgeProps::default(), vec![], vec![text("Gallery")]);
    let title = styled_heading::heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl3,
            ..HeadingProps::default()
        },
        vec![],
        vec![text("既存部品だけで画像グリッドを組み立てる")],
    );
    let lead = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "列数・枚数の異なる 6 パターンを、image パーツの並べ方だけで表現します。",
        )],
    );
    div(
        vec![("class", "blocks-gallery-image-grid-header")],
        vec![eyebrow, title, lead],
    )
}

/// バリエーション 1 件分のキャプション。
fn caption(label: &str) -> Node {
    p(
        vec![("data-blocks-gallery-image-grid-caption", "")],
        vec![text(label)],
    )
}

/// グリッド 1 セル分（画像 1 枚）。`featured` なら先頭セルへ 2 列幅指定を
/// 付与する（モジュール doc「6 バリエーション」節）。
fn image_cell(aspect: AspectRatio, featured: bool) -> Node {
    let mut attrs = vec![("data-blocks-gallery-image-grid-cell", "")];
    if featured {
        attrs.push(("data-blocks-gallery-image-grid-featured", ""));
    }
    div(
        attrs,
        vec![image::image(
            &ImageProps {
                aspect_ratio: aspect,
                ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
            },
            vec![("data-blocks-gallery-image-grid-image", "")],
        )],
    )
}

/// バリエーション 1 件分のグリッド本体。
fn grid(variant: &Variant) -> Node {
    let mut attrs = vec![("class", "blocks-gallery-image-grid-grid")];
    let columns_str;
    if variant.columns >= 2 {
        columns_str = variant.columns.to_string();
        attrs.push((
            "data-blocks-gallery-image-grid-columns",
            columns_str.as_str(),
        ));
    }
    let cells = (0..variant.image_count)
        .map(|i| image_cell(variant.aspect, variant.featured && i == 0))
        .collect();
    div(attrs, cells)
}

/// バリエーション 1 件分（キャプション + グリッド）。
fn section(variant: &Variant) -> Node {
    div(
        vec![("class", "blocks-gallery-image-grid-section")],
        vec![caption(variant.caption), grid(variant)],
    )
}

/// `gallery-image-grid` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    let mut children = vec![header()];
    children.extend(VARIANTS.iter().map(section));
    div(vec![("class", "blocks-gallery-image-grid-stack")], children)
}
```
