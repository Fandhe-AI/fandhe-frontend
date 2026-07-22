//! `examples/headless-pre-styled-ui` の CLI ブラックボックス統合テスト
//! （イシュー #552）。
//!
//! `src/main.rs` 内の `#[cfg(test)]` ユニットテストが anatomy・`data-state`・
//! ARIA 属性・既定エスケープをライブラリレベルで固定するのに対し、本ファイル
//! は `examples/ssg-blog/tests/ssg_output.rs` と同じ方針でビルド済みバイナリ
//! をサブプロセス起動し、実際に `cargo run` した利用者が目にする `dist/`
//! 出力（ファイル配置・エスケープ済み HTML・静的 CSS 同梱）を検証する。

use std::path::PathBuf;
use std::process::Command;

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
        let path = std::env::temp_dir().join(format!(
            "fandhe-frontend-example-headless-pre-styled-ui-test-{tag}-{}-{unique}",
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

/// `src/main.rs` のバイナリを一意な一時ディレクトリを `current_dir` として
/// 起動し、生成された `dist/` を含むディレクトリを返す。
fn run_cli_in_scratch_dir(tag: &str) -> TempDir {
    let scratch = TempDir::new(tag);
    std::fs::create_dir_all(&scratch.0).expect("failed to create scratch dir");

    let output = Command::new(env!(
        "CARGO_BIN_EXE_fandhe-frontend-example-headless-pre-styled-ui"
    ))
    .current_dir(&scratch.0)
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

/// `cargo run` で `dist/index.html`・`dist/assets/ui.css` が生成されること
/// を固定する。
#[test]
fn cli_generates_expected_dist_files() {
    let scratch = run_cli_in_scratch_dir("dist-files");
    let dist = scratch.0.join("dist");

    assert!(dist.join("index.html").is_file());
    assert!(dist.join("assets").join("ui.css").is_file());
}

/// 全コンポーネントの `data-scope` セレクタが出力へ含まれることを固定する
/// （受け入れ条件(a): anatomy の実演）。
#[test]
fn cli_output_includes_all_component_scopes() {
    let scratch = run_cli_in_scratch_dir("component-scopes");
    let body = std::fs::read_to_string(scratch.0.join("dist/index.html"))
        .expect("index.html should be readable");

    for scope in [
        "data-scope=\"tabs\"",
        "data-scope=\"accordion\"",
        "data-scope=\"dialog\"",
        "data-scope=\"switch\"",
        "data-scope=\"radio-group\"",
        "data-scope=\"avatar\"",
    ] {
        assert!(body.contains(scope), "missing {scope} in dist/index.html");
    }
}

/// 全ページに `@view-transition { navigation: auto; }` が含まれることを
/// 固定する（`layout()` が出力する契約）。
#[test]
fn cli_output_includes_view_transition_style() {
    let scratch = run_cli_in_scratch_dir("view-transition");
    let body = std::fs::read_to_string(scratch.0.join("dist/index.html"))
        .expect("index.html should be readable");

    assert!(body.contains("<style>@view-transition { navigation: auto; }</style>"));
    assert!(body.starts_with("<!DOCTYPE html>"));
}

/// 既定エスケープ回帰（REQ-1・OWASP A03）: `<script>` ペイロードを含む
/// トリガーラベルが実体参照化されて出力され、生の `<script>` タグとしては
/// 現れないことを CLI 経由で固定する。
#[test]
fn cli_escapes_xss_payload() {
    let scratch = run_cli_in_scratch_dir("xss-payload");
    let body = std::fs::read_to_string(scratch.0.join("dist/index.html"))
        .expect("index.html should be readable");

    assert!(!body.contains("<script>alert"));
    assert!(body.contains("&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"));
}
