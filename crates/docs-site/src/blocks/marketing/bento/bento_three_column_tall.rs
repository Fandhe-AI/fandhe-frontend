//! `bento-three-column-tall` block（イシュー #2748。親 #2747「Blocks
//! 目的別パーツ拡充ツリー」#2730 配下、両端のセルが縦 2 行にまたがる
//! 3 列 bento の骨格・主要領域を実装する。本イシューは全体（親 #2747）を
//! 2 分割した前半にあたり、後半（CTA 行の追加・一部セルのメディアを
//! コード表示枠/端末風の枠へ差し替え・状態の並記・「差分メモ」節の追加）は
//! #2749 が担う）。
//!
//! # 使用部品
//!
//! `badge`（eyebrow）+ `heading`（見出し帯 H3・各セル見出し H4）+ `text`
//! （リード文・各セル説明文）+ `card`（セル本体）+ `image`（各セルの
//! メディア）の 5 部品を合成する（[`BLOCK`] の `parts` に一致させる契約。
//! #2749 で `button`/`code` が追加される見込み）。
//!
//! # レイアウト（両端セルが縦 2 行にまたがる 3 列グリッド）
//!
//! `>= 64rem`（`fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Lg` =
//! 1024px と一致するリテラル値、`blog_list_image`/
//! `blog_featured_with_list` と同じ判断。テーマの breakpoint トークンは
//! `@media` 条件式の中では解決できないため直書きする）で 3 列 2 行の
//! グリッドにし、1 枚目・4 枚目のセルを `grid-row: span 2` で縦 2 行へ
//! またがせ、2 枚目・3 枚目を中央列の 1 行目・2 行目へ積む。`< 64rem` では
//! 1 列積みへ切り替え、`grid-row` を一切指定しないため縦長セルも自動的に
//! 通常の高さへ戻る（明示的な打ち消し宣言を書く必要がない）。
//!
//! # 見出しレベルに `H3`/`H4` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため、見出し帯は
//! `HeadingLevel::H3`、各セル見出しは `HeadingLevel::H4` にする
//! （`blog_list_image` と同じ判断）。`card::title` は `<h3>` 固定のため
//! セル見出しには使わず、`heading(HeadingLevel::H4, ...)` を `card::body`
//! 直下へ子として渡す。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `badge::badge`/`heading::heading`/`text::text`/`image::image` は
//! いずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を
//! 黙って除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-bento-three-column-tall-*` 属性で渡し、[`LAYOUT_CSS`] 側も
//! 同じ属性セレクタで対応する。`card::root` も `drop_class_attr` を経由する
//! ため、各セルの配置（`grid-column`/`grid-row`）は `data-blocks-
//! bento-three-column-tall-cell` の値（`start`/`center-top`/
//! `center-bottom`/`end`）へ紐づける。素の `div` には `class` がそのまま
//! 効くため、`intro`/`grid` ラッパーは従来どおり
//! `.blocks-bento-three-column-tall-*` クラスセレクタを使う
//! （`crate::blocks` モジュール doc「CSS フックが `class` と `[data-*]` で
//! 混在する理由」節と同じ判断軸）。
//!
//! # `id` 属性・aria 参照を持たない
//!
//! `crates/docs-site/tests/blocks_contract.rs` の
//! `demo_output_has_no_dangling_aria_references_or_duplicate_ids` が
//! 全 block 横断で検証するため、本 Demo は `id`/`aria-controls`/
//! `aria-labelledby`/`aria-describedby` のいずれも出力しない。
//!
//! # `#2749` に残す範囲
//!
//! CTA ボタン行（R0107 由来）、一部セルのメディアをコード表示枠・端末風
//! の枠へ差し替え/追加する構成（R0768 由来）、「1 枚目のみ縦長 + CTA」等の
//! 状態の並記、原稿「差分メモ」節は本イシューでは扱わない
//! （イシュー #2748/#2749 のスコープ境界、実装計画参照）。
//!
//! # `<form>` を持たない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。文言・画像はすべて架空のもの（`dummy_assets` のビルド時
//! 生成プレースホルダー）であり、実企業名・実クレデンシャル・PII を
//! 含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 1 枚分のセルデータ（架空の開発者向けプラットフォームの機能紹介、
/// 実企業名・実サービス名は含まない）。`slot` はグリッド内の配置を表す
/// 識別子で、[`LAYOUT_CSS`] の `[data-blocks-bento-three-column-tall-cell]`
/// セレクタの値と一致させる。
struct BentoCell {
    slot: &'static str,
    title: &'static str,
    description: &'static str,
    image_src: &'static str,
    aspect: AspectRatio,
}

/// セル 4 件（架空、実データなし）。1 枚目（`start`）・4 枚目（`end`）が
/// 縦長（[`AspectRatio::Portrait`]）、2・3 枚目（`center-top`/
/// `center-bottom`）は横長（[`AspectRatio::Video`]）にする
/// （モジュール doc「レイアウト」節参照）。
const CELLS: [BentoCell; 4] = [
    BentoCell {
        slot: "start",
        title: "統合ダッシュボード",
        description: "複数サービスの稼働状況を 1 画面へ集約して表示します。",
        image_src: dummy_assets::SCREENSHOT_SRC,
        aspect: AspectRatio::Portrait,
    },
    BentoCell {
        slot: "center-top",
        title: "自動デプロイ",
        description: "コミットからビルド・検証・配信までを自動化します。",
        image_src: dummy_assets::PRODUCT_SRC,
        aspect: AspectRatio::Video,
    },
    BentoCell {
        slot: "center-bottom",
        title: "チーム権限管理",
        description: "ロールごとに閲覧・操作範囲を細かく制御します。",
        image_src: dummy_assets::LOGO_SRC,
        aspect: AspectRatio::Video,
    },
    BentoCell {
        slot: "end",
        title: "利用量アラート",
        description: "しきい値を超えた利用量を検知し即座に通知します。",
        image_src: dummy_assets::BACKGROUND_SRC,
        aspect: AspectRatio::Portrait,
    },
];

/// 見出し帯（eyebrow badge + 見出し + リード文）。CTA 行は #2749 で追加する。
fn intro() -> Node {
    div(
        vec![("class", "blocks-bento-three-column-tall-intro")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![],
                vec![text("プラットフォーム機能")],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![],
                vec![text("必要な機能をひとつの基盤に")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "運用・デプロイ・権限管理をまとめて提供する、開発者向けプラットフォームの主要機能です。",
                )],
            ),
        ],
    )
}

/// 1 枚分の bento セル（`card` + カバー画像 + 見出し/説明）を組み立てる。
fn cell(item: &BentoCell) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-bento-three-column-tall-cell", item.slot)],
        vec![
            card::cover(
                vec![],
                vec![image::image(
                    &ImageProps {
                        aspect_ratio: item.aspect,
                        ..ImageProps::new(item.image_src, "")
                    },
                    vec![],
                )],
            ),
            card::body(
                vec![("class", "blocks-bento-three-column-tall-body")],
                vec![
                    heading::heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(item.title)],
                    ),
                    styled_text::text(
                        &TextProps {
                            size: TextSize::Sm,
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(item.description)],
                    ),
                ],
            ),
        ],
    )
}

/// `bento-three-column-tall` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（見出し帯 + 4 枚セルのグリッド、モジュール doc「レイアウト」
/// 節）。
pub fn demo() -> Node {
    let cells: Vec<Node> = CELLS.iter().map(cell).collect();
    div(
        vec![("class", "blocks-bento-three-column-tall")],
        vec![
            intro(),
            div(
                vec![("class", "blocks-bento-three-column-tall-grid")],
                cells,
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/bento-three-column-tall/",
    title: "bento-three-column-tall",
    category: BlockCategory::Bento,
    rust_source: "crates/docs-site/src/blocks/marketing/bento/bento_three_column_tall.rs",
    demo_class: "blocks-bento-three-column-tall",
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
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Image",
            path: "/themes/image/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `bento_three_column_tall` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で本ファイル内 private
/// 定数として `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-bento-three-column-tall-*` と
/// `[data-blocks-bento-three-column-tall-cell]` のみを用い、他 block や
/// 部品の素のセレクタへ影響させない。
const LAYOUT_CSS: &str = "\
.blocks-bento-three-column-tall {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-bento-three-column-tall-intro {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  max-width: 40rem;\n}\n\
.blocks-bento-three-column-tall-grid {\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-bento-three-column-tall-cell] {\n  display: flex;\n  flex-direction: column;\n  height: 100%;\n}\n\
.blocks-bento-three-column-tall-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  flex: 1;\n}\n\
@media (min-width: 64rem) {\n  .blocks-bento-three-column-tall-grid {\n    grid-template-columns: repeat(3, minmax(0, 1fr));\n    grid-template-rows: repeat(2, auto);\n  }\n  [data-blocks-bento-three-column-tall-cell=\"start\"] {\n    grid-column: 1;\n    grid-row: 1 / span 2;\n  }\n  [data-blocks-bento-three-column-tall-cell=\"center-top\"] {\n    grid-column: 2;\n    grid-row: 1;\n  }\n  [data-blocks-bento-three-column-tall-cell=\"center-bottom\"] {\n    grid-column: 2;\n    grid-row: 2;\n  }\n  [data-blocks-bento-three-column-tall-cell=\"end\"] {\n    grid-column: 3;\n    grid-row: 1 / span 2;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する部品・構造・非対話制約を満たしていることの単体
    /// 回帰（`crates/docs-site/tests/blocks_contract.rs` の横断検査と重複
    /// し過ぎない範囲での個別固定）。
    #[test]
    fn demo_composes_expected_parts_and_avoids_dead_links() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"card\" data-part=\"root\"",
            "data-scope=\"image\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("data-scope=\"card\" data-part=\"root\"")
                .count(),
            4,
            "bento-three-column-tall should render exactly 4 cards"
        );
        for slot in ["start", "center-top", "center-bottom", "end"] {
            let needle = format!("data-blocks-bento-three-column-tall-cell=\"{slot}\"");
            assert!(
                html.contains(&needle),
                "demo output should place a cell at slot {slot}"
            );
        }
        for absent in ["<form", "<script", "src=\"data:", " id=\""] {
            assert!(
                !html.contains(absent),
                "bento-three-column-tall should never contain {absent}"
            );
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイント条件・グリッド配置を
    /// 持つこと。
    #[test]
    fn layout_css_declares_lg_breakpoint_and_span_two_cells() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("grid-template-columns: repeat(3"));
        assert!(LAYOUT_CSS.contains("grid-row: 1 / span 2"));
        assert!(LAYOUT_CSS.contains("[data-blocks-bento-three-column-tall-cell"));
    }
}
