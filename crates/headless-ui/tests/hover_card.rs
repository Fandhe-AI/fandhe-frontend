//! HoverCard（イシュー #759）の統合テスト。
//!
//! `crates/headless-ui/src/hover_card.rs` の inline unit tests がパーツ単体の
//! 属性出力を固定するのに対し、本ファイルは
//! 「root > trigger + positioner > (content + arrow)」の組み立て全体の
//! data-* 対応・dispatch 統合・SSR/hydration 両経路・positioning 接続・XSS
//! 回帰をクレート外部から（公開 API のみを使って）固定する。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::hover_card::{self, HoverCard, HoverCardDelays};
use fandhe_frontend_headless_ui::{
    compute_position, css_vars_style, placement_attrs, Align, OpenState, Placement,
    PositioningConfig, Rect, Side, Size,
};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

#[test]
fn full_assembly_wires_all_parts_and_delays() {
    let trigger = hover_card::trigger(
        OpenState::Open,
        Some("https://example.com/preview"),
        vec![],
        vec![],
    );
    let arrow = hover_card::arrow(vec![], vec![hover_card::arrow_tip(vec![], vec![])]);
    let content = hover_card::content(OpenState::Open, None, vec![], vec![arrow]);
    let positioner = hover_card::positioner(OpenState::Open, vec![], vec![content]);
    let root = hover_card::root(
        OpenState::Open,
        HoverCardDelays::default(),
        vec![],
        vec![trigger, positioner],
    );

    let html = render(&root);
    assert!(html.contains(r#"data-scope="hover-card""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="trigger""#));
    assert!(html.contains(r#"data-part="positioner""#));
    assert!(html.contains(r#"data-part="content""#));
    assert!(html.contains(r#"data-part="arrow""#));
    assert!(html.contains(r#"data-part="arrow-tip""#));
    assert!(html.contains(r#"data-open-delay="600""#));
    assert!(html.contains(r#"data-close-delay="300""#));
    assert!(html.contains(r#"href="https://example.com/preview""#));
    // WAI-ARIA に hover card 専用パターンは存在しないため使わない。
    assert!(!html.contains("aria-expanded"));
    assert!(!html.contains("aria-controls"));
    assert!(!html.contains("aria-haspopup"));
    assert!(!html.contains("role=\"tooltip\""));
    // open 状態のため content/positioner に hidden 存在属性は付かない。arrow
    // の aria-hidden="true"（substring として "hidden" を含む）とは区別する
    // ため、hidden 存在属性の具体的な出現形で判定する。
    assert!(!html.contains(r#" hidden="""#));
}

#[test]
fn positioner_connects_to_positioning_module_via_attrs() {
    let anchor = Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 30.0,
    };
    let floating = Size {
        width: 240.0,
        height: 120.0,
    };
    let viewport = Size {
        width: 1024.0,
        height: 768.0,
    };
    let config = PositioningConfig {
        placement: Placement::new(Side::Right, Align::Center),
        offset: 8.0,
        flip: true,
        shift: true,
        same_width: false,
    };
    let resolved = compute_position(anchor, floating, viewport, &config, true);
    let style = css_vars_style(&resolved, anchor.width, config.same_width);
    let mut attrs: Vec<(&str, &str)> = vec![("style", &style)];
    attrs.extend(placement_attrs(resolved.placement));

    let html = render(&hover_card::positioner(OpenState::Open, attrs, vec![]));
    assert!(html.contains("--fandhe-arrow-x:") || html.contains("--fandhe-arrow-y:"));
    assert!(html.contains(r#"data-side="right""#));
}

#[test]
fn dispatch_toggle_flips_data_state_across_parts() {
    let mut hc = HoverCard::default();
    assert_eq!(hc.state(), OpenState::Closed);
    assert!(render(&hc.content(None, vec![], vec![])).contains(r#"hidden="""#));

    assert!(dispatch(&mut hc, "toggle", ""));
    assert!(render(&hc.root(vec![], vec![])).contains(r#"data-state="open""#));
    assert!(!render(&hc.content(None, vec![], vec![])).contains("hidden"));

    assert!(dispatch(&mut hc, "open", ""));
    assert_eq!(hc.state(), OpenState::Open);

    assert!(dispatch(&mut hc, "close", ""));
    assert_eq!(hc.state(), OpenState::Closed);

    assert!(!dispatch(&mut hc, "no_such_action", ""));
    assert_eq!(hc.state(), OpenState::Closed);
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr() {
    let hc = HoverCard::default();
    let html = render(&hc.view());
    assert!(!html.contains("data-hydrate-"));
    assert!(html.contains(r#"data-state="closed""#));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let hc = HoverCard::new(OpenState::Open, HoverCardDelays::default());
    let html = render(&render_for_hydration(&hc));
    assert!(html.contains(r#"data-hydrate-state="open""#));

    let restored = HoverCard::from_hydration_attrs(&hc.hydration_attrs()).unwrap();
    assert_eq!(restored, hc);
}

#[test]
fn hydration_does_not_carry_delays() {
    // delays は SSR 静的設定であり hydration 属性へ含まれない（モジュール doc
    // §遅延設定値参照）。非既定値で生成しても hydration ラウンドトリップ後は
    // 既定値へ戻ることを公開 API 経由でも固定する。
    let hc = HoverCard::new(
        OpenState::Open,
        HoverCardDelays {
            open_ms: 1200,
            close_ms: 50,
        },
    );
    let restored = HoverCard::from_hydration_attrs(&hc.hydration_attrs()).unwrap();
    assert_eq!(restored.delays(), HoverCardDelays::default());
}

#[test]
fn hydration_tampered_value_returns_error_not_panic() {
    for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
        let attrs = vec![("data-hydrate-state".to_string(), bogus.to_string())];
        let err = HoverCard::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値すべてに攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn href_and_content_id_payloads_are_escaped_end_to_end() {
    let trigger = hover_card::trigger(OpenState::Closed, Some(ATTR_BREAK_PAYLOAD), vec![], vec![]);
    let content = hover_card::content(OpenState::Closed, Some(ATTR_BREAK_PAYLOAD), vec![], vec![]);
    let html = render(&hover_card::root(
        OpenState::Closed,
        HoverCardDelays::default(),
        vec![],
        vec![trigger, content],
    ));

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let html = render(&hover_card::root(
        OpenState::Closed,
        HoverCardDelays::default(),
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}

#[test]
fn javascript_scheme_href_is_dropped_end_to_end() {
    let html = render(&hover_card::trigger(
        OpenState::Closed,
        Some("javascript:alert(1)"),
        vec![],
        vec![],
    ));
    assert!(!html.contains("javascript:"));
    assert!(!html.contains("href="));
}
