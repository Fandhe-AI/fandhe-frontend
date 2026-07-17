//! `templates/default/.github/workflows/deny.yml`（TASK-4.2 / REQ-4）に対する回帰テスト。
//!
//! REQ-4 の受け入れ基準「禁止クレート（許可リスト外の依存）を追加した変更が
//! CI でブロックされること」を、テンプレート同梱ワークフローの静的検証と
//! 実行検証（cargo-deny 導入環境限定）の両面で保証する。
//! `templates/default/deny.toml` 自体の回帰テストは `template_deny_config.rs`
//! が担うため、本ファイルはワークフロー YAML の構造・fail-closed 設計・
//! PoC-7 `negative-banned-dependency` の自動化に専念する。
//!
//! 外部 YAML/TOML パーサは追加しない（REQ-3・xtask 外部依存ゼロ方針。
//! 依存追加のユーザー承認を得られない自動運転では「追加しない」側に倒す）。
//! そのため静的検証は行ベースの文字列一致に留める。

use std::path::PathBuf;
use std::process::Command;

/// workspace ルート（`xtask/` の親ディレクトリ）の絶対パスを返す。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask/ には親ディレクトリ（workspace ルート）が存在する")
        .to_path_buf()
}

fn workflow_path() -> PathBuf {
    workspace_root().join("templates/default/.github/workflows/deny.yml")
}

fn template_deny_toml_path() -> PathBuf {
    workspace_root().join("templates/default/deny.toml")
}

fn read_workflow() -> String {
    std::fs::read_to_string(workflow_path())
        .expect("templates/default/.github/workflows/deny.yml の読み込みに失敗した")
}

/// コメント行（`#` 始まり、行頭の空白は許容）を除いた実行行のみを返す。
///
/// YAML パーサを追加しない方針のため、コメント中の文字列を実チェック内容と
/// 誤認しないよう、行ベースの単純な除外で近似する。
fn non_comment_lines(contents: &str) -> Vec<&str> {
    contents
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect()
}

#[test]
fn template_deny_workflow_exists() {
    let path = workflow_path();
    assert!(
        path.is_file(),
        "TASK-4.2 の成果物 templates/default/.github/workflows/deny.yml が \
         見つからない: {}",
        path.display()
    );
}

#[test]
fn template_deny_workflow_declares_required_triggers() {
    let contents = read_workflow();
    assert!(
        contents.contains("pull_request"),
        "on: に pull_request トリガーが見つからない（PR での CI ブロックが \
         機能しない）"
    );
    assert!(
        contents.contains("push:"),
        "on: に push トリガーが見つからない"
    );
}

#[test]
fn template_deny_workflow_declares_minimal_permissions() {
    let contents = read_workflow();
    let has_read_only_permissions = non_comment_lines(&contents)
        .iter()
        .any(|line| line.trim() == "permissions: contents: read")
        || contents.contains("permissions:") && contents.contains("contents: read");
    assert!(
        has_read_only_permissions,
        "permissions: contents: read（最小権限）が見つからない"
    );
}

#[test]
fn template_deny_workflow_pins_action_refs_to_full_sha() {
    let contents = read_workflow();
    let uses_lines: Vec<&str> = non_comment_lines(&contents)
        .into_iter()
        .filter(|line| line.trim_start().starts_with("uses:") || line.contains("uses:"))
        .collect();

    assert!(
        !uses_lines.is_empty(),
        "uses: 行が見つからない（Action 参照がない構成になっている）"
    );

    for line in uses_lines {
        let reference = line
            .split("uses:")
            .nth(1)
            .expect("uses: の後に参照文字列がある")
            .trim();
        let sha = reference
            .rsplit('@')
            .next()
            .expect("uses: 参照は action@ref 形式である");
        assert!(
            sha.len() == 40 && sha.chars().all(|c| c.is_ascii_hexdigit()),
            "uses: 参照がフル SHA 固定になっていない（タグ・floating 参照の \
             混入によるサプライチェーンリスク）: {line}"
        );
    }
}

#[test]
fn template_deny_workflow_runs_bans_licenses_sources_gate() {
    let contents = read_workflow();
    let executable_contents = non_comment_lines(&contents).join("\n");

    assert!(
        executable_contents.contains("cargo deny check"),
        "cargo deny check の実行行が見つからない（ゲート本体が欠落している）"
    );
    for check in ["bans", "licenses", "sources"] {
        assert!(
            executable_contents.contains(check),
            "cargo deny check の対象から {check} が外れている（ポリシー \
             ゲートの弱体化）"
        );
    }
}

#[test]
fn template_deny_workflow_has_no_fail_open_escape_hatch() {
    let contents = read_workflow();
    // コメント行では forbidden ワードそのものを説明のために言及するため
    // （本ワークフロー冒頭のセキュリティ不変条件コメント参照）、実行行のみを
    // 対象に骨抜きの有無を検証する。
    let executable_contents = non_comment_lines(&contents).join("\n");
    assert!(
        !executable_contents.contains("continue-on-error"),
        "continue-on-error が設定されている（fail-closed 設計の骨抜き）"
    );
    assert!(
        !executable_contents.contains("|| true"),
        "|| true が使われている（fail-closed 設計の骨抜き）"
    );
}

#[test]
fn template_deny_workflow_installs_cargo_deny_with_locked_and_pinned_version() {
    let contents = read_workflow();
    let executable_contents = non_comment_lines(&contents).join("\n");

    assert!(
        executable_contents.contains("cargo install cargo-deny"),
        "cargo-deny のインストール行が見つからない"
    );
    assert!(
        executable_contents.contains("--locked"),
        "cargo install cargo-deny に --locked が付与されていない"
    );
    assert!(
        executable_contents.contains("--version"),
        "cargo install cargo-deny にバージョン固定（--version）が付与されて \
         いない"
    );
}

/// PoC-7 `negative-banned-dependency` ケースの自動化。
///
/// テンプレート `deny.toml` の `[bans].deny` に workspace グラフ上に確実に
/// 存在するクレート（`rws-core`）を追記した一時 config を生成し、
/// `cargo deny check bans` が非 0 終了かつ `banned` を含む出力で失敗する
/// ことを確認する。固定引数のみでプロセスを組み立て、外部入力から
/// コマンドライン引数を構成しない（インジェクション対策）。
///
/// 正例（テンプレート config での pass）は `template_deny_config.rs` の
/// `cargo_deny_check_passes_on_template_config_when_available` が担保済みの
/// ため、ここでは重複させない。
#[test]
fn cargo_deny_check_blocks_banned_dependency_when_available() {
    let cargo_deny_available = Command::new("cargo")
        .args(["deny", "--version"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !cargo_deny_available {
        eprintln!(
            "warning: cargo-deny が見つからないため \
             cargo_deny_check_blocks_banned_dependency_when_available の \
             実行チェックをスキップする（静的検証テストは実行済み）"
        );
        return;
    }

    let base_config = std::fs::read_to_string(template_deny_toml_path())
        .expect("templates/default/deny.toml の読み込みに失敗した");

    // workspace グラフ上に確実に存在する rws-core を bans.deny 配列へ追記した
    // 一時 config を作る。TOML パーサを使わず、`deny = [` 直後に行ベースで
    // 単純に挿入するのみに留める（REQ-3・xtask 外部依存ゼロ方針）。
    let augmented_config =
        base_config.replacen("deny = [", "deny = [\n    { name = \"rws-core\" },", 1);

    let temp_path =
        std::env::temp_dir().join("xtask-template-deny-workflow-negative-test-deny.toml");
    std::fs::write(&temp_path, augmented_config).expect("一時 deny.toml の書き込みに失敗した");

    let output = Command::new("cargo")
        .args([
            "deny",
            "check",
            "bans",
            "--config",
            temp_path
                .to_str()
                .expect("一時ファイルパスは有効な UTF-8 である"),
        ])
        .current_dir(workspace_root())
        .output()
        .expect("cargo deny check の起動に失敗した");

    let _ = std::fs::remove_file(&temp_path);

    assert!(
        !output.status.success(),
        "禁止クレート（rws-core）を追加した設定で cargo deny check bans が \
         成功してしまった（CI ブロック機構が機能していない）。\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let combined_output = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined_output.contains("banned") || combined_output.contains("rws-core"),
        "失敗理由が禁止クレート由来であることを出力から確認できなかった: \
         {combined_output}"
    );
}
