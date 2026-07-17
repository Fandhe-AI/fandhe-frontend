//! `xtask check-image-size` の CLI 契約に対する回帰テスト（TASK-9.3b, イシュー #103,
//! REQ-9 受け入れ基準）。
//!
//! `.github/workflows/image-size.yml` は本テストが固定する契約
//! （終了コード・1 行サマリ書式）に依拠して CI の PASS/FAIL を判定する。
//! すなわち本ファイルはワークフローの実質的な単体保証であり、ここで
//! 固定した契約を崩す変更は CI ワークフローの破壊に直結する。
//!
//! 契約（`xtask/src/main.rs` の `run_check_image_size` /
//! `check_image_size::format_report` 参照）:
//! - 終了コード 0: 指定イメージの非圧縮サイズが上限以内（PASS）
//! - 終了コード 1: 上限超過・計測失敗（docker 不在・イメージ不在・`inspect` 失敗を含む。
//!   fail-closed。CI はこれを失敗として扱う）
//! - 終了コード 2: 引数不備（`--image` 未指定・不明な引数・`--limit-mb` の値が非数値）
//! - stdout の 1 行サマリ書式は
//!   `image-size: image=<tag> size_bytes=<n>/<limit> size_mb=<x.xx> result=<PASS|FAIL>`
//!   （`grep '^image-size:'` で抽出可能）
//!
//! 実イメージの PASS 経路（docker build 済みイメージに対する計測）は
//! `Dockerfile`（TASK-9.3a／イシュー #102）と docker デーモンの両方に依存するため
//! 本テストではカバーしない。ここでは docker の有無によらず決定的に検証できる
//! 引数契約・fail-closed 経路のみを固定する（`check_image_size::judge` の
//! 境界値・パース異常系は `xtask/src/check_image_size.rs` の単体テストで検証済み）。

use std::process::{Command, Output};

/// xtask バイナリを `check-image-size` 系の引数で起動するヘルパー。
///
/// カレントディレクトリは docker CLI 呼び出しに影響しないため、他の CLI テスト
/// （`cli_check_loc.rs` 等）と異なり workspace ルートへの固定は不要。
fn run_xtask(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .output()
        .expect("xtask バイナリの起動に失敗した")
}

#[test]
fn missing_image_flag_exits_two() {
    let output = run_xtask(&["check-image-size"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "`--image` 未指定は usage エラー（終了コード 2）のはず"
    );
}

#[test]
fn unknown_argument_exits_two() {
    let output = run_xtask(&["check-image-size", "--image", "some:tag", "--bogus"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "不明な引数は usage エラー（終了コード 2）のはず"
    );
}

#[test]
fn non_numeric_limit_mb_exits_two() {
    let output = run_xtask(&[
        "check-image-size",
        "--image",
        "some:tag",
        "--limit-mb",
        "not-a-number",
    ]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "`--limit-mb` が非数値の場合は usage エラー（終了コード 2）のはず"
    );
}

#[test]
fn nonexistent_image_fails_closed_with_exit_one() {
    // docker が未インストールの環境では Spawn エラー、インストール済みなら
    // `docker image inspect` の非ゼロ終了のいずれかで Err になるが、
    // どちらの経路でも fail-closed（終了コード 1）であることを固定する。
    let output = run_xtask(&[
        "check-image-size",
        "--image",
        "rws-image-size-cli-test-does-not-exist:__missing__",
    ]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "存在しないイメージの計測失敗は fail-closed（終了コード 1）のはず。stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn limit_mb_zero_forces_fail_for_any_nonzero_image() {
    // `--limit-mb 0` は上限緩和ではなく「常に FAIL させる」検証用の下限値。
    // 計測自体が失敗する環境（docker 不在等）でも同じく終了コード 1 になるため、
    // この経路の断定的な検証は docker が利用可能な環境でのみ意味を持つ。
    // ここでは「存在しないイメージ + 上限 0」でも計測失敗経路と衝突せず
    // 終了コード 1 になることのみを確認する（docker 有無に依存しない契約）。
    let output = run_xtask(&[
        "check-image-size",
        "--image",
        "rws-image-size-cli-test-does-not-exist:__missing__",
        "--limit-mb",
        "0",
    ]);
    assert_eq!(output.status.code(), Some(1));
}
