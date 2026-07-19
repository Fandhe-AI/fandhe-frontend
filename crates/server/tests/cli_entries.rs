//! CLI エントリ 2 バイナリ（`fandhe-frontend-server`＝SSR / `ssg`＝SSG）の実プロセス起動
//! 回帰テスト（TASK-6.1c、イシュー #44）。
//!
//! # `three_mode_integration.rs`・`ssr_ssg_parity.rs` との役割分担
//!
//! 既存の統合テストはいずれも `fandhe_frontend_server::ssr::respond` /
//! `fandhe_frontend_server::ssg::generate` を**ライブラリ API として直接呼ぶ**のみで、
//! `server/src/main.rs`（SSR CLI）・`server/src/bin/ssg.rs`（SSG CLI）という
//! 「バイナリクレートルート」を実プロセスとして起動する経路は検証していない
//! （`server/src/bin/ssg.rs` は `parse_out_dir` の unit test のみを持つ）。
//! 本ファイルはその欠落を埋め、**CLI プロセス境界**（引数解析・終了コード・
//! stdout/stderr 形式）のみを担う。ライブラリ API レベルの三モード整合は
//! `three_mode_integration.rs`（#45）・`ssr_ssg_parity.rs`（#50）の責務であり、
//! 重複させない。
//!
//! # セキュリティ不変条件（回帰対象）
//!
//! - REQ-1: XSS ペイロード（`demo_items()` の id "2"）が CLI 経由の stdout /
//!   生成ファイルでも `&lt;script&gt;` として既定エスケープされること。
//! - `security.md`「機微情報の露出」: 未一致パスの stderr が固定文言のみで
//!   あり、内部パス（`Cargo`・`/home/` 等）を含まないこと。
//! - OWASP A01 パストラバーサル: `ssg` バイナリの生成物がすべて `--out` 配下
//!   に限定されること（外形回帰）。

use fandhe_frontend_app::demo_items;
use std::fs;
use std::path::Path;
use std::process::Command;

// `server/src/ssg.rs` の unit test 等と重複実装しない共有ヘルパー。
// `include!` はソースを直接展開するため、integration test クレートからも
// 追加の外部依存（`tempfile` 等）なしに一時ディレクトリを扱える
// （`server/tests/support/temp_dir.rs` 参照、REQ-3）。
include!("support/temp_dir.rs");

/// SSR エントリ（`fandhe-frontend-server` バイナリ）を起動する共通処理。
///
/// `CARGO_BIN_EXE_fandhe-frontend-server` は cargo がテストビルド時に注入する、対象
/// バイナリの絶対パス（`cargo test` の標準機能。追加の実行委譲ライブラリは
/// 使わない）。
fn run_ssr(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_fandhe-frontend-server"))
        .args(args)
        .output()
        .expect("fandhe-frontend-server binary should spawn")
}

/// SSG エントリ（`ssg` バイナリ）を起動する共通処理。
fn run_ssg(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ssg"))
        .args(args)
        .output()
        .expect("ssg binary should spawn")
}

/// 引数なし（既定パス `/`）で起動した SSR エントリが、終了コード 0・
/// `Status:`/`Content-Type:` ヘッダ行・既定エスケープ済み HTML ボディを
/// stdout へ出力すること。
#[test]
fn ssr_entry_default_path_returns_200_html() {
    let output = run_ssr(&[]);
    assert!(output.status.success(), "exit status should be 0 (success)");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    assert!(stdout.contains("Status: 200"));
    assert!(stdout.contains("Content-Type: text/html; charset=utf-8"));
    assert!(stdout.contains("<!DOCTYPE html>"));
}

/// 既知の `id` を指定した詳細ページが 200 で描画されること。
#[test]
fn ssr_entry_known_item_path_returns_detail_page() {
    let output = run_ssr(&["/items/1"]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    assert!(stdout.contains("Status: 200"));
    assert!(stdout.contains("Rust 製フロントエンド基盤の構想"));
}

/// 未知の `id` はルート自体には一致するため 404 応答（`Some`）だが、
/// `main.rs` は `status >= 400` を非 0 終了コードへ変換する契約
/// （`server/src/main.rs` 参照）。
#[test]
fn ssr_entry_unknown_item_id_returns_404_and_nonzero_exit() {
    let output = run_ssr(&["/items/does-not-exist"]);
    assert!(
        !output.status.success(),
        "404 response should map to a non-zero exit code"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    assert!(stdout.contains("Status: 404"));
    assert!(stdout.contains("見つかりません"));
}

/// 固定ルート表に一致しないパスは `respond()` が `None` を返し、`main.rs` は
/// 固定文言のみを stderr へ出力して非 0 終了する（内部パス等の機微情報を
/// 含まないことを固定する、`security.md`）。
#[test]
fn ssr_entry_unmatched_path_fails_with_fixed_stderr_message() {
    let output = run_ssr(&["/nope"]);
    assert!(!output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    assert!(stdout.is_empty(), "no response body should be printed");

    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");
    assert!(stderr.contains("no route matched path"));
    assert!(!stderr.contains("Cargo"));
    assert!(!stderr.contains("/home/"));
    assert!(!stderr.contains("panicked"));
}

/// REQ-1 の CLI 経路回帰: XSS ペイロード id（`demo_items()` の id "2"）の
/// stdout ボディが既定エスケープされ、`<script>` として解釈されないこと。
#[test]
fn ssr_entry_escapes_xss_payload_item() {
    let payload_item = demo_items()
        .into_iter()
        .find(|it| it.id == "2")
        .expect("demo_items() should contain the XSS payload fixture (id \"2\")");

    let output = run_ssr(&[&format!("/items/{}", payload_item.id)]);
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    assert!(stdout.contains("&lt;script&gt;"));
    assert!(!stdout.contains("<script>alert"));
}

/// SSG エントリが `--out <dir>` 配下へ `index.html` と各アイテムの
/// `items/<id>/index.html` を生成し、終了コード 0 で完了すること。
#[test]
fn ssg_entry_writes_index_and_item_pages_under_out_dir() {
    let dir = TempDir::new("cli-ssg-basic");
    let out_dir = dir.0.to_str().expect("temp dir path should be UTF-8");

    let output = run_ssg(&["--out", out_dir]);
    assert!(output.status.success(), "ssg generation should succeed");

    assert!(dir.0.join("index.html").exists());
    for item in demo_items() {
        assert!(dir
            .0
            .join("items")
            .join(&item.id)
            .join("index.html")
            .exists());
    }
}

/// REQ-6 の CLI 経路外形確認: `ssg` バイナリが生成した `index.html` の
/// バイト列が、SSR エントリの stdout ボディ（ヘッダ行を除いた HTML 部分）と
/// 一致すること。
#[test]
fn ssg_entry_index_matches_ssr_entry_body_bytes() {
    let dir = TempDir::new("cli-ssg-parity");
    let out_dir = dir.0.to_str().expect("temp dir path should be UTF-8");

    let ssg_output = run_ssg(&["--out", out_dir]);
    assert!(ssg_output.status.success());
    let ssg_index = fs::read_to_string(dir.0.join("index.html")).expect("index.html should exist");

    let ssr_output = run_ssr(&[]);
    assert!(ssr_output.status.success());
    let ssr_stdout =
        String::from_utf8(ssr_output.stdout).expect("SSR stdout should be valid UTF-8");
    // `main.rs` は `Status:`/`Content-Type:` ヘッダ行 + 空行 + HTML ボディの
    // 順で出力する契約（`server/src/main.rs` 参照）。空行以降がボディ本体。
    let ssr_body = ssr_stdout
        .split_once("\n\n")
        .map(|(_, body)| body)
        .expect("SSR stdout should contain a blank line separating headers from body");

    assert_eq!(ssg_index.trim_end(), ssr_body.trim_end());
}

/// OWASP A01 の CLI 経路外形回帰: `ssg` バイナリの生成物がすべて `--out`
/// 配下に限定され、親ディレクトリへ書き出されないこと。
#[test]
fn ssg_entry_confines_output_to_out_dir() {
    let dir = TempDir::new("cli-ssg-confinement");
    let out_dir = dir.0.to_str().expect("temp dir path should be UTF-8");

    let output = run_ssg(&["--out", out_dir]);
    assert!(output.status.success());

    let canonical_out = fs::canonicalize(&dir.0).expect("out dir should exist after generation");
    for entry in walk(&dir.0) {
        let canonical_entry = fs::canonicalize(&entry).expect("generated file should exist");
        assert!(
            canonical_entry.starts_with(&canonical_out),
            "generated file {canonical_entry:?} escaped out dir {canonical_out:?}"
        );
    }
}

/// `dir` 配下のファイルを再帰的に列挙する（テスト専用の素朴な走査。外部
/// クレートは追加しない、REQ-3）。
fn walk(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current).expect("directory should be readable") {
            let entry = entry.expect("dir entry should be readable").path();
            if entry.is_dir() {
                stack.push(entry);
            } else {
                files.push(entry);
            }
        }
    }
    files
}
