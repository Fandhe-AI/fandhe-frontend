//! Checkbox の動的状態機械（[`Checkbox`]、イシュー #595）の統合テスト。
//!
//! `tests/checkbox_escape.rs` が SSR パーツ関数群（`checked: CheckedState`
//! を受け取る純粋関数、#535）の XSS 回帰・attrs マージを固定するのに対し、
//! 本ファイルは `Checkbox`（`Component`/`Hydrate` + `dispatch` 統合、#595 で
//! `crate::state::Checkable` を共通化昇格した後に追加）を公開 API のみを
//! 使って固定する（`tests/switch.rs` と同じ位置付け）。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::{Checkbox, CheckboxFlags};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

#[test]
fn dispatch_check_uncheck_toggle_flip_data_state_across_parts() {
    let mut cb = Checkbox::default();
    assert!(!cb.is_checked());
    assert!(
        render(&cb.hidden_input("terms", "on", CheckboxFlags::default(), vec![]))
            .contains(r#"data-state="unchecked""#)
    );

    assert!(dispatch(&mut cb, "check", ""));
    assert!(cb.is_checked());
    assert!(render(&cb.root(CheckboxFlags::default(), vec![], vec![]))
        .contains(r#"data-state="checked""#));
    assert!(
        render(&cb.control(CheckboxFlags::default(), vec![], vec![]))
            .contains(r#"data-state="checked""#)
    );
    assert!(
        render(&cb.indicator(CheckboxFlags::default(), vec![], vec![]))
            .contains(r#"data-state="checked""#)
    );
    assert!(render(&cb.label(CheckboxFlags::default(), vec![], vec![]))
        .contains(r#"data-state="checked""#));
    assert!(
        render(&cb.hidden_input("terms", "on", CheckboxFlags::default(), vec![]))
            .contains(r#"checked="""#)
    );

    assert!(dispatch(&mut cb, "uncheck", ""));
    assert!(!cb.is_checked());

    assert!(dispatch(&mut cb, "toggle", ""));
    assert!(cb.is_checked());

    assert!(!dispatch(&mut cb, "no_such_action", ""));
    assert!(cb.is_checked());
}

#[test]
fn convenience_methods_reflect_flags() {
    let cb = Checkbox::new(true);
    let flags = CheckboxFlags {
        disabled: true,
        invalid: true,
        required: true,
        readonly: true,
    };
    let html = render(&cb.root(flags, vec![], vec![]));
    assert!(html.contains(r#"data-disabled="""#));
    assert!(html.contains(r#"data-invalid="""#));
    assert!(html.contains(r#"data-required="""#));
    assert!(html.contains(r#"data-readonly="""#));
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr() {
    let cb = Checkbox::default();
    let html = render(&cb.view());
    assert!(!html.contains("data-hydrate-"));
    assert!(html.contains(r#"data-state="unchecked""#));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let cb = Checkbox::new(true);
    let html = render(&render_for_hydration(&cb));
    assert!(html.contains(r#"data-hydrate-checked="checked""#));

    let restored = Checkbox::from_hydration_attrs(&cb.hydration_attrs()).unwrap();
    assert_eq!(restored, cb);
}

#[test]
fn hydration_tampered_or_indeterminate_value_returns_error_not_panic() {
    // 共通機械 Checkable は 2 値のみを扱うため、SSR 静的 props
    // （`CheckedState::Indeterminate`）で表現可能な "indeterminate" も、
    // dispatch/hydration 経路の Checkbox では改ざん入力として拒否される
    // （§設計判断: インタラクティブな tri-state は #595 の out-of-scope）。
    for bogus in ["indeterminate", "CHECKED", "<script>alert(1)</script>", ""] {
        let attrs = vec![("data-hydrate-checked".to_string(), bogus.to_string())];
        let err = Checkbox::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

#[test]
fn hydration_missing_attr_returns_error_not_panic() {
    let err = Checkbox::from_hydration_attrs(&[]).unwrap_err();
    assert_eq!(
        err,
        HydrateError::MissingAttr("data-hydrate-checked".to_string())
    );
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値に攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn name_value_payloads_are_escaped_end_to_end() {
    let cb = Checkbox::default();
    let html = render(&cb.hidden_input(
        ATTR_BREAK_PAYLOAD,
        ATTR_BREAK_PAYLOAD,
        CheckboxFlags::default(),
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let cb = Checkbox::default();
    let html = render(&cb.root(
        CheckboxFlags::default(),
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}
