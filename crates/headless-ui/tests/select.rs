//! Select（イシュー #541）の統合テスト。
//!
//! `crates/headless-ui/src/select.rs` の inline unit tests がパーツ単体の
//! 属性出力を固定するのに対し、本ファイルは
//! `label + control(trigger, value_text, clear_trigger) +`
//! `positioner(content(item_group(item(item_text, item_indicator)))) +`
//! `hidden_select` という全体の組み立てにおける data-*/ARIA 対応・dispatch
//! 統合（closeOnSelect 含む）・SSR/hydration 両経路・XSS 回帰をクレート
//! 外部から（公開 API のみを使って）固定する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::select::{self, Select};
use fandhe_frontend_headless_ui::OpenState;
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

#[test]
fn full_assembly_wires_aria_controls_labelledby_and_all_parts_appear() {
    let label = select::label(Some("select-label-1"), vec![], vec![text("Framework")]);

    let trigger = select::trigger(
        OpenState::Open,
        false,
        Some("select-content-1"),
        Some("select-label-1"),
        vec![],
        vec![],
    );
    let value_text = select::value_text(false, vec![], vec![text("Vue")]);
    let clear_trigger = select::clear_trigger(vec![("aria-label", "Clear")], vec![]);
    let control = select::control(
        OpenState::Open,
        vec![],
        vec![trigger, value_text, clear_trigger],
    );

    let item_text = select::item_text(Some("item-text-vue"), vec![], vec![text("Vue")]);
    let item_indicator = select::item_indicator(OpenState::Open, vec![], vec![text("✓")]);
    let item = select::item(
        OpenState::Open,
        false,
        true,
        "vue",
        Some("item-vue"),
        vec![],
        vec![item_text, item_indicator],
    );
    let item_group_label =
        select::item_group_label(Some("group-label-1"), vec![], vec![text("Frameworks")]);
    let item_group =
        select::item_group(Some("group-label-1"), vec![], vec![item_group_label, item]);
    let content = select::content(
        OpenState::Open,
        Some("select-content-1"),
        Some("select-label-1"),
        Some("item-vue"),
        vec![],
        vec![item_group],
    );
    let positioner = select::positioner(OpenState::Open, vec![], vec![content]);

    let hidden_select = select::hidden_select(
        Some("vue"),
        Some("framework"),
        false,
        vec![],
        vec![("vue", "Vue"), ("react", "React")],
    );

    let root = select::root(
        OpenState::Open,
        vec![],
        vec![label, control, positioner, hidden_select],
    );

    let html = render(&root);

    // 全 data-part の出現を固定する。
    for part in [
        "root",
        "label",
        "control",
        "trigger",
        "value-text",
        "clear-trigger",
        "positioner",
        "content",
        "item-group",
        "item-group-label",
        "item",
        "item-text",
        "item-indicator",
        "hidden-select",
    ] {
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "missing data-part=\"{part}\" in: {html}"
        );
    }

    // aria-controls/aria-labelledby の id 対応。
    assert!(html.contains(r#"aria-controls="select-content-1""#));
    assert!(html.contains(r#"id="select-content-1""#));
    assert!(html.contains(r#"aria-labelledby="select-label-1""#));
    assert!(html.contains(r#"id="select-label-1""#));

    // role / aria-* の付与。
    assert!(html.contains(r#"aria-haspopup="listbox""#));
    assert!(html.contains(r#"role="listbox""#));
    assert!(html.contains(r#"role="group""#));
    assert!(html.contains(r#"aria-selected="true""#));

    // highlight の SSR 表現（イシュー #599）: item の data-highlighted/id と
    // content の aria-activedescendant が同一 id で対応する。
    assert!(html.contains(r#"data-highlighted="""#));
    assert!(html.contains(r#"id="item-vue""#));
    assert!(html.contains(r#"aria-activedescendant="item-vue""#));

    // hidden_select のフォーム統合。
    assert!(html.contains(r#"<select"#));
    assert!(html.contains(r#"aria-hidden="true""#));
    assert!(html.contains(r#"name="framework""#));
    assert!(html.contains(r#"<option value="vue" selected="">Vue</option>"#));

    // open 状態なので positioner/content に hidden 存在属性は付かない
    // （hidden_select パーツが `aria-hidden="true"` を固定で持つため、
    // 部分一致 "hidden" ではなく存在属性そのもの `hidden=""` の不在を見る）。
    assert!(!html.contains(r#" hidden="""#));

    // value-text は data-bind-text 束縛マーカーを常時持つ（イシュー #642）。
    // wasm-full の headless_select 配線層がこのマーカーを頼りに select/
    // deselect dispatch 後のラベル再同期を行う契約。
    assert!(html.contains(&format!(r#"data-bind-text="{}""#, select::VALUE_TEXT_FIELD)));
}

#[test]
fn positioner_and_content_closed_have_hidden_and_no_role_leak() {
    let content = select::content(OpenState::Closed, None, None, None, vec![], vec![]);
    let positioner = select::positioner(OpenState::Closed, vec![], vec![content]);
    let html = render(&positioner);
    assert!(html.contains(r#"data-state="closed""#));
    // positioner と content の両方に hidden が付く。
    assert_eq!(html.matches(r#"hidden="""#).count(), 2);
}

#[test]
fn dispatch_open_close_toggle_flip_data_state_across_parts() {
    let mut s = Select::default();
    assert!(!s.is_open());
    assert!(render(&s.content(None, None, None, vec![], vec![])).contains(r#"hidden="""#));

    assert!(dispatch(&mut s, "open", ""));
    assert!(s.is_open());
    assert!(render(&s.root(vec![], vec![])).contains(r#"data-state="open""#));
    assert!(
        render(&s.trigger(false, None, None, vec![], vec![])).contains(r#"aria-expanded="true""#)
    );
    assert!(render(&s.positioner(vec![], vec![])).contains(r#"data-state="open""#));
    assert!(!render(&s.content(None, None, None, vec![], vec![])).contains("hidden"));

    assert!(dispatch(&mut s, "close", ""));
    assert!(!s.is_open());

    assert!(dispatch(&mut s, "toggle", ""));
    assert!(s.is_open());
    assert!(dispatch(&mut s, "toggle", ""));
    assert!(!s.is_open());
}

#[test]
fn dispatch_select_updates_value_and_closes_listbox_close_on_select() {
    let mut s = Select::default();
    dispatch(&mut s, "open", "");
    assert!(s.is_open());

    assert!(dispatch(&mut s, "select", "vue"));
    assert_eq!(s.selected(), Some("vue"));
    assert!(!s.is_open());

    assert!(render(&s.item("vue", false, false, None, vec![], vec![]))
        .contains(r#"aria-selected="true""#));
    assert!(render(&s.item("react", false, false, None, vec![], vec![]))
        .contains(r#"aria-selected="false""#));
}

#[test]
fn dispatch_deselect_clears_selection() {
    let mut s = Select::default();
    dispatch(&mut s, "select", "vue");
    assert!(dispatch(&mut s, "deselect", ""));
    assert_eq!(s.selected(), None);
}

#[test]
fn dispatch_ignores_unknown_action() {
    let mut s = Select::default();
    dispatch(&mut s, "select", "vue");
    assert!(!dispatch(&mut s, "no_such_action", "x"));
    assert_eq!(s.selected(), Some("vue"));
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr() {
    let s = Select::default();
    let html = render(&s.view());
    assert!(!html.contains("data-hydrate-"));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let mut s = Select::default();
    dispatch(&mut s, "open", "");
    dispatch(&mut s, "select", "vue");
    // select は listbox を閉じるため、開いた状態のラウンドトリップを
    // 確認するために再度 open する。
    dispatch(&mut s, "open", "");

    let html = render(&render_for_hydration(&s));
    assert!(html.contains(r#"data-hydrate-state="open""#));
    assert!(html.contains("data-hydrate-selected="));

    let restored = Select::from_hydration_attrs(&s.hydration_attrs()).unwrap();
    assert_eq!(restored, s);
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
        ];
        let err = Select::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

#[test]
fn hydration_missing_attrs_returns_error_not_panic() {
    let err = Select::from_hydration_attrs(&[]).unwrap_err();
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
    ];
    let err = Select::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値すべてに攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn controls_labelledby_value_and_option_payloads_are_escaped_end_to_end() {
    let trigger = select::trigger(
        OpenState::Closed,
        false,
        Some(ATTR_BREAK_PAYLOAD),
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![],
    );
    let item = select::item(
        OpenState::Closed,
        false,
        false,
        ATTR_BREAK_PAYLOAD,
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![],
    );
    let content = select::content(
        OpenState::Closed,
        None,
        None,
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![],
    );
    let hidden_select = select::hidden_select(
        None,
        None,
        false,
        vec![],
        vec![(ATTR_BREAK_PAYLOAD, "<script>alert(1)</script>")],
    );
    let html = render(&select::root(
        OpenState::Closed,
        vec![],
        vec![trigger, item, content, hidden_select],
    ));

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&quot;"));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn dispatch_select_payload_is_escaped_end_to_end() {
    let mut s = Select::default();
    let payload = "\"><script>alert(1)</script>";
    assert!(dispatch(&mut s, "select", payload));

    let html = render(&render_for_hydration(&s));
    assert!(html.contains("data-hydrate-selected="));
    assert!(html.contains("&lt;script&gt;"));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(!html.contains(r#""><script"#));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let html = render(&select::root(
        OpenState::Closed,
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}
