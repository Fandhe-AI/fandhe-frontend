//! `docs-layout-toc-progress` block。ページ内目次を先頭 10 項目の番号付き
//! 縦リストとして表示し、現在節までの進捗を左の縦トラックで示す実例。
//! Docs Layout カテゴリの block 登録点は `docs_layout/mod.rs` を正とする。
//! `_/blocks-intake/` の対応ファイルは本 worktree に存在しないため、
//! 原稿・本コメントには対応表 ID のみを記す（`docs_layout_page_header.rs`
//! 等と同じ扱い）。
//!
//! # 使用部品
//!
//! `link` / `text` の 2 部品のみを合成する（[`BLOCK`] の `parts` に一致
//! させる契約）。新しい UI 部品は追加しない。目次の構造自体（`nav`/`ol`/
//! `li`/`span`）は `fandhe_frontend_core` のノード木 API で直接組み立てる。
//!
//! # 対応表 ID
//!
//! 対応表 ID R0367（主参照、集約元は同 1 件のため Demo は 1 インスタンス）。
//!
//! # 構成（本文列 + 目次列）
//!
//! `linkcheck::check_links` は `#fragment` リンクを同一ページの id 集合と
//! 突合し、fail-closed で落とす（兄弟 `docs-layout-toc` と同じ制約）。この
//! ため Demo には目次の遷移先となる本文プレースホルダー列を置き、各節の
//! 先頭要素（`text` 部品の見出し）に `id` を付ける。本文見出しを生の
//! `h2`/`h3` にすると `layout::with_heading_anchors` が docs サイト右目次
//! へ誤って収集してしまうため、`data-scope="text"` を持つ `text` 部品で
//! 代替する（収集対象外になる）。
//!
//! # 先頭 10 項目への打ち切り
//!
//! 本文は [`SECTIONS`] 12 件を持つが、目次は [`TOC_LIMIT`]（10）で先頭
//! 10 件だけを表示する。11・12 項目目は目次に現れず、本文列にのみ残る
//! （打ち切りが Demo から読み取れるようにするため）。
//!
//! # 現在位置の表現（静的固定）
//!
//! [`CURRENT_INDEX`]（3、4 項目目）を現在位置として固定する。各 `li` に
//! `data-blocks-docs-layout-toc-progress-item-state` を `done`/`current`/
//! `upcoming` で付与し、現在項目のリンクにのみ `aria-current="location"`
//! を付ける（`LinkProps.current` は `aria-current="page"` 固定のため使わ
//! ず、`attrs` 経由で直接渡す。兄弟 `docs-layout-toc` と同じ規約）。実際の
//! スクロール追従（scroll spy）による動的更新は、無 JS の docs サイトでは
//! 行わない（配線層の責務、本 Demo のスコープ外）。
//!
//! # 可視の目次見出しを持たない
//!
//! 目次自体は「このページの内容」等の可視見出しを持たず、`nav` の
//! `aria-label` でアクセシブルネームのみを与える。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である（`docs/policy/intentional-non-adoption.md` §3.25）。文言はすべて
//! 独自の架空の日本語ダミー（実企業名・実クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, li, nav, ol, span, text, Node};
use fandhe_frontend_pre_styled_ui::link::{self, LinkProps};
use fandhe_frontend_pre_styled_ui::recipe::ColorPalette;
use fandhe_frontend_pre_styled_ui::text::{
    self as styled_text, TextProps, TextVariant, TextWeight,
};

/// 本文の 1 節（目次項目と 1 対 1 対応する）。
struct Section {
    id: &'static str,
    number: &'static str,
    title: &'static str,
}

/// 本文の全 12 節（架空の目次項目）。先頭 [`TOC_LIMIT`] 件のみを目次に出す。
const SECTIONS: &[Section] = &[
    Section {
        id: "blocks-docs-layout-toc-progress-overview",
        number: "01",
        title: "概要",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-install",
        number: "02",
        title: "インストール",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-setup",
        number: "03",
        title: "初期設定",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-routing",
        number: "04",
        title: "ルーティング",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-state",
        number: "05",
        title: "状態管理",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-styling",
        number: "06",
        title: "スタイル",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-testing",
        number: "07",
        title: "テスト",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-build",
        number: "08",
        title: "ビルド",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-deploy",
        number: "09",
        title: "デプロイ",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-operation",
        number: "10",
        title: "運用",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-troubleshooting",
        number: "11",
        title: "トラブルシューティング",
    },
    Section {
        id: "blocks-docs-layout-toc-progress-appendix",
        number: "12",
        title: "付録",
    },
];

/// 目次に表示する先頭項目数（11・12 項目目は目次から打ち切られる）。
const TOC_LIMIT: usize = 10;

/// 現在位置として固定するインデックス（0 始まり、4 項目目）。
const CURRENT_INDEX: usize = 3;

/// 目次項目 1 件の進捗状態。
#[derive(Clone, Copy, PartialEq, Eq)]
enum ProgressState {
    Done,
    Current,
    Upcoming,
}

impl ProgressState {
    fn as_attr(self) -> &'static str {
        match self {
            ProgressState::Done => "done",
            ProgressState::Current => "current",
            ProgressState::Upcoming => "upcoming",
        }
    }
}

/// 目次（`nav` > `ol` > `li`）を組み立てる。マーカー `01`〜`10` は
/// `aria-hidden="true"` にし、`ol` の暗黙の順序と重複読み上げしない。
fn toc() -> Node {
    let items = SECTIONS
        .iter()
        .take(TOC_LIMIT)
        .enumerate()
        .map(|(index, section)| {
            let state = if index < CURRENT_INDEX {
                ProgressState::Done
            } else if index == CURRENT_INDEX {
                ProgressState::Current
            } else {
                ProgressState::Upcoming
            };
            let href = format!("#{}", section.id);
            let mut link_attrs: Vec<(&str, &str)> = vec![];
            if state == ProgressState::Current {
                link_attrs.push(("aria-current", "location"));
            }
            li(
                vec![
                    ("data-blocks-docs-layout-toc-progress-item", ""),
                    (
                        "data-blocks-docs-layout-toc-progress-item-state",
                        state.as_attr(),
                    ),
                ],
                vec![
                    span(
                        vec![
                            ("aria-hidden", "true"),
                            ("data-blocks-docs-layout-toc-progress-marker", ""),
                        ],
                        vec![text(section.number)],
                    ),
                    link::root(
                        &href,
                        &LinkProps {
                            palette: ColorPalette::Neutral,
                            ..LinkProps::default()
                        },
                        link_attrs,
                        vec![text(section.title)],
                    ),
                ],
            )
        })
        .collect::<Vec<_>>();
    nav(
        vec![
            ("aria-label", "このページの目次"),
            ("data-blocks-docs-layout-toc-progress-nav", ""),
        ],
        vec![ol(
            vec![("data-blocks-docs-layout-toc-progress-list", "")],
            items,
        )],
    )
}

/// 本文プレースホルダー列（全 [`SECTIONS`]。各節見出しに目次遷移先の `id`
/// を付ける）。
fn body() -> Node {
    let sections = SECTIONS
        .iter()
        .map(|section| {
            div(
                vec![],
                vec![
                    styled_text::text(
                        &TextProps {
                            weight: TextWeight::Semibold,
                            ..TextProps::default()
                        },
                        vec![("id", section.id)],
                        vec![text(section.title)],
                    ),
                    styled_text::text(
                        &TextProps {
                            variant: TextVariant::Muted,
                            ..TextProps::default()
                        },
                        vec![],
                        vec![text(
                            "節の本文はダミーのプレースホルダーです。実際のページでは解説が入ります。",
                        )],
                    ),
                ],
            )
        })
        .collect::<Vec<_>>();
    div(
        vec![("class", "blocks-docs-layout-toc-progress-body")],
        sections,
    )
}

/// `docs-layout-toc-progress` の Demo 本体（本文列 + 目次列。呼び出しごとに
/// 同一の `Node` を返す純関数）。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-docs-layout-toc-progress-layout")],
        vec![body(), toc()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの
/// `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/docs-layout-toc-progress/",
    title: "docs-layout-toc-progress",
    category: BlockCategory::DocsLayout,
    rust_source: "crates/docs-site/src/blocks/docs/docs_layout/docs_layout_toc_progress.rs",
    demo_class: "blocks-docs-layout-toc-progress",
    parts: &[
        Part {
            label: "Link",
            path: "/themes/link/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `docs_layout_toc_progress` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節と同型で、本ファイル内 private 定数として
/// `super::stylesheet` 経由の `push_css` で連結される）。
///
/// セレクタは `.blocks-docs-layout-toc-progress-*` と
/// `[data-blocks-docs-layout-toc-progress-*]` のみを用いる。ルート class を
/// `demo_class` と別名にする（`docs_layout_page_header` 等と同じ Bugbot
/// 教訓の回避）。`docs-toc` トークンは出力しない（`site.js` の scroll spy が
/// 誤って掴むため）。`position: sticky` も使わない（`.blocks-demo` が
/// スクロールコンテナになるため。実アプリでの sticky 想定は原稿の差分メモ
/// に書く）。
const LAYOUT_CSS: &str = "\
.blocks-docs-layout-toc-progress-layout {\n  container-type: inline-size;\n  container-name: blocks-docs-layout-toc-progress;\n  display: grid;\n  grid-template-columns: minmax(0, 1fr);\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-docs-layout-toc-progress-body {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n  min-width: 0;\n}\n\
[data-blocks-docs-layout-toc-progress-list] {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n}\n\
[data-blocks-docs-layout-toc-progress-item] {\n  position: relative;\n  display: grid;\n  grid-template-columns: 1.75rem minmax(0, 1fr);\n  align-items: start;\n  column-gap: var(--fandhe-space-2);\n  padding-block-end: var(--fandhe-space-4);\n}\n\
[data-blocks-docs-layout-toc-progress-item]::before {\n  content: \"\";\n  position: absolute;\n  top: 0.875rem;\n  bottom: -0.125rem;\n  left: 0.75rem;\n  width: 2px;\n  background: var(--fandhe-color-border);\n}\n\
[data-blocks-docs-layout-toc-progress-item-state=\"done\"]::before {\n  background: var(--fandhe-color-accent);\n}\n\
[data-blocks-docs-layout-toc-progress-item-state=\"current\"]::before {\n  background: linear-gradient(to bottom, var(--fandhe-color-accent) 50%, var(--fandhe-color-border) 50%);\n}\n\
[data-blocks-docs-layout-toc-progress-item]:last-child::before {\n  display: none;\n}\n\
[data-blocks-docs-layout-toc-progress-marker] {\n  position: relative;\n  z-index: 1;\n  display: flex;\n  align-items: center;\n  justify-content: center;\n  width: 1.5rem;\n  height: 1.5rem;\n  border-radius: var(--fandhe-radius-full);\n  background: var(--fandhe-color-bg);\n  border: 2px solid var(--fandhe-color-border);\n  font-size: var(--fandhe-font-font-size-xs);\n  color: var(--fandhe-color-fg-muted);\n}\n\
[data-blocks-docs-layout-toc-progress-item-state=\"done\"] [data-blocks-docs-layout-toc-progress-marker] {\n  border-color: var(--fandhe-color-accent);\n}\n\
[data-blocks-docs-layout-toc-progress-item-state=\"current\"] [data-blocks-docs-layout-toc-progress-marker] {\n  border-color: var(--fandhe-color-accent);\n  background: var(--fandhe-color-accent);\n  color: var(--fandhe-color-accent-fg);\n}\n\
[data-blocks-docs-layout-toc-progress-nav] [data-scope=\"link\"][data-part=\"root\"] {\n  padding-block-start: 0.125rem;\n}\n\
[data-blocks-docs-layout-toc-progress-nav] [data-scope=\"link\"][data-part=\"root\"][aria-current=\"location\"] {\n  font-weight: var(--fandhe-font-weight-semibold);\n}\n\
@container blocks-docs-layout-toc-progress (min-width: 40rem) {\n  .blocks-docs-layout-toc-progress-layout {\n    grid-template-columns: minmax(0, 1fr) 14rem;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, CURRENT_INDEX, LAYOUT_CSS, SECTIONS, TOC_LIMIT};
    use fandhe_frontend_core::render;

    /// Demo が使用部品（link/text）の anatomy をすべて実際に出力していること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        assert!(html.contains("data-scope=\"link\""));
        assert!(html.contains("data-scope=\"text\""));
    }

    /// 目次は先頭 10 項目ちょうど。11・12 項目目の href は目次に現れない。
    /// マーカー 01〜10 がすべて出力される。
    #[test]
    fn toc_lists_exactly_first_ten_numbered_items() {
        assert!(SECTIONS.len() > TOC_LIMIT);
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-docs-layout-toc-progress-item=\"\"")
                .count(),
            TOC_LIMIT
        );
        for section in SECTIONS.iter().skip(TOC_LIMIT) {
            let href = format!("href=\"#{}\"", section.id);
            assert!(
                !html.contains(&href),
                "11th+ item should not appear in toc: {href}"
            );
        }
        for number in ["01", "02", "03", "04", "05", "06", "07", "08", "09", "10"] {
            assert!(html.contains(number), "marker {number} should be present");
        }
    }

    /// 目次の各 `href="#x"` に対し、本文側に `id="x"` がちょうど 1 件ある。
    #[test]
    fn every_toc_link_targets_exactly_one_id() {
        let html = render(&demo());
        for section in SECTIONS.iter().take(TOC_LIMIT) {
            let href = format!("href=\"#{}\"", section.id);
            let id = format!("id=\"{}\"", section.id);
            assert_eq!(
                html.matches(&href).count(),
                1,
                "missing href for {}",
                section.id
            );
            assert_eq!(
                html.matches(&id).count(),
                1,
                "missing/duplicate id for {}",
                section.id
            );
        }
    }

    /// 進捗状態は done が CURRENT_INDEX 件・current が 1 件・aria-current="location"
    /// が 1 件・aria-current="page" が 0 件。
    #[test]
    fn progress_states_and_aria_current() {
        let html = render(&demo());
        assert_eq!(
            html.matches("data-blocks-docs-layout-toc-progress-item-state=\"done\"")
                .count(),
            CURRENT_INDEX
        );
        assert_eq!(
            html.matches("data-blocks-docs-layout-toc-progress-item-state=\"current\"")
                .count(),
            1
        );
        assert_eq!(html.matches("aria-current=\"location\"").count(), 1);
        assert_eq!(html.matches("aria-current=\"page\"").count(), 0);
    }

    /// 目次は可視見出しを持たず、aria-label のみでアクセシブルネームを持つ。
    #[test]
    fn toc_has_no_visible_heading() {
        let html = render(&demo());
        assert!(html.contains("aria-label=\"このページの目次\""));
        for level in ["<h1", "<h2", "<h3", "<h4", "<h5", "<h6"] {
            assert!(!html.contains(level), "demo should not emit {level}");
        }
    }

    /// `<form>`・`type="submit"`・`href="#"`・`data:` URI を出力しない
    /// （`crate::blocks` モジュール doc）。
    #[test]
    fn demo_has_no_form_dead_link_or_data_uri() {
        let html = render(&demo());
        for absent in ["<form", "type=\"submit\"", "href=\"#\"", "src=\"data:"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
    }

    /// `docs-toc` トークンを出力・CSS 宣言のいずれにも持たない
    /// （scroll spy との誤結合防止）。
    #[test]
    fn never_emits_docs_toc_class() {
        let html = render(&demo());
        assert!(!html.contains("docs-toc"));
        assert!(!LAYOUT_CSS.contains("docs-toc"));
    }

    /// [`LAYOUT_CSS`] は `<` を含まず（REQ-1: `</style>` 脱出防止）、既定
    /// 1 列の規則が `@container` より前にあり、現在リンク規則のセレクタが
    /// `[data-blocks-docs-layout-toc-progress-nav]` を使う。
    #[test]
    fn layout_css_is_safe_and_mobile_first() {
        assert!(!LAYOUT_CSS.contains('<'));
        let default_pos = LAYOUT_CSS
            .find(".blocks-docs-layout-toc-progress-layout {")
            .expect("default rule should exist");
        let container_pos = LAYOUT_CSS
            .find("@container blocks-docs-layout-toc-progress")
            .expect("container query should exist");
        assert!(default_pos < container_pos);
        assert!(LAYOUT_CSS.contains("[data-blocks-docs-layout-toc-progress-nav] [data-scope=\"link\"][data-part=\"root\"][aria-current=\"location\"]"));
    }

    /// ルート class（`demo_class` とは別名）が `demo()` の出力へ実際に
    /// 現れること。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-docs-layout-toc-progress-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-docs-layout-toc-progress-layout"
        );
    }
}
