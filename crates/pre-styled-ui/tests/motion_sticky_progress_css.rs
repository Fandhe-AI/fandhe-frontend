//! イシュー #2534「scroll-linked parallax / sticky ユーティリティを実装する」
//! のうち `SlotRecipe::sticky_progress` の契約テスト。`motion_parallax_css.rs`
//! と同型の契約群（golden 一致・フォールバック存在・出力順・fail-closed
//! スキップ）を固定する。`motion` feature 無効時は本ファイルごと
//! コンパイルされない。

#![cfg(feature = "motion")]

use fandhe_frontend_pre_styled_ui::decl;
use fandhe_frontend_pre_styled_ui::recipe::SlotRecipe;

/// golden 全文バイト一致。
#[test]
fn sticky_progress_golden_css() {
    let recipe = SlotRecipe::new("card", &["root"]).sticky_progress("root");
    assert_eq!(
        recipe.css(),
        "@supports (animation-timeline: view()) {\n  @keyframes fandhe-motion-sticky-progress {\n    from {\n      opacity: 0.6;\n      scale: 0.96;\n    }\n    to {\n      opacity: 1;\n      scale: 1;\n    }\n  }\n\n  [data-scope=\"card\"][data-part=\"root\"] {\n    animation-name: fandhe-motion-sticky-progress;\n    animation-timing-function: linear;\n    animation-fill-mode: backwards;\n    animation-timeline: view();\n    animation-range: contain 0% contain 100%;\n  }\n}\n\n@supports not (animation-timeline: view()) {\n  [data-scope=\"card\"][data-part=\"root\"] {\n    opacity: calc(0.6 + (var(--fandhe-motion-scroll-progress, 0) * 0.4));\n    scale: calc(0.96 + (var(--fandhe-motion-scroll-progress, 0) * 0.04));\n  }\n}\n\n@media (prefers-reduced-motion: reduce) {\n  [data-scope=\"card\"][data-part=\"root\"] {\n    animation: none;\n    opacity: 1;\n    scale: none;\n  }\n}\n"
    );
}

/// 非対応ブラウザ向けフォールバックが独立ブロックとして存在し、`<`
/// （`</style>` 脱出）を含まないこと。
#[test]
fn sticky_progress_has_fallback_block_without_style_escape() {
    let css = SlotRecipe::new("card", &["root"])
        .sticky_progress("root")
        .css();
    assert!(css.contains("@supports not (animation-timeline: view()) {"));
    assert!(!css.contains('<'));
}

/// ネイティブ・フォールバック・reduced-motion の 3 ブロックがこの順で
/// 出力される。
#[test]
fn sticky_progress_block_order_is_native_then_fallback_then_reduced_motion() {
    let css = SlotRecipe::new("card", &["root"])
        .sticky_progress("root")
        .css();
    let native_pos = css.find("@supports (animation-timeline: view())").unwrap();
    let fallback_pos = css
        .find("@supports not (animation-timeline: view())")
        .unwrap();
    let reduced_pos = css.find("@media (prefers-reduced-motion: reduce)").unwrap();
    assert!(native_pos < fallback_pos);
    assert!(fallback_pos < reduced_pos);
}

/// reduced-motion ブロックがネイティブ・フォールバック双方の効果を
/// `opacity: 1;`/`scale: none;` の静的値で凍結すること（`scale` は `1`
/// ではなく `none` を使う。`1` は非 `none` 値のため包含ブロックを作り
/// 続けてしまうため、codex-review P1 是正・PR #2563）。
#[test]
fn sticky_progress_reduced_motion_resets_both_paths() {
    let css = SlotRecipe::new("card", &["root"])
        .sticky_progress("root")
        .css();
    let reduced_pos = css.find("@media (prefers-reduced-motion: reduce)").unwrap();
    let reduced_block = &css[reduced_pos..];
    assert!(reduced_block.contains("animation: none;"));
    assert!(reduced_block.contains("opacity: 1;"));
    assert!(reduced_block.contains("scale: none;"));
}

/// 未宣言 slot への `sticky_progress` は panic せず出力から除外される。
#[test]
fn sticky_progress_undeclared_slot_is_skipped_not_panicking() {
    let recipe = SlotRecipe::new("card", &["root"]).sticky_progress("missing");
    assert_eq!(recipe.css(), "");
}

/// `parallax` ブロックの後・breakpoints の前という出力順を固定する
/// （`SlotRecipe::css` 本体の `write_parallax_blocks` → `write_sticky_
/// progress_blocks` 呼び出し順）。
#[test]
fn sticky_progress_blocks_are_ordered_after_parallax_and_before_breakpoints() {
    use fandhe_frontend_pre_styled_ui::recipe::{Breakpoint, ParallaxSpeed};

    let css = SlotRecipe::new("card", &["root", "indicator"])
        .base("root", vec![decl("display", "flex")])
        .parallax("root", ParallaxSpeed::Normal)
        .sticky_progress("indicator")
        .breakpoint(
            "root",
            Breakpoint::Sm,
            vec![decl("padding", "var(--fandhe-space-4)")],
        )
        .css();

    let parallax_pos = css.find("fandhe-motion-parallax").unwrap();
    let sticky_pos = css.find("fandhe-motion-sticky-progress").unwrap();
    let breakpoint_pos = css.find("@media (min-width:").unwrap();

    assert!(parallax_pos < sticky_pos);
    assert!(sticky_pos < breakpoint_pos);
}

/// Cursor Bugbot 指摘是正（PR #2563、threadId `PRRT_kwDOTarxgc6iTQVF`）の
/// 回帰テスト: `animation-fill-mode` は `both`/`forwards` ではなく
/// `backwards` であること。`both`/`forwards` だと `contain` 区間終了後
/// も `opacity`/`scale` の終端値 (`1`) を animation 優先度で保持し続け、
/// 後続の hover/state/variant の `opacity`/`scale` ルールが常に負けて
/// しまう（`scroll_reveal` が `entry` 区間で同じ理由から既に `both` を
/// 不採用にしている）。加えて `scale: 1`（非 `none` 値）が
/// `position: fixed` な子孫のための包含ブロックを恒久的に維持し続ける
/// 問題も併発する。`backwards` のみを使うことで、区間終了後は通常の
/// カスケードへ戻り、両方の問題を同時に解消する。
#[test]
fn sticky_progress_uses_backwards_fill_mode_not_both() {
    let css = SlotRecipe::new("card", &["root"])
        .sticky_progress("root")
        .css();
    let native_pos = css.find("@supports (animation-timeline: view())").unwrap();
    let fallback_pos = css
        .find("@supports not (animation-timeline: view())")
        .unwrap();
    let native_block = &css[native_pos..fallback_pos];
    assert!(
        native_block.contains("animation-fill-mode: backwards;"),
        "sticky_progress のネイティブ経路は animation-fill-mode: backwards \
         を使うはず（both/forwards は contain 区間終了後も opacity/scale \
         の終端値を animation 優先度で保持し続け、hover/state/variant の \
         上書きを妨げてしまう回帰）: native_block={native_block}"
    );
    assert!(
        !native_block.contains("animation-fill-mode: both;"),
        "animation-fill-mode: both は使わないはず: native_block={native_block}"
    );
}
