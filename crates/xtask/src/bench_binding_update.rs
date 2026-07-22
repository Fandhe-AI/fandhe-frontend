//! 「全再描画 vs 束縛点差分更新」のネイティブ計測ハーネス（イシュー #592）。
//!
//! PR #557（headless-ui の Disclosure/SingleSelect 状態機械）は
//! 「`DirtyTracked` 実装（wasm 束縛点差分更新用）は Phase 2 の具象コンポーネント
//! 束縛点設計と同時に判断」として out-of-scope とした。本モジュールはこの
//! 判断のための定量データを採取する。`fandhe_frontend_interactive::DirtyTracked`
//! 自体は既に実装済み（イシュー #341）で `fandhe_frontend_wasm_full::Runtime<C>`
//! が消費している。本モジュールが計測するのは「headless-ui 状態機械へ同トレイト
//! を実装する追加コストが、全再描画（`view()` + `core::render()` の HTML
//! 再生成）比で不利にならないか」という採否判断の根拠。
//!
//! # 計測対象・限界
//!
//! 計測はすべてネイティブ実行（DOM 操作を伴わない）。実際の DOM 反映コスト
//! （`set_attribute`/`textContent` 書き換え等）は既存 `docs/ci/perf-browser-harness.md`
//! （wasm-full 実ブラウザ計測）が別途裏付けており、本モジュールは「再描画
//! ペイロード生成コスト（`view()` + `render()`）vs dirty 列挙・値解決コスト
//! （`dirty_fields()` 呼び出しのみ）」という Rust 側の相対比較に限定する。
//! 数値は実行環境（CPU・負荷）に依存するため、しきい値判定は行わない
//! report-only ハーネスであり、CI ゲート化はしない（`bench-binding-update`
//! サブコマンドの終了コードは常に成功。`xtask/tests/cli_bench_binding_update.rs`
//! が出力形式のみを検証する）。
//!
//! # シナリオ
//!
//! - `appstate-increment`: [`fandhe_frontend_interactive::AppState`] の
//!   `Action::Increment` dispatch。全再描画経路は `view()` の `Node` 生成 +
//!   `fandhe_frontend_core::render()` による HTML 文字列化、差分経路は
//!   `dirty_fields()` の読み出しのみ。
//! - `disclosure-toggle`: [`fandhe_frontend_headless_ui::Disclosure`] の
//!   `"toggle"` dispatch。同様の比較。
//! - `single-select-select`: [`fandhe_frontend_headless_ui::SingleSelect`] の
//!   `"select"` dispatch。同様の比較。

use std::fmt;
use std::hint::black_box;
use std::time::Instant;

use fandhe_frontend_core::render;
use fandhe_frontend_headless_ui::{Disclosure, SingleSelect};
use fandhe_frontend_interactive::{dispatch, AppState, Component, DirtyTracked};

/// 1 シナリオあたりの反復回数。ウォームアップ分は測定に含めない。
///
/// ネイティブ実行の 1 回あたりコストはマイクロ秒未満になりうるため、
/// `Instant` の分解能に対して十分な回数を反復し、反復回数で割った平均値を
/// 採る（中央値ではなく単純平均。report-only でありゲート化しないため、
/// 実装の単純さ（明示性・機械検証可能性）を優先する判断）。
const ITERATIONS: u32 = 10_000;
/// 反復開始前のウォームアップ回数（JIT 相当の最適化・キャッシュ温めは
/// Rust ネイティブでは主要因ではないが、測定開始直後の外れ値を避けるため
/// 最小限のウォームアップを行う）。
const WARMUP_ITERATIONS: u32 = 100;

/// 1 シナリオの計測結果。
///
/// 平均所要時間はナノ秒未満へ丸め込まれる `Duration` ではなく `f64`
/// （ナノ秒）で保持する。`Duration` に格納すると `ITERATIONS` による整数除算で
/// サブ ns の平均が `0ns` へ切り捨てられ、[`speedup_ratio`](Self::speedup_ratio)
/// が実際には「計測不能なほど高速」ではなく「精度不足で判別できない」場合にも
/// `f64::INFINITY` を返してしまう問題があった（イシュー #592 PR #623 レビュー
/// 指摘）。`f64` 保持により合計経過時間を反復回数で割った実数平均を保持でき、
/// この誤判定を避ける。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScenarioReport {
    /// シナリオ名（`bench-binding-update: scenario=<name> ...` の `<name>`）。
    pub scenario: &'static str,
    /// 全再描画経路（`view()` + `render()`）の 1 回あたり平均所要時間（ナノ秒）。
    pub full_rerender_ns: f64,
    /// 差分更新経路（`dirty_fields()` 読み出しのみ）の 1 回あたり平均所要時間（ナノ秒）。
    pub dirty_update_ns: f64,
}

impl ScenarioReport {
    /// 全再描画に対する差分更新の高速化倍率（`full_rerender_ns / dirty_update_ns`）。
    ///
    /// `dirty_update_ns` が厳密に 0.0 の場合（`Instant` の分解能を超えて合計
    /// 経過時間そのものが 0 だった場合。`f64` 平均採用後も理論上あり得るため
    /// 除算 panic を避ける fail-closed フォールバックとして残す）は
    /// `f64::INFINITY` を返す。
    #[must_use]
    pub fn speedup_ratio(&self) -> f64 {
        if self.dirty_update_ns == 0.0 {
            return f64::INFINITY;
        }
        self.full_rerender_ns / self.dirty_update_ns
    }
}

impl fmt::Display for ScenarioReport {
    /// 機械可読な 1 行サマリ（既存 `check_deps::format_report` 等のパターン
    /// 踏襲）。呼び出し元（`main.rs::run_bench_binding_update`）が
    /// シナリオごとに 1 行ずつ stdout へ出力する。ナノ秒平均は小数点以下 2 桁
    /// まで表示し、サブ ns の平均が `0` に丸め込まれて見えることを避ける。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "bench-binding-update: scenario={} full_ns={:.2} dirty_ns={:.2} ratio={:.2}",
            self.scenario,
            self.full_rerender_ns,
            self.dirty_update_ns,
            self.speedup_ratio()
        )
    }
}

/// `AppState` の `"increment"` dispatch について全再描画 vs 差分更新を計測する。
#[must_use]
pub fn bench_appstate_increment() -> ScenarioReport {
    let full_rerender_ns = measure(|| {
        let mut state = AppState::new();
        dispatch(&mut state, "increment", "");
        // 全再描画経路: view() の Node 生成 + render() による HTML 文字列化。
        // 戻り値を measure() 経由で black_box に通し、リリースビルドでの
        // 最適化除去（デッドコード化）を避ける。
        render(&state.view())
    });

    let dirty_update_ns = measure(|| {
        let mut state = AppState::new();
        dispatch(&mut state, "increment", "");
        // 差分更新経路: dirty_fields() の読み出しのみ（DOM 反映自体は
        // wasm-full 側の BindingTable::apply_dirty の責務であり、本モジュールの
        // 計測対象は「どのフィールドを更新すべきか」を特定するコストに限定）。
        // dirty_fields() は state を借用する &[&'static str] を返すため
        // measure() の戻り値としてそのまま持ち出せない。ここで black_box に
        // 通してから破棄し、呼び出し自体の最適化除去を避ける。
        black_box(state.dirty_fields());
    });

    ScenarioReport {
        scenario: "appstate-increment",
        full_rerender_ns,
        dirty_update_ns,
    }
}

/// [`Disclosure`] の `"toggle"` dispatch について全再描画 vs 差分更新を計測する。
#[must_use]
pub fn bench_disclosure_toggle() -> ScenarioReport {
    let full_rerender_ns = measure(|| {
        let mut state = Disclosure::default();
        dispatch(&mut state, "toggle", "");
        render(&state.view())
    });

    let dirty_update_ns = measure(|| {
        let mut state = Disclosure::default();
        dispatch(&mut state, "toggle", "");
        // dirty_fields() は state を借用する &[&'static str] を返すため
        // measure() の戻り値としてそのまま持ち出せない。ここで black_box に
        // 通してから破棄し、呼び出し自体の最適化除去を避ける。
        black_box(state.dirty_fields());
    });

    ScenarioReport {
        scenario: "disclosure-toggle",
        full_rerender_ns,
        dirty_update_ns,
    }
}

/// [`SingleSelect`] の `"select"` dispatch について全再描画 vs 差分更新を計測する。
#[must_use]
pub fn bench_single_select_select() -> ScenarioReport {
    let full_rerender_ns = measure(|| {
        let mut state = SingleSelect::default();
        dispatch(&mut state, "select", "panel-1");
        render(&state.view())
    });

    let dirty_update_ns = measure(|| {
        let mut state = SingleSelect::default();
        dispatch(&mut state, "select", "panel-1");
        // dirty_fields() は state を借用する &[&'static str] を返すため
        // measure() の戻り値としてそのまま持ち出せない。ここで black_box に
        // 通してから破棄し、呼び出し自体の最適化除去を避ける。
        black_box(state.dirty_fields());
    });

    ScenarioReport {
        scenario: "single-select-select",
        full_rerender_ns,
        dirty_update_ns,
    }
}

/// 全シナリオを計測順（宣言順）に実行する。`main.rs::run_bench_binding_update`
/// が本関数の戻り値を 1 行ずつ表示する。
#[must_use]
pub fn run_all_scenarios() -> Vec<ScenarioReport> {
    vec![
        bench_appstate_increment(),
        bench_disclosure_toggle(),
        bench_single_select_select(),
    ]
}

/// `f` を [`WARMUP_ITERATIONS`] 回実行した後、[`ITERATIONS`] 回計測し
/// 1 回あたりの平均所要時間（ナノ秒、`f64`）を返す（内部ヘルパ）。
///
/// 平均は合計経過時間（`Duration::as_nanos()` を `f64` へ変換）を
/// `ITERATIONS` の `f64` で割って求める。`Duration` 型のまま整数除算すると
/// サブ ns の平均が `0ns` へ切り捨てられ、[`ScenarioReport::speedup_ratio`] が
/// 実際には精度不足なだけの計測を「無限大の高速化」と誤判定する
/// （イシュー #592 PR #623 レビュー指摘）。`f` の戻り値は
/// [`std::hint::black_box`] へ通してから破棄し、リリースビルドで dirty パス
/// 自体がコンパイラに最適化除去されることを避ける。
fn measure<T>(mut f: impl FnMut() -> T) -> f64 {
    for _ in 0..WARMUP_ITERATIONS {
        black_box(f());
    }
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        black_box(f());
    }
    start.elapsed().as_nanos() as f64 / f64::from(ITERATIONS)
}

#[cfg(test)]
mod tests {
    use super::*;

    // report-only（しきい値判定なし）のため、計測値そのものは検証しない。
    // シナリオ名・シナリオ数（呼び出し契約）のみを固定する。

    #[test]
    fn run_all_scenarios_returns_three_reports_with_expected_names() {
        let reports = run_all_scenarios();
        let names: Vec<&str> = reports.iter().map(|r| r.scenario).collect();
        assert_eq!(
            names,
            vec![
                "appstate-increment",
                "disclosure-toggle",
                "single-select-select",
            ]
        );
    }

    #[test]
    fn scenario_report_display_matches_expected_format() {
        let report = ScenarioReport {
            scenario: "example",
            full_rerender_ns: 1000.0,
            dirty_update_ns: 100.0,
        };
        assert_eq!(
            report.to_string(),
            "bench-binding-update: scenario=example full_ns=1000.00 dirty_ns=100.00 ratio=10.00"
        );
    }

    #[test]
    fn scenario_report_display_does_not_truncate_sub_nanosecond_average() {
        // 整数 Duration 除算では 0ns に切り捨てられていたサブ ns 平均
        // （例: 10000 反復で合計 5000ns → 1 回あたり 0.5ns）が `f64` 保持後は
        // 保持され、`speedup_ratio` が誤って `INFINITY` を返さないことを固定する
        // （イシュー #592 PR #623 レビュー指摘の回帰防止）。
        let report = ScenarioReport {
            scenario: "example",
            full_rerender_ns: 1000.0,
            dirty_update_ns: 0.5,
        };
        assert_eq!(
            report.to_string(),
            "bench-binding-update: scenario=example full_ns=1000.00 dirty_ns=0.50 ratio=2000.00"
        );
        assert!(report.speedup_ratio().is_finite());
    }

    #[test]
    fn scenario_report_speedup_ratio_does_not_panic_on_zero_dirty_duration() {
        let report = ScenarioReport {
            scenario: "example",
            full_rerender_ns: 1000.0,
            dirty_update_ns: 0.0,
        };
        assert_eq!(report.speedup_ratio(), f64::INFINITY);
    }
}
