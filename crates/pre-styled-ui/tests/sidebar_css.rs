//! styled Sidebar（イシュー #2073、親 #2071）の決定的 CSS 出力ゴールデン
//! テスト。
//!
//! `crates/pre-styled-ui/tests/command_css.rs` と同型の golden fixture
//! テスト。`sidebar` recipe は 22 slot を宣言し、`size`/`variant`/
//! `color-palette` いずれの軸クラスも持たない（`data-*` 属性セレクタで
//! variant/collapsible/side を表現する、`src/sidebar.rs` モジュール doc
//! 参照）。

use fandhe_frontend_pre_styled_ui::sidebar;

const SIDEBAR_GOLDEN_CSS: &str = "[data-scope=\"sidebar\"][data-part=\"provider\"] {
  display: flex;
  min-height: 100svh;
  width: 100%;
}

[data-scope=\"sidebar\"][data-part=\"root\"] {
  position: relative;
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
  width: var(--fandhe-sidebar-width, 16rem);
  flex-shrink: 0;
  background: var(--fandhe-color-sidebar-bg);
  color: var(--fandhe-color-sidebar-fg);
  border-inline-end: 1px solid var(--fandhe-color-sidebar-border);
}

[data-scope=\"sidebar\"][data-part=\"root\"] {
  transition-property: width, transform, visibility;
  transition-duration: var(--fandhe-motion-duration-normal), var(--fandhe-motion-duration-normal), 0s;
  transition-timing-function: var(--fandhe-motion-easing-standard), var(--fandhe-motion-easing-standard), linear;
  transition-delay: 0s, 0s, 0s;
}

[data-scope=\"sidebar\"][data-part=\"header\"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-2);
  padding: var(--fandhe-space-2);
}

[data-scope=\"sidebar\"][data-part=\"content\"] {
  flex: 1;
  min-height: 0;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-2);
}

[data-scope=\"sidebar\"][data-part=\"footer\"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-2);
  padding: var(--fandhe-space-2);
}

[data-scope=\"sidebar\"][data-part=\"separator\"] {
  margin: 0 var(--fandhe-space-2);
  border: 0;
  border-top: 1px solid var(--fandhe-color-sidebar-border);
}

[data-scope=\"sidebar\"][data-part=\"input\"] {
  display: block;
  width: 100%;
  box-sizing: border-box;
  height: var(--fandhe-size-control-height-sm, 2rem);
  padding: 0 var(--fandhe-space-2);
  border: 1px solid var(--fandhe-color-sidebar-border);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-color-sidebar-bg);
  color: inherit;
  font: inherit;
  font-size: var(--fandhe-font-font-size-sm);
  outline: none;
}

[data-scope=\"sidebar\"][data-part=\"group\"] {
  position: relative;
  display: flex;
  flex-direction: column;
  padding: var(--fandhe-space-2);
}

[data-scope=\"sidebar\"][data-part=\"group-label\"] {
  display: flex;
  align-items: center;
  height: 2rem;
  padding: 0 var(--fandhe-space-2);
  font-size: var(--fandhe-font-font-size-xs);
  font-weight: var(--fandhe-font-font-weight-medium);
  color: var(--fandhe-color-sidebar-fg);
  opacity: 0.7;
}

[data-scope=\"sidebar\"][data-part=\"group-content\"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-1);
}

[data-scope=\"sidebar\"][data-part=\"group-action\"] {
  position: absolute;
  inset-inline-end: var(--fandhe-space-2);
  top: var(--fandhe-space-1);
  width: 1.25rem;
  height: 1.25rem;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  background: transparent;
  border-radius: var(--fandhe-radius-sm);
  color: inherit;
  cursor: pointer;
}

[data-scope=\"sidebar\"][data-part=\"menu\"] {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-1);
}

[data-scope=\"sidebar\"][data-part=\"menu-item\"] {
  position: relative;
}

[data-scope=\"sidebar\"][data-part=\"menu-button\"] {
  display: flex;
  align-items: center;
  width: 100%;
  box-sizing: border-box;
  height: 2rem;
  padding-block: 0;
  padding-inline-start: var(--fandhe-space-2);
  padding-inline-end: calc(var(--fandhe-space-2) + 1.25rem + var(--fandhe-space-1));
  border: 0;
  border-radius: var(--fandhe-radius-md);
  background: transparent;
  color: inherit;
  font: inherit;
  font-size: var(--fandhe-font-font-size-sm);
  text-align: start;
  text-decoration: none;
  cursor: pointer;
}

[data-scope=\"sidebar\"][data-part=\"menu-button\"] {
  transition-property: background, color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope=\"sidebar\"][data-part=\"menu-action\"] {
  position: absolute;
  inset-inline-end: var(--fandhe-space-2);
  top: 1rem;
  transform: translateY(-50%);
  width: 1.25rem;
  height: 1.25rem;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 0;
  background: transparent;
  border-radius: var(--fandhe-radius-sm);
  color: inherit;
  cursor: pointer;
}

[data-scope=\"sidebar\"][data-part=\"menu-badge\"] {
  position: absolute;
  inset-inline-end: var(--fandhe-space-2);
  top: 1rem;
  transform: translateY(-50%);
  font-size: var(--fandhe-font-font-size-xs);
  padding: 0 var(--fandhe-space-1);
  border-radius: var(--fandhe-radius-md);
  box-sizing: border-box;
  min-width: 1.25rem;
  text-align: center;
  pointer-events: none;
}

[data-scope=\"sidebar\"][data-part=\"menu-sub\"] {
  list-style: none;
  margin: 0;
  margin-inline-start: var(--fandhe-space-3);
  padding: 0;
  padding-inline-start: var(--fandhe-space-2);
  border-inline-start: 1px solid var(--fandhe-color-sidebar-border);
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-1);
}

[data-scope=\"sidebar\"][data-part=\"menu-sub-item\"] {
  position: relative;
}

[data-scope=\"sidebar\"][data-part=\"menu-sub-button\"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  width: 100%;
  box-sizing: border-box;
  height: 1.75rem;
  padding: 0 var(--fandhe-space-2);
  border: 0;
  border-radius: var(--fandhe-radius-md);
  background: transparent;
  color: inherit;
  font: inherit;
  font-size: var(--fandhe-font-font-size-xs);
  text-align: start;
  text-decoration: none;
  cursor: pointer;
  overflow: hidden;
  white-space: nowrap;
}

[data-scope=\"sidebar\"][data-part=\"rail\"] {
  position: absolute;
  inset-block: 0;
  inset-inline-end: -1rem;
  width: 1rem;
  border: 0;
  padding: 0;
  background: transparent;
  cursor: ew-resize;
  z-index: 1;
}

[data-scope=\"sidebar\"][data-part=\"trigger\"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  border: 0;
  background: transparent;
  border-radius: var(--fandhe-radius-md);
  color: inherit;
  cursor: pointer;
}

[data-scope=\"sidebar\"][data-part=\"inset\"] {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  background: var(--fandhe-color-bg);
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-side=\"right\"] {
  order: 1;
  border-inline-end: 0;
  border-inline-start: 1px solid var(--fandhe-color-sidebar-border);
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-variant=\"floating\"] {
  margin: var(--fandhe-space-2);
  border: 1px solid var(--fandhe-color-sidebar-border);
  border-radius: var(--fandhe-radius-lg);
  box-shadow: var(--fandhe-shadow-md);
  height: calc(100svh - var(--fandhe-space-4));
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-variant=\"inset\"] {
  margin: var(--fandhe-space-2);
  border: 0;
  background: transparent;
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] {
  width: var(--fandhe-sidebar-width-icon, 3rem);
  min-width: var(--fandhe-sidebar-width-icon, 3rem);
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"offcanvas\"] {
  width: 0;
  border: 0;
  overflow: hidden;
  visibility: hidden;
  transition-delay: 0s, 0s, var(--fandhe-motion-duration-normal);
  margin: 0;
}

[data-scope=\"sidebar\"][data-part=\"group-action\"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope=\"sidebar\"][data-part=\"menu-action\"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope=\"sidebar\"][data-part=\"menu-button\"][data-variant=\"outline\"] {
  background: var(--fandhe-color-bg);
  box-shadow: 0 0 0 1px var(--fandhe-color-sidebar-border);
}

[data-scope=\"sidebar\"][data-part=\"menu-button\"][data-size=\"sm\"] {
  height: 1.75rem;
  font-size: var(--fandhe-font-font-size-xs);
}

[data-scope=\"sidebar\"][data-part=\"menu-button\"][data-size=\"lg\"] {
  height: 3rem;
  padding-inline-start: var(--fandhe-space-3);
  padding-inline-end: calc(var(--fandhe-space-3) + 1.25rem + var(--fandhe-space-1));
}

[data-scope=\"sidebar\"][data-part=\"menu-button\"][data-active] {
  background: var(--fandhe-color-sidebar-accent);
  color: var(--fandhe-color-sidebar-accent-fg);
  font-weight: var(--fandhe-font-font-weight-medium);
}

[data-scope=\"sidebar\"][data-part=\"menu-button\"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope=\"sidebar\"][data-part=\"menu-sub-button\"][data-active] {
  background: var(--fandhe-color-sidebar-accent);
  color: var(--fandhe-color-sidebar-accent-fg);
  font-weight: var(--fandhe-font-font-weight-medium);
}

[data-scope=\"sidebar\"][data-part=\"menu-sub-button\"][data-size=\"md\"] {
  font-size: var(--fandhe-font-font-size-sm);
}

[data-scope=\"sidebar\"][data-part=\"menu-sub-button\"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope=\"sidebar\"][data-part=\"trigger\"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope=\"sidebar\"][data-part=\"input\"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
  outline-color: var(--fandhe-color-sidebar-focus-ring);
}

@media (hover: hover) {
  [data-scope=\"sidebar\"][data-part=\"group-action\"]:hover:not([data-disabled]) {
    background: var(--fandhe-color-sidebar-muted);
  }

  [data-scope=\"sidebar\"][data-part=\"menu-action\"]:hover:not([data-disabled]) {
    background: var(--fandhe-color-sidebar-muted);
  }

  [data-scope=\"sidebar\"][data-part=\"menu-button\"]:hover:not([data-disabled]):not([data-active]) {
    background: var(--fandhe-color-sidebar-muted);
  }

  [data-scope=\"sidebar\"][data-part=\"menu-sub-button\"]:hover:not([data-disabled]):not([data-active]) {
    background: var(--fandhe-color-sidebar-muted);
  }

  [data-scope=\"sidebar\"][data-part=\"trigger\"]:hover:not([data-disabled]) {
    background: var(--fandhe-color-sidebar-muted);
  }

  [data-scope=\"sidebar\"][data-part=\"rail\"]:hover:not([data-disabled]) {
    box-shadow: inset 2px 0 0 var(--fandhe-color-sidebar-border);
  }
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-scope=\"sidebar\"][data-part=\"group-label\"] {
  display: none;
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-scope=\"sidebar\"][data-part=\"menu-badge\"] {
  display: none;
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-scope=\"sidebar\"][data-part=\"menu-action\"] {
  display: none;
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-scope=\"sidebar\"][data-part=\"group-action\"] {
  display: none;
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-scope=\"sidebar\"][data-part=\"menu-sub\"] {
  display: none;
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-scope=\"sidebar\"][data-part=\"input\"] {
  display: none;
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-scope=\"sidebar\"][data-part=\"menu-button\"] {
  padding: 0;
  width: 2rem;
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-scope=\"sidebar\"][data-part=\"menu-button\"] > span > span[data-fandhe-sidebar-menu-button-label] {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  overflow-wrap: normal;
  border-width: 0;
}

[data-scope=\"sidebar\"][data-part=\"menu-action\"] ~ [data-scope=\"sidebar\"][data-part=\"menu-badge\"] {
  inset-inline-end: calc(var(--fandhe-space-2) + 1.25rem + var(--fandhe-space-1));
}

[data-scope=\"sidebar\"][data-part=\"menu-badge\"] ~ [data-scope=\"sidebar\"][data-part=\"menu-action\"] {
  inset-inline-end: calc(var(--fandhe-space-2) + 1.25rem + var(--fandhe-space-1));
}

[data-scope=\"sidebar\"][data-part=\"menu-item\"]:has(> [data-scope=\"sidebar\"][data-part=\"menu-action\"]):has(> [data-scope=\"sidebar\"][data-part=\"menu-badge\"]) > [data-scope=\"sidebar\"][data-part=\"menu-button\"] {
  padding-inline-end: calc(var(--fandhe-space-2) + 2 * 1.25rem + 2 * var(--fandhe-space-1));
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] [data-scope=\"sidebar\"][data-part=\"menu-item\"]:has(> [data-scope=\"sidebar\"][data-part=\"menu-action\"]):has(> [data-scope=\"sidebar\"][data-part=\"menu-badge\"]) > [data-scope=\"sidebar\"][data-part=\"menu-button\"] {
  padding-inline-end: 0;
}

[data-scope=\"sidebar\"][data-part=\"menu-button\"] > span {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  min-width: 0;
  flex: 1 1 auto;
  overflow: hidden;
  white-space: nowrap;
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-mobile]:not([data-collapsible=\"none\"]) {
  position: fixed;
  inset-block: 0;
  inset-inline-start: 0;
  width: var(--fandhe-sidebar-width-mobile, 18rem);
  height: 100%;
  margin: 0;
  border: 0;
  border-radius: 0;
  background: var(--fandhe-color-sidebar-bg);
  z-index: var(--fandhe-z-index-modal, 1001);
  box-shadow: var(--fandhe-shadow-lg);
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-side=\"right\"][data-mobile] {
  inset-inline-start: auto;
  inset-inline-end: 0;
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-mobile][data-state=\"collapsed\"]:not([data-collapsible=\"none\"]) {
  width: var(--fandhe-sidebar-width-mobile, 18rem);
  transform: translateX(-100%);
  visibility: hidden;
  transition-delay: 0s, 0s, var(--fandhe-motion-duration-normal);
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-side=\"right\"][data-mobile][data-state=\"collapsed\"]:not([data-collapsible=\"none\"]) {
  transform: translateX(100%);
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-mobile][data-state=\"collapsed\"]:not([data-collapsible=\"none\"]):dir(rtl) {
  transform: translateX(100%);
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-side=\"right\"][data-mobile][data-state=\"collapsed\"]:not([data-collapsible=\"none\"]):dir(rtl) {
  transform: translateX(-100%);
}

[data-scope=\"sidebar\"][data-part=\"provider\"][data-variant=\"inset\"] > [data-scope=\"sidebar\"][data-part=\"inset\"] {
  margin: var(--fandhe-space-2);
  margin-inline-start: 0;
  border-radius: var(--fandhe-radius-lg);
  box-shadow: var(--fandhe-shadow-sm);
  background: var(--fandhe-color-bg);
}

[data-scope=\"sidebar\"][data-part=\"provider\"][data-variant=\"inset\"][data-side=\"right\"] > [data-scope=\"sidebar\"][data-part=\"inset\"] {
  margin-inline-start: var(--fandhe-space-2);
  margin-inline-end: 0;
}

[data-scope=\"sidebar\"][data-part=\"menu-button\"][data-size=\"sm\"] ~ [data-scope=\"sidebar\"][data-part=\"menu-action\"] {
  top: 0.875rem;
}

[data-scope=\"sidebar\"][data-part=\"menu-button\"][data-size=\"lg\"] ~ [data-scope=\"sidebar\"][data-part=\"menu-action\"] {
  top: 1.5rem;
}

[data-scope=\"sidebar\"][data-part=\"menu-button\"][data-size=\"sm\"] ~ [data-scope=\"sidebar\"][data-part=\"menu-badge\"] {
  top: 0.875rem;
}

[data-scope=\"sidebar\"][data-part=\"menu-button\"][data-size=\"lg\"] ~ [data-scope=\"sidebar\"][data-part=\"menu-badge\"] {
  top: 1.5rem;
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-side=\"right\"] > [data-scope=\"sidebar\"][data-part=\"rail\"] {
  inset-inline-end: auto;
  inset-inline-start: -1rem;
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] > [data-scope=\"sidebar\"][data-part=\"header\"] {
  overflow-x: hidden;
}

[data-scope=\"sidebar\"][data-part=\"root\"][data-state=\"collapsed\"][data-collapsible=\"icon\"] > [data-scope=\"sidebar\"][data-part=\"footer\"] {
  overflow-x: hidden;
}

@media (hover: hover) {
  [data-scope=\"sidebar\"][data-part=\"root\"][data-side=\"right\"] > [data-scope=\"sidebar\"][data-part=\"rail\"]:hover:not([data-disabled]) {
    box-shadow: inset -2px 0 0 var(--fandhe-color-sidebar-border);
  }

  [data-scope=\"sidebar\"][data-part=\"root\"] > [data-scope=\"sidebar\"][data-part=\"rail\"]:hover:not([data-disabled]):dir(rtl) {
    box-shadow: inset -2px 0 0 var(--fandhe-color-sidebar-border);
  }

  [data-scope=\"sidebar\"][data-part=\"root\"][data-side=\"right\"] > [data-scope=\"sidebar\"][data-part=\"rail\"]:hover:not([data-disabled]):dir(rtl) {
    box-shadow: inset 2px 0 0 var(--fandhe-color-sidebar-border);
  }
}
";

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
    // Bugbot 指摘の回帰: icon collapse 時、header/footer のテキストが
    // レール外へはみ出さないよう root 自身が横方向をクリップする。
    assert!(out.contains("overflow-x: hidden;"));
}

// Bugbot 指摘の回帰: `data-size="lg"` は `padding-inline`（両端一括）で
// `menu_button_base` が予約する action/badge ガター（
// `padding-inline-end`）を上書きしてはならない。開始側のみ広げ、終了側は
// lg トークンでガター予約式を再宣言する。
#[test]
fn css_menu_button_lg_size_preserves_action_badge_gutter() {
    let out = sidebar::stylesheet();
    assert!(out.contains(r#"[data-scope="sidebar"][data-part="menu-button"][data-size="lg"] {"#));
    assert!(out.contains("padding-inline-start: var(--fandhe-space-3);"));
    assert!(out.contains(
        "padding-inline-end: calc(var(--fandhe-space-3) + 1.25rem + var(--fandhe-space-1));"
    ));
}

#[test]
fn css_mobile_collapsed_translate_rule_exists() {
    let out = sidebar::stylesheet();
    // `:not([data-collapsible="none"])` は codex-review P1 是正
    // （collapsible="none" はモバイルでも常時表示という公開契約のため、
    // 退避規則を offcanvas/icon 専用に限定する）。
    assert!(out.contains(
        r#"[data-scope="sidebar"][data-part="root"][data-mobile][data-state="collapsed"]:not([data-collapsible="none"]) {"#
    ));
    assert!(out.contains("transform: translateX(-100%);"));
    assert!(out.contains(
        r#"[data-scope="sidebar"][data-part="root"][data-side="right"][data-mobile][data-state="collapsed"]:not([data-collapsible="none"]) {"#
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

// codex-review P1 指摘の回帰: `collapsible="none"` は headless モジュール
// doc「`data-state`/`data-collapsible`」節の公開契約でモバイルでも常時
// 表示のため、モバイルの退避規則（画面外への `transform`・`visibility:
// hidden`）は `data-collapsible="none"` を明示的に除外していなければ
// ならない。
#[test]
fn css_mobile_collapsed_hide_rules_exclude_collapsible_none() {
    let out = sidebar::stylesheet();
    assert!(out.contains(
        r#"[data-scope="sidebar"][data-part="root"][data-mobile][data-state="collapsed"]:not([data-collapsible="none"]) {"#
    ));
    assert!(out.contains(
        r#"[data-scope="sidebar"][data-part="root"][data-side="right"][data-mobile][data-state="collapsed"]:not([data-collapsible="none"]) {"#
    ));
    assert!(out.contains(
        r#"[data-scope="sidebar"][data-part="root"][data-mobile][data-state="collapsed"]:not([data-collapsible="none"]):dir(rtl) {"#
    ));
    assert!(out.contains(
        r#"[data-scope="sidebar"][data-part="root"][data-side="right"][data-mobile][data-state="collapsed"]:not([data-collapsible="none"]):dir(rtl) {"#
    ));
    // 素の（`:not` 無し）セレクタは一切残っていないこと（後方互換の緩い
    // 部分一致では is-not なしの旧形を見逃すため、除外文字列で確認する）。
    assert!(!out.contains(
        r#"[data-scope="sidebar"][data-part="root"][data-mobile][data-state="collapsed"] {"#
    ));
}
