//! styled Sidebar（イシュー #2073、親 #2071）の決定的 CSS 出力ゴールデン
//! テスト。
//!
//! `crates/pre-styled-ui/tests/command_css.rs` と同型の golden fixture
//! テスト。`sidebar` recipe は 22 slot を宣言し、`size`/`variant`/
//! `color-palette` いずれの軸クラスも持たない（`data-*` 属性セレクタで
//! variant/collapsible/side を表現する、`src/sidebar.rs` モジュール doc
//! 参照）。

use fandhe_frontend_pre_styled_ui::sidebar;

const SIDEBAR_GOLDEN_CSS: &str = "[data-scope=\"sidebar\"][data-part=\"provider\"] {\n  display: flex;\n  min-height: 100svh;\n  width: 100%;\n}\n\n[data-scope=\"sidebar\"][data-part=\"root\"] {\n  position: relative;\n  display: flex;\n  flex-direction: column;\n  box-sizing: border-box;\n  width: var(--fandhe-sidebar-width, 16rem);\n  flex-shrink: 0;\n  background: var(--fandhe-color-sidebar-bg);\n  color: var(--fandhe-color-sidebar-fg);\n  border-inline-end: 1px solid var(--fandhe-color-sidebar-border);\n}\n\n[data-scope=\"sidebar\"][data-part=\"root\"] {\n  transition-property: width, transform;\n  transition-duration: var(--fandhe-motion-duration-normal);\n  transition-timing-function: var(--fandhe-motion-easing-standard);\n}\n\n[data-scope=\"sidebar\"][data-part=\"header\"] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  padding: var(--fandhe-space-2);\n}\n\n[data-scope=\"sidebar\"][data-part=\"content\"] {\n  flex: 1;\n  min-height: 0;\n  overflow: auto;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n}\n\n[data-scope=\"sidebar\"][data-part=\"footer\"] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-2);\n  padding: var(--fandhe-space-2);\n}\n\n[data-scope=\"sidebar\"][data-part=\"separator\"] {\n  margin: 0 var(--fandhe-space-2);\n  border: 0;\n  border-top: 1px solid var(--fandhe-color-sidebar-border);\n}\n\n[data-scope=\"sidebar\"][data-part=\"input\"] {\n  display: block;\n  width: 100%;\n  box-sizing: border-box;\n  height: var(--fandhe-size-control-height-sm, 2rem);\n  padding: 0 var(--fandhe-space-2);\n  border: 1px solid var(--fandhe-color-sidebar-border);\n  border-radius: var(--fandhe-radius-md);\n  background: var(--fandhe-color-sidebar-bg);\n  color: inherit;\n  font: inherit;\n  font-size: var(--fandhe-font-font-size-sm);\n  outline: none;\n}\n\n[data-scope=\"sidebar\"][data-part=\"group\"] {\n  position: relative;\n  display: flex;\n  flex-direction: column;\n  padding: var(--fandhe-space-2);\n}\n\n[data-scope=\"sidebar\"][data-part=\"group-label\"] {\n  display: flex;\n  align-items: center;\n  height: 2rem;\n  padding: 0 var(--fandhe-space-2);\n  font-size: var(--fandhe-font-font-size-xs);\n  font-weight: var(--fandhe-font-font-weight-medium);\n  color: var(--fandhe-color-sidebar-fg);\n  opacity: 0.7;\n}\n\n[data-scope=\"sidebar\"][data-part=\"group-content\"] {\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\n[data-scope=\"sidebar\"][data-part=\"group-action\"] {\n  position: absolute;\n  inset-inline-end: var(--fandhe-space-2);\n  top: var(--fandhe-space-1);\n  width: 1.25rem;\n  height: 1.25rem;\n  display: inline-flex;\n  align-items: center;\n  justify-content: center;\n  border: 0;\n  background: transparent;\n  border-radius: var(--fandhe-radius-sm);\n  color: inherit;\n  cursor: pointer;\n}\n\n[data-scope=\"sidebar\"][data-part=\"menu\"] {\n  list-style: none;\n  margin: 0;\n  padding: 0;\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\n[data-scope=\"sidebar\"][data-part=\"menu-item\"] {\n  position: relative;\n}\n\n[data-scope=\"sidebar\"][data-part=\"menu-button\"] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  width: 100%;\n  box-sizing: border-box;\n  height: 2rem;\n  padding: 0 var(--fandhe-space-2);\n  border: 0;\n  border-radius: var(--fandhe-radius-md);\n  background: transparent;\n  color: inherit;\n  font: inherit;\n  font-size: var(--fandhe-font-font-size-sm);\n  text-align: start;\n  text-decoration: none;\n  cursor: pointer;\n  overflow: hidden;\n  white-space: nowrap;\n}\n\n[data-scope=\"sidebar\"][data-part=\"menu-button\"] {\n  transition-property: background, color;\n  transition-duration: var(--fandhe-motion-duration-fast);\n  transition-timing-function: var(--fandhe-motion-easing-standard);\n}\n\n[data-scope=\"sidebar\"][data-part=\"menu-action\"] {\n  position: absolute;\n  inset-inline-end: var(--fandhe-space-2);\n  top: 50%;\n  transform: translateY(-50%);\n  width: 1.25rem;\n  height: 1.25rem;\n  display: inline-flex;\n  align-items: center;\n  justify-content: center;\n  border: 0;\n  background: transparent;\n  border-radius: var(--fandhe-radius-sm);\n  color: inherit;\n  cursor: pointer;\n}\n\n[data-scope=\"sidebar\"][data-part=\"menu-badge\"] {\n  margin-inline-start: auto;\n  font-size: var(--fandhe-font-font-size-xs);\n  padding: 0 var(--fandhe-space-1);\n  border-radius: var(--fandhe-radius-md);\n  min-width: 1.25rem;\n  text-align: center;\n}\n\n[data-scope=\"sidebar\"][data-part=\"menu-sub\"] {\n  list-style: none;\n  margin: 0;\n  margin-inline-start: var(--fandhe-space-3);\n  padding: 0;\n  padding-inline-start: var(--fandhe-space-2);\n  border-inline-start: 1px solid var(--fandhe-color-sidebar-border);\n  display: flex;\n  flex-direction: column;\n  gap: var(--fandhe-space-1);\n}\n\n[data-scope=\"sidebar\"][data-part=\"menu-sub-item\"] {\n  position: relative;\n}\n\n[data-scope=\"sidebar\"][data-part=\"menu-sub-button\"] {\n  display: flex;\n  align-items: center;\n  gap: var(--fandhe-space-2);\n  width: 100%;\n  box-sizing: border-box;\n  height: 1.75rem;\n  padding: 0 var(--fandhe-space-2);\n  border: 0;\n  border-radius: var(--fandhe-radius-md);\n  background: transparent;\n  color: inherit;\n  font: inherit;\n  font-size: var(--fandhe-font-font-size-xs);\n  text-align: start;\n  text-decoration: none;\n  cursor: pointer;\n  overflow: hidden;\n  white-space: nowrap;\n}\n\n[data-scope=\"sidebar\"][data-part=\"rail\"] {\n  position: absolute;\n  inset-block: 0;\n  inset-inline-end: -1rem;\n  width: 1rem;\n  border: 0;\n  padding: 0;\n  background: transparent;\n  cursor: ew-resize;\n}\n\n[data-scope=\"sidebar\"][data-part=\"trigger\"] {\n  display: inline-flex;\n  align-items: center;\n  justify-content: center;\n  width: 1.75rem;\n  height: 1.75rem;\n  border: 0;\n  background: transparent;\n  border-radius: var(--fandhe-radius-md);\n  color: inherit;\n  cursor: pointer;\n}\n\n[data-scope=\"sidebar\"][data-part=\"inset\"] {\n  flex: 1;\n  min-width: 0;\n  display: flex;\n  flex-direction: column;\n  background: var(--fandhe-color-bg);\n}\n\n[data-scope=\"sidebar\"][data-part=\"root\"][data-side=\"right\"] {\n  order: 1;\n  border-inline-end: 0;\n  border-inline-start: 1px solid var(--fandhe-color-sidebar-border);\n}\n\n[data-scope=\"sidebar\"][data-part=\"root\"][data-variant=\"floating\"] {\n  margin: var(--fandhe-space-2);\n  border: 1px solid var(--fandhe-color-sidebar-border);\n  border-radius: var(--fandhe-radius-lg);\n  box-shadow: var(--fandhe-shadow-md);\n  height: calc(100svh - var(--fandhe-space-4));\n}\n\n[data-scope=\"sidebar\"][data-part=\"root\"][data-variant=\"inset\"] {\n  margin: var(--fandhe-space-2);\n  border: 0;\n  background: transparent;\n}\n\n[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] {\n  width: var(--fandhe-sidebar-width-icon, 3rem);\n}\n\n[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"offcanvas\"] {\n  width: 0;\n  border: 0;\n  overflow: hidden;\n}\n\n[data-scope=\"sidebar\"][data-part=\"root\"][data-mobile] {\n  position: fixed;\n  inset-block: 0;\n  inset-inline-start: 0;\n  width: var(--fandhe-sidebar-width-mobile, 18rem);\n  z-index: var(--fandhe-z-index-modal, 1001);\n  box-shadow: var(--fandhe-shadow-lg);\n}\n\n[data-scope=\"sidebar\"][data-part=\"group-action\"]:focus-visible {\n  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));\n  outline-offset: var(--fandhe-focus-ring-offset, 2px);\n}\n\n[data-scope=\"sidebar\"][data-part=\"menu-action\"]:focus-visible {\n  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));\n  outline-offset: var(--fandhe-focus-ring-offset, 2px);\n}\n\n[data-scope=\"sidebar\"][data-part=\"menu-button\"][data-active] {\n  background: var(--fandhe-color-sidebar-accent);\n  color: var(--fandhe-color-sidebar-accent-fg);\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\n[data-scope=\"sidebar\"][data-part=\"menu-button\"][data-size=\"sm\"] {\n  height: 1.75rem;\n  font-size: var(--fandhe-font-font-size-xs);\n}\n\n[data-scope=\"sidebar\"][data-part=\"menu-button\"][data-size=\"lg\"] {\n  height: 3rem;\n  padding-inline: var(--fandhe-space-3);\n}\n\n[data-scope=\"sidebar\"][data-part=\"menu-button\"][data-variant=\"outline\"] {\n  background: var(--fandhe-color-bg);\n  box-shadow: 0 0 0 1px var(--fandhe-color-sidebar-border);\n}\n\n[data-scope=\"sidebar\"][data-part=\"menu-button\"]:focus-visible {\n  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));\n  outline-offset: var(--fandhe-focus-ring-offset, 2px);\n}\n\n[data-scope=\"sidebar\"][data-part=\"menu-sub-button\"][data-active] {\n  background: var(--fandhe-color-sidebar-accent);\n  color: var(--fandhe-color-sidebar-accent-fg);\n  font-weight: var(--fandhe-font-font-weight-medium);\n}\n\n[data-scope=\"sidebar\"][data-part=\"menu-sub-button\"][data-size=\"md\"] {\n  font-size: var(--fandhe-font-font-size-sm);\n}\n\n[data-scope=\"sidebar\"][data-part=\"menu-sub-button\"]:focus-visible {\n  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));\n  outline-offset: var(--fandhe-focus-ring-offset, 2px);\n}\n\n[data-scope=\"sidebar\"][data-part=\"trigger\"]:focus-visible {\n  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));\n  outline-offset: var(--fandhe-focus-ring-offset, 2px);\n}\n\n[data-scope=\"sidebar\"][data-part=\"input\"]:focus-visible {\n  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));\n  outline-offset: var(--fandhe-focus-ring-offset, 2px);\n  outline-color: var(--fandhe-color-sidebar-focus-ring);\n}\n\n@media (hover: hover) {\n  [data-scope=\"sidebar\"][data-part=\"group-action\"]:hover:not([data-disabled]) {\n    background: var(--fandhe-color-sidebar-muted);\n  }\n\n  [data-scope=\"sidebar\"][data-part=\"menu-action\"]:hover:not([data-disabled]) {\n    background: var(--fandhe-color-sidebar-muted);\n  }\n\n  [data-scope=\"sidebar\"][data-part=\"menu-button\"]:hover:not([data-disabled]):not([data-active]) {\n    background: var(--fandhe-color-sidebar-muted);\n  }\n\n  [data-scope=\"sidebar\"][data-part=\"menu-sub-button\"]:hover:not([data-disabled]):not([data-active]) {\n    background: var(--fandhe-color-sidebar-muted);\n  }\n\n  [data-scope=\"sidebar\"][data-part=\"trigger\"]:hover:not([data-disabled]) {\n    background: var(--fandhe-color-sidebar-muted);\n  }\n\n  [data-scope=\"sidebar\"][data-part=\"rail\"]:hover:not([data-disabled]) {\n    box-shadow: inset 2px 0 0 var(--fandhe-color-sidebar-border);\n  }\n}\n\n[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-scope=\"sidebar\"][data-part=\"group-label\"] {\n  display: none;\n}\n\n[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-scope=\"sidebar\"][data-part=\"menu-badge\"] {\n  display: none;\n}\n\n[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-scope=\"sidebar\"][data-part=\"menu-action\"] {\n  display: none;\n}\n\n[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-scope=\"sidebar\"][data-part=\"group-action\"] {\n  display: none;\n}\n\n[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-scope=\"sidebar\"][data-part=\"menu-sub\"] {\n  display: none;\n}\n\n[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-scope=\"sidebar\"][data-part=\"input\"] {\n  display: none;\n}\n\n[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-scope=\"sidebar\"][data-part=\"menu-button\"] {\n  justify-content: center;\n  padding: 0;\n  width: 2rem;\n}\n\n[data-scope=\"sidebar\"][data-part=\"root\"][data-mobile][data-state=\"collapsed\"] {\n  width: var(--fandhe-sidebar-width-mobile, 18rem);\n  transform: translateX(-100%);\n}\n\n[data-scope=\"sidebar\"][data-part=\"root\"][data-side=\"right\"][data-mobile][data-state=\"collapsed\"] {\n  inset-inline-start: auto;\n  inset-inline-end: 0;\n  transform: translateX(100%);\n}\n\n[data-scope=\"sidebar\"][data-part=\"provider\"][data-variant=\"inset\"] > [data-scope=\"sidebar\"][data-part=\"inset\"] {\n  margin: var(--fandhe-space-2);\n  margin-inline-start: 0;\n  border-radius: var(--fandhe-radius-lg);\n  box-shadow: var(--fandhe-shadow-sm);\n  background: var(--fandhe-color-bg);\n}\n\n[data-scope=\"sidebar\"][data-part=\"provider\"][data-side=\"right\"] > [data-scope=\"sidebar\"][data-part=\"inset\"] {\n  margin-inline-start: var(--fandhe-space-2);\n  margin-inline-end: 0;\n}\n\n[data-scope=\"sidebar\"][data-part=\"root\"][data-side=\"right\"] > [data-scope=\"sidebar\"][data-part=\"rail\"] {\n  inset-inline-end: auto;\n  inset-inline-start: -1rem;\n}\n";

#[test]
fn stylesheet_matches_golden_css() {
    assert_eq!(sidebar::stylesheet(), SIDEBAR_GOLDEN_CSS);
}

#[test]
fn sidebar_css_is_deterministic() {
    let a = sidebar::stylesheet();
    let b = sidebar::stylesheet();
    assert_eq!(a, b);
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let out = sidebar::stylesheet();
    assert!(!out.contains("</style"));
    assert!(!out.contains('<'));
}

#[test]
fn css_icon_collapsed_width_rule_exists() {
    let out = sidebar::stylesheet();
    assert!(out.contains(
        r#"[data-scope="sidebar"][data-part="root"][data-state="collapsed"][data-collapsible="icon"] {"#
    ));
    assert!(out.contains("width: var(--fandhe-sidebar-width-icon, 3rem);"));
}

#[test]
fn css_mobile_collapsed_translate_rule_exists() {
    let out = sidebar::stylesheet();
    assert!(out.contains(
        r#"[data-scope="sidebar"][data-part="root"][data-mobile][data-state="collapsed"] {"#
    ));
    assert!(out.contains("transform: translateX(-100%);"));
    assert!(out.contains(
        r#"[data-scope="sidebar"][data-part="root"][data-side="right"][data-mobile][data-state="collapsed"] {"#
    ));
    assert!(out.contains("transform: translateX(100%);"));
}

#[test]
fn css_active_menu_button_and_hover_exclusion_exist() {
    let out = sidebar::stylesheet();
    assert!(out.contains(r#"[data-scope="sidebar"][data-part="menu-button"][data-active] {"#));
    assert!(out.contains(":hover:not([data-disabled]):not([data-active])"));
}

#[test]
fn css_inset_variant_child_rule_exists() {
    let out = sidebar::stylesheet();
    assert!(out.contains(
        r#"[data-scope="sidebar"][data-part="provider"][data-variant="inset"] > [data-scope="sidebar"][data-part="inset"] {"#
    ));
}

#[test]
fn css_focus_visible_rings_exist() {
    let out = sidebar::stylesheet();
    assert!(out.contains(r#"[data-scope="sidebar"][data-part="menu-button"]:focus-visible {"#));
    assert!(out.contains(r#"[data-scope="sidebar"][data-part="input"]:focus-visible {"#));
    assert!(out.contains("outline-color: var(--fandhe-color-sidebar-focus-ring);"));
}

#[test]
fn stylesheet_never_generates_class_based_variant_classes() {
    let out = sidebar::stylesheet();
    assert!(!out.contains("fd-sidebar--"));
}
