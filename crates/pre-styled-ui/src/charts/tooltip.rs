//! チャートツールチップ（イシュー #847、chakra-ui `charts/tooltip.md`
//! 相当。[`crate::tooltip`]（汎用 headless Tooltip、hover/focus で JS が
//! 表示制御する）とは別物であり、モジュールパス `charts::tooltip` で区別する
//! （chakra-ui 側も「chart 専用であり general-purpose Tooltip とは別」と
//! 明記しており対応が一致する）。
//!
//! # SSR ツールチップ方式（JS を使わない設計）
//!
//! マウス追従型のリッチツールチップ（recharts `<Tooltip>` の cursor 追従）は
//! JS ランタイムが必須のためスコープ外とする（`crates/pre-styled-ui/src/charts/mod.rs`
//! のスコープ外節参照）。代わりに、データ点要素（`datum` slot）へ:
//!
//! 1. 子 `<title>` 要素（ブラウザネイティブな hover 表示。SVG 標準機能で
//!    JS 不要）
//! 2. `aria-label` 属性（スクリーンリーダー向け、`<title>` と同一文字列）
//! 3. [`crate::recipe::StateCondition::Hover`] による `:hover` 時の視覚的
//!    強調（`stroke`/`stroke-width` 変更、CSS のみ。base で背景色ハロー
//!    （`stroke: var(--fandhe-color-bg)`）を敷いてから hover で前景色
//!    （`--fandhe-color-fg`）へ切り替える方式とし、SVG の既定 `stroke: none`
//!    に依存しない。イシュー #1425 で当該規則は
//!    `@media (hover: hover)` 内・セレクタ末尾 `:not([data-disabled])` 付き
//!    で出力される形へ変更された。タッチ端末での hover 貼り付き回避と
//!    disabled 規則との勝敗を記述順に依存させないための共通対策であり、
//!    イシュー #1425 時点では本モジュールのコード自体は無変更だった。
//!    その後 #1593 で base に背景色ハロー（上記 `stroke: var(--fandhe-color-bg)`）
//!    と transition を追加している）
//!
//! を組み合わせて埋め込み、JS なしで「ホバーで詳細が分かる」体験を実現する。
//!
//! `stroke`/`stroke-width` は本モジュールの CSS が専有する。呼び出し側
//! （[`datum`]）は `fill` 等の見た目属性のみを渡す契約とし、CSS 側で
//! `fill` を宣言しない（SVG ではプレゼンテーション属性より CSS の
//! `fill`/`stroke` が優先されるため、CSS で `fill` を書くと呼び出し側の
//! 系列色指定が潰れる）。
//!
//! # 参考サイト基準への調整（イシュー #1593）
//!
//! 参照 4 サイト（ark-ui / chakra-ui / Radix Primitives / Radix Themes）に
//! 対応部品が無いため、評価軸は内部整合のみとした。
//!
//! | 観点 | 判定 |
//! |---|---|
//! | サイズ / バリアント / colorPalette | 非該当（表示専用・参照軸なし） |
//! | 色 | 是正（hover ストローク色を `--fandhe-color-accent-emphasized` から `--fandhe-color-fg` へ） |
//! | 状態・`data-*` | 変更なし |
//! | ダーク | 是正（`accent-emphasized` の dark 値 `#63b3ed` が `chart-1` の dark 値と同一で hover 強調が消える問題を解消） |
//! | フォーカス | 非該当（`tabindex` 付与は anatomy 変更のため別判断） |
//! | 余白・角丸・影 | 非該当 |
//! | hover / disabled / transition | 是正（`transition_declarations` 未導入だったため追加） |
//! | 内部整合 | 是正（隣接・重なった異系列データ点の識別のため base に背景色ハローを追加） |
//!
//! ## 是正した点
//!
//! - **hover ストローク色**: `--fandhe-color-accent-emphasized` は dark で
//!   `#63b3ed` を返し、6 系列パレットの `chart-1`（dark `#63b3ed`）と
//!   完全一致するため、ダークテーマで系列 1 のデータ点は hover 強調が
//!   視覚的に消えていた。`--fandhe-color-fg` は light/dark とも背景との
//!   コントラスト比が高く（約 15:1）、6 系列色のいずれとも一致しない
//!   ため採用した（`--fandhe-color-focus-ring`（focus-ring 色トークン）は dark で同じ `#63b3ed`
//!   衝突を起こすため候補から外した）。
//! - **transition**: hover 状態遷移を持つ唯一のインタラクティブ slot
//!   だったが `transition_declarations` が未導入だった
//!   （`docs/design/pre-styled-ui-interaction-visual-language.md` §5）。
//!   `prefers-reduced-motion: reduce` は [`crate::theme::Theme::to_css`]
//!   の `--fandhe-motion-duration-*: 0ms` 一括上書きで自動対応するため、
//!   本モジュールに `@media` を追加する必要はない。
//! - **背景色ハロー**: 隣接・重なった異系列のデータ点の輪郭を識別できる
//!   よう、base に `stroke: var(--fandhe-color-bg)` / `stroke-width: 1`
//!   を追加した（[`crate::area_chart`] の `point` slot と同型）。
//!
//! ## 意図的に合わせなかった点
//!
//! - 6 系列パレット（`chart-1`〜`chart-6`、[`crate::theme`]）自体の見直し
//!   は本イシューのスコープ外とした（`theme.rs` の変更は docs-site の
//!   契約テストへ波及し他部品にも影響するため）。
//! - `vector-effect: non-scaling-stroke` 等のスケーリング対策はスコープ
//!   外とした（兄弟部品との線幅の見え方の乖離を避けるため）。
//!
//! # shadcn/ui 突合（イシュー #2086）
//!
//! shadcn/ui Charts（tooltip ページ）の registry 9 バリアントを突合した。
//!
//! | shadcn/ui | 対応 |
//! |---|---|
//! | `chart-tooltip-default`（見出し + 複数系列行） | [`datum_label_lines`] を純追加（`<title>`/`aria-label` の複数行テキスト） |
//! | `chart-tooltip-label-custom` / `-label-formatter` | `heading: Option<&str>` に呼び出し側の任意文字列を渡せる（既存 API の明示化） |
//! | `chart-tooltip-label-none` | `heading: None` |
//! | `chart-tooltip-formatter`（値の書式） | `entries` の値は呼び出し側が整形済み文字列で渡す契約（`.claude/rules/coding-rust.md` §「数値・日時整形は UI コンポーネント層の責務外」と同じ判断軸。本モジュールは値を整形しない） |
//! | `chart-tooltip-advanced`（Total footer） | `footer: Option<&str>` |
//! | `chart-tooltip-indicator-line` / `-indicator-none` | 採用しない。ツールチップ DOM・indicator バリアントは #2129/#2131 のスコープ |
//! | `chart-tooltip-icons` | 採用しない。ツールチップ DOM への icon 合成は #2129 のスコープ |
//!
//! [`datum`]/[`datum_label`]/[`css`] の出力はバイト不変（golden 純追加
//! 原則、`tests/charts_parts_css.rs`/`tests/charts_legend_tooltip.rs` 参照）。
//! マウス追従・hover 配線は #2128 系（#2129/#2130/#2131）。凡例の系列
//! トグルの SSR 構造（`[data-hidden]` で `tooltip-item` を隠す CSS 規則）は
//! 本イシュー（#2133）で追加した。SSR 側は `tooltip-item` 自体へ
//! `data-hidden` を出力せず（`layer_from_entries` の引数拡張は #2134 へ
//! 引き継ぐ、モジュール doc「本イシューのスコープ外」節）、wasm-full 側
//! （#2134）が click 時に付け外しする設計。

use super::data::SeriesColor;
use super::svg::{fmt_coord, fmt_value};
use crate::css::decl;
use crate::recipe::{
    focus_ring_declarations, transition_declarations, FocusRingColor, FocusRingOffset,
    MotionDuration, SlotRecipe, StateCondition,
};
use fandhe_frontend_headless_ui::fandhe_frontend_core::{el, text, Node};

/// 本モジュールの anatomy scope（[`super::axis`]/[`super::grid`] と共有）。
const SCOPE: &str = "chart";

/// [`recipe`] に渡す slot 一覧（イシュー #2129 で hit-area・SSR ツールチップ
/// DOM の 9 slot を純追加。`datum` は先頭を維持し既存 golden の base 出力
/// 順を不変に保つ）。
const SLOTS: &[&str] = &[
    "datum",
    "hit-area",
    "frame",
    "tooltip-layer",
    "tooltip",
    "tooltip-label",
    "tooltip-item",
    "tooltip-indicator",
    "tooltip-name",
    "tooltip-value",
];

/// [`datum`] が固定する属性名（呼び出し側 `attrs` からの偽装を fail-closed
/// で除去する対象。`crates/pre-styled-ui/src/table.rs` の `COLUMN_HEADER_RESERVED`
/// と同型の判断）。
const DATUM_RESERVED: &[&str] = &["data-scope", "data-part", "cx", "cy", "r", "aria-label"];

/// Tooltip（データ点強調表示）の recipe（scope `"chart"`、[`SLOTS`] の
/// 1 パーツ）。
fn recipe() -> SlotRecipe {
    let mut base = vec![
        decl("cursor", "default"),
        decl("stroke", "var(--fandhe-color-bg)"),
        decl("stroke-width", "1"),
    ];
    base.extend(transition_declarations(
        "stroke, stroke-width",
        MotionDuration::Fast,
    ));
    SlotRecipe::new(SCOPE, SLOTS)
        .base("datum", base)
        .base(
            "hit-area",
            vec![decl("outline", "none"), decl("cursor", "default")],
        )
        .base("frame", vec![decl("position", "relative")])
        .base(
            "tooltip-layer",
            vec![
                decl("position", "absolute"),
                decl("inset", "0"),
                decl("pointer-events", "none"),
            ],
        )
        .base(
            "tooltip",
            vec![
                decl("position", "absolute"),
                decl("left", "var(--fandhe-chart-tooltip-x, 0px)"),
                decl("top", "var(--fandhe-chart-tooltip-y, 0px)"),
                decl(
                    "transform",
                    "translate(-50%, calc(-100% - var(--fandhe-space-2)))",
                ),
                decl("pointer-events", "none"),
                decl("background", "var(--fandhe-color-bg)"),
                decl("color", "var(--fandhe-color-fg)"),
                decl("border", "1px solid var(--fandhe-color-border)"),
                decl("border-radius", "var(--fandhe-radius-md, 0.375rem)"),
                decl("box-shadow", "var(--fandhe-shadow-md)"),
                decl("padding", "var(--fandhe-space-2) var(--fandhe-space-3)"),
                decl("font-size", "var(--fandhe-font-font-size-xs)"),
                decl("font-family", "var(--fandhe-font-font-body)"),
                decl("white-space", "nowrap"),
            ],
        )
        .base(
            "tooltip-label",
            vec![
                decl("font-weight", "600"),
                decl("margin-bottom", "var(--fandhe-space-1)"),
            ],
        )
        .base(
            "tooltip-item",
            vec![
                decl("display", "flex"),
                decl("align-items", "center"),
                decl("gap", "var(--fandhe-space-2)"),
            ],
        )
        .base(
            "tooltip-indicator",
            vec![
                decl("display", "inline-block"),
                decl("width", "var(--fandhe-space-2)"),
                decl("height", "var(--fandhe-space-2)"),
                decl("border-radius", "2px"),
                decl(
                    "background",
                    "var(--fandhe-chart-tooltip-color, currentColor)",
                ),
            ],
        )
        .base("tooltip-name", vec![])
        .base(
            "tooltip-value",
            vec![
                decl("margin-left", "auto"),
                decl("font-variant-numeric", "tabular-nums"),
            ],
        )
        .state(
            "datum",
            StateCondition::Hover,
            vec![
                decl("stroke", "var(--fandhe-color-fg)"),
                decl("stroke-width", "2"),
            ],
        )
        .state(
            "hit-area",
            StateCondition::FocusVisible,
            focus_ring_declarations(FocusRingColor::Token, FocusRingOffset::Outside),
        )
        // イシュー #2133: 各チャートの系列要素と同じ `[data-hidden]`
        // セレクタで `tooltip-item` 行を非表示にする（凡例トグルで隠した
        // 系列のツールチップ行も連動して消える。末尾純追加、既存ブロックは
        // 不変）。SSR 側は `tooltip-item` へ `data-hidden` を出力しない
        // （モジュール doc「本イシューのスコープ外」節、#2134 が付け外しを
        // 担う）。
        .state(
            "tooltip-item",
            StateCondition::Attr("data-hidden"),
            vec![decl("display", "none")],
        )
}

/// Tooltip の静的 CSS 全文。
#[must_use]
pub fn css() -> String {
    recipe().css()
}

/// カテゴリ・系列名・値からツールチップ本文（`<title>`/`aria-label` 共用）の
/// 決定的文字列を組み立てる。値の文字列化は [`super::svg::fmt_coord`] のみを
/// 経由する（数値の決定的文字列化の一元化、`crates/pre-styled-ui/src/charts/mod.rs`
/// 冒頭 doc 不変条件 2）。
///
/// # 系列設定（`label`/`color`/`icon`、イシュー #2077）との契約
///
/// `series` 引数には呼び出し側が [`super::data::Series::display_label`]
/// の戻り値を渡す想定とする（`label` が未設定なら `name` を返すため、
/// 既存呼び出しは変更不要）。本関数自体は `series` を単なる文字列として
/// 連結するのみで `label`/`color`/`icon` を消費・解決しない。SSR
/// ツールチップ DOM 側で `color`/`icon` を表示へ反映する（例: ツールチップ
/// 内へマーカー色・アイコンを合成する）ことは本イシューのスコープ外とし、
/// 実装するときは #2129（ツールチップ DOM・hit-area）を参照する。
#[must_use]
pub fn datum_label(category: &str, series: &str, value: f64) -> String {
    format!("{category} · {series}: {}", fmt_coord(value))
}

/// 複数行のツールチップ本文（`<title>`/`aria-label` 共用）を組み立てる
/// （shadcn/ui `chart-tooltip-default`/`chart-tooltip-advanced` 相当、
/// イシュー #2086）。[`datum_label`] は 1 系列・1 行専用だが、本関数は
/// 見出し（`heading`）+ 複数系列行（`entries`）+ 合計等の末尾行
/// （`footer`）を `'\n'` 区切りで連結する。
///
/// `entries` の各要素は `(系列表示ラベル, 整形済み値文字列)`。値の数値
/// フォーマットは呼び出し側の責務（`.claude/rules/coding-rust.md` の
/// 「数値・日時整形は UI コンポーネント層の責務外」判断軸、[`fmt_coord`]
/// を使いたい場合は呼び出し側で呼ぶ）とし、本関数は文字列を単純連結する
/// のみで数値を解釈しない。
///
/// `heading`/`footer` が `None` の場合は該当行を出力しない。`entries` が
/// 空でも panic せず、`heading`/`footer` のみ（両方 `None` かつ `entries`
/// も空なら空文字列）を返す決定的な文字列を返す（fail-soft。呼び出し元が
/// 空の系列集合を渡す状況を事前に弾く責務は [`super::data::ChartData`] 側
/// が担う）。
///
/// 出力先（`<title>`/`aria-label`）はともに [`fandhe_frontend_core::render`]
/// の既定エスケープ（REQ-1）を経由するため改行はそのまま実体化されず通る
/// （`&`/`<`/`>`/`"`/`'` のみ実体参照化される）。
#[must_use]
pub fn datum_label_lines(
    heading: Option<&str>,
    entries: &[(&str, &str)],
    footer: Option<&str>,
) -> String {
    let mut lines: Vec<String> = Vec::new();
    if let Some(heading) = heading {
        lines.push(heading.to_string());
    }
    for (series, value_text) in entries {
        lines.push(format!("{series}: {value_text}"));
    }
    if let Some(footer) = footer {
        lines.push(footer.to_string());
    }
    lines.join("\n")
}

/// データ点（`<circle>`）を組み立てる。子 `<title>` 要素と `aria-label`
/// 属性の両方に `label` を埋め込む（モジュール doc「SSR ツールチップ方式」
/// 参照）。
///
/// `attrs` に本関数が固定するキー（[`DATUM_RESERVED`]）が含まれていても
/// 除去してから連結する（fail-closed。呼び出し側は `fill` 等の見た目属性
/// のみを追加する想定、後続チャート部品 #848〜#851 の消費経路）。
///
/// 座標は [`super::svg::fmt_coord`] のみを経由して文字列化する。`cx`/`cy`/`r`
/// が非有限の場合の出力は未規定（[`super::svg::fmt_coord`] の契約と同じく、
/// 呼び出し元は [`super::data::ChartData::new`] の検証を経由した有限値のみを
/// 渡す契約とする）。
#[must_use]
pub fn datum<'a>(cx: f64, cy: f64, r: f64, label: &str, attrs: Vec<(&'a str, &'a str)>) -> Node {
    let (cx, cy, r) = (fmt_coord(cx), fmt_coord(cy), fmt_coord(r));
    let mut merged: Vec<(&str, &str)> = vec![
        ("data-scope", SCOPE),
        ("data-part", "datum"),
        ("cx", cx.as_str()),
        ("cy", cy.as_str()),
        ("r", r.as_str()),
        ("aria-label", label),
    ];
    merged.extend(
        attrs
            .into_iter()
            .filter(|(k, _)| !DATUM_RESERVED.iter().any(|r| k.eq_ignore_ascii_case(r))),
    );
    el(
        "circle",
        merged,
        vec![el("title", vec![], vec![text(label)])],
    )
}

// ---------------------------------------------------------------------
// hit-area・SSR ツールチップ DOM（イシュー #2129、親 #2128）
//
// #2128 の設計方針 1（SSR 側）を担う。データ点・バー・スライスごとに
// 透明な hit-area（[`hit_area_rect`]/[`hit_area_circle`]/[`hit_area_path`]）
// と、`hidden` 属性で SSR 時は非表示のツールチップ本体（[`layer`]/
// [`layer_from_entries`]）を出力する。JS ランタイム（#2130）は
// `pointer-events`/`hidden`/`--fandhe-chart-tooltip-x`/
// `--fandhe-chart-tooltip-y` の付け外し・更新のみを行い、文字列を組み
// 立てない（値・ラベルは本モジュールが既定エスケープ経由で埋め込み済み、
// REQ-1）。
//
// # 各チャートへの引き継ぎ契約
//
// - **配置規則**（#2130 が唯一のロケータとして使う）: [`layer`] が返す
//   `tooltip-layer` は、hit-area を含む `<svg>` の直後の兄弟要素であり、
//   両者の親要素は `position: relative`（[`frame`] または各 styled
//   チャートの `root` slot）である。
// - hit-area は SSR では `fill="none"`/`pointer-events="none"` を出力する
//   （JS 無効時に既存の `datum`/bar の `:hover` と `<title>` を一切妨げない、
//   progressive enhancement）。#2130 がハイドレーション時に `pointer-events`
//   を `all` へ切り替える。
// - `tabindex="-1"`（プログラム的フォーカスのみ）。roving tabindex（`0`/`-1`）
//   への昇格と `svg_root` の `role="img"` 内包問題の扱いは #2130/#2128 への
//   引き継ぎとし、本モジュールでは変更しない。
// - `data-active` は本モジュールでは出力しない（#2130 が hit-area・視覚
//   要素へ付与し、#2131 が CSS で消費する語彙として予約する）。

/// ツールチップ本文 1 行（1 系列分）。名前は
/// [`super::data::Series::display_label`]、色は
/// [`super::data::ChartData::series_color_var`] を呼び出し側が渡す契約
/// （[`entries_from_chart_data`] 参照）。
#[derive(Debug, Clone, PartialEq)]
pub struct TooltipRow {
    /// 系列の表示ラベル（`tooltip-name` のテキストに使う。
    /// [`super::data::Series::display_label`] 相当）。`data-series` 属性
    /// には代わりに [`TooltipRow::series`]（系列の生の名前）を使う（両者が
    /// 一致しない呼び出し元＝scatter で `data-series` に表示用の合成文字列
    /// が漏れるのを構造的に防ぐ、イシュー #2129 レビュー指摘）。
    pub name: String,
    /// 系列の生の名前（`tooltip-item` の `data-series` 属性値。hit-area・
    /// `point`/`bar` 等の同属性と同じ値を渡す契約）。
    pub series: String,
    /// 系列値（生の `f64`。文字列化は [`super::svg::fmt_coord`] のみを
    /// 経由する、[`super`] モジュール doc の不変条件 2）。
    pub value: f64,
    /// `tooltip-indicator` の `style` へ渡す色。[`super::data::SeriesColor`]
    /// 型そのものを保持することで、`var(--fandhe-color-<allowlisted>)`
    /// 固定形以外の任意文字列が `style` 属性へ混入する経路を型で閉じる
    /// （REQ-1 相当、`.claude/rules/security.md` A03。以前は `String` で
    /// 「呼び出し側の任意文字列を連結しない契約」をコメントのみで保証して
    /// いたが、[`layer_from_entries`] が `pub` である以上コメントは強制力を
    /// 持たないため、イシュー #2129 レビュー指摘を受けて型で保証する）。
    pub color: SeriesColor,
}

/// ツールチップ 1 個分（1 カテゴリ、または scatter の系列 × 点 1 個）。
#[derive(Debug, Clone, PartialEq)]
pub struct TooltipEntry {
    /// カテゴリ / 点の 0 起点序数（hit-area・ツールチップ双方の
    /// `data-index` と一致させる）。
    pub index: usize,
    /// scatter（系列 × 点）のように系列単位で分かれる場合の系列表示名。
    /// カテゴリ単位（bar/line/area/pie 等）では `None`。
    pub series: Option<String>,
    /// 見出し（カテゴリ名、または scatter の点ラベル）。
    pub label: String,
    /// 系列ごとの行（[`TooltipRow`]）。
    pub rows: Vec<TooltipRow>,
}

/// [`super::data::ChartData`] からカテゴリ単位の [`TooltipEntry`] 一覧を
/// 組み立てる（カテゴリ 1 件 = エントリ 1 件、行 = 全系列）。scatter は
/// 系列 × 点単位のため本関数を使わず、独自に [`TooltipEntry`] を組み立てて
/// [`layer_from_entries`] へ渡す（[`TooltipEntry::series`] 参照）。
#[must_use]
pub fn entries_from_chart_data(data: &super::data::ChartData) -> Vec<TooltipEntry> {
    data.categories()
        .iter()
        .enumerate()
        .map(|(index, category)| {
            let rows = data
                .series()
                .iter()
                .enumerate()
                .map(|(series_index, series)| TooltipRow {
                    name: series.display_label().to_string(),
                    series: series.name.clone(),
                    value: series.values[index],
                    color: series.color.clone().unwrap_or_else(|| {
                        SeriesColor::chart_slot(series_index % 6 + 1)
                            .expect("series_index % 6 + 1 は常に 1..=6 の範囲内")
                    }),
                })
                .collect();
            TooltipEntry {
                index,
                series: None,
                label: category.clone(),
                rows,
            }
        })
        .collect()
}

/// hit-area の `aria-label`（カテゴリ + 全系列の値、[`datum_label`] と同型の
/// 区切り記号 `" · "`）。
///
/// `row.value` はピクセル座標ではなくデータ値そのものであるため
/// [`super::svg::fmt_value`] を経由する（[`crate::charts`] モジュール doc
/// 不変条件 2「ピクセル座標なら `fmt_coord`、データ値なら `fmt_value`」。
/// PR #2261 codex-review P1 指摘: `fmt_coord`（小数点以下 2 桁固定）では
/// `0.001` のような小さいデータ値が `aria-label`/ツールチップ本文で `0`
/// に丸め落ちしていた）。`|v| >= 0.01` では `fmt_coord` とバイト同一のため
/// [`datum_label`] と同一文字列になるのは単一系列かつこの範囲の値のみ。
#[must_use]
pub fn hit_area_label(entry: &TooltipEntry) -> String {
    let mut parts = vec![entry.label.clone()];
    parts.extend(
        entry
            .rows
            .iter()
            .map(|row| format!("{}: {}", row.name, fmt_value(row.value))),
    );
    parts.join(" · ")
}

/// hit-area が固定する属性名（呼び出し側からの偽装余地を作らないため、
/// [`hit_area_rect`]/[`hit_area_circle`]/[`hit_area_path`] は `attrs` を
/// 受け取らず、幾何・`index`・`series`・`label` のみを引数に取る設計とする）。
fn hit_area_attrs<'a>(
    index: &'a str,
    series: Option<&'a str>,
    label: &'a str,
) -> Vec<(&'a str, &'a str)> {
    let mut attrs: Vec<(&str, &str)> = vec![
        ("data-scope", SCOPE),
        ("data-part", "hit-area"),
        ("data-index", index),
    ];
    if let Some(series) = series {
        attrs.push(("data-series", series));
    }
    attrs.extend([
        ("fill", "none"),
        ("pointer-events", "none"),
        ("tabindex", "-1"),
        ("aria-label", label),
    ]);
    attrs
}

/// 矩形 hit-area（bar/line/area の帯型ヒットターゲット）。`<title>` は
/// 付けない（`pointer-events="none"` のためホバー表示は成立しない、
/// モジュール doc「引き継ぎ契約」参照）。
#[must_use]
pub fn hit_area_rect(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    index: usize,
    series: Option<&str>,
    label: &str,
) -> Node {
    let (x, y, width, height) = (
        fmt_coord(x),
        fmt_coord(y),
        fmt_coord(width),
        fmt_coord(height),
    );
    let index_str = index.to_string();
    let mut merged: Vec<(&str, &str)> = vec![
        ("x", x.as_str()),
        ("y", y.as_str()),
        ("width", width.as_str()),
        ("height", height.as_str()),
    ];
    merged.extend(hit_area_attrs(&index_str, series, label));
    el("rect", merged, vec![])
}

/// 円形 hit-area（scatter の各点用）。
#[must_use]
pub fn hit_area_circle(
    cx: f64,
    cy: f64,
    r: f64,
    index: usize,
    series: Option<&str>,
    label: &str,
) -> Node {
    let (cx, cy, r) = (fmt_coord(cx), fmt_coord(cy), fmt_coord(r));
    let index_str = index.to_string();
    let mut merged: Vec<(&str, &str)> =
        vec![("cx", cx.as_str()), ("cy", cy.as_str()), ("r", r.as_str())];
    merged.extend(hit_area_attrs(&index_str, series, label));
    el("circle", merged, vec![])
}

/// 弧型 hit-area（pie/donut/radial の扇形・リング用）。`d` は
/// [`super::svg::PathBuilder`]/[`super::pie`] が生成した値のみを渡す契約
/// （呼び出し元の任意文字列を連結しない、`.claude/rules/security.md` A03）。
///
/// `evenodd` は `d` が [`super::pie::annulus_full_ring_path`]（内外 2 円を
/// 同方向に描く全周リング）由来のときに `true` を渡す契約とする
/// （イシュー #2129 codex-review 指摘）。この形状は `fill-rule="evenodd"`
/// が無いと内側の穴も塗りつぶされ、`pointer-events="all"`
/// （ハイドレーション後、#2130）で外側リングが内側カテゴリの hit-area を
/// 覆ってしまう。呼び出し側の描画コード（`segment_attrs`/`track_attrs`/
/// `bar_attrs` 等）が同じ `d` に対して既に `fill-rule="evenodd"` を
/// 付けているかどうかと必ず一致させる。
#[must_use]
pub fn hit_area_path(
    d: &str,
    index: usize,
    series: Option<&str>,
    label: &str,
    evenodd: bool,
) -> Node {
    let index_str = index.to_string();
    let mut merged: Vec<(&str, &str)> = vec![("d", d)];
    if evenodd {
        merged.push(("fill-rule", "evenodd"));
    }
    merged.extend(hit_area_attrs(&index_str, series, label));
    el("path", merged, vec![])
}

/// 1 エントリ分のツールチップ本体（`data-part="tooltip"`）を組み立てる
/// （内部ヘルパ、[`layer_from_entries`] のみが呼ぶ）。`active` が
/// `entry.index` と一致しない場合のみ `hidden` を出力する。
fn tooltip_node(entry: &TooltipEntry, active: Option<(usize, Option<&str>)>) -> Node {
    let index_str = entry.index.to_string();
    let mut attrs: Vec<(&str, &str)> = vec![
        ("data-scope", SCOPE),
        ("data-part", "tooltip"),
        ("data-index", index_str.as_str()),
    ];
    // イシュー #2129 codex-review 指摘: scatter（系列 × 点単位）は
    // `entry.series` に系列名を持つが、この属性が無いと同じ点番号
    // （`data-index`）を持つ複数系列のツールチップを hit-area の
    // `data-series` と組で照合できない。hit-area 側（[`hit_area_attrs`]）
    // と同じ語彙・同じ省略規則（カテゴリ単位＝`None` のときは省略）で
    // 揃える。
    if let Some(series) = entry.series.as_deref() {
        attrs.push(("data-series", series));
    }
    // `active` は `(index, series)` の組で比較する（`index` 単独比較だと
    // scatter のように系列をまたいで `index` が重複するモデルで複数
    // エントリが同時に active 扱いになってしまうため、イシュー #2129
    // codex-review 指摘）。カテゴリ単位（[`layer`] 経由、`entry.series`
    // は常に `None`）は従来どおり `index` のみで一意に定まる。
    if active != Some((entry.index, entry.series.as_deref())) {
        attrs.push(("hidden", ""));
    }

    let mut children: Vec<Node> = vec![el(
        "div",
        vec![("data-scope", SCOPE), ("data-part", "tooltip-label")],
        vec![text(entry.label.as_str())],
    )];
    for row in &entry.rows {
        let style = format!("--fandhe-chart-tooltip-color: {}", row.color.var());
        // `row.value` はピクセル座標ではなくデータ値そのものであるため
        // `fmt_value` を経由する（[`hit_area_label`] と同じ根拠、PR #2261
        // codex-review P1 指摘）。
        let value_text = fmt_value(row.value);
        children.push(el(
            "div",
            vec![
                ("data-scope", SCOPE),
                ("data-part", "tooltip-item"),
                ("data-series", row.series.as_str()),
            ],
            vec![
                el(
                    "span",
                    vec![
                        ("data-scope", SCOPE),
                        ("data-part", "tooltip-indicator"),
                        ("style", style.as_str()),
                        ("aria-hidden", "true"),
                    ],
                    vec![],
                ),
                el(
                    "span",
                    vec![("data-scope", SCOPE), ("data-part", "tooltip-name")],
                    vec![text(row.name.as_str())],
                ),
                el(
                    "span",
                    vec![("data-scope", SCOPE), ("data-part", "tooltip-value")],
                    vec![text(value_text.as_str())],
                ),
            ],
        ));
    }
    el("div", attrs, children)
}

/// [`super::data::ChartData`] から直接ツールチップ層を組み立てる（内部で
/// [`entries_from_chart_data`] を経由する薄いラッパ）。`active` が
/// `Some(index)` の場合、そのカテゴリのツールチップのみ `hidden` を省く
/// （静的な「開いた」状態の Demo 用、#2131 が消費）。
#[must_use]
pub fn layer(data: &super::data::ChartData, active: Option<usize>) -> Node {
    layer_from_entries(
        &entries_from_chart_data(data),
        active.map(|index| (index, None)),
    )
}

/// 事前に組み立てた [`TooltipEntry`] 列からツールチップ層を組み立てる
/// （scatter のように系列 × 点単位で `TooltipEntry` を独自構築する呼び出し
/// 元向け）。層全体へ `aria-hidden="true"` を固定する（値は hit-area の
/// `aria-label` と `datum` の `<title>` が既に提供しており、視覚的な重複
/// 表示のため）。`active` は `(index, series)` の組で比較する（[`layer`]
/// はカテゴリ単位＝`series: None` で呼ぶため `index` 単独比較と等価だが、
/// scatter のように `index` が系列をまたいで重複するモデルでも一意に
/// 1 エントリだけを開ける、イシュー #2129 codex-review 指摘）。
#[must_use]
pub fn layer_from_entries(entries: &[TooltipEntry], active: Option<(usize, Option<&str>)>) -> Node {
    let children = entries
        .iter()
        .map(|entry| tooltip_node(entry, active))
        .collect();
    el(
        "div",
        vec![
            ("data-scope", SCOPE),
            ("data-part", "tooltip-layer"),
            ("aria-hidden", "true"),
        ],
        children,
    )
}

/// `<svg>` とツールチップ層を包む外側ラッパ（`data-part="frame"`、素の
/// `<svg data-part="root">` を返す基盤 3 部品（bar/scatter/radar）専用。
/// 既に `div[root] > svg[plot|chart]` を持つ styled 6 部品は使わず、各
/// root recipe の base へ `position: relative` を直接追加する）。
#[must_use]
pub fn frame(children: Vec<Node>) -> Node {
    el(
        "div",
        vec![("data-scope", SCOPE), ("data-part", "frame")],
        children,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fandhe_frontend_core::render;

    #[test]
    fn datum_label_joins_category_series_and_formatted_value() {
        assert_eq!(datum_label("Jan", "visits", 12.5), "Jan · visits: 12.5");
    }

    #[test]
    fn datum_label_is_deterministic() {
        assert_eq!(datum_label("a", "b", 1.0), datum_label("a", "b", 1.0));
    }

    #[test]
    fn datum_label_lines_joins_heading_entries_and_footer_with_newlines() {
        let out = datum_label_lines(
            Some("Jan"),
            &[("Visits", "120"), ("Signups", "20")],
            Some("Total: 140"),
        );
        assert_eq!(out, "Jan\nVisits: 120\nSignups: 20\nTotal: 140");
    }

    #[test]
    fn datum_label_lines_omits_heading_line_when_none() {
        let out = datum_label_lines(None, &[("Visits", "120")], None);
        assert_eq!(out, "Visits: 120");
    }

    #[test]
    fn datum_label_lines_returns_empty_string_for_no_heading_entries_or_footer() {
        assert_eq!(datum_label_lines(None, &[], None), "");
    }

    #[test]
    fn datum_label_lines_is_deterministic() {
        let a = datum_label_lines(Some("h"), &[("s", "v")], Some("f"));
        let b = datum_label_lines(Some("h"), &[("s", "v")], Some("f"));
        assert_eq!(a, b);
    }

    #[test]
    fn xss_regression_datum_label_lines_all_inputs_are_escaped() {
        let payload = "</title><script>alert(1)</script>";
        let label = datum_label_lines(Some(payload), &[(payload, payload)], Some(payload));
        let html = render(&datum(0.0, 0.0, 1.0, &label, vec![]));
        assert!(!html.contains("<script>"));
        assert_eq!(html.matches("&lt;script&gt;").count(), 8);
    }

    #[test]
    fn datum_renders_circle_with_title_and_aria_label() {
        let label = datum_label("Jan", "visits", 10.0);
        let html = render(&datum(1.0, 2.0, 4.0, &label, vec![("fill", "red")]));
        assert!(
            html.starts_with(r#"<circle data-scope="chart" data-part="datum" cx="1" cy="2" r="4""#)
        );
        assert!(html.contains(r#"aria-label="Jan · visits: 10""#));
        assert!(html.contains("<title>Jan · visits: 10</title>"));
        assert!(html.contains(r#"fill="red""#));
    }

    #[test]
    fn datum_drops_caller_supplied_reserved_attrs() {
        let html = render(&datum(
            0.0,
            0.0,
            1.0,
            "safe",
            vec![
                ("data-scope", "attacker"),
                ("data-part", "attacker"),
                ("cx", "999"),
                ("cy", "999"),
                ("r", "999"),
                ("aria-label", "attacker"),
                ("fill", "blue"),
            ],
        ));
        assert!(html.contains(r#"data-scope="chart""#));
        assert!(html.contains(r#"data-part="datum""#));
        assert!(html.contains(r#"cx="0""#));
        assert!(html.contains(r#"cy="0""#));
        assert!(html.contains(r#"r="1""#));
        assert!(html.contains(r#"aria-label="safe""#));
        assert_eq!(html.matches("aria-label=").count(), 1);
        assert_eq!(html.matches(" cx=").count(), 1);
        assert!(html.contains(r#"fill="blue""#));
    }

    #[test]
    fn xss_regression_label_is_escaped_in_title_and_aria_label() {
        let payload = "</title><script>alert(1)</script>";
        let html = render(&datum(0.0, 0.0, 1.0, payload, vec![]));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn css_output_declares_hover_state_and_is_closed_charset() {
        let out = css();
        assert!(out.contains(":hover"));
        assert!(out.contains("stroke: var(--fandhe-color-fg)"));
        assert!(out.contains("stroke: var(--fandhe-color-bg)"));
        assert!(out.contains("transition-duration: var(--fandhe-motion-duration-fast)"));
        assert!(!out.contains('<'));
    }

    #[test]
    fn css_output_declares_hit_area_and_tooltip_slots() {
        let out = css();
        assert!(out.contains(r#"[data-scope="chart"][data-part="hit-area"]"#));
        assert!(out.contains(r#"[data-scope="chart"][data-part="frame"]"#));
        assert!(out.contains(r#"[data-scope="chart"][data-part="tooltip-layer"]"#));
        assert!(out.contains(r#"[data-scope="chart"][data-part="tooltip"]"#));
        assert!(out.contains(r#"[data-scope="chart"][data-part="tooltip-label"]"#));
        assert!(out.contains(r#"[data-scope="chart"][data-part="tooltip-item"]"#));
        assert!(out.contains(r#"[data-scope="chart"][data-part="tooltip-indicator"]"#));
        assert!(out.contains(r#"[data-scope="chart"][data-part="tooltip-value"]"#));
        assert!(out.contains(":focus-visible"));
        assert!(!out.contains('<'));
    }

    fn sample_data() -> super::super::data::ChartData {
        super::super::data::ChartData::new(
            vec!["Jan".to_string(), "Feb".to_string()],
            vec![
                super::super::data::Series::new("visits", vec![10.0, 20.0]),
                super::super::data::Series::new("signups", vec![1.0, 2.0]),
            ],
        )
        .unwrap()
    }

    #[test]
    fn entries_from_chart_data_builds_one_entry_per_category_with_all_series_rows() {
        let data = sample_data();
        let entries = entries_from_chart_data(&data);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].index, 0);
        assert_eq!(entries[0].label, "Jan");
        assert_eq!(entries[0].rows.len(), 2);
        assert_eq!(entries[0].rows[0].name, "visits");
        assert_eq!(entries[0].rows[0].value, 10.0);
        assert_eq!(
            entries[0].rows[0].color.var(),
            "var(--fandhe-color-chart-1)"
        );
    }

    #[test]
    fn hit_area_label_matches_datum_label_for_single_series() {
        let data = super::super::data::ChartData::new(
            vec!["Jan".to_string()],
            vec![super::super::data::Series::new("visits", vec![10.0])],
        )
        .unwrap();
        let entries = entries_from_chart_data(&data);
        assert_eq!(
            hit_area_label(&entries[0]),
            datum_label("Jan", "visits", 10.0)
        );
    }

    #[test]
    fn hit_area_label_joins_all_series_for_multi_series() {
        let data = sample_data();
        let entries = entries_from_chart_data(&data);
        assert_eq!(hit_area_label(&entries[0]), "Jan · visits: 10 · signups: 1");
    }

    #[test]
    fn hit_area_label_preserves_small_data_values() {
        // イシュー #2129 codex-review P1 指摘: `row.value` はデータ値その
        // ものであり、`fmt_coord`（小数点以下 2 桁固定）を使うと
        // `0.001` のような小さい値が `aria-label` で `0` に丸め落ちる。
        // `fmt_value` 経由への修正を固定する。
        let data = super::super::data::ChartData::new(
            vec!["Jan".to_string()],
            vec![super::super::data::Series::new("visits", vec![0.001])],
        )
        .unwrap();
        let entries = entries_from_chart_data(&data);
        assert_eq!(hit_area_label(&entries[0]), "Jan · visits: 0.001");
    }

    #[test]
    fn hit_area_rect_renders_fill_none_pointer_events_none_and_tabindex() {
        let html = render(&hit_area_rect(0.0, 0.0, 10.0, 20.0, 0, None, "label"));
        assert!(html.starts_with(
            r#"<rect x="0" y="0" width="10" height="20" data-scope="chart" data-part="hit-area" data-index="0" fill="none" pointer-events="none" tabindex="-1" aria-label="label">"#
        ));
        assert!(!html.contains("<title>"));
    }

    #[test]
    fn hit_area_rect_includes_data_series_when_provided() {
        let html = render(&hit_area_rect(0.0, 0.0, 1.0, 1.0, 2, Some("visits"), "l"));
        assert!(html.contains(r#"data-series="visits""#));
    }

    #[test]
    fn hit_area_circle_and_path_render_expected_geometry() {
        let circle_html = render(&hit_area_circle(1.0, 2.0, 3.0, 0, None, "l"));
        assert!(circle_html.starts_with(r#"<circle cx="1" cy="2" r="3""#));

        let path_html = render(&hit_area_path("M0,0 L1,1 Z", 0, None, "l", false));
        assert!(path_html.contains(r#"d="M0,0 L1,1 Z""#));
        assert!(!path_html.contains("fill-rule"));

        let evenodd_html = render(&hit_area_path("M0,0 L1,1 Z", 0, None, "l", true));
        assert!(evenodd_html.contains(r#"fill-rule="evenodd""#));
    }

    #[test]
    fn layer_renders_one_hidden_tooltip_per_category_by_default() {
        let data = sample_data();
        let html = render(&layer(&data, None));
        assert!(html.starts_with(
            r#"<div data-scope="chart" data-part="tooltip-layer" aria-hidden="true">"#
        ));
        assert_eq!(html.matches(r#"data-part="tooltip""#).count(), 2);
        assert_eq!(html.matches(r#"hidden="""#).count(), 2);
    }

    #[test]
    fn layer_omits_hidden_only_for_the_active_index() {
        let data = sample_data();
        let html = render(&layer(&data, Some(0)));
        assert_eq!(html.matches(r#"hidden="""#).count(), 1);
        let active_pos = html.find(r#"data-index="0""#).unwrap();
        let hidden_pos = html.find(r#"hidden="""#);
        // 唯一の hidden はカテゴリ 1（index 1）側であり、index 0 側には出ない。
        assert!(hidden_pos.is_none_or(|p| p > active_pos));
    }

    #[test]
    fn layer_renders_series_rows_with_indicator_color_and_value() {
        let data = sample_data();
        let html = render(&layer(&data, None));
        assert!(html.contains(r#"data-part="tooltip-item" data-series="visits""#));
        assert!(html.contains("--fandhe-chart-tooltip-color: var(--fandhe-color-chart-1)"));
        assert!(html.contains(r#"data-part="tooltip-name">visits<"#));
        assert!(html.contains(">10<"));
    }

    #[test]
    fn layer_preserves_small_data_values_in_tooltip_item_text() {
        // イシュー #2129 codex-review P1 指摘: ツールチップ本文の値表示
        // （`tooltip-item` 内テキスト）も `fmt_coord` ではなく `fmt_value`
        // を経由するため、`0.001` のような小さい値が `0` へ丸め落ちない
        // ことを固定する。
        let data = super::super::data::ChartData::new(
            vec!["Jan".to_string()],
            vec![super::super::data::Series::new("visits", vec![0.001])],
        )
        .unwrap();
        let html = render(&layer(&data, None));
        assert!(html.contains(">0.001<"));
        assert!(!html.contains(">0<"));
    }

    #[test]
    fn frame_wraps_children_in_a_relative_positioned_div() {
        let html = render(&frame(vec![text("child")]));
        assert_eq!(
            html,
            r#"<div data-scope="chart" data-part="frame">child</div>"#
        );
    }

    #[test]
    fn xss_regression_hit_area_and_tooltip_layer_escape_all_untrusted_inputs() {
        let payload = "</title><script>alert(1)</script>";
        let html = render(&hit_area_rect(
            0.0,
            0.0,
            1.0,
            1.0,
            0,
            Some(payload),
            payload,
        ));
        assert!(!html.contains("<script>"));

        let data = super::super::data::ChartData::new(
            vec![payload.to_string()],
            vec![super::super::data::Series::new(payload, vec![1.0])],
        )
        .unwrap();
        let html = render(&layer(&data, None));
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn layer_from_entries_and_layer_agree_for_equivalent_data() {
        let data = sample_data();
        let a = render(&layer(&data, None));
        let b = render(&layer_from_entries(&entries_from_chart_data(&data), None));
        assert_eq!(a, b);
    }
}
