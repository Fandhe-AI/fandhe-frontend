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

// --- 参考サイト（ark-ui / Radix Primitives）突合契約（イシュー #1602） ---
//
// 上記のテスト群が `Checkbox`（dispatch/hydration 統合）の挙動を固定する
// のに対し、以下は SSR パーツ関数群（`checkbox::{root, control, indicator,
// label, hidden_input}`、`CheckedState` を受け取る純粋関数、#535）が
// ark-ui の Checkbox anatomy・`data-*` 語彙・ARIA と一致し続けることを
// fail-closed に固定する（`tests/navigation_menu.rs::no_part_outputs_data_motion`
// と同型の趣旨）。差分調査の詳細はイシュー #1602 コメント参照。

use fandhe_frontend_core::text;
use fandhe_frontend_headless_ui::checkbox::{
    control, hidden_input, indicator, label, root, CheckboxProps, CheckedState,
};

/// 5 パーツすべてを描画し、`data-scope="checkbox"` の存在と `data-part` の
/// 集合が ark-ui の Checkbox anatomy（root/control/indicator/label/
/// hidden-input）と一致することを固定する。
#[test]
fn reference_anatomy_part_names_match_ark_ui() {
    let props = CheckboxProps::default();
    let parts: [(&str, String); 5] = [
        ("root", render(&root(&props, vec![], vec![]))),
        ("control", render(&control(&props, vec![], vec![]))),
        ("indicator", render(&indicator(&props, vec![], vec![]))),
        ("label", render(&label(&props, vec![], vec![]))),
        (
            "hidden-input",
            render(&hidden_input(&props, "n", "v", vec![])),
        ),
    ];
    for (part, html) in &parts {
        assert!(
            html.contains(r#"data-scope="checkbox""#),
            "{part} が data-scope=\"checkbox\" を持たない: {html}"
        );
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "{part} が期待する data-part を持たない: {html}"
        );
    }
}

/// `Unchecked`/`Checked`/`Indeterminate` の 3 値が全パーツの `data-state`
/// へ一貫して反映されることを固定する（ark-ui の「全パーツが data-state を
/// 持つ」規約）。
#[test]
fn data_state_vocabulary_is_three_valued_on_every_part() {
    for (state, expected) in [
        (CheckedState::Unchecked, "unchecked"),
        (CheckedState::Checked, "checked"),
        (CheckedState::Indeterminate, "indeterminate"),
    ] {
        let props = CheckboxProps {
            checked: state,
            ..Default::default()
        };
        let expected_attr = format!(r#"data-state="{expected}""#);
        assert!(render(&root(&props, vec![], vec![])).contains(&expected_attr));
        assert!(render(&control(&props, vec![], vec![])).contains(&expected_attr));
        assert!(render(&indicator(&props, vec![], vec![])).contains(&expected_attr));
        assert!(render(&label(&props, vec![], vec![])).contains(&expected_attr));
        assert!(render(&hidden_input(&props, "n", "v", vec![])).contains(&expected_attr));
    }
}

/// ark-ui は pointer/focus のローカル操作状態として全パーツへ
/// `data-hover`/`data-active`/`data-focus`/`data-motion` を付与するが、
/// 本実装は SSR 静的出力にこれらを持たせない設計判断（DOM ローカル状態は
/// CSS 擬似クラス、または `data-focus-visible` のように明示的に合成可能な
/// 属性でのみ表現する。`crate::data_attrs::data_focus_visible`/
/// `data_highlighted` と同型の契約）を固定する。
#[test]
fn no_part_outputs_pointer_or_focus_interaction_attrs() {
    for state in [
        CheckedState::Unchecked,
        CheckedState::Checked,
        CheckedState::Indeterminate,
    ] {
        let props = CheckboxProps {
            checked: state,
            disabled: true,
            invalid: true,
            required: true,
            readonly: true,
        };
        let html = render(&root(
            &props,
            vec![],
            vec![
                control(
                    &props,
                    vec![],
                    vec![indicator(&props, vec![], vec![text("✓")])],
                ),
                label(&props, vec![], vec![text("Label")]),
                hidden_input(&props, "n", "v", vec![]),
            ],
        ));
        assert!(!html.contains("data-hover"), "{html}");
        assert!(!html.contains("data-active"), "{html}");
        assert!(
            !html.contains("data-focus=\"") && !html.contains("data-focus \""),
            "{html}"
        );
        assert!(!html.contains("data-motion"), "{html}");
    }
}

/// ネイティブ `<input type="checkbox">` の `checked` プロパティは
/// ブラウザが写像するため `Checked`/`Unchecked` では `aria-checked` を
/// 出力しない。`Indeterminate` のみ ARIA での補完として `aria-checked="mixed"`
/// を出力する。`role` 属性はネイティブ input の暗黙ロールのため一切
/// 出力しない（Radix の `button role="checkbox"` パターンは不採用、
/// checkbox.rs モジュール doc 参照）。
#[test]
fn hidden_input_relies_on_native_checked_mapping() {
    for (state, expect_mixed) in [
        (CheckedState::Unchecked, false),
        (CheckedState::Checked, false),
        (CheckedState::Indeterminate, true),
    ] {
        let props = CheckboxProps {
            checked: state,
            ..Default::default()
        };
        let html = render(&hidden_input(&props, "n", "v", vec![]));
        assert_eq!(html.contains(r#"aria-checked="mixed""#), expect_mixed);
        assert!(!html.contains(r#"aria-checked="true""#));
        assert!(!html.contains(r#"aria-checked="false""#));
        assert!(!html.contains(r#"role="#));
    }
}

/// `control` は視覚表現のみを担うため 3 状態すべてで常時
/// `aria-hidden="true"` を持つ。`indicator` は `Unchecked` のときのみ
/// `hidden` 存在属性を持つ（`Checked`/`Indeterminate` では表示する）。
#[test]
fn control_is_always_aria_hidden_and_indicator_hidden_only_when_unchecked() {
    for (state, indicator_hidden) in [
        (CheckedState::Unchecked, true),
        (CheckedState::Checked, false),
        (CheckedState::Indeterminate, false),
    ] {
        let props = CheckboxProps {
            checked: state,
            ..Default::default()
        };
        let control_html = render(&control(&props, vec![], vec![]));
        assert!(control_html.contains(r#"aria-hidden="true""#));

        let indicator_html = render(&indicator(&props, vec![], vec![]));
        assert_eq!(indicator_html.contains(r#"hidden"#), indicator_hidden);
    }
}

/// `readonly`/`required` フラグは全パーツへ `data-readonly`/`data-required`
/// 存在属性として反映される（ark-ui の「全パーツが状態フラグの data-* を
/// 持つ」規約に合わせた設計）。
#[test]
fn readonly_and_required_are_data_attrs_on_every_part() {
    let props = CheckboxProps {
        readonly: true,
        required: true,
        ..Default::default()
    };
    for html in [
        render(&root(&props, vec![], vec![])),
        render(&control(&props, vec![], vec![])),
        render(&indicator(&props, vec![], vec![])),
        render(&label(&props, vec![], vec![])),
        render(&hidden_input(&props, "n", "v", vec![])),
    ] {
        assert!(html.contains(r#"data-readonly="""#), "{html}");
        assert!(html.contains(r#"data-required="""#), "{html}");
    }
}
