//! `templates/default/deny.toml`（TASK-4.1 / REQ-4）に対する回帰テスト。
//!
//! REQ-4 の受け入れ基準「標準プロジェクトテンプレートに `cargo-deny` 設定
//! （`bans`/`licenses` ポリシー）が既定同梱されていること」を機械的に保証する。
//! 併せて `[sources]` の `unknown-registry`/`unknown-git` が `"deny"` であること
//! （PoC-7 の `"warn"` から強化した差分）も固定し、AI エージェントによる
//! ポリシー緩和（許可リストの拡大・禁止クレートの削除・sources の deny 解除等）
//! を検知する安全網として機能する。
//!
//! 外部 TOML パーサは追加しない（REQ-3・xtask 外部依存ゼロ方針。依存追加の
//! ユーザー承認を得られない自動運転では「追加しない」側に倒す）。そのため
//! 検証は行に対する文字列一致の静的チェックに留める。
//!
//! cargo-deny がインストールされている環境では、実際に
//! `cargo deny check bans licenses sources --config templates/default/deny.toml`
//! を子プロセスで実行して成功することも確認する（advisories はネットワーク
//! アクセスを要するため対象外。PoC-7 の発見・deny.toml 内コメント参照）。
//! 未インストール環境ではこの実行チェックのみを明示的な警告出力つきで
//! スキップする（静的検証は常に実行し、`#[ignore]` は使わない）。

use std::path::PathBuf;
use std::process::Command;

/// workspace ルート（`xtask/` の親ディレクトリ）の絶対パスを返す。
///
/// `templates/default/deny.toml` は workspace ルート直下の `templates/`
/// にあるため、`CARGO_MANIFEST_DIR`（`xtask/`）から 1 階層親を辿る。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask/ には親ディレクトリ（workspace ルート）が存在する")
        .to_path_buf()
}

fn template_deny_toml_path() -> PathBuf {
    workspace_root().join("templates/default/deny.toml")
}

#[test]
fn template_deny_toml_exists() {
    let path = template_deny_toml_path();
    assert!(
        path.is_file(),
        "TASK-4.1 の成果物 templates/default/deny.toml が見つからない: {}",
        path.display()
    );
}

#[test]
fn template_deny_toml_declares_required_sections() {
    let contents = std::fs::read_to_string(template_deny_toml_path())
        .expect("templates/default/deny.toml の読み込みに失敗した");

    for section in ["[bans]", "[licenses]", "[sources]"] {
        assert!(
            contents.contains(section),
            "REQ-4 受け入れ基準（bans/licenses ポリシー同梱）に必要な \
             セクション {section} が templates/default/deny.toml に見つからない"
        );
    }
}

#[test]
fn template_deny_toml_licenses_allow_includes_baseline_licenses() {
    let contents = std::fs::read_to_string(template_deny_toml_path())
        .expect("templates/default/deny.toml の読み込みに失敗した");

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
fn template_deny_toml_bans_deny_includes_openssl_sys() {
    let contents = std::fs::read_to_string(template_deny_toml_path())
        .expect("templates/default/deny.toml の読み込みに失敗した");

    let bans_section = section_body(&contents, "[bans]");
    // コメント行（`#` 始まり）にも "openssl-sys" という文字列が出現するため、
    // 単純な部分文字列一致だと `{ name = "openssl-sys" }` エントリ自体を
    // 削除してもコメントの残存だけでテストが pass してしまう
    // （Bugbot 指摘: bans.deny に対する回帰防止テストの弱体化）。
    // コメント行を除外したうえで、実際の deny エントリ
    // `{ name = "openssl-sys" }` が存在することを行ベースで確認する。
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
fn template_deny_toml_sources_deny_unknown_registry_and_git() {
    let contents = std::fs::read_to_string(template_deny_toml_path())
        .expect("templates/default/deny.toml の読み込みに失敗した");

    let sources_section = section_body(&contents, "[sources]");
    for key in ["unknown-registry", "unknown-git"] {
        let line = sources_section
            .lines()
            .find(|line| line.trim_start().starts_with(key))
            .unwrap_or_else(|| panic!("[sources] に {key} が見つからない: {sources_section}"));
        assert!(
            line.contains("\"deny\""),
            "{key} が \"deny\" ではない（PoC-7 の \"warn\" から強化した \
             サプライチェーン対策が後退している）: {line}"
        );
    }
}

#[test]
fn cargo_deny_check_passes_on_template_config_when_available() {
    let cargo_deny_available = Command::new("cargo")
        .args(["deny", "--version"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !cargo_deny_available {
        eprintln!(
            "warning: cargo-deny が見つからないため \
             cargo_deny_check_passes_on_template_config_when_available の \
             実行チェックをスキップする（静的検証テストは実行済み）"
        );
        return;
    }

    // advisories はネットワークアクセスを要するため対象外とする
    // （PoC-7 の発見。deny.toml 内コメント参照）。固定引数のみを渡し、
    // 外部入力からコマンドライン引数を組み立てない（インジェクション対策）。
    let output = Command::new("cargo")
        .args([
            "deny",
            "check",
            "bans",
            "licenses",
            "sources",
            "--config",
            "templates/default/deny.toml",
        ])
        .current_dir(workspace_root())
        .output()
        .expect("cargo deny check の起動に失敗した");

    assert!(
        output.status.success(),
        "cargo deny check bans licenses sources が失敗した。\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// 指定した `[section]` ヘッダから次のトップレベルセクション（`[` 始まりの行）
/// 直前までの本文を抜き出す簡易ヘルパー。
///
/// 外部 TOML パーサを追加しない方針（REQ-3）のため、行ベースの単純な
/// 抽出に留める。本ファイルが検証する対象は自プロジェクトが管理する
/// 固定フォーマットの deny.toml のみであり、汎用 TOML パースは不要。
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
