//! `cli/tests/scenarios/` 共有ハーネス（TASK-13.4a・#144、設計文書
//! `docs/scenario-regression-design.md` §4.2/§4.3）。
//!
//! PoC-7 が検証した代表的改修シナリオ（バグ修正・UI 改善・機能追加、
//! `docs/spec/03-poc/ai-self-maintenance/scenarios/`）を製品 CLI（`fw`）に
//! 対する統合テストとして再現するための、フィクスチャ生成・`fw` 起動・
//! JSON フィールド抽出の共通処理を提供する。
//!
//! `cli/tests/negative_cases.rs`（TASK-13.5・#262）が確立したヘルメチックな
//! フィクスチャ生成パターン（`ScratchProject` Drop ガード・
//! `cargo generate-lockfile --offline`・フィクスチャごとの `CARGO_TARGET_DIR`
//! 分離・cargo-deny 有無の環境差吸収）をそのまま踏襲する。統合テストは
//! ターゲット単位で独立コンパイルされるため cargo クレート間でのコード共有は
//! できず、`negative_cases.rs` とロジックが重複するが、これは意図的な複製
//! （テストターゲット独立の制約によるもの）であり、二重管理を避けるための
//! 抽出先は用意しない。
//!
//! 本サブタスク（TASK-13.4a）ではベースライン smoke test
//! （`cli/tests/scenarios/main.rs`）のみがこのハーネスを利用する。後続
//! TASK-13.4b/c/d（#145〜#147）がシナリオ 1〜3 固有のフィクスチャ拡張・
//! `fw impact` JSON 検証にこのハーネスを利用する契約（設計文書 §4.4）。

#![allow(dead_code)] // ベースライン smoke test は一部のヘルパのみ使用する。残りは #145〜#147 が利用する契約。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// `write_scenario_project` が書き出した一時プロジェクトディレクトリを保持し、
/// スコープを抜けるタイミングで自身を削除するガード
/// （`negative_cases.rs::ScratchProject` と同一方針）。
pub struct ScenarioProject(PathBuf);

impl std::ops::Deref for ScenarioProject {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl Drop for ScenarioProject {
    fn drop(&mut self) {
        // 削除失敗（他プロセスによるロック等）はテスト結果の正当性に
        // 影響しないため、ベストエフォートとして無視する。
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// 一時プロジェクトを書き出すスクラッチルート。`CARGO_TARGET_TMPDIR`
/// （cargo がテストバイナリ実行時に設定する target 配下の一時ディレクトリ）が
/// あればそこに閉じ、未設定環境向けに OS 標準の一時領域へフォールバックする
/// （`negative_cases.rs` と同一パターン、パストラバーサル対策の一環）。
pub fn scratch_root() -> PathBuf {
    std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
}

/// ベースライン（正例）となる `app/src/main.rs` の内容。PoC-7
/// `target-project`（`Item` / `find_item`）相当の最小構成で、依存ゼロ・
/// clippy クリーン・`raw_html` 文字列を一切含まない
/// （`negative_cases.rs::baseline_main_rs` と同一内容。本サブタスクの
/// ベースライン smoke test 専用であり、シナリオ 1〜3 固有のフィクスチャ
/// 拡張（`server` クレート・ルート定義の追加等）は #145〜#147 が
/// このハーネスに追加する契約、設計文書 §4.2）。
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

/// 一意な一時プロジェクトディレクトリに以下を書き出す:
///
/// ```text
/// <fixture>/
/// ├── structure.toml   ([directories.app], role = "component")
/// ├── Cargo.toml       (virtual workspace, members = ["app"])
/// ├── deny.toml        (templates/default/deny.toml と同ポリシーの最小版)
/// ├── clippy.toml      (disallowed-methods: rws_core::raw_html)
/// └── app/
///     ├── Cargo.toml   (name = "scenario-fixture-app", 依存ゼロ)
///     └── src/main.rs  (main_rs_content)
/// ```
///
/// `cargo generate-lockfile --offline` で `Cargo.lock` を生成する（依存ゼロの
/// ため決定的・ネットワーク不要）。`fw gate` は `--locked` で `cargo`
/// サブコマンドを起動するため、ロックファイルなしでは各チェックがロック
/// ファイル欠落自体で failed になり、注入した欠陥とは無関係な失敗理由に
/// なってしまう（ケースの特定性を損なう）ため、ここで確実に用意する
/// （`negative_cases.rs::write_case_project` と同一方針）。
///
/// `scenario_name` はスクラッチディレクトリ名の一意化のみに使う（ファイル内容には
/// 影響しない）。
pub fn write_scenario_project(scenario_name: &str, main_rs_content: &str) -> ScenarioProject {
    let dest = scratch_root().join(format!(
        "scenario-{scenario_name}-{}-{}",
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
        r#"
[manifest]
version = 1

[directories.app]
role = "component"
crate = "scenario-fixture-app"
description = "TASK-13.4 scenario regression fixture"
"#,
    )
    .expect("structure.toml の書き込みに失敗した");

    fs::write(
        dest.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    )
    .expect("workspace Cargo.toml の書き込みに失敗した");

    // `templates/default/deny.toml` と同じ主要ポリシー（bans/licenses/sources）
    // を持つ最小版。`policy` チェックが `deny.toml` 実在確認の先で実際に
    // `cargo deny check bans licenses sources` を走らせられるようにする
    // （`negative_cases.rs` と同一内容）。
    fs::write(
        dest.join("deny.toml"),
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
"#,
    )
    .expect("deny.toml の書き込みに失敗した");

    fs::write(
        dest.join("app").join("Cargo.toml"),
        "[package]\nname = \"scenario-fixture-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\nlicense = \"MIT\"\npublish = false\n",
    )
    .expect("app/Cargo.toml の書き込みに失敗した");

    // イシュー #157/#263（`gate.rs::clippy_policy_check`）: `lint` チェックは
    // `project_dir` 直下の `clippy.toml` に `disallowed-methods` の
    // `rws_core::raw_html` エントリが存在することを fail-closed で前提とする
    // （欠落時は cargo clippy を起動する前に `lint` を failed とする）。
    fs::write(
        dest.join("clippy.toml"),
        r#"disallowed-methods = [
    { path = "rws_core::raw_html", reason = "REQ-1 の唯一のエスケープ迂回経路。レビュー済みの呼び出しには `#[expect(clippy::disallowed_methods, reason = \"ESCAPE-REVIEWED: <根拠>\")]` を呼び出し文へ直接付与すること（`#[allow(...)]` によるブランケット抑止は禁止、docs/raw-html-review-gate.md 参照）" },
]
"#,
    )
    .expect("clippy.toml の書き込みに失敗した");

    fs::write(app_src.join("main.rs"), main_rs_content).expect("main.rs の書き込みに失敗した");

    // 依存ゼロのためネットワークアクセスなしで決定的にロックファイルを生成できる。
    let lockfile_output = Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(&dest)
        .output()
        .expect("cargo generate-lockfile の起動に失敗した");
    assert!(
        lockfile_output.status.success(),
        "cargo generate-lockfile --offline に失敗した（フィクスチャ自体が壊れている）: {}",
        String::from_utf8_lossy(&lockfile_output.stderr)
    );

    ScenarioProject(dest)
}

/// 一意な部分文字列 `from` を `to` へちょうど 1 箇所だけ置換する。複数箇所・
/// 0 箇所にマッチした場合は panic し、フィクスチャのリファクタリングで
/// 注入前提が崩れたことをテスト失敗として顕在化させる
/// （`negative_cases.rs::replace_unique` と同一方針。シナリオ 1〜3 の
/// before/after 変更適用に #145〜#147 が使う契約）。
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

/// `fw` バイナリを任意のサブコマンド（`structure` / `gate` / `impact`）で
/// 起動し、(終了コード, stdout, stderr) を返す。`--project <dir>` を固定で
/// 付与し、`CARGO_TARGET_DIR` をフィクスチャ配下の専用ディレクトリへ上書きする
/// （`negative_cases.rs::run_fw_gate` と同一方針。self-hosted runner 等で
/// 継承された `CARGO_TARGET_DIR` をそのまま使うと、同名パッケージ
/// `scenario-fixture-app` を使う複数フィクスチャ間でビルドキャッシュ/
/// フィンガープリントが衝突し、直前のフィクスチャの結果を誤って再利用して
/// しまう偽陰性を招くため、フィクスチャごとに独立させる）。
///
/// `extra_args` はサブコマンド固有の追加引数（例: `fw impact <symbol>` の
/// `<symbol>`）を `--project` より前に渡す。
pub fn run_fw(subcommand: &str, extra_args: &[&str], project_dir: &Path) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_fw"))
        .arg(subcommand)
        .args(extra_args)
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
/// ため、`bool` ではなく `Option<bool>` を返す。`negative_cases.rs::check_passed`
/// と同一実装）。
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
/// （リポジトリ自身の CI には未導入、ローカル開発環境には導入済みという
/// 差を吸収するための補助関数。`negative_cases.rs::cargo_deny_available`
/// と同一実装。設計文書 §4.3 の環境差吸収方針を参照）。
pub fn cargo_deny_available() -> bool {
    Command::new("cargo")
        .args(["deny", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// `fw impact` の JSON レポート中の文字列フィールド
/// `"<field>":"<value>"` を抽出する。専用 JSON パーサ依存を持ち込まず、
/// `check_passed` と同じ「文字列走査による軽量抽出」方針を踏襲する
/// （`cli` の外部依存ゼロを維持、設計文書 §4.3）。フィールドが見つからない
/// 場合は `None`（欠落と空文字列を区別する）。
///
/// `breaking_risk`（`"high"`/`"medium"`/`"low"`）等、値に `"` を含まない
/// フィールドの抽出に使う。#145〜#147 が `fw impact` の JSON アサーションに
/// 利用する契約。
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
/// 含むかを判定する軽量ヘルパ。配列全体をパースせず、期待する要素
/// （例: 新設ルート `/search`）が配列内に文字列として現れるかどうかの
/// 部分一致検証に使う（`affected_routes` / `affected_crates` の非空・
/// 特定要素含有の検証、設計文書 §4.3）。
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
