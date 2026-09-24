//! `bento-asymmetric-rows` block（イシュー #2745。親トラッキング #2744
//! 「Blocks 目的別パーツ拡充ツリー」・Marketing/Bento カテゴリ配下、規模 L
//! のため前半 #2745（本ファイル、骨格・主要領域）と後半 #2746（残りの
//! バリエーション・原稿仕上げ）へ分割済み）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `card` / `image` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約）。新しい UI 部品は追加しない。
//!
//! # #2745（本件）と #2746（後続）のスコープ境界
//!
//! 本ファイルが実装するのは主参照 R0769 に対応する基準形（6 列グリッドで
//! 1 行目 4+2・2 行目 2+4 の幅違いセルを組む配置）のみである。集約元の
//! 残りのバリエーション（3+3・2+2+2・3 列ジグザグ・淡色カード・下段の
//! 小さな feature 一覧等）は後続の #2746 で追加する。取得手段・ファイル名・
//! 内部コンポーネント識別子は記載しない（購入者限定素材のライセンス上の
//! 転記制限、`docs/design/motion-reference-adoption-policy.md` §9 と同じ
//! 方針。記載してよいのは対応表 ID `R0769` のみ）。
//!
//! # `drop_class_attr` を考慮した CSS フックの選び方
//!
//! `card::root` / `badge::badge` / `heading::heading` / `text::text`
//! （`styled_text` として import、`fandhe_frontend_core::text` との名前
//! 衝突を避けるため `blog_featured_article` と同じ判断）/ `image::image`
//! はいずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を
//! 黙って除去してから合成する契約を持つため、セルの幅区分は
//! `data-blocks-bento-asymmetric-rows-cell="wide"|"narrow"` 属性で渡し、
//! [`LAYOUT_CSS`] 側も同じ属性セレクタで `grid-column: span` を切り替える。
//! `card::cover`/`card::body` と素の `div` には `class` がそのまま効くため、
//! それらは `class` で渡す。
//!
//! # セル本文の見出し・説明文の間隔
//!
//! `card::body` は `card::header` と異なり `gap` を持たない base スタイル
//! （`fandhe_frontend_pre_styled_ui::card` 参照）のため、`heading::heading`
//! と `card::description` をそのまま入れると余白なしで密着表示になる。
//! `card::title` を使わず素の `heading::heading` を消費する本 block では
//! `card::header` へ差し替える判断は採らず、`card::body` に
//! `blocks-bento-asymmetric-rows-body` class を付与して `gap` を持たせる
//! （[`LAYOUT_CSS`] 参照。Bugbot 指摘 #3162 で是正）。
//!
//! # 見出しレベル（H3/H4）の理由
//!
//! Demo は本文の `h2`「Demo」配下に挿入されるため、block 側の最上位見出しは
//! `<h3>` にする（`heading::heading` を素通しで消費）。`card::title` は
//! `<h3>` 固定でセル見出しに使うと文書アウトラインが重複するため、セルの
//! 見出しは `heading::heading(HeadingLevel::H4, ...)` にする（`card::title`
//! を使わない判断。`blog_featured_article` のグリッドカード見出しと同じ
//! 判断軸）。
//!
//! # レスポンシブをモバイルファースト（`min-width`）で書く理由
//!
//! 本 block の仕様は「md 未満 1 列・md 以上 2 列・lg 以上 6 列（4+2/2+4）」
//! という段階的な拡張であり、`max-width` 型の縮小記述より `min-width` 型の
//! 拡張記述のほうが意図に忠実に読める。48rem/64rem は
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`/`Lg` の
//! `min_width()`（768px/1024px、16px 基準の rem 換算）と一致する値であり、
//! 本ファイル末尾の `#[cfg(test)]` がドリフトを検知する。
//!
//! # `<form>` を使わない・実データを持たない
//!
//! `crate::blocks` モジュール doc の不変条件どおり、本 Demo は `<form>` を
//! 出力しない。機能名・説明文はすべて架空のものであり、実企業名・実サービス
//! 名・実クレデンシャル・PII を含まない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{self, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::image::{self, AspectRatio, ImageProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// セルの幅区分（[`LAYOUT_CSS`] の `grid-column: span` を切り替える唯一の
/// 軸）。`props::*` へは昇格せず、本 block ローカルの列挙型とする
/// （`alert::Severity`/`progress::ProgressShape` と同じ判断軸）。
#[derive(Clone, Copy)]
enum CellWidth {
    /// 6 列中 4 列分（lg 以上）。
    Wide,
    /// 6 列中 2 列分（lg 以上）。
    Narrow,
}

impl CellWidth {
    /// [`LAYOUT_CSS`] の `[data-blocks-bento-asymmetric-rows-cell="..."]`
    /// セレクタと一致させる値。
    const fn value(self) -> &'static str {
        match self {
            CellWidth::Wide => "wide",
            CellWidth::Narrow => "narrow",
        }
    }
}

/// 1 枚分のセルデータ（架空の SaaS 機能名 + 1 行説明 + 幅区分）。
struct Cell {
    title: &'static str,
    description: &'static str,
    width: CellWidth,
}

/// 4 セル（1 行目 4+2、2 行目 2+4）。並び順がグリッドの既定の自動配置
/// （`grid-auto-flow: row`）と組み合わさって基準形（R0769）の配置を作る
/// 不変条件を、本ファイル末尾の `#[cfg(test)]` が固定する。
const CELLS: [Cell; 4] = [
    Cell {
        title: "Unified Workspace",
        description: "散らばっていたツールを 1 つの画面に集約し、切り替えの手間を無くします。",
        width: CellWidth::Wide,
    },
    Cell {
        title: "Instant Handoff",
        description: "担当者の引き継ぎをワンクリックで完了します。",
        width: CellWidth::Narrow,
    },
    Cell {
        title: "Version History",
        description: "変更履歴を自動保存し、いつでも巻き戻せます。",
        width: CellWidth::Narrow,
    },
    Cell {
        title: "Cross-team Reporting",
        description: "部門をまたいだ進捗を 1 枚のレポートにまとめ、共有の手間を減らします。",
        width: CellWidth::Wide,
    },
];

/// 見出しエリア（eyebrow badge + `<h3>` + リード文）。
fn header() -> Node {
    div(
        vec![("class", "blocks-bento-asymmetric-rows-header")],
        vec![
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-bento-asymmetric-rows-eyebrow", "")],
                vec![text("Platform")],
            ),
            heading::heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    ..HeadingProps::default()
                },
                vec![("data-blocks-bento-asymmetric-rows-title", "")],
                vec![text("チームの仕事をひとつの流れにまとめる")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Md,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![("data-blocks-bento-asymmetric-rows-lead", "")],
                vec![text(
                    "分断されがちな作業を、幅の異なるカードで用途ごとに見渡せるようにしました。",
                )],
            ),
        ],
    )
}

/// 1 枚分の bento セル（`card`。画像を上、見出しと説明を下に置く）。
fn cell(item: &Cell) -> Node {
    card::root(
        CardProps::default(),
        vec![("data-blocks-bento-asymmetric-rows-cell", item.width.value())],
        vec![
            card::cover(
                vec![("class", "blocks-bento-asymmetric-rows-cover")],
                vec![image::image(
                    &ImageProps {
                        aspect_ratio: AspectRatio::Landscape,
                        ..ImageProps::new(
                            dummy_assets::SCREENSHOT_SRC,
                            "機能のプレースホルダー画像",
                        )
                    },
                    vec![],
                )],
            ),
            card::body(
                vec![("class", "blocks-bento-asymmetric-rows-body")],
                vec![
                    heading::heading(
                        HeadingLevel::H4,
                        &HeadingProps::default(),
                        vec![],
                        vec![text(item.title)],
                    ),
                    card::description(vec![], vec![text(item.description)]),
                ],
            ),
        ],
    )
}

/// `bento-asymmetric-rows` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数。見出しエリアの下に 6 列（lg 以上）/2 列（md 以上）/1 列
/// （md 未満）で切り替わるグリッドを置く。
pub fn demo() -> Node {
    let cells: Vec<Node> = CELLS.iter().map(cell).collect();
    div(
        vec![("class", "blocks-bento-asymmetric-rows")],
        vec![
            header(),
            div(vec![("class", "blocks-bento-asymmetric-rows-grid")], cells),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/bento-asymmetric-rows/",
    title: "bento-asymmetric-rows",
    category: BlockCategory::Bento,
    rust_source: "crates/docs-site/src/blocks/marketing/bento/bento_asymmetric_rows.rs",
    demo_class: "blocks-bento-asymmetric-rows",
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

/// `bento_asymmetric_rows` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `super::stylesheet`
/// から連結される）。48rem/64rem の根拠はモジュール doc「レスポンシブを
/// モバイルファーストで書く理由」節参照（本ファイル末尾の `#[cfg(test)]`
/// が `Breakpoint::Md`/`Lg` とのドリフトを検知する）。
const LAYOUT_CSS: &str = "\
.blocks-bento-asymmetric-rows-header {\n  display: flex;\n  flex-direction: column;\n  align-items: flex-start;\n  gap: var(--fandhe-space-2);\n  margin-bottom: var(--fandhe-space-6);\n}\n\
.blocks-bento-asymmetric-rows-grid {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-bento-asymmetric-rows-cover img {\n  width: 100%;\n}\n\
.blocks-bento-asymmetric-rows-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1-5);\n}\n\
@media (min-width: 48rem) {\n  .blocks-bento-asymmetric-rows-grid {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n  [data-blocks-bento-asymmetric-rows-cell] {\n    grid-column: span 1;\n  }\n}\n\
@media (min-width: 64rem) {\n  .blocks-bento-asymmetric-rows-grid {\n    grid-template-columns: repeat(6, minmax(0, 1fr));\n  }\n  [data-blocks-bento-asymmetric-rows-cell=\"wide\"] {\n    grid-column: span 4;\n  }\n  [data-blocks-bento-asymmetric-rows-cell=\"narrow\"] {\n    grid-column: span 2;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, CellWidth, LAYOUT_CSS};
    use fandhe_frontend_core::render;
    use fandhe_frontend_pre_styled_ui::recipe::Breakpoint;

    /// [`LAYOUT_CSS`] の 48rem/64rem が
    /// `fandhe_frontend_pre_styled_ui::recipe::Breakpoint::Md`/`Lg` の
    /// `min_width()`（768px/1024px、16px 基準で 48rem/64rem）と実際に
    /// 一致し、`span 4`/`span 2` を含むこと（モジュール doc「レスポンシブを
    /// モバイルファーストで書く理由」節が参照する対応のドリフト検知）。
    #[test]
    fn layout_css_declares_both_breakpoints_and_spans() {
        assert_eq!(Breakpoint::Md.min_width(), "768px");
        assert_eq!(Breakpoint::Lg.min_width(), "1024px");
        // 768px / 16 = 48rem, 1024px / 16 = 64rem（`Breakpoint::min_width`
        // の rustdoc・`docs/design/pre-styled-ui-scale-tokens.md` §3.6 と
        // 同じ px→rem 換算）。
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
        assert!(LAYOUT_CSS.contains("grid-column: span 4"));
        assert!(LAYOUT_CSS.contains("grid-column: span 2"));
    }

    /// [`super::CELLS`] の幅区分が基準形（R0769、1 行目 4+2・2 行目 2+4）と
    /// 一致し、[`demo`] の出力にちょうど 2 件ずつの `wide`/`narrow` が
    /// この順で現れること。
    #[test]
    fn demo_renders_expected_wide_narrow_sequence() {
        let widths: Vec<&str> = super::CELLS.iter().map(|c| c.width.value()).collect();
        assert_eq!(widths, vec!["wide", "narrow", "narrow", "wide"]);
        assert!(matches!(super::CELLS[0].width, CellWidth::Wide));
        assert!(matches!(super::CELLS[3].width, CellWidth::Wide));

        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-bento-asymmetric-rows-cell=\"wide\"")
                .count(),
            2
        );
        assert_eq!(
            html.matches("data-blocks-bento-asymmetric-rows-cell=\"narrow\"")
                .count(),
            2
        );
        assert!(!html.contains("<form"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"data:"));
    }
}
