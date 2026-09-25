//! `gallery-masonry` block（イシュー #2779。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、Marketing/Gallery カテゴリの
//! 2 件目。対応表 ID R0504（比率の違う 9 枚の段組み）を参照元とする合成
//! 例。取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （`gallery_image_grid` 等と同じライセンス上の転記制限）。
//!
//! # 使用部品
//!
//! `badge`（見出し上のタグライン）/ `heading` / `text` / `image` の 4 部品を
//! 合成する（[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は
//! 追加しない。
//!
//! # masonry の実装方式（CSS `column-count`）
//!
//! JS を使わない無 JS 制約下で比率の異なる画像を段組みへ流し込むため、
//! CSS の `column-count`（複数段カラム）を使う。段内の画像は上から下へ
//! 詰められてから次の段へ移る（グリッドのような行揃えにはならない）
//! 素朴な masonry 風の見え方であり、実際の masonry ライブラリのような
//! 最短列への割り当て最適化は行わない。この挙動は原稿側の差分メモにも
//! 明記する。
//!
//! # 比率の与え方（image variant + ラッパ側オーバーライドの併用）
//!
//! イシューの要件どおり、各画像の比率は 2 つの手段を併用して与える。
//!
//! 1. 7 枚は [`AspectRatio`] の variant（`Portrait`/`Landscape`/`Square`/
//!    `Video`）で与える。
//! 2. 残り 2 枚は [`AspectRatio::Auto`] にしたうえで、ラッパ `div` の
//!    `data-blocks-gallery-masonry-ratio="tall"`（2:3）/`"wide"`（3:2）
//!    属性を介して [`LAYOUT_CSS`] 側の `aspect-ratio` 直接指定で上書きする
//!    （既存 variant では表現できない比率を追加する手段の実例）。
//!
//! # breakpoint は sm/lg の 2 段（md は置かない）
//!
//! イシューが明示する境界は「`sm`（640px）未満で 1 段」「最大 3 段」の
//! 2 点のみで、中間段の境界は指定されていない。過剰なブレークポイント
//! 設計をしない判断（`gallery_image_grid` が `md` 単一境界に単純化した
//! のと同型の判断軸）から、2 段目を
//! [`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Sm`]（640px）、
//! 3 段目を同 [`Breakpoint::Lg`]（1024px）に割り当て、`md` 境界は新設
//! しない。
//!
//! # `alt=""` にする理由
//!
//! 同一プレースホルダー画像を 9 枚並べる際、内容を伝えない同一文言の
//! `alt` を繰り返すとスクリーンリーダーで同じ文言が連呼される（WCAG
//! 1.1.1、`gallery_image_grid` 等で Bugbot 指摘済みの教訓）ため、装飾
//! 用途として `alt=""` を使う。
//!
//! # `width: 100%` を明示する理由
//!
//! `image` の base CSS は `max-width: 100%` のみで `width` を固定しない
//! ため、`column-count` のカラム幅いっぱいにタイル化するには
//! [`LAYOUT_CSS`] 側で明示的に `width: 100%` を当てる必要がある
//! （`gallery_image_grid` の同種オーバーライドと同じ理由）。
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
//! 黙って除去する契約を持つため、比率オーバーライドの指定は
//! `data-blocks-gallery-masonry-*` 属性で渡し、[`LAYOUT_CSS`] 側も同じ
//! 属性セレクタで対応する。素の `div` には `class` がそのまま効くため、
//! 段組み本体・見出しエリア・スタックのラッパはクラスセレクタを使う。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    self as styled_heading, HeadingLevel, HeadingProps, HeadingSize,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// 段組みタイル 1 枚分の比率指定。`ratio_override` が `Some` のときは
/// `aspect` を [`AspectRatio::Auto`] にしたうえでラッパ属性値として使う
/// （モジュール doc「比率の与え方」節）。
struct Item {
    aspect: AspectRatio,
    ratio_override: Option<&'static str>,
}

const ITEMS: [Item; 9] = [
    Item {
        aspect: AspectRatio::Portrait,
        ratio_override: None,
    },
    Item {
        aspect: AspectRatio::Landscape,
        ratio_override: None,
    },
    Item {
        aspect: AspectRatio::Auto,
        ratio_override: Some("tall"),
    },
    Item {
        aspect: AspectRatio::Square,
        ratio_override: None,
    },
    Item {
        aspect: AspectRatio::Video,
        ratio_override: None,
    },
    Item {
        aspect: AspectRatio::Portrait,
        ratio_override: None,
    },
    Item {
        aspect: AspectRatio::Auto,
        ratio_override: Some("wide"),
    },
    Item {
        aspect: AspectRatio::Landscape,
        ratio_override: None,
    },
    Item {
        aspect: AspectRatio::Square,
        ratio_override: None,
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
        vec![text("比率違いの画像を段組みで流し込む")],
    );
    let lead = styled_text::text(
        &TextProps {
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![],
        vec![text(
            "比率の異なる 9 枚の画像を CSS の段組みへ流し込みます。sm 未満は 1 段、sm 以上で 2 段、lg 以上で 3 段になります。",
        )],
    );
    div(
        vec![("class", "blocks-gallery-masonry-header")],
        vec![eyebrow, title, lead],
    )
}

/// 段組み 1 タイル分（画像 1 枚）。`ratio_override` があればラッパへ
/// `data-blocks-gallery-masonry-ratio` 属性を付与する。
fn item(entry: &Item) -> Node {
    let mut attrs = vec![("class", "blocks-gallery-masonry-item")];
    if let Some(ratio) = entry.ratio_override {
        attrs.push(("data-blocks-gallery-masonry-ratio", ratio));
    }
    div(
        attrs,
        vec![image::image(
            &ImageProps {
                aspect_ratio: entry.aspect,
                shape: ImageShape::Rounded,
                ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
            },
            vec![("data-blocks-gallery-masonry-image", "")],
        )],
    )
}

/// `gallery-masonry` の Demo 本体。呼び出しごとに同一の `Node` を返す
/// 純関数。
pub fn demo() -> Node {
    let grid = div(
        vec![("class", "blocks-gallery-masonry-grid")],
        ITEMS.iter().map(item).collect(),
    );
    div(
        vec![("class", "blocks-gallery-masonry-stack")],
        vec![header(), grid],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ。
pub const BLOCK: Block = Block {
    path: "/blocks/gallery-masonry/",
    title: "gallery-masonry",
    category: BlockCategory::Gallery,
    rust_source: "crates/docs-site/src/blocks/marketing/gallery/gallery_masonry.rs",
    demo_class: "blocks-gallery-masonry",
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

/// `gallery_masonry` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。セレクタは
/// `.blocks-gallery-masonry-*` と `[data-blocks-gallery-masonry-*]`、
/// およびそれらで絞り込んだ `[data-scope="image"]` のみを用いる。
const LAYOUT_CSS: &str = "\
.blocks-gallery-masonry-stack {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-gallery-masonry-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  align-items: flex-start;\n}\n\
.blocks-gallery-masonry-grid {\n  column-count: 1;\n  column-gap: var(--fandhe-space-4);\n}\n\
.blocks-gallery-masonry-item {\n  break-inside: avoid;\n  margin-bottom: var(--fandhe-space-4);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-gallery-masonry-image] {\n  display: block;\n  width: 100%;\n}\n\
.blocks-gallery-masonry-item[data-blocks-gallery-masonry-ratio=\"tall\"] [data-scope=\"image\"][data-part=\"root\"] {\n  aspect-ratio: 2 / 3;\n}\n\
.blocks-gallery-masonry-item[data-blocks-gallery-masonry-ratio=\"wide\"] [data-scope=\"image\"][data-part=\"root\"] {\n  aspect-ratio: 3 / 2;\n}\n\
@media (min-width: 40rem) {\n  .blocks-gallery-masonry-grid {\n    column-count: 2;\n  }\n}\n\
@media (min-width: 64rem) {\n  .blocks-gallery-masonry-grid {\n    column-count: 3;\n  }\n}\n";

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
        assert_eq!(html.matches("<img").count(), 9);
        assert_eq!(html.matches("alt=\"\"").count(), 9);
        assert_eq!(html.matches("data-blocks-gallery-masonry-ratio").count(), 2);
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

    /// [`LAYOUT_CSS`] が sm/lg breakpoint（640px/1024px）と一致する
    /// `@media` を含み、段数・タイル配置・比率オーバーライドの規則を
    /// 持つこと。
    #[test]
    fn layout_css_declares_sm_lg_breakpoints_and_column_count() {
        assert_eq!(
            fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Sm.min_width(),
            "640px"
        );
        assert_eq!(
            fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg.min_width(),
            "1024px"
        );
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("column-count: 1"));
        assert!(LAYOUT_CSS.contains("break-inside: avoid"));
        assert!(LAYOUT_CSS.contains("width: 100%"));
        assert!(LAYOUT_CSS.contains("aspect-ratio: 2 / 3"));
        assert!(LAYOUT_CSS.contains("aspect-ratio: 3 / 2"));
    }
}
