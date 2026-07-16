//! `xtask`: このワークスペースの CI 計測・自己保守用ツール群のエントリポイント。
//! 開発者用ツールであり、配布物（rws-* クレート）には含めない。
//!
//! サブコマンド:
//! - `check-deps --package <NAME> [--package <NAME> ...]`: REQ-3（依存グラフ上限
//!   60 件以内・深さ 6 以内、`docs/spec/04-requirements.md`）の実測（`check_deps`
//!   モジュールの TASK-3.1a 計測ロジック）と判定（TASK-3.1b の `judge`）を行い、
//!   TASK-3.1c の CI（`.github/workflows/deps-check.yml`）が終了コードと
//!   1 行サマリ（`check_deps::format_report` 参照）で PASS/FAIL を判定できるようにする。
//!   CLI 契約の回帰テストは `xtask/tests/cli_check_deps.rs`。
//!
//! 複数 `--package` 指定時も `cargo metadata` は 1 回のみ実行する
//! （`check_deps::measure_many_from_cargo_metadata` 参照。Bugbot 指摘
//! 「metadata rerun per package」への対応）。
//!
//! `core` / `interactive` と異なりプロセス起動（`std::process::Command`）を行うが、
//! `unsafe` は使わない（REQ-2 は core/interactive 限定だが、xtask でも forbid する。
//! core/tests/unsafe_boundary.rs の WASM/FFI 境界許可リストにも含まれない）。

#![forbid(unsafe_code)]

mod check_deps;
mod json;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("check-deps") => run_check_deps(&args[2..]),
        Some(other) => {
            eprintln!("xtask: unknown subcommand `{other}`");
            print_usage();
            // `check-deps` の引数不備（`--package` 不足等）は終了コード 2 を
            // 返す契約になっており、サブコマンド自体が不明・未指定の場合も
            // 同じ「usage エラー」区分として揃える。ExitCode::FAILURE (1) の
            // ままだと呼び出し元が判定失敗（1）と usage エラー（2）を区別
            // できない（Bugbot 指摘: wrong exit code for usage errors）。
            ExitCode::from(2)
        }
        None => {
            print_usage();
            ExitCode::from(2)
        }
    }
}

fn print_usage() {
    eprintln!("Usage: xtask <subcommand> [options]");
    eprintln!();
    eprintln!("Subcommands:");
    eprintln!("  check-deps --package <NAME> [--package <NAME> ...]");
    eprintln!("      Measure resolved dependency count and max depth for each package");
    eprintln!("      and judge them against the REQ-3 limits (60 packages / depth 6).");
}

/// `check-deps` サブコマンド: `--package <NAME>` を 1 つ以上受け取り、
/// それぞれについて依存件数・最大深さを計測して上限判定し、結果を stdout に表示する。
///
/// `cargo metadata` は全パッケージ分をまとめて 1 回だけ実行する
/// （[`check_deps::measure_many_from_cargo_metadata`]）。
/// 引数不備・計測失敗・上限超過（`CheckResult::Fail`）のいずれかがあれば
/// 終了コード 1（fail-closed）を返す。
fn run_check_deps(args: &[String]) -> ExitCode {
    let mut packages = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--package" => {
                let Some(name) = args.get(i + 1) else {
                    eprintln!("xtask check-deps: `--package` requires a value");
                    return ExitCode::from(2);
                };
                packages.push(name.clone());
                i += 2;
            }
            other => {
                eprintln!("xtask check-deps: unknown argument `{other}`");
                return ExitCode::from(2);
            }
        }
    }

    if packages.is_empty() {
        eprintln!("xtask check-deps: at least one `--package <NAME>` is required");
        return ExitCode::from(2);
    }

    let results = match check_deps::measure_many_from_cargo_metadata(&packages) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("xtask check-deps: failed to run cargo metadata: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut had_failure = false;
    for (name, measurement) in results {
        match measurement {
            Ok(m) => {
                let check_result = check_deps::judge(m.into());
                print!("{}", check_deps::format_report(&check_result));
                if !check_result.is_pass() {
                    had_failure = true;
                }
            }
            Err(e) => {
                eprintln!("xtask check-deps: failed to measure `{name}`: {e}");
                had_failure = true;
            }
        }
    }

    if had_failure {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
