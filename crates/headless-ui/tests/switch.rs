//! Switch（イシュー #537、参照突合イシュー #1622）の統合テスト。
//!
//! `crates/headless-ui/src/switch.rs` の inline unit tests がパーツ単体の
//! 属性出力を固定するのに対し、本ファイルは「root > control(thumb) + label +
//! hidden_input」の組み立て全体の data-*/ARIA 対応・dispatch 統合・
//! SSR/hydration 両経路・XSS 回帰をクレート外部から（公開 API のみを使って）
//! 固定する。イシュー #1622 で ark-ui / Radix Primitives との anatomy /
//! `data-*` / キーボード操作の突合契約テスト（`reference_*`/
//! `data_state_vocabulary_*`/`no_part_outputs_*`/`disabled_invalid_readonly_required_*`/
//! `reserved_state_keys_*`）を追加した。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::switch::{self, Switch, SwitchProps};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

fn plain() -> SwitchProps {
    SwitchProps::default()
}

#[test]
fn full_assembly_wires_root_control_thumb_label_and_hidden_input() {
    let props = plain();
    let control = switch::control(
        true,
        &props,
        vec![],
        vec![switch::thumb(true, &props, vec![], vec![])],
    );
    let label = switch::label(
        true,
        &props,
        vec![],
        vec![fandhe_frontend_core::text("Wi-Fi")],
    );
    let hidden_input = switch::hidden_input("wifi", "on", true, &props, vec![]);
    let root = switch::root(true, &props, vec![], vec![control, label, hidden_input]);

    let html = render(&root);
    assert!(html.contains(r#"data-scope="switch""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="control""#));
    assert!(html.contains(r#"data-part="thumb""#));
    assert!(html.contains(r#"data-part="label""#));
    assert!(html.contains(r#"data-part="hidden-input""#));
    assert!(html.contains(r#"data-state="checked""#));
    assert!(html.contains(r#"type="checkbox""#));
    assert!(html.contains(r#"role="switch""#));
    assert!(html.contains(r#"name="wifi""#));
    assert!(html.contains("Wi-Fi"));
    assert!(!html.contains("aria-checked"));
}

#[test]
fn dispatch_toggle_flips_data_state_across_parts() {
    let mut s = Switch::default();
    let props = plain();
    assert!(!s.is_checked());
    assert!(!render(&s.hidden_input("wifi", "on", &props, vec![])).contains(r#"checked="""#));

    assert!(dispatch(&mut s, "toggle", ""));
    assert!(s.is_checked());
    assert!(render(&s.root(&props, vec![], vec![])).contains(r#"data-state="checked""#));
    assert!(render(&s.control(&props, vec![], vec![])).contains(r#"data-state="checked""#));
    assert!(render(&s.thumb(&props, vec![], vec![])).contains(r#"data-state="checked""#));
    assert!(render(&s.label(&props, vec![], vec![])).contains(r#"data-state="checked""#));
    assert!(render(&s.hidden_input("wifi", "on", &props, vec![])).contains(r#"checked="""#));

    assert!(dispatch(&mut s, "uncheck", ""));
    assert!(!s.is_checked());

    assert!(dispatch(&mut s, "check", ""));
    assert!(s.is_checked());

    assert!(!dispatch(&mut s, "no_such_action", ""));
    assert!(s.is_checked());
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr() {
    let s = Switch::default();
    let html = render(&s.view());
    assert!(!html.contains("data-hydrate-"));
    assert!(html.contains(r#"data-state="unchecked""#));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let s = Switch::new(true);
    let html = render(&render_for_hydration(&s));
    assert!(html.contains(r#"data-hydrate-checked="checked""#));

    let restored = Switch::from_hydration_attrs(&s.hydration_attrs()).unwrap();
    assert_eq!(restored, s);
}

#[test]
fn hydration_tampered_value_returns_error_not_panic() {
    for bogus in ["CHECKED", "<script>alert(1)</script>", ""] {
        let attrs = vec![("data-hydrate-checked".to_string(), bogus.to_string())];
        let err = Switch::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

// --- イシュー #1622: ark-ui / Radix Primitives との参照突合契約 ---

#[test]
fn reference_anatomy_part_names_match_ark_ui() {
    // ark-ui Switch anatomy: Root / Control / Thumb / Label / HiddenInput。
    let props = plain();
    let node = switch::root(
        false,
        &props,
        vec![],
        vec![
            switch::hidden_input("wifi", "on", false, &props, vec![]),
            switch::control(
                false,
                &props,
                vec![],
                vec![switch::thumb(false, &props, vec![], vec![])],
            ),
            switch::label(false, &props, vec![], vec![]),
        ],
    );
    let html = render(&node);
    assert!(html.contains(r#"data-scope="switch""#));
    for part in ["root", "control", "thumb", "label", "hidden-input"] {
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "missing part {part} in {html}"
        );
    }
}

#[test]
fn data_state_vocabulary_is_two_valued_on_every_part() {
    // イシュー #1622: hidden_input へも他 4 パーツと同じ data-state を出力
    // する（ark の HiddenInput は data-state を持たないが、
    // crate::checkbox::hidden_input と同契約に合わせる意図的な差分。
    // モジュール doc「参考サイトとの意図的な差分」節参照）。
    let props = plain();
    for checked in [true, false] {
        let expected = if checked { "checked" } else { "unchecked" };
        let root = render(&switch::root(checked, &props, vec![], vec![]));
        let control = render(&switch::control(checked, &props, vec![], vec![]));
        let thumb = render(&switch::thumb(checked, &props, vec![], vec![]));
        let label = render(&switch::label(checked, &props, vec![], vec![]));
        let hidden_input = render(&switch::hidden_input("wifi", "on", checked, &props, vec![]));
        for (name, html) in [
            ("root", &root),
            ("control", &control),
            ("thumb", &thumb),
            ("label", &label),
            ("hidden_input", &hidden_input),
        ] {
            assert!(
                html.contains(&format!(r#"data-state="{expected}""#)),
                "{name} missing data-state={expected}: {html}"
            );
        }
    }
}

#[test]
fn no_part_outputs_pointer_or_focus_interaction_attrs() {
    // 意図的非採用: data-hover/data-active/data-focus/data-motion
    // （`docs/policy/intentional-non-adoption.md` §3.25 規則 2）。
    let props = plain();
    let html = render(&switch::root(
        true,
        &props,
        vec![],
        vec![
            switch::control(true, &props, vec![], vec![]),
            switch::thumb(true, &props, vec![], vec![]),
            switch::label(true, &props, vec![], vec![]),
            switch::hidden_input("wifi", "on", true, &props, vec![]),
        ],
    ));
    for forbidden in ["data-hover", "data-active", "data-focus=", "data-motion"] {
        assert!(
            !html.contains(forbidden),
            "unexpected {forbidden} in {html}"
        );
    }
}

#[test]
fn hidden_input_relies_on_native_checked_mapping_and_role_switch() {
    let html = render(&switch::hidden_input("wifi", "on", true, &plain(), vec![]));
    assert!(html.contains(r#"type="checkbox""#));
    assert!(html.contains(r#"role="switch""#));
    assert!(!html.contains("aria-checked"));
}

#[test]
fn control_is_always_aria_hidden() {
    let html = render(&switch::control(false, &plain(), vec![], vec![]));
    assert!(html.contains(r#"aria-hidden="true""#));
}

#[test]
fn disabled_invalid_readonly_required_are_data_attrs_on_every_part() {
    let props = SwitchProps {
        disabled: true,
        invalid: true,
        readonly: true,
        required: true,
    };
    let root = render(&switch::root(false, &props, vec![], vec![]));
    let control = render(&switch::control(false, &props, vec![], vec![]));
    let thumb = render(&switch::thumb(false, &props, vec![], vec![]));
    let label = render(&switch::label(false, &props, vec![], vec![]));
    let hidden_input = render(&switch::hidden_input("wifi", "on", false, &props, vec![]));

    for (name, html) in [
        ("root", &root),
        ("control", &control),
        ("thumb", &thumb),
        ("label", &label),
        ("hidden_input", &hidden_input),
    ] {
        for data_attr in [
            "data-disabled",
            "data-invalid",
            "data-required",
            "data-readonly",
        ] {
            assert!(
                html.contains(data_attr),
                "{name} missing {data_attr}: {html}"
            );
        }
    }

    assert!(hidden_input.contains(r#"aria-invalid="true""#));
    assert!(hidden_input.contains(r#"required="""#));
    assert!(hidden_input.contains(r#"disabled="""#));
    assert!(!hidden_input.contains(r#" readonly"#));
}

#[test]
fn reserved_state_keys_in_caller_attrs_are_dropped() {
    let html = render(&switch::root(
        false,
        &plain(),
        vec![
            ("data-state", "checked"),
            ("DATA-DISABLED", ""),
            ("data-invalid", ""),
            ("data-required", ""),
            ("data-readonly", ""),
        ],
        vec![],
    ));
    assert!(html.contains(r#"data-state="unchecked""#));
    assert_eq!(html.matches("data-state").count(), 1);
    assert!(!html.contains("data-disabled"));
    assert!(!html.contains("data-invalid"));
    assert!(!html.contains("data-required"));
    assert!(!html.contains("data-readonly"));
}

#[test]
fn reserved_keys_on_hidden_input_are_dropped() {
    let html = render(&switch::hidden_input(
        "wifi",
        "on",
        false,
        &plain(),
        vec![
            ("type", "text"),
            ("ROLE", "textbox"),
            ("checked", "checked"),
            ("aria-checked", "true"),
            ("aria-invalid", "true"),
            ("name", "attacker"),
            ("value", "attacker"),
            ("disabled", ""),
            ("required", ""),
        ],
    ));
    assert!(html.contains(r#"type="checkbox""#));
    assert!(html.contains(r#"role="switch""#));
    assert!(html.contains(r#"name="wifi""#));
    assert!(html.contains(r#"value="on""#));
    assert!(!html.contains("aria-checked"));
    assert!(!html.contains("aria-invalid"));
    assert!(!html.contains("attacker"));
    assert!(!html.contains(r#"disabled="""#));
    assert!(!html.contains(r#"required="""#));
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値すべてに攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn name_value_payloads_are_escaped_end_to_end() {
    let hidden_input = switch::hidden_input(
        ATTR_BREAK_PAYLOAD,
        ATTR_BREAK_PAYLOAD,
        false,
        &plain(),
        vec![],
    );
    let html = render(&switch::root(false, &plain(), vec![], vec![hidden_input]));

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let html = render(&switch::root(
        false,
        &plain(),
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}
