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
//! - `list-build-scripts --package <NAME> [--package <NAME> ...]`: REQ-3
//!   （サプライチェーン監査可能性、PoC-2 脅威モデル）のうち `build.rs` 保有クレートの
//!   機械的列挙（TASK-3.2a、`list_build_scripts` モジュール）。CI ワークフロー
//!   （`.github/workflows/deps-check.yml`）への組み込みは TASK-3.2b（イシュー #21）で、
//!   1 行サマリ（`list_build_scripts::format_inventory` 参照）を Step Summary に
//!   転記する。CLI 契約の回帰テストは `xtask/tests/cli_list_build_scripts.rs`。
//!
//! - `check-core-deps`（引数なし）: イシュー #154（REQ-3 受け入れ基準 1）。
//!   `check_deps::ZERO_DEP_CRATES` と実 workspace メンバーの積集合について
//!   `Normal`/`Dev`/`Build` すべての辺を辿り、workspace 内の第一者パッケージ
//!   （path dependency）を除いた「真の外部依存」が 1 件でも存在しないかを
//!   `check_deps::measure_external_only` で計測し判定する（`check_deps::judge_zero`）。
//!   `check-deps --package rws-core` の 60/6 判定とは別に「ゼロであること」を
//!   専用ゲートとして強制する。判定対象は CLI 引数で差し替え不可
//!   （`ZERO_DEP_CRATES` 参照。上限を弱める経路を作らない設計）。
//!   CLI 契約の回帰テストは `xtask/tests/cli_check_core_deps.rs`。
//!
//! `core` / `interactive` と異なりプロセス起動（`std::process::Command`）を行うが、
//! `unsafe` は使わない（REQ-2 は core/interactive 限定だが、xtask でも forbid する。
//! core/tests/unsafe_boundary.rs の WASM/FFI 境界許可リストにも含まれない）。

#![forbid(unsafe_code)]

mod check_deps;
mod json;
mod list_build_scripts;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("check-deps") => run_check_deps(&args[2..]),
        Some("list-build-scripts") => run_list_build_scripts(&args[2..]),
        Some("check-core-deps") => run_check_core_deps(&args[2..]),
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
    eprintln!("  list-build-scripts --package <NAME> [--package <NAME> ...]");
    eprintln!("      List crates with a custom build script (build.rs) reachable from");
    eprintln!("      each package (REQ-3 supply-chain audit visibility).");
    eprintln!("  check-core-deps");
    eprintln!("      Enforce zero external dependencies (normal/dev/build) for the core");
    eprintln!("      crates listed in check_deps::ZERO_DEP_CRATES (REQ-3 acceptance");
    eprintln!("      criterion 1, issue #154). Takes no arguments by design.");
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

/// `check-core-deps` サブコマンド（イシュー #154, REQ-3 受け入れ基準 1）: 引数を
/// 一切取らない。`check_deps::ZERO_DEP_CRATES` と実 workspace メンバーの積集合
/// （`check_deps::fetch_zero_dep_targets`）について、それぞれ `Normal`/`Dev`/`Build`
/// すべての辺を辿り、workspace 内の第一者パッケージ（path dependency）を除いた
/// 依存パッケージ数を `check_deps::measure_external_only` で計測し、1 件でも
/// あれば Fail とする（`check_deps::judge_zero`）。
///
/// 判定を弱める CLI 引数・環境変数は意図的に設けない（不明な引数は終了コード 2）。
/// 積集合が空（`ZERO_DEP_CRATES` の定数値が陳腐化し workspace に 1 件も実在しない）
/// 場合・計測失敗・上限超過のいずれも終了コード 1（fail-closed）とする。
fn run_check_core_deps(args: &[String]) -> ExitCode {
    if let Some(unknown) = args.first() {
        eprintln!("xtask check-core-deps: unknown argument `{unknown}` (this subcommand takes no arguments)");
        return ExitCode::from(2);
    }

    let (graph, targets, workspace_members) = match check_deps::fetch_zero_dep_targets() {
        Ok(triple) => triple,
        Err(e) => {
            eprintln!("xtask check-core-deps: failed to resolve zero-dep targets: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut had_failure = false;
    for name in targets {
        let kinds = [
            check_deps::DepKind::Normal,
            check_deps::DepKind::Dev,
            check_deps::DepKind::Build,
        ];
        // 到達可能パッケージから workspace 内の第一者パッケージ（path dependency）を
        // 除外し、真の外部依存のみを数える（reviewer 指摘: イシュー #154）。
        match check_deps::measure_external_only(&graph, &name, &kinds, &workspace_members) {
            Ok(m) => {
                let check_result = check_deps::judge_zero(m.into());
                print!("{}", check_deps::format_zero_report(&check_result));
                if !check_result.is_pass() {
                    had_failure = true;
                }
            }
            Err(e) => {
                eprintln!("xtask check-core-deps: failed to measure `{name}`: {e}");
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

/// `list-build-scripts` サブコマンド: `--package <NAME>` を 1 つ以上受け取り、
/// それぞれについて到達可能な build.rs 保有クレートを列挙して stdout に表示する
/// （TASK-3.2a、`list_build_scripts` モジュール）。
///
/// 列挙自体は上限判定を伴わないため、正常に列挙できれば件数（0 件含む）によらず
/// 終了コード 0 を返す。`cargo metadata` の実行失敗・想定外の出力構造・ルート未検出は
/// fail-closed で終了コード 1（「列挙できなかったのに成功扱い」になる経路を作らない）。
/// 引数不備（`--package` 未指定・不明な引数）は `check-deps` と契約を統一し
/// 終了コード 2 とする。
fn run_list_build_scripts(args: &[String]) -> ExitCode {
    let mut packages = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--package" => {
                let Some(name) = args.get(i + 1) else {
                    eprintln!("xtask list-build-scripts: `--package` requires a value");
                    return ExitCode::from(2);
                };
                packages.push(name.clone());
                i += 2;
            }
            other => {
                eprintln!("xtask list-build-scripts: unknown argument `{other}`");
                return ExitCode::from(2);
            }
        }
    }

    if packages.is_empty() {
        eprintln!("xtask list-build-scripts: at least one `--package <NAME>` is required");
        return ExitCode::from(2);
    }

    let results = match list_build_scripts::list_many_from_cargo_metadata(&packages) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("xtask list-build-scripts: failed to run cargo metadata: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut had_failure = false;
    for (name, inventory) in results {
        match inventory {
            Ok(crates) => {
                print!("{}", list_build_scripts::format_inventory(&name, &crates));
            }
            Err(e) => {
                eprintln!("xtask list-build-scripts: failed to list `{name}`: {e}");
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
