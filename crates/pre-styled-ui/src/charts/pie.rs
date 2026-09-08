//! 円弧ジオメトリ（角度計算・sector/annulus path 生成、イシュー #850）。
//!
//! [`pie_chart`](crate::pie_chart)/[`donut_chart`](crate::donut_chart)
//! （styled 層、`src/pie_chart.rs`/`src/donut_chart.rs`）の描画対象となる
//! 円弧・扇形・環状セクタの `d` 属性を組み立てる決定的純関数群を提供する。
//! 本モジュール自体はマークアップ（[`fandhe_frontend_core::Node`]）を
//! 生成せず、`String`（`d` 属性値）と角度（`f64`、ラジアン）のみを返す
//! （[`super::data`] の「関心の分離」方針を継承）。
//!
//! # 角度規約
//!
//! - 開始角は 12 時方向（`-π/2`）・時計回り。SVG は y 軸が下向きのため、
//!   角度 `θ` に対し `(cx + r・cos θ, cy + r・sin θ)` は `θ` の増加ととも
//!   に右 → 下 → 左 → 上と時計回りに進む（一般的な円グラフの描画方向）。
//! - セグメント `i` の割合 `f_i = v_i / total`。境界角は与えられた順の
//!   累積和 `θ_i = -π/2 + 2π・Σf` から算出する。
//! - 角度・座標の中間計算は `f64` のまま伝搬し、丸めは文字列化境界の
//!   [`super::svg::fmt_coord`]（小数第 2 位）1 箇所のみで行う（三角関数の
//!   最終 ulp 差は 2 桁丸めで吸収される設計、`svg` モジュール doc 参照）。
//!
//! # 累積誤差の閉じ規則
//!
//! 最終セグメントの終端角は累積和ではなく必ず全周（開始角 + `2π`）に固定
//! する（[`segment_angles`] 参照）。これにより浮動小数の累積誤差による
//! 隙間・重なりを構造的に排除する。
//!
//! # 境界規則
//!
//! - 値 `0` のセグメントは境界角が退化する（`start == end`）。styled 層は
//!   このセグメントに対して path を生成しない契約とする（本モジュールは
//!   角度のみを返し、描画スキップの判断は呼び出し元の責務）。
//! - 非ゼロ値が 1 個のみ（全周セグメント）の場合、[`sector_path`]/
//!   [`annulus_sector_path`] は始点・終点が一致する退化 arc の `d` 属性を
//!   返す（多くの SVG 実装で不可視または不定形になりうる）。styled 層は
//!   この場合を検出し、pie は `circle` 要素、donut は
//!   [`annulus_full_ring_path`]（外周・内周それぞれを独立した閉円として
//!   `fill-rule="evenodd"` で組み合わせる、放射方向の結合線を持たない単一
//!   path）へ切り替えて描画する契約とする（`pie_chart`/`donut_chart`
//!   モジュール doc 参照）。
//! - `large_arc` フラグは弧の中心角が半周（`π`）を超える場合
//!   （`end - start > π`）に真とする。
//!
//! # 角丸端（[`annulus_sector_rounded_path`]、イシュー #2079）
//!
//! [`crate::radial_chart`] の shape/text バリアントが要求する、環状セクタの
//! 弧端（外周・内周それぞれの両端）を丸めた path を組み立てる。角丸半径
//! `rc` は次の 2 段階でクランプする: (1) `rc <= (r_outer - r_inner) / 2`
//! （帯の半分を超える丸めは幾何的に破綻するため）、(2) 弧の中心角
//! （`span = end - start`）が狭く内周コーナー同士が重なる場合
//! （`span < 2·δ_i`。`δ_i = asin(rc / (r_inner + rc))` は内周コーナー円の
//! 中心が占める角度オフセット）はさらに `rc` を縮小する。`rc <= 0` または
//! 非有限、あるいは縮小後に `rc <= 0` となった場合は角丸なし
//! （[`annulus_sector_path`] と同一出力）へフォールバックする。

use super::svg::PathBuilder;
use std::f64::consts::{FRAC_PI_2, PI};

/// 本モジュールおよび styled 層（`pie_chart`/`donut_chart`）が共有する
/// 構築エラー。
///
/// `Display` はユーザーデータの値そのものを含めず、検証に失敗した理由の
/// みを記述する（`.claude/rules/security.md`「機微情報の露出」対応、
/// [`super::ChartError`] と同型の判断）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieChartError {
    /// `values`（またはカテゴリ）が空。
    EmptyData,
    /// いずれかの値が `NaN`/`±inf`。
    NonFiniteValue,
    /// いずれかの値が負（円グラフの割合として意味を持たない）。
    NegativeValue,
    /// 全セグメントの値の合計が `0`（描画すべき弧が存在しない）。
    ZeroTotal,
    /// 系列数が 1 でない（円グラフは単一系列のみを描画対象とする、
    /// `pie_chart`/`donut_chart` モジュール doc 参照）。
    MultiSeries,
    /// [`donut_chart`](crate::donut_chart) の `inner_ratio` が
    /// `0.0 < ratio < 1.0` の範囲外、または非有限。
    InvalidInnerRatio,
}

impl std::fmt::Display for PieChartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            PieChartError::EmptyData => "values must not be empty",
            PieChartError::NonFiniteValue => "value must be finite (NaN/inf is rejected)",
            PieChartError::NegativeValue => "value must not be negative",
            PieChartError::ZeroTotal => "sum of values must be greater than 0",
            PieChartError::MultiSeries => "pie/donut chart accepts exactly one series",
            PieChartError::InvalidInnerRatio => {
                "inner_ratio must be finite and within 0.0 < ratio < 1.0"
            }
        };
        write!(f, "{message}")
    }
}

impl std::error::Error for PieChartError {}

/// 12 時方向（時計盤の 0 分位置）を表す開始角（ラジアン）。
const START_ANGLE: f64 = -FRAC_PI_2;

/// 全周（1 周）を表す角度差（ラジアン）。
const FULL_CIRCLE: f64 = 2.0 * PI;

/// `values` からセグメントごとの境界角 `(start, end)`（ラジアン、モジュール
/// doc の角度規約に従う）を算出する。
///
/// 返り値の要素数は `values.len()` と一致し、順序は `values` の順序を
/// 保持する（色割り当て等で呼び出し元がインデックス対応させる前提）。
///
/// # Errors
///
/// - `values` が空の場合 [`PieChartError::EmptyData`]
/// - いずれかの値が非有限の場合 [`PieChartError::NonFiniteValue`]
/// - いずれかの値が負の場合 [`PieChartError::NegativeValue`]
/// - 合計が `0` の場合 [`PieChartError::ZeroTotal`]
pub fn segment_angles(values: &[f64]) -> Result<Vec<(f64, f64)>, PieChartError> {
    if values.is_empty() {
        return Err(PieChartError::EmptyData);
    }
    if values.iter().any(|v| !v.is_finite()) {
        return Err(PieChartError::NonFiniteValue);
    }
    if values.iter().any(|&v| v < 0.0) {
        return Err(PieChartError::NegativeValue);
    }
    let total: f64 = values.iter().sum();
    // 各要素が有限でも、合計（特に `f64::MAX` 近傍の値同士の和）はオーバー
    // フローして `inf` になりうる。`total <= 0.0` の判定だけでは
    // `inf`（`0.0` より大きい）をすり抜けてしまい、`cumulative / total` が
    // 崩壊した／全周に退化したセグメント角度を生成しスタイル付き
    // pie/donut パスがそのまま描画してしまうため、合計自体の有限性も
    // 個々の値の検証と同じ扱いで拒否する（イシュー #850 レビュー指摘）。
    if !total.is_finite() {
        return Err(PieChartError::NonFiniteValue);
    }
    if total <= 0.0 {
        return Err(PieChartError::ZeroTotal);
    }

    let last_index = values.len() - 1;
    let mut angles = Vec::with_capacity(values.len());
    let mut cumulative = 0.0_f64;
    for (i, &v) in values.iter().enumerate() {
        let start = START_ANGLE + FULL_CIRCLE * (cumulative / total);
        cumulative += v;
        // 累積誤差の閉じ規則: 最終セグメントの終端角は累積和の再計算ではなく
        // 常に「開始角 + 全周」に固定する。これにより浮動小数の丸め誤差が
        // 弧の隙間・重なりとして視覚化される事態を構造的に排除する。
        let end = if i == last_index {
            START_ANGLE + FULL_CIRCLE
        } else {
            START_ANGLE + FULL_CIRCLE * (cumulative / total)
        };
        angles.push((start, end));
    }
    Ok(angles)
}

/// 弧の中心角が半周（`π`）を超えるかどうかを判定する（SVG `large-arc-flag`）。
fn is_large_arc(start: f64, end: f64) -> bool {
    (end - start) > PI
}

/// 中心 `(cx, cy)`・半径 `r`・角度 `theta`（ラジアン、モジュール doc の
/// 角度規約に従う）の円周上の座標を返す。
///
/// `pub(crate)`（イシュー #2079）: [`crate::radial_chart`] のグリッド
/// （同心円境界・放射スポーク）・ラベル配置が本クレート内の他モジュールから
/// 同じ極座標→直交変換を必要とするため公開範囲をクレート内へ広げた
/// （`fmt_coord` 以外の文字列化を持ち込まない不変条件は維持、呼び出し元は
/// 返り値をそのまま `svg::fmt_coord` 経由の属性値へ渡す）。
pub(crate) fn point_on_circle(cx: f64, cy: f64, r: f64, theta: f64) -> (f64, f64) {
    (cx + r * theta.cos(), cy + r * theta.sin())
}

/// 扇形（pie の 1 セグメント）の `d` 属性値を組み立てる
/// （`M 中心 → L 外周始点 → A 外周弧 → Z`）。
///
/// `start`/`end` は [`segment_angles`] が返す境界角をそのまま渡す契約
/// とする（値の検証は [`segment_angles`] 側で完了済みの前提）。
#[must_use]
pub fn sector_path(cx: f64, cy: f64, r: f64, start: f64, end: f64) -> String {
    let (x1, y1) = point_on_circle(cx, cy, r, start);
    let (x2, y2) = point_on_circle(cx, cy, r, end);
    PathBuilder::new()
        .move_to(cx, cy)
        .line_to(x1, y1)
        // sweep=true（時計回り）: モジュール doc の角度規約（θ 増加 = 時計回り）
        // と揃える。
        .arc_to(r, r, 0.0, is_large_arc(start, end), true, x2, y2)
        .close()
        .build()
}

/// 環状セクタ（donut の 1 セグメント）の `d` 属性値を組み立てる
/// （`M 外周始点 → A 外周弧 → L 内周終点 → A 内周弧（逆向き）→ Z`）。
///
/// 内周弧は外周弧と逆方向（`end` → `start`）にたどるため `sweep` フラグを
/// 反転する（`false`）。`r_outer > r_inner > 0` は呼び出し元
/// （[`donut_chart`](crate::donut_chart)）の責務とする。
#[must_use]
pub fn annulus_sector_path(
    cx: f64,
    cy: f64,
    r_outer: f64,
    r_inner: f64,
    start: f64,
    end: f64,
) -> String {
    let (x1, y1) = point_on_circle(cx, cy, r_outer, start);
    let (x2, y2) = point_on_circle(cx, cy, r_outer, end);
    let (x3, y3) = point_on_circle(cx, cy, r_inner, end);
    let (x4, y4) = point_on_circle(cx, cy, r_inner, start);
    let large_arc = is_large_arc(start, end);
    PathBuilder::new()
        .move_to(x1, y1)
        .arc_to(r_outer, r_outer, 0.0, large_arc, true, x2, y2)
        .line_to(x3, y3)
        .arc_to(r_inner, r_inner, 0.0, large_arc, false, x4, y4)
        .close()
        .build()
}

/// 弧端（外周・内周の両端）を丸めた環状セクタの `d` 属性値を組み立てる
/// （[`crate::radial_chart`] の shape/text バリアント、イシュー #2079）。
///
/// モジュール doc「角丸端」節の手順に従う。`rc`（角丸半径、viewBox 単位）
/// が非有限・`0` 以下、または帯厚・弧長からのクランプ後に `0` 以下となった
/// 場合は角丸を諦め [`annulus_sector_path`] と**バイト同一**の出力を返す
/// （呼び出し元が `rc` の事前検証をせずそのまま渡せる fail-soft 設計）。
///
/// `r_outer > r_inner > 0` は [`annulus_sector_path`] と同じく呼び出し元の
/// 責務とする。
#[must_use]
pub fn annulus_sector_rounded_path(
    cx: f64,
    cy: f64,
    r_outer: f64,
    r_inner: f64,
    start: f64,
    end: f64,
    rc: f64,
) -> String {
    if !(rc.is_finite() && rc > 0.0) {
        return annulus_sector_path(cx, cy, r_outer, r_inner, start, end);
    }
    // 手順 2: 帯厚の半分を超える丸めは幾何的に破綻するためクランプする。
    let mut rc = rc.min((r_outer - r_inner) / 2.0);
    if rc <= 0.0 {
        return annulus_sector_path(cx, cy, r_outer, r_inner, start, end);
    }

    // 手順 3: コーナー円中心半径・角度オフセットを算出する（`rc` の値ごとに
    // 再利用するためクロージャ化。手順 4 で `rc` を縮小した場合に同じ計算を
    // 再実行する）。
    let corner_geometry = |rc: f64| {
        let rho_o = r_outer - rc;
        let delta_o = (rc / rho_o).asin();
        let rho_i = r_inner + rc;
        let delta_i = (rc / rho_i).asin();
        (rho_o, delta_o, rho_i, delta_i)
    };

    let span = end - start;
    let (_, _, _, delta_i_initial) = corner_geometry(rc);
    // 手順 4: 弧の中心角が半周（`π`）以上ならコーナー同士は重なり得ないため
    // 縮小不要。半周未満かつ内周コーナーが重なる幅（`span < 2·δ_i`）の場合
    // のみ `rc` を縮小して再計算する。
    if span < PI && span < 2.0 * delta_i_initial {
        let s = (span / 2.0).sin();
        rc = rc.min(r_inner * s / (1.0 - s));
        if rc <= 0.0 {
            return annulus_sector_path(cx, cy, r_outer, r_inner, start, end);
        }
    }
    let (rho_o, delta_o, rho_i, delta_i) = corner_geometry(rc);

    // 外周コーナー・内周コーナーが放射線（`start`/`end`）と接する点の半径
    // （コーナー円の中心からコーナー半径 `rc` を差し引いた直角三角形の
    // 残る一辺）。
    let outer_corner_tangent_r = (rho_o * rho_o - rc * rc).sqrt();
    let inner_corner_tangent_r = (rho_i * rho_i - rc * rc).sqrt();

    let outer_arc_start = point_on_circle(cx, cy, r_outer, start + delta_o);
    let outer_arc_end = point_on_circle(cx, cy, r_outer, end - delta_o);
    let outer_large_arc = is_large_arc(start + delta_o, end - delta_o);
    let end_outer_tangent = point_on_circle(cx, cy, outer_corner_tangent_r, end);
    let end_inner_tangent = point_on_circle(cx, cy, inner_corner_tangent_r, end);
    let inner_arc_end = point_on_circle(cx, cy, r_inner, end - delta_i);
    let inner_arc_start = point_on_circle(cx, cy, r_inner, start + delta_i);
    let inner_large_arc = is_large_arc(start + delta_i, end - delta_i);
    let start_inner_tangent = point_on_circle(cx, cy, inner_corner_tangent_r, start);
    let start_outer_tangent = point_on_circle(cx, cy, outer_corner_tangent_r, start);

    PathBuilder::new()
        .move_to(outer_arc_start.0, outer_arc_start.1)
        .arc_to(
            r_outer,
            r_outer,
            0.0,
            outer_large_arc,
            true,
            outer_arc_end.0,
            outer_arc_end.1,
        )
        .arc_to(
            rc,
            rc,
            0.0,
            false,
            true,
            end_outer_tangent.0,
            end_outer_tangent.1,
        )
        .line_to(end_inner_tangent.0, end_inner_tangent.1)
        .arc_to(rc, rc, 0.0, false, true, inner_arc_end.0, inner_arc_end.1)
        .arc_to(
            r_inner,
            r_inner,
            0.0,
            inner_large_arc,
            false,
            inner_arc_start.0,
            inner_arc_start.1,
        )
        .arc_to(
            rc,
            rc,
            0.0,
            false,
            true,
            start_inner_tangent.0,
            start_inner_tangent.1,
        )
        .line_to(start_outer_tangent.0, start_outer_tangent.1)
        .arc_to(
            rc,
            rc,
            0.0,
            false,
            true,
            outer_arc_start.0,
            outer_arc_start.1,
        )
        .close()
        .build()
}

/// 全周ドーナツ（非ゼロセグメントが 1 個のみ）の `d` 属性値を組み立てる。
///
/// 外周・内周それぞれを独立した閉円（`M → A → A → Z`、始点=終点を通る
/// 180° arc を 2 本つないで 1 周を描く）として出力し、呼び出し元は
/// `fill-rule="evenodd"` を付けて描画する契約とする。2 円の交わりにより
/// evenodd 塗りつぶしがリング状の穴を作るため、[`annulus_sector_path`] を
/// 2 回呼んで組み立てる旧方式（外周半周 + 内周半周を放射方向の `L` で
/// つなぐ 2 本の path）と異なり、環全体を貫く放射方向の線分を一切持たない
/// （放射線分に対する `stroke` がリングの直径上に継ぎ目として見えてしまう
/// 不具合の修正、`donut_chart` モジュール doc「全周セグメントの描画」節）。
#[must_use]
pub fn annulus_full_ring_path(cx: f64, cy: f64, r_outer: f64, r_inner: f64) -> String {
    let outer = full_circle_path(cx, cy, r_outer);
    let inner = full_circle_path(cx, cy, r_inner);
    format!("{outer} {inner}")
}

/// 中心 `(cx, cy)`・半径 `r` の閉円 1 個分の `d` 属性値を組み立てる
/// （180° arc を 2 本つないで 1 周を描く。始点=終点の退化 arc を避けるため
/// [`sector_path`]/[`annulus_sector_path`] のような単一 arc では表現しない）。
fn full_circle_path(cx: f64, cy: f64, r: f64) -> String {
    let (x_right, y_mid) = point_on_circle(cx, cy, r, 0.0);
    let (x_left, _) = point_on_circle(cx, cy, r, PI);
    // 180° ちょうどは `is_large_arc`（`> π` の狭義比較）の判定と揃え
    // `large_arc=false` とする（2 通りの弧が同一長になる境界のため結果は
    // 変わらないが、`is_large_arc` の閾値と表記を一致させる）。
    PathBuilder::new()
        .move_to(x_right, y_mid)
        .arc_to(r, r, 0.0, false, true, x_left, y_mid)
        .arc_to(r, r, 0.0, false, true, x_right, y_mid)
        .close()
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_angles_rejects_empty_values() {
        assert_eq!(segment_angles(&[]).unwrap_err(), PieChartError::EmptyData);
    }

    #[test]
    fn segment_angles_rejects_non_finite_value() {
        assert_eq!(
            segment_angles(&[1.0, f64::NAN]).unwrap_err(),
            PieChartError::NonFiniteValue
        );
        assert_eq!(
            segment_angles(&[1.0, f64::INFINITY]).unwrap_err(),
            PieChartError::NonFiniteValue
        );
    }

    #[test]
    fn segment_angles_rejects_non_finite_total_overflow() {
        // 個々の値は有限（`f64::MAX` 近傍）でも合計がオーバーフローして
        // `inf` になるケース。`total <= 0.0` だけでは `inf` をすり抜ける
        // ため、合計自体の有限性チェックで拒否されることを確認する
        // （イシュー #850 レビュー指摘の回帰テスト）。
        assert_eq!(
            segment_angles(&[f64::MAX, f64::MAX]).unwrap_err(),
            PieChartError::NonFiniteValue
        );
    }

    #[test]
    fn segment_angles_rejects_negative_value() {
        assert_eq!(
            segment_angles(&[1.0, -1.0]).unwrap_err(),
            PieChartError::NegativeValue
        );
    }

    #[test]
    fn segment_angles_rejects_zero_total() {
        assert_eq!(
            segment_angles(&[0.0, 0.0]).unwrap_err(),
            PieChartError::ZeroTotal
        );
    }

    #[test]
    fn segment_angles_matches_known_boundaries_400_300_300_200() {
        // 合計 1200: 割合 1/3・1/4・1/4・1/6 → 120°/90°/90°/60°。
        let angles = segment_angles(&[400.0, 300.0, 300.0, 200.0]).unwrap();
        let expected_deg = [(-90.0, 30.0), (30.0, 120.0), (120.0, 210.0), (210.0, 270.0)];
        for ((start, end), (exp_start, exp_end)) in angles.iter().zip(expected_deg.iter()) {
            assert!((start.to_degrees() - exp_start).abs() < 1e-9);
            assert!((end.to_degrees() - exp_end).abs() < 1e-9);
        }
    }

    #[test]
    fn segment_angles_last_segment_closes_exactly_at_full_circle() {
        // 10 セグメント（各値 1、合計 10）は 1/10 が 2 進数で割り切れず
        // 累積和には丸め誤差が生じ得るが、最終セグメントの終端角は
        // 累積和を経由せず「開始角 + 2π」に固定するため、ビット単位で
        // 一致することを固定する（累積誤差の閉じ規則）。
        let values = vec![1.0; 10];
        let angles = segment_angles(&values).unwrap();
        let (_, last_end) = *angles.last().unwrap();
        assert_eq!(last_end, START_ANGLE + FULL_CIRCLE);
    }

    #[test]
    fn segment_angles_zero_value_segment_is_degenerate() {
        let angles = segment_angles(&[1.0, 0.0, 1.0]).unwrap();
        let (start, end) = angles[1];
        assert!((start - end).abs() < 1e-9);
    }

    #[test]
    fn segment_angles_single_full_circle_segment_is_degenerate_boundary() {
        let angles = segment_angles(&[5.0]).unwrap();
        let (start, end) = angles[0];
        assert!((start - START_ANGLE).abs() < 1e-9);
        assert!((end - (START_ANGLE + FULL_CIRCLE)).abs() < 1e-9);
    }

    #[test]
    fn segment_angles_is_deterministic() {
        let a = segment_angles(&[400.0, 300.0, 300.0, 200.0]).unwrap();
        let b = segment_angles(&[400.0, 300.0, 300.0, 200.0]).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn sector_path_produces_expected_d_for_quarter_circle() {
        // 0°(右) → 90°(下) のクォータ円（r=10、中心 (50,50)）。
        let d = sector_path(50.0, 50.0, 10.0, 0.0, FRAC_PI_2);
        assert_eq!(d, "M50,50 L60,50 A10,10,0,0,1,50,60 Z");
    }

    #[test]
    fn sector_path_sets_large_arc_flag_for_majority_segment() {
        // 240° 分（2/3 周）は large-arc-flag=1 になる。
        let d = sector_path(0.0, 0.0, 1.0, 0.0, 2.0 * PI * (2.0 / 3.0));
        assert!(d.contains(",1,1,"));
    }

    #[test]
    fn annulus_sector_path_uses_reversed_sweep_for_inner_arc() {
        let d = annulus_sector_path(50.0, 50.0, 45.0, 27.0, 0.0, FRAC_PI_2);
        // 外周弧は sweep=1、内周弧は sweep=0（逆向き）。
        let arc_segments: Vec<&str> = d.split(' ').filter(|s| s.starts_with('A')).collect();
        assert_eq!(arc_segments.len(), 2);
        assert!(arc_segments[0].contains(",0,1,"), "outer arc: {d}");
        assert!(arc_segments[1].contains(",0,0,"), "inner arc: {d}");
    }

    #[test]
    fn is_large_arc_boundary_exactly_half_circle_is_false() {
        assert!(!is_large_arc(0.0, PI));
    }

    #[test]
    fn annulus_full_ring_path_has_no_radial_line_segment() {
        // 外周・内周それぞれ独立した閉円（`M`+`A`×2+`Z`）のみで構成され、
        // 環をまたぐ放射方向の `L`（line-to）が存在しないことを固定する
        // （継ぎ目バグの回帰防止、donut_chart モジュール doc 参照）。
        let d = annulus_full_ring_path(50.0, 50.0, 45.0, 27.0);
        assert!(!d.contains('L'));
        assert_eq!(d.matches('M').count(), 2);
        assert_eq!(d.matches('A').count(), 4);
        assert_eq!(d.matches('Z').count(), 2);
    }

    #[test]
    fn annulus_full_ring_path_is_deterministic() {
        let a = annulus_full_ring_path(50.0, 50.0, 45.0, 27.0);
        let b = annulus_full_ring_path(50.0, 50.0, 45.0, 27.0);
        assert_eq!(a, b);
    }

    #[test]
    fn annulus_sector_rounded_path_with_zero_or_non_finite_rc_matches_unrounded() {
        let unrounded = annulus_sector_path(50.0, 50.0, 45.0, 20.0, 0.0, FRAC_PI_2);
        for rc in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                annulus_sector_rounded_path(50.0, 50.0, 45.0, 20.0, 0.0, FRAC_PI_2, rc),
                unrounded,
                "rc={rc}"
            );
        }
    }

    #[test]
    fn annulus_sector_rounded_path_has_four_corner_arcs_and_two_main_arcs() {
        let d = annulus_sector_rounded_path(50.0, 50.0, 45.0, 20.0, 0.0, FRAC_PI_2, 4.0);
        assert_eq!(d.matches('A').count(), 6, "d={d}");
        assert_eq!(d.matches("A4,4,").count(), 4, "d={d}");
    }

    #[test]
    fn annulus_sector_rounded_path_is_deterministic() {
        let a = annulus_sector_rounded_path(50.0, 50.0, 45.0, 20.0, 0.0, FRAC_PI_2, 4.0);
        let b = annulus_sector_rounded_path(50.0, 50.0, 45.0, 20.0, 0.0, FRAC_PI_2, 4.0);
        assert_eq!(a, b);
    }

    #[test]
    fn annulus_sector_rounded_path_clamps_oversized_rc_to_half_band_width() {
        // 帯厚 (45-20)=25 の半分は 12.5。これを大幅に超える rc=100 は
        // クランプ後の rc=12.5 と同一出力になる。
        let clamped = annulus_sector_rounded_path(50.0, 50.0, 45.0, 20.0, 0.0, FRAC_PI_2, 12.5);
        let oversized = annulus_sector_rounded_path(50.0, 50.0, 45.0, 20.0, 0.0, FRAC_PI_2, 100.0);
        assert_eq!(clamped, oversized);
    }

    #[test]
    fn annulus_sector_rounded_path_shrinks_rc_for_narrow_span_without_breaking_structure() {
        // 内周半径が小さく弧が狭い（0.05 rad）と、既定のコーナー半径では
        // 内周コーナー同士が重なるため縮小が発動する。縮小後も 6 本の
        // `A` セグメント構造（コーナー 4 + 主弧 2）を維持することを固定する。
        let d = annulus_sector_rounded_path(50.0, 50.0, 45.0, 5.0, 0.0, 0.05, 4.0);
        assert_eq!(d.matches('A').count(), 6, "d={d}");
        assert!(!d.contains("NaN"), "d={d}");
    }

    #[test]
    fn annulus_sector_rounded_path_output_charset_is_closed() {
        let d = annulus_sector_rounded_path(50.0, 50.0, 45.0, 20.0, 0.0, FRAC_PI_2, 4.0);
        assert!(is_closed_path_charset(&d), "d={d}");
    }

    /// [`super::svg`] のテストヘルパー `is_closed_charset` と同型の判定
    /// （`M`/`L`/`A`/`Z` + 数字・`.`・`-`・空白・`,` のみ）。
    fn is_closed_path_charset(s: &str) -> bool {
        s.chars().all(|c| {
            c.is_ascii_digit()
                || c == '.'
                || c == '-'
                || c == ' '
                || c == ','
                || matches!(c, 'M' | 'L' | 'Z' | 'A')
        })
    }
}
