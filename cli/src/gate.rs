//! `fw gate`: REQ-13 の第 4 要素「検証・制約の強制」を製品化する検証ゲート
//! （TASK-13.3, #138, 親 PoC-7 `cmd_gate` の Rust 移植）。
//!
//! [`crate::structure`]（TASK-13.1）が定義する `structure.toml` を唯一の情報源
//! として宣言クレート・ディレクトリを求め、5 チェック（型チェック・既定エスケープ
//! 検査・lint・テスト・依存ポリシー）を実行し、集約結果を JSON で stdout へ出力する。
//! AI 自己保守フック・CI からは本サブコマンドの終了コード（0 = PASS / 1 = BLOCKED /
//! 2 = 使用法エラー）と JSON の `gate_result` を照合し、変更適用の可否を判断する
//! 契約とする（`main.rs` 冒頭 doc コメントと同じ「黙示的成功を返さない」契約を
//! 本モジュールでも維持する）。
//!
//! セキュリティ不変条件:
//! - 外部コマンドは [`std::process::Command`] に引数配列で渡し、シェル文字列連結を
//!   行わない（security.md A03）。
//! - `structure.toml` 読み込み・パース失敗、`deny.toml` 欠落、外部コマンドの起動
//!   失敗はすべて「そのチェック failed」として扱い、スキップ・黙示的 PASS に
//!   倒さない（security.md A05, fail-closed）。
//! - `default_escape_check` は REQ-1（既定エスケープ）の唯一の許容迂回経路である
//!   `raw_html()` の呼び出しを、`ESCAPE-REVIEWED:` マーカーが同一行・直前行に
//!   ない限り違反として報告する（security.md A08）。

use crate::json_out::quoted;
use crate::structure::{self, Role, StructureManifest};
use std::path::Path;
use std::process::Command;

/// コマンド出力を JSON へ格納する際の丸め上限（末尾からの文字数）。
/// 肥大化防止・秘密情報の意図しない大量転記防止（security.md A09）。
const OUTPUT_TRUNCATE_CHARS: usize = 4000;

/// 1 チェックの結果。PoC-7 互換の JSON 形状（`name`/`passed`/`output`）を保つ。
pub(crate) struct GateCheck {
    pub(crate) name: &'static str,
    pub(crate) passed: bool,
    pub(crate) output: String,
}

/// ゲート全体の集約結果。
pub(crate) struct GateReport {
    pub(crate) checks: Vec<GateCheck>,
    pub(crate) gate_result: &'static str,
    pub(crate) action: String,
}

/// 外部コマンド起動を注入可能にする境界（テストで実プロセスを起動せずに
/// 集約ロジック・JSON 組み立てを検証するための抽象化）。
///
/// 実装は [`RealCommandRunner`] のみを本番経路で用いる。
trait CommandRunner {
    /// `cwd` で `program args...` を実行し、成功可否・stdout+stderr 結合出力を返す。
    /// 起動自体に失敗した場合（バイナリ不在等）は `Ok` の `success: false` として
    /// 返す（呼び出し元が `Result` の `Err` 分岐を用意せずに fail-closed 集約できる
    /// ようにする）。
    fn run(&self, program: &str, args: &[&str], cwd: &Path) -> (bool, String);
}

struct RealCommandRunner;

impl CommandRunner for RealCommandRunner {
    fn run(&self, program: &str, args: &[&str], cwd: &Path) -> (bool, String) {
        match Command::new(program).args(args).current_dir(cwd).output() {
            Ok(output) => {
                let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
                combined.push_str(&String::from_utf8_lossy(&output.stderr));
                (output.status.success(), combined)
            }
            Err(e) => (false, format!("failed to launch `{program}`: {e}")),
        }
    }
}

/// `fw gate` 本体。`main.rs` の `run()` からディスパッチされるエントリポイント。
///
/// 1. `--project` 引数を解決（[`crate::parse_project_arg`] を再利用、`structure` と
///    同一の使用法エラー規約）
/// 2. `<project>/structure.toml` を [`structure::load`] + [`StructureManifest::validate`]
///    で読み込む。失敗時は即 BLOCKED（fail-closed。マニフェストが読めない時点で
///    宣言クレート一覧が定まらず、以降のチェックが無意味になるため）
/// 3. 5 チェックをすべて実行（早期打ち切りしない。AI エージェントが一括修正できる
///    よう全違反を報告する PoC-7 の方針を踏襲）
/// 4. JSON レポートを stdout へ出力し、`gate_result` に応じた終了コードを返す
pub(crate) fn run_gate(args: &[String]) -> i32 {
    let project_dir = match crate::parse_project_arg(args) {
        Ok(dir) => dir,
        Err(()) => {
            eprintln!("fw gate: usage: fw gate [--project <dir>]");
            return 2;
        }
    };

    let manifest_path = project_dir.join("structure.toml");
    let manifest = match structure::load(&manifest_path) {
        Ok(m) => m,
        Err(e) => {
            let report = GateReport {
                checks: vec![GateCheck {
                    name: "structure_manifest",
                    passed: false,
                    output: format!("failed to load structure.toml: {e}"),
                }],
                gate_result: "BLOCKED",
                action: "fix structure.toml and re-run `fw gate`".to_string(),
            };
            println!("{}", render_report(&report));
            eprintln!("fw gate: BLOCKED (structure.toml could not be loaded: {e})");
            return 1;
        }
    };

    if let Err(errors) = manifest.validate() {
        let output = errors
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        let report = GateReport {
            checks: vec![GateCheck {
                name: "structure_manifest",
                passed: false,
                output,
            }],
            gate_result: "BLOCKED",
            action: "fix structure.toml and re-run `fw gate`".to_string(),
        };
        println!("{}", render_report(&report));
        eprintln!("fw gate: BLOCKED (structure.toml failed internal validation)");
        return 1;
    }

    let report = run_all_checks(&manifest, &project_dir, &RealCommandRunner);
    println!("{}", render_report(&report));
    if report.gate_result == "PASS" {
        0
    } else {
        eprintln!("fw gate: BLOCKED (see JSON report above for failing checks)");
        1
    }
}

/// 宣言クレート名一覧（`crate` を持つディレクトリのみ）。`cargo` 系チェックの
/// `-p` 引数に使う。
fn declared_crate_names(manifest: &StructureManifest) -> Vec<&str> {
    manifest
        .directories
        .iter()
        .filter_map(|d| d.crate_name.as_deref())
        .collect()
}

/// 5 チェックを実行して [`GateReport`] を組み立てる（実プロセス起動を伴う本番経路）。
///
/// テストからは `runner` に実行を伴わないフェイクを注入して集約ロジックのみを
/// 検証する（実プロセス起動なしのテスト容易性、計画 §3.3）。
fn run_all_checks(
    manifest: &StructureManifest,
    project_dir: &Path,
    runner: &dyn CommandRunner,
) -> GateReport {
    let crates = declared_crate_names(manifest);

    let checks = vec![
        run_cargo_check(runner, project_dir, &crates),
        default_escape_check(manifest, project_dir),
        run_cargo_clippy(runner, project_dir, &crates),
        run_cargo_test(runner, project_dir, &crates),
        policy_check(runner, project_dir),
    ];

    aggregate(checks)
}

/// 全チェック通過 → PASS、1 件でも不合格 → BLOCKED（起動失敗も不合格扱い、
/// fail-closed）。
fn aggregate(checks: Vec<GateCheck>) -> GateReport {
    let all_passed = checks.iter().all(|c| c.passed);
    if all_passed {
        GateReport {
            checks,
            gate_result: "PASS",
            action: "all checks passed; changes may proceed".to_string(),
        }
    } else {
        GateReport {
            checks,
            gate_result: "BLOCKED",
            action: "fix the reported failing checks and re-run `fw gate`".to_string(),
        }
    }
}

/// 末尾 [`OUTPUT_TRUNCATE_CHARS`] 文字に丸める（肥大化防止・security.md A09）。
fn truncate_output(output: &str) -> String {
    let char_count = output.chars().count();
    if char_count <= OUTPUT_TRUNCATE_CHARS {
        output.to_string()
    } else {
        output
            .chars()
            .skip(char_count - OUTPUT_TRUNCATE_CHARS)
            .collect()
    }
}

/// `crates` を対象に `-p <crate>` を連ねて外部コマンドを実行する共通ヘルパー。
/// `--locked` はロックファイル逸脱（依存すり替え）検出のため常に付与する
/// （security.md A06）。
fn run_locked_cargo_subcommand(
    runner: &dyn CommandRunner,
    project_dir: &Path,
    name: &'static str,
    subcommand_args: &[&str],
    crates: &[&str],
) -> GateCheck {
    let mut args: Vec<&str> = subcommand_args.to_vec();
    for c in crates {
        args.push("-p");
        args.push(c);
    }
    let (passed, output) = runner.run("cargo", &args, project_dir);
    GateCheck {
        name,
        passed,
        output: truncate_output(&output),
    }
}

fn run_cargo_check(runner: &dyn CommandRunner, project_dir: &Path, crates: &[&str]) -> GateCheck {
    run_locked_cargo_subcommand(
        runner,
        project_dir,
        "type_check",
        &["check", "--locked"],
        crates,
    )
}

fn run_cargo_clippy(runner: &dyn CommandRunner, project_dir: &Path, crates: &[&str]) -> GateCheck {
    // `--locked` はロックファイル逸脱（依存すり替え）検出のため `type_check` /
    // `test`（`run_locked_cargo_subcommand`）と同様に常に付与する
    // （security.md A06。Bugbot 指摘: PR #261 #2 — `lint` だけ `--locked` を
    // 欠くと依存差し替え検知の抜け道になり得る）。
    // `-- -D warnings` は cargo 引数の後段（サブコマンド固有引数)として渡す
    // （coding-rust.md: `cargo clippy -- -D warnings` を通す規約と同一コマンド）。
    let mut args: Vec<&str> = vec!["clippy", "--locked"];
    for c in crates {
        args.push("-p");
        args.push(c);
    }
    args.push("--");
    args.push("-D");
    args.push("warnings");
    let (passed, output) = runner.run("cargo", &args, project_dir);
    GateCheck {
        name: "lint",
        passed,
        output: truncate_output(&output),
    }
}

fn run_cargo_test(runner: &dyn CommandRunner, project_dir: &Path, crates: &[&str]) -> GateCheck {
    run_locked_cargo_subcommand(runner, project_dir, "test", &["test", "--locked"], crates)
}

/// `deny.toml` の存在確認（TASK-4.1 との接続点）→ `cargo deny check bans licenses
/// sources` を実行する。`advisories` はネットワーク前提でオフラインゲート対象外
/// とする（`templates/default/deny.toml` 冒頭コメント・`docs/cargo-deny-advisories.md`
/// 参照）。
///
/// `deny.toml` が存在しない場合は cargo-deny を起動せず即 failed とする
/// （fail-closed。ポリシー設定が存在しないプロジェクトを「ポリシー違反なし」と
/// 誤認させない）。
fn policy_check(runner: &dyn CommandRunner, project_dir: &Path) -> GateCheck {
    let deny_toml = project_dir.join("deny.toml");
    if !deny_toml.is_file() {
        return GateCheck {
            name: "policy",
            passed: false,
            output: format!(
                "deny.toml not found at {} (see templates/default/deny.toml)",
                deny_toml.display()
            ),
        };
    }
    let (passed, output) = runner.run(
        "cargo",
        &["deny", "check", "bans", "licenses", "sources"],
        project_dir,
    );
    GateCheck {
        name: "policy",
        passed,
        output: truncate_output(&output),
    }
}

/// `raw_html` 呼び出しの後続 1 文字を確認するための状態機械の最小走査（正規表現
/// クレートを使わず手書きで判定する。`cli` 外部依存ゼロ方針）。
///
/// `line` 中の `raw_html` の各出現位置について、直後の空白（半角スペース・タブ）を
/// 読み飛ばした先が `(` であれば呼び出しとみなす。コメント内の出現も検出対象と
/// する（偽陽性は許容・偽陰性は不許容の保守側実装、計画 §3.2）。
fn line_has_raw_html_call(line: &str) -> bool {
    let bytes = line.as_bytes();
    let needle = b"raw_html";
    let mut start = 0;
    while let Some(rel) = find_subslice(&bytes[start..], needle) {
        let mut i = start + rel + needle.len();
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'(' {
            return true;
        }
        start += rel + 1;
        if start >= bytes.len() {
            break;
        }
    }
    false
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

const ESCAPE_REVIEWED_MARKER: &str = "ESCAPE-REVIEWED:";

/// `role = "core"` 以外の宣言ディレクトリの `src/` 配下 `*.rs`（`tests/` は
/// 走査対象外。PoC-7 の粒度を維持し、テストコード内の `raw_html()` 利用は
/// TASK-13.5 以降の負例回帰テストの対象とする限界を持つ）を走査し、
/// `ESCAPE-REVIEWED:` マーカー（同一行または直前行）を伴わない `raw_html()`
/// 呼び出しを違反として `file:line` 付きで列挙する（REQ-1 の唯一の迂回経路を
/// 明示レビュー済み宣言に限定する契約、security.md A08）。
fn default_escape_check(manifest: &StructureManifest, project_dir: &Path) -> GateCheck {
    let mut violations: Vec<String> = Vec::new();

    for dir in &manifest.directories {
        if dir.role == Role::Core {
            continue;
        }
        let src_dir = project_dir.join(&dir.name).join("src");
        if !src_dir.is_dir() {
            continue;
        }
        scan_dir_for_violations(&src_dir, &mut violations);
    }

    violations.sort();
    let passed = violations.is_empty();
    let output = if passed {
        "no unreviewed raw_html() calls found".to_string()
    } else {
        truncate_output(&violations.join("\n"))
    };
    GateCheck {
        name: "default_escape_check",
        passed,
        output,
    }
}

/// `dir` 配下（再帰）の `*.rs` ファイルを走査する。I/O エラー（読み取り不可等）は
/// 違反として計上せず黙って読み飛ばす想定外パスとし、スキャナ自体の堅牢性を
/// 優先する（`fw gate` 全体としては他チェックの failed で fail-closed が働く）。
///
/// シンボリックリンク（ディレクトリ・ファイルいずれも）は辿らず無条件にスキップ
/// する。`path.is_dir()`（メタデータ経由でリンクを辿る）ではなく
/// `DirEntry::file_type()` の `is_symlink()` を明示チェックすることで、
/// 自己参照リンクによる無限再帰（fail-closed の実行自体を阻害する DoS）と、
/// プロジェクト外を指すリンクを辿ってのパストラバーサル（`.rs` ファイル内容が
/// 絶対パス付きで JSON レポートへ漏えいする経路）を防ぐ。`cli/src/routes.rs`
/// の `list_rs_files_inner`（レビュー指摘 #127 対応）と同一方針（OWASP A01/A05）。
fn scan_dir_for_violations(dir: &Path, violations: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            scan_dir_for_violations(&path, violations);
        } else if file_type.is_file() && path.extension().and_then(|e| e.to_str()) == Some("rs") {
            scan_file_for_violations(&path, violations);
        }
    }
}

fn scan_file_for_violations(path: &Path, violations: &mut Vec<String>) {
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    let lines: Vec<&str> = content.lines().collect();
    for (idx, line) in lines.iter().enumerate() {
        if !line_has_raw_html_call(line) {
            continue;
        }
        let reviewed_here = line.contains(ESCAPE_REVIEWED_MARKER);
        let reviewed_prev = idx > 0 && lines[idx - 1].contains(ESCAPE_REVIEWED_MARKER);
        if reviewed_here || reviewed_prev {
            continue;
        }
        violations.push(format!(
            "{}:{}: unreviewed raw_html() call",
            path.display(),
            idx + 1
        ));
    }
}

/// PoC-7 互換の JSON 形状で [`GateReport`] を組み立てる。値は必ず
/// [`crate::json_out::quoted`] を経由し、コマンド出力（`"`・改行・制御文字を
/// 含み得る）が JSON 構造を破壊しないことを保証する（security.md A08, 上記
/// `json_out::escape_str` と同一契約）。
fn render_report(report: &GateReport) -> String {
    let mut buf = String::new();
    buf.push('{');
    buf.push_str("\"checks\":[");
    for (i, check) in report.checks.iter().enumerate() {
        if i > 0 {
            buf.push(',');
        }
        buf.push('{');
        buf.push_str("\"name\":");
        buf.push_str(&quoted(check.name));
        buf.push_str(",\"passed\":");
        buf.push_str(if check.passed { "true" } else { "false" });
        buf.push_str(",\"output\":");
        buf.push_str(&quoted(&check.output));
        buf.push('}');
    }
    buf.push(']');
    buf.push_str(",\"gate_result\":");
    buf.push_str(&quoted(report.gate_result));
    buf.push_str(",\"action\":");
    buf.push_str(&quoted(&report.action));
    buf.push('}');
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct FakeRunner {
        // 呼ばれた回数分の (成功可否, 出力) を順番に返す。
        responses: Mutex<Vec<(bool, String)>>,
    }

    impl CommandRunner for FakeRunner {
        fn run(&self, _program: &str, _args: &[&str], _cwd: &Path) -> (bool, String) {
            self.responses.lock().unwrap().remove(0)
        }
    }

    fn manifest_with_one_crate() -> StructureManifest {
        StructureManifest {
            version: 1,
            directories: vec![structure::DirectoryEntry {
                name: "core".to_string(),
                role: Role::Core,
                crate_name: Some("rws-core".to_string()),
                description: "test".to_string(),
                depends_on: Vec::new(),
                allowed_dependents: Vec::new(),
            }],
            routing: None,
        }
    }

    #[test]
    fn aggregate_all_passed_is_pass() {
        let checks = vec![
            GateCheck {
                name: "a",
                passed: true,
                output: String::new(),
            },
            GateCheck {
                name: "b",
                passed: true,
                output: String::new(),
            },
        ];
        let report = aggregate(checks);
        assert_eq!(report.gate_result, "PASS");
    }

    #[test]
    fn aggregate_one_failure_is_blocked() {
        let checks = vec![
            GateCheck {
                name: "a",
                passed: true,
                output: String::new(),
            },
            GateCheck {
                name: "b",
                passed: false,
                output: "boom".to_string(),
            },
        ];
        let report = aggregate(checks);
        assert_eq!(report.gate_result, "BLOCKED");
    }

    #[test]
    fn run_all_checks_treats_launch_failure_as_failed_not_skipped() {
        let manifest = manifest_with_one_crate();
        let dir = std::env::temp_dir().join("fw-gate-test-launch-failure");
        let _ = std::fs::create_dir_all(&dir);
        // 5 チェックすべてがコマンド起動を試みるわけではない（escape_check は
        // 純粋関数、policy は deny.toml 欠落で早期 failed）。cargo 系 3 チェック分の
        // フェイク応答を積む。
        let runner = FakeRunner {
            responses: Mutex::new(vec![
                (false, "cargo: command not found".to_string()),
                (false, "cargo: command not found".to_string()),
                (false, "cargo: command not found".to_string()),
            ]),
        };
        let report = run_all_checks(&manifest, &dir, &runner);
        assert_eq!(report.gate_result, "BLOCKED");
        let type_check = report
            .checks
            .iter()
            .find(|c| c.name == "type_check")
            .unwrap();
        assert!(!type_check.passed);
        let policy = report.checks.iter().find(|c| c.name == "policy").unwrap();
        assert!(
            !policy.passed,
            "deny.toml is absent in the tempdir fixture, so policy must fail-closed"
        );
    }

    #[test]
    fn declared_crate_names_collects_only_directories_with_crate() {
        let manifest = StructureManifest {
            version: 1,
            directories: vec![
                structure::DirectoryEntry {
                    name: "core".to_string(),
                    role: Role::Core,
                    crate_name: Some("rws-core".to_string()),
                    description: "test".to_string(),
                    depends_on: Vec::new(),
                    allowed_dependents: Vec::new(),
                },
                structure::DirectoryEntry {
                    name: "static".to_string(),
                    role: Role::Asset,
                    crate_name: None,
                    description: "test".to_string(),
                    depends_on: Vec::new(),
                    allowed_dependents: Vec::new(),
                },
            ],
            routing: None,
        };
        assert_eq!(declared_crate_names(&manifest), vec!["rws-core"]);
    }

    #[test]
    fn line_has_raw_html_call_detects_direct_call() {
        assert!(line_has_raw_html_call("let x = raw_html(user_input);"));
    }

    #[test]
    fn line_has_raw_html_call_detects_call_with_space_before_paren() {
        assert!(line_has_raw_html_call("let x = raw_html (user_input);"));
    }

    #[test]
    fn line_has_raw_html_call_ignores_non_call_occurrence() {
        assert!(!line_has_raw_html_call(
            "// see raw_html module docs for details"
        ));
    }

    #[test]
    fn scan_file_reports_unreviewed_call() {
        let dir = std::env::temp_dir().join("fw-gate-test-escape-unreviewed");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("lib.rs");
        std::fs::write(&file, "fn f() {\n    raw_html(x);\n}\n").unwrap();

        let mut violations = Vec::new();
        scan_file_for_violations(&file, &mut violations);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("lib.rs:2"));
    }

    #[test]
    fn scan_file_allows_marker_on_same_line() {
        let dir = std::env::temp_dir().join("fw-gate-test-escape-same-line-marker");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("lib.rs");
        std::fs::write(
            &file,
            "fn f() {\n    raw_html(x); // ESCAPE-REVIEWED: sanitized upstream\n}\n",
        )
        .unwrap();

        let mut violations = Vec::new();
        scan_file_for_violations(&file, &mut violations);
        assert!(violations.is_empty());
    }

    #[test]
    fn scan_file_allows_marker_on_previous_line() {
        let dir = std::env::temp_dir().join("fw-gate-test-escape-prev-line-marker");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("lib.rs");
        std::fs::write(
            &file,
            "fn f() {\n    // ESCAPE-REVIEWED: sanitized upstream\n    raw_html(x);\n}\n",
        )
        .unwrap();

        let mut violations = Vec::new();
        scan_file_for_violations(&file, &mut violations);
        assert!(violations.is_empty());
    }

    #[test]
    fn default_escape_check_excludes_core_role_directories() {
        let dir = std::env::temp_dir().join("fw-gate-test-escape-core-excluded");
        let core_src = dir.join("core").join("src");
        let _ = std::fs::create_dir_all(&core_src);
        std::fs::write(core_src.join("lib.rs"), "raw_html(x);\n").unwrap();

        let manifest = StructureManifest {
            version: 1,
            directories: vec![structure::DirectoryEntry {
                name: "core".to_string(),
                role: Role::Core,
                crate_name: Some("rws-core".to_string()),
                description: "test".to_string(),
                depends_on: Vec::new(),
                allowed_dependents: Vec::new(),
            }],
            routing: None,
        };

        let check = default_escape_check(&manifest, &dir);
        assert!(
            check.passed,
            "role=core directories must be excluded from the scan (core owns raw_html itself)"
        );
    }

    #[test]
    #[cfg(unix)]
    fn scan_dir_for_violations_does_not_follow_symlinked_directory() {
        // レビュー指摘: `src/loop -> src/` のような自己参照シンボリックリンクを
        // 辿ると無限再帰でスタックオーバーフローする（fail-closed 自体を阻害する
        // DoS）。`is_symlink()` による明示除外でリンクを一律スキップすることを
        // 検証する。
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-escape-symlink-dir-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // シンボリックリンクの走査先に violation を仕込み、辿られていれば
        // 検出されてしまうことをもってテストの有効性を担保する。
        let outside = std::env::temp_dir().join(format!(
            "fw-gate-test-escape-symlink-outside-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&outside);
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("lib.rs"), "raw_html(x);\n").unwrap();
        std::os::unix::fs::symlink(&outside, dir.join("link_to_outside")).unwrap();
        // 自己参照リンクも仕込み、無限再帰を誘発しないことを確認する。
        std::os::unix::fs::symlink(&dir, dir.join("self_link")).unwrap();

        let mut violations = Vec::new();
        scan_dir_for_violations(&dir, &mut violations);
        assert!(
            violations.is_empty(),
            "symlinked directory must not be followed: found {violations:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&outside);
    }

    #[test]
    #[cfg(unix)]
    fn scan_dir_for_violations_does_not_follow_symlinked_file() {
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-escape-symlink-file-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let target = std::env::temp_dir().join(format!(
            "fw-gate-test-escape-symlink-target-{}.rs",
            std::process::id()
        ));
        std::fs::write(&target, "raw_html(x);\n").unwrap();
        std::os::unix::fs::symlink(&target, dir.join("link.rs")).unwrap();

        let mut violations = Vec::new();
        scan_dir_for_violations(&dir, &mut violations);
        assert!(
            violations.is_empty(),
            "symlinked file must not be followed: found {violations:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_file(&target);
    }

    #[test]
    fn default_escape_check_truncates_large_violation_output() {
        // Low 指摘: escape check の出力も他チェック（type_check/lint/test/policy）
        // と同様に `truncate_output` を通し、大量の未レビュー raw_html() 呼び出しが
        // ある場合でも JSON レポートが際限なく肥大化しないことを保証する。
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-escape-truncate-{}",
            std::process::id()
        ));
        let app_src = dir.join("app").join("src");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&app_src).unwrap();
        // 1 呼び出しあたりの違反メッセージは十分短いため、大量の行を書き込んで
        // 合計文字数が OUTPUT_TRUNCATE_CHARS を超えるようにする。
        let mut content = String::new();
        for _ in 0..(OUTPUT_TRUNCATE_CHARS / 4 + 10) {
            content.push_str("raw_html(x);\n");
        }
        std::fs::write(app_src.join("lib.rs"), content).unwrap();

        let manifest = StructureManifest {
            version: 1,
            directories: vec![structure::DirectoryEntry {
                name: "app".to_string(),
                role: Role::Component,
                crate_name: Some("rws-app".to_string()),
                description: "test".to_string(),
                depends_on: Vec::new(),
                allowed_dependents: Vec::new(),
            }],
            routing: None,
        };

        let check = default_escape_check(&manifest, &dir);
        assert!(!check.passed);
        assert!(check.output.chars().count() <= OUTPUT_TRUNCATE_CHARS);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn render_report_escapes_command_output_with_quotes_and_control_chars() {
        let report = GateReport {
            checks: vec![GateCheck {
                name: "lint",
                passed: false,
                output: "warning: \"unused\"\ncontrol\x07char".to_string(),
            }],
            gate_result: "BLOCKED",
            action: "fix".to_string(),
        };
        let json = render_report(&report);
        assert!(json.contains("\\\"unused\\\""));
        assert!(json.contains("\\n"));
        assert!(json.contains("\\u0007"));
        // 出力全体が 1 行の JSON であること（生の改行が構造を壊していないこと）。
        assert_eq!(json.lines().count(), 1);
    }

    #[test]
    fn truncate_output_keeps_tail_within_limit() {
        let long = "a".repeat(OUTPUT_TRUNCATE_CHARS + 100);
        let truncated = truncate_output(&long);
        assert_eq!(truncated.chars().count(), OUTPUT_TRUNCATE_CHARS);
    }

    #[test]
    fn truncate_output_leaves_short_output_untouched() {
        assert_eq!(truncate_output("short"), "short");
    }
}
