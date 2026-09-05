//! ToggleTip（イシュー #761）の統合テスト。
//!
//! `crates/headless-ui/src/toggle_tip.rs` の inline unit tests がパーツ単体の
//! 属性出力を固定するのに対し、本ファイルは
//! 「root > trigger + positioner > (content + arrow)」の組み立て全体の
//! data-*/ARIA 対応（controls↔id）・dispatch 統合・SSR/hydration 両経路・
//! XSS 回帰をクレート外部から（公開 API のみを使って）固定する。
//!
//! # 参照突合（イシュー #1644）
//!
//! ark-ui に Toggle Tip コンポーネントは存在しない
//! （`docs/design/component-coverage-map.md` 658 行目で ark-ui 名 `—`、
//! `ark-ui.com/docs/components/toggle-tip` は 2026-09-06 時点で HTTP 404）。
//! Radix Primitives / Radix Themes にも該当部品は無い。唯一の直接参照は
//! chakra-ui ToggleTip（Ark `Popover` を内包したラッパー、sub-parts は
//! `Root`/`Trigger`/`Positioner`/`Content`/`Arrow`/`ArrowTip` + 任意
//! `Portal`）であり、突合の結果**具体的な欠落は見つからず、
//! `crates/headless-ui/src/toggle_tip.rs` の是正は不要**と判定した
//! （差分メモの詳細はイシュー #1644 コメント参照）。
//!
//! 是正なしの根拠（意図的差分として `src/` を変更しなかった項目）:
//!
//! - anatomy: chakra の Root/Trigger/Positioner/Content/Arrow/ArrowTip の
//!   6 パーツと完全一致。`Portal` は DOM 配置の関心のため全体方針で不採用
//!   （`docs/policy/intentional-non-adoption.md` §3.23、hover-card #1641・
//!   popover #1642 と同型の判断）。
//! - `data-*`: `data-state`（`"open"`/`"closed"`）・`data-disabled` は
//!   同義。Zag/chakra 由来の `data-placement`（→ `positioner` の
//!   `data-side`/`data-align` が同役割）・`data-expanded`（`data-state` +
//!   `aria-expanded` と重複）・複数トリガー用 `data-ownedby`/`data-value`/
//!   `data-current` は popover #1642 / hover-card #1641 と同じ判断軸で
//!   意図的に非採用。パート・`data-*` の増減は無いため、Themes 側イシュー
//!   #1546（closed）への追加コメントは行っていない。
//! - ARIA: 本実装は disclosure パターン（trigger: `aria-expanded` +
//!   `aria-controls`、`aria-haspopup` なし。content: `role` なし）を採る。
//!   chakra は Popover 基盤のため `aria-haspopup="dialog"` /
//!   `role="dialog"` / `tabindex="-1"` を持つが、ToggleTip の content は
//!   非対話の短文テキストであり `dialog` ロールは不適合と判断し、モジュール
//!   doc §3 者境界の既存判断を維持した（意図的差分）。`tabindex="-1"` も
//!   dialog パターン前提のため付与しない
//!   （[`crate::hover_card`] #1641 と同型、配線後に再評価）。
//! - キーボード: `fandhe-frontend-wasm-full` に `"toggle-tip"` scope が
//!   `headless::MAPPING_TABLE`/`overlay::OverlayKind::from_scope`/
//!   `position::PositionedKind::from_scope` のいずれにも未登録であり、
//!   click → `"toggle"` dispatch・Escape 閉鎖・外側クリック閉鎖・
//!   placement 計算は未配線（wasm-full 側への `"toggle-tip"` scope 登録は
//!   別イシュー提案）。ネイティブ `<button type="button">` のため Tab
//!   フォーカス到達（disabled 時は除外）とクリック発火まではブラウザ標準
//!   で成立するが、開閉へは接続されない。

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::toggle_tip::{self, ToggleTip};
use fandhe_frontend_headless_ui::{
    compute_position, css_vars_style, placement_attrs, Align, OpenState, Placement,
    PositioningConfig, Rect, Side, Size,
};
use fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};

#[test]
fn full_assembly_wires_aria_controls_to_content_id() {
    let trigger = toggle_tip::trigger(OpenState::Open, false, Some("tip-1"), vec![], vec![]);
    let arrow = toggle_tip::arrow(vec![], vec![toggle_tip::arrow_tip(vec![], vec![])]);
    let content = toggle_tip::content(OpenState::Open, Some("tip-1"), vec![], vec![arrow]);
    let positioner = toggle_tip::positioner(OpenState::Open, vec![], vec![content]);
    let root = toggle_tip::root(OpenState::Open, vec![], vec![trigger, positioner]);

    let html = render(&root);
    assert!(html.contains(r#"data-scope="toggle-tip""#));
    assert!(html.contains(r#"data-part="root""#));
    assert!(html.contains(r#"data-part="trigger""#));
    assert!(html.contains(r#"data-part="positioner""#));
    assert!(html.contains(r#"data-part="content""#));
    assert!(html.contains(r#"data-part="arrow""#));
    assert!(html.contains(r#"data-part="arrow-tip""#));
    assert!(html.contains(r#"aria-expanded="true""#));
    assert!(html.contains(r#"aria-controls="tip-1""#));
    assert!(html.contains(r#"id="tip-1""#));
    // ToggleTip の 3 者境界（モジュール doc）: Popover と異なり
    // aria-haspopup は付与せず、Tooltip と異なり role="tooltip" も
    // aria-describedby も使わない。
    assert!(!html.contains("aria-haspopup"));
    assert!(!html.contains("role=\"tooltip\""));
    assert!(!html.contains("aria-describedby"));
    // open 状態のため content に hidden 存在属性は付かない。arrow の
    // aria-hidden="true"（装飾目的、substring として "hidden" を含む）とは
    // 区別するため、hidden 存在属性の具体的な出現形で判定する。
    assert!(!html.contains(r#" hidden="""#));
}

// --- 参照突合（イシュー #1644）: 意図的差分を機械固定する回帰 ---

#[test]
fn trigger_is_native_button_with_type_button_and_aria_expanded() {
    let html = render(&toggle_tip::trigger(
        OpenState::Closed,
        false,
        None,
        vec![],
        vec![],
    ));
    assert!(html.starts_with("<button"));
    assert!(html.contains(r#"type="button""#));
    assert!(html.contains(r#"aria-expanded="false""#));
}

#[test]
fn trigger_and_content_carry_only_reference_aligned_attrs() {
    // chakra-ui ToggleTip（Ark Popover 基盤）の aria-haspopup="dialog" /
    // role="dialog" / tabindex="-1" と、Zag 由来の data-placement /
    // data-expanded / 複数トリガー識別属性は、いずれも意図的に非採用
    // （モジュール doc §参照突合参照）。呼び出し側 attrs を経由しても
    // これらが混入しないことを固定する。
    let trigger = toggle_tip::trigger(OpenState::Open, false, Some("tt-1"), vec![], vec![]);
    let content = toggle_tip::content(OpenState::Open, Some("tt-1"), vec![], vec![]);
    let html = format!("{}{}", render(&trigger), render(&content));

    for absent in [
        "aria-haspopup",
        "role=",
        "tabindex",
        "data-placement",
        "data-expanded",
        "data-ownedby",
        "data-value",
        "data-current",
        "aria-describedby",
    ] {
        assert!(!html.contains(absent), "unexpected attribute: {absent}");
    }
}

#[test]
fn closed_state_hides_positioner_and_content() {
    let positioner = toggle_tip::positioner(
        OpenState::Closed,
        vec![],
        vec![toggle_tip::content(OpenState::Closed, None, vec![], vec![])],
    );
    let html = render(&positioner);
    assert_eq!(html.matches(r#" hidden="""#).count(), 2);
    assert_eq!(html.matches(r#"data-state="closed""#).count(), 2);
}

#[test]
fn disabled_trigger_emits_native_and_data_disabled() {
    let html = render(&toggle_tip::trigger(
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

    let positioner_html = render(&toggle_tip::positioner(OpenState::Open, attrs, vec![]));
    assert!(positioner_html.contains("data-side="));
    assert!(positioner_html.contains("data-align="));

    let content_html = render(&toggle_tip::content(OpenState::Open, None, vec![], vec![]));
    assert!(!content_html.contains("data-side="));
    assert!(!content_html.contains("data-align="));
}

#[test]
fn dispatch_toggle_flips_data_state_across_parts() {
    let mut t = ToggleTip::default();
    assert_eq!(t.state(), OpenState::Closed);
    assert!(render(&t.content(None, vec![], vec![])).contains(r#"hidden="""#));

    assert!(dispatch(&mut t, "toggle", ""));
    assert!(render(&t.root(vec![], vec![])).contains(r#"data-state="open""#));
    assert!(!render(&t.content(None, vec![], vec![])).contains("hidden"));
    assert!(render(&t.trigger(false, None, vec![], vec![])).contains(r#"aria-expanded="true""#));

    assert!(dispatch(&mut t, "open", ""));
    assert_eq!(t.state(), OpenState::Open);

    assert!(dispatch(&mut t, "close", ""));
    assert_eq!(t.state(), OpenState::Closed);

    assert!(!dispatch(&mut t, "no_such_action", ""));
    assert_eq!(t.state(), OpenState::Closed);
}

#[test]
fn ssr_initial_render_has_no_hydrate_attr() {
    let t = ToggleTip::default();
    let html = render(&t.view());
    assert!(!html.contains("data-hydrate-"));
    assert!(html.contains(r#"data-state="closed""#));
}

#[test]
fn hydration_round_trip_via_public_api() {
    let t = ToggleTip::new(OpenState::Open);
    let html = render(&render_for_hydration(&t));
    assert!(html.contains(r#"data-hydrate-state="open""#));

    let restored = ToggleTip::from_hydration_attrs(&t.hydration_attrs()).unwrap();
    assert_eq!(restored, t);
}

#[test]
fn hydration_tampered_value_returns_error_not_panic() {
    for bogus in ["OPEN", "<script>alert(1)</script>", ""] {
        let attrs = vec![("data-hydrate-state".to_string(), bogus.to_string())];
        let err = ToggleTip::from_hydration_attrs(&attrs).unwrap_err();
        assert!(matches!(err, HydrateError::InvalidValue { .. }));
    }
}

// --- XSS 回帰: 呼び出し側が制御しうる動的値すべてに攻撃ペイロードを通す ---

const ATTR_BREAK_PAYLOAD: &str = "\" onmouseover=\"alert(1)";

#[test]
fn controls_and_id_payloads_are_escaped_end_to_end() {
    let trigger = toggle_tip::trigger(
        OpenState::Closed,
        false,
        Some(ATTR_BREAK_PAYLOAD),
        vec![],
        vec![],
    );
    let content = toggle_tip::content(OpenState::Closed, Some(ATTR_BREAK_PAYLOAD), vec![], vec![]);
    let html = render(&toggle_tip::root(
        OpenState::Closed,
        vec![],
        vec![trigger, content],
    ));

    assert!(!html.contains("onmouseover=\"alert(1)"));
    assert!(html.contains("&quot;"));
}

#[test]
fn caller_attrs_payload_is_escaped_end_to_end() {
    let html = render(&toggle_tip::root(
        OpenState::Closed,
        vec![("data-testid", ATTR_BREAK_PAYLOAD)],
        vec![],
    ));
    assert!(!html.contains("onmouseover=\"alert(1)"));
}
