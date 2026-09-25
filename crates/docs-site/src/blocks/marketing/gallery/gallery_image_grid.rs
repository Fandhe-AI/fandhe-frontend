//! `gallery-image-grid` block（イシュー #2778。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、Marketing/Gallery カテゴリの
//! 最初の block。対応表 ID R0501/R0505/R0506/R0507/R0508/R0509 の 6 件を
//! 参照元とする合成例。取得手段・ファイル名・内部コンポーネント識別子は
//! 記載しない（`content_image_tiles` 等と同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `badge`（見出し上のタグライン）/ `heading` / `text` / `image` の 4 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は
//! 追加しない。
//!
//! # 6 バリエーション
//!
//! 見出しエリアの下に、キャプション付きで列数・枚数の異なる 6 パターンを
//! 縦に並べる（`bento_asymmetric_rows` と同じ「状態違いの並記」）。
//!
//! 1. R0501: 1 枚をフル幅で見せる最小形（`columns` 属性なし）。
//! 2. R0505: 正方形の画像を 2 枚並べる。
//! 3. R0506: `md` 以上で 3 列に並べる基準形。
//! 4. R0507: `md` 以上で 4 列に並べる。
//! 5. R0508: 先頭 1 枚だけ 2 列幅にする featured 配置。`aspect-ratio` は
//!    セル自身の幅基準で決まるため、2 列幅のまま [`AspectRatio::Square`]
//!    （1:1）を使うと隣接する 1 列幅セル（1:1）の 2 倍の高さになり行に
//!    空白が生じる。[`LAYOUT_CSS`] の featured 用オーバーライドで
//!    `aspect-ratio: 2 / 1` を当て、1 列幅セルと高さを揃える。
//! 6. R0509: 6 枚を 2 段 × 3 列で並べる。
//!
//! # ブレークポイントは `md` 単一境界に単純化する
//!
//! 参照元の個別記述には「lg で 3 列」「lg で 4 列」という表現があるが、
//! 本 block の受け入れ条件で明示される境界は「`md` 未満で 1 列」のみで
//! あり、`md`/`lg` の 2 段階を新設する根拠が無いため
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`]（768px =
//! 48rem）単一の `min-width` に単純化する（過剰なブレークポイント設計を
//! しない判断、`content_image_tiles` が単一 `lg` 境界のみを使うのと同型）。
//!
//! # `alt=""` にする理由
//!
//! 同一プレースホルダー画像を複数枚並べる際、内容を伝えない同一文言の
//! `alt` を繰り返すとスクリーンリーダーで同じ文言が連呼される（WCAG
//! 1.1.1、`bento_asymmetric_rows`/`content_image_tiles`/`login_04` で
//! Bugbot 指摘済みの教訓）ため、装飾用途として `alt=""` を使う。
//!
//! # `<form>` を持たない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo はフォーム・
//! 送信処理・データ取得を持たない静的な合成例である。文言はすべて
//! 架空のものであり、実企業名・実クレデンシャル・PII を含まない。
//! 画像は [`crate::blocks::dummy_assets::PRODUCT_SRC`]（ビルド時生成の
//! 商品プレースホルダー SVG）を使う。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge`/`heading::heading`/`text::text`/`image::image` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を
//! 黙って除去する契約を持つため、グリッド列数・featured 指定は
//! `data-blocks-gallery-image-grid-*` 属性で渡し、[`LAYOUT_CSS`] 側も
//! 同じ属性セレクタで対応する。素の `div`/`p` には `class` がそのまま
//! 効くため、グリッド本体・キャプション・スタックのラッパはクラス
//! セレクタを使う。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/gallery-image-grid/",
    title: "gallery-image-grid",
    category: BlockCategory::Gallery,
    rust_source: "crates/docs-site/src/blocks/marketing/gallery/gallery_image_grid.rs",
    demo_class: "blocks-gallery-image-grid",
    parts: &[
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Heading",
            path: "/themes/heading/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `gallery_image_grid` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-gallery-image-grid-*` と `[data-blocks-gallery-image-grid-*]`
/// のみを用いる。
const LAYOUT_CSS: &str = "\
.blocks-gallery-image-grid-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-gallery-image-grid-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  align-items: flex-start;\n}\n\
.blocks-gallery-image-grid-section {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
[data-blocks-gallery-image-grid-caption] {\n  margin: 0;\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-size-sm);\n}\n\
.blocks-gallery-image-grid-grid {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-4);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-gallery-image-grid-image] {\n  display: block;\n  width: 100%;\n}\n\
@media (min-width: 48rem) {\n  .blocks-gallery-image-grid-grid[data-blocks-gallery-image-grid-columns=\"2\"] {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n  .blocks-gallery-image-grid-grid[data-blocks-gallery-image-grid-columns=\"3\"] {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n  }\n  .blocks-gallery-image-grid-grid[data-blocks-gallery-image-grid-columns=\"4\"] {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n  [data-blocks-gallery-image-grid-featured] {\n    grid-column: span 2;\n  }\n  [data-blocks-gallery-image-grid-featured] [data-scope=\"image\"][data-part=\"root\"] {\n    aspect-ratio: 2 / 1;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_forms() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"image\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        // 1 + 2 + 3 + 4 + 5 + 6 = 21 枚。
        assert_eq!(html.matches("<img").count(), 21);
        assert_eq!(
            html.matches("data-blocks-gallery-image-grid-caption")
                .count(),
            6
        );
        assert_eq!(
            html.matches("data-blocks-gallery-image-grid-featured")
                .count(),
            1
        );
        assert!(!html.contains("<form"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("id=\""));
    }

    /// `demo()` が決定的（呼び出しごとに同じ `Node`）であること。
    #[test]
    fn demo_is_deterministic() {
        assert_eq!(render(&demo()), render(&demo()));
    }

    /// [`LAYOUT_CSS`] が `Breakpoint::Md`（768px = 48rem）と一致する
    /// `@media (min-width: 48rem)` を含み、featured セルの 2 列幅指定を
    /// 持つこと。
    #[test]
    fn layout_css_declares_md_breakpoint_and_featured_span() {
        assert_eq!(
            fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md.min_width(),
            "768px"
        );
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("grid-column: span 2"));
    }
}
