//! `fandhe-frontend-headless-ui` の Dialog 公開 API 統合テスト（イシュー #531）。
//!
//! `crates/headless-ui/src/dialog.rs` 内のユニットテストとは別に、クレート
//! 公開面（`fandhe_frontend_headless_ui::{dialog, Dialog}`）経由で anatomy
//! パーツ・状態機械・hydration・XSS 回帰が期待通り動作することを固定する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::dialog::{self, ContentIds, DialogRole};
use fandhe_frontend_headless_ui::state::OpenState;
use fandhe_frontend_headless_ui::Dialog;
use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

#[test]
fn dialog_parts_output_expected_data_attrs_via_public_api() {
    let root_html = render(&dialog::root(OpenState::Closed, vec![], vec![]));
    assert!(root_html.contains(r#"data-scope="dialog""#));
    assert!(root_html.contains(r#"data-part="root""#));
    assert!(root_html.contains(r#"data-state="closed""#));

    let trigger_html = render(&dialog::trigger(
        OpenState::Closed,
        Some("d1"),
        vec![],
        vec![],
    ));
    assert!(trigger_html.contains(r#"aria-haspopup="dialog""#));
    assert!(trigger_html.contains(r#"aria-controls="d1""#));

    let content_html = render(&dialog::content(
        OpenState::Open,
        DialogRole::Alertdialog,
        true,
        ContentIds {
            id: Some("d1"),
            labelledby: Some("d1-title"),
            describedby: Some("d1-desc"),
        },
        vec![],
        vec![text("body")],
    ));
    assert!(content_html.contains(r#"role="alertdialog""#));
    assert!(content_html.contains(r#"aria-modal="true""#));
    assert!(content_html.contains(r#"id="d1""#));
    assert!(content_html.contains(r#"aria-labelledby="d1-title""#));
    assert!(content_html.contains(r#"aria-describedby="d1-desc""#));
    assert!(content_html.contains(r#"tabindex="-1""#));
}

#[test]
fn dialog_state_machine_dispatch_via_public_api() {
    let mut d = Dialog::default();
    assert_eq!(d.state(), OpenState::Closed);

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
fn dialog_convenience_methods_inject_current_state() {
    let mut d = Dialog::default();
    assert!(render(&d.positioner(vec![], vec![])).contains(r#"hidden="""#));

    dispatch(&mut d, "open", "");
    assert!(!render(&d.positioner(vec![], vec![])).contains("hidden"));
    assert!(render(&d.backdrop(vec![], vec![])).contains(r#"data-state="open""#));
    assert!(render(&d.content(
        DialogRole::Dialog,
        true,
        ContentIds::default(),
        vec![],
        vec![]
    ))
    .contains(r#"data-state="open""#));
}

#[test]
fn dialog_hydration_round_trip_via_public_api() {
    let d = Dialog::new(OpenState::Open);
    let rendered = render(&render_for_hydration(&d));
    assert!(rendered.contains(r#"data-hydrate-state="open""#));

    let restored = Dialog::from_hydration_attrs(&d.hydration_attrs()).unwrap();
    assert_eq!(restored, d);
}

#[test]
fn dialog_hydration_rejects_tampered_state_without_panicking() {
    let attrs = vec![(
        "data-hydrate-state".to_string(),
        "<script>alert(1)</script>".to_string(),
    )];
    let err = Dialog::from_hydration_attrs(&attrs).unwrap_err();
    assert!(format!("{err:?}").contains("InvalidValue"));
}

#[test]
fn dialog_xss_payload_in_title_id_is_escaped_on_render() {
    let payload = "\" onmouseover=\"alert(1)";
    let html = render(&dialog::title(Some(payload), vec![], vec![text("t")]));
    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn dialog_xss_payload_in_children_is_escaped_on_render() {
    let html = render(&dialog::description(
        None,
        vec![],
        vec![text("<img src=x onerror=alert(1)>")],
    ));
    assert!(!html.contains("<img src=x onerror=alert(1)>"));
    assert!(html.contains("&lt;img"));
}
