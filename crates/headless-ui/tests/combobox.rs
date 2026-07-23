//! Combobox（イシュー #749）の統合テスト。
//!
//! `crates/headless-ui/src/combobox.rs` の inline unit tests がパーツ単体の
//! 属性出力を固定するのに対し、本ファイルは
//! `label + control(input, trigger, clear_trigger) +`
//! `positioner(content(item_group(item(item_text, item_indicator))))` という
//! 全体の組み立てにおける data-*/ARIA 対応・dispatch 統合（closeOnSelect/
//! openOnChange 含む）・SSR/hydration 両経路・XSS 回帰をクレート外部から
//! （公開 API のみを使って）固定する（`crates/headless-ui/tests/select.rs`
//! と同じ粒度）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::combobox::{self, Combobox};
use fandhe_frontend_headless_ui::OpenState;
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

#[test]
fn full_assembly_wires_aria_controls_labelledby_and_all_parts_appear() {
    let label = combobox::label(
        Some("combobox-label-1"),
        Some("combobox-input-1"),
        vec![],
        vec![text("Framework")],
    );

    let input = combobox::input(
        OpenState::Open,
        "vu",
        false,
        Some("combobox-content-1"),
        Some("item-vue"),
        Some("framework"),
        vec![("id", "combobox-input-1")],
    );
    let trigger = combobox::trigger(
        OpenState::Open,
        false,
        Some("combobox-content-1"),
        vec![],
        vec![],
    );
    let clear_trigger = combobox::clear_trigger(vec![("aria-label", "Clear")], vec![]);
    let control = combobox::control(OpenState::Open, vec![], vec![input, trigger, clear_trigger]);

    let item_text = combobox::item_text(Some("item-text-vue"), vec![], vec![text("Vue")]);
    let item_indicator = combobox::item_indicator(OpenState::Open, vec![], vec![text("✓")]);
    let item = combobox::item(
        OpenState::Open,
        false,
        true,
        "vue",
        Some("item-vue"),
        vec![],
        vec![item_text, item_indicator],
    );
    let item_group_label =
        combobox::item_group_label(Some("group-label-1"), vec![], vec![text("Frameworks")]);
    let item_group =
        combobox::item_group(Some("group-label-1"), vec![], vec![item_group_label, item]);
    let content = combobox::content(
        OpenState::Open,
        Some("combobox-content-1"),
        Some("combobox-label-1"),
        vec![],
        vec![item_group],
    );
    let positioner = combobox::positioner(OpenState::Open, vec![], vec![content]);

    let root = combobox::root(OpenState::Open, vec![], vec![label, control, positioner]);

    let html = render(&root);

    // 全 data-part の出現を固定する。
    for part in [
        "root",
        "label",
        "control",
        "input",
        "trigger",
        "clear-trigger",
        "positioner",
        "content",
        "item-group",
        "item-group-label",
        "item",
        "item-text",
        "item-indicator",
    ] {
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "missing data-part=\"{part}\" in: {html}"
        );
    }

    // aria-controls/aria-labelledby/label[for] の id 対応。
    assert!(html.contains(r#"aria-controls="combobox-content-1""#));
    assert!(html.contains(r#"id="combobox-content-1""#));
    assert!(html.contains(r#"aria-labelledby="combobox-label-1""#));
    assert!(html.contains(r#"id="combobox-label-1""#));
    assert!(html.contains(r#"for="combobox-input-1""#));
    assert!(html.contains(r#"id="combobox-input-1""#));

    // role / aria-* の付与。
    assert!(html.contains(r#"role="combobox""#));
    assert!(html.contains(r#"aria-haspopup="listbox""#));
    assert!(html.contains(r#"role="listbox""#));
    assert!(html.contains(r#"role="group""#));
    assert!(html.contains(r#"aria-selected="true""#));
    assert!(html.contains(r#"aria-autocomplete="list""#));
    assert!(html.contains(r#"autocomplete="off""#));

    // aria-activedescendant は input 側にのみ配線され、content には付与
    // されない（モジュール doc「aria-activedescendant の配線先」参照。
    // Select（content 側）との差異）。
    assert!(html.contains(r#"aria-activedescendant="item-vue""#));
    assert_eq!(html.matches("aria-activedescendant").count(), 1);

    // trigger はタブ順から外れる（フォーカスは input が保持する）。
    assert!(html.contains(r#"tabindex="-1""#));

    // highlight の SSR 表現: item の data-highlighted/id と input の
    // aria-activedescendant が同一 id で対応する。
    assert!(html.contains(r#"data-highlighted="""#));
    assert!(html.contains(r#"id="item-vue""#));

    // input の現在値。
    assert!(html.contains(r#"value="vu""#));

    // open 状態なので positioner/content に hidden 存在属性は付かない。
    assert!(!html.contains(r#" hidden="""#));
}

#[test]
fn positioner_and_content_closed_have_hidden_and_no_role_leak() {
    let content = combobox::content(OpenState::Closed, None, None, vec![], vec![]);
    let positioner = combobox::positioner(OpenState::Closed, vec![], vec![content]);
    let html = render(&positioner);
    assert!(html.contains(r#"data-state="closed""#));
    // positioner と content の両方に hidden が付く。
    assert_eq!(html.matches(r#"hidden="""#).count(), 2);
}

#[test]
fn dispatch_open_close_toggle_flip_data_state_across_parts() {
    let mut c = Combobox::default();
    assert!(!c.is_open());
    assert!(render(&c.content(None, None, vec![], vec![])).contains(r#"hidden="""#));

    assert!(dispatch(&mut c, "open", ""));
    assert!(c.is_open());
    assert!(render(&c.root(vec![], vec![])).contains(r#"data-state="open""#));
    assert!(render(&c.input(false, None, None, None, vec![])).contains(r#"aria-expanded="true""#));
    assert!(render(&c.trigger(false, None, vec![], vec![])).contains(r#"aria-expanded="true""#));
    assert!(render(&c.positioner(vec![], vec![])).contains(r#"data-state="open""#));
    assert!(!render(&c.content(None, None, vec![], vec![])).contains("hidden"));

    assert!(dispatch(&mut c, "close", ""));
    assert!(!c.is_open());

    assert!(dispatch(&mut c, "toggle", ""));
    assert!(c.is_open());
    assert!(dispatch(&mut c, "toggle", ""));
    assert!(!c.is_open());
}

#[test]
fn dispatch_select_updates_value_and_closes_listbox_close_on_select() {
    let mut c = Combobox::default();
    dispatch(&mut c, "open", "");
    assert!(c.is_open());

    assert!(dispatch(&mut c, "select", "vue"));
    assert_eq!(c.selected(), Some("vue"));
    assert!(!c.is_open());

    assert!(render(&c.item("vue", false, false, None, vec![], vec![]))
        .contains(r#"aria-selected="true""#));
    assert!(render(&c.item("react", false, false, None, vec![], vec![]))
        .contains(r#"aria-selected="false""#));
}

#[test]
fn dispatch_deselect_clears_selection() {
    let mut c = Combobox::default();
    dispatch(&mut c, "select", "vue");
    assert!(dispatch(&mut c, "deselect", ""));
    assert_eq!(c.selected(), None);
}

#[test]
fn dispatch_input_updates_value_and_opens_listbox_open_on_change() {
    let mut c = Combobox::default();
    assert!(!c.is_open());

    assert!(dispatch(&mut c, "input", "re"));
    assert_eq!(c.input_value(), "re");
    assert!(c.is_open());

    let options = [("vue", "Vue"), ("react", "React")];
    assert_eq!(c.filtered_options(&options), vec![("react", "React")]);
}

#[test]
fn dispatch_clear_clears_input_and_selection() {
    let mut c = Combobox::default();
    dispatch(&mut c, "input", "vu");
    dispatch(&mut c, "select", "vue");

    assert!(dispatch(&mut c, "clear", ""));
    assert_eq!(c.input_value(), "");
    assert_eq!(c.selected(), None);
}

#[test]
fn dispatch_ignores_unknown_action() {
    let mut c = Combobox::default();
    dispatch(&mut c, "select", "vue");
    assert!(!dispatch(&mut c, "no_such_action", "x"));
    assert_eq!(c.selected(), Some("vue"));
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr() {
    let c = Combobox::default();
    let html = render(&c.view());
    assert!(!html.contains("data-hydrate-"));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let mut c = Combobox::default();
    dispatch(&mut c, "input", "vu");
    dispatch(&mut c, "select", "vue");
    // select は listbox を閉じるため、開いた状態のラウンドトリップを
    // 確認するために再度 open する。
    dispatch(&mut c, "open", "");

    let html = render(&render_for_hydration(&c));
    assert!(html.contains(r#"data-hydrate-state="open""#));
    assert!(html.contains("data-hydrate-selected="));
    assert!(html.contains("data-hydrate-input="));

    let restored = Combobox::from_hydration_attrs(&c.hydration_attrs()).unwrap();
    assert_eq!(restored, c);
}

#[test]
fn hydration_tampered_state_value_returns_error_not_panic() {
    for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
        let attrs = vec![
            ("data-hydrate-state".to_string(), bogus.to_string()),
            (
                "data-hydrate-selected".to_string(),
                fandhe_frontend_interactive::codec::encode_list(&[]),
            ),
            (
                "data-hydrate-input".to_string(),
                fandhe_frontend_interactive::codec::encode_list(&[String::new()]),
            ),
        ];
        let err = Combobox::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

#[test]
fn hydration_missing_attrs_returns_error_not_panic() {
    let err = Combobox::from_hydration_attrs(&[]).unwrap_err();
    assert_eq!(
        err,
        HydrateError::MissingAttr("data-hydrate-state".to_string())
    );
}

#[test]
fn hydration_tampered_multiple_selected_returns_error_not_panic() {
    use fandhe_frontend_interactive::codec;
    let bogus = codec::encode_list(&["a".to_string(), "b".to_string()]);
    let attrs = vec![
        ("data-hydrate-state".to_string(), "closed".to_string()),
        ("data-hydrate-selected".to_string(), bogus),
        (
            "data-hydrate-input".to_string(),
            codec::encode_list(&[String::new()]),
        ),
    ];
    let err = Combobox::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

#[test]
fn hydration_tampered_input_value_count_returns_error_not_panic() {
    use fandhe_frontend_interactive::codec;
    let bogus = codec::encode_list(&["a".to_string(), "b".to_string()]);
    let attrs = vec![
        ("data-hydrate-state".to_string(), "closed".to_string()),
        ("data-hydrate-selected".to_string(), codec::encode_list(&[])),
        ("data-hydrate-input".to_string(), bogus),
    ];
    let err = Combobox::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値すべてに攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn controls_labelledby_value_and_activedescendant_payloads_are_escaped_end_to_end() {
    let input = combobox::input(
        OpenState::Closed,
        ATTR_BREAK_PAYLOAD,
        false,
        Some(ATTR_BREAK_PAYLOAD),
        Some(ATTR_BREAK_PAYLOAD),
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
    );
    let trigger = combobox::trigger(
        OpenState::Closed,
        false,
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![],
    );
    let item = combobox::item(
        OpenState::Closed,
        false,
        false,
        ATTR_BREAK_PAYLOAD,
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![],
    );
    let content = combobox::content(
        OpenState::Closed,
        Some(ATTR_BREAK_PAYLOAD),
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![],
    );
    let html = render(&combobox::root(
        OpenState::Closed,
        vec![],
        vec![input, trigger, item, content],
    ));

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn dispatch_input_payload_is_escaped_end_to_end() {
    let mut c = Combobox::default();
    let payload = "\"><script>alert(1)</script>";
    assert!(dispatch(&mut c, "input", payload));

    let html = render(&render_for_hydration(&c));
    assert!(html.contains("data-hydrate-input="));
    assert!(html.contains("&lt;script&gt;"));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(!html.contains(r#""><script"#));
}

#[test]
fn dispatch_select_payload_is_escaped_end_to_end() {
    let mut c = Combobox::default();
    let payload = "\"><script>alert(1)</script>";
    assert!(dispatch(&mut c, "select", payload));

    let html = render(&render_for_hydration(&c));
    assert!(html.contains("data-hydrate-selected="));
    assert!(html.contains("&lt;script&gt;"));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(!html.contains(r#""><script"#));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let html = render(&combobox::root(
        OpenState::Closed,
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}

#[test]
fn filter_options_end_to_end_with_dynamic_query_and_labels() {
    let options = [("vue", "Vue"), ("react", "React"), ("svelte", "Svelte")];
    let mut c = Combobox::default();
    dispatch(&mut c, "input", "re");
    assert_eq!(c.filtered_options(&options), vec![("react", "React")]);

    dispatch(&mut c, "input", "");
    assert_eq!(c.filtered_options(&options), options.to_vec());
}
