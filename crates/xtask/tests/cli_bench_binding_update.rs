//! `xtask bench-binding-update` の CLI 契約に対する回帰テスト（イシュー #592）。
//!
//! 本サブコマンドは実行環境依存の計測値を出力する report-only ハーネスであり、
//! CI ゲート化はしない（`bench_binding_update` モジュール doc 参照）。そのため
//! 本テストは**出力形式のみ**を検証し、数値そのもの（`full_ns`/`dirty_ns`/`ratio`）
//! は検証しない（環境依存のため、数値検証は偽陽性・偽陰性の温床になる）。
//!
//! 契約（`xtask/src/main.rs` の `run_bench_binding_update` /
//! `bench_binding_update::ScenarioReport` の `Display` 実装参照）:
//! - 終了コード 0: 常に（計測自体が実行できたことのみを保証。しきい値判定なし）
//! - 終了コード 2: 引数不備（本サブコマンドは引数を一切取らない）
//! - stdout の 1 行サマリ書式は
//!   `bench-binding-update: scenario=<name> full_ns=<n> dirty_ns=<n> ratio=<x.xx>`
//!   （`grep '^bench-binding-update:'` で抽出可能）。シナリオは
//!   `appstate-increment` / `disclosure-toggle` / `single-select-select` の
//!   3 件（`bench_binding_update::run_all_scenarios` の宣言順）。

use std::process::{Command, Output};

fn run_bench_binding_update(extra_args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("bench-binding-update")
        .args(extra_args)
        .output()
        .expect("xtask バイナリの起動に失敗した")
}

#[test]
fn bench_binding_update_exits_zero_and_reports_all_scenarios() {
    let output = run_bench_binding_update(&[]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(
        output.status.code(),
        Some(0),
        "report-only のため常に終了コード 0 のはず。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    for scenario in [
        "appstate-increment",
        "disclosure-toggle",
        "single-select-select",
    ] {
        assert!(
            stdout.lines().any(|line| {
                line.starts_with("bench-binding-update: ")
                    && line.contains(&format!("scenario={scenario}"))
                    && line.contains("full_ns=")
                    && line.contains("dirty_ns=")
                    && line.contains("ratio=")
            }),
            "シナリオ `{scenario}` の 1 行サマリが stdout に見つからない: {stdout}"
        );
    }

    assert_eq!(
        stdout
            .lines()
            .filter(|l| l.starts_with("bench-binding-update: "))
            .count(),
        3,
        "シナリオ 3 件分のサマリが出力されるはず: {stdout}"
    );
}

#[test]
fn bench_binding_update_with_unknown_argument_exits_two() {
    let output = run_bench_binding_update(&["--scenario", "foo"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "bench-binding-update は引数を一切取らない契約（不明な引数は usage エラー）"
    );
}
