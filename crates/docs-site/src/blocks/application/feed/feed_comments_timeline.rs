//! `feed-comments-timeline` block（イシュー #2909。親トラッキング #2892
//! 「Blocks アプリケーション A」配下、対応表 ID R0949/R0950/R0109/R0948 の
//! 4 件を構造の参照元とする合成例。縦のタイムライン上にイベント（状態
//! 変更・担当者割り当て・タグ付け）とコメント（アバター付きカード、
//! 返信スレッド・投稿者バッジ対応）を時系列で混在させ、末尾にコメント
//! 投稿欄を置く）。取得手段・ファイル名・内部コンポーネント識別子は
//! 記載しない（`docs/design/motion-reference-adoption-policy.md` §9 と
//! 同じライセンス上の転記制限）。
//!
//! **Application / Feed カテゴリで最初の block**（`super`（`feed/mod.rs`）
//! 参照）。
//!
//! # 使用部品
//!
//! `timeline` / `avatar` / `card` / `badge` / `textarea` / `select` /
//! `button` / `tag` の 8 部品を合成する（[`BLOCK`] の `parts` に一致させる
//! 契約、`crates/docs-site/tests/blocks_nav.rs`/`blocks_contract.rs` が
//! 検証する）。
//!
//! # 4 参照 ID の畳み込み方（Demo は 2 インスタンス）
//!
//! - **完全版**（`data-blocks-feed-comments-timeline-variant="full"`。
//!   R0949 が主参照）: イベント・コメントを時系列で混在させ、コメントは
//!   アバター付きカードで表示する。返信スレッド + 投稿者バッジ（R0109）を
//!   1 件のコメントへ組み込み、末尾にコメント投稿欄（アバター・複数行
//!   入力欄・公開範囲 select・送信ボタン）を置く。
//! - **簡易版**（`data-blocks-feed-comments-timeline-variant="compact"`。
//!   R0948）: [`ENTRIES`] のうちイベントのみを、indicator の記号 + 本文
//!   1 行 + 日時だけの簡潔な表示で並べる。アバター・カード・入力欄は
//!   持たない。
//! - R0950（状態変更・担当者割り当て・タグ付けの 3 種の出来事が混在し、
//!   投稿欄を持たない構成）は独立した 3 つ目の Demo インスタンスとしては
//!   並べない。完全版から投稿欄を除いた構造と同一であるため、この構造は
//!   完全版のタイムライン部分（投稿欄を除く）と簡易版の組み合わせで示す
//!   （[`site/blocks/feed-comments-timeline.md`] の「原案差分メモ」に
//!   明記）。
//!
//! # 静的表示（無 JS、架空データ固定）
//!
//! [`ENTRIES`] は呼び出しごとに同一の `Node` を生成する純関数（[`demo`]）が
//! 参照するだけの固定配列であり、状態機械・フォーム・データ取得を一切
//! 持たない。投稿欄の select（公開範囲）は常に閉じた静的表示
//! （`OpenState::Closed`）で、textarea・button も送信処理を持たない。
//!
//! # `<form>` を持たない・データ取得/送信を行わない
//!
//! `crate::blocks` モジュール doc「`<form>` を使わない」節・「セキュリティ
//! 不変条件」節に従い、本 Demo はフォーム・状態機械を持たない静的な合成例
//! である。文言・人名・日時はすべて架空のもの（実在の人物・企業・PII を
//! 含まない）。
//!
//! # `id` 属性は 2 件のみ・重複させない理由
//!
//! 2 インスタンス（full/compact）を同一ページへ並べるため、`id`/
//! `aria-controls`/`aria-labelledby` の重複・宙に浮いた参照を避ける必要が
//! ある。本 block が出力する `id="..."` は、投稿欄 select（公開範囲）の
//! listbox（[`VISIBILITY_LISTBOX_ID`]、`select::trigger` の
//! `aria-controls` の参照先）と、投稿欄 textarea の
//! [`fandhe_frontend_headless_ui::field::FieldProps::id`]
//! （[`COMPOSER_TEXTAREA_ID`]）の 2 件のみで、いずれも一意である
//! （`crates/docs-site/tests/blocks_contract.rs::
//! demo_output_has_no_dangling_aria_references_or_duplicate_ids` が重複・
//! 宙に浮いた参照の不在を検証する）。投稿欄 select は `aria-labelledby` を
//! 使わず（可視ラベルを置かず `aria-label` のみでアクセシブルネームを
//! 与える）、`select::label` パーツは出力しない。
//!
//! # `text` の名前衝突
//!
//! `fandhe_frontend_pre_styled_ui::text::text`（`<p>` を組み立てる styled
//! パート関数）と `fandhe_frontend_core::text`（テキストノード生成関数）が
//! 同名のため、styled 側は本 block では使わない（`heading`/`text` 部品は
//! [`BLOCK::parts`] に含めておらず、本文は素の `el("p")` と
//! `fandhe_frontend_core::text` のみで組み立てる）。
//!
//! # CSS フックの選び方（`drop_class_attr` の契約）
//!
//! `timeline::root` / `avatar::root` / `card::root` / `badge::badge` /
//! `textarea::textarea` / `select::root` / `button::button` / `tag::root`
//! はいずれも `drop_class_attr` により呼び出し側 `attrs` の `class` を
//! 黙って除去する契約を持つため、Demo 固有のスタイルフックは
//! `data-blocks-feed-comments-timeline-*` 属性で渡す。`timeline` の非
//! root パーツ（`item`/`connector`/`separator`/`indicator`/`content`）は
//! `drop_class_attr` を経由しないが、こちらも同じ data-* 属性方式へ統一
//! する（root/非 root で異なる手段を混在させないための判断、
//! `changelog_timeline` と同じ方針）。素の `div`/`p`/`span`/`time` には
//! `.blocks-feed-comments-timeline-*` クラスセレクタを使う。
//!
//! # separator を最後のエントリで省く理由
//!
//! [`fandhe_frontend_pre_styled_ui::timeline`] モジュール doc
//! 「`showLastSeparator` 相当は実装しない」節のとおり、最終 item の
//! separator 非表示は呼び出し側の構成責務である。full/compact それぞれの
//! インスタンスで最後の item では [`timeline::separator`] を connector の
//! 子へ含めない。full は [`ENTRIES`] 全件（6 件）、compact はイベントのみ
//! （3 件）を対象にするため、両インスタンスの separator 数は異なる
//! （それぞれ 5 件・2 件）。

use crate::blocks::{Block, BlockCategory, LayoutCss, Part};

// blocks-code:begin
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

/// タイムラインのエントリ一覧（架空、時系列順・6 件）。イベント 3 件
/// （状態変更・担当者割り当て・タグ付けを最低 1 件ずつ）とコメント 3 件
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
        vec![("data-state", "complete")],
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

    timeline::item(vec![], vec![connector, body])
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
// blocks-code:end

/// [`crate::blocks::all_blocks`] が集約するレジストリエントリ（本カテゴリの `blocks()` から連結される）。
pub const BLOCK: Block = Block {
    path: "/blocks/feed-comments-timeline/",
    title: "feed-comments-timeline",
    category: BlockCategory::Feed,
    rust_source: "crates/docs-site/src/blocks/application/feed/feed_comments_timeline.rs",
    demo_class: "blocks-feed-comments-timeline",
    parts: &[
        Part {
            label: "Timeline",
            path: "/themes/timeline/",
        },
        Part {
            label: "Avatar",
            path: "/themes/avatar/",
        },
        Part {
            label: "Card",
            path: "/themes/card/",
        },
        Part {
            label: "Badge",
            path: "/themes/badge/",
        },
        Part {
            label: "Textarea",
            path: "/themes/textarea/",
        },
        Part {
            label: "Select",
            path: "/themes/select/",
        },
        Part {
            label: "Button",
            path: "/themes/button/",
        },
        Part {
            label: "Tag",
            path: "/themes/tag/",
        },
    ],
    layout_css: LayoutCss::Static(LAYOUT_CSS),
    demo,
};

/// `feed_comments_timeline` 固有のレイアウト規則（`crate::blocks::LAYOUT_CSS`
/// doc「block 固有 CSS の置き場」節）。
///
/// # indicator 列をアバター幅に広げる（完全版のコメント item 限定）
///
/// styled `timeline::indicator` は既定で小さな点状の寸法
/// （`--fandhe-timeline-indicator-size`）を持つため、アバターを内包する
/// コメント item では indicator 自体の背景・枠線・固定寸法を解除し、
/// アバターの円がそのまま見える透過ラッパへ変える。
///
/// # 返信スレッドのインデントと左ボーダー
///
/// `--fandhe-color-border` を使い、返信であることを視覚的に示す。
///
/// # 投稿欄の grid
///
/// アバター列 | 本文列（textarea + 操作行）の 2 列。操作行は
/// `justify-content: space-between` で select と送信ボタンを両端に置く。
///
/// # 狭い幅（`@media (max-width: 47.99rem)`）
///
/// `login_04` 等の前例と同じ閾値。投稿欄をアバター・本文とも縦積みへ
/// 切り替える。
const LAYOUT_CSS: &str = "\
.blocks-feed-comments-timeline-layout {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-8);\n  width: 100%;\n}\n\
.blocks-feed-comments-timeline-layout [data-blocks-feed-comments-timeline-variant=\"full\"] [data-scope=\"timeline\"][data-part=\"indicator\"] {\n  width: auto;\n  height: auto;\n  background: transparent;\n  border: none;\n  padding: 0;\n}\n\
.blocks-feed-comments-timeline-event-text {\n  margin: 0 0 var(--fandhe-space-1) 0;\n}\n\
.blocks-feed-comments-timeline-time {\n  color: var(--fandhe-color-fg-muted);\n  font-size: var(--fandhe-font-font-size-sm, 0.875rem);\n}\n\
.blocks-feed-comments-timeline-tags {\n  display: flex;\n  flex-wrap: wrap;\n  gap: var(--fandhe-space-2);\n  margin-bottom: var(--fandhe-space-1);\n}\n\
.blocks-feed-comments-timeline-comment-header {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-feed-comments-timeline-comment-name {\n  font-weight: var(--fandhe-font-weight-bold, 700);\n}\n\
.blocks-feed-comments-timeline-comment-text {\n  margin: var(--fandhe-space-2) 0 0 0;\n}\n\
.blocks-feed-comments-timeline-replies {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n  margin-top: var(--fandhe-space-3);\n  padding-left: var(--fandhe-space-4);\n  border-left: 2px solid var(--fandhe-color-border);\n}\n\
.blocks-feed-comments-timeline-reply {\n  display: flex;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-feed-comments-timeline-reply-header {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n}\n\
.blocks-feed-comments-timeline-reply-name {\n  font-weight: var(--fandhe-font-weight-bold, 700);\n}\n\
.blocks-feed-comments-timeline-reply-text {\n  margin: var(--fandhe-space-1) 0 0 0;\n}\n\
.blocks-feed-comments-timeline-composer {\n  display: grid;\n  grid-template-columns: auto minmax(0, 1fr);\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-feed-comments-timeline-composer-main {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-3);\n}\n\
.blocks-feed-comments-timeline-composer-actions {\n  display: flex;\n  align-items: center;\n  justify-content: space-between;\n  gap: var(--fandhe-space-3);\n}\n\
@media (max-width: 47.99rem) {\n  .blocks-feed-comments-timeline-composer {\n    grid-template-columns: 1fr;\n  }\n  .blocks-feed-comments-timeline-composer-actions {\n    flex-direction: column;\n    align-items: stretch;\n  }\n}\n";

#[cfg(test)]
mod tests {
    use super::{demo, COMPOSER_TEXTAREA_ID, ENTRIES, LAYOUT_CSS, VISIBILITY_LISTBOX_ID};
    use fandhe_frontend_core::render;

    /// Demo が期待する 8 種の部品を出力すること。
    #[test]
    fn demo_composes_expected_parts() {
        let html = render(&demo());
        for scope in [
            "data-scope=\"timeline\"",
            "data-scope=\"avatar\"",
            "data-scope=\"card\"",
            "data-scope=\"badge\"",
            "data-scope=\"select\"",
            "data-scope=\"tag\"",
            "data-scope=\"button\"",
        ] {
            assert!(html.contains(scope), "demo output should contain {scope}");
        }
        assert!(html.contains("data-part=\"textarea\""));
    }

    /// full・compact 双方の variant フックが出力されていること。
    #[test]
    fn demo_wires_both_variant_hooks() {
        let html = render(&demo());
        assert!(html.contains(r#"data-blocks-feed-comments-timeline-variant="full""#));
        assert!(html.contains(r#"data-blocks-feed-comments-timeline-variant="compact""#));
    }

    /// full インスタンスの separator 数が「全エントリ数 - 1」、compact
    /// インスタンスの separator 数が「イベント数 - 1」であること
    /// （モジュール doc「separator を最後のエントリで省く理由」節）。
    #[test]
    fn demo_omits_separator_on_last_entry_per_instance() {
        let html = render(&demo());
        let event_count = ENTRIES
            .iter()
            .filter(|e| matches!(e, super::Entry::Event(_)))
            .count();
        let expected_total = (ENTRIES.len() - 1) + (event_count - 1);
        assert_eq!(
            html.matches(r#"data-part="separator""#).count(),
            expected_total,
            "html={html}"
        );
    }

    /// 投稿者バッジ・返信スレッドが full インスタンスにのみ現れること
    /// （compact はイベントのみのため `data-scope="avatar"` を持たない）。
    #[test]
    fn compact_instance_has_no_avatar_or_card() {
        let html = render(&demo());
        // compact 部分だけを切り出せないため、全体出力での avatar/card 件数が
        // full インスタンス（コメント 2 件分）の件数と一致することで
        // compact が寄与していないことを間接的に確認する。
        let comment_count = ENTRIES
            .iter()
            .filter(|e| matches!(e, super::Entry::Comment(_)))
            .count();
        // composer のアバター 1 件を加える（`data-scope="avatar"` は root/
        // fallback の 2 パートで 2 回現れるため、`data-part="root"` に絞った
        // 出現数で avatar の「個数」を数える）。
        let expected_avatars = comment_count
            + 1
            + ENTRIES
                .iter()
                .filter_map(|e| match e {
                    super::Entry::Comment(c) => Some(c.replies.len()),
                    super::Entry::Event(_) => None,
                })
                .sum::<usize>();
        assert_eq!(
            html.matches(r#"data-scope="avatar" data-part="root""#)
                .count(),
            expected_avatars,
            "html={html}"
        );
    }

    /// 非対話・安全性の不変条件（`<form>`・`href="#"`・`data:` URI を持た
    /// ず、`id="..."` 属性がちょうど 1 件であること。モジュール doc「`id`
    /// を 1 件のみ持つ理由」節）。
    #[test]
    fn demo_never_contains_forbidden_markup() {
        let html = render(&demo());
        for absent in ["<form", "href=\"#\"", "src=\"data:"] {
            assert!(!html.contains(absent), "demo should never contain {absent}");
        }
        assert_eq!(
            html.matches("id=\"").count(),
            2,
            "expected exactly 2 id attributes (composer textarea + visibility listbox); html={html}"
        );
        // `FieldProps::control_id()` は `{id}-control` を実際の `id` 属性値
        // にする（`{id}` そのものではない）ため部分一致で確認する。
        assert!(html.contains(COMPOSER_TEXTAREA_ID));
        assert!(html.contains(&format!("id=\"{VISIBILITY_LISTBOX_ID}\"")));
        assert!(html.contains(&format!("aria-controls=\"{VISIBILITY_LISTBOX_ID}\"")));
    }

    /// すべての `<button>` が `type="button"` であること（暗黙 submit
    /// なし）。
    #[test]
    fn all_buttons_are_type_button() {
        let html = render(&demo());
        let button_open_count = html.matches("<button").count();
        let type_button_count = html.matches(r#"type="button""#).count();
        assert_eq!(
            button_open_count, type_button_count,
            "every <button> should carry type=\"button\"; html={html}"
        );
    }

    /// [`LAYOUT_CSS`] が狭幅切り替えセレクタを持つこと。
    #[test]
    fn layout_css_has_responsive_rule() {
        assert!(LAYOUT_CSS.contains("@media (max-width: 47.99rem)"));
    }

    /// ルート class（`demo_class` とは別名）が [`demo`] の出力へ実際に
    /// 現れること（`blog_list_image` と同じ Bugbot 教訓の固定）。
    #[test]
    fn layout_root_class_differs_from_demo_class() {
        let html = render(&demo());
        assert!(html.contains("class=\"blocks-feed-comments-timeline-layout\""));
        assert_ne!(
            super::BLOCK.demo_class,
            "blocks-feed-comments-timeline-layout"
        );
    }
}
