//! リポジトリ直下 `deny.toml`（イシュー #372: `fw gate --project .` の自己適用用）
//! に対する回帰テスト。`xtask/tests/template_deny_config.rs` と同型のパターンを
//! 踏襲し、`templates/default/deny.toml` とのポリシー強度の乖離（緩和方向）を
//! 検出する。
//!
//! 背景: `fw gate` の `policy` チェックは `<project>/deny.toml` の存在を fail-closed
//! で要求する（欠落＝failed、docs/design/gate-design.md §3）。本リポジトリ自身へ
//! `fw gate --project .` を実行する際にもこの契約が適用されるため、
//! テンプレート同等ポリシーのリポジトリ直下 `deny.toml` を新設した
//! （イシュー #372）。本ファイルはその静的検証・弱体化の回帰検出を担う。
//!
//! 外部 TOML パーサは追加しない（REQ-3・xtask 外部依存ゼロ方針）。検証は行に
//! 対する文字列一致の静的チェックに留める（`template_deny_config.rs` と同じ
//! 制約・同じ理由）。

use std::path::PathBuf;
use std::process::Command;

/// workspace ルート（`xtask/` の親ディレクトリ）の絶対パスを返す。
/// リポジトリ直下 `deny.toml` は workspace ルート直下にあるため、
/// `CARGO_MANIFEST_DIR`（`xtask/`）から 1 階層親を辿る。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/xtask/ から 2 段上でワークスペースルートに到達する（イシュー #436）")
        .to_path_buf()
}

fn root_deny_toml_path() -> PathBuf {
    workspace_root().join("deny.toml")
}

#[test]
fn root_deny_toml_exists() {
    let path = root_deny_toml_path();
    assert!(
        path.is_file(),
        "イシュー #372 の成果物であるリポジトリ直下 deny.toml が見つからない \
         （fw gate --project . の policy チェックが常時 failed になる）: {}",
        path.display()
    );
}

#[test]
fn root_deny_toml_declares_required_sections() {
    let contents =
        std::fs::read_to_string(root_deny_toml_path()).expect("deny.toml の読み込みに失敗した");

    for section in ["[bans]", "[licenses]", "[sources]"] {
        assert!(
            contents.contains(section),
            "policy チェック（cargo deny check bans licenses sources）に \
             必要なセクション {section} がリポジトリ直下 deny.toml に見つからない"
        );
    }
}

#[test]
fn root_deny_toml_licenses_allow_includes_baseline_licenses() {
    let contents =
        std::fs::read_to_string(root_deny_toml_path()).expect("deny.toml の読み込みに失敗した");

    let licenses_section = section_body(&contents, "[licenses]");
    let allow_line = licenses_section
        .lines()
        .find(|line| line.trim_start().starts_with("allow"))
        .unwrap_or_else(|| panic!("[licenses] に allow が見つからない: {licenses_section}"));

    for required in ["MIT", "Apache-2.0"] {
        assert!(
            allow_line.contains(required),
            "licenses.allow が緩和され過ぎている可能性がある \
             （必須ライセンス {required} が含まれていない）: {allow_line}"
        );
    }
    assert!(
        !allow_line.trim_end_matches(['[', ']']).trim().is_empty(),
        "licenses.allow が空になっている（許可リスト方式の骨抜き）: {allow_line}"
    );
}

#[test]
fn root_deny_toml_bans_deny_includes_openssl_sys() {
    let contents =
        std::fs::read_to_string(root_deny_toml_path()).expect("deny.toml の読み込みに失敗した");

    let bans_section = section_body(&contents, "[bans]");
    // コメント行を除外したうえで、実際の deny エントリ
    // `{ name = "openssl-sys" }` が存在することを行ベースで確認する
    // （template_deny_config.rs と同じ落とし穴を避ける）。
    let has_active_entry = bans_section
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .any(|line| line.contains("name") && line.contains("\"openssl-sys\""));
    assert!(
        has_active_entry,
        "bans.deny から openssl-sys の禁止が外れている（サプライチェーン \
         対策の後退）: {bans_section}"
    );
}

#[test]
fn root_deny_toml_sources_deny_unknown_registry_and_git() {
    let contents =
        std::fs::read_to_string(root_deny_toml_path()).expect("deny.toml の読み込みに失敗した");

    let sources_section = section_body(&contents, "[sources]");
    for key in ["unknown-registry", "unknown-git"] {
        let line = sources_section
            .lines()
            .find(|line| line.trim_start().starts_with(key))
            .unwrap_or_else(|| panic!("[sources] に {key} が見つからない: {sources_section}"));
        assert!(
            line.contains("\"deny\""),
            "{key} が \"deny\" ではない（サプライチェーン対策が後退している）: {line}"
        );
    }
}

#[test]
fn cargo_deny_check_passes_on_root_config_when_available() {
    let cargo_deny_available = Command::new("cargo")
        .args(["deny", "--version"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !cargo_deny_available {
        eprintln!(
            "warning: cargo-deny が見つからないため \
             cargo_deny_check_passes_on_root_config_when_available の \
             実行チェックをスキップする（静的検証テストは実行済み）"
        );
        return;
    }

    // advisories はネットワークアクセスを要するため対象外とする
    // （root deny.toml 内コメント参照）。固定引数のみを渡し、外部入力から
    // コマンドライン引数を組み立てない（インジェクション対策）。
    let output = Command::new("cargo")
        .args(["deny", "--offline", "check", "bans", "licenses", "sources"])
        .current_dir(workspace_root())
        .output()
        .expect("cargo deny check の起動に失敗した");

    assert!(
        output.status.success(),
        "cargo deny check bans licenses sources がリポジトリ直下 deny.toml で \
         失敗した（fw gate --project . の policy チェックが BLOCKED になる）。\
         \nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// 指定した `[section]` ヘッダから次のトップレベルセクション（`[` 始まりの行）
/// 直前までの本文を抜き出す簡易ヘルパー（`template_deny_config.rs` と同一実装。
/// 外部 TOML パーサを追加しない方針のため重複を許容する — xtask 単一クレート内の
/// 2 テストファイル間の共有ヘルパーモジュール化は本イシューのスコープ外）。
fn section_body<'a>(contents: &'a str, header: &str) -> &'a str {
    let start = contents
        .find(header)
        .unwrap_or_else(|| panic!("セクション {header} が見つからない"));
    let after_header = &contents[start + header.len()..];
    let end = after_header
        .find("\n[")
        .map(|idx| idx + 1)
        .unwrap_or(after_header.len());
    &after_header[..end]
}
