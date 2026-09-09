//! `fandhe-frontend-headless-ui` の Message（[`message`] モジュール、イシュー
//! #2105）の公開 API 経由の統合テスト。
//!
//! `crates/headless-ui/src/message.rs` 内の `#[cfg(test)]` ユニットテストが
//! 内部実装を含めた網羅を担うのに対し、本ファイルは
//! `fandhe-frontend-headless-ui` の公開 API（`lib.rs` の再エクスポート
//! `pub mod message;`）のみを経由し、`fandhe-frontend-pre-styled-ui`
//! （styled ラッパー、#2106）が実際に使う想定の外部からの利用形態
//! （group > root × 3（user/assistant/system、align 違い、loading/error
//! 表示）+ avatar スロットへの [`mod@fandhe_frontend_headless_ui::avatar`]
//! 埋め込み）を固定する回帰テスト（`tests/item.rs` と同型の位置付け）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::avatar::{self, ImageStatus};
use fandhe_frontend_headless_ui::message::{self, MessageAlign, MessageRole, MessageRootProps};

/// `group` > `root`×3（user/assistant/system、align start/end、loading、
/// error）で 6 パーツ全ての `data-part` が出現し、`role="list"`/
/// `role="listitem"` が出現し、`aria-live`/`aria-busy`/`data-hydrate-` が
/// 一切出現しないことを固定する。
#[test]
fn group_with_three_roles_has_all_six_parts_and_no_notification_attrs() {
    let node = message::group(
        "Conversation",
        vec![],
        vec![
            message::root(
                MessageRootProps {
                    role: MessageRole::User,
                    align: MessageAlign::End,
                    ..Default::default()
                },
                vec![],
                vec![
                    message::avatar(vec![], vec![text("U")]),
                    message::header(vec![], vec![text("You")]),
                    message::content(vec![], vec![text("Hello")]),
                    message::footer(vec![], vec![text("just now")]),
                ],
            ),
            message::root(
                MessageRootProps {
                    role: MessageRole::Assistant,
                    align: MessageAlign::Start,
                    loading: true,
                    ..Default::default()
                },
                vec![],
                vec![message::content(vec![], vec![text("Thinking...")])],
            ),
            message::root(
                MessageRootProps {
                    role: MessageRole::System,
                    align: MessageAlign::Start,
                    error: true,
                    ..Default::default()
                },
                vec![],
                vec![message::content(vec![], vec![text("Failed to send")])],
            ),
        ],
    );
    let html = render(&node);

    assert!(html.contains(r#"data-scope="message""#));
    for part in ["root", "avatar", "header", "content", "footer", "group"] {
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "part={part} が出力されていない: html={html}"
        );
    }

    assert!(html.contains(r#"role="list""#));
    assert!(html.contains(r#"role="listitem""#));
    assert!(html.contains(r#"data-role="user""#));
    assert!(html.contains(r#"data-role="assistant""#));
    assert!(html.contains(r#"data-role="system""#));
    assert!(html.contains(r#"data-align="end""#));
    assert!(html.contains(r#"data-align="start""#));
    assert!(html.contains(r#"data-loading="""#));
    assert!(html.contains(r#"data-error="""#));

    // 通知系属性（アプリ責務）は一切出力されない不変条件（モジュール doc
    // 「`aria-live`/`aria-busy` を付けない理由」参照）。
    assert!(!html.contains("aria-live"));
    assert!(!html.contains("aria-busy"));

    // 状態機械を持たないため hydration 属性は一切出力されない。
    assert!(!html.contains("data-hydrate-"));
}

/// `avatar` スロットへ既存 `avatar` モジュールを埋め込んでも
/// `data-scope="message"` 側の属性が汚染されないことを固定する
/// （モジュール doc「`avatar` はスロット」参照。他 scope を内包しない）。
#[test]
fn avatar_slot_can_embed_existing_avatar_module_without_scope_pollution() {
    let node = message::root(
        MessageRootProps::default(),
        vec![],
        vec![message::avatar(
            vec![],
            vec![avatar::image(
                ImageStatus::Loaded,
                "/avatar.png",
                "User",
                vec![],
            )],
        )],
    );
    let html = render(&node);

    assert!(html.contains(r#"data-scope="message""#));
    assert!(html.contains(r#"data-part="avatar""#));
    assert!(html.contains(r#"data-scope="avatar""#));
    assert!(html.contains(r#"data-part="image""#));
    assert!(html.contains(r#"src="/avatar.png""#));
}

/// `MessageRootProps::default()` が `role="listitem"`・`data-role="user"`・
/// `data-align="start"` になることを公開 API 経由で固定する。
#[test]
fn default_root_props_render_listitem_user_start() {
    let html = render(&message::root(MessageRootProps::default(), vec![], vec![]));
    assert!(html.starts_with("<div"));
    assert!(html.contains(r#"role="listitem""#));
    assert!(html.contains(r#"data-role="user""#));
    assert!(html.contains(r#"data-align="start""#));
}
