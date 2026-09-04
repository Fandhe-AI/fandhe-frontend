//! Sparkline（イシュー #848、親 Phase #845）: [`crate::area_chart`] の単一
//! 系列専用の縮小版。ラベル・軸なしの小さな「面 + 線」チャートとして、単一の
//! 数値列 `&[f64]` から直接描画する。
//!
//! chakra-ui `charts/sparkline.md` の `w={28} h={12}`（Chakra size token、
//! `28 * 4px = 112px` / `12 * 4px = 48px` 相当）を既定 `viewBox` 寸法として
//! 採用する。座標写像・path 生成・数値文字列化・x/y 軸の写像規則・エッジ
//! ケース（`n == 1`・負値・フラット）は [`crate::line_chart`]/
//! [`crate::area_chart`] モジュール doc と同一の方針に従う
//! （[`crate::line_chart::category_x`]/[`crate::line_chart::view_box_from_dims`]
//! を共有ヘルパとして再利用する）。
//!
//! # `ChartData` への内部変換
//!
//! 公開 API は `values: &[f64]` のみを受け取る（chakra `Sparkline` も
//! `data`/`dataKey` のみのシンプルな入力形）。内部で
//! `ChartData::new(合成カテゴリ, [Series])` を経由して構築し、空データ・
//! 非有限値混入を [`ChartError`] として fail-closed に拒否する
//! （[`ChartData::new`](crate::charts::data::ChartData::new) の不変条件を
//! 素通しする。合成カテゴリはインデックス文字列 `"0"`, `"1"`, ... とし、
//! 表示には使わない内部専用の値であるため既定エスケープ経由でも安全）。
//!
//! # セキュリティ不変条件
//!
//! [`crate::line_chart`] モジュール doc と同一。
//!
//! # 本イシューのスコープ外
//!
//! 複数系列・軸/グリッド/凡例/ツールチップは提供しない（Sparkline はラベル
//! なしの縮小表示という定義上のスコープ、chakra 本家も単一系列専用）。
//! `examples/headless-pre-styled-ui` への追随は crates.io 公開後
//! （[`crate::line_chart`] と同じ判断）。
//!
//! # 参考サイト基準への調整（イシュー #1599）
//!
//! 親 Phase #1588「Themes / Charts のスタイル調整」の子。参照 4 サイト
//! （chakra-ui / Ark UI / Radix Primitives / Radix Themes）に Sparkline
//! 相当のチャート部品が存在しないため、評価軸は**内部整合のみ**
//! （`--fandhe-*` トークン適用・ダーク時の可読性・系列色の識別性・
//! コントラスト）に限定する（兄弟部品 [`crate::area_chart`]（#1589）/
//! [`crate::line_chart`]（#1595）と同じ判断）。
//!
//! | 軸 | 結論 |
//! |---|---|
//! | サイズ | 現状維持（Xs〜Xl は #1681 で整備済み） |
//! | バリアント / colorPalette | 非採用（参照軸なし。系列色は `chart-1` 固定） |
//! | 色 | 現状維持（全宣言がトークン経由。生の色リテラルなし） |
//! | 状態 `data-*` | 非該当（headless 由来の `data-*` を持たない pre-styled-only 部品） |
//! | ダークモード | 追加規則なし（`chart-1`・`--fandhe-color-bg` は dark 値定義済み） |
//! | フォーカス | 非該当（`svg` は `role="img"` でフォーカス不可） |
//! | 余白・角丸・影 | 非該当（面・線のみの SVG 描画） |
//! | hover / disabled / トランジション | 非採用（表示専用部品、状態遷移なし） |
//! | 内部整合（実欠陥） | **是正**（下記「是正した点」） |
//!
//! ## 是正した点
//!
//! - `plot` slot に `overflow: visible` を追加し、domain の max/min に接する
//!   折れ線が UA 既定 `overflow: hidden` で viewBox 上下端において
//!   `stroke-width: 1.5` の半分をクリップされる欠陥を、ジオメトリを変えず
//!   CSS のみで是正した（sparkline は高さが最小 16px（Xs）と小さく、この
//!   欠陥が相対的に最も目立つ。先例: [`crate::area_chart`] #1589 /
//!   [`crate::line_chart`] #1595）
//! - `series-line` slot に `stroke-linejoin: round` / `stroke-linecap: round`
//!   を追加し、折れ線の鋭角部での miter 突出を抑えた（先例: 上記 2 部品）
//! - `point` slot（`n == 1` 時の点マーカー）に背景色のハロー
//!   （`stroke: var(--fandhe-color-bg)`）を追加した。`n == 1` では
//!   `series-area` は描かれず circle 単体になるため「面と同色」の問題では
//!   なく、単独マーカーがページ背景・隣接インライン内容に対して輪郭を
//!   持たない点と、兄弟部品（背景色ハロー付与済み）との `point` 規則の
//!   整合が目的
//!
//! ## 意図的に合わせなかった点
//!
//! - `series-line` への `vector-effect: non-scaling-stroke` は、sparkline が
//!   viewBox 高さ 48 を 16px（Xs）へ縮小すると `stroke-width: 1.5` が
//!   約 0.5px 相当になり、5 部品中で最もこのトレードオフが大きい。それでも
//!   area-chart / line-chart（#1593 が非採用のまま完了）との線幅の見え方の
//!   整合を優先し非採用とする
//! - `overflow: visible` × `display: inline-block` root: `plot` が
//!   `width: auto`（viewBox のアスペクト比から算出）であるため、ストローク
//!   がレイアウトボックス外へ stroke 幅の半分 × size 比率分（Xs で約 0.25px、
//!   Md で約 0.75px、Xl で約 1.25px）はみ出しインライン隣接要素に重なり
//!   得る。実害はないが明記しておく
//! - root への `vertical-align` 追加は検討したが不採用。`plot` が
//!   `display: block` のため inline-block の baseline は下端に一致しており、
//!   既存利用者のインラインレイアウトを変えない
//! - `view_box_from_dims` / `category_x`（[`crate::line_chart`] と共有する
//!   ヘルパ）へのパディング追加は CSS の `overflow: visible` で足りるため
//!   不要（#1595 が本 issue に譲った論点をここで「不要」として閉じる）
//! - `fill-opacity` のトークン化は `theme.rs` 変更が docs-site 契約テストへ
//!   波及するため見送った（兄弟部品と同じ判断）

use crate::charts::data::{ChartData, Series};
use crate::charts::scale::LinearScale;
use crate::charts::svg::{fmt_coord, svg_root, PathBuilder};
use crate::charts::{series_color_var, ChartError};
use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::line_chart::{category_x, view_box_from_dims};
use crate::recipe::{Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, Node};
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="sparkline"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("sparkline");

/// [`SlotRecipe::new`] に渡す slot 一覧（[`crate::area_chart`] と同じ
/// エッジケース規則により `point` を含める）。
const SLOTS: &[&str] = &["root", "plot", "series-area", "series-line", "point"];

/// `viewBox` 幅の既定値（chakra `w={28}` トークン相当、`28 * 4px`）。
pub const DEFAULT_WIDTH: f64 = 112.0;
/// `viewBox` 高さの既定値（chakra `h={12}` トークン相当、`12 * 4px`）。
pub const DEFAULT_HEIGHT: f64 = 48.0;

/// 単一カテゴリ時に描く点マーカーの半径。
const POINT_RADIUS: f64 = 2.5;

/// 内部合成 [`ChartData`] における唯一の系列名（表示には使わない、
/// モジュール doc「`ChartData` への内部変換」参照）。
const SERIES_NAME: &str = "value";

/// [`sparkline`] の入力。
pub struct SparklineProps<'a> {
    /// 単一系列の値列。
    pub values: &'a [f64],
    /// `svg` 要素へ付与する `aria-label`（必須）。
    pub aria_label: &'a str,
    /// `viewBox` 幅。
    pub width: f64,
    /// `viewBox` 高さ。
    pub height: f64,
    /// root へ付与する寸法 variant。
    pub size: Size,
}

impl<'a> SparklineProps<'a> {
    /// 既定寸法（`DEFAULT_WIDTH`/`DEFAULT_HEIGHT`・[`Size::Md`]）で組み立てる。
    #[must_use]
    pub fn new(values: &'a [f64], aria_label: &'a str) -> Self {
        SparklineProps {
            values,
            aria_label,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            size: Size::Md,
        }
    }
}

/// この styled Sparkline の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("sparkline", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-block"),
                decl("--fandhe-sparkline-height", "48px"),
            ],
        )
        .base(
            "plot",
            vec![
                decl("display", "block"),
                decl("width", "auto"),
                decl("height", "var(--fandhe-sparkline-height, auto)"),
                // イシュー #1599: SVG 非ルート要素は UA 既定で `overflow: hidden`
                // となるため、domain の max/min に接する折れ線
                // （`stroke-width: 1.5`）が viewBox 上下端で半分クリップされる。
                // ジオメトリ（`view_box_from_dims`/`category_x`）は変えず、
                // CSS のみで表示上のクリップを解除する
                // （先例: area_chart #1589 / line_chart #1595）。
                decl("overflow", "visible"),
            ],
        )
        .base(
            "series-area",
            vec![decl("fill-opacity", "0.2"), decl("stroke", "none")],
        )
        .base(
            "series-line",
            vec![
                decl("fill", "none"),
                decl("stroke-width", "1.5"),
                // イシュー #1599: 先例 line_chart #1595 / area_chart #1589。
                // 折れ線の鋭角部での miter 突出を抑え、端点の見た目を整える。
                decl("stroke-linejoin", "round"),
                decl("stroke-linecap", "round"),
            ],
        )
        .base(
            "point",
            // イシュー #1599: `n == 1` 時は `series-area` が描かれず circle
            // 単体になるため、ページ背景・隣接インライン内容に対する輪郭を
            // 背景色のハローで付与し、兄弟部品（line_chart/area_chart）との
            // `point` 規則の整合を保つ。`--fandhe-color-bg` はダーク時の値へ
            // トークン経由で自動追随する。
            vec![
                decl("stroke", "var(--fandhe-color-bg)"),
                decl("stroke-width", "1"),
            ],
        )
        // イシュー #1681: Xs/Xl は Sm→Md→Lg の 16px 刻み等差進行を外挿。
        .variant(
            Size::Xs,
            "root",
            vec![decl("--fandhe-sparkline-height", "16px")],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl("--fandhe-sparkline-height", "32px")],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl("--fandhe-sparkline-height", "48px")],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl("--fandhe-sparkline-height", "64px")],
        )
        .variant(
            Size::Xl,
            "root",
            vec![decl("--fandhe-sparkline-height", "80px")],
        )
        .default_variant(Size::Md)
}

/// この styled Sparkline が生成する静的 CSS 全量を返す（決定的）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// `values` から内部専用の合成 [`ChartData`] を構築する（内部ヘルパ）。
///
/// # Errors
///
/// `values` が空の場合 [`ChartError::EmptyData`]、非有限値を含む場合
/// [`ChartError::NonFiniteValue`]（[`ChartData::new`] の不変条件をそのまま
/// 素通しする）。
fn to_chart_data(values: &[f64]) -> Result<ChartData, ChartError> {
    let categories = (0..values.len()).map(|i| i.to_string()).collect();
    ChartData::new(categories, vec![Series::new(SERIES_NAME, values.to_vec())])
}

/// 単一系列を「面 + 線」（`n >= 2`）または中央の点マーカー（`n == 1`）として
/// 描く（内部ヘルパ）。`baseline_y` は [`crate::area_chart`] と同じ規則
/// （domain 下端の写像座標）。
fn render_series(width: f64, y_scale: &LinearScale, baseline_y: f64, values: &[f64]) -> Vec<Node> {
    let n = values.len();
    let color = series_color_var(0);

    if n <= 1 {
        let x = category_x(width, n, 0);
        let y = values.first().copied().map_or(0.0, |v| y_scale.scale(v));
        let (cx, cy, r) = (fmt_coord(x), fmt_coord(y), fmt_coord(POINT_RADIUS));
        return vec![el(
            "circle",
            vec![
                ("data-scope", "sparkline"),
                ("data-part", "point"),
                ("cx", cx.as_str()),
                ("cy", cy.as_str()),
                ("r", r.as_str()),
                ("fill", color.as_str()),
            ],
            vec![],
        )];
    }

    let points: Vec<(f64, f64)> = values
        .iter()
        .enumerate()
        .map(|(i, &v)| (category_x(width, n, i), y_scale.scale(v)))
        .collect();

    let mut area_builder = PathBuilder::new();
    for (i, &(x, y)) in points.iter().enumerate() {
        area_builder = if i == 0 {
            area_builder.move_to(x, y)
        } else {
            area_builder.line_to(x, y)
        };
    }
    let (last_x, _) = points[points.len() - 1];
    let (first_x, _) = points[0];
    let area_d = area_builder
        .line_to(last_x, baseline_y)
        .line_to(first_x, baseline_y)
        .close()
        .build();

    let mut line_builder = PathBuilder::new();
    for (i, &(x, y)) in points.iter().enumerate() {
        line_builder = if i == 0 {
            line_builder.move_to(x, y)
        } else {
            line_builder.line_to(x, y)
        };
    }
    let line_d = line_builder.build();

    vec![
        el(
            "path",
            vec![
                ("data-scope", "sparkline"),
                ("data-part", "series-area"),
                ("d", area_d.as_str()),
                ("fill", color.as_str()),
            ],
            vec![],
        ),
        el(
            "path",
            vec![
                ("data-scope", "sparkline"),
                ("data-part", "series-line"),
                ("d", line_d.as_str()),
                ("stroke", color.as_str()),
                ("fill", "none"),
            ],
            vec![],
        ),
    ]
}

/// Sparkline 本体を組み立てる。
///
/// # Errors
///
/// - `props.values` が空の場合 [`ChartError::EmptyData`]
/// - 非有限値を含む場合 [`ChartError::NonFiniteValue`]
/// - `props.width`/`props.height` が非有限の場合 [`ChartError::NonFiniteValue`]、
///   0 以下の場合 [`ChartError::DegenerateDomain`]
///   （[`crate::line_chart::view_box_from_dims`] 参照）
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::sparkline::{sparkline, SparklineProps};
///
/// let values = [10.0, 30.0, 20.0, 40.0];
/// let node = sparkline(&SparklineProps::new(&values, "weekly trend"), vec![]).unwrap();
/// let html = render(&node);
/// assert!(html.contains(r#"data-scope="sparkline" data-part="root""#));
/// ```
pub fn sparkline<'a>(
    props: &SparklineProps<'a>,
    attrs: Vec<(&'a str, &'a str)>,
) -> Result<Node, ChartError> {
    let data = to_chart_data(props.values)?;
    let view_box = view_box_from_dims(props.width, props.height)?;
    let y_scale = LinearScale::new(data.domain(), (props.height, 0.0))?;
    let (dom_lo, _dom_hi) = data.domain();
    let baseline_y = y_scale.scale(dom_lo);

    let plot_children = render_series(props.width, &y_scale, baseline_y, props.values);

    let plot = svg_root(
        &view_box,
        vec![
            ("data-scope", "sparkline"),
            ("data-part", "plot"),
            ("aria-label", props.aria_label),
        ],
        plot_children,
    );

    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", props.size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));
    Ok(ANATOMY.part("root", "div", merged, vec![plot]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn renders_root_and_plot_with_aria_label() {
        let values = [1.0, 2.0, 3.0];
        let node = sparkline(&SparklineProps::new(&values, "sample"), vec![]).unwrap();
        let html = render(&node);
        assert!(html.starts_with(r#"<div data-scope="sparkline" data-part="root""#));
        assert!(html.contains(r#"data-scope="sparkline" data-part="plot""#));
        assert!(html.contains(r#"role="img""#));
        assert!(html.contains(r#"aria-label="sample""#));
    }

    #[test]
    fn multi_value_renders_area_and_line_paths() {
        let values = [1.0, 5.0, 2.0];
        let node = sparkline(&SparklineProps::new(&values, "multi"), vec![]).unwrap();
        let html = render(&node);
        assert!(html.contains(r#"data-part="series-area""#));
        assert!(html.contains(r#"data-part="series-line""#));
    }

    #[test]
    fn single_value_renders_point_not_path() {
        let values = [5.0];
        let node = sparkline(&SparklineProps::new(&values, "single"), vec![]).unwrap();
        let html = render(&node);
        assert!(html.contains(r#"data-part="point""#));
        assert!(!html.contains("<path"));
    }

    #[test]
    fn empty_values_is_rejected() {
        let values: [f64; 0] = [];
        assert_eq!(
            sparkline(&SparklineProps::new(&values, "empty"), vec![]).unwrap_err(),
            ChartError::EmptyData
        );
    }

    #[test]
    fn non_finite_value_is_rejected() {
        let values = [1.0, f64::NAN, 2.0];
        assert_eq!(
            sparkline(&SparklineProps::new(&values, "nan"), vec![]).unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn negative_values_stay_deterministic() {
        let values = [-1.0, -5.0, -2.0];
        let a = render(&sparkline(&SparklineProps::new(&values, "neg"), vec![]).unwrap());
        let b = render(&sparkline(&SparklineProps::new(&values, "neg"), vec![]).unwrap());
        assert_eq!(a, b);
        assert!(a.contains('-'));
    }

    #[test]
    fn flat_values_render_deterministically() {
        let values = [5.0, 5.0, 5.0];
        let a = render(&sparkline(&SparklineProps::new(&values, "flat"), vec![]).unwrap());
        let b = render(&sparkline(&SparklineProps::new(&values, "flat"), vec![]).unwrap());
        assert_eq!(a, b);
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let values = [1.0, 2.0];
        let mut props = SparklineProps::new(&values, "class-test");
        props.size = Size::Lg;
        let html = render(&sparkline(&props, vec![("class", "attacker")]).unwrap());
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker"));
        assert!(html.contains("fd-sparkline--size-lg"));
    }

    #[test]
    fn xss_payload_in_aria_label_is_escaped() {
        let values = [1.0, 2.0];
        let payload = "\"><img src=x onerror=alert(1)>";
        let html = render(&sparkline(&SparklineProps::new(&values, payload), vec![]).unwrap());
        assert!(!html.contains("<img"));
        assert!(html.contains("&lt;img"));
    }

    #[test]
    fn xss_payload_in_caller_attrs_is_escaped() {
        let values = [1.0, 2.0];
        let payload = "\"><script>alert(1)</script>";
        let html = render(
            &sparkline(
                &SparklineProps::new(&values, "attrs"),
                vec![("data-testid", payload)],
            )
            .unwrap(),
        );
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn stylesheet_is_deterministic_and_targets_expected_selectors() {
        let a = stylesheet();
        let b = stylesheet();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="sparkline"][data-part="series-area"]"#));
        assert!(a.contains(r#"[data-scope="sparkline"][data-part="plot"]"#));
        assert!(a.contains("overflow: visible"));
        assert!(a.contains(r#"[data-scope="sparkline"][data-part="series-line"]"#));
        assert!(a.contains("stroke-linejoin: round"));
        assert!(a.contains("stroke-linecap: round"));
        assert!(a.contains(r#"[data-scope="sparkline"][data-part="point"]"#));
        assert!(a.contains("stroke: var(--fandhe-color-bg)"));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }
}
