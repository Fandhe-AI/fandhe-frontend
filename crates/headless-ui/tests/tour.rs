//! Tour（イシュー #841）の統合テスト。
//!
//! `crates/headless-ui/src/tour.rs` の inline unit tests がパーツ単体・
//! `Tour` 内部状態遷移を固定するのに対し、本ファイルは「root > backdrop +
//! spotlight + positioner(arrow + arrow-tip + content(title + description +
//! progress-text + close-trigger + action-trigger))」の組み立て全体・
//! dispatch 統合・SSR/hydration 両経路をクレート外部から（公開 API のみを
//! 使って）固定する。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::positioning::{Align, Placement, Side};
use fandhe_frontend_headless_ui::tour::{ContentIds, Tour, TourStatus, TourStep, TourTriggerKind};
use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

fn steps() -> Vec<TourStep> {
    vec![
        TourStep {
            id: "welcome".to_string(),
            target: Some("#welcome-panel".to_string()),
            title: "Welcome".to_string(),
            description: "This is the dashboard.".to_string(),
            placement: Placement::new(Side::Bottom, Align::Center),
        },
        TourStep {
            id: "settings".to_string(),
            target: Some("#settings-button".to_string()),
            title: "Settings".to_string(),
            description: "Configure your account here.".to_string(),
            placement: Placement::new(Side::Left, Align::Start),
        },
    ]
}

#[test]
fn full_assembly_wires_all_thirteen_anatomy_parts() {
    let mut tour = Tour::new(steps());
    assert!(dispatch(&mut tour, "start", ""));

    let node = tour.root(
        vec![],
        vec![
            tour.backdrop(vec![], vec![]),
            tour.spotlight(vec![], vec![]),
            tour.positioner(
                vec![],
                vec![
                    tour.arrow(vec![], vec![tour.arrow_tip(vec![], vec![])]),
                    tour.content(
                        ContentIds {
                            id: Some("tour-content"),
                            labelledby: Some("tour-title"),
                            describedby: Some("tour-desc"),
                        },
                        vec![],
                        vec![
                            tour.title(Some("tour-title"), vec![], vec![text("Welcome")]),
                            tour.description(
                                Some("tour-desc"),
                                vec![],
                                vec![text("This is the dashboard.")],
                            ),
                            tour.progress_text(vec![], vec![text("Step 1 of 2")]),
                            tour.control(
                                vec![],
                                vec![
                                    tour.action_trigger(
                                        TourTriggerKind::Prev,
                                        vec![],
                                        vec![text("Prev")],
                                    ),
                                    tour.action_trigger(
                                        TourTriggerKind::Next,
                                        vec![],
                                        vec![text("Next")],
                                    ),
                                    tour.close_trigger(vec![], vec![text("Close")]),
                                ],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    );

    let html = render(&node);
    for part in [
        "root",
        "backdrop",
        "spotlight",
        "positioner",
        "arrow",
        "arrow-tip",
        "content",
        "title",
        "description",
        "progress-text",
        "control",
        "close-trigger",
        "action-trigger",
    ] {
        assert!(
            html.contains(&format!(r#"data-part="{part}""#)),
            "missing part {part} in {html}"
        );
    }
    assert!(html.contains(r#"role="dialog""#));
    assert!(html.contains(r#"tabindex="-1""#));
    assert!(html.contains(r#"data-step="welcome""#));
    assert!(html.contains(r#"aria-live="polite""#));
    assert!(html.contains("data-target=\"#welcome-panel\""));
    assert!(html.contains(r#"data-side="bottom""#));
    assert!(html.contains(r#"data-align="center""#));
    // 最初の step のため Prev は disabled、Next は disabled ではない。
    assert!(html.contains(r#"data-type="prev" disabled="" data-disabled="""#));
    assert!(html.contains(r#"data-type="next""#));
}

#[test]
fn dispatch_transition_table_matches_module_contract() {
    let mut tour = Tour::new(steps());
    assert_eq!(tour.status(), TourStatus::Idle);

    assert!(dispatch(&mut tour, "start", ""));
    assert_eq!(tour.status(), TourStatus::Active { step: 0 });

    assert!(dispatch(&mut tour, "next", ""));
    assert_eq!(tour.status(), TourStatus::Active { step: 1 });

    assert!(dispatch(&mut tour, "next", ""));
    assert_eq!(tour.status(), TourStatus::Completed);

    // 終端状態からのいずれのアクションも no-op。
    for action in ["start", "next", "prev", "skip", "complete"] {
        assert!(dispatch(&mut tour, action, ""));
        assert_eq!(tour.status(), TourStatus::Completed);
    }
}

#[test]
fn skip_path_reaches_skipped_terminal_state() {
    let mut tour = Tour::new(steps());
    dispatch(&mut tour, "start", "");
    dispatch(&mut tour, "next", "");
    assert!(dispatch(&mut tour, "skip", ""));
    assert_eq!(tour.status(), TourStatus::Skipped);
}

#[test]
fn ssr_then_hydration_round_trip_across_dispatch() {
    let mut tour = Tour::new(steps());
    let ssr_html = render(&tour.root(vec![], vec![]));
    assert!(!ssr_html.contains("data-hydrate-"));
    assert!(ssr_html.contains(r#"data-status="idle""#));

    dispatch(&mut tour, "start", "");
    dispatch(&mut tour, "next", "");

    let hydrate_html = render(&render_for_hydration(&tour));
    assert!(hydrate_html.contains(r#"data-hydrate-status="active""#));
    assert!(hydrate_html.contains(r#"data-hydrate-step="1""#));

    let restored = Tour::from_hydration_attrs(&tour.hydration_attrs()).unwrap();
    assert_eq!(restored, tour);
}

#[test]
fn empty_steps_start_completes_without_panicking() {
    let mut tour = Tour::new(Vec::new());
    assert!(dispatch(&mut tour, "start", ""));
    assert_eq!(tour.status(), TourStatus::Completed);
    // Completed でも root/content の描画は成立する（overlay は closed 表示）。
    let html = render(&tour.root(
        vec![],
        vec![tour.content(ContentIds::default(), vec![], vec![])],
    ));
    assert!(html.contains(r#"data-status="completed""#));
    assert!(html.contains("hidden"));
}
