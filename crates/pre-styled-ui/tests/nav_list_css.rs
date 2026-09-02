//! styled NavList（イシュー #756）の決定的 CSS 出力ゴールデンテスト。
//!
//! `crates/pre-styled-ui/tests/breadcrumb_css.rs` の golden fixture テストの
//! 前例に倣い、`stylesheet()` が返す CSS 全文をバイト単位で固定する。出力順
//! （base → states）が崩れた場合や意図しない宣言の追加・欠落があった場合に、
//! この golden テストが即座に検知する。
//!
//! イシュー #1529（参考サイト基準への調整）で以下を追加・変更した:
//! `link` の色を `--fandhe-color-fg` から `--fandhe-color-fg-muted` へ変更・
//! `link` の余白（`padding`）・角丸（`border-radius`）追加・
//! `list` の縦積み（`display: flex; flex-direction: column; gap`）追加・
//! `link` の hover（`@media (hover: hover)` 集約出力、`fg-muted` → `fg` +
//! `background: bg-muted`）・
//! `link` のキーボードフォーカスリング（`focus_ring_declarations(Token,
//! Outside)`）・`link` の色・背景 transition。意図的に追随しない差分
//! （size / variant 軸不採用・ダークモード個別規則なし・disabled 状態なし）
//! は `crates/pre-styled-ui/src/nav_list.rs` モジュール doc「参考サイト
//! 基準への調整（イシュー #1529）」節に記録する。
//!
//! PR #1805 の codex-review / Bugbot 指摘（「Hover washes out current
//! page」）を受け、`link` の hover を
//! `StateCondition::Hover`（`:hover:not([data-disabled])`,
//! specificity (0,4,0)）から
//! `StateCondition::HoverExcept("aria-current", "page")`
//! （`:hover:not([data-disabled]):not([aria-current="page"])`）へ変更した。
//! 素の `Hover` は現在ページを示す `[aria-current="page"]` 規則
//! （specificity (0,3,0)）より高い specificity を持つため、現在ページの
//! link にホバーすると accent 色が `fg` へ上書きされてしまっていた。

use fandhe_frontend_pre_styled_ui::nav_list;

const NAV_LIST_GOLDEN_CSS: &str = r#"[data-scope="nav-list"][data-part="heading"] {
  font-size: var(--fandhe-font-font-size-sm);
  font-weight: var(--fandhe-font-font-weight-medium);
  color: var(--fandhe-color-fg-muted);
  margin: 0;
}

[data-scope="nav-list"][data-part="list"] {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: var(--fandhe-space-1, 0.25rem);
}

[data-scope="nav-list"][data-part="link"] {
  display: block;
  color: var(--fandhe-color-fg-muted);
  text-decoration: none;
  padding: var(--fandhe-space-1, 0.25rem) var(--fandhe-space-2, 0.5rem);
  border-radius: var(--fandhe-radius-sm, 0.25rem);
}

[data-scope="nav-list"][data-part="link"] {
  transition-property: color, background;
  transition-duration: var(--fandhe-motion-duration-fast);
  transition-timing-function: var(--fandhe-motion-easing-standard);
}

[data-scope="nav-list"][data-part="link"][aria-current="page"] {
  color: var(--fandhe-color-accent, var(--fandhe-color-fg));
  font-weight: var(--fandhe-font-font-weight-medium);
}

[data-scope="nav-list"][data-part="link"]:focus-visible {
  outline: var(--fandhe-focus-ring-width, 2px) solid var(--fandhe-color-focus-ring, var(--fandhe-color-accent));
  outline-offset: var(--fandhe-focus-ring-offset, 2px);
}

@media (hover: hover) {
  [data-scope="nav-list"][data-part="link"]:hover:not([data-disabled]):not([aria-current="page"]) {
    color: var(--fandhe-color-fg);
    background: var(--fandhe-color-bg-muted);
  }
}
"#;

#[test]
fn nav_list_stylesheet_matches_golden_fixture() {
    assert_eq!(nav_list::stylesheet(), NAV_LIST_GOLDEN_CSS);
}

#[test]
fn stylesheet_is_byte_identical_across_calls() {
    assert_eq!(nav_list::stylesheet(), nav_list::stylesheet());
}

#[test]
fn stylesheet_never_contains_style_breakout_sequences() {
    let css = nav_list::stylesheet();
    assert!(!css.contains("</style"));
    assert!(!css.contains('<'));
}

#[test]
fn link_hover_excludes_current_page_so_accent_is_not_overridden() {
    // PR #1805 の codex-review(P1) / Bugbot(Medium) 指摘「Hover washes out
    // current page」の回帰防止。hover が `[aria-current="page"]` な要素を
    // 対象から除外することを固定する（`StateCondition::HoverExcept`
    // rustdoc・`crate::color_picker` の同型対応参照）。現在ページと hover
    // のセレクタが互いに排他的であるため、現在ページの link にホバーしても
    // 現在ページ側の accent 規則がそのまま適用される。
    let css = nav_list::stylesheet();
    assert!(css.contains(
        "[data-scope=\"nav-list\"][data-part=\"link\"]:hover:not([data-disabled]):not([aria-current=\"page\"]) {\n    color: var(--fandhe-color-fg);\n    background: var(--fandhe-color-bg-muted);\n  }"
    ));
    assert!(
        !css.contains("[data-scope=\"nav-list\"][data-part=\"link\"]:hover:not([data-disabled]) {")
    );
}
