//! `comparison-feature-rows` block（イシュー #2823。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、対応表 ID R0438（1 件）を構造の
//! 参照元とする合成例。見出しブロックの下に、機能ごとの比較行を縦に
//! 並べる「機能別の比較行」レイアウト）。取得手段・ファイル名・内部
//! コンポーネント識別子は記載しない（`docs/design/motion-reference-
//! adoption-policy.md` §9 と同じライセンス上の転記制限）。
//!
//! **Marketing / Comparison カテゴリで最初の block**（`super`
//! （`comparison/mod.rs`）参照。カテゴリ雛形からの卒業も本 block が担う）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `icon` / `separator` の 5 部品を合成する
//! （[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # レイアウト構造（`## Demo` 見出しに続く 1 インスタンス）
//!
//! 見出しブロック（タグライン `badge` + セクション見出し `heading H3` +
//! リード文 `text`）の下に、機能行を [`ROWS`] の件数だけ縦に並べる。各行は
//! 「アイコン付きの機能名（`icon` + `heading H4`）」「自社の説明
//! （`badge` ラベル + `text`）」「他社の説明（`badge` ラベル + `text`）」の
//! 3 要素で構成し、行と行の間は [`fandhe_frontend_pre_styled_ui::separator`]
//! で区切る（行数 - 1 本、最後の行の後には置かない。`changelog_timeline`
//! の「separator を最後のエントリで省く理由」と同型の判断）。
//!
//! # レスポンシブ（`LAYOUT_CSS` 参照）
//!
//! 既定（狭い幅）は 3 要素を縦積みにする。`>= 48rem`（md）で行を 2 列
//! グリッドへ変え、機能名を全幅（`grid-column: 1 / -1`）にして自社・他社を
//! 横並びにする。`>= 64rem`（lg）で行を 3 列グリッドへ変え、機能名・自社・
//! 他社を横一列に並べる。ブレークポイントのリテラル rem 値は
//! [`fandhe_frontend_pre_styled_ui`] の `Breakpoint`（md=768px/lg=1024px）に
//! 合わせている（`@media` 内ではトークン変数を使えないため）。
//!
//! # アイコンは独自の抽象図形のみ
//!
//! 参照元の SVG path 文字列は一切コピーしない。[`glyph_circle`]/
//! [`glyph_square`]/[`glyph_lines`] は `feature_expand::geo_icon`/
//! `bento_staggered::geo_icon` と同型の判断で、単純な幾何図形（円・四角・
//! 水平線 3 本）を自前の `d`/`children` で描く。
//!
//! # 自社・他社ラベルは `badge` で表す
//!
//! 列見出し（自社/他社）は既存の [`fandhe_frontend_pre_styled_ui::badge`]
//! で表現し、`props.rs` へ新型を追加しない。自社側は
//! [`BadgeVariant::Solid`] + [`ColorPalette::Accent`]（強調）、他社側は
//! [`BadgeVariant::Outline`] + [`ColorPalette::Neutral`]（中立）で視覚的に
//! 区別する。実在の企業名・製品名・競合名は使わず、中立的な「自社」
//! 「他社」ラベルに固定する。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`changelog_
//! timeline` 等、`crate::blocks` 内の他 block と同じ回避方法）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `badge::badge` / `icon::icon` /
//! `separator::separator` はいずれも `drop_class_attr` により呼び出し側
//! `attrs` の `class` を黙って除去する契約を持つため、Demo 固有のスタイル
//! フックは `data-blocks-comparison-feature-rows-*` 属性で渡す。素の
//! `div` には `.blocks-comparison-feature-rows-*` クラスセレクタを使う。
//!
//! # `id` 属性を出力しない理由
//!
//! `id`/`aria-labelledby` を持つ部品を使わないため、宙に浮いた ARIA 参照や
//! id 重複を防ぐための出力自体を行わない（`changelog_timeline` と同じ
//! 判断、`crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` の対象）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言はすべて架空のもの（実在の製品・企業名・PII を含まない）で、
//! リンク・ボタン・`href="#"`・`data:` URI は一切持たない。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::icon::{icon, IconProps};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::ColorPalette;

/// 円のみの抽象アイコン（`stroke` のみで塗り面を持たない、`feature_expand::
/// geo_icon` と同型の対処）。
fn glyph_circle() -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "circle",
            vec![
                ("cx", "12"),
                ("cy", "12"),
                ("r", "8"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
            ],
            vec![],
        )],
    )
}

/// 四角の輪郭のみの抽象アイコン。
fn glyph_square() -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M5 5h14v14H5z"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linejoin", "round"),
            ],
            vec![],
        )],
    )
}

/// 水平線 3 本の抽象アイコン（一覧・段階性を示す図形）。
fn glyph_lines() -> Node {
    icon(
        &IconProps::default(),
        vec![],
        vec![el(
            "path",
            vec![
                ("d", "M4 6h16M4 12h16M4 18h16"),
                ("fill", "none"),
                ("stroke", "currentColor"),
                ("stroke-width", "2"),
                ("stroke-linecap", "round"),
            ],
            vec![],
        )],
    )
}

/// 機能 1 行分のダミーデータ（架空、実在の製品・企業とは無関係）。
struct FeatureRow {
    glyph: fn() -> Node,
    name: &'static str,
    ours: &'static str,
    theirs: &'static str,
}

/// 比較行一覧（架空、3 行固定。検索インデックスの肥大を抑えるため件数を
/// 増やさない）。
const ROWS: [FeatureRow; 3] = [
    FeatureRow {
        glyph: glyph_circle,
        name: "同時編集",
        ours: "何人でも同時に編集でき、変更はリアルタイムに反映されます。",
        theirs: "同時編集は上位プランのみで、人数に上限があります。",
    },
    FeatureRow {
        glyph: glyph_square,
        name: "オフライン対応",
        ours: "オフラインでも編集を続けられ、復帰時に自動で同期します。",
        theirs: "オフライン編集には対応していません。",
    },
    FeatureRow {
        glyph: glyph_lines,
        name: "変更履歴",
        ours: "すべての変更履歴を無期限に保持し、いつでも復元できます。",
        theirs: "変更履歴の保持期間は 30 日間に限られます。",
    },
];

/// 見出しブロック（タグライン `badge` + セクション見出し + リード文）。
fn intro() -> Node {
    div(
        vec![("class", "blocks-comparison-feature-rows-intro")],
        vec![
            badge::badge(&BadgeProps::default(), vec![], vec![text("比較")]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("機能で比較する")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("主要な機能を自社・他社で並べて確認できます。")],
            ),
        ],
    )
}

/// 機能名セル（アイコン + `heading H4`）。
fn feature_cell(row: &FeatureRow) -> Node {
    div(
        vec![("data-blocks-comparison-feature-rows-feature", "")],
        vec![
            (row.glyph)(),
            heading(
                HeadingLevel::H4,
                &HeadingProps {
                    size: HeadingSize::Md,
                    weight: HeadingWeight::Semibold,
                },
                vec![],
                vec![text(row.name)],
            ),
        ],
    )
}

/// 自社/他社の説明セル（列ラベル `badge` + 説明 `text`）。
fn party_cell(
    hook: &'static str,
    label: &'static str,
    variant: BadgeVariant,
    description: &str,
) -> Node {
    div(
        vec![(hook, "")],
        vec![
            badge::badge(
                &BadgeProps {
                    variant,
                    palette: if hook.ends_with("ours") {
                        ColorPalette::Accent
                    } else {
                        ColorPalette::Neutral
                    },
                    ..BadgeProps::default()
                },
                vec![],
                vec![text(label)],
            ),
            styled_text::text(&TextProps::default(), vec![], vec![text(description)]),
        ],
    )
}

/// 機能 1 行分（機能名・自社・他社の 3 要素）。
fn feature_row(row: &FeatureRow) -> Node {
    div(
        vec![("data-blocks-comparison-feature-rows-row", "")],
        vec![
            feature_cell(row),
            party_cell(
                "data-blocks-comparison-feature-rows-ours",
                "自社",
                BadgeVariant::Solid,
                row.ours,
            ),
            party_cell(
                "data-blocks-comparison-feature-rows-theirs",
                "他社",
                BadgeVariant::Outline,
                row.theirs,
            ),
        ],
    )
}

/// `comparison-feature-rows` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（[`crate::blocks`] モジュール doc「静的表示」節）。
pub fn demo() -> Node {
    let last_index = ROWS.len().saturating_sub(1);
    let mut list_children: Vec<Node> = Vec::new();
    for (index, row) in ROWS.iter().enumerate() {
        list_children.push(feature_row(row));
        if index != last_index {
            list_children.push(separator::separator(
                &SeparatorProps::default(),
                vec![("data-blocks-comparison-feature-rows-separator", "")],
            ));
        }
    }

    div(
        vec![("data-blocks-comparison-feature-rows-root", "")],
        vec![
            intro(),
            div(
                vec![("data-blocks-comparison-feature-rows-list", "")],
                list_children,
            ),
        ],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/comparison-feature-rows/",
    title: "comparison-feature-rows",
    category: BlockCategory::Comparison,
    rust_source: "crates/docs-site/src/blocks/marketing/comparison/comparison_feature_rows.rs",
    demo_class: "blocks-comparison-feature-rows",
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
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Icon",
            path: "/themes/icon/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `comparison_feature_rows` 固有のレイアウト規則（`crate::blocks::
/// LAYOUT_CSS` doc「block 固有 CSS の置き場」節）。モバイルファーストで
/// 既定（狭い幅）は縦積み、`>= 48rem`（md）で 2 列（機能名は全幅・自社と
/// 他社が横並び）、`>= 64rem`（lg）で 3 列（機能名・自社・他社を横一列）
/// へ切り替える。ブレークポイントは `fandhe_frontend_pre_styled_ui` の
/// `Breakpoint`（md=768px/lg=1024px）に合わせたリテラル rem 値
/// （`@media` 内ではトークン変数を使えないため）。
const LAYOUT_CSS: &str = "\
[data-blocks-comparison-feature-rows-root] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  width: 100%;\n}\n\
.blocks-comparison-feature-rows-intro {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  max-width: 40rem;\n  align-items: flex-start;\n}\n\
[data-blocks-comparison-feature-rows-list] {\n  display: flex;\n  flex-direction: column;\n}\n\
[data-blocks-comparison-feature-rows-row] {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-4);\n  padding-block: var(--fandhe-space-5);\n}\n\
[data-blocks-comparison-feature-rows-feature] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-comparison-feature-rows-ours],\n[data-blocks-comparison-feature-rows-theirs] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  align-items: flex-start;\n}\n\
@media (min-width: 48rem) {\n  [data-blocks-comparison-feature-rows-row] {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n  [data-blocks-comparison-feature-rows-feature] {\n    grid-column: 1 / -1;\n  }\n}\n\
@media (min-width: 64rem) {\n  [data-blocks-comparison-feature-rows-row] {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 1.5fr) minmax(0, 1.5fr);\n  }\n  [data-blocks-comparison-feature-rows-feature] {\n    grid-column: auto;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS, ROWS};
    use fandhe_frontend_core::render;

    /// Demo は呼び出しごとに同一の `Node` を返す純関数であること
    /// （`crate::blocks` モジュール doc「静的表示」節）。
    #[test]
    fn demo_is_deterministic() {
        assert_eq!(render(&demo()), render(&demo()));
    }

    /// Demo が期待する 5 種の部品を出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"badge\"",
            "data-scope=\"icon\"",
            "data-scope=\"separator\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// 行フックが [`ROWS`] の件数だけ、separator が「件数 - 1」だけ出力
    /// されること（最後の行の後には置かない契約）。
    #[test]
    fn demo_wires_row_and_separator_counts() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-comparison-feature-rows-row=\"\"")
                .count(),
            ROWS.len(),
            "demo should render exactly {} rows",
            ROWS.len()
        );
        assert_eq!(
            html.matches("data-blocks-comparison-feature-rows-separator=\"\"")
                .count(),
            ROWS.len() - 1,
            "demo should render exactly {} separators (one fewer than rows)",
            ROWS.len() - 1
        );
    }

    /// 非対話・安全性の不変条件（`<form>`・`<button>`・`<a`・`href="#"`・
    /// `data:` URI・`id=` 属性を持たないこと。モジュール doc「id 属性を
    /// 出力しない理由」節）。
    #[test]
    fn demo_never_contains_forbidden_markup() {
        let html = render(&demo());
        for absent in [
            "<form",
            "<button",
            "<a ",
            "href=\"#\"",
            "src=\"data:",
            "id=\"",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が md/lg 2 段のブレークポイント切り替えを持つこと。
    #[test]
    fn layout_css_has_responsive_breakpoints() {
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
    }
}
