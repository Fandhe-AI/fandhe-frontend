//! HoverCard（イシュー #759）の統合テスト。
//!
//! `crates/headless-ui/src/hover_card.rs` の inline unit tests がパーツ単体の
//! 属性出力を固定するのに対し、本ファイルは
//! 「root > trigger + positioner > (content + arrow)」の組み立て全体の
//! data-* 対応・dispatch 統合・SSR/hydration 両経路・positioning 接続・XSS
//! 回帰をクレート外部から（公開 API のみを使って）固定する。
//!
//! # 参照突合（イシュー #1641）
//!
//! Zag.js `hover-card.connect.ts`（GitHub `main`、取得日 2026-09-05）・
//! ark-ui docs・Radix Primitives（Hover Card）docs・Radix Themes・chakra-ui
//! の一次情報と本実装を突合した結果、**両参照に対する具体的な欠落は無く、
//! `crates/headless-ui/src/hover_card.rs` の是正は不要**と判定した
//! （差分メモの詳細はイシュー #1641 コメント参照）。
//!
//! 是正なしの根拠（意図的差分として `src/` を変更しなかった項目）:
//!
//! - anatomy: ark-ui の Root > Trigger > Positioner > (Arrow > ArrowTip) +
//!   Content の 6 パートと完全一致。Radix の `Portal` は JS ランタイム固有
//!   utility として全体方針で非採用（`docs/policy/intentional-non-adoption.md`
//!   §3.23）。
//! - `data-side`/`data-align`: Radix は `content` へ、Zag は
//!   `trigger`/`content` の双方へ出すが、本実装は `positioning`（#590）の
//!   規約どおり [`crate::hover_card::positioner`] へ透過させる
//!   （[`mod@crate::tooltip`]/[`mod@crate::popover`] と同型。hover-card だけを
//!   変えると規約が分裂するため意図的差分）。
//! - ARIA: `trigger`/`content` に `role`/`aria-*` を付けない点は Zag・Radix
//!   いずれとも一致。
//! - `tabindex="-1"`（content）・`dir`・自動 `id`・複数トリガーの
//!   `data-value`/`data-current`/`data-ownedby`（Zag のみ）: hover/focus の
//!   タイマー・DOM 配線が `fandhe-frontend-wasm-full` に未実装の段階で
//!   `tabindex` を固定付与しない（tags-input #1623 と同型の判断）。複数
//!   トリガーは機能拡張でありスコープ外候補（下記参照）。
//! - Keyboard: [`trigger`](crate::hover_card::trigger) はネイティブ `a` 要素
//!   であり、`href` が `Some` のときブラウザ標準で Tab フォーカス到達・
//!   Enter によるリンク遷移が成立する（Radix の「Tab で hover card を開閉」
//!   相当の focus/blur 配線は `fandhe-frontend-wasm-full` 側の責務で未配線）。
//!
//! `data-*`・パートの増減は無いため、Themes 側イシュー #1523（closed）への
//! 追加コメントは不要と判断した。

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

// --- 参照突合（イシュー #1641）: 意図的差分を機械固定する回帰テスト ---

#[test]
fn trigger_is_anchor_and_omits_href_when_none() {
    let html = render(&hover_card::trigger(
        OpenState::Closed,
        None,
        vec![],
        vec![],
    ));
    assert!(html.starts_with("<a"));
    assert!(!html.contains("href="));
}

#[test]
fn trigger_and_content_carry_only_reference_aligned_attrs() {
    // Zag.js/Radix が出すが本実装が意図的に付与しない属性（§参照突合参照）
    // が trigger/content のいずれにも現れないことを固定する。
    let trigger = hover_card::trigger(
        OpenState::Open,
        Some("https://example.com/preview"),
        vec![],
        vec![],
    );
    let content = hover_card::content(OpenState::Open, Some("hc-content"), vec![], vec![]);
    let html = render(&hover_card::root(
        OpenState::Open,
        HoverCardDelays::default(),
        vec![],
        vec![trigger, content],
    ));

    for absent in [
        "data-placement",
        "data-side",
        "data-align",
        "tabindex",
        " dir=",
        "data-value",
        "data-current",
        "data-ownedby",
        "role=",
        "aria-expanded",
        "aria-controls",
        "aria-haspopup",
    ] {
        assert!(
            !html.contains(absent),
            "unexpected attribute present: {absent}"
        );
    }
}

#[test]
fn closed_state_hides_positioner_and_content() {
    let content = hover_card::content(OpenState::Closed, None, vec![], vec![]);
    let positioner = hover_card::positioner(OpenState::Closed, vec![], vec![content]);
    let html = render(&positioner);

    assert_eq!(html.matches(r#" hidden="""#).count(), 2);
    assert!(html.contains(r#"data-part="positioner" data-state="closed""#));
    assert!(html.contains(r#"data-part="content" data-state="closed""#));
}

#[test]
fn placement_attrs_reach_positioner_not_content() {
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
        placement: Placement::new(Side::Top, Align::Start),
        offset: 8.0,
        flip: true,
        shift: true,
        same_width: false,
    };
    let resolved = compute_position(anchor, floating, viewport, &config, true);
    let side_align: Vec<(&str, &str)> = placement_attrs(resolved.placement).to_vec();
    let expected_side = format!(r#"data-side="{}""#, resolved.placement.side().as_str());
    let expected_align = format!(r#"data-align="{}""#, resolved.placement.align().as_str());

    let positioner_html = render(&hover_card::positioner(OpenState::Open, side_align, vec![]));
    assert!(positioner_html.contains(&expected_side));
    assert!(positioner_html.contains(&expected_align));

    // content 側には positioner 由来の data-side/data-align を渡さない設計
    // （呼び出し側が誤って content へ渡しても、本テストは positioner 経由の
    // 配線のみが規約であることを固定する）。
    let content_html = render(&hover_card::content(OpenState::Open, None, vec![], vec![]));
    assert!(!content_html.contains("data-side"));
    assert!(!content_html.contains("data-align"));
}

#[test]
fn caller_attrs_cannot_spoof_scope_and_part() {
    let html = render(&hover_card::root(
        OpenState::Closed,
        HoverCardDelays::default(),
        vec![("data-scope", "attacker"), ("data-part", "attacker")],
        vec![],
    ));
    assert!(html.contains(r#"data-scope="hover-card""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(!html.contains("attacker"));
}

#[test]
fn arrow_parts_are_aria_hidden_and_stateless() {
    let arrow_html = render(&hover_card::arrow(vec![], vec![]));
    let arrow_tip_html = render(&hover_card::arrow_tip(vec![], vec![]));

    for html in [&arrow_html, &arrow_tip_html] {
        assert!(html.contains(r#"aria-hidden="true""#));
        assert!(!html.contains("data-state"));
    }
}
