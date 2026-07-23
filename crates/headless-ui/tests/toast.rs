//! Toast（イシュー #760）の統合テスト。
//!
//! `crates/headless-ui/src/toast.rs` の inline unit tests がパーツ単体・
//! `Toaster` 内部状態遷移を固定するのに対し、本ファイルは「group > root >
//! title + description + close-trigger」の組み立て全体・dispatch 統合・
//! SSR/hydration 両経路・XSS 回帰をクレート外部から（公開 API のみを使って）
//! 固定する。

use fandhe_frontend_core::{render, text, Node};
use fandhe_frontend_headless_ui::toast::{
    self, ToastAction, ToastEntry, ToastPlacement, ToastStatus, Toaster,
};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

fn entry(id: &str, status: ToastStatus, title: &str, description: &str) -> ToastEntry {
    ToastEntry {
        id: id.to_string(),
        status,
        title: title.to_string(),
        description: description.to_string(),
    }
}

#[test]
fn full_assembly_wires_group_root_title_description_and_close_trigger() {
    let node = toast::group(
        ToastPlacement::BottomEnd,
        "Notifications",
        vec![],
        vec![toast::root(
            ToastStatus::Success,
            vec![],
            vec![
                toast::title(vec![], vec![text("Saved")]),
                toast::description(vec![], vec![text("Your changes were saved.")]),
                toast::close_trigger(vec![], vec![text("Close")]),
            ],
        )],
    );

    let html = render(&node);
    assert!(html.contains(r#"data-scope="toast""#));
    assert!(html.contains(r#"data-part="group""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="title""#));
    assert!(html.contains(r#"data-part="description""#));
    assert!(html.contains(r#"data-part="close-trigger""#));
    assert!(html.contains(r#"role="region""#));
    assert!(html.contains(r#"role="status""#));
    assert!(html.contains(r#"data-type="success""#));
    assert!(html.contains("Saved"));
}

#[test]
fn action_trigger_part_is_present_and_typed_button() {
    let html = render(&toast::action_trigger(vec![], vec![text("Undo")]));
    assert!(html.contains(r#"data-part="action-trigger""#));
    assert!(html.contains(r#"type="button""#));
    assert!(html.contains("Undo"));
}

#[test]
fn dispatch_push_then_dismiss_via_typed_action() {
    let mut t = Toaster::new(5, ToastPlacement::Bottom);
    t.update(ToastAction::Push(entry("a", ToastStatus::Info, "A", "")));
    assert_eq!(t.entries().len(), 1);

    assert!(dispatch(&mut t, "dismiss", "a"));
    assert!(t.entries().is_empty());
}

#[test]
fn dispatch_ignores_push_string_action() {
    // 本モジュール冒頭の rustdoc「スコープ外」節: 文字列 dispatch は
    // "push" を受理しない（型付き Component::update のみが Push を扱える）。
    let mut t = Toaster::default();
    assert!(!dispatch(&mut t, "push", "irrelevant"));
    assert!(t.entries().is_empty());
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr() {
    let t = Toaster::default();
    let html = render(&t.view());
    assert!(!html.contains("data-hydrate-"));
    assert!(html.contains(r#"data-scope="toast""#));
}

#[test]
fn view_root_is_element_node_for_hydration() {
    assert!(matches!(Toaster::default().view(), Node::Element { .. }));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let mut t = Toaster::new(3, ToastPlacement::TopStart);
    t.push(entry("a", ToastStatus::Warning, "Heads up", "check this"));

    let html = render(&render_for_hydration(&t));
    assert!(html.contains("data-hydrate-ids="));
    assert!(html.contains(r#"data-hydrate-placement="top-start""#));

    let restored = Toaster::from_hydration_attrs(&t.hydration_attrs()).unwrap();
    assert_eq!(restored, t);
}

#[test]
fn hydration_tampered_placement_returns_error_not_panic() {
    let attrs = vec![
        ("data-hydrate-ids".to_string(), String::new()),
        ("data-hydrate-statuses".to_string(), String::new()),
        ("data-hydrate-titles".to_string(), String::new()),
        ("data-hydrate-descriptions".to_string(), String::new()),
        ("data-hydrate-max".to_string(), "5".to_string()),
        (
            "data-hydrate-placement".to_string(),
            "<script>alert(1)</script>".to_string(),
        ),
    ];
    let err = Toaster::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

#[test]
fn hydration_missing_attr_returns_error_not_panic() {
    let err = Toaster::from_hydration_attrs(&[]).unwrap_err();
    assert_eq!(
        err,
        HydrateError::MissingAttr("data-hydrate-ids".to_string())
    );
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値すべてに攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";
const SCRIPT_PAYLOAD: &str = "<script>alert(1)</script>";

#[test]
fn group_label_payload_is_escaped_end_to_end() {
    let html = render(&toast::group(
        ToastPlacement::Bottom,
        ATTR_BREAK_PAYLOAD,
        vec![],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let html = render(&toast::root(
        ToastStatus::Info,
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}

#[test]
fn title_description_script_payload_is_escaped_end_to_end() {
    let node = toast::root(
        ToastStatus::Error,
        vec![],
        vec![
            toast::title(vec![], vec![text(SCRIPT_PAYLOAD)]),
            toast::description(vec![], vec![text(SCRIPT_PAYLOAD)]),
        ],
    );
    let html = render(&node);
    assert!(!html.contains(SCRIPT_PAYLOAD));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn caller_supplied_scope_and_part_are_dropped_end_to_end() {
    let html = render(&toast::root(
        ToastStatus::Info,
        vec![("data-scope", "attacker"), ("data-part", "attacker")],
        vec![],
    ));
    assert!(html.contains(r#"data-scope="toast""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(!html.contains("attacker"));
}

#[test]
fn push_entry_with_xss_payload_is_escaped_in_view() {
    let mut t = Toaster::new(5, ToastPlacement::Bottom);
    t.push(entry(
        "a",
        ToastStatus::Info,
        SCRIPT_PAYLOAD,
        SCRIPT_PAYLOAD,
    ));
    let html = render(&t.view());
    assert!(!html.contains(SCRIPT_PAYLOAD));
    assert!(html.contains("&lt;script&gt;"));
}
