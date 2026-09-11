//! `fandhe_frontend_wasm_full::message_scroller`（イシュー #2122、親 #2120）の
//! headless-ui 出力とのドリフト検知テスト。
//!
//! `crates/headless-ui/src/message_scroller.rs` の anatomy 出力
//! （`data-scope`/`data-part`）と本クレートの [`SCOPE`]/`PART_*` 定数が
//! 一致していること、[`MessageScrollerStuck`] の語彙と headless-ui 側の
//! `data-stuck` 出力・[`stuck_from_attr`] が往復すること、
//! `jump_to_latest` の `hidden`/`data-visible` 排他契約が本クレートの
//! [`jump_visible`] 判定と一致することを固定する。
//!
//! 実 DOM 経由の検証（scroll/click/MutationObserver 配線）は
//! `wasm-full/tests/message_scroller_browser.rs` が担当する。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::message_scroller::{
    content, jump_to_latest, load_more, root, viewport, MessageScrollerRootProps,
    MessageScrollerStuck as HeadlessStuck,
};
use fandhe_frontend_wasm_full::message_scroller::{
    jump_visible, stuck_from_attr, MessageScrollerStuck, ACTION_LOAD_MORE, PART_CONTENT,
    PART_JUMP_TO_LATEST, PART_LOAD_MORE, PART_ROOT, PART_VIEWPORT, SCOPE,
};

#[test]
fn scope_and_part_constants_match_headless_ui_output() {
    let node = root(MessageScrollerRootProps::default(), vec![], vec![]);
    let html = render(&node);
    assert!(html.contains(&format!(r#"data-scope="{SCOPE}""#)));
    assert!(html.contains(&format!(r#"data-part="{PART_ROOT}""#)));

    let html = render(&viewport("", vec![], vec![]));
    assert!(html.contains(&format!(r#"data-part="{PART_VIEWPORT}""#)));

    let html = render(&content(vec![], vec![]));
    assert!(html.contains(&format!(r#"data-part="{PART_CONTENT}""#)));

    let html = render(&jump_to_latest("", true, vec![], vec![]));
    assert!(html.contains(&format!(r#"data-part="{PART_JUMP_TO_LATEST}""#)));

    let html = render(&load_more(false, false, vec![], vec![]));
    assert!(html.contains(&format!(r#"data-part="{PART_LOAD_MORE}""#)));
}

#[test]
fn stuck_vocabulary_round_trips_with_headless_ui() {
    for (headless, expected) in [
        (HeadlessStuck::Bottom, MessageScrollerStuck::Bottom),
        (HeadlessStuck::Free, MessageScrollerStuck::Free),
    ] {
        let node = root(
            MessageScrollerRootProps {
                stuck: headless,
                has_new: false,
            },
            vec![],
            vec![],
        );
        let html = render(&node);
        let attr_start = html.find(r#"data-stuck=""#).expect("data-stuck present") + 12;
        let attr_value = &html[attr_start..];
        let attr_value = &attr_value[..attr_value.find('"').expect("closing quote")];
        assert_eq!(stuck_from_attr(Some(attr_value)), expected);
    }
}

#[test]
fn stuck_from_attr_unknown_value_is_fail_closed_free() {
    // headless-ui 側は `bottom`/`free` の 2 値しか出力しないため、改ざん・
    // 欠落を模した値を直接 `stuck_from_attr` へ渡して fail-closed 挙動を
    // 固定する（モジュール doc「最下部判定」参照）。
    assert_eq!(stuck_from_attr(Some("evil")), MessageScrollerStuck::Free);
    assert_eq!(stuck_from_attr(None), MessageScrollerStuck::Free);
}

#[test]
fn jump_to_latest_visibility_exclusive_contract_matches_jump_visible() {
    // headless-ui `jump_to_latest(_, false, ...)` は `hidden` を持ち
    // `data-visible` を持たない。`jump_to_latest(_, true, ...)` はその逆。
    // 配線層 `jump_visible` の判定と同じ排他契約であることを固定する。
    let hidden_html = render(&jump_to_latest("", false, vec![], vec![]));
    assert!(hidden_html.contains("hidden"));
    assert!(!hidden_html.contains("data-visible"));
    assert!(!jump_visible(MessageScrollerStuck::Bottom));

    let visible_html = render(&jump_to_latest("", true, vec![], vec![]));
    assert!(visible_html.contains(r#"data-visible="""#));
    assert!(!visible_html.contains("hidden"));
    assert!(jump_visible(MessageScrollerStuck::Free));
}

#[test]
fn action_load_more_is_namespaced_like_other_wasm_full_modules() {
    // `questionnaire:*`/`timer:*` と同じ命名規約であることを固定する。
    assert_eq!(ACTION_LOAD_MORE, "message-scroller:load-more");
    assert!(ACTION_LOAD_MORE.starts_with("message-scroller:"));
}
