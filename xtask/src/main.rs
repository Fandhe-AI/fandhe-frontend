//! `xtask`: このワークスペースの CI 計測・自己保守用ツール群のエントリポイント。
//!
//! `cargo run -p xtask -- <subcommand>` で起動する。REQ-3（依存グラフ上限:
//! 60 件以内・深さ 6 以内、`docs/spec/04-requirements.md`）に関する計測は
//! `check_deps` モジュールが担う。しきい値判定・CI 失敗化は TASK-3.1b/c（#17/#18）で
//! 本ファイルのサブコマンド dispatch に積み増される想定。
//!
//! `core` / `interactive` と異なりプロセス起動（`std::process::Command`）を行うが、
//! `unsafe` は使わない（REQ-2 は core/interactive 限定だが、xtask でも forbid する）。

#![forbid(unsafe_code)]

mod check_deps;
mod json;

use check_deps::measure_from_cargo_metadata;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("check-deps") => run_check_deps(&args[2..]),
        Some(other) => {
            eprintln!("xtask: unknown subcommand `{other}`");
            print_usage();
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
    eprintln!("      Measure resolved dependency count and max depth for each package.");
    eprintln!("      (TASK-3.1a: measurement only; threshold enforcement is a follow-up task.)");
}

/// `check-deps` サブコマンド: `--package <NAME>` を 1 つ以上受け取り、
/// それぞれについて依存件数・最大深さを計測して stdout に表示する。
///
/// しきい値判定は行わない（TASK-3.1a のスコープ外、#17 が積み増す）。
/// 引数不備・計測失敗時は終了コード 1 を返す。
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

    let mut had_error = false;
    for name in &packages {
        match measure_from_cargo_metadata(name) {
            Ok(m) => {
                println!(
                    "package {}: {} dependencies, max depth {}",
                    m.root, m.package_count, m.max_depth
                );
            }
            Err(e) => {
                eprintln!("xtask check-deps: failed to measure `{name}`: {e}");
                had_error = true;
            }
        }
    }

    if had_error {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
