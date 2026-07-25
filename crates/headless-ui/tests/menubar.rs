//! `fandhe-frontend-headless-ui` の Menubar（[`menubar`] モジュール、
//! イシュー #992）の公開 API 経由の統合テスト。
//!
//! `crates/headless-ui/src/menubar.rs` 内の `#[cfg(test)]` ユニットテストが
//! 内部実装を含めた網羅を担うのに対し、本ファイルは
//! `fandhe-frontend-headless-ui` の公開 API（`lib.rs` の再エクスポート）
//! のみを経由し、`fandhe-frontend-pre-styled-ui`（styled ラッパー）が実際に
//! 使う想定の外部からの利用形態（SSR → dispatch → hydration の一巡、
//! サブメニューを [`Menu`] から注入する構成）を固定する回帰テスト
//! （`tests/toolbar.rs` と同型の位置付け）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::menubar;
use fandhe_frontend_headless_ui::{Menu, Menubar, MenubarAction, OpenState, Orientation};
use fandhe_frontend_interactive::{dispatch, render_for_hydration, Component, Hydrate};

/// SSR（状態なし初期描画）→ dispatch（`"open"` → `"next"`）→ hydration の
/// 一巡が公開 API のみで完結することを固定する。root/menu/trigger/
/// positioner/content/item を 1 つのノード木に組み合わせても anatomy が
/// 破綻しないことも併せて確認する。
#[test]
fn full_cycle_ssr_then_dispatch_then_hydration() {
    let initial = Menubar::new(0, 2, None, false, Orientation::Horizontal);

    let ssr_html = render(&initial.root(
        "App menu",
        vec![],
        vec![
            initial.menu(
                0,
                vec![],
                vec![
                    initial.trigger(0, false, false, Some("menu-0"), vec![], vec![text("File")]),
                    initial.positioner(
                        0,
                        vec![],
                        vec![initial.content(
                            0,
                            Some("menu-0"),
                            None,
                            vec![],
                            vec![menubar::item(
                                "save",
                                false,
                                false,
                                vec![],
                                vec![text("Save")],
                            )],
                        )],
                    ),
                ],
            ),
            initial.menu(
                1,
                vec![],
                vec![initial.trigger(1, false, false, Some("menu-1"), vec![], vec![text("Edit")])],
            ),
        ],
    ));
    assert!(ssr_html.contains(r#"role="menubar""#));
    assert!(ssr_html.contains(r#"role="none""#));
    assert!(ssr_html.contains(r#"aria-label="App menu""#));
    assert!(ssr_html.contains(r#"role="menu""#));
    assert!(ssr_html.contains(r#"data-value="save""#));
    assert!(!ssr_html.contains("data-hydrate-"));

    // クライアント側（wasm-full 相当）の dispatch で Menu 0 を開く。
    let mut client_state = initial;
    assert!(dispatch(&mut client_state, "open", "0"));
    assert_eq!(client_state.open(), Some(0));
    assert_eq!(client_state.focused(), 0);

    // 開いた状態で次のトリガーへ移動すると、開く Menu も追随する
    // （本イシューの主題）。
    assert!(dispatch(&mut client_state, "next", ""));
    assert_eq!(client_state.focused(), 1);
    assert_eq!(
        client_state.open(),
        Some(1),
        "開く Menu がフォーカス移動に追随する"
    );

    // 別の SSR リクエスト（focused=1, open=Some(1) 状態）は hydration 属性
    // 込みで出力される。
    let hydrated_html = render(&render_for_hydration(&client_state));
    assert!(hydrated_html.contains(r#"data-hydrate-focused="1""#));
    assert!(hydrated_html.contains(r#"data-hydrate-trigger-count="2""#));
    assert!(hydrated_html.contains(r#"data-hydrate-open="1""#));

    // サーバーが同じ hydration 属性から状態を復元できる（改ざんされない
    // 限り panic せず、SSR 側と一致する）。
    let restored = Menubar::from_hydration_attrs(&client_state.hydration_attrs()).unwrap();
    assert_eq!(restored, client_state);
}

/// サブメニューは [`Menu`]（= [`fandhe_frontend_headless_ui::Disclosure`]
/// 埋め込み）から状態を注入する。[`menubar::sub_trigger`]/
/// [`menubar::sub_content`] の `aria-expanded`/`data-state` は親
/// [`Menubar`] ではなくサブメニュー側インスタンスに従う（モジュール doc
/// 「`menu` mod 再利用の内訳」参照）。
#[test]
fn submenu_state_is_injected_from_separate_menu_instance() {
    let sub_menu = Menu::new(OpenState::Open);

    let html = render(&menubar::sub_trigger(
        sub_menu.state(),
        false,
        false,
        Some("sub-content-1"),
        vec![],
        vec![text("Export")],
    ));
    assert!(html.contains(r#"aria-expanded="true""#));
    assert!(html.contains(r#"data-state="open""#));

    let closed_sub_menu = Menu::new(OpenState::Closed);
    let closed_html = render(&menubar::sub_content(
        closed_sub_menu.state(),
        Some("sub-content-1"),
        None,
        vec![],
        vec![],
    ));
    assert!(closed_html.contains(r#"hidden="""#));
}

/// `loop_focus = true` の circular 遷移が公開 API 経由でも成立する。
#[test]
fn loop_focus_enabled_wraps_at_both_ends_via_public_api() {
    let mut m = Menubar::new(2, 3, None, true, Orientation::Horizontal);
    assert!(dispatch(&mut m, "next", ""));
    assert_eq!(m.focused(), 0);
    assert!(dispatch(&mut m, "prev", ""));
    assert_eq!(m.focused(), 2);
}

/// `"focus"` dispatch が有効な index へのみ実際に遷移する（範囲外は
/// 認識されつつも no-op、[`menubar`] モジュール doc 参照）。
#[test]
fn focus_dispatch_moves_within_bounds_via_public_api() {
    let mut m = Menubar::new(0, 4, None, false, Orientation::Horizontal);
    assert!(dispatch(&mut m, "focus", "3"));
    assert_eq!(m.focused(), 3);
    assert!(dispatch(&mut m, "focus", "10"));
    assert_eq!(m.focused(), 3, "範囲外 focus は現在位置を変えない");
}

/// 型付き API（`Component::update`）経由でも `Open`/`Close`/`Toggle` が
/// 成立する。
#[test]
fn typed_update_open_close_toggle() {
    let mut m = Menubar::new(0, 3, None, false, Orientation::Horizontal);
    m.update(MenubarAction::Open(1));
    assert_eq!(m.open(), Some(1));
    assert_eq!(m.focused(), 1);
    m.update(MenubarAction::Close);
    assert_eq!(m.open(), None);
    m.update(MenubarAction::Toggle(2));
    assert_eq!(m.open(), Some(2));
    m.update(MenubarAction::Toggle(2));
    assert_eq!(m.open(), None);
}

/// 自由関数（静的 SSR）版も公開 API（`menubar` モジュール）経由で直接
/// 呼び出せる（`fandhe-frontend-pre-styled-ui` が styled root を新設する際に
/// 委譲する想定の経路）。
#[test]
fn free_functions_are_reachable_via_public_module_path() {
    let html = render(&menubar::root(
        Orientation::Vertical,
        "Sidebar menu",
        vec![],
        vec![menubar::menu(
            OpenState::Closed,
            vec![],
            vec![menubar::trigger(
                true,
                OpenState::Closed,
                false,
                false,
                None,
                vec![],
                vec![],
            )],
        )],
    ));
    assert!(html.contains(r#"data-orientation="vertical""#));
    assert!(html.contains(r#"aria-orientation="vertical""#));
    assert!(html.contains(r#"tabindex="0""#));
}

/// SSR 出力に `data-hydrate-` が現れないこと（状態なし初期描画）と、
/// `render_for_hydration` 経由では現れることを固定する。
#[test]
fn ssr_view_has_no_hydrate_attr_but_render_for_hydration_does() {
    let m = Menubar::default();
    let ssr_html = render(&m.view());
    assert!(!ssr_html.contains("data-hydrate-"));

    let hydrated_html = render(&render_for_hydration(&m));
    assert!(hydrated_html.contains("data-hydrate-focused"));
}
