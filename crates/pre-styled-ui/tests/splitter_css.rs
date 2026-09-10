//! styled Splitter（イシュー #826）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/steps_css.rs`/`slider`（`src/slider.rs` 内
//! インラインテスト）の golden fixture テストの前例に倣い、`stylesheet()`
//! が返す CSS 全文をバイト単位で固定する。出力順（base → variants →
//! states → pseudo-elements → hover）が崩れた場合や意図しない宣言の
//! 追加・欠落があった場合に、この golden テストが即座に検知する。
//! `resize-trigger` の `::after`（イシュー #2202、`SlotRecipe::
//! pseudo_element` 経由）は states の後・`@media (hover: hover)` の前に
//! 出力される（`crate::recipe::SlotRecipe::css` rustdoc の出力順序節参照）。

use fandhe_frontend_pre_styled_ui::splitter;

const SPLITTER_GOLDEN_CSS: &str = r#"[data-scope="splitter"][data-part="root"] {
  display: flex;
  align-items: stretch;
  width: 100%;
  box-sizing: border-box;
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-color-bg);
  overflow: hidden;
}

[data-scope="splitter"][data-part="panel"] {
  flex-basis: var(--fandhe-splitter-size, auto);
  flex-grow: 0;
  flex-shrink: 1;
  overflow: hidden;
  box-sizing: border-box;
  padding: var(--fandhe-splitter-panel-padding, var(--fandhe-space-3));
}

[data-scope="splitter"][data-part="resize-trigger"] {
  flex: 0 0 var(--fandhe-splitter-trigger-size, 0.25rem);
  min-width: 0;
  min-height: 0;
  background: var(--fandhe-color-border);
  cursor: col-resize;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--fandhe-radius-full, 999px);
  --fandhe-hover-bg: var(--fandhe-splitter-root-disabled-hover-bg, var(--fandhe-palette-emphasized, var(--fandhe-color-accent-emphasized)));
  position: relative;
  --fandhe-splitter-hit-inset: 0 calc(-1 * var(--fandhe-splitter-hit-extension, 0.25rem));
}

[data-scope="splitter"][data-part="resize-trigger"] {
  transition-property: background, box-shadow;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="splitter"][data-part="resize-trigger-indicator"] {
  width: 0.75rem;
  height: 0.75rem;
  flex-shrink: 0;
  background: var(--fandhe-color-bg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-full, 999px);
  box-shadow: var(--fandhe-shadow-sm);
  pointer-events: none;
}

[data-scope="splitter"][data-part="root"].fd-splitter--size-xs {
  --fandhe-splitter-trigger-size: 0.0625rem;
  --fandhe-splitter-panel-padding: var(--fandhe-space-1);
}

[data-scope="splitter"][data-part="root"].fd-splitter--size-sm {
  --fandhe-splitter-trigger-size: 0.125rem;
  --fandhe-splitter-panel-padding: var(--fandhe-space-2);
}

[data-scope="splitter"][data-part="root"].fd-splitter--size-md {
  --fandhe-splitter-trigger-size: 0.25rem;
  --fandhe-splitter-panel-padding: var(--fandhe-space-3);
}

[data-scope="splitter"][data-part="root"].fd-splitter--size-lg {
  --fandhe-splitter-trigger-size: 0.375rem;
  --fandhe-splitter-panel-padding: var(--fandhe-space-4);
}

[data-scope="splitter"][data-part="root"].fd-splitter--size-xl {
  --fandhe-splitter-trigger-size: 0.5rem;
  --fandhe-splitter-panel-padding: var(--fandhe-space-5);
}

[data-scope="splitter"][data-part="root"].fd-splitter--color-palette-accent {
  --fandhe-palette: var(--fandhe-color-accent);
  --fandhe-palette-emphasized: var(--fandhe-color-accent-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-accent-fg);
  --fandhe-palette-subtle: var(--fandhe-color-accent-subtle);
  --fandhe-palette-muted: var(--fandhe-color-accent-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-accent-fg-subtle);
}

[data-scope="splitter"][data-part="root"].fd-splitter--color-palette-info {
  --fandhe-palette: var(--fandhe-color-info);
  --fandhe-palette-emphasized: var(--fandhe-color-info-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-info-fg);
  --fandhe-palette-subtle: var(--fandhe-color-info-subtle);
  --fandhe-palette-muted: var(--fandhe-color-info-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-info-fg-subtle);
}

[data-scope="splitter"][data-part="root"].fd-splitter--color-palette-success {
  --fandhe-palette: var(--fandhe-color-success);
  --fandhe-palette-emphasized: var(--fandhe-color-success-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-success-fg);
  --fandhe-palette-subtle: var(--fandhe-color-success-subtle);
  --fandhe-palette-muted: var(--fandhe-color-success-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-success-fg-subtle);
}

[data-scope="splitter"][data-part="root"].fd-splitter--color-palette-warning {
  --fandhe-palette: var(--fandhe-color-warning);
  --fandhe-palette-emphasized: var(--fandhe-color-warning-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-warning-fg);
  --fandhe-palette-subtle: var(--fandhe-color-warning-subtle);
  --fandhe-palette-muted: var(--fandhe-color-warning-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-warning-fg-subtle);
}

[data-scope="splitter"][data-part="root"].fd-splitter--color-palette-danger {
  --fandhe-palette: var(--fandhe-color-danger);
  --fandhe-palette-emphasized: var(--fandhe-color-danger-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-danger-fg);
  --fandhe-palette-subtle: var(--fandhe-color-danger-subtle);
  --fandhe-palette-muted: var(--fandhe-color-danger-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-danger-fg-subtle);
}

[data-scope="splitter"][data-part="root"].fd-splitter--color-palette-neutral {
  --fandhe-palette: var(--fandhe-color-neutral);
  --fandhe-palette-emphasized: var(--fandhe-color-neutral-emphasized);
  --fandhe-palette-fg: var(--fandhe-color-neutral-fg);
  --fandhe-palette-subtle: var(--fandhe-color-neutral-subtle);
  --fandhe-palette-muted: var(--fandhe-color-neutral-muted);
  --fandhe-palette-fg-subtle: var(--fandhe-color-neutral-fg-subtle);
}

[data-scope="splitter"][data-part="root"][data-orientation="vertical"] {
  flex-direction: column;
}

[data-scope="splitter"][data-part="root"][data-disabled] {
  opacity: 0.5;
  --fandhe-splitter-root-disabled-hover-bg: var(--fandhe-color-border);
}

[data-scope="splitter"][data-part="resize-trigger"][data-orientation="vertical"] {
  cursor: row-resize;
  --fandhe-splitter-hit-inset: calc(-1 * var(--fandhe-splitter-hit-extension, 0.25rem)) 0;
}

[data-scope="splitter"][data-part="resize-trigger"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope="splitter"][data-part="resize-trigger"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-palette, var(--fandhe-color-focus-ring, var(--fandhe-color-accent)));
  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));
}

[data-scope="splitter"][data-part="resize-trigger"]::after {
  content: "";
  position: absolute;
  inset: var(--fandhe-splitter-hit-inset);
}

@media (hover: hover) {
  [data-scope="splitter"][data-part="resize-trigger"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}
"#;

#[test]
fn splitter_stylesheet_matches_golden_fixture() {
    assert_eq!(splitter::stylesheet(), SPLITTER_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(splitter::stylesheet(), splitter::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = splitter::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

#[test]
fn stylesheet_never_references_external_resources() {
    let css = splitter::stylesheet();
    assert!(!css.contains("url("));
}

// イシュー #1536 codex-review P1 是正: `root(disabled: true)` と
// `resize_trigger(disabled: false)` を独立指定した構成（headless 層の
// API 契約上成立する）でも、resize-trigger の hover 強調が抑止される
// ことを固定する。`root` の `[data-disabled]` 規則が
// `--fandhe-splitter-root-disabled-hover-bg: var(--fandhe-color-border)`
// を定義し、CSS の通常のプロパティ継承（子孫結合子を使わない、
// `SlotRecipe` の制約に沿う）で resize-trigger の `--fandhe-hover-bg`
// フォールバック連鎖の最優先値として効くことを検証する。値を
// `transparent` ではなく `var(--fandhe-color-border)`（resize-trigger の
// base 規則と同じ細線色）にするのは、codex-review 再指摘（同一 Issue
// 別ラウンド）是正: `transparent` だと hover 中に境界線そのものが
// 消える視覚回帰になるため、強調色のみを抑止し境界線は見え続ける値へ
// 変更した。
#[test]
fn root_disabled_rule_defines_hover_bg_override_for_resize_trigger_inheritance() {
    let css = splitter::stylesheet();
    let root_disabled_rule = css
        .find(r#"[data-scope="splitter"][data-part="root"][data-disabled] {"#)
        .expect("root disabled rule must exist");
    let root_disabled_block_end = css[root_disabled_rule..]
        .find('}')
        .expect("root disabled rule must be closed");
    let root_disabled_block =
        &css[root_disabled_rule..root_disabled_rule + root_disabled_block_end];
    assert!(
        root_disabled_block
            .contains("--fandhe-splitter-root-disabled-hover-bg: var(--fandhe-color-border);"),
        "root disabled rule must define the inherited hover-bg override: {root_disabled_block}"
    );

    let resize_trigger_base_rule = css
        .find(r#"[data-scope="splitter"][data-part="resize-trigger"] {"#)
        .expect("resize-trigger base rule must exist");
    let resize_trigger_block_end = css[resize_trigger_base_rule..]
        .find('}')
        .expect("resize-trigger base rule must be closed");
    let resize_trigger_block =
        &css[resize_trigger_base_rule..resize_trigger_base_rule + resize_trigger_block_end];
    assert!(
        resize_trigger_block.contains(
            "--fandhe-hover-bg: var(--fandhe-splitter-root-disabled-hover-bg, var(--fandhe-palette-emphasized, var(--fandhe-color-accent-emphasized)));"
        ),
        "resize-trigger base rule must reference the root-disabled override as its \
         highest-priority fallback: {resize_trigger_block}"
    );
}

// イシュー #2202: `resize-trigger` の `::after` が見えないヒットエリア
// 拡張（shadcn/ui `ResizableHandle` 相当）を提供することを固定する。
// `::after` 規則自体は無条件 1 本（`content`/`position: absolute`/
// `inset: var(--fandhe-splitter-hit-inset)`）だが、`--fandhe-splitter-
// hit-inset` は base（水平既定）と `data-orientation="vertical"` state
// のそれぞれが疑似要素へ継承される custom property として個別に定義する
// ため、両ブロックにそれぞれ水平/垂直の値が存在することも併せて検証する
// （疑似要素は `SlotRecipe` 上、状態条件付きの規則を直接持てないため、
// custom property 継承で向きを切り替える設計、`crate::splitter` rustdoc
// 「イシュー #2202」節参照）。
#[test]
fn resize_trigger_after_pseudo_element_expands_hit_area_via_inherited_inset() {
    let css = splitter::stylesheet();

    // `::after` 規則は CSS 全文で 1 回だけ出現する（無条件・状態条件別に
    // 複数登録しない設計であることの固定）。
    assert_eq!(
        css.matches(r#"[data-scope="splitter"][data-part="resize-trigger"]::after {"#)
            .count(),
        1,
        "resize-trigger ::after rule must appear exactly once: {css}"
    );

    let after_rule = css
        .find(r#"[data-scope="splitter"][data-part="resize-trigger"]::after {"#)
        .expect("resize-trigger ::after rule must exist");
    let after_block_end = css[after_rule..]
        .find('}')
        .expect("resize-trigger ::after rule must be closed");
    let after_block = &css[after_rule..after_rule + after_block_end];
    assert!(
        after_block.contains("content: \"\";"),
        "::after must have an auto-injected empty content: {after_block}"
    );
    assert!(
        after_block.contains("position: absolute;"),
        "::after must be absolutely positioned relative to resize-trigger: {after_block}"
    );
    assert!(
        after_block.contains("inset: var(--fandhe-splitter-hit-inset);"),
        "::after must expand via the inherited hit-inset custom property: {after_block}"
    );

    let resize_trigger_base_rule = css
        .find(r#"[data-scope="splitter"][data-part="resize-trigger"] {"#)
        .expect("resize-trigger base rule must exist");
    let resize_trigger_block_end = css[resize_trigger_base_rule..]
        .find('}')
        .expect("resize-trigger base rule must be closed");
    let resize_trigger_block =
        &css[resize_trigger_base_rule..resize_trigger_base_rule + resize_trigger_block_end];
    assert!(
        resize_trigger_block.contains("position: relative;"),
        "resize-trigger base rule must be positioned so ::after anchors to it: {resize_trigger_block}"
    );
    assert!(
        resize_trigger_block.contains(
            "--fandhe-splitter-hit-inset: 0 calc(-1 * var(--fandhe-splitter-hit-extension, 0.25rem));"
        ),
        "resize-trigger base rule must define the horizontal (default) hit-inset: {resize_trigger_block}"
    );

    let vertical_rule = css
        .find(
            r#"[data-scope="splitter"][data-part="resize-trigger"][data-orientation="vertical"] {"#,
        )
        .expect("resize-trigger vertical state rule must exist");
    let vertical_block_end = css[vertical_rule..]
        .find('}')
        .expect("resize-trigger vertical state rule must be closed");
    let vertical_block = &css[vertical_rule..vertical_rule + vertical_block_end];
    assert!(
        vertical_block.contains(
            "--fandhe-splitter-hit-inset: calc(-1 * var(--fandhe-splitter-hit-extension, 0.25rem)) 0;"
        ),
        "resize-trigger vertical state rule must override hit-inset for the vertical axis: {vertical_block}"
    );
}
