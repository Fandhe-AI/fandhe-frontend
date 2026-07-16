//! xtask::check_deps — REQ-3（依存グラフの浅さ・監査可能性）を CI で機械的に強制する判定・レポート層。
//!
//! TASK-3.1 は 3 段階に分割されている:
//! - TASK-3.1a: `cargo metadata` の解析による実測値（パッケージ数・依存グラフ最大深さ）の計測ロジック
//! - TASK-3.1b（本モジュール）: 実測値に対する上限判定（60 件/深さ 6）とレポート出力・終了コード制御
//! - TASK-3.1c: CI ワークフローへの組み込み
//!
//! 3.1a の成果物が未マージのため、本モジュールは計測結果を表す `DepsMetrics` を
//! 純粋なデータとして受け取り判定する形にとどめ、`cargo metadata` の実行・解析には関与しない。
//! 3.1a マージ後は `collect_metrics`（xtask/src/main.rs 参照）を実装で置き換えて本モジュールへ結線する。

use std::fmt;

/// フレームワーク標準構成で許容する解決済み依存パッケージ数の上限。
///
/// PoC-3 実測（純 Rust 方式・`rws-server` 相当構成: 52 件）を基準に、
/// 実装拡張分の余裕を含めて設定する（REQ-3 / docs/spec/04-requirements.md 59 行目）。
/// 上限緩和のための CLI 引数・環境変数は意図的に設けない
/// （coding-rust.md「依存グラフ上限を弱めない」/ security.md 参照）。
pub const MAX_PACKAGES: usize = 60;

/// フレームワーク標準構成で許容する依存グラフ最大深さの上限。
///
/// PoC-3 実測（純 Rust 方式・`rws-server` 相当構成: 深さ 5）を基準に、
/// 実装拡張分の余裕を含めて設定する（REQ-3 / docs/spec/04-requirements.md 59 行目）。
pub const MAX_DEPTH: usize = 6;

/// TASK-3.1a（`cargo metadata` 解析）が計測した依存グラフの実測値。
///
/// この構造体は 3.1a と 3.1b の間の契約であり、`collect_metrics`（3.1a 側の実装）が
/// 生成し、本モジュールの `judge` が消費する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepsMetrics {
    /// 判定対象の構成名（レポート表示用。例: "standard-server"）。
    pub target: String,
    /// `cargo metadata` で解決された依存パッケージ数（対象自身を除く）。
    pub package_count: usize,
    /// 依存グラフの最大深さ（対象パッケージを深さ 0 とする）。
    pub max_depth: usize,
}

/// 上限超過の内訳。`judge` が `CheckResult::Fail` に含める違反リスト。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Violation {
    /// 依存パッケージ数が `MAX_PACKAGES` を超過。
    PackageCount { actual: usize, limit: usize },
    /// 依存グラフ最大深さが `MAX_DEPTH` を超過。
    MaxDepth { actual: usize, limit: usize },
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Violation::PackageCount { actual, limit } => {
                write!(f, "package count {actual} exceeds limit {limit}")
            }
            Violation::MaxDepth { actual, limit } => {
                write!(f, "dependency graph depth {actual} exceeds limit {limit}")
            }
        }
    }
}

/// 上限判定の結果。CI（TASK-3.1c）の終了コード制御はこの値の可否を fail-closed で反映する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckResult {
    /// 実測値がすべて上限内。
    Pass(DepsMetrics),
    /// いずれかの上限を超過。内側の `Vec<Violation>` は空にならない。
    Fail(DepsMetrics, Vec<Violation>),
}

impl CheckResult {
    /// CI（3.1c）が終了コードを決定する際に参照する契約:
    /// `Pass` のみ成功（終了コード 0）、それ以外は失敗として扱う。
    pub fn is_pass(&self) -> bool {
        matches!(self, CheckResult::Pass(_))
    }
}

/// 実測値 `metrics` を上限（`MAX_PACKAGES` / `MAX_DEPTH`）に照らして判定する純粋関数。
///
/// I/O を一切行わないため単体テストで境界値を網羅できる。
/// rws-server（将来の標準サーバー構成）を計測対象として想定するが、
/// 本関数自体は対象の種類を問わず `DepsMetrics` の値のみで判定する。
pub fn judge(metrics: DepsMetrics) -> CheckResult {
    let mut violations = Vec::new();

    if metrics.package_count > MAX_PACKAGES {
        violations.push(Violation::PackageCount {
            actual: metrics.package_count,
            limit: MAX_PACKAGES,
        });
    }
    if metrics.max_depth > MAX_DEPTH {
        violations.push(Violation::MaxDepth {
            actual: metrics.max_depth,
            limit: MAX_DEPTH,
        });
    }

    if violations.is_empty() {
        CheckResult::Pass(metrics)
    } else {
        CheckResult::Fail(metrics, violations)
    }
}

/// 人間可読なサマリと、CI ログから機械抽出可能な 1 行サマリを整形する。
///
/// 1 行サマリの書式（`deps-check: packages=<n>/<limit> depth=<n>/<limit> result=<PASS|FAIL>`）は
/// TASK-3.1c の CI がログから判定結果を抽出する際の契約とみなし、安易に変更しない。
/// ユーザー向け文字列は英語（japanese-style.md: フレームワーク成果物は国際利用を想定）。
pub fn format_report(result: &CheckResult) -> String {
    let (metrics, violations, verdict): (&DepsMetrics, &[Violation], &str) = match result {
        CheckResult::Pass(metrics) => (metrics, &[], "PASS"),
        CheckResult::Fail(metrics, violations) => (metrics, violations.as_slice(), "FAIL"),
    };

    let mut out = String::new();
    out.push_str(&format!(
        "Dependency graph check for target \"{}\"\n",
        metrics.target
    ));
    out.push_str(&format!(
        "  packages: {} (limit {})\n",
        metrics.package_count, MAX_PACKAGES
    ));
    out.push_str(&format!(
        "  max depth: {} (limit {})\n",
        metrics.max_depth, MAX_DEPTH
    ));
    if violations.is_empty() {
        out.push_str("  result: PASS\n");
    } else {
        out.push_str("  result: FAIL\n");
        for violation in violations {
            out.push_str(&format!("  violation: {violation}\n"));
        }
    }

    out.push_str(&format!(
        "deps-check: packages={}/{} depth={}/{} result={}\n",
        metrics.package_count, MAX_PACKAGES, metrics.max_depth, MAX_DEPTH, verdict
    ));

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(package_count: usize, max_depth: usize) -> DepsMetrics {
        DepsMetrics {
            target: "standard-server".to_string(),
            package_count,
            max_depth,
        }
    }

    #[test]
    fn judge_passes_at_exact_limits() {
        let result = judge(metrics(MAX_PACKAGES, MAX_DEPTH));
        assert!(result.is_pass());
    }

    #[test]
    fn judge_fails_when_package_count_exceeds_limit() {
        let result = judge(metrics(MAX_PACKAGES + 1, MAX_DEPTH));
        match result {
            CheckResult::Fail(_, violations) => {
                assert_eq!(violations.len(), 1);
                assert!(matches!(violations[0], Violation::PackageCount { .. }));
            }
            CheckResult::Pass(_) => panic!("expected Fail"),
        }
    }

    #[test]
    fn judge_fails_when_depth_exceeds_limit() {
        let result = judge(metrics(MAX_PACKAGES, MAX_DEPTH + 1));
        match result {
            CheckResult::Fail(_, violations) => {
                assert_eq!(violations.len(), 1);
                assert!(matches!(violations[0], Violation::MaxDepth { .. }));
            }
            CheckResult::Pass(_) => panic!("expected Fail"),
        }
    }

    #[test]
    fn judge_reports_both_violations_when_both_exceed_limit() {
        let result = judge(metrics(MAX_PACKAGES + 10, MAX_DEPTH + 2));
        match result {
            CheckResult::Fail(_, violations) => {
                assert_eq!(violations.len(), 2);
            }
            CheckResult::Pass(_) => panic!("expected Fail"),
        }
    }

    #[test]
    fn judge_passes_with_headroom() {
        let result = judge(metrics(0, 0));
        assert!(result.is_pass());
    }

    #[test]
    fn format_report_pass_contains_machine_readable_summary_line() {
        let result = judge(metrics(52, 5));
        let report = format_report(&result);
        assert!(report.contains("deps-check: packages=52/60 depth=5/6 result=PASS"));
    }

    #[test]
    fn format_report_fail_contains_violation_lines_and_summary() {
        let result = judge(metrics(61, 7));
        let report = format_report(&result);
        assert!(report.contains("deps-check: packages=61/60 depth=7/6 result=FAIL"));
        assert!(report.contains("violation: package count 61 exceeds limit 60"));
        assert!(report.contains("violation: dependency graph depth 7 exceeds limit 6"));
    }
}
