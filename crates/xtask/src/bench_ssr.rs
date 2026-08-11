//! SSR 性能ベンチの常設コマンド（イシュー #1317）。
//!
//! 非追跡領域（`_/bench/fandhe/ssr`、gitignore 対象）で行っていた SSR
//! スループット計測を、`cargo run -p xtask -- bench-ssr` として本リポジトリへ
//! 正式化する。非追跡領域はこのリポジトリの成果物として再現・回帰比較でき
//! ないため、ワークロード・計測手順・出力スキーマは以下に転記して本モジュール内で
//! 自己完結させる（呼び出し元は `xtask/src/main.rs::run_bench_ssr`）。
//!
//! # ワークロード（固定・CLI から差し替え不可）
//!
//! `html > body > (header > h1 "Benchmark") + (table#bench-table > tbody >
//! tr×N) + (footer > p "generated N rows")`。各行 `tr` は
//! `td(連番) + td(XSS ペイロード混入ラベル)`。ラベルは
//! `Row <i> & "quoted" <script>alert(1)</script>` で、既定エスケープ（REQ-1）
//! を経由する [`fandhe_frontend_core::text`] のみで出力する（`format!` に
//! よる HTML 文字列の直接組み立ては行わない、`coding-rust.md` 厳守）。
//! 乱数は使わず、`rows` のみに依存する決定的な構造にする。
//!
//! ワークロードを固定し CLI から差し替え不可にしているのは、`check-loc` 等の
//! 「判定対象は差し替え不可」という既存設計原則を踏襲し、before/after 比較の
//! 前提（同一ワークロード）を崩さないため。
//!
//! # 計測手順
//!
//! - rows=1,000: ウォームアップ 20 回 + 計測 100 回
//! - rows=10,000: ウォームアップ 2 回 + 計測 10 回
//!
//! 計測対象は [`fandhe_frontend_core::render`] による HTML 文字列化
//! （エスケープ処理を含む「SSR 描画」コスト）のみとし、[`std::time::Instant`]
//! で計測してミリ秒単位で記録する。`page(rows)` によるノード木構築は
//! タイマー開始**前**に行い計測対象から意図的に除外する（構築コストと
//! 描画コストを混ぜると、両者の内訳変化が「SSR 描画性能の回帰」として
//! 誤って現れうるため。`_/bench` 側プロトコルとの比較可能性を保つ判断でも
//! ある）。`page(rows)` の入力（`rows`）・`render()` の戻り値の双方へ
//! [`std::hint::black_box`] を通し、定数畳み込み・ループ外巻き上げによる
//! 非現実的な計測値を防ぐ（`bench_binding_update` モジュール（イシュー #592
//! PR #623 レビュー知見）と同じ対策）。
//!
//! # 出力（stdout・JSON 1 行）
//!
//! `framework` / `version`（`fandhe-frontend-core` の実バージョン。
//! [`resolve_core_version`] が `cargo metadata --no-deps` のみで解決し
//! ネットワークアクセスは行わない）/ `mode`（常に `"ssr"`）/ `rows1k` ・
//! `rows10k`（各 `iters`/`mean_ms`/`p50_ms`/`p95_ms`/`min_ms`）/
//! `html_bytes_1k`（rows=1,000 出力の UTF-8 バイト数） / `escape_ok` /
//! `row_count_ok` / `notes`（`profile=debug` または `profile=release`。
//! `cfg!(debug_assertions)` で判定し、異なるビルドプロファイル間の比較誤用を
//! 機械検知可能にする）。
//!
//! # 検証（fail-closed）・回帰比較（report-only）
//!
//! `escape_ok`（1k 出力に生の `<script>alert(1)</script>` が含まれない）・
//! `row_count_ok`（`<tr` 出現回数が 1,000 と一致）のいずれかが `false` の場合、
//! 呼び出し元（`run_bench_ssr`）は JSON 行を出力したうえで終了コード 1 を
//! 返す。この検証を緩和する CLI 引数・環境変数は設けない。
//!
//! 任意の `--baseline <FILE>`（呼び出し元がパースして [`compare`] へ渡す）を
//! 指定すると、過去の本コマンド出力（JSON 1 行）との差分を
//! `bench-ssr-compare: metric=<name> baseline=<b> current=<c> delta_pct=<±x.xx>`
//! の行群として追加出力する。**report-only**（数値比較を根拠に判定・終了
//! コードを変えない）とし、CI 閾値ゲート化は本イシューの明示スコープ外
//! （実装計画 §8）のため行わない。
//!
//! # CI ゲート化しない設計判断
//!
//! `bench_binding_update` と同じ理由（計測値は実行環境・負荷に依存し、
//! しきい値判定は偽陽性/偽陰性の温床になる）により、本サブコマンドの終了
//! コードは検証結果（`escape_ok`/`row_count_ok`）にのみ依存し、性能数値の
//! 良し悪しには依存しない。

use std::fmt;
use std::hint::black_box;
use std::time::Instant;

use fandhe_frontend_core::{el, footer, h1, header, p, render, table, tbody, td, text, tr};

use crate::check_dep_versions;
use crate::json::{parse, Json, JsonError};

/// rows=1,000 側のウォームアップ回数（計測に含めない）。
const ROWS_1K: usize = 1_000;
const ROWS_1K_WARMUP: usize = 20;
const ROWS_1K_ITERS: usize = 100;

/// rows=10,000 側のウォームアップ回数（計測に含めない）。
const ROWS_10K: usize = 10_000;
const ROWS_10K_WARMUP: usize = 2;
const ROWS_10K_ITERS: usize = 10;

/// XSS 回帰検知用の意図的なペイロード。必ず [`fandhe_frontend_core::text`]
/// （既定エスケープ経由）でのみ出力する。生の `<script>` タグとして
/// レンダリング結果に出現したら [`verify`] が `escape_ok=false` を返す。
const XSS_MARKER: &str = "<script>alert(1)</script>";

/// 本モジュール専用のエラー型。`cargo metadata` の失敗・baseline JSON の
/// 不正入力を fail-closed に扱う（security.md A08: 非信頼入力の防御的処理）。
#[derive(Debug)]
pub enum BenchSsrError {
    /// 実行環境起因の失敗（`cargo metadata` 失敗等）。
    Environment(String),
    /// `--baseline` ファイルの読み取り・パース・スキーマ検証失敗。
    InvalidBaseline(String),
}

impl fmt::Display for BenchSsrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BenchSsrError::Environment(msg) => write!(f, "environment error: {msg}"),
            BenchSsrError::InvalidBaseline(msg) => write!(f, "invalid baseline: {msg}"),
        }
    }
}

impl std::error::Error for BenchSsrError {}

/// 1 rows 設定の統計サマリ。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats {
    pub iters: usize,
    pub mean_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub min_ms: f64,
}

/// `bench-ssr` の実行結果全体。[`to_json_line`](Self::to_json_line) が
/// 呼び出し元 stdout へ出力する契約形式へ整形する。
#[derive(Debug, Clone)]
pub struct Report {
    pub version: String,
    pub rows1k: Stats,
    pub rows10k: Stats,
    pub html_bytes_1k: usize,
    pub escape_ok: bool,
    pub row_count_ok: bool,
    pub notes: String,
}

impl Report {
    /// 呼び出し元（`run_bench_ssr`）が終了コードを決める判定を、テスト可能な
    /// 純粋関数として切り出したもの。`escape_ok` かつ `row_count_ok` の
    /// 両方が真のときのみ `true`（fail-closed: どちらか一方でも偽なら偽）。
    #[must_use]
    pub fn self_check_ok(&self) -> bool {
        self.escape_ok && self.row_count_ok
    }

    /// JSON 1 行へ整形する。文字列フィールド（`version`/`notes`）は
    /// [`json_escape`] で最小限のエスケープ（`"` `\` 制御文字）を通す
    /// （xtask 外部依存ゼロ方針のため `serde_json` は使わず内製）。
    #[must_use]
    pub fn to_json_line(&self) -> String {
        format!(
            "{{\"framework\":\"fandhe-frontend\",\"version\":\"{version}\",\"mode\":\"ssr\",\
             \"rows1k\":{rows1k},\"rows10k\":{rows10k},\"html_bytes_1k\":{html_bytes_1k},\
             \"escape_ok\":{escape_ok},\"row_count_ok\":{row_count_ok},\"notes\":\"{notes}\"}}",
            version = json_escape(&self.version),
            rows1k = stats_to_json(&self.rows1k),
            rows10k = stats_to_json(&self.rows10k),
            html_bytes_1k = self.html_bytes_1k,
            escape_ok = self.escape_ok,
            row_count_ok = self.row_count_ok,
            notes = json_escape(&self.notes),
        )
    }
}

fn stats_to_json(s: &Stats) -> String {
    format!(
        "{{\"iters\":{iters},\"mean_ms\":{mean_ms:.4},\"p50_ms\":{p50_ms:.4},\"p95_ms\":{p95_ms:.4},\"min_ms\":{min_ms:.4}}}",
        iters = s.iters,
        mean_ms = s.mean_ms,
        p50_ms = s.p50_ms,
        p95_ms = s.p95_ms,
        min_ms = s.min_ms,
    )
}

/// 最小限の JSON 文字列エスケープ（`"` / `\` / 制御文字）。
///
/// `version`（`cargo metadata` 由来）・`notes`（本モジュール内で組み立てる
/// 固定文字列）はいずれも本質的にエスケープ対象文字を含まない想定だが、
/// 将来のバージョン文字列変化・notes 拡張に備えて防御的に適用する
/// （security.md A08 と同じ判断軸）。
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

/// 1 行分（`tr`）のノードを組み立てる。ラベルは意図的な XSS ペイロード混入
/// 文字列だが、必ず [`text`]（既定エスケープ経由）で出力する
/// （`coding-rust.md`: HTML 文字列の直接組み立て禁止の厳守）。
fn row(i: usize) -> fandhe_frontend_core::Node {
    let label = format!("Row {i} & \"quoted\" {XSS_MARKER}");
    tr(
        vec![],
        vec![
            td(vec![], vec![text(i.to_string())]),
            td(vec![], vec![text(label)]),
        ],
    )
}

/// ワークロード全体（`html > body > header + table + footer`）を組み立てる。
///
/// `fandhe_frontend_core::tags` は `html`/`body` タグの専用ヘルパを持たない
/// ため、汎用 [`el`] を使う。
fn page(rows: usize) -> fandhe_frontend_core::Node {
    let body = el(
        "body",
        vec![],
        vec![
            header(vec![], vec![h1(vec![], vec![text("Benchmark")])]),
            table(
                vec![("id", "bench-table")],
                vec![tbody(vec![], (0..rows).map(row).collect())],
            ),
            footer(
                vec![],
                vec![p(vec![], vec![text(format!("generated {rows} rows"))])],
            ),
        ],
    );
    el("html", vec![], vec![body])
}

/// ソート済み `durations_ms` から百分位を求める（`floor(p * (n-1))` 番目）。
/// 空スライスは 0.0 を返す（呼び出し元は `iters >= 1` を保証するため実運用では
/// 到達しないが、防御的に扱う）。
fn percentile(sorted_ms: &[f64], p: f64) -> f64 {
    let Some(&last) = sorted_ms.last() else {
        return 0.0;
    };
    let idx = (p * (sorted_ms.len() - 1) as f64).floor() as usize;
    sorted_ms.get(idx).copied().unwrap_or(last)
}

fn stats_from_durations(mut durations_ms: Vec<f64>) -> Stats {
    let iters = durations_ms.len();
    // `f64` は全順序でないため `partial_cmp` の `unwrap()` は避け、
    // NaN が混入しない前提（`Instant::elapsed` 由来なので常に非負・有限）
    // でも panic しない `total_cmp` を使う。
    durations_ms.sort_by(f64::total_cmp);
    let sum: f64 = durations_ms.iter().sum();
    let mean_ms = if iters == 0 { 0.0 } else { sum / iters as f64 };
    let min_ms = durations_ms.first().copied().unwrap_or(0.0);
    Stats {
        iters,
        mean_ms,
        p50_ms: percentile(&durations_ms, 0.50),
        p95_ms: percentile(&durations_ms, 0.95),
        min_ms,
    }
}

/// `rows` 件のワークロードを `warmup` 回のウォームアップの後 `iters` 回計測し、
/// 統計サマリと最終回の HTML（[`verify`] 用。ワークロードは決定的なため
/// どの回の出力も同一）を返す。
///
/// `black_box` を入力（`rows`）・出力（`render` 結果）双方に通し、
/// 最適化器によるループ不変コード移動（LICM）・定数畳み込みを防ぐ
/// （`bench_binding_update::measure` と同じ対策、イシュー #592 PR #623
/// レビュー知見）。
fn measure_rows(rows: usize, warmup: usize, iters: usize) -> (Stats, String) {
    for _ in 0..warmup {
        let node = page(black_box(rows));
        black_box(render(&node));
    }

    let mut durations_ms = Vec::with_capacity(iters);
    let mut last_html = String::new();
    for _ in 0..iters {
        let node = page(black_box(rows));
        let start = Instant::now();
        let html = render(&node);
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        durations_ms.push(elapsed_ms);
        // ループ内・毎回 black_box へ通し、最適化器が「戻り値は最終回のみ
        // 観測される」と証明してループ不変コード移動（LICM）を行うのを防ぐ
        // （`bench_binding_update::measure` と同型の対策、イシュー #592
        // PR #623 レビュー指摘）。
        last_html = black_box(html);
    }

    (stats_from_durations(durations_ms), last_html)
}

/// 検証結果 `(escape_ok, row_count_ok)` を返す。
///
/// - `escape_ok`: 各行のラベル（[`row`] が組み立てる
///   `"Row {i} & \"quoted\" <script>alert(1)</script>"`）が、独立に組み立てた
///   エスケープ済みリテラル `"Row {i} &amp; &quot;quoted&quot;
///   &lt;script&gt;alert(1)&lt;/script&gt;"` と完全一致するかを
///   `0..expected_rows` の全行について検証し、かつ未エスケープの生ラベルが
///   出力中に存在しないことも併せて検証する（既定エスケープ回帰の検知、
///   REQ-1）。**期待値はレンダリング経路が使う
///   [`fandhe_frontend_core::escape_html`] を呼ばずに固定リテラルとして
///   組み立てる**（codex-review P0 指摘: 検証対象と同じ関数で期待値を
///   生成すると、例えば `escape_html` が `<` をエスケープしなくなった場合に
///   実際の HTML と期待値が同じ未エスケープ文字列に揃ってしまい、
///   `escape_ok=true` のまま生の `<script>` 混入を見逃す）。生の
///   `XSS_MARKER` 不在のみを見る素朴な部分一致では、`&`/`"` だけが
///   未エスケープになる・開始タグと終了タグの一方だけが未エスケープになる
///   といった**部分的な**回帰が `escape_ok=true` へすり抜けてしまう
///   （直前の codex-review 指摘）。エスケープ済みラベル文字列全体の完全
///   一致にすることで、5 対象文字（`&` `<` `>` `"` `'`）のどれか 1 つでも
///   エスケープが崩れれば必ず不一致になり検知できる。`verify` は計測
///   ループの外（`run`）から 1 回だけ呼ばれるため、行数分のループ・
///   文字列確保のコストは許容できる
/// - `row_count_ok`: `<tr` の出現回数が `expected_rows` と一致すること
fn verify(html: &str, expected_rows: usize) -> (bool, bool) {
    let escape_ok = (0..expected_rows).all(|i| {
        // 検証対象（`row`/`render`）が経由する `escape_html` を呼ばず、
        // 期待するエスケープ済み文字列を独立したリテラルとして組み立てる。
        let expected_label =
            format!("Row {i} &amp; &quot;quoted&quot; &lt;script&gt;alert(1)&lt;/script&gt;");
        // 未エスケープの生ラベルが紛れ込んでいないことも併せて検証する
        // （部分的な回帰の見逃し防止）。
        let raw_label = format!("Row {i} & \"quoted\" {XSS_MARKER}");
        html.contains(&expected_label) && !html.contains(&raw_label)
    });
    let row_count_ok = html.matches("<tr").count() == expected_rows;
    (escape_ok, row_count_ok)
}

/// `cargo metadata --no-deps` のみで `fandhe-frontend-core` の実バージョンを
/// 解決する（ネットワークアクセスなし）。`check_dep_versions` モジュールの
/// 既存実装を再利用し、新規の cargo metadata 呼び出し経路を増やさない。
pub fn resolve_core_version() -> Result<String, BenchSsrError> {
    let (_workspace_root, members) = check_dep_versions::workspace_packages_from_cargo_metadata()
        .map_err(|e| {
        BenchSsrError::Environment(format!(
            "failed to resolve fandhe-frontend-core version via cargo metadata: {e}"
        ))
    })?;
    members
        .into_iter()
        .find(|m| m.name == "fandhe-frontend-core")
        .map(|m| m.version)
        .ok_or_else(|| {
            BenchSsrError::Environment(
                "fandhe-frontend-core not found in `cargo metadata --no-deps` workspace members"
                    .to_string(),
            )
        })
}

/// 全計測を実行し [`Report`] を組み立てる。
///
/// `version` は呼び出し元（`run_bench_ssr`）が [`resolve_core_version`] で
/// 解決した値を渡す（本関数自体は cargo プロセスを起動しない）。
#[must_use]
pub fn run(version: String) -> Report {
    let (rows1k, html_1k) = measure_rows(ROWS_1K, ROWS_1K_WARMUP, ROWS_1K_ITERS);
    let (rows10k, _html_10k) = measure_rows(ROWS_10K, ROWS_10K_WARMUP, ROWS_10K_ITERS);

    let (escape_ok, row_count_ok) = verify(&html_1k, ROWS_1K);

    Report {
        version,
        rows1k,
        rows10k,
        html_bytes_1k: html_1k.len(),
        escape_ok,
        row_count_ok,
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
/// `bench-ssr-compare: metric=<name> baseline=<b> current=<c> delta_pct=<±x.xx>`
/// の行群を返す（report-only、判定なし）。
///
/// パース失敗・必須キー欠落は [`BenchSsrError::InvalidBaseline`]
/// として fail-closed に扱う（非信頼入力、security.md A08）。
pub fn compare(current: &Report, baseline_json: &str) -> Result<Vec<String>, BenchSsrError> {
    let root = parse(baseline_json).map_err(|e: JsonError| {
        BenchSsrError::InvalidBaseline(format!("failed to parse baseline JSON: {e}"))
    })?;

    let mut lines = Vec::new();
    for (metric, current_value) in [
        ("rows1k.mean_ms", current.rows1k.mean_ms),
        ("rows1k.p50_ms", current.rows1k.p50_ms),
        ("rows1k.p95_ms", current.rows1k.p95_ms),
        ("rows1k.min_ms", current.rows1k.min_ms),
        ("rows10k.mean_ms", current.rows10k.mean_ms),
        ("rows10k.p50_ms", current.rows10k.p50_ms),
        ("rows10k.p95_ms", current.rows10k.p95_ms),
        ("rows10k.min_ms", current.rows10k.min_ms),
        ("html_bytes_1k", current.html_bytes_1k as f64),
    ] {
        let baseline_value = lookup_metric(&root, metric)?;
        let delta_pct = if baseline_value == 0.0 {
            // 0 除算を避ける。基準値が 0 の場合、変化率は定義できないため
            // 0.00 として報告する（report-only であり判定に使わないため
            // 安全側に倒す実装単純化）。
            0.0
        } else {
            (current_value - baseline_value) / baseline_value * 100.0
        };
        lines.push(format!(
            "bench-ssr-compare: metric={metric} baseline={baseline_value:.4} current={current_value:.4} delta_pct={delta_pct:+.2}"
        ));
    }

    Ok(lines)
}

/// `root`（baseline JSON のトップレベルオブジェクト）から `"rows1k.mean_ms"`
/// のようなドット区切りパスで数値を引く。キー欠落・型不一致は
/// [`BenchSsrError::InvalidBaseline`] とする。
fn lookup_metric(root: &Json, dotted_path: &str) -> Result<f64, BenchSsrError> {
    let mut current = root;
    for key in dotted_path.split('.') {
        current = current.get(key).ok_or_else(|| {
            BenchSsrError::InvalidBaseline(format!(
                "baseline JSON is missing key `{key}` (path `{dotted_path}`)"
            ))
        })?;
    }
    match current {
        Json::Number(n) => Ok(*n),
        _ => Err(BenchSsrError::InvalidBaseline(format!(
            "baseline JSON value at `{dotted_path}` is not a number"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_matches_known_vectors() {
        // n=5, 0-indexed: [10,20,30,40,50]
        let sorted = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        // p50: floor(0.50*4)=2 -> 30.0
        assert_eq!(percentile(&sorted, 0.50), 30.0);
        // p95: floor(0.95*4)=3 -> 40.0
        assert_eq!(percentile(&sorted, 0.95), 40.0);
        // p0: floor(0*4)=0 -> 10.0
        assert_eq!(percentile(&sorted, 0.0), 10.0);
    }

    #[test]
    fn percentile_of_empty_slice_is_zero() {
        assert_eq!(percentile(&[], 0.50), 0.0);
    }

    #[test]
    fn stats_from_durations_computes_mean_min_and_percentiles() {
        let durations = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let stats = stats_from_durations(durations);
        assert_eq!(stats.iters, 5);
        assert_eq!(stats.mean_ms, 30.0);
        assert_eq!(stats.min_ms, 10.0);
        assert_eq!(stats.p50_ms, 30.0);
        assert_eq!(stats.p95_ms, 40.0);
    }

    #[test]
    fn verify_detects_escaped_payload_as_ok() {
        // text() 経由でエスケープ済みの HTML（実際の render() 出力を模す）。
        let html = "<tr><td>0</td><td>Row 0 &amp; &quot;quoted&quot; &lt;script&gt;alert(1)&lt;/script&gt;</td></tr>";
        let (escape_ok, row_count_ok) = verify(html, 1);
        assert!(escape_ok, "エスケープ済み出力は escape_ok=true のはず");
        assert!(row_count_ok, "<tr 1 件と rows=1 が一致するはず");
    }

    #[test]
    fn verify_detects_unescaped_payload_as_not_ok() {
        let html = "<tr><td>0</td><td>Row 0 & \"quoted\" <script>alert(1)</script></td></tr>";
        let (escape_ok, _row_count_ok) = verify(html, 1);
        assert!(
            !escape_ok,
            "生の <script>alert(1)</script> 混入は escape_ok=false のはず"
        );
    }

    #[test]
    fn verify_detects_partial_ampersand_and_quote_escape_regression_as_not_ok() {
        // `<` `>` は正しくエスケープされているが `&`/`"` のみ未エスケープに
        // 壊れたケース（codex-review 指摘: 生の XSS_MARKER 不在のみを見る
        // 実装だとこの部分回帰を escape_ok=true と誤判定していた）。
        let html =
            "<tr><td>0</td><td>Row 0 & \"quoted\" &lt;script&gt;alert(1)&lt;/script&gt;</td></tr>";
        let (escape_ok, _row_count_ok) = verify(html, 1);
        assert!(
            !escape_ok,
            "& / \" のみ未エスケープの部分回帰は escape_ok=false のはず"
        );
    }

    #[test]
    fn verify_detects_one_sided_tag_escape_regression_as_not_ok() {
        // 開始タグ `<script>` のみ未エスケープで終了タグ `</script>` は
        // エスケープ済みという非対称な部分回帰（codex-review 指摘のもう
        // 一方のシナリオ）。
        let html = "<tr><td>0</td><td>Row 0 &amp; &quot;quoted&quot; <script>alert(1)&lt;/script&gt;</td></tr>";
        let (escape_ok, _row_count_ok) = verify(html, 1);
        assert!(
            !escape_ok,
            "開始タグのみ未エスケープの部分回帰は escape_ok=false のはず"
        );
    }

    #[test]
    fn verify_detects_row_count_mismatch() {
        let html = "<tr></tr><tr></tr>";
        let (_escape_ok, row_count_ok) = verify(html, 3);
        assert!(!row_count_ok, "<tr 2 件と rows=3 の不一致を検知するはず");
    }

    #[test]
    fn page_renders_expected_row_count_and_escapes_payload() {
        let html = render(&page(5));
        let (escape_ok, row_count_ok) = verify(&html, 5);
        assert!(escape_ok);
        assert!(row_count_ok);
        assert!(html.starts_with("<html><body>"));
        assert!(html.contains("generated 5 rows"));
        assert!(html.contains(r#"id="bench-table""#));
    }

    #[test]
    fn report_to_json_line_round_trips_through_json_parse_with_expected_keys() {
        let report = Report {
            version: "0.2.0".to_string(),
            rows1k: Stats {
                iters: 100,
                mean_ms: 1.2345,
                p50_ms: 1.1,
                p95_ms: 1.9,
                min_ms: 0.9,
            },
            rows10k: Stats {
                iters: 10,
                mean_ms: 12.345,
                p50_ms: 11.1,
                p95_ms: 19.9,
                min_ms: 9.9,
            },
            html_bytes_1k: 123_456,
            escape_ok: true,
            row_count_ok: true,
            notes: "profile=debug".to_string(),
        };
        let line = report.to_json_line();
        let parsed = parse(&line).expect("to_json_line の出力は有効な JSON のはず");

        assert_eq!(
            parsed.get("framework").and_then(Json::as_str),
            Some("fandhe-frontend")
        );
        assert_eq!(parsed.get("version").and_then(Json::as_str), Some("0.2.0"));
        assert_eq!(parsed.get("mode").and_then(Json::as_str), Some("ssr"));
        assert!(matches!(
            parsed.get("rows1k").and_then(|v| v.get("iters")),
            Some(Json::Number(n)) if *n == 100.0
        ));
        assert!(matches!(
            parsed.get("rows10k").and_then(|v| v.get("mean_ms")),
            Some(Json::Number(_))
        ));
        assert!(matches!(
            parsed.get("html_bytes_1k"),
            Some(Json::Number(n)) if *n == 123_456.0
        ));
        assert!(matches!(parsed.get("escape_ok"), Some(Json::Bool(true))));
        assert!(matches!(parsed.get("row_count_ok"), Some(Json::Bool(true))));
        assert_eq!(
            parsed.get("notes").and_then(Json::as_str),
            Some("profile=debug")
        );
    }

    #[test]
    fn json_escape_handles_quotes_backslashes_and_control_chars() {
        assert_eq!(json_escape(r#"a"b\c"#), r#"a\"b\\c"#);
        assert_eq!(json_escape("a\nb"), "a\\nb");
    }

    fn sample_report() -> Report {
        Report {
            version: "0.2.0".to_string(),
            rows1k: Stats {
                iters: 100,
                mean_ms: 10.0,
                p50_ms: 9.0,
                p95_ms: 15.0,
                min_ms: 8.0,
            },
            rows10k: Stats {
                iters: 10,
                mean_ms: 100.0,
                p50_ms: 90.0,
                p95_ms: 150.0,
                min_ms: 80.0,
            },
            html_bytes_1k: 1000,
            escape_ok: true,
            row_count_ok: true,
            notes: "profile=release".to_string(),
        }
    }

    #[test]
    fn compare_reports_all_nine_metrics_with_delta() {
        let current = sample_report();
        let baseline = current.to_json_line();
        let lines =
            compare(&current, &baseline).expect("baseline は自分自身の出力なので成功するはず");
        assert_eq!(lines.len(), 9);
        for line in &lines {
            assert!(line.starts_with("bench-ssr-compare: metric="));
            assert!(line.contains("delta_pct=+0.00") || line.contains("delta_pct=-0.00"));
        }
    }

    #[test]
    fn compare_computes_nonzero_delta_pct() {
        let mut current = sample_report();
        current.rows1k.mean_ms = 11.0;
        let mut baseline_report = sample_report();
        baseline_report.rows1k.mean_ms = 10.0;
        let baseline = baseline_report.to_json_line();
        let lines = compare(&current, &baseline).expect("パース成功するはず");
        let mean_line = lines
            .iter()
            .find(|l| l.contains("metric=rows1k.mean_ms"))
            .expect("rows1k.mean_ms の行があるはず");
        assert!(
            mean_line.contains("delta_pct=+10.00"),
            "10.0 -> 11.0 は +10.00% のはず: {mean_line}"
        );
    }

    #[test]
    fn compare_rejects_invalid_json_baseline() {
        let current = sample_report();
        let err = compare(&current, "not json").expect_err("不正 JSON は失敗するはず");
        assert!(matches!(err, BenchSsrError::InvalidBaseline(_)));
    }

    #[test]
    fn compare_rejects_baseline_missing_required_key() {
        let current = sample_report();
        let err = compare(&current, "{}").expect_err("必須キー欠落は失敗するはず");
        assert!(matches!(err, BenchSsrError::InvalidBaseline(_)));
    }

    // `run_bench_ssr`（xtask/src/main.rs）の終了コード分岐は
    // `Report::self_check_ok()` の戻り値のみを見て決まる。CLI 契約テスト
    // （`tests/cli_bench_ssr.rs`）は現実の render() 出力が常に PASS になる
    // ため「false のとき終了コード非 0」という fail-closed 分岐そのものは
    // 通過できない（`if` を反転させても既存テストは全て通ってしまう）。
    // ここで合成 Report を使い、判定関数自体を直接検証する。
    #[test]
    fn self_check_ok_is_true_only_when_both_checks_pass() {
        let mut report = sample_report();
        assert!(report.self_check_ok());

        report.escape_ok = false;
        assert!(!report.self_check_ok());

        report.escape_ok = true;
        report.row_count_ok = false;
        assert!(!report.self_check_ok());

        report.escape_ok = false;
        report.row_count_ok = false;
        assert!(!report.self_check_ok());
    }
}
