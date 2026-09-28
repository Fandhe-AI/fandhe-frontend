# feed-comments-timeline

`timeline` / `avatar` / `card` / `badge` / `textarea` / `select` / `button` / `tag` の 8 部品を合成した、コメントとイベントが混在するアクティビティフィードの実例です。Blocks セクションは新規部品を追加するものではなく、既存の Themes/Primitives 部品を組み合わせた実例集であることに注意してください（主参照は対応表 ID R0949、集約元は R0950・R0109・R0948。出典の固有名・ファイル名は記載しません）。

- 静的表示です。docs サイトは無 JS のため、状態機械・フォーム・データ取得を持たない固定描画にしています。
- 文言・人名・日時はすべて架空のものです。
- データ取得・送信は行わず、`<form>` は使いません。ボタンは `type="button"` のまま送信先を持ちません。
- Demo には 2 種類のインスタンスを並べています。上が状態変更・担当者割り当て・タグ付けのイベントとアバター付きコメントカードが時系列で混在する構成に、末尾へコメント投稿欄（アバター・複数行入力欄・公開範囲 select・送信ボタン）を添えたもの（対応表 ID R0949）、下がイベントのみを indicator の記号＋本文 1 行＋日時だけの簡潔な表示で並べた簡易版（対応表 ID R0948）です。
- 上のインスタンスには返信スレッドと投稿者バッジ（対応表 ID R0109）を 1 件のコメントへ組み込んでいます。
- 公開範囲の select は常に閉じた静的表示で、開閉には `fandhe-frontend-wasm-full` の JS 配線が必要です（docs サイトは JS ハイドレーションを行いません）。
- 狭い幅（48rem 未満）では、投稿欄のアバターと入力欄・操作行を縦積みに切り替えます。
- 集約元は 4 件です（対応表 ID R0949・R0950・R0109・R0948）。取り込んだのは領域配置・部品構成・状態の見せ方といった構造のみで、文言・配色・装飾・アイコン・内部識別子は取り込んでいません。

## Rust コード

```rust
use fandhe_frontend_core::{div, el, span, text, Node};
use fandhe_frontend_pre_styled_ui::avatar::{self, AvatarProps, ImageStatus};
use fandhe_frontend_pre_styled_ui::badge::{self, BadgeProps};
use fandhe_frontend_pre_styled_ui::button::{self, ButtonProps, ButtonVariant};
use fandhe_frontend_pre_styled_ui::card::{self, CardProps};
use fandhe_frontend_pre_styled_ui::select::{self, OpenState, SelectProps};
use fandhe_frontend_pre_styled_ui::tag::{self, TagProps};
use fandhe_frontend_pre_styled_ui::textarea::{self, FieldIds, FieldProps, TextareaProps};
use fandhe_frontend_pre_styled_ui::timeline::{self, TimelineVariant};
use fandhe_frontend_pre_styled_ui::Size;

/// 投稿欄 textarea の `id`（本 block が出力する唯一の `id` 属性、モジュール
/// doc「`id` を 1 件のみ持つ理由」節）。
const COMPOSER_TEXTAREA_ID: &str = "blocks-feed-comments-timeline-composer-textarea";
/// 投稿欄 select（公開範囲）の listbox `id`（`aria-controls` の参照先）。
const VISIBILITY_LISTBOX_ID: &str = "blocks-feed-comments-timeline-visibility-listbox";

/// 返信 1 件分のダミーデータ（架空）。
struct Reply {
    author: &'static str,
    is_author: bool,
    body: &'static str,
}

/// イベント 1 件分のダミーデータ（状態変更・担当者割り当て・タグ付けの
/// いずれか。`tags` は空スライスなら非表示）。
struct EventData {
    /// indicator に表示する記号（`aria-hidden` の装飾文字）。
    glyph: &'static str,
    actor: &'static str,
    action: &'static str,
    tags: &'static [&'static str],
    time_iso: &'static str,
    time_label: &'static str,
}

/// コメント 1 件分のダミーデータ。
struct CommentData {
    author: &'static str,
    /// この投稿がチケットの投稿者本人によるものか（`true` のとき
    /// 「投稿者」badge を表示する、R0109 の投稿者バッジ）。
    is_author: bool,
    body: &'static str,
    time_iso: &'static str,
    time_label: &'static str,
    replies: &'static [Reply],
}

/// タイムラインの 1 行（イベントまたはコメント）。
enum Entry {
    Event(EventData),
    Comment(CommentData),
}

/// タイムラインのエントリ一覧（架空、時系列順・6 件）。イベント 4 件
/// （状態変更・担当者割り当て・タグ付けを最低 1 件ずつ）とコメント 2 件
/// （うち 1 件に返信スレッド + 投稿者バッジ）を混在させる。
const ENTRIES: [Entry; 6] = [
    Entry::Event(EventData {
        glyph: "＋",
        actor: "鈴木",
        action: "がチケットを作成しました",
        tags: &[],
        time_iso: "2026-09-20T09:00:00",
        time_label: "9月20日 9:00",
    }),
    Entry::Event(EventData {
        glyph: "→",
        actor: "鈴木",
        action: "が担当者を田中に割り当てました",
        tags: &[],
        time_iso: "2026-09-20T09:05:00",
        time_label: "9月20日 9:05",
    }),
    Entry::Comment(CommentData {
        author: "佐藤",
        is_author: true,
        body: "不具合の再現手順を追加しました。特定の条件下でのみ発生するようです。",
        time_iso: "2026-09-20T10:30:00",
        time_label: "9月20日 10:30",
        replies: &[Reply {
            author: "田中",
            is_author: false,
            body: "確認しました。手元でも再現できています。",
        }],
    }),
    Entry::Event(EventData {
        glyph: "◐",
        actor: "田中",
        action: "がステータスを「未対応」から「対応中」に変更しました",
        tags: &[],
        time_iso: "2026-09-20T11:00:00",
        time_label: "9月20日 11:00",
    }),
    Entry::Event(EventData {
        glyph: "#",
        actor: "田中",
        action: "がタグを追加しました",
        tags: &["優先度:高", "バグ"],
        time_iso: "2026-09-20T11:05:00",
        time_label: "9月20日 11:05",
    }),
    Entry::Comment(CommentData {
        author: "田中",
        is_author: false,
        body: "原因を特定しました。明日中に修正版をリリースする予定です。",
        time_iso: "2026-09-20T16:45:00",
        time_label: "9月20日 16:45",
        replies: &[],
    }),
];

/// 名前の先頭 1 文字をアバターのフォールバック表示に使う（架空の人名は
/// いずれも姓 1 文字が識別に十分なため、頭文字抽出以上の処理を持たない）。
fn initial_avatar(name: &str, size: Size, extra_attrs: Vec<(&str, &str)>) -> Node {
    let initial: String = name.chars().take(1).collect();
    avatar::root(
        &AvatarProps {
            size,
            ..AvatarProps::default()
        },
        extra_attrs,
        vec![avatar::fallback(
            ImageStatus::default(),
            vec![],
            vec![text(initial)],
        )],
    )
}

/// `<time datetime>` を組み立てる（機械可読な ISO 値と表示値は常に同じ
/// 日時を指す組にする不変条件、`changelog_timeline` と同じ判断）。
fn entry_time(iso: &str, label: &str) -> Node {
    el(
        "time",
        vec![
            ("class", "blocks-feed-comments-timeline-time"),
            ("datetime", iso),
        ],
        vec![text(label)],
    )
}

/// イベント 1 件分の `timeline::item`（完全版・簡易版 共通、モジュール
/// doc「4 参照 ID の畳み込み方」節の R0950 相当を担う）。
fn event_item(event: &EventData, is_last: bool) -> Node {
    let mut connector_children = vec![timeline::indicator(
        vec![("data-state", "complete")],
        vec![span(vec![("aria-hidden", "true")], vec![text(event.glyph)])],
    )];
    if !is_last {
        connector_children.push(timeline::separator(
            vec![("data-state", "complete")],
            vec![],
        ));
    }
    let connector = timeline::connector(vec![], connector_children);

    let mut body_children = vec![el(
        "p",
        vec![("class", "blocks-feed-comments-timeline-event-text")],
        vec![text(format!("{}{}", event.actor, event.action))],
    )];
    if !event.tags.is_empty() {
        body_children.push(div(
            vec![("class", "blocks-feed-comments-timeline-tags")],
            event
                .tags
                .iter()
                .map(|t| {
                    tag::root(
                        &TagProps::default(),
                        vec![("data-blocks-feed-comments-timeline-tag", "")],
                        vec![tag::label(vec![], vec![text(*t)])],
                    )
                })
                .collect(),
        ));
    }
    body_children.push(entry_time(event.time_iso, event.time_label));
    let body = timeline::content(
        vec![("data-blocks-feed-comments-timeline-body", "")],
        body_children,
    );

    timeline::item(vec![], vec![connector, body])
}

/// 返信 1 件分のマークアップ（アバター・名前・任意の投稿者バッジ・本文）。
fn reply_node(reply: &Reply) -> Node {
    let mut header_children = vec![span(
        vec![("class", "blocks-feed-comments-timeline-reply-name")],
        vec![text(reply.author)],
    )];
    if reply.is_author {
        header_children.push(badge::badge(
            &BadgeProps::default(),
            vec![("data-blocks-feed-comments-timeline-author-badge", "")],
            vec![text("投稿者")],
        ));
    }
    div(
        vec![("class", "blocks-feed-comments-timeline-reply")],
        vec![
            initial_avatar(
                reply.author,
                Size::Xs,
                vec![("data-blocks-feed-comments-timeline-reply-avatar", "")],
            ),
            div(
                vec![("class", "blocks-feed-comments-timeline-reply-body")],
                vec![
                    div(
                        vec![("class", "blocks-feed-comments-timeline-reply-header")],
                        header_children,
                    ),
                    el(
                        "p",
                        vec![("class", "blocks-feed-comments-timeline-reply-text")],
                        vec![text(reply.body)],
                    ),
                ],
            ),
        ],
    )
}

/// コメント 1 件分の `timeline::item`（完全版のみ。indicator にアバター、
/// content にカード形式の本文・返信スレッド・返信ボタンを置く）。
fn comment_item(comment: &CommentData, is_last: bool) -> Node {
    let mut connector_children = vec![timeline::indicator(
        vec![
            ("data-state", "complete"),
            ("data-blocks-feed-comments-timeline-comment-indicator", ""),
        ],
        vec![initial_avatar(
            comment.author,
            Size::Sm,
            vec![("data-blocks-feed-comments-timeline-avatar", "")],
        )],
    )];
    if !is_last {
        connector_children.push(timeline::separator(
            vec![("data-state", "complete")],
            vec![],
        ));
    }
    let connector = timeline::connector(vec![], connector_children);

    let mut header_children = vec![span(
        vec![("class", "blocks-feed-comments-timeline-comment-name")],
        vec![text(comment.author)],
    )];
    if comment.is_author {
        header_children.push(badge::badge(
            &BadgeProps::default(),
            vec![("data-blocks-feed-comments-timeline-author-badge", "")],
            vec![text("投稿者")],
        ));
    }
    header_children.push(entry_time(comment.time_iso, comment.time_label));

    let mut card_body_children = vec![el(
        "p",
        vec![("class", "blocks-feed-comments-timeline-comment-text")],
        vec![text(comment.body)],
    )];
    if !comment.replies.is_empty() {
        card_body_children.push(div(
            vec![("class", "blocks-feed-comments-timeline-replies")],
            comment.replies.iter().map(reply_node).collect(),
        ));
    }

    let card = card::root(
        CardProps::default(),
        vec![("data-blocks-feed-comments-timeline-card", "")],
        vec![
            card::header(
                vec![("class", "blocks-feed-comments-timeline-comment-header")],
                header_children,
            ),
            card::body(vec![], card_body_children),
            card::footer(
                vec![],
                vec![button::button(
                    &ButtonProps {
                        variant: ButtonVariant::Ghost,
                        size: Size::Sm,
                        ..ButtonProps::default()
                    },
                    vec![],
                    vec![text("返信")],
                )],
            ),
        ],
    );

    let body = timeline::content(
        vec![("data-blocks-feed-comments-timeline-body", "")],
        vec![card],
    );

    timeline::item(
        vec![("data-blocks-feed-comments-timeline-comment-item", "")],
        vec![connector, body],
    )
}

/// 完全版インスタンス（イベント・コメントを混在させる、R0949 主参照）。
fn full_instance() -> Node {
    let last_index = ENTRIES.len().saturating_sub(1);
    let items: Vec<Node> = ENTRIES
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let is_last = index == last_index;
            match entry {
                Entry::Event(event) => event_item(event, is_last),
                Entry::Comment(comment) => comment_item(comment, is_last),
            }
        })
        .collect();
    timeline::root(
        TimelineVariant::Solid,
        Size::Sm,
        fandhe_frontend_pre_styled_ui::ColorPalette::default(),
        vec![("data-blocks-feed-comments-timeline-variant", "full")],
        items,
    )
}

/// 簡易版インスタンス（イベントのみ、アイコン indicator + 本文 1 行 +
/// 日時だけの簡潔表示、R0948）。
fn compact_instance() -> Node {
    let events: Vec<&EventData> = ENTRIES
        .iter()
        .filter_map(|entry| match entry {
            Entry::Event(event) => Some(event),
            Entry::Comment(_) => None,
        })
        .collect();
    let last_index = events.len().saturating_sub(1);
    let items: Vec<Node> = events
        .iter()
        .enumerate()
        .map(|(index, event)| event_item(event, index == last_index))
        .collect();
    timeline::root(
        TimelineVariant::Outline,
        Size::Sm,
        fandhe_frontend_pre_styled_ui::ColorPalette::default(),
        vec![("data-blocks-feed-comments-timeline-variant", "compact")],
        items,
    )
}

/// 投稿欄の公開範囲 select（常に閉じた静的表示、`aria-label` のみで
/// アクセシブルネームを与え可視ラベルは置かない、モジュール doc「`id` を
/// 1 件のみ持つ理由」節）。
fn visibility_select() -> Node {
    let props = SelectProps::default();
    let options: [(&str, &str, bool); 2] =
        [("team", "チーム全員", true), ("private", "自分のみ", false)];
    let items: Vec<Node> = options
        .iter()
        .map(|(value, label, selected)| {
            let state = if *selected {
                OpenState::Open
            } else {
                OpenState::Closed
            };
            select::item(
                state,
                &props,
                false,
                false,
                value,
                None,
                vec![],
                vec![select::item_text(
                    state,
                    &props,
                    false,
                    false,
                    None,
                    vec![],
                    vec![text(*label)],
                )],
            )
        })
        .collect();
    select::root(
        Size::Sm,
        OpenState::Closed,
        &props,
        vec![("data-blocks-feed-comments-timeline-visibility", "")],
        vec![
            select::control(
                OpenState::Closed,
                &props,
                vec![],
                vec![select::trigger(
                    OpenState::Closed,
                    &props,
                    false,
                    Some(VISIBILITY_LISTBOX_ID),
                    None,
                    vec![("aria-label", "公開範囲")],
                    vec![
                        select::value_text(false, &props, vec![], vec![text("チーム全員")]),
                        select::indicator(OpenState::Closed, &props, vec![], vec![]),
                    ],
                )],
            ),
            select::positioner(
                OpenState::Closed,
                vec![],
                vec![select::content(
                    OpenState::Closed,
                    Some(VISIBILITY_LISTBOX_ID),
                    None,
                    None,
                    vec![],
                    items,
                )],
            ),
        ],
    )
}

/// コメント投稿欄（アバター・複数行入力欄・公開範囲 select・送信ボタン）。
fn composer() -> Node {
    let field = FieldProps {
        id: COMPOSER_TEXTAREA_ID,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    };
    div(
        vec![("class", "blocks-feed-comments-timeline-composer")],
        vec![
            initial_avatar(
                "あなた",
                Size::Sm,
                vec![("data-blocks-feed-comments-timeline-composer-avatar", "")],
            ),
            div(
                vec![("class", "blocks-feed-comments-timeline-composer-main")],
                vec![
                    textarea::textarea(
                        &TextareaProps::default(),
                        &field,
                        false,
                        vec![("aria-label", "コメントを入力"), ("rows", "3")],
                        vec![],
                    ),
                    div(
                        vec![("class", "blocks-feed-comments-timeline-composer-actions")],
                        vec![
                            visibility_select(),
                            button::button(
                                &ButtonProps::default(),
                                vec![],
                                vec![text("コメントする")],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    )
}

/// `feed-comments-timeline` の Demo 本体。呼び出しごとに同一の `Node` を
/// 返す純関数（モジュール doc「静的表示」節）。完全版（R0949/R0109）・
/// 簡易版（R0948）の 2 インスタンスと、完全版の下に投稿欄を縦に並べる。
pub fn demo() -> Node {
    div(
        vec![("class", "blocks-feed-comments-timeline-layout")],
        vec![full_instance(), composer(), compact_instance()],
    )
}
```

## 原案差分メモ

- R0950（状態変更・担当者割り当て・タグ付けの 3 種の出来事が混在し、投稿欄を持たない構成）は独立した 3 つ目の Demo インスタンスとしては並べていません。完全版のタイムライン部分（投稿欄を除く）自体が、この構造をそのまま含んでいます。
- R0109 の返信スレッド・投稿者バッジは、独立インスタンスにせず完全版の 1 コメントへ組み込みました。
- R0948 は簡易版インスタンスとして再現しました（アバター・カード・投稿欄は持たず、indicator の記号＋本文＋日時のみ）。
- 文言・人名・日時・タグ名はすべて独自に書いた架空のものです。
- 配色・余白・角丸は既存のテーマトークンに従っています。
- アイコンは使わず、indicator の記号は装飾用のテキスト文字（`aria-hidden`）のみで構成しています。
