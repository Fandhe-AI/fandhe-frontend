//! `fandhe-frontend-headless-ui` の Navigation Menu（[`navigation_menu`]
//! モジュール、イシュー #993）の公開 API 経由の統合テスト。
//!
//! `crates/headless-ui/src/navigation_menu.rs` 内の `#[cfg(test)]`
//! ユニットテストが内部実装を含めた網羅を担うのに対し、本ファイルは
//! `fandhe-frontend-headless-ui` の公開 API（`lib.rs` の再エクスポート）
//! のみを経由し、`fandhe-frontend-pre-styled-ui`（styled ラッパー）が実際に
//! 使う想定の外部からの利用形態（SSR → dispatch → hydration の一巡、
//! アクティブリンクの表現）を固定する回帰テスト（`tests/menubar.rs` と
//! 同型の位置付け）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::navigation_menu;
use fandhe_frontend_headless_ui::{NavigationMenu, OpenState};
use fandhe_frontend_interactive::{dispatch, render_for_hydration, Hydrate};

/// SSR（状態なし初期描画）→ dispatch（`"select"`）→ hydration の一巡が
/// 公開 API のみで完結することを固定する。root/list/item/trigger/content/
/// link を 1 つのノード木に組み合わせても anatomy が破綻しないことも
/// 併せて確認する。
#[test]
fn full_cycle_ssr_then_dispatch_then_hydration() {
    let initial = NavigationMenu::default();

    let ssr_html = render(&navigation_menu::root(
        "Main",
        vec![],
        vec![navigation_menu::list(
            vec![],
            vec![
                initial.item(
                    "products",
                    false,
                    vec![],
                    vec![
                        initial.trigger(
                            "products",
                            false,
                            Some("trigger-products"),
                            Some("content-products"),
                            vec![],
                            vec![text("Products")],
                        ),
                        initial.content(
                            "products",
                            Some("content-products"),
                            Some("trigger-products"),
                            vec![],
                            vec![navigation_menu::link(
                                "/products/a",
                                false,
                                vec![],
                                vec![text("Product A")],
                            )],
                        ),
                    ],
                ),
                navigation_menu::item(
                    OpenState::Closed,
                    false,
                    vec![],
                    vec![navigation_menu::link(
                        "/about",
                        true,
                        vec![],
                        vec![text("About")],
                    )],
                ),
            ],
        )],
    ));
    assert!(ssr_html.starts_with("<nav"));
    assert!(ssr_html.contains(r#"aria-label="Main""#));
    assert!(ssr_html.contains(r#"aria-expanded="false""#));
    assert!(ssr_html.contains(r#"hidden="""#));
    assert!(ssr_html.contains(r#"aria-current="page""#));
    assert!(!ssr_html.contains("role="));
    assert!(!ssr_html.contains("data-motion"));
    assert!(!ssr_html.contains("data-hydrate-"));

    // クライアント側（wasm-full 相当）の dispatch で products を開く。
    let mut client_state = initial;
    assert!(dispatch(&mut client_state, "select", "products"));
    assert_eq!(client_state.open_value(), Some("products"));

    let opened_trigger_html = render(&client_state.trigger(
        "products",
        false,
        Some("trigger-products"),
        Some("content-products"),
        vec![],
        vec![text("Products")],
    ));
    assert!(opened_trigger_html.contains(r#"aria-expanded="true""#));

    // hydration ラウンドトリップ（SSR に埋め込む属性を再構築できる）。
    let hydrated_html = render(&render_for_hydration(&client_state));
    assert!(hydrated_html.contains("data-hydrate-selected="));
    assert!(hydrated_html.contains("products"));

    let restored = NavigationMenu::from_hydration_attrs(&client_state.hydration_attrs()).unwrap();
    assert_eq!(restored, client_state);
}

/// アクティブリンク（`current: true`）が `aria-current="page"` として
/// 表現され、`role` を一切持たないことを公開 API 経由で固定する
/// （`nav_list` との使い分けの根拠である「役割の暗黙依拠」を再確認する）。
#[test]
fn active_link_is_expressed_via_aria_current_without_role() {
    let html = render(&navigation_menu::link(
        "/about",
        true,
        vec![],
        vec![text("About")],
    ));
    assert!(html.contains(r#"aria-current="page""#));
    assert!(html.contains("data-current"));
    assert!(!html.contains("role="));
}
