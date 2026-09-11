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
/// 接尾辞・倍率のみへ縮約する。ロケール依存の日付フォーマット等は
/// スコープ外、`crate::charts::mod` rustdoc 参照）。
///
/// 値本体の文字列化は常に [`super::svg::fmt_coord`] を経由する
/// （`.claude/rules/coding-rust.md` の数値決定的文字列化の一元化）。
///
/// `label_scale`（イシュー #2081 追補）は `crate::area_chart` の
/// `AreaStack::Expand`（domain `(0.0, 1.0)` の比率）のように、目盛の
/// 位置計算に使う domain 値とラベル表示値が異なる（比率 0.25 を
/// 「25%」と表示したい）場合の乗数。位置計算（`scale.scale(t)`）は
/// 生の domain 値 `t` を使い、ラベル文字列化のみ `t * label_scale` を
/// 使う（`f64` は `Eq` を実装しないため本型は `PartialEq` のみ導出する）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TickLabelFormat {
    /// 値の前に付ける固定文字列（例: `"$"`）。
    pub prefix: &'static str,
    /// 値の後に付ける固定文字列（例: `"%"`）。
    pub suffix: &'static str,
    /// ラベル表示前に値へ掛ける倍率（既定 `1.0`、位置計算には影響しない）。
    pub label_scale: f64,
}

impl Default for TickLabelFormat {
    fn default() -> Self {
        TickLabelFormat {
            prefix: "",
            suffix: "",
            label_scale: 1.0,
        }
    }
}

impl TickLabelFormat {
    /// 値 `v` をこの書式でラベル文字列化する（`prefix` +
    /// [`super::svg::fmt_coord`]`(v * label_scale)` + `suffix`）。
    ///
    /// # Errors
    ///
    /// `label_scale` 自身が非有限、または `v * label_scale` が非有限
    /// （有限同士の乗算でもオーバーフローで `inf` になり得る）の場合
    /// [`ChartError::NonFiniteValue`] を返す。本メソッドは公開 API として
    /// 直接呼び出し可能であり、[`y_axis`]/[`x_axis_linear`] が呼び出し前
    /// に行う同種の検証（呼び出し元の `ticks` 全体に対する事前チェック）
    /// の有無に関わらず、この入口自身で [`super::svg::fmt_coord`] の
    /// 有限値限定契約（違反時 debug panic / release 非有限文字列）を
    /// 満たす（イシュー #2081 追補）。
    pub fn format(&self, v: f64) -> Result<String, ChartError> {
        let scaled = v * self.label_scale;
        if !self.label_scale.is_finite() || !scaled.is_finite() {
            return Err(ChartError::NonFiniteValue);
        }
        Ok(format!(
            "{}{}{}",
            self.prefix,
            super::svg::fmt_coord(scaled),
            self.suffix
        ))
    }
}

/// [`y_axis`]/[`x_axis_linear`]/[`x_axis_categories`] 共通の見た目 props。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisProps {
    /// 軸線（原点から端までの直線）を描画するかどうか（既定 `true`）。
    pub show_axis_line: bool,
    /// 目盛線（軸から突き出す短い線）を描画するかどうか（既定 `true`）。
    pub show_tick_lines: bool,
    /// 目盛ラベル（`<text>`）を描画するかどうか（既定 `true`、イシュー
    /// #2081 追補）。`crate::area_chart` が軸線のみを別スケール（カテゴリ
    /// 位置合わせ用の `(0.0, 1.0)` ダミースケール）で描くために `false` を
    /// 使う（`ticks` が空でないことを [`x_axis_linear`]/[`y_axis`] が要求
    /// するため、ラベル自体は要らなくても `ticks` は渡す必要がある構成）。
    pub show_tick_labels: bool,
    /// 目盛ラベルの書式（既定は接頭辞・接尾辞なし）。
    pub format: TickLabelFormat,
}

impl Default for AxisProps {
    fn default() -> Self {
        AxisProps {
            show_axis_line: true,
            show_tick_lines: true,
            show_tick_labels: true,
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
/// 求め、目盛線はそこから左へ `TICK_LENGTH` だけ突き出す。
///
/// # Errors
///
/// - `ticks` が空の場合 [`ChartError::EmptyData`]
/// - `x`・`ticks` のいずれかの要素・`props.format.label_scale`・
///   各 `tick * label_scale` のいずれかが非有限の場合
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
    // `label_scale` は `AxisProps` 経由で呼び出し側から渡される未検証値
    // （`TickLabelFormat` doc 参照）。`t` 自体は上で有限を確認済みだが、
    // `label_scale` が NaN/inf、または有限同士でも積がオーバーフローする
    // 場合、`format()` 内部で `svg::fmt_coord` の有限値限定契約に違反し
    // debug ビルドで panic・release ビルドで非有限ラベルを生む。ここで
    // fail-closed に検出する。
    if !props.format.label_scale.is_finite()
        || ticks
            .iter()
            .any(|t| !(t * props.format.label_scale).is_finite())
    {
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
        if props.show_tick_labels {
            let label = props.format.format(t)?;
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
/// [`y_axis`] と同様（`ticks` 空 → [`ChartError::EmptyData`]、`y`/`ticks`/
/// `props.format.label_scale`/各 `tick * label_scale` のいずれかが
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
    // y_axis と同じ理由（上記コメント参照）で label_scale・積の有限性を
    // 描画前に検証する。
    if !props.format.label_scale.is_finite()
        || ticks
            .iter()
            .any(|t| !(t * props.format.label_scale).is_finite())
    {
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
        if props.show_tick_labels {
            let label = props.format.format(t)?;
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
        if props.show_tick_labels {
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
    fn y_axis_rejects_non_finite_or_overflowing_label_scale() {
        // codex-review 指摘（PR #2254）: label_scale が非有限、または有限
        // 同士の積がオーバーフローする場合、是正前は検証なしで
        // `svg::fmt_coord` の有限値限定契約に違反していた。
        let mut props = AxisProps::default();
        props.format.label_scale = f64::NAN;
        assert_eq!(
            y_axis(&scale(), &[10.0], 0.0, &props).unwrap_err(),
            ChartError::NonFiniteValue
        );

        let mut props = AxisProps::default();
        props.format.label_scale = f64::INFINITY;
        assert_eq!(
            y_axis(&scale(), &[10.0], 0.0, &props).unwrap_err(),
            ChartError::NonFiniteValue
        );

        // 有限同士でも積がオーバーフローするケース。
        let mut props = AxisProps::default();
        props.format.label_scale = f64::MAX;
        assert_eq!(
            y_axis(&scale(), &[10.0], 0.0, &props).unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn x_axis_linear_rejects_non_finite_or_overflowing_label_scale() {
        let mut props = AxisProps::default();
        props.format.label_scale = f64::NAN;
        assert_eq!(
            x_axis_linear(&scale(), &[10.0], 0.0, &props).unwrap_err(),
            ChartError::NonFiniteValue
        );

        let mut props = AxisProps::default();
        props.format.label_scale = f64::MAX;
        assert_eq!(
            x_axis_linear(&scale(), &[10.0], 0.0, &props).unwrap_err(),
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

    /// イシュー #2081 追補: `show_tick_labels: false` は `y_axis`/
    /// `x_axis_linear`/`x_axis_categories` いずれでも目盛ラベルを抑止し、
    /// 軸線自体はそのまま描画することを固定する（`crate::area_chart` が
    /// カテゴリラベルを別途描く際に軸線のみを再利用する契約の前提）。
    #[test]
    fn show_tick_labels_false_suppresses_labels_for_all_three_axes() {
        let props = AxisProps {
            show_tick_labels: false,
            ..AxisProps::default()
        };

        let y_html = render(&y_axis(&scale(), &[0.0, 100.0], 0.0, &props).unwrap());
        assert!(!y_html.contains(r#"data-part="tick-label""#));
        assert!(y_html.contains(r#"data-part="axis-line""#));

        let x_html = render(&x_axis_linear(&scale(), &[0.0, 100.0], 0.0, &props).unwrap());
        assert!(!x_html.contains(r#"data-part="tick-label""#));
        assert!(x_html.contains(r#"data-part="axis-line""#));

        let categories = vec!["a".to_string(), "b".to_string()];
        let cat_html = render(&x_axis_categories((0.0, 100.0), &categories, 0.0, &props).unwrap());
        assert!(!cat_html.contains(r#"data-part="tick-label""#));
        assert!(cat_html.contains(r#"data-part="axis-line""#));
    }

    #[test]
    fn tick_label_format_applies_prefix_and_suffix() {
        let format = TickLabelFormat {
            prefix: "$",
            suffix: "%",
            ..TickLabelFormat::default()
        };
        assert_eq!(format.format(12.5), Ok("$12.5%".to_string()));
        assert_eq!(
            TickLabelFormat::default().format(12.5),
            Ok("12.5".to_string())
        );
    }

    /// `label_scale`（イシュー #2081 追補、`crate::area_chart` の
    /// `AreaStack::Expand` 用）は位置計算に使わず、ラベル文字列化のみに
    /// 掛かる倍率であることを固定する。
    #[test]
    fn label_scale_multiplies_before_formatting() {
        let percent = TickLabelFormat {
            suffix: "%",
            label_scale: 100.0,
            ..TickLabelFormat::default()
        };
        assert_eq!(percent.format(0.25), Ok("25%".to_string()));
        assert_eq!(percent.format(0.3), Ok("30%".to_string()));
        assert_eq!(TickLabelFormat::default().label_scale, 1.0);
    }

    /// イシュー #2081 追補（codex-review P1 是正）: `TickLabelFormat::format`
    /// は公開 API として `y_axis`/`x_axis_linear` を経由せず直接呼び出せる
    /// ため、この入口自身で `v * label_scale` の有限性を検証しなければ
    /// ならない。`label_scale: 100.0` で `format(1e308)` を呼ぶと、
    /// `label_scale` 自体・`v` 自体はいずれも有限でも積が `inf` になり、
    /// `fmt_coord` の有限値限定契約に違反する（是正前は debug panic /
    /// release で非有限ラベルを生成していた）。
    #[test]
    fn format_rejects_finite_inputs_whose_product_overflows_to_infinite() {
        let percent = TickLabelFormat {
            label_scale: 100.0,
            ..TickLabelFormat::default()
        };
        assert!(percent.label_scale.is_finite());
        assert!(1e308_f64.is_finite());
        assert_eq!(percent.format(1e308), Err(ChartError::NonFiniteValue));
    }

    /// `label_scale` 自体が非有限（`NaN`/`inf`）の場合も同様に拒否する。
    #[test]
    fn format_rejects_non_finite_label_scale() {
        let broken = TickLabelFormat {
            label_scale: f64::INFINITY,
            ..TickLabelFormat::default()
        };
        assert_eq!(broken.format(1.0), Err(ChartError::NonFiniteValue));

        let nan_scale = TickLabelFormat {
            label_scale: f64::NAN,
            ..TickLabelFormat::default()
        };
        assert_eq!(nan_scale.format(1.0), Err(ChartError::NonFiniteValue));
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
            ..TickLabelFormat::default()
        };
        assert_eq!(format.format(1.0), Ok("1".to_string()));
    }

    #[test]
    fn css_output_is_closed_charset_and_never_contains_angle_bracket() {
        let out = css();
        assert!(!out.contains('<'));
        assert!(out.contains("data-part=\"axis-line\""));
        assert!(out.contains("data-part=\"tick-label\""));
    }
}
