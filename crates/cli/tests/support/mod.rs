//! `cli/tests/negative_cases.rs`（TASK-13.5・#148）と
//! `cli/tests/xss_regression_link.rs`（TASK-13.3c・#141）、および
//! `cli/tests/impact_ambiguous.rs` / `cli/tests/impact_wasm_thin.rs`
//! （イシュー #293、`docs/design/impact-analysis-design.md` §3.4 観点 2・4 の
//! 独立 e2e 化）が共用する fixture 基盤。
//!
//! `negative_cases.rs` / `xss_regression_link.rs` はいずれも `fw gate`
//! （`cli/src/gate.rs`, TASK-13.3・#138）を実バイナリとして起動し、一時的な
//! 最小プロジェクト（`structure.toml` + virtual workspace + `deny.toml` +
//! `clippy.toml`）に対して欠陥を注入して BLOCKED まで到達させる e2e テスト
//! であり、フィクスチャの書き出し・起動・JSON レポート判定のヘルパー群が
//! 完全に重複するためここへ集約する。`impact_*.rs` は `fw impact`
//! （`cli/src/impact.rs`, TASK-13.2・#133〜#136）を対象とするため
//! `structure.toml`/`deny.toml`/`clippy.toml` を要さない別系統のフィクスチャ
//! ライタ（[`write_impact_workspace`]）を追加で持つ。
//! `tests/support/mod.rs` に置くことで cargo からは独立したテストバイナリ
//! として扱われず（`tests/` 直下の `.rs` のみが個別クレートになる）、
//! 各テストファイルから `mod support;` で参照できる。
//!
//! フィクスチャ自体の設計判断（`CARGO_TARGET_DIR` を専用化する理由・
//! `cargo generate-lockfile --offline` を要する理由等）は `negative_cases.rs`
//! 由来のためコメントもそのまま引き継ぐ。
//!
//! 本モジュールは `tests/` 配下の複数バイナリ（`negative_cases.rs` /
//! `xss_regression_link.rs` / `impact_ambiguous.rs` / `impact_wasm_thin.rs`）
//! から個別にコンパイルされ、各バイナリは公開ヘルパーの部分集合しか使わない
//! （例: `negative_cases.rs` は `write_xss_case_project` を使わない）。
//! dead_code lint はバイナリ単位で判定されるため、他方でのみ使われる関数が
//! もう一方のコンパイル単位では未使用警告になる。共有ヘルパーモジュールの
//! 典型的な事情のため、モジュール全体で `dead_code` を許可する。
//!
//! `run_fw`/`json_string_field`/`json_bool_field`/`json_array_contains_str`
//! は `cli/tests/scenarios/common.rs` の同名ヘルパーと実装が重複するが、
//! これは意図的な複製である（cargo のテストターゲットは独立コンパイル単位
//! のため、`tests/` 直下のバイナリから `tests/scenarios/` 配下のモジュール
//! を参照できない制約による。`scenarios/common.rs` 冒頭コメント・
//! `docs/design/scenario-regression-design.md` §4.4 が同じ方針を明文化済み）。

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// `fw` バイナリを任意のサブコマンド（`gate` / `impact` 等）で起動し、
/// (終了コード, stdout, stderr) を返す（`cli/tests/scenarios/common.rs::run_fw`
/// と同一方針の汎用ランナー。#293 で `impact_ambiguous.rs`/`impact_wasm_thin.rs`
/// が `fw impact <symbol>` を起動するために一般化した）。
///
/// `CARGO_TARGET_DIR` はフィクスチャ間で共有しない（`raw_html_lint_e2e.rs`
/// と同一方針）。self-hosted runner では `CARGO_TARGET_DIR=/cargo-target` が
/// プロセス環境に既定で設定されており、本テストの全フィクスチャは同名パッケージ
/// を再利用するため、これを継承したまま `cargo` を起動するとフィクスチャ間で
/// ビルドキャッシュ/フィンガープリントが衝突し、直前に生成した別フィクスチャの
/// チェック結果を誤って再利用してしまう（欠陥を注入したはずのケースが
/// 再コンパイルされず誤って PASS する偽陰性）。ここで `project_dir` 配下の
/// 専用 `target/` を明示指定し、継承された値を上書きすることで各フィクスチャを
/// 独立させる（`fw` から起動される `cargo` 子プロセスにも env は継承される
/// ため、これで `gate.rs`/`impact.rs` 側の変更は不要）。
///
/// `extra_args` はサブコマンド固有の追加引数（例: `fw impact <symbol>` の
/// `<symbol>`）を `--project` より前に渡す。
pub fn run_fw(subcommand: &str, extra_args: &[&str], project_dir: &Path) -> (i32, String, String) {
    run_fw_with_target_dir(
        subcommand,
        extra_args,
        project_dir,
        &project_dir.join("target"),
    )
}

/// [`run_fw`] の一般化版（イシュー #505）: `CARGO_TARGET_DIR` を
/// `project_dir/target` 固定ではなく呼び出し側が指定した `target_dir` に
/// 差し替えて `fw` バイナリを起動する。
///
/// 用途は `crates/cli/tests/new_gate_e2e.rs` の examples e2e 4 件
/// （`fw_new_example_*_output_passes_fw_gate`）が、パッケージ名の相互一意な
/// examples 間でのみビルドキャッシュ（crates.io 依存の再ビルド）を共有する
/// ためのもの。`negative_cases.rs` 等の欠陥注入フィクスチャ（同名パッケージを
/// 異内容で再利用する）は本関数を使わず引き続き [`run_fw`]（フィクスチャ
/// ごとに専用の `project_dir/target`）を使うこと。フィンガープリント衝突に
/// よる偽陰性（欠陥注入ケースの誤 PASS）を防ぐための区別であり、
/// `new_gate_e2e.rs` 側の安全性根拠は同ファイルの `example_shared_target_dir`
/// doc コメントを参照。
pub fn run_fw_with_target_dir(
    subcommand: &str,
    extra_args: &[&str],
    project_dir: &Path,
    target_dir: &Path,
) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_fw"))
        .arg(subcommand)
        .args(extra_args)
        .arg("--project")
        .arg(project_dir)
        .env("CARGO_TARGET_DIR", target_dir)
        .output()
        .expect("failed to spawn `fw` binary");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// `fw` バイナリを `gate --project <dir>` で起動する（`negative_cases.rs`/
/// `xss_regression_link.rs` 既存呼び出し元のシグネチャ・挙動を変えないための
/// 薄いラッパー。実体は [`run_fw`] へ委譲する）。
pub fn run_fw_gate(project_dir: &Path) -> (i32, String, String) {
    run_fw("gate", &[], project_dir)
}

/// [`run_fw_gate`] の一般化版（イシュー #505）: `CARGO_TARGET_DIR` を
/// `target_dir` に差し替えて `fw gate --project <dir>` を起動する。実体は
/// [`run_fw_with_target_dir`] へ委譲する（`run_fw_gate` と [`run_fw`] の関係と
/// 同じ薄いラッパー）。
pub fn run_fw_gate_with_target_dir(project_dir: &Path, target_dir: &Path) -> (i32, String, String) {
    run_fw_with_target_dir("gate", &[], project_dir, target_dir)
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

/// 一時プロジェクトを書き出すスクラッチルート。
///
/// cargo が `CARGO_TARGET_TMPDIR` を設定するのはテストバイナリの
/// **コンパイル時のみ**（Cargo Book「Environment variables Cargo sets for
/// crates」）であり、実行時の `std::env::var` 参照は常に失敗する。かつて
/// この事実誤認により実行時フォールバック（`std::env::temp_dir()` = `/tmp`）
/// が常用され、self-hosted runner の tmpfs を恒常的に消費していた
/// （イシュー #637）。既定はコンパイル時に確定する `env!("CARGO_TARGET_TMPDIR")`
/// （`<target>/tmp` 配下）を使い、`cargo clean` /
/// `.github/workflows/runner-maintenance.yml`（stale tmp 検査）の既存管理
/// 範囲に閉じる。実行時 env による明示上書き（特殊なテスト実行環境向け）は
/// 引き続き許容するが、既定経路としては期待しない。
/// `<target>/tmp` は cargo が実在を保証しないため `create_dir_all` で
/// 作成する（`negative_type_error.rs` と同一パターン、パストラバーサル
/// 対策の一環）。
pub fn scratch_root() -> PathBuf {
    let root = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_TARGET_TMPDIR")));
    let _ = fs::create_dir_all(&root);
    root
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
/// `fandhe_frontend_core::raw_html` エントリが存在することを fail-closed で前提とする
/// （欠落時は cargo clippy を起動する前に `lint` を failed とする）。本
/// フィクスチャはワークスペースルートの `clippy.toml` と同一ポリシーを配布
/// する `templates/default/clippy.toml` と同内容を複製し、`lint` チェックを
/// 実体化させる。
fn clippy_toml_content() -> &'static str {
    r#"disallowed-methods = [
    { path = "fandhe_frontend_core::raw_html", reason = "REQ-1 の唯一のエスケープ迂回経路。レビュー済みの呼び出しには `#[expect(clippy::disallowed_methods, reason = \"ESCAPE-REVIEWED: <根拠>\")]` を呼び出し文へ直接付与すること（`#[allow(...)]` によるブランケット抑止は禁止、docs/policy/raw-html-review-gate.md 参照）" },
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

/// `url_validation_check`（`cli/src/gate.rs`, イシュー #401）の U2/U3 負例・
/// 正例向け専用フィクスチャ。`write_case_project` との違いは
/// `[directories.core]`（`role = "core"`）を宣言し、`core/src/url.rs` に
/// `url_rs_content` を書き出す点にある（U2: allowlist ピン検査・U3: ガード
/// 呼び出し実在確認は `role = "core"` 宣言ディレクトリのみを対象とするため）。
///
/// ```text
/// <fixture>/
/// ├── structure.toml   ([directories.core], role = "core")
/// ├── Cargo.toml       (virtual workspace, members = ["core"])
/// ├── deny.toml
/// ├── clippy.toml
/// └── core/
///     ├── Cargo.toml   (name = "negative-fixture-core", 依存ゼロ)
///     └── src/
///         ├── lib.rs   (url モジュールのガード関数を呼ぶ最小コード。U3 のため)
///         └── url.rs   (url_rs_content)
/// ```
pub fn write_core_case_project(case_name: &str, url_rs_content: &str) -> ScratchProject {
    let dest = scratch_root().join(format!(
        "negative-cases-core-{case_name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dest);
    let core_src = dest.join("core").join("src");
    fs::create_dir_all(&core_src).expect("一時プロジェクトディレクトリの作成に失敗した");

    fs::write(
        dest.join("structure.toml"),
        r#"
[manifest]
version = 1

[directories.core]
role = "core"
crate = "negative-fixture-core"
description = "TASK-401 url_validation_check negative case fixture"
"#,
    )
    .expect("structure.toml の書き込みに失敗した");

    fs::write(
        dest.join("Cargo.toml"),
        "[workspace]\nmembers = [\"core\"]\nresolver = \"2\"\n",
    )
    .expect("workspace Cargo.toml の書き込みに失敗した");

    fs::write(dest.join("deny.toml"), deny_toml_content()).expect("deny.toml の書き込みに失敗した");

    fs::write(dest.join("clippy.toml"), clippy_toml_content())
        .expect("clippy.toml の書き込みに失敗した");

    fs::write(
        dest.join("core").join("Cargo.toml"),
        "[package]\nname = \"negative-fixture-core\"\nversion = \"0.1.0\"\nedition = \"2021\"\nlicense = \"MIT\"\npublish = false\n",
    )
    .expect("core/Cargo.toml の書き込みに失敗した");

    // lib.rs はガード 4 種すべてを呼ぶ最小コード（U3: ガード呼び出し実在
    // チェックが `url_rs_content` 自体の自己呼び出し（`is_safe_srcset` →
    // `is_safe_url`）に依存しすぎないよう、通常の呼び出し元コードを模す）。
    fs::write(
        core_src.join("lib.rs"),
        "mod url;\n\npub fn check(name: &str, value: &str) -> bool {\n    \
         if url::is_event_handler_attr(name) {\n        return false;\n    }\n    \
         if url::is_url_attr(name) {\n        return url::is_safe_url(value);\n    }\n    \
         url::is_safe_srcset(value)\n}\n",
    )
    .expect("core/src/lib.rs の書き込みに失敗した");

    fs::write(core_src.join("url.rs"), url_rs_content)
        .expect("core/src/url.rs の書き込みに失敗した");

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

// --- `fw impact` 向け JSON 抽出ヘルパ（イシュー #293） ---
//
// `cli/tests/scenarios/common.rs` の同名関数と実装が完全に一致するが、
// テストターゲット独立の制約による意図的な複製（ファイル冒頭コメント参照）。

/// `fw impact` の JSON レポート中の文字列フィールド `"<field>":"<value>"` を
/// 抽出する。専用 JSON パーサ依存を持ち込まず、`check_passed` と同じ
/// 「文字列走査による軽量抽出」方針を踏襲する（`cli` の外部依存ゼロを維持）。
/// フィールドが見つからない場合は `None`（欠落と空文字列を区別する）。
pub fn json_string_field(stdout: &str, field: &str) -> Option<String> {
    let needle = format!("\"{field}\":\"");
    let start = stdout.find(&needle)? + needle.len();
    let rest = &stdout[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// `fw impact` の JSON レポート中の真偽値フィールド
/// `"<field>":true`/`"<field>":false` を抽出する
/// （`requires_human_approval` / `ambiguous` 等）。フィールドが見つからない
/// 場合は `None`。
pub fn json_bool_field(stdout: &str, field: &str) -> Option<bool> {
    if stdout.contains(&format!("\"{field}\":true")) {
        Some(true)
    } else if stdout.contains(&format!("\"{field}\":false")) {
        Some(false)
    } else {
        None
    }
}

/// `fw impact` の JSON レポート中の文字列配列フィールド
/// `"<field>":["a","b"]` の要素をそのまま（クォート込みの生テキストとして）
/// 含むかを判定する軽量ヘルパ。`affected_crates` の含有検証に使う。
pub fn json_array_contains_str(stdout: &str, field: &str, expected_element: &str) -> bool {
    let needle = format!("\"{field}\":[");
    let Some(start) = stdout.find(&needle) else {
        return false;
    };
    let rest = &stdout[start + needle.len()..];
    let Some(end) = rest.find(']') else {
        return false;
    };
    let array_body = &rest[..end];
    array_body.contains(&format!("\"{expected_element}\""))
}

/// [`write_impact_workspace`] に渡す 1 lib クレート分のフィクスチャ内容
/// （イシュー #293）。
///
/// `fw impact` の入力契約（`cli/src/main.rs::run_impact` →
/// `metadata::fetch`（`cargo metadata --locked` なし）→ `impact::analyze`）は
/// `structure.toml` / `deny.toml` / `clippy.toml` を読まない（それらは
/// `fw gate` 専用）ため、`write_case_project`/`write_workspace_project` とは
/// 異なり `Cargo.toml`（virtual workspace）+ 各メンバー lib クレートのみの
/// 最小構成とする。
pub struct ImpactMemberSpec {
    /// ワークスペースルート直下のディレクトリ名（cargo workspace member 名と
    /// 一致させる）。
    pub dir: &'static str,
    /// `Cargo.toml` の `package.name`（`fw impact` の JSON レポート中の
    /// `affected_crates` にはこの値が現れる）。
    pub package_name: &'static str,
    /// このメンバーが path 依存する他メンバーの `dir` 一覧（同じ `members`
    /// スライス内に存在する必要がある）。
    pub path_deps: &'static [&'static str],
    /// `src/lib.rs` に書き出すソース全文。
    pub source: &'static str,
}

/// `fw impact` 専用の複数 lib クレートワークスペースを一意な一時プロジェクト
/// ディレクトリへ書き出す（イシュー #293）:
///
/// ```text
/// <scratch>/impact-<label>-<pid>-<nanos>/
/// ├── Cargo.toml       (virtual workspace, members = members の dir 一覧)
/// └── <member.dir>/
///     ├── Cargo.toml   (path_deps は `../<dep_dir>` として依存宣言)
///     └── src/lib.rs   (member.source)
/// ```
///
/// `structure.toml`/`deny.toml`/`clippy.toml` は書き出さない（`fw impact` は
/// gate 専用のこれらファイルを読まないため不要、上記 [`ImpactMemberSpec`]
/// doc コメント参照）。
///
/// `cargo generate-lockfile --offline` で `Cargo.lock` を生成する（path 依存
/// のみのため決定的・ネットワーク不要。`write_case_project` と同一方針）。
pub fn write_impact_workspace(label: &str, members: &[ImpactMemberSpec]) -> ScratchProject {
    let dest = scratch_root().join(format!(
        "impact-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dest);
    fs::create_dir_all(&dest).expect("一時プロジェクトディレクトリの作成に失敗した");

    let members_list = members
        .iter()
        .map(|m| format!("\"{}\"", m.dir))
        .collect::<Vec<_>>()
        .join(", ");
    fs::write(
        dest.join("Cargo.toml"),
        format!("[workspace]\nmembers = [{members_list}]\nresolver = \"2\"\n"),
    )
    .expect("workspace Cargo.toml の書き込みに失敗した");

    for m in members {
        let member_dir = dest.join(m.dir);
        let src_dir = member_dir.join("src");
        fs::create_dir_all(&src_dir).expect("member src ディレクトリの作成に失敗した");

        let mut cargo_toml = format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\nlicense = \"MIT\"\npublish = false\n",
            m.package_name
        );
        if !m.path_deps.is_empty() {
            cargo_toml.push_str("\n[dependencies]\n");
            for dep_dir in m.path_deps {
                let dep = members
                    .iter()
                    .find(|candidate| &candidate.dir == dep_dir)
                    .unwrap_or_else(|| panic!("path_deps が未知の dir `{dep_dir}` を参照している"));
                cargo_toml.push_str(&format!(
                    "{} = {{ path = \"../{}\" }}\n",
                    dep.package_name, dep_dir
                ));
            }
        }
        fs::write(member_dir.join("Cargo.toml"), cargo_toml)
            .expect("member Cargo.toml の書き込みに失敗した");

        fs::write(src_dir.join("lib.rs"), m.source)
            .expect("member src/lib.rs の書き込みに失敗した");
    }

    generate_lockfile(&dest);

    ScratchProject(dest)
}
