//! `examples/wireframe-ui` の integration test（イシュー #2667）。
//!
//! `src/main.rs` はバイナリクレートのため本ファイルからは内部関数
//! （`build_page`/`sections::build_sections`）を `use` できない。本サンプルは
//! `fandhe_frontend_wireframe_ui` の各部品関数（公開 API）を利用者と同じ形で
//! 直接呼ぶ既定エスケープ回帰と、ビルド済みバイナリを
//! `env!("CARGO_BIN_EXE_fandhe-frontend-example-wireframe-ui")` でサブプロセス
//! 起動する CLI ブラックボックス検証の二本立てで構成する
//! （`examples/ssg-blog/tests/ssg_output.rs` と同型の方針）。

use fandhe_frontend_core::render;
use fandhe_frontend_wireframe_ui::{self as wire, Primary, Size};
use std::path::PathBuf;
use std::process::Command;

/// テスト専用の一時ディレクトリ。`Drop` でベストエフォート削除する
/// （`crates/server/tests/support/temp_dir.rs` と同じ方針。`tempfile` 等の
/// 外部クレートを追加しない、REQ-3）。
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        // cargo が `CARGO_TARGET_TMPDIR` を設定するのはテストバイナリの
        // コンパイル時のみ（Cargo Book）であり、実行時 `std::env::var` 参照は
        // 常に失敗する。既定はコンパイル時に確定する
        // `env!("CARGO_TARGET_TMPDIR")`（`<target>/tmp` 配下。本サンプルは
        // root workspace から意図的に切り離された独立 `[workspace]` のため、
        // ここでの `<target>` は `examples/wireframe-ui/target`）を使い、
        // `/tmp` へは一切フォールバックしない（イシュー #637/#658）。実行時
        // env による明示上書きは引き続き許容する。
        let root = std::env::var("CARGO_TARGET_TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
        let _ = std::fs::create_dir_all(&root);
        let path = root.join(format!(
            "fandhe-frontend-example-wireframe-ui-test-{tag}-{}-{unique}",
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

/// 既定エスケープ回帰（REQ-1）: `<script>` を含むタイトルが `annotation`
/// 部品を経由して実体参照化されて出力され、生の `<script>` タグとしては
/// 現れないことを固定する。
#[test]
fn wire_component_escapes_text_content() {
    let node = wire::annotation(
        "<script>alert('xss')</script>",
        None,
        Size::Md,
        Primary(false),
    );
    let html = render(&node);
    assert!(!html.contains("<script>alert"));
    assert!(html.contains("&lt;script&gt;"));
}

/// `wireframe_css()` は `<` を一切含まない（`<style>` 文脈でも `<link>`
/// 別ファイル経路でも安全に扱える CSS のみを返す）契約を固定する。
#[test]
fn wireframe_css_contains_no_angle_bracket_open() {
    let css = wire::wireframe_css();
    assert!(!css.contains('<'));
}

// --- 2. CLI ブラックボックス検証 ---

/// `src/main.rs` のバイナリを一意な一時ディレクトリを `current_dir` として
/// 起動し、生成された `dist/` を含むディレクトリのパスを返す。
fn run_cli_in_scratch_dir(tag: &str) -> TempDir {
    let scratch = TempDir::new(tag);
    std::fs::create_dir_all(&scratch.0).expect("failed to create scratch dir");

    let output = Command::new(env!("CARGO_BIN_EXE_fandhe-frontend-example-wireframe-ui"))
        .current_dir(&scratch.0)
        .output()
        .expect("binary should spawn and run to completion");
    assert!(
        output.status.success(),
        "CLI should exit 0: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("dist/index.html"));
    assert!(stdout.contains("dist/assets/wireframe.css"));

    scratch
}

/// 受け入れ条件: `cargo run` で `dist/index.html` と
/// `dist/assets/wireframe.css` の 2 ファイルが生成されることを固定する。
#[test]
fn cli_generates_expected_dist_files() {
    let scratch = run_cli_in_scratch_dir("dist-files");
    let dist = scratch.0.join("dist");

    assert!(dist.join("index.html").is_file());
    assert!(dist.join("assets").join("wireframe.css").is_file());

    let html = std::fs::read_to_string(dist.join("index.html")).expect("index.html を読めること");
    // 既定エスケープ回帰（REQ-1）: main.rs のショーケース本文にも
    // XSS 実演節が含まれ、生の `<script>` としては出力されないこと。
    assert!(!html.contains("<script>alert"));
    assert!(html.contains("&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"));
    // CSS は別ファイル参照であり、CSS ルール本体はページ本文に現れない
    // （`src/main.rs` rustdoc「CSS の出力方式」節の回帰）。
    assert!(html.contains(r#"<link rel="stylesheet" href="assets/wireframe.css">"#));

    let css = std::fs::read_to_string(dist.join("assets").join("wireframe.css"))
        .expect("wireframe.css を読めること");
    assert_eq!(css, fandhe_frontend_wireframe_ui::wireframe_css());
}
