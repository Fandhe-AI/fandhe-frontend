//! `fandhe-frontend-headless-ui` が再エクスポートする
//! `fandhe_frontend_interactive`（イシュー #712）の到達性を固定するテスト。
//!
//! [`fandhe_frontend_headless_ui::fandhe_frontend_interactive`] 経由で
//! `Component`/`Hydrate`/`dispatch`/`HydrateError`/`render_for_hydration` に
//! 到達できることをコンパイル + 実行時アサーションで固定する。本ファイルの
//! import は `fandhe_frontend_headless_ui::` パスのみを使用し、
//! `fandhe-frontend-interactive` への直接依存を必要としない（`Cargo.toml` の
//! dev-dependencies に `fandhe-frontend-interactive` を追加しないことがこの
//! テストの意義そのもの）。

use fandhe_frontend_headless_ui::dialog::Dialog;
use fandhe_frontend_headless_ui::fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Hydrate, HydrateError,
};
use fandhe_frontend_headless_ui::state::OpenState;

/// `dispatch`/`Component`/`Hydrate`/`render_for_hydration` が
/// `fandhe_frontend_headless_ui::fandhe_frontend_interactive` パスのみで
/// SSR → hydration 属性生成 → dispatch まで一通り接続することを固定する。
#[test]
fn dialog_ssr_and_dispatch_round_trip_via_headless_ui_interactive_reexport() {
    let mut dialog = Dialog::new(OpenState::Closed);

    // SSR: render_for_hydration が Component::view のルート要素へ
    // Hydrate::hydration_attrs を付与した Node を返す契約（interactive の
    // 責務、headless-ui はこれを再エクスポート経由でそのまま呼べる）。
    let node = render_for_hydration(&dialog);
    let html = fandhe_frontend_headless_ui::fandhe_frontend_core::render(&node);
    assert!(html.contains(r#"data-state="closed""#));

    // dispatch: 文字列コマンド経由で Component::update を駆動できることを固定。
    dispatch(&mut dialog, "open", "");
    assert_eq!(dialog.state(), OpenState::Open);
}

/// [`Hydrate::from_hydration_attrs`] が改ざんされうる入力に対し panic せず
/// [`HydrateError`] を返す契約（interactive 不変条件 3）が、再エクスポート
/// 経由でも弱まらないことを固定する。
#[test]
fn dialog_from_hydration_attrs_rejects_tampered_attrs_without_panicking() {
    let dialog = Dialog::new(OpenState::Open);
    let mut attrs = dialog.hydration_attrs();
    for (name, value) in attrs.iter_mut() {
        if name.ends_with("state") {
            *value = "not-a-valid-state".to_string();
        }
    }

    let restored = Dialog::from_hydration_attrs(&attrs);
    assert!(matches!(restored, Err(HydrateError::InvalidValue { .. })));
}
