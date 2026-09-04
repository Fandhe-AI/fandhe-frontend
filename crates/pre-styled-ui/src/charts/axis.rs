//! X/Y 軸（イシュー #847、chakra-ui `charts/axes.md` 相当）。
//!
//! [`super::scale::LinearScale`]（座標写像）・[`super::svg`]（SVG ノード木
//! 生成）を合成し、軸線・目盛線・目盛ラベルの `<g>` を組み立てる。後続の
//! 各チャート部品（Area/Bar/Line/Pie、#848〜#851）は本モジュールの
//! [`y_axis`]/[`x_axis_linear`]/[`x_axis_categories`] を、自身が描く
//! データ系列と同じ [`super::scale::LinearScale`]・座標系の上に重ねて使う
//! 想定である。
//!
//! # データ点数の DoS 耐性について
//!
//! `ticks`/`categories` はいずれもスライス引数であり、本モジュール自体は
//! 追加のループ上限を持たない。呼び出し元（[`super::scale::LinearScale::ticks`]）
//! が `target` を 1..=50 に制限しているため（`.claude/rules/security.md`
//! A04 対応、`scale.rs` 参照）、呼び出し元の契約を守る限り目盛本数は
//! 有界である。

use super::scale::LinearScale;
use super::svg::{group, line, svg_text};
use super::ChartError;
use crate::css::decl;
use crate::recipe::SlotRecipe;
use fandhe_frontend_headless_ui::fandhe_frontend_core::{text, Node};

/// 本モジュールの anatomy scope（[`super::grid`] と共有、
/// `crates/pre-styled-ui/src/charts/mod.rs` §「anatomy / recipe 設計」参照）。
const SCOPE: &str = "chart";

/// [`recipe`] に渡す slot 一覧。
const SLOTS: &[&str] = &["x-axis", "y-axis", "axis-line", "tick-line", "tick-label"];

/// 目盛ラベルの書式（chakra-ui `tickFormatter` クロージャを固定接頭辞・
/// 接尾辞のみへ縮約する。ロケール依存の日付フォーマット等は
/// スコープ外、[`crate::charts::mod`] rustdoc 参照）。
///
/// 値本体の文字列化は常に [`super::svg::fmt_coord`] を経由する
/// （`.claude/rules/coding-rust.md` の数値決定的文字列化の一元化）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TickLabelFormat {
    /// 値の前に付ける固定文字列（例: `"$"`）。
    pub prefix: &'static str,
    /// 値の後に付ける固定文字列（例: `"%"`）。
    pub suffix: &'static str,
}

impl TickLabelFormat {
    /// 値 `v` をこの書式でラベル文字列化する（`prefix` + [`super::svg::fmt_coord`]
    /// + `suffix`）。
    #[must_use]
    pub fn format(&self, v: f64) -> String {
        format!("{}{}{}", self.prefix, super::svg::fmt_coord(v), self.suffix)
    }
}

/// [`y_axis`]/[`x_axis_linear`]/[`x_axis_categories`] 共通の見た目 props。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxisProps {
    /// 軸線（原点から端までの直線）を描画するかどうか（既定 `true`）。
    pub show_axis_line: bool,
    /// 目盛線（軸から突き出す短い線）を描画するかどうか（既定 `true`）。
    pub show_tick_lines: bool,
    /// 目盛ラベルの書式（既定は接頭辞・接尾辞なし）。
    pub format: TickLabelFormat,
}

impl Default for AxisProps {
    fn default() -> Self {
        AxisProps {
            show_axis_line: true,
            show_tick_lines: true,
            format: TickLabelFormat::default(),
        }
    }
}

/// 目盛線の突き出し長（px、固定値。chakra-ui の既定 tick length 相当）。
const TICK_LENGTH: f64 = 6.0;
/// 目盛ラベルと軸線の間隔（px、固定値）。
const LABEL_GAP: f64 = 10.0;

/// Axis の recipe（scope `"chart"`、[`SLOTS`] の 5 パーツ）。
///
/// [`super::grid`] も同じ scope `"chart"` を使うが、slot 名（本モジュールの
/// `axis-line`/`tick-line`/`tick-label` と grid の `grid`/`grid-line`）が
/// 互いに素であるため CSS セレクタは衝突しない
/// （`SlotRecipe` は scope の一意性を要求しない、`crate::recipe` 冒頭 doc 参照）。
///
/// # 参考サイト基準への調整（イシュー #1593）
///
/// 参照 4 サイトに対応部品が無いため内部整合のみを評価軸とした。
/// `tick-line` のストローク色を `axis-line` と同じ `--fandhe-color-border`
/// へ統一し（従来は `border-muted` で軸線より薄く、同じ軸の一部としての
/// 一貫性を欠いていた）、`tick-label` に `font-variant-numeric: tabular-nums`
/// を追加した（数値目盛の桁幅を揃える。クレート内先例多数、
/// 例: `crate::bar_list::value`）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new(SCOPE, SLOTS)
        .base(
            "axis-line",
            vec![
                decl("stroke", "var(--fandhe-color-border)"),
                decl("stroke-width", "1"),
            ],
        )
        .base(
            "tick-line",
            vec![
                // イシュー #1593: axis-line と同じ濃さの --fandhe-color-border
                // へ統一（従来の border-muted は dark で背景とのコントラストが
                // 乏しく、同じ軸の一部である axis-line と濃度が不揃いだった）。
                decl("stroke", "var(--fandhe-color-border)"),
                decl("stroke-width", "1"),
            ],
        )
        .base(
            "tick-label",
            vec![
                decl("fill", "var(--fandhe-color-fg-muted)"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("font-family", "var(--fandhe-font-font-body)"),
                // イシュー #1593: 数値目盛の桁幅を揃え、隣接する目盛ラベル間で
                // 数字の位置が横方向にぶれないようにする。
                decl("font-variant-numeric", "tabular-nums"),
            ],
        )
}

/// Axis の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// Y 軸（縦軸）を組み立てる。`x` は軸が描画される垂直線の x 座標
/// （通常はプロット領域の左端）。目盛の y 座標は `scale.scale(tick)` で
/// 求め、目盛線はそこから左へ [`TICK_LENGTH`] だけ突き出す。
///
/// # Errors
///
/// - `ticks` が空の場合 [`ChartError::EmptyData`]
/// - `x` または `ticks` のいずれかの要素が非有限の場合
///   [`ChartError::NonFiniteValue`]
pub fn y_axis(
    scale: &LinearScale,
    ticks: &[f64],
    x: f64,
    props: &AxisProps,
) -> Result<Node, ChartError> {
    if ticks.is_empty() {
        return Err(ChartError::EmptyData);
    }
    if !x.is_finite() || ticks.iter().any(|t| !t.is_finite()) {
        return Err(ChartError::NonFiniteValue);
    }

    let (r0, r1) = scale.range();
    let mut children = Vec::new();

    if props.show_axis_line {
        children.push(line(
            x,
            r0,
            x,
            r1,
            vec![("data-scope", SCOPE), ("data-part", "axis-line")],
        ));
    }

    for &t in ticks {
        let y = scale.scale(t);
        if props.show_tick_lines {
            children.push(line(
                x - TICK_LENGTH,
                y,
                x,
                y,
                vec![("data-scope", SCOPE), ("data-part", "tick-line")],
            ));
        }
        let label = props.format.format(t);
        children.push(svg_text(
            x - LABEL_GAP,
            y,
            vec![
                ("data-scope", SCOPE),
                ("data-part", "tick-label"),
                ("text-anchor", "end"),
                ("dominant-baseline", "middle"),
            ],
            vec![text(&label)],
        ));
    }

    Ok(group(
        vec![("data-scope", SCOPE), ("data-part", "y-axis")],
        children,
    ))
}

/// X 軸（横軸、連続値目盛）を組み立てる。`y` は軸が描画される水平線の y
/// 座標（通常はプロット領域の下端）。[`y_axis`] の水平版であり、目盛線は
/// 下へ突き出す。
///
/// # Errors
///
/// [`y_axis`] と同様（`ticks` 空 → [`ChartError::EmptyData`]、`y`/`ticks`
/// 非有限 → [`ChartError::NonFiniteValue`]）。
pub fn x_axis_linear(
    scale: &LinearScale,
    ticks: &[f64],
    y: f64,
    props: &AxisProps,
) -> Result<Node, ChartError> {
    if ticks.is_empty() {
        return Err(ChartError::EmptyData);
    }
    if !y.is_finite() || ticks.iter().any(|t| !t.is_finite()) {
        return Err(ChartError::NonFiniteValue);
    }

    let (r0, r1) = scale.range();
    let mut children = Vec::new();

    if props.show_axis_line {
        children.push(line(
            r0,
            y,
            r1,
            y,
            vec![("data-scope", SCOPE), ("data-part", "axis-line")],
        ));
    }

    for &t in ticks {
        let x = scale.scale(t);
        if props.show_tick_lines {
            children.push(line(
                x,
                y,
                x,
                y + TICK_LENGTH,
                vec![("data-scope", SCOPE), ("data-part", "tick-line")],
            ));
        }
        let label = props.format.format(t);
        children.push(svg_text(
            x,
            y + TICK_LENGTH + LABEL_GAP,
            vec![
                ("data-scope", SCOPE),
                ("data-part", "tick-label"),
                ("text-anchor", "middle"),
            ],
            vec![text(&label)],
        ));
    }

    Ok(group(
        vec![("data-scope", SCOPE), ("data-part", "x-axis")],
        children,
    ))
}

/// X 軸（横軸、カテゴリ目盛）を組み立てる。[`super::data::ChartData::categories`]
/// をそのまま渡す想定（棒グラフ・折れ線グラフのカテゴリ軸、#848〜#851）。
///
/// `range` はプロット領域の水平方向の値域（`(左端, 右端)`）。各カテゴリの
/// ラベル位置はカテゴリ帯（band）の中心 `start + (i + 0.5) * width / n`
/// （`n = categories.len()`）とする（chakra-ui/d3 の `scaleBand` 中心配置
/// 相当）。
///
/// # Errors
///
/// - `categories` が空の場合 [`ChartError::EmptyData`]
/// - `range` のいずれかの要素または `y` が非有限の場合
///   [`ChartError::NonFiniteValue`]
pub fn x_axis_categories(
    range: (f64, f64),
    categories: &[String],
    y: f64,
    props: &AxisProps,
) -> Result<Node, ChartError> {
    if categories.is_empty() {
        return Err(ChartError::EmptyData);
    }
    if !range.0.is_finite() || !range.1.is_finite() || !y.is_finite() {
        return Err(ChartError::NonFiniteValue);
    }

    let n = categories.len() as f64;
    let width = (range.1 - range.0) / n;
    let mut children = Vec::new();

    if props.show_axis_line {
        children.push(line(
            range.0,
            y,
            range.1,
            y,
            vec![("data-scope", SCOPE), ("data-part", "axis-line")],
        ));
    }

    for (i, category) in categories.iter().enumerate() {
        let cx = range.0 + (i as f64 + 0.5) * width;
        if props.show_tick_lines {
            children.push(line(
                cx,
                y,
                cx,
                y + TICK_LENGTH,
                vec![("data-scope", SCOPE), ("data-part", "tick-line")],
            ));
        }
        children.push(svg_text(
            cx,
            y + TICK_LENGTH + LABEL_GAP,
            vec![
                ("data-scope", SCOPE),
                ("data-part", "tick-label"),
                ("text-anchor", "middle"),
            ],
            vec![text(category)],
        ));
    }

    Ok(group(
        vec![("data-scope", SCOPE), ("data-part", "x-axis")],
        children,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn scale() -> LinearScale {
        LinearScale::new((0.0, 100.0), (200.0, 0.0)).unwrap()
    }

    #[test]
    fn y_axis_rejects_empty_ticks() {
        assert_eq!(
            y_axis(&scale(), &[], 0.0, &AxisProps::default()).unwrap_err(),
            ChartError::EmptyData
        );
    }

    #[test]
    fn y_axis_rejects_non_finite_x_or_ticks() {
        assert_eq!(
            y_axis(&scale(), &[0.0, f64::NAN], 0.0, &AxisProps::default()).unwrap_err(),
            ChartError::NonFiniteValue
        );
        assert_eq!(
            y_axis(&scale(), &[0.0], f64::INFINITY, &AxisProps::default()).unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn y_axis_renders_axis_line_tick_lines_and_labels() {
        let node = y_axis(&scale(), &[0.0, 50.0, 100.0], 0.0, &AxisProps::default()).unwrap();
        let html = render(&node);
        assert!(html.contains(r#"data-part="y-axis""#));
        assert!(html.contains(r#"data-part="axis-line""#));
        assert!(html.contains(r#"data-part="tick-line""#));
        assert!(html.contains(r#"data-part="tick-label""#));
        // scale((0,100) -> (200,0)) の 0 は range 上端 200、100 は下端 0。
        assert!(html.contains(r#"y1="200""#));
        assert!(html.contains(r#"y2="0""#));
    }

    #[test]
    fn y_axis_props_can_disable_axis_line_and_tick_lines() {
        let props = AxisProps {
            show_axis_line: false,
            show_tick_lines: false,
            ..AxisProps::default()
        };
        let html = render(&y_axis(&scale(), &[0.0, 100.0], 0.0, &props).unwrap());
        assert!(!html.contains(r#"data-part="axis-line""#));
        assert!(!html.contains(r#"data-part="tick-line""#));
        assert!(html.contains(r#"data-part="tick-label""#));
    }

    #[test]
    fn tick_label_format_applies_prefix_and_suffix() {
        let format = TickLabelFormat {
            prefix: "$",
            suffix: "%",
        };
        assert_eq!(format.format(12.5), "$12.5%");
        assert_eq!(TickLabelFormat::default().format(12.5), "12.5");
    }

    #[test]
    fn x_axis_linear_rejects_empty_ticks_and_non_finite_values() {
        assert_eq!(
            x_axis_linear(&scale(), &[], 0.0, &AxisProps::default()).unwrap_err(),
            ChartError::EmptyData
        );
        assert_eq!(
            x_axis_linear(&scale(), &[0.0], f64::NAN, &AxisProps::default()).unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn x_axis_linear_renders_horizontal_axis() {
        let html =
            render(&x_axis_linear(&scale(), &[0.0, 100.0], 0.0, &AxisProps::default()).unwrap());
        assert!(html.contains(r#"data-part="x-axis""#));
        assert!(html.contains(r#"data-part="axis-line""#));
    }

    #[test]
    fn x_axis_categories_rejects_empty_categories_and_non_finite_range() {
        assert_eq!(
            x_axis_categories((0.0, 100.0), &[], 0.0, &AxisProps::default()).unwrap_err(),
            ChartError::EmptyData
        );
        assert_eq!(
            x_axis_categories(
                (f64::NAN, 100.0),
                &["a".to_string()],
                0.0,
                &AxisProps::default()
            )
            .unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn x_axis_categories_centers_labels_in_bands() {
        let categories = vec!["a".to_string(), "b".to_string()];
        let html = render(
            &x_axis_categories((0.0, 100.0), &categories, 0.0, &AxisProps::default()).unwrap(),
        );
        // band 幅 50: "a" の中心 25、"b" の中心 75。
        assert!(html.contains(r#"x="25""#));
        assert!(html.contains(r#"x="75""#));
    }

    #[test]
    fn xss_regression_category_and_suffix_labels_are_escaped() {
        let payload = "</text><script>alert(1)</script>";
        let categories = vec![payload.to_string()];
        let html = render(
            &x_axis_categories((0.0, 100.0), &categories, 0.0, &AxisProps::default()).unwrap(),
        );
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));

        let format = TickLabelFormat {
            prefix: "",
            suffix: "",
        };
        assert_eq!(format.format(1.0), "1");
    }

    #[test]
    fn css_output_is_closed_charset_and_never_contains_angle_bracket() {
        let out = css();
        assert!(!out.contains('<'));
        assert!(out.contains("data-part=\"axis-line\""));
        assert!(out.contains("data-part=\"tick-label\""));
    }
}
