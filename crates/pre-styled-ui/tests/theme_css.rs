//! `fandhe-frontend-pre-styled-ui` のテーマトークン CSS 出力・ダークモード両対応の
//! 統合テスト（イシュー #547 受け入れ条件 1・2）。
//!
//! ここで固定する出力構造は `Theme::to_css` の rustdoc（`src/theme.rs`）に記載の
//! 契約と一致させる。#548（variant API）・#550/#551（styled 部品）は本テストが
//! 固定する CSS custom property 名（`--fandhe-<group>-<name>`）に依存するため、
//! 破壊的変更時は本ファイルの更新とあわせて周知する。

use fandhe_frontend_pre_styled_ui::theme::{
    color_var, radius_var, shadow_var, space_var, typography_var, Theme,
};

#[test]
fn default_theme_css_contains_expected_structure() {
    let css = Theme::default().to_css();

    assert!(css.contains(":root {"));
    assert!(css.contains("color-scheme: light dark;"));
    assert!(css.contains("--fandhe-color-bg: #ffffff;"));
    assert!(css.contains("--fandhe-space-4: 1rem;"));
    assert!(css.contains("--fandhe-font-font-size-md: 1rem;"));
    assert!(css.contains(":root[data-theme=\"light\"] { color-scheme: light; }"));
    assert!(css.contains("@media (prefers-color-scheme: dark) {"));
    assert!(css.contains(":root:not([data-theme=\"light\"]) {"));
    assert!(css.contains(":root[data-theme=\"dark\"] {"));
    assert!(css.contains("--fandhe-color-bg: #111111;"));
    assert!(css.contains("--fandhe-radius-md: 0.375rem;"));
    assert!(css.contains("--fandhe-shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.12);"));
}

#[test]
fn default_theme_shadow_dark_value_appears_in_media_and_data_theme_blocks() {
    // shadow はダークモードで光量の異なる値を持つ（イシュー #606）。既定テーマの
    // `shadow-sm` について、colors と同じ「media クエリブロック + data-theme
    // ブロックの計 2 箇所」という dark 値の出力規約を満たすことを固定する。
    let css = Theme::default().to_css();
    let dark_count = css
        .matches("--fandhe-shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.32);")
        .count();
    assert_eq!(dark_count, 2);
}

#[test]
fn data_theme_dark_block_is_ordered_after_media_query_block() {
    // `@media` と `:root[data-theme="dark"]` は同特異度のため、後勝ちである
    // CSS のカスケード規則上、`data-theme` ブロックが出力順で後に来ることが
    // 「明示指定が OS 設定より常に勝つ」という仕様の必須条件になる。
    let css = Theme::default().to_css();

    let media_pos = css
        .find("@media (prefers-color-scheme: dark)")
        .expect("media query block must exist");
    let data_theme_dark_pos = css
        .find(":root[data-theme=\"dark\"]")
        .expect("data-theme dark block must exist");

    assert!(
        media_pos < data_theme_dark_pos,
        "data-theme dark block must be ordered after the media query block"
    );
}

#[test]
fn to_css_is_deterministic_across_calls_and_instances() {
    let theme_a = Theme::default();
    let theme_b = Theme::default();

    assert_eq!(theme_a.to_css(), theme_a.to_css());
    assert_eq!(theme_a.to_css(), theme_b.to_css());
}

#[test]
fn custom_theme_output_matches_full_snapshot() {
    let mut theme = Theme::empty();
    theme.push_color("bg", "#ffffff", "#000000").unwrap();
    theme.push_color("fg", "#000000", "#ffffff").unwrap();
    theme.push_space("4", "1rem").unwrap();
    theme.push_typography("font-size-md", "1rem").unwrap();

    let expected = "\
:root {
  color-scheme: light dark;
  --fandhe-color-bg: #ffffff;
  --fandhe-color-fg: #000000;
  --fandhe-space-4: 1rem;
  --fandhe-font-font-size-md: 1rem;
}
:root[data-theme=\"light\"] { color-scheme: light; }
@media (prefers-color-scheme: dark) {
  :root:not([data-theme=\"light\"]) {
    color-scheme: dark;
    --fandhe-color-bg: #000000;
    --fandhe-color-fg: #ffffff;
  }
}
:root[data-theme=\"dark\"] {
  color-scheme: dark;
  --fandhe-color-bg: #000000;
  --fandhe-color-fg: #ffffff;
}
";

    assert_eq!(theme.to_css(), expected);
}

#[test]
fn custom_color_token_appears_in_all_three_blocks() {
    let mut theme = Theme::empty();
    theme.push_color("brand", "#abcdef", "#123456").unwrap();

    let css = theme.to_css();
    let light_count = css.matches("--fandhe-color-brand: #abcdef;").count();
    let dark_count = css.matches("--fandhe-color-brand: #123456;").count();

    assert_eq!(
        light_count, 1,
        "light value must appear exactly once (:root block)"
    );
    assert_eq!(
        dark_count, 2,
        "dark value must appear exactly twice (media block + data-theme block)"
    );
}

#[test]
fn var_reference_helpers_match_generated_property_names() {
    let theme = Theme::default();
    let css = theme.to_css();

    assert_eq!(color_var("bg").unwrap(), "var(--fandhe-color-bg)");
    assert_eq!(space_var("4").unwrap(), "var(--fandhe-space-4)");
    assert_eq!(
        typography_var("font-size-md").unwrap(),
        "var(--fandhe-font-font-size-md)"
    );
    assert_eq!(radius_var("md").unwrap(), "var(--fandhe-radius-md)");
    assert_eq!(shadow_var("sm").unwrap(), "var(--fandhe-shadow-sm)");

    assert!(css.contains("--fandhe-color-bg:"));
    assert!(css.contains("--fandhe-space-4:"));
    assert!(css.contains("--fandhe-font-font-size-md:"));
    assert!(css.contains("--fandhe-radius-md:"));
    assert!(css.contains("--fandhe-shadow-sm:"));
}

#[test]
fn custom_radii_and_shadows_extend_full_snapshot_without_breaking_pre_606_output() {
    // radii/shadows を push しないテーマは `custom_theme_output_matches_full_snapshot`
    // （本ファイル）の既存スナップショットとバイト同一のままであることが
    // #606 の後方互換要件。ここでは radii/shadows を追加した場合の出力構造を
    // 個別に固定する（フルスナップショットへ混入させ既存テストを壊さない）。
    let mut theme = Theme::empty();
    theme.push_color("bg", "#ffffff", "#000000").unwrap();
    theme.push_radius("md", "0.375rem").unwrap();
    theme
        .push_shadow(
            "sm",
            "0 1px 3px rgba(0, 0, 0, 0.12)",
            "0 1px 3px rgba(0, 0, 0, 0.32)",
        )
        .unwrap();

    let expected = "\
:root {
  color-scheme: light dark;
  --fandhe-color-bg: #ffffff;
  --fandhe-radius-md: 0.375rem;
  --fandhe-shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.12);
}
:root[data-theme=\"light\"] { color-scheme: light; }
@media (prefers-color-scheme: dark) {
  :root:not([data-theme=\"light\"]) {
    color-scheme: dark;
    --fandhe-color-bg: #000000;
    --fandhe-shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.32);
  }
}
:root[data-theme=\"dark\"] {
  color-scheme: dark;
  --fandhe-color-bg: #000000;
  --fandhe-shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.32);
}
";

    assert_eq!(theme.to_css(), expected);
}
