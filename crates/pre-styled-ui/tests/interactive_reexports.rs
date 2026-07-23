//! `fandhe-frontend-pre-styled-ui` が推移的に再エクスポートする
//! `fandhe_frontend_interactive`（イシュー #712）の到達性を固定するテスト。
//!
//! 本ファイルの import は `fandhe_frontend_pre_styled_ui::` パスのみを使用し、
//! `fandhe-frontend-interactive`（`fandhe-frontend-headless-ui`）への直接
//! 依存を必要としない。これまで hydration/dispatch まで書く利用者は
//! pre-styled-ui 単独依存では完結できず（`headless_reexports.rs` が #685
//! 時点で `fandhe_frontend_interactive` dev-dependency を直接 import して
//! いたのがその証跡）、本テストが「pre-styled-ui のみへの依存で SSR →
//! hydration 属性生成 → dispatch まで完結できる」契約をコンパイル + 実行時
//! アサーションで固定する。

use fandhe_frontend_pre_styled_ui::accordion::{Accordion, SingleSelectAction};
use fandhe_frontend_pre_styled_ui::dialog::{
    Dialog, DisclosureAction, OpenState as DialogOpenState,
};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_core::render;
use fandhe_frontend_pre_styled_ui::fandhe_frontend_interactive::{
    dispatch, render_for_hydration, Component, Hydrate, HydrateError,
};
use fandhe_frontend_pre_styled_ui::switch::{Switch, SwitchAction};

/// styled [`Dialog`] で SSR（`render_for_hydration`）→ hydration 属性復元
/// （[`Hydrate::from_hydration_attrs`]）→ dispatch までの往復が、
/// pre-styled-ui パスのみで完結することを固定する。
#[test]
fn dialog_ssr_hydration_and_dispatch_round_trip_via_pre_styled_ui_path_alone() {
    let mut dialog = Dialog::new(DialogOpenState::Closed);

    let node = render_for_hydration(&dialog);
    let html = render(&node);
    assert!(html.contains(r#"data-state="closed""#));

    let attrs = dialog.hydration_attrs();
    let restored = Dialog::from_hydration_attrs(&attrs).expect("valid attrs");
    assert_eq!(restored.state(), DialogOpenState::Closed);

    dispatch(&mut dialog, "open", "");
    assert_eq!(dialog.state(), DialogOpenState::Open);
    dialog.update(DisclosureAction::Close);
    assert_eq!(dialog.state(), DialogOpenState::Closed);
}

/// styled [`Accordion`]（single モード）で dispatch → hydration 属性生成 →
/// 復元までが pre-styled-ui パスのみで完結することを固定する。
#[test]
fn accordion_dispatch_and_hydration_round_trip_via_pre_styled_ui_path_alone() {
    let mut accordion = Accordion::default();
    dispatch(&mut accordion, "select", "item-one");
    assert_eq!(accordion.expanded(), Some("item-one"));

    let attrs = accordion.hydration_attrs();
    let restored = Accordion::from_hydration_attrs(&attrs).expect("valid attrs");
    assert_eq!(restored.expanded(), Some("item-one"));

    accordion.update(SingleSelectAction::Deselect);
    assert_eq!(accordion.expanded(), None);
}

/// styled [`Switch`] で dispatch → hydration 属性生成 → 復元までが
/// pre-styled-ui パスのみで完結することを固定する。
#[test]
fn switch_dispatch_and_hydration_round_trip_via_pre_styled_ui_path_alone() {
    let mut switch = Switch::default();
    dispatch(&mut switch, "check", "");
    assert!(switch.is_checked());

    let attrs = switch.hydration_attrs();
    let restored = Switch::from_hydration_attrs(&attrs).expect("valid attrs");
    assert!(restored.is_checked());

    switch.update(SwitchAction::Uncheck);
    assert!(!switch.is_checked());
}

/// [`Hydrate::from_hydration_attrs`] が改ざんされうる入力に対し panic せず
/// [`HydrateError`] を返す契約（interactive 不変条件 3）が、pre-styled-ui
/// 経由の再エクスポートでも弱まらないことを固定する。
#[test]
fn dialog_from_hydration_attrs_rejects_tampered_attrs_without_panicking() {
    let dialog = Dialog::new(DialogOpenState::Open);
    let mut attrs = dialog.hydration_attrs();
    for (name, value) in attrs.iter_mut() {
        if name.ends_with("state") {
            *value = "not-a-valid-state".to_string();
        }
    }

    let restored = Dialog::from_hydration_attrs(&attrs);
    assert!(matches!(restored, Err(HydrateError::InvalidValue { .. })));
}
