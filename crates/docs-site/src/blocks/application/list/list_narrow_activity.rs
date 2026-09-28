//! `list-narrow-activity` block（イシュー #2922。親トラッキング #2892
//! 「Blocks アプリケーション A」配下、対応表 ID R1295（代表構成）/R1296
//! （コミット活動を 1 行に要約した版）の 2 件を構造の参照元とする合成例。
//! サイドパネルや補助カラムのような狭い幅に置くアクティビティリスト。
//! 各行は小さなアバター・1 行目（名前と時刻）・2 行目（本文または作業
//! 内容）で構成する。取得手段・ファイル名・内部コンポーネント識別子は
//! 記載しない（`docs/design/motion-reference-adoption-policy.md` §9 と
//! 同じライセンス上の転記制限）。
//!
//! **Application / List カテゴリで最初の block**（`super`（`list/mod.rs`）
//! 参照）。
//!
//! # 使用部品
//!
//! `list` / `avatar` / `text` の 3 部品を合成する（[`BLOCK`] の `parts` に
//! 一致させる契約、`crates/docs-site/tests/blocks_nav.rs`/
//! `blocks_contract.rs` が検証する）。
//!
//! # 2 参照 ID の畳み込み方（Demo は 2 インスタンス）
//!
//! - **代表版**（`data-blocks-list-narrow-activity-variant="activity"`。
//!   R1295 が主参照）: 名前・時刻・本文の行を並べる。本文は
//!   `-webkit-line-clamp: 2` で 2 行に省略する。
//! - **コミット要約版**（`data-blocks-list-narrow-activity-variant=
//!   "commits"`。R1296 が集約元）: 名前・時刻・作業内容 1 行要約を並べる。
//!   要約は `white-space: nowrap` + `text-overflow: ellipsis` で 1 行に
//!   切り詰める。
//!
//! いずれも省略は CSS だけで行い、DOM には全文が残る（スクリーンリーダーは
//! 全文を読める）。
//!
//! # 静的表示（無 JS、架空データ固定）
//!
//! [`ACTIVITY_ENTRIES`]/[`COMMIT_ENTRIES`] は呼び出しごとに同一の `Node` を
//! 生成する純関数（[`demo`]）が参照するだけの固定配列であり、状態機械・
//! フォーム・データ取得を一切持たない。
//!
//! # `<form>` を持たない・`id` 属性を持たない理由
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。ボタンも置かない。2 インスタンス（activity/commits）を同一
//! ページへ並べるが、`id`/`aria-controls`/`aria-labelledby` を一切出力しない
//! ため重複・宙に浮いた参照は発生しない（`aria-label` のみで各リストの
//! アクセシブルネームを与える）。文言・人名・日時はすべて架空のもの
//! （実在の人物・企業・PII を含まない）。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、本 block では前者を `styled_text` へ別名 import する
//! （`feed_comments_timeline` と異なり、本 block は 2 行目本文に styled
//! `text` を使うため衝突を避ける必要がある）。1 行目（名前・時刻）は
//! core の `div`/`span`/`el` のみで組み、`text::text` は 2 行目にだけ使う。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `list::root`/`avatar::root`/`text::text` はいずれも `drop_class_attr`
//! により呼び出し側 `attrs` の `class` を黙って除去する契約を持つため、
//! Demo 固有のスタイルフックは `data-blocks-list-narrow-activity-*` 属性で
//! 渡す。素の `div`/`span`/`time` には `.blocks-list-narrow-activity-*`
//! クラスセレクタを使う（`feed_comments_timeline` と同じ判断）。
//!
//! # `list` recipe の item 余白を上書きする理由
//!
//! `pre-styled-ui` の `list` recipe は item に `margin-block:
//! var(--fandhe-space-1)` を既定で与えるが、本 block は行間に区切り線
//! （border-top）を挟む密な一覧を志向するため、詳細度
//! `.blocks-list-narrow-activity-panel [data-blocks-list-narrow-activity-item]`
//! （複合セレクタで recipe の単一 `[data-part="item"]` 宣言より詳細度を
//! 上げる）で上書きする。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::list::{self, ListType, ListVariant};
use fandhe_frontend_pre_styled_ui::text::{self as styled_text, TextProps, TextSize, TextVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// アクティビティ 1 件分のダミーデータ（R1295 主参照。架空）。
struct ActivityEntry {
    name: &'static str,
    time_iso: &'static str,
    time_label: &'static str,
    body: &'static str,
}

/// コミット活動 1 行要約のダミーデータ（R1296 集約元。架空）。
struct CommitEntry {
    name: &'static str,
    time_iso: &'static str,
    time_label: &'static str,
    summary: &'static str,
}

/// 代表版（activity）のエントリ一覧（架空、5 件）。本文のうち複数件を
/// 2 行を超える長さにし、省略（line-clamp）を実際に見せる。
const ACTIVITY_ENTRIES: [ActivityEntry; 5] = [
    ActivityEntry {
        name: "佐藤",
        time_iso: "2026-09-28T09:12:00",
        time_label: "9:12",
        body: "不具合の再現手順を確認しました。特定の条件下（狭幅レイアウト・長い本文）でのみ発生するようです。明日中に修正版を用意します。",
    },
    ActivityEntry {
        name: "田中",
        time_iso: "2026-09-28T09:40:00",
        time_label: "9:40",
        body: "レビューコメントに対応しました。",
    },
    ActivityEntry {
        name: "鈴木",
        time_iso: "2026-09-28T10:05:00",
        time_label: "10:05",
        body: "デザインのトークン適用を見直し、狭幅パネルでも余白が破綻しないことを確認しました。スクリーンショットを添付します。",
    },
    ActivityEntry {
        name: "山田",
        time_iso: "2026-09-28T10:31:00",
        time_label: "10:31",
        body: "承認しました。",
    },
    ActivityEntry {
        name: "高橋",
        time_iso: "2026-09-28T11:02:00",
        time_label: "11:02",
        body: "テストを追加しました。境界値（0 行・1 行・複数行）を網羅しています。",
    },
];

/// コミット要約版（commits）のエントリ一覧（架空、4 件）。作業内容は
/// 1 行に収まらない長さのものを含める（省略〔ellipsis〕を実際に見せる）。
const COMMIT_ENTRIES: [CommitEntry; 4] = [
    CommitEntry {
        name: "佐藤",
        time_iso: "2026-09-28T08:50:00",
        time_label: "8:50",
        summary: "3 件のコミットを feature/sample-branch-alpha へ push しました（認証まわりのバグ修正・テスト追加・ドキュメント更新）",
    },
    CommitEntry {
        name: "田中",
        time_iso: "2026-09-28T09:20:00",
        time_label: "9:20",
        summary: "feature/sample-branch-beta を main へマージしました",
    },
    CommitEntry {
        name: "鈴木",
        time_iso: "2026-09-28T09:55:00",
        time_label: "9:55",
        summary: "1 件のコミットを feature/sample-branch-gamma へ push しました（表示崩れの修正）",
    },
    CommitEntry {
        name: "山田",
        time_iso: "2026-09-28T10:15:00",
        time_label: "10:15",
        summary: "リリースタグ sample-v1.4.0 を作成しました",
    },
];

/// 名前の先頭 1 文字をアバターのフォールバック表示に使う（架空の人名は
/// いずれも姓 1 文字が識別に十分なため、`feed_comments_timeline` と同じ
/// 判断）。
fn initial_avatar(name: &str) -> Node {
    let initial: String = name.chars().take(1).collect();
    avatar::root(
        &AvatarProps {
            size: Size::Xs,
            ..AvatarProps::default()
        },
        vec![("data-blocks-list-narrow-activity-avatar", "")],
        vec![avatar::fallback(
            ImageStatus::default(),
            vec![],
            vec![text(initial)],
        )],
    )
}

/// `<time datetime>` を組み立てる（機械可読な ISO 値と表示値は常に同じ
/// 日時を指す組にする不変条件、`feed_comments_timeline` と同じ判断）。
fn entry_time(iso: &str, label: &str) -> Node {
    el(
        "time",
        vec![
            ("class", "blocks-list-narrow-activity-time"),
            ("datetime", iso),
        ],
        vec![text(label)],
    )
}

/// 1 行目（名前・時刻）を組み立てる（両インスタンス共通）。
fn row_header(name: &str, time_iso: &str, time_label: &str) -> Node {
    div(
        vec![("class", "blocks-list-narrow-activity-header")],
        vec![
            span(
                vec![("class", "blocks-list-narrow-activity-name")],
                vec![text(name)],
            ),
            entry_time(time_iso, time_label),
        ],
    )
}

/// 行の外枠（アバター列 + 本文列の grid）を組み立てる。
fn row(avatar_node: Node, header: Node, second_line: Node) -> Node {
    div(
        vec![("class", "blocks-list-narrow-activity-row")],
        vec![
            avatar_node,
            div(
                vec![("class", "blocks-list-narrow-activity-main")],
                vec![header, second_line],
            ),
        ],
    )
}

/// 代表版（activity）の 1 行分の `list::item`。
fn activity_item(entry: &ActivityEntry) -> Node {
    let body = styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-list-narrow-activity-body", "")],
        vec![text(entry.body)],
    );
    list::item(
        vec![("data-blocks-list-narrow-activity-item", "")],
        vec![row(
            initial_avatar(entry.name),
            row_header(entry.name, entry.time_iso, entry.time_label),
            body,
        )],
    )
}

/// コミット要約版（commits）の 1 行分の `list::item`。
fn commit_item(entry: &CommitEntry) -> Node {
    let summary = styled_text::text(
        &TextProps {
            size: TextSize::Sm,
            variant: TextVariant::Muted,
            ..TextProps::default()
        },
        vec![("data-blocks-list-narrow-activity-summary", "")],
        vec![text(entry.summary)],
    );
    list::item(
        vec![("data-blocks-list-narrow-activity-item", "")],
        vec![row(
            initial_avatar(entry.name),
            row_header(entry.name, entry.time_iso, entry.time_label),
            summary,
        )],
    )
}

/// 代表版パネル（R1295 主参照。本文 2 行クランプ）。
fn activity_panel() -> Node {
    div(
        vec![
            ("class", "blocks-list-narrow-activity-panel"),
            ("data-blocks-list-narrow-activity-variant", "activity"),
        ],
        vec![
            span(
                vec![("class", "blocks-list-narrow-activity-panel-heading")],
                vec![text("最近のアクティビティ")],
            ),
            list::root(
                ListType::Unordered,
                ListVariant::Plain,
                vec![("aria-label", "最近のアクティビティ")],
                ACTIVITY_ENTRIES.iter().map(activity_item).collect(),
            ),
        ],
    )
}

/// コミット要約版パネル（R1296 集約元。作業内容 1 行切り詰め）。
fn commits_panel() -> Node {
    div(
        vec![
            ("class", "blocks-list-narrow-activity-panel"),
            ("data-blocks-list-narrow-activity-variant", "commits"),
        ],
        vec![
            span(
                vec![("class", "blocks-list-narrow-activity-panel-heading")],
                vec![text("最近のコミット活動")],
            ),
            list::root(
                ListType::Unordered,
                ListVariant::Plain,
                vec![("aria-label", "最近のコミット活動")],
                COMMIT_ENTRIES.iter().map(commit_item).collect(),
            ),
        ],
    )
}

/// `list-narrow-activity` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「静的表示」節）。代表版（R1295）・コミット
/// 要約版（R1296）の 2 インスタンスを幅固定で横並びに置く。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-list-narrow-activity-layout")],
        vec![activity_panel(), commits_panel()],
    )
}
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/list-narrow-activity/",
    title: "list-narrow-activity",
    category: BlockCategory::List,
    rust_source: "crates/docs-site/src/blocks/application/list/list_narrow_activity.rs",
    demo_class: "blocks-list-narrow-activity",
    parts: &[
        Part {
            label: "List",
            path: "/themes/list/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Text",
            path: "/themes/text/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `list_narrow_activity` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。
///
/// # パネル幅の固定
///
/// 各パネルは `width: 20rem; max-width: 100%;` で固定し、狭い画面幅では
/// `max-width` により縮小させる（`flex-wrap: wrap` と併用し、2 パネルが
/// 収まらない幅では縦積みに折り返す）。
///
/// # 行の grid
///
/// アバター列（`auto`）| 本文列（`minmax(0, 1fr)`）の 2 列。`min-width: 0`
/// を本文列（`main`）に与え、`text-overflow: ellipsis`/`line-clamp` が
/// 実際に効くようにする（`minmax(0, 1fr)` だけでは孫要素の `min-width`
/// 既定値 `auto` により省略が効かないため、`main` 自身にも明示する）。
///
/// `list` recipe の `ListVariant::Plain` item は `display: flex` の
/// row 方向コンテナだが、flex item である本 grid（`row`）自身は既定の
/// `flex-grow: 0` のままでは内容幅にしか広がらず、短いエントリで
/// `header` の `justify-content: space-between` が効く前の余白が狭く
/// タイムスタンプの縦位置が行ごとに揃わない。`flex: 1 1 auto;
/// min-width: 0;` を明示して item の残り幅いっぱいに `row` を広げ、
/// header 内の space-between が item 全幅を基準に効くようにする。
///
/// # 本文の 2 行クランプ・要約の 1 行切り詰め
///
/// `-webkit-line-clamp: 2`（`blog_grid_text` と同じ書き方）と
/// `white-space: nowrap; overflow: hidden; text-overflow: ellipsis;` を
/// それぞれ専用の `data-*` フックへ適用する。
///
/// # item 間の区切り線・余白の上書き
///
/// `list` recipe の item 既定余白（`margin-block: var(--fandhe-space-1)`）
/// を、詳細度で確実に勝つ複合セレクタ
/// （`.blocks-list-narrow-activity-panel [data-blocks-list-narrow-activity-item]`）
/// で上書きし、`border-top` による区切り線を追加する（モジュール doc
/// 「`list` recipe の item 余白を上書きする理由」節）。
const LAYOUT_CSS: &str = "\
.blocks-list-narrow-activity-layout {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-6);\n}\n\
.blocks-list-narrow-activity-panel {\n  width: 20rem;\n  max-width: 100%;\n  border: 1px solid var(--fandhe-color-border);\n  border-radius: var(--fandhe-radius-md, 0.375rem);\n  padding: var(--fandhe-space-4);\n}\n\
.blocks-list-narrow-activity-panel-heading {\n  display: block;\n  margin-bottom: var(--fandhe-space-3);\n  font-weight: var(--fandhe-font-weight-bold, 700);\n}\n\
.blocks-list-narrow-activity-panel [data-blocks-list-narrow-activity-item] {\n  margin-block: 0;\n  padding-block: var(--fandhe-space-3);\n  border-top: 1px solid var(--fandhe-color-border);\n}\n\
.blocks-list-narrow-activity-panel [data-blocks-list-narrow-activity-item]:first-child {\n  border-top: none;\n  padding-top: 0;\n}\n\
.blocks-list-narrow-activity-row {\n  display: grid;\n  grid-template-columns: auto minmax(0, 1fr);\n  gap: var(--fandhe-space-2);\n  flex: 1 1 auto;\n  min-width: 0;\n}\n\
.blocks-list-narrow-activity-main {\n  min-width: 0;\n}\n\
.blocks-list-narrow-activity-header {\n  display: flex;\n  align-items: baseline;\n  justify-content: space-between;\n  gap: var(--fandhe-space-2);\n  min-width: 0;\n}\n\
.blocks-list-narrow-activity-name {\n  font-weight: var(--fandhe-font-weight-bold, 700);\n  overflow: hidden;\n  text-overflow: ellipsis;\n  white-space: nowrap;\n}\n\
.blocks-list-narrow-activity-time {\n  flex-shrink: 0;\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
.blocks-list-narrow-activity-panel [data-blocks-list-narrow-activity-body] {\n  margin-top: var(--fandhe-space-1);\n  display: -webkit-box;\n  -webkit-box-orient: vertical;\n  -webkit-line-clamp: 2;\n  line-clamp: 2;\n  overflow: hidden;\n}\n\
.blocks-list-narrow-activity-panel [data-blocks-list-narrow-activity-summary] {\n  margin-top: var(--fandhe-space-1);\n  white-space: nowrap;\n  overflow: hidden;\n  text-overflow: ellipsis;\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, ACTIVITY_ENTRIES, COMMIT_ENTRIES, LAYOUT_CSS};
    use fandhe_frontend_core::render;

    /// Demo が期待する 3 種の部品を出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"list\"",
            "data-scope=\"avatar\"",
            "data-scope=\"text\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
    }

    /// activity・commits 双方の variant フックが出力されていること。
    #[test]
    fn demo_wires_both_variant_hooks() {
        let html = render(&demo());
        assert!(html.contains(r#"data-blocks-list-narrow-activity-variant="activity""#));
        assert!(html.contains(r#"data-blocks-list-narrow-activity-variant="commits""#));
    }

    /// [`LAYOUT_CSS`] が 2 行クランプ・省略記号の宣言を持つこと。
    #[test]
    fn layout_css_has_clamp_and_ellipsis_rules() {
        assert!(LAYOUT_CSS.contains("-webkit-line-clamp: 2;"));
        assert!(LAYOUT_CSS.contains("text-overflow: ellipsis;"));
    }

    /// 非対話・安全性の不変条件（`<form>`・`id="..."` を持たないこと）。
    #[test]
    fn demo_never_contains_forbidden_markup() {
        let html = render(&demo());
        assert!(!html.contains("<form"));
        assert!(!html.contains("id=\""));
        assert!(!html.contains("<button"));
    }

    /// `<time datetime>` の件数がデータ件数の合計と一致すること。
    #[test]
    fn time_element_count_matches_entry_count() {
        let html = render(&demo());
        let expected = ACTIVITY_ENTRIES.len() + COMMIT_ENTRIES.len();
        assert_eq!(html.matches("<time ").count(), expected, "html={html}");
        assert_eq!(
            html.matches(" datetime=\"").count(),
            expected,
            "html={html}"
        );
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`blog_list_image` と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-list-narrow-activity-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-list-narrow-activity-layout"
        );
    }
}
