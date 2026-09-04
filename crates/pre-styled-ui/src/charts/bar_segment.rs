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
//! | 状態 `data-*` | headless 由来の `data-*` は持たない。区切り線の
//!   要否は DOM 構造（`segment-divider` 子要素の有無）で表現するため、
//!   打ち消し用の専有 `data-*` は持たない（下記「是正した点」参照。
//!   当初 codex-review 指摘・codex-review/Cursor Bugbot 再指摘を受けて
//!   `data-fandhe-bar-segment-end`/`data-fandhe-bar-segment-empty` の
//!   2 種を新設していたが、後続の codex-review P1 再指摘（PR #1865）を
//!   受けた是正で構造的手法へ置き換え、両属性は廃止した） |
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
//! - **セグメント間の区切り線**: 隣接カテゴリの色境界を明確にするため、
//!   `segment` の子要素として `position: absolute` の
//!   [`segment-divider`]（`inset-block: 0`・`inset-inline-end: 0`・
//!   `width: 1px`・`background: var(--fandhe-color-bg)`）を条件付きで
//!   描画する（`segment()` 参照。区切り線を要素の境界線ではなく通常
//!   フローに参加しない絶対配置要素で表現するため、`segment` 自身の
//!   `width`〔合計に対する比率〕には一切影響しない）。
//!
//!   この形に落ち着くまでに 2 段階の是正を経ている。
//!   (1) 当初 `box-shadow: inset -1px 0 0 ...`〔物理右辺固定〕を用いて
//!   いたが、`direction: rtl` 継承時に打ち消し対象が逆転する欠陥があり、
//!   `border-inline-end`（論理方向プロパティ）+
//!   `box-sizing: border-box` へ置き換えた。合わせて「最後の可視（正値）
//!   segment の右端（RTL では左端）は `bar` の `overflow: hidden` +
//!   `border-radius` で直線に切れるため区切り線を残すと 1px の欠けに
//!   見える」問題も、DOM 順の `:last-child`（値 0 の末尾カテゴリで対象が
//!   不可視要素に奪われる欠陥があった）から `root()` が算出する「系列中
//!   最後の正値」index 基準の判定へ是正した（以上 codex-review 指摘、
//!   イシュー #1592）。
//!   (2) 続けて「値 0 の segment（幅 0%）にも border-inline-end は
//!   出力されるが実害はない」としていたが誤りで、`box-sizing: border-box`
//!   の要素は自身の border 幅より外形幅を小さくできないため、幅 0% の
//!   segment も border ぶん 1px の外形幅を持ち、先頭・中間の 0 値 segment
//!   が後続の正値 segment を圧迫していた（codex-review/Cursor Bugbot
//!   再指摘）。
//!   (3) さらに、border-box の「border 幅より小さくならない」制約は
//!   値 0 の segment に限らず、1px 未満の**極小な正値** segment
//!   （例: 100px 幅で 0.1%）にも及ぶことが後続の codex-review P1 再指摘
//!   （PR #1865）で判明した。値の正負で border-inline-end を出し分ける
//!   属性方式では原理的に塞げないため（border/box-sizing という box
//!   model 自体の制約であり、CSS 宣言の組み合わせでは回避できない）、
//!   区切り線を通常フローに参加しない絶対配置の子要素へ置き換える現在の
//!   形に是正した。フローに参加しないため `segment` の外形幅は常に
//!   `width` の指定値どおりになり、極小な正値・値 0 のいずれでも比率の
//!   真正性を崩さない。
//! - **凡例のマーカー寸法・間隔**: 同 crate の [`super::legend`] と
//!   数値が不一致だったため揃えた: `legend-marker` は `0.625rem` →
//!   `0.75rem`、`legend-item` の `gap` は `0.375rem` →
//!   `var(--fandhe-space-2, 0.5rem)`。同一 crate 内の凡例表現で寸法が異なる
//!   不整合を解消する。トークン置換箇所はすべて `Theme::empty()`/`css()`
//!   単体利用（テーマ CSS 未注入）時のフォールバック値を付す
//!   （`switch.rs`/`splitter.rs`/`timeline.rs` 等の先例と同型）。
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
use crate::recipe::SlotRecipe;
use fandhe_frontend_headless_ui::fandhe_frontend_core::{text, Node};
use fandhe_frontend_headless_ui::{anatomy, Anatomy};

/// `data-scope="bar-segment"` を固定した anatomy。
const ANATOMY: Anatomy = anatomy("bar-segment");

/// [`SlotRecipe::new`] に渡す slot 一覧。
const SLOTS: &[&str] = &[
    "root",
    "bar",
    "segment",
    "segment-divider",
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
                decl("gap", "var(--fandhe-space-3, 0.75rem)"),
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
                // イシュー #1592 P1 再是正（codex-review 指摘、PR #1865）:
                // 区切り線をレイアウト幅を消費する境界線
                // （`border-inline-end` + `box-sizing: border-box`）で
                // 表現すると、`box-sizing: border-box` の要素は自身の
                // border 幅より外形幅を小さくできないため、1px 未満の
                // 極小な正値 segment（例: 100px 幅で 0.1%）でも border
                // ぶんの 1px へ強制的に拡大され、後続 segment を圧迫して
                // 100% 積み上げの比率真正性を崩す欠陥があった
                // （`data-fandhe-bar-segment-empty` は値 0 の segment しか
                // 救えず、極小の正値 segment は対象外だった）。区切り線を
                // `segment` 自身の境界線ではなく、`position: absolute` で
                // フローから外した子要素 [`segment-divider`] による描画へ
                // 置き換える。絶対配置要素は通常フローの幅計算に一切
                // 参加しないため、`segment` の実描画幅が 1px 未満でも
                // `width`（合計に対する比率）で確保した値をそのまま外形幅
                // として保てる。基準点を持つため `position: relative` を
                // 付ける。
                decl("position", "relative"),
            ],
        )
        .base(
            "segment-divider",
            vec![
                // イシュー #1592 P1 再是正: `inset-inline-end`（論理方向
                // プロパティ）で `direction: rtl` 継承時も物理辺を自動
                // 反転させ、以前の `border-inline-end` と同じ RTL 非依存を
                // 維持する。`position: absolute` のため `segment` の外形幅
                // には一切影響しない（極小 segment でも幅を拡大しない）。
                decl("position", "absolute"),
                decl("inset-block", "0"),
                decl("inset-inline-end", "0"),
                decl("width", "1px"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("pointer-events", "none"),
            ],
        )
        .base(
            "legend",
            vec![
                decl("display", "flex"),
                decl("flex-wrap", "wrap"),
                // イシュー #1592: 生リテラル 0.75rem/1rem を
                // `--fandhe-space-3`/`--fandhe-space-4`（等価値）へ統一。
                decl(
                    "gap",
                    "var(--fandhe-space-3, 0.75rem) var(--fandhe-space-4, 1rem)",
                ),
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
                decl("gap", "var(--fandhe-space-2, 0.5rem)"),
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
                decl("border-radius", "var(--fandhe-radius-full, 9999px)"),
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
    // イシュー #1592 P1 是正（codex-review 指摘）: 「最後の可視（正値）
    // segment」を DOM 順（`:last-child`）ではなく値そのものから求める。
    // `series.values.iter().any(|&v| v < 0.0)` を上で既に拒否しているが、
    // `data::total(series) == 0.0` は非有限値（NaN 等）が混在すると
    // `false` を返し得るため正値の存在を無条件には保証しない
    // （`.claude/rules/coding-rust.md` 「ライブラリコードでの `unwrap()`/
    // `expect()`/`panic!` を避ける」に従い `expect()` は使わず、
    // `rposition` が `None` を返す経路を [`ChartError::ZeroTotal`] で
    // fail-closed に扱う。正値寄与が 1 件も無い＝比率が定義できないという
    // 意味論はモジュール doc「fail-closed」節の `ZeroTotal` 契約と一致する）。
    let Some(last_positive_idx) = series.values.iter().rposition(|&v| v > 0.0) else {
        return Err(ChartError::ZeroTotal);
    };
    let segments: Vec<Node> = categories
        .iter()
        .zip(series.values.iter())
        .enumerate()
        .map(|(idx, (_category, &value))| segment(idx, value, series, idx == last_positive_idx))
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
fn segment(idx: usize, value: f64, series: &data::Series, is_last_visible: bool) -> Node {
    let percent = data::value_percent(series, value);
    let color = series_color_var(idx);
    let style = format!(
        "--fandhe-bar-segment-percent: {}%; background: {color}",
        fmt_coord(percent)
    );
    // イシュー #1592 P1 再是正（codex-review 指摘、PR #1865）: 区切り線を
    // `segment` 自身の border ではなく、フローに参加しない
    // `position: absolute` の子要素 [`segment-divider`]（`recipe()`
    // 参照）で表現する。値が正でも 0 でも `segment` の外形幅は常に
    // `width`（合計に対する比率）のみで決まり、区切り線の有無に左右
    // されない。区切り線が要らないケース（値 0＝隣接カテゴリとの境界を
    // 引く意味がない・最後の可視〔正値〕segment＝`bar` の丸角と重なって
    // 1px の欠けに見える、モジュール doc「セグメント間の区切り線」節参照）
    // では子要素自体を生成しない。
    let children = if value > 0.0 && !is_last_visible {
        vec![ANATOMY.part(
            "segment-divider",
            "span",
            vec![("aria-hidden", "true")],
            vec![],
        )]
    } else {
        vec![]
    };
    ANATOMY.part("segment", "div", vec![("style", style.as_str())], children)
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
    fn root_omits_divider_on_last_positive_segment_when_tail_is_zero() {
        // イシュー #1592 P1 再是正の回帰テスト（codex-review 指摘、
        // PR #1865）: 末尾カテゴリが 0 値（`[100, 0]`）のとき、区切り線
        // （`segment-divider` 子要素）は DOM 上の最終 segment（不可視・
        // 幅 0%、2 番目）ではなく、実際にバー右端を占める最後の正値
        // segment（1 番目、index 0）を基準に「出さない」判定が行われる
        // （＝どちらの segment にも `segment-divider` は現れない）。
        let data = ChartData::new(
            vec!["a".to_string(), "b".to_string()],
            vec![Series::new("s", vec![100.0, 0.0])],
        )
        .unwrap();
        let html = render(&root(&data, "s").unwrap());
        assert_eq!(html.matches("data-part=\"segment-divider\"").count(), 0);
        // segment 単位の div を分割し、いずれの segment にも divider 子要素
        // が含まれないことを直接確認する。
        let segment_divs: Vec<&str> = html
            .split("<div data-scope=\"bar-segment\" data-part=\"segment\"")
            .skip(1)
            .collect();
        assert_eq!(segment_divs.len(), 2, "html: {html}");
        assert!(
            segment_divs[0].contains("--fandhe-bar-segment-percent: 100%")
                && !segment_divs[0].contains("segment-divider"),
            "segment[0]: {}",
            segment_divs[0]
        );
        assert!(
            segment_divs[1].contains("--fandhe-bar-segment-percent: 0%")
                && !segment_divs[1].contains("segment-divider"),
            "segment[1]: {}",
            segment_divs[1]
        );
    }

    #[test]
    fn root_omits_divider_on_zero_value_segments_regardless_of_position() {
        // イシュー #1592 追加是正 + P1 再是正の回帰テスト（codex-review/
        // Cursor Bugbot 再指摘、PR #1865）: 先頭・中間に 0 値カテゴリが
        // あるとき、それぞれの segment には区切り線（`segment-divider`
        // 子要素）が生成されない（値 0 の segment は隣接カテゴリとの境界を
        // 引く意味がないため）。末尾の最後の正値 segment にも同様に
        // divider は生成されない（`bar` の丸角との重なりを避けるため）。
        let data = ChartData::new(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec![Series::new("s", vec![0.0, 0.0, 100.0])],
        )
        .unwrap();
        let html = render(&root(&data, "s").unwrap());
        let segment_divs: Vec<&str> = html
            .split("<div data-scope=\"bar-segment\" data-part=\"segment\"")
            .skip(1)
            .collect();
        assert_eq!(segment_divs.len(), 3, "html: {html}");
        // 先頭（index 0、0%）: divider なし。
        assert!(
            segment_divs[0].contains("--fandhe-bar-segment-percent: 0%")
                && !segment_divs[0].contains("segment-divider"),
            "segment[0]: {}",
            segment_divs[0]
        );
        // 中間（index 1、0%）: 同様に divider なし。
        assert!(
            segment_divs[1].contains("--fandhe-bar-segment-percent: 0%")
                && !segment_divs[1].contains("segment-divider"),
            "segment[1]: {}",
            segment_divs[1]
        );
        // 末尾（index 2、100%、最後の正値）: divider なし。
        assert!(
            segment_divs[2].contains("--fandhe-bar-segment-percent: 100%")
                && !segment_divs[2].contains("segment-divider"),
            "segment[2]: {}",
            segment_divs[2]
        );
        assert_eq!(html.matches("data-part=\"segment-divider\"").count(), 0);
    }

    #[test]
    fn root_renders_divider_between_non_final_positive_segments() {
        // イシュー #1592 P1 再是正の回帰テスト（codex-review 指摘、
        // PR #1865）: 極小な正値 segment（100 分の 1 未満に丸められる値）
        // を含む複数正値カテゴリでは、最後の正値以外の各 segment に
        // `segment-divider` 子要素が 1 件ずつ生成される。区切り線は
        // `position: absolute` の子要素でありレイアウト幅を消費しない
        // ため、極小 segment でも `width`（比率）を拡大しない
        // （`recipe()` の `segment-divider` base 規則参照）。
        let data = ChartData::new(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec![Series::new("s", vec![0.1, 0.1, 99.8])],
        )
        .unwrap();
        let html = render(&root(&data, "s").unwrap());
        // 最後の正値（index 2）を除く 2 件（index 0, 1）に divider が付く。
        assert_eq!(html.matches("data-part=\"segment-divider\"").count(), 2);
        let segment_divs: Vec<&str> = html
            .split("<div data-scope=\"bar-segment\" data-part=\"segment\"")
            .skip(1)
            .collect();
        assert_eq!(segment_divs.len(), 3, "html: {html}");
        assert!(
            segment_divs[0].contains("segment-divider"),
            "{}",
            segment_divs[0]
        );
        assert!(
            segment_divs[1].contains("segment-divider"),
            "{}",
            segment_divs[1]
        );
        assert!(
            !segment_divs[2].contains("segment-divider"),
            "{}",
            segment_divs[2]
        );
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
        assert!(a.contains("var(--fandhe-space-3, 0.75rem)"));
        assert!(a.contains("var(--fandhe-bar-segment-bar-height, 0.75rem)"));
        assert!(a.contains("var(--fandhe-color-bg-muted)"));
        // イシュー #1592 P1 再是正（codex-review 指摘、PR #1865）:
        // 区切り線を `segment` 自身の border ではなく、フローに参加しない
        // 絶対配置の `segment-divider` slot で表現していることを固定する
        // （border-inline-end 方式は極小な正値 segment で box-sizing:
        // border-box の最小幅制約に抵触するため撤回済み）。
        assert!(a.contains(r#"[data-scope="bar-segment"][data-part="segment"] {"#));
        assert!(a.contains("position: relative;"));
        assert!(a.contains(r#"[data-scope="bar-segment"][data-part="segment-divider"] {"#));
        assert!(a.contains("position: absolute;"));
        assert!(a.contains("inset-block: 0;"));
        assert!(a.contains("inset-inline-end: 0;"));
        assert!(!a.contains("border-inline-end"));
        assert!(!a.contains("data-fandhe-bar-segment"));
        assert!(a.contains("var(--fandhe-radius-full, 9999px)"));
        assert!(a.contains("var(--fandhe-space-2, 0.5rem)"));
    }
}
