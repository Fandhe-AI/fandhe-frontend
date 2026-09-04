//! LineChart（イシュー #848、親 Phase #845）: `charts` 基盤（#846）の最初の
//! 消費者。系列ごとの折れ線を SVG ノード木として描画する自己完結部品。
//!
//! chakra-ui `charts/line-chart.md` は recharts（`<LineChart><CartesianGrid/>
//! <XAxis/>...<Line/></LineChart>` の合成）に依存するが、本フレームワークは
//! 外部 JS ランタイムを持たないため、軸・グリッド・凡例・ツールチップ
//! （chakra の `CartesianGrid`/`XAxis`/`YAxis`/`ChartLegend`/`ChartTooltip`
//! 相当）は並行イシュー **#847 のスコープ**とし、本モジュールは「プロット
//! 領域（折れ線のみ）を描く」ことに責務を限定する。#847 の軸/グリッド部品と
//! 合成する場合、呼び出し側が [`svg_root`](crate::charts::svg::svg_root) の
//! children として本モジュールの [`plot`] 出力と #847 の軸要素を並べる想定
//! （統合ポイント、chakra は 1 コンポーネント内で JSX 合成するが、本実装は
//! 呼び出し側の明示的な組み立てに委ねる。REQ-5 のマクロ DSL 非採用方針と
//! 整合）。
//!
//! # 座標写像・数値文字列化の一元化
//!
//! 座標写像は [`crate::charts::scale::LinearScale`]、`path` の `d` 属性は
//! [`crate::charts::svg::PathBuilder`]、数値の決定的文字列化は
//! [`crate::charts::svg::fmt_coord`]（`PathBuilder`/`circle`/`svg_root` 内部
//! 経由）のみを通す。本モジュール自身は独自の数値フォーマット・座標計算式を
//! 実装しない（`.claude/rules/coding-rust.md` 決定性の一元化）。
//!
//! # x/y 軸の写像規則
//!
//! - x 軸: カテゴリ index `i`（`0..n`）を [`category_x`] で等間隔配置する。
//!   `n == 1`（単一カテゴリ）は `i * width / (n - 1)` が 0 除算になるため
//!   特別扱いし、`width / 2.0` の中央 1 点のみへ配置する。
//! - y 軸: [`ChartData::domain`](crate::charts::data::ChartData::domain)
//!   （フラットデータの非退化パディング込み）を
//!   `LinearScale::new(domain, (height, 0.0))`（SVG の y 下向き正のため range
//!   を反転）で写像する。`nice()` は適用しない（本モジュールは軸を持たず、
//!   domain 拡張は無意味であり、#847 の軸合成時に呼び出し側が選ぶ余地を残す
//!   ため）。
//!
//! # エッジケース（golden テスト対象、`tests/charts_line_area_sparkline.rs`）
//!
//! | 入力 | 挙動 |
//! |------|------|
//! | 単一カテゴリ（`n == 1`） | `path` を生成せず、中央に [`point`] マーカーのみ描く |
//! | 負値・負のみの系列 | domain がそのまま負域を含み、`fmt_coord` の `-` 付き座標で決定的出力 |
//! | フラット（全値同値） | `ChartData::domain` のパディングにより中央水平線 |
//!
//! # セキュリティ不変条件
//!
//! マークアップはすべて [`fandhe_frontend_headless_ui::fandhe_frontend_core::el`]/
//! [`ANATOMY`] 経由のノード木 API のみで組み立て、`raw_html()`・SVG/HTML
//! 文字列の直接組み立ては一切行わない（REQ-1）。`aria_label`・呼び出し側
//! `attrs`（`data-testid` 等）は `fandhe_frontend_core::render` の既定
//! エスケープを必ず経由する。`path`/`circle` の座標・寸法は [`fmt_coord`]
//! （文字集合 `[0-9.-]` に閉じる）のみを経由し、任意文字列混入経路を持たない。
//! CSS 宣言値はすべてコンパイル時静的リテラルであり、動的値を CSS へ流し込む
//! 経路はない。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - 軸・グリッド・凡例・ツールチップ・曲線補間（`curveType`）・積み上げは
//!   #847 以降。
//! - `examples/headless-pre-styled-ui` への追随は crates.io 公開後に別途
//!   行う（[`crate::qr_code`] の先例と同じ判断）。
//!
//! # 参考サイト基準への調整（イシュー #1595）
//!
//! 親 Phase #1588「Themes / Charts のスタイル調整」の子。参照 4 サイト
//! （chakra-ui / Ark UI / Radix Primitives / Radix Themes）にチャート部品が
//! 存在しないため、評価軸は**内部整合のみ**（`--fandhe-*` トークン適用・
//! ダーク時の可読性・系列色の識別性・ラベルのコントラスト）に限定する
//! （兄弟部品 [`crate::area_chart`]（#1589）と同じ判断）。
//!
//! | 軸 | 結論 |
//! |---|---|
//! | サイズ | 現状維持（Xs〜Xl は #1681 で整備済み） |
//! | バリアント / colorPalette | 非採用（参照軸なし。系列色は `chart-1〜6` 固定ローテーション） |
//! | 色 | 現状維持（全宣言がトークン経由。生の色リテラルなし） |
//! | 状態 `data-*` | 非該当（headless 由来の `data-*` を持たない pre-styled-only 部品） |
//! | ダークモード | 追加規則なし（系列色・`--fandhe-color-bg` は dark 値定義済み） |
//! | フォーカス | 非該当（`svg` は `role="img"` でフォーカス不可） |
//! | 余白・角丸・影 | 非該当（線のみの SVG 描画） |
//! | hover / disabled / トランジション | 非採用（表示専用部品、状態遷移なし） |
//! | 内部整合（実欠陥） | **是正**（下記「是正した点」） |
//!
//! ## 是正した点
//!
//! - `plot` slot に `overflow: visible` を追加し、domain の max/min に接する
//!   折れ線が UA 既定 `overflow: hidden` で viewBox 上下端において
//!   `stroke-width: 2` の半分をクリップされる欠陥を、ジオメトリを変えず
//!   CSS のみで是正した（先例: [`crate::area_chart`] #1589）
//! - `series-line` slot に `stroke-linejoin: round` / `stroke-linecap: round`
//!   を追加し、折れ線の鋭角部での miter 突出を抑えた
//!   （先例: [`crate::signature_pad`] / [`crate::progress`] / [`crate::area_chart`]）
//! - `point` slot（`n == 1` 時の点マーカー）に背景色のハロー
//!   （`stroke: var(--fandhe-color-bg)`）を追加し、背景・隣接系列色との
//!   識別性を高めた
//!
//! ## 意図的に合わせなかった点
//!
//! - `series-line` への `vector-effect: non-scaling-stroke` は、#1863（area-chart）
//!   が「必要なら #1593 で横断的に」と先送りしたが #1593 は非採用のまま完了
//!   したため、area-chart / sparkline との線幅の見え方の整合を保つべく
//!   本 PR でも非採用とする
//! - `view_box_from_dims` / `category_x`（[`crate::area_chart`]/[`crate::sparkline`]
//!   と共有するヘルパ）へのパディング追加は #1599（sparkline）と競合する
//!   ため見送った
//! - `theme.rs` への opacity 等トークン新設は、消費者が chart 系のみで
//!   契約テストへ波及するため見送った

use crate::charts::data::ChartData;
use crate::charts::scale::LinearScale;
use crate::charts::svg::{fmt_coord, svg_root, PathBuilder, ViewBox, ViewBoxError};
use crate::charts::{series_color_var, ChartError};
use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, Node};
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="line-chart"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("line-chart");

/// [`SlotRecipe::new`] に渡す slot 一覧。
const SLOTS: &[&str] = &["root", "plot", "series-line", "point"];

/// `viewBox` 幅の既定値（chakra `charts/line-chart.md` の代表例に近い横長比率）。
pub const DEFAULT_WIDTH: f64 = 300.0;
/// `viewBox` 高さの既定値。
pub const DEFAULT_HEIGHT: f64 = 150.0;

/// 単一カテゴリ（`n == 1`）時に描く点マーカーの半径（`viewBox` 座標系）。
const POINT_RADIUS: f64 = 2.5;

/// [`line_chart`] の入力。
pub struct LineChartProps<'a> {
    /// 描画するチャートデータ（[`ChartData::new`](crate::charts::data::ChartData::new)
    /// を経由して構築済みであることが不変条件。空データ・非有限値混入は
    /// 構築時に拒否済みのため本関数へは到達しない）。
    pub data: &'a ChartData,
    /// `svg` 要素へ付与する `aria-label`（必須。装飾ではなくデータ可視化のため
    /// スクリーンリーダー向け説明を必須とする、`.claude/rules/security.md`
    /// 以前に a11y 上の必須要件）。
    pub aria_label: &'a str,
    /// `viewBox` 幅（描画座標系。CSS 表示寸法は [`Size`] variant が別途制御する）。
    pub width: f64,
    /// `viewBox` 高さ。
    pub height: f64,
    /// root へ付与する寸法 variant（svg の CSS 表示高さを切替える、
    /// [`crate::qr_code`] と同型）。
    pub size: Size,
}

impl<'a> LineChartProps<'a> {
    /// 既定寸法（`DEFAULT_WIDTH`/`DEFAULT_HEIGHT`・[`Size::Md`]）で組み立てる。
    #[must_use]
    pub fn new(data: &'a ChartData, aria_label: &'a str) -> Self {
        LineChartProps {
            data,
            aria_label,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            size: Size::Md,
        }
    }
}

/// `width`/`height` から `viewBox` を構築する（内部ヘルパ、[`crate::area_chart`]/
/// [`crate::sparkline`] も共有する）。
///
/// [`ViewBoxError`] を [`ChartError`] へ写像する: `NonFinite` →
/// [`ChartError::NonFiniteValue`]、`NonPositiveSize`（幅/高さ 0 以下）→
/// [`ChartError::DegenerateDomain`]（描画領域が退化しているという意味的な
/// 近さから、`charts` 基盤 #846 が定義する既存バリアントを転用する判断。
/// 本クレートは基盤モジュールへ新規バリアントを追加しない）。
pub(crate) fn view_box_from_dims(width: f64, height: f64) -> Result<ViewBox, ChartError> {
    ViewBox::new(0.0, 0.0, width, height).map_err(|e| match e {
        ViewBoxError::NonFinite => ChartError::NonFiniteValue,
        ViewBoxError::NonPositiveSize => ChartError::DegenerateDomain,
    })
}

/// カテゴリ index `i`（`0..n`）を x 座標へ等間隔で写像する（内部ヘルパ、
/// [`crate::area_chart`]/[`crate::sparkline`] も共有する）。
///
/// `n <= 1` の場合は `i * width / (n - 1)` が 0 除算になるため、描画領域の
/// 中央 1 点（`width / 2.0`）へ配置する特別扱いをする（モジュール doc
/// 「x/y 軸の写像規則」参照）。
pub(crate) fn category_x(width: f64, n: usize, i: usize) -> f64 {
    if n <= 1 {
        width / 2.0
    } else {
        (i as f64) * width / ((n - 1) as f64)
    }
}

/// この styled LineChart の既定 CSS を組み立てる（内部ヘルパ、[`stylesheet`]
/// のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("line-chart", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "block"),
                decl("--fandhe-line-chart-height", "150px"),
            ],
        )
        .base(
            "plot",
            vec![
                decl("display", "block"),
                decl("width", "100%"),
                decl("height", "var(--fandhe-line-chart-height, auto)"),
                // イシュー #1595: SVG 非ルート要素は UA 既定で `overflow: hidden`
                // となるため、domain の max/min に接する折れ線
                // （`stroke-width: 2`）が viewBox 上下端で半分クリップされる。
                // ジオメトリ（`view_box_from_dims`/`category_x`）は変えず、
                // CSS のみで表示上のクリップを解除する（先例: area_chart #1589）。
                decl("overflow", "visible"),
            ],
        )
        .base(
            "series-line",
            vec![
                decl("fill", "none"),
                decl("stroke-width", "2"),
                // イシュー #1595: 先例 signature_pad / progress / area_chart。
                // 折れ線の鋭角部での miter 突出を抑え、端点の見た目を整える。
                decl("stroke-linejoin", "round"),
                decl("stroke-linecap", "round"),
            ],
        )
        .base(
            "point",
            // イシュー #1595: `n == 1` 時の点マーカーに背景色のハローを
            // 付け、背景・隣接系列色との識別性を高める。
            // `--fandhe-color-bg` はダーク時の値へトークン経由で自動追随
            // する（`theme.rs` の DEFAULT_COLORS 選定根拠を参照）。
            vec![
                decl("stroke", "var(--fandhe-color-bg)"),
                decl("stroke-width", "1"),
            ],
        )
        // イシュー #1681: `crate::area_chart::recipe` と同一の高さ値・
        // 導出根拠（差分 54→70 の拡大則を外挿）を共有する。
        .variant(
            Size::Xs,
            "root",
            vec![decl("--fandhe-line-chart-height", "58px")],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl("--fandhe-line-chart-height", "96px")],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl("--fandhe-line-chart-height", "150px")],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl("--fandhe-line-chart-height", "220px")],
        )
        .variant(
            Size::Xl,
            "root",
            vec![decl("--fandhe-line-chart-height", "306px")],
        )
        .default_variant(Size::Md)
}

/// この styled LineChart が生成する静的 CSS 全量を返す（決定的。
/// [`crate::qr_code::stylesheet`] と同じ契約）。
#[must_use]
pub fn stylesheet() -> String {
    recipe().css()
}

/// 系列 1 本を折れ線 `path`（`n >= 2`）または中央の点マーカー（`n == 1`）
/// として描く（内部ヘルパ）。
fn render_series(width: f64, y_scale: &LinearScale, values: &[f64], series_index: usize) -> Node {
    let n = values.len();
    let color = series_color_var(series_index);
    if n <= 1 {
        let x = category_x(width, n, 0);
        let y = values.first().copied().map_or(0.0, |v| y_scale.scale(v));
        let (cx, cy, r) = (fmt_coord(x), fmt_coord(y), fmt_coord(POINT_RADIUS));
        return el(
            "circle",
            vec![
                ("data-scope", "line-chart"),
                ("data-part", "point"),
                ("cx", cx.as_str()),
                ("cy", cy.as_str()),
                ("r", r.as_str()),
                ("fill", color.as_str()),
            ],
            vec![],
        );
    }

    let mut builder = PathBuilder::new();
    for (i, &v) in values.iter().enumerate() {
        let x = category_x(width, n, i);
        let y = y_scale.scale(v);
        builder = if i == 0 {
            builder.move_to(x, y)
        } else {
            builder.line_to(x, y)
        };
    }
    let d = builder.build();
    el(
        "path",
        vec![
            ("data-scope", "line-chart"),
            ("data-part", "series-line"),
            ("d", d.as_str()),
            ("stroke", color.as_str()),
            ("fill", "none"),
        ],
        vec![],
    )
}

/// LineChart 本体を組み立てる。
///
/// # Errors
///
/// `props.width`/`props.height` が非有限の場合 [`ChartError::NonFiniteValue`]、
/// 0 以下の場合 [`ChartError::DegenerateDomain`]（[`view_box_from_dims`] 参照）。
/// `props.data` は呼び出し側が [`ChartData::new`](crate::charts::data::ChartData::new)
/// を経由して構築済みであるため、それ以外のエラーは発生しない。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
/// use fandhe_frontend_pre_styled_ui::line_chart::{line_chart, LineChartProps};
///
/// let data = ChartData::new(
///     vec!["Jan".to_string(), "Feb".to_string(), "Mar".to_string()],
///     vec![Series::new("visits", vec![10.0, 30.0, 20.0])],
/// )
/// .unwrap();
/// let node = line_chart(&LineChartProps::new(&data, "monthly visits"), vec![]).unwrap();
/// let html = render(&node);
/// assert!(html.contains(r#"data-scope="line-chart" data-part="root""#));
/// assert!(html.contains("<path"));
/// ```
pub fn line_chart<'a>(
    props: &LineChartProps<'a>,
    attrs: Vec<(&'a str, &'a str)>,
) -> Result<Node, ChartError> {
    let view_box = view_box_from_dims(props.width, props.height)?;
    let y_scale = LinearScale::new(props.data.domain(), (props.height, 0.0))?;

    let plot_children: Vec<Node> = props
        .data
        .series()
        .iter()
        .enumerate()
        .map(|(i, s)| render_series(props.width, &y_scale, &s.values, i))
        .collect();

    let plot = svg_root(
        &view_box,
        vec![
            ("data-scope", "line-chart"),
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
    use crate::charts::data::Series;
    use fandhe_frontend_core::render;

    fn data(values: Vec<f64>) -> ChartData {
        let categories = (0..values.len()).map(|i| i.to_string()).collect();
        ChartData::new(categories, vec![Series::new("s", values)]).unwrap()
    }

    #[test]
    fn renders_root_and_plot_with_aria_label() {
        let d = data(vec![1.0, 2.0, 3.0]);
        let node = line_chart(&LineChartProps::new(&d, "sample chart"), vec![]).unwrap();
        let html = render(&node);
        assert!(html.starts_with(r#"<div data-scope="line-chart" data-part="root""#));
        assert!(html.contains(r#"data-scope="line-chart" data-part="plot""#));
        assert!(html.contains(r#"role="img""#));
        assert!(html.contains(r#"aria-label="sample chart""#));
    }

    #[test]
    fn single_category_renders_point_not_path() {
        let d = data(vec![5.0]);
        let node = line_chart(&LineChartProps::new(&d, "single"), vec![]).unwrap();
        let html = render(&node);
        assert!(html.contains(r#"data-part="point""#));
        assert!(!html.contains("<path"));
    }

    #[test]
    fn multi_category_renders_path_not_point() {
        let d = data(vec![1.0, 2.0]);
        let node = line_chart(&LineChartProps::new(&d, "multi"), vec![]).unwrap();
        let html = render(&node);
        assert!(html.contains("<path"));
        assert!(!html.contains(r#"data-part="point""#));
    }

    #[test]
    fn negative_values_produce_negative_coordinates_deterministically() {
        let d = data(vec![-10.0, 0.0, 10.0]);
        let node = line_chart(&LineChartProps::new(&d, "negative"), vec![]).unwrap();
        let html = render(&node);
        assert!(html.contains('-'));
    }

    #[test]
    fn flat_data_renders_horizontal_line() {
        let d = data(vec![5.0, 5.0, 5.0]);
        let node1 = line_chart(&LineChartProps::new(&d, "flat"), vec![]).unwrap();
        let node2 = line_chart(&LineChartProps::new(&d, "flat"), vec![]).unwrap();
        assert_eq!(render(&node1), render(&node2));
    }

    #[test]
    fn same_input_produces_same_output_deterministically() {
        let d = data(vec![1.0, 4.0, 2.0, 8.0]);
        let a = render(&line_chart(&LineChartProps::new(&d, "det"), vec![]).unwrap());
        let b = render(&line_chart(&LineChartProps::new(&d, "det"), vec![]).unwrap());
        assert_eq!(a, b);
    }

    #[test]
    fn non_finite_width_is_rejected() {
        let d = data(vec![1.0, 2.0]);
        let mut props = LineChartProps::new(&d, "bad");
        props.width = f64::NAN;
        assert_eq!(
            line_chart(&props, vec![]).unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn non_positive_height_is_rejected() {
        let d = data(vec![1.0, 2.0]);
        let mut props = LineChartProps::new(&d, "bad");
        props.height = 0.0;
        assert_eq!(
            line_chart(&props, vec![]).unwrap_err(),
            ChartError::DegenerateDomain
        );
    }

    #[test]
    fn caller_class_attr_on_root_is_dropped_not_duplicated() {
        let d = data(vec![1.0, 2.0]);
        let mut props = LineChartProps::new(&d, "class-test");
        props.size = Size::Lg;
        let html = render(&line_chart(&props, vec![("class", "attacker")]).unwrap());
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker"));
        assert!(html.contains("fd-line-chart--size-lg"));
    }

    #[test]
    fn xss_payload_in_aria_label_is_escaped() {
        let d = data(vec![1.0, 2.0]);
        let payload = "\"><img src=x onerror=alert(1)>";
        let node = line_chart(&LineChartProps::new(&d, payload), vec![]).unwrap();
        let html = render(&node);
        assert!(!html.contains("<img"));
        assert!(html.contains("&lt;img"));
    }

    #[test]
    fn xss_payload_in_caller_attrs_is_escaped() {
        let d = data(vec![1.0, 2.0]);
        let payload = "\"><script>alert(1)</script>";
        let html = render(
            &line_chart(
                &LineChartProps::new(&d, "attrs-test"),
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
        assert!(a.contains(r#"[data-scope="line-chart"][data-part="plot"]"#));
        assert!(a.contains("overflow: visible"));
        assert!(a.contains(r#"[data-scope="line-chart"][data-part="series-line"]"#));
        assert!(a.contains("stroke-linejoin: round"));
        assert!(a.contains("stroke-linecap: round"));
        assert!(a.contains(r#"[data-scope="line-chart"][data-part="point"]"#));
        assert!(a.contains("stroke: var(--fandhe-color-bg)"));
    }

    #[test]
    fn stylesheet_never_contains_style_breakout_sequences() {
        let css = stylesheet();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }
}
