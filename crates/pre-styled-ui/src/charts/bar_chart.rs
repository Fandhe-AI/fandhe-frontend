//! BarChart（SVG 棒グラフ、イシュー #849・親 Phase #845）。
//!
//! chakra-ui `charts/bar-chart.md`（recharts `BarComposition` 相当）を、
//! [`super::data::ChartData`]（複数系列）+ [`super::scale::LinearScale`]
//! （値軸の domain → range 写像）+ [`super::svg`]（マークアップ生成）の
//! 3 層のみを組み合わせて、外部依存ゼロ・決定的なグループ棒グラフとして
//! 再構成する。
//!
//! # レイアウト規則（決定的。本モジュールが唯一の正）
//!
//! 1. **値軸**: `data.domain()` を基準に `(0.0 を含むよう拡張) → LinearScale::new
//!    → nice()` を経由する（棒はベースライン 0 起点、chakra-ui/recharts の
//!    既定と同じ）。`data.domain()` は必ず 0 を跨ぐとは限らないため、値域を
//!    `(domain.0.min(0.0), domain.1.max(0.0))` へ明示的に広げてから
//!    `LinearScale::new` に渡す（正値のみ・負値のみのデータでもベースライン 0
//!    が描画範囲に含まれることを保証する）。
//! 2. **カテゴリ軸（バンドレイアウト）**: カテゴリ数 `n` に対し
//!    `band = plot_length / n`。各バンド内は両端に `BAND_EDGE_PADDING_FRAC`
//!    （10%）ずつの余白を置き、残り 80% を系列数で均等分割して棒幅とする
//!    （系列間の追加ギャップは設けない、純算術で決定的）。
//! 3. **座標の文字列化**: すべて [`super::svg::fmt_coord`] のみを経由する
//!    （独自フォーマット禁止、[`crate::charts`] モジュール doc 不変条件 2）。
//! 4. **軸線・グリッド・凡例・ツールチップ**: 本モジュールのスコープ外
//!    （イシュー #847 が担当。本モジュールはカテゴリラベルの最小出力のみ
//!    行う）。
//!
//! # a11y
//!
//! [`super::svg::svg_root`] が既定付与する `role="img"` に加え、呼び出し側
//! 必須の `aria_label` 引数を出力する（画像として読み上げられるため代替
//! テキストが必須、`progress`/`image` 等の alt 必須パターンと同じ発想）。
//!
//! # セキュリティ不変条件
//!
//! マークアップはすべて [`super::svg`] 経由（`el`/`text` を最終的に呼ぶ）で
//! 組み立て、`raw_html()` は使用しない（REQ-1）。系列名・カテゴリ名・
//! `aria_label` はすべて [`fandhe_frontend_core::text`] のテキストノードとして
//! 渡すため `render()` の既定エスケープを必ず通る。座標・寸法は
//! [`ChartData::new`](super::data::ChartData::new)/
//! [`LinearScale::new`](super::scale::LinearScale::new) が有限性検証済みの
//! `f64` のみを [`super::svg::fmt_coord`] へ渡すため、文字列注入経路を持たない。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - 軸線・グリッド・凡例・ツールチップ（#847）。
//! - `examples/headless-pre-styled-ui` への追随は crates.io 公開後に別途
//!   行う（`qr_code`/`rating_group` の先例と同じ判断）。
//!
//! ## イシュー #1590（参考サイト基準へのスタイル調整、内部整合軸）でのスコープ外判断
//!
//! 参照 4 サイト（chakra-ui / Radix Themes / Radix Primitives / ark-ui）には
//! チャート部品が存在しないため、評価軸は内部整合（トークン経由の配色・
//! ダークモード可読性・系列色の識別性）のみに限定される。この軸に基づき
//! 以下は意図的に是正しない:
//!
//! - **hover / transition**: `bar`/`category-label` は表示専用 slot
//!   （`role="img"` の SVG 内 `<rect>`/`<text>`。`cursor: pointer` も
//!   `<button>`/`<a>`/interactive role も持たない）であり、
//!   `docs/design/pre-styled-ui-interaction-visual-language.md` §3 の
//!   「hover 付与の判定基準: インタラクティブ slot のみ」に該当しない
//!   （`docs/policy/intentional-non-adoption.md` の JS ランタイム前提論では
//!   なく、この判定基準に基づく判断へ更新した）。[`super::tooltip`] の
//!   `datum` が hover を持つのは子 `<title>` によるネイティブツールチップ
//!   表示と組み合わせるための例外であり、本モジュールの棒は `<title>` を
//!   持たない（`<title>` 追加はマークアップ変更であり本イシューの CSS
//!   調整の範囲外）。transition が無いため `prefers-reduced-motion` も
//!   対象外。
//! - **focus**: フォーカス可能要素が存在しない（`svg` は `tabindex` を
//!   持たない）。
//! - **disabled**: `data-disabled` を出力する経路が無い（静的部品）。
//! - **size 軸**: 姉妹の line/area/pie/donut/sparkline は `plot` slot の
//!   固定高さを `Size` で切り替えるが、本モジュールは `plot` slot を
//!   持たず高さは [`BarChartProps`]（`width`/`height`）の viewBox
//!   アスペクト比で決まる。size 追加には全 `pub` フィールドを持つ
//!   [`BarChartProps`] へのフィールド追加（フルリテラル構築を前提とする
//!   単体テストを壊す 0.x 破壊的変更）が必要であり、参照サイト由来の
//!   variant/size 網羅性を評価しない本イシューでは見送る（親 #1588 への
//!   スコープ外報告候補）。
//! - **角丸（`rx`）**: chakra-ui BarChart（recharts）の既定は角丸なしで
//!   あり、SVG `rx` の CSS プロパティ化はブラウザ差があるため内部整合軸の
//!   範囲を超える。
//! - **幾何（バンド余白・ラベル位置定数）**: Rust 側の定数であり、変更す
//!   るとレンダリング結果と手計算ジオメトリテストが変わる。CSS 調整の
//!   範囲外。
//! - **系列色トークン（`chart-1`〜`chart-6`）**: light/dark 両方が
//!   `theme.rs` に定義済み（`docs/design/color-token-system.md`）であり
//!   変更しない。

use super::data::ChartData;
use super::scale::LinearScale;
use super::svg::{self, svg_text, ViewBox, ViewBoxError};
use super::{series_color_var, ChartError};
use crate::css::decl;
use crate::recipe::SlotRecipe;
use fandhe_frontend_headless_ui::fandhe_frontend_core::{text, Node};

/// バンド内の両端余白（片側、バンド幅に対する比率）。
const BAND_EDGE_PADDING_FRAC: f64 = 0.1;

/// カテゴリラベル（`svg_text`）用に確保する軸方向の余白（px 相当）。
///
/// [`Orientation::Vertical`] では棒の下側にラベルを 1 行分収める用途のため
/// 24px で足りる（`text-anchor="middle"` でバンド内に収まり、高さ方向に
/// 折り返しがないため）。
const CATEGORY_LABEL_SPACE: f64 = 24.0;

/// [`Orientation::Horizontal`] でカテゴリラベル用に確保する `viewBox` 右側の
/// 余白（px 相当）。
///
/// Horizontal はラベルを `plot_width + 4` から `text-anchor="start"` で
/// 右方向に伸ばす（[`category_label`]）ため、[`CATEGORY_LABEL_SPACE`]
/// （24px、Vertical のバンド下余白流用）のままだとラベル文字列が
/// `viewBox` 右端を越えてクリップされ、ほぼ判読不能になっていた
/// （PR #877 Bugbot 指摘、イシュー #849）。カテゴリ名は任意長でありテキスト
/// 幅を事前計測できない（フォントメトリクス非依存が本モジュールの方針）
/// ため、厳密な無クリップ保証はできないが、一般的なラベル長を収める実用的
/// な既定値としてより広い余白を確保する。
const CATEGORY_LABEL_SPACE_HORIZONTAL: f64 = 96.0;

/// `data-scope="bar-chart"` の part 一覧（recipe と揃える）。
const SLOTS: &[&str] = &["root", "bar", "category-label"];

/// 棒の並べ方（chakra-ui BarChart `layout` の `vertical`/`horizontal` を
/// 縮約。命名は「棒が伸びる向き」ではなく「カテゴリ軸の向き」を表す
/// chakra-ui の用語をそのまま踏襲する）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Orientation {
    /// カテゴリ軸が横（x 軸）、値軸が縦（y 軸）。棒は縦に伸びる（既定）。
    #[default]
    Vertical,
    /// カテゴリ軸が縦（y 軸）、値軸が横（x 軸）。棒は横に伸びる。
    Horizontal,
}

/// [`root`] の描画パラメータ。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BarChartProps {
    /// 棒の向き（既定 [`Orientation::Vertical`]）。
    pub orientation: Orientation,
    /// `viewBox` の幅（px 相当。既定 480.0）。
    pub width: f64,
    /// `viewBox` の高さ（px 相当。既定 300.0）。
    pub height: f64,
}

impl Default for BarChartProps {
    fn default() -> Self {
        BarChartProps {
            orientation: Orientation::default(),
            width: 480.0,
            height: 300.0,
        }
    }
}

/// この BarChart の既定 CSS を組み立てる（内部ヘルパ、[`css`] のみが呼ぶ）。
///
/// 色は棒ごとに [`series_color_var`] のインライン `fill` 属性で決まるため、
/// recipe は寸法系の最小宣言のみを持つ（[`crate::qr_code`] の
/// 「前景/背景は固定トークン・variant は寸法のみ」判断と同型ではなく、本
/// 部品は variant 自体を持たない静的部品。[`crate::table`] の
/// 「状態機械を持たない静的 styled 部品」に分類される）。
///
/// # 不変条件（イシュー #1590）
///
/// - **`bar` の base に `fill` を書かない**: 棒の色は [`root`] が各棒へ
///   `fill="var(--fandhe-color-chart-N)"`（[`series_color_var`]）を
///   presentation 属性として直接付与している。SVG の presentation 属性は
///   author origin の specificity 0 として扱われるため、`[data-scope=
///   "bar-chart"][data-part="bar"]` セレクタを持つ CSS 宣言のほうが優先
///   され、presentation 属性の系列色を上書きしてしまう。つまり recipe に
///   `fill` を 1 本でも書くと **現時点で既に** 全系列が同色に潰れる
///   （将来 presentation 属性側を外すリファクタを待たずに壊れる）。この
///   不変条件は `bar_rule_has_stroke_but_never_fill`（下記テスト）が機械
///   固定する。
/// - `bar` の `stroke`/`stroke-width` は隣接する系列棒の境界を明示する
///   （[`super::scatter_chart`] の `point`・[`super::pie`] の slice と
///   同型。棒が密着しているとライト/ダーク両テーマで境界が判別しづらい
///   内部整合上の不足だった、イシュー #1590）。色は背景トークン
///   `--fandhe-color-bg` を使うため、ダーク時もテーマ再定義経由で自動的に
///   背景色へ追随する。
/// - `stroke-width` の値は単位なし `"1"` を採用する（[`super::axis`]・
///   [`super::grid`] と同じ多数派表記。[`super::scatter_chart`] の `"1px"`
///   は少数派表記であり本部品では踏襲しない）。
/// - **`root` に `overflow: visible` を付与する**: `bar` の
///   `stroke-width: 1` は rect の外側へ 0.5 ユーザー単位はみ出して
///   描かれる。最大値の縦棒は `y == 0`、横棒の baseline は `x == 0` に
///   接するため、このはみ出しが viewBox の外側へ出て UA 既定
///   `svg:not(:root) { overflow: hidden }` にクリップされる（stroke の
///   一辺が欠けて見える）。兄弟部品 scatter（[`super::scatter_chart`]、
///   #1598）/ line・area の `root`/`plot` と同じ理由・同じ対処であり、
///   ジオメトリ（`bar()` の座標）を変えずに CSS のみで整合を取る。
/// - `category-label` の `font-family` は [`super::axis`] の `tick-label`
///   と同じ書体トークン `--fandhe-font-font-body` を使う（同じ SVG 内
///   テキストで書体指定の有無が食い違っていた内部整合上の不足の是正）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("bar-chart", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "block"),
                decl("max-width", "100%"),
                // bar の stroke（1 ユーザー単位）が rect 外側へ 0.5 単位
                // はみ出し、最大値の棒で viewBox の外に出るのを UA 既定
                // overflow: hidden でクリップさせない（イシュー #1590、
                // scatter/line/area の root/plot と同型）。
                decl("overflow", "visible"),
            ],
        )
        .base(
            "bar",
            vec![
                decl("stroke", "var(--fandhe-color-bg)"),
                decl("stroke-width", "1"),
            ],
        )
        .base(
            "category-label",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("font-family", "var(--fandhe-font-font-body)"),
                decl("fill", "var(--fandhe-color-fg-muted)"),
            ],
        )
}

/// この BarChart が生成する静的 CSS 全量を返す（決定的）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// BarChart 本体を組み立てる。
///
/// `aria_label` は `svg_root` の `role="img"` に対する代替テキストとして
/// 必須（モジュール doc「a11y」節参照）。
///
/// # Errors
///
/// - `data` の値軸 domain・`viewBox` 寸法のいずれかが非有限、または
///   `props.width`/`props.height` が 0 以下の場合、内部の
///   [`ViewBox::new`]/[`LinearScale::new`] の失敗を [`ChartError`] へ変換して
///   返す（[`ChartError::NonFiniteValue`]/[`ChartError::DegenerateDomain`]）。
/// - `props.width`/`props.height` が正でも、カテゴリラベル用余白
///   （[`Orientation::Vertical`] は [`CATEGORY_LABEL_SPACE`]、
///   [`Orientation::Horizontal`] は [`CATEGORY_LABEL_SPACE_HORIZONTAL`]）を
///   差し引いた結果プロット領域の幅・高さが 0 以下になる場合
///   [`ChartError::PlotAreaTooSmall`]（`ViewBox::new` は寸法の正値のみを
///   検証し、余白差し引き後までは検証しないため、放置するとバーが潰れる、
///   または viewBox 外へ無警告で描画される、PR #877 レビュー指摘）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::charts::bar_chart::{root, BarChartProps};
/// use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
///
/// let data = ChartData::new(
///     vec!["Jan".to_string(), "Feb".to_string()],
///     vec![Series::new("visits", vec![10.0, 30.0])],
/// )
/// .unwrap();
/// let node = root(&data, BarChartProps::default(), "monthly visits").unwrap();
/// assert!(render(&node).contains(r#"role="img""#));
/// ```
pub fn root(data: &ChartData, props: BarChartProps, aria_label: &str) -> Result<Node, ChartError> {
    let view_box = ViewBox::new(0.0, 0.0, props.width, props.height).map_err(|e| match e {
        // 非有限（NaN/±inf）はデータ・寸法の値そのものが壊れているため
        // NonFiniteValue、width/height が 0 以下（正だが degenerate）は
        // 「描画不能な退化寸法」として DegenerateDomain へ、姉妹チャート
        // （line_chart 相当の判断）と同じくマッピングする（PR #877 Bugbot
        // 指摘、イシュー #849。旧実装は非正の width/height も一律
        // NonFiniteValue に丸めており # Errors ドキュメントの契約と乖離
        // していた）。
        ViewBoxError::NonFinite => ChartError::NonFiniteValue,
        ViewBoxError::NonPositiveSize => ChartError::DegenerateDomain,
    })?;

    let (dmin, dmax) = data.domain();
    let (dmin, dmax) = (dmin.min(0.0), dmax.max(0.0));
    // domain() は非退化を保証するが、上記の 0 拡張で min==max==0 になる
    // ケースは発生しない（domain() が既に min<max を保証しているため、
    // 0 を含めて広げても min<=0<=max かつ min<max のまま）。
    let categories = data.categories();
    let n_categories = categories.len();
    let series = data.series();
    let n_series = series.len().max(1);

    let (plot_width, plot_height) = match props.orientation {
        Orientation::Vertical => (props.width, props.height - CATEGORY_LABEL_SPACE),
        Orientation::Horizontal => (props.width - CATEGORY_LABEL_SPACE_HORIZONTAL, props.height),
    };
    // `ViewBox::new` は width/height が正であることのみ検証し、カテゴリ
    // ラベル余白差し引き後の実プロット領域までは検証しない。ここで拒否
    // しないと、幅・高さが 0 以下のままバンド幅・棒寸法が 0/負値になり、
    // バーが潰れる、または viewBox 外に無警告で描画される
    // （PR #877 レビュー指摘、イシュー #849）。
    if plot_width <= 0.0 || plot_height <= 0.0 {
        return Err(ChartError::PlotAreaTooSmall);
    }

    let value_range = match props.orientation {
        // SVG は y が下向き正のため、値の大小を上下反転させる。
        Orientation::Vertical => (plot_height, 0.0),
        Orientation::Horizontal => (0.0, plot_width),
    };
    let value_scale =
        LinearScale::new((dmin, dmax), value_range).map_err(|_| ChartError::NonFiniteValue)?;
    let baseline = value_scale.scale(0.0);

    let band = match props.orientation {
        Orientation::Vertical => plot_width / n_categories as f64,
        Orientation::Horizontal => plot_height / n_categories as f64,
    };
    let usable = band * (1.0 - 2.0 * BAND_EDGE_PADDING_FRAC);
    let edge_offset = band * BAND_EDGE_PADDING_FRAC;
    let bar_thickness = usable / n_series as f64;

    let mut bars: Vec<Node> = Vec::new();
    for (cat_idx, category) in categories.iter().enumerate() {
        let band_start = band * cat_idx as f64;
        for (series_idx, s) in series.iter().enumerate() {
            let value = s.values[cat_idx];
            let scaled = value_scale.scale(value);
            let color = series_color_var(series_idx);
            let attrs = vec![
                ("data-scope", "bar-chart"),
                ("data-part", "bar"),
                ("fill", color.as_str()),
            ];
            let rect = match props.orientation {
                Orientation::Vertical => {
                    let x = band_start + edge_offset + bar_thickness * series_idx as f64;
                    let y = scaled.min(baseline);
                    let h = (scaled - baseline).abs();
                    svg::rect(x, y, bar_thickness, h, attrs)
                }
                Orientation::Horizontal => {
                    let y = band_start + edge_offset + bar_thickness * series_idx as f64;
                    let x = scaled.min(baseline);
                    let w = (scaled - baseline).abs();
                    svg::rect(x, y, w, bar_thickness, attrs)
                }
            };
            bars.push(rect);
        }
        let label = category_label(category, band_start, band, plot_width, plot_height, props);
        bars.push(label);
    }

    let attrs = vec![("data-scope", "bar-chart"), ("data-part", "root")];
    let mut merged_attrs = vec![("aria-label", aria_label)];
    merged_attrs.extend(attrs);
    Ok(svg::svg_root(&view_box, merged_attrs, bars))
}

/// カテゴリラベル（[`svg_text`]）を組み立てる（内部ヘルパ）。
fn category_label(
    category: &str,
    band_start: f64,
    band: f64,
    plot_width: f64,
    plot_height: f64,
    props: BarChartProps,
) -> Node {
    let (x, y, attrs) = match props.orientation {
        Orientation::Vertical => (
            band_start + band / 2.0,
            plot_height + CATEGORY_LABEL_SPACE / 2.0 + 4.0,
            vec![("text-anchor", "middle")],
        ),
        Orientation::Horizontal => (
            plot_width + 4.0,
            band_start + band / 2.0,
            vec![("text-anchor", "start")],
        ),
    };
    let mut merged: Vec<(&str, &str)> =
        vec![("data-scope", "bar-chart"), ("data-part", "category-label")];
    merged.extend(attrs);
    svg_text(x, y, merged, vec![text(category.to_string())])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::data::Series;
    use fandhe_frontend_core::render;

    fn sample() -> ChartData {
        ChartData::new(
            vec!["Jan".to_string(), "Feb".to_string()],
            vec![Series::new("visits", vec![10.0, 30.0])],
        )
        .unwrap()
    }

    #[test]
    fn root_rejects_width_or_height_too_small_for_category_label_space() {
        // PR #877 レビュー指摘: height/width がカテゴリラベル用余白
        // （Vertical は CATEGORY_LABEL_SPACE 24.0、Horizontal は
        // CATEGORY_LABEL_SPACE_HORIZONTAL 96.0）以下だとプロット領域が
        // 0 以下になり、バーが潰れる/viewBox 外描画になる silent failure
        // だった。fail-closed で拒否する。
        let vertical_too_small = BarChartProps {
            orientation: Orientation::Vertical,
            width: 480.0,
            height: 24.0,
        };
        assert_eq!(
            root(&sample(), vertical_too_small, "label").unwrap_err(),
            ChartError::PlotAreaTooSmall
        );

        let horizontal_too_small = BarChartProps {
            orientation: Orientation::Horizontal,
            width: 96.0,
            height: 300.0,
        };
        assert_eq!(
            root(&sample(), horizontal_too_small, "label").unwrap_err(),
            ChartError::PlotAreaTooSmall
        );
    }

    #[test]
    fn root_maps_non_finite_and_non_positive_view_box_to_distinct_errors() {
        // PR #877 Bugbot 指摘: `ViewBox::new` の失敗が非有限（NaN/±inf）か
        // 非正 width/height（0 以下）かによらず一律 NonFiniteValue に丸め
        // られていた。`# Errors` ドキュメントが約束する
        // NonFiniteValue/DegenerateDomain の使い分けを固定する。
        let non_finite = BarChartProps {
            orientation: Orientation::Vertical,
            width: f64::NAN,
            height: 300.0,
        };
        assert_eq!(
            root(&sample(), non_finite, "label").unwrap_err(),
            ChartError::NonFiniteValue
        );

        let non_positive = BarChartProps {
            orientation: Orientation::Vertical,
            width: 0.0,
            height: 300.0,
        };
        assert_eq!(
            root(&sample(), non_positive, "label").unwrap_err(),
            ChartError::DegenerateDomain
        );
    }

    #[test]
    fn root_renders_role_img_and_aria_label() {
        let node = root(&sample(), BarChartProps::default(), "monthly visits").unwrap();
        let html = render(&node);
        assert!(html.contains(r#"role="img""#));
        assert!(html.contains(r#"aria-label="monthly visits""#));
        assert!(html.contains(r#"data-scope="bar-chart" data-part="root""#));
    }

    #[test]
    fn root_renders_one_bar_per_category_series_pair() {
        let node = root(&sample(), BarChartProps::default(), "label").unwrap();
        let html = render(&node);
        assert_eq!(html.matches(r#"data-part="bar""#).count(), 2);
    }

    #[test]
    fn root_is_deterministic() {
        let a = render(&root(&sample(), BarChartProps::default(), "label").unwrap());
        let b = render(&root(&sample(), BarChartProps::default(), "label").unwrap());
        assert_eq!(a, b);
    }

    #[test]
    fn vertical_bar_geometry_matches_hand_computed_values() {
        // domain: (0,30) を nice せず 0 起点のまま使う本モジュールの規則
        // （nice() は #847 の軸描画スコープであり本モジュールは適用しない）。
        // plot_height = 300 - 24 = 276, plot_width = 480。
        // band = 480 / 2 = 240, edge_offset = 24, usable = 192,
        // bar_thickness = 192 (1 系列)。
        // Jan (value=10): scaled = 276 - (10/30)*276 = 184, baseline = 276。
        //   y = min(184,276)=184, h = |184-276| = 92, x = 0+24 = 24。
        let node = root(&sample(), BarChartProps::default(), "label").unwrap();
        let html = render(&node);
        assert!(html.contains(r#"x="24""#));
        assert!(html.contains(r#"y="184""#));
        assert!(html.contains(r#"height="92""#));
        assert!(html.contains(r#"width="192""#));
    }

    #[test]
    fn horizontal_orientation_swaps_axes() {
        let props = BarChartProps {
            orientation: Orientation::Horizontal,
            ..BarChartProps::default()
        };
        let node = root(&sample(), props, "label").unwrap();
        let html = render(&node);
        assert!(html.contains(r#"data-part="bar""#));
    }

    #[test]
    fn horizontal_category_label_start_stays_within_view_box() {
        // PR #877 Bugbot 指摘（Medium）: Horizontal ではラベルが
        // `plot_width + 4` から `text-anchor="start"` で右方向に伸びるため、
        // 余白が狭すぎるとラベル文字列が viewBox 右端を越えてクリップされる
        // （イシュー #849）。CATEGORY_LABEL_SPACE_HORIZONTAL 導入後は
        // ラベル開始位置 (`plot_width + 4`) と viewBox 右端
        // (`props.width`) の間に十分な余白が残ることを固定する。
        let props = BarChartProps {
            orientation: Orientation::Horizontal,
            ..BarChartProps::default()
        };
        let node = root(&sample(), props, "label").unwrap();
        let html = render(&node);
        // plot_width = 480 - 96 = 384, label x = 384 + 4 = 388。
        assert!(html.contains(r#"x="388""#));
        // ラベル開始位置から viewBox 右端までの残り余白（92px）が
        // クリップ再発防止の下限としてゼロより十分大きいことを保証する。
        let label_start = 388.0;
        assert!(props.width - label_start >= 90.0);
    }

    #[test]
    fn negative_and_positive_values_share_baseline_zero() {
        let data = ChartData::new(
            vec!["a".to_string(), "b".to_string()],
            vec![Series::new("s", vec![-10.0, 10.0])],
        )
        .unwrap();
        let node = root(&data, BarChartProps::default(), "label").unwrap();
        let html = render(&node);
        // 両方とも描画され、baseline をまたいでも panic しない。
        assert_eq!(html.matches(r#"data-part="bar""#).count(), 2);
    }

    #[test]
    fn multi_series_renders_bars_side_by_side_within_band() {
        let data = ChartData::new(
            vec!["a".to_string()],
            vec![Series::new("s1", vec![10.0]), Series::new("s2", vec![20.0])],
        )
        .unwrap();
        let node = root(&data, BarChartProps::default(), "label").unwrap();
        let html = render(&node);
        assert_eq!(html.matches(r#"data-part="bar""#).count(), 2);
        // 2 系列は色トークンが異なる。
        assert!(html.contains("chart-1"));
        assert!(html.contains("chart-2"));
    }

    #[test]
    fn category_and_series_names_are_escaped() {
        let data = ChartData::new(
            vec!["<script>".to_string()],
            vec![Series::new("s", vec![1.0])],
        )
        .unwrap();
        let node = root(&data, BarChartProps::default(), "<script>alert(1)</script>").unwrap();
        let html = render(&node);
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn css_is_deterministic_and_has_no_breakout_sequences() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(!a.contains('<'));
        assert!(a.contains(r#"[data-scope="bar-chart"]"#));
    }

    /// `[data-scope="bar-chart"][data-part="bar"]` 規則のみを切り出す。
    ///
    /// [`recipe`] doc の不変条件（`bar` は `fill` を持たない）をブロック
    /// 単位で検査するため、`css()` 全体を対象にすると `category-label` の
    /// `fill` 宣言に誤検知してしまう問題を避ける。
    fn bar_rule_block(css: &str) -> &str {
        let selector = r#"[data-scope="bar-chart"][data-part="bar"] {"#;
        let start = css
            .find(selector)
            .expect("bar rule block must be present in css()");
        let rest = &css[start..];
        let end = rest.find("}\n").expect("bar rule block must be closed");
        &rest[..end]
    }

    #[test]
    fn bar_rule_has_stroke_but_never_fill() {
        // イシュー #1590: bar の色は root() が presentation 属性
        // fill="var(--fandhe-color-chart-N)" で系列ごとに与える。SVG の
        // presentation 属性は author origin の specificity 0 のため、
        // recipe() 側に fill を書くと CSS 宣言が presentation 属性より
        // 優先され現時点で全系列が同色に潰れる。CSS 側に fill が無いことを
        // 固定する。
        let out = css();
        let block = bar_rule_block(&out);
        assert!(block.contains("stroke: var(--fandhe-color-bg);"));
        assert!(block.contains("stroke-width: 1;"));
        assert!(!block.contains("fill:"));
    }

    #[test]
    fn category_label_uses_body_font_token() {
        // super::axis の tick-label と同じ書体トークンで整合させる
        // （イシュー #1590）。
        assert!(css().contains("font-family: var(--fandhe-font-font-body);"));
    }

    #[test]
    fn css_has_no_raw_color_literals() {
        let out = css();
        assert!(!out.contains('#'));
        assert!(!out.contains("rgb("));
    }

    #[test]
    fn css_has_no_hover_or_transition_rules() {
        // bar/category-label は表示専用 slot（モジュール doc「イシュー
        // #1590 でのスコープ外判断」参照）。将来 <title> 付与等で hover を
        // 導入する場合はこのテストを意図的に更新すること。
        let out = css();
        assert!(!out.contains(":hover"));
        assert!(!out.contains("transition-"));
    }
}
