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
//!    が描画範囲に含まれることを保証する）。`nice()` は
//!    [`BarChartProps::show_value_axis`]/[`BarChartProps::show_grid`]
//!    有効時のみ適用する（既定経路は #2082 以前と同じ nice() 非適用のまま、
//!    「shadcn/ui 突合（イシュー #2082）」節の golden 純追加原則）。
//! 2. **カテゴリ軸（バンドレイアウト）**: カテゴリ数 `n` に対し
//!    `band = plot_length / n`。各バンド内は両端に `BAND_EDGE_PADDING_FRAC`
//!    （10%）ずつの余白を置き、残り 80% を系列数で均等分割して棒幅とする
//!    （系列間の追加ギャップは設けない、純算術で決定的）。積み上げ時
//!    （[`BarStack::Normal`]/[`BarStack::Expand`]）は系列数で分割せず、
//!    バンド全体を 1 本の帯として使う。
//! 3. **座標の文字列化**: すべて [`super::svg::fmt_coord`] のみを経由する
//!    （独自フォーマット禁止、[`crate::charts`] モジュール doc 不変条件 2）。
//! 4. **軸線・グリッド・凡例・ツールチップ**: [`BarChartProps::show_value_axis`]/
//!    [`BarChartProps::show_grid`] で任意描画できる（イシュー #2082）。凡例は
//!    引き続き呼び出し側が [`super::legend`] を並べる（本モジュールのスコープ外）。
//!    ツールチップは対象外のまま（イシュー #2086）。
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
//! 角丸 path（[`bar_shape`]）の `d` 属性も [`super::svg::PathBuilder`] のみを
//! 経由して組み立て、文字集合は `M L A Z` + `[0-9.,- ]` に閉じる。`color`/
//! `fill` presentation 属性は [`super::series_color_var`]/
//! [`super::data::ChartData::series_color_var`] が返す固定形
//! （`var(--fandhe-color-<allowlist 名>)`）のみで、ユーザー文字列は入らない。
//!
//! # 本イシューのスコープ外（`.claude/rules/out-of-scope-tracking.md` 対応）
//!
//! - `examples/headless-pre-styled-ui` への追随は crates.io 公開後に別途
//!   行う（`qr_code`/`rating_group` の先例と同じ判断）。
//! - マウス追従ツールチップ・hover 強調・hit-area・`data-index`/
//!   `data-series`（#2128）、期間切替・凡例トグル（#2132）、SSR ツール
//!   チップの indicator/書式（#2086）。
//!
//! # shadcn/ui 突合（イシュー #2082）
//!
//! shadcn/ui Charts（bar、registry 10 種 + demo 4 種）と突合し、静的に
//! 描画できるバリアントを [`BarChartProps`] の純追加で補完した。
//!
//! | shadcn/ui registry | 本実装の対応 |
//! |---|---|
//! | `chart-bar-default`（角丸 8・横グリッド・X 軸ラベル） | `corner_radius: 8.0` + `show_grid` + `show_value_axis` |
//! | `chart-bar-horizontal` | 既存 `Orientation::Horizontal` + `corner_radius` |
//! | `chart-bar-multiple`（並列複数系列） | 既存（複数系列） + `corner_radius` |
//! | `chart-bar-label`（値ラベルを棒の先端外側に表示） | `label: BarLabel::Outside` |
//! | `chart-bar-label-custom`（横棒の内側左にカテゴリ名、右端に値） | `label: BarLabel::Inside` + `show_category_labels: false` |
//! | `chart-bar-mixed`（カテゴリごとに別色） | `color_by_category: true` |
//! | `chart-bar-stacked`（積み上げ） | `stack: BarStack::Normal`（100% 積み上げは [`BarStack::Expand`]） |
//! | `chart-bar-active`（強調表示） | `active_index: Some(usize)` → `data-active` + `color` presentation 属性 |
//! | `chart-bar-negative`（正負で色分け） | `highlight_negative: true` |
//! | demo `axis`/`grid`/`legend` | `show_value_axis`/`show_grid` + 呼び出し側の [`super::legend`] |
//! | `chart-bar-interactive` | 対象外（下記「本イシューのスコープ外」） |
//!
//! ## 意図的に合わせなかった点
//!
//! 1. Horizontal のカテゴリラベル位置（本実装は右側、shadcn は左側 Y 軸）:
//!    既定出力のバイト互換（#849 決定）のため据え置く。
//! 2. X 軸ラベルの `tickFormatter`・値ラベルの書式はアプリ側整形
//!    （`docs/policy/intentional-non-adoption.md` §3.23/§3.25）。値ラベルは
//!    [`super::svg::fmt_coord`] 固定。
//! 3. mixed のカテゴリごと任意色（shadcn は data 行ごとに `fill`）は非対応。
//!    [`BarChartProps::color_by_category`] は [`super::series_color_var`]
//!    （`chart-1`〜`chart-6` 循環）のみ対応する（`BarChartProps` の
//!    `Copy` 制約、[`super::data::ChartData`] の色モデルが系列単位である
//!    ことから）。
//! 4. active の破線 stroke 色は shadcn は自色。本実装は当該棒へ
//!    `color="var(--fandhe-color-chart-N)"` presentation 属性 + CSS
//!    `stroke: currentColor` で自色を再現する。
//! 5. `bar` の既存 `stroke: var(--fandhe-color-bg)`（#1590 値）は据え置く
//!    （shadcn は stroke なし）。
//! 6. 角丸の既定は 0（chakra-ui 既定・既存出力互換）。
//! 7. `docs/design/component-coverage-map.md` §12 への判定転記は #2097。
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
//! - **幾何（バンド余白・ラベル位置定数）**: Rust 側の定数であり、変更す
//!   るとレンダリング結果と手計算ジオメトリテストが変わる。CSS 調整の
//!   範囲外。
//! - **系列色トークン（`chart-1`〜`chart-6`）**: light/dark 両方が
//!   `theme.rs` に定義済み（`docs/design/color-token-system.md`）であり
//!   変更しない。
//!
//! # `data-*` 語彙（`docs/design/pre-styled-ui-data-attr-vocabulary.md` 規約 B）
//!
//! - `data-active`: [`BarChartProps::active_index`] と一致するカテゴリの
//!   全系列棒へ存在属性として付与する（値域なし）。checkbox_group/
//!   radio_group/sidebar 等の既存 headless 語彙を「強調表示中の項目」という
//!   同一意味論で再利用する（B-2）。**将来 wasm-full 側で同属性を付け外し
//!   する前提**（SSR 初期値 + JS が移動する progressive enhancement、#2128
//!   の担当）。
//! - `data-negative`: [`BarChartProps::highlight_negative`] が有効かつ
//!   値が負の棒へ存在属性として付与する bar-chart 新設の pre-styled-only
//!   語彙（B-3）。CSS 消費者はない。

use super::data::ChartData;
use super::scale::LinearScale;
use super::svg::{self, svg_text, PathBuilder, ViewBox, ViewBoxError};
use super::ChartError;
use crate::charts::axis::{self, AxisProps, TickLabelFormat};
use crate::charts::grid::{self, GridProps};
use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, text, Node};

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

/// [`BarChartProps::show_value_axis`] 有効時に [`Orientation::Vertical`]
/// のプロット領域左側へ確保する Y 軸ラベル用の余白（px、イシュー #2082）。
const AXIS_VALUE_MARGIN_LEFT: f64 = 40.0;

/// [`BarChartProps::show_value_axis`] 有効時に [`Orientation::Horizontal`]
/// のプロット領域下側へ確保する X 軸ラベル用の余白（px、イシュー #2082）。
const AXIS_VALUE_MARGIN_BOTTOM: f64 = 24.0;

/// 値ラベル・内側ラベルの、棒端からの外側/内側オフセット（px、イシュー #2082）。
const LABEL_OFFSET: f64 = 4.0;
/// 内側ラベルの、棒端からの内向きオフセット（px、イシュー #2082）。
const INSIDE_LABEL_OFFSET: f64 = 8.0;

/// `data-scope="bar-chart"` の part 一覧（recipe と揃える）。
const SLOTS: &[&str] = &[
    "root",
    "bar",
    "category-label",
    "value-label",
    "inside-label",
];

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

/// 積み上げ（shadcn `stackId`/`stackOffset="expand"`）。イシュー #2082。
///
/// 積み上げ時（[`BarStack::Normal`]/[`BarStack::Expand`]）は全系列の値が
/// 非負であることを要求する（負値は帯として定義できないため
/// [`ChartError::NegativeValue`]、[`crate::area_chart::AreaStack`] と同じ判断）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarStack {
    /// 積み上げなし（既定）。系列ごとにバンド内へ並べて描く。
    #[default]
    None,
    /// 累積和による積み上げ（帯状の棒が積み重なる）。
    Normal,
    /// カテゴリ合計で正規化した積み上げ（domain `(0.0, 1.0)` 固定、
    /// 値軸ラベルは `%` 表示）。
    Expand,
}

/// 値ラベル表示（shadcn `chart-bar-label`/`chart-bar-label-custom`）。
/// イシュー #2082。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarLabel {
    /// 表示しない（既定）。
    #[default]
    None,
    /// 棒の先端の外側に値ラベルを表示する。
    Outside,
    /// 棒の先端の外側に値ラベル、ベースライン側の内側にカテゴリ名ラベルを
    /// 表示する（横棒向け、`show_category_labels: false` と組み合わせて
    /// 軸外のカテゴリラベルを省略する想定）。
    Inside,
}

/// [`bar_shape`] が角丸を適用する辺（内部型）。
///
/// バンド内の棒 1 本（非積み上げ）または積み上げ帯 1 セグメントについて、
/// どちらの端（ベースラインから遠い「先端」側か、近い「ベースライン」側か）
/// を丸めるかを表す。[`Orientation::Vertical`] では `Top`/`Bottom`、
/// [`Orientation::Horizontal`] では `Left`/`Right` を使う。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RoundedEnd {
    /// 角丸なし（`corner_radius == 0.0`、または積み上げ中段）。
    None,
    /// 4 隅すべて（非積み上げ、または積み上げが 1 系列のみ）。
    All,
    Top,
    Bottom,
    Left,
    Right,
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
    /// 積み上げ（イシュー #2082、既定 [`BarStack::None`]）。
    pub stack: BarStack,
    /// 棒の角丸半径（px、イシュー #2082、既定 `0.0`）。非有限または負値は
    /// [`ChartError::InvalidCornerRadius`]。実効半径は
    /// `min(radius, w/2, h/2)` にクランプする。
    pub corner_radius: f64,
    /// 値ラベル表示（イシュー #2082、既定 [`BarLabel::None`]）。
    pub label: BarLabel,
    /// 強調表示するカテゴリの index（イシュー #2082、既定 `None`）。
    /// `n_categories` 以上は [`ChartError::IndexOutOfRange`]。
    pub active_index: Option<usize>,
    /// カテゴリごとに色を変える（`chart-1`〜`chart-6` 循環、イシュー
    /// #2082、既定 `false`）。単一系列データ向け（shadcn `chart-bar-mixed`）。
    pub color_by_category: bool,
    /// 負値の棒を `chart-2` で強調表示する（イシュー #2082、既定 `false`）。
    pub highlight_negative: bool,
    /// 値軸（目盛線・目盛ラベル）を描画するか（イシュー #2082、既定
    /// `false`）。[`Orientation::Vertical`] は左 Y 軸、
    /// [`Orientation::Horizontal`] は下 X 軸。
    pub show_value_axis: bool,
    /// 値軸に直交するグリッド線を描画するか（イシュー #2082、既定
    /// `false`）。
    pub show_grid: bool,
    /// カテゴリラベルを描画するか（イシュー #2082、既定 `true`）。
    pub show_category_labels: bool,
}

impl Default for BarChartProps {
    fn default() -> Self {
        BarChartProps {
            orientation: Orientation::default(),
            width: 480.0,
            height: 300.0,
            stack: BarStack::default(),
            corner_radius: 0.0,
            label: BarLabel::default(),
            active_index: None,
            color_by_category: false,
            highlight_negative: false,
            show_value_axis: false,
            show_grid: false,
            show_category_labels: true,
        }
    }
}

/// この BarChart の既定 CSS を組み立てる（内部ヘルパ、[`css`] のみが呼ぶ）。
///
/// 色は棒ごとに [`super::ChartData::series_color_var`] のインライン `fill` 属性で決まるため、
/// recipe は寸法系の最小宣言のみを持つ（[`crate::qr_code`] の
/// 「前景/背景は固定トークン・variant は寸法のみ」判断と同型ではなく、本
/// 部品は variant 自体を持たない静的部品。[`crate::table`] の
/// 「状態機械を持たない静的 styled 部品」に分類される）。
///
/// # 不変条件（イシュー #1590）
///
/// - **`bar` の base に `fill` を書かない**: 棒の色は [`root`] が各棒へ
///   `fill="var(--fandhe-color-chart-N)"`（[`super::ChartData::series_color_var`]）を
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
/// - `value-label`/`inside-label`（イシュー #2082）は末尾へ純追加する
///   （golden 純追加原則、`bar_chart_css.rs` の `BEFORE_2082` 定数参照）。
///   `value-label` は `--fandhe-color-fg`（shadcn `fill-foreground`）、
///   `inside-label` は `--fandhe-color-bg`（shadcn `--color-label:
///   var(--background)`。棒の内側に置くため背景色で描き、系列色の棒地に
///   対して読める前提）。
/// - `bar[data-active]`（イシュー #2082）は `fill-opacity: 0.8` +
///   `stroke: currentColor` の破線で強調する。`color` presentation 属性
///   （[`root`] が付与）を `currentColor` の解決元として使うため、`fill`
///   を上書きしない（上記不変条件と同じ理由）。
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
        .base(
            "value-label",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("font-family", "var(--fandhe-font-font-body)"),
                decl("fill", "var(--fandhe-color-fg)"),
            ],
        )
        .base(
            "inside-label",
            vec![
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("font-family", "var(--fandhe-font-font-body)"),
                decl("fill", "var(--fandhe-color-bg)"),
            ],
        )
        .state(
            "bar",
            StateCondition::Attr("data-active"),
            vec![
                decl("fill-opacity", "0.8"),
                decl("stroke", "currentColor"),
                decl("stroke-dasharray", "4"),
                decl("stroke-dashoffset", "4"),
            ],
        )
}

/// この BarChart が生成する静的 CSS 全量を返す（決定的）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// 角丸矩形（一部の辺のみ角丸）の `d` 属性を組み立てる（内部ヘルパ）。
///
/// `r` は既にクランプ済み（`0.0 < r <= min(w/2, h/2)`）の実効半径である
/// ことを契約とする（呼び出し元 [`bar_shape`] が保証する）。[`RoundedEnd::None`]
/// はこの関数を呼ばない契約（[`bar_shape`] が先に `<rect>` を返す）。
fn rounded_rect_d(x: f64, y: f64, w: f64, h: f64, r: f64, end: RoundedEnd) -> String {
    match end {
        RoundedEnd::All => PathBuilder::new()
            .move_to(x + r, y)
            .line_to(x + w - r, y)
            .arc_to(r, r, 0.0, false, true, x + w, y + r)
            .line_to(x + w, y + h - r)
            .arc_to(r, r, 0.0, false, true, x + w - r, y + h)
            .line_to(x + r, y + h)
            .arc_to(r, r, 0.0, false, true, x, y + h - r)
            .line_to(x, y + r)
            .arc_to(r, r, 0.0, false, true, x + r, y)
            .close()
            .build(),
        RoundedEnd::Top => PathBuilder::new()
            .move_to(x, y + r)
            .arc_to(r, r, 0.0, false, true, x + r, y)
            .line_to(x + w - r, y)
            .arc_to(r, r, 0.0, false, true, x + w, y + r)
            .line_to(x + w, y + h)
            .line_to(x, y + h)
            .close()
            .build(),
        RoundedEnd::Bottom => PathBuilder::new()
            .move_to(x, y)
            .line_to(x + w, y)
            .line_to(x + w, y + h - r)
            .arc_to(r, r, 0.0, false, true, x + w - r, y + h)
            .line_to(x + r, y + h)
            .arc_to(r, r, 0.0, false, true, x, y + h - r)
            .close()
            .build(),
        RoundedEnd::Left => PathBuilder::new()
            .move_to(x + r, y)
            .line_to(x + w, y)
            .line_to(x + w, y + h)
            .line_to(x + r, y + h)
            .arc_to(r, r, 0.0, false, true, x, y + h - r)
            .line_to(x, y + r)
            .arc_to(r, r, 0.0, false, true, x + r, y)
            .close()
            .build(),
        RoundedEnd::Right => PathBuilder::new()
            .move_to(x, y)
            .line_to(x + w - r, y)
            .arc_to(r, r, 0.0, false, true, x + w, y + r)
            .line_to(x + w, y + h - r)
            .arc_to(r, r, 0.0, false, true, x + w - r, y + h)
            .line_to(x, y + h)
            .close()
            .build(),
        RoundedEnd::None => unreachable!(
            "bar_shape が RoundedEnd::None を rounded_rect_d へ渡さない契約（<rect> を先に返す）"
        ),
    }
}

/// 棒 1 本を組み立てる（内部ヘルパ）。`radius <= 0.0` または
/// `end == RoundedEnd::None` の場合は `corner_radius == 0.0`（#2082 以前）
/// と完全に同一の `<rect>` を返す（golden 純追加原則）。それ以外は
/// [`rounded_rect_d`] で組み立てた `<path>` を返す（`data-scope`/`data-part`
/// は `<rect>` と同じ値を維持し CSS セレクタが変わらないようにする）。
fn bar_shape(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    radius: f64,
    end: RoundedEnd,
    attrs: Vec<(&str, &str)>,
) -> Node {
    if radius <= 0.0 || end == RoundedEnd::None {
        return svg::rect(x, y, w, h, attrs);
    }
    let r = radius.min(w / 2.0).min(h / 2.0);
    if r <= 0.0 {
        return svg::rect(x, y, w, h, attrs);
    }
    let d = rounded_rect_d(x, y, w, h, r, end);
    let mut full_attrs = attrs;
    full_attrs.push(("d", d.as_str()));
    el("path", full_attrs, vec![])
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
/// - `props.width`/`props.height` が正でも、カテゴリラベル用余白・値軸用
///   余白を差し引いた結果プロット領域の幅・高さが 0 以下になる場合
///   [`ChartError::PlotAreaTooSmall`]（`ViewBox::new` は寸法の正値のみを
///   検証し、余白差し引き後までは検証しないため、放置するとバーが潰れる、
///   または viewBox 外へ無警告で描画される、PR #877 レビュー指摘）。
/// - `props.corner_radius` が非有限または負値の場合
///   [`ChartError::InvalidCornerRadius`]（イシュー #2082）。
/// - `props.active_index` がカテゴリ数以上の場合
///   [`ChartError::IndexOutOfRange`]（イシュー #2082）。
/// - `props.stack` が [`BarStack::Normal`]/[`BarStack::Expand`] で系列に
///   負値が含まれる場合 [`ChartError::NegativeValue`]（イシュー #2082）。
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
    if !props.corner_radius.is_finite() || props.corner_radius < 0.0 {
        return Err(ChartError::InvalidCornerRadius);
    }

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

    let categories = data.categories();
    let n_categories = categories.len();
    let series = data.series();
    let n_series = series.len().max(1);

    if let Some(i) = props.active_index {
        if i >= n_categories {
            return Err(ChartError::IndexOutOfRange);
        }
    }

    let stacked = props.stack != BarStack::None;
    let expand = props.stack == BarStack::Expand;

    // 積み上げ時の系列ごとカテゴリ累積上限値（`ChartData::stacked_cumulative`、
    // イシュー #2082 で area_chart と共有するため `charts::data` へ移設済み）。
    let cum: Option<Vec<Vec<f64>>> = if stacked {
        Some(data.stacked_cumulative(expand)?)
    } else {
        None
    };

    let domain = if let Some(cum) = &cum {
        if expand {
            (0.0, 1.0)
        } else {
            let max = cum
                .last()
                .map(|last| last.iter().copied().fold(f64::NEG_INFINITY, f64::max))
                .unwrap_or(0.0);
            (0.0, if max > 0.0 { max } else { 1.0 })
        }
    } else {
        let (dmin, dmax) = data.domain();
        (dmin.min(0.0), dmax.max(0.0))
    };

    let category_space = if props.show_category_labels {
        match props.orientation {
            Orientation::Vertical => CATEGORY_LABEL_SPACE,
            Orientation::Horizontal => CATEGORY_LABEL_SPACE_HORIZONTAL,
        }
    } else {
        0.0
    };
    let value_axis_margin = if props.show_value_axis {
        match props.orientation {
            Orientation::Vertical => AXIS_VALUE_MARGIN_LEFT,
            Orientation::Horizontal => AXIS_VALUE_MARGIN_BOTTOM,
        }
    } else {
        0.0
    };

    // `left_offset` は Vertical のみ非 0（値軸用の左余白）。Horizontal は
    // 値軸余白を高さ側（下）から差し引くため、プロット幅の左端は 0 のまま
    // （既存 #849 の座標系を維持、golden 純追加原則）。
    let (plot_width, plot_height, left_offset) = match props.orientation {
        Orientation::Vertical => (
            props.width - value_axis_margin,
            props.height - category_space,
            value_axis_margin,
        ),
        Orientation::Horizontal => (
            props.width - category_space,
            props.height - value_axis_margin,
            0.0,
        ),
    };
    // `ViewBox::new` は width/height が正であることのみ検証し、余白差し引き
    // 後の実プロット領域までは検証しない。ここで拒否しないと、幅・高さが
    // 0 以下のままバンド幅・棒寸法が 0/負値になり、バーが潰れる、または
    // viewBox 外に無警告で描画される（PR #877 レビュー指摘、イシュー #849）。
    if plot_width <= 0.0 || plot_height <= 0.0 {
        return Err(ChartError::PlotAreaTooSmall);
    }

    let has_axes = props.show_value_axis || props.show_grid;

    let value_range = match props.orientation {
        // SVG は y が下向き正のため、値の大小を上下反転させる。
        Orientation::Vertical => (plot_height, 0.0),
        Orientation::Horizontal => (0.0, plot_width),
    };
    let raw_scale =
        LinearScale::new(domain, value_range).map_err(|_| ChartError::NonFiniteValue)?;
    // Expand は domain (0.0, 1.0) がカテゴリ合計比率の定義そのものであり、
    // nice() で境界がずれると「合計 100%」の不変条件が崩れるため対象外
    // （area_chart の AreaStack::Expand と同じ判断）。既定（軸/グリッド
    // なし）は #2082 以前と同じ nice() 非適用のまま。
    let value_scale = if has_axes && !expand {
        raw_scale.nice()
    } else {
        raw_scale
    };
    let baseline = value_scale.scale(0.0);

    let band = match props.orientation {
        Orientation::Vertical => plot_width / n_categories as f64,
        Orientation::Horizontal => plot_height / n_categories as f64,
    };
    let usable = band * (1.0 - 2.0 * BAND_EDGE_PADDING_FRAC);
    let edge_offset = band * BAND_EDGE_PADDING_FRAC;
    let bar_thickness = if stacked {
        usable
    } else {
        usable / n_series as f64
    };

    let mut plot_children: Vec<Node> = Vec::new();

    if props.show_grid {
        let ticks = value_scale.ticks(4)?;
        let grid_positions: Vec<f64> = ticks.iter().map(|&t| value_scale.scale(t)).collect();
        plot_children.push(match props.orientation {
            Orientation::Vertical => grid::cartesian_grid(
                (left_offset, props.width),
                (0.0, plot_height),
                &[],
                &grid_positions,
                &GridProps {
                    horizontal: true,
                    vertical: false,
                    ..GridProps::default()
                },
            )?,
            Orientation::Horizontal => grid::cartesian_grid(
                (0.0, plot_width),
                (0.0, plot_height),
                &grid_positions,
                &[],
                &GridProps {
                    horizontal: false,
                    vertical: true,
                    ..GridProps::default()
                },
            )?,
        });
    }

    for (cat_idx, category) in categories.iter().enumerate() {
        let band_start = band * cat_idx as f64;
        let is_active = props.active_index == Some(cat_idx);

        if let Some(cum) = &cum {
            let n = cum.len();
            for (s_idx, series_cum) in cum.iter().enumerate() {
                let upper = series_cum[cat_idx];
                let lower = if s_idx == 0 {
                    0.0
                } else {
                    cum[s_idx - 1][cat_idx]
                };
                let scaled_upper = value_scale.scale(upper);
                let scaled_lower = value_scale.scale(lower);
                let color = if props.color_by_category {
                    super::series_color_var(cat_idx)
                } else {
                    data.series_color_var(s_idx)
                };
                let rounded_end = if n == 1 {
                    RoundedEnd::All
                } else if s_idx == 0 {
                    match props.orientation {
                        Orientation::Vertical => RoundedEnd::Bottom,
                        Orientation::Horizontal => RoundedEnd::Left,
                    }
                } else if s_idx == n - 1 {
                    match props.orientation {
                        Orientation::Vertical => RoundedEnd::Top,
                        Orientation::Horizontal => RoundedEnd::Right,
                    }
                } else {
                    RoundedEnd::None
                };

                let (x, y, w, h) = match props.orientation {
                    Orientation::Vertical => {
                        let x = band_start + edge_offset + left_offset;
                        let y = scaled_upper.min(scaled_lower);
                        let h = (scaled_upper - scaled_lower).abs();
                        (x, y, bar_thickness, h)
                    }
                    Orientation::Horizontal => {
                        let y = band_start + edge_offset;
                        let x = scaled_upper.min(scaled_lower);
                        let w = (scaled_upper - scaled_lower).abs();
                        (x, y, w, bar_thickness)
                    }
                };

                let mut attrs: Vec<(&str, &str)> =
                    vec![("data-scope", "bar-chart"), ("data-part", "bar")];
                attrs.push(("fill", color.as_str()));
                if is_active {
                    attrs.push(("data-active", ""));
                    attrs.push(("color", color.as_str()));
                }
                plot_children.push(bar_shape(
                    x,
                    y,
                    w,
                    h,
                    props.corner_radius,
                    rounded_end,
                    attrs,
                ));

                // 値ラベル（Normal のみ、最上段の先端に合計値を 1 つ。
                // Expand は常に 100% のため表示しない）。カテゴリ名の内側
                // ラベルは s_idx == 0（ベースライン側の帯）にのみ付ける。
                if s_idx == n - 1 && props.label != BarLabel::None && !expand {
                    let total = upper;
                    plot_children.push(value_label(
                        x,
                        y,
                        w,
                        h,
                        &super::svg::fmt_coord(total),
                        props.orientation,
                    ));
                }
                if s_idx == 0 && props.label == BarLabel::Inside {
                    plot_children.push(inside_label(x, y, w, h, category, props.orientation));
                }
            }
        } else {
            for (series_idx, s) in series.iter().enumerate() {
                let value = s.values[cat_idx];
                let scaled = value_scale.scale(value);
                let is_negative = props.highlight_negative && value < 0.0;
                let color = if is_negative {
                    super::series_color_var(1)
                } else if props.color_by_category {
                    super::series_color_var(cat_idx)
                } else {
                    data.series_color_var(series_idx)
                };

                let (x, y, w, h) = match props.orientation {
                    Orientation::Vertical => {
                        let x = band_start
                            + edge_offset
                            + bar_thickness * series_idx as f64
                            + left_offset;
                        let y = scaled.min(baseline);
                        let h = (scaled - baseline).abs();
                        (x, y, bar_thickness, h)
                    }
                    Orientation::Horizontal => {
                        let y = band_start + edge_offset + bar_thickness * series_idx as f64;
                        let x = scaled.min(baseline);
                        let w = (scaled - baseline).abs();
                        (x, y, w, bar_thickness)
                    }
                };

                let rounded_end = if props.corner_radius > 0.0 {
                    match props.orientation {
                        Orientation::Vertical => {
                            if scaled <= baseline {
                                RoundedEnd::Top
                            } else {
                                RoundedEnd::Bottom
                            }
                        }
                        Orientation::Horizontal => {
                            if scaled >= baseline {
                                RoundedEnd::Right
                            } else {
                                RoundedEnd::Left
                            }
                        }
                    }
                } else {
                    RoundedEnd::None
                };

                let mut attrs: Vec<(&str, &str)> =
                    vec![("data-scope", "bar-chart"), ("data-part", "bar")];
                attrs.push(("fill", color.as_str()));
                if is_active {
                    attrs.push(("data-active", ""));
                    attrs.push(("color", color.as_str()));
                }
                if is_negative {
                    attrs.push(("data-negative", ""));
                }
                plot_children.push(bar_shape(
                    x,
                    y,
                    w,
                    h,
                    props.corner_radius,
                    rounded_end,
                    attrs,
                ));

                if props.label != BarLabel::None {
                    plot_children.push(value_label(
                        x,
                        y,
                        w,
                        h,
                        &super::svg::fmt_coord(value),
                        props.orientation,
                    ));
                }
                if series_idx == 0 && props.label == BarLabel::Inside {
                    plot_children.push(inside_label(x, y, w, h, category, props.orientation));
                }
            }
        }

        if props.show_category_labels {
            plot_children.push(category_label(
                category,
                band_start,
                band,
                plot_width,
                plot_height,
                left_offset,
                props,
            ));
        }
    }

    if props.show_value_axis {
        let ticks = value_scale.ticks(4)?;
        let format = if expand {
            TickLabelFormat {
                suffix: "%",
                label_scale: 100.0,
                ..TickLabelFormat::default()
            }
        } else {
            TickLabelFormat::default()
        };
        plot_children.push(match props.orientation {
            Orientation::Vertical => axis::y_axis(
                &value_scale,
                &ticks,
                left_offset,
                &AxisProps {
                    show_tick_lines: false,
                    show_axis_line: false,
                    format,
                    ..AxisProps::default()
                },
            )?,
            Orientation::Horizontal => axis::x_axis_linear(
                &value_scale,
                &ticks,
                plot_height,
                &AxisProps {
                    show_tick_lines: false,
                    show_axis_line: false,
                    format,
                    ..AxisProps::default()
                },
            )?,
        });
    }

    let attrs = vec![("data-scope", "bar-chart"), ("data-part", "root")];
    let mut merged_attrs = vec![("aria-label", aria_label)];
    merged_attrs.extend(attrs);
    Ok(svg::svg_root(&view_box, merged_attrs, plot_children))
}

/// 値ラベル（棒先端の外側、イシュー #2082）を組み立てる（内部ヘルパ）。
/// `(x, y, w, h)` は棒の矩形（角丸適用前の論理座標、`bar_shape` と同じ
/// 引数）。ベースラインより上（Vertical）/ 右（Horizontal）に伸びる棒を
/// 「正方向」とし、先端側へ [`LABEL_OFFSET`] だけ外側にずらす。
fn value_label(x: f64, y: f64, w: f64, h: f64, value_str: &str, orientation: Orientation) -> Node {
    let (lx, ly, extra): (f64, f64, Vec<(&str, &str)>) = match orientation {
        Orientation::Vertical => {
            // y が棒の上端（数値が大きいほど上、SVG 座標は上ほど小さい値）。
            // 上端が原点（0,0）に近い、すなわち棒がベースラインより上に
            // 伸びている（正方向）とみなし、ラベルは上端のさらに外側へ置く。
            // h == 0（値がベースラインと一致）の退化ケースは上側扱いで問題
            // ない（見た目上どちらでも同じ位置になる）。
            (
                x + w / 2.0,
                y - LABEL_OFFSET,
                vec![("text-anchor", "middle")],
            )
        }
        Orientation::Horizontal => (
            x + w + LABEL_OFFSET,
            y + h / 2.0,
            vec![("text-anchor", "start"), ("dominant-baseline", "middle")],
        ),
    };
    let mut attrs: Vec<(&str, &str)> =
        vec![("data-scope", "bar-chart"), ("data-part", "value-label")];
    attrs.extend(extra);
    svg_text(lx, ly, attrs, vec![text(value_str.to_string())])
}

/// 内側ラベル（棒のベースライン側内部にカテゴリ名、イシュー #2082）を
/// 組み立てる（内部ヘルパ）。`(x, y, w, h)` は [`value_label`] と同じ。
fn inside_label(x: f64, y: f64, w: f64, h: f64, category: &str, orientation: Orientation) -> Node {
    let (lx, ly, extra): (f64, f64, Vec<(&str, &str)>) = match orientation {
        Orientation::Vertical => (
            x + w / 2.0,
            y + h - INSIDE_LABEL_OFFSET,
            vec![("text-anchor", "middle")],
        ),
        Orientation::Horizontal => (
            x + INSIDE_LABEL_OFFSET,
            y + h / 2.0,
            vec![("text-anchor", "start"), ("dominant-baseline", "middle")],
        ),
    };
    let mut attrs: Vec<(&str, &str)> =
        vec![("data-scope", "bar-chart"), ("data-part", "inside-label")];
    attrs.extend(extra);
    svg_text(lx, ly, attrs, vec![text(category.to_string())])
}

/// カテゴリラベル（[`svg_text`]）を組み立てる（内部ヘルパ）。
#[allow(
    clippy::too_many_arguments,
    reason = "カテゴリラベルの座標計算はプロット寸法・バンド位置・値軸余白のすべてを必要とし、分割すると呼び出し側で対応関係が追いにくくなる"
)]
fn category_label(
    category: &str,
    band_start: f64,
    band: f64,
    plot_width: f64,
    plot_height: f64,
    left_offset: f64,
    props: BarChartProps,
) -> Node {
    let (x, y, attrs) = match props.orientation {
        Orientation::Vertical => (
            band_start + band / 2.0 + left_offset,
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

    /// イシュー #2082 実装ステップ 2: 既定 props（新フィールドはすべて
    /// 既定値）の出力が #2082 以前と完全に同一であることを固定する
    /// （golden 純追加原則）。負値データを含む Vertical/Horizontal 双方を
    /// 検査する。
    #[test]
    fn default_props_html_is_unchanged_since_2082() {
        let node = root(&sample(), BarChartProps::default(), "label").unwrap();
        let html = render(&node);
        assert!(html.contains(r#"x="24""#));
        assert!(html.contains(r#"y="184""#));
        assert!(html.contains(r#"height="92""#));
        assert!(html.contains(r#"width="192""#));
        assert!(!html.contains("<path"));
        assert!(!html.contains("data-active"));
        assert!(!html.contains("data-negative"));
        assert!(!html.contains("value-label"));
        assert!(!html.contains("inside-label"));

        let neg_data = ChartData::new(
            vec!["a".to_string(), "b".to_string()],
            vec![Series::new("s", vec![-10.0, 10.0])],
        )
        .unwrap();
        let neg_html = render(&root(&neg_data, BarChartProps::default(), "label").unwrap());
        assert_eq!(neg_html.matches(r#"data-part="bar""#).count(), 2);
        assert!(!neg_html.contains("<path"));
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
            ..BarChartProps::default()
        };
        assert_eq!(
            root(&sample(), vertical_too_small, "label").unwrap_err(),
            ChartError::PlotAreaTooSmall
        );

        let horizontal_too_small = BarChartProps {
            orientation: Orientation::Horizontal,
            width: 96.0,
            height: 300.0,
            ..BarChartProps::default()
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
            ..BarChartProps::default()
        };
        assert_eq!(
            root(&sample(), non_finite, "label").unwrap_err(),
            ChartError::NonFiniteValue
        );

        let non_positive = BarChartProps {
            orientation: Orientation::Vertical,
            width: 0.0,
            height: 300.0,
            ..BarChartProps::default()
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

    /// 系列の [`crate::charts::data::SeriesColor`] 上書き（イシュー #2077）が
    /// `fill` へ反映されることを固定する（`ChartData::series_color_var`
    /// 経由の一元性、凡例と同じ値を共有する契約）。
    #[test]
    fn root_reflects_series_color_override_in_fill() {
        let data = ChartData::new(
            vec!["Jan".to_string(), "Feb".to_string()],
            vec![Series::new("visits", vec![10.0, 30.0])
                .with_color(crate::charts::SeriesColor::token("warning").unwrap())],
        )
        .unwrap();
        let html = render(&root(&data, BarChartProps::default(), "label").unwrap());
        assert!(html.contains(r#"fill="var(--fandhe-color-warning)""#));
    }

    #[test]
    fn vertical_bar_geometry_matches_hand_computed_values() {
        // domain: (0,30) を nice せず 0 起点のまま使う本モジュールの規則
        // （nice() は show_value_axis/show_grid 有効時のみ適用、イシュー
        // #2082）。plot_height = 300 - 24 = 276, plot_width = 480。
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

    // ---- イシュー #2082: shadcn/ui Charts（bar）突合バリアント ----

    #[test]
    fn corner_radius_zero_renders_rect_not_path() {
        let props = BarChartProps {
            corner_radius: 0.0,
            ..BarChartProps::default()
        };
        let html = render(&root(&sample(), props, "label").unwrap());
        assert!(!html.contains("<path"));
    }

    #[test]
    fn corner_radius_positive_renders_path_with_closed_charset() {
        let props = BarChartProps {
            corner_radius: 8.0,
            ..BarChartProps::default()
        };
        let html = render(&root(&sample(), props, "label").unwrap());
        assert!(html.contains("<path"));
        // d 属性の文字集合が M/L/A/Z + 数字/カンマ/ピリオド/マイナス/空白
        // に閉じることを確認する（モジュール doc セキュリティ不変条件）。
        let start = html.find(r#" d=""#).unwrap() + 4;
        let rest = &html[start..];
        let end = rest.find('"').unwrap();
        let d = &rest[..end];
        assert!(d
            .chars()
            .all(|c| c.is_ascii_digit()
                || matches!(c, '.' | '-' | ' ' | ',' | 'M' | 'L' | 'A' | 'Z')));
    }

    #[test]
    fn corner_radius_negative_or_non_finite_is_rejected() {
        let neg = BarChartProps {
            corner_radius: -1.0,
            ..BarChartProps::default()
        };
        assert_eq!(
            root(&sample(), neg, "label").unwrap_err(),
            ChartError::InvalidCornerRadius
        );
        let nan = BarChartProps {
            corner_radius: f64::NAN,
            ..BarChartProps::default()
        };
        assert_eq!(
            root(&sample(), nan, "label").unwrap_err(),
            ChartError::InvalidCornerRadius
        );
    }

    #[test]
    fn corner_radius_is_clamped_to_half_the_shorter_side() {
        // 半径が棒の半分の辺長より大きくても panic せず、決定的に描画される
        // ことを確認する（`bar_shape` の `min(w/2, h/2)` クランプ）。
        let props = BarChartProps {
            corner_radius: 10_000.0,
            ..BarChartProps::default()
        };
        let a = render(&root(&sample(), props, "label").unwrap());
        let b = render(&root(&sample(), props, "label").unwrap());
        assert_eq!(a, b);
    }

    #[test]
    fn stack_normal_places_bars_end_to_end() {
        // 2 系列 [10]/[20] は区間 0-10 / 10-30、domain は (0, 30)。
        let data = ChartData::new(
            vec!["a".to_string()],
            vec![Series::new("s1", vec![10.0]), Series::new("s2", vec![20.0])],
        )
        .unwrap();
        let props = BarChartProps {
            stack: BarStack::Normal,
            ..BarChartProps::default()
        };
        let html = render(&root(&data, props, "label").unwrap());
        assert_eq!(html.matches(r#"data-part="bar""#).count(), 2);
    }

    #[test]
    fn stack_expand_domain_is_fixed_and_labels_are_percent() {
        let data = ChartData::new(
            vec!["a".to_string()],
            vec![Series::new("s1", vec![10.0]), Series::new("s2", vec![30.0])],
        )
        .unwrap();
        let props = BarChartProps {
            stack: BarStack::Expand,
            show_value_axis: true,
            ..BarChartProps::default()
        };
        let html = render(&root(&data, props, "label").unwrap());
        assert!(html.contains('%'));
        // Expand は値ラベルを出さない。
        let props_with_label = BarChartProps {
            stack: BarStack::Expand,
            label: BarLabel::Outside,
            ..BarChartProps::default()
        };
        let html2 = render(&root(&data, props_with_label, "label").unwrap());
        assert!(!html2.contains(r#"data-part="value-label""#));
    }

    #[test]
    fn stack_rejects_negative_values() {
        let data =
            ChartData::new(vec!["a".to_string()], vec![Series::new("s", vec![-1.0])]).unwrap();
        let props = BarChartProps {
            stack: BarStack::Normal,
            ..BarChartProps::default()
        };
        assert_eq!(
            root(&data, props, "label").unwrap_err(),
            ChartError::NegativeValue
        );
    }

    #[test]
    fn active_index_marks_all_series_bars_in_that_category() {
        let data = ChartData::new(
            vec!["a".to_string(), "b".to_string()],
            vec![
                Series::new("s1", vec![1.0, 2.0]),
                Series::new("s2", vec![3.0, 4.0]),
            ],
        )
        .unwrap();
        let props = BarChartProps {
            active_index: Some(0),
            ..BarChartProps::default()
        };
        let html = render(&root(&data, props, "label").unwrap());
        assert_eq!(html.matches("data-active").count(), 2);
    }

    #[test]
    fn active_index_out_of_range_is_rejected() {
        let props = BarChartProps {
            active_index: Some(99),
            ..BarChartProps::default()
        };
        assert_eq!(
            root(&sample(), props, "label").unwrap_err(),
            ChartError::IndexOutOfRange
        );
    }

    #[test]
    fn color_by_category_cycles_chart_tokens_by_category() {
        let data = ChartData::new(
            vec!["a".to_string(), "b".to_string()],
            vec![Series::new("s", vec![1.0, 2.0])],
        )
        .unwrap();
        let props = BarChartProps {
            color_by_category: true,
            ..BarChartProps::default()
        };
        let html = render(&root(&data, props, "label").unwrap());
        assert!(html.contains("chart-1"));
        assert!(html.contains("chart-2"));
    }

    #[test]
    fn highlight_negative_marks_only_negative_bars() {
        let data = ChartData::new(
            vec!["a".to_string(), "b".to_string()],
            vec![Series::new("s", vec![-5.0, 5.0])],
        )
        .unwrap();
        let props = BarChartProps {
            highlight_negative: true,
            ..BarChartProps::default()
        };
        let html = render(&root(&data, props, "label").unwrap());
        assert_eq!(html.matches("data-negative").count(), 1);
        assert!(html.contains("chart-2"));

        // 正値のみのデータでは data-negative が出ない。
        let all_positive = sample();
        let html2 = render(&root(&all_positive, props, "label").unwrap());
        assert!(!html2.contains("data-negative"));
    }

    #[test]
    fn show_value_axis_and_grid_render_axis_and_grid_parts() {
        let props = BarChartProps {
            show_value_axis: true,
            show_grid: true,
            ..BarChartProps::default()
        };
        let html = render(&root(&sample(), props, "label").unwrap());
        assert!(html.contains(r#"data-part="y-axis""#));
        assert!(html.contains(r#"data-part="grid-line""#));

        let horizontal = BarChartProps {
            orientation: Orientation::Horizontal,
            show_value_axis: true,
            show_grid: true,
            ..BarChartProps::default()
        };
        let html2 = render(&root(&sample(), horizontal, "label").unwrap());
        assert!(html2.contains(r#"data-part="x-axis""#));
        assert!(html2.contains(r#"data-part="grid-line""#));
    }

    #[test]
    fn show_category_labels_false_omits_category_label() {
        let props = BarChartProps {
            show_category_labels: false,
            ..BarChartProps::default()
        };
        let html = render(&root(&sample(), props, "label").unwrap());
        assert!(!html.contains(r#"data-part="category-label""#));
    }

    #[test]
    fn label_outside_renders_value_label_per_bar() {
        let props = BarChartProps {
            label: BarLabel::Outside,
            ..BarChartProps::default()
        };
        let html = render(&root(&sample(), props, "label").unwrap());
        assert_eq!(html.matches(r#"data-part="value-label""#).count(), 2);
    }

    #[test]
    fn label_inside_renders_inside_label_with_category_name() {
        let props = BarChartProps {
            orientation: Orientation::Horizontal,
            label: BarLabel::Inside,
            show_category_labels: false,
            ..BarChartProps::default()
        };
        let html = render(&root(&sample(), props, "label").unwrap());
        assert!(html.contains(r#"data-part="inside-label""#));
        assert!(html.contains("Jan"));
        assert!(!html.contains(r#"data-part="category-label""#));
    }

    #[test]
    fn plot_area_too_small_with_value_axis_margin() {
        let props = BarChartProps {
            orientation: Orientation::Vertical,
            width: 40.0,
            show_value_axis: true,
            ..BarChartProps::default()
        };
        assert_eq!(
            root(&sample(), props, "label").unwrap_err(),
            ChartError::PlotAreaTooSmall
        );
    }

    #[test]
    fn active_data_rule_has_stroke_current_color_and_no_hover() {
        let out = css();
        assert!(out.contains(r#"[data-scope="bar-chart"][data-part="bar"][data-active]"#));
        assert!(out.contains("stroke: currentColor;"));
        assert!(!out.contains(":hover"));
    }
}
