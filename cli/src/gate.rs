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
//!   `raw_html()` の呼び出しを検出する。**主防御**は `lint` チェック
//!   （`cargo clippy` + workspace ルート `clippy.toml` の `disallowed-methods`）で
//!   あり、コンパイラのパス解決（HIR）に基づくため `// ESCAPE-REVIEWED:` の
//!   ようなコメントでは偽装できない。`default_escape_check` 自体はテキスト走査の
//!   **保険層**として残し、受理条件を「コメントマーカー」から「同一行・直前行の
//!   `#[expect(clippy::disallowed_methods, reason = \"ESCAPE-REVIEWED: ...\")]`
//!   属性」へ変更した（属性はソース上に残り `unknown_lints` 等で偽装しにくく、
//!   `#[expect]` は呼び出しが消えた後の残置も clippy が
//!   `unfulfilled_lint_expectations` で検出する）。あわせて `#[allow(...)]` に
//!   よるブランケット抑止（ファイル・モジュール一括無効化）を独立の違反として
//!   監査する（[`scan_file_for_violations`] 参照。イシュー #157/#158/#159、
//!   詳細な脅威モデル・方式比較は `docs/raw-html-lint-design.md`）。
//! - `lint` チェックは workspace ルート `clippy.toml` に `disallowed-methods` の
//!   `rws_core::raw_html` エントリが存在することを前提とする。`clippy.toml` の
//!   欠落・エントリ欠落は「検出ポリシーの沈黙」＝黙示的 PASS を招くため、
//!   `cargo clippy` を起動する前に fail-closed で検出する
//!   （[`clippy_policy_is_configured`] 参照、security.md A05）。

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

/// `structure.toml` に宣言クレートが 1 つもない場合の fail-closed メッセージ。
///
/// `crates` が空のまま `cargo check`/`clippy`/`test` を `-p` なしで実行すると
/// ワークスペース全体（宣言外クレート含む）を検証してしまい、「`structure.toml`
/// を唯一の情報源として宣言クレートのみを検証する」契約が崩れる。宣言 0 件を
/// 「検証対象なし＝ PASS」と黙って通す（過小検証）のでも、範囲不明なワーク
/// スペース全体検証に暗黙にフォールバックする（範囲逸脱）のでもなく、設定不備
/// として明示的に fail-closed する（security.md A05。Bugbot 指摘: PR #261 #2）。
fn no_declared_crates_message() -> String {
    "no crate declared in structure.toml (no directory sets `crate = \"...\"`); \
refusing to fall back to whole-workspace verification, declare at least one crate"
        .to_string()
}

/// `crates` を対象に `-p <crate>` を連ねて外部コマンドを実行する共通ヘルパー。
/// `--locked` はロックファイル逸脱（依存すり替え）検出のため常に付与する
/// （security.md A06）。`crates` が空の場合はワークスペース全体へのフォール
/// バックを避け fail-closed する（[`no_declared_crates_message`] 参照）。
fn run_locked_cargo_subcommand(
    runner: &dyn CommandRunner,
    project_dir: &Path,
    name: &'static str,
    subcommand_args: &[&str],
    crates: &[&str],
) -> GateCheck {
    if crates.is_empty() {
        return GateCheck {
            name,
            passed: false,
            output: no_declared_crates_message(),
        };
    }
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

/// `clippy.toml` に検出したい `disallowed-methods` の対象パスが含まれるかを
/// テキストで検証する（TOML パーサ追加は `cli` 外部依存ゼロ方針に反するため、
/// `path` 文字列と `disallowed-methods` キーの併存という緩い判定に留める）。
///
/// 偽陰性（欠落しているのに見逃す）よりも偽陽性（設定はあるが厳密でない）を
/// 許容する保守側の実装。目的は「ファイルごと消された/エントリを削られた」
/// 明白な沈黙化を fail-closed で検出することであり、TOML の完全な意味検証では
/// ない。
///
/// ただし各行の `#` 以降（TOML のコメント）は [`crate::toml::strip_comment`]
/// （`structure.toml` 用パーサの内部ヘルパーを再利用）で判定前に除去する
/// （Bugbot 指摘 PR #263 "Clippy policy substring false pass"）。コメント
/// アウトされた（または削除済みの）`disallowed-methods` ブロックが
/// `disallowed-methods` / `rws_core::raw_html` という文字列断片をコメントとして
/// 残しているだけの場合に「設定済み」と誤判定すると、実際には
/// `cargo clippy` が当該エントリを読み込んでおらず `raw_html()` の未レビュー
/// 呼び出しを検出できないにもかかわらず `lint` チェックが素通りする
/// （検出ポリシーの沈黙、security.md A05）。
fn clippy_policy_is_configured(clippy_toml_path: &Path) -> bool {
    let Ok(content) = std::fs::read_to_string(clippy_toml_path) else {
        return false;
    };
    let active_content: String = content
        .lines()
        .map(crate::toml::strip_comment)
        .collect::<Vec<_>>()
        .join("\n");
    active_content.contains("disallowed-methods") && active_content.contains("rws_core::raw_html")
}

/// `lint` チェックが `cargo clippy` を起動する前に検証する fail-closed ガード。
///
/// clippy は `disallowed-methods` を起動時のカレントディレクトリ
/// （`CLIPPY_CONF_DIR` 未設定時）の `clippy.toml` から読み込む。本チェックは
/// `runner.run` に `project_dir` を cwd として渡すため、`clippy.toml` が
/// `project_dir` 直下に存在し `rws_core::raw_html` を宣言していることを前提と
/// する。ここが欠落・削除されると `disallowed_methods` が沈黙し、`raw_html()`
/// の未レビュー呼び出しが検出されないまま `lint` チェックが PASS してしまう
/// （検出ポリシーの黙示的無効化という fail-open の穴、security.md A05）。
/// `default_escape_check`（テキスト走査の保険層）とは独立に、主防御である
/// clippy 側の設定自体の健全性をここで担保する。
fn clippy_policy_check(project_dir: &Path) -> Option<GateCheck> {
    let clippy_toml = project_dir.join("clippy.toml");
    if !clippy_policy_is_configured(&clippy_toml) {
        return Some(GateCheck {
            name: "lint",
            passed: false,
            output: format!(
                "{} is missing or lacks a `disallowed-methods` entry for `rws_core::raw_html`; \
without it `cargo clippy` cannot detect unreviewed raw_html() calls (see templates/default/clippy.toml)",
                clippy_toml.display()
            ),
        });
    }
    None
}

fn run_cargo_clippy(runner: &dyn CommandRunner, project_dir: &Path, crates: &[&str]) -> GateCheck {
    // `--locked` はロックファイル逸脱（依存すり替え）検出のため `type_check` /
    // `test`（`run_locked_cargo_subcommand`）と同様に常に付与する
    // （security.md A06。Bugbot 指摘: PR #261 #2 — `lint` だけ `--locked` を
    // 欠くと依存差し替え検知の抜け道になり得る）。
    // `crates` が空の場合は他の 2 チェックと同様にワークスペース全体への
    // フォールバックを避け fail-closed する（[`no_declared_crates_message`] 参照。
    // Bugbot 指摘: PR #261 #2 — `lint` は `run_locked_cargo_subcommand` を介さず
    // 独自に引数を組み立てるため、この分岐を個別に持つ必要がある）。
    if crates.is_empty() {
        return GateCheck {
            name: "lint",
            passed: false,
            output: no_declared_crates_message(),
        };
    }
    // イシュー #157: `clippy.toml` の `disallowed-methods` 設定が欠落したまま
    // `cargo clippy` を起動すると「検出項目が何もない」正常終了になり得るため
    // （黙示的 PASS）、起動前にポリシー設定自体の存在を検証する。
    if let Some(check) = clippy_policy_check(project_dir) {
        return check;
    }
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

/// ファイル内容全体（複数行にまたがってよい）から `raw_html` 呼び出しの開始
/// バイトオフセットを列挙する状態機械の最小走査（正規表現クレートを使わず
/// 手書きで判定する。`cli` 外部依存ゼロ方針）。
///
/// `content` 中の `raw_html` の各出現位置について、直後の空白文字（半角
/// スペース・タブ・改行を含む ASCII 空白）を読み飛ばした先が `(` であれば
/// 呼び出しとみなす。コメント内の出現も検出対象とする（偽陽性は許容・
/// 偽陰性は不許容の保守側実装、計画 §3.2）。
///
/// 行単位走査（旧 `line_has_raw_html_call`）では `raw_html` 識別子と `(` が
/// 改行を挟んで別々の行に置かれた呼び出し（`raw_html\n    (user_input)` 等）
/// を見逃していた（「見逃しなし」方針に反する検出漏れ、Bugbot 指摘:
/// PR #261 #1）。空白の読み飛ばしを改行にも及ぼすことでこれを解消する。
fn find_raw_html_call_positions(content: &str) -> Vec<usize> {
    let bytes = content.as_bytes();
    let needle = b"raw_html";
    let mut positions = Vec::new();
    let mut start = 0;
    while let Some(rel) = find_subslice(&bytes[start..], needle) {
        let match_start = start + rel;
        let mut i = match_start + needle.len();
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'(' {
            positions.push(match_start);
        }
        start = match_start + 1;
        if start >= bytes.len() {
            break;
        }
    }
    positions
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// レビュー宣言の根拠文言。単独のコメントでは受理せず、必ず
/// `clippy::disallowed_methods` への `#[expect(...)]` 属性内の `reason` に
/// この文字列が含まれることを要求する（[`line_has_reviewed_expect_attribute`]）。
const ESCAPE_REVIEWED_MARKER: &str = "ESCAPE-REVIEWED:";

/// レビュー済みオプトインの属性が呼び出し文へ付与されていることを示す部分文字列。
/// `#[expect(clippy::disallowed_methods, reason = "ESCAPE-REVIEWED: ...")]` の
/// 形を想定し、`clippy::disallowed_methods` 側の実体験証は
/// `raw_html_lint_e2e.rs`（実 clippy 起動）が担う。ここでは「該当属性が
/// ソース上に存在するか」という保険層のテキスト判定に留める。
const EXPECT_DISALLOWED_METHODS_MARKER: &str = "expect(clippy::disallowed_methods";

/// ブランケット抑止（ファイル・モジュール一括での `disallowed_methods` 無効化）を
/// 検出するための内部属性プレフィックス。`#![allow(clippy::disallowed_methods)]`
/// や `#![expect(clippy::disallowed_methods)]` は clippy 側の主防御そのものを
/// 沈黙させるため、呼び出し個別のレビュー宣言とは区別して一律に違反とする
/// （[`scan_file_for_violations`] 内の走査）。
const BLANKET_DISALLOWED_METHODS_MARKERS: [&str; 2] = [
    "#![allow(clippy::disallowed_methods",
    "#![expect(clippy::disallowed_methods",
];

/// 行 `line` に実際の（コンパイラが解釈する）ブランケット抑止属性が存在するかを
/// 判定する。単純な部分文字列一致（`str::contains`）は、行コメント・
/// ドキュメンテーションコメント（`//` `///` `//!`）中の説明文言や文字列リテラル
/// （本ファイルのテストフィクスチャが該当）にもマッチしてしまい、実際には
/// inner attribute が存在しないのに違反として誤検出する（Bugbot 指摘
/// PR #263 "Blanket scan matches docs and literals"）。
///
/// 完全な Rust トークナイザは持たないため、以下の 2 段の簡易判定に留める
/// （それでも本チェックは clippy（主防御）の保険層であり、厳密な構文解析は
/// clippy 側が担う。モジュール冒頭 doc コメント参照）:
/// 1. 行头（先頭の空白を除いた）が `//` で始まる場合はコメント行とみなし除外する
///    （`//` `///` `//!` をまとめて除外できる）。
/// 2. マーカーの出現位置がダブルクォート文字列リテラルの内側（マーカー手前の
///    非エスケープ `"` の個数が奇数）であれば除外する。
fn line_has_real_blanket_attribute(line: &str) -> bool {
    if line.trim_start().starts_with("//") {
        return false;
    }
    BLANKET_DISALLOWED_METHODS_MARKERS
        .iter()
        .any(|marker| match line.find(marker) {
            Some(pos) => !position_is_inside_string_literal(line, pos),
            None => false,
        })
}

/// `line` の `pos` バイト位置が、`pos` より前に現れるダブルクォート文字列
/// リテラルの内側にあるかどうかを判定する（バックスラッシュエスケープされた
/// `"` は文字列の開始・終了とみなさない）。TOML/Rust の完全なパーサではなく、
/// 「引用符の個数の偶奇」による簡易判定であるため、複数行文字列・生文字列
/// （`r"..."` 等）までは追跡しない（この用途では十分な近似）。
fn position_is_inside_string_literal(line: &str, pos: usize) -> bool {
    let mut in_string = false;
    let mut escaped = false;
    for c in line[..pos].chars() {
        if escaped {
            escaped = false;
            continue;
        }
        match c {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            _ => {}
        }
    }
    in_string
}

/// 行 `line` が「`clippy::disallowed_methods` へのレビュー済み `#[expect]` 属性」で
/// あるとみなせるかを判定する。同一行に `expect(clippy::disallowed_methods` と
/// `ESCAPE-REVIEWED:` の両方が含まれることを要求し、旧方式（`ESCAPE-REVIEWED:`
/// コメント単体）は受理しない（イシュー #157: コメントは clippy に検証されず
/// 偽装可能なため、コンパイラが解釈する属性であることを必須化する）。
fn line_has_reviewed_expect_attribute(line: &str) -> bool {
    line.contains(EXPECT_DISALLOWED_METHODS_MARKER) && line.contains(ESCAPE_REVIEWED_MARKER)
}

/// `role = "core"` 以外の宣言ディレクトリの `src/` 配下 `*.rs`（`tests/` は
/// 走査対象外。PoC-7 の粒度を維持し、テストコード内の `raw_html()` 利用は
/// TASK-13.5 以降の負例回帰テストの対象とする限界を持つ）を走査し、
/// `#[expect(clippy::disallowed_methods, reason = "ESCAPE-REVIEWED: ...")]`
/// 属性（同一行または直前行）を伴わない `raw_html()` 呼び出しと、ブランケット
/// 抑止属性（[`BLANKET_DISALLOWED_METHODS_MARKERS`]）を違反として `file:line`
/// 付きで列挙する（REQ-1 の唯一の迂回経路を明示レビュー済み宣言に限定する契約、
/// security.md A08）。本チェックは clippy（主防御）の保険層であり、`lint`
/// チェック（`cargo clippy` + `clippy.toml`）が偽装不能な検出を担う
/// （モジュール冒頭 doc コメント参照）。
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

/// `path` の内容から未レビューの `raw_html()` 呼び出しとブランケット抑止属性を
/// 検出し `violations` へ `file:line` 形式で追記する。呼び出しが複数行に
/// またがる場合（識別子と `(` が別行）も [`find_raw_html_call_positions`]
/// により検出したうえで、その呼び出し「開始行」を基準に同一行・直前行の
/// [`line_has_reviewed_expect_attribute`]（レビュー済み `#[expect]` 属性）の
/// 有無を判定する（属性は呼び出し全体ではなく開始位置に対して書かれる運用を
/// 想定。statement 属性として呼び出し文自体に付与するのが標準形、
/// `docs/raw-html-review-gate.md` 参照）。
fn scan_file_for_violations(path: &Path, violations: &mut Vec<String>) {
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    let lines: Vec<&str> = content.lines().collect();
    // 各行の開始バイトオフセットを前計算し、マッチ位置 → 行番号の変換を
    // 線形走査 1 回で済ませる（ファイルサイズに対して O(n) を維持し、
    // マッチのたびに先頭から数え直す O(n^2) 化を避ける）。
    let mut line_starts: Vec<usize> = Vec::with_capacity(lines.len());
    let mut offset = 0usize;
    for line in &lines {
        line_starts.push(offset);
        offset += line.len() + 1; // `\n` の 1 バイト分（末尾に改行がなくてもズレは許容範囲）。
    }

    for match_start in find_raw_html_call_positions(&content) {
        let line_idx = match line_starts.binary_search(&match_start) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        let reviewed_here = lines
            .get(line_idx)
            .is_some_and(|l| line_has_reviewed_expect_attribute(l));
        let reviewed_prev = line_idx > 0
            && lines
                .get(line_idx - 1)
                .is_some_and(|l| line_has_reviewed_expect_attribute(l));
        if reviewed_here || reviewed_prev {
            continue;
        }
        violations.push(format!(
            "{}:{}: unreviewed raw_html() call",
            path.display(),
            line_idx + 1
        ));
    }

    // ブランケット抑止監査: `#![allow(clippy::disallowed_methods)]` /
    // `#![expect(clippy::disallowed_methods)]` はファイル・モジュール全体の
    // `disallowed_methods` 検出を無効化し、主防御（clippy）そのものを沈黙
    // させる。呼び出し個別のレビュー宣言（外側 `#[expect]`）とは異なり、
    // どのような文脈で書かれていても一律に違反として列挙する（イシュー #157、
    // security.md A05: 一括無効化による検出ポリシーの黙示的骨抜き防止）。
    //
    // ただし [`line_has_real_blanket_attribute`] でコメント（行コメント・
    // ドキュメンテーションコメント）と文字列リテラル内の出現を除外する
    // （Bugbot 指摘 PR #263: 本ファイル自身のモジュール冒頭 doc コメントや
    // 下記テストのフィクスチャ文字列リテラルが、実際には inner attribute で
    // ないにもかかわらず誤検出されていた）。
    for (line_idx, line) in lines.iter().enumerate() {
        if line_has_real_blanket_attribute(line) {
            violations.push(format!(
                "{}:{}: blanket suppression of clippy::disallowed_methods is not allowed \
(remove the file/module-level #![allow(...)]/#![expect(...)] and use a per-call \
#[expect(clippy::disallowed_methods, reason = \"ESCAPE-REVIEWED: ...\")] instead)",
                path.display(),
                line_idx + 1
            ));
        }
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

    /// 呼ばれたら即座に panic するフェイク（「起動しないこと」自体を検証する
    /// テスト専用。実行されればテスト失敗として顕在化する）。
    struct PanicIfCalledRunner;

    impl CommandRunner for PanicIfCalledRunner {
        fn run(&self, program: &str, args: &[&str], _cwd: &Path) -> (bool, String) {
            panic!("cargo must not be invoked when no crate is declared: {program} {args:?}");
        }
    }

    #[test]
    fn run_locked_cargo_subcommand_fails_closed_when_no_crates_declared() {
        // Bugbot 指摘: PR #261 #2 — 宣言クレートが 0 件のとき `-p` なしで
        // ワークスペース全体を検証してしまってはならない。cargo を一切起動せず
        // fail-closed することを検証する。
        let dir = std::env::temp_dir();
        let check =
            run_locked_cargo_subcommand(&PanicIfCalledRunner, &dir, "type_check", &["check"], &[]);
        assert!(!check.passed);
        assert!(check.output.contains("no crate declared"));
    }

    #[test]
    fn run_cargo_clippy_fails_closed_when_no_crates_declared() {
        let dir = std::env::temp_dir();
        let check = run_cargo_clippy(&PanicIfCalledRunner, &dir, &[]);
        assert!(!check.passed);
        assert!(check.output.contains("no crate declared"));
    }

    #[test]
    fn run_locked_cargo_subcommand_runs_when_crates_declared() {
        let dir = std::env::temp_dir();
        let runner = FakeRunner {
            responses: Mutex::new(vec![(true, "ok".to_string())]),
        };
        let check = run_locked_cargo_subcommand(
            &runner,
            &dir,
            "type_check",
            &["check", "--locked"],
            &["rws-core"],
        );
        assert!(check.passed);
    }

    #[test]
    fn find_raw_html_call_positions_detects_direct_call() {
        assert_eq!(
            find_raw_html_call_positions("let x = raw_html(user_input);").len(),
            1
        );
    }

    #[test]
    fn find_raw_html_call_positions_detects_call_with_space_before_paren() {
        assert_eq!(
            find_raw_html_call_positions("let x = raw_html (user_input);").len(),
            1
        );
    }

    #[test]
    fn find_raw_html_call_positions_ignores_non_call_occurrence() {
        assert!(find_raw_html_call_positions("// see raw_html module docs for details").is_empty());
    }

    #[test]
    fn find_raw_html_call_positions_detects_call_split_across_lines() {
        // Bugbot 指摘: PR #261 #1 — 識別子と `(` が改行を挟んで別行に置かれた
        // 呼び出しも見逃してはならない。
        assert_eq!(
            find_raw_html_call_positions("let x = raw_html\n    (user_input);").len(),
            1
        );
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
    fn scan_file_reports_call_split_across_lines() {
        // Bugbot 指摘: PR #261 #1 — 行単位走査だと `raw_html` 識別子と `(` が
        // 別行にまたがる呼び出しを見逃す。`scan_file_for_violations` を通しても
        // 検出できることを回帰として固定する。
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-escape-multiline-{}",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("lib.rs");
        std::fs::write(&file, "fn f() {\n    raw_html\n        (x);\n}\n").unwrap();

        let mut violations = Vec::new();
        scan_file_for_violations(&file, &mut violations);
        assert_eq!(violations.len(), 1, "violations: {violations:?}");
        assert!(violations[0].contains("lib.rs:2"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scan_file_allows_reviewed_expect_attribute_on_previous_line_for_split_call() {
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-escape-multiline-marker-{}",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("lib.rs");
        std::fs::write(
            &file,
            "fn f() {\n    #[expect(clippy::disallowed_methods, reason = \"ESCAPE-REVIEWED: sanitized upstream\")]\n    raw_html\n        (x);\n}\n",
        )
        .unwrap();

        let mut violations = Vec::new();
        scan_file_for_violations(&file, &mut violations);
        assert!(violations.is_empty(), "violations: {violations:?}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scan_file_allows_reviewed_expect_attribute_on_same_line() {
        let dir = std::env::temp_dir().join("fw-gate-test-escape-same-line-marker");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("lib.rs");
        std::fs::write(
            &file,
            "fn f() {\n    #[expect(clippy::disallowed_methods, reason = \"ESCAPE-REVIEWED: sanitized upstream\")] raw_html(x);\n}\n",
        )
        .unwrap();

        let mut violations = Vec::new();
        scan_file_for_violations(&file, &mut violations);
        assert!(violations.is_empty());
    }

    #[test]
    fn scan_file_allows_reviewed_expect_attribute_on_previous_line() {
        let dir = std::env::temp_dir().join("fw-gate-test-escape-prev-line-marker");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("lib.rs");
        std::fs::write(
            &file,
            "fn f() {\n    #[expect(clippy::disallowed_methods, reason = \"ESCAPE-REVIEWED: sanitized upstream\")]\n    raw_html(x);\n}\n",
        )
        .unwrap();

        let mut violations = Vec::new();
        scan_file_for_violations(&file, &mut violations);
        assert!(violations.is_empty());
    }

    #[test]
    fn scan_file_rejects_comment_only_marker_as_spoofable() {
        // イシュー #157 の中核回帰: 偽装可能な `// ESCAPE-REVIEWED:` コメント
        // 単体（属性を伴わない）はもはや受理してはならない。TASK-13.3 時点の
        // 旧方式（マーカー方式）ではここが PASS していたが、コメントは
        // コンパイラに検証されず偽装できるため BLOCKED へ倒す。
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-escape-comment-only-spoof-{}",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("lib.rs");
        std::fs::write(
            &file,
            "fn f() {\n    // ESCAPE-REVIEWED: sanitized upstream\n    raw_html(x);\n}\n",
        )
        .unwrap();

        let mut violations = Vec::new();
        scan_file_for_violations(&file, &mut violations);
        assert_eq!(
            violations.len(),
            1,
            "comment-only marker must no longer suppress detection: violations={violations:?}"
        );
        assert!(violations[0].contains("lib.rs:3"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scan_file_reports_blanket_allow_suppression() {
        // イシュー #157: `#![allow(clippy::disallowed_methods)]` はファイル全体の
        // 主防御（clippy）を無効化するブランケット抑止であり、呼び出し個別の
        // レビュー宣言とは独立に違反として列挙されなければならない。
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-escape-blanket-allow-{}",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("lib.rs");
        std::fs::write(
            &file,
            "#![allow(clippy::disallowed_methods)]\nfn f() {\n    raw_html(x);\n}\n",
        )
        .unwrap();

        let mut violations = Vec::new();
        scan_file_for_violations(&file, &mut violations);
        assert!(
            violations
                .iter()
                .any(|v| v.contains("lib.rs:1") && v.contains("blanket suppression")),
            "violations: {violations:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scan_file_reports_blanket_expect_suppression() {
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-escape-blanket-expect-{}",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("lib.rs");
        std::fs::write(
            &file,
            "#![expect(clippy::disallowed_methods)]\nfn f() {\n    raw_html(x);\n}\n",
        )
        .unwrap();

        let mut violations = Vec::new();
        scan_file_for_violations(&file, &mut violations);
        assert!(
            violations
                .iter()
                .any(|v| v.contains("lib.rs:1") && v.contains("blanket suppression")),
            "violations: {violations:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scan_file_ignores_blanket_marker_inside_comment() {
        // Bugbot 指摘 PR #263 "Blanket scan matches docs and literals": doc
        // コメント中で `#![allow(clippy::disallowed_methods)]` を説明のために
        // 言及しているだけの行は、実際の inner attribute ではないため違反として
        // 検出してはならない。
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-escape-blanket-comment-{}",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("lib.rs");
        std::fs::write(
            &file,
            "/// `#![allow(clippy::disallowed_methods)]` は禁止という説明。\nfn f() {}\n",
        )
        .unwrap();

        let mut violations = Vec::new();
        scan_file_for_violations(&file, &mut violations);
        assert!(
            violations.is_empty(),
            "doc comment mentioning the marker text must not be flagged: violations={violations:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scan_file_ignores_blanket_marker_inside_string_literal() {
        // Bugbot 指摘 PR #263: 本ファイル自身のテストフィクスチャのような
        // 文字列リテラル内の出現（実際の attribute ではない）も除外する。
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-escape-blanket-literal-{}",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("lib.rs");
        std::fs::write(
            &file,
            "const MARKER: &str = \"#![allow(clippy::disallowed_methods\";\n",
        )
        .unwrap();

        let mut violations = Vec::new();
        scan_file_for_violations(&file, &mut violations);
        assert!(
            violations.is_empty(),
            "marker text inside a string literal must not be flagged: violations={violations:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn clippy_policy_check_fails_closed_when_clippy_toml_missing() {
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-clippy-policy-missing-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let check = clippy_policy_check(&dir);
        assert!(
            check.is_some(),
            "missing clippy.toml must fail the lint check closed"
        );
        assert!(!check.unwrap().passed);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn clippy_policy_check_fails_closed_when_entry_missing() {
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-clippy-policy-entry-missing-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("clippy.toml"), "# no disallowed-methods entry\n").unwrap();

        let check = clippy_policy_check(&dir);
        assert!(check.is_some());
        assert!(!check.unwrap().passed);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn clippy_policy_check_passes_when_entry_present() {
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-clippy-policy-ok-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("clippy.toml"),
            "disallowed-methods = [{ path = \"rws_core::raw_html\", reason = \"x\" }]\n",
        )
        .unwrap();

        assert!(clippy_policy_check(&dir).is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn clippy_policy_check_fails_closed_when_entry_is_commented_out() {
        // Bugbot 指摘 PR #263 "Clippy policy substring false pass": コメント
        // アウトされた `disallowed-methods` ブロックがテキストとして
        // `disallowed-methods` / `rws_core::raw_html` を含んでいても、実際に
        // 有効な TOML エントリではないため「設定済み」と誤判定してはならない。
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-clippy-policy-commented-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("clippy.toml"),
            "# disallowed-methods = [{ path = \"rws_core::raw_html\", reason = \"x\" }]\n",
        )
        .unwrap();

        let check = clippy_policy_check(&dir);
        assert!(
            check.is_some(),
            "a commented-out entry must not be treated as configured"
        );
        assert!(!check.unwrap().passed);

        let _ = std::fs::remove_dir_all(&dir);
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
