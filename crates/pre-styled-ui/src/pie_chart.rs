//! styled PieChart（イシュー #850、親 Phase #845）。
//!
//! chakra-ui `charts/pie-chart.md`（recharts `PieChart`/`Pie`/`Cell` 依存）
//! 相当の円グラフを、外部依存ゼロ・[`crate::charts`] 基盤の SVG ノード木
//! 生成のみで実装する（`docs/policy/intentional-non-adoption.md` §7 の
//! 保留解除、[`crate::charts`] モジュール doc「保留解除トリガー」参照）。
//!
//! ark-ui には対応する headless anatomy が存在しないため、[`crate::marquee`]/
//! [`crate::stat`] と同型の判断で headless-ui は変更せず、本クレートのみで
//! 新規 anatomy `data-scope="pie-chart"` を定義する。
//!
//! # anatomy（4 パーツ）
//!
//! - `root`（`<div>`）: 寸法 variant のクラスを持つ唯一のパーツ。
//! - `chart`（`<svg>`、[`crate::charts::svg::svg_root`] 経由）: `viewBox`
//!   `"0 0 100 100"` 固定・`role="img"`（`svg_root` が既定付与）・
//!   `aria-label`（既定 `"pie chart"`、[`PieChartProps::aria_label`] で上書き
//!   可能）。
//! - `segment`（`<path>`。全周セグメントは `<circle>`、後述）: 系列 1 本の
//!   各カテゴリに対応する扇形。塗り色は [`crate::charts::series_color_var`]
//!   （`chart-1`〜`chart-6` トークン循環）。
//! - `label`（`<text>`）: [`PieChartProps::show_labels`] が `true` の場合
//!   のみ出力するカテゴリ名ラベル（中間角位置、既定エスケープ経由の
//!   テキストノード、REQ-1）。
//!
//! # 幾何・角度計算
//!
//! 境界角の算出・丸め規則・境界規則（値 `0` セグメントのスキップ・単一
//! 全周セグメントの特別扱い）は [`crate::charts::pie`] モジュール doc を
//! 参照。固定寸法として中心 `(50, 50)`・外径 `r = 45`（viewBox
//! `"0 0 100 100"` に対する定数、[`root`]/[`chart`] doc 参照）を用いる。
//!
//! # 単一系列専用（多系列は fail-closed で拒否）
//!
//! 円グラフは「全体に対する各カテゴリの割合」を表す性質上、複数系列を
//! 同時に扇形へ写像する意味を持たない。[`pie_chart`] は
//! `data.series().len() != 1` の場合 [`PieChartError::MultiSeries`] を返す
//! （構築時 fail-closed、[`crate::charts::data::ChartData`] 自体は複数系列を
//! 許容する汎用モデルであるため、本モジュール側で追加検証する）。
//!
//! # `size` variant（寸法のみ）
//!
//! [`crate::recipe::Size`]（既定 `Md`）のみを `root` へ付与し、
//! `--fandhe-pie-chart-size` の root スコープ custom property（通常の CSS
//! 継承により `chart` へ伝わる）経由で寸法を切り替える（[`crate::qr_code`]
//! と同型）。`color-palette` 軸は提供しない（セグメント配色はチャート共通
//! パレットの循環で決まるため、[`crate::qr_code`] と同型の判断）。
//!
//! # セキュリティ不変条件
//!
//! 本モジュールは `raw_html()` を使用しない。マークアップは `d`/`cx`/`cy`/
//! `r`/`x`/`y` 属性の数値文字列化を [`crate::charts::svg::fmt_coord`] のみに
//! 一元化した [`crate::charts::pie`]/[`crate::charts::svg`] のヘルパー経由
//! でのみ組み立て、任意文字列を SVG 属性値へ直接結合する経路を持たない。
//! カテゴリ名ラベル・`aria_label`・呼び出し側 `attrs` はすべて
//! `fandhe_frontend_core::render` の既定エスケープを経由する（REQ-1）。
//! `class` 属性は [`crate::class_attr::drop_class_attr`] により常に単一化
//! する。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - Legend / Tooltip（#847）。
//! - アニメーション（chakra は `isAnimationActive={false}` を推奨例として
//!   おり非対応で整合）。
//! - `paddingAngle`・`startAngle`/`endAngle` の任意指定・カスタム shape・
//!   中央テキスト（呼び出し側 children での代替は本 API のスコープ外）。
//! - `examples/headless-pre-styled-ui` への反映は crates.io 公開後に別途
//!   （[`crate::qr_code`]/[`crate::rating_group`] の先例と同じ判断）。
//!
//! # 参考サイト基準への調整（イシュー #1596）
//!
//! 親 Phase #1588「Themes / Charts のスタイル調整」の子。参照 4 サイト
//! （chakra-ui / Ark UI / Radix Primitives / Radix Themes）にチャート部品が
//! 存在しないため、評価軸は**内部整合のみ**（`--fandhe-*` トークン適用・
//! ダーク時の可読性・系列色の識別性・データラベルのコントラスト）に限定する。
//!
//! | 軸 | 結論 |
//! |---|---|
//! | サイズ | 現状維持（Xs〜Xl は #1681 で整備済み） |
//! | バリアント / colorPalette | 非採用（参照軸なし。系列色は `chart-1〜6` 固定ローテーション） |
//! | 色 | 現状維持（全宣言がトークン経由。`label` の `font-size: 6px` は viewBox ユーザー単位のため静的リテラルのまま） |
//! | 状態 `data-*` | 非該当（headless 由来の `data-*` を持たない pre-styled-only 部品） |
//! | ダークモード | ラベルのコントラストはハローで是正（下記）。系列パレット自体の見直しはスコープ外 |
//! | フォーカス | 非該当（`svg` は `role="img"` でフォーカス不可） |
//! | 余白・角丸・影 | 非該当（扇形 SVG 描画のみ） |
//! | hover / disabled / トランジション | 非採用（表示専用部品、状態遷移なし） |
//! | 内部整合（実欠陥） | **是正**（下記「是正した点」） |
//!
//! ## 是正した点
//!
//! - `label` slot に `dominant-baseline: central` を追加し、ラベルを
//!   扇形中心へ垂直方向にセンタリングした。従来は `text-anchor: middle`
//!   のみでベースライン調整が無く、狭い扇形ほど文字がベースライン基準で
//!   上側へ浮き扇形外へはみ出していた
//! - `label` slot へ背景色ハロー（`paint-order: stroke` /
//!   `stroke: var(--fandhe-color-bg)` / `stroke-width: 1` /
//!   `stroke-linejoin: round`）を追加した。dark モードでは `fill: var(--fandhe-color-fg)`
//!   が系列色の dark 値（`chart-1`/`chart-2` 等）に対して WCAG 4.5:1 を
//!   大きく下回り（`theme.rs` の light/dark トークン値からの概算）、light
//!   モードでも一部系列色で 4.5:1 未満だったため、系列色・ページ背景の
//!   どちらの上でも可読なハローで是正した（先例:
//!   [`crate::donut_chart`]（#1594）/ [`crate::area_chart`] `point` /
//!   `charts::tooltip` `datum`）。`paint-order: stroke` によりストローク
//!   を塗りの下へ回すため文字形は太らない
//! - `segment` slot に `stroke-linejoin: round` を追加した。各扇形 path は
//!   `M 中心 L 外周始点 A 外周弧 Z`（[`crate::charts::pie::sector_path`]）
//!   で閉じるため、**全セグメントが中心点を鋭角の共有頂点として持つ**。
//!   既定の miter では背景色ストローク（`stroke: var(--fandhe-color-bg)`）
//!   が中心から隣接セグメント側へ突き出し、描画順（後勝ち）に依存して
//!   背景色のスパイクが見えていた（単一全周セグメントの `<circle>` 分岐
//!   には結合部が無く無害）。donut（内周・外周の 4 頂点）より pie の方が
//!   中心 1 点に全セグメントが集まる分、症状が顕著だった
//!
//! 上記 3 点は兄弟部品 [`crate::donut_chart`]（#1594）で先行是正済みであり、
//! 本イシューはその引き継ぎとして pie 側に同型の是正を適用する。
//!
//! ## 意図的に合わせなかった点
//!
//! - `chart` slot への `overflow: visible` は、外径 45 + ストローク半幅
//!   0.5 が viewBox（100×100）内に収まるため不要
//! - `segment` slot への `vector-effect: non-scaling-stroke` は、兄弟部品
//!   [`crate::donut_chart`] と線幅の見え方が乖離するため見送る
//! - `Xs`（4rem）+ `show_labels` 時、`font-size: 6px` は実寸約 3.8px で
//!   判読が難しくなるが、ラベル表示は呼び出し側の選択であり本 PR では
//!   制約しない
//! - `label` の `font-weight` 引き上げ・`pointer-events: none` の付与は、
//!   効果が限定的で donut-chart（#1594）との整合を崩すため見送る
//! - `label` の `font-size` トークン化・系列パレット見直し等、上記 3 点を
//!   超える変更は双子部品（donut-chart）との整合を崩すため本 PR に含めない

use crate::charts::pie::{sector_path, segment_angles, PieChartError};
use crate::charts::svg::{circle, svg_root, svg_text, ViewBox};
use crate::charts::{series_color_var, ChartData};
use crate::class_attr::drop_class_attr;
use crate::css::decl;
use crate::recipe::{Size, SlotRecipe, VariantValue};
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, text, Node};
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="pie-chart"` を固定した本コンポーネントの anatomy。
const ANATOMY: Anatomy = anatomy("pie-chart");

/// [`SlotRecipe::new`] に渡す slot 一覧。
const SLOTS: &[&str] = &["root", "chart", "segment", "label"];

/// viewBox に対する中心 X 座標（固定、モジュール doc「幾何・角度計算」節）。
const CENTER_X: f64 = 50.0;
/// viewBox に対する中心 Y 座標（固定）。
const CENTER_Y: f64 = 50.0;
/// viewBox に対する外径（固定）。
const OUTER_RADIUS: f64 = 45.0;
/// ラベルを配置する半径（外径に対する比率。セグメント内側寄りに置く）。
const LABEL_RADIUS_RATIO: f64 = 0.6;

/// [`chart`] へ既定で付与する `aria-label`（[`PieChartProps::aria_label`]
/// が `None` の場合に使う）。
const DEFAULT_ARIA_LABEL: &str = "pie chart";

/// [`pie_chart`] の設定。
#[derive(Debug, Clone, Copy)]
pub struct PieChartProps<'a> {
    /// 寸法（既定 `Md`）。
    pub size: Size,
    /// `chart`（svg）へ付与する `aria-label`。`None` なら
    /// [`DEFAULT_ARIA_LABEL`]（`"pie chart"`）を使う。
    pub aria_label: Option<&'a str>,
    /// `true` ならカテゴリ名ラベルをセグメント上に描画する（既定 `false`）。
    pub show_labels: bool,
}

impl Default for PieChartProps<'_> {
    fn default() -> Self {
        Self {
            size: Size::Md,
            aria_label: None,
            show_labels: false,
        }
    }
}

/// この styled PieChart の既定 CSS を組み立てる（内部ヘルパ、[`css`] のみが
/// 呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("pie-chart", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "inline-flex"),
                decl("--fandhe-pie-chart-size", "16rem"),
            ],
        )
        .base(
            "chart",
            vec![
                decl("width", "var(--fandhe-pie-chart-size)"),
                decl("height", "var(--fandhe-pie-chart-size)"),
            ],
        )
        .base(
            "segment",
            vec![
                decl("stroke", "var(--fandhe-color-bg)"),
                decl("stroke-width", "1"),
                // イシュー #1596: 全セグメントが中心点を鋭角の共有頂点として
                // 持つため（`sector_path` が `M 中心 L 外周始点 A 外周弧 Z`
                // で閉じる）、既定の miter だと背景色セパレータが中心から
                // 隣接セグメント側へ突出して見える（donut #1594 と同型）。
                decl("stroke-linejoin", "round"),
            ],
        )
        .base(
            "label",
            vec![
                decl("fill", "var(--fandhe-color-fg)"),
                decl("font-size", "6px"),
                decl("text-anchor", "middle"),
                // イシュー #1596: ラベルを扇形中心へ垂直センタリングする
                // （`text-anchor` のみでは水平方向しか揃わず、狭い扇形で
                // 文字がベースライン基準で上側へ浮き扇形外へはみ出していた）。
                decl("dominant-baseline", "central"),
                // イシュー #1596: 背景色ハローで系列色・ページ背景どちらの
                // 上でも可読性を確保する（dark モードで `fg` が系列色の
                // dark 値に対し WCAG 4.5:1 を大きく下回るための是正、
                // donut #1594 と同型）。`paint-order: stroke` でストロークを
                // 塗りの下へ回し、文字形が太って見えるのを防ぐ。
                decl("paint-order", "stroke"),
                decl("stroke", "var(--fandhe-color-bg)"),
                decl("stroke-width", "1"),
                decl("stroke-linejoin", "round"),
            ],
        )
        // イシュー #1681: `crate::donut_chart::recipe` と同一の 6rem 刻み
        // 進行を共有する（size 値は donut と揃えている）。
        .variant(
            Size::Xs,
            "root",
            vec![decl("--fandhe-pie-chart-size", "4rem")],
        )
        .variant(
            Size::Sm,
            "root",
            vec![decl("--fandhe-pie-chart-size", "10rem")],
        )
        .variant(
            Size::Md,
            "root",
            vec![decl("--fandhe-pie-chart-size", "16rem")],
        )
        .variant(
            Size::Lg,
            "root",
            vec![decl("--fandhe-pie-chart-size", "22rem")],
        )
        .variant(
            Size::Xl,
            "root",
            vec![decl("--fandhe-pie-chart-size", "28rem")],
        )
        .default_variant(Size::Md)
}

/// この styled PieChart が生成する静的 CSS 全量を返す（決定的。
/// [`crate::qr_code::stylesheet`] と同じ契約）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// PieChart 1 個を組み立てる（`root` > `chart`(svg) > `segment`(path/circle)
/// [+ `label`(text)]）。
///
/// `data` はカテゴリ数 = セグメント数、系列数は必ず 1（モジュール doc
/// 「単一系列専用」節参照）。呼び出し側 `attrs` は `root` へ合成する
/// （`class` は [`drop_class_attr`] で除去してから recipe クラスへ一本化）。
///
/// # Errors
///
/// - `data.series().len() != 1` の場合 [`PieChartError::MultiSeries`]
/// - 系列の値に非有限・負値が含まれる、または合計が `0` の場合
///   [`crate::charts::pie::segment_angles`] のエラーをそのまま返す
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::charts::{ChartData, Series};
/// use fandhe_frontend_pre_styled_ui::pie_chart::{pie_chart, PieChartProps};
///
/// let data = ChartData::new(
///     vec!["A".to_string(), "B".to_string()],
///     vec![Series::new("total", vec![60.0, 40.0])],
/// )
/// .unwrap();
/// let node = pie_chart(&PieChartProps::default(), &data, vec![]).unwrap();
/// let html = render(&node);
/// assert!(html.contains(r#"role="img""#));
/// ```
pub fn pie_chart<'a>(
    props: &PieChartProps<'a>,
    data: &ChartData,
    attrs: Vec<(&'a str, &'a str)>,
) -> Result<Node, PieChartError> {
    if data.series().len() != 1 {
        return Err(PieChartError::MultiSeries);
    }
    let categories = data.categories();
    let values = &data.series()[0].values;
    let angles = segment_angles(values)?;

    // 非ゼロ値のセグメントがちょうど 1 個の場合（全周セグメント）は
    // sector_path が始点=終点の退化 arc を返すため、代わりに <circle> を
    // 描画する（`crate::charts::pie` モジュール doc「境界規則」節）。
    let non_zero_count = values.iter().filter(|&&v| v > 0.0).count();
    let is_full_circle = non_zero_count == 1;

    let mut segment_and_label_nodes: Vec<Node> = Vec::new();
    for (i, (&(start, end), &value)) in angles.iter().zip(values.iter()).enumerate() {
        // 値 0 のセグメントは境界角が退化するため描画しない。
        if value <= 0.0 {
            continue;
        }
        let fill = series_color_var(i);
        let segment_node = if is_full_circle {
            circle(
                CENTER_X,
                CENTER_Y,
                OUTER_RADIUS,
                vec![
                    ("data-scope", "pie-chart"),
                    ("data-part", "segment"),
                    ("fill", fill.as_str()),
                ],
            )
        } else {
            let d = sector_path(CENTER_X, CENTER_Y, OUTER_RADIUS, start, end);
            el(
                "path",
                vec![
                    ("data-scope", "pie-chart"),
                    ("data-part", "segment"),
                    ("d", d.as_str()),
                    ("fill", fill.as_str()),
                ],
                vec![],
            )
        };
        segment_and_label_nodes.push(segment_node);

        if props.show_labels {
            let mid = (start + end) / 2.0;
            let lx = CENTER_X + OUTER_RADIUS * LABEL_RADIUS_RATIO * mid.cos();
            let ly = CENTER_Y + OUTER_RADIUS * LABEL_RADIUS_RATIO * mid.sin();
            let category = categories.get(i).map(String::as_str).unwrap_or_default();
            let label_node = svg_text(
                lx,
                ly,
                vec![("data-scope", "pie-chart"), ("data-part", "label")],
                vec![text(category)],
            );
            segment_and_label_nodes.push(label_node);
        }
    }

    let view_box = ViewBox::new(0.0, 0.0, 100.0, 100.0)
        .expect("固定 viewBox 100x100 は常に有効な正の寸法である");
    let aria_label_value = props.aria_label.unwrap_or(DEFAULT_ARIA_LABEL);
    let chart_node = svg_root(
        &view_box,
        vec![
            ("data-scope", "pie-chart"),
            ("data-part", "chart"),
            ("aria-label", aria_label_value),
        ],
        segment_and_label_nodes,
    );

    let recipe = recipe();
    let class = recipe.variant_classes(&[("size", props.size.value())]);
    let mut merged: Vec<(&str, &str)> = vec![("class", class.as_str())];
    merged.extend(drop_class_attr(attrs));

    Ok(ANATOMY.part("root", "div", merged, vec![chart_node]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::Series;
    use fandhe_frontend_core::render;

    fn two_category_data() -> ChartData {
        ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![Series::new("total", vec![60.0, 40.0])],
        )
        .unwrap()
    }

    #[test]
    fn renders_root_chart_and_segments_with_default_aria_label() {
        let node = pie_chart(&PieChartProps::default(), &two_category_data(), vec![]).unwrap();
        let html = render(&node);
        assert!(html.contains(r#"data-scope="pie-chart" data-part="root""#));
        assert!(html.contains(r#"data-scope="pie-chart" data-part="chart""#));
        assert!(html.contains(r#"role="img""#));
        assert!(html.contains(r#"aria-label="pie chart""#));
        assert!(html.contains(r#"viewBox="0 0 100 100""#));
        assert_eq!(html.matches(r#"data-part="segment""#).count(), 2);
        assert!(html.contains("var(--fandhe-color-chart-1)"));
        assert!(html.contains("var(--fandhe-color-chart-2)"));
    }

    #[test]
    fn custom_aria_label_overrides_default() {
        let props = PieChartProps {
            aria_label: Some("revenue split"),
            ..PieChartProps::default()
        };
        let html = render(&pie_chart(&props, &two_category_data(), vec![]).unwrap());
        assert!(html.contains(r#"aria-label="revenue split""#));
        assert!(!html.contains("pie chart"));
    }

    #[test]
    fn show_labels_renders_category_text_nodes() {
        let props = PieChartProps {
            show_labels: true,
            ..PieChartProps::default()
        };
        let html = render(&pie_chart(&props, &two_category_data(), vec![]).unwrap());
        assert_eq!(html.matches(r#"data-part="label""#).count(), 2);
        assert!(html.contains(">A<"));
        assert!(html.contains(">B<"));
    }

    #[test]
    fn default_hides_labels() {
        let html =
            render(&pie_chart(&PieChartProps::default(), &two_category_data(), vec![]).unwrap());
        assert!(!html.contains(r#"data-part="label""#));
    }

    #[test]
    fn zero_value_segment_is_skipped() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec![Series::new("total", vec![50.0, 0.0, 50.0])],
        )
        .unwrap();
        let html = render(&pie_chart(&PieChartProps::default(), &data, vec![]).unwrap());
        assert_eq!(html.matches(r#"data-part="segment""#).count(), 2);
    }

    #[test]
    fn single_non_zero_segment_renders_as_circle() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![Series::new("total", vec![100.0, 0.0])],
        )
        .unwrap();
        let html = render(&pie_chart(&PieChartProps::default(), &data, vec![]).unwrap());
        assert_eq!(html.matches("<circle").count(), 1);
        assert!(!html.contains("<path"));
    }

    #[test]
    fn multi_series_is_rejected() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![
                Series::new("s1", vec![1.0, 2.0]),
                Series::new("s2", vec![3.0, 4.0]),
            ],
        )
        .unwrap();
        assert_eq!(
            pie_chart(&PieChartProps::default(), &data, vec![]).unwrap_err(),
            PieChartError::MultiSeries
        );
    }

    #[test]
    fn zero_total_propagates_geometry_error() {
        let data = ChartData::new(
            vec!["A".to_string(), "B".to_string()],
            vec![Series::new("total", vec![0.0, 0.0])],
        )
        .unwrap();
        assert_eq!(
            pie_chart(&PieChartProps::default(), &data, vec![]).unwrap_err(),
            PieChartError::ZeroTotal
        );
    }

    #[test]
    fn size_variant_applies_root_class() {
        let node = pie_chart(
            &PieChartProps {
                size: Size::Lg,
                ..PieChartProps::default()
            },
            &two_category_data(),
            vec![],
        )
        .unwrap();
        assert!(render(&node).contains("pie-chart--size-lg"));
    }

    #[test]
    fn caller_class_attr_is_dropped_not_duplicated() {
        let html = render(
            &pie_chart(
                &PieChartProps::default(),
                &two_category_data(),
                vec![("class", "attacker-controlled")],
            )
            .unwrap(),
        );
        assert_eq!(html.matches("class=\"").count(), 1);
        assert!(!html.contains("attacker-controlled"));
    }

    #[test]
    fn css_output_is_deterministic_and_targets_data_scope_selectors() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="pie-chart"][data-part="chart"]"#));
        assert!(!a.contains("color-palette"));
    }

    #[test]
    fn recipe_includes_issue_1596_corrections() {
        // イシュー #1596: ラベルの垂直センタリング・背景色ハロー・
        // セパレータ線の miter 突出抑止が実出力に現れることを固定する
        // （黙って除外されていないことの確認、donut #1594 の
        // `recipe_includes_issue_1594_corrections` と同型）。
        let a = css();
        assert!(a.contains(r#"[data-scope="pie-chart"][data-part="label"]"#));
        assert!(a.contains("dominant-baseline: central"));
        assert!(a.contains("paint-order: stroke"));
        assert!(a.contains("stroke-linejoin: round"));
    }
}
