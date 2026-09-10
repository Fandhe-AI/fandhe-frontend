//! styled Message Scroller（イシュー #2123、親 #2120。headless 側 anatomy は
//! #2121）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/message_css.rs` と同型の golden fixture
//! テスト。`message-scroller` recipe は `root`/`viewport`/`content`/
//! `anchor`/`jump-to-latest`/`load-more` の 6 slot を宣言し、
//! `data-stuck`/`data-has-new`/`data-visible`/`data-loading`/
//! `data-disabled` は headless の `data-*` を `AttrEq`/`Attr` で参照する
//! のみで class ベースの軸を持たない（`src/message_scroller.rs` モジュール
//! doc「`data-stuck`/`data-has-new`/`data-visible`/`data-loading`/
//! `data-disabled` の表現」節参照）。`root[data-has-new]`/
//! `root[data-stuck="bottom"]` の raw CSS 追記（子結合子）も golden に
//! 含む。

use fandhe_frontend_pre_styled_ui::message_scroller;

const MESSAGE_SCROLLER_GOLDEN_CSS: &str = "[data-scope=\"message-scroller\"][data-part=\"root\"] {
  position: relative;
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: var(--fandhe-message-scroller-height, 24rem);
  overflow: hidden;
  border: 1px solid var(--fandhe-color-border);
  border-radius: var(--fandhe-radius-lg);
}

[data-scope=\"message-scroller\"][data-part=\"viewport\"] {
  flex: 1 1 auto;
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior: contain;
  scroll-behavior: smooth;
  scrollbar-width: thin;
  scrollbar-color: var(--fandhe-message-scroller-thumb-bg, var(--fandhe-color-fg-subtle, var(--fandhe-color-border-emphasized, var(--fandhe-color-border)))) transparent;
  mask-image: linear-gradient(to bottom, transparent 0, #000 var(--fandhe-message-scroller-fade-start, 1.5rem), #000 calc(100% - var(--fandhe-message-scroller-fade-end, 1.5rem)), transparent 100%);
  mask-repeat: no-repeat;
}

[data-scope=\"message-scroller\"][data-part=\"content\"] {
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-3);
  padding: var(--fandhe-space-4);
}

[data-scope=\"message-scroller\"][data-part=\"anchor\"] {
  flex: none;
  height: 1px;
  width: 100%;
}

[data-scope=\"message-scroller\"][data-part=\"jump-to-latest\"] {
  position: absolute;
  inset-inline: 0;
  bottom: var(--fandhe-space-3);
  margin-inline: auto;
  width: fit-content;
  display: inline-flex;
  align-items: center;
  gap: var(--fandhe-space-2);
  padding: var(--fandhe-space-2) var(--fandhe-space-3);
  border-radius: var(--fandhe-radius-full);
  background: var(--fandhe-color-bg);
  border: 1px solid var(--fandhe-color-border);
  box-shadow: var(--fandhe-shadow-md);
  font-size: var(--fandhe-font-font-size-sm);
  color: var(--fandhe-color-fg);
  cursor: pointer;
  z-index: 1;
  --fandhe-hover-bg: var(--fandhe-color-bg-muted);
}

[data-scope=\"message-scroller\"][data-part=\"load-more\"] {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--fandhe-space-2);
  width: fit-content;
  margin: var(--fandhe-message-scroller-fade-start, 1.5rem) auto var(--fandhe-space-2);
  padding: var(--fandhe-space-1) var(--fandhe-space-3);
  border-radius: var(--fandhe-radius-md);
  background: transparent;
  border: 0;
  color: var(--fandhe-color-fg-muted);
  font-size: var(--fandhe-font-font-size-xs);
  cursor: pointer;
}

[data-scope=\"message-scroller\"][data-part=\"viewport\"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: calc(-1 * var(--fandhe-focus-ring-offset, 2px));
}

[data-scope=\"message-scroller\"][data-part=\"jump-to-latest\"][hidden] {
  display: none;
}

[data-scope=\"message-scroller\"][data-part=\"jump-to-latest\"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope=\"message-scroller\"][data-part=\"jump-to-latest\"][data-visible] {
  transition-property: background, border-color, box-shadow;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope=\"message-scroller\"][data-part=\"load-more\"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

[data-scope=\"message-scroller\"][data-part=\"load-more\"][data-disabled] {
  opacity: 0.5;
  cursor: not-allowed;
}

[data-scope=\"message-scroller\"][data-part=\"load-more\"][data-loading] {
  cursor: progress;
}

@media (hover: hover) {
  [data-scope=\"message-scroller\"][data-part=\"jump-to-latest\"]:hover:not([data-disabled]) {
    background: var(--fandhe-hover-bg);
  }
}

[data-scope=\"message-scroller\"][data-part=\"root\"][data-has-new] > [data-scope=\"message-scroller\"][data-part=\"jump-to-latest\"] {
  background: var(--fandhe-color-accent);
  color: var(--fandhe-color-accent-fg);
  border-color: transparent;
}

[data-scope=\"message-scroller\"][data-part=\"root\"][data-stuck=\"bottom\"] > [data-scope=\"message-scroller\"][data-part=\"viewport\"] {
  --fandhe-message-scroller-fade-end: 0px;
}

@media (prefers-reduced-motion: reduce) {
  [data-scope=\"message-scroller\"][data-part=\"viewport\"] {
    scroll-behavior: auto;
  }
}
";

#[test]
fn stylesheet_matches_golden_snapshot() {
    let css = message_scroller::stylesheet();
    assert_eq!(
        css, MESSAGE_SCROLLER_GOLDEN_CSS,
        "message_scroller::stylesheet() の出力が golden と一致しません。\n         意図した変更であれば docs/internal/pre-styled-ui-golden-test-update-guide.md\n         の手順に従い golden を更新してください。\n実際の出力:\n{css}"
    );
}

/// `jump-to-latest` の `hidden` 属性が `display: none` で確実に上書きされる
/// ことを固定する（`src/message_scroller.rs` モジュール doc「`hidden`
/// 属性の上書き」節参照。base の `display: inline-flex` が UA の
/// `[hidden] { display: none }` を詳細度で上書きしてしまう問題の回帰
/// テスト）。
#[test]
fn css_overrides_hidden_attribute_with_display_none() {
    let css = message_scroller::stylesheet();
    assert!(css.contains(
        "[data-scope=\"message-scroller\"][data-part=\"jump-to-latest\"][hidden] {\n  display: none;\n}\n"
    ));
}

/// `root[data-has-new] > jump-to-latest` の子結合子規則を固定する
/// （`src/message_scroller.rs` モジュール doc「raw CSS 追記の理由」節
/// 参照。`SlotRecipe` は別 slot の宣言を切り替える規則を組めないため
/// raw CSS で補う）。
#[test]
fn css_highlights_jump_to_latest_when_root_has_new() {
    let css = message_scroller::stylesheet();
    assert!(css.contains(
        "[data-scope=\"message-scroller\"][data-part=\"root\"][data-has-new] > [data-scope=\"message-scroller\"][data-part=\"jump-to-latest\"] {\n  background: var(--fandhe-color-accent);\n  color: var(--fandhe-color-accent-fg);\n  border-color: transparent;\n}\n"
    ));
}

/// `root[data-stuck="bottom"] > viewport` の子結合子規則を固定する
/// （`src/message_scroller.rs` モジュール doc「端フェードの採否」節参照。
/// 最下部に張り付いているときは末尾フェードを解除し、最新メッセージを
/// 霞ませない）。
#[test]
fn css_clears_trailing_fade_when_stuck_at_bottom() {
    let css = message_scroller::stylesheet();
    assert!(css.contains(
        "[data-scope=\"message-scroller\"][data-part=\"root\"][data-stuck=\"bottom\"] > [data-scope=\"message-scroller\"][data-part=\"viewport\"] {\n  --fandhe-message-scroller-fade-end: 0px;\n}\n"
    ));
}

/// `stylesheet()` が決定的（同一入力で常に同一出力）であり、`</style`
/// 脱出シーケンス・`<` を含まないことを固定する（`src/message_scroller.rs`
/// モジュール doc「セキュリティ不変条件」節参照）。
#[test]
fn stylesheet_is_deterministic_and_has_no_style_breakout_sequences() {
    let a = message_scroller::stylesheet();
    let b = message_scroller::stylesheet();
    assert_eq!(a, b);
    assert!(!a.contains("</style"));
    assert!(!a.contains('<'));
}
