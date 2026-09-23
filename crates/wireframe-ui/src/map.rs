//! 地図タイルの配置イメージ部品（`Map`、イシュー #2664、Phase 8
//! 「Media・データ表示」配下、[`chart`](crate::chart) に続く部品）。
//!
//! 街路・区画・幹線道路の線画とズーム段階、任意のマーカーだけを示す、
//! 非インタラクティブなローファイ・モノクロのプレースホルダー。実地図
//! タイル（`<img>`・外部 URL）は一切読み込まない。実際の地図表示が
//! 必要な利用者には外部の地図サービスや独自実装を案内する
//! （`site/wireframes/map.md` 参照）。
//!
//! # 呼び出し文脈
//!
//! `fandhe-frontend-docs-site` の `wireframes::map` showcase
//! （`/wireframes/map/`）から呼ばれる。本モジュールは `&str` 引数を
//! 一切持たない（`zoom` は列挙型、`marker` は構築済みの `Node`、`size`
//! は [`Size`]）ため、既定エスケープ（REQ-1）の対象となる動的文字列が
//! 構造的に存在しない（[`chart`](crate::chart) と同じ設計判断、
//! `crates/wireframe-ui/tests/map.rs` 冒頭コメント・
//! `site/wireframes/map.md` の「原案差分メモ」節も参照）。
//!
//! # API 設計の由来
//!
//! `docs/design/wireframe-ui-architecture.md` §2/§7 により、blocks.pm の
//! Figma プロパティ（Zoom の段階値等）は保留中の参照取り込み（#2602）が
//! 解消するまで書き写さない（実装計画時点の判断、視覚的な参照は
//! <https://www.blocks.pm/> への外部リンクに限る）。API は同文書 §4/§6 の
//! 汎用変換規約と、クレート内の既存パターン（[`alert::Severity`]・
//! [`tooltip::TooltipSide`]・[`progress::ProgressShape`] と同型の部品
//! ローカル列挙型、[`link`](crate::link)/[`file_drop`](crate::file_drop)/
//! [`alert`](crate::alert) と同型の `Option<Node>` アイコンスロット規約
//! §11.4）から独自に設計した。
//!
//! [`MapZoom`] は 3 段のズームを表す部品ローカル列挙型で、修飾 class を
//! 1 つだけルートへ付与し、CSS 側で街路グリッドのピッチ
//! （`--fw-wire-map-cell`）だけを切り替える。街路・区画・幹線道路の位置は
//! すべて CSS の固定ルールで描く（Rust 側で乱数・ハッシュは使わない。
//! golden テストと docs デモが純関数であるための決定性）。
//!
//! `marker` は既存の [`crate::icon`] にピン専用のグリフがないため
//! `Option<Node>` スロットとし、呼び出し側が任意の既存アイコン
//! （デモでは [`crate::icon::house`]）を渡す設計とする。**新しいピン
//! アイコンは追加しない**（追加すると `icon::ALL`・
//! `crates/wireframe-ui/tests/icon.rs`・§11.7 の追記が必要になり、
//! 並行 PR との衝突が増えるため）。
//!
//! 地名ラベル・凡例・帰属表示・検索欄・＋/−ズームボタン・実タイル画像や
//! 外部 URL・複数マーカーは本部品のスコープ外とする
//! （`site/wireframes/map.md` 参照）。
//!
//! # 非対話制約（最重要）
//!
//! `docs/design/wireframe-ui-architecture.md` §7 に従い、ルートと各パート
//! は `div` のみで構成し `role`/`aria-*`/`tabindex`/`style`/`on*`・
//! `<img>`/`<iframe>`/`<a>`/`href`/`<svg>`/`<canvas>`/`data-*` は一切
//! 出力しない。ただし `marker` スロットに渡された [`crate::icon`] の
//! `<svg aria-hidden data-icon>` は、スロット由来として許容する
//! （§11.4 の標準スロット規約、[`alert`](crate::alert) と同じ扱い）。

use fandhe_frontend_core::{div, el_owned, Node};

use crate::class::class_list;
use crate::size::Size;

/// 街路の線画パートの class（部品ルートなしで単独使用しない、[`map`] 専用）。
const TILE_CLASS: &str = "fw-wire-map-tile";
/// 公園・水面に見立てた区画パートの class（同上）。
const AREA_CLASS: &str = "fw-wire-map-area";
/// 横方向の幹線道路パートの class（同上）。
const ROAD_H_CLASS: &str = "fw-wire-map-road-h";
/// 縦方向の幹線道路パートの class（同上）。
const ROAD_V_CLASS: &str = "fw-wire-map-road-v";
/// マーカーホストパートの class（同上）。
const MARKER_CLASS: &str = "fw-wire-map-marker";

/// 地図のズーム段階。既定は [`MapZoom::Medium`]。
///
/// モジュール doc「API 設計の由来」節の通り、[`crate::alert::Severity`]・
/// [`crate::tooltip::TooltipSide`] と同じ部品ローカルの型として定義する
/// （`props.rs` へは昇格しない）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MapZoom {
    /// 遠景（街路グリッドを細かく密に描く）。
    Far,
    /// 標準（既定）。
    #[default]
    Medium,
    /// 近景（街路グリッドを粗く疎に描く）。
    Near,
}

impl MapZoom {
    /// 全ズーム段階を宣言順（`Far`〜`Near`）で列挙する。テスト・showcase
    /// が走査に使う。
    pub const ALL: [MapZoom; 3] = [MapZoom::Far, MapZoom::Medium, MapZoom::Near];

    /// このズーム段階に対応する修飾 class（`fw-wire-map-zoom-<段階>`）。
    pub const fn class(self) -> &'static str {
        match self {
            MapZoom::Far => "fw-wire-map-zoom-far",
            MapZoom::Medium => "fw-wire-map-zoom-medium",
            MapZoom::Near => "fw-wire-map-zoom-near",
        }
    }
}

/// 地図 CSS（モノクロ、`ColorPalette` 非依存）。[`crate::css::PARTS`] へ
/// 登録される。
///
/// 街路は `repeating-linear-gradient` を横・縦 2 本重ねて描き、ピッチは
/// `--fw-wire-map-cell`（[`MapZoom`] ごとに切り替わるカスタムプロパティ）
/// で制御する。区画・幹線道路は固定位置・固定太さの矩形として直書きし、
/// 値は [`crate::size::css`] が定義する `--fw-wire-control-size` を
/// `var()` で参照する（[`crate::chart::CHART_CSS`] と同型の設計）。
/// `@keyframes`/`animation`/`url(` は使わない。
pub const MAP_CSS: &str = "\
.fw-wire-map {
  position: relative;
  display: inline-block;
  box-sizing: border-box;
  width: 100%;
  max-width: 32em;
  height: calc(var(--fw-wire-control-size, 2rem) * 6);
  overflow: hidden;
  border: var(--fw-wire-line-width) solid var(--fw-wire-line);
  border-radius: var(--fw-wire-radius);
  background: var(--fw-wire-paper);
}
.fw-wire-map-zoom-far {
  --fw-wire-map-cell: calc(var(--fw-wire-control-size, 2rem) * 0.5);
}
.fw-wire-map-zoom-medium {
  --fw-wire-map-cell: var(--fw-wire-control-size, 2rem);
}
.fw-wire-map-zoom-near {
  --fw-wire-map-cell: calc(var(--fw-wire-control-size, 2rem) * 2);
}
.fw-wire-map-tile {
  position: absolute;
  inset: 0;
  background-image:
    repeating-linear-gradient(
      0deg,
      var(--fw-wire-line-subtle) 0,
      var(--fw-wire-line-subtle) var(--fw-wire-line-width),
      transparent var(--fw-wire-line-width),
      transparent var(--fw-wire-map-cell)
    ),
    repeating-linear-gradient(
      90deg,
      var(--fw-wire-line-subtle) 0,
      var(--fw-wire-line-subtle) var(--fw-wire-line-width),
      transparent var(--fw-wire-line-width),
      transparent var(--fw-wire-map-cell)
    );
}
.fw-wire-map-area {
  position: absolute;
  top: 15%;
  left: 10%;
  width: 30%;
  height: 25%;
  background: var(--fw-wire-fill-subtle);
}
.fw-wire-map-road-h {
  position: absolute;
  top: 55%;
  left: 0;
  width: 100%;
  height: calc(var(--fw-wire-line-width) * 3);
  background: var(--fw-wire-line);
}
.fw-wire-map-road-v {
  position: absolute;
  top: 0;
  left: 40%;
  width: calc(var(--fw-wire-line-width) * 3);
  height: 100%;
  background: var(--fw-wire-line);
}
.fw-wire-map-marker {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  color: var(--fw-wire-ink);
}
.fw-wire-map-marker .fw-wire-icon-glyph {
  width: 1.5em;
  height: 1.5em;
}
";

/// 地図の配置イメージを組み立てる。
///
/// - `zoom`: [`MapZoom`] 3 段（既定 `Medium`）。街路グリッドのピッチだけを
///   切り替える修飾 class を 1 つ付与する。
/// - `marker`: 省略可能なマーカースロット（[`crate::icon::house`] 等の
///   戻り値をそのまま渡す。§11.4 の標準スロット規約）。`None` のときは
///   マーカーのパート要素自体を出力しない。
/// - `size`: [`Size`] 5 段。ルート class `fw-wire-size-<段階>` として
///   付与し、タイル・区画・道路の基準寸法は `--fw-wire-control-size` を
///   `var()` で参照する。
///
/// root class の順序は `fw-wire-map <zoom> <size>`。
///
/// anatomy:
/// - `div.fw-wire-map.<zoom>.<size>`
///   - `div.fw-wire-map-tile`（街路の線画）
///   - `div.fw-wire-map-area`（公園・水面に見立てた区画）
///   - `div.fw-wire-map-road-h`（横方向の幹線道路）
///   - `div.fw-wire-map-road-v`（縦方向の幹線道路）
///   - （`marker` が `Some` のとき）`div.fw-wire-map-marker` > マーカー
///
/// `role`/`aria-*`/`tabindex`/`style`/`on*`・`<img>`/`<iframe>`/`<a>`/
/// `href`/`<svg>`/`<canvas>`/`data-*` は一切出力しない（`marker` スロット
/// 由来の `<svg aria-hidden data-icon>` を除く）。
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_wireframe_ui::map::MapZoom;
/// use fandhe_frontend_wireframe_ui::{icon, map, Size};
///
/// let node = map(MapZoom::Near, Some(icon::house(Size::Md)), Size::Md);
/// let html = render(&node);
/// assert!(html.contains(r#"class="fw-wire-map fw-wire-map-zoom-near fw-wire-size-md""#));
/// assert!(html.contains(r#"class="fw-wire-map-tile""#));
/// assert!(html.contains(r#"class="fw-wire-map-area""#));
/// assert!(html.contains(r#"class="fw-wire-map-road-h""#));
/// assert!(html.contains(r#"class="fw-wire-map-road-v""#));
/// assert!(html.contains(r#"class="fw-wire-map-marker""#));
/// assert!(html.contains("<svg"));
///
/// // marker を省略するとパート要素自体が出力されない。
/// let no_marker = map(MapZoom::Medium, None, Size::Md);
/// let no_marker_html = render(&no_marker);
/// assert!(!no_marker_html.contains("fw-wire-map-marker"));
/// assert!(!no_marker_html.contains("<svg"));
///
/// // 既定は Medium。
/// assert_eq!(MapZoom::default(), MapZoom::Medium);
///
/// // 非対話制約: `<img>`/`<iframe>`/`<a>`/`<canvas>`/`role`/`tabindex`/
/// // `style`/`data-*` は一切出力しない（marker なしの入力）。
/// assert!(!no_marker_html.contains("<img"));
/// assert!(!no_marker_html.contains("<iframe"));
/// assert!(!no_marker_html.contains("<a "));
/// assert!(!no_marker_html.contains("<canvas"));
/// assert!(!no_marker_html.contains(" role=\""));
/// assert!(!no_marker_html.contains("tabindex"));
/// assert!(!no_marker_html.contains(" style=\""));
/// assert!(!no_marker_html.contains("data-"));
/// ```
#[must_use]
pub fn map(zoom: MapZoom, marker: Option<Node>, size: Size) -> Node {
    let class = class_list("fw-wire-map", &[Some(zoom.class()), Some(size.class())]);

    let mut children: Vec<Node> = vec![
        div(vec![("class", TILE_CLASS)], vec![]),
        div(vec![("class", AREA_CLASS)], vec![]),
        div(vec![("class", ROAD_H_CLASS)], vec![]),
        div(vec![("class", ROAD_V_CLASS)], vec![]),
    ];
    if let Some(marker) = marker {
        children.push(el_owned(
            "div",
            vec![("class".to_string(), MARKER_CLASS.to_string())],
            vec![marker],
        ));
    }

    el_owned("div", vec![("class".to_string(), class)], children)
}
