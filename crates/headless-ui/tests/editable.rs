//! Editable（イシュー #745）の統合テスト。
//!
//! `crates/headless-ui/src/editable.rs` の inline unit tests がパーツ単体の
//! 属性出力・状態機械の遷移を固定するのに対し、本ファイルは「root >
//! label + area(input + preview) + control(edit/submit/cancel トリガー)」の
//! 組み立て全体の data-*/hidden 対応・dispatch 統合・SSR/hydration 両経路・
//! XSS 回帰を、クレート外部から（公開 API のみを使って）固定する
//! （`tests/pin_input.rs`/`tests/number_input.rs` と同型の構成）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::editable::{
    self, EditMode, Editable, EditableActivationMode, EditableInputFlags, EditableInputProps,
    EditableSubmitMode,
};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

#[test]
fn full_assembly_wires_root_label_area_input_preview_and_triggers() {
    let e = Editable::new("Ada", Some(20));
    let mode = e.mode();

    let label = e.label(false, Some("name-input"), vec![], vec![text("Name")]);
    let input = e.input(
        "name",
        Some("name-input"),
        Some("Enter your name"),
        EditableInputFlags::default(),
        vec![],
    );
    let preview = e.preview(vec![], vec![text(e.value())]);
    let area = e.area(vec![], vec![input, preview]);
    let edit_trigger = e.edit_trigger(false, vec![], vec![text("Edit")]);
    let submit_trigger = e.submit_trigger(false, vec![], vec![text("Save")]);
    let cancel_trigger = e.cancel_trigger(false, vec![], vec![text("Cancel")]);
    let control = e.control(vec![], vec![edit_trigger, submit_trigger, cancel_trigger]);
    let root = e.root(
        false,
        false,
        EditableActivationMode::Focus,
        EditableSubmitMode::Both,
        vec![],
        vec![label, area, control],
    );

    let html = render(&root);
    assert!(html.contains(r#"data-scope="editable""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="label""#));
    assert!(html.contains(r#"data-part="area""#));
    assert!(html.contains(r#"data-part="input""#));
    assert!(html.contains(r#"data-part="preview""#));
    assert!(html.contains(r#"data-part="control""#));
    assert!(html.contains(r#"data-part="edit-trigger""#));
    assert!(html.contains(r#"data-part="submit-trigger""#));
    assert!(html.contains(r#"data-part="cancel-trigger""#));
    assert_eq!(mode, EditMode::Preview);
    // preview モード: input は hidden、preview は可視、submit/cancel は hidden。
    assert!(html.contains(r#"name="name" value="Ada""#));
    assert!(html.contains("Ada"));
    assert!(html.contains(r#"data-activation-mode="focus""#));
    assert!(html.contains(r#"data-submit-mode="both""#));
}

#[test]
fn dispatch_edit_set_submit_flow_via_public_api() {
    let mut e = Editable::new("Ada", None);
    assert!(!e.is_editing());

    assert!(dispatch(&mut e, "edit", ""));
    assert!(e.is_editing());
    assert!(
        render(&e.input("name", None, None, EditableInputFlags::default(), vec![]))
            .contains(r#"value="Ada""#)
    );

    assert!(dispatch(&mut e, "set", "Grace Hopper"));
    assert_eq!(e.draft(), "Grace Hopper");
    assert_eq!(e.value(), "Ada");

    assert!(dispatch(&mut e, "submit", ""));
    assert!(!e.is_editing());
    assert_eq!(e.value(), "Grace Hopper");

    assert!(!dispatch(&mut e, "no_such_action", ""));
}

#[test]
fn dispatch_edit_set_cancel_flow_discards_draft_via_public_api() {
    let mut e = Editable::new("Ada", None);
    assert!(dispatch(&mut e, "edit", ""));
    assert!(dispatch(&mut e, "set", "Grace Hopper"));
    assert_eq!(e.draft(), "Grace Hopper");

    assert!(dispatch(&mut e, "cancel", ""));
    assert!(!e.is_editing());
    assert_eq!(e.value(), "Ada");
    assert_eq!(e.draft(), "Ada");
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr() {
    let e = Editable::default();
    let html = render(&e.view());
    assert!(!html.contains("data-hydrate-"));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let mut e = Editable::new("Ada", Some(50));
    dispatch(&mut e, "edit", "");
    dispatch(&mut e, "set", "Grace Hopper");

    let html = render(&render_for_hydration(&e));
    assert!(html.contains(r#"data-hydrate-mode="edit""#));
    assert!(html.contains(r#"data-hydrate-value="Ada""#));
    assert!(html.contains(r#"data-hydrate-draft="Grace Hopper""#));
    assert!(html.contains(r#"data-hydrate-max-length="50""#));

    let restored = Editable::from_hydration_attrs(&e.hydration_attrs()).unwrap();
    assert_eq!(restored.value(), e.value());
    assert_eq!(restored.draft(), e.draft());
    assert_eq!(restored.mode(), e.mode());
    assert_eq!(restored.max_length(), e.max_length());
}

#[test]
fn hydration_tampered_mode_returns_error_not_panic() {
    let attrs = vec![
        ("data-hydrate-mode".to_string(), "attacker".to_string()),
        ("data-hydrate-value".to_string(), "Ada".to_string()),
        ("data-hydrate-draft".to_string(), "Ada".to_string()),
        ("data-hydrate-max-length".to_string(), "none".to_string()),
    ];
    let err = Editable::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値すべてに攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn name_and_value_payloads_are_escaped_end_to_end() {
    let input = editable::input(
        EditMode::Edit,
        ATTR_BREAK_PAYLOAD,
        ATTR_BREAK_PAYLOAD,
        EditableInputProps::default(),
        EditableInputFlags::default(),
        vec![],
    );
    let html = render(&editable::root(
        EditMode::Edit,
        false,
        false,
        EditableActivationMode::default(),
        EditableSubmitMode::default(),
        vec![],
        vec![input],
    ));

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let html = render(&editable::root(
        EditMode::Preview,
        false,
        false,
        EditableActivationMode::default(),
        EditableSubmitMode::default(),
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}
