//! `fandhe-frontend-headless-ui` の Toolbar（[`toolbar`] モジュール、
//! イシュー #991）の公開 API 経由の統合テスト。
//!
//! `crates/headless-ui/src/toolbar.rs` 内の `#[cfg(test)]` ユニットテストが
//! 内部実装を含めた網羅を担うのに対し、本ファイルは
//! `fandhe-frontend-headless-ui` の公開 API（`lib.rs` の再エクスポート）
//! のみを経由し、`fandhe-frontend-pre-styled-ui`（styled ラッパー）が実際に
//! 使う想定の外部からの利用形態（SSR → dispatch → hydration の一巡、
//! ToggleGroup との組み合わせ）を固定する回帰テスト（[`crate::carousel`]
//! 相当の `tests/carousel.rs` と同型の位置付け）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::toolbar;
use fandhe_frontend_headless_ui::{
    Orientation, ToggleGroup, ToggleGroupProps, Toolbar, ToolbarAction,
};
use fandhe_frontend_interactive::{dispatch, render_for_hydration, Component, Hydrate};

/// SSR（状態なし初期描画）→ dispatch（`"next"`）→ hydration の一巡が公開
/// API のみで完結することを固定する。button/separator/toggle-group/
/// toggle-item を 1 つのノード木に組み合わせても anatomy が破綻しないこと
/// も併せて確認する。
#[test]
fn full_cycle_ssr_then_dispatch_then_hydration() {
    let initial = Toolbar::new(0, 3, false, Orientation::Horizontal);
    let group = ToggleGroup::default();

    let ssr_html = render(&initial.root(
        "Text formatting",
        vec![],
        vec![
            initial.button(0, false, vec![], vec![text("Undo")]),
            initial.separator(vec![], vec![]),
            toolbar::toggle_group(
                vec![],
                vec![group.item(
                    &ToggleGroupProps::default(),
                    "bold",
                    false,
                    false,
                    vec![],
                    vec![text("B")],
                )],
            ),
            initial.link(2, "/docs", false, vec![], vec![text("Docs")]),
        ],
    ));
    assert!(ssr_html.contains(r#"role="toolbar""#));
    assert!(ssr_html.contains(r#"aria-orientation="horizontal""#));
    assert!(ssr_html.contains(r#"aria-label="Text formatting""#));
    assert!(ssr_html.contains(r#"role="separator""#));
    assert!(ssr_html.contains(r#"role="group""#));
    assert!(ssr_html.contains(r#"data-value="bold""#));
    assert!(!ssr_html.contains("data-hydrate-"));

    // クライアント側（wasm-full 相当）の dispatch で次項目へフォーカス。
    let mut client_state = initial;
    assert!(dispatch(&mut client_state, "next", ""));
    assert_eq!(client_state.focused(), 1);

    // 別の SSR リクエスト（focused=1 状態）は hydration 属性込みで出力される。
    let hydrated_html = render(&render_for_hydration(&client_state));
    assert!(hydrated_html.contains(r#"data-hydrate-focused="1""#));
    assert!(hydrated_html.contains(r#"data-hydrate-item-count="3""#));

    // サーバーが同じ hydration 属性から状態を復元できる（改ざんされない
    // 限り panic せず、SSR 側と一致する）。
    let restored = Toolbar::from_hydration_attrs(&client_state.hydration_attrs()).unwrap();
    assert_eq!(restored, client_state);
}

/// `loop_focus = true` の circular 遷移が公開 API 経由でも成立する。
#[test]
fn loop_focus_enabled_wraps_at_both_ends_via_public_api() {
    let mut t = Toolbar::new(2, 3, true, Orientation::Horizontal);
    assert!(dispatch(&mut t, "next", ""));
    assert_eq!(t.focused(), 0);
    assert!(dispatch(&mut t, "prev", ""));
    assert_eq!(t.focused(), 2);
}

/// `"focus"` dispatch が有効な index へのみ実際に遷移する（範囲外は
/// 認識されつつも no-op、[`crate::toolbar`] モジュール doc 参照）。
#[test]
fn focus_dispatch_moves_within_bounds_via_public_api() {
    let mut t = Toolbar::new(0, 4, false, Orientation::Horizontal);
    assert!(dispatch(&mut t, "focus", "3"));
    assert_eq!(t.focused(), 3);
    assert!(dispatch(&mut t, "focus", "10"));
    assert_eq!(t.focused(), 3, "範囲外 focus は現在位置を変えない");
}

/// 型付き API（`Component::update`）経由でも `First`/`Last` が成立する。
#[test]
fn typed_update_first_and_last() {
    let mut t = Toolbar::new(1, 5, false, Orientation::Horizontal);
    t.update(ToolbarAction::Last);
    assert_eq!(t.focused(), 4);
    t.update(ToolbarAction::First);
    assert_eq!(t.focused(), 0);
}

/// 自由関数（静的 SSR）版も公開 API（`toolbar` モジュール）経由で直接
/// 呼び出せる（`fandhe-frontend-pre-styled-ui` が styled root を新設する際に
/// 委譲する想定の経路）。
#[test]
fn free_functions_are_reachable_via_public_module_path() {
    let html = render(&toolbar::root(
        Orientation::Vertical,
        "Gallery controls",
        vec![],
        vec![toolbar::button(true, false, vec![], vec![])],
    ));
    assert!(html.contains(r#"data-orientation="vertical""#));
    assert!(html.contains(r#"aria-orientation="vertical""#));
    assert!(html.contains(r#"tabindex="0""#));
}

/// vertical toolbar のセパレータは horizontal になる（直交規則が公開 API
/// 経由でも維持される）。
#[test]
fn separator_orientation_is_orthogonal_via_public_api() {
    let html = render(&toolbar::separator(Orientation::Vertical, vec![], vec![]));
    assert!(html.contains(r#"aria-orientation="horizontal""#));
}
