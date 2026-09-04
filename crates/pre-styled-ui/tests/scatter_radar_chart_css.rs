//! ScatterChart / RadarChart（イシュー #851）の決定的 CSS 出力ゴールデン
//! テスト。
//!
//! `crates/pre-styled-ui/tests/qr_code_css.rs` の golden fixture テストの
//! 前例に倣い、`css()` が返す CSS 全文をバイト単位で固定する
//! （受け入れ条件「golden CSS」）。出力順（base の登録順）が崩れた場合や
//! 意図しない宣言の追加・欠落があった場合に、この golden テストが即座に
//! 検知する。
//!
//! `SCATTER_CHART_GOLDEN_CSS` はイシュー #1598（`root` の
//! `overflow: visible` 追加・`point` の `stroke-width` 表記統一）で更新済み。

use fandhe_frontend_pre_styled_ui::charts::{radar_chart, scatter_chart};

const SCATTER_CHART_GOLDEN_CSS: &str = r#"[data-scope="scatter-chart"][data-part="root"] {
  display: block;
  max-width: 100%;
  overflow: visible;
}

[data-scope="scatter-chart"][data-part="point"] {
  stroke: var(--fandhe-color-bg);
  stroke-width: 1;
}
"#;

const RADAR_CHART_GOLDEN_CSS: &str = r#"[data-scope="radar-chart"][data-part="root"] {
  display: block;
  max-width: 100%;
}

[data-scope="radar-chart"][data-part="grid"] {
  stroke: var(--fandhe-color-border);
  fill: none;
}

[data-scope="radar-chart"][data-part="spoke"] {
  stroke: var(--fandhe-color-border);
}

[data-scope="radar-chart"][data-part="axis-label"] {
  font-size: var(--fandhe-font-font-size-xs);
  fill: var(--fandhe-color-fg-muted);
  font-family: var(--fandhe-font-font-body);
}

[data-scope="radar-chart"][data-part="series"] {
  fill-opacity: 0.2;
  stroke-width: 2;
  stroke-linejoin: round;
}
"#;

#[test]
fn scatter_chart_css_matches_golden_fixture() {
    assert_eq!(scatter_chart::css(), SCATTER_CHART_GOLDEN_CSS);
}

#[test]
fn radar_chart_css_matches_golden_fixture() {
    assert_eq!(radar_chart::css(), RADAR_CHART_GOLDEN_CSS);
}

#[test]
fn scatter_and_radar_css_are_byte_identical_across_calls() {
    assert_eq!(scatter_chart::css(), scatter_chart::css());
    assert_eq!(radar_chart::css(), radar_chart::css());
}

#[test]
fn scatter_and_radar_css_never_contain_style_breakout_sequences() {
    for css in [scatter_chart::css(), radar_chart::css()] {
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }
}
