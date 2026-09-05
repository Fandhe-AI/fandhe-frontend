//! Progress（イシュー #544）の統合テスト。
//!
//! `crates/headless-ui/src/progress.rs` の inline unit tests がパーツ単体の
//! 属性出力を固定するのに対し、本ファイルは「root > label + value_text +
//! track > range」の組み立て全体の data-*/ARIA 対応・SSR/hydration 両経路・
//! XSS 回帰をクレート外部から（公開 API のみを使って）固定する。
//!
//! Progress は [`switch::root`] のような自由関数を持たず、
//! [`Progress`] のメソッド（[`Progress::root`] 等）のみを公開 API とする
//! （`crates/headless-ui/src/progress.rs` モジュール doc §設計方針参照）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::data_attrs::Orientation;
use fandhe_frontend_headless_ui::{Progress, ProgressAction};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

#[test]
fn full_assembly_wires_root_label_value_text_track_and_range() {
    let p = Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal);

    let range = p.range(vec![], vec![]);
    let track = p.track(vec![], vec![range]);
    let label = p.label(vec![], vec![text("Upload progress")]);
    let value_text = p.value_text(vec![], vec![text("40%")]);
    let root = p.root(Some("40 percent"), vec![], vec![label, value_text, track]);

    let html = render(&root);
    assert!(html.contains(r#"data-scope="progress""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="label""#));
    assert!(html.contains(r#"data-part="value-text""#));
    assert!(html.contains(r#"data-part="track""#));
    assert!(html.contains(r#"data-part="range""#));
    assert!(html.contains(r#"data-state="loading""#));
    assert!(html.contains(r#"role="progressbar""#));
    assert!(html.contains(r#"aria-valuemin="0""#));
    assert!(html.contains(r#"aria-valuemax="100""#));
    assert!(html.contains(r#"aria-valuenow="40""#));
    assert!(html.contains(r#"aria-valuetext="40 percent""#));
    assert!(html.contains(r#"data-orientation="horizontal""#));
    // イシュー #1633 是正: label の data-orientation・value-text の aria-live。
    assert!(html.contains(r#"aria-live="polite""#));
    assert!(html.contains("Upload progress"));
    assert!(html.contains("40%"));
}

/// 意図的に非採用とした属性群が組み立て全体から見ても現れないことを
/// 固定する（イシュー #1633 突合の記録。unit tests 側の同名テストと対で
/// 「クレート外部から見た契約」を固定する）。
#[test]
fn intentionally_omitted_attributes_are_absent() {
    let p = Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal);

    let root_html = render(&p.root(None, vec![], vec![]));
    assert!(!root_html.contains("--percent"));
    assert!(!root_html.contains(r#"data-part="view""#));

    let track_html = render(&p.track(vec![], vec![]));
    assert!(!track_html.contains("role"));

    let range_html = render(&p.range(vec![], vec![]));
    assert!(!range_html.contains("role"));
    assert!(!range_html.contains("data-value"));
    assert!(!range_html.contains("data-max"));

    let circle_html = render(&p.circle(vec![], vec![]));
    assert!(!circle_html.contains("data-orientation"));
}

#[test]
fn dispatch_set_and_indeterminate_flip_data_state_across_parts() {
    let mut p = Progress::default();
    assert_eq!(p.value(), Some(0.0));

    assert!(dispatch(&mut p, "set", "75"));
    assert_eq!(p.value(), Some(75.0));
    assert!(render(&p.root(None, vec![], vec![])).contains(r#"data-state="loading""#));
    assert!(render(&p.track(vec![], vec![])).contains(r#"data-state="loading""#));
    assert!(render(&p.range(vec![], vec![])).contains(r#"data-state="loading""#));
    assert!(render(&p.label(vec![], vec![])).contains(r#"data-state="loading""#));
    assert!(render(&p.value_text(vec![], vec![])).contains(r#"data-state="loading""#));

    assert!(dispatch(&mut p, "set", "100"));
    assert!(render(&p.root(None, vec![], vec![])).contains(r#"data-state="complete""#));

    assert!(dispatch(&mut p, "indeterminate", ""));
    assert!(render(&p.root(None, vec![], vec![])).contains(r#"data-state="indeterminate""#));
    assert!(!render(&p.root(None, vec![], vec![])).contains("aria-valuenow"));

    assert!(!dispatch(&mut p, "no_such_action", "x"));
    assert_eq!(p.value(), None);
}

#[test]
fn decode_action_rejects_non_finite_and_unparsable_payload() {
    assert_eq!(
        Progress::decode_action("set", "42"),
        Some(ProgressAction::SetValue(42.0))
    );
    assert_eq!(Progress::decode_action("set", "NaN"), None);
    assert_eq!(Progress::decode_action("set", "inf"), None);
    assert_eq!(Progress::decode_action("set", "abc"), None);
    assert_eq!(
        Progress::decode_action("indeterminate", ""),
        Some(ProgressAction::SetIndeterminate)
    );
    assert_eq!(Progress::decode_action("unknown", ""), None);
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr() {
    let p = Progress::default();
    let html = render(&p.view());
    assert!(!html.contains("data-hydrate-"));
    assert!(html.contains(r#"data-state="loading""#));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let p = Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal);
    let html = render(&render_for_hydration(&p));
    assert!(html.contains(r#"data-hydrate-min="0""#));
    assert!(html.contains(r#"data-hydrate-max="100""#));
    assert!(html.contains(r#"data-hydrate-value="40""#));

    let restored = Progress::from_hydration_attrs(&p.hydration_attrs()).unwrap();
    assert_eq!(restored, p);
}

#[test]
fn hydration_tampered_value_returns_error_not_panic() {
    let attrs = vec![
        ("data-hydrate-min".to_string(), "0".to_string()),
        ("data-hydrate-max".to_string(), "100".to_string()),
        (
            "data-hydrate-value".to_string(),
            "<script>alert(1)</script>".to_string(),
        ),
        (
            "data-hydrate-orientation".to_string(),
            "horizontal".to_string(),
        ),
    ];
    let err = Progress::from_hydration_attrs(&attrs).unwrap_err();
    assert!(matches!(err, HydrateError::InvalidValue { .. }));
}

#[test]
fn hydration_missing_attrs_returns_error_not_panic() {
    let err = Progress::from_hydration_attrs(&[]).unwrap_err();
    assert_eq!(
        err,
        HydrateError::MissingAttr("data-hydrate-min".to_string())
    );
}

// --- Circular（Circle/CircleTrack/CircleRange, SVG, イシュー #600）---

#[test]
fn circular_assembly_wires_root_label_value_text_circle_track_range() {
    let p = Progress::new(0.0, 100.0, Some(40.0), Orientation::Horizontal);

    let circle_range = p.circle_range(vec![], vec![]);
    let circle_track = p.circle_track(vec![], vec![]);
    let circle = p.circle(vec![], vec![circle_track, circle_range]);
    let label = p.label(vec![], vec![text("Upload progress")]);
    let value_text = p.value_text(vec![], vec![text("40%")]);
    let root = p.root(Some("40 percent"), vec![], vec![label, value_text, circle]);

    let html = render(&root);
    assert!(html.contains(r#"data-scope="progress""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="label""#));
    assert!(html.contains(r#"data-part="value-text""#));
    assert!(html.contains(r#"data-part="circle""#));
    assert!(html.contains(r#"data-part="circle-track""#));
    assert!(html.contains(r#"data-part="circle-range""#));
    assert!(html.contains("<svg"));
    assert!(html.contains("<circle"));
    assert!(html.contains(r#"data-state="loading""#));
    assert!(html.contains(r#"role="progressbar""#));
    assert!(html.contains(r#"aria-valuenow="40""#));
    // circular パーツに data-orientation は付与しない（linear との意図的な差分）。
    assert!(!html.contains(r#"data-part="circle" data-orientation"#));
    assert!(html.contains("Upload progress"));
    assert!(html.contains("40%"));
}

#[test]
fn circular_indeterminate_assembly_omits_progress_style_values() {
    let p = Progress::new(0.0, 100.0, None, Orientation::Horizontal);

    let circle_range = p.circle_range(vec![], vec![]);
    let circle_track = p.circle_track(vec![], vec![]);
    let circle = p.circle(vec![], vec![circle_track, circle_range]);
    let root = p.root(None, vec![], vec![circle]);

    let html = render(&root);
    assert!(html.contains(r#"data-state="indeterminate""#));
    assert!(!html.contains("aria-valuenow"));
    assert!(!html.contains("--percent"));
    assert!(!html.contains("stroke-dasharray"));
    assert!(!html.contains("stroke-dashoffset"));
}

#[test]
fn circular_caller_attrs_and_children_payload_is_escaped_end_to_end() {
    let p = Progress::default();
    let circle_html = render(&p.circle(
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![text("<script>alert(1)</script>")],
    ));
    assert!(!circle_html.contains("onmouseover=\"alert(1)"));
    assert!(!circle_html.contains("<script>alert(1)</script>"));
    assert!(circle_html.contains("&lt;script&gt;"));
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値すべてに攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn aria_valuetext_payload_is_escaped_end_to_end() {
    let p = Progress::default();
    let html = render(&p.root(Some(ATTR_BREAK_PAYLOAD), vec![], vec![]));
    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let p = Progress::default();
    let html = render(&p.root(None, vec![("data-testid", ATTR_BREAK_PAYLOAD)], vec![]));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}

#[test]
fn children_text_payload_is_escaped_end_to_end() {
    let p = Progress::default();
    let html = render(&p.value_text(vec![], vec![text("<script>alert(1)</script>")]));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(html.contains("&lt;script&gt;"));
}
