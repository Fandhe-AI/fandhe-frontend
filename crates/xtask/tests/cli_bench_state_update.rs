//! `xtask bench-state-update` の CLI 契約に対する回帰テスト（イシュー #1328）。
//!
//! 本サブコマンドは実行環境依存の計測値を出力する report-only ハーネスであり、
//! 性能数値そのもの（`mean_us`/`p50_us`/`p95_us`/`min_us`）は環境依存のため
//! 検証しない（`cli_bench_ssr.rs`/`cli_bench_binding_update.rs` の既存 CLI
//! 契約テストと同じ判断軸）。本テストは**形式・契約**のみを検証する:
//! - 終了コード 0: 検証（`escape_ok`/`noop_ok`）PASS
//! - 終了コード 1: 検証 FAIL・環境エラー（`--baseline` 読み取り/パース失敗を含む）
//! - 終了コード 2: 引数不備
//! - stdout に JSON 1 行（`framework`/`version`/`mode`/`bindings`/`grid1k`/
//!   `appstate1k`/`escape_ok`/`noop_ok`/`notes` を含む）
//! - `--baseline <FILE>` 指定時は `bench-state-update-compare:` 行群が
//!   追加出力される
//!
//! フィクスチャ（baseline ファイル）は `CARGO_TARGET_TMPDIR`（イシュー #637 の
//! 一時領域配置規約）配下に一意名で生成する。

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

fn xtask_bin() -> &'static str {
    env!("CARGO_BIN_EXE_xtask")
}

fn run_bench_state_update(extra_args: &[&str]) -> Output {
    Command::new(xtask_bin())
        .arg("bench-state-update")
        .args(extra_args)
        .output()
        .expect("xtask バイナリの起動に失敗した")
}

fn scratch_root() -> PathBuf {
    let root = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
    // cargo は `CARGO_TARGET_TMPDIR` の実在を保証しない（イシュー #637）ため、
    // 呼び出し側で作成する（`cli_bench_ssr.rs::scratch_root` と同一方針）。
    let _ = std::fs::create_dir_all(&root);
    root
}

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// `CARGO_TARGET_TMPDIR` 配下に一意名のフィクスチャファイルパスを組み立てる
/// （まだ作成はしない。呼び出し元が内容を書き込む）。
fn unique_fixture_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = FIXTURE_COUNTER.fetch_add(1, Ordering::SeqCst);
    scratch_root().join(format!(
        "xtask-bench-state-update-{label}-{}-{nanos}-{n}.json",
        std::process::id()
    ))
}

#[test]
fn bench_state_update_exits_zero_and_reports_expected_json_shape() {
    let output = run_bench_state_update(&[]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        output.status.code(),
        Some(0),
        "自己検証（escape_ok/noop_ok）は常に PASS のはず。stdout: {stdout}\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let json_lines: Vec<&str> = stdout.lines().filter(|l| l.starts_with('{')).collect();
    assert_eq!(
        json_lines.len(),
        1,
        "JSON 行はちょうど 1 行のはず: {stdout}"
    );
    let line = json_lines[0];

    for expected in [
        "\"framework\":\"fandhe-frontend\"",
        "\"mode\":\"state-update\"",
        "\"bindings\":1000",
        "\"grid1k\":",
        "\"appstate1k\":",
        "\"update\":",
        "\"binding_apply\":",
        "\"render\":",
        "\"noop_update\":",
        "\"escape_ok\":true",
        "\"noop_ok\":true",
        "\"iters\":",
        "\"mean_us\":",
        "\"p50_us\":",
        "\"p95_us\":",
        "\"min_us\":",
        "\"notes\":\"profile=",
        "\"version\":\"",
    ] {
        assert!(
            line.contains(expected),
            "JSON 行に `{expected}` が含まれていない: {line}"
        );
    }
}

#[test]
fn bench_state_update_with_unknown_argument_exits_two() {
    let output = run_bench_state_update(&["--bindings", "5"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "未知の引数は usage エラー（終了コード 2）のはず"
    );
}

#[test]
fn bench_state_update_with_baseline_missing_value_exits_two() {
    let output = run_bench_state_update(&["--baseline"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "`--baseline` に値がない場合は usage エラー（終了コード 2）のはず"
    );
}

#[test]
fn bench_state_update_with_invalid_baseline_json_exits_one() {
    let baseline_path = unique_fixture_path("invalid-json");
    fs::write(&baseline_path, "not json").expect("フィクスチャ書き込みに失敗した");

    let baseline_arg = baseline_path.to_string_lossy().into_owned();
    let output = run_bench_state_update(&["--baseline", &baseline_arg]);

    assert_eq!(
        output.status.code(),
        Some(1),
        "不正な JSON baseline は終了コード 1（環境エラー扱い）のはず"
    );

    let _ = fs::remove_file(&baseline_path);
}

#[test]
fn bench_state_update_with_baseline_missing_file_exits_one() {
    let baseline_path = unique_fixture_path("missing-file");
    let baseline_arg = baseline_path.to_string_lossy().into_owned();

    let output = run_bench_state_update(&["--baseline", &baseline_arg]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "baseline ファイル不在は終了コード 1 のはず"
    );
}

#[test]
fn bench_state_update_with_valid_baseline_emits_compare_lines_and_exits_zero() {
    // 1 回目の実行結果を baseline として保存する。
    let first = run_bench_state_update(&[]);
    assert_eq!(first.status.code(), Some(0));
    let first_stdout = String::from_utf8_lossy(&first.stdout);
    let json_line = first_stdout
        .lines()
        .find(|l| l.starts_with('{'))
        .expect("1 回目の実行に JSON 行があるはず");

    let baseline_path = unique_fixture_path("valid");
    fs::write(&baseline_path, json_line).expect("baseline フィクスチャ書き込みに失敗した");

    let baseline_arg = baseline_path.to_string_lossy().into_owned();
    let second = run_bench_state_update(&["--baseline", &baseline_arg]);
    let second_stdout = String::from_utf8_lossy(&second.stdout);

    assert_eq!(
        second.status.code(),
        Some(0),
        "有効な baseline 指定時も検証 PASS なら終了コード 0 のはず。stdout: {second_stdout}"
    );

    let compare_lines: Vec<&str> = second_stdout
        .lines()
        .filter(|l| l.starts_with("bench-state-update-compare: "))
        .collect();
    // 2 シナリオ（grid1k/appstate1k）× 4 フェーズ（update/binding_apply/render/
    // noop_update）× 4 指標（mean_us/p50_us/p95_us/min_us）= 32 行。
    assert_eq!(
        compare_lines.len(),
        32,
        "32 指標分の比較行が出力されるはず: {second_stdout}"
    );
    for line in &compare_lines {
        assert!(line.contains("metric="));
        assert!(line.contains("baseline="));
        assert!(line.contains("current="));
        assert!(line.contains("delta_pct="));
    }

    let _ = fs::remove_file(&baseline_path);
}
