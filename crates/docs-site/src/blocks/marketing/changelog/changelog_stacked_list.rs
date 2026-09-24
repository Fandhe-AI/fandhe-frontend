//! `changelog-stacked-list` block（イシュー #2819。親トラッキング #2730
//! 「Blocks 目的別パーツ拡充ツリー」配下、対応表 ID R0045（下線区切りの
//! 縦積み changelog）を主参照、R0044（カード + 全面リンク化）を副参照と
//! する合成例。取得手段・ファイル名・内部コンポーネント識別子は記載しない
//! （`docs/design/motion-reference-adoption-policy.md` §9 と同じライセンス
//! 上の転記制限）。
//!
//! # 使用部品
//!
//! `heading` / `text` / `badge` / `card` / `list` / `link-overlay` /
//! `separator` の 7 部品を合成する（[`BLOCK`] の `parts` に一致させる契約、
//! `crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が検証する）。
//!
//! # 2 つの表示例を並べる理由
//!
//! Issue が要求する区切り方（罫線区切り／カード化）は互いに排他な選択肢
//! だが、`card`・`link-overlay` を Demo 上に実際に出力する契約
//! （`blocks_contract.rs` の「使用部品と一致」検証）を満たすため、主参照
//! （R0045: 罫線区切り、`.blocks-changelog-stacked-list-rows`）と副参照
//! （R0044: カード + 全面リンク、`.blocks-changelog-stacked-list-cards`）の
//! 両方を同一データから並べて示す。カード版は変更点リストと「最新」badge
//! を省く（原稿「原案差分メモ」参照）。
//!
//! # 狭い幅でも日付を折り返さない（CSS）
//!
//! `.blocks-changelog-stacked-list-title-row` を `flex` + `justify-content:
//! space-between` にし、タイトル側（`[data-blocks-changelog-stacked-list-
//! title]`）へ `flex: 1 1 auto; min-width: 0; overflow-wrap: anywhere;`、
//! 日付側（`.blocks-changelog-stacked-list-date`）へ `flex-shrink: 0;
//! white-space: nowrap;` を与える。幅が縮んでもタイトル側が折り返し・
//! 縮小し、日付は 1 行のまま右端に留まる。
//!
//! # `link_overlay` の配置（`card::root` の直接の子）
//!
//! `link_overlay::root` の `border-radius: inherit` は 1 段先の親要素の
//! 計算値しか継承できないため、`card::root` の**直接の子**として置く
//! （`blog_list_image` 等の先例と同じ 2 段連鎖の判断軸）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（styled パート関数）と
//! `fandhe_frontend_core::text`（テキストノード生成関数）が同名のため、
//! styled 側を `styled_text` として取り込む（`crate::blocks` 内の他 block
//! と同じ回避方法）。
//!
//! # `<time datetime>` と表示日付の一致
//!
//! 各リリースは機械可読な ISO 8601 日付（`datetime` 属性）と表示用の
//! 日本語表記を別々のフィールドとして持つが、常に同じ日を指す値を組に
//! する（`blog_list_image` と同じ不変条件）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `heading::heading` / `text::text` / `badge::badge` / `card::root` /
//! `list::root` / `separator::separator` / `link_overlay::root` はいずれも
//! `drop_class_attr` により呼び出し側 `attrs` の `class` を黙って除去する
//! 契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-changelog-stacked-list-*` 属性で渡す。素の `div`/`article`/
//! `time` には `class` がそのまま効くため、それらは
//! `.blocks-changelog-stacked-list-*` クラスセレクタを使う。`card::body`
//! （variant を持たないパート）にも `class` がそのまま効く。
//!
//! # 見出しレベルに `H3`/`H4` を使う理由
//!
//! ページ側が `## Demo` として `h2` を出すため、セクション見出しは
//! `HeadingLevel::H3`、リリースタイトルは `HeadingLevel::H4` にする
//! （`blog_list_image` と同じ判断）。
//!
//! # `href="#"` を使わない・`base_path` を受け取れない制約
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節と同じ判断軸で、
//! [`Block::demo`] は `fn() -> Node` のため `base_path` を受け取れない。
//! `linkcheck::check_links` は外部リンクを検証対象外とするため、リンク先は
//! すべて [`REPO`]（実在する GitHub リポジトリへの外部絶対 URL）に固定
//! する（`blog_list_image` と同じ先例判断）。
//!
//! # id を出力しない
//!
//! 全 block を横断する契約テスト（`demo_output_has_no_dangling_aria_
//! references_or_duplicate_ids`）が id の重複を検知するため、id は出力
//! しない。カード版のアクセシブルな名前は `overlay` の `aria-label` で
//! 付ける。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言・バージョン番号・日付はすべて架空のもの（実企業名・実
//! クレデンシャル・PII を含まない）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{article, div, el, text, Node};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps, BadgeVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps, CardVariant};
use fandhe_frontend_pre_styled_ui::heading::{
    heading, HeadingLevel, HeadingProps, HeadingSize, HeadingWeight,
};
use fandhe_frontend_pre_styled_ui::link_overlay::{self, overlay};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::separator::{separator, SeparatorProps};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextVariant};

/// リンク先の固定外部 URL（モジュール doc「`href="#"` を使わない」節参照）。
const REPO: &str = "https://github.com/Fandhe-AI/fandhe-frontend";

/// リリース 1 件分のダミーデータ（架空、実在の製品・企業とは無関係）。
struct Release {
    version: &'static str,
    date_iso: &'static str,
    date_label: &'static str,
    title: &'static str,
    tags: &'static [&'static str],
    description: &'static str,
    changes: &'static [&'static str],
}

/// リリース一覧（架空、3 件）。1 件目はタイトルを長めにし、狭い幅でも
/// 日付が折り返さずタイトル側だけが縮むことを示す（モジュール doc「狭い
/// 幅でも日付を折り返さない」節）。
const RELEASES: [Release; 3] = [
    Release {
        version: "v3.4.0",
        date_iso: "2026-09-20",
        date_label: "2026年9月20日",
        title: "検索インデックスの決定性検証とページ分割の下準備を追加",
        tags: &["Feature", "Search"],
        description: "全文検索インデックスの生成手順を見直し、将来のセクション単位分割に備えた土台を整えました。",
        changes: &[
            "検索インデックスのビルド時サイズ計測を強化",
            "索引対象からナビゲーション専用ページを除外",
        ],
    },
    Release {
        version: "v3.3.2",
        date_iso: "2026-09-12",
        date_label: "2026年9月12日",
        title: "カード合成部品のフォーカスリング角丸追従を修正",
        tags: &["Fix", "Accessibility"],
        description: "カード全面リンク化時にフォーカスリングが角丸に追従しない不具合を修正しました。",
        changes: &["border-radius の継承連鎖を 2 段へ拡張"],
    },
    Release {
        version: "v3.3.1",
        date_iso: "2026-09-02",
        date_label: "2026年9月2日",
        title: "変更履歴ページのテンプレートを整理",
        tags: &["Chore"],
        description: "変更履歴の記述フォーマットを統一し、リリースごとの差分を追いやすくしました。",
        changes: &["タグの表記ゆれを解消", "日付表記を ISO 8601 に統一"],
    },
];

/// `<time datetime>` を組み立てる（モジュール doc「`<time datetime>` と
/// 表示日付の一致」節）。
fn release_date(iso: &str, label: &str) -> Node {
    el(
        "time",
        vec![
            ("class", "blocks-changelog-stacked-list-date"),
            ("datetime", iso),
        ],
        vec![text(label)],
    )
}

/// タイトル行（タイトル左・日付右の両端揃え）。
fn title_row(release: &Release, level: HeadingLevel) -> Node {
    div(
        vec![("class", "blocks-changelog-stacked-list-title-row")],
        vec![
            heading(
                level,
                &HeadingProps::default(),
                vec![("data-blocks-changelog-stacked-list-title", "")],
                vec![text(format!("{} — {}", release.version, release.title))],
            ),
            release_date(release.date_iso, release.date_label),
        ],
    )
}

/// タグの badge 群。
fn tag_row(release: &Release, featured: bool) -> Node {
    let mut badges: Vec<Node> = release
        .tags
        .iter()
        .map(|tag| {
            badge::badge(
                &BadgeProps::default(),
                vec![("data-blocks-changelog-stacked-list-tag", "")],
                vec![text(*tag)],
            )
        })
        .collect();
    if featured {
        badges.insert(
            0,
            badge::badge(
                &BadgeProps {
                    variant: BadgeVariant::Solid,
                    ..BadgeProps::default()
                },
                vec![("data-blocks-changelog-stacked-list-tag-latest", "")],
                vec![text("最新")],
            ),
        );
    }
    div(
        vec![("class", "blocks-changelog-stacked-list-tags")],
        badges,
    )
}

/// 罫線区切り表示（R0045、主参照）のリリース 1 件。
fn stacked_entry(release: &Release, featured: bool) -> Node {
    article(
        vec![("class", "blocks-changelog-stacked-list-entry")],
        vec![
            title_row(release, HeadingLevel::H4),
            tag_row(release, featured),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text(release.description)],
            ),
            list::root(
                ListType::Unordered,
                ListVariant::Marker,
                vec![("data-blocks-changelog-stacked-list-changes", "")],
                release
                    .changes
                    .iter()
                    .map(|c| list::item(vec![], vec![text(*c)]))
                    .collect(),
            ),
        ],
    )
}

/// カード + 全面リンク化表示（R0044、副参照）のリリース 1 件。変更点
/// リストは省く（原稿「原案差分メモ」参照）。
fn card_entry(release: &Release) -> Node {
    card::root(
        CardProps {
            variant: CardVariant::Outline,
            ..CardProps::default()
        },
        vec![("data-blocks-changelog-stacked-list-card", "")],
        vec![link_overlay::root(
            vec![("data-blocks-changelog-stacked-list-card-link", "")],
            vec![
                card::body(
                    vec![],
                    vec![
                        title_row(release, HeadingLevel::H4),
                        tag_row(release, false),
                        styled_text::text(
                            &TextProps {
                                variant: TextVariant::Muted,
                                ..TextProps::default()
                            },
                            vec![],
                            vec![text(release.description)],
                        ),
                    ],
                ),
                overlay(REPO, vec![("aria-label", release.title)], vec![]),
            ],
        )],
    )
}

/// `changelog-stacked-list` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「2 つの表示例を並べる理由」節）。
pub fn demo() -> Node {
    let header = div(
        vec![("class", "blocks-changelog-stacked-list-header")],
        vec![
            heading(
                HeadingLevel::H3,
                &HeadingProps {
                    size: HeadingSize::Xl2,
                    weight: HeadingWeight::Bold,
                },
                vec![],
                vec![text("変更履歴")],
            ),
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("直近のリリース内容をまとめています。")],
            ),
        ],
    );

    let mut rows: Vec<Node> = Vec::new();
    for (index, release) in RELEASES.iter().enumerate() {
        rows.push(stacked_entry(release, index == 0));
        if index + 1 < RELEASES.len() {
            rows.push(separator(
                &SeparatorProps::default(),
                vec![("data-blocks-changelog-stacked-list-separator", "")],
            ));
        }
    }

    let stacked_example = div(
        vec![("class", "blocks-changelog-stacked-list-example")],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("罫線区切り")],
            ),
            div(vec![("class", "blocks-changelog-stacked-list-rows")], rows),
        ],
    );

    let card_example = div(
        vec![("class", "blocks-changelog-stacked-list-example")],
        vec![
            styled_text::text(
                &TextProps {
                    variant: TextVariant::Muted,
                    ..TextProps::default()
                },
                vec![],
                vec![text("カード（全面リンク）")],
            ),
            div(
                vec![("class", "blocks-changelog-stacked-list-cards")],
                RELEASES.iter().map(card_entry).collect(),
            ),
        ],
    );

    div(
        vec![("class", "blocks-changelog-stacked-list-layout")],
        vec![header, stacked_example, card_example],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/changelog-stacked-list/",
    title: "changelog-stacked-list",
    category: BlockCategory::Changelog,
    rust_source: "crates/docs-site/src/blocks/marketing/changelog/changelog_stacked_list.rs",
    demo_class: "blocks-changelog-stacked-list",
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
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Link Overlay",
            path: "/themes/link-overlay/",
        },
        Part {
            label: "Separator",
            path: "/themes/separator/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `changelog_stacked_list` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節。他 block と同型で `pub(super)` ではなく
/// 本ファイル内 private 定数として `super::stylesheet` 経由の `push_css` で
/// 連結される）。
///
/// セレクタは `.blocks-changelog-stacked-list-*` と
/// `[data-blocks-changelog-stacked-list-*]` のみを用い、他 block や部品の
/// 素のセレクタへ影響させない（`blog_list_image` と同じ名前空間分離）。
const LAYOUT_CSS: &str = "\
.blocks-changelog-stacked-list-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-10);\n}\n\
.blocks-changelog-stacked-list-header {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-changelog-stacked-list-example {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
.blocks-changelog-stacked-list-rows {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-changelog-stacked-list-entry {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  min-width: 0;\n}\n\
.blocks-changelog-stacked-list-title-row {\n  display: flex;\n  align-items: baseline;\n  justify-content: space-between;\n  gap: var(--fandhe-space-4);\n  min-width: 0;\n}\n\
[data-blocks-changelog-stacked-list-title] {\n  flex: 1 1 auto;\n  min-width: 0;\n  overflow-wrap: anywhere;\n  margin: 0;\n}\n\
.blocks-changelog-stacked-list-date {\n  flex-shrink: 0;\n  white-space: nowrap;\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
.blocks-changelog-stacked-list-tags {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-2);\n}\n\
[data-blocks-changelog-stacked-list-separator] {\n  margin: 0;\n}\n\
.blocks-changelog-stacked-list-cards {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-4);\n}\n\
[data-blocks-changelog-stacked-list-card] {\n  display: flex;\n  flex-direction: column;\n}\n\
[data-blocks-changelog-stacked-list-card-link] {\n  display: flex;\n  flex-direction: column;\n}\n";

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
            "data-scope=\"heading\"",
            "data-scope=\"text\"",
            "data-scope=\"badge\"",
            "data-scope=\"card\"",
            "data-scope=\"list\"",
            "data-scope=\"link-overlay\"",
            "data-scope=\"separator\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains("datetime=\"2026-09-20\""));
        // overlay の <a> はカード版 3 件のみ（罫線版はリンクを持たない）。
        assert_eq!(html.matches("<a ").count(), 3);
        // separator はリリース 3 件のうち区切りが必要な 2 箇所のみ。
        assert_eq!(
            html.matches("data-blocks-changelog-stacked-list-separator")
                .count(),
            2
        );
        assert!(!html.contains("href=\"#\""));
        assert!(!html.contains("<form"));
        assert!(!html.contains("src=\"data:"));
        assert!(!html.contains("id=\""));
    }

    /// [`LAYOUT_CSS`] が日付を折り返さないための宣言を持つこと。
    #[test]
    fn layout_css_keeps_date_unwrapped() {
        assert!(LAYOUT_CSS.contains("white-space: nowrap"));
        assert!(LAYOUT_CSS.contains("flex-shrink: 0"));
        assert!(LAYOUT_CSS.contains("min-width: 0"));
        assert!(!LAYOUT_CSS.contains('<'));
    }

    /// ルート grid class（`demo_class` とは別名）が `demo()` の出力へ
    /// 実際に現れること（`blog_list_image` 等と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-changelog-stacked-list-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-changelog-stacked-list-layout"
        );
    }
}
