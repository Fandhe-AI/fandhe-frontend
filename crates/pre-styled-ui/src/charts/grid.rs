//! CartesianGrid（イシュー #847、chakra-ui `charts/cartesian-grid.md` 相当）。
//!
//! プロット領域内の水平・垂直グリッド線を描画する。目盛位置（`x_positions`/
//! `y_positions`）は通常 [`super::axis`] と同じ
//! [`super::scale::LinearScale::ticks`] の写像結果を渡す想定であり、軸と
//! グリッドが同じ目盛集合を共有することで視覚的に整合する。

use super::svg::{group, line};
use super::ChartError;
use crate::css::decl;
use crate::recipe::{SlotRecipe, VariantValue};

/// 本モジュールの anatomy scope（[`super::axis`] と共有、
/// `crates/pre-styled-ui/src/charts/mod.rs` §「anatomy / recipe 設計」参照）。
const SCOPE: &str = "chart";

/// [`recipe`] に渡す slot 一覧。
const SLOTS: &[&str] = &["grid", "grid-line"];

/// グリッド線のスタイル軸（chakra-ui `strokeDasharray` を実線/破線の 2 択へ
/// 縮約する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GridLines {
    /// 実線（既定）。
    #[default]
    Solid,
    /// 破線（`stroke-dasharray: 3 3`）。
    Dashed,
}

impl VariantValue for GridLines {
    fn axis(self) -> &'static str {
        "lines"
    }

    fn value(self) -> &'static str {
        match self {
            GridLines::Solid => "solid",
            GridLines::Dashed => "dashed",
        }
    }
}

/// [`cartesian_grid`] の見た目 props。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridProps {
    /// 水平グリッド線を描画するかどうか（既定 `true`）。
    pub horizontal: bool,
    /// 垂直グリッド線を描画するかどうか（既定 `true`）。
    pub vertical: bool,
    /// 線種（既定 [`GridLines::Solid`]）。
    pub lines: GridLines,
}

impl Default for GridProps {
    fn default() -> Self {
        GridProps {
            horizontal: true,
            vertical: true,
            lines: GridLines::Solid,
        }
    }
}

/// CartesianGrid の recipe（scope `"chart"`、[`SLOTS`] の 2 パーツ）。
///
/// # 参考サイト基準への調整（イシュー #1593）
///
/// 参照 4 サイトに対応部品が無いため内部整合のみを評価軸とし、本モジュールは
/// CSS 変更なしと判定した。`grid-line` は「プロット内の補助線」として
/// [`super::axis`] の `axis-line`/`tick-line`（#1593 で `--fandhe-color-border`
/// へ統一）より控えめな `--fandhe-color-border-muted` を意図的に維持する
/// （軸線と同じ濃さにすると補助線がデータ系列と競合して見づらくなるため）。
/// `stroke-dasharray: 3 3` はジオメトリ値でありトークン軸を持たないため
/// トークン化しない。
fn recipe() -> SlotRecipe {
    SlotRecipe::new(SCOPE, SLOTS)
        .base(
            "grid-line",
            vec![
                decl("stroke", "var(--fandhe-color-border-muted)"),
                decl("stroke-width", "1"),
            ],
        )
        .variant(
            GridLines::Dashed,
            "grid-line",
            vec![decl("stroke-dasharray", "3 3")],
        )
        .default_variant(GridLines::Solid)
}

/// CartesianGrid の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// 水平・垂直グリッド線の `<g>` を組み立てる。
///
/// `x_range`/`y_range` はプロット領域の値域（各線の端点に使う）。
/// `x_positions`（垂直線の x 座標列）・`y_positions`（水平線の y 座標列）は
/// 通常 [`super::scale::LinearScale::ticks`] の写像結果を渡す。
///
/// # Errors
///
/// `x_range`/`y_range`・`x_positions`/`y_positions` のいずれかに非有限値が
/// 含まれる場合 [`ChartError::NonFiniteValue`]。
pub fn cartesian_grid(
    x_range: (f64, f64),
    y_range: (f64, f64),
    x_positions: &[f64],
    y_positions: &[f64],
    props: &GridProps,
) -> Result<fandhe_frontend_headless_ui::fandhe_frontend_core::Node, ChartError> {
    let ranges_finite = x_range.0.is_finite()
        && x_range.1.is_finite()
        && y_range.0.is_finite()
        && y_range.1.is_finite();
    if !ranges_finite {
        return Err(ChartError::NonFiniteValue);
    }
    if x_positions.iter().any(|v| !v.is_finite()) || y_positions.iter().any(|v| !v.is_finite()) {
        return Err(ChartError::NonFiniteValue);
    }

    let recipe = recipe();
    let class = recipe.variant_classes(&[("lines", props.lines.value())]);
    let mut children = Vec::new();

    if props.horizontal {
        for &y in y_positions {
            children.push(line(
                x_range.0,
                y,
                x_range.1,
                y,
                vec![
                    ("data-scope", SCOPE),
                    ("data-part", "grid-line"),
                    ("class", class.as_str()),
                ],
            ));
        }
    }
    if props.vertical {
        for &x in x_positions {
            children.push(line(
                x,
                y_range.0,
                x,
                y_range.1,
                vec![
                    ("data-scope", SCOPE),
                    ("data-part", "grid-line"),
                    ("class", class.as_str()),
                ],
            ));
        }
    }

    Ok(group(
        vec![("data-scope", SCOPE), ("data-part", "grid")],
        children,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn cartesian_grid_rejects_non_finite_ranges_or_positions() {
        assert_eq!(
            cartesian_grid((f64::NAN, 1.0), (0.0, 1.0), &[], &[], &GridProps::default())
                .unwrap_err(),
            ChartError::NonFiniteValue
        );
        assert_eq!(
            cartesian_grid(
                (0.0, 1.0),
                (0.0, 1.0),
                &[f64::NAN],
                &[],
                &GridProps::default()
            )
            .unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn cartesian_grid_renders_horizontal_and_vertical_lines() {
        let node = cartesian_grid(
            (0.0, 100.0),
            (0.0, 100.0),
            &[25.0, 75.0],
            &[10.0, 90.0],
            &GridProps::default(),
        )
        .unwrap();
        let html = render(&node);
        assert!(html.contains(r#"data-part="grid""#));
        assert_eq!(html.matches(r#"data-part="grid-line""#).count(), 4);
        assert!(html.contains(r#"x1="25""#));
        assert!(html.contains(r#"y1="10""#));
    }

    #[test]
    fn cartesian_grid_props_can_disable_horizontal_or_vertical() {
        let props = GridProps {
            horizontal: false,
            vertical: true,
            lines: GridLines::Solid,
        };
        let html =
            render(&cartesian_grid((0.0, 100.0), (0.0, 100.0), &[25.0], &[10.0], &props).unwrap());
        assert_eq!(html.matches(r#"data-part="grid-line""#).count(), 1);
    }

    #[test]
    fn cartesian_grid_dashed_variant_applies_class() {
        let props = GridProps {
            lines: GridLines::Dashed,
            ..GridProps::default()
        };
        let html =
            render(&cartesian_grid((0.0, 100.0), (0.0, 100.0), &[25.0], &[], &props).unwrap());
        assert!(html.contains("fd-chart--lines-dashed"));
    }

    #[test]
    fn css_output_declares_dashed_variant_and_is_closed_charset() {
        let out = css();
        assert!(out.contains("stroke-dasharray"));
        assert!(out.contains("fd-chart--lines-dashed"));
        assert!(!out.contains('<'));
    }
}
