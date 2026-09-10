//! ScatterChart（SVG 散布図、イシュー #851・親 Phase #845）。
//!
//! chakra-ui `charts/scatter-chart.md`（recharts `ScatterChart` 相当）を、
//! 外部依存ゼロ・決定的な SVG ノード木生成のみで再構成する。[`super::data::ChartData`]
//! はカテゴリ軸 + 系列値の形状（棒/折れ線向け）であり、散布図が必要とする
//! `(x, y)` 数値ペアの集合を表現できないため、本モジュールは独自に
//! [`ScatterSeries`]/[`ScatterData`] を定義する（`data.rs` は変更しない。
//! `bar_chart`/`radar_chart` 等 [`super::data::ChartData`] を使う並行実装との
//! 競合面を最小化する判断）。
//!
//! # レイアウト規則（決定的。本モジュールが唯一の正）
//!
//! 1. **2 軸線形スケール**: 全系列・全点を横断した x/y それぞれの
//!    `(min, max)` を算出し、[`super::scale::LinearScale::new`] → `nice()`
//!    を経由して `viewBox` の描画領域へ写像する（x: `(0, width)`、
//!    y: `(height, 0)` で SVG の下向き正の y 軸を反転する）。
//! 2. **退化 domain（`min == max`）**: [`super::data::ChartData::domain`] と
//!    同じ [`super::data::flat_domain_pad`]（下限 1.0 と値の大きさに比例した
//!    パディングのうち大きい方を採用し、`v` が `f64` 精度限界付近でも
//!    パディングが丸め誤差で no-op 化しない）を再利用した
//!    [`flat_domain_bounds`] で `v` を中心とした対称区間へ拡張してから
//!    `LinearScale::new` へ渡す（1 点のみ・全点同一座標のデータでも
//!    `ChartError::DegenerateDomain`/`NonFiniteValue` を誘発しない。固定
//!    `±1.0` のみだと `f64::MAX` 付近で退化・非有限化が再発する不具合が
//!    あった、Cursor Bugbot 指摘、イシュー #851 追補）。
//! 3. **数値の文字列化**: ピクセル座標（`cx`/`cy`/`r`/hit-area 幾何）は
//!    [`super::svg::fmt_coord`] のみを経由し、データ値そのもの（ツール
//!    チップ本文の `x`/`y` 表示）は [`super::svg::fmt_value`] のみを経由
//!    する（独自フォーマット禁止、[`crate::charts`] モジュール doc
//!    不変条件 2「ピクセル座標なら `fmt_coord`、データ値なら
//!    `fmt_value`」。イシュー #2129 PR #2261 codex-review P1 是正）。
//! 4. **軸線・グリッド・凡例・ツールチップ**: 本モジュールのスコープ外
//!    （イシュー #847 が担当。ただしイシュー #2129 で hit-area・SSR
//!    ツールチップ DOM の一部を [`root`] 内で担う、下記参照）。
//!
//! # a11y
//!
//! [`super::svg::svg_root`] が既定付与する `role="img"` に加え、呼び出し側
//! 必須の `aria_label` 引数を出力する（`bar_chart` と同型の alt 必須判断）。
//!
//! # セキュリティ不変条件
//!
//! マークアップはすべて [`super::svg`] 経由（`el`/`text` を最終的に呼ぶ）で
//! 組み立て、`raw_html()` は使用しない（REQ-1）。系列名・`aria_label` は
//! テキストノード/属性値として [`fandhe_frontend_core::render`] の既定
//! エスケープを必ず通る。座標・半径は [`ScatterData::new`]/
//! [`super::scale::LinearScale::new`] が有限性検証済みの `f64` のみを
//! [`super::svg::fmt_coord`] へ渡すため、文字列注入経路を持たない。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - 軸線・グリッド・凡例・ツールチップ（#847）。
//! - ホバーインタラクション・アニメーション（意図的非採用、
//!   `docs/policy/intentional-non-adoption.md`）。
//! - `examples/headless-pre-styled-ui` への追随は crates.io 公開後に別途
//!   行う（`qr_code`/`bar_chart` の先例と同じ判断）。
//!
//! # `data-series` 語彙（イシュー #1063）
//!
//! `data-series`（散布点要素へ付与、値は系列名。[`ScatterSeries::name`]
//! フィールド rustdoc も参照）は `fandhe-frontend-headless-ui` に対応部品を
//! 持たない pre-styled-only 語彙である
//! （`docs/design/pre-styled-ui-data-attr-vocabulary.md` 規約 B、
//! [`super::radar_chart`] と共通）。現在の recipe に CSS 消費者はなく、
//! 利用者側 CSS/JS が任意でフックするための識別子に留まる。
//!
//! # 参考サイト基準への調整（イシュー #1598）
//!
//! 親 Phase #1588「Themes / Charts のスタイル調整」の子。参照 4 サイト
//! （chakra-ui / Ark UI / Radix Primitives / Radix Themes）に散布図部品が
//! 存在しないため、評価軸は**内部整合のみ**（`--fandhe-*` トークン適用・
//! ダーク時の可読性・系列色の識別性・データラベルのコントラスト）に
//! 限定する。
//!
//! | 軸 | 結論 |
//! |---|---|
//! | サイズ | 非該当（`ScatterChartProps { width, height, point_radius }`
//!   は viewBox の px 相当長で `Size` variant 軸ではない。新設は 0.x
//!   破壊的変更＝minor バンプ対象で「内部整合のみ」の評価軸を超えるため
//!   非採用、[`super::radar_chart`] と同じ判断） |
//! | バリアント / colorPalette | 非採用（参照軸なし。系列色は
//!   [`series_color_var`] による `chart-1〜6` 固定ローテーション、インライン
//!   `fill`） |
//! | 色 | 現状維持（全宣言がトークン経由。生の色リテラルなし） |
//! | 状態 `data-*` | 非該当（headless-ui 由来の `data-*` を持たない
//!   pre-styled-only 部品。`data-series` は CSS 消費者を持たない識別子の
//!   ままとし増減しない） |
//! | ダークモード | 現状維持（点のハロー `--fandhe-color-bg`・系列色
//!   `chart-1〜6` は dark 値定義済み） |
//! | フォーカス | 非該当（[`super::svg::svg_root`] が `role="img"` を付与し
//!   フォーカス不可） |
//! | 余白・角丸・影 | 非該当（点マーカーの SVG 描画のみ） |
//! | hover / disabled / トランジション | 非採用（表示専用部品、
//!   `role="img"` 配下の `<circle>` はインタラクティブ slot ではない、
//!   `docs/design/pre-styled-ui-interaction-visual-language.md` §3。
//!   インタラクティブな対応物は `charts::tooltip::datum`（#1866 で
//!   hover + transition 適用済み）） |
//! | 内部整合（実欠陥） | **是正**（下記「是正した点」） |
//!
//! ## 是正した点
//!
//! - `point` の `stroke-width` を `1px` から `1`（単位なし）へ表記統一
//!   した。他のチャート系 `decl("stroke-width", ...)`（axis / grid /
//!   tooltip / pie / donut / line / area / radar / sparkline）はすべて
//!   単位なし（SVG ユーザー単位）表記であり、`1px` を使うのは scatter の
//!   `point` のみだった。SVG では `1px` は 1 ユーザー単位として解釈される
//!   ため描画結果に変化はなく、値は不変（#1593 legend の表記統一と同型）
//! - `root` に `overflow: visible` を追加した。`root()` は range を
//!   `point_radius`（`r`）だけ内側へ縮めて円本体が viewBox 内に収まる
//!   ことを保証しているが、ハロー（`stroke-width` の半幅 0.5 ユーザー
//!   単位）は円本体の外側に描かれるため、domain 両端の点（`cx == r`
//!   相当）ではハローの外側 0.5 が viewBox 外へ出る。HTML 内の `<svg>` は
//!   UA 既定 `svg:not(:root) { overflow: hidden }` のためこの部分が
//!   クリップされていた。円本体（`fill`）は欠けないが、card 等の非
//!   `bg` 面ではリング外縁が平らに欠けて見える。兄弟部品 line
//!   （[`crate::line_chart`]、#1595）/ area（[`crate::area_chart`]、
//!   #1589）の `plot` も同じ理由で `overflow: visible` を採っており、
//!   ジオメトリ（`root()` の inset・座標）を変えずに CSS のみで整合を
//!   取った
//!
//! ## 意図的に合わせなかった点
//!
//! - `Size` variant 軸の新設（上表「サイズ」行）
//! - `point` への hover / transition（表示専用、`charts::tooltip::datum`
//!   が担う）
//! - `point` の `fill-opacity` 低減（重なり表現）: 兄弟 line / area の
//!   `point` は不透明であり、重なり識別はハローが担う。chakra/recharts
//!   既定も不透明
//! - `point` への `vector-effect: non-scaling-stroke`（兄弟部品との線幅の
//!   見え方乖離回避、#1593/#1595/#1596/#1597 と同じ判断）
//! - 系列パレット `chart-1〜6` の dark 近接見直し（#1866/#1867/#1870 と
//!   同じくスコープ外）
//! - `data-series` への CSS 消費者追加（系列別スタイルは利用者側フックと
//!   して残す、#1063 の設計）
//!

use super::data::flat_domain_pad;
use super::scale::LinearScale;
use super::svg::{self, fmt_value, ViewBox};
use super::{series_color_var, tooltip, ChartError};
use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};
use fandhe_frontend_headless_ui::fandhe_frontend_core::Node;

/// `data-scope="scatter-chart"` の part 一覧（recipe と揃える）。
const SLOTS: &[&str] = &["root", "point"];

/// 退化 domain（`min == max`）を対称区間へ広げる（[`ScatterData::domain`] の
/// 唯一の呼び出し元）。
///
/// パディング幅は [`super::data::flat_domain_pad`] を再利用し、
/// [`super::data::ChartData::domain`] と同型の「値の大きさに比例した
/// パディング + 下限 1.0」（`v` が `0`/微小値でも退化しない）を散布図側にも
/// 適用する。固定 `±1.0` のみだと `v` が `f64` の精度限界付近
/// （およそ `2^53` 以上）で `v ± 1.0` が丸め誤差により `v` 自身へ丸め戻り
/// no-op になる退化を再現してしまうため（Cursor Bugbot 指摘、イシュー #851
/// 追補、`data.rs` 側の同種修正がイシュー #846 追補）。
///
/// `v ± pad` が非有限（`v` が `±f64::MAX` 付近で `pad` 加算がオーバーフロー
/// する場合）は `ChartData::domain` と同型に `v` 自身へフォールバックする
/// （後続の [`LinearScale::new`] が非有限入力を [`ChartError::NonFiniteValue`]
/// として拒否できるよう、ここでは非有限値を作らない）。
#[must_use]
fn flat_domain_bounds(v: f64) -> (f64, f64) {
    let pad = flat_domain_pad(v);
    let lo = v - pad;
    let hi = v + pad;
    (
        if lo.is_finite() { lo } else { v },
        if hi.is_finite() { hi } else { v },
    )
}

/// 1 系列分の散布点集合。
#[derive(Debug, Clone, PartialEq)]
pub struct ScatterSeries {
    /// 系列名（`data-series` 属性値として出力する。凡例は #847 のスコープ）。
    pub name: String,
    /// `(x, y)` 座標列。
    pub points: Vec<(f64, f64)>,
}

impl ScatterSeries {
    /// 新しい系列を組み立てる（検証なしの薄いコンストラクタ。値の検証は
    /// [`ScatterData::new`] が一括で行う）。
    #[must_use]
    pub fn new(name: impl Into<String>, points: Vec<(f64, f64)>) -> Self {
        ScatterSeries {
            name: name.into(),
            points,
        }
    }
}

/// 散布図のデータモデル（系列の集合。カテゴリ軸を持たない、
/// [`super::data::ChartData`] とは独立した形状）。
///
/// [`ScatterData::new`] を経由した構築のみを公開し、以下を不変条件として
/// 保証する。
///
/// 1. 系列は 1 件以上、かつ全系列合計で点が 1 件以上。
/// 2. 全ての座標が有限（`NaN`/`±inf` を含まない）。
/// 3. 系列名はすべて一意（イシュー #2129、PR #2261 codex-review P1 指摘）。
///    hit-area/tooltip の識別キーは「系列名（`data-series`）+ 系列内の点
///    序数（`data-index`）」のみで構成されるため（`root` 関数 doc
///    参照）、系列名が重複すると異なる系列の点が同じキーへ衝突し
///    hit-area とツールチップの対応付けが一意に定まらなくなる。
#[derive(Debug, Clone, PartialEq)]
pub struct ScatterData {
    series: Vec<ScatterSeries>,
}

impl ScatterData {
    /// 系列群から散布図データを構築する。
    ///
    /// # Errors
    ///
    /// - `series` が空、または全系列合計で点が 0 件の場合
    ///   [`ChartError::EmptyData`]
    /// - いずれかの座標が `NaN`/`±inf` の場合 [`ChartError::NonFiniteValue`]
    /// - 系列名に重複がある場合 [`ChartError::DuplicateSeriesName`]
    ///   （hit-area/tooltip の識別キー〔系列名 + 点序数〕の一意性を構築時
    ///   に保証するため、イシュー #2129 codex-review P1 指摘）
    pub fn new(series: Vec<ScatterSeries>) -> Result<Self, ChartError> {
        if series.is_empty() || series.iter().all(|s| s.points.is_empty()) {
            return Err(ChartError::EmptyData);
        }
        for s in &series {
            if s.points
                .iter()
                .any(|(x, y)| !x.is_finite() || !y.is_finite())
            {
                return Err(ChartError::NonFiniteValue);
            }
        }
        {
            let mut names: Vec<&str> = series.iter().map(|s| s.name.as_str()).collect();
            names.sort_unstable();
            if names.windows(2).any(|pair| pair[0] == pair[1]) {
                return Err(ChartError::DuplicateSeriesName);
            }
        }
        Ok(ScatterData { series })
    }

    /// 系列一覧（挿入順）を返す。
    #[must_use]
    pub fn series(&self) -> &[ScatterSeries] {
        &self.series
    }

    /// 全系列・全点を横断した x/y それぞれの値域 `(min, max)` を返す。
    ///
    /// [`ScatterData::new`] の不変条件により全系列合計で点は必ず 1 件以上
    /// かつ全値有限であるため、本関数は必ず有限な値域を返す。`min == max`
    /// の退化は [`super::data::ChartData::domain`] と同型に対称区間へ
    /// 拡張する（モジュール doc「レイアウト規則」節参照）。
    #[must_use]
    fn domain(&self) -> ((f64, f64), (f64, f64)) {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for s in &self.series {
            for &(x, y) in &s.points {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
        let x_domain = if min_x == max_x {
            flat_domain_bounds(min_x)
        } else {
            (min_x, max_x)
        };
        let y_domain = if min_y == max_y {
            flat_domain_bounds(min_y)
        } else {
            (min_y, max_y)
        };
        (x_domain, y_domain)
    }
}

/// [`root`] の描画パラメータ。
///
/// イシュー #2133 で `range`/`hidden_series`（`String`/`Vec<String>`）を
/// 純追加したため `Copy` は外れ `Clone` のみになった（0.x の破壊的変更）。
#[derive(Debug, Clone, PartialEq)]
pub struct ScatterChartProps {
    /// `viewBox` の幅（px 相当。既定 480.0）。
    pub width: f64,
    /// `viewBox` の高さ（px 相当。既定 300.0）。
    pub height: f64,
    /// 点マーカーの半径（px 相当。既定 4.0）。
    pub point_radius: f64,
    /// `true`（既定）なら hit-area・`data-index` と `hidden` の SSR
    /// ツールチップ DOM（[`super::tooltip::layer`]）を出力する（イシュー
    /// #2129、親 #2128）。`true` の場合、戻り値は素の `<svg data-part=
    /// "root">` ではなく [`super::tooltip::frame`] で包んだ
    /// `<div data-scope="chart" data-part="frame">` になる。`false` の
    /// 場合は本イシュー以前の出力（素の `<svg>`）とバイト一致する
    /// （progressive enhancement の opt-out 経路）。
    pub show_tooltip: bool,
    /// 表示範囲の不透明な識別子（イシュー #2133、親 #2132）。`Some(v)` の
    /// とき root（`svg[data-part="root"]`）へ `data-range="<v>"` を出力
    /// する（既定 `None`＝非出力）。
    pub range: Option<String>,
    /// 非表示系列名の一覧（イシュー #2133）。[`ScatterSeries::name`] と
    /// 完全一致する `point` へ値なし属性 `data-hidden` を付与する。
    /// データに存在しない名前を指定してもエラーにしない（fail-soft）。
    pub hidden_series: Vec<String>,
}

impl Default for ScatterChartProps {
    fn default() -> Self {
        ScatterChartProps {
            width: 480.0,
            height: 300.0,
            point_radius: 4.0,
            show_tooltip: true,
            range: None,
            hidden_series: Vec::new(),
        }
    }
}

/// hit-area 半径 = `point_radius * HIT_AREA_RADIUS_FACTOR`（イシュー
/// #2129）。可視の点マーカーより大きめのヒットターゲットを確保する
/// （ポインタ操作のしやすさのための定数、根拠は WCAG 2.5.5 相当の実務
/// 慣行であり、正確な最小サイズ規定への準拠を主張するものではない）。
const HIT_AREA_RADIUS_FACTOR: f64 = 2.5;

/// この ScatterChart の既定 CSS を組み立てる（内部ヘルパ、[`css`] のみが
/// 呼ぶ）。
///
/// 色は点ごとに [`series_color_var`] のインライン `fill` 属性で決まるため、
/// recipe は寸法系・視認性向上の最小宣言のみを持つ（[`crate::charts::bar_chart`]
/// と同型の「variant を持たない静的部品」判断）。`root` の
/// `overflow: visible` と `point` の `stroke-width` 表記統一は
/// イシュー #1598 の是正（モジュール doc「参考サイト基準への調整」節参照）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("scatter-chart", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "block"),
                decl("max-width", "100%"),
                // 点のハロー（`point` の `stroke-width`）半幅がドメイン端の
                // 点で viewBox 外へはみ出すのを UA 既定 `overflow: hidden`
                // でクリップさせない（円本体は欠けない。イシュー #1598、
                // line/area の `plot` と同型）。
                decl("overflow", "visible"),
            ],
        )
        .base(
            "point",
            vec![
                decl("stroke", "var(--fandhe-color-bg)"),
                // 他のチャート系 slot と同じ単位なし（SVG ユーザー単位）
                // 表記へ統一（イシュー #1598、値は不変）。
                decl("stroke-width", "1"),
            ],
        )
        // イシュー #2133: `hidden_series` で指定した系列の描画要素を
        // 非表示にする（末尾純追加、既存ブロックは不変）。
        .state(
            "point",
            StateCondition::Attr("data-hidden"),
            vec![decl("display", "none")],
        )
}

/// この ScatterChart が生成する静的 CSS 全量を返す（決定的）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// ScatterChart 本体を組み立てる。
///
/// `aria_label` は `svg_root` の `role="img"` に対する代替テキストとして
/// 必須（モジュール doc「a11y」節参照）。
///
/// # Errors
///
/// - `props.width`/`props.height` が 0 以下、または非有限の場合
///   （[`ViewBox::new`] の失敗を変換して）[`ChartError::NonFiniteValue`]
/// - `props.point_radius` が非有限、または 0 以下の場合
///   [`ChartError::NonFiniteValue`]
/// - `props.point_radius * HIT_AREA_RADIUS_FACTOR`（hit-area 半径）が
///   非有限になる場合（PR #2261 codex-review P1 指摘: `point_radius` 単体は
///   有限でも `f64::MAX` 級の極端な値では乗算結果がオーバーフローし
///   `inf` になり得り、`svg::fmt_coord` の有限値契約に違反するため）
///   [`ChartError::NonFiniteValue`]
/// - x/y いずれかの domain 算出後の [`LinearScale::new`] が失敗した場合、
///   その失敗をそのまま返す（[`ChartData::domain`] 同型の退化パディングに
///   より通常は発生しない）
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::charts::scatter_chart::{
///     root, ScatterChartProps, ScatterData, ScatterSeries,
/// };
///
/// let data = ScatterData::new(vec![ScatterSeries::new(
///     "a",
///     vec![(1.0, 2.0), (3.0, 4.0)],
/// )])
/// .unwrap();
/// let node = root(&data, ScatterChartProps::default(), "scatter demo").unwrap();
/// let html = render(&node);
/// assert!(html.contains(r#"data-scope="scatter-chart" data-part="point""#));
/// ```
pub fn root(
    data: &ScatterData,
    props: ScatterChartProps,
    aria_label: &str,
) -> Result<Node, ChartError> {
    if !props.point_radius.is_finite() || props.point_radius <= 0.0 {
        return Err(ChartError::NonFiniteValue);
    }
    // PR #2261 codex-review P1 指摘: `point_radius` 単体の有限性検証だけでは
    // 不十分。`point_radius * HIT_AREA_RADIUS_FACTOR`（hit-area 半径、下記
    // ループ内で使用）は `point_radius` が有限でも極端に大きい場合
    // （例: `f64::MAX / 2` 超）に乗算でオーバーフローし `inf` になり得る。
    // 既定の `show_tooltip: true` 経路で `svg::fmt_coord` の有限値契約に
    // 違反し、debug ビルドでは panic、release ビルドでは不正な `r="inf"`
    // を出力してしまうため、構築前に一括して検証し fail-closed に弾く。
    if !(props.point_radius * HIT_AREA_RADIUS_FACTOR).is_finite() {
        return Err(ChartError::NonFiniteValue);
    }
    let view_box = ViewBox::new(0.0, 0.0, props.width, props.height)
        .map_err(|_| ChartError::NonFiniteValue)?;

    // マーカーは正の半径を持つ円であり、range を viewBox 全体
    // （`0..width`/`height..0`）へそのまま写像すると domain 両端の点が
    // 円の中心座標として境界線上に乗り、半径分だけ viewBox 外へはみ出して
    // クリップされる（Cursor Bugbot 指摘、イシュー #851 追補）。range 側を
    // 内側へ `point_radius` だけ縮めることで、domain 両端の点でも円全体が
    // viewBox 内に収まることを保証する。
    //
    // `width`/`height` が `2 * point_radius` 以下の極端な構成では inset 後の
    // range が反転・縮退し得るが、[`LinearScale::new`] は range の等値・反転を
    // エラーとしない（doc 参照）ため `scale()` は縮退した定数写像として動作し
    // 失敗しない（見た目上のクリップは残るが、それは `point_radius` に対して
    // viewBox が過小という利用側の構成問題であり、本モジュールの責務外）。
    let (x_domain, y_domain) = data.domain();
    let r = props.point_radius;
    let x_scale = LinearScale::new(x_domain, (r, props.width - r))?.nice();
    let y_scale = LinearScale::new(y_domain, (props.height - r, r))?.nice();

    // イシュー #2129: hit-area・SSR ツールチップ DOM。scatter は「系列 ×
    // 点」単位のため `charts::tooltip::entries_from_chart_data` の
    // カテゴリ単位モデルを使わず、`TooltipEntry` を独自に組み立てる
    // （§2.5「scatter は系列 × 点単位」）。`index` は系列内の点序数、
    // `series` は系列表示名（hit-area/point/tooltip の 3 者で共有）。
    let mut points: Vec<Node> = Vec::new();
    let mut hit_areas: Vec<Node> = Vec::new();
    let mut entries: Vec<tooltip::TooltipEntry> = Vec::new();
    for (series_idx, series) in data.series().iter().enumerate() {
        let fill = series_color_var(series_idx);
        // hit-area・tooltip の色は `SeriesColor` 型で保持する（`String` の
        // 任意連結を型で閉じる契約、`tooltip::TooltipRow::color` 参照）。
        // `fill`（point の描画色）と同じ 6 色循環だが、`series.color` の
        // 個別上書きは既存の `fill` 計算（`series_color_var` 直呼び）と
        // 同様に反映しない（scatter の既存挙動を維持、イシュー #2129 の
        // スコープ外）。
        let tooltip_color = super::data::SeriesColor::chart_slot(series_idx % 6 + 1)
            .expect("series_idx % 6 + 1 は常に 1..=6 の範囲内");
        for (point_idx, &(x, y)) in series.points.iter().enumerate() {
            let cx = x_scale.scale(x);
            let cy = y_scale.scale(y);
            let point_idx_str = point_idx.to_string();
            // イシュー #2129 codex-review 指摘: `data-index` は本イシューの
            // 新規追加のため `show_tooltip` が `true` のときのみ付与する
            // （`ScatterChartProps::show_tooltip` rustdoc の「バイト一致」
            // 契約、bar_chart と同型）。`data-series` はイシュー #1063 由来
            // の既存属性のため `show_tooltip` に関わらず常に付与する。
            let mut point_attrs: Vec<(&str, &str)> =
                vec![("data-scope", "scatter-chart"), ("data-part", "point")];
            if props.show_tooltip {
                point_attrs.push(("data-index", point_idx_str.as_str()));
            }
            point_attrs.push(("data-series", series.name.as_str()));
            point_attrs.push(("fill", fill.as_str()));
            if props.hidden_series.contains(&series.name) {
                point_attrs.push(("data-hidden", ""));
            }
            points.push(svg::circle(cx, cy, props.point_radius, point_attrs));

            if props.show_tooltip {
                let entry = tooltip::TooltipEntry {
                    index: point_idx,
                    series: Some(series.name.clone()),
                    label: format!("{} #{}", series.name, point_idx),
                    rows: vec![tooltip::TooltipRow {
                        // `x`/`y` はピクセル座標（`cx`/`cy`）ではなくデータ値
                        // そのものであるため `fmt_value` を経由する
                        // （[`crate::charts`] モジュール doc 不変条件 2、PR
                        // #2261 codex-review P1 指摘: `fmt_coord` では
                        // 小さいデータ値がツールチップ本文で `0` に丸め
                        // 落ちていた）。
                        name: format!("x: {}, y: {}", fmt_value(x), fmt_value(y)),
                        series: series.name.clone(),
                        value: y,
                        color: tooltip_color.clone(),
                    }],
                };
                let label = tooltip::hit_area_label(&entry);
                hit_areas.push(tooltip::hit_area_circle(
                    cx,
                    cy,
                    props.point_radius * HIT_AREA_RADIUS_FACTOR,
                    point_idx,
                    Some(series.name.as_str()),
                    &label,
                ));
                entries.push(entry);
            }
        }
    }
    points.extend(hit_areas);

    let mut root_attrs: Vec<(&str, &str)> = vec![
        ("data-scope", "scatter-chart"),
        ("data-part", "root"),
        ("aria-label", aria_label),
    ];
    if let Some(range) = &props.range {
        root_attrs.push(("data-range", range.as_str()));
    }
    let svg_node = svg::svg_root(&view_box, root_attrs, points);

    if props.show_tooltip {
        Ok(tooltip::frame(vec![
            svg_node,
            tooltip::layer_from_entries(&entries, None),
        ]))
    } else {
        Ok(svg_node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    fn sample_data() -> ScatterData {
        ScatterData::new(vec![
            ScatterSeries::new("a", vec![(0.0, 0.0), (10.0, 20.0)]),
            ScatterSeries::new("b", vec![(5.0, 5.0)]),
        ])
        .unwrap()
    }

    #[test]
    fn scatter_data_rejects_empty_series() {
        assert_eq!(ScatterData::new(vec![]).unwrap_err(), ChartError::EmptyData);
        assert_eq!(
            ScatterData::new(vec![ScatterSeries::new("a", vec![])]).unwrap_err(),
            ChartError::EmptyData
        );
    }

    #[test]
    fn scatter_data_rejects_duplicate_series_names() {
        // イシュー #2129 codex-review P1 指摘: 同名系列を許すと hit-area/
        // tooltip の識別キー（系列名 + 系列内の点序数）が衝突し、異なる
        // 系列の点が同じ `data-index`/`data-series` を持ってしまう。
        // 構築時に fail-closed で拒否することを固定する。
        assert_eq!(
            ScatterData::new(vec![
                ScatterSeries::new("a", vec![(0.0, 0.0)]),
                ScatterSeries::new("a", vec![(1.0, 1.0)]),
            ])
            .unwrap_err(),
            ChartError::DuplicateSeriesName
        );
        // 3 系列中 2 件のみ重複していても検知する。
        assert_eq!(
            ScatterData::new(vec![
                ScatterSeries::new("a", vec![(0.0, 0.0)]),
                ScatterSeries::new("b", vec![(1.0, 1.0)]),
                ScatterSeries::new("a", vec![(2.0, 2.0)]),
            ])
            .unwrap_err(),
            ChartError::DuplicateSeriesName
        );
        // 系列名が一意なら許可される（既存挙動を壊さない）。
        assert!(ScatterData::new(vec![
            ScatterSeries::new("a", vec![(0.0, 0.0)]),
            ScatterSeries::new("b", vec![(1.0, 1.0)]),
        ])
        .is_ok());
    }

    #[test]
    fn scatter_data_rejects_non_finite_points() {
        assert_eq!(
            ScatterData::new(vec![ScatterSeries::new("a", vec![(f64::NAN, 0.0)])]).unwrap_err(),
            ChartError::NonFiniteValue
        );
        assert_eq!(
            ScatterData::new(vec![ScatterSeries::new("a", vec![(0.0, f64::INFINITY)])])
                .unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn root_rejects_non_positive_point_radius() {
        let data = sample_data();
        assert_eq!(
            root(
                &data,
                ScatterChartProps {
                    point_radius: 0.0,
                    ..ScatterChartProps::default()
                },
                "label"
            )
            .unwrap_err(),
            ChartError::NonFiniteValue
        );
        assert_eq!(
            root(
                &data,
                ScatterChartProps {
                    point_radius: f64::NAN,
                    ..ScatterChartProps::default()
                },
                "label"
            )
            .unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn root_rejects_point_radius_whose_hit_area_multiplication_overflows() {
        // PR #2261 codex-review P1 指摘の再現・回帰: `point_radius` 自体は
        // 有限（`is_finite()` を通過）でも、`point_radius * 2.5`
        // （`HIT_AREA_RADIUS_FACTOR`）がオーバーフローして `inf` になる
        // 極端な入力（`width`/`height` も同程度の巨大値で domain/scale の
        // 計算自体は有限のまま通過する構成）で、必ず `ChartError` を返し
        // `svg::fmt_coord` の有限値契約違反（debug panic / release での
        // 不正な `r="inf"` 出力）へ到達しないことを固定する。
        let data = sample_data();
        let huge = f64::MAX / 2.0;
        assert_eq!(
            root(
                &data,
                ScatterChartProps {
                    width: huge,
                    height: huge,
                    point_radius: huge,
                    ..ScatterChartProps::default()
                },
                "label"
            )
            .unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn root_rejects_non_positive_view_box() {
        let data = sample_data();
        assert_eq!(
            root(
                &data,
                ScatterChartProps {
                    width: 0.0,
                    ..ScatterChartProps::default()
                },
                "label"
            )
            .unwrap_err(),
            ChartError::NonFiniteValue
        );
    }

    #[test]
    fn root_renders_role_img_and_aria_label() {
        let data = sample_data();
        let html = render(&root(&data, ScatterChartProps::default(), "scatter demo").unwrap());
        assert!(html.contains(r#"role="img""#));
        assert!(html.contains(r#"aria-label="scatter demo""#));
        assert!(html.contains(r#"data-scope="scatter-chart" data-part="root""#));
    }

    #[test]
    fn root_renders_one_point_per_coordinate() {
        let data = sample_data();
        let html = render(&root(&data, ScatterChartProps::default(), "label").unwrap());
        assert_eq!(
            html.matches(r#"data-part="point""#).count(),
            3,
            "系列 a に 2 点、系列 b に 1 点で合計 3 点"
        );
    }

    #[test]
    fn root_cycles_series_color_var_across_series() {
        let data = sample_data();
        let html = render(&root(&data, ScatterChartProps::default(), "label").unwrap());
        assert!(html.contains("var(--fandhe-color-chart-1)"));
        assert!(html.contains("var(--fandhe-color-chart-2)"));
    }

    #[test]
    fn root_is_deterministic() {
        let data = sample_data();
        let a = render(&root(&data, ScatterChartProps::default(), "label").unwrap());
        let b = render(&root(&data, ScatterChartProps::default(), "label").unwrap());
        assert_eq!(a, b);
    }

    #[test]
    fn root_handles_degenerate_single_point_domain() {
        let data = ScatterData::new(vec![ScatterSeries::new("a", vec![(3.0, 3.0)])]).unwrap();
        let html = render(&root(&data, ScatterChartProps::default(), "label").unwrap());
        assert!(html.contains(r#"data-part="point""#));
    }

    #[test]
    fn root_reserves_point_radius_margin_so_edge_points_are_not_clipped() {
        // domain (0, 10) は `nice()` を経ても不変（境界がすでに 1-2-5 ステップに
        // 揃っている）ため、range を viewBox 全体（0..width/height..0）へ直接
        // 写像すると domain 両端の点の中心座標がちょうど viewBox 境界線上
        // （x=0/x=width、y=height/y=0）に乗り、`point_radius` 分だけ円が
        // viewBox 外へはみ出してクリップされる（Cursor Bugbot 指摘
        // 「Edge scatter points get clipped」の回帰、イシュー #851 追補）。
        // range を `point_radius` だけ内側へ縮めた後は、両端の点でも中心座標が
        // `point_radius` 以上・`width/height - point_radius` 以下に収まり、
        // 円全体が viewBox 内に収まることを固定する。
        let data = ScatterData::new(vec![ScatterSeries::new(
            "a",
            vec![(0.0, 0.0), (10.0, 10.0)],
        )])
        .unwrap();
        let props = ScatterChartProps::default();
        let html = render(&root(&data, props.clone(), "label").unwrap());

        let cx_values: Vec<f64> = html
            .split("cx=\"")
            .skip(1)
            .map(|rest| rest.split('"').next().unwrap().parse().unwrap())
            .collect();
        let cy_values: Vec<f64> = html
            .split("cy=\"")
            .skip(1)
            .map(|rest| rest.split('"').next().unwrap().parse().unwrap())
            .collect();
        // イシュー #2129: hit-area（既定 `show_tooltip: true`）は各点と
        // 同じ中心座標の `<circle>` を追加するため、点 2 + hit-area 2 の
        // 計 4 に純増する（中心座標の値自体は不変）。
        assert_eq!(cx_values.len(), 4);
        assert_eq!(cy_values.len(), 4);
        for &cx in &cx_values {
            assert!(cx >= props.point_radius - 1e-9);
            assert!(cx <= props.width - props.point_radius + 1e-9);
        }
        for &cy in &cy_values {
            assert!(cy >= props.point_radius - 1e-9);
            assert!(cy <= props.height - props.point_radius + 1e-9);
        }
    }

    #[test]
    fn root_handles_flat_domain_at_f64_extreme_magnitude() {
        // 全点が同一座標（x/y とも退化 domain）かつその座標が `f64::MAX`/`MIN`
        // 付近の場合、固定 `±1.0` パディングのみだと丸め誤差で `v` 自身へ
        // 丸め戻る（退化再発）か、桁あふれで `±inf`（`NonFiniteValue`）になる
        // 不具合があった（Cursor Bugbot 指摘、イシュー #851 追補）。
        // `flat_domain_bounds`（`data::flat_domain_pad` 再利用）が両方を
        // 回避し、`root` が成功することを固定する回帰テスト。
        for v in [f64::MAX, f64::MIN] {
            let data = ScatterData::new(vec![ScatterSeries::new("a", vec![(v, v)])]).unwrap();
            let html = render(&root(&data, ScatterChartProps::default(), "label").unwrap());
            assert!(html.contains(r#"data-part="point""#));
        }
    }

    #[test]
    fn root_escapes_series_name_and_aria_label() {
        let data = ScatterData::new(vec![ScatterSeries::new(
            "<script>alert(1)</script>",
            vec![(0.0, 0.0)],
        )])
        .unwrap();
        let html =
            render(&root(&data, ScatterChartProps::default(), "<script>xss</script>").unwrap());
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn tooltip_and_hit_area_label_preserve_small_data_values() {
        // イシュー #2129 codex-review P1 指摘: `fmt_coord`（小数点以下 2 桁
        // 固定）でデータ値を整形すると `0.001` のような小さい値がツール
        // チップ本文・`aria-label` で `0` に丸め落ちる。`fmt_value` 経由に
        // 修正したことを、SSR 出力の文字列に有効数字が残ることで固定する。
        let data = ScatterData::new(vec![ScatterSeries::new("a", vec![(0.001, 0.001)])]).unwrap();
        let html = render(&root(&data, ScatterChartProps::default(), "label").unwrap());
        // `fmt_coord` なら "x: 0, y: 0" になるところ、`fmt_value` では
        // 有効数字を保持した "0.001" になる。
        assert!(html.contains("x: 0.001, y: 0.001"));
        assert!(!html.contains("x: 0, y: 0"));
    }

    #[test]
    fn css_is_deterministic_and_targets_data_scope_selectors() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(a.contains(r#"[data-scope="scatter-chart"][data-part="root"]"#));
        assert!(a.contains(r#"[data-scope="scatter-chart"][data-part="point"]"#));
    }

    #[test]
    fn css_never_contains_style_breakout_sequences() {
        let css = css();
        assert!(!css.contains("</style"));
        assert!(!css.contains('<'));
    }

    /// イシュー #1598 の是正 2 点（モジュール doc「参考サイト基準への調整」
    /// 節参照）を固定する。`stroke-width: 1;`（セミコロン込み）で判定し、
    /// `stroke-width: 1px;` 形との誤一致を避ける。
    #[test]
    fn recipe_includes_issue_1598_corrections() {
        let css = css();
        assert!(css.contains("overflow: visible"));
        assert!(css.contains("stroke-width: 1;"));
        assert!(!css.contains("stroke-width: 1px"));
    }

    // イシュー #2133: 期間切替・凡例トグルの SSR 構造。

    #[test]
    fn range_none_omits_data_range() {
        let html = render(&root(&sample_data(), ScatterChartProps::default(), "range").unwrap());
        assert!(!html.contains("data-range"));
    }

    #[test]
    fn range_some_emits_data_range_on_root() {
        let props = ScatterChartProps {
            range: Some("90d".to_string()),
            ..ScatterChartProps::default()
        };
        let html = render(&root(&sample_data(), props, "range").unwrap());
        assert!(html.contains(r#"data-range="90d""#));
    }

    #[test]
    fn hidden_series_adds_data_hidden_to_matching_points_only() {
        let props = ScatterChartProps {
            hidden_series: vec!["b".to_string()],
            ..ScatterChartProps::default()
        };
        let html = render(&root(&sample_data(), props, "hidden").unwrap());
        let b_idx = html.find(r#"data-series="b""#).unwrap();
        let b_tag_end = html[b_idx..].find('>').unwrap();
        assert!(html[b_idx..b_idx + b_tag_end].contains("data-hidden"));
        let a_idx = html.find(r#"data-series="a""#).unwrap();
        let a_tag_end = html[a_idx..].find('>').unwrap();
        assert!(!html[a_idx..a_idx + a_tag_end].contains("data-hidden"));
    }

    #[test]
    fn hidden_series_unknown_name_is_fail_soft() {
        let props = ScatterChartProps {
            hidden_series: vec!["does-not-exist".to_string()],
            ..ScatterChartProps::default()
        };
        let result = root(&sample_data(), props, "hidden");
        assert!(result.is_ok());
        assert!(!render(&result.unwrap()).contains("data-hidden"));
    }
}
