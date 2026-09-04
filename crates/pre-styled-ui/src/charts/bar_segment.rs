//! BarSegment（構成比バー、100% 積み上げ、イシュー #849・親 Phase #845）。
//!
//! chakra-ui `charts/bar-segment.md` 相当を HTML（`<div>` ベース）で再構成
//! する。[`super::data::ChartData`] の 1 系列を対象に、各カテゴリを 1
//! セグメントとして「系列合計に対する比率」で幅を割り当てた単一の横棒
//! （100% 積み上げ）として描画する。新規 anatomy `data-scope="bar-segment"`
//! を本モジュールで定義する（[`crate::table`]/[`crate::charts::bar_list`] と
//! 同型の判断、`fandhe-frontend-headless-ui` 側に対応する anatomy はない）。
//!
//! # 配色
//!
//! 各セグメントはカテゴリ index を [`super::series_color_var`] に渡して
//! `chart-1`〜`chart-6` を循環させる（chakra-ui BarSegment がアイテムごとに
//! 色を割り当てる挙動に対応。[`super::bar_chart`] が系列 index で循環させる
//! のとは対象が異なる点に注意）。
//!
//! # 比率の伝搬（インライン custom property）
//!
//! セグメント幅は [`super::data::value_percent`]（合計に対する割合、0 合計は
//! `0.0` を返す既存契約）を [`super::svg::fmt_coord`] で文字列化し、
//! `style="--fandhe-bar-segment-percent: <n>%"` としてインライン伝搬する
//! （[`super::bar_list`] の `--fandhe-bar-list-percent` 方式と同型）。
//!
//! # fail-closed（`.claude/rules/security.md` A04 対応、[`super::bar_list`] との違い）
//!
//! - 対象系列が存在しない場合 [`ChartError::UnknownSeriesName`]。
//! - 系列中に負値が 1 件でもあれば [`ChartError::NegativeValue`]。
//! - **系列合計が 0 の場合は [`ChartError::ZeroTotal`] で構築を拒否する**
//!   （[`super::data::value_percent`] の「合計 0 → `0.0` を返す」契約に
//!   黙って乗ると、全セグメント幅 0% の空バーが「データなし」なのか
//!   「構成比が定義できない」なのか利用者が区別できない silent failure に
//!   なる。[`super::bar_list`] の「値 0 → 幅 0」は個々の値と幅の対応関係が
//!   自明だが、本部品は「合計に対する比率」という関係性そのものが失われる
//!   ため、両部品で挙動を意図的に変えている、モジュール doc に明記する
//!   実装判断）。
//!
//! # セキュリティ不変条件
//!
//! マークアップはすべてノード木 API 経由（`raw_html()` 不使用、REQ-1）。
//! 値の文字列化は [`super::svg::fmt_coord`] にのみ一元化する。インライン
//! `style` 属性値は固定テンプレートのみで構成する（[`super::bar_list`] と
//! 同型の不変条件）。
//!
//! # legend（`showPercent` 相当）
//!
//! [`legend`] は各セグメントの色マーカー・ラベル・比率テキストを静的出力する
//! 最小実装であり、#847 の汎用 Legend（軸/凡例横断部品）とは独立している
//! （境界を明示する。将来的な統合は #847 側の設計判断に委ねる）。
//!
//! # 本イシューのスコープ外
//!
//! - `examples/headless-pre-styled-ui` への追随は crates.io 公開後に別途。
//!
//! # 参考サイト基準への調整（イシュー #1592）
//!
//! 親 Phase #1588「Themes / Charts のスタイル調整」の子。参照 4 サイト
//! （chakra-ui / Ark UI / Radix Primitives / Radix Themes）に対応部品が
//! 存在しないため、評価軸は**内部整合のみ**（`--fandhe-*` トークン適用・
//! ダーク時の可読性・系列色パレットの識別性・データラベルのコントラスト）
//! に限定する（[`crate::area_chart`] イシュー #1589 と同じ判断）。
//!
//! | 軸 | 結論 |
//! |---|---|
//! | サイズ | 非該当（size バリアントを持たない。参照軸なし） |
//! | バリアント / colorPalette | 非採用（参照軸なし。配色はカテゴリ index で `chart-1`〜`chart-6` 循環） |
//! | 色 | 是正 1 点（下記「`bar` の track 背景」）。他は全宣言がトークン経由で現状維持 |
//! | 状態 `data-*` | 非該当（pre-styled-only、headless 由来の `data-*` を持たない） |
//! | ダークモード | 現状維持（`chart-N`・`fg`・`fg-muted`・`bg-muted` はいずれも dark 値定義済み。凡例テキストのコントラストは light `fg-muted #4a4a4a`/`bg #ffffff` ≈ 8.9:1、dark `#cccccc`/`#111111` ≈ 11.8:1 で本文 4.5:1 を満たす） |
//! | フォーカス | 非該当（表示専用、フォーカス可能要素なし） |
//! | 余白・角丸・影 | 是正（生リテラルを `--fandhe-space-*`/`--fandhe-radius-*` スケールへ統一。影は不使用のまま） |
//! | hover / disabled / トランジション | 非採用（表示専用部品、状態遷移なし） |
//! | 内部整合（実欠陥） | 是正 3 点（下記） |
//!
//! ## 是正した点
//!
//! - **`bar` の track 背景**: `background: var(--fandhe-color-bg-muted)` を
//!   追加した。丸め剰余（[`super::data::value_percent`] の百分率丸め）で
//!   各セグメント幅の合計が 100% にわずかに満たない場合にページ背景が
//!   透けて見えるのを防ぎ、[`super::bar_list`]/`progress` の track 面
//!   （いずれも `bg-muted` 背景）と整合させる。
//! - **セグメント間の区切り線**: `segment` に
//!   `box-shadow: inset -1px 0 0 var(--fandhe-color-bg)` を追加し、隣接
//!   カテゴリの色境界を明確にした（幅そのものは変えないため比率の真正性
//!   は崩さない）。ただし最終セグメントの右端は `bar` の `overflow: hidden`
//!   と `border-radius` により直線で切れるため、区切り線を残すと 1px の
//!   欠けに見える。[`crate::recipe::StateCondition::LastChild`]（`steps.rs`
//!   先例と同型）で最終セグメントのみ `box-shadow: none` に戻す。
//! - **凡例のマーカー寸法・間隔**: 同 crate の [`super::legend`] と
//!   数値が不一致だったため揃えた: `legend-marker` は `0.625rem` →
//!   `0.75rem`、`legend-item` の `gap` は `0.375rem` → `var(--fandhe-space-2)`
//!   （`0.5rem`）。同一 crate 内の凡例表現で寸法が異なる不整合を解消する。
//!
//! ## 意図的に合わせなかった点
//!
//! - chakra `barSize` 既定 `2.5rem` への追随はしない。値・ラベルをセグメント
//!   内に描画しない本部品では現行の細いバーで足り、`bar` の高さは
//!   `var(--fandhe-bar-segment-bar-height, 0.75rem)` で利用者が上書き
//!   可能にするに留める（[`super::bar_list`]/`progress` の
//!   `--fandhe-bar-list-track-height`/`--fandhe-progress-track-height`
//!   先例と同型）。
//! - 極小セグメントの最小幅は設けない（比率の真正性を崩さないため。
//!   [`super::bar_list`] イシュー #1591 と同じ判断）。
//! - `segment` へ `border-radius: inherit` は付けない。[`super::bar_list`]
//!   と異なりセグメントは隙間なく隣接充填するため、付けると内側の境界が
//!   丸まり隙間状に見えてしまう（`bar` の `overflow: hidden` で外側の端は
//!   既に丸く切れている）。
//! - `bar` の角丸段（`radius-sm`）は維持する（[`super::bar_list`] イシュー
//!   #1591 が「bar-segment と揃えるため radius-sm 維持」とした判断を
//!   踏襲し、本部品側から変えない）。
//! - chakra の `Value`/`Label`（セグメント直上直下の描画）・`Reference`・
//!   `Tooltip` に相当する anatomy 追加は行わない（`data-part` 契約の拡張は
//!   本イシューの内部整合スコープ外）。

use super::data::{self, ChartData};
use super::svg::fmt_coord;
use super::{series_color_var, ChartError};
use crate::css::decl;
use crate::recipe::{SlotRecipe, StateCondition};
use fandhe_frontend_headless_ui::fandhe_frontend_core::{text, Node};
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="bar-segment"` を固定した anatomy。
const ANATOMY: Anatomy = anatomy("bar-segment");

/// [`SlotRecipe::new`] に渡す slot 一覧。
const SLOTS: &[&str] = &[
    "root",
    "bar",
    "segment",
    "legend",
    "legend-item",
    "legend-marker",
    "legend-label",
    "legend-value",
];

/// この BarSegment の既定 CSS を組み立てる（内部ヘルパ、[`css`] のみが呼ぶ）。
fn recipe() -> SlotRecipe {
    SlotRecipe::new("bar-segment", SLOTS)
        .base(
            "root",
            vec![
                decl("display", "flex"),
                decl("flex-direction", "column"),
                // イシュー #1592: 生リテラル 0.75rem を `--fandhe-space-3`
                // （等価値）へ統一。
                decl("gap", "var(--fandhe-space-3)"),
                decl("width", "100%"),
            ],
        )
        .base(
            "bar",
            vec![
                decl("display", "flex"),
                decl("width", "100%"),
                // イシュー #1592: 呼び出し側からの高さ上書きを可能にする
                // （[`super::bar_list`] の
                // `--fandhe-bar-list-track-height`/progress の
                // `--fandhe-progress-track-height` と同型。フォールバックは
                // 従来の生リテラル値を維持）。
                decl("height", "var(--fandhe-bar-segment-bar-height, 0.75rem)"),
                decl("border-radius", "var(--fandhe-radius-sm)"),
                // イシュー #1592: track 背景を追加。百分率丸め
                // （[`super::data::value_percent`]）でセグメント幅の合計が
                // 100% にわずかに満たない場合にページ背景が透けて見えるのを
                // 防ぐ（[`super::bar_list`]/progress の track 面と同じ役割）。
                decl("background", "var(--fandhe-color-bg-muted)"),
                decl("overflow", "hidden"),
            ],
        )
        .base(
            "segment",
            vec![
                decl("height", "100%"),
                decl("width", "var(--fandhe-bar-segment-percent, 0%)"),
                // イシュー #1592: 隣接セグメント間に 1px の区切り線を入れて
                // 色境界を明確にする（幅は変えないため比率の真正性は保つ）。
                decl("box-shadow", "inset -1px 0 0 var(--fandhe-color-bg)"),
            ],
        )
        // イシュー #1592: 最終セグメントは右端が `bar` の
        // `overflow: hidden` + `border-radius` で直線に切れるため、上記
        // 区切り線を残すと 1px の欠けに見える。states は base とは独立に
        // 常に base 群の後段で出力される（[`SlotRecipe::css`] 契約）ため
        // 登録位置自体は問わないが、同一 slot（`segment`）への他の state
        // 規則より後に登録する契約（`steps.rs` 先例参照）は維持する
        // （現状 `segment` への state はこの 1 件のみのため実害はないが、
        // 将来追加される他の `segment` state に対して打ち消しが後勝ちで
        // 効くようにする）。
        .state(
            "segment",
            StateCondition::LastChild,
            vec![decl("box-shadow", "none")],
        )
        .base(
            "legend",
            vec![
                decl("display", "flex"),
                decl("flex-wrap", "wrap"),
                // イシュー #1592: 生リテラル 0.75rem/1rem を
                // `--fandhe-space-3`/`--fandhe-space-4`（等価値）へ統一。
                decl("gap", "var(--fandhe-space-3) var(--fandhe-space-4)"),
            ],
        )
        .base(
            "legend-item",
            vec![
                decl("display", "inline-flex"),
                decl("align-items", "center"),
                // イシュー #1592: 0.375rem → `--fandhe-space-2`（0.5rem）。
                // 同 crate の [`super::legend`] の `item` gap と揃える
                // （値変更を伴う是正、rustdoc「是正した点」参照）。
                decl("gap", "var(--fandhe-space-2)"),
                decl("font-size", "var(--fandhe-font-font-size-sm)"),
            ],
        )
        .base(
            "legend-marker",
            vec![
                // イシュー #1592: 0.625rem → 0.75rem。同 crate の
                // [`super::legend`] の `marker` と同寸に揃える（寸法は
                // 余白/角丸/影のトークン区分外のため生リテラルのまま）。
                decl("width", "0.75rem"),
                decl("height", "0.75rem"),
                decl("border-radius", "var(--fandhe-radius-full)"),
                decl("flex-shrink", "0"),
            ],
        )
        .base(
            "legend-label",
            vec![decl("color", "var(--fandhe-color-fg)")],
        )
        .base(
            "legend-value",
            vec![
                decl("color", "var(--fandhe-color-fg-muted)"),
                decl("font-variant-numeric", "tabular-nums"),
            ],
        )
}

/// この BarSegment が生成する静的 CSS 全量を返す（決定的）。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// BarSegment 本体（`bar` + [`legend`]）を組み立てる。
///
/// `data` から `series_name` の系列を取り出し、[`ChartData::categories`] の
/// 順にセグメントを描画する。
///
/// # Errors
///
/// - `series_name` に一致する系列がない場合 [`ChartError::UnknownSeriesName`]
/// - 系列中に負値が含まれる場合 [`ChartError::NegativeValue`]
/// - 系列合計が 0 の場合 [`ChartError::ZeroTotal`]（モジュール doc
///   「fail-closed」節参照）
///
/// # Examples
///
/// ```
/// use fandhe_frontend_core::render;
/// use fandhe_frontend_pre_styled_ui::charts::bar_segment::root;
/// use fandhe_frontend_pre_styled_ui::charts::data::{ChartData, Series};
///
/// let data = ChartData::new(
///     vec!["a".to_string(), "b".to_string()],
///     vec![Series::new("visits", vec![25.0, 75.0])],
/// )
/// .unwrap();
/// let node = root(&data, "visits").unwrap();
/// assert!(render(&node).contains(r#"data-scope="bar-segment" data-part="root""#));
/// ```
pub fn root(data: &ChartData, series_name: &str) -> Result<Node, ChartError> {
    let series = data
        .series()
        .iter()
        .find(|s| s.name == series_name)
        .ok_or(ChartError::UnknownSeriesName)?;

    if series.values.iter().any(|&v| v < 0.0) {
        return Err(ChartError::NegativeValue);
    }
    if data::total(series) == 0.0 {
        return Err(ChartError::ZeroTotal);
    }

    let categories = data.categories();
    let segments: Vec<Node> = categories
        .iter()
        .zip(series.values.iter())
        .enumerate()
        .map(|(idx, (_category, &value))| segment(idx, value, series))
        .collect();
    let bar = ANATOMY.part("bar", "div", vec![], segments);

    let legend = legend(categories, series);

    Ok(ANATOMY.part("root", "div", vec![], vec![bar, legend]))
}

/// 1 セグメント（`segment`）を組み立てる（内部ヘルパ）。
///
/// `background` はベアな HTML 属性としては存在しないため（ブラウザは無視し
/// `<div>` は無色描画のままになる、PR #877 レビュー指摘）、legend マーカー
/// （[`legend`] 内）と同様に `style` 属性値の一部として埋め込む。
fn segment(idx: usize, value: f64, series: &data::Series) -> Node {
    let percent = data::value_percent(series, value);
    let color = series_color_var(idx);
    let style = format!(
        "--fandhe-bar-segment-percent: {}%; background: {color}",
        fmt_coord(percent)
    );
    ANATOMY.part("segment", "div", vec![("style", style.as_str())], vec![])
}

/// 凡例（[`legend`] モジュール doc 参照）を組み立てる（内部ヘルパ）。
fn legend(categories: &[String], series: &data::Series) -> Node {
    let items: Vec<Node> = categories
        .iter()
        .zip(series.values.iter())
        .enumerate()
        .map(|(idx, (category, &value))| {
            let percent = data::value_percent(series, value);
            let color = series_color_var(idx);
            let marker_style = format!("background: {color}");
            ANATOMY.part(
                "legend-item",
                "span",
                vec![],
                vec![
                    ANATOMY.part(
                        "legend-marker",
                        "span",
                        vec![("style", marker_style.as_str())],
                        vec![],
                    ),
                    ANATOMY.part(
                        "legend-label",
                        "span",
                        vec![],
                        vec![text(category.to_string())],
                    ),
                    ANATOMY.part(
                        "legend-value",
                        "span",
                        vec![],
                        vec![text(format!("{}%", fmt_coord(percent)))],
                    ),
                ],
            )
        })
        .collect();
    ANATOMY.part("legend", "div", vec![], items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::data::Series;
    use fandhe_frontend_core::render;

    fn sample() -> ChartData {
        ChartData::new(
            vec!["a".to_string(), "b".to_string()],
            vec![Series::new("visits", vec![25.0, 75.0])],
        )
        .unwrap()
    }

    #[test]
    fn root_unknown_series_is_error() {
        assert_eq!(
            root(&sample(), "missing").unwrap_err(),
            ChartError::UnknownSeriesName
        );
    }

    #[test]
    fn root_rejects_negative_values() {
        let data =
            ChartData::new(vec!["a".to_string()], vec![Series::new("s", vec![-1.0])]).unwrap();
        assert_eq!(root(&data, "s").unwrap_err(), ChartError::NegativeValue);
    }

    #[test]
    fn root_rejects_zero_total() {
        let data = ChartData::new(
            vec!["a".to_string(), "b".to_string()],
            vec![Series::new("z", vec![0.0, 0.0])],
        )
        .unwrap();
        assert_eq!(root(&data, "z").unwrap_err(), ChartError::ZeroTotal);
    }

    #[test]
    fn root_computes_percent_relative_to_total() {
        let html = render(&root(&sample(), "visits").unwrap());
        assert!(html.contains("--fandhe-bar-segment-percent: 25%"));
        assert!(html.contains("--fandhe-bar-segment-percent: 75%"));
    }

    #[test]
    fn root_rounds_and_sums_to_100_for_thirds() {
        let data = ChartData::new(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec![Series::new("s", vec![1.0, 1.0, 1.0])],
        )
        .unwrap();
        let html = render(&root(&data, "s").unwrap());
        // 33.333...% は fmt_coord の丸め規則（{:.2} → 末尾ゼロ除去）で 33.33%。
        // 3 カテゴリそれぞれについて segment の custom property・legend の
        // 比率テキストの計 2 箇所ずつ出現する（合計 6 箇所）。
        assert_eq!(html.matches("33.33%").count(), 6);
        assert_eq!(
            html.matches("--fandhe-bar-segment-percent: 33.33%").count(),
            3
        );
        assert_eq!(html.matches(">33.33%<").count(), 3);
    }

    #[test]
    fn legend_lists_all_categories_with_percent() {
        let html = render(&root(&sample(), "visits").unwrap());
        assert!(html.contains(r#"data-part="legend""#));
        assert!(html.contains(r#"data-part="legend-item""#));
        assert!(html.contains(">a<"));
        assert!(html.contains(">25%<"));
        assert!(html.contains(">b<"));
        assert!(html.contains(">75%<"));
    }

    #[test]
    fn segment_color_is_set_via_style_not_bare_attribute() {
        // PR #877 レビュー指摘: 'background' がベア HTML 属性のままだと
        // ブラウザは CSS として扱わず無色描画になる。style 属性値の一部
        // として埋め込まれていることを確認する（bare な `background="..."`
        // 属性は存在しないことも合わせて検証する）。
        let html = render(&root(&sample(), "visits").unwrap());
        assert!(html.contains(
            "style=\"--fandhe-bar-segment-percent: 25%; background: var(--fandhe-color-chart-1)\""
        ));
        assert!(!html.contains(" background=\"var(--fandhe-color-chart-1)\""));
    }

    #[test]
    fn categories_cycle_through_six_color_slots() {
        let data = ChartData::new(
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
                "e".to_string(),
                "f".to_string(),
                "g".to_string(),
            ],
            vec![Series::new("s", vec![1.0; 7])],
        )
        .unwrap();
        let html = render(&root(&data, "s").unwrap());
        assert!(html.contains("chart-1"));
        assert!(html.contains("chart-6"));
    }

    #[test]
    fn root_is_deterministic() {
        let a = render(&root(&sample(), "visits").unwrap());
        let b = render(&root(&sample(), "visits").unwrap());
        assert_eq!(a, b);
    }

    #[test]
    fn category_labels_are_escaped() {
        let data = ChartData::new(
            vec!["<script>alert(1)</script>".to_string()],
            vec![Series::new("s", vec![1.0])],
        )
        .unwrap();
        let html = render(&root(&data, "s").unwrap());
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn css_is_deterministic_and_has_no_breakout_sequences() {
        let a = css();
        let b = css();
        assert_eq!(a, b);
        assert!(!a.contains('<'));
        assert!(a.contains(r#"[data-scope="bar-segment"]"#));
        // イシュー #1592: 是正した宣言が実際に出力されていることを固定する。
        assert!(a.contains("var(--fandhe-space-3)"));
        assert!(a.contains("var(--fandhe-bar-segment-bar-height, 0.75rem)"));
        assert!(a.contains("var(--fandhe-color-bg-muted)"));
        assert!(a.contains("inset -1px 0 0 var(--fandhe-color-bg)"));
        assert!(a.contains(":last-child"));
        assert!(a.contains("var(--fandhe-radius-full)"));
        assert!(a.contains("var(--fandhe-space-2)"));
    }
}
