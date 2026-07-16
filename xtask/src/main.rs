//! xtask — 開発者用 CI 計測タスクのエントリポイント。配布物（rws-* クレート）には含めない。
//!
//! サブコマンド:
//! - `check-deps`: REQ-3（依存グラフ上限 60 件/深さ 6）の判定を行い、TASK-3.1c の CI が
//!   終了コードで PASS/FAIL を判定できるようにする（`check_deps` モジュール参照）。

// REQ-2 / core/tests/unsafe_boundary.rs: xtask は WASM/FFI 境界クレートの
// 許可リストに含まれない safe 域クレートのため、unsafe を一切使用しない。
#![forbid(unsafe_code)]

mod check_deps;

use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("check-deps") => run_check_deps(),
        Some(other) => {
            eprintln!("xtask: unknown subcommand \"{other}\"");
            eprintln!("usage: cargo run -p xtask -- check-deps");
            ExitCode::FAILURE
        }
        None => {
            eprintln!("usage: cargo run -p xtask -- check-deps");
            ExitCode::FAILURE
        }
    }
}

/// `check-deps` サブコマンドの本体。
///
/// TASK-3.1a（`cargo metadata` 解析による実測値の計測、#16）が本ブランチ作成時点で
/// 未マージのため、実測値の収集手段を持たない。fail-closed 方針（security.md /
/// このタスクの実装計画 §3-2）に従い、「計測できない」を「PASS 相当でスキップ」に
/// 倒さず、非ゼロ終了・明示的なエラーメッセージで停止する。
///
/// 3.1a マージ後は、この関数の内部を `cargo metadata --locked --format-version 1` の
/// 実行・解析結果から `check_deps::DepsMetrics` を構築する実装に置き換え、
/// `check_deps::judge` 以降の結線（このまま流用可能）へつなぐ。
fn run_check_deps() -> ExitCode {
    match collect_metrics() {
        Ok(metrics) => {
            let result = check_deps::judge(metrics);
            print!("{}", check_deps::format_report(&result));
            if result.is_pass() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(message) => {
            eprintln!("xtask: check-deps failed: {message}");
            ExitCode::FAILURE
        }
    }
}

/// 依存グラフの実測値を収集する。
///
/// 現時点では TASK-3.1a（#16）の計測ロジックが未実装のため、`cargo metadata` の
/// 実行・解析は行わずエラーを返す（fail-closed）。3.1a マージ後、ここを実装で
/// 置き換えて `check_deps::DepsMetrics` を返すようにする。
fn collect_metrics() -> Result<check_deps::DepsMetrics, String> {
    Err(
        "dependency graph measurement is not implemented yet (see TASK-3.1a / issue #16); \
         refusing to report a result rather than skipping the check"
            .to_string(),
    )
}
