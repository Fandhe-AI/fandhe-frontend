//! イシュー #2385「scroll-driven reveal ユーティリティを実装する」の契約テスト。
//!
//! `SlotRecipe::scroll_reveal`（`motion` feature 配下、既定 off）を対象に
//! golden CSS 一致・プログレッシブエンハンスメント契約（`@supports` 外に
//! `opacity: 0` を出さない）・reduced-motion ブロックの出力順・未宣言 slot
//! での fail-closed スキップ・他ブロックとの出力順を固定する。`motion`
//! feature 無効時は本ファイルごとコンパイルされない。

#![cfg(feature = "motion")]

use fandhe_frontend_pre_styled_ui::decl;
use fandhe_frontend_pre_styled_ui::recipe::{
    Breakpoint, MotionDuration, SlotRecipe, StateCondition,
};

/// §3.2 の golden 全文バイト一致。
#[test]
fn scroll_reveal_golden_css() {
    let recipe = SlotRecipe::new("card", &["root"]).scroll_reveal("root");
    assert_eq!(
        recipe.css(),
        "@supports (animation-timeline: view()) {\n  @keyframes fandhe-motion-scroll-reveal {\n    from {\n      opacity: 0;\n      translate: 0 var(--fandhe-motion-scroll-reveal-distance, 1rem);\n    }\n    to {\n      opacity: 1;\n      translate: none;\n    }\n  }\n\n  [data-scope=\"card\"][data-part=\"root\"] {\n    animation-name: fandhe-motion-scroll-reveal;\n    animation-timing-function: linear;\n    animation-fill-mode: both;\n    animation-timeline: view();\n    animation-range: entry 0% entry 100%;\n  }\n}\n\n@media (prefers-reduced-motion: reduce) {\n  [data-scope=\"card\"][data-part=\"root\"] {\n    animation: none;\n  }\n}\n"
    );
}

/// 非対応ブラウザで要素が常に可視であること（受入基準 2）: `opacity: 0`
/// の出現は 1 回のみで `@supports (animation-timeline: view())` ブロックの
/// 内側にしかなく、出力全体が `@supports` から始まる（`@supports` 外に
/// base 規則を一切出さない）。`<`（`</style>` 脱出）も含まない。
#[test]
fn scroll_reveal_keeps_element_visible_without_supports() {
    let css = SlotRecipe::new("card", &["root"])
        .scroll_reveal("root")
        .css();
    assert_eq!(css.matches("opacity: 0").count(), 1);
    let supports_start = css
        .find("@supports (animation-timeline: view()) {")
        .unwrap();
    let reduced_start = css
        .find("@media (prefers-reduced-motion: reduce) {")
        .unwrap();
    let opacity_pos = css.find("opacity: 0").unwrap();
    assert!(supports_start < opacity_pos);
    assert!(opacity_pos < reduced_start);
    assert!(css.starts_with("@supports"));
    assert!(!css.contains('<'));
}

/// reduced-motion ブロックは `@supports` ブロックより後に出力され、
/// 同一セレクタへ `animation: none` を持つ（受入基準 3）。
#[test]
fn scroll_reveal_reduced_motion_block_follows_supports_block() {
    let css = SlotRecipe::new("card", &["root"])
        .scroll_reveal("root")
        .css();
    let supports_pos = css.find("@supports (animation-timeline: view())").unwrap();
    let reduced_pos = css.find("@media (prefers-reduced-motion: reduce)").unwrap();
    assert!(supports_pos < reduced_pos);
    let reduced_block = &css[reduced_pos..];
    assert!(reduced_block.contains("animation: none;"));
}

/// 未宣言 slot への `scroll_reveal` は panic せず出力から除外される
/// （`@keyframes`/`@supports`/reduced-motion のいずれも出力しない）。
#[test]
fn scroll_reveal_undeclared_slot_is_skipped_not_panicking() {
    let recipe = SlotRecipe::new("card", &["root"]).scroll_reveal("missing");
    assert_eq!(recipe.css(), "");
}

/// `@supports not (height: calc-size(...))` の後・`@media
/// (prefers-reduced-motion: reduce)` の後・breakpoints の前・
/// `@media (hover: hover)` の前という出力順を固定する。
#[test]
fn scroll_reveal_blocks_are_ordered_between_supports_not_and_breakpoints() {
    let css = SlotRecipe::new("card", &["root"])
        .base("root", vec![decl("display", "flex")])
        .content_height_transition("root", MotionDuration::Normal)
        .scroll_reveal("root")
        .breakpoint(
            "root",
            Breakpoint::Sm,
            vec![decl("padding", "var(--fandhe-space-4)")],
        )
        .state("root", StateCondition::Hover, vec![decl("opacity", "0.9")])
        .css();

    let supports_not_pos = css
        .find("@supports not (height: calc-size(auto, size))")
        .unwrap();
    let supports_view_pos = css.find("@supports (animation-timeline: view())").unwrap();
    let reduced_motion_pos = css.find("@media (prefers-reduced-motion: reduce)").unwrap();
    let breakpoint_pos = css.find("@media (min-width:").unwrap();
    let hover_pos = css.find("@media (hover: hover)").unwrap();

    assert!(supports_not_pos < supports_view_pos);
    assert!(supports_view_pos < reduced_motion_pos);
    assert!(reduced_motion_pos < breakpoint_pos);
    assert!(breakpoint_pos < hover_pos);
}
