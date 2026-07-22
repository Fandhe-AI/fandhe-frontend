//! Avatar（イシュー #543）の統合テスト。
//!
//! `crates/headless-ui/src/avatar.rs` の inline unit tests がパーツ単体の
//! 属性出力を固定するのに対し、本ファイルは「root > image + fallback」の
//! 組み立て全体の data-* 出力・dispatch 統合・SSR/hydration 両経路・XSS 回帰を
//! クレート外部から（公開 API のみを使って）固定する。

use fandhe_frontend_core::{render, text, Node};
use fandhe_frontend_headless_ui::avatar::{self, Avatar, ImageStatus};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

#[test]
fn full_assembly_wires_root_image_and_fallback() {
    let image = avatar::image(ImageStatus::Loaded, "/avatar.png", "Naoko Miyazaki", vec![]);
    let fallback = avatar::fallback(ImageStatus::Loaded, vec![], vec![text("NM")]);
    let root = avatar::root(vec![], vec![image, fallback]);

    let html = render(&root);
    assert!(html.contains(r#"data-scope="avatar""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="image""#));
    assert!(html.contains(r#"data-part="fallback""#));
    assert!(html.contains(r#"src="/avatar.png""#));
    assert!(html.contains(r#"alt="Naoko Miyazaki""#));
    assert!(html.contains(r#"data-state="visible""#)); // image
    assert!(html.contains(r#"data-state="hidden""#)); // fallback
    assert!(html.contains("NM"));
}

#[test]
fn dispatch_transitions_flip_visibility_across_parts() {
    let mut a = Avatar::default();
    assert_eq!(a.status(), ImageStatus::Loading);
    assert!(render(&a.image("/a.png", "avatar", vec![])).contains(r#"data-state="hidden""#));
    assert!(render(&a.fallback(vec![], vec![])).contains(r#"data-state="visible""#));

    assert!(dispatch(&mut a, "loaded", ""));
    assert_eq!(a.status(), ImageStatus::Loaded);
    assert!(render(&a.image("/a.png", "avatar", vec![])).contains(r#"data-state="visible""#));
    assert!(render(&a.fallback(vec![], vec![])).contains(r#"data-state="hidden""#));

    assert!(dispatch(&mut a, "error", ""));
    assert_eq!(a.status(), ImageStatus::Error);
    assert!(render(&a.image("/a.png", "avatar", vec![])).contains(r#"data-state="hidden""#));
    assert!(render(&a.fallback(vec![], vec![])).contains(r#"data-state="visible""#));

    assert!(dispatch(&mut a, "reset", ""));
    assert_eq!(a.status(), ImageStatus::Loading);

    assert!(!dispatch(&mut a, "no_such_action", ""));
    assert_eq!(a.status(), ImageStatus::Loading);
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr() {
    let a = Avatar::default();
    let html = render(&a.view());
    assert!(!html.contains("data-hydrate-"));
    assert!(html.contains(r#"data-state="hidden""#)); // fallback visible => image hidden
}

#[test]
fn view_root_is_element_node_for_hydration() {
    // render_for_hydration の前提（ルートが Node::Element であること）を
    // 公開 API 経由で固定する。
    assert!(matches!(Avatar::default().view(), Node::Element { .. }));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let a = Avatar::new(ImageStatus::Loaded);
    let html = render(&render_for_hydration(&a));
    assert!(html.contains(r#"data-hydrate-status="loaded""#));

    let restored = Avatar::from_hydration_attrs(&a.hydration_attrs()).unwrap();
    assert_eq!(restored, a);
}

#[test]
fn hydration_tampered_value_returns_error_not_panic() {
    for bogus in ["LOADED", "<script>alert(1)</script>", ""] {
        let attrs = vec![("data-hydrate-status".to_string(), bogus.to_string())];
        let err = Avatar::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

#[test]
fn hydration_missing_attr_returns_error_not_panic() {
    let err = Avatar::from_hydration_attrs(&[]).unwrap_err();
    assert_eq!(
        err,
        HydrateError::MissingAttr("data-hydrate-status".to_string())
    );
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値すべてに攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";
const SCRIPT_PAYLOAD: &str = "<script>alert(1)</script>";

#[test]
fn src_alt_payloads_are_escaped_end_to_end() {
    let image = avatar::image(
        ImageStatus::Loaded,
        ATTR_BREAK_PAYLOAD,
        ATTR_BREAK_PAYLOAD,
        vec![],
    );
    let html = render(&avatar::root(vec![], vec![image]));

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let html = render(&avatar::root(
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}

#[test]
fn fallback_children_script_payload_is_escaped_end_to_end() {
    let fallback = avatar::fallback(ImageStatus::Loading, vec![], vec![text(SCRIPT_PAYLOAD)]);
    let html = render(&avatar::root(vec![], vec![fallback]));

    assert!(!html.contains(SCRIPT_PAYLOAD));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn caller_supplied_scope_and_part_are_dropped_end_to_end() {
    let html = render(&avatar::root(
        vec![("data-scope", "attacker"), ("data-part", "attacker")],
        vec![],
    ));
    assert!(html.contains(r#"data-scope="avatar""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(!html.contains("attacker"));
}
