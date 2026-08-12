//! 状態変更負荷ベンチの常設コマンド（イシュー #1328）。
//!
//! `fandhe-frontend-interactive`（状態管理コア）には、状態変更（`update()` 1 回
//! あたり）のコストを定量把握する常設の計測経路がなかった。既存の
//! `bench_binding_update`（イシュー #592）は全再描画 vs dirty 読み出しの
//! マイクロ比較、`bench_ssr`（イシュー #1317）は SSR 描画スループットのみで、
//! 「通知（dirty tracking）・再計算（`view()`/`render()`）・束縛点適用相当の
//! 解決」という状態変更経路の内訳を 1,000 束縛点規模で測るハーネスが存在
//! しなかった。本モジュールはその内訳を計測し、`xtask/src/main.rs::run_bench_state_update`
//! から呼ばれる。
//!
//! # ワークロード（固定・CLI から差し替え不可・乱数不使用）
//!
//! - **grid-1k**: 本モジュール内定義の合成コンポーネント [`BenchGrid`]。
//!   `values: Vec<i64>`（[`BINDINGS`] 件、決定的初期値 `0..BINDINGS`）を状態に持ち、
//!   `view()` は各値をルート `div` 配下の `bind_text("span", .., field, value)`
//!   （[`fandhe_frontend_core::bind_text`]）1,000 個として出力する。field 名は
//!   `f0`..`f999` を起動時に 1 回だけ [`field_names`] が `Box::leak` で
//!   `&'static str` 化する（[`DirtyTracked`] の `&'static str` 契約を満たす
//!   ための、有界（1,000 個の短い文字列）・1 回限りのリークであり `unsafe` では
//!   ない。プロセス終了まで解放されないが、xtask はサブコマンド 1 回の実行で
//!   終了する開発者ツールのため実害はない）。index 0 の値ラベルに
//!   [`XSS_MARKER`] を混入させ、[`verify_grid_escape`] が render 出力へ生の
//!   `<script>` が出ないことを fail-closed 検証する。
//! - **appstate-1k**: [`fandhe_frontend_interactive::AppState`] の実体に
//!   [`BINDINGS`] 件の決定的ラベル項目を投入した状態（[`build_appstate_1k`]）。
//!   `update`/`noop_update` フェーズは [`fandhe_frontend_interactive::Action::Increment`]/
//!   [`fandhe_frontend_interactive::Action::Reset`]（counter==0 時 no-op）を使い、
//!   `render` フェーズは `view()` の keyed_list 経路（イシュー #1328 で撤去した
//!   `render_with_root_attrs` の無条件 `plain_items` clone の改善が直接現れる）
//!   を計測する。
//!
//! # 計測フェーズ（各シナリオ、ウォームアップ + 計測反復）
//!
//! [`std::time::Instant`] + [`std::hint::black_box`] を入出力双方に通し、
//! ループ不変コード移動（LICM）・定数畳み込みを防ぐ（`bench_ssr`/
//! `bench_binding_update` と同じ対策、イシュー #592 PR #623 レビュー知見）。
//!
//! 1. `update`: 単一フィールドの状態変更（通知コスト。dirty 記録を含む）
//! 2. `binding_apply`: `dirty_fields()` 読み出し + 事前構築済み束縛点対応表
//!    （[`BindingTable`]）への解決・適用値文字列生成（束縛点適用の DOM 非依存
//!    前段。実 DOM 適用コストは `docs/ci/perf-browser-harness.md` の責務であり
//!    本ベンチのスコープ外）
//! 3. `render`: `view()` + [`fandhe_frontend_core::render`]（全再計算の参照値）
//! 4. `noop_update`: 変更なし値の状態変更（同値 set / counter==0 での reset）。
//!    dirty が空・binding_apply の適用件数 0 であることを fail-closed 自己検証する
//!    （[`Report::noop_ok`]）
//!
//! # 出力（stdout・JSON 1 行）
//!
//! `framework` / `version`（`fandhe-frontend-interactive` の実バージョン、
//! [`resolve_interactive_version`] が `cargo metadata --no-deps` のみで解決。
//! ネットワークアクセスなし）/ `mode`（常に `"state-update"`）/
//! `workload_schema_version`（[`WORKLOAD_SCHEMA_VERSION`]）/ `bindings`
//! （[`BINDINGS`]）/ `grid1k`・`appstate1k`（各シナリオ、`update`/`binding_apply`/
//! `render`/`noop_update` の 4 フェーズごとに `{iters, mean_us, p50_us, p95_us,
//! min_us}`）/ `escape_ok` / `noop_ok` / `notes`（`profile=debug`/`profile=release`）。
//!
//! # 検証（fail-closed）・回帰比較（report-only）
//!
//! `escape_ok`（grid-1k の render 出力に生の `<script>alert(1)</script>` が
//! 含まれない）・`noop_ok`（両シナリオの `noop_update` フェーズで dirty が空、
//! かつ `binding_apply` の適用件数が 0）のいずれかが `false` の場合、呼び出し元
//! （`run_bench_state_update`）は JSON 行を出力したうえで終了コード 1 を返す。
//! この検証を緩和する CLI 引数・環境変数は設けない。
//!
//! 任意の `--baseline <FILE>` を指定すると、過去の本コマンド出力（JSON 1 行）
//! との差分を `bench-state-update-compare: metric=<name> baseline=<b>
//! current=<c> delta_pct=<±x.xx>` の行群として追加出力する。**report-only**
//! （数値比較を根拠に判定・終了コードを変えない）とし、CI 閾値ゲート化は
//! 行わない（`bench_ssr`/`bench_binding_update` と同一の設計判断: 計測値は
//! 実行環境・負荷に依存し、しきい値判定は偽陽性/偽陰性の温床になる）。

use std::collections::HashMap;
use std::fmt;
use std::hint::black_box;
use std::sync::OnceLock;
use std::time::Instant;

use fandhe_frontend_core::{bind_text, el, render, Node};
use fandhe_frontend_interactive::{Action, AppState, Component, DirtyTracked};

use crate::check_dep_versions;
use crate::json::{parse, Json, JsonError};

/// 1 シナリオあたりの束縛点数（固定・CLI から差し替え不可）。
pub const BINDINGS: usize = 1_000;

const WARMUP: usize = 20;
const ITERS: usize = 200;

/// XSS 回帰検知用の意図的なペイロード。[`bind_text`]（既定エスケープ経由）
/// でのみ出力する。生の `<script>` タグとして出現したら [`verify_grid_escape`]
/// が `escape_ok=false` を返す。
const XSS_MARKER: &str = "<script>alert(1)</script>";

/// ワークロード定義（[`BenchGrid`]/[`build_appstate_1k`] の構造・
/// [`BINDINGS`]・フェーズ構成）と出力スキーマ（[`Report::to_json_line`]
/// のキー集合）を束ねた fingerprint。`bench_ssr::WORKLOAD_SCHEMA_VERSION`
/// と同じ役割・同じ運用規約（変更時は必ずインクリメントする）。
const WORKLOAD_SCHEMA_VERSION: u32 = 1;

/// 本モジュール専用のエラー型。`bench_ssr::BenchSsrError` と同型。
#[derive(Debug)]
pub enum BenchStateUpdateError {
    /// 実行環境起因の失敗（`cargo metadata` 失敗等）。
    Environment(String),
    /// `--baseline` ファイルの読み取り・パース・スキーマ検証失敗。
    InvalidBaseline(String),
}

impl fmt::Display for BenchStateUpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BenchStateUpdateError::Environment(msg) => write!(f, "environment error: {msg}"),
            BenchStateUpdateError::InvalidBaseline(msg) => write!(f, "invalid baseline: {msg}"),
        }
    }
}

impl std::error::Error for BenchStateUpdateError {}

/// 1 フェーズ分の統計サマリ（単位: マイクロ秒）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats {
    pub iters: usize,
    pub mean_us: f64,
    pub p50_us: f64,
    pub p95_us: f64,
    pub min_us: f64,
}

/// 1 シナリオ（grid-1k/appstate-1k）分の 4 フェーズ計測結果。
#[derive(Debug, Clone, Copy)]
pub struct ScenarioReport {
    pub update: Stats,
    pub binding_apply: Stats,
    pub render: Stats,
    pub noop_update: Stats,
}

/// `bench-state-update` の実行結果全体。
#[derive(Debug, Clone)]
pub struct Report {
    pub version: String,
    pub grid1k: ScenarioReport,
    pub appstate1k: ScenarioReport,
    pub escape_ok: bool,
    pub noop_ok: bool,
    pub notes: String,
}

impl Report {
    /// 呼び出し元（`run_bench_state_update`）が終了コードを決める判定を、
    /// テスト可能な純粋関数として切り出したもの（fail-closed: どちらか
    /// 一方でも偽なら偽）。
    #[must_use]
    pub fn self_check_ok(&self) -> bool {
        self.escape_ok && self.noop_ok
    }

    /// JSON 1 行へ整形する。文字列フィールドは [`json_escape`] で最小限の
    /// エスケープを通す（xtask 外部依存ゼロ方針のため `serde_json` は使わず内製）。
    #[must_use]
    pub fn to_json_line(&self) -> String {
        format!(
            "{{\"framework\":\"fandhe-frontend\",\"version\":\"{version}\",\"mode\":\"state-update\",\
             \"workload_schema_version\":{workload_schema_version},\"bindings\":{bindings},\
             \"grid1k\":{grid1k},\"appstate1k\":{appstate1k},\
             \"escape_ok\":{escape_ok},\"noop_ok\":{noop_ok},\"notes\":\"{notes}\"}}",
            version = json_escape(&self.version),
            workload_schema_version = WORKLOAD_SCHEMA_VERSION,
            bindings = BINDINGS,
            grid1k = scenario_to_json(&self.grid1k),
            appstate1k = scenario_to_json(&self.appstate1k),
            escape_ok = self.escape_ok,
            noop_ok = self.noop_ok,
            notes = json_escape(&self.notes),
        )
    }
}

fn scenario_to_json(s: &ScenarioReport) -> String {
    format!(
        "{{\"update\":{update},\"binding_apply\":{binding_apply},\"render\":{render},\"noop_update\":{noop_update}}}",
        update = stats_to_json(&s.update),
        binding_apply = stats_to_json(&s.binding_apply),
        render = stats_to_json(&s.render),
        noop_update = stats_to_json(&s.noop_update),
    )
}

fn stats_to_json(s: &Stats) -> String {
    format!(
        "{{\"iters\":{iters},\"mean_us\":{mean_us:.4},\"p50_us\":{p50_us:.4},\"p95_us\":{p95_us:.4},\"min_us\":{min_us:.4}}}",
        iters = s.iters,
        mean_us = s.mean_us,
        p50_us = s.p50_us,
        p95_us = s.p95_us,
        min_us = s.min_us,
    )
}

/// 最小限の JSON 文字列エスケープ（`bench_ssr::json_escape` と同一実装。
/// モジュール間で共有すると呼び出し関係が増えるため、既存の慣例
/// （`check_deps`/`check_loc` 等も自 module 内に閉じる）に従いここでも
/// 独立して定義する）。
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// grid-1k の field 名（`f0`..`f999`）を起動時に 1 回だけ確定させる。
///
/// [`DirtyTracked::dirty_fields`] は `&'static str` を要求するため、動的に
/// 決めた個数分の field 名を静的寿命へ昇格させる必要がある。`Box::leak` は
/// `unsafe` ではなく、[`BINDINGS`] 個（既定 1,000）の短い文字列という有界な
/// 量を、プロセス内で 1 回だけ（[`OnceLock`] により多重初期化を防止）
/// リークする。xtask は 1 サブコマンド実行ごとに終了する開発者ツールであり、
/// 常駐プロセスでの無限増殖リークとは性質が異なる（実害なし）。
fn field_names() -> &'static [&'static str] {
    static NAMES: OnceLock<Vec<&'static str>> = OnceLock::new();
    NAMES.get_or_init(|| {
        (0..BINDINGS)
            .map(|i| -> &'static str { Box::leak(format!("f{i}").into_boxed_str()) })
            .collect()
    })
}

/// grid-1k の合成コンポーネント。`values[i]` の束縛点 field 名は
/// `field_names()[i]`。[`DirtyTracked`] を手動実装し、`set` は変更なし値
/// （同値 set）のとき dirty を記録しない（`fandhe-frontend-interactive::AppState`
/// の既存 no-op ガードと同じ設計判断、`noop_update` フェーズの前提）。
struct BenchGrid {
    values: Vec<i64>,
    dirty: Vec<&'static str>,
}

impl BenchGrid {
    /// 決定的初期値 `0..BINDINGS`（乱数不使用）。
    fn new() -> Self {
        Self {
            values: (0..BINDINGS as i64).collect(),
            dirty: Vec::new(),
        }
    }

    /// `index` 番目の値を更新する（通知コスト計測対象）。同値 set は no-op。
    fn set(&mut self, index: usize, value: i64) {
        if self.values[index] == value {
            return;
        }
        self.values[index] = value;
        let field = field_names()[index];
        if !self.dirty.contains(&field) {
            self.dirty.push(field);
        }
    }

    fn dirty_fields(&self) -> &[&'static str] {
        &self.dirty
    }

    /// `render` フェーズの計測対象。index 0 のラベルにのみ [`XSS_MARKER`] を
    /// 混入させ、[`verify_grid_escape`] が既定エスケープ回帰を検知できる
    /// ようにする。
    fn view(&self) -> Node {
        let children: Vec<Node> = self
            .values
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let field = field_names()[i];
                let value = if i == 0 {
                    format!("{v} {XSS_MARKER}")
                } else {
                    v.to_string()
                };
                bind_text("span", vec![], field, value)
            })
            .collect();
        el("div", vec![("data-testid", "bench-grid")], children)
    }
}

/// 束縛点適用の DOM 非依存前段（`fandhe-frontend-wasm-full`/`-wasm-client` の
/// `BindingTable::apply_dirty` が DOM 呼び出しを行う直前に相当する解決処理、
/// `docs/design/dom-binding-update-design.md` #345 参照）。field 名 →
/// index の対応表を 1 回構築し、`dirty_fields()` の読み出し結果を解決して
/// 適用値文字列を生成する（実 DOM への `set_text_content` 等は行わない。
/// 本ベンチのスコープ外、モジュール doc 参照）。
struct BindingTable {
    field_to_index: HashMap<&'static str, usize>,
}

impl BindingTable {
    fn new() -> Self {
        let field_to_index = field_names()
            .iter()
            .enumerate()
            .map(|(i, f)| (*f, i))
            .collect();
        Self { field_to_index }
    }

    /// `dirty` に含まれる各 field を解決し、対応する値の文字列表現を生成する。
    /// 戻り値は適用件数（[`Report`] の `noop_ok` 検証・`binding_apply`
    /// フェーズの意味論確認に使う）。
    fn resolve_and_format(&self, values: &[i64], dirty: &[&'static str]) -> usize {
        let mut applied = 0usize;
        for field in dirty {
            if let Some(&idx) = self.field_to_index.get(field) {
                let formatted = values[idx].to_string();
                black_box(formatted);
                applied += 1;
            }
        }
        applied
    }
}

/// ソート済み `us` から百分位を求める（`bench_ssr::percentile` と同一の
/// `floor(p * (n-1))` 方式）。空スライスは 0.0 を返す。
fn percentile(sorted_us: &[f64], p: f64) -> f64 {
    let Some(&last) = sorted_us.last() else {
        return 0.0;
    };
    let idx = (p * (sorted_us.len() - 1) as f64).floor() as usize;
    sorted_us.get(idx).copied().unwrap_or(last)
}

fn stats_from_durations(mut durations_us: Vec<f64>) -> Stats {
    let iters = durations_us.len();
    durations_us.sort_by(f64::total_cmp);
    let sum: f64 = durations_us.iter().sum();
    let mean_us = if iters == 0 { 0.0 } else { sum / iters as f64 };
    let min_us = durations_us.first().copied().unwrap_or(0.0);
    Stats {
        iters,
        mean_us,
        p50_us: percentile(&durations_us, 0.50),
        p95_us: percentile(&durations_us, 0.95),
        min_us,
    }
}

/// `AppState` に [`BINDINGS`] 件の決定的ラベル項目を投入した状態を構築する
/// （`appstate-1k` シナリオの初期状態）。項目ラベルの 1 件目にのみ
/// [`XSS_MARKER`] を混入させる（`render` フェーズの既定エスケープ経路は
/// `AppState::view` 側で既に `crates/interactive/tests/xss_escape.rs` が
/// 回帰固定しているため、本モジュールの `escape_ok` 検証対象は grid-1k の
/// みとし、二重管理しない）。
fn build_appstate_1k() -> AppState {
    let mut state = AppState::new();
    state.items = (0..BINDINGS)
        .map(|i| {
            if i == 0 {
                format!("item-{i} {XSS_MARKER}")
            } else {
                format!("item-{i}")
            }
        })
        .collect();
    state.item_ids = (0..BINDINGS as u64).collect();
    state.next_item_id = BINDINGS as u64;
    state.dirty.clear();
    state
}

/// grid-1k シナリオを計測する。各イテレーションはタイマー開始**前**に
/// フェーズ固有の前提状態を構築し（構築コストを計測対象から除外する、
/// `bench_ssr::measure_rows` と同じ判断）、対象操作のみを計測する。
fn measure_grid_1k() -> (ScenarioReport, String) {
    let table = BindingTable::new();

    // --- update: 単一フィールドの状態変更（通知コスト） ---
    for _ in 0..WARMUP {
        let mut grid = BenchGrid::new();
        grid.set(black_box(0), black_box(999_999));
        black_box(&grid);
    }
    let mut update_us = Vec::with_capacity(ITERS);
    for _ in 0..ITERS {
        let mut grid = BenchGrid::new();
        let start = Instant::now();
        grid.set(black_box(0), black_box(999_999));
        let elapsed = start.elapsed().as_secs_f64() * 1_000_000.0;
        update_us.push(elapsed);
        black_box(&grid);
    }

    // --- binding_apply: dirty 読み出し + 束縛点解決・適用値生成 ---
    for _ in 0..WARMUP {
        let mut grid = BenchGrid::new();
        grid.set(0, 999_999);
        let applied =
            table.resolve_and_format(black_box(&grid.values), black_box(grid.dirty_fields()));
        black_box(applied);
    }
    let mut binding_apply_us = Vec::with_capacity(ITERS);
    for _ in 0..ITERS {
        let mut grid = BenchGrid::new();
        grid.set(0, 999_999);
        let start = Instant::now();
        let applied =
            table.resolve_and_format(black_box(&grid.values), black_box(grid.dirty_fields()));
        let elapsed = start.elapsed().as_secs_f64() * 1_000_000.0;
        binding_apply_us.push(elapsed);
        black_box(applied);
    }

    // --- render: view() + render()（全再計算の参照値） ---
    for _ in 0..WARMUP {
        let grid = BenchGrid::new();
        let node = grid.view();
        black_box(render(black_box(&node)));
    }
    let mut render_us = Vec::with_capacity(ITERS);
    let mut last_html = String::new();
    for _ in 0..ITERS {
        let grid = BenchGrid::new();
        let start = Instant::now();
        let node = grid.view();
        let html = render(black_box(&node));
        let elapsed = start.elapsed().as_secs_f64() * 1_000_000.0;
        render_us.push(elapsed);
        last_html = black_box(html);
    }

    // --- noop_update: 変更なし値（同値 set）の状態変更 ---
    for _ in 0..WARMUP {
        let mut grid = BenchGrid::new();
        let existing = grid.values[0];
        grid.set(black_box(0), black_box(existing));
        black_box(&grid);
    }
    let mut noop_update_us = Vec::with_capacity(ITERS);
    for _ in 0..ITERS {
        let mut grid = BenchGrid::new();
        let existing = grid.values[0];
        let start = Instant::now();
        grid.set(black_box(0), black_box(existing));
        let elapsed = start.elapsed().as_secs_f64() * 1_000_000.0;
        noop_update_us.push(elapsed);
        black_box(&grid);
    }

    let report = ScenarioReport {
        update: stats_from_durations(update_us),
        binding_apply: stats_from_durations(binding_apply_us),
        render: stats_from_durations(render_us),
        noop_update: stats_from_durations(noop_update_us),
    };
    (report, last_html)
}

/// appstate-1k シナリオを計測する。`update`/`noop_update` は
/// [`fandhe_frontend_interactive::Component::update`] を直接呼ぶ（`dispatch`
/// を経由しないのは文字列パースコストを計測対象から除外するため。
/// `dispatch` の文字列境界コストは `bench_binding_update` の既存スコープ）。
fn measure_appstate_1k() -> ScenarioReport {
    // --- update: counter の単一フィールド更新（通知コスト） ---
    for _ in 0..WARMUP {
        let mut state = build_appstate_1k();
        state.update(black_box(Action::Increment));
        black_box(&state);
    }
    let mut update_us = Vec::with_capacity(ITERS);
    for _ in 0..ITERS {
        let mut state = build_appstate_1k();
        let start = Instant::now();
        state.update(black_box(Action::Increment));
        let elapsed = start.elapsed().as_secs_f64() * 1_000_000.0;
        update_us.push(elapsed);
        black_box(&state);
    }

    // --- binding_apply: dirty 読み出し + 適用値生成（counter フィールドのみ） ---
    for _ in 0..WARMUP {
        let mut state = build_appstate_1k();
        state.update(Action::Increment);
        let applied = resolve_appstate_dirty(black_box(&state));
        black_box(applied);
    }
    let mut binding_apply_us = Vec::with_capacity(ITERS);
    for _ in 0..ITERS {
        let mut state = build_appstate_1k();
        state.update(Action::Increment);
        let start = Instant::now();
        let applied = resolve_appstate_dirty(black_box(&state));
        let elapsed = start.elapsed().as_secs_f64() * 1_000_000.0;
        binding_apply_us.push(elapsed);
        black_box(applied);
    }

    // --- render: view() + render()（keyed_list フォールバック撤去の改善が
    // 直接現れる 1,000 項目描画） ---
    for _ in 0..WARMUP {
        let state = build_appstate_1k();
        let node = state.view();
        black_box(render(black_box(&node)));
    }
    let mut render_us = Vec::with_capacity(ITERS);
    for _ in 0..ITERS {
        let state = build_appstate_1k();
        let start = Instant::now();
        let node = state.view();
        let html = render(black_box(&node));
        let elapsed = start.elapsed().as_secs_f64() * 1_000_000.0;
        render_us.push(elapsed);
        black_box(html);
    }

    // --- noop_update: counter==0 での Reset（既存 no-op ガード） ---
    for _ in 0..WARMUP {
        let mut state = build_appstate_1k();
        state.update(black_box(Action::Reset));
        black_box(&state);
    }
    let mut noop_update_us = Vec::with_capacity(ITERS);
    for _ in 0..ITERS {
        let mut state = build_appstate_1k();
        let start = Instant::now();
        state.update(black_box(Action::Reset));
        let elapsed = start.elapsed().as_secs_f64() * 1_000_000.0;
        noop_update_us.push(elapsed);
        black_box(&state);
    }

    ScenarioReport {
        update: stats_from_durations(update_us),
        binding_apply: stats_from_durations(binding_apply_us),
        render: stats_from_durations(render_us),
        noop_update: stats_from_durations(noop_update_us),
    }
}

/// `AppState::dirty_fields()` を解決し、`counter` フィールドについてのみ
/// 適用値文字列を生成する（appstate-1k は counter 単一フィールド更新の
/// みを対象とするため、grid-1k の [`BindingTable`] のような field→index の
/// 汎用対応表は不要。戻り値は適用件数）。
fn resolve_appstate_dirty(state: &AppState) -> usize {
    let mut applied = 0usize;
    for field in state.dirty_fields() {
        if *field == AppState::FIELD_COUNTER {
            let formatted = state.counter.to_string();
            black_box(formatted);
            applied += 1;
        }
    }
    applied
}

/// grid-1k の render 出力を検証する。`escape_ok`: index 0 のラベル
/// （`"999999 <script>alert(1)</script>"`）がエスケープ済みリテラルと
/// 完全一致し、かつ生の `XSS_MARKER` が出力に含まれないこと
/// （検証対象と同じ関数で期待値を作らない、`bench_ssr::verify` と同じ
/// codex-review P0 知見の踏襲）。
fn verify_grid_escape(html: &str, index0_value: i64) -> bool {
    let expected = format!("{index0_value} &lt;script&gt;alert(1)&lt;/script&gt;");
    let raw = format!("{index0_value} {XSS_MARKER}");
    html.contains(&expected) && !html.contains(&raw)
}

/// `cargo metadata --no-deps` のみで `fandhe-frontend-interactive` の実
/// バージョンを解決する（ネットワークアクセスなし）。`bench_ssr::resolve_core_version`
/// と同型の実装で、`check_dep_versions` の既存 metadata 呼び出しを再利用する。
pub fn resolve_interactive_version() -> Result<String, BenchStateUpdateError> {
    let (_workspace_root, members) = check_dep_versions::workspace_packages_from_cargo_metadata()
        .map_err(|e| {
        BenchStateUpdateError::Environment(format!(
            "failed to resolve fandhe-frontend-interactive version via cargo metadata: {e}"
        ))
    })?;
    members
        .into_iter()
        .find(|m| m.name == "fandhe-frontend-interactive")
        .map(|m| m.version)
        .ok_or_else(|| {
            BenchStateUpdateError::Environment(
                "fandhe-frontend-interactive not found in `cargo metadata --no-deps` workspace members"
                    .to_string(),
            )
        })
}

/// 全計測を実行し [`Report`] を組み立てる。
#[must_use]
pub fn run(version: String) -> Report {
    let (grid1k, grid_html) = measure_grid_1k();
    let appstate1k = measure_appstate_1k();

    // `measure_grid_1k` の render フェーズは毎回 `BenchGrid::new()`（index 0
    // の初期値 0）を描画するため、検証側もその値と一致させる。
    let escape_ok = verify_grid_escape(&grid_html, 0);

    // noop_update フェーズが実際に「変更なし」であることを、計測ループとは
    // 独立した 1 回の検証実行で確認する（計測ループ内で毎回検証すると
    // 検証コスト自体が計測対象に混入するため）。
    let mut grid_for_noop = BenchGrid::new();
    let existing = grid_for_noop.values[0];
    grid_for_noop.set(0, existing);
    let grid_noop_dirty_empty = grid_for_noop.dirty_fields().is_empty();
    let table = BindingTable::new();
    let grid_noop_applied =
        table.resolve_and_format(&grid_for_noop.values, grid_for_noop.dirty_fields());

    let mut appstate_for_noop = build_appstate_1k();
    appstate_for_noop.update(Action::Reset);
    let appstate_noop_dirty_empty = appstate_for_noop.dirty_fields().is_empty();
    let appstate_noop_applied = resolve_appstate_dirty(&appstate_for_noop);

    let noop_ok = grid_noop_dirty_empty
        && grid_noop_applied == 0
        && appstate_noop_dirty_empty
        && appstate_noop_applied == 0;

    Report {
        version,
        grid1k,
        appstate1k,
        escape_ok,
        noop_ok,
        notes: format!(
            "profile={}",
            if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            }
        ),
    }
}

/// `--baseline` で渡された過去の JSON 1 行出力と `current` を比較し、
/// `bench-state-update-compare:` 行群を返す（report-only、判定なし）。
/// `bench_ssr::compare` と同型（数値比較の前に [`validate_baseline_workload`]
/// でスキーマ一致を検証し、`version` のみバージョン間比較を許容する例外
/// とする）。
pub fn compare(
    current: &Report,
    baseline_json: &str,
) -> Result<Vec<String>, BenchStateUpdateError> {
    let root = parse(baseline_json).map_err(|e: JsonError| {
        BenchStateUpdateError::InvalidBaseline(format!("failed to parse baseline JSON: {e}"))
    })?;

    validate_baseline_workload(current, &root)?;

    let mut lines = Vec::new();
    for (scenario_name, scenario) in [
        ("grid1k", &current.grid1k),
        ("appstate1k", &current.appstate1k),
    ] {
        for (phase_name, stats) in [
            ("update", &scenario.update),
            ("binding_apply", &scenario.binding_apply),
            ("render", &scenario.render),
            ("noop_update", &scenario.noop_update),
        ] {
            for (metric_suffix, current_value) in [
                ("mean_us", stats.mean_us),
                ("p50_us", stats.p50_us),
                ("p95_us", stats.p95_us),
                ("min_us", stats.min_us),
            ] {
                let metric = format!("{scenario_name}.{phase_name}.{metric_suffix}");
                let baseline_value = lookup_metric(&root, &metric)?;
                let delta_pct = if baseline_value == 0.0 {
                    0.0
                } else {
                    (current_value - baseline_value) / baseline_value * 100.0
                };
                lines.push(format!(
                    "bench-state-update-compare: metric={metric} baseline={baseline_value:.4} current={current_value:.4} delta_pct={delta_pct:+.2}"
                ));
            }
        }
    }

    Ok(lines)
}

/// baseline JSON が `current` と「同一ワークロード」であることを検証する
/// （[`compare`] が数値比較を行う前の必須スキーマ検証。`bench_ssr::validate_baseline_workload`
/// と同型）。`version` のみ例外として検証対象から除外する。
fn validate_baseline_workload(current: &Report, root: &Json) -> Result<(), BenchStateUpdateError> {
    lookup_str_exact(root, "framework", "fandhe-frontend")?;
    lookup_str_exact(root, "mode", "state-update")?;
    lookup_str_exact(root, "notes", &current.notes)?;
    lookup_usize_exact(
        root,
        "workload_schema_version",
        WORKLOAD_SCHEMA_VERSION as usize,
    )?;
    lookup_usize_exact(root, "bindings", BINDINGS)?;
    for scenario in ["grid1k", "appstate1k"] {
        for phase in ["update", "binding_apply", "render", "noop_update"] {
            let path = format!("{scenario}.{phase}.iters");
            let expected = match (scenario, phase) {
                ("grid1k", _) | ("appstate1k", _) => ITERS,
                _ => unreachable!(),
            };
            lookup_usize_exact(root, &path, expected)?;
        }
    }
    Ok(())
}

fn lookup_str_exact(
    root: &Json,
    dotted_path: &str,
    expected: &str,
) -> Result<(), BenchStateUpdateError> {
    let value = lookup_json(root, dotted_path)?;
    match value.as_str() {
        Some(s) if s == expected => Ok(()),
        Some(s) => Err(BenchStateUpdateError::InvalidBaseline(format!(
            "baseline JSON value at `{dotted_path}` (`{s}`) does not match current (`{expected}`); refusing to compare incompatible workloads"
        ))),
        None => Err(BenchStateUpdateError::InvalidBaseline(format!(
            "baseline JSON value at `{dotted_path}` is not a string"
        ))),
    }
}

fn lookup_usize_exact(
    root: &Json,
    dotted_path: &str,
    expected: usize,
) -> Result<(), BenchStateUpdateError> {
    let value = lookup_json(root, dotted_path)?;
    match value {
        Json::Number(n) if *n == expected as f64 => Ok(()),
        Json::Number(n) => Err(BenchStateUpdateError::InvalidBaseline(format!(
            "baseline JSON value at `{dotted_path}` (`{n}`) does not match current (`{expected}`); refusing to compare incompatible workloads"
        ))),
        _ => Err(BenchStateUpdateError::InvalidBaseline(format!(
            "baseline JSON value at `{dotted_path}` is not a number"
        ))),
    }
}

fn lookup_json<'a>(root: &'a Json, dotted_path: &str) -> Result<&'a Json, BenchStateUpdateError> {
    let mut current = root;
    for key in dotted_path.split('.') {
        current = current.get(key).ok_or_else(|| {
            BenchStateUpdateError::InvalidBaseline(format!(
                "baseline JSON is missing key `{key}` (path `{dotted_path}`)"
            ))
        })?;
    }
    Ok(current)
}

fn lookup_metric(root: &Json, dotted_path: &str) -> Result<f64, BenchStateUpdateError> {
    match lookup_json(root, dotted_path)? {
        Json::Number(n) => Ok(*n),
        _ => Err(BenchStateUpdateError::InvalidBaseline(format!(
            "baseline JSON value at `{dotted_path}` is not a number"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_matches_known_vectors() {
        let sorted = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        assert_eq!(percentile(&sorted, 0.50), 30.0);
        assert_eq!(percentile(&sorted, 0.95), 40.0);
        assert_eq!(percentile(&sorted, 0.0), 10.0);
    }

    #[test]
    fn percentile_of_empty_slice_is_zero() {
        assert_eq!(percentile(&[], 0.50), 0.0);
    }

    #[test]
    fn field_names_are_stable_and_unique() {
        let names = field_names();
        assert_eq!(names.len(), BINDINGS);
        assert_eq!(names[0], "f0");
        assert_eq!(names[BINDINGS - 1], format!("f{}", BINDINGS - 1));
        let mut sorted = names.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), BINDINGS, "field 名は一意のはず");
    }

    #[test]
    fn field_names_are_idempotent_across_calls() {
        // OnceLock により 2 回目以降の呼び出しは同一の静的スライスを返す
        // （多重初期化・多重 leak が起きないことの回帰）。
        let a = field_names();
        let b = field_names();
        assert_eq!(a.as_ptr(), b.as_ptr());
    }

    #[test]
    fn bench_grid_set_records_dirty_field() {
        let mut grid = BenchGrid::new();
        assert!(grid.dirty_fields().is_empty());
        grid.set(3, 42);
        assert_eq!(grid.dirty_fields(), &["f3"]);
        assert_eq!(grid.values[3], 42);
    }

    #[test]
    fn bench_grid_set_same_value_is_noop() {
        let mut grid = BenchGrid::new();
        let existing = grid.values[7];
        grid.set(7, existing);
        assert!(grid.dirty_fields().is_empty());
    }

    #[test]
    fn bench_grid_set_does_not_duplicate_dirty_entries() {
        let mut grid = BenchGrid::new();
        grid.set(1, 10);
        grid.set(1, 20);
        assert_eq!(grid.dirty_fields(), &["f1"]);
    }

    #[test]
    fn bench_grid_view_renders_all_bindings_and_escapes_payload() {
        let grid = BenchGrid::new();
        let html = render(&grid.view());
        assert!(verify_grid_escape(&html, grid.values[0]));
        // 全束縛点が出力に含まれることの簡易確認（最後の field 名）。
        assert!(html.contains(&format!("data-bind-text=\"f{}\"", BINDINGS - 1)));
    }

    #[test]
    fn bench_grid_view_detects_unescaped_payload_as_not_ok() {
        let raw_html = format!("999999 {XSS_MARKER}");
        assert!(!verify_grid_escape(&raw_html, 999_999));
    }

    #[test]
    fn binding_table_resolve_and_format_counts_only_dirty_fields() {
        let mut grid = BenchGrid::new();
        grid.set(0, 1);
        grid.set(5, 2);
        let table = BindingTable::new();
        let applied = table.resolve_and_format(&grid.values, grid.dirty_fields());
        assert_eq!(applied, 2);
    }

    #[test]
    fn binding_table_resolve_and_format_is_zero_when_dirty_is_empty() {
        let grid = BenchGrid::new();
        let table = BindingTable::new();
        let applied = table.resolve_and_format(&grid.values, grid.dirty_fields());
        assert_eq!(applied, 0);
    }

    #[test]
    fn build_appstate_1k_has_expected_binding_count_and_ids() {
        let state = build_appstate_1k();
        assert_eq!(state.items.len(), BINDINGS);
        assert_eq!(state.item_ids.len(), BINDINGS);
        assert_eq!(state.item_ids, (0..BINDINGS as u64).collect::<Vec<_>>());
        assert!(state.dirty_fields().is_empty());
    }

    #[test]
    fn appstate_update_increment_reports_dirty_counter_only() {
        let mut state = build_appstate_1k();
        state.update(Action::Increment);
        assert_eq!(state.dirty_fields(), &["counter"]);
        let applied = resolve_appstate_dirty(&state);
        assert_eq!(applied, 1);
    }

    #[test]
    fn appstate_noop_reset_at_zero_has_no_dirty_and_no_applied() {
        let mut state = build_appstate_1k();
        assert_eq!(state.counter, 0);
        state.update(Action::Reset);
        assert!(state.dirty_fields().is_empty());
        assert_eq!(resolve_appstate_dirty(&state), 0);
    }

    #[test]
    fn report_self_check_ok_is_true_only_when_both_checks_pass() {
        let mut report = sample_report();
        assert!(report.self_check_ok());

        report.escape_ok = false;
        assert!(!report.self_check_ok());

        report.escape_ok = true;
        report.noop_ok = false;
        assert!(!report.self_check_ok());
    }

    fn sample_stats() -> Stats {
        Stats {
            iters: ITERS,
            mean_us: 10.0,
            p50_us: 9.0,
            p95_us: 15.0,
            min_us: 8.0,
        }
    }

    fn sample_scenario() -> ScenarioReport {
        ScenarioReport {
            update: sample_stats(),
            binding_apply: sample_stats(),
            render: sample_stats(),
            noop_update: sample_stats(),
        }
    }

    fn sample_report() -> Report {
        Report {
            version: "0.2.2".to_string(),
            grid1k: sample_scenario(),
            appstate1k: sample_scenario(),
            escape_ok: true,
            noop_ok: true,
            notes: "profile=release".to_string(),
        }
    }

    #[test]
    fn report_to_json_line_round_trips_through_json_parse_with_expected_keys() {
        let report = sample_report();
        let line = report.to_json_line();
        let parsed = parse(&line).expect("to_json_line の出力は有効な JSON のはず");

        assert_eq!(
            parsed.get("framework").and_then(Json::as_str),
            Some("fandhe-frontend")
        );
        assert_eq!(
            parsed.get("mode").and_then(Json::as_str),
            Some("state-update")
        );
        assert!(matches!(
            parsed.get("workload_schema_version"),
            Some(Json::Number(n)) if *n == WORKLOAD_SCHEMA_VERSION as f64
        ));
        assert!(matches!(
            parsed.get("bindings"),
            Some(Json::Number(n)) if *n == BINDINGS as f64
        ));
        assert!(matches!(
            parsed.get("grid1k").and_then(|v| v.get("update")).and_then(|v| v.get("iters")),
            Some(Json::Number(n)) if *n == ITERS as f64
        ));
        assert!(matches!(parsed.get("escape_ok"), Some(Json::Bool(true))));
        assert!(matches!(parsed.get("noop_ok"), Some(Json::Bool(true))));
    }

    #[test]
    fn compare_reports_all_metrics_with_delta() {
        let current = sample_report();
        let baseline = current.to_json_line();
        let lines =
            compare(&current, &baseline).expect("baseline は自分自身の出力なので成功するはず");
        // 2 シナリオ × 4 フェーズ × 4 指標 = 32 行。
        assert_eq!(lines.len(), 32);
        for line in &lines {
            assert!(line.starts_with("bench-state-update-compare: metric="));
        }
    }

    #[test]
    fn compare_allows_version_mismatch() {
        let current = sample_report();
        let mut baseline_report = sample_report();
        baseline_report.version = "0.1.0".to_string();
        let baseline = baseline_report.to_json_line();
        let lines = compare(&current, &baseline).expect("version のみの不一致は許容されるはず");
        assert_eq!(lines.len(), 32);
    }

    #[test]
    fn compare_rejects_invalid_json_baseline() {
        let current = sample_report();
        let err = compare(&current, "not json").expect_err("不正 JSON は失敗するはず");
        assert!(matches!(err, BenchStateUpdateError::InvalidBaseline(_)));
    }

    #[test]
    fn compare_rejects_baseline_missing_required_key() {
        let current = sample_report();
        let err = compare(&current, "{}").expect_err("必須キー欠落は失敗するはず");
        assert!(matches!(err, BenchStateUpdateError::InvalidBaseline(_)));
    }

    #[test]
    fn compare_rejects_mismatched_workload_schema_version() {
        let current = sample_report();
        let baseline = current.to_json_line().replace(
            &format!("\"workload_schema_version\":{WORKLOAD_SCHEMA_VERSION}"),
            "\"workload_schema_version\":999999",
        );
        let err = compare(&current, &baseline)
            .expect_err("workload_schema_version 不一致の baseline は拒否されるはず");
        assert!(matches!(err, BenchStateUpdateError::InvalidBaseline(_)));
    }

    #[test]
    fn compare_rejects_mismatched_build_profile() {
        let current = sample_report();
        let mut baseline_report = sample_report();
        baseline_report.notes = "profile=debug".to_string();
        let baseline = baseline_report.to_json_line();
        let err = compare(&current, &baseline)
            .expect_err("異なるビルドプロファイルの baseline は拒否されるはず");
        assert!(matches!(err, BenchStateUpdateError::InvalidBaseline(_)));
    }
}
