//! `stats-split` block（イシュー #2804。親トラッキング #2730「Blocks
//! 目的別パーツ拡充ツリー」配下、対応表 ID R0337（主参照。左見出し + 右に
//! 2 列で 6 指標）を構造の参照元とする合成例。集約元は R0339（横並びの
//! 導入行 + 左罫線 4 指標）・R0708（2×2 の 4 指標）・R1306（左に本文 2
//! 段落 + 右に 3 指標）。取得手段・ファイル名・内部コンポーネント識別子は
//! 記載しない（`contact_split_info` モジュール doc と同じライセンス上の
//! 転記制限）。
//!
//! # 使用部品
//!
//! `badge` / `heading` / `text` / `stat` / `separator` の 5 部品のみを
//! 合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//! 新しい UI 部品は追加しない。
//!
//! # 2 インスタンス構成（集約元との差分）
//!
//! - **基準形**（`data-blocks-stats-split-variant="base"`、R0337 主参照）:
//!   左列に `badge`（タグライン）+ `heading`（H3）+ `text`（説明）を縦積み
//!   し、右列に `stat` を 2 列グリッドで 6 件並べる。各指標の下へ罫線を
//!   引く（R0708 の「指標へ罫線を添える」考え方も併せて表現する）。
//! - **導入行 + 左罫線**（`data-blocks-stats-split-variant="intro-row"`、
//!   R0339）: `heading`（H3）+ `text` を横並びの導入行にし、`separator`
//!   （Horizontal・Solid）を挟んでから `stat` を 4 件並べる。各指標へ左
//!   罫線を付ける。
//!
//! R0708（2×2 の 4 指標）・R1306（左に本文 2 段落 + 右に 3 指標）は Demo に
//! インスタンス化しない（Demo を 2 件に留める判断）。前者は基準形の
//! グリッドが `sm` 未満で自然に 2×2 相当（1 列 6 段）へ折り返るため罫線
//! 表現を共有でき、後者は左列を本文 2 段落・右列の `stat` 件数を 3 件に
//! 差し替えるだけで基準形のヘルパをそのまま再利用できる（`site/blocks/
//! stats-split.md` の「原案差分メモ」節で説明する）。
//!
//! # 見出しレベル
//!
//! ページ側が `## Demo` として `h2` を出すため、各インスタンスの見出しは
//! `HeadingLevel::H3` にする。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側を `styled_text` として取り込む（`crate::blocks`
//! 内の他 block と同じ回避方法）。
//!
//! # ブレークポイント（40rem=sm/48rem=md をリテラル直書きする理由）
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint` の `Sm`（640px =
//! 40rem）・`Md`（768px = 48rem）と一致するリテラル値を [`LAYOUT_CSS`] へ
//! 直書きする（`contact_split_info` と同じ判断）。`md` 未満は基準形の左右
//! 2 列・導入行をいずれも 1 列（縦積み）にし、`md` 以上で横並びへ切り替
//! える。導入行の指標グリッドのみ `sm` 以上で 2 列の中間段を挟む。
//!
//! # 数値・文言は架空
//!
//! `crate::blocks` モジュール doc「セキュリティ不変条件」節に従い、指標の
//! 値・単位・見出し・説明文はすべて架空のものであり、実企業名・実指標・
//! PII を含まない。`<form>` は出さず、リンク・ボタンも持たない静的な合成
//! 例である。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{badge, BadgeProps};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::recipe::Size;
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Orientation;

/// 数値指標 1 件分（`stat::root` + `label`/`value_text`/`value_unit`）。
/// `bordered_attr` は罫線バリアント区別用の `data-*` 属性 1 件
/// （下罫線・左罫線のいずれかを [`LAYOUT_CSS`] のセレクタで切り替える）。
fn stat_item(label: &str, value: &str, unit: &str, bordered_attr: (&str, &str)) -> Node {
    stat::root(
        Size::Md,
        vec![bordered_attr],
        vec![
            stat::label(vec![], vec![text(label)]),
            stat::value_text(
                vec![],
                vec![text(value), stat::value_unit(vec![], vec![text(unit)])],
            ),
        ],
    )
}

/// 基準形（R0337 主参照）: 左に `badge`/`heading`/`text`、右に 2 列 6 指標
/// （下罫線）。
fn base_variant() -> Node {
    let left = div(
        vec![("class", "blocks-stats-split-left")],
        vec![
            badge(&BadgeProps::default(), vec![], vec![text("実績")]),
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("数字で見る導入実績")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(
                    "架空のダミー指標です。導入企業数・稼働率・処理件数などの傾向を\
                     まとめて示す想定の Demo です。",
                )],
            ),
        ],
    );

    let items = [
        ("導入企業数", "1,240", "社"),
        ("稼働率", "99.9", "%"),
        ("月間処理件数", "3.6", "M件"),
        ("平均応答時間", "48", "ms"),
        ("継続利用率", "96", "%"),
        ("サポート満足度", "4.8", "/5"),
    ];
    let grid = div(
        vec![("class", "blocks-stats-split-grid")],
        items
            .iter()
            .map(|(label, value, unit)| {
                stat_item(
                    label,
                    value,
                    unit,
                    ("data-blocks-stats-split-stat-bottom", ""),
                )
            })
            .collect(),
    );

    div(
        vec![
            ("data-blocks-stats-split-row", ""),
            ("data-blocks-stats-split-variant", "base"),
        ],
        vec![left, grid],
    )
}

/// 導入行 + 左罫線（R0339）: `heading`/`text` の横並び導入行 + `separator`
/// + 4 指標（左罫線）。
fn intro_row_variant() -> Node {
    let intro = div(
        vec![("class", "blocks-stats-split-intro")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("プラットフォーム全体の状況")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("直近期間の架空サマリーです。")],
            ),
        ],
    );

    let items = [
        ("アクティブ組織", "820", "社"),
        ("新規登録", "154", "件/月"),
        ("平均処理速度", "1.2", "秒"),
        ("障害件数", "0", "件"),
    ];
    let grid = div(
        vec![("class", "blocks-stats-split-grid-intro")],
        items
            .iter()
            .map(|(label, value, unit)| {
                stat_item(
                    label,
                    value,
                    unit,
                    ("data-blocks-stats-split-stat-left", ""),
                )
            })
            .collect(),
    );

    div(
        vec![("data-blocks-stats-split-variant", "intro-row")],
        vec![
            intro,
            separator::separator(
                &SeparatorProps {
                    orientation: Orientation::Horizontal,
                    ..SeparatorProps::default()
                },
                vec![],
            ),
            grid,
        ],
    )
}

/// `stats-split` の Demo 本体（基準形 + 導入行の 2 インスタンスを縦に
/// 並べる）。呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-stats-split-layout")],
        vec![base_variant(), intro_row_variant()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/stats-split/",
    title: "stats-split",
    category: BlockCategory::Stats,
    rust_source: "crates/docs-site/src/blocks/marketing/stats/stats_split.rs",
    demo_class: "blocks-stats-split",
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
            label: "Stat",
            path: "/themes/stat/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `stats_split` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節と同型で `pub(super)` ではなく本ファイル内
/// private 定数として `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-stats-split-*` と `[data-blocks-stats-split-*]` の
/// みを用い、他 block や部品の素のセレクタへ影響させない
/// （`contact_split_info` と同じ名前空間分離）。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-stats-split` だが、`demo()` が返す
/// ルート `div` の class は `blocks-stats-split-layout` という別名にする
/// （`contact_split_info` と同じ Bugbot 教訓の回避）。
const LAYOUT_CSS: &str = "\
.blocks-stats-split-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-12);\n}\n\
[data-blocks-stats-split-row] {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-stats-split-left {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  max-width: 28rem;\n}\n\
.blocks-stats-split-grid {\n  display: grid;\n  grid-template-columns: repeat(2, minmax(0, 1fr));\n  gap: var(--fandhe-space-6);\n}\n\
[data-blocks-stats-split-stat-bottom] {\n  border-bottom: 1px solid var(--fandhe-color-border);\n  padding-block-end: var(--fandhe-space-4);\n}\n\
.blocks-stats-split-intro {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-stats-split-grid-intro {\n  display: grid;\n  grid-template-columns: 1fr;\n  gap: var(--fandhe-space-6);\n  margin-top: var(--fandhe-space-6);\n}\n\
[data-blocks-stats-split-stat-left] {\n  border-inline-start: 2px solid var(--fandhe-color-border);\n  padding-inline-start: var(--fandhe-space-4);\n}\n\
@media (min-width: 40rem) {\n  .blocks-stats-split-grid-intro {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n\
@media (min-width: 48rem) {\n  [data-blocks-stats-split-row] {\n    grid-template-columns: minmax(0, 1fr) minmax(0, 1.5fr);\n    align-items: start;\n  }\n  .blocks-stats-split-intro {\n    flex-direction: row;\n    justify-content: space-between;\n    align-items: flex-end;\n    gap: var(--fandhe-space-6);\n  }\n  .blocks-stats-split-grid-intro {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（badge/heading/text/stat/separator）の anatomy を
    /// すべて実際に出力していることと、variant 数・指標件数を固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"badge\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"stat\"",
            "data-scope=\"separator\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("data-blocks-stats-split-variant").count(),
            2,
            "demo should render exactly 2 variants (base / intro-row)"
        );
        assert_eq!(
            html.matches("data-scope=\"stat\" data-part=\"root\"")
                .count(),
            10,
            "demo should render exactly 10 stat items (6 base + 4 intro-row)"
        );
    }

    /// 非対話・XSS 回帰の不変条件（`crate::blocks` モジュール doc）を固定
    /// する。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in [
            "<form",
            "type=\"submit\"",
            "href=\"#\"",
            "src=\"data:",
            "id=\"",
        ] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイントを持ち、`<` を含まないこと
    /// （REQ-1: `</style>` によるスタイル脱出を防ぐ）。
    #[test]
    fn layout_css_declares_breakpoints() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
        assert!(LAYOUT_CSS.contains("@media (min-width: 48rem)"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に現れ
    /// ること（モジュール doc「ルート class を `demo_class` と別名にする
    /// 理由」節の固定、`contact_split_info` と同じ回帰）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-stats-split-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-stats-split-layout");
    }
}
