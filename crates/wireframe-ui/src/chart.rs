//! 棒グラフの配置イメージ部品（`Chart`、イシュー #2663、Phase 8「Media・
//! データ表示」配下）。
//!
//! 数値列を棒の高さ（縦棒）/長さ（横棒）として配置イメージだけを示す、
//! 非インタラクティブなローファイ・プレースホルダー。`<svg>`/`<canvas>`
//! を一切使わず `div` の入れ子だけで組み立てる。実データを描画する
//! グラフが必要な利用者には Themes（`/themes/bar-chart/`・
//! `/themes/charts/`）を案内する（`site/wireframes/chart.md` 参照）。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::chart` showcase
//! （`/wireframes/chart/`）から呼ばれる。`values` は `u8` 列であり
//! `&str` 引数を一切持たないため、既定エスケープ（REQ-1）の対象となる
//! 動的文字列は本モジュールに構造的に存在しない
//! （`crates/wireframe-ui/tests/chart.rs` 冒頭コメント・
//! `site/wireframes/chart.md` の「原案差分メモ」節も参照）。
//!
//! # API 設計の由来
//!
//! `docs/design/wireframe-ui-architecture.md` §2/§7 により、blocks.pm の
//! Figma プロパティ（種別列挙・系列数など）は書き写さない。API は
//! 同文書 §4/§6 の汎用変換規約と、クレート内の既存パターン（
//! [`progress`](crate::progress) の 5 刻み量子化、
//! [`grid::MAX_COLUMNS`](crate::grid::MAX_COLUMNS) の資源有界化、
//! [`props::Orientation`](crate::props::Orientation) の再利用）から
//! 独自に設計した。棒 1 本の値は 0〜100 へクランプしたのち 5 刻みへ
//! 量子化（四捨五入相当）して固定 class（`fw-wire-chart-value-<q>`、
//! 21 種）を付与する。丸め・量子化ロジックは
//! [`progress`](crate::progress)/[`slider`](crate::slider) と重複させて
//! 本モジュール内に独立して持つ（並行実装との衝突を避けるための判断、
//! `progress` の先例に合わせる）。
//!
//! 折れ線・面・円・散布などの種別、凡例、軸ラベル、タイトル、複数系列
//! （グループ化・積み上げ棒）は本部品のスコープ外とする（§8 参照）。
//!
//! # 非対話制約（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7 に従い、ルートは `div`
//! とし `role`/`aria-*`/`tabindex`/`style`/`on*`・`<svg>`/`<canvas>` は
//! 一切出力しない。`data-*` も一切出力しない（表示専用のため
//! `Active`/`Disabled` を持たない）。

use fandhe_frontend_core::{div, el_owned, Node};

use crate::class::class_list;
use crate::props::Orientation;
use crate::size::Size;

/// `values` の上限本数。[`crate::grid::MAX_COLUMNS`] と同じ値を採用する
/// （[`chart`] 参照。これを超える入力は先頭 [`MAX_BARS`] 本だけを描画し、
/// panic しない資源有界化、A05）。
pub const MAX_BARS: usize = 12;

/// プロット領域（軸線を兼ねる）のパート class（部品ルートなしで単独
/// 使用しない、[`chart`] 専用）。
const PLOT_CLASS: &str = "fw-wire-chart-plot";
/// 棒 1 本のパート class（同上）。
const BAR_CLASS: &str = "fw-wire-chart-bar";

/// `quantized`（0/5/…/100 のいずれか）に対応する固定 class を返す。
/// [`crate::progress::value_class`] と同型の全域関数（21 種の固定分岐）。
const fn value_class(quantized: u8) -> &'static str {
    match quantized {
        0 => "fw-wire-chart-value-0",
        5 => "fw-wire-chart-value-5",
        10 => "fw-wire-chart-value-10",
        15 => "fw-wire-chart-value-15",
        20 => "fw-wire-chart-value-20",
        25 => "fw-wire-chart-value-25",
        30 => "fw-wire-chart-value-30",
        35 => "fw-wire-chart-value-35",
        40 => "fw-wire-chart-value-40",
        45 => "fw-wire-chart-value-45",
        50 => "fw-wire-chart-value-50",
        55 => "fw-wire-chart-value-55",
        60 => "fw-wire-chart-value-60",
        65 => "fw-wire-chart-value-65",
        70 => "fw-wire-chart-value-70",
        75 => "fw-wire-chart-value-75",
        80 => "fw-wire-chart-value-80",
        85 => "fw-wire-chart-value-85",
        90 => "fw-wire-chart-value-90",
        95 => "fw-wire-chart-value-95",
        _ => "fw-wire-chart-value-100",
    }
}

/// `value`（0〜255、丸めなし）を 0〜100 へクランプしたうえで 5 刻みへ
/// 量子化した class を返す（[`chart`] から呼ばれる）。
const fn quantized_value_class(value: u8) -> &'static str {
    let clamped = if value > 100 { 100 } else { value };
    // `clamped` は 0..=100 のため `clamped + 2` は最大 102 で u8 に収まる
    // （オーバーフローしない）。
    let quantized = (clamped + 2) / 5 * 5;
    value_class(quantized)
}

/// 棒グラフ CSS（anatomy + value class 21 行）。[`crate::css::PARTS`]
/// へ登録される。
///
/// 太さ・間隔・プロット領域の高さは値を書き写さず
/// [`crate::size::css`] が定義する `--fw-wire-control-size` を `var()`
/// で参照する（`progress`/`slider` と同型の設計）。
pub const CHART_CSS: &str = "\
.fw-wire-chart {
  display: inline-block;
  box-sizing: border-box;
  width: 100%;
  max-width: 32em;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
}
.fw-wire-chart-plot {
  display: flex;
  align-items: flex-end;
  gap: calc(var(--fw-wire-control-size, 2rem) * 0.25);
  box-sizing: border-box;
  border-inline-start: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-block-end: var(--fw-wire-line-width) solid var(--fw-wire-line);
  padding: calc(var(--fw-wire-control-size, 2rem) * 0.25);
  height: calc(var(--fw-wire-control-size, 2rem) * 4);
}
.fw-wire-chart-bar {
  flex: 1 0 auto;
  box-sizing: border-box;
  min-width: calc(var(--fw-wire-control-size, 2rem) * 0.25);
  background: var(--fw-wire-ink-muted, var(--fw-wire-ink));
  border-radius: var(--fw-wire-radius) var(--fw-wire-radius) 0 0;
  height: var(--fw-wire-chart-value);
}
.fw-wire-chart.fw-wire-horizontal .fw-wire-chart-plot {
  flex-direction: column;
  align-items: stretch;
  height: auto;
}
.fw-wire-chart.fw-wire-horizontal .fw-wire-chart-bar {
  flex: 0 0 auto;
  min-width: 0;
  min-height: calc(var(--fw-wire-control-size, 2rem) * 0.25);
  width: var(--fw-wire-chart-value);
  height: calc(var(--fw-wire-control-size, 2rem) * 0.5);
  border-radius: 0 var(--fw-wire-radius) var(--fw-wire-radius) 0;
}
.fw-wire-chart-value-0 { --fw-wire-chart-value: 0%; }
.fw-wire-chart-value-5 { --fw-wire-chart-value: 5%; }
.fw-wire-chart-value-10 { --fw-wire-chart-value: 10%; }
.fw-wire-chart-value-15 { --fw-wire-chart-value: 15%; }
.fw-wire-chart-value-20 { --fw-wire-chart-value: 20%; }
.fw-wire-chart-value-25 { --fw-wire-chart-value: 25%; }
.fw-wire-chart-value-30 { --fw-wire-chart-value: 30%; }
.fw-wire-chart-value-35 { --fw-wire-chart-value: 35%; }
.fw-wire-chart-value-40 { --fw-wire-chart-value: 40%; }
.fw-wire-chart-value-45 { --fw-wire-chart-value: 45%; }
.fw-wire-chart-value-50 { --fw-wire-chart-value: 50%; }
.fw-wire-chart-value-55 { --fw-wire-chart-value: 55%; }
.fw-wire-chart-value-60 { --fw-wire-chart-value: 60%; }
.fw-wire-chart-value-65 { --fw-wire-chart-value: 65%; }
.fw-wire-chart-value-70 { --fw-wire-chart-value: 70%; }
.fw-wire-chart-value-75 { --fw-wire-chart-value: 75%; }
.fw-wire-chart-value-80 { --fw-wire-chart-value: 80%; }
.fw-wire-chart-value-85 { --fw-wire-chart-value: 85%; }
.fw-wire-chart-value-90 { --fw-wire-chart-value: 90%; }
.fw-wire-chart-value-95 { --fw-wire-chart-value: 95%; }
.fw-wire-chart-value-100 { --fw-wire-chart-value: 100%; }
";

/// 棒グラフの配置イメージを組み立てる。
///
/// - `values`: 棒 1 本につき 1 値（0〜255 を想定する `u8`）。先頭
///   [`MAX_BARS`] 本だけを描画する（超過分は無視。panic しない資源
///   有界化）。各値は 100 超は 100 へクランプし、5 刻みへ量子化
///   （四捨五入相当、`42 → 40`・`43 → 45`）してから固定 class
///   （`fw-wire-chart-value-<q>`）を付与する。空スライスのときは棒 0 本の
///   プロット領域だけを描画する。
/// - `orientation`: [`Orientation`]。`Vertical` は棒が上へ伸びる縦棒、
///   `Horizontal`（既定）は右へ伸びる横棒。
/// - `size`: [`Size`] 5 段。太さ・間隔・プロット領域の高さは
///   [`crate::size::css`] が定義する `--fw-wire-control-size` を `var()`
///   で参照する（本モジュールは値を書き写さない）。
///
/// root class の順序は `fw-wire-chart <orientation> <size>`。
///
/// anatomy:
/// - `div.fw-wire-chart.<orientation>.<size>`
///   - `div.fw-wire-chart-plot`（軸線を兼ねるプロット領域）
///     - N 本の `div.fw-wire-chart-bar.fw-wire-chart-value-<q>`
///
/// `role`/`aria-*`/`tabindex`/`style`/`on*`・`<svg>`/`<canvas>`・`data-*`
/// は一切出力しない（`docs/design/wireframe-ui-architecture.md` §5/§7）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{chart, Orientation, Size};
///
/// let node = chart(&[20, 42, 80], Orientation::Vertical, Size::Md);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-chart fw-wire-vertical fw-wire-size-md""#));
/// assert!(html.contains(r#"class="fw-wire-chart-plot""#));
/// assert_eq!(html.matches("fw-wire-chart-bar").count(), 3);
///
/// // 丸め: 42 は 40 へ量子化される。
/// assert!(html.contains("fw-wire-chart-bar fw-wire-chart-value-40"));
///
/// // 100 超は 100 へクランプされる。
/// let clamped = chart(&[255], Orientation::Vertical, Size::Md);
/// assert!(render(&clamped).contains("fw-wire-chart-value-100"));
///
/// // MAX_BARS を超える入力は先頭 12 本だけを描画する。
/// let saturated = chart(&[10; 100], Orientation::Vertical, Size::Md);
/// assert_eq!(render(&saturated).matches("fw-wire-chart-bar").count(), 12);
///
/// // 空スライスでもプロット領域だけを出す（panic しない）。
/// let empty = chart(&[], Orientation::Vertical, Size::Md);
/// let empty_html = render(&empty);
/// assert!(empty_html.contains("fw-wire-chart-plot"));
/// assert!(!empty_html.contains("fw-wire-chart-bar"));
///
/// // 非対話制約: `<svg>`/`<canvas>`/`role`/`tabindex`/`style`/`data-*` は
/// // 一切出力しない（`docs/design/wireframe-ui-architecture.md` §7）。
/// assert!(!html.contains("<svg"));
/// assert!(!html.contains("<canvas"));
/// assert!(!html.contains(" role=\""));
/// assert!(!html.contains("tabindex"));
/// assert!(!html.contains(" style=\""));
/// assert!(!html.contains("data-"));
/// ```
#[must_use]
pub fn chart(values: &[u8], orientation: Orientation, size: Size) -> Node {
    let class = class_list(
        "fw-wire-chart",
        &[Some(orientation.class()), Some(size.class())],
    );

    let bars: Vec<Node> = values
        .iter()
        .take(MAX_BARS)
        .map(|&value| {
            let bar_class = class_list(BAR_CLASS, &[Some(quantized_value_class(value))]);
            el_owned("div", vec![("class".to_string(), bar_class)], vec![])
        })
        .collect();

    let plot = div(vec![("class", PLOT_CLASS)], bars);

    el_owned("div", vec![("class".to_string(), class)], vec![plot])
}
