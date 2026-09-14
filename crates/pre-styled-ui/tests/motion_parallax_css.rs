//! イシュー #2534「scroll-linked parallax / sticky ユーティリティを実装する」
//! のうち `SlotRecipe::parallax` の契約テスト。
//!
//! `motion_scroll_reveal_css.rs` を雛形に、golden CSS 全文一致・
//! プログレッシブエンハンスメント契約（`@supports (animation-timeline:
//! view())` の内側にのみネイティブ宣言が現れる）・フォールバックブロック
//! （`@supports not`）の存在・reduced-motion ブロックの出力順・未宣言 slot
//! での fail-closed スキップ・他ブロックとの出力順を固定する。`motion`
//! feature 無効時は本ファイルごとコンパイルされない。

#![cfg(feature = "motion")]

use fandhe_frontend_pre_styled_ui::decl;
use fandhe_frontend_pre_styled_ui::recipe::{
    Breakpoint, MotionDuration, ParallaxSpeed, SlotRecipe, StateCondition,
};

/// golden 全文バイト一致。
#[test]
fn parallax_golden_css() {
    let recipe = SlotRecipe::new("card", &["root"]).parallax("root", ParallaxSpeed::Normal);
    assert_eq!(
        recipe.css(),
        "@supports (animation-timeline: view()) {\n  @keyframes fandhe-motion-parallax {\n    from {\n      translate: 0 0;\n    }\n    to {\n      translate: 0 var(--fandhe-motion-parallax-distance, -4rem);\n    }\n  }\n\n  [data-scope=\"card\"][data-part=\"root\"] {\n    --fandhe-motion-parallax-distance: -4rem;\n    animation-name: fandhe-motion-parallax;\n    animation-timing-function: linear;\n    animation-fill-mode: both;\n    animation-timeline: view();\n    animation-range: cover 0% cover 100%;\n  }\n}\n\n@supports not (animation-timeline: view()) {\n  [data-scope=\"card\"][data-part=\"root\"] {\n    --fandhe-motion-parallax-distance: -4rem;\n    translate: 0 calc(var(--fandhe-motion-scroll-progress, 0) * var(--fandhe-motion-parallax-distance, -4rem));\n  }\n}\n\n@media (prefers-reduced-motion: reduce) {\n  [data-scope=\"card\"][data-part=\"root\"] {\n    animation: none;\n    translate: none;\n  }\n}\n"
    );
}

/// `ParallaxSpeed::Slow`/`Fast` が異なる距離リテラルを `--fandhe-motion-
/// parallax-distance` へ書き込むこと（速度トークンの意味論固定）。
#[test]
fn parallax_speed_tokens_map_to_distinct_distances() {
    let slow = SlotRecipe::new("card", &["root"])
        .parallax("root", ParallaxSpeed::Slow)
        .css();
    let fast = SlotRecipe::new("card", &["root"])
        .parallax("root", ParallaxSpeed::Fast)
        .css();
    assert!(slow.contains("--fandhe-motion-parallax-distance: -2rem;"));
    assert!(fast.contains("--fandhe-motion-parallax-distance: -8rem;"));
}

/// 非対応ブラウザ向けフォールバック（`@supports not (animation-timeline:
/// view())`）が独立したブロックとして存在し、`<`（`</style>` 脱出）を
/// 含まないこと（受入基準: 非対応ブラウザでも安全側の CSS のみを出力）。
#[test]
fn parallax_has_fallback_block_without_style_escape() {
    let css = SlotRecipe::new("card", &["root"])
        .parallax("root", ParallaxSpeed::Normal)
        .css();
    assert!(css.contains("@supports not (animation-timeline: view()) {"));
    assert!(!css.contains('<'));
}

/// ネイティブ・フォールバック・reduced-motion の 3 ブロックがこの順で
/// 出力される。
#[test]
fn parallax_block_order_is_native_then_fallback_then_reduced_motion() {
    let css = SlotRecipe::new("card", &["root"])
        .parallax("root", ParallaxSpeed::Normal)
        .css();
    let native_pos = css.find("@supports (animation-timeline: view())").unwrap();
    let fallback_pos = css
        .find("@supports not (animation-timeline: view())")
        .unwrap();
    let reduced_pos = css.find("@media (prefers-reduced-motion: reduce)").unwrap();
    assert!(native_pos < fallback_pos);
    assert!(fallback_pos < reduced_pos);
}

/// 未宣言 slot への `parallax` は panic せず出力から除外される。
#[test]
fn parallax_undeclared_slot_is_skipped_not_panicking() {
    let recipe = SlotRecipe::new("card", &["root"]).parallax("missing", ParallaxSpeed::Normal);
    assert_eq!(recipe.css(), "");
}

/// `@supports not (height: calc-size(...))` の後・`scroll_reveal` の後・
/// breakpoints の前という出力順を固定する（`write_scroll_reveal_blocks`
/// 直後に `write_parallax_blocks` を呼ぶ `SlotRecipe::css` 本体の配線）。
#[test]
fn parallax_blocks_are_ordered_after_scroll_reveal_and_before_breakpoints() {
    let css = SlotRecipe::new("card", &["root"])
        .base("root", vec![decl("display", "flex")])
        .content_height_transition("root", MotionDuration::Normal)
        .scroll_reveal("root")
        .parallax("root", ParallaxSpeed::Normal)
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
    let scroll_reveal_pos = css.find("fandhe-motion-scroll-reveal").unwrap();
    let parallax_pos = css.find("fandhe-motion-parallax").unwrap();
    let breakpoint_pos = css.find("@media (min-width:").unwrap();
    let hover_pos = css.find("@media (hover: hover)").unwrap();

    assert!(supports_not_pos < scroll_reveal_pos);
    assert!(scroll_reveal_pos < parallax_pos);
    assert!(parallax_pos < breakpoint_pos);
    assert!(breakpoint_pos < hover_pos);
}
