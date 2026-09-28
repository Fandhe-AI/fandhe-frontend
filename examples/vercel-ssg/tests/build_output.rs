//! `examples/vercel-ssg` の integration test（イシュー #3290）。
//!
//! `src/main.rs` はバイナリクレートのため本ファイルからは内部関数を `use`
//! できない。`examples/ssg-blog/tests/ssg_output.rs` と同じ二本立てで
//! 検証する:
//!
//! 1. ライブラリ直接検証: `fandhe_frontend_server::ssg::generate_pages` /
//!    `generate_assets` を利用者と同じ形で直接呼び、既定エスケープ・
//!    fail-closed 検証の回帰を固定する。
//! 2. CLI ブラックボックス検証: ビルド済みバイナリを
//!    `env!("CARGO_BIN_EXE_fandhe-frontend-example-vercel-ssg")` で
//!    サブプロセス起動し、`.vercel/output/` の生成結果を確認する。
//!    Basic 認証 Routing Middleware（イシュー #3343、opt-in・既定 off）の
//!    有効・無効両モードをここで検証する。フラグ環境変数
//!    （`BASIC_AUTH_FLAG_ENV`、`src/main.rs` と同じ文字列リテラル）は
//!    サブプロセス起動時に明示指定し、開発者のシェル環境の値が結果へ
//!    紛れ込まないよう全呼び出しで `env_remove` する。

use fandhe_frontend_core::{el, render, text};
use fandhe_frontend_server::ssg::{generate_assets, generate_pages, SsgError};
use std::path::PathBuf;
use std::process::Command;

/// `src/main.rs::BASIC_AUTH_FLAG_ENV` と同じ環境変数名（バイナリクレート
/// のため直接 `use` できず、リテラルとして複製する）。
const BASIC_AUTH_FLAG_ENV: &str = "FANDHE_VERCEL_SSG_BASIC_AUTH";

/// テスト専用の一時ディレクトリ。`Drop` でベストエフォート削除する
/// （`examples/ssg-blog/tests/ssg_output.rs::TempDir` と同じ方針。`tempfile`
/// 等の外部クレートを追加しない、REQ-3）。
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        // `CARGO_TARGET_TMPDIR` はコンパイル時にのみ設定される（Cargo Book）
        // ため、実行時 `std::env::var` は使わず `env!` を既定にする
        // （examples/ssg-blog と同じ理由、イシュー #637/#658）。
        let root = std::env::var("CARGO_TARGET_TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
        let _ = std::fs::create_dir_all(&root);
        let path = root.join(format!(
            "fandhe-frontend-example-vercel-ssg-test-{tag}-{}-{unique}",
            std::process::id()
        ));
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

// --- 1. ライブラリ直接検証 ---

/// 既定エスケープ回帰（REQ-1）: `<script>` を含むタイトルが実体参照化されて
/// 出力され、生の `<script>` タグとしては現れないことを固定する。
#[test]
fn generate_pages_escapes_text_content() {
    let tmp = TempDir::new("escape");
    let node = el("html", vec![], vec![text("<script>alert('xss')</script>")]);

    let written = generate_pages(&[("/".to_string(), node)], &tmp.0)
        .expect("valid single page should generate successfully");
    assert_eq!(written.len(), 1);

    let body = std::fs::read_to_string(&written[0]).expect("generated file should be readable");
    assert!(!body.contains("<script>alert"));
    assert!(body.contains("&lt;script&gt;"));
}

/// fail-closed 回帰: 不正なページパス（`..` を含む）が 1 件でも混ざると
/// `SsgError::UnsafePagePath` を返し、他の正当なページも含めて 1 つも
/// 書き出さないことを固定する。
#[test]
fn generate_pages_rejects_unsafe_path_without_partial_writes() {
    let tmp = TempDir::new("unsafe-path");
    let pages = vec![
        ("/ok/".to_string(), el("html", vec![], vec![])),
        ("/../etc".to_string(), el("html", vec![], vec![])),
    ];

    let result = generate_pages(&pages, &tmp.0);
    assert!(
        matches!(result, Err(SsgError::UnsafePagePath(_))),
        "expected UnsafePagePath, got {result:?}"
    );
    assert!(
        !tmp.0.exists(),
        "no files should be written when any page path fails validation (fail-closed)"
    );
}

/// 404 本文（`generate_assets` 経由）も `render` を経由する限り既定
/// エスケープが適用されることを固定する（`src/main.rs::not_found_asset` の
/// 契約を単独 API 呼び出しで再現する）。
#[test]
fn rendered_not_found_body_is_escaped() {
    let node = el("html", vec![], vec![text("<script>alert('xss')</script>")]);
    let body = format!("<!DOCTYPE html>\n{}", render(&node));
    assert!(!body.contains("<script>alert"));
    assert!(body.contains("&lt;script&gt;"));
}

/// `generate_assets` の無加工書き出し contract 回帰: `config.json` のような
/// 静的コンテンツがバイト無加工で書かれることを固定する。
#[test]
fn generate_assets_writes_content_verbatim_without_escaping() {
    let tmp = TempDir::new("assets-verbatim");
    let raw = r#"{"version": 3, "routes": []}"#;
    let assets = vec![("/config.json".to_string(), raw.to_string())];

    let written =
        generate_assets(&assets, &tmp.0).expect("valid single asset should write successfully");
    assert_eq!(written.len(), 1);

    let body = std::fs::read_to_string(&written[0]).expect("written asset should be readable");
    assert_eq!(
        body, raw,
        "generate_assets must write content byte-for-byte without default escaping"
    );
}

// --- 2. CLI ブラックボックス検証 ---

/// `src/main.rs` のバイナリを一意な一時ディレクトリを `current_dir` として
/// 起動し、生成された `.vercel/output/` を含むディレクトリのパスを返す。
/// `BASIC_AUTH_FLAG_ENV` は常に `env_remove` してから起動する
/// （開発者のシェル環境の値が紛れ込まないようにする既定の無効モード）。
fn run_cli_in_scratch_dir(tag: &str) -> TempDir {
    run_cli_in_scratch_dir_with_flag(tag, None)
}

/// [`run_cli_in_scratch_dir`] のフラグ指定版。`flag` が `None` なら
/// `env_remove`（未設定）、`Some(v)` なら `BASIC_AUTH_FLAG_ENV=v` を設定
/// してから CLI を起動する。
fn run_cli_in_scratch_dir_with_flag(tag: &str, flag: Option<&str>) -> TempDir {
    let scratch = TempDir::new(tag);
    std::fs::create_dir_all(&scratch.0).expect("failed to create scratch dir");

    let mut command = Command::new(env!("CARGO_BIN_EXE_fandhe-frontend-example-vercel-ssg"));
    command
        .current_dir(&scratch.0)
        .env_remove(BASIC_AUTH_FLAG_ENV);
    if let Some(value) = flag {
        command.env(BASIC_AUTH_FLAG_ENV, value);
    }
    let output = command
        .output()
        .expect("binary should spawn and run to completion");
    assert!(
        output.status.success(),
        "CLI should exit 0: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    scratch
}

/// 受け入れ条件 1: `cargo run` で Build Output API 形式の `.vercel/output/`
/// が生成されることを固定する（`config.json` + `static/` 以下の HTML 群）。
#[test]
fn cli_generates_expected_output_tree() {
    let scratch = run_cli_in_scratch_dir("output-tree");
    let output = scratch.0.join(".vercel/output");

    assert!(output.join("config.json").is_file());
    let static_dir = output.join("static");
    assert!(static_dir.join("index.html").is_file());
    assert!(static_dir.join("404.html").is_file());
    for slug in ["about", "default-escaping"] {
        assert!(
            static_dir
                .join("pages")
                .join(slug)
                .join("index.html")
                .is_file(),
            "static/pages/{slug}/index.html should exist"
        );
    }
}

/// 受け入れ条件 2: `config.json` の `routes` によってファイルシステムに
/// 一致しないパスへ 404 ステータスで `/404.html` を返す設定が含まれることを
/// 固定する。
#[test]
fn cli_config_json_declares_filesystem_fallback_to_404() {
    let scratch = run_cli_in_scratch_dir("config-404");
    let config = std::fs::read_to_string(scratch.0.join(".vercel/output/config.json"))
        .expect("config.json should be readable");

    assert!(config.contains("\"version\": 3"));
    assert!(config.contains("\"handle\": \"filesystem\""));
    assert!(config.contains("\"status\": 404"));
    assert!(config.contains("\"dest\": \"/404.html\""));
}

/// `src/main.rs::CONFIG_JSON` のリテラルコピー。無効時のバイト同一性を
/// 固定するため、ここでも独立に定義する（バイナリクレートのため `use`
/// できない。乖離した場合は下記テストが検知する）。
const CONFIG_JSON_LITERAL_COPY: &str = r#"{
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
"#;

/// Basic 認証（イシュー #3343）無効時の受け入れ条件: `functions/` が
/// 生成されず、`config.json` が本機能導入前とバイト単位で同一であること
/// を固定する（無効時の出力に一切影響しないことの機械的な保証）。
#[test]
fn cli_basic_auth_disabled_by_default_produces_unchanged_output() {
    let scratch = run_cli_in_scratch_dir("basic-auth-default");
    let output = scratch.0.join(".vercel/output");

    assert!(
        !output.join("functions").exists(),
        "functions/ should not exist when basic auth is disabled (default)"
    );

    let config = std::fs::read_to_string(output.join("config.json"))
        .expect("config.json should be readable");
    assert!(
        !config.contains("middlewarePath"),
        "config.json must not reference middlewarePath when basic auth is disabled"
    );
    assert_eq!(
        config, CONFIG_JSON_LITERAL_COPY,
        "config.json must remain byte-identical to the pre-#3343 CONFIG_JSON when disabled"
    );
}

/// Basic 認証フラグが `"1"` 以外の値（未設定・空文字・`"0"`・`"true"`）
/// のときはすべて無効扱い（`functions/` 非生成）になることを固定する
/// （fail-closed: 誤った値での意図しない有効化を防ぐ）。
#[test]
fn cli_basic_auth_non_one_values_are_treated_as_disabled() {
    for value in [None, Some(""), Some("0"), Some("true")] {
        let tag = format!("basic-auth-disabled-{}", value.unwrap_or("unset"));
        let scratch = run_cli_in_scratch_dir_with_flag(&tag, value);
        assert!(
            !scratch.0.join(".vercel/output/functions").exists(),
            "functions/ should not exist for flag value {value:?}"
        );
    }
}

/// Basic 認証（イシュー #3343）有効時（`"1"`）の受け入れ条件:
/// ミドルウェア Function 2 ファイル + middlewarePath 入りの `config.json`
/// が生成され、静的ページ群と 404 も引き続き生成されることを固定する。
#[test]
fn cli_basic_auth_enabled_generates_middleware_function() {
    let scratch = run_cli_in_scratch_dir_with_flag("basic-auth-enabled", Some("1"));
    let output = scratch.0.join(".vercel/output");

    let vc_config = output.join("functions/_middleware.func/.vc-config.json");
    let index_js = output.join("functions/_middleware.func/index.js");
    assert!(vc_config.is_file(), ".vc-config.json should be generated");
    assert!(index_js.is_file(), "index.js should be generated");

    let vc_config_body =
        std::fs::read_to_string(&vc_config).expect(".vc-config.json should be readable");
    assert!(vc_config_body.contains("\"runtime\": \"edge\""));

    let config = std::fs::read_to_string(output.join("config.json"))
        .expect("config.json should be readable");
    // middlewarePath ルートが `{"handle": "filesystem"}` より前（先頭）に
    // 置かれていることを文字列位置で確認する（全パスを保護する契約）。
    let middleware_pos = config
        .find("middlewarePath")
        .expect("config.json should declare middlewarePath");
    let filesystem_pos = config
        .find("\"handle\": \"filesystem\"")
        .expect("config.json should declare filesystem handler");
    assert!(
        middleware_pos < filesystem_pos,
        "middlewarePath route must precede the filesystem handler"
    );

    assert!(output.join("static/index.html").is_file());
    assert!(output.join("static/404.html").is_file());
}

/// ミドルウェア本体の静的検査（イシュー #3343）: Node をテストランナーへ
/// 持ち込まず（REQ-12）、生成された `index.js` の文字列内容だけで
/// fail-closed 実装であることと機微情報の非ログ出力を固定する。
/// 実際の認証応答の最終確認はコードレビューとガイドの curl 手順
/// （Vercel 上のデプロイ）で行う。
#[test]
fn cli_basic_auth_index_js_has_expected_fail_closed_shape() {
    let scratch = run_cli_in_scratch_dir_with_flag("basic-auth-index-js-shape", Some("1"));
    let index_js = std::fs::read_to_string(
        scratch
            .0
            .join(".vercel/output/functions/_middleware.func/index.js"),
    )
    .expect("index.js should be readable");

    for expected in [
        "status: 503",
        "status: 401",
        "WWW-Authenticate",
        "Basic realm=",
        "x-middleware-next",
        "process.env.BASIC_AUTH_USER",
        "process.env.BASIC_AUTH_PASSWORD",
        "crypto.subtle.digest",
    ] {
        assert!(
            index_js.contains(expected),
            "index.js should contain {expected:?}"
        );
    }

    assert!(
        !index_js.contains("console."),
        "index.js must not log to console (avoid leaking credentials/Authorization)"
    );
    assert!(
        !index_js.contains(".length !=="),
        "index.js must not short-circuit constant-time comparison on length mismatch"
    );
}

/// 古い出力の削除（OWASP A05）: 事前に置いた古いファイルが実行後に消え、
/// `.vercel/project.json`（`vercel link` 相当のダミー）は残ることを固定する。
#[test]
fn cli_removes_stale_output_but_keeps_vercel_project_files() {
    let scratch = TempDir::new("stale-cleanup");
    std::fs::create_dir_all(&scratch.0).expect("failed to create scratch dir");
    std::fs::create_dir_all(scratch.0.join(".vercel/output/static")).unwrap();
    std::fs::write(
        scratch.0.join(".vercel/output/static/stale.html"),
        "<html>stale</html>",
    )
    .unwrap();
    std::fs::write(scratch.0.join(".vercel/project.json"), "{}").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_fandhe-frontend-example-vercel-ssg"))
        .current_dir(&scratch.0)
        .env_remove(BASIC_AUTH_FLAG_ENV)
        .output()
        .expect("binary should spawn and run to completion");
    assert!(output.status.success());

    assert!(
        !scratch.0.join(".vercel/output/static/stale.html").exists(),
        "stale output should be removed before regeneration"
    );
    assert!(
        scratch.0.join(".vercel/project.json").is_file(),
        ".vercel/project.json (vercel link output) must not be deleted"
    );
}

/// XSS 回帰（REQ-1）: `default-escaping` ページのタイトルが既定エスケープ
/// され、生の `<script>` を含まないことを CLI 経由で固定する。
#[test]
fn cli_escapes_xss_payload_title() {
    let scratch = run_cli_in_scratch_dir("xss-title");
    let body = std::fs::read_to_string(
        scratch
            .0
            .join(".vercel/output/static/pages/default-escaping/index.html"),
    )
    .expect("default-escaping page should be readable");

    assert!(!body.contains("<script>alert"));
    assert!(body.contains("&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"));
}

/// 全ページに `@view-transition { navigation: auto; }` が含まれることを
/// 固定する（`layout()` が全ページ共通で出力する契約）。
#[test]
fn cli_output_includes_view_transition_style() {
    let scratch = run_cli_in_scratch_dir("view-transition");
    let output = scratch.0.join(".vercel/output/static");

    for rel in [
        "index.html",
        "pages/about/index.html",
        "pages/default-escaping/index.html",
        "404.html",
    ] {
        let body = std::fs::read_to_string(output.join(rel))
            .unwrap_or_else(|e| panic!("{rel} should be readable: {e}"));
        assert!(
            body.contains("<style>@view-transition { navigation: auto; }</style>"),
            "{rel} should include the view-transition style"
        );
        assert!(body.starts_with("<!DOCTYPE html>"));
    }
}

#[cfg(unix)]
/// fail-closed 回帰: `.vercel/output` がシンボリックリンクの場合は非ゼロ
/// 終了し、リンク先を削除しないことを固定する（unix 限定）。
#[test]
fn cli_refuses_to_clean_symlinked_output_dir() {
    use std::os::unix::fs::symlink;

    let scratch = TempDir::new("symlink-guard");
    std::fs::create_dir_all(&scratch.0).expect("failed to create scratch dir");
    let real_target = scratch.0.join("real-target");
    std::fs::create_dir_all(&real_target).unwrap();
    std::fs::write(real_target.join("marker.txt"), "keep-me").unwrap();
    std::fs::create_dir_all(scratch.0.join(".vercel")).unwrap();
    symlink(&real_target, scratch.0.join(".vercel/output")).expect("symlink should be created");

    let output = Command::new(env!("CARGO_BIN_EXE_fandhe-frontend-example-vercel-ssg"))
        .current_dir(&scratch.0)
        .env_remove(BASIC_AUTH_FLAG_ENV)
        .output()
        .expect("binary should spawn and run to completion");

    assert!(
        !output.status.success(),
        "CLI should refuse to run when .vercel/output is a symlink"
    );
    assert!(
        real_target.join("marker.txt").is_file(),
        "symlink target must not be deleted"
    );
}

#[cfg(unix)]
/// fail-closed 回帰: `.vercel` 自体（`.vercel/output` の親要素）が外部
/// ディレクトリへのシンボリックリンクの場合も非ゼロ終了し、リンク先を
/// 削除しないことを固定する（unix 限定）。`.vercel/output` 単体の
/// `symlink_metadata` だけを見る実装だと、このケースはリンクを辿った先の
/// `output` に対して `remove_dir_all` が実行されてしまう。
#[test]
fn cli_refuses_to_clean_when_vercel_parent_is_symlink() {
    use std::os::unix::fs::symlink;

    let scratch = TempDir::new("symlink-guard-parent");
    std::fs::create_dir_all(&scratch.0).expect("failed to create scratch dir");
    let real_target = scratch.0.join("real-target");
    std::fs::create_dir_all(real_target.join("output")).unwrap();
    std::fs::write(real_target.join("output/marker.txt"), "keep-me").unwrap();
    symlink(&real_target, scratch.0.join(".vercel")).expect("symlink should be created");

    let output = Command::new(env!("CARGO_BIN_EXE_fandhe-frontend-example-vercel-ssg"))
        .current_dir(&scratch.0)
        .env_remove(BASIC_AUTH_FLAG_ENV)
        .output()
        .expect("binary should spawn and run to completion");

    assert!(
        !output.status.success(),
        "CLI should refuse to run when .vercel (parent of output) is a symlink"
    );
    assert!(
        real_target.join("output/marker.txt").is_file(),
        "symlink target must not be deleted"
    );
}
