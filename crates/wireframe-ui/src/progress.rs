//! 進捗表示部品（`Progress`、イシュー #2648、Phase 6「Overlay・Feedback」の
//! 2 番目の部品）。
//!
//! バー形・円形の 2 通りで進捗率（%）の配置イメージだけを示す、
//! 非インタラクティブなローファイ・プレースホルダー。`<progress>` 要素・
//! `role="progressbar"`・`aria-valuenow`・`aria-valuemin`/`aria-valuemax`の
//! いずれも実装しない表示専用部品である点に注意する。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::progress` showcase
//! （`/wireframes/progress/`）から呼ばれる。テキスト引数を持たないため
//! （進捗は `u8`）、既定エスケープ（REQ-1）の対象となる動的文字列は
//! 本モジュールに存在しない（構造的に充足、`crates/wireframe-ui/tests/progress.rs`
//! 冒頭コメント・`site/wireframes/progress.md` の「原案差分メモ」節も参照）。
//!
//! # API 設計の由来
//!
//! `docs/design/wireframe-ui-architecture.md` §6 の汎用変換規約から独立
//! 設計した（`site/wireframes/progress.md` の「原案差分メモ」節も参照）。
//! blocks.pm の外観・anatomy・プロパティ構成の実装への転用は書面許諾が
//! 得られるまで保留されている（同文書 §2、イシュー #2602）ため、本部品の
//! 引数構成は blocks.pm の Figma プロパティを参照・書き写さず、一般的な
//! UI キットの線形/円形進捗表示パターンから独自に設計した。
//!
//! 進捗は [`slider`](crate::slider) と同じ規則で `value: u8`（0〜255）を
//! 受け取る。100 を超える値は 100 へクランプし、5 刻みへ量子化（四捨五入
//! 相当）してから固定 class（`fw-wire-progress-value-<q>`、21 種）を付与
//! する。`style="--…: 42%"` のような属性値の動的組み立てや `data-value`
//! の出力は行わない（`class_list` の型制約〔`&'static str` 限定、A03〕と、
//! `docs/design/wireframe-ui-architecture.md` §5「表示状態を示す `data-*`
//! まで」の責務境界に従う）。丸め・量子化ロジックは [`slider`](crate::slider)
//! と重複させて独立に持つ（同モジュールへ触れて並行実装と衝突するのを
//! 避けるための判断、`grid`/`slider` の先例に合わせる）。
//!
//! 形状の切り替えは部品ローカルの列挙型 [`ProgressShape`] で表す（既定は
//! `Bar`）。bool 引数にしないのは、Rust API Guidelines が bool 引数を
//! 推奨していないことと、Figma の boolean プロパティをそのまま写さない
//! ためである。
//!
//! # 非対話制約（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7 に従い、ルートは `div` と
//! し `role`/`aria-*`/`tabindex`/`style`/`on*`・`<progress>` は一切出力
//! しない。`data-*` も一切出力しない（進捗表示は表示専用で、フォーカスや
//! 無効状態に意味がないため `Active`/`Disabled` も持たない）。実際に
//! 操作可能な進捗表示が必要な利用者には Themes（`/themes/progress/`）/
//! Primitives（`/primitives/progress/`）を案内する
//! （`site/wireframes/progress.md` 参照）。

use fandhe_frontend_core::{div, el_owned, Node};

use crate::class::class_list;
use crate::size::Size;

/// トラック（地）のパート class（部品ルートなしで単独使用しない、
/// [`progress`] 専用）。
const TRACK_CLASS: &str = "fw-wire-progress-track";
/// バー形の塗り部分（進捗の視覚化）のパート class（同上）。
const FILL_CLASS: &str = "fw-wire-progress-fill";
/// 円形のくり抜き（リングの内側）のパート class（同上）。
const HOLE_CLASS: &str = "fw-wire-progress-hole";

/// 進捗表示の形状。既定は [`ProgressShape::Bar`]。
///
/// blocks.pm の Figma boolean プロパティをそのまま bool 引数として写さず、
/// 部品ローカルの列挙型として独立設計した（モジュール doc参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProgressShape {
    /// 水平のバー形（既定）。
    #[default]
    Bar,
    /// 円形（`conic-gradient` + くり抜きで表現）。
    Circle,
}

impl ProgressShape {
    /// 付与する修飾 class（`fw-wire-progress-bar` / `fw-wire-progress-circle`）。
    /// `frame` の `fw-wire-frame-bordered` と同型の「部品固有の修飾 class」
    /// （`docs/design/wireframe-ui-architecture.md` §10.1）。
    pub const fn class(self) -> &'static str {
        match self {
            ProgressShape::Bar => "fw-wire-progress-bar",
            ProgressShape::Circle => "fw-wire-progress-circle",
        }
    }
}

/// `value`（0〜100 へ丸め済み）を 5 刻みへ量子化した固定 class を返す。
///
/// 四捨五入相当の丸め（[`quantized_value_class`] が行う）済みの値を
/// 受け取り、`class_list` の型制約（`&'static str` 限定）に合わせ、
/// [`crate::slider::value_class`] と同型の固定分岐で 21 種類
/// （`0`/`5`/`10`/…/`100`）のいずれかを返す全域関数。
const fn value_class(quantized: u8) -> &'static str {
    match quantized {
        0 => "fw-wire-progress-value-0",
        5 => "fw-wire-progress-value-5",
        10 => "fw-wire-progress-value-10",
        15 => "fw-wire-progress-value-15",
        20 => "fw-wire-progress-value-20",
        25 => "fw-wire-progress-value-25",
        30 => "fw-wire-progress-value-30",
        35 => "fw-wire-progress-value-35",
        40 => "fw-wire-progress-value-40",
        45 => "fw-wire-progress-value-45",
        50 => "fw-wire-progress-value-50",
        55 => "fw-wire-progress-value-55",
        60 => "fw-wire-progress-value-60",
        65 => "fw-wire-progress-value-65",
        70 => "fw-wire-progress-value-70",
        75 => "fw-wire-progress-value-75",
        80 => "fw-wire-progress-value-80",
        85 => "fw-wire-progress-value-85",
        90 => "fw-wire-progress-value-90",
        95 => "fw-wire-progress-value-95",
        _ => "fw-wire-progress-value-100",
    }
}

/// `value`（0〜255、丸めなし）を 0〜100 へクランプしたうえで 5 刻みへ
/// 量子化した class を返す（[`progress`] から呼ばれる）。
const fn quantized_value_class(value: u8) -> &'static str {
    let clamped = if value > 100 { 100 } else { value };
    // `clamped` は 0..=100 のため `clamped + 2` は最大 102 で u8 に収まる
    // （オーバーフローしない）。
    let quantized = (clamped + 2) / 5 * 5;
    value_class(quantized)
}

/// 進捗表示 CSS（Bar/Circle の anatomy + value class 21 行）。
/// [`crate::css::PARTS`] へ登録される。
///
/// `--fw-wire-progress-value` は各 `fw-wire-progress-value-<q>` class
/// のみが供給する（root 自体には既定値を宣言しない。[`progress`] が
/// 全域関数の [`quantized_value_class`] を通じて必ず 1 個の value class
/// を付与するため、[`slider`](crate::slider) と同型の設計）。太さ・直径は
/// 値を書き写さず [`crate::size::css`] が定義する `--fw-wire-control-size`
/// を `var()` で参照する。
pub const PROGRESS_CSS: &str = "\
.fw-wire-progress {
  display: inline-block;
  box-sizing: border-box;
  width: 100%;
  max-width: 24em;
  font-family: var(--fw-wire-font-family);
  font-size: var(--fw-wire-font-size, 1rem);
  color: var(--fw-wire-ink);
}
.fw-wire-progress-track {
  position: relative;
  box-sizing: border-box;
  background: var(--fw-wire-fill);
  border-radius: 999px;
  height: calc(var(--fw-wire-control-size, 2rem) * 0.25);
}
.fw-wire-progress-fill {
  position: absolute;
  inset-block: 0;
  left: 0;
  width: var(--fw-wire-progress-value);
  background: var(--fw-wire-ink);
  border-radius: inherit;
}
.fw-wire-progress.fw-wire-progress-circle {
  width: auto;
}
.fw-wire-progress.fw-wire-progress-circle .fw-wire-progress-track {
  position: relative;
  box-sizing: border-box;
  width: calc(var(--fw-wire-control-size, 2rem) * 2);
  height: calc(var(--fw-wire-control-size, 2rem) * 2);
  border-radius: 50%;
  background: conic-gradient(
    var(--fw-wire-ink) var(--fw-wire-progress-value),
    var(--fw-wire-fill) 0
  );
}
.fw-wire-progress-hole {
  position: absolute;
  inset: calc(var(--fw-wire-control-size, 2rem) * 0.3);
  box-sizing: border-box;
  border-radius: 50%;
  background: var(--fw-wire-paper);
}
.fw-wire-progress-value-0 { --fw-wire-progress-value: 0%; }
.fw-wire-progress-value-5 { --fw-wire-progress-value: 5%; }
.fw-wire-progress-value-10 { --fw-wire-progress-value: 10%; }
.fw-wire-progress-value-15 { --fw-wire-progress-value: 15%; }
.fw-wire-progress-value-20 { --fw-wire-progress-value: 20%; }
.fw-wire-progress-value-25 { --fw-wire-progress-value: 25%; }
.fw-wire-progress-value-30 { --fw-wire-progress-value: 30%; }
.fw-wire-progress-value-35 { --fw-wire-progress-value: 35%; }
.fw-wire-progress-value-40 { --fw-wire-progress-value: 40%; }
.fw-wire-progress-value-45 { --fw-wire-progress-value: 45%; }
.fw-wire-progress-value-50 { --fw-wire-progress-value: 50%; }
.fw-wire-progress-value-55 { --fw-wire-progress-value: 55%; }
.fw-wire-progress-value-60 { --fw-wire-progress-value: 60%; }
.fw-wire-progress-value-65 { --fw-wire-progress-value: 65%; }
.fw-wire-progress-value-70 { --fw-wire-progress-value: 70%; }
.fw-wire-progress-value-75 { --fw-wire-progress-value: 75%; }
.fw-wire-progress-value-80 { --fw-wire-progress-value: 80%; }
.fw-wire-progress-value-85 { --fw-wire-progress-value: 85%; }
.fw-wire-progress-value-90 { --fw-wire-progress-value: 90%; }
.fw-wire-progress-value-95 { --fw-wire-progress-value: 95%; }
.fw-wire-progress-value-100 { --fw-wire-progress-value: 100%; }
";

/// 進捗表示を組み立てる。
///
/// - `value`: 進捗（0〜100 を想定する `u8`）。100 超は 100 へクランプし、
///   5 刻みへ量子化（四捨五入相当、`42 → 40`・`43 → 45`）してから固定
///   class（`fw-wire-progress-value-<q>`）を付与する。丸め規則は
///   [`value_class`] を参照。
/// - `shape`: [`ProgressShape`] バー形（既定）/円形。ルート class
///   `fw-wire-progress-bar`/`fw-wire-progress-circle` として付与する。
/// - `size`: [`Size`] 5 段。太さ・直径は [`crate::size::css`] が定義する
///   `--fw-wire-control-size` を `var()` で参照する（本モジュールは値を
///   書き写さない）。
///
/// root class の順序は `fw-wire-progress <shape> <size> <value>`。
///
/// anatomy:
/// - Bar: `div.fw-wire-progress.fw-wire-progress-bar` の中に
///   `div.fw-wire-progress-track`、その中に `div.fw-wire-progress-fill`
/// - Circle: `div.fw-wire-progress.fw-wire-progress-circle` の中に
///   `div.fw-wire-progress-track`、その中に `div.fw-wire-progress-hole`
///   （リングの内側をくり抜く部分）
///
/// `role`/`aria-*`/`tabindex`/`style`/`on*`・`<progress>`・`data-*` は
/// 一切出力しない（`docs/design/wireframe-ui-architecture.md` §5/§7）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::{progress, ProgressShape, Size};
///
/// let node = progress(80, ProgressShape::Bar, Size::Md);
/// let html = render(&node);
/// assert!(html.contains(
///     r#"class="fw-wire-progress fw-wire-progress-bar fw-wire-size-md fw-wire-progress-value-80""#
/// ));
/// assert!(html.contains(r#"class="fw-wire-progress-track""#));
/// assert!(html.contains(r#"class="fw-wire-progress-fill""#));
///
/// // 丸め: 42 は 40 へ量子化される。
/// let rounded = progress(42, ProgressShape::Bar, Size::Md);
/// assert!(render(&rounded).contains("fw-wire-progress-value-40"));
///
/// // 100 超は 100 へクランプされる。
/// let clamped = progress(255, ProgressShape::Bar, Size::Md);
/// assert!(render(&clamped).contains("fw-wire-progress-value-100"));
///
/// // Circle は hole パートを持つ。
/// let circle = progress(80, ProgressShape::Circle, Size::Md);
/// let circle_html = render(&circle);
/// assert!(circle_html.contains("fw-wire-progress-circle"));
/// assert!(circle_html.contains(r#"class="fw-wire-progress-hole""#));
///
/// // 非対話制約: `<progress>`/`role`/`tabindex`/`style`/`data-*` は
/// // 一切出力しない（`docs/design/wireframe-ui-architecture.md` §7）。
/// assert!(!circle_html.contains("<progress"));
/// assert!(!circle_html.contains(" role=\""));
/// assert!(!circle_html.contains("tabindex"));
/// assert!(!circle_html.contains(" style=\""));
/// assert!(!circle_html.contains("data-value"));
/// ```
#[must_use]
pub fn progress(value: u8, shape: ProgressShape, size: Size) -> Node {
    let class = class_list(
        "fw-wire-progress",
        &[
            Some(shape.class()),
            Some(size.class()),
            Some(quantized_value_class(value)),
        ],
    );

    let inner = match shape {
        ProgressShape::Bar => div(vec![("class", FILL_CLASS)], vec![]),
        ProgressShape::Circle => div(vec![("class", HOLE_CLASS)], vec![]),
    };

    let track = div(vec![("class", TRACK_CLASS)], vec![inner]);

    el_owned("div", vec![("class".to_string(), class)], vec![track])
}
