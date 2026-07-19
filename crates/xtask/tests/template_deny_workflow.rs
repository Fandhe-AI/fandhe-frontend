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
        .and_then(|p| p.parent())
        .expect("crates/xtask/ から 2 段上でワークスペースルートに到達する（イシュー #436）")
        .to_path_buf()
}

fn workflow_path() -> PathBuf {
    workspace_root().join("templates/default/.github/workflows/deny.yml")
}

fn template_deny_toml_path() -> PathBuf {
    workspace_root().join("templates/default/deny.toml")
}

fn ensure_gate_tools_path() -> PathBuf {
    workspace_root().join("tools/ci/ensure-gate-tools.sh")
}

fn cargo_deny_advisories_doc_path() -> PathBuf {
    workspace_root().join("docs/policy/cargo-deny-advisories.md")
}

fn read_ensure_gate_tools() -> String {
    std::fs::read_to_string(ensure_gate_tools_path())
        .expect("tools/ci/ensure-gate-tools.sh の読み込みに失敗した")
}

fn read_cargo_deny_advisories_doc() -> String {
    std::fs::read_to_string(cargo_deny_advisories_doc_path())
        .expect("docs/policy/cargo-deny-advisories.md の読み込みに失敗した")
}

/// `pattern` に続くバージョン文字列（`X.Y.Z` 形式）を抽出する。
///
/// イシュー #314: cargo-deny の pin の正は `tools/ci/ensure-gate-tools.sh` の
/// `CARGO_DENY_VERSION` のみとし、テンプレート・docs の pin 値がそこから
/// ドリフトしていないことを本ファイルのテストで強制する（手動同期に頼らない）。
/// 外部 YAML/TOML パーサは追加しない方針（REQ-3）のため、行ベースの単純な
/// 前方一致抽出に留める。
///
/// 抽出できなかった場合は空文字列ではなく panic する。空文字列同士の比較で
/// テストが誤って pass する（vacuous pass）ことを避けるため。
fn extract_version_after(contents: &str, pattern: &str) -> String {
    let start = contents.find(pattern).unwrap_or_else(|| {
        panic!("パターン `{pattern}` が見つからない: 抽出元のドリフト検知が機能しない")
    });
    let rest = &contents[start + pattern.len()..];
    let version: String = rest
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    assert!(
        !version.is_empty() && version.contains('.'),
        "パターン `{pattern}` の直後からバージョン文字列を抽出できなかった \
         （空文字列同士の比較による vacuous pass を避けるため、空値は許容しない）"
    );
    version
}

/// `pattern` に続く SHA256 チェックサム文字列（16 進数 64 文字）を抽出する。
fn extract_sha256_after(contents: &str, pattern: &str) -> String {
    let start = contents.find(pattern).unwrap_or_else(|| {
        panic!("パターン `{pattern}` が見つからない: 抽出元のドリフト検知が機能しない")
    });
    let rest = &contents[start + pattern.len()..];
    let sha: String = rest.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
    assert!(
        sha.len() >= 32,
        "パターン `{pattern}` の直後から SHA256 チェックサムを抽出できなかった \
         （空値・極端に短い値は許容しない）"
    );
    sha
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
    let non_comment_owned: Vec<String> = non_comment_lines(&contents)
        .into_iter()
        .map(str::to_owned)
        .collect();

    // `permissions:` は行頭（トップレベルキー）で始まる想定。その直後から、
    // 次のトップレベルキー（行頭にインデントのない行）が現れるまでを
    // 「permissions ブロック」とみなし、ブロック内の記述のみを検証する。
    // これにより `contents: read` と `contents: write` が別ブロックに
    // 独立して存在するケースを「関連なし」として誤 pass しないようにする。
    let permissions_index = non_comment_owned
        .iter()
        .position(|line| line.trim_start() == "permissions:" && !line.starts_with(' '))
        .expect("トップレベルの permissions: が見つからない");

    let block: Vec<&str> = non_comment_owned[permissions_index + 1..]
        .iter()
        .take_while(|line| line.starts_with(' ') || line.trim().is_empty())
        .map(String::as_str)
        .collect();
    let block_contents = block.join("\n");

    assert!(
        block_contents.contains("contents: read"),
        "permissions: ブロック内に contents: read（最小権限）が見つからない: \
         {block_contents}"
    );
    assert!(
        !block_contents.contains("contents: write"),
        "permissions: ブロック内に contents: write が含まれている（最小権限の \
         逸脱）: {block_contents}"
    );
}

#[test]
fn template_deny_workflow_pins_action_refs_to_full_sha() {
    let contents = read_workflow();
    let uses_lines: Vec<&str> = non_comment_lines(&contents)
        .into_iter()
        .filter(|line| line.contains("uses:"))
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

/// `run:` ステップの実コマンド部分のみを抽出する。
///
/// `non_comment_lines` はコメント行を除くだけで `name:` のような他フィールドを
/// 含んだままにするため、`name: cargo deny check bans/licenses/sources` の
/// ように *説明のためだけの文字列* を `run:` の実コマンドと誤認識しうる
/// （Bugbot 指摘: name: 行の文言だけでテストが通り、run: を空にする骨抜きを
/// 検知できない）。ここでは `run:` キー（インライン形式・`|`/`>` ブロック
/// スカラー形式の両方）のみを対象にし、インデントで一致するブロック内容を
/// 追跡することで、実行コマンド以外の行を混入させない。
fn run_command_contents(contents: &str) -> String {
    let lines = non_comment_lines(contents);
    let mut result = String::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        if let Some(rest) = trimmed.strip_prefix("run:") {
            let rest = rest.trim();
            if rest.is_empty() || rest == "|" || rest == "|-" || rest == ">" || rest == ">-" {
                // ブロックスカラー形式: 後続の、より深くインデントされた
                // （または空の）行を実コマンドとして取り込む。
                i += 1;
                while i < lines.len() {
                    let block_line = lines[i];
                    let block_trimmed = block_line.trim_start();
                    let block_indent = block_line.len() - block_trimmed.len();
                    if block_trimmed.is_empty() || block_indent > indent {
                        result.push_str(block_line);
                        result.push('\n');
                        i += 1;
                    } else {
                        break;
                    }
                }
                continue;
            }
            // インライン形式: `run: <command>`
            result.push_str(rest);
            result.push('\n');
        }
        i += 1;
    }
    result
}

#[test]
fn template_deny_workflow_runs_bans_licenses_sources_gate() {
    let contents = read_workflow();
    // `name:` フィールドの説明文言（例:
    // 「Run cargo deny check (bans / licenses / sources)」）を誤って
    // ゲート実行の証拠と扱わないよう、`run:` の実コマンドのみを検証する。
    let run_contents = run_command_contents(&contents);

    assert!(
        run_contents.contains("cargo deny check"),
        "cargo deny check の実行行（run:）が見つからない（ゲート本体が \
         欠落している）"
    );
    for check in ["bans", "licenses", "sources"] {
        assert!(
            run_contents.contains(check),
            "cargo deny check の run: コマンドの対象から {check} が \
             外れている（ポリシーゲートの弱体化。name: の説明文言だけを \
             書き換えて run: を骨抜きにする回帰を検知するための検査）"
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

/// イシュー #314: cargo-deny の導入はテンプレート単体でも
/// `tools/ci/ensure-gate-tools.sh`（本リポジトリ自身の CI）と同一の
/// 「バージョン固定 + SHA256 検証済みプリビルトバイナリ」パターンに統一する
/// （`cargo install` によるソースからの任意最新版コンパイルは行わない）。
/// 旧パターン（`cargo install cargo-deny --locked --version ...`）への
/// 回帰も検知する。
#[test]
fn template_deny_workflow_installs_cargo_deny_with_pinned_prebuilt_binary() {
    let contents = read_workflow();
    let executable_contents = non_comment_lines(&contents).join("\n");

    assert!(
        !executable_contents.contains("cargo install cargo-deny"),
        "cargo-deny が旧パターン（cargo install によるソースからのコンパイル）で \
         導入されている（イシュー #314: プリビルトバイナリ + SHA256 検証への \
         統一に回帰している）"
    );
    assert!(
        executable_contents.contains("CARGO_DENY_VERSION="),
        "cargo-deny のバージョン pin（CARGO_DENY_VERSION）が見つからない"
    );
    assert!(
        executable_contents.contains("CARGO_DENY_SHA256="),
        "cargo-deny の SHA256 チェックサム pin（CARGO_DENY_SHA256）が見つからない"
    );
    assert!(
        executable_contents.contains("sha256sum -c"),
        "cargo-deny アーカイブの SHA256 検証（sha256sum -c）が見つからない \
         （検証なしのダウンロード実行はサプライチェーン対策の骨抜き）"
    );
    assert!(
        executable_contents.contains("github.com/EmbarkStudios/cargo-deny/releases/download"),
        "cargo-deny の取得元が公式リリース URL になっていない"
    );
}

/// イシュー #314: cargo-deny の pin 値の正は
/// `tools/ci/ensure-gate-tools.sh` の `CARGO_DENY_VERSION` /
/// `CARGO_DENY_SHA256` のみとする。テンプレート・docs 側の pin 値が
/// そこから乖離（ドリフト）していないことを `cargo test -p xtask` / CI で
/// 強制検知し、「1 箇所の変更で全ワークフローに波及する」ことを担保する。
#[test]
fn cargo_deny_version_pin_matches_ensure_gate_tools_across_template_and_docs() {
    let ensure_gate_tools = read_ensure_gate_tools();
    let template = read_workflow();
    let advisories_doc = read_cargo_deny_advisories_doc();

    let canonical_version = extract_version_after(&ensure_gate_tools, "CARGO_DENY_VERSION=\"");
    let canonical_sha256 = extract_sha256_after(&ensure_gate_tools, "CARGO_DENY_SHA256=\"");

    let template_version = extract_version_after(&template, "CARGO_DENY_VERSION=\"");
    let template_sha256 = extract_sha256_after(&template, "CARGO_DENY_SHA256=\"");
    assert_eq!(
        template_version, canonical_version,
        "templates/default/.github/workflows/deny.yml の CARGO_DENY_VERSION \
         ({template_version}) が tools/ci/ensure-gate-tools.sh の pin \
         ({canonical_version}) からドリフトしている"
    );
    assert_eq!(
        template_sha256, canonical_sha256,
        "templates/default/.github/workflows/deny.yml の CARGO_DENY_SHA256 \
         が tools/ci/ensure-gate-tools.sh の pin からドリフトしている"
    );

    // docs/policy/cargo-deny-advisories.md のサンプルワークフロー（第 5 節）は
    // テンプレートと同一の CARGO_DENY_VERSION / CARGO_DENY_SHA256 変数を
    // コード例として埋め込んでいる。同じアンカー文字列で抽出する。
    let doc_version = extract_version_after(&advisories_doc, "CARGO_DENY_VERSION=\"");
    let doc_sha256 = extract_sha256_after(&advisories_doc, "CARGO_DENY_SHA256=\"");
    assert_eq!(
        doc_version, canonical_version,
        "docs/policy/cargo-deny-advisories.md の CARGO_DENY_VERSION \
         ({doc_version}) が tools/ci/ensure-gate-tools.sh の pin \
         ({canonical_version}) からドリフトしている"
    );
    assert_eq!(
        doc_sha256, canonical_sha256,
        "docs/policy/cargo-deny-advisories.md の CARGO_DENY_SHA256 が \
         tools/ci/ensure-gate-tools.sh の pin からドリフトしている"
    );
}

/// PoC-7 `negative-banned-dependency` ケースの自動化。
///
/// テンプレート `deny.toml` の `[bans].deny` に workspace グラフ上に確実に
/// 存在するクレート（`fandhe-frontend-core`）を追記した一時 config を生成し、
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

    // workspace グラフ上に確実に存在する fandhe-frontend-core を bans.deny 配列へ追記した
    // 一時 config を作る。TOML パーサを使わず、`deny = [` 直後に行ベースで
    // 単純に挿入するのみに留める（REQ-3・xtask 外部依存ゼロ方針）。
    let augmented_config = base_config.replacen(
        "deny = [",
        "deny = [\n    { name = \"fandhe-frontend-core\" },",
        1,
    );

    // プロセス ID を付与し、並列テスト実行時の一時ファイル名衝突を避ける。
    let temp_path = std::env::temp_dir().join(format!(
        "xtask-template-deny-workflow-negative-test-deny-{}.toml",
        std::process::id()
    ));
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
        "禁止クレート（fandhe-frontend-core）を追加した設定で cargo deny check bans が \
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
        combined_output.contains("banned") || combined_output.contains("fandhe-frontend-core"),
        "失敗理由が禁止クレート由来であることを出力から確認できなかった: \
         {combined_output}"
    );
}
