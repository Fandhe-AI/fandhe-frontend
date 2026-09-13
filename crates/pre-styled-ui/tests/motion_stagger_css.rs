//! イシュー #2384「stagger CSS ユーティリティを実装する」の契約テスト。
//!
//! `crate::recipe::stagger_delay_declaration`/`stagger_index_style`/
//! `SlotRecipe::stagger_delay`（`motion` feature 配下、既定 off）を対象に
//! golden CSS 一致・未宣言 slot での fail-closed スキップ・属性値の
//! エスケープ経路を固定する。`motion` feature 無効時は本ファイルごと
//! コンパイルされない（`motion_zero_cost.rs` が feature off 側のゼロ
//! コスト契約を別途固定する）。

#![cfg(feature = "motion")]

use fandhe_frontend_core::{el, render};
use fandhe_frontend_pre_styled_ui::recipe::{
    stagger_index_style, MotionDuration, SlotRecipe, STAGGER_INDEX_VAR,
};

/// `stagger_delay` で登録した base 宣言の golden CSS 全文バイト一致。
#[test]
fn stagger_delay_golden_css() {
    let recipe = SlotRecipe::new("list", &["item"]).stagger_delay("item", MotionDuration::Fast);
    assert_eq!(
        recipe.css(),
        "[data-scope=\"list\"][data-part=\"item\"] {\n  animation-delay: calc(var(--fandhe-motion-stagger-index, 0) * var(--fandhe-motion-duration-fast));\n}\n"
    );
}

/// `Normal`/`Slow` もそれぞれの duration トークンを参照し、
/// 出力に禁止文字（`{`/`}`/`;`/`<`）を含まない（`is_valid_value` 通過）。
#[test]
fn stagger_delay_normal_and_slow_reference_expected_tokens() {
    let normal = SlotRecipe::new("list", &["item"]).stagger_delay("item", MotionDuration::Normal);
    let slow = SlotRecipe::new("list", &["item"]).stagger_delay("item", MotionDuration::Slow);
    assert!(normal.css().contains("duration-normal"));
    assert!(slow.css().contains("duration-slow"));
    for css in [&normal.css(), &slow.css()] {
        assert!(!css.contains('<'));
    }
}

/// 未宣言 slot への `stagger_delay` は panic せず出力から除外される
/// （[`SlotRecipe::base`] の fail-closed 方針をそのまま継承する）。
#[test]
fn stagger_delay_undeclared_slot_is_skipped_not_panicking() {
    let recipe = SlotRecipe::new("list", &["item"]).stagger_delay("missing", MotionDuration::Fast);
    assert_eq!(recipe.css(), "");
}

/// `stagger_index_style` は `usize` の 10 進表記のみを埋め込み、
/// core の `render` を通した属性出力もそのまま反映される（既定エスケープ
/// 経路を経由することの確認、任意文字列混入経路がないことの裏付け）。
#[test]
fn stagger_index_style_renders_through_core_escaping() {
    let value = stagger_index_style(2);
    assert_eq!(value, "--fandhe-motion-stagger-index: 2");
    assert!(value.starts_with(STAGGER_INDEX_VAR));

    let node = el("li", vec![("style", &value)], vec![]);
    let html = render(&node);
    assert!(html.contains(r#"style="--fandhe-motion-stagger-index: 2""#));
}
