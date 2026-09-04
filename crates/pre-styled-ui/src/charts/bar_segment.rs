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
//! | 状態 `data-*` | headless 由来の `data-*` は持たない。モジュール専有の
//!   `data-fandhe-bar-segment-end`（最後の正値 segment、区切り線の内部
//!   打ち消し用）・`data-fandhe-bar-segment-empty`（値 0 の segment、同じく
//!   区切り線の内部打ち消し用）の 2 種を新設した（下記「是正した点」参照。
//!   前者は codex-review 指摘、後者は codex-review/Cursor Bugbot 再指摘、
//!   いずれもイシュー #1592） |
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
//!   `border-inline-end: 1px solid var(--fandhe-color-bg)`（+
//!   `box-sizing: border-box`）を追加し、隣接カテゴリの色境界を明確にした
//!   （論理方向プロパティのため `direction: rtl` 継承時も物理辺が自動で
//!   反転し、`box-sizing: border-box` により border 追加後も比率どおりの
//!   外形幅を保つ。当初 `box-shadow: inset -1px 0 0 ...`
//!   〔物理右辺固定〕を用いていたが RTL で打ち消し対象が逆転する欠陥が
//!   あり、codex-review 指摘（イシュー #1592）を受けて是正した）。
//!   最後の可視（正値）セグメントの右端（RTL では左端）は `bar` の
//!   `overflow: hidden` と `border-radius` により直線で切れるため、区切り
//!   線を残すと 1px の欠けに見える。`root()` が系列中「最後の正値」の
//!   index を算出して [`crate::recipe::StateCondition::Attr`]
//!   （`data-fandhe-bar-segment-end` 存在属性、本モジュール専有）で
//!   打ち消す。当初 `StateCondition::LastChild`（`:last-child`、`steps.rs`
//!   先例と同型）を用いていたが、`root()` は値 0 のカテゴリも幅 0% の
//!   segment を生成するため末尾が 0 値だと DOM 上の `:last-child` が
//!   不可視要素に奪われ、実際の右端（可視の最後の正値セグメント）に
//!   欠けが残る欠陥があり、同じ codex-review 指摘で是正した。
//!
//!   **追加是正（codex-review/Cursor Bugbot 再指摘）**: 当初「値 0 の
//!   segment（幅 0%）にも border-inline-end は出力されるが実害はない」と
//!   記していたが誤りだった。`box-sizing: border-box` の要素は自身の
//!   border 幅より小さくはならないため、幅 0% の segment も 1px の外形幅を
//!   持つ。この 1px は flex row 内で他の兄弟の描画順に関係なくレイアウト
//!   幅を消費するため、先頭・中間の 0 値 segment では後続の正値 segment を
//!   1px 分圧迫し、末尾の 0 値 segment では `bar` の右端（RTL では左端）に
//!   1px の隙間を生む。いずれも「比率の真正性を崩さない」（本 doc「意図的
//!   に合わせなかった点」節）契約に反する。位置に関わらず value <= 0.0 の
//!   すべての segment に `data-fandhe-bar-segment-empty`（本モジュール専有
//!   の存在属性）を付与し、`data-fandhe-bar-segment-end` と同じ
//!   `border-inline-end: none` で打ち消す（`segment()` 参照）。
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
                // イシュー #1592 P1 是正（codex-review 指摘）:
                // `box-shadow: inset -1px 0 0 ...` は物理右辺固定のため
                // `direction: rtl` 継承時（flex row の先頭が右端になる）に
                // 区切り線の位置が意図と逆転する欠陥があった。
                // `border-inline-end`（論理方向プロパティ、UA が
                // `direction`/`writing-mode` に応じて物理辺へ自動解決する）
                // へ置き換えて RTL 非依存にする。`box-sizing: border-box`
                // と組み合わせることで border 追加後も `width`（合計に対する
                // 比率）で確保した外形幅は変わらず、100% 積み上げの比率
                // 真正性を崩さない（[`super::bar_list`]/[`crate::card`] 等の
                // border-box 先例と同型）。
                decl("box-sizing", "border-box"),
                decl("border-inline-end", "1px solid var(--fandhe-color-bg)"),
            ],
        )
        // イシュー #1592 P1 是正（codex-review 指摘）: 従来の
        // `StateCondition::LastChild`（`:last-child`）は DOM 上の最終
        // `segment` を対象にしていたが、`root()` は値 0 のカテゴリも幅 0%
        // の segment として生成するため、末尾カテゴリが 0 値の場合
        // （例: `[100, 0]`）は不可視の 2 番目が `:last-child` になり、
        // 実際にバー右端（RTL では左端）を占める最後の可視（正値）
        // segment には区切り線が残って 1px の欠けが生じていた。
        // `root()` が系列中「最後の正値」segment の index を算出し
        // [`segment`] へ渡して `data-fandhe-bar-segment-end` 属性
        // （本モジュール専有の pre-styled-only 状態表現、headless 由来では
        // ない）を付与する方式へ変更し、DOM 順・ゼロ幅要素の有無に
        // 依存しない判定にした。
        .state(
            "segment",
            StateCondition::Attr("data-fandhe-bar-segment-end"),
            vec![decl("border-inline-end", "none")],
        )
        // イシュー #1592 追加是正（codex-review/Cursor Bugbot 指摘）:
        // 値 0 の segment は `border-inline-end` +
        // `box-sizing: border-box` により幅 0% でも 1px の外形幅を持ち、
        // 先頭・中間に現れると後続の正値 segment を 1px 分縮小させ
        // 100% 積み上げの比率真正性を崩す（末尾でのみ無害、というモジュール
        // doc の従来記述は誤りだった）。`data-fandhe-bar-segment-empty`
        // （本モジュール専有の存在属性、`segment()` が value <= 0 の
        // すべての segment に付与）で border-inline-end を無条件に打ち消す。
        .state(
            "segment",
            StateCondition::Attr("data-fandhe-bar-segment-empty"),
            vec![decl("border-inline-end", "none")],
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
    let mut attrs: Vec<(&str, &str)> = vec![("style", style.as_str())];
    if is_last_visible {
        // イシュー #1592 P1 是正: `recipe()` の
        // `StateCondition::Attr("data-fandhe-bar-segment-end")` が拾う
        // 存在属性（`timer.rs`/`calendar.rs` 等の `("data-disabled", "")`
        // 先例と同型の空値存在属性）。
        attrs.push(("data-fandhe-bar-segment-end", ""));
    }
    if value <= 0.0 {
        // イシュー #1592 追加是正（codex-review/Cursor Bugbot 指摘）:
        // 値 0 の segment が border-inline-end 分の外形幅（1px）を持たない
        // よう、位置（先頭・中間・末尾）に関わらず打ち消し属性を付ける。
        attrs.push(("data-fandhe-bar-segment-empty", ""));
    }
    ANATOMY.part("segment", "div", attrs, vec![])
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
    fn root_marks_last_positive_segment_when_tail_is_zero() {
        // イシュー #1592 P1 是正の回帰テスト（codex-review 指摘）:
        // 末尾カテゴリが 0 値（`[100, 0]`）のとき、`data-fandhe-bar-segment-end`
        // は DOM 上の最終 segment（不可視・幅 0%、2 番目）ではなく、実際に
        // バー右端を占める最後の正値 segment（1 番目、index 0）へ付く。
        let data = ChartData::new(
            vec!["a".to_string(), "b".to_string()],
            vec![Series::new("s", vec![100.0, 0.0])],
        )
        .unwrap();
        let html = render(&root(&data, "s").unwrap());
        assert_eq!(html.matches("data-fandhe-bar-segment-end").count(), 1);
        // segment 単位の div を分割し、100% 幅の segment（1 番目）にのみ
        // 属性が付いていて、0% 幅の segment（2 番目）には付いていないことを
        // 直接確認する。
        let segment_divs: Vec<&str> = html
            .split("<div data-scope=\"bar-segment\" data-part=\"segment\"")
            .skip(1)
            .collect();
        assert_eq!(segment_divs.len(), 2, "html: {html}");
        assert!(
            segment_divs[0].contains("--fandhe-bar-segment-percent: 100%")
                && segment_divs[0].contains("data-fandhe-bar-segment-end"),
            "segment[0]: {}",
            segment_divs[0]
        );
        assert!(
            segment_divs[1].contains("--fandhe-bar-segment-percent: 0%")
                && !segment_divs[1].contains("data-fandhe-bar-segment-end"),
            "segment[1]: {}",
            segment_divs[1]
        );
    }

    #[test]
    fn root_marks_leading_and_middle_zero_segments_as_empty() {
        // イシュー #1592 追加是正の回帰テスト（codex-review/Cursor Bugbot
        // 再指摘）: 先頭・中間に 0 値カテゴリがあるとき、それぞれの
        // segment に `data-fandhe-bar-segment-empty` が付き、区切り線
        // （border-inline-end）が内部で打ち消されることを固定する。
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
        // 先頭（index 0、0%）: empty 属性あり、end 属性なし。
        assert!(
            segment_divs[0].contains("--fandhe-bar-segment-percent: 0%")
                && segment_divs[0].contains("data-fandhe-bar-segment-empty")
                && !segment_divs[0].contains("data-fandhe-bar-segment-end"),
            "segment[0]: {}",
            segment_divs[0]
        );
        // 中間（index 1、0%）: 同様に empty 属性あり、end 属性なし。
        assert!(
            segment_divs[1].contains("--fandhe-bar-segment-percent: 0%")
                && segment_divs[1].contains("data-fandhe-bar-segment-empty")
                && !segment_divs[1].contains("data-fandhe-bar-segment-end"),
            "segment[1]: {}",
            segment_divs[1]
        );
        // 末尾（index 2、100%、最後の正値）: end 属性あり、empty 属性なし。
        assert!(
            segment_divs[2].contains("--fandhe-bar-segment-percent: 100%")
                && segment_divs[2].contains("data-fandhe-bar-segment-end")
                && !segment_divs[2].contains("data-fandhe-bar-segment-empty"),
            "segment[2]: {}",
            segment_divs[2]
        );
        assert_eq!(html.matches("data-fandhe-bar-segment-empty").count(), 2);
        assert_eq!(html.matches("data-fandhe-bar-segment-end").count(), 1);
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
        // イシュー #1592 P1 是正（codex-review 指摘）: RTL 非依存の区切り線
        // （論理方向プロパティ）とゼロ値末尾セグメントを跨いだ「最後の
        // 可視セグメント」判定（data 属性）を固定する。
        assert!(a.contains("border-inline-end: 1px solid var(--fandhe-color-bg)"));
        assert!(a.contains(r#"[data-fandhe-bar-segment-end]"#));
        assert!(a.contains(r#"[data-fandhe-bar-segment-empty]"#));
        assert!(a.contains("border-inline-end: none"));
        assert!(a.contains("var(--fandhe-radius-full, 9999px)"));
        assert!(a.contains("var(--fandhe-space-2, 0.5rem)"));
    }
}
