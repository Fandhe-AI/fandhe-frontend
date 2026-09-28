# list-narrow-activity

`list` / `avatar` / `text` の 3 部品を合成した、サイドパネルや補助カラムのような狭い幅に置くアクティビティリストの実例です。Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID R1295、集約元は R1296。出典の固有名・ファイル名は記載しません）。

- 静的表示です。docs サイトは無 JS のため、状態機械・フォーム・データ取得を持たない固定描画にしています。
- 文言・人名・日時はすべて架空のものです。
- データ取得・送信は行わず、`<form>` は使いません。ボタンも置きません。
- Demo には 2 種類のインスタンスを並べています。左が名前・時刻・本文を並べた代表構成（対応表 ID R1295）で、本文は 2 行を超えると省略します。右がコミット活動を 1 行に要約した版（対応表 ID R1296）で、作業内容は 1 行を超えると省略します。
- 省略はいずれも CSS だけ（`-webkit-line-clamp`/`text-overflow: ellipsis`）で行っており、DOM には全文が残ります。スクリーンリーダーは全文を読めます。
- 各パネルは幅 20rem に固定しています。画面が狭いときは折り返して縦に並びます。
- 集約元は 2 件です（対応表 ID R1295・R1296）。取り込んだのは領域配置・部品構成・状態の見せ方といった構造のみで、文言・配色・装飾・アイコン・内部識別子は取り込んでいません。

## Rust コード

```rust
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
```

## 原案差分メモ

- 対応表 ID R1295（代表構成）と R1296（コミット活動を 1 行に要約した版）の差分を、1 つの Demo 内に「代表版」「コミット要約版」の 2 パネルとして並置しています。
- 差分の中心は 2 行目の省略方式です。代表版は本文を `-webkit-line-clamp: 2` で 2 行に収め、コミット要約版は作業内容を `white-space: nowrap` + `text-overflow: ellipsis` で 1 行に切り詰めます。
- 参照元の文言・配色・アイコンは使用せず、独自に書いています。実在の人物・企業名等は含みません。

関連情報: [List](../themes/list.md) / [Avatar](../themes/avatar.md) / [Text](../themes/text.md)
