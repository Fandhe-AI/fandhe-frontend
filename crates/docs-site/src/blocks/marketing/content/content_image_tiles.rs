//! `content-image-tiles` block（イシュー #2754。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、対応表 ID R0869 の 1 件のみを
//! 構造の参照元とする合成例。見出し + 本文/画像タイルの 2 列 + 下段の
//! 数値指標）。取得手段・ファイル名・内部コンポーネント識別子は記載
//! しない（`content_columns_screenshot` モジュール doc と同じライセンス
//! 上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `image` / `stat` の 4 部品を合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。
//!
//! # レイアウトとブレークポイント（lg = 64rem を境に列数を切り替える）
//!
//! `>= 64rem` で本文列と画像タイル列の 2 列、`< 64rem` で 1 列（本文の
//! 後にタイルが続く）にする。テーマの breakpoint トークンは `@media`
//! 条件式の中では解決できないため（CSS custom property は宣言側でのみ
//! 有効）、[`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg`]
//! （1024px = 64rem）と一致するリテラル値 `63.99rem` を [`LAYOUT_CSS`] へ
//! 直書きする（`content_columns_screenshot`/`login_04`/`footer_newsletter`
//! と同じ判断）。ルート grid の class
//! （`blocks-content-image-tiles-layout`）は [`Block::demo_class`]
//! （`blocks-content-image-tiles`）とは意図的に別名にする
//! （`content_columns_screenshot` と同じ Bugbot 教訓の回避）。DOM 順は
//! 「本文 → タイル」であり、1 列化時に `order` 指定なしでタイルが本文の
//! 後へ流れる。
//!
//! # タイルの配置（2 × 2、偶数番目を下へずらす）
//!
//! 4 枚のタイルは 2 列 × 2 段の grid に並べ、2・4 枚目（偶数番目）だけ
//! `data-blocks-content-image-tiles-offset` を明示的に付与して
//! `margin-top` でずらす。`:nth-child(even)` の CSS でも表現できるが、
//! テストで数えやすくするため属性を明示する側を採る。
//!
//! # 角丸をテーマトークンで揃える
//!
//! タイル画像は [`fandhe_frontend_pre_styled_ui::image::ImageShape::Rounded`]
//! （recipe が `var(--fandhe-radius-md)` を宣言する）を指定するだけで
//! 揃うため、[`LAYOUT_CSS`] 側で独自の `border-radius` は書かない。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `image::image` / `stat::root` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を
//! 黙って除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-content-image-tiles-*` 属性で渡し、[`LAYOUT_CSS`] 側も
//! 同じ属性セレクタで対応する。素の `div` には `class` がそのまま
//! 効くため、それらは従来どおり `.blocks-content-image-tiles-*`
//! クラスセレクタを使う。
//!
//! # 詳細度の罠（`text` recipe への勝ち方）
//!
//! `text::text`（`data-scope="text" data-part="root"`）の recipe は
//! margin を持つため、段落の `margin: 0` 上書きは
//! `[data-scope="text"][data-part="root"][data-blocks-content-image-tiles-paragraph]`
//! （詳細度 (0,3,0)）で行う（`content_columns_screenshot` と同じ判断）。
//!
//! # 見出しレベルに `H3` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため、見出しは
//! `HeadingLevel::H3` にする（`content_columns_screenshot` と同じ判断）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む
//! （`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言・数値指標はすべて架空のもの（実企業名・実クレデンシャル・
//! PII を含まない）。画像は [`crate::blocks::dummy_assets::PRODUCT_SRC`]
//! （ビルド時生成の、viewBox が正方形の商品プレースホルダー SVG）を使い、
//! `alt=""`（装飾扱い）で出力する。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps, ImageShape};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 本文の段落群（架空文言、1〜2 文程度に短くして検索インデックスの
/// サイズを抑える）。
const PARAGRAPHS: [&str; 2] = [
    "画像タイルは既存の image パーツを並べるだけで組み立てており、独自の画像コンポーネントは追加していません。",
    "下段の数値指標は stat パーツをそのまま並べた合成であり、送信処理やデータ取得は行いません。",
];

/// 下段の数値指標（架空値、`(ラベル, 値)` の組）。
const STATS: [(&str, &str); 4] = [
    ("導入チーム", "1,200+"),
    ("平均応答", "120ms"),
    ("稼働継続", "99.9%"),
    ("対応言語", "18"),
];

/// タイル 1 枚（正方形・角丸の商品プレースホルダー画像）。偶数番目
/// （2・4 枚目）は `data-blocks-content-image-tiles-offset` を付与して
/// 下へずらす（モジュール doc「タイルの配置」節）。
fn tile(offset: bool) -> Node {
    let mut attrs = vec![("class", "blocks-content-image-tiles-tile")];
    if offset {
        attrs.push(("data-blocks-content-image-tiles-offset", ""));
    }
    div(
        attrs,
        vec![image::image(
            &ImageProps {
                aspect_ratio: AspectRatio::Square,
                shape: ImageShape::Rounded,
                ..ImageProps::new(dummy_assets::PRODUCT_SRC, "")
            },
            vec![("data-blocks-content-image-tiles-image", "")],
        )],
    )
}

/// 数値指標 1 件分（`stat::root` + `label`/`value_text`）。
fn stat_item(label: &str, value: &str) -> Node {
    stat::root(
        Size::Md,
        vec![("data-blocks-content-image-tiles-stat", "")],
        vec![
            stat::label(vec![], vec![text(label)]),
            stat::value_text(vec![], vec![text(value)]),
        ],
    )
}

/// `content-image-tiles` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「レイアウトとブレークポイント」節）。
pub fn demo() -> Node {
    let header = heading(
        HeadingLevel::H3,
        &HeadingProps {
            size: HeadingSize::Xl3,
            weight: HeadingWeight::Bold,
        },
        vec![],
        vec![text("既存部品だけで画像タイルと指標を組み立てる")],
    );

    let body = div(
        vec![("class", "blocks-content-image-tiles-body")],
        PARAGRAPHS
            .iter()
            .map(|paragraph| {
                styled_text::text(
                    &TextProps {
                        variant: TextVariant::Muted,
                        ..TextProps::default()
                    },
                    vec![("data-blocks-content-image-tiles-paragraph", "")],
                    vec![text(*paragraph)],
                )
            })
            .collect(),
    );

    let tiles = div(
        vec![("class", "blocks-content-image-tiles-tiles")],
        vec![tile(false), tile(true), tile(false), tile(true)],
    );

    let columns = div(
        vec![("class", "blocks-content-image-tiles-columns")],
        vec![body, tiles],
    );

    let stats = div(
        vec![("class", "blocks-content-image-tiles-stats")],
        STATS
            .iter()
            .map(|(label, value)| stat_item(label, value))
            .collect(),
    );

    div(
        vec![("class", "blocks-content-image-tiles-layout")],
        vec![header, columns, stats],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/content-image-tiles/",
    title: "content-image-tiles",
    category: BlockCategory::Content,
    rust_source: "crates/docs-site/src/blocks/marketing/content/content_image_tiles.rs",
    demo_class: "blocks-content-image-tiles",
    parts: &[
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
        Part {
            label: "Stat",
            path: "/themes/stat/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `content_image_tiles` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` では
/// なく本ファイル内 private 定数として `super::stylesheet` 経由の
/// `push_css` で連結される）。
///
/// セレクタは `.blocks-content-image-tiles-*` と
/// `[data-blocks-content-image-tiles-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない（`content_columns_screenshot` と同じ名前
/// 空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-content-image-tiles-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-content-image-tiles-columns {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-8);\n  align-items: start;\n}\n\
.blocks-content-image-tiles-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n}\n\
[data-scope=\"text\"][data-part=\"root\"][data-blocks-content-image-tiles-paragraph] {\n  margin: 0;\n}\n\
.blocks-content-image-tiles-tiles {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-content-image-tiles-offset] {\n  margin-top: var(--fandhe-space-8);\n}\n\
[data-scope=\"image\"][data-part=\"root\"][data-blocks-content-image-tiles-image] {\n  display: block;\n  width: 100%;\n}\n\
.blocks-content-image-tiles-stats {\n  display: grid;\n  grid-template-columns: repeat(4, minmax(0, 1fr));\n  gap: var(--fandhe-space-6);\n  border-top: 1px solid var(--fandhe-color-border);\n  padding-top: var(--fandhe-space-6);\n}\n\
@media (max-width: 63.99rem) {\n  .blocks-content-image-tiles-columns {\n    grid-template-columns: 1fr;\n  }\n  .blocks-content-image-tiles-stats {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n";

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
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"image\"",
            "data-scope=\"stat\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(html.matches("<img").count(), 4);
        assert_eq!(
            html.matches("data-scope=\"stat\" data-part=\"root\"")
                .count(),
            4
        );
        assert_eq!(
            html.matches("data-blocks-content-image-tiles-offset")
                .count(),
            2
        );
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("id=\""));
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件・2 列 grid の宣言を
    /// 持つこと。
    #[test]
    fn layout_css_declares_lg_breakpoint_and_two_column_grids() {
        assert!(LAYOUT_CSS.contains("@media (max-width: 63.99rem)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: repeat(2, minmax(0, 1fr))"));
    }

    /// ルート grid class（`demo_class` とは別名）が `demo()` の出力へ
    /// 実際に現れること（モジュール doc「レイアウトとブレークポイント」
    /// 節の Bugbot 教訓の固定、`content_columns_screenshot` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-content-image-tiles-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-content-image-tiles-layout");
    }
}
