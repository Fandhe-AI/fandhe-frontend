//! `fandhe-frontend-pre-styled-ui::charts::{axis,grid}` の統合テスト
//! （イシュー #847）。
//!
//! `tests/charts_foundation.rs`（#846 基盤）と同型: クレート公開 API を
//! 外部から呼び出す形で、[`fandhe_frontend_pre_styled_ui::charts::scale::LinearScale`]
//! との合成・目盛座標の決定性・ラベルの既定エスケープ（REQ-1）を固定する。
//!
//! # 削除・弱体化の禁止
//!
//! `.claude/rules/coding-rust.md` の規約により、本ファイルの XSS 回帰
//! テストは以後の削除・弱体化・`#[ignore]` 化を禁止する。

use fandhe_frontend_core::render;
use fandhe_frontend_pre_styled_ui::charts::axis::{self, AxisProps, TickLabelFormat};
use fandhe_frontend_pre_styled_ui::charts::grid::{self, GridLines, GridProps};
use fandhe_frontend_pre_styled_ui::charts::scale::LinearScale;
use fandhe_frontend_pre_styled_ui::charts::ChartError;

/// 既知の domain/target から生成した tick 値をそのまま [`axis::y_axis`] へ
/// 渡した座標が決定的であることを固定する（`crates/pre-styled-ui/src/charts/scale.rs`
/// の `ticks_known_domain_produces_expected_nice_values` と同じ入力を使う）。
#[test]
fn y_axis_tick_coordinates_are_deterministic_for_known_domain() {
    let scale = LinearScale::new((0.0, 100.0), (0.0, 100.0)).unwrap();
    let ticks = scale.ticks(5).unwrap();
    assert_eq!(ticks, vec![0.0, 20.0, 40.0, 60.0, 80.0, 100.0]);

    let html_a = render(&axis::y_axis(&scale, &ticks, 0.0, &AxisProps::default()).unwrap());
    let html_b = render(&axis::y_axis(&scale, &ticks, 0.0, &AxisProps::default()).unwrap());
    assert_eq!(html_a, html_b);

    // range (0,100) の domain (0,100) は恒等写像のため tick 座標がそのまま
    // y 座標になる。
    for y in [
        "y1=\"0\"",
        "y1=\"20\"",
        "y1=\"40\"",
        "y1=\"60\"",
        "y1=\"80\"",
        "y1=\"100\"",
    ] {
        assert!(html_a.contains(y), "missing {y} in {html_a}");
    }
}

/// [`axis::x_axis_categories`] の band 中心配置が既知の入力で期待座標に
/// なることを固定する（設計 `start + (i + 0.5) * width / n`）。
#[test]
fn x_axis_categories_band_centers_match_known_layout() {
    let categories = vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
        "d".to_string(),
    ];
    let html = render(
        &axis::x_axis_categories((0.0, 100.0), &categories, 0.0, &AxisProps::default()).unwrap(),
    );
    // band 幅 25: 中心は 12.5, 37.5, 62.5, 87.5。
    for x in ["x=\"12.5\"", "x=\"37.5\"", "x=\"62.5\"", "x=\"87.5\""] {
        assert!(html.contains(x), "missing {x} in {html}");
    }
}

/// [`axis::x_axis_linear`] の目盛座標が [`LinearScale::scale`] の写像結果と
/// 一致することを固定する（軸と grid が同じスケールを共有する前提の検証）。
#[test]
fn x_axis_linear_tick_positions_match_scale_mapping() {
    let scale = LinearScale::new((0.0, 100.0), (0.0, 200.0)).unwrap();
    let ticks = vec![0.0, 50.0, 100.0];
    let html = render(&axis::x_axis_linear(&scale, &ticks, 300.0, &AxisProps::default()).unwrap());
    for x in ["x1=\"0\"", "x1=\"100\"", "x1=\"200\""] {
        assert!(html.contains(x), "missing {x} in {html}");
    }
    assert!(html.contains("y1=\"300\""));
}

/// [`grid::cartesian_grid`] のグリッド線座標が [`LinearScale::ticks`] の
/// 写像結果と一致し、軸（[`axis::y_axis`]）と同じ目盛集合を共有できることを
/// 固定する。
#[test]
fn cartesian_grid_lines_align_with_axis_tick_positions() {
    let scale = LinearScale::new((0.0, 100.0), (0.0, 100.0)).unwrap();
    let ticks = scale.ticks(5).unwrap();

    let axis_html = render(&axis::y_axis(&scale, &ticks, 0.0, &AxisProps::default()).unwrap());
    let grid_html = render(
        &grid::cartesian_grid(
            (0.0, 200.0),
            (0.0, 100.0),
            &[],
            &ticks,
            &GridProps::default(),
        )
        .unwrap(),
    );

    for y in &ticks {
        let coord = fandhe_frontend_pre_styled_ui::charts::svg::fmt_coord(*y);
        assert!(axis_html.contains(&format!("y1=\"{coord}\"")));
        assert!(grid_html.contains(&format!("y1=\"{coord}\"")));
    }
}

#[test]
fn grid_dashed_lines_variant_renders_stroke_dasharray_class() {
    let props = GridProps {
        lines: GridLines::Dashed,
        ..GridProps::default()
    };
    let html =
        render(&grid::cartesian_grid((0.0, 100.0), (0.0, 100.0), &[50.0], &[], &props).unwrap());
    assert!(html.contains("fd-chart--lines-dashed"));
}

/// fail-closed 検証: 空 ticks/categories・非有限座標は `ChartError` として
/// 拒否される（クレート公開 API 経由）。
#[test]
fn public_api_rejects_empty_and_non_finite_inputs() {
    let scale = LinearScale::new((0.0, 100.0), (0.0, 100.0)).unwrap();
    assert_eq!(
        axis::y_axis(&scale, &[], 0.0, &AxisProps::default()).unwrap_err(),
        ChartError::EmptyData
    );
    assert_eq!(
        axis::x_axis_categories((0.0, 100.0), &[], 0.0, &AxisProps::default()).unwrap_err(),
        ChartError::EmptyData
    );
    assert_eq!(
        grid::cartesian_grid((f64::NAN, 1.0), (0.0, 1.0), &[], &[], &GridProps::default())
            .unwrap_err(),
        ChartError::NonFiniteValue
    );
}

/// XSS 回帰: カテゴリ名に攻撃ペイロードを与えても、SVG テキストノードとして
/// 既定エスケープされること（REQ-1）を固定する。tick prefix/suffix
/// （[`TickLabelFormat`]）は `&'static str`（ソース内リテラル）のみを受け付ける
/// 型のため、動的ペイロードの混入経路自体が存在しない（型レベルでの遮断）。
#[test]
fn xss_regression_category_names_are_escaped_in_svg_text() {
    let payload = "</text><script>alert(1)</script>";
    let categories = vec![payload.to_string()];
    let html = render(
        &axis::x_axis_categories((0.0, 100.0), &categories, 0.0, &AxisProps::default()).unwrap(),
    );
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn tick_label_format_applies_prefix_and_suffix() {
    let format = TickLabelFormat {
        prefix: "$",
        suffix: "%",
    };
    assert_eq!(format.format(1.0), "$1%");
}
