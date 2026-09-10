//! `fandhe_frontend_wasm_full::questionnaire`（イシュー #2118、親 #2117）の
//! 統合レベル native テスト。
//!
//! `wasm-full/src/questionnaire.rs` 内のインラインテストは本モジュール単体の
//! 判定関数（[`trigger_action`]/[`notification_action`]/
//! [`questionnaire_from_display_attrs`]/[`question_data_state`]/
//! [`progress_values`]/[`trigger_boundary_transition`]）と headless-ui 実
//! 出力とのドリフト検知までを検証している。本ファイルはその先、
//! これら公開 API がクレート外から使えること、[`trigger_action`] の結果を
//! [`fandhe_frontend_interactive::dispatch`] へ渡す統合経路が
//! `fandhe-frontend-headless-ui` の `Questionnaire`/`QuestionnaireAction`
//! 実装と一致することを固定する（`headless_timer.rs`〔native〕と同じ役割
//! 分担）。
//!
//! 実 DOM 経由の検証（trigger click → dispatch → `data-state`/`hidden`/
//! progress/trigger disabled 反映）は
//! `wasm-full/tests/questionnaire_browser.rs` が担当する。

use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::questionnaire::Questionnaire;
use fandhe_frontend_interactive::dispatch;
use fandhe_frontend_wasm_full::questionnaire::{
    notification_action, progress_values, question_data_state, questionnaire_from_display_attrs,
    trigger_action, trigger_boundary_transition, ACTION_NEXT, ACTION_PREV, ACTION_SKIP,
};

// --- 公開 API がクレート外から使えること ---------------------------------

#[test]
fn public_constants_are_reachable_from_outside_the_crate() {
    assert_eq!(ACTION_PREV, "questionnaire:prev");
    assert_eq!(ACTION_NEXT, "questionnaire:next");
    assert_eq!(ACTION_SKIP, "questionnaire:skip");
}

// --- trigger_action → dispatch 統合経路が headless-ui の Questionnaire と
// 一致すること ------------------------------------------------------------

#[test]
fn next_trigger_action_dispatch_advances_step() {
    let mut q = Questionnaire::new(3, 0, Orientation::Horizontal);
    let action = trigger_action(Some("questionnaire"), Some("next")).expect("next must resolve");
    assert!(dispatch(&mut q, action, ""));
    assert_eq!(q.step(), 1);
}

#[test]
fn prev_trigger_action_dispatch_retreats_step() {
    let mut q = Questionnaire::new(3, 1, Orientation::Horizontal);
    let action = trigger_action(Some("questionnaire"), Some("back")).expect("back must resolve");
    assert!(dispatch(&mut q, action, ""));
    assert_eq!(q.step(), 0);
}

#[test]
fn skip_trigger_action_dispatch_advances_step_same_as_next() {
    let mut q = Questionnaire::new(3, 1, Orientation::Horizontal);
    let action = trigger_action(Some("questionnaire"), Some("skip")).expect("skip must resolve");
    assert!(dispatch(&mut q, action, ""));
    assert_eq!(q.step(), 2);
}

#[test]
fn next_at_boundary_is_a_dispatch_noop_transition() {
    let mut q = Questionnaire::new(2, 2, Orientation::Horizontal);
    let before = q;
    let action = trigger_action(Some("questionnaire"), Some("next")).expect("next must resolve");
    // headless-ui の `decode_action` は境界であっても `Some` を返すため
    // dispatch 自体は成功する（`true`）が、状態は変化しない
    // （モジュール冒頭「アプリ状態 `C` への通知」節の no-op 判定根拠）。
    assert!(dispatch(&mut q, action, ""));
    assert_eq!(before, q);
}

// --- notification_action と trigger_action の組み合わせが一貫すること ---

#[test]
fn notification_action_covers_every_trigger_action_output() {
    for part in ["back", "next", "skip"] {
        let action = trigger_action(Some("questionnaire"), Some(part))
            .unwrap_or_else(|| panic!("{part} must resolve to a dispatch action"));
        assert!(
            notification_action(action).is_some(),
            "{action} must map to a notification action"
        );
    }
}

// --- questionnaire_from_display_attrs → 実際の Questionnaire::new 一致 ---

#[test]
fn from_display_attrs_matches_questionnaire_new_for_valid_input() {
    let restored =
        questionnaire_from_display_attrs(Some("2"), Some("vertical"), 5).expect("must reconstruct");
    let expected = Questionnaire::new(5, 2, Orientation::Vertical);
    assert_eq!(restored.count(), expected.count());
    assert_eq!(restored.step(), expected.step());
    assert_eq!(restored.orientation(), expected.orientation());
}

// --- question_data_state / progress_values / trigger_boundary_transition
// の統合的な一貫性（3 状態の総和が count に一致すること） -----------------

#[test]
fn question_data_state_partitions_all_indices_exhaustively() {
    let count = 5;
    let step = 2;
    let mut completed = 0;
    let mut active = 0;
    let mut upcoming = 0;
    for index in 0..count {
        match question_data_state(step, index) {
            "completed" => completed += 1,
            "active" => active += 1,
            "upcoming" => upcoming += 1,
            other => panic!("unexpected data-state value: {other}"),
        }
    }
    assert_eq!(completed, step);
    assert_eq!(active, 1);
    assert_eq!(upcoming, count - step - 1);
}

#[test]
fn progress_values_and_boundary_transition_agree_on_completion_edge() {
    let before = Questionnaire::new(4, 3, Orientation::Horizontal);
    let after = Questionnaire::new(4, 4, Orientation::Horizontal);
    let (_now, _text, complete) = progress_values(&after);
    let boundary = trigger_boundary_transition(&before, &after);
    assert!(complete);
    assert_eq!(boundary.next_skip, Some(true));
}
