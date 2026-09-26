//! `stats-timeline` block（イシュー #2805。親トラッキング #2730/#2738
//! 「Blocks 目的別パーツ拡充（マーケティング A）」配下、対応表 ID R1303 の
//! 1 件を構造の参照元とする合成例。日付付きの出来事（沿革）を横一列に
//! 並べる）。取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `separator` の 3 部品のみを合成する（[`BLOCK`] の
//! `parts` に一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。新しい UI 部品は追加しない。
//!
//! # `timeline` 部品を使わない理由
//!
//! `crates/pre-styled-ui::timeline` は縦向きの沿革表示にしか対応していない
//! （水平方向の配置を持たない）。本 block は「lg 以上で横 1 列」という
//! 要求があるため `timeline` を使わず、配置は本ファイル固有の
//! [`LAYOUT_CSS`] と `separator` の組み合わせで独自に構成する。
//!
//! # レイアウト
//!
//! 見出し + リード文の下へ、出来事 4 件を並べる。各件は
//! 「`<time>` 日付 → ドット + 罫線 → 題（`h4`）→ 説明」の縦積みで、
//! `ol`（時系列であるためリスト意味論に `ol` を使う）の 1 項目として並ぶ。
//! ページ側が `## Demo` として `h2` を出すため、大見出しは `h3`
//! （[`heading::HeadingLevel::H3`]）にする。
//!
//! - `lg`（64rem）以上: 4 列（横 1 列）
//! - `sm`（40rem）以上 `lg` 未満: 2 列
//! - `sm` 未満: 1 列（縦積み）
//!
//! # ドット・罫線の実現手段
//!
//! ドットは装飾のみのため `aria-hidden="true"` を付けた `<span>` を
//! `border-radius: var(--fandhe-radius-full)` の円として CSS で描く。罫線は
//! [`separator::separator`]（既定の `Horizontal`/`Solid`）を使い、
//! `flex: 1 1 auto` で残り幅いっぱいへ伸ばす。
//!
//! # ブレークポイント（40rem/64rem をリテラル直書きする理由）
//!
//! テーマの breakpoint トークンは `@media` 条件式の中では解決できない
//! （CSS custom property は宣言側でのみ有効）ため、
//! `fandhe_frontend_pre_styled_ui::recipe::Breakpoint` の `Sm`（640px =
//! 40rem）・`Lg`（1024px = 64rem）と一致するリテラル値を [`LAYOUT_CSS`] へ
//! 直書きする（`section_heading_stats`/`contact_split_info` と同じ判断）。
//!
//! # `drop_class_attr` と詳細度
//!
//! `heading` / `text` / `separator` はいずれも `drop_class_attr` で `class`
//! を黙って除去するため、これらの部品には `data-blocks-stats-timeline-*`
//! の data 属性フックを使う。素の `ol`/`li`/`div`/`time`/`span` には
//! `.blocks-stats-timeline-*` の class を使う。`separator` は
//! `[data-scope="separator"][data-part="root"]`（詳細度 0,2,0）の base 宣言
//! （`border-width: 0`）を持つため、これに `flex` を追加する宣言は
//! `[data-scope="separator"][data-part="root"][data-blocks-stats-timeline-rule]`
//! のように前置して詳細度を揃える（`section_heading_stats` と同じ既知の
//! 落とし穴への対策）。
//!
//! # データ・文言
//!
//! 出来事 4 件は架空のプロジェクト沿革（実企業名・PII・実データは含まない、
//! 原稿の導入文にも明記する）。コードフェンス単体でのコンパイルを保つため
//! `dummy_assets` は参照せず、文言はすべてリテラルで直接持つ。日付・題は
//! 4 件とも相異なる。
//!
//! # `<form>` を持たない・静的表示
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である（本 block はそもそも開閉状態を持たない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, text, Node};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};

/// 出来事 1 件分のダミーデータ（架空、実企業名・PII とは無関係）。
struct EventItem {
    /// `<time datetime>` へそのまま使う ISO 8601 表記（年月）。
    date_attr: &'static str,
    /// 表示用の日付文言。
    date_label: &'static str,
    title: &'static str,
    body: &'static str,
}

/// 出来事一覧（架空、4 件、日付・題とも相異なる）。
const EVENTS: [EventItem; 4] = [
    EventItem {
        date_attr: "2023-04",
        date_label: "2023 年 4 月",
        title: "プロジェクト発足",
        body: "説明用の架空のキックオフです。最小構成の設計をまとめました。",
    },
    EventItem {
        date_attr: "2023-11",
        date_label: "2023 年 11 月",
        title: "最初の公開版",
        body: "説明用の架空の初回リリースです。中核機能を公開しました。",
    },
    EventItem {
        date_attr: "2024-07",
        date_label: "2024 年 7 月",
        title: "利用チームの拡大",
        body: "説明用の架空の拡大期です。導入チーム数が大きく伸びました。",
    },
    EventItem {
        date_attr: "2025-05",
        date_label: "2025 年 5 月",
        title: "大型刷新",
        body: "説明用の架空の刷新です。既定エスケープの製品化を完了しました。",
    },
];

/// 出来事 1 件（`<time>` + ドット/罫線 + 題 + 説明）。
fn event_item(item: &EventItem) -> Node {
    el(
        "li",
        vec![("class", "blocks-stats-timeline-item")],
        vec![
            el(
                "time",
                vec![
                    ("datetime", item.date_attr),
                    ("class", "blocks-stats-timeline-date"),
                ],
                vec![text(item.date_label)],
            ),
            div(
                vec![("class", "blocks-stats-timeline-marker")],
                vec![
                    el(
                        "span",
                        vec![
                            ("aria-hidden", "true"),
                            ("class", "blocks-stats-timeline-dot"),
                        ],
                        vec![],
                    ),
                    separator(
                        &SeparatorProps::default(),
                        vec![
                            ("aria-hidden", "true"),
                            ("data-blocks-stats-timeline-rule", ""),
                        ],
                    ),
                ],
            ),
            heading(
                HeadingLevel::H4,
                &HeadingProps {
                    size: HeadingSize::Lg,
                    weight: HeadingWeight::Semibold,
                },
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
                vec![text(item.body)],
            ),
        ],
    )
}

/// `stats-timeline` の Demo 本体（大見出し + リード文 + 出来事 4 件）。
/// 呼び出しごとに同一の `Node` を返す純関数。
pub fn demo() -> Node {
    let intro = div(
        vec![("class", "blocks-stats-timeline-intro")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl3,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("プロジェクトの歩み")],
            ),
            styled_text::text(
                &TextProps {
                    size: TextSize::Lg,
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("日付と出来事はすべて説明用の架空の内容です。")],
            ),
        ],
    );

    let list = el(
        "ol",
        vec![("class", "blocks-stats-timeline-list"), ("role", "list")],
        EVENTS.iter().map(event_item).collect(),
    );

    div(
        vec![("class", "blocks-stats-timeline-layout")],
        vec![intro, list],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/stats-timeline/",
    title: "stats-timeline",
    category: BlockCategory::Stats,
    rust_source: "crates/docs-site/src/blocks/marketing/stats/stats_timeline.rs",
    demo_class: "blocks-stats-timeline",
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
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `stats_timeline` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS` doc
/// 「block 固有 CSS の置き場」節と同型で `pub(super)` ではなく本ファイル内
/// private 定数として `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-stats-timeline-*` と `[data-blocks-stats-timeline-*]`
/// のみを用い、他 block や部品の素のセレクタへ影響させない。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-stats-timeline` だが、`demo()` が返す
/// ルート `div` の class は `blocks-stats-timeline-layout` という別名にする
/// （`section_heading_stats`/`contact_split_info` と同じ Bugbot 教訓の
/// 回避。ページ側が `demo_class` を `.blocks-demo` の隣に付与するラッパーと
/// block 自身のレイアウトルートを区別するため）。
const LAYOUT_CSS: &str = "\
.blocks-stats-timeline-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-stats-timeline-intro {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  max-width: 40rem;\n}\n\
.blocks-stats-timeline-list {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-8);\n}\n\
.blocks-stats-timeline-item {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-stats-timeline-date {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\
.blocks-stats-timeline-marker {\n  display: flex;\n  align-items: center;\n  gap: 0;\n}\n\
.blocks-stats-timeline-dot {\n  width: var(--fandhe-space-3);\n  height: var(--fandhe-space-3);\n  flex-shrink: 0;\n  border-radius: var(--fandhe-radius-full);\n  background: var(--fandhe-color-accent);\n}\n\
[data-scope=\"separator\"][data-part=\"root\"][data-blocks-stats-timeline-rule] {\n  flex: 1 1 auto;\n}\n\
@media (min-width: 40rem) {\n  .blocks-stats-timeline-list {\n    grid-template-columns: repeat(2, minmax(0, 1fr));\n  }\n}\n\
@media (min-width: 64rem) {\n  .blocks-stats-timeline-list {\n    grid-template-columns: repeat(4, minmax(0, 1fr));\n    column-gap: 0;\n  }\n  .blocks-stats-timeline-item {\n    padding-right: var(--fandhe-space-6);\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, EVENTS, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（heading/text/separator）の anatomy をすべて実際に
    /// 出力していることと、件数を固定する。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"separator\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert_eq!(
            html.matches("<time datetime=").count(),
            4,
            "demo should render exactly 4 <time> elements"
        );
        assert_eq!(
            html.matches("role=\"separator\"").count(),
            4,
            "demo should render exactly 4 separators"
        );
        assert_eq!(
            html.matches("blocks-stats-timeline-dot").count(),
            4,
            "demo should render exactly 4 decorative dots"
        );
        assert_eq!(
            html.matches("aria-hidden=\"true\"").count(),
            8,
            "demo should render exactly 4 aria-hidden dots + 4 aria-hidden separators"
        );
        assert_eq!(
            html.matches("data-scope=\"heading\"").count(),
            5,
            "demo should render 5 headings (1 intro H3 + 4 item H4)"
        );
        assert!(
            html.contains("role=\"list\""),
            "ol should keep list semantics via role=\"list\" (list-style: none removes it in some AT)"
        );

        let mut titles: Vec<&'static str> = EVENTS.iter().map(|item| item.title).collect();
        titles.sort_unstable();
        titles.dedup();
        assert_eq!(titles.len(), 4, "all 4 event titles should be distinct");

        let mut dates: Vec<&'static str> = EVENTS.iter().map(|item| item.date_attr).collect();
        dates.sort_unstable();
        dates.dedup();
        assert_eq!(dates.len(), 4, "all 4 event dates should be distinct");
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
            "<img",
            " id=\"",
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
        assert!(LAYOUT_CSS.contains("@media (min-width: 64rem)"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること（モジュール doc「ルート class を `demo_class` と別名に
    /// する理由」節の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-stats-timeline-layout\""));
        assert_ne!(super::BLOCK.demo_class, "blocks-stats-timeline-layout");
    }
}
