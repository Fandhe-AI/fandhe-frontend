//! styled Tab Nav（イシュー #996、イシュー #1541 で参考サイト基準へ調整、
//! イシュー #3125 で pill variant / palette 軸を純追加、イシュー #3126 で
//! bar variant を純追加）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/tabs_css.rs` と同型の golden fixture テスト。
//! イシュー #1541 で `crate::tabs` の `pub(crate)` ヘルパ共有をやめ、
//! `tab_nav.rs` は自前の宣言列を持つ（`tab_nav.rs` モジュール冒頭 rustdoc
//! 「参考サイト基準への調整」節参照。並列実行される兄弟イシュー #1542 が
//! `tabs.rs` を変更しても本 golden が影響を受けないようにする独立化）。
//! 本 golden はその宣言列（size 軸・hover・フォーカスリング・トランジション・
//! pill variant・palette 軸・bar variant・forced-colors 補強を含む）の出力を
//! バイト単位で固定する。
//!
//! イシュー #3125: `TAB_NAV_GOLDEN_CSS_BEFORE_3125` は本イシュー直前
//! （size 軸までの既存出力）の golden を保持し、その各ブロック（`\n\n` 区切り）
//! が現行 `TAB_NAV_GOLDEN_CSS` の中に元の順序どおり全て含まれることを固定
//! する「純追加の固定」テストを持つ。既存ブロックの変更（削除・順序入れ替え・
//! 内容改変のいずれか）が起きた場合にこのテストが検知する。
//!
//! イシュー #3126: `TAB_NAV_GOLDEN_CSS_BEFORE_3126` は本イシュー直前
//! （pill variant / palette 軸までの既存出力、`TAB_NAV_GOLDEN_CSS` と同一）
//! の golden を保持し、同型の「純追加の固定」テストを持つ。

use fandhe_frontend_pre_styled_ui::tab_nav;

/// イシュー #3125 直前（pill variant / palette 軸追加前）の golden 出力。
/// 本イシューが「純追加のみ」であることを固定するための比較対象であり、
/// `tab_nav::stylesheet()` の戻り値としては使わない。
const TAB_NAV_GOLDEN_CSS_BEFORE_3125: &str = r#"[data-scope="tab-nav"][data-part="root"] {
  display: flex;
  gap: var(--fandhe-space-2);
  border-bottom: 1px solid var(--fandhe-color-border);
}

[data-scope="tab-nav"][data-part="link"] {
  padding: var(--fandhe-tab-nav-link-padding, var(--fandhe-space-2) var(--fandhe-space-4));
  font-size: var(--fandhe-tab-nav-font-size, var(--fandhe-font-font-size-sm));
  background: transparent;
  color: var(--fandhe-color-fg-muted);
  border: 0;
  border-bottom: 2px solid transparent;
  border-radius: var(--fandhe-radius-sm, 0.25rem) var(--fandhe-radius-sm, 0.25rem) 0 0;
  cursor: pointer;
  text-decoration: none;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="tab-nav"][data-part="link"] {
  transition-property: color, background, border-color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--size-xs {
  --fandhe-tab-nav-link-padding: var(--fandhe-space-0-5) var(--fandhe-space-2);
  --fandhe-tab-nav-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--size-sm {
  --fandhe-tab-nav-link-padding: var(--fandhe-space-1) var(--fandhe-space-3);
  --fandhe-tab-nav-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--size-md {
  --fandhe-tab-nav-link-padding: var(--fandhe-space-2) var(--fandhe-space-4);
  --fandhe-tab-nav-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--size-lg {
  --fandhe-tab-nav-link-padding: var(--fandhe-space-3) var(--fandhe-space-5);
  --fandhe-tab-nav-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--size-xl {
  --fandhe-tab-nav-link-padding: var(--fandhe-space-4) var(--fandhe-space-6);
  --fandhe-tab-nav-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="tab-nav"][data-part="link"][aria-current="page"] {
  color: var(--fandhe-color-fg);
  border-bottom-color: var(--fandhe-palette, var(--fandhe-color-accent));
  font-weight: var(--fandhe-font-font-weight-medium);
}

[data-scope="tab-nav"][data-part="link"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="tab-nav"][data-part="link"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
    color: var(--fandhe-color-fg);
  }
}
"#;

/// イシュー #3126 直前（bar variant 追加前、イシュー #3125 到達時点）の
/// golden 出力。`TAB_NAV_GOLDEN_CSS`（旧）と同一内容。
const TAB_NAV_GOLDEN_CSS_BEFORE_3126: &str = r#"[data-scope="tab-nav"][data-part="root"] {
  display: flex;
  gap: var(--fandhe-space-2);
  border-bottom: 1px solid var(--fandhe-color-border);
}

[data-scope="tab-nav"][data-part="link"] {
  padding: var(--fandhe-tab-nav-link-padding, var(--fandhe-space-2) var(--fandhe-space-4));
  font-size: var(--fandhe-tab-nav-font-size, var(--fandhe-font-font-size-sm));
  background: transparent;
  color: var(--fandhe-color-fg-muted);
  border: 0;
  border-bottom: 2px solid transparent;
  border-radius: var(--fandhe-radius-sm, 0.25rem) var(--fandhe-radius-sm, 0.25rem) 0 0;
  cursor: pointer;
  text-decoration: none;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="tab-nav"][data-part="link"] {
  transition-property: color, background, border-color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="tab-nav"][data-part="link"] {
  border-bottom: var(--fandhe-tab-nav-link-border-bottom, 2px solid transparent);
  border-radius: var(--fandhe-tab-nav-link-radius, var(--fandhe-radius-sm, 0.25rem) var(--fandhe-radius-sm, 0.25rem) 0 0);
  --fandhe-hover-bg: var(--fandhe-tab-nav-hover-bg, var(--fandhe-color-bg-muted));
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--size-xs {
  --fandhe-tab-nav-link-padding: var(--fandhe-space-0-5) var(--fandhe-space-2);
  --fandhe-tab-nav-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--size-sm {
  --fandhe-tab-nav-link-padding: var(--fandhe-space-1) var(--fandhe-space-3);
  --fandhe-tab-nav-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--size-md {
  --fandhe-tab-nav-link-padding: var(--fandhe-space-2) var(--fandhe-space-4);
  --fandhe-tab-nav-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--size-lg {
  --fandhe-tab-nav-link-padding: var(--fandhe-space-3) var(--fandhe-space-5);
  --fandhe-tab-nav-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--size-xl {
  --fandhe-tab-nav-link-padding: var(--fandhe-space-4) var(--fandhe-space-6);
  --fandhe-tab-nav-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--variant-pill {
  border-bottom: 0;
  background: var(--fandhe-color-bg-muted);
  border-radius: var(--fandhe-radius-md);
  padding: var(--fandhe-space-1);
  --fandhe-tab-nav-link-border-bottom: 0;
  --fandhe-tab-nav-link-radius: var(--fandhe-radius-sm, 0.25rem);
  --fandhe-tab-nav-hover-bg: var(--fandhe-color-bg);
  --fandhe-tab-nav-current-bg: var(--fandhe-tab-nav-pill-current-bg, var(--fandhe-color-bg));
  --fandhe-tab-nav-current-fg: var(--fandhe-tab-nav-pill-current-fg, var(--fandhe-color-fg));
  --fandhe-tab-nav-current-shadow: var(--fandhe-tab-nav-pill-current-shadow, var(--fandhe-shadow-sm));
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
  --fandhe-tab-nav-pill-current-bg: var(--fandhe-palette-subtle);
  --fandhe-tab-nav-pill-current-fg: var(--fandhe-palette-fg-subtle);
  --fandhe-tab-nav-pill-current-shadow: none;
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
  --fandhe-tab-nav-pill-current-bg: var(--fandhe-palette-subtle);
  --fandhe-tab-nav-pill-current-fg: var(--fandhe-palette-fg-subtle);
  --fandhe-tab-nav-pill-current-shadow: none;
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
  --fandhe-tab-nav-pill-current-bg: var(--fandhe-palette-subtle);
  --fandhe-tab-nav-pill-current-fg: var(--fandhe-palette-fg-subtle);
  --fandhe-tab-nav-pill-current-shadow: none;
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
  --fandhe-tab-nav-pill-current-bg: var(--fandhe-palette-subtle);
  --fandhe-tab-nav-pill-current-fg: var(--fandhe-palette-fg-subtle);
  --fandhe-tab-nav-pill-current-shadow: none;
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
  --fandhe-tab-nav-pill-current-bg: var(--fandhe-palette-subtle);
  --fandhe-tab-nav-pill-current-fg: var(--fandhe-palette-fg-subtle);
  --fandhe-tab-nav-pill-current-shadow: none;
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
  --fandhe-tab-nav-pill-current-bg: var(--fandhe-palette-subtle);
  --fandhe-tab-nav-pill-current-fg: var(--fandhe-palette-fg-subtle);
  --fandhe-tab-nav-pill-current-shadow: none;
}

[data-scope="tab-nav"][data-part="link"][aria-current="page"] {
  color: var(--fandhe-color-fg);
  border-bottom-color: var(--fandhe-palette, var(--fandhe-color-accent));
  font-weight: var(--fandhe-font-font-weight-medium);
}

[data-scope="tab-nav"][data-part="link"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="tab-nav"][data-part="link"][aria-current="page"] {
  background: var(--fandhe-tab-nav-current-bg, transparent);
  color: var(--fandhe-tab-nav-current-fg, var(--fandhe-color-fg));
  box-shadow: var(--fandhe-tab-nav-current-shadow, none);
  --fandhe-hover-bg: var(--fandhe-tab-nav-current-bg, var(--fandhe-tab-nav-hover-bg, var(--fandhe-color-bg-muted)));
  --fandhe-tab-nav-hover-fg: var(--fandhe-tab-nav-current-fg, var(--fandhe-color-fg));
}

@media (hover: hover) {
  [data-scope="tab-nav"][data-part="link"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
    color: var(--fandhe-color-fg);
  }

  [data-scope="tab-nav"][data-part="link"]:hover:not([data-disabled]) {
    color: var(--fandhe-tab-nav-hover-fg, var(--fandhe-color-fg));
  }
}


@media (forced-colors: active) {
  [data-scope="tab-nav"][data-part="root"].fd-tab-nav--variant-pill [data-scope="tab-nav"][data-part="link"][aria-current="page"] {
    border: 1px solid CanvasText;
  }
}
"#;

/// イシュー #3126: bar variant（`TabNavVariant::Bar`）を純追加した現行の
/// golden 出力。
const TAB_NAV_GOLDEN_CSS: &str = r#"[data-scope="tab-nav"][data-part="root"] {
  display: flex;
  gap: var(--fandhe-space-2);
  border-bottom: 1px solid var(--fandhe-color-border);
}

[data-scope="tab-nav"][data-part="link"] {
  padding: var(--fandhe-tab-nav-link-padding, var(--fandhe-space-2) var(--fandhe-space-4));
  font-size: var(--fandhe-tab-nav-font-size, var(--fandhe-font-font-size-sm));
  background: transparent;
  color: var(--fandhe-color-fg-muted);
  border: 0;
  border-bottom: 2px solid transparent;
  border-radius: var(--fandhe-radius-sm, 0.25rem) var(--fandhe-radius-sm, 0.25rem) 0 0;
  cursor: pointer;
  text-decoration: none;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope="tab-nav"][data-part="link"] {
  transition-property: color, background, border-color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="tab-nav"][data-part="link"] {
  border-bottom: var(--fandhe-tab-nav-link-border-bottom, 2px solid transparent);
  border-radius: var(--fandhe-tab-nav-link-radius, var(--fandhe-radius-sm, 0.25rem) var(--fandhe-radius-sm, 0.25rem) 0 0);
  --fandhe-hover-bg: var(--fandhe-tab-nav-hover-bg, var(--fandhe-color-bg-muted));
}

[data-scope="tab-nav"][data-part="link"] {
  flex: var(--fandhe-tab-nav-link-flex, 0 1 auto);
  border-inline-end: var(--fandhe-tab-nav-link-divider, 0);
  min-width: var(--fandhe-tab-nav-link-min-width, auto);
  overflow: var(--fandhe-tab-nav-link-overflow, visible);
  text-overflow: var(--fandhe-tab-nav-link-text-overflow, clip);
  white-space: var(--fandhe-tab-nav-link-white-space, normal);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--size-xs {
  --fandhe-tab-nav-link-padding: var(--fandhe-space-0-5) var(--fandhe-space-2);
  --fandhe-tab-nav-font-size: var(--fandhe-font-font-size-xs);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--size-sm {
  --fandhe-tab-nav-link-padding: var(--fandhe-space-1) var(--fandhe-space-3);
  --fandhe-tab-nav-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--size-md {
  --fandhe-tab-nav-link-padding: var(--fandhe-space-2) var(--fandhe-space-4);
  --fandhe-tab-nav-font-size: var(--fandhe-font-font-size-sm);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--size-lg {
  --fandhe-tab-nav-link-padding: var(--fandhe-space-3) var(--fandhe-space-5);
  --fandhe-tab-nav-font-size: var(--fandhe-font-font-size-md);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--size-xl {
  --fandhe-tab-nav-link-padding: var(--fandhe-space-4) var(--fandhe-space-6);
  --fandhe-tab-nav-font-size: var(--fandhe-font-font-size-lg);
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--variant-pill {
  border-bottom: 0;
  background: var(--fandhe-color-bg-muted);
  border-radius: var(--fandhe-radius-md);
  padding: var(--fandhe-space-1);
  --fandhe-tab-nav-link-border-bottom: 0;
  --fandhe-tab-nav-link-radius: var(--fandhe-radius-sm, 0.25rem);
  --fandhe-tab-nav-hover-bg: var(--fandhe-color-bg);
  --fandhe-tab-nav-current-bg: var(--fandhe-tab-nav-pill-current-bg, var(--fandhe-color-bg));
  --fandhe-tab-nav-current-fg: var(--fandhe-tab-nav-pill-current-fg, var(--fandhe-color-fg));
  --fandhe-tab-nav-current-shadow: var(--fandhe-tab-nav-pill-current-shadow, var(--fandhe-shadow-sm));
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--variant-bar {
  gap: 0;
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-color-bg);
  box-shadow: var(--fandhe-shadow-sm);
  overflow: hidden;
  text-align: center;
  --fandhe-tab-nav-link-flex: 1 1 0%;
  --fandhe-tab-nav-link-divider: 1px solid var(--fandhe-color-border);
  --fandhe-tab-nav-link-radius: 0;
  --fandhe-tab-nav-focus-ring-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));
  --fandhe-tab-nav-link-min-width: 0;
  --fandhe-tab-nav-link-overflow: hidden;
  --fandhe-tab-nav-link-text-overflow: ellipsis;
  --fandhe-tab-nav-link-white-space: nowrap;
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
  --fandhe-tab-nav-pill-current-bg: var(--fandhe-palette-subtle);
  --fandhe-tab-nav-pill-current-fg: var(--fandhe-palette-fg-subtle);
  --fandhe-tab-nav-pill-current-shadow: none;
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
  --fandhe-tab-nav-pill-current-bg: var(--fandhe-palette-subtle);
  --fandhe-tab-nav-pill-current-fg: var(--fandhe-palette-fg-subtle);
  --fandhe-tab-nav-pill-current-shadow: none;
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
  --fandhe-tab-nav-pill-current-bg: var(--fandhe-palette-subtle);
  --fandhe-tab-nav-pill-current-fg: var(--fandhe-palette-fg-subtle);
  --fandhe-tab-nav-pill-current-shadow: none;
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
  --fandhe-tab-nav-pill-current-bg: var(--fandhe-palette-subtle);
  --fandhe-tab-nav-pill-current-fg: var(--fandhe-palette-fg-subtle);
  --fandhe-tab-nav-pill-current-shadow: none;
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
  --fandhe-tab-nav-pill-current-bg: var(--fandhe-palette-subtle);
  --fandhe-tab-nav-pill-current-fg: var(--fandhe-palette-fg-subtle);
  --fandhe-tab-nav-pill-current-shadow: none;
}

[data-scope="tab-nav"][data-part="root"].fd-tab-nav--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
  --fandhe-tab-nav-pill-current-bg: var(--fandhe-palette-subtle);
  --fandhe-tab-nav-pill-current-fg: var(--fandhe-palette-fg-subtle);
  --fandhe-tab-nav-pill-current-shadow: none;
}

[data-scope="tab-nav"][data-part="link"][aria-current="page"] {
  color: var(--fandhe-color-fg);
  border-bottom-color: var(--fandhe-palette, var(--fandhe-color-accent));
  font-weight: var(--fandhe-font-font-weight-medium);
}

[data-scope="tab-nav"][data-part="link"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="tab-nav"][data-part="link"]:last-child {
  border-inline-end: 0;
}

[data-scope="tab-nav"][data-part="link"]:focus-visible {
  outline-offset: var(--fandhe-tab-nav-focus-ring-offset, var(--fandhe-focus-ring-offset, 2px));
}

[data-scope="tab-nav"][data-part="link"][aria-current="page"] {
  background: var(--fandhe-tab-nav-current-bg, transparent);
  color: var(--fandhe-tab-nav-current-fg, var(--fandhe-color-fg));
  box-shadow: var(--fandhe-tab-nav-current-shadow, none);
  --fandhe-hover-bg: var(--fandhe-tab-nav-current-bg, var(--fandhe-tab-nav-hover-bg, var(--fandhe-color-bg-muted)));
  --fandhe-tab-nav-hover-fg: var(--fandhe-tab-nav-current-fg, var(--fandhe-color-fg));
}

@media (hover: hover) {
  [data-scope="tab-nav"][data-part="link"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
    color: var(--fandhe-color-fg);
  }

  [data-scope="tab-nav"][data-part="link"]:hover:not([data-disabled]) {
    color: var(--fandhe-tab-nav-hover-fg, var(--fandhe-color-fg));
  }
}


@media (forced-colors: active) {
  [data-scope="tab-nav"][data-part="root"].fd-tab-nav--variant-pill [data-scope="tab-nav"][data-part="link"][aria-current="page"] {
    border: 1px solid CanvasText;
  }
}
"#;

#[test]
fn tab_nav_stylesheet_matches_golden_fixture() {
    assert_eq!(tab_nav::stylesheet(), TAB_NAV_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(tab_nav::stylesheet(), tab_nav::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = tab_nav::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

/// イシュー #3125: 「純追加の固定」テスト。イシュー #3125 直前の golden
/// （`TAB_NAV_GOLDEN_CSS_BEFORE_3125`）が `\n\n` 区切りで持つ各ブロックが、
/// 現行の `tab_nav::stylesheet()` の中に元の相対順序どおり全て含まれる
/// ことを固定する。既存ブロックの削除・改変・順序入れ替えのいずれかが
/// 起きた場合にこのテストが検知する（新規ブロックの追加だけは許容する）。
///
/// `@media (hover: hover) { ... }` ブロックは本イシューで内側に 2 つ目の
/// hover 規則を追加したため、ラッパーごとの完全一致は成立しない。
/// `@media` で始まるブロックに限り、末尾の閉じ `\n}`（ラッパー自身の
/// 閉じ括弧）を 1 つだけ除いた「内側の規則まで」を部分一致対象とする
/// （内側の規則自体の削除・改変は引き続き検知する。閉じ括弧 1 つ分だけの
/// 緩和は `@media` ラッパーのみに限定し、他ブロックには適用しない）。
#[test]
fn stylesheet_is_pure_addition_over_pre_3125_golden() {
    let before_blocks: Vec<&str> = TAB_NAV_GOLDEN_CSS_BEFORE_3125
        .trim_end_matches('\n')
        .split("\n\n")
        .collect();
    let current = tab_nav::stylesheet();

    let mut search_from = 0usize;
    for block in before_blocks {
        let needle = if block.starts_with("@media") {
            block.strip_suffix("\n}").unwrap_or(block)
        } else {
            block
        };
        let found = current[search_from..].find(needle);
        assert!(
            found.is_some(),
            "pre-#3125 のブロックが現行 stylesheet に見つからない、または順序が崩れている: {needle:?}"
        );
        search_from += found.unwrap() + needle.len();
    }
}

/// イシュー #3126: 「純追加の固定」テスト（#3125 版と同型）。イシュー #3126
/// 直前の golden（`TAB_NAV_GOLDEN_CSS_BEFORE_3126`、#3125 到達時点の golden
/// そのもの）が `\n\n` 区切りで持つ各ブロックが、現行の
/// `tab_nav::stylesheet()` の中に元の相対順序どおり全て含まれることを
/// 固定する。既存ブロックの削除・改変・順序入れ替えのいずれかが起きた
/// 場合にこのテストが検知する（新規ブロックの追加だけは許容する）。
#[test]
fn stylesheet_is_pure_addition_over_pre_3126_golden() {
    let before_blocks: Vec<&str> = TAB_NAV_GOLDEN_CSS_BEFORE_3126
        .trim_end_matches('\n')
        .split("\n\n")
        .collect();
    let current = tab_nav::stylesheet();

    let mut search_from = 0usize;
    for block in before_blocks {
        let needle = if block.starts_with("@media") {
            block.strip_suffix("\n}").unwrap_or(block)
        } else {
            block
        };
        let found = current[search_from..].find(needle);
        assert!(
            found.is_some(),
            "pre-#3126 のブロックが現行 stylesheet に見つからない、または順序が崩れている: {needle:?}"
        );
        search_from += found.unwrap() + needle.len();
    }
}
