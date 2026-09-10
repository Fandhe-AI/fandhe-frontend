//! `fandhe-frontend-headless-ui` の Message Scroller（[`message_scroller`]
//! モジュール、イシュー #2121）の公開 API 経由の統合テスト。
//!
//! `crates/headless-ui/src/message_scroller.rs` 内の `#[cfg(test)]`
//! ユニットテストが内部実装を含めた網羅を担うのに対し、本ファイルは
//! `fandhe-frontend-headless-ui` の公開 API（`lib.rs` の再エクスポート
//! `pub mod message_scroller;`）のみを経由し、`fandhe-frontend-pre-styled-ui`
//! （styled ラッパー、#2123）が実際に使う想定の外部からの利用形態
//! （`root` > `viewport(label)` > `content` > `message::group` >
//! `message::root` × 2、`anchor`、`jump_to_latest`、`load_more`）を固定する
//! 回帰テスト（`tests/message.rs` と同型の位置付け）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::message::{self, MessageRole, MessageRootProps};
use fandhe_frontend_headless_ui::message_scroller::{
    self, MessageScrollerRootProps, MessageScrollerStuck,
};

/// `root` > `viewport(label)` > `content` > `message::group` >
/// `message::root` × 2、`anchor`、`jump_to_latest`、`load_more` の合成で
/// 6 パーツ全ての `data-part` が出現し、`message` scope の属性が
/// `message-scroller` scope を汚染しないことを固定する。
#[test]
fn root_viewport_content_anchor_jump_to_latest_load_more_all_appear() {
    let node = message_scroller::root(
        MessageScrollerRootProps {
            stuck: MessageScrollerStuck::Free,
            has_new: true,
        },
        vec![],
        vec![
            message_scroller::viewport(
                "Conversation",
                vec![],
                vec![
                    message_scroller::content(
                        vec![],
                        vec![message::group(
                            "",
                            vec![],
                            vec![
                                message::root(
                                    MessageRootProps {
                                        role: MessageRole::User,
                                        ..Default::default()
                                    },
                                    vec![],
                                    vec![message::content(vec![], vec![text("Hi")])],
                                ),
                                message::root(
                                    MessageRootProps {
                                        role: MessageRole::Assistant,
                                        ..Default::default()
                                    },
                                    vec![],
                                    vec![message::content(vec![], vec![text("Hello!")])],
                                ),
                            ],
                        )],
                    ),
                    message_scroller::anchor(vec![]),
                ],
            ),
            message_scroller::jump_to_latest("Jump to latest", true, vec![], vec![]),
            message_scroller::load_more(false, false, vec![], vec![text("Load more")]),
        ],
    );
    let html = render(&node);

    assert!(html.contains(r#"data-scope="message-scroller""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="viewport""#));
    assert!(html.contains(r#"data-part="content""#));
    assert!(html.contains(r#"data-part="anchor""#));
    assert!(html.contains(r#"data-part="jump-to-latest""#));
    assert!(html.contains(r#"data-part="load-more""#));

    assert!(html.contains(r#"data-stuck="free""#));
    assert!(html.contains(r#"data-has-new="""#));
    assert!(html.contains(r#"tabindex="0""#));
    assert!(html.contains(r#"role="region""#));
    assert!(html.contains(r#"aria-label="Conversation""#));
    assert!(html.contains(r#"aria-hidden="true""#));
    assert!(html.contains(r#"data-visible="""#));

    // message scope の属性（role="listitem"/role="list"/data-role 等）が
    // 混入せず、message-scroller scope の外殻として汚染されないことを固定。
    assert!(html.contains(r#"data-scope="message""#));
    assert!(html.contains(r#"role="listitem""#));
    assert!(html.contains(r#"role="list""#));

    // 計測・通知系の状態は一切出現しない（本モジュールのスコープ外）。
    assert!(!html.contains("data-hydrate-"));
    assert!(!html.contains("data-autoscrolling"));
    assert!(!html.contains("data-pending-scroll"));
    assert!(!html.contains("data-scrollable"));
    assert!(!html.contains("aria-live"));
    assert!(!html.contains("aria-busy"));
    assert!(!html.contains("aria-posinset"));
    assert!(!html.contains("aria-setsize"));
}

/// `MessageScrollerRootProps::default()` は `bottom`/`has-new` なしである
/// こと（SSR の決定的な初期状態、モジュール doc「`data-stuck`/
/// `data-has-new`」参照）。
#[test]
fn root_props_default_is_bottom_without_has_new() {
    let html = render(&message_scroller::root(
        MessageScrollerRootProps::default(),
        vec![],
        vec![],
    ));
    assert!(html.contains(r#"data-stuck="bottom""#));
    assert!(!html.contains("data-has-new"));
}
