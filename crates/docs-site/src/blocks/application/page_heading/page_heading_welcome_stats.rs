//! `page-heading-welcome-stats` block（イシュー #2935。親トラッキング
//! #2892「Blocks 目的別パーツ拡充」配下）。統計付きウェルカム見出し
//! カード。上段にアバター・挨拶文・名前・役職とプロフィール導線ボタン、
//! 下段に区切り線で区切った統計値 3 件を並べる。
//!
//! 対応表 ID R1131（`_/blocks-intake/` は本 worktree に存在しないため
//! 参照ファイルとの突合は行っていない。原稿・本 doc にはこの対応表 ID
//! のみを記録する。他の block と同じ扱い）。集約元は主参照と同一の 1 件
//! のみで、差分はない。
//!
//! # 使用部品
//!
//! `card` / `avatar` / `heading` / `text` / `button` / `stat` /
//! `separator` の 7 部品のみを合成する（[`BLOCK`] の `parts` に一致させる
//! 契約）。新しい UI 部品は追加しない。
//!
//! # 統計帯の区切りは `separator` ではなく CSS の罫線で表現する
//!
//! 統計 3 件の境界線は `stat` パーツ間の CSS 罫線（狭幅は
//! `border-block-start`、`40rem` 以上は `border-inline-start`）で表す。
//! `separator`（`Orientation::Vertical`）を統計間へ静的に挿入する案は
//! 採らなかった: 静的な `Node` は幅に応じて `aria-orientation` を
//! 切り替えられず、狭幅で縦並びになったときに水平の区切りを
//! `aria-orientation="vertical"` のまま出力する意味論の食い違いが生じる
//! ため。`separator` 部品自体は上段/下段を区切る水平の 1 本として使い、
//! `parts` 契約（demo 出力に `data-scope="separator"` が現れること）を
//! 満たす。
//!
//! # 狭幅では上段を縦積み・統計帯を 1 列にする
//!
//! `header` は既定で縦積み（`flex-direction: column`）、統計帯は 1 列
//! グリッド。`40rem` 以上で `header` を横並び（`row` +
//! `justify-content: space-between`）へ、統計帯を 3 列グリッドへ切り替え、
//! 罫線も `border-block-start` から `border-inline-start` へ切り替える
//! （他の block と同じリテラル値。テーマの breakpoint トークンは `@media`
//! 条件式の中では解決できない）。
//!
//! # `avatar::image` の `alt` は空文字
//!
//! 直後に名前テキストが続くため装飾扱いとする
//! （`team_avatar_grid.rs` と同じ判断）。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。ボタンは `button::button` の既定 `type="button"` のまま送信先を
//! 持たない（`docs/policy/intentional-non-adoption.md` §3.25：バリデーション・
//! 送信処理は UI コンポーネント層の責務外）。文言・数値はすべて独自の
//! 架空のダミー（実企業名・実クレデンシャル・PII を含まない）。
//!
//! # 「プロフィールを見る」ボタンは `disabled: true`
//!
//! 他の page-heading 系 block（`page_heading_actions`/`page_heading_cover`/
//! `page_heading_meta`）の操作ボタンは「公開する」「設定を開く」等、その場の
//! 状態を変える操作動詞のラベルであり、押しても何も起きなくても導線として
//! 誤解を招かない。一方このボタンは「見る」（=遷移先へ移動する）という
//! リンク相当の意味論を持つラベルでありながら遷移先を持たないため、押下が
//! 無反応であることが利用者に伝わらない（イシュー #2935 PR #3387 の指摘）。
//! `ButtonProps::disabled` で `disabled` 属性・`data-disabled`・
//! `aria-disabled="true"` を付与し、静的な表示例であることを見た目・
//! 支援技術の双方に明示する。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use crate::blocks::dummy_assets;
use fandhe_frontend_core::{div, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::heading::{heading, HeadingLevel, HeadingProps, HeadingSize};
use fandhe_frontend_pre_styled_ui::separator::{self, SeparatorProps};
use fandhe_frontend_pre_styled_ui::stat;
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 1 件分の統計（ラベル・値）を `stat` パーツへ組み立てる。
fn stat_item(label: &'static str, value: &'static str) -> Node {
    stat::root(
        Size::Md,
        vec![("data-blocks-page-heading-welcome-stats-stat", "")],
        vec![
            stat::label(vec![], vec![text(label)]),
            stat::value_text(vec![], vec![text(value)]),
        ],
    )
}

/// `page-heading-welcome-stats` の Demo 本体（呼び出しごとに同一の
/// `Node` を返す純関数）。
pub fn demo() -> Node {
    let name = dummy_assets::PERSON_NAMES[0];
    let title = dummy_assets::JOB_TITLES[0];

    let identity = div(
        vec![("data-blocks-page-heading-welcome-stats-identity", "")],
        vec![
            avatar::root(
                &AvatarProps {
                    size: Size::Lg,
                    ..AvatarProps::default()
                },
                vec![],
                vec![avatar::image(
                    ImageStatus::Loaded,
                    dummy_assets::AVATAR_SRC,
                    "",
                    vec![],
                )],
            ),
            div(
                vec![("data-blocks-page-heading-welcome-stats-greeting", "")],
                vec![
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text("おはようございます")],
                    ),
                    heading(
                        HeadingLevel::H1,
                        &HeadingProps {
                            size: HeadingSize::Lg,
                            ..HeadingProps::default()
                        },
                        vec![],
                        vec![text(name)],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(title)],
                    ),
                ],
            ),
        ],
    );

    let header = card::header(
        vec![("data-blocks-page-heading-welcome-stats-header", "")],
        vec![
            identity,
            button::button(
                &ButtonProps {
                    variant: ButtonVariant::Outline,
                    disabled: true,
                    ..ButtonProps::default()
                },
                vec![],
                vec![text("プロフィールを見る")],
            ),
        ],
    );

    let stats = card::body(
        vec![("data-blocks-page-heading-welcome-stats-stats", "")],
        vec![
            stat_item("今週の完了タスク", "12"),
            stat_item("未読の通知", "3"),
            stat_item("進行中のプロジェクト", "5"),
        ],
    );

    let card_node = card::root(
        CardProps::default(),
        vec![],
        vec![
            header,
            separator::separator(&SeparatorProps::default(), vec![]),
            stats,
        ],
    );

    div(
        vec![("class", "blocks-page-heading-welcome-stats-layout")],
        vec![card_node],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/page-heading-welcome-stats/",
    title: "page-heading-welcome-stats",
    category: BlockCategory::PageHeading,
    rust_source:
        "crates/docs-site/src/blocks/application/page_heading/page_heading_welcome_stats.rs",
    demo_class: "blocks-page-heading-welcome-stats",
    parts: &[
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
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
            label: "Button",
            path: "/themes/button/",
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

/// `page_heading_welcome_stats` 固有のレイアウト規則
/// （`crate::blocks` モジュール doc「CSS の置き場」節と同型で、
/// 本ファイル内 private 定数として `super::stylesheet` 経由の `push_css` で
/// 連結される）。
///
/// セレクタは `.blocks-page-heading-welcome-stats-*` と
/// `[data-blocks-page-heading-welcome-stats-*]` を基本とする。ただし
/// header/stats（`card::header`/`card::body`）は Card レシピが
/// `[data-scope="card"][data-part="header"|"body"]`（2 属性）で狙っており
/// 単一 `[data-blocks-*]` 属性（1 属性）では CSS 詳細度で負けるため、
/// これらの選択子には `[data-scope="card"][data-part="..."]` を前置して
/// 詳細度を揃える（`[data-scope="stat"][data-part="root"][data-blocks-*]`
/// と同じ手当て）。
///
/// # ルート class を `demo_class` と別名にする理由
///
/// [`Block::demo_class`] は `blocks-page-heading-welcome-stats` だが、
/// `demo()` が返すルート `div` の class は
/// `blocks-page-heading-welcome-stats-layout` という別名にする
/// （`page_heading_actions` 等と同じ Bugbot 教訓の回避）。
const LAYOUT_CSS: &str = "\
.blocks-page-heading-welcome-stats-layout {\n  display: flex;\n  flex-direction: column;\n}\n\
[data-scope=\"card\"][data-part=\"header\"][data-blocks-page-heading-welcome-stats-header] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  padding: var(--fandhe-space-6);\n}\n\
[data-blocks-page-heading-welcome-stats-identity] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-page-heading-welcome-stats-greeting] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\
[data-scope=\"card\"][data-part=\"body\"][data-blocks-page-heading-welcome-stats-stats] {\n  display: grid;\n  grid-template-columns: 1fr;\n  padding: 0;\n}\n\
[data-scope=\"stat\"][data-part=\"root\"][data-blocks-page-heading-welcome-stats-stat] {\n  padding: var(--fandhe-space-4) var(--fandhe-space-6);\n}\n\
[data-blocks-page-heading-welcome-stats-stat] + [data-blocks-page-heading-welcome-stats-stat] {\n  border-block-start: 1px solid var(--fandhe-color-border);\n}\n\
@media (min-width: 40rem) {\n  [data-scope=\"card\"][data-part=\"header\"][data-blocks-page-heading-welcome-stats-header] {\n    flex-direction: row;\n    justify-content: space-between;\n    align-items: center;\n  }\n  [data-scope=\"card\"][data-part=\"body\"][data-blocks-page-heading-welcome-stats-stats] {\n    grid-template-columns: repeat(3, 1fr);\n  }\n  [data-blocks-page-heading-welcome-stats-stat] + [data-blocks-page-heading-welcome-stats-stat] {\n    border-block-start: 0;\n    border-inline-start: 1px solid var(--fandhe-color-border);\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（card/avatar/heading/text/button/stat/separator）の
    /// anatomy をすべて実際に出力していること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"card\"",
            "data-scope=\"avatar\"",
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"button\"",
            "data-scope=\"stat\"",
            "data-scope=\"separator\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// 統計はちょうど 3 件。
    #[test]
    fn demo_stat_count() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-scope=\"stat\" data-part=\"root\"")
                .count(),
            3
        );
    }

    /// `<form>`・`href="#"`・`data:` URI・`type="submit"` を出力しない
    /// （`crate::blocks` モジュール doc）。
    #[test]
    fn demo_has_no_form_or_unsafe_output() {
        let html = render(&demo());
        for absent in ["<form", "type=\"submit\"", "href=\"#\"", "src=\"data:"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// [`LAYOUT_CSS`] が想定するブレークポイントを持ち、`<` を含まない
    /// （REQ-1: `</style>` によるスタイル脱出を防ぐ）。
    #[test]
    fn layout_css_declares_breakpoint_and_no_angle_bracket() {
        assert!(!LAYOUT_CSS.contains('<'));
        assert!(LAYOUT_CSS.contains("@media (min-width: 40rem)"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-page-heading-welcome-stats-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-page-heading-welcome-stats-layout"
        );
    }
}
