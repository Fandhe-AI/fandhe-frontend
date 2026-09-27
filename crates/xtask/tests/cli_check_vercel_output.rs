//! `xtask check-vercel-output` の CLI 契約に対する回帰テスト（イシュー #3292）。
//!
//! `.github/workflows/ci.yml` の `test` ジョブは本テストが固定する契約
//! （終了コード・1 行サマリ書式・総括行書式）に依拠して CI の PASS/FAIL を
//! 判定し、Step Summary へ転記する。ここで固定した契約を崩す変更は CI
//! ワークフローの破壊に直結する。
//!
//! 契約（`xtask/src/main.rs` の `run_check_vercel_output` /
//! `check_vercel_output` モジュール参照）:
//! - 終了コード 0: 全チェックが PASS
//! - 終了コード 1: チェックのいずれかが FAIL（fail-closed）
//! - 終了コード 2: 引数不備（`--output-dir` 欠落・未知の引数）
//! - stdout の各行は `check-vercel-output: check=<name> result=<PASS|FAIL>[ detail=<理由>]`
//!   （`grep '^check-vercel-output:'` で抽出可能）
//! - 最終行は総括行
//!   `check-vercel-output: output_dir=<DIR> result=<PASS|FAIL> failed=<N>`
//!
//! 判定対象（許可リスト・必須ページ一覧）はコード定数で固定されており CLI
//! 引数での差し替えはできないため、フィクスチャツリーを一時ディレクトリへ
//! 直接組み立てて検証する（`cli_check_loc.rs` と同じ構成）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 統合テストのスクラッチ基点。`CARGO_TARGET_TMPDIR` は cargo が統合テスト
/// バイナリの**コンパイル時のみ**設定する（Cargo Book）ため `env!` で確定する
/// （イシュー #637/#658、`cli_check_loc.rs` と同一パターン）。
fn scratch_root() -> PathBuf {
    let root = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
    let _ = fs::create_dir_all(&root);
    root
}

/// テスト専用の一時ディレクトリを用意する（プロセス PID + テスト名で一意化）。
fn make_fixture_dir(test_name: &str) -> PathBuf {
    let dir = scratch_root().join(format!(
        "xtask-check-vercel-output-test-{test_name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("フィクスチャディレクトリの作成に失敗した");
    dir
}

/// #3290 の実生成物と同一内容の正常な Build Output API v3 ツリーを作る。
fn write_valid_tree(root: &Path) {
    let output = root.join(".vercel/output");
    let static_dir = output.join("static");
    fs::create_dir_all(static_dir.join("pages/about")).unwrap();
    fs::create_dir_all(static_dir.join("pages/default-escaping")).unwrap();

    fs::write(static_dir.join("index.html"), "<html>index</html>").unwrap();
    fs::write(static_dir.join("404.html"), "<html>404</html>").unwrap();
    fs::write(
        static_dir.join("pages/about/index.html"),
        "<html>about</html>",
    )
    .unwrap();
    fs::write(
        static_dir.join("pages/default-escaping/index.html"),
        "<html>escaping</html>",
    )
    .unwrap();

    fs::write(
        output.join("config.json"),
        r#"{
  "version": 3,
  "routes": [
    {
      "src": "/(.*)",
      "headers": {
        "X-Content-Type-Options": "nosniff",
        "Referrer-Policy": "strict-origin-when-cross-origin"
      },
      "continue": true
    },
    { "handle": "filesystem" },
    { "src": "/(.*)", "status": 404, "dest": "/404.html" }
  ]
}
"#,
    )
    .unwrap();
}

fn run_check_vercel_output(extra_args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .arg("check-vercel-output")
        .args(extra_args)
        .output()
        .expect("xtask バイナリの起動に失敗した")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn valid_tree_passes_with_exit_code_zero() {
    let dir = make_fixture_dir("valid-tree");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert!(
        result.status.success(),
        "stdout={} stderr={}",
        stdout(&result),
        String::from_utf8_lossy(&result.stderr)
    );
    let out = stdout(&result);
    assert!(out.lines().all(|line| !line.contains("result=FAIL")));
    let summary_line = out
        .lines()
        .last()
        .expect("stdout should have at least the summary line");
    assert_eq!(
        summary_line,
        format!(
            "check-vercel-output: output_dir={} result=PASS failed=0",
            output_dir.display()
        )
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn missing_output_dir_fails_with_exit_code_one() {
    let dir = make_fixture_dir("missing-output-dir");
    let output_dir = dir.join("does-not-exist");

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    let out = stdout(&result);
    assert!(out.contains("check=output_dir result=FAIL"));
    // 後続チェックは黙って PASS にせず skipped として明示される（fail-closed）。
    assert!(
        out.contains("check=config_json_parse result=FAIL detail=skipped: depends on output_dir")
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn missing_config_json_fails() {
    let dir = make_fixture_dir("missing-config-json");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");
    fs::remove_file(output_dir.join("config.json")).unwrap();

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    let out = stdout(&result);
    assert!(out.contains("check=config_json_parse result=FAIL"));
    assert!(out
        .contains("check=config_version result=FAIL detail=skipped: depends on config_json_parse"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn invalid_json_fails() {
    let dir = make_fixture_dir("invalid-json");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");
    fs::write(output_dir.join("config.json"), "{ not json").unwrap();

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    assert!(stdout(&result).contains("check=config_json_parse result=FAIL"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn version_as_string_fails() {
    let dir = make_fixture_dir("version-string");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");
    fs::write(
        output_dir.join("config.json"),
        r#"{"version": "3", "routes": [{"handle": "filesystem"}, {"src": "/(.*)", "status": 404, "dest": "/404.html"}]}"#,
    )
    .unwrap();

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    assert!(stdout(&result).contains("check=config_version result=FAIL"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn version_two_fails() {
    let dir = make_fixture_dir("version-two");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");
    fs::write(
        output_dir.join("config.json"),
        r#"{"version": 2, "routes": [{"handle": "filesystem"}, {"src": "/(.*)", "status": 404, "dest": "/404.html"}]}"#,
    )
    .unwrap();

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    assert!(stdout(&result).contains("check=config_version result=FAIL"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn unknown_route_key_typo_fails() {
    let dir = make_fixture_dir("typo-key");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");
    // 本イシューの動機となった打ち間違い（`status` → `stauts`）を再現する。
    fs::write(
        output_dir.join("config.json"),
        r#"{"version": 3, "routes": [{"handle": "filesystem"}, {"src": "/(.*)", "stauts": 404, "dest": "/404.html"}]}"#,
    )
    .unwrap();

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    let out = stdout(&result);
    assert!(out.contains("check=config_routes_shape result=FAIL"));
    assert!(out.contains("routes[1]"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn handler_route_with_extra_key_fails() {
    let dir = make_fixture_dir("handler-extra-key");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");
    fs::write(
        output_dir.join("config.json"),
        r#"{"version": 3, "routes": [{"handle": "filesystem", "bogus": true}, {"src": "/(.*)", "status": 404, "dest": "/404.html"}]}"#,
    )
    .unwrap();

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    assert!(stdout(&result).contains("check=config_routes_shape result=FAIL"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn fallback_before_filesystem_fails() {
    let dir = make_fixture_dir("fallback-before-filesystem");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");
    fs::write(
        output_dir.join("config.json"),
        r#"{"version": 3, "routes": [{"src": "/(.*)", "status": 404, "dest": "/404.html"}, {"handle": "filesystem"}]}"#,
    )
    .unwrap();

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    assert!(stdout(&result).contains("check=config_routes_404_fallback result=FAIL"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn missing_404_fallback_route_fails() {
    let dir = make_fixture_dir("missing-404-fallback");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");
    fs::write(
        output_dir.join("config.json"),
        r#"{"version": 3, "routes": [{"handle": "filesystem"}]}"#,
    )
    .unwrap();

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    assert!(stdout(&result).contains("check=config_routes_404_fallback result=FAIL"));

    let _ = fs::remove_dir_all(&dir);
}

/// PR #3322 レビュー指摘（Codex P1）の回帰テスト: `src` が `/(.*)`（全パス
/// 捕捉）でない部分一致ルートは `status`/`dest` が一致していても FAIL
/// する（`/foo` 配下にしか適用されないフォールバックは受け入れ基準 2 の
/// 「ファイルシステム未一致はすべて 404」を満たさない）。
#[test]
fn fallback_with_partial_match_src_fails() {
    let dir = make_fixture_dir("fallback-partial-match-src");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");
    fs::write(
        output_dir.join("config.json"),
        r#"{"version": 3, "routes": [{"handle": "filesystem"}, {"src": "/foo", "status": 404, "dest": "/404.html"}]}"#,
    )
    .unwrap();

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    assert!(stdout(&result).contains("check=config_routes_404_fallback result=FAIL"));

    let _ = fs::remove_dir_all(&dir);
}

/// PR #3322 レビュー指摘（Codex P1）の回帰テスト: `dest` が `$` 置換参照を
/// 含んでいても、先頭 `/../../` のような明白な親ディレクトリ脱出は境界
/// 検証（`is_safe_relative_dest` 相当）で検知する（`$` を含むと即
/// `continue` して境界検証自体を素通りする実装は本テストで FAIL する）。
#[test]
fn dest_with_replacement_ref_and_parent_traversal_fails() {
    let dir = make_fixture_dir("dest-replacement-ref-traversal");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");
    fs::write(
        output_dir.join("config.json"),
        r#"{"version": 3, "routes": [{"handle": "filesystem"}, {"src": "/(.*)", "status": 404, "dest": "/../../$1"}]}"#,
    )
    .unwrap();

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    assert!(stdout(&result).contains("check=routes_dest_targets result=FAIL"));

    let _ = fs::remove_dir_all(&dir);
}

/// PR #3322 レビュー指摘（Codex P1）の回帰テスト: `$` 置換参照を含む
/// `dest` でも、置換参照を含まないリテラルな空パス要素（`/foo//$1` の
/// 2 番目のセグメント）は明示的に拒否する（置換参照を含む dest 全体を
/// 素通りさせると `is_safe_relative_dest` が非置換 dest に課す境界検証
/// 条件と食い違ってしまう）。
#[test]
fn dest_with_replacement_ref_and_empty_segment_fails() {
    let dir = make_fixture_dir("dest-replacement-ref-empty-segment");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");
    fs::write(
        output_dir.join("config.json"),
        r#"{"version": 3, "routes": [{"handle": "filesystem"}, {"src": "/(.*)", "status": 404, "dest": "/foo//$1"}]}"#,
    )
    .unwrap();

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    assert!(stdout(&result).contains("check=routes_dest_targets result=FAIL"));

    let _ = fs::remove_dir_all(&dir);
}

/// 同上（先頭が空セグメントになる `//$1` パターン）。
#[test]
fn dest_with_replacement_ref_and_leading_empty_segment_fails() {
    let dir = make_fixture_dir("dest-replacement-ref-leading-empty-segment");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");
    fs::write(
        output_dir.join("config.json"),
        r#"{"version": 3, "routes": [{"handle": "filesystem"}, {"src": "/(.*)", "status": 404, "dest": "//$1"}]}"#,
    )
    .unwrap();

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    assert!(stdout(&result).contains("check=routes_dest_targets result=FAIL"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn dest_pointing_at_missing_file_fails() {
    let dir = make_fixture_dir("dest-missing-file");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");
    fs::write(
        output_dir.join("config.json"),
        r#"{"version": 3, "routes": [{"handle": "filesystem"}, {"src": "/(.*)", "status": 404, "dest": "/does-not-exist.html"}]}"#,
    )
    .unwrap();

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    assert!(stdout(&result).contains("check=routes_dest_targets result=FAIL"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn dest_with_parent_traversal_fails_and_does_not_escape_static() {
    let dir = make_fixture_dir("dest-traversal");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");
    // ツリー外の秘密ファイルを用意し、`dest` の `..` がそれを指しても
    // 決して「存在するファイル」として PASS しないことを固定する（A01）。
    fs::write(dir.join("secret.txt"), "top secret").unwrap();
    fs::write(
        output_dir.join("config.json"),
        r#"{"version": 3, "routes": [{"handle": "filesystem"}, {"src": "/(.*)", "status": 404, "dest": "/../../secret.txt"}]}"#,
    )
    .unwrap();

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    assert!(stdout(&result).contains("check=routes_dest_targets result=FAIL"));

    let _ = fs::remove_dir_all(&dir);
}

/// PR #3322 レビュー指摘（Codex P1、1 回目）の回帰テスト: 404 フォール
/// バックのルート自体に `"continue": true` が付いていると、そのルートで
/// 応答が確定せず後続ルートへ処理が続くため、`status`/`dest` が一致して
/// いても FAIL する。
#[test]
fn fallback_route_with_continue_true_fails() {
    let dir = make_fixture_dir("fallback-continue-true");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");
    fs::write(
        output_dir.join("config.json"),
        r#"{"version": 3, "routes": [{"handle": "filesystem"}, {"src": "/(.*)", "status": 404, "dest": "/404.html", "continue": true}]}"#,
    )
    .unwrap();

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    assert!(stdout(&result).contains("check=config_routes_404_fallback result=FAIL"));

    let _ = fs::remove_dir_all(&dir);
}

/// PR #3322 レビュー指摘（Codex P1、2 回目）の回帰テスト: `filesystem` より
/// 前に終端する（`continue: true` を持たない）キャッチオールルートが
/// あると、`filesystem` ステージにも 404 フォールバックにも到達しない
/// ため FAIL する。
#[test]
fn terminating_route_before_filesystem_fails() {
    let dir = make_fixture_dir("terminating-route-before-filesystem");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");
    fs::write(
        output_dir.join("config.json"),
        r#"{"version": 3, "routes": [{"src": "/(.*)", "status": 200, "dest": "/index.html"}, {"handle": "filesystem"}, {"src": "/(.*)", "status": 404, "dest": "/404.html"}]}"#,
    )
    .unwrap();

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    assert!(stdout(&result).contains("check=config_routes_404_fallback result=FAIL"));

    let _ = fs::remove_dir_all(&dir);
}

/// PR #3322 レビュー指摘（Codex P1、3 回目）の回帰テスト: `config.json` の
/// トップレベルに重複キー（有効な `routes` の後に不正な `routes`）が
/// あると、`json::parse` が拒否して `config_json_parse` で FAIL する
/// （`Json::get` の「最初の一致キーを返す」実装だけに頼って最初の値のみ
/// 検証し PASS してしまう抜け道を塞ぐ）。
#[test]
fn duplicate_top_level_key_in_config_json_fails() {
    let dir = make_fixture_dir("duplicate-top-level-key");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");
    fs::write(
        output_dir.join("config.json"),
        r#"{"version": 3, "routes": [{"handle": "filesystem"}, {"src": "/(.*)", "status": 404, "dest": "/404.html"}], "routes": [{"handle": "filesystem"}]}"#,
    )
    .unwrap();

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    assert!(stdout(&result).contains("check=config_json_parse result=FAIL"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn missing_static_404_page_fails() {
    let dir = make_fixture_dir("missing-404-page");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");
    fs::remove_file(output_dir.join("static/404.html")).unwrap();

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    let out = stdout(&result);
    assert!(out.contains("check=static_required_pages result=FAIL"));
    assert!(out.contains("404.html"));

    let _ = fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[test]
fn symlink_in_static_fails() {
    let dir = make_fixture_dir("symlink-in-static");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");
    let target = dir.join("outside.html");
    fs::write(&target, "<html>outside</html>").unwrap();
    std::os::unix::fs::symlink(&target, output_dir.join("static/evil.html")).unwrap();

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    assert!(stdout(&result).contains("check=static_no_symlinks result=FAIL"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn functions_dir_present_fails() {
    let dir = make_fixture_dir("functions-dir-present");
    write_valid_tree(&dir);
    let output_dir = dir.join(".vercel/output");
    fs::create_dir_all(output_dir.join("functions")).unwrap();

    let result = run_check_vercel_output(&["--output-dir", output_dir.to_str().unwrap()]);

    assert_eq!(result.status.code(), Some(1));
    assert!(stdout(&result).contains("check=no_functions_dir result=FAIL"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn missing_output_dir_flag_is_usage_error() {
    let result = run_check_vercel_output(&[]);
    assert_eq!(result.status.code(), Some(2));
}

#[test]
fn unknown_argument_is_usage_error() {
    let result = run_check_vercel_output(&["--bogus"]);
    assert_eq!(result.status.code(), Some(2));
}
