//! styled Attachment（イシュー #2112、親 #2110、headless 側 anatomy は
//! #2111）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/bubble_css.rs` と同型の golden fixture
//! テスト。`attachment` recipe は `root`/`media`/`content`/`name`/`meta`/
//! `progress`/`actions`/`action` の 8 slot を宣言し、variant/state/
//! disabled は headless の `data-*` を `AttrEq`/`Attr` で参照するのみで
//! class ベースの軸を持たない（`src/attachment.rs` モジュール doc参照）。
//! `data-variant="image"` の `media`/`img`/`actions` 配置、
//! `data-state="error"` 時の `meta` 文字色、`actions` の hover 表示
//! （タッチ端末対策込み）は `root`/`media`/`meta`/`actions` にまたがる
//! 組み合わせセレクタのため `SlotRecipe::state`（単一 slot 前提）では
//! 表現できず raw CSS として追記する（`message`/`item` と同型の意図的
//! 差分、モジュール doc「`actions` の hover 表示とタッチ端末対策」節参照）。

use fandhe_frontend_pre_styled_ui::attachment;

const ATTACHMENT_GOLDEN_CSS: &str = "[data-scope=\"attachment\"][data-part=\"root\"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  padding: var(--fandhe-space-2) var(--fandhe-space-3);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-lg);
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
}

[data-scope=\"attachment\"][data-part=\"media\"] {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: var(--fandhe-space-10);
  height: var(--fandhe-space-10);
  border-radius: var(--fandhe-radius-md);
  background: var(--fandhe-color-bg-muted);
  overflow: hidden;
}

[data-scope=\"attachment\"][data-part=\"content\"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-0-5);
  flex: 1 1 auto;
  min-width: 0;
}

[data-scope=\"attachment\"][data-part=\"name\"] {
  font-weight: var(--fandhe-font-font-weight-medium);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--fandhe-color-fg);
}

[data-scope=\"attachment\"][data-part=\"meta\"] {
  font-size: var(--fandhe-font-font-size-xs);
  color: var(--fandhe-color-fg-muted);
}

[data-scope=\"attachment\"][data-part=\"progress\"] {
  width: 100%;
}

[data-scope=\"attachment\"][data-part=\"actions\"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-1);
  margin-inline-start: auto;
}

[data-scope=\"attachment\"][data-part=\"actions\"] {
  transition-property: opacity;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope=\"attachment\"][data-part=\"action\"] {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  width: var(--fandhe-space-6);
  height: var(--fandhe-space-6);
  padding: 0;
  border: none;
  border-radius: var(--fandhe-radius-sm);
  background: transparent;
  color: inherit;
  cursor: pointer;
  line-height: 1;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope=\"attachment\"][data-part=\"action\"] {
  transition-property: background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope=\"attachment\"][data-part=\"root\"][data-variant=\"image\"] {
  flex-direction: column;
  align-items: stretch;
  position: relative;
  width: var(--fandhe-attachment-image-width, 12rem);
}

[data-scope=\"attachment\"][data-part=\"root\"][data-state=\"error\"] {
  border-color: var(--fandhe-color-danger);
}

[data-scope=\"attachment\"][data-part=\"root\"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope=\"attachment\"][data-part=\"action\"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope=\"attachment\"][data-part=\"action\"][data-disabled] {
  cursor: not-allowed;
}

@media (hover: hover) {
  [data-scope=\"attachment\"][data-part=\"action\"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}

[data-scope=\"attachment\"][data-part=\"root\"][data-variant=\"image\"] > [data-scope=\"attachment\"][data-part=\"media\"] {
  aspect-ratio: 1 / 1;
  width: 100%;
  height: auto;
}

[data-scope=\"attachment\"][data-part=\"root\"][data-variant=\"image\"] > [data-scope=\"attachment\"][data-part=\"media\"] img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

[data-scope=\"attachment\"][data-part=\"root\"][data-variant=\"image\"] > [data-scope=\"attachment\"][data-part=\"actions\"] {
  position: absolute;
  top: var(--fandhe-space-2);
  right: var(--fandhe-space-2);
  margin-inline-start: 0;
}

[data-scope=\"attachment\"][data-part=\"root\"][data-state=\"error\"] [data-scope=\"attachment\"][data-part=\"meta\"] {
  color: var(--fandhe-color-danger);
}

@media (hover: hover) {
  [data-scope=\"attachment\"][data-part=\"root\"][data-variant=\"image\"] > [data-scope=\"attachment\"][data-part=\"actions\"] {
    opacity: 0;
  }

  [data-scope=\"attachment\"][data-part=\"root\"][data-variant=\"image\"]:hover > [data-scope=\"attachment\"][data-part=\"actions\"], [data-scope=\"attachment\"][data-part=\"root\"][data-variant=\"image\"]:focus-within > [data-scope=\"attachment\"][data-part=\"actions\"] {
    opacity: 1;
  }
}
";

/// `stylesheet()` の全文バイト一致を固定する（golden）。意図しない宣言の
/// 混入・欠落を検知する（`docs/internal/pre-styled-ui-golden-test-update-guide.md`
/// の更新手順に従う）。
#[test]
fn css_matches_golden_snapshot() {
    assert_eq!(attachment::stylesheet(), ATTACHMENT_GOLDEN_CSS);
}

/// 出力は毎回同一（非決定性のハッシュ順走査等が混入していないこと）。
#[test]
fn css_is_deterministic() {
    assert_eq!(attachment::stylesheet(), attachment::stylesheet());
}

/// `<style>` タグからの脱出シーケンス（`</style`）や生の `<` を一切
/// 含まないことを固定する（REQ-1 と独立した CSS 埋め込み文脈の不変条件）。
#[test]
fn css_never_contains_style_breakout_sequences() {
    let css = attachment::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

/// `data-variant`（`image`）・`data-state`（`error`）・`data-disabled` の
/// state 規則が漏れなく存在すること、class ベースのセレクタ
/// （`fd-attachment--` プレフィックス）を生成しないことを固定する。
#[test]
fn css_declares_variant_state_and_disabled_rules_without_class_selectors() {
    let css = attachment::stylesheet();
    assert!(
        css.contains("[data-scope=\"attachment\"][data-part=\"root\"][data-variant=\"image\"] {")
    );
    assert!(css.contains("[data-scope=\"attachment\"][data-part=\"root\"][data-state=\"error\"] {"));
    assert!(css.contains("[data-scope=\"attachment\"][data-part=\"root\"][data-disabled] {"));
    assert!(!css.contains("fd-attachment--"));
    assert!(!css.contains("class="));
}

/// `image` variant の `media`/`img`/`actions` 配置規則を固定する。
#[test]
fn css_positions_image_variant_media_and_actions() {
    let css = attachment::stylesheet();
    assert!(css.contains("aspect-ratio: 1 / 1;"));
    // `aspect-ratio` は `width`/`height` の両方が明示指定されていると
    // 無視される（CSS 仕様）。`media` の base 宣言が固定 `height:
    // var(--fandhe-space-10)` を持つため、image variant 側で
    // `height: auto` に戻さないと正方形サムネイルにならない回帰
    // （advisor 指摘・golden 側では検知できない意味論のため個別固定）。
    assert!(css.contains("height: auto;"));
    assert!(css.contains("object-fit: cover;"));
    assert!(css.contains("position: absolute;"));
}

/// `data-state="error"` 時の `meta` 文字色（root とは別 slot の raw CSS）
/// を固定する。
#[test]
fn css_colors_meta_on_error_state() {
    let css = attachment::stylesheet();
    assert!(css.contains(
        "[data-scope=\"attachment\"][data-part=\"root\"][data-state=\"error\"] [data-scope=\"attachment\"][data-part=\"meta\"] {\n  color: var(--fandhe-color-danger);"
    ));
}

/// `actions` の hover 表示（タッチ端末対策、モジュール doc「`actions` の
/// hover 表示とタッチ端末対策」節参照）: `opacity: 0` は `@media (hover:
/// hover)` 配下にのみ現れ、hover 機構を持たない端末では CSS 初期値
/// `opacity: 1` のまま残ることを固定する。
#[test]
fn css_hides_image_actions_only_inside_hover_media_and_reveals_on_focus_within() {
    let css = attachment::stylesheet();
    let media_start = css.find("@media (hover: hover) {\n  [data-scope=\"attachment\"][data-part=\"root\"][data-variant=\"image\"]").expect("image actions hover media block");
    let before_media = &css[..media_start];
    assert!(!before_media.contains("opacity: 0;"));
    assert!(css[media_start..].contains("opacity: 0;"));
    assert!(css[media_start..].contains(":focus-within"));
    assert!(css[media_start..].contains("opacity: 1;"));
}

/// `action` の hover は `@media (hover: hover)` へ集約出力される
/// （イシュー #1425 の共通ビジュアル言語規約）。
#[test]
fn css_collects_action_hover_under_media_hover() {
    let css = attachment::stylesheet();
    assert!(css.contains("@media (hover: hover) {"));
    assert!(css.contains(
        "[data-scope=\"attachment\"][data-part=\"action\"]:hover:not([data-disabled]) {"
    ));
}
