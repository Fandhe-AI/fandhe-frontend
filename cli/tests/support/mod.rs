//! `cli/tests/negative_cases.rs`（TASK-13.5・#148）と
//! `cli/tests/xss_regression_link.rs`（TASK-13.3c・#141）が共用する
//! fixture 基盤。
//!
//! 両ファイルはいずれも `fw gate`（`cli/src/gate.rs`, TASK-13.3・#138）を
//! 実バイナリとして起動し、一時的な最小プロジェクト（`structure.toml` +
//! virtual workspace + `deny.toml` + `clippy.toml`）に対して欠陥を注入して
//! BLOCKED まで到達させる e2e テストであり、フィクスチャの書き出し・
//! 起動・JSON レポート判定のヘルパー群が完全に重複するためここへ集約する。
//! `tests/support/mod.rs` に置くことで cargo からは独立したテストバイナリ
//! として扱われず（`tests/` 直下の `.rs` のみが個別クレートになる）、
//! 両テストファイルから `mod support;` で参照できる。
//!
//! フィクスチャ自体の設計判断（`CARGO_TARGET_DIR` を専用化する理由・
//! `cargo generate-lockfile --offline` を要する理由等）は `negative_cases.rs`
//! 由来のためコメントもそのまま引き継ぐ。
//!
//! 本モジュールは `tests/` 配下の複数バイナリ（`negative_cases.rs` /
//! `xss_regression_link.rs`）から個別にコンパイルされ、各バイナリは公開
//! ヘルパーの部分集合しか使わない（例: `negative_cases.rs` は
//! `write_xss_case_project` を使わない）。dead_code lint はバイナリ単位で
//! 判定されるため、他方でのみ使われる関数がもう一方のコンパイル単位では
//! 未使用警告になる。共有ヘルパーモジュールの典型的な事情のため、モジュール
//! 全体で `dead_code` を許可する。

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// `fw` バイナリを `gate --project <dir>` で起動し、(終了コード, stdout, stderr)
/// を返す（`gate_integration.rs` の `run_fw_gate` と同一パターン）。
///
/// `CARGO_TARGET_DIR` はフィクスチャ間で共有しない（`raw_html_lint_e2e.rs`
/// と同一方針）。self-hosted runner では `CARGO_TARGET_DIR=/cargo-target` が
/// プロセス環境に既定で設定されており、本テストの全フィクスチャは同名パッケージ
/// （`negative-fixture-app` あるいは `xss-fixture-app`）のため、これを継承した
/// まま `cargo` を起動するとフィクスチャ間でビルドキャッシュ/フィンガープリント
/// が衝突し、直前に生成した別フィクスチャのチェック結果を誤って再利用して
/// しまう（欠陥を注入したはずのケースが再コンパイルされず誤って PASS する
/// 偽陰性）。ここで `project_dir` 配下の専用 `target/` を明示指定し、継承
/// された値を上書きすることで各フィクスチャを独立させる（`fw` から起動される
/// `cargo` 子プロセスにも env は継承されるため、これで `gate.rs` 側の変更は
/// 不要）。
pub fn run_fw_gate(project_dir: &Path) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_fw"))
        .arg("gate")
        .arg("--project")
        .arg(project_dir)
        .env("CARGO_TARGET_DIR", project_dir.join("target"))
        .output()
        .expect("failed to spawn `fw` binary");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// `stdout`（`fw gate` の JSON レポート）中の `"name":"<name>"` エントリの
/// `passed` 値を判定する。該当エントリが見つからない場合は `None`
/// （「チェック自体が JSON に現れていない」ことと「passed:false」を区別する
/// ため、`bool` ではなく `Option<bool>` を返す）。
pub fn check_passed(stdout: &str, name: &str) -> Option<bool> {
    if stdout.contains(&format!("\"name\":\"{name}\",\"passed\":true")) {
        Some(true)
    } else if stdout.contains(&format!("\"name\":\"{name}\",\"passed\":false")) {
        Some(false)
    } else {
        None
    }
}

/// 実行環境に `cargo-deny` サブコマンドが導入済みかを判定する
/// （リポジトリ自身の CI には TASK-13.3c（#141）でインストールステップが
/// 導入済みだが、ローカル開発環境やそれ以外の CI では未導入の場合がある差を
/// 吸収するための補助関数）。
pub fn cargo_deny_available() -> bool {
    Command::new("cargo")
        .args(["deny", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// `write_case_project` / `write_xss_case_project` が書き出した一時プロジェクト
/// ディレクトリを保持し、スコープを抜けるタイミングで自身を削除するガード
/// （`templates/default/tests/negative_type_error.rs` の `ScratchProject` と
/// 同一方針）。
pub struct ScratchProject(pub PathBuf);

impl std::ops::Deref for ScratchProject {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl Drop for ScratchProject {
    fn drop(&mut self) {
        // 削除失敗（他プロセスによるロック等）はテスト結果の正当性に
        // 影響しないため、ベストエフォートとして無視する。
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// 一時プロジェクトを書き出すスクラッチルート。`CARGO_TARGET_TMPDIR`
/// （cargo がテストバイナリ実行時に設定する target 配下の一時ディレクトリ）が
/// あればそこに閉じ、未設定環境向けに OS 標準の一時領域へフォールバックする
/// （`negative_type_error.rs` と同一パターン、パストラバーサル対策の一環）。
pub fn scratch_root() -> PathBuf {
    std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
}

/// ベースライン（正例）となる `app/src/main.rs` の内容。PoC-7
/// `target-project`（`Item` / `find_item`）相当の最小構成で、依存ゼロ・
/// clippy クリーン・`raw_html` 文字列を一切含まない。
///
/// 各負例ケースはこの文字列に対して注入対象の部分文字列を一意に
/// 置換することで欠陥を混入させる（`negative_type_error.rs` の
/// 「注入対象を実行可能コードに限定する」方針を踏襲）。
pub fn baseline_main_rs() -> &'static str {
    r#"struct Item {
    id: String,
    name: String,
}

fn find_item<'a>(items: &'a [Item], target_id: &str) -> Option<&'a Item> {
    items.iter().find(|it| it.id == target_id)
}

fn main() {
    let items = vec![
        Item {
            id: "1".to_string(),
            name: "widget".to_string(),
        },
        Item {
            id: "2".to_string(),
            name: "gadget".to_string(),
        },
    ];
    if let Some(item) = find_item(&items, "1") {
        println!("found: {}", item.name);
    }
}
"#
}

/// `deny.toml` の共通内容（`templates/default/deny.toml` と同じ主要ポリシー
/// [bans]/[licenses]/[sources]）。`policy` チェックが実在確認の先で実際に
/// `cargo deny check bans licenses sources` を走らせられるようにする。
fn deny_toml_content() -> &'static str {
    r#"[graph]
targets = []

[bans]
multiple-versions = "warn"
deny = [
    { name = "openssl-sys" },
]

[licenses]
allow = ["MIT", "Apache-2.0", "Unicode-3.0", "BSD-3-Clause"]

[sources]
unknown-registry = "deny"
unknown-git = "deny"
"#
}

/// イシュー #157/#263（`gate.rs::clippy_policy_check`）: `lint` チェックは
/// `project_dir` 直下の `clippy.toml` に `disallowed-methods` の
/// `rws_core::raw_html` エントリが存在することを fail-closed で前提とする
/// （欠落時は cargo clippy を起動する前に `lint` を failed とする）。本
/// フィクスチャはワークスペースルートの `clippy.toml` と同一ポリシーを配布
/// する `templates/default/clippy.toml` と同内容を複製し、`lint` チェックを
/// 実体化させる。
fn clippy_toml_content() -> &'static str {
    r#"disallowed-methods = [
    { path = "rws_core::raw_html", reason = "REQ-1 の唯一のエスケープ迂回経路。レビュー済みの呼び出しには `#[expect(clippy::disallowed_methods, reason = \"ESCAPE-REVIEWED: <根拠>\")]` を呼び出し文へ直接付与すること（`#[allow(...)]` によるブランケット抑止は禁止、docs/raw-html-review-gate.md 参照）" },
]
"#
}

/// `structure.toml` の共通内容（`[directories.app]`、`role = "component"`）。
/// `crate_name` のみフィクスチャごとに差し替える。
fn structure_toml_content(crate_name: &str, description: &str) -> String {
    format!(
        r#"
[manifest]
version = 1

[directories.app]
role = "component"
crate = "{crate_name}"
description = "{description}"
"#
    )
}

/// 一意な部分文字列 `from` を `to` へちょうど 1 箇所だけ置換する。複数箇所・
/// 0 箇所にマッチした場合は panic し、フィクスチャのリファクタリングで
/// 注入前提が崩れたことをテスト失敗として顕在化させる
/// （`negative_type_error.rs` の注入方針と同じ）。
pub fn replace_unique(content: &str, from: &str, to: &str) -> String {
    assert_eq!(
        content.matches(from).count(),
        1,
        "注入対象の部分文字列 `{from}` が一意に見つからない（ベースラインの \
         リファクタリングでこのテストの前提が崩れている）"
    );
    let injected = content.replacen(from, to, 1);
    assert_ne!(content, injected, "置換後の内容が変化していない");
    injected
}

/// `cargo generate-lockfile --offline` で `Cargo.lock` を生成する（依存ゼロの
/// ため決定的・ネットワーク不要）。`fw gate` は `--locked` で `cargo`
/// サブコマンドを起動するため、ロックファイルなしでは各チェックがロック
/// ファイル欠落自体で failed になり、注入した欠陥とは無関係な失敗理由に
/// なってしまう（ケースの特定性を損なう）ため、呼び出し側で確実に実行する。
fn generate_lockfile(dest: &Path) {
    let lockfile_output = Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(dest)
        .output()
        .expect("cargo generate-lockfile の起動に失敗した");
    assert!(
        lockfile_output.status.success(),
        "cargo generate-lockfile --offline に失敗した（フィクスチャ自体が壊れている）: {}",
        String::from_utf8_lossy(&lockfile_output.stderr)
    );
}

/// 一意な一時プロジェクトディレクトリに以下を書き出す:
///
/// ```text
/// <fixture>/
/// ├── structure.toml   ([directories.app], role = "component")
/// ├── Cargo.toml       (virtual workspace, members = ["app"])
/// ├── deny.toml        (templates/default/deny.toml と同ポリシーの最小版)
/// ├── clippy.toml      (templates/default/clippy.toml と同内容)
/// └── app/
///     ├── Cargo.toml   (name = "negative-fixture-app", 依存ゼロ)
///     └── src/main.rs  (main_rs_content)
/// ```
///
/// `negative_cases.rs`（TASK-13.5）専用。既存 3 負例のフィクスチャ構成を
/// 変更しないため、リファクタリング前と完全に同じ内容・同じシグネチャで
/// 維持する。
pub fn write_case_project(case_name: &str, main_rs_content: &str) -> ScratchProject {
    let dest = scratch_root().join(format!(
        "negative-cases-{case_name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dest);
    let app_src = dest.join("app").join("src");
    fs::create_dir_all(&app_src).expect("一時プロジェクトディレクトリの作成に失敗した");

    fs::write(
        dest.join("structure.toml"),
        structure_toml_content("negative-fixture-app", "TASK-13.5 negative case fixture"),
    )
    .expect("structure.toml の書き込みに失敗した");

    fs::write(
        dest.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    )
    .expect("workspace Cargo.toml の書き込みに失敗した");

    fs::write(dest.join("deny.toml"), deny_toml_content()).expect("deny.toml の書き込みに失敗した");

    fs::write(dest.join("clippy.toml"), clippy_toml_content())
        .expect("clippy.toml の書き込みに失敗した");

    fs::write(
        dest.join("app").join("Cargo.toml"),
        "[package]\nname = \"negative-fixture-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\nlicense = \"MIT\"\npublish = false\n",
    )
    .expect("app/Cargo.toml の書き込みに失敗した");

    fs::write(app_src.join("main.rs"), main_rs_content).expect("main.rs の書き込みに失敗した");

    generate_lockfile(&dest);

    ScratchProject(dest)
}

/// `xss_regression_link.rs`（TASK-13.3c・#141）専用フィクスチャ。
///
/// `write_case_project` との違いは、`app` クレートに `lib.rs`（エスケープ
/// 実装）と `tests/xss_escape.rs`（XSS 回帰テスト、PoC-7/`core/tests/xss_escape.rs`
/// 相当の代表ペイロード検証）を追加で持たせる点にある。`fw gate` の `test`
/// チェック（`cargo test --locked -p <crate>`）がこの回帰テストの合否を
/// そのまま反映することを検証するのが目的（TASK-1.2 との連携）。
///
/// `main.rs` は `lib.rs` の関数を呼ぶだけの最小実装とし、`escape_html_content`
/// が実際にバイナリ・ライブラリ双方から参照される構成にする。
pub fn write_xss_case_project(case_name: &str, escape_html_content: &str) -> ScratchProject {
    let dest = scratch_root().join(format!(
        "xss-regression-link-{case_name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dest);
    let app_src = dest.join("app").join("src");
    let app_tests = dest.join("app").join("tests");
    fs::create_dir_all(&app_src).expect("一時プロジェクトディレクトリの作成に失敗した");
    fs::create_dir_all(&app_tests).expect("tests/ ディレクトリの作成に失敗した");

    fs::write(
        dest.join("structure.toml"),
        structure_toml_content("xss-fixture-app", "TASK-13.3c XSS regression link fixture"),
    )
    .expect("structure.toml の書き込みに失敗した");

    fs::write(
        dest.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    )
    .expect("workspace Cargo.toml の書き込みに失敗した");

    fs::write(dest.join("deny.toml"), deny_toml_content()).expect("deny.toml の書き込みに失敗した");

    fs::write(dest.join("clippy.toml"), clippy_toml_content())
        .expect("clippy.toml の書き込みに失敗した");

    fs::write(
        dest.join("app").join("Cargo.toml"),
        "[package]\nname = \"xss-fixture-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\nlicense = \"MIT\"\npublish = false\n",
    )
    .expect("app/Cargo.toml の書き込みに失敗した");

    // main.rs は lib.rs の escape_html_content を呼ぶだけの最小バイナリ。
    // `default_escape_check`（`gate.rs`）はソース中の未レビュー `raw_html()`
    // 呼び出しのみを検出対象とするため、raw_html を一切使わない本フィクス
    // チャは常に default_escape_check を通過する
    // （= 検知が `test` チェック経由であることの前提）。
    fs::write(
        app_src.join("main.rs"),
        "fn main() {\n    println!(\"{}\", xss_fixture_app::escape_html_content(\"<script>alert(1)</script>\"));\n}\n",
    )
    .expect("main.rs の書き込みに失敗した");

    fs::write(app_src.join("lib.rs"), escape_html_content).expect("lib.rs の書き込みに失敗した");

    fs::write(
        app_tests.join("xss_escape.rs"),
        xss_regression_test_rs_content(),
    )
    .expect("tests/xss_escape.rs の書き込みに失敗した");

    generate_lockfile(&dest);

    ScratchProject(dest)
}

/// 期待どおりにエスケープされることを検証する最小 XSS 回帰テスト
/// （`core/tests/xss_escape.rs` の代表ペイロードの一部を移植）。
/// `escape_html_content` を退行させると本テストが failed になり、
/// `fw gate` の `test` チェック経由で BLOCKED になることを検証する土台。
fn xss_regression_test_rs_content() -> &'static str {
    r#"//! `xss-fixture-app::escape_html_content` の XSS 回帰テスト
//! （TASK-13.3c・#141 フィクスチャ、`core/tests/xss_escape.rs` 相当の
//! 代表ペイロードを移植した最小版）。

#[test]
fn script_tag_payload_is_escaped() {
    let escaped = xss_fixture_app::escape_html_content("<script>alert(1)</script>");
    assert_eq!(escaped, "&lt;script&gt;alert(1)&lt;/script&gt;");
    assert!(!escaped.contains('<'));
    assert!(!escaped.contains('>'));
}

#[test]
fn attribute_breakout_payload_is_escaped() {
    let escaped = xss_fixture_app::escape_html_content("\"><img src=x onerror=alert(1)>");
    assert!(!escaped.contains('"'));
    assert!(!escaped.contains('<'));
    assert!(!escaped.contains('>'));
}

#[test]
fn ampersand_is_escaped_first() {
    // & を最初に処理しないと後続の &lt; 等が二重エスケープされる
    // （core/src/escape.rs の仕様と同じ順序契約）。
    assert_eq!(xss_fixture_app::escape_html_content("&<"), "&amp;&lt;");
}
"#
}
