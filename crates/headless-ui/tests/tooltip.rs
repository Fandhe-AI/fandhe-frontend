//! Tooltip（イシュー #533）の統合テスト。
//!
//! `crates/headless-ui/src/tooltip.rs` の inline unit tests がパーツ単体の
//! 属性出力を固定するのに対し、本ファイルは
//! 「root > trigger + positioner > (content + arrow)」の組み立て全体の
//! data-*/ARIA 対応（describedby↔id）・dispatch 統合・SSR/hydration
//! 両経路・XSS 回帰をクレート外部から（公開 API のみを使って）固定する。
//!
//! # 参照突合（イシュー #1645）
//!
//! ark-ui（`.claude/skills/ark-ui/references/components/overlays/tooltip.md`）・
//! chakra-ui・Radix Primitives（`docs/design/radix-primitives-inventory.md`
//! 96 行目）と突合し、**具体的な欠落は見つからず、
//! `crates/headless-ui/src/tooltip.rs` の是正は不要**と判定した（差分メモの
//! 詳細はイシュー #1645 コメント参照）。
//!
//! 是正なしの根拠（意図的差分として `src/` を変更しなかった項目）:
//!
//! - anatomy: ark-ui/chakra の Root/Trigger/Positioner/Content/Arrow/
//!   ArrowTip の 6 パーツ、Radix の Provider > Root > Trigger > Portal >
//!   Content > Arrow と完全一致（`Provider`/`Portal` は遅延設定共有・DOM
//!   配置の関心のため全体方針で不採用、`docs/policy/intentional-non-adoption.md`
//!   §3.25、hover-card #1641・popover #1642・toggle-tip #1644 と同型の判断）。
//! - `data-*`: `data-state`（`"open"`/`"closed"`）・`data-disabled` は同義。
//!   zag/chakra の `data-expanded`（`data-state` と重複）・
//!   `data-placement`/Radix の `[data-side]`/`[data-align]`（→ positioner の
//!   `data-side`/`data-align`、#590 `positioning` が同役割）・Radix の
//!   `[data-state]` 語彙 `delayed-open`/`instant-open`（本実装は
//!   `OpenState` 語彙統一を優先）は toggle-tip #1644・popover #1642・
//!   hover-card #1641 と同じ判断軸で意図的に非採用。パート・`data-*` の
//!   増減は無いため、Themes 側イシュー #1548（closed）への追加コメントは
//!   行っていない。
//! - ARIA: `aria-describedby`（trigger）↔ `role="tooltip"`（content）は
//!   一致。zag/Radix は `aria-describedby` を open 時のみ出力するが、
//!   本実装は `describedby` が `Some` のとき状態に関係なく常時出力する
//!   （SSR 静的出力の性質上。`hidden` な参照先も accessible description の
//!   算出に含まれるため害はなく、むしろ SSR/no-JS で説明が結び付く利点が
//!   ある。意図的差分）。
//! - キーボード: `fandhe-frontend-wasm-full` の
//!   [`tooltip::wiring::TooltipDelayController::register_tooltip`]
//!   （trigger の `focusin`/`focusout` → 即時 open/close 要求、
//!   `tests/tooltip_delay_browser.rs::focus_opens_and_blur_closes_immediately_ignoring_delay`
//!   でブラウザ実測済み）・[`overlay::OverlayKind::Tooltip`]
//!   （`close_on_escape()` が `true`）・`headless::MAPPING_TABLE`
//!   （trigger → `"toggle"`）が実 DOM 配線を担う。参照（ark
//!   `closeOnClick: true`、Radix の Space/Enter）は「閉じるのみ」だが
//!   本実装の trigger click は `"toggle"`（開閉反転）であり、この差は
//!   headless-ui の anatomy 欠落ではなく wasm-full 側の挙動差として記録する
//!   （本イシュー対象外、スコープ外候補）。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::tooltip::{self, Tooltip};
use fandhe_frontend_headless_ui::{
    compute_position, css_vars_style, placement_attrs, Align, OpenState, Placement,
    PositioningConfig, Rect, Side, Size,
};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

#[test]
fn full_assembly_wires_aria_describedby_to_content_id() {
    let trigger = tooltip::trigger(OpenState::Open, false, Some("tip-1"), vec![], vec![]);
    let arrow = tooltip::arrow(vec![], vec![tooltip::arrow_tip(vec![], vec![])]);
    let content = tooltip::content(OpenState::Open, Some("tip-1"), vec![], vec![arrow]);
    let positioner = tooltip::positioner(OpenState::Open, vec![], vec![content]);
    let root = tooltip::root(OpenState::Open, vec![], vec![trigger, positioner]);

    let html = render(&root);
    assert!(html.contains(r#"data-scope="tooltip""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="trigger""#));
    assert!(html.contains(r#"data-part="positioner""#));
    assert!(html.contains(r#"data-part="content""#));
    assert!(html.contains(r#"data-part="arrow""#));
    assert!(html.contains(r#"data-part="arrow-tip""#));
    assert!(html.contains(r#"role="tooltip""#));
    assert!(html.contains(r#"aria-describedby="tip-1""#));
    assert!(html.contains(r#"id="tip-1""#));
    // tooltip パターンでは aria-expanded/aria-controls を使わない。
    assert!(!html.contains("aria-expanded"));
    assert!(!html.contains("aria-controls"));
    // open 状態のため content に hidden 存在属性は付かない。arrow の
    // aria-hidden="true"（装飾目的、substring として "hidden" を含む）とは
    // 区別するため、hidden 存在属性の具体的な出現形で判定する。
    assert!(!html.contains(r#" hidden="""#));
}

#[test]
fn trigger_is_native_button_with_type_button_and_aria_describedby() {
    let html = render(&tooltip::trigger(
        OpenState::Closed,
        false,
        Some("tip-2"),
        vec![],
        vec![],
    ));
    assert!(html.trim_start().starts_with("<button"));
    assert!(html.contains(r#"type="button""#));
    assert!(html.contains(r#"aria-describedby="tip-2""#));
}

#[test]
fn trigger_and_content_carry_only_reference_aligned_attrs() {
    // 参照サイト（ark-ui/chakra-ui/Radix）と突合した結果、tooltip パターン
    // では不要と判断した属性群（disclosure 系の aria-expanded/
    // aria-controls、zag/Radix の data-expanded/data-placement、他部品の
    // tabindex/data-ownedby/data-value/data-current）が混入していないこと
    // を固定する（イシュー #1645）。content の role="tooltip" は仕様どおり
    // 存在するため除外リストへ入れない。
    let trigger = tooltip::trigger(OpenState::Open, false, Some("tip-3"), vec![], vec![]);
    let content = tooltip::content(OpenState::Open, Some("tip-3"), vec![], vec![]);
    let html = render(&trigger) + &render(&content);

    for forbidden in [
        "aria-expanded",
        "aria-controls",
        "data-expanded",
        "data-placement",
        "tabindex",
        "data-ownedby",
        "data-value",
        "data-current",
    ] {
        assert!(
            !html.contains(forbidden),
            "unexpected attribute `{forbidden}` in tooltip trigger/content output"
        );
    }
}

#[test]
fn closed_state_hides_positioner_and_content() {
    let content = tooltip::content(OpenState::Closed, Some("tip-4"), vec![], vec![]);
    let positioner = tooltip::positioner(OpenState::Closed, vec![], vec![content]);
    let html = render(&positioner);

    assert_eq!(html.matches(r#"hidden="""#).count(), 2);
    assert_eq!(html.matches(r#"data-state="closed""#).count(), 2);
}

#[test]
fn disabled_trigger_emits_native_and_data_disabled() {
    let html = render(&tooltip::trigger(
        OpenState::Closed,
        true,
        None,
        vec![],
        vec![],
    ));
    assert!(html.contains(r#"disabled="""#));
    assert!(html.contains(r#"data-disabled="""#));
}

#[test]
fn placement_attrs_reach_positioner_not_content() {
    let anchor = Rect {
        x: 0.0,
        y: 0.0,
        width: 40.0,
        height: 20.0,
    };
    let floating = Size {
        width: 120.0,
        height: 60.0,
    };
    let viewport = Size {
        width: 800.0,
        height: 600.0,
    };
    let config = PositioningConfig {
        placement: Placement::new(Side::Bottom, Align::Center),
        offset: 4.0,
        flip: true,
        shift: true,
        same_width: false,
    };
    let resolved = compute_position(anchor, floating, viewport, &config, true);
    let style = css_vars_style(&resolved, anchor.width, config.same_width);
    let mut attrs: Vec<(&str, &str)> = vec![("style", &style)];
    attrs.extend(placement_attrs(resolved.placement));

    let positioner_html = render(&tooltip::positioner(OpenState::Open, attrs, vec![]));
    assert!(positioner_html.contains("data-side="));
    assert!(positioner_html.contains("data-align="));

    let content_html = render(&tooltip::content(OpenState::Open, None, vec![], vec![]));
    assert!(!content_html.contains("data-side="));
    assert!(!content_html.contains("data-align="));
}

#[test]
fn arrow_and_arrow_tip_are_aria_hidden() {
    let html = render(&tooltip::arrow(
        vec![],
        vec![tooltip::arrow_tip(vec![], vec![])],
    ));
    assert_eq!(html.matches(r#"aria-hidden="true""#).count(), 2);
}

#[test]
fn dispatch_toggle_flips_data_state_across_parts() {
    let mut t = Tooltip::default();
    assert_eq!(t.state(), OpenState::Closed);
    assert!(render(&t.content(None, vec![], vec![])).contains(r#"hidden="""#));

    assert!(dispatch(&mut t, "toggle", ""));
    assert!(render(&t.root(vec![], vec![])).contains(r#"data-state="open""#));
    assert!(!render(&t.content(None, vec![], vec![])).contains("hidden"));

    assert!(dispatch(&mut t, "open", ""));
    assert_eq!(t.state(), OpenState::Open);

    assert!(dispatch(&mut t, "close", ""));
    assert_eq!(t.state(), OpenState::Closed);

    assert!(!dispatch(&mut t, "no_such_action", ""));
    assert_eq!(t.state(), OpenState::Closed);
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr() {
    let t = Tooltip::default();
    let html = render(&t.view());
    assert!(!html.contains("data-hydrate-"));
    assert!(html.contains(r#"data-state="closed""#));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let t = Tooltip::new(OpenState::Open);
    let html = render(&render_for_hydration(&t));
    assert!(html.contains(r#"data-hydrate-state="open""#));

    let restored = Tooltip::from_hydration_attrs(&t.hydration_attrs()).unwrap();
    assert_eq!(restored, t);
}

#[test]
fn hydration_tampered_value_returns_error_not_panic() {
    for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
        let attrs = vec![("data-hydrate-state".to_string(), bogus.to_string())];
        let err = Tooltip::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値すべてに攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn describedby_and_id_payloads_are_escaped_end_to_end() {
    let trigger = tooltip::trigger(
        OpenState::Closed,
        false,
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![],
    );
    let content = tooltip::content(OpenState::Closed, Some(ATTR_BREAK_PAYLOAD), vec![], vec![]);
    let html = render(&tooltip::root(
        OpenState::Closed,
        vec![],
        vec![trigger, content],
    ));

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let html = render(&tooltip::root(
        OpenState::Closed,
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}
