//! `fandhe-frontend-pre-styled-ui` が再エクスポートする headless 型の固定テスト
//! （イシュー #685）。
//!
//! PR #679 で `fandhe-frontend-docs-site` が `fandhe-frontend-headless-ui` へ
//! 直接依存せざるを得なかった理由（`OpenState`/`Orientation` 等、ラッパー呼び出しに
//! 必要な headless 型が pre-styled-ui のパスから import できなかったこと）を
//! 解消したことを固定する。本ファイルの import は `fandhe_frontend_pre_styled_ui::`
//! パス（クレート再エクスポート [`fandhe_frontend_pre_styled_ui::fandhe_frontend_core`]・
//! [`fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui`] を含む）のみを
//! 使用し、「pre-styled-ui 単独依存でラッパーを呼び出せる」契約をコンパイルで
//! 固定する。`dispatch`/`Component` は #685 時点では
//! `fandhe_frontend_interactive` dev-dependency 経由の直接 import だったが、
//! イシュー #712 で `fandhe_frontend_pre_styled_ui::fandhe_frontend_interactive`
//! クレート再エクスポートが確立したため、本ファイルもそちらへ切り替えて
//! 契約テストとしての純度を上げる（`Component`/`Hydrate` 等の hydration まで
//! 含めた単独依存到達性の固定は `interactive_reexports.rs` が担う）。

use fandhe_frontend_pre_styled_ui::fandhe_frontend_core::{el, render, text};
use fandhe_frontend_pre_styled_ui::fandhe_frontend_interactive::{dispatch, Component};
use fandhe_frontend_pre_styled_ui::{
    accordion, dialog, menu, popover, select, tabs, toggle_tip, tooltip,
};
// ルート再エクスポート（docs-site 実利用パスと同型の import、イシュー #685）。
use fandhe_frontend_pre_styled_ui::{ColorPalette, OpenState, Orientation, Size};

/// [`tabs::TabsProps::orientation`] が pre-styled-ui のパスのみで組み立てられる
/// ことを固定する（`data_attrs::Orientation` の再エクスポート）。
#[test]
fn tabs_orientation_reexport_is_usable_via_pre_styled_ui_path_alone() {
    use fandhe_frontend_pre_styled_ui::tabs::Orientation as TabsOrientation;

    let props = tabs::TabsProps {
        id: "t1",
        selected: "one",
        orientation: TabsOrientation::Horizontal,
        activation_mode: tabs::ActivationMode::Automatic,
        loop_focus: true,
        indicator: false,
    };
    let items = vec![tabs::TabItem {
        value: "one",
        trigger: vec![text("tab one")],
        content: vec![],
        disabled: false,
    }];
    let html = render(&tabs::tabs(Size::Md, ColorPalette::Accent, &props, items));
    assert!(html.contains(r#"data-orientation="horizontal""#));
}

/// [`accordion::item`]/[`accordion::item_trigger`] 等の `state` 引数
/// （`state::OpenState`）が pre-styled-ui のパスのみで組み立てられることと、
/// `Accordion`/`MultiAccordion` の `Component::Action`
/// （`SingleSelectAction`/`MultiSelectAction`）が dispatch まで接続することを
/// 固定する。
#[test]
fn accordion_state_and_action_reexports_are_usable_via_pre_styled_ui_path_alone() {
    use fandhe_frontend_pre_styled_ui::accordion::{
        Accordion, MultiAccordion, MultiSelectAction, OpenState as AccordionOpenState,
        SingleSelectAction,
    };

    let html = render(&accordion::item(
        AccordionOpenState::Open,
        false,
        vec![],
        vec![el("span", vec![], vec![text("body")])],
    ));
    assert!(html.contains(r#"data-state="open""#));

    let mut single = Accordion::default();
    dispatch(&mut single, "select", "one");
    assert_eq!(single.expanded(), Some("one"));
    single.update(SingleSelectAction::Deselect);
    assert_eq!(single.expanded(), None);

    let mut multi = MultiAccordion::default();
    multi.update(MultiSelectAction::Select("two".to_string()));
    assert!(multi.is_open("two"));
}

/// [`dialog::root`]/[`dialog::trigger`] 等の `state` 引数（`state::OpenState`）が
/// pre-styled-ui のパスのみで組み立てられることを固定する。`Dialog`（headless
/// 状態機械）はイシュー #729 により本モジュールから再エクスポートされないため
/// （`crates/pre-styled-ui/src/dialog.rs` rustdoc「選択的 re-export」節参照）、
/// dispatch まで接続する動作確認はエスケープハッチ
/// （`fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::dialog::Dialog`）
/// 経由で行う。
#[test]
fn dialog_state_and_action_reexports_are_usable_via_pre_styled_ui_path_alone() {
    use fandhe_frontend_pre_styled_ui::dialog::{DisclosureAction, OpenState as DialogOpenState};
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::dialog::Dialog;

    let html = render(&dialog::root(
        Size::Md,
        DialogOpenState::Open,
        vec![],
        vec![el("span", vec![], vec![text("body")])],
    ));
    assert!(html.contains(r#"data-state="open""#));

    let mut d = Dialog::new(DialogOpenState::Closed);
    dispatch(&mut d, "open", "");
    assert_eq!(d.state(), DialogOpenState::Open);
    d.update(DisclosureAction::Close);
    assert_eq!(d.state(), DialogOpenState::Closed);
}

/// [`menu::root`] 等の `state` 引数と `MenuCheckboxItem`/`MenuRadioItemGroup`
/// の `Component::Action`（`CheckableAction`/`SingleSelectAction`）が
/// pre-styled-ui のパスのみで組み立てられることを固定する。`Menu`（headless
/// 状態機械）はイシュー #729 により本モジュールから再エクスポートされないため
/// （`crates/pre-styled-ui/src/menu.rs` rustdoc「選択的 re-export」節参照）、
/// dispatch まで接続する動作確認はエスケープハッチ
/// （`fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::menu::Menu`）
/// 経由で行う。
#[test]
fn menu_state_and_action_reexports_are_usable_via_pre_styled_ui_path_alone() {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui::menu::Menu;
    use fandhe_frontend_pre_styled_ui::menu::{
        CheckableAction, DisclosureAction, OpenState as MenuOpenState, SingleSelectAction,
    };

    let html = render(&menu::root(
        Size::Md,
        MenuOpenState::Open,
        vec![],
        vec![el("span", vec![], vec![text("body")])],
    ));
    assert!(html.contains(r#"data-state="open""#));

    let mut m = Menu::new(MenuOpenState::Closed);
    m.update(DisclosureAction::Open);
    assert_eq!(m.state(), MenuOpenState::Open);

    // CheckableAction/SingleSelectAction は Menu 自体ではなく
    // MenuCheckboxItem/MenuRadioItemGroup の Action だが、型として
    // pre-styled-ui のパスから到達できることのみを固定する（実インスタンスの
    // 組み立ては menu.rs 内部の完全なコンストラクタ引数を要するため
    // 対象外、doc 契約の固定が目的）。
    let _checkable: CheckableAction = CheckableAction::Check;
    let _select: SingleSelectAction = SingleSelectAction::Select("x".to_string());
}

/// [`select::root`]/[`select::control`] の `state` 引数（`state::OpenState`）が
/// pre-styled-ui のパスのみで組み立てられることを固定する。
#[test]
fn select_state_reexport_is_usable_via_pre_styled_ui_path_alone() {
    use fandhe_frontend_pre_styled_ui::select::OpenState as SelectOpenState;

    let html = render(&select::root(
        Size::Md,
        SelectOpenState::Open,
        vec![],
        vec![el("span", vec![], vec![text("body")])],
    ));
    assert!(html.contains(r#"data-state="open""#));
}

/// [`popover::root`] の `state` 引数と `Popover` の `Component::Action`
/// （`DisclosureAction`）が pre-styled-ui のパスのみで組み立てられ、dispatch
/// まで接続することを固定する。
#[test]
fn popover_state_and_action_reexports_are_usable_via_pre_styled_ui_path_alone() {
    use fandhe_frontend_pre_styled_ui::popover::{
        DisclosureAction, OpenState as PopoverOpenState, Popover,
    };

    let html = render(&popover::root(
        PopoverOpenState::Open,
        vec![],
        vec![el("span", vec![], vec![text("body")])],
    ));
    assert!(html.contains(r#"data-state="open""#));

    let mut p = Popover::new(PopoverOpenState::Closed);
    dispatch(&mut p, "toggle", "");
    assert_eq!(p.state(), PopoverOpenState::Open);
    p.update(DisclosureAction::Close);
    assert_eq!(p.state(), PopoverOpenState::Closed);
}

/// [`tooltip::root`] の `state` 引数と `Tooltip` の `Component::Action`
/// （`DisclosureAction`）が pre-styled-ui のパスのみで組み立てられ、dispatch
/// まで接続することを固定する。
#[test]
fn tooltip_state_and_action_reexports_are_usable_via_pre_styled_ui_path_alone() {
    use fandhe_frontend_pre_styled_ui::tooltip::{
        DisclosureAction, OpenState as TooltipOpenState, Tooltip,
    };

    let html = render(&tooltip::root(
        TooltipOpenState::Open,
        vec![],
        vec![el("span", vec![], vec![text("body")])],
    ));
    assert!(html.contains(r#"data-state="open""#));

    let mut t = Tooltip::new(TooltipOpenState::Closed);
    dispatch(&mut t, "open", "");
    assert_eq!(t.state(), TooltipOpenState::Open);
    t.update(DisclosureAction::Close);
    assert_eq!(t.state(), TooltipOpenState::Closed);
}

/// [`toggle_tip::root`] の `state` 引数と `ToggleTip` の `Component::Action`
/// （`DisclosureAction`）が pre-styled-ui のパスのみで組み立てられ、dispatch
/// まで接続することを固定する（[`mod@tooltip`] 分と同型、イシュー #761）。
#[test]
fn toggle_tip_state_and_action_reexports_are_usable_via_pre_styled_ui_path_alone() {
    use fandhe_frontend_pre_styled_ui::toggle_tip::{
        DisclosureAction as ToggleTipDisclosureAction, OpenState as ToggleTipOpenState, ToggleTip,
    };

    let html = render(&toggle_tip::root(
        ToggleTipOpenState::Open,
        vec![],
        vec![el("span", vec![], vec![text("body")])],
    ));
    assert!(html.contains(r#"data-state="open""#));

    let mut t = ToggleTip::new(ToggleTipOpenState::Closed);
    dispatch(&mut t, "open", "");
    assert_eq!(t.state(), ToggleTipOpenState::Open);
    t.update(ToggleTipDisclosureAction::Close);
    assert_eq!(t.state(), ToggleTipOpenState::Closed);
}

/// ルート再エクスポート（[`OpenState`]・[`Orientation`]）が docs-site 実利用
/// パス（`fandhe_frontend_headless_ui::{OpenState, Orientation}`）と同型の
/// import で使えることを固定する。
#[test]
fn root_reexports_of_open_state_and_orientation_are_usable() {
    assert!(OpenState::Open.is_open());
    assert_eq!(Orientation::Horizontal, Orientation::Horizontal);
}

/// クレート自体の再エクスポート（[`fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui`]）
/// が到達可能であることを固定する（headless 全 API へのエスケープハッチ、
/// #550 と同型のパターン）。
#[test]
fn headless_ui_crate_reexport_is_reachable() {
    use fandhe_frontend_pre_styled_ui::fandhe_frontend_headless_ui;

    let (attr, value) = fandhe_frontend_headless_ui::data_state("open");
    assert_eq!(attr, "data-state");
    assert_eq!(value, "open");
}
