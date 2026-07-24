//! BarSegment（構成比バー、100% 積み上げ、イシュー #849・親 Phase #845）。
//!
//! chakra-ui `charts/bar-segment.md` 相当を HTML（`<div>` ベース）で再構成
//! する。[`super::data::ChartData`] の 1 系列を対象に、各カテゴリを 1
//! セグメントとして「系列合計に対する比率」で幅を割り当てた単一の横棒
//! （100% 積み上げ）として描画する。新規 anatomy `data-scope="bar-segment"`
//! を本モジュールで定義する（[`crate::table`]/[`crate::charts::bar_list`] と
//! 同型の判断、`fandhe-frontend-headless-ui` 側に対応する anatomy はない）。
//!
//! # 配色
//!
//! 各セグメントはカテゴリ index を [`super::series_color_var`] に渡して
//! `chart-1`〜`chart-6` を循環させる（chakra-ui BarSegment がアイテムごとに
//! 色を割り当てる挙動に対応。[`super::bar_chart`] が系列 index で循環させる
//! のとは対象が異なる点に注意）。
//!
//! # 比率の伝搬（インライン custom property）
//!
//! セグメント幅は [`super::data::value_percent`]（合計に対する割合、0 合計は
//! `0.0` を返す既存契約）を [`super::svg::fmt_coord`] で文字列化し、
//! `style="--fandhe-bar-segment-percent: <n>%"` としてインライン伝搬する
//! （[`super::bar_list`] の `--fandhe-bar-list-percent` 方式と同型）。
//!
//! # fail-closed（`.claude/rules/security.md` A04 対応、[`super::bar_list`] との違い）
//!
//! - 対象系列が存在しない場合 [`ChartError::UnknownSeriesName`]。
//! - 系列中に負値が 1 件でもあれば [`ChartError::NegativeValue`]。
//! - **系列合計が 0 の場合は [`ChartError::ZeroTotal`] で構築を拒否する**
//!   （[`super::data::value_percent`] の「合計 0 → `0.0` を返す」契約に
//!   黙って乗ると、全セグメント幅 0% の空バーが「データなし」なのか
//!   「構成比が定義できない」なのか利用者が区別できない silent failure に
//!   なる。[`super::bar_list`] の「値 0 → 幅 0」は個々の値と幅の対応関係が
//!   自明だが、本部品は「合計に対する比率」という関係性そのものが失われる
//!   ため、両部品で挙動を意図的に変えている、モジュール doc に明記する
//!   実装判断）。
//!
//! # セキュリティ不変条件
//!
//! マークアップはすべてノード木 API 経由（`raw_html()` 不使用、REQ-1）。
//! 値の文字列化は [`super::svg::fmt_coord`] にのみ一元化する。インライン
//! `style` 属性値は固定テンプレートのみで構成する（[`super::bar_list`] と
//! 同型の不変条件）。
//!
//! # legend（`showPercent` 相当）
//!
//! [`legend`] は各セグメントの色マーカー・ラベル・比率テキストを静的出力する
//! 最小実装であり、#847 の汎用 Legend（軸/凡例横断部品）とは独立している
//! （境界を明示する。将来的な統合は #847 側の設計判断に委ねる）。
//!
//! # 本イシューのスコープ外
//!
//! - `examples/headless-pre-styled-ui` への追随は crates.io 公開後に別途。

use super::data::{self, ChartData};
use super::svg::fmt_coord;
use super::{series_color_var, ChartError};
use crate::css::decl;
use crate::recipe::SlotRecipe;
use fandhe_frontend_headless_ui::fandhe_frontend_core::{text, Node};
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="bar-segment"` を固定した anatomy。
const ANATOMY: Anatomy = anatomy("bar-segment");

/// [`SlotRecipe::new`] に渡す slot 一覧。
const SLOTS: &[&str] = &[
    "root",
    "bar",
    "segment",
    "legend",
    "legend-item",
    "legend-marker",
    "legend-label",
    "legend-value",
];

/// この BarSegment の既定 CSS を組み立てる（内部ヘルパ、[`css`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("bar-segment", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "0.75rem"),
                decl("width", "100%"),
            ],
        )
        .base(
            "bar",
            vec![
                decl("display", "flex"),
                decl("width", "100%"),
                decl("height", "0.75rem"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("overflow", "hidden"),
            ],
        )
        .base(
            "segment",
            vec![
                decl("height", "100%"),
                decl("width", "var(--fandhe-bar-segment-percent, 0%)"),
            ],
        )
        .base(
            "legend",
            vec![
                decl("display", "flex"),
                decl("flex-wrap", "wrap"),
                decl("gap", "0.75rem 1rem"),
            ],
        )
        .base(
            "legend-item",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                decl("gap", "0.375rem"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .base(
            "legend-marker",
            vec![
                decl("width", "0.625rem"),
                decl("height", "0.625rem"),
                decl("border-radius", "9999px"),
                decl("flex-shrink", "0"),
            ],
        )
        .base(
            "legend-label",
            vec![decl("color", "var(--fandhe-color-fg)")],
        )
        .base(
            "legend-value",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-variant-numeric", "tabular-nums"),
            ],
        )
}

/// この BarSegment が生成する静的 CSS 全量を返す（決定的）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// BarSegment 本体（`bar` + [`legend`]）を組み立てる。
///
/// `data` から `series_name` の系列を取り出し、[`ChartData::categories`] の
/// 順にセグメントを描画する。
///
/// # Errors
///
/// - `series_name` に一致する系列がない場合 [`ChartError::UnknownSeriesName`]
/// - 系列中に負値が含まれる場合 [`ChartError::NegativeValue`]
/// - 系列合計が 0 の場合 [`ChartError::ZeroTotal`]（モジュール doc
///   「fail-closed」節参照）
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::charts::bar_segment::root;
/// use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
///
/// let data = ChartData::new(
///     vec!["a".to_string(), "b".to_string()],
///     vec![Series::new("visits", vec![25.0, 75.0])],
/// )
/// .unwrap();
/// let node = root(&data, "visits").unwrap();
/// assert!(render(&node).contains(r#"data-scope="bar-segment" data-part="root""#));
/// ```
pub fn root(data: &ChartData, series_name: &str) -> Result<Node, ChartError> {
    let series = data
        .series()
        .iter()
        .find(|s| s.name == series_name)
        .ok_or(ChartError::UnknownSeriesName)?;

    if series.values.iter().any(|&v| v < 0.0) {
        return Err(ChartError::NegativeValue);
    }
    if data::total(series) == 0.0 {
        return Err(ChartError::ZeroTotal);
    }

    let categories = data.categories();
    let segments: Vec<Node> = categories
        .iter()
        .zip(series.values.iter())
        .enumerate()
        .map(|(idx, (_category, &value))| segment(idx, value, series))
        .collect();
    let bar = ANATOMY.part("bar", "div", vec![], segments);

    let legend = legend(categories, series);

    Ok(ANATOMY.part("root", "div", vec![], vec![bar, legend]))
}

/// 1 セグメント（`segment`）を組み立てる（内部ヘルパ）。
///
/// `background` はベアな HTML 属性としては存在しないため（ブラウザは無視し
/// `<div>` は無色描画のままになる、PR #877 レビュー指摘）、legend マーカー
/// （[`legend`] 内）と同様に `style` 属性値の一部として埋め込む。
fn segment(idx: usize, value: f64, series: &data::Series) -> Node {
    let percent = data::value_percent(series, value);
    let color = series_color_var(idx);
    let style = format!(
        "--fandhe-bar-segment-percent: {}%; background: {color}",
        fmt_coord(percent)
    );
    ANATOMY.part("segment", "div", vec![("style", style.as_str())], vec![])
}

/// 凡例（[`legend`] モジュール doc 参照）を組み立てる（内部ヘルパ）。
fn legend(categories: &[String], series: &data::Series) -> Node {
    let items: Vec<Node> = categories
        .iter()
        .zip(series.values.iter())
        .enumerate()
        .map(|(idx, (category, &value))| {
            let percent = data::value_percent(series, value);
            let color = series_color_var(idx);
            let marker_style = format!("background: {color}");
            ANATOMY.part(
                "legend-item",
                "span",
                vec![],
                vec![
                    ANATOMY.part(
                        "legend-marker",
                        "span",
                        vec![("style", marker_style.as_str())],
                        vec![],
                    ),
                    ANATOMY.part(
                        "legend-label",
                        "span",
                        vec![],
                        vec![text(category.to_string())],
                    ),
                    ANATOMY.part(
                        "legend-value",
                        "span",
                        vec![],
                        vec![text(format!("{}%", fmt_coord(percent)))],
                    ),
                ],
            )
        })
        .collect();
    ANATOMY.part("legend", "div", vec![], items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::data::Series;
    use fandhe_frontend_core::render;

    fn sample() -> ChartData {
        ChartData::new(
            vec!["a".to_string(), "b".to_string()],
            vec![Series::new("visits", vec![25.0, 75.0])],
        )
        .unwrap()
    }

    #[test]
    fn root_unknown_series_is_error() {
        assert_eq!(
            root(&sample(), "missing").unwrap_err(),
            ChartError::UnknownSeriesName
        );
    }

    #[test]
    fn root_rejects_negative_values() {
        let data =
            ChartData::new(vec!["a".to_string()], vec![Series::new("s", vec![-1.0])]).unwrap();
        assert_eq!(root(&data, "s").unwrap_err(), ChartError::NegativeValue);
    }

    #[test]
    fn root_rejects_zero_total() {
        let data = ChartData::new(
            vec!["a".to_string(), "b".to_string()],
            vec![Series::new("z", vec![0.0, 0.0])],
        )
        .unwrap();
        assert_eq!(root(&data, "z").unwrap_err(), ChartError::ZeroTotal);
    }

    #[test]
    fn root_computes_percent_relative_to_total() {
        let html = render(&root(&sample(), "visits").unwrap());
        assert!(html.contains("--fandhe-bar-segment-percent: 25%"));
        assert!(html.contains("--fandhe-bar-segment-percent: 75%"));
    }

    #[test]
    fn root_rounds_and_sums_to_100_for_thirds() {
        let data = ChartData::new(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec![Series::new("s", vec![1.0, 1.0, 1.0])],
        )
        .unwrap();
        let html = render(&root(&data, "s").unwrap());
        // 33.333...% は fmt_coord の丸め規則（{:.2} → 末尾ゼロ除去）で 33.33%。
        // 3 カテゴリそれぞれについて segment の custom property・legend の
        // 比率テキストの計 2 箇所ずつ出現する（合計 6 箇所）。
        assert_eq!(html.matches("33.33%").count(), 6);
        assert_eq!(
            html.matches("--fandhe-bar-segment-percent: 33.33%").count(),
            3
        );
        assert_eq!(html.matches(">33.33%<").count(), 3);
    }

    #[test]
    fn legend_lists_all_categories_with_percent() {
        let html = render(&root(&sample(), "visits").unwrap());
        assert!(html.contains(r#"data-part="legend""#));
        assert!(html.contains(r#"data-part="legend-item""#));
        assert!(html.contains(">a<"));
        assert!(html.contains(">25%<"));
        assert!(html.contains(">b<"));
        assert!(html.contains(">75%<"));
    }

    #[test]
    fn segment_color_is_set_via_style_not_bare_attribute() {
        // PR #877 レビュー指摘: 'background' がベア HTML 属性のままだと
        // ブラウザは CSS として扱わず無色描画になる。style 属性値の一部
        // として埋め込まれていることを確認する（bare な `background="..."`
        // 属性は存在しないことも合わせて検証する）。
        let html = render(&root(&sample(), "visits").unwrap());
        assert!(html.contains(
            "style=\"--fandhe-bar-segment-percent: 25%; background: var(--fandhe-color-chart-1)\""
        ));
        assert!(!html.contains(" background=\"var(--fandhe-color-chart-1)\""));
    }

    #[test]
    fn categories_cycle_through_six_color_slots() {
        let data = ChartData::new(
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
                "e".to_string(),
                "f".to_string(),
                "g".to_string(),
            ],
            vec![Series::new("s", vec![1.0; 7])],
        )
        .unwrap();
        let html = render(&root(&data, "s").unwrap());
        assert!(html.contains("chart-1"));
        assert!(html.contains("chart-6"));
    }

    #[test]
    fn root_is_deterministic() {
        let a = render(&root(&sample(), "visits").unwrap());
        let b = render(&root(&sample(), "visits").unwrap());
        assert_eq!(a, b);
    }

    #[test]
    fn category_labels_are_escaped() {
        let data = ChartData::new(
            vec!["<script>alert(1)</script>".to_string()],
            vec![Series::new("s", vec![1.0])],
        )
        .unwrap();
        let html = render(&root(&data, "s").unwrap());
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn css_is_deterministic_and_has_no_breakout_sequences() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(!a.contains('<'));
        assert!(a.contains(r#"[data-scope="bar-segment"]"#));
    }
}
