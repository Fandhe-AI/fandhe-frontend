//! `fandhe-frontend-headless-ui` の Bubble（[`bubble`] モジュール、イシュー
//! #2108）の公開 API 経由の統合テスト。
//!
//! `crates/headless-ui/src/bubble.rs` 内の `#[cfg(test)]` ユニットテストが
//! 内部実装を含めた網羅を担うのに対し、本ファイルは
//! `fandhe-frontend-headless-ui` の公開 API（`lib.rs` の再エクスポート
//! `pub mod bubble;`）のみを経由し、`fandhe-frontend-pre-styled-ui`
//! （styled ラッパー、#2109）が実際に使う想定の外部からの利用形態
//! （variant/align/group-position の組み合わせ + reactions > reaction
//! （selected あり/なし）+ collapse-trigger/collapse-content の開閉、
//! `message::content` スロットへの入れ子）を固定する回帰テスト
//! （`tests/message.rs` と同型の位置付け）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::bubble::{
    self, BubbleGroupPosition, BubbleRootProps, BubbleVariant,
};
use fandhe_frontend_headless_ui::message::{self, MessageAlign};
use fandhe_frontend_headless_ui::state::OpenState;

/// root（variant×3・align×2・group-position×4 のうち代表値）+ content +
/// reactions > reaction（selected あり/なし）+ collapse-trigger/
/// collapse-content で、6 パーツ全ての `data-part` が出現し、全語彙値が
/// 出現し、`data-hydrate-` が一切出現しないことを固定する。
#[test]
fn root_with_all_parts_has_all_six_parts_and_full_vocabulary() {
    let node = bubble::root(
        BubbleRootProps {
            variant: BubbleVariant::Outline,
            align: MessageAlign::End,
            group_position: BubbleGroupPosition::First,
        },
        vec![],
        vec![
            bubble::content(vec![], vec![text("Hello there")]),
            bubble::reactions(
                "2 reactions",
                vec![],
                vec![
                    bubble::reaction(true, vec![], vec![text("👍")]),
                    bubble::reaction(false, vec![], vec![text("❤")]),
                ],
            ),
            bubble::collapse_trigger(
                OpenState::Closed,
                Some("bubble-detail-1"),
                vec![],
                vec![text("Show details")],
            ),
            bubble::collapse_content(
                OpenState::Closed,
                Some("bubble-detail-1"),
                vec![],
                vec![text("Sent at 10:00")],
            ),
        ],
    );
    let html = render(&node);

    assert!(html.contains(r#"data-scope="bubble""#));
    for part in [
        "root",
        "content",
        "reactions",
        "reaction",
        "collapse-trigger",
        "collapse-content",
    ] {
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "part={part} が出力されていない: html={html}"
        );
    }

    assert!(html.contains(r#"data-variant="outline""#));
    assert!(html.contains(r#"data-align="end""#));
    assert!(html.contains(r#"data-group-position="first""#));
    assert!(html.contains(r#"role="group""#));
    assert!(html.contains(r#"aria-label="2 reactions""#));
    assert!(html.contains(r#"data-selected="""#));
    assert!(html.contains(r#"aria-controls="bubble-detail-1""#));
    assert!(html.contains(r#"aria-expanded="false""#));
    assert!(html.contains(r#"data-state="closed""#));
    assert!(html.contains("hidden"));
    assert!(html.contains(r#"id="bubble-detail-1""#));

    // 状態機械を持たないため hydration 属性は一切出力されない。
    assert!(!html.contains("data-hydrate-"));
}

/// `collapse-trigger`/`collapse-content` が open のとき `aria-expanded`/
/// `data-state` が同期し、`hidden` が出力されないことを固定する。
#[test]
fn collapse_open_state_syncs_trigger_and_content() {
    let trigger = render(&bubble::collapse_trigger(
        OpenState::Open,
        Some("d1"),
        vec![],
        vec![],
    ));
    assert!(trigger.contains(r#"aria-expanded="true""#));
    assert!(trigger.contains(r#"data-state="open""#));

    let content = render(&bubble::collapse_content(
        OpenState::Open,
        Some("d1"),
        vec![],
        vec![text("detail")],
    ));
    assert!(content.contains(r#"data-state="open""#));
    assert!(!content.contains("hidden"));
    assert!(content.contains("detail"));
}

/// `BubbleRootProps::default()` が `solid`/`start`/`single` になることを
/// 公開 API 経由で固定する。
#[test]
fn default_root_props_render_solid_start_single() {
    let html = render(&bubble::root(BubbleRootProps::default(), vec![], vec![]));
    assert!(html.starts_with("<div"));
    assert!(html.contains(r#"data-variant="solid""#));
    assert!(html.contains(r#"data-align="start""#));
    assert!(html.contains(r#"data-group-position="single""#));
}

/// `message::content` スロット内へ複数の `bubble::root`（連続発言）を
/// 入れ子にしても、それぞれの scope が独立して固定されることを固定する
/// （会話 1 発言の中に複数の吹き出しを並べる想定の回帰）。
#[test]
fn nesting_multiple_bubbles_inside_message_content_keeps_scopes_independent() {
    let node = message::content(
        vec![],
        vec![
            bubble::root(
                BubbleRootProps {
                    group_position: BubbleGroupPosition::First,
                    ..Default::default()
                },
                vec![],
                vec![bubble::content(vec![], vec![text("first")])],
            ),
            bubble::root(
                BubbleRootProps {
                    group_position: BubbleGroupPosition::Last,
                    ..Default::default()
                },
                vec![],
                vec![bubble::content(vec![], vec![text("last")])],
            ),
        ],
    );
    let html = render(&node);

    assert!(html.contains(r#"data-scope="message""#));
    assert!(html.contains(r#"data-scope="bubble""#));
    assert!(html.contains(r#"data-group-position="first""#));
    assert!(html.contains(r#"data-group-position="last""#));
}
