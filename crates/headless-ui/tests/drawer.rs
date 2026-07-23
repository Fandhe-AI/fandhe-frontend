//! `fandhe-frontend-headless-ui` の Drawer 公開 API 統合テスト（イシュー #758）。
//!
//! `crates/headless-ui/src/drawer.rs` 内のユニットテストとは別に、クレート
//! 公開面（`fandhe_frontend_headless_ui::{drawer, Drawer, DrawerPlacement}`）
//! 経由で anatomy パーツ・placement・状態機械（dialog への委譲）・hydration・
//! XSS 回帰が期待通り動作することを固定する（`tests/dialog.rs` と同型の構成）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::dialog::ContentIds;
use fandhe_frontend_headless_ui::drawer::{self};
use fandhe_frontend_headless_ui::state::OpenState;
use fandhe_frontend_headless_ui::{Drawer, DrawerPlacement};
use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

#[test]
fn drawer_parts_output_expected_data_attrs_via_public_api() {
    let root_html = render(&drawer::root(
        OpenState::Closed,
        DrawerPlacement::Start,
        vec![],
        vec![],
    ));
    assert!(root_html.contains(r#"data-scope="drawer""#));
    assert!(root_html.contains(r#"data-part="root""#));
    assert!(root_html.contains(r#"data-state="closed""#));
    assert!(root_html.contains(r#"data-placement="start""#));

    let trigger_html = render(&drawer::trigger(
        OpenState::Closed,
        Some("dw1"),
        vec![],
        vec![],
    ));
    assert!(trigger_html.contains(r#"aria-haspopup="dialog""#));
    assert!(trigger_html.contains(r#"aria-controls="dw1""#));

    let content_html = render(&drawer::content(
        OpenState::Open,
        DrawerPlacement::End,
        true,
        ContentIds {
            id: Some("dw1"),
            labelledby: Some("dw1-title"),
            describedby: Some("dw1-desc"),
        },
        vec![],
        vec![text("body")],
    ));
    assert!(content_html.contains(r#"role="dialog""#));
    assert!(content_html.contains(r#"aria-modal="true""#));
    assert!(content_html.contains(r#"id="dw1""#));
    assert!(content_html.contains(r#"aria-labelledby="dw1-title""#));
    assert!(content_html.contains(r#"aria-describedby="dw1-desc""#));
    assert!(content_html.contains(r#"data-placement="end""#));
}

#[test]
fn drawer_all_placements_output_via_public_api() {
    for (placement, expected) in [
        (DrawerPlacement::Start, "start"),
        (DrawerPlacement::End, "end"),
        (DrawerPlacement::Top, "top"),
        (DrawerPlacement::Bottom, "bottom"),
    ] {
        let html = render(&drawer::positioner(
            OpenState::Open,
            placement,
            vec![],
            vec![],
        ));
        assert!(html.contains(&format!(r#"data-placement="{expected}""#)));
    }
}

#[test]
fn drawer_state_machine_dispatch_via_public_api_delegates_to_dialog() {
    let mut d = Drawer::default();
    assert_eq!(d.state(), OpenState::Closed);
    assert_eq!(d.placement(), DrawerPlacement::End);

    assert!(dispatch(&mut d, "open", ""));
    assert!(d.is_open());
    assert_eq!(d.data_state(), "open");

    assert!(dispatch(&mut d, "close", ""));
    assert!(!d.is_open());

    assert!(dispatch(&mut d, "toggle", ""));
    assert!(d.is_open());

    // 未知アクションは no-op。
    assert!(!dispatch(&mut d, "bogus", ""));
    assert!(d.is_open());
}

#[test]
fn drawer_convenience_methods_inject_current_state_and_placement() {
    let mut d = Drawer::new(OpenState::Closed, DrawerPlacement::Top);
    assert!(render(&d.positioner(vec![], vec![])).contains(r#"hidden="""#));
    assert!(render(&d.positioner(vec![], vec![])).contains(r#"data-placement="top""#));

    dispatch(&mut d, "open", "");
    assert!(!render(&d.positioner(vec![], vec![])).contains("hidden"));
    assert!(render(&d.backdrop(vec![], vec![])).contains(r#"data-state="open""#));
    assert!(
        render(&d.content(true, ContentIds::default(), vec![], vec![]))
            .contains(r#"data-state="open""#)
    );
}

#[test]
fn drawer_hydration_round_trip_via_public_api_carries_placement() {
    let d = Drawer::new(OpenState::Open, DrawerPlacement::Bottom);
    let rendered = render(&render_for_hydration(&d));
    assert!(rendered.contains(r#"data-hydrate-state="open""#));
    assert!(rendered.contains(r#"data-hydrate-placement="bottom""#));

    let restored = Drawer::from_hydration_attrs(&d.hydration_attrs()).unwrap();
    assert_eq!(restored, d);
}

#[test]
fn drawer_hydration_rejects_tampered_placement_without_panicking() {
    let attrs = vec![
        ("data-hydrate-state".to_string(), "open".to_string()),
        (
            "data-hydrate-placement".to_string(),
            "<script>alert(1)</script>".to_string(),
        ),
    ];
    let err = Drawer::from_hydration_attrs(&attrs).unwrap_err();
    assert!(format!("{err:?}").contains("InvalidValue"));
}

#[test]
fn drawer_xss_payload_in_title_id_is_escaped_on_render() {
    let payload = "\" onmouseover=\"alert(1)";
    let html = render(&drawer::title(Some(payload), vec![], vec![text("t")]));
    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn drawer_xss_payload_in_children_is_escaped_on_render() {
    let html = render(&drawer::description(
        None,
        vec![],
        vec![text("<img src=x onerror=alert(1)>")],
    ));
    assert!(!html.contains("<img src=x onerror=alert(1)>"));
    assert!(html.contains("&lt;img"));
}
