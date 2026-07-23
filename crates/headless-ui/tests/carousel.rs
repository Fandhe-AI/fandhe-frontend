//! `fandhe-frontend-headless-ui` の Carousel（[`carousel`] モジュール、
//! イシュー #754）の公開 API 経由の統合テスト。
//!
//! `crates/headless-ui/src/carousel.rs` 内の `#[cfg(test)]` ユニットテストが
//! 内部実装を含めた網羅を担うのに対し、本ファイルは
//! `fandhe-frontend-headless-ui` の公開 API（`lib.rs` の再エクスポート）
//! のみを経由し、`fandhe-frontend-pre-styled-ui`（styled ラッパー）が実際に
//! 使う想定の外部からの利用形態（SSR → dispatch → hydration の一巡）を
//! 固定する回帰テスト（[`crate::slider`] 相当の `tests/state_machine.rs` と
//! 同型の位置付け）。

use fandhe_frontend_core::{render, text};
use fandhe_frontend_headless_ui::carousel;
use fandhe_frontend_headless_ui::{Carousel, Orientation};
use fandhe_frontend_interactive::{dispatch, render_for_hydration, Component, Hydrate};

/// SSR（状態なし初期描画）→ dispatch（`"next"`）→ hydration の一巡が公開
/// API のみで完結することを固定する。
#[test]
fn full_cycle_ssr_then_dispatch_then_hydration() {
    let initial = Carousel::new(0, 3, false, Orientation::Horizontal);
    let ssr_html = render(&initial.root(
        "Featured products",
        vec![],
        vec![
            initial.control(
                vec![],
                vec![
                    initial.prev_trigger("Previous slide", vec![], vec![]),
                    initial.item_group(
                        vec![],
                        vec![
                            initial.item(0, vec![], vec![text("A")]),
                            initial.item(1, vec![], vec![text("B")]),
                            initial.item(2, vec![], vec![text("C")]),
                        ],
                    ),
                    initial.next_trigger("Next slide", vec![], vec![]),
                ],
            ),
            initial.indicator_group(
                vec![],
                vec![
                    initial.indicator(0, vec![]),
                    initial.indicator(1, vec![]),
                    initial.indicator(2, vec![]),
                ],
            ),
        ],
    ));
    assert!(ssr_html.contains(r#"role="region""#));
    assert!(ssr_html.contains(r#"aria-roledescription="carousel""#));
    assert!(ssr_html.contains(r#"aria-label="1 of 3""#));
    assert!(!ssr_html.contains("data-hydrate-"));

    // クライアント側（wasm-full 相当）の dispatch で次スライドへ。
    let mut client_state = initial;
    assert!(dispatch(&mut client_state, "next", ""));
    assert_eq!(client_state.index(), 1);

    // 別の SSR リクエスト（index=1 状態）は hydration 属性込みで出力される。
    let hydrated_html = render(&render_for_hydration(&client_state));
    assert!(hydrated_html.contains(r#"data-hydrate-index="1""#));
    assert!(hydrated_html.contains(r#"data-hydrate-slide-count="3""#));

    // サーバーが同じ hydration 属性から状態を復元できる（改ざんされない
    // 限り panic せず、SSR 側と一致する）。
    let restored = Carousel::from_hydration_attrs(&client_state.hydration_attrs()).unwrap();
    assert_eq!(restored, client_state);
}

/// `loop = true` の circular 遷移が公開 API 経由でも成立する。
#[test]
fn loop_enabled_wraps_at_both_ends_via_public_api() {
    let mut c = Carousel::new(2, 3, true, Orientation::Horizontal);
    assert!(dispatch(&mut c, "next", ""));
    assert_eq!(c.index(), 0);
    assert!(dispatch(&mut c, "prev", ""));
    assert_eq!(c.index(), 2);
}

/// `"goto"` dispatch が有効な index へのみ実際に遷移する（範囲外は
/// 認識されつつも no-op、[`crate::carousel`] モジュール doc 参照）。
#[test]
fn goto_dispatch_moves_within_bounds_via_public_api() {
    let mut c = Carousel::new(0, 4, false, Orientation::Horizontal);
    assert!(dispatch(&mut c, "goto", "3"));
    assert_eq!(c.index(), 3);
    assert!(dispatch(&mut c, "goto", "10"));
    assert_eq!(c.index(), 3, "範囲外 goto は現在位置を変えない");
}

/// 自由関数（静的 SSR）版も公開 API（`carousel` モジュール）経由で直接
/// 呼び出せる（`fandhe-frontend-pre-styled-ui` が styled root を新設する際に
/// 委譲する想定の経路）。
#[test]
fn free_functions_are_reachable_via_public_module_path() {
    let html = render(&carousel::root(
        Orientation::Vertical,
        "Gallery",
        vec![],
        vec![carousel::indicator(0, true, vec![])],
    ));
    assert!(html.contains(r#"data-orientation="vertical""#));
    assert!(html.contains(r#"aria-current="true""#));
}
