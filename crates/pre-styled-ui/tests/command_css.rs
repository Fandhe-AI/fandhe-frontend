//! styled Command（イシュー #2070、親 #2067）の決定的 CSS 出力ゴールデン
//! テスト。
//!
//! `crates/pre-styled-ui/tests/item_css.rs` と同型の golden fixture
//! テスト。`command` recipe は `root`/`input`/`list`/`empty`/`group`/
//! `group-heading`/`item`/`shortcut`/`separator`/`dialog` の 10 slot を
//! 宣言し、`size`/`variant`/`color-palette` いずれの軸も持たない（`src/command.rs`
//! モジュール doc「軸を持たない理由」節参照）。

use fandhe_frontend_pre_styled_ui::command;

const COMMAND_GOLDEN_CSS: &str = "[data-scope=\"command\"][data-part=\"root\"] {
  display: flex;
  flex-direction: column;
  width: 100%;
  min-width: 0;
  box-sizing: border-box;
  overflow: hidden;
  background: var(--fandhe-color-bg);
  color: var(--fandhe-color-fg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-md);
}

[data-scope=\"command\"][data-part=\"input\"] {
  display: block;
  width: 100%;
  box-sizing: border-box;
  padding: var(--fandhe-space-3);
  border: 0;
  border-bottom: 1px solid var(--fandhe-color-border);
  background: transparent;
  color: inherit;
  font: inherit;
  font-size: var(--fandhe-font-font-size-sm);
  outline: none;
}

[data-scope=\"command\"][data-part=\"list\"] {
  max-height: var(--fandhe-command-list-max-height, 18.75rem);
  overflow-y: auto;
  overflow-x: hidden;
  padding: var(--fandhe-space-1);
}

[data-scope=\"command\"][data-part=\"empty\"] {
  display: none;
  padding: var(--fandhe-space-6) 0;
  text-align: center;
  font-size: var(--fandhe-font-font-size-sm);
  color: var(--fandhe-color-fg-muted);
}

[data-scope=\"command\"][data-part=\"group\"] {
  padding: var(--fandhe-space-1);
  color: var(--fandhe-color-fg);
}

[data-scope=\"command\"][data-part=\"group-heading\"] {
  padding: var(--fandhe-space-1) var(--fandhe-space-2);
  font-size: var(--fandhe-font-font-size-xs);
  font-weight: var(--fandhe-font-font-weight-medium);
  color: var(--fandhe-color-fg-muted);
}

[data-scope=\"command\"][data-part=\"item\"] {
  display: flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  padding: var(--fandhe-space-2);
  border-radius: var(--fandhe-radius-sm);
  font-size: var(--fandhe-font-font-size-sm);
  cursor: default;
  user-select: none;
  outline: none;
  transition: background var(--fandhe-motion-duration-fast, 150ms) ease, color var(--fandhe-motion-duration-fast, 150ms) ease;
}

[data-scope=\"command\"][data-part=\"shortcut\"] {
  margin-inline-start: auto;
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-1);
  font-size: var(--fandhe-font-font-size-xs);
  letter-spacing: 0.1em;
  color: var(--fandhe-color-fg-muted);
}

[data-scope=\"command\"][data-part=\"separator\"] {
  height: 1px;
  margin: var(--fandhe-space-1) 0;
  background: var(--fandhe-color-border);
}

[data-scope=\"command\"][data-part=\"dialog\"] {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 100%;
  max-width: var(--fandhe-command-dialog-max-width, 32rem);
  max-height: calc(100vh - var(--fandhe-space-8));
  box-sizing: border-box;
  padding: 0;
  overflow-y: auto;
  overflow-x: hidden;
  z-index: var(--fandhe-z-index-modal, 1001);
  background: var(--fandhe-color-bg);
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-lg);
  box-shadow: var(--fandhe-shadow-lg);
  outline: none;
}

[data-scope=\"command\"][data-part=\"root\"]:focus-within {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope=\"command\"][data-part=\"dialog\"]:focus-within {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope=\"command\"][data-part=\"empty\"][data-empty] {
  display: block;
}

[data-scope=\"command\"][data-part=\"item\"][data-selected] {
  background: var(--fandhe-color-bg-muted);
  color: var(--fandhe-color-fg);
}

[data-scope=\"command\"][data-part=\"item\"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope=\"command\"][data-part=\"dialog\"][hidden] {
  display: none;
}

@media (hover: hover) {
  [data-scope=\"command\"][data-part=\"item\"]:hover:not([data-disabled]):not([data-selected]) {
    background: var(--fandhe-color-bg-subtle);
  }
}

[data-scope=\"command\"][data-part=\"dialog\"] > [data-scope=\"command\"][data-part=\"root\"] {
  border: 0;
  border-radius: 0;
  box-shadow: none;
}
";

#[test]
fn command_css_matches_golden_fixture() {
    assert_eq!(command::stylesheet(), COMMAND_GOLDEN_CSS);
}

#[test]
fn command_css_is_deterministic() {
    assert_eq!(command::stylesheet(), command::stylesheet());
}

/// `empty` の表示切替（`display: none` 既定 → `[data-empty]` で
/// `display: block`）が両方存在することを固定する（`src/command.rs`
/// モジュール doc「`empty` の表示切替 CSS」節参照）。
#[test]
fn css_empty_visibility_toggle_exists() {
    let css = command::stylesheet();
    assert!(css.contains("[data-scope=\"command\"][data-part=\"empty\"] {\n  display: none;"));
    assert!(css.contains(
        "[data-scope=\"command\"][data-part=\"empty\"][data-empty] {\n  display: block;\n}"
    ));
}

/// 選択行の背景（`[data-selected]`）と、それを hover が洗い流さない
/// ための除外規則（`HoverExceptAttr`）の両方が存在することを固定する
/// （`src/command.rs` モジュール doc「`item` の選択表現」節参照）。
#[test]
fn css_selected_item_background_and_hover_exclusion_exist() {
    let css = command::stylesheet();
    assert!(css.contains("[data-scope=\"command\"][data-part=\"item\"][data-selected] {"));
    assert!(css.contains(
        "[data-scope=\"command\"][data-part=\"item\"]:hover:not([data-disabled]):not([data-selected]) {"
    ));
}

/// closed 時の `dialog` を確実に非表示化する `[hidden]` 規則が存在する
/// ことを固定する（`src/command.rs` モジュール doc「`dialog` の
/// `[hidden]`」節参照）。
#[test]
fn css_dialog_hidden_forces_display_none() {
    let css = command::stylesheet();
    assert!(css
        .contains("[data-scope=\"command\"][data-part=\"dialog\"][hidden] {\n  display: none;\n}"));
}

/// dialog 内 root の二重枠を解除する raw CSS 追記（子結合子セレクタ）が
/// 存在することを固定する（`src/command.rs` モジュール doc「raw CSS
/// 追記の理由」節参照）。
#[test]
fn css_appends_dialog_root_double_border_removal_rule() {
    let css = command::stylesheet();
    assert!(css.contains(
        "[data-scope=\"command\"][data-part=\"dialog\"] > [data-scope=\"command\"][data-part=\"root\"] {\n  border: 0;\n  border-radius: 0;\n  box-shadow: none;\n}"
    ));
}

/// `input` の `outline: none` を補う `root` の `:focus-within` canonical
/// フォーカスリングが存在することを固定する（`src/command.rs` モジュール
/// doc「フォーカスリング」節参照。PR #2241 codex/Bugbot レビュー対応）。
#[test]
fn css_root_focus_within_ring_exists() {
    let css = command::stylesheet();
    assert!(css.contains("[data-scope=\"command\"][data-part=\"root\"]:focus-within {"));
    assert!(css.contains(
        "outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));"
    ));
}

/// `dialog` 自身にも `:focus-within` canonical フォーカスリングが存在する
/// ことを固定する。`dialog` → `root` → `input` と入れ子にした公開
/// Examples の構成では `root` の `Outside` リングが祖先 `dialog` の
/// `overflow-x: hidden`/`overflow-y: auto` によってクリップされ不可視に
/// なるため、`dialog` 自身の `:focus-within` が可視化手段として必要
/// （`src/command.rs` モジュール doc「`dialog` 内では `root` のリングだけ
/// では不十分」節参照。PR #2241 codex レビュー P1 対応）。
#[test]
fn css_dialog_focus_within_ring_exists() {
    let css = command::stylesheet();
    assert!(css.contains("[data-scope=\"command\"][data-part=\"dialog\"]:focus-within {"));
    let (_, after) = css
        .split_once("[data-scope=\"command\"][data-part=\"dialog\"]:focus-within {")
        .expect("dialog focus-within rule must exist");
    assert!(after.starts_with(
        "\n  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));\n  outline-offset: var(--fandhe-focus-ring-offset, 2px);\n}"
    ));
}

/// `dialog` が `overflow-y: auto`（横方向のみ `hidden`）でスクロール
/// コンテナになることを固定する（画面高が低く候補が多い環境で `root` の
/// 末尾がクリップされたまま到達不能にならないための対策、`src/command.rs`
/// モジュール doc「`dialog` の `[hidden]`」節末尾参照。PR #2241 codex
/// レビュー対応）。
#[test]
fn css_dialog_is_vertically_scrollable() {
    let css = command::stylesheet();
    assert!(css.contains("[data-scope=\"command\"][data-part=\"dialog\"] {\n  position: fixed;"));
    assert!(css.contains("  overflow-y: auto;\n  overflow-x: hidden;\n  z-index:"));
    assert!(!css.contains("  overflow: hidden;\n  z-index: var(--fandhe-z-index-modal"));
}

/// `stylesheet()` は `</style` 断片・`<` を一切含まない（RAWTEXT 文脈から
/// の脱出経路を持たない、`crate::stylesheet::StyleSheet` 冒頭 doc の根拠と
/// 同じ）ことを固定する。
#[test]
fn css_never_contains_style_breakout_sequences() {
    let css = command::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

/// 見た目クラス（`fd-command--` プレフィックス）を一切生成しないことを
/// 固定する（`src/command.rs` モジュール doc「軸を持たない理由」節参照）。
#[test]
fn css_never_generates_class_based_variant_classes() {
    let css = command::stylesheet();
    assert!(!css.contains("fd-command--"));
    assert!(!css.contains("class="));
}
