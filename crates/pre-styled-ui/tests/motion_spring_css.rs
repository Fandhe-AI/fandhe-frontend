//! イシュー #2381「spring 近似 easing プリセット（opt-in）を実装する」の
//! 契約テスト。
//!
//! `crate::theme::Theme::push_spring_easing`（`motion` feature 配下、既定
//! off）を対象に、`fandhe-animation` の同一パラメータで再計算した数値との
//! パリティ・[`CssValue`] allowlist 通過・golden 純追加・重複拒否を固定
//! する。`motion` feature 無効時は本ファイルごとコンパイルされない
//! （`motion_zero_cost.rs` が feature off 側のゼロコスト契約を別途固定）。

#![cfg(feature = "motion")]

use fandhe_animation::easing::sample;
use fandhe_animation::spring::{Spring, SpringConfig};
use fandhe_frontend_pre_styled_ui::theme::{
    Theme, ThemeError, SPRING_DURATION_MS, SPRING_EASING_LINEAR, SPRING_SAMPLE_COUNT,
};

/// [`SPRING_EASING_LINEAR`]/[`SPRING_DURATION_MS`] は `theme.rs` 冒頭の
/// rustdoc が定める手順（既定 `SpringConfig`・`from=0.0`/`to=1.0`/
/// `initial_velocity=0.0`・`d = settle_duration()`・`SPRING_SAMPLE_COUNT`
/// 点サンプリング・各値 `{:.3}`）で再計算した値と数値一致する
/// （クロスプラットフォームの libm 最終桁差を許容するため、文字列完全
/// 一致ではなく許容誤差での数値比較）。値を変更する場合は本テストの
/// 失敗メッセージに出力される再計算文字列をそのまま定数へ転記する。
#[test]
fn spring_linear_preset_matches_fandhe_animation_resampling() {
    let spring = Spring::new(SpringConfig::default(), 0.0, 1.0, 0.0)
        .expect("既定 SpringConfig は from != to のため必ず Some を返す");
    let duration = spring.settle_duration();
    let recomputed: Vec<f64> = sample(|t| spring.at(t * duration).value, SPRING_SAMPLE_COUNT);
    let recomputed_str = recomputed
        .iter()
        .map(|v| format!("{v:.3}"))
        .collect::<Vec<_>>()
        .join(", ");
    let recomputed_ms = (duration * 1000.0).round() as i64;

    let inner = SPRING_EASING_LINEAR
        .strip_prefix("linear(")
        .and_then(|s| s.strip_suffix(')'))
        .expect("SPRING_EASING_LINEAR は `linear(...)` の形をしている");
    let embedded: Vec<f64> = inner
        .split(", ")
        .map(|s| {
            s.parse::<f64>()
                .expect("各サンプル値は f64 として parse できる")
        })
        .collect();

    assert_eq!(
        embedded.len(),
        SPRING_SAMPLE_COUNT,
        "サンプル点数が SPRING_SAMPLE_COUNT と一致しない"
    );
    assert_eq!(embedded.first(), Some(&0.0), "先頭は 0.000 のはず");
    assert_eq!(
        embedded.last(),
        Some(&1.000),
        "末尾は 1.000（収束済み）のはず"
    );

    for (i, (e, r)) in embedded.iter().zip(recomputed.iter()).enumerate() {
        assert!(
            (e - r).abs() <= 6e-4,
            "index {i}: 埋め込み値 {e} と再計算値 {r} の差が許容誤差を超えた。\n\
             再計算結果を SPRING_EASING_LINEAR へ転記すること:\nlinear({recomputed_str})"
        );
    }

    let embedded_ms: i64 = SPRING_DURATION_MS
        .strip_suffix("ms")
        .expect("SPRING_DURATION_MS は `<int>ms` の形をしている")
        .parse()
        .expect("duration は整数 ms として parse できる");
    assert!(
        (embedded_ms - recomputed_ms).abs() <= 1,
        "埋め込み duration {embedded_ms}ms と再計算 duration {recomputed_ms}ms の差が\
         許容誤差を超えた。SPRING_DURATION_MS を \"{recomputed_ms}ms\" へ転記すること"
    );
}

/// プリセット値が [`Theme::push_motion`]（内部で `CssValue`/`TokenName` の
/// allowlist を通す）を通過し、危険文字（`<`/`{`/`}`/`;`）を含まない。
#[test]
fn spring_preset_values_pass_css_value_allowlist() {
    let mut theme = Theme::empty();
    theme
        .push_motion("easing-spring", SPRING_EASING_LINEAR)
        .expect("SPRING_EASING_LINEAR は CssValue allowlist を満たす");
    theme
        .push_motion("duration-spring", SPRING_DURATION_MS)
        .expect("SPRING_DURATION_MS は CssValue allowlist を満たす");

    for value in [SPRING_EASING_LINEAR, SPRING_DURATION_MS] {
        assert!(value.len() <= 256, "CSS_VALUE_MAX_LEN を超えている");
        for forbidden in ['<', '{', '}', ';'] {
            assert!(!value.contains(forbidden), "禁止文字 {forbidden} を含む");
        }
    }
}

/// [`Theme::push_spring_easing`] は既定テーマへ 2 行（`:root` の
/// `easing-spring`/`duration-spring`）と reduced-motion ブロック内の
/// `duration-spring: 0ms;` 1 行を純追加するだけで、それ以外の出力は
/// [`Theme::default`] の `to_css()` とバイト一致する（golden 純追加）。
#[test]
fn push_spring_easing_is_pure_addition_to_default_theme() {
    let base = Theme::default().to_css();

    let mut theme = Theme::default();
    theme
        .push_spring_easing()
        .expect("既定テーマは easing-spring/duration-spring 未登録のため Ok");
    let with_spring = theme.to_css();

    let root_line = format!("  --fandhe-motion-easing-spring: {SPRING_EASING_LINEAR};\n");
    let root_duration_line = format!("  --fandhe-motion-duration-spring: {SPRING_DURATION_MS};\n");
    let reduced_line = "    --fandhe-motion-duration-spring: 0ms;\n";

    assert!(
        with_spring.contains(&root_line),
        ":root に easing-spring 行が無い:\n{with_spring}"
    );
    assert!(
        with_spring.contains(&root_duration_line),
        ":root に duration-spring 行が無い:\n{with_spring}"
    );
    assert!(
        with_spring.contains(reduced_line),
        "reduced-motion ブロックに duration-spring: 0ms 行が無い:\n{with_spring}"
    );

    let stripped = with_spring
        .replacen(&root_line, "", 1)
        .replacen(&root_duration_line, "", 1)
        .replacen(reduced_line, "", 1);
    assert_eq!(
        stripped, base,
        "spring プリセット追加後の CSS が、追加 3 行を除いて既定テーマと一致しない"
    );
}

/// 2 回連続で呼ぶと 2 回目は `Err(DuplicateTokenName)` を返し、出力は
/// 1 回目のまま変化しない（部分更新が残らないことの確認）。
#[test]
fn push_spring_easing_twice_is_rejected_without_partial_mutation() {
    let mut theme = Theme::default();
    theme.push_spring_easing().expect("1 回目は Ok");
    let after_first = theme.to_css();

    let err = theme
        .push_spring_easing()
        .expect_err("2 回目は重複エラーになるはず");
    assert!(matches!(err, ThemeError::DuplicateTokenName { .. }));
    assert_eq!(
        theme.to_css(),
        after_first,
        "2 回目の呼び出しで出力が変化した（部分更新が残っている）"
    );
}

/// `easing-spring` のみ事前に `push_motion` 済みのテーマでも `Err` になり、
/// `duration-spring` は追加されない（片方だけ登録済みの重複検知）。
#[test]
fn push_spring_easing_rejects_when_easing_already_registered() {
    let mut theme = Theme::empty();
    theme
        .push_motion("easing-spring", "linear(0, 1)")
        .expect("独自の easing-spring 登録は Ok");

    let err = theme
        .push_spring_easing()
        .expect_err("easing-spring が既に登録済みのため Err");
    assert!(matches!(err, ThemeError::DuplicateTokenName { .. }));
    assert!(
        !theme.to_css().contains("duration-spring"),
        "duration-spring が部分的に追加されている"
    );
}
