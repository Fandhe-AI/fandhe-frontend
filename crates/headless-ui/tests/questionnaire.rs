//! `questionnaire::*`/`Questionnaire`（イシュー #2117）の公開 API 経由
//! 統合テスト。
//!
//! クレートルート・`questionnaire` モジュールからの re-export が実際に
//! 使えることを確認したうえで、全 11 パーツの `data-part` 網羅・
//! `options`/`freeform` スロットへの [`radio_group`]/[`field::textarea`]
//! 入れ子と scope 独立・完了状態の横断挙動・hydration ラウンドトリップ・
//! dispatch 統合を固定する。値ごとの詳細な属性検証は
//! `crates/headless-ui/src/questionnaire.rs` 側のユニットテストに置き、
//! 本ファイルは「公開 API 経由で壊れていないか」の統合確認に絞る
//! （`tabs_public_api.rs`/`tests/field.rs` と同じ方針）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::field::{textarea, FieldIds, FieldProps};
use fandhe_frontend_headless_ui::questionnaire::QuestionProps;
use fandhe_frontend_headless_ui::radio_group::{self, RadioGroupProps};
use fandhe_frontend_headless_ui::{Orientation, Questionnaire};
use fandhe_frontend_interactive::{dispatch, render_for_hydration};

fn base_field_props(id: &str) -> FieldProps<'_> {
    FieldProps {
        id,
        ids: FieldIds::default(),
        disabled: false,
        invalid: false,
        required: false,
        readonly: false,
        has_helper_text: false,
    }
}

/// 全 11 パーツが `data-scope="questionnaire"` の下で正しい `data-part` を
/// 持ち、`options`/`freeform` へ入れ子にした `radio_group`/`field::textarea`
/// の scope が独立して共存することを固定する。
#[test]
fn questionnaire_public_api_covers_all_eleven_parts_with_nested_scopes() {
    let q = Questionnaire::new(3, 1, Orientation::Horizontal);

    let radio_props = RadioGroupProps::default();
    let options_children = vec![radio_group::root(
        &radio_props,
        None,
        None,
        vec![],
        vec![radio_group::item(
            false,
            &radio_props,
            "yes",
            vec![],
            vec![text("Yes")],
        )],
    )];

    let field_props = base_field_props("freeform-answer");
    let freeform_children = vec![textarea(&field_props, false, vec![], vec![])];

    let node = q.root(
        vec![],
        vec![
            q.progress("Questionnaire progress", vec![], vec![]),
            q.question(
                1,
                QuestionProps::default(),
                vec![],
                vec![
                    q.prompt(vec![], vec![text("Do you like Rust?")]),
                    q.description(vec![], vec![text("Pick one.")]),
                    q.options(vec![], options_children),
                    q.freeform(vec![], freeform_children),
                    q.actions(
                        vec![],
                        vec![
                            q.back(false, vec![], vec![text("Back")]),
                            q.next(false, vec![], vec![text("Next")]),
                            q.skip(false, vec![], vec![text("Skip")]),
                        ],
                    ),
                ],
            ),
        ],
    );
    let html = render(&node);

    // Questionnaire scope の 11 パーツ全件。
    assert!(html.contains(r#"data-scope="questionnaire""#));
    for part in [
        "root",
        "progress",
        "question",
        "prompt",
        "description",
        "options",
        "freeform",
        "actions",
        "back",
        "next",
        "skip",
    ] {
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "questionnaire scope に data-part=\"{part}\" が見当たらない: {html}"
        );
    }

    // 入れ子にした radio_group / field の scope が独立して共存する。
    assert!(html.contains(r#"data-scope="radio-group""#));
    assert!(html.contains(r#"data-scope="field""#));

    // 質問文・補足文のテキストが実際に描画されている。
    assert!(html.contains("Do you like Rust?"));
    assert!(html.contains("Pick one."));
}

/// 非 active な question は `hidden` 属性で隠れ、active な question のみ
/// 可視である（[`Questionnaire::question`] の表示契約）。
#[test]
fn non_active_questions_are_hidden_and_active_question_is_visible() {
    let q = Questionnaire::new(3, 1, Orientation::Horizontal);
    let props = QuestionProps::default();

    let completed_html = render(&q.question(0, props, vec![], vec![text("Q0")]));
    assert!(completed_html.contains("hidden"));
    assert!(completed_html.contains(r#"data-state="completed""#));

    let active_html = render(&q.question(1, props, vec![], vec![text("Q1")]));
    assert!(!active_html.contains("hidden"));
    assert!(active_html.contains(r#"data-state="active""#));

    let upcoming_html = render(&q.question(2, props, vec![], vec![text("Q2")]));
    assert!(upcoming_html.contains("hidden"));
    assert!(upcoming_html.contains(r#"data-state="upcoming""#));
}

/// 完了状態（`step == count`）では root に `data-complete` が付き、
/// next/skip は無条件で disabled になる。
#[test]
fn completed_state_marks_root_and_disables_next_and_skip() {
    let done = Questionnaire::new(2, 2, Orientation::Horizontal);
    assert!(done.is_completed());

    let root_html = render(&done.root(vec![], vec![]));
    assert!(root_html.contains("data-complete"));

    let next_html = render(&done.next(false, vec![], vec![]));
    assert!(next_html.contains("disabled"));
    assert!(next_html.contains("data-disabled"));

    let skip_html = render(&done.skip(false, vec![], vec![]));
    assert!(skip_html.contains("disabled"));
    assert!(skip_html.contains("data-disabled"));

    let back_html = render(&done.back(false, vec![], vec![]));
    assert!(!back_html.contains("disabled"));
}

/// `render_for_hydration` 経由のときのみ `data-hydrate-*` が出力され、
/// パーツメソッド直接呼び出し（SSR の通常経路）では出力されない。
#[test]
fn hydrate_attrs_appear_only_via_render_for_hydration() {
    let q = Questionnaire::new(4, 2, Orientation::Vertical);

    let direct_html = render(&q.root(
        vec![],
        vec![q.question(2, QuestionProps::default(), vec![], vec![])],
    ));
    assert!(!direct_html.contains("data-hydrate-"));

    let hydrate_html = render(&render_for_hydration(&q));
    assert!(hydrate_html.contains(r#"data-hydrate-count="4""#));
    assert!(hydrate_html.contains(r#"data-hydrate-step="2""#));
    assert!(hydrate_html.contains(r#"data-hydrate-orientation="vertical""#));
}

/// dispatch（`"next"`/`"prev"`/`"skip"`/`"goto"`）による状態遷移が
/// 公開 API 経由でも機能する。
#[test]
fn dispatch_transitions_state_via_public_api() {
    let mut q = Questionnaire::new(3, 0, Orientation::Horizontal);

    assert!(dispatch(&mut q, "next", ""));
    assert_eq!(q.step(), 1);

    assert!(dispatch(&mut q, "skip", ""));
    assert_eq!(q.step(), 2);

    assert!(dispatch(&mut q, "prev", ""));
    assert_eq!(q.step(), 1);

    assert!(dispatch(&mut q, "goto", "3"));
    assert_eq!(q.step(), 3);
    assert!(q.is_completed());

    assert!(!dispatch(&mut q, "unknown", ""));
    assert_eq!(q.step(), 3);
}
