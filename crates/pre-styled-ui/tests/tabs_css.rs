//! styled Tabs（`size`/`color-palette` variant 展開、イシュー #729）の
//! 決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/switch_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → variants → states）が崩れた場合や意図しない宣言の追加・欠落が
//! あった場合に、この golden テストが即座に検知する。
//!
//! `trigger`/`content` の寸法は `root` の `size` variant が登録する root
//! スコープ CSS custom property（`--fandhe-tabs-trigger-padding`/
//! `-content-padding`）を `var(..., <Md 既定値>)` で参照する形へ変更した
//! （フォールバック値は変更前の固定値と同一、headless 直接利用時の現行
//! 外観を維持する）。選択中 `trigger` の強調色（`border-bottom-color`）は
//! `color-palette` variant が登録する `--fandhe-palette`（`var(...,
//! var(--fandhe-color-accent))`）経由に変更した（フォールバックは Accent
//! 相当で変更前と同一）。
//!
//! # イシュー #1542（参考サイト基準への調整）による golden 更新
//!
//! `crates/pre-styled-ui/src/tabs.rs` `recipe()` の是正（hover・disabled・
//! フォーカスリング canonical 化・トランジション・`data-orientation=
//! "vertical"` 対応・`--fandhe-tabs-font-size`/`-content-padding-inline`
//! の純追加）に伴い、以下を更新した:
//!
//! - `trigger` base: `inline-flex`/`align-items`/`gap`/`font-size`/
//!   `font-weight`/`line-height`/`white-space`/`margin-bottom`/
//!   `border-radius`/`--fandhe-hover-bg` を追加
//! - `trigger` base: `transition-property`/`-duration`/`-timing-function`
//!   を追加（新規ブロック）
//! - `size` variant（5 段）: `--fandhe-tabs-font-size`/
//!   `--fandhe-tabs-content-padding-inline` を各段へ追加
//! - `state` 追加: `trigger[data-disabled]`・`content:focus-visible`・
//!   `root/list/trigger/content[data-orientation="vertical"]`・
//!   `trigger[data-state="active"][data-orientation="vertical"]`・
//!   `trigger:hover:not([data-disabled])`（`@media (hover: hover)` 末尾集約）
//! - `trigger:focus-visible` の `outline` を直書きから
//!   `focus_ring_declarations(FocusRingColor::Palette, ...)` へ canonical 化
//!
//! PR #1816 レビュー指摘（Bugbot「Vertical divider does not span
//! content」）の是正: `root[data-orientation="vertical"]` の
//! `align-items: flex-start` を削除し既定値（`stretch`）へ戻した
//! （`crates/pre-styled-ui/src/tabs.rs` `recipe()` 参照）。
//!
//! # イシュー #2039（shadcn/ui `enclosed` variant 追加）による golden 更新
//!
//! shadcn/ui 突合で判明した「セグメント/ピル型」スタイルの欠落を
//! `TabsVariant::Enclosed` として補完した（`crates/pre-styled-ui/src/tabs.rs`
//! 冒頭 rustdoc「`variant`」節参照）。以下を更新した:
//!
//! - `list` base: `border-bottom`/`background`/`border-radius`/`padding` を
//!   custom property 化（`background`/`border-radius`/`padding` は新規宣言）
//! - `trigger` base: `background`/`border-bottom`/`margin-bottom`/
//!   `border-radius` を custom property 化
//! - `trigger[data-state="active"]` state: `background`/`box-shadow` を追加
//! - `trigger` base: `--fandhe-hover-bg` を固定値
//!   （`var(--fandhe-color-bg-muted)`）から `var(--fandhe-tabs-hover-bg,
//!   var(--fandhe-color-bg-muted))` へ custom property 化（レビュー指摘:
//!   固定値のままだと Enclosed の selected trigger を hover したとき
//!   list と同化して選択解除されたように見える不具合があった）
//! - vertical state（`list`/`trigger`）: `border-inline-end`/
//!   `margin-inline-end`/論理角丸 4 プロパティを custom property 化
//! - `variant` variant（2 値）を新設: `fd-tabs--variant-line`（既定、
//!   フォールバックと同一値。`--fandhe-tabs-hover-bg` も
//!   `var(--fandhe-color-bg-muted)` で不変）/`fd-tabs--variant-enclosed`
//!   （shadcn/ui 既定 variant 相当。`--fandhe-tabs-hover-bg` は selected
//!   trigger と同色の `var(--fandhe-color-bg)`）
//!
//! Line（既定）は上記いずれもフォールバック値と同一の custom property を
//! 明示登録するため、テキストバイトは変わるが Line 選択時のレンダリング
//! 結果（computed style）は不変。
//!
//! # イシュー #2039 codex-review 追補（forced-colors 対応）による golden 更新
//!
//! Enclosed variant の selected trigger は背景色・`box-shadow` のみで選択
//! 状態を表現しており、Windows 強制配色モード（`forced-colors: active`）
//! では両方ともシステム色/`none` へ丸められ選択中 trigger を識別できなく
//! なる指摘（[`crate::status`] と同型の課題）を受け、`stylesheet()`
//! （`crates/pre-styled-ui/src/tabs.rs`）の末尾へ
//! `@media (forced-colors: active)` ブロックを 1 つ追記した。Enclosed の
//! selected trigger（`.fd-tabs--variant-enclosed[data-state="active"]`）へ
//! `border: 1px solid CanvasText` を足す。方向（`data-orientation`）に
//! 依存しない単一セレクタで水平・垂直双方をカバーする。

use fandhe_frontend_pre_styled_ui::tabs;

const TABS_GOLDEN_CSS: &str = r#"[data-scope="tabs"][data-part="list"] {
  display: flex;
  gap: var(--fandhe-space-2);
  border-bottom: var(--fandhe-tabs-list-border-bottom, 1px solid var(--fandhe-color-border));
  background: var(--fandhe-tabs-list-background, transparent);
  border-radius: var(--fandhe-tabs-list-radius, 0);
  padding: var(--fandhe-tabs-list-padding, 0);
}

[data-scope="tabs"][data-part="trigger"] {
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  padding: var(--fandhe-tabs-trigger-padding, var(--fandhe-space-2) var(--fandhe-space-4));
  font-size: var(--fandhe-tabs-font-size, var(--fandhe-font-font-size-sm));
  font-weight: var(--fandhe-font-font-weight-medium);
  line-height: var(--fandhe-font-line-height-normal);
  white-space: nowrap;
  background: var(--fandhe-tabs-trigger-background, transparent);
  color: var(--fandhe-color-fg-muted);
  border: 0;
  border-bottom: var(--fandhe-tabs-trigger-border-bottom, 2px solid transparent);
  margin-bottom: var(--fandhe-tabs-trigger-margin-bottom, -1px);
  border-radius: var(--fandhe-tabs-trigger-radius, var(--fandhe-radius-sm, 0.25rem) var(--fandhe-radius-sm, 0.25rem) 0 0);
  cursor: pointer;
  --fandhe-hover-bg: var(--fandhe-tabs-hover-bg, var(--fandhe-color-bg-muted));
}

[data-scope="tabs"][data-part="trigger"] {
  transition-property: color, background, border-color;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="tabs"][data-part="content"] {
  padding: var(--fandhe-tabs-content-padding, var(--fandhe-space-4) 0);
  color: var(--fandhe-color-fg);
}

[data-scope="tabs"][data-part="root"].fd-tabs--size-xs {
  --fandhe-tabs-trigger-padding: var(--fandhe-space-0-5) var(--fandhe-space-2);
  --fandhe-tabs-content-padding: var(--fandhe-space-2) 0;
  --fandhe-tabs-font-size: var(--fandhe-font-font-size-xs);
  --fandhe-tabs-content-padding-inline: var(--fandhe-space-2);
}

[data-scope="tabs"][data-part="root"].fd-tabs--size-sm {
  --fandhe-tabs-trigger-padding: var(--fandhe-space-1) var(--fandhe-space-3);
  --fandhe-tabs-content-padding: var(--fandhe-space-3) 0;
  --fandhe-tabs-font-size: var(--fandhe-font-font-size-sm);
  --fandhe-tabs-content-padding-inline: var(--fandhe-space-3);
}

[data-scope="tabs"][data-part="root"].fd-tabs--size-md {
  --fandhe-tabs-trigger-padding: var(--fandhe-space-2) var(--fandhe-space-4);
  --fandhe-tabs-content-padding: var(--fandhe-space-4) 0;
  --fandhe-tabs-font-size: var(--fandhe-font-font-size-sm);
  --fandhe-tabs-content-padding-inline: var(--fandhe-space-4);
}

[data-scope="tabs"][data-part="root"].fd-tabs--size-lg {
  --fandhe-tabs-trigger-padding: var(--fandhe-space-3) var(--fandhe-space-5);
  --fandhe-tabs-content-padding: var(--fandhe-space-5) 0;
  --fandhe-tabs-font-size: var(--fandhe-font-font-size-md);
  --fandhe-tabs-content-padding-inline: var(--fandhe-space-5);
}

[data-scope="tabs"][data-part="root"].fd-tabs--size-xl {
  --fandhe-tabs-trigger-padding: var(--fandhe-space-4) var(--fandhe-space-6);
  --fandhe-tabs-content-padding: var(--fandhe-space-6) 0;
  --fandhe-tabs-font-size: var(--fandhe-font-font-size-lg);
  --fandhe-tabs-content-padding-inline: var(--fandhe-space-6);
}

[data-scope="tabs"][data-part="root"].fd-tabs--variant-line {
  --fandhe-tabs-list-border-bottom: 1px solid var(--fandhe-color-border);
  --fandhe-tabs-list-background: transparent;
  --fandhe-tabs-list-radius: 0;
  --fandhe-tabs-list-padding: 0;
  --fandhe-tabs-trigger-border-bottom: 2px solid transparent;
  --fandhe-tabs-trigger-margin-bottom: -1px;
  --fandhe-tabs-trigger-radius: var(--fandhe-radius-sm, 0.25rem) var(--fandhe-radius-sm, 0.25rem) 0 0;
  --fandhe-tabs-trigger-background: transparent;
  --fandhe-tabs-trigger-active-background: transparent;
  --fandhe-tabs-trigger-active-shadow: none;
  --fandhe-tabs-hover-bg: var(--fandhe-color-bg-muted);
  --fandhe-tabs-vertical-list-border-inline-end: 1px solid var(--fandhe-color-border);
  --fandhe-tabs-vertical-trigger-border-inline-end: 2px solid transparent;
  --fandhe-tabs-vertical-trigger-margin-inline-end: -1px;
  --fandhe-tabs-vertical-trigger-radius-start: var(--fandhe-radius-sm, 0.25rem);
  --fandhe-tabs-vertical-trigger-radius-end: 0;
}

[data-scope="tabs"][data-part="root"].fd-tabs--variant-enclosed {
  --fandhe-tabs-list-border-bottom: 0;
  --fandhe-tabs-list-background: var(--fandhe-color-bg-muted);
  --fandhe-tabs-list-radius: var(--fandhe-radius-md);
  --fandhe-tabs-list-padding: var(--fandhe-space-1);
  --fandhe-tabs-trigger-border-bottom: 0;
  --fandhe-tabs-trigger-margin-bottom: 0;
  --fandhe-tabs-trigger-radius: var(--fandhe-radius-sm, 0.25rem);
  --fandhe-tabs-trigger-background: transparent;
  --fandhe-tabs-trigger-active-background: var(--fandhe-color-bg);
  --fandhe-tabs-trigger-active-shadow: var(--fandhe-shadow-sm);
  --fandhe-tabs-hover-bg: var(--fandhe-color-bg);
  --fandhe-tabs-vertical-list-border-inline-end: 0;
  --fandhe-tabs-vertical-trigger-border-inline-end: 0;
  --fandhe-tabs-vertical-trigger-margin-inline-end: 0;
  --fandhe-tabs-vertical-trigger-radius-start: var(--fandhe-radius-sm, 0.25rem);
  --fandhe-tabs-vertical-trigger-radius-end: var(--fandhe-radius-sm, 0.25rem);
}

[data-scope="tabs"][data-part="root"].fd-tabs--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="tabs"][data-part="root"].fd-tabs--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="tabs"][data-part="root"].fd-tabs--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="tabs"][data-part="root"].fd-tabs--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="tabs"][data-part="root"].fd-tabs--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="tabs"][data-part="root"].fd-tabs--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="tabs"][data-part="trigger"][data-state="active"] {
  color: var(--fandhe-color-fg);
  border-bottom-color: var(--fandhe-palette, var(--fandhe-color-accent));
  background: var(--fandhe-tabs-trigger-active-background, transparent);
  box-shadow: var(--fandhe-tabs-trigger-active-shadow, none);
}

[data-scope="tabs"][data-part="content"][data-state="inactive"] {
  display: none;
}

[data-scope="tabs"][data-part="trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="tabs"][data-part="trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="tabs"][data-part="content"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope="tabs"][data-part="root"][data-orientation="vertical"] {
  display: flex;
}

[data-scope="tabs"][data-part="list"][data-orientation="vertical"] {
  flex-direction: column;
  border-bottom: 0;
  border-inline-end: var(--fandhe-tabs-vertical-list-border-inline-end, 1px solid var(--fandhe-color-border));
}

[data-scope="tabs"][data-part="trigger"][data-orientation="vertical"] {
  justify-content: flex-start;
  border-bottom: 0;
  margin-bottom: 0;
  border-inline-end: var(--fandhe-tabs-vertical-trigger-border-inline-end, 2px solid transparent);
  margin-inline-end: var(--fandhe-tabs-vertical-trigger-margin-inline-end, -1px);
  border-start-start-radius: var(--fandhe-tabs-vertical-trigger-radius-start, var(--fandhe-radius-sm, 0.25rem));
  border-end-start-radius: var(--fandhe-tabs-vertical-trigger-radius-start, var(--fandhe-radius-sm, 0.25rem));
  border-start-end-radius: var(--fandhe-tabs-vertical-trigger-radius-end, 0);
  border-end-end-radius: var(--fandhe-tabs-vertical-trigger-radius-end, 0);
}

[data-scope="tabs"][data-part="trigger"][data-state="active"][data-orientation="vertical"] {
  border-inline-end-color: var(--fandhe-palette, var(--fandhe-color-accent));
}

[data-scope="tabs"][data-part="content"][data-orientation="vertical"] {
  flex: 1;
  padding: 0 var(--fandhe-tabs-content-padding-inline, var(--fandhe-space-4));
}

@media (hover: hover) {
  [data-scope="tabs"][data-part="trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
    color: var(--fandhe-color-fg);
  }
}


@media (forced-colors: active) {
  [data-scope="tabs"][data-part="trigger"].fd-tabs--variant-enclosed[data-state="active"] {
    border: 1px solid CanvasText;
  }
}
"#;

#[test]
fn tabs_stylesheet_matches_golden_fixture() {
    assert_eq!(tabs::stylesheet(), TABS_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(tabs::stylesheet(), tabs::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = tabs::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}
