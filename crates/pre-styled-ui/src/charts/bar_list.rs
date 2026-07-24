//! BarList（ランキング型バーリスト、イシュー #849・親 Phase #845）。
//!
//! chakra-ui `charts/bar-list.md` 相当を HTML（`<div>` ベース）で再構成する。
//! [`super::data::ChartData`] の 1 系列を対象に、各カテゴリの値を「その系列の
//! 最大値に対する比率」で幅を決めた横棒として並べる（recharts を使わず、
//! HTML/CSS の `width` のみで表現できるため SVG 層を経由しない。
//! [`crate::charts`] モジュール doc「本モジュールはそれらの上位部品を持たない」
//! の通り、本モジュールは新規 anatomy `data-scope="bar-list"` を定義する
//! （`fandhe-frontend-headless-ui` 側に対応する anatomy はない、
//! `crates/pre-styled-ui/src/table.rs` と同型の判断）。
//!
//! # 比率の伝搬（インライン custom property）
//!
//! バー幅は `value / max * 100` を [`super::svg::fmt_coord`] で文字列化し、
//! `style="--fandhe-bar-list-percent: <n>%"` としてインライン伝搬する
//! （`crate::slider` の `--fandhe-slider-percent` 方式と同型）。CSS 側は
//! `width: var(--fandhe-bar-list-percent)` を `bar` slot の base 宣言に持つ。
//!
//! # fail-closed（`.claude/rules/security.md` A04 対応）
//!
//! - 対象系列が存在しない場合 [`ChartError::UnknownSeriesName`]。
//! - 系列中に負値が 1 件でもあれば [`ChartError::NegativeValue`]（比率
//!   `value / max` が負・100 超になり得て意味を持たないため構築時に拒否する）。
//! - 系列の最大値が 0（全値 0）の場合は比率が定義できないため、全バー幅を
//!   決定的に 0% として描画する（`silent failure` ではなく「値が 0 なら幅も
//!   0」という利用者にとって自明な対応関係であり、[`crate::charts::bar_segment`]
//!   の合計 0 拒否（構成比自体が無意味）とは性質が異なると判断した、rustdoc
//!   に明記）。
//!
//! # セキュリティ不変条件
//!
//! マークアップはすべて [`fandhe_frontend_headless_ui::fandhe_frontend_core::el`]/
//! `text` 経由（`raw_html()` 不使用、REQ-1）。値ラベルの文字列化は
//! [`super::svg::fmt_coord`] にのみ一元化する。インライン `style` 属性値は
//! `--fandhe-bar-list-percent: <fmt_coord 出力>%` の固定テンプレートのみで
//! 構成し、任意文字列の混入経路を持たない。
//!
//! # 本イシューのスコープ外
//!
//! - ソート（呼び出し側が [`super::data::ChartData::sort_by_series`] を
//!   事前に呼ぶ想定。本モジュールは並び順を変更しない）。
//! - `examples/headless-pre-styled-ui` への追随は crates.io 公開後に別途。

use super::data::{self, ChartData};
use super::svg::fmt_coord;
use super::ChartError;
use crate::css::decl;
use crate::recipe::SlotRecipe;
use fandhe_frontend_headless_ui::fandhe_frontend_core::{text, Node};
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="bar-list"` を固定した anatomy。
const ANATOMY: Anatomy = anatomy("bar-list");

/// [`SlotRecipe::new`] に渡す slot 一覧。
const SLOTS: &[&str] = &["root", "item", "label", "track", "bar", "value"];

/// この BarList の既定 CSS を組み立てる（内部ヘルパ、[`css`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("bar-list", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                decl("gap", "0.5rem"),
                decl("width", "100%"),
            ],
        )
        .base(
            "item",
            vec![
                decl("display", "grid"),
                decl("grid-template-columns", "minmax(0, 1fr) auto"),
                decl("align-items", "center"),
                decl("gap", "0.75rem"),
            ],
        )
        .base(
            "label",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("overflow", "hidden"),
                decl("text-overflow", "ellipsis"),
                decl("white-space", "nowrap"),
            ],
        )
        .base(
            "track",
            vec![
                decl("grid-column", "1 / -1"),
                decl("background", "var(--fandhe-color-bg-subtle)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                decl("overflow", "hidden"),
                decl("height", "0.5rem"),
            ],
        )
        .base(
            "bar",
            vec![
                decl("height", "100%"),
                decl("width", "var(--fandhe-bar-list-percent, 0%)"),
                decl("background", "var(--fandhe-color-chart-1)"),
            ],
        )
        .base(
            "value",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
                decl("font-variant-numeric", "tabular-nums"),
                decl("color", "var(--fandhe-color-fg-muted)"),
            ],
        )
}

/// この BarList が生成する静的 CSS 全量を返す（決定的）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// BarList 本体を組み立てる。`data` から `series_name` の系列を取り出し、
/// カテゴリ順（[`ChartData::categories`]）にランキング行を生成する。
///
/// # Errors
///
/// - `series_name` に一致する系列がない場合 [`ChartError::UnknownSeriesName`]
/// - 系列中に負値が含まれる場合 [`ChartError::NegativeValue`]
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::charts::bar_list::root;
/// use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
///
/// let data = ChartData::new(
///     vec!["a".to_string(), "b".to_string()],
///     vec![Series::new("visits", vec![10.0, 30.0])],
/// )
/// .unwrap();
/// let node = root(&data, "visits").unwrap();
/// assert!(render(&node).contains(r#"data-scope="bar-list" data-part="root""#));
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

    let max = data::max(series).unwrap_or(0.0);
    let items: Vec<Node> = data
        .categories()
        .iter()
        .zip(series.values.iter())
        .map(|(category, &value)| item(category, value, max))
        .collect();

    Ok(ANATOMY.part("root", "div", vec![], items))
}

/// 1 行（`item`/`label`/`track`/`bar`/`value`）を組み立てる（内部ヘルパ）。
fn item(category: &str, value: f64, max: f64) -> Node {
    let percent = if max == 0.0 { 0.0 } else { value / max * 100.0 };
    let percent = percent.clamp(0.0, 100.0);
    let style = format!("--fandhe-bar-list-percent: {}%", fmt_coord(percent));
    let value_text = fmt_coord(value);

    ANATOMY.part(
        "item",
        "div",
        vec![],
        vec![
            ANATOMY.part("label", "span", vec![], vec![text(category.to_string())]),
            ANATOMY.part(
                "track",
                "div",
                vec![],
                vec![ANATOMY.part("bar", "div", vec![("style", style.as_str())], vec![])],
            ),
            ANATOMY.part("value", "span", vec![], vec![text(value_text)]),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::data::Series;
    use fandhe_frontend_core::render;

    fn sample() -> ChartData {
        ChartData::new(
            vec!["Jan".to_string(), "Feb".to_string(), "Mar".to_string()],
            vec![Series::new("visits", vec![10.0, 30.0, 20.0])],
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
    fn root_computes_percent_relative_to_max() {
        let html = render(&root(&sample(), "visits").unwrap());
        // max = 30 -> Jan: 10/30*100 = 33.33..%(丸め 2 桁 -> 33.33), Feb: 100%.
        assert!(html.contains("--fandhe-bar-list-percent: 33.33%"));
        assert!(html.contains("--fandhe-bar-list-percent: 100%"));
    }

    #[test]
    fn root_all_zero_values_render_zero_percent_bars() {
        let data = ChartData::new(
            vec!["a".to_string(), "b".to_string()],
            vec![Series::new("z", vec![0.0, 0.0])],
        )
        .unwrap();
        let html = render(&root(&data, "z").unwrap());
        assert_eq!(html.matches("--fandhe-bar-list-percent: 0%").count(), 2);
    }

    #[test]
    fn root_preserves_category_order() {
        let html = render(&root(&sample(), "visits").unwrap());
        let jan_pos = html.find("Jan").unwrap();
        let feb_pos = html.find("Feb").unwrap();
        let mar_pos = html.find("Mar").unwrap();
        assert!(jan_pos < feb_pos && feb_pos < mar_pos);
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
    fn parts_use_expected_data_scope_and_part() {
        let html = render(&root(&sample(), "visits").unwrap());
        assert!(html.contains(r#"data-scope="bar-list" data-part="root""#));
        assert!(html.contains(r#"data-part="item""#));
        assert!(html.contains(r#"data-part="label""#));
        assert!(html.contains(r#"data-part="track""#));
        assert!(html.contains(r#"data-part="bar""#));
        assert!(html.contains(r#"data-part="value""#));
    }

    #[test]
    fn css_is_deterministic_and_has_no_breakout_sequences() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(!a.contains('<'));
        assert!(a.contains(r#"[data-scope="bar-list"]"#));
    }
}
