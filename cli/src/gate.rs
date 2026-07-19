//! `fw gate`: REQ-13 の第 4 要素「検証・制約の強制」を製品化する検証ゲート
//! （TASK-13.3, #138, 親 PoC-7 `cmd_gate` の Rust 移植）。
//!
//! 本モジュールが実装する判定ルール（6 チェックの定義・fail-closed 条件・
//! 集約規則・JSON 契約）の正式な設計文書は `docs/design/gate-design.md`
//! （TASK-13.3a, #139）を参照。本コメントおよび各関数の doc コメントは
//! 実装詳細の説明に留め、判定ルールの単一の情報源は同文書とする。
//!
//! [`crate::structure`]（TASK-13.1）が定義する `structure.toml` を唯一の情報源
//! として宣言クレート・ディレクトリを求め、6 チェック（型チェック・既定エスケープ
//! 検査・URL 属性検証（イシュー #401）・lint・テスト・依存ポリシー）を実行し、
//! 集約結果を JSON で stdout へ出力する。
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
//!   詳細な脅威モデル・方式比較は `docs/design/raw-html-lint-design.md`）。
//! - `lint` チェックは `--all-targets` を付与し、テストターゲット
//!   （`#[cfg(test)]` / `tests/` 配下）内の未レビュー `raw_html()` 呼び出しも
//!   検出する（イシュー #315）。`default_escape_check`（保険層）は `tests/` を
//!   走査対象外のままとするが、主防御である本チェックが `--all-targets` により
//!   その死角を埋めるため、ローカルゲート・AI 自己保守フックと CI `clippy`
//!   ジョブ（イシュー #299）の検出範囲は一致する。
//! - `lint` チェックは workspace ルート `clippy.toml` に `disallowed-methods` の
//!   `rws_core::raw_html` エントリが存在することを前提とする。`clippy.toml` の
//!   欠落・エントリ欠落は「検出ポリシーの沈黙」＝黙示的 PASS を招くため、
//!   `cargo clippy` を起動する前に fail-closed で検出する
//!   （[`clippy_policy_is_configured`] 参照、security.md A05）。

use crate::json_out::quoted;
use crate::structure::{self, Role, StructureManifest};
use std::path::{Path, PathBuf};
use std::process::Command;

/// コマンド出力を JSON へ格納する際の丸め上限（末尾からの文字数）。
/// 肥大化防止・秘密情報の意図しない大量転記防止（security.md A09）。
const OUTPUT_TRUNCATE_CHARS: usize = 4000;

/// 「コード起因の FAIL」と「実行環境にツールが無いだけの failed」を区別する
/// ための決定的な出力プレフィックス（イシュー #292）。
///
/// self-hosted runner プールは clippy component / cargo-deny の導入状態が
/// インスタンスごとに異なり、`lint` / `policy` チェックがどの runner に
/// 当たるかで間欠的に BLOCKED になっていた。JSON 契約（`checks[].name` /
/// `passed` / `output`、PoC-7 互換）の形状は変えず、`output` の先頭にこの
/// プレフィックスを置くことで区別を表現する。**SKIP や黙示的 PASS には
/// 倒さない**（fail-closed 維持、security.md A05）。ツールの自動インストールも
/// ここでは行わない（検証ゲートは検証のみに専念させ、ネットワーク非依存・
/// サプライチェーン面の不拡大を維持する。導入は `tools/ci/ensure-gate-tools.sh`
/// の責務とする）。
const ENVIRONMENT_ERROR_PREFIX: &str = "environment error: ";

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
/// 3. 6 チェックをすべて実行（早期打ち切りしない。AI エージェントが一括修正できる
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

/// 6 チェックを実行して [`GateReport`] を組み立てる（実プロセス起動を伴う本番経路）。
///
/// テストからは `runner` に実行を伴わないフェイクを注入して集約ロジックのみを
/// 検証する（実プロセス起動なしのテスト容易性、計画 §3.3）。
///
/// `fw new --template embed`（イシュー #410）が生成する静的単一ファイル
/// プロジェクトのように cargo パッケージを持たない構成では、cargo 系 4 チェック
/// （`type_check`/`lint`/`test`/`policy`）は検証対象クレートが存在せず
/// 「検証不能」と「検証したが違反なし」を区別できない。[`is_asset_only_project`]
/// が明示宣言（同関数 doc コメント参照）を検出した場合のみ、この 4 チェックを
/// [`not_applicable_check`] で not-applicable PASS 化する。テキスト走査ベースの
/// `default_escape_check`（保険層）・`url_validation_check` は cargo パッケージの
/// 有無に依存しないため静的専用モードでも通常どおり実行し、asset ディレクトリ
/// 配下へ Rust コードが混入した場合の回帰検出を維持する（security.md A05）。
fn run_all_checks(
    manifest: &StructureManifest,
    project_dir: &Path,
    runner: &dyn CommandRunner,
) -> GateReport {
    let crates = declared_crate_names(manifest);

    let checks = if is_asset_only_project(manifest) {
        vec![
            not_applicable_check("type_check"),
            default_escape_check(manifest, project_dir),
            url_validation_check(manifest, project_dir),
            not_applicable_check("lint"),
            not_applicable_check("test"),
            not_applicable_check("policy"),
        ]
    } else {
        vec![
            run_cargo_check(runner, project_dir, &crates),
            default_escape_check(manifest, project_dir),
            url_validation_check(manifest, project_dir),
            run_cargo_clippy(runner, project_dir, &crates),
            run_cargo_test(runner, project_dir, &crates),
            policy_check(runner, project_dir),
        ]
    };

    aggregate(checks)
}

/// 静的専用（asset-only）判定条件: 宣言クレートが 0 件、かつ宣言ディレクトリ
/// 全件が `role = "asset"` であること（イシュー #410 実装計画 §2.1、
/// `docs/design/gate-design.md` §2.4）。
///
/// `structure.toml` 上の明示宣言によるオプトインであり黙示的 PASS ではない
/// （security.md A05）。`crate` キーの削除し忘れ等で非 asset ロールが 1 件でも
/// 残っていれば本関数は `false` を返し、[`no_declared_crates_message`] による
/// 従来どおりの fail-closed（BLOCKED）が働く。`manifest.directories` が空の
/// ケースは通常 [`StructureManifest::validate`] が `NoDirectories` として
/// `run_gate` の時点で先に BLOCKED にするが、テストから本関数が直接呼ばれる
/// 場合に備えて防御的に空集合も非該当として扱う。
fn is_asset_only_project(manifest: &StructureManifest) -> bool {
    !manifest.directories.is_empty()
        && declared_crate_names(manifest).is_empty()
        && manifest.directories.iter().all(|d| d.role == Role::Asset)
}

/// 静的専用モードにおける cargo 系チェックの not-applicable PASS 文言。
///
/// 「検証不能」を隠蔽せず、なぜ実行しなかったか（cargo パッケージが存在せず
/// 対象なし）を決定的な文言で明示する（security.md A09。環境情報以外の内部
/// 情報は含めない）。
const STATIC_ONLY_NOT_APPLICABLE_MESSAGE: &str = "static-only project (all directories declare \
role = \"asset\" with no crate): cargo-based check not applicable";

fn not_applicable_check(name: &'static str) -> GateCheck {
    GateCheck {
        name,
        passed: true,
        output: STATIC_ONLY_NOT_APPLICABLE_MESSAGE.to_string(),
    }
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

/// `cargo clippy` 本実行の直前に疎通確認する（イシュー #292）。
///
/// clippy component 不在の runner では `cargo clippy` 起動自体が失敗するが、
/// その失敗はコード内容とは無関係な環境要因であり、コード起因の lint 違反と
/// 区別できないと「同じ BLOCKED」が原因不明のまま繰り返される。ここで軽量な
/// `cargo clippy --version` を先に起動し、失敗時のみ [`ENVIRONMENT_ERROR_PREFIX`]
/// 付きの決定的なメッセージ（是正コマンド付き）で `lint` を failed とする。
/// 疎通確認自体が成功した場合は `None` を返し、呼び出し元が本実行へ進む。
fn clippy_environment_preflight(
    runner: &dyn CommandRunner,
    project_dir: &Path,
) -> Option<GateCheck> {
    let (available, output) = runner.run("cargo", &["clippy", "--version"], project_dir);
    if available {
        return None;
    }
    Some(GateCheck {
        name: "lint",
        passed: false,
        output: format!(
            "{ENVIRONMENT_ERROR_PREFIX}`cargo clippy` is not available on this runner \
({}); run `rustup component add clippy` or `tools/ci/ensure-gate-tools.sh` to install it, \
then re-run `fw gate`",
            truncate_output(&output)
        ),
    })
}

/// `cargo deny` 本実行の直前に疎通確認する（イシュー #292、
/// [`clippy_environment_preflight`] と同一方針）。
///
/// cargo-deny 未導入の runner では `cargo deny check ...` の起動自体が
/// 失敗し、`deny.toml` の実際のポリシー違反と区別が付かない。`deny.toml`
/// 存在確認の後・本実行の前に `cargo deny --version` で疎通確認し、失敗時のみ
/// [`ENVIRONMENT_ERROR_PREFIX`] 付きの決定的なメッセージで `policy` を
/// failed とする。
fn cargo_deny_environment_preflight(
    runner: &dyn CommandRunner,
    project_dir: &Path,
) -> Option<GateCheck> {
    let (available, output) = runner.run("cargo", &["deny", "--version"], project_dir);
    if available {
        return None;
    }
    Some(GateCheck {
        name: "policy",
        passed: false,
        output: format!(
            "{ENVIRONMENT_ERROR_PREFIX}`cargo deny` is not available on this runner ({}); \
run `tools/ci/ensure-gate-tools.sh` to install it, then re-run `fw gate`",
            truncate_output(&output)
        ),
    })
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
    // イシュー #292: ポリシー設定の健全性確認の後・本実行の直前に、clippy
    // component 自体が起動可能かを疎通確認する（runner 環境差の決定的な区別）。
    if let Some(check) = clippy_environment_preflight(runner, project_dir) {
        return check;
    }
    // `-- -D warnings` は cargo 引数の後段（サブコマンド固有引数)として渡す
    // （coding-rust.md: `cargo clippy -- -D warnings` を通す規約と同一コマンド）。
    // `--all-targets` は CI `clippy` ジョブ（イシュー #299）と検出範囲を一致させる
    // ため付与する（イシュー #315）。テストターゲット（`#[cfg(test)]` / `tests/`
    // 配下）内の未レビュー `raw_html()` 呼び出しは `default_escape_check`（保険層、
    // `src/` のみ走査）では検出できないため、本チェック（主防御）がテストターゲット
    // まで含めて検出することで検出境界差を解消する。
    let mut args: Vec<&str> = vec!["clippy", "--locked", "--all-targets"];
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
/// とする（`templates/default/deny.toml` 冒頭コメント・`docs/policy/cargo-deny-advisories.md`
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
    // イシュー #292: `deny.toml` 存在確認の後・本実行の直前に、cargo-deny 本体が
    // 起動可能かを疎通確認する（runner 環境差の決定的な区別）。
    if let Some(check) = cargo_deny_environment_preflight(runner, project_dir) {
        return check;
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
/// `content` 中の `raw_html` の各出現位置について、以下 2 条件をいずれも
/// 満たす場合のみ呼び出しとみなす（イシュー #372: 走査の「コード文脈限定」への
/// 精密化。詳細な設計根拠は `docs/design/gate-design.md` §2.2 を参照）。
/// 1. [`code_context_mask`] 上で出現開始位置がコード文脈（コメント・文字列
///    リテラル・文字リテラルの外側）であること
/// 2. 出現開始位置の直前バイトが識別子構成文字（`[A-Za-z0-9_]`）でないこと
///    （`..._raw_html(` のような別識別子のサフィックスを呼び出しと誤認しない。
///    `rws_core::raw_html(`（直前 `:`）や `.raw_html(`（メソッド形）は
///    直前バイトが識別子構成文字でないため引き続き検出する）
///
/// さらに、直後の空白文字（半角スペース・タブ・改行を含む ASCII 空白）を
/// 読み飛ばした先が `(` であれば呼び出しとみなす。
///
/// コメント・文字列リテラル・別識別子は Rust の字句規則上 `raw_html()` の
/// 呼び出しになり得ないため、これらの除外は偽陽性（誤検知）のみを削り
/// 偽陰性（見逃し）を生まない。主防御は従来どおり `lint` チェック
/// （`cargo clippy` + `disallowed-methods`、HIR パス解決）であり無変更
/// （モジュール冒頭 doc コメント参照）。
///
/// 行単位走査（旧 `line_has_raw_html_call`）では `raw_html` 識別子と `(` が
/// 改行を挟んで別々の行に置かれた呼び出し（`raw_html\n    (user_input)` 等）
/// を見逃していた（「見逃しなし」方針に反する検出漏れ、Bugbot 指摘:
/// PR #261 #1）。空白の読み飛ばしを改行にも及ぼすことでこれを解消する
/// （この挙動は本変更でも維持する）。
fn find_raw_html_call_positions(content: &str) -> Vec<usize> {
    find_code_context_call_positions(content, b"raw_html", false)
}

/// [`find_raw_html_call_positions`] の走査本体を needle 引数化した共通実装
/// （イシュー #401: `url_validation_check` の U1〜U3 判定が同一の「コード文脈
/// 限定・識別子左境界チェック」を `set_attribute` / `is_safe_url` 等の別 needle
/// へ適用する必要が生じたための一般化）。挙動は `raw_html` 呼び出し検出時と
/// 完全に同一（呼び出し元の `find_raw_html_call_positions` は薄いラッパーとして
/// 維持し、既存テストで挙動不変を保証する）。
///
/// `exclude_fn_defs` が `true` の場合、[`is_fn_definition_call`] により
/// `fn <needle>(...)` という関数定義行自体へのマッチを除外する（U3: ガード
/// 関数の「呼び出し」のみを数え、定義行を呼び出しと誤認しないため）。
fn find_code_context_call_positions(
    content: &str,
    needle: &[u8],
    exclude_fn_defs: bool,
) -> Vec<usize> {
    let bytes = content.as_bytes();
    let mask = code_context_mask(content);
    let mut positions = Vec::new();
    let mut start = 0;
    while let Some(rel) = find_subslice(&bytes[start..], needle) {
        let match_start = start + rel;
        let in_code_context = mask.get(match_start).copied().unwrap_or(true);
        let has_ident_left_neighbor = match_start > 0
            && (bytes[match_start - 1].is_ascii_alphanumeric() || bytes[match_start - 1] == b'_');
        if in_code_context && !has_ident_left_neighbor {
            let mut i = match_start + needle.len();
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < bytes.len()
                && bytes[i] == b'('
                && !(exclude_fn_defs && is_fn_definition_call(bytes, match_start))
            {
                positions.push(match_start);
            }
        }
        start = match_start + 1;
        if start >= bytes.len() {
            break;
        }
    }
    positions
}

/// `match_start` の呼び出しらしき出現が `fn <name>(` という関数定義自体か
/// どうかを、直前の非空白トークンが独立した `fn` キーワードかで近似判定する
/// （イシュー #401 U3: `fn is_safe_url(...)` の定義行をガード関数の
/// 「呼び出し」として誤って数えないため）。`pub fn` / `pub(crate) fn` 等の
/// 可視性修飾子は `fn` の直前に来るため対象外（`fn` トークンの左境界のみ見る）。
/// ジェネリクス付き宣言（`fn foo<T>(`）はこのリポジトリの対象関数群には
/// 存在しないため未対応（近似実装であることの明示、既知の限界）。
fn is_fn_definition_call(bytes: &[u8], match_start: usize) -> bool {
    let mut i = match_start;
    while i > 0 && bytes[i - 1].is_ascii_whitespace() {
        i -= 1;
    }
    if i >= 2 && &bytes[i - 2..i] == b"fn" {
        i - 2 == 0 || !(bytes[i - 3].is_ascii_alphanumeric() || bytes[i - 3] == b'_')
    } else {
        false
    }
}

/// Rust ソース `content` の各バイト位置が「コード文脈」か否かを表すマスクを
/// 構築する状態機械（イシュー #372）。[`find_raw_html_call_positions`] の
/// 偽陽性（コメント・文字列リテラル中の `raw_html(` 言及の誤検知）を削減する
/// ための専用の簡易近似実装であり、完全な Rust 字句解析器ではない
/// （正規表現・字句解析クレートを使わず手書きで判定する。`cli` 外部依存
/// ゼロ方針）。
///
/// 除外する非コード文脈:
/// - 行コメント（`//` `///` `//!`）・ブロックコメント（`/* */`、ネスト対応）
/// - 通常文字列リテラル（`"..."`、`\` エスケープ対応）・バイト文字列
///   （`b"..."`）
/// - raw 文字列リテラル（`r"..."` / `r#"..."#` 等）・raw バイト文字列
///   （`br"..."` 等）
/// - 文字リテラル（`'x'`、エスケープ・Unicode エスケープ対応）
///
/// 文字リテラルとライフタイム（`'a`）の判別は「`'` の直後がエスケープ列
/// または任意 1 文字＋閉じ `'`」であるかの近似で行う。判別不能・曖昧な
/// 入力（ライフタイムの可能性を排除できない `'`、不正な UTF-8 境界等）は
/// **コード文脈（true）側に倒す**（fail-closed、security.md A05: 保険層の
/// 偽陰性ゼロを優先し、偽陽性の残存は許容する）。
fn code_context_mask(content: &str) -> Vec<bool> {
    #[derive(Clone, Copy)]
    enum State {
        Code,
        LineComment,
        BlockComment(u32),
        Str,
        RawStr(u32),
    }

    let bytes = content.as_bytes();
    let mut mask = vec![true; bytes.len()];
    let mut state = State::Code;
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        match state {
            State::Code => {
                let prev_is_ident_char =
                    i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
                if b == b'/' && bytes.get(i + 1) == Some(&b'/') {
                    mask[i] = false;
                    mask[i + 1] = false;
                    state = State::LineComment;
                    i += 2;
                    continue;
                }
                if b == b'/' && bytes.get(i + 1) == Some(&b'*') {
                    mask[i] = false;
                    mask[i + 1] = false;
                    state = State::BlockComment(1);
                    i += 2;
                    continue;
                }
                if b == b'"' {
                    mask[i] = false;
                    state = State::Str;
                    i += 1;
                    continue;
                }
                // raw 文字列 / raw バイト文字列プレフィックス（`r"..`, `r#"..`#,
                // `br"..`, `br#"..`#）。トークンの途中（識別子のサフィックスと
                // しての `r`/`b`、例: `bar"x"` は構文上不成立だが念のため）で
                // 誤爆しないよう、直前バイトが識別子構成文字でないことを要求する。
                if !prev_is_ident_char {
                    if b == b'r' {
                        if let Some(hashes) = raw_string_hash_count(bytes, i + 1) {
                            mask[i] = false;
                            let mut j = i + 1;
                            for _ in 0..hashes {
                                mask[j] = false;
                                j += 1;
                            }
                            mask[j] = false; // 開き `"`
                            state = State::RawStr(hashes as u32);
                            i = j + 1;
                            continue;
                        }
                    }
                    if b == b'b' {
                        if bytes.get(i + 1) == Some(&b'"') {
                            mask[i] = false;
                            mask[i + 1] = false;
                            state = State::Str;
                            i += 2;
                            continue;
                        }
                        if bytes.get(i + 1) == Some(&b'r') {
                            if let Some(hashes) = raw_string_hash_count(bytes, i + 2) {
                                mask[i] = false;
                                mask[i + 1] = false;
                                let mut j = i + 2;
                                for _ in 0..hashes {
                                    mask[j] = false;
                                    j += 1;
                                }
                                mask[j] = false; // 開き `"`
                                state = State::RawStr(hashes as u32);
                                i = j + 1;
                                continue;
                            }
                        }
                    }
                }
                if b == b'\'' {
                    if let Some(end) = char_literal_end(bytes, i) {
                        mask[i..=end].fill(false);
                        i = end + 1;
                        continue;
                    }
                    // ライフタイムまたは判別不能: Code のまま次バイトへ進める
                    // （fail-closed: 誤ってコメント/文字列扱いにしない）。
                }
                i += 1;
            }
            State::LineComment => {
                mask[i] = false;
                if b == b'\n' {
                    state = State::Code;
                }
                i += 1;
            }
            State::BlockComment(depth) => {
                mask[i] = false;
                if b == b'/' && bytes.get(i + 1) == Some(&b'*') {
                    mask[i + 1] = false;
                    state = State::BlockComment(depth + 1);
                    i += 2;
                    continue;
                }
                if b == b'*' && bytes.get(i + 1) == Some(&b'/') {
                    mask[i + 1] = false;
                    state = if depth <= 1 {
                        State::Code
                    } else {
                        State::BlockComment(depth - 1)
                    };
                    i += 2;
                    continue;
                }
                i += 1;
            }
            State::Str => {
                mask[i] = false;
                if b == b'\\' {
                    if i + 1 < bytes.len() {
                        mask[i + 1] = false;
                    }
                    i += 2;
                    continue;
                }
                if b == b'"' {
                    state = State::Code;
                }
                i += 1;
            }
            State::RawStr(hashes) => {
                mask[i] = false;
                if b == b'"' {
                    let mut k = 0u32;
                    while k < hashes && bytes.get(i + 1 + k as usize) == Some(&b'#') {
                        k += 1;
                    }
                    if k == hashes {
                        for j in 0..hashes as usize {
                            mask[i + 1 + j] = false;
                        }
                        i += 1 + hashes as usize;
                        state = State::Code;
                        continue;
                    }
                }
                i += 1;
            }
        }
    }
    mask
}

/// `bytes[start..]` が raw 文字列の「開き部分」（0 個以上の `#` の後に `"`）
/// として成立するかを判定し、成立する場合はハッシュ数を返す
/// （[`code_context_mask`] の raw 文字列プレフィックス判定の共通処理）。
fn raw_string_hash_count(bytes: &[u8], start: usize) -> Option<usize> {
    let mut i = start;
    let mut hashes = 0usize;
    while bytes.get(i) == Some(&b'#') {
        hashes += 1;
        i += 1;
    }
    if bytes.get(i) == Some(&b'"') {
        Some(hashes)
    } else {
        None
    }
}

/// `bytes[quote_pos]`（`'`）が文字リテラルの開始であるとみなせる場合、
/// 閉じ `'` のバイト位置を返す。ライフタイム（`'a` 等）や判別不能な入力では
/// `None` を返す（[`code_context_mask`] 呼び出し元が Code 文脈のまま扱う）。
fn char_literal_end(bytes: &[u8], quote_pos: usize) -> Option<usize> {
    let start = quote_pos + 1;
    match bytes.get(start) {
        Some(b'\\') => {
            // エスケープ列: `\u{...}` / `\xNN` / 単純エスケープ（`\n` `\t` `\\` `\'` `\"` `\0` 等）。
            let mut i = start + 1;
            if bytes.get(i) == Some(&b'u') && bytes.get(i + 1) == Some(&b'{') {
                i += 2;
                while bytes.get(i).is_some_and(|c| *c != b'}') {
                    i += 1;
                }
                if bytes.get(i) == Some(&b'}') {
                    i += 1;
                }
            } else if bytes.get(i) == Some(&b'x') {
                i += 1;
                for _ in 0..2 {
                    if bytes.get(i).is_some_and(u8::is_ascii_hexdigit) {
                        i += 1;
                    }
                }
            } else if i < bytes.len() {
                i += 1;
            }
            if bytes.get(i) == Some(&b'\'') {
                Some(i)
            } else {
                None
            }
        }
        Some(&c) if c != b'\'' => {
            // 通常の 1 文字（マルチバイト UTF-8 文字を含む）+ 閉じ `'`。
            let char_len = utf8_char_len(c);
            let after = start + char_len;
            if bytes.get(after) == Some(&b'\'') {
                Some(after)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// UTF-8 先頭バイトからその文字の総バイト長を求める（不正な先頭バイトは
/// 1 バイトとして扱い、無限ループ・パニックを避ける保守側の近似）。
fn utf8_char_len(first_byte: u8) -> usize {
    if first_byte & 0x80 == 0 {
        1
    } else if first_byte & 0xE0 == 0xC0 {
        2
    } else if first_byte & 0xF0 == 0xE0 {
        3
    } else if first_byte & 0xF8 == 0xF0 {
        4
    } else {
        1
    }
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
///
/// `pub(crate)`: `loaders.rs`（`extract_loader_impls_from_source`、イシュー #353）も
/// 同種の「文字列リテラル内の疑似マッチを誤検知しない」判定を必要とするため、
/// 同一ロジックを共有する（重複実装しない）。
pub(crate) fn position_is_inside_string_literal(line: &str, pos: usize) -> bool {
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
/// 走査対象外。PoC-7 の粒度を維持する）を走査し、テストターゲット内の
/// `raw_html()` 利用は本関数ではなく `lint` チェック（`--all-targets` 付き
/// `cargo clippy`、イシュー #315）が検出を担う役割分担とし、
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
        let src_dir = escape_check_src_dir(project_dir, &dir.name);
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

// --- `url_validation_check`（イシュー #401） ---
//
// `core/src/url.rs`（イシュー #373、PR #386）が導入した URL スキーム検証
// （`is_safe_url`/`is_safe_srcset`/`is_url_attr`/`is_event_handler_attr`、
// 正本 allowlist `URL_ATTRS`）は SSR（`render_into`）・CSR 実 DOM
// （`wasm-client::binding_dom::apply_one`/`keyed_dom::build_element`）の
// 3 経路へ適用されているが、この保証はレビュー・テストのみに依存しており
// `fw gate` は弱体化（検証を経由しない新規経路の追加・allowlist の緩和・
// 既存ガード呼び出しの削除）を機械検出できていなかった。本セクションは
// `default_escape_check` と同型の「外部コマンド起動なしの純粋関数走査」
// （保険層。行動保証の本体は XSS 回帰テスト = `test` チェックが担う）として
// 3 ルール（U1〜U3）を実装する。判定ルールの正式な定義は
// `docs/design/gate-design.md` §2.4 を参照。

/// `set_attribute` 系 DOM 属性設定 API の呼び出し検出 needle（U1）。
/// `set_attribute` は `set_attribute_ns`/`set_attribute_node` の呼び出しの
/// 前方一致にはならない（`find_code_context_call_positions` は needle 直後の
/// 空白を読み飛ばした先が `(` である場合のみマッチとするため、`_ns(`/`_node(`
/// が続く出現はここでは捕捉されず、専用 needle 側でのみ捕捉される）。
const URL_SINK_NEEDLES: &[&[u8]] = &[b"set_attribute", b"set_attribute_ns", b"set_attribute_node"];

/// URL 検証ガード関数 4 種の呼び出し needle（U1: 同一ファイル内の共起判定 /
/// U3: core ディレクトリ内の実在判定）。`core/src/url.rs` が公開する契約
/// （[`core::url`] doc コメント参照）と同一の 4 種で固定する。
const URL_GUARD_NEEDLES: &[&[u8]] = &[
    b"is_url_attr",
    b"is_safe_url",
    b"is_safe_srcset",
    b"is_event_handler_attr",
];

/// U2 でピンする `URL_ATTRS`（`core/src/url.rs`）の許可属性 12 種。削除
/// （＝ allowlist の緩和）を検出する基準集合。追加は強化のため許容する
/// （U2 判定は「ピン集合をすべて含むか」の片方向チェック）。
const URL_VALIDATION_PINNED_ATTRS: &[&str] = &[
    "href",
    "src",
    "action",
    "formaction",
    "xlink:href",
    "poster",
    "cite",
    "data",
    "background",
    "ping",
    "dynsrc",
    "lowsrc",
];

/// U2 でピンする `is_safe_url`（`core/src/url.rs`）の許可スキーム 4 種。
/// 同ファイル内の `eq_ignore_ascii_case("<literal>")` 比較リテラル集合が
/// この集合の部分集合であることを要求する（スキーム追加＝緩和の検出）。
const URL_VALIDATION_PINNED_SCHEMES: &[&str] = &["http", "https", "mailto", "tel"];

/// `role = "core"` ディレクトリの `src/` 配下から `URL_ATTRS` **定義**ファイルを
/// 特定するための needle。単なる `URL_ATTRS` 識別子ではなく `const URL_ATTRS`
/// という定義パターンを要求する（`core/src/lib.rs` の
/// `pub use url::{..., URL_ATTRS};` のような再エクスポート・言及箇所を定義と
/// 誤認し、無関係な後続 `];`（他の配列リテラル等）までを誤ってブロックとして
/// 抽出してしまう偽陽性を防ぐ）。
const URL_ATTRS_DEFINITION_NEEDLE: &str = "const URL_ATTRS";

/// URL 属性検証の弱体化を検出する（イシュー #401、U1〜U3、`GateCheck` 名
/// `url_validation_check`）。`run_all_checks` から `default_escape_check` の
/// 直後に組み込まれる（JSON `checks` 配列の並び、`docs/design/gate-design.md`
/// §4 の出力契約）。
///
/// # 判定ルール
///
/// - **U1**（非 core ディレクトリ）: [`URL_SINK_NEEDLES`] のいずれかを呼ぶ
///   ファイルが、同一ファイル内に [`URL_GUARD_NEEDLES`] の 4 種すべてを
///   呼んでいない場合、呼び出しごとに違反とする。ファイル単位の共起判定の
///   ため「同一ファイル内にガード済み呼び出しと未ガード呼び出しが併存」は
///   見逃す（既知の限界。行動保証の本体は XSS 回帰テストが担う）。
/// - **U2**（core ディレクトリ、allowlist のピン検査）: [`URL_ATTRS_DEFINITION_NEEDLE`]
///   を持つファイルが存在しない場合、属性/スキーム集合がピンを満たさない
///   場合、ガード関数 4 種の定義が見当たらない場合はいずれも違反
///   （fail-closed）。
/// - **U3**（core ディレクトリ、ガード呼び出しの実在）: [`URL_GUARD_NEEDLES`]
///   それぞれについて、定義行を除いたコード文脈の呼び出しが core src 内に
///   1 箇所も無ければ違反（`render_into` からのガード削除の検出）。
///   `is_safe_url` は `is_safe_srcset` 内部呼び出しで自明に成立する
///   （既知の限界、実効的な検出対象は他 3 種）。
///
/// core role が `structure.toml` に宣言されていないプロジェクト（`fw new`
/// 生成物・既存フィクスチャの多くは `vendor/` を意図的に非宣言）では U2/U3
/// は対象なしで素通しする（`template_vendor_drift.rs` が vendor 配下の
/// ドリフト検知を別途担う）。
fn url_validation_check(manifest: &StructureManifest, project_dir: &Path) -> GateCheck {
    let mut violations: Vec<String> = Vec::new();

    let mut core_src_dirs: Vec<PathBuf> = Vec::new();
    for dir in &manifest.directories {
        let src_dir = escape_check_src_dir(project_dir, &dir.name);
        if !src_dir.is_dir() {
            continue;
        }
        if dir.role == Role::Core {
            core_src_dirs.push(src_dir);
        } else {
            check_url_sink_guard_cooccurrence(&src_dir, &mut violations);
        }
    }

    if !core_src_dirs.is_empty() {
        check_core_url_validation_module(&core_src_dirs, &mut violations);
        check_core_guard_calls_exist(&core_src_dirs, &mut violations);
    }

    violations.sort();
    let passed = violations.is_empty();
    let output = if passed {
        "no URL validation weakening detected".to_string()
    } else {
        truncate_output(&violations.join("\n"))
    };
    GateCheck {
        name: "url_validation_check",
        passed,
        output,
    }
}

/// U1: `src_dir` 配下の各 `*.rs` ファイルについて、[`URL_SINK_NEEDLES`] の
/// 呼び出しが存在するのに [`URL_GUARD_NEEDLES`] 4 種すべてを同一ファイル内で
/// 呼んでいない場合を違反として `violations` へ追記する。
fn check_url_sink_guard_cooccurrence(src_dir: &Path, violations: &mut Vec<String>) {
    walk_rs_files(src_dir, &mut |path| {
        let Ok(content) = std::fs::read_to_string(path) else {
            return;
        };
        let sink_positions: Vec<usize> = URL_SINK_NEEDLES
            .iter()
            .flat_map(|needle| find_code_context_call_positions(&content, needle, false))
            .collect();
        if sink_positions.is_empty() {
            return;
        }
        let has_all_guards = URL_GUARD_NEEDLES
            .iter()
            .all(|needle| !find_code_context_call_positions(&content, needle, false).is_empty());
        if has_all_guards {
            return;
        }
        let line_starts = line_start_offsets(&content);
        let mut sorted_positions = sink_positions;
        sorted_positions.sort_unstable();
        for pos in sorted_positions {
            let line_idx = offset_to_line_idx(&line_starts, pos);
            violations.push(format!(
                "{}:{}: DOM attribute sink call without co-located URL validation guards \
(is_url_attr/is_safe_url/is_safe_srcset/is_event_handler_attr) in the same file",
                path.display(),
                line_idx + 1
            ));
        }
    });
}

/// U2: `core_src_dirs` 配下から `URL_ATTRS` 定義ファイルを特定し、allowlist
/// （属性集合・スキーム集合・ガード関数定義）の緩和を検出する。
fn check_core_url_validation_module(core_src_dirs: &[PathBuf], violations: &mut Vec<String>) {
    let mut found_definition = false;

    for src_dir in core_src_dirs {
        walk_rs_files(src_dir, &mut |path| {
            let Ok(content) = std::fs::read_to_string(path) else {
                return;
            };
            let mask = code_context_mask(&content);
            let Some(def_pos) =
                find_code_context_occurrence(&content, &mask, URL_ATTRS_DEFINITION_NEEDLE)
            else {
                return;
            };
            found_definition = true;

            // `URL_ATTRS` 定義ブロックを、出現位置から直近の `];` までとして
            // 抽出する（`core/src/url.rs` の `pub const URL_ATTRS: &[&str] = &[...]`
            // 形式を前提とする単純な近似。ネストした配列リテラルは本リポジトリの
            // 定義に現れないため未対応、既知の限界）。
            let block_end = content[def_pos..]
                .find("];")
                .map(|rel| def_pos + rel)
                .unwrap_or(content.len());
            let block = &content[def_pos..block_end];
            let attrs = extract_string_literals(block);
            for pinned in URL_VALIDATION_PINNED_ATTRS {
                if !attrs.iter().any(|a| a == pinned) {
                    violations.push(format!(
                        "{}: URL_ATTRS allowlist is missing pinned attribute \"{pinned}\" \
(allowlist relaxation detected)",
                        path.display()
                    ));
                }
            }

            // スキーム比較リテラル: `eq_ignore_ascii_case("<literal>")` の
            // 引数がピン集合の部分集合であることを要求する（コード文脈限定、
            // ファイル全体を対象にする。`is_safe_url` 本体のみが該当する想定）。
            let schemes = extract_eq_ignore_ascii_case_literals(&content, &mask);
            for scheme in &schemes {
                if !URL_VALIDATION_PINNED_SCHEMES
                    .iter()
                    .any(|pinned| pinned.eq_ignore_ascii_case(scheme))
                {
                    violations.push(format!(
                        "{}: is_safe_url compares against non-pinned scheme \"{scheme}\" \
(allowlist relaxation detected)",
                        path.display()
                    ));
                }
            }
        });
    }

    if !found_definition {
        violations.push(
            "no URL_ATTRS definition found in any core-role src/ directory (URL validation \
module missing)"
                .to_string(),
        );
        // URL_ATTRS モジュール自体が無ければガード定義の有無を判定する意味が
        // ないため、以降の定義探索はスキップする（重複違反の抑制）。
        return;
    }

    // ガード関数 4 種の定義存在チェックは `URL_ATTRS` 定義ファイルに限定せず
    // core src 全体を対象にする（`docs/design/gate-design.md` §2.4 U2:
    // 将来 url.rs の実装が複数ファイルへ分割されても検出が追従できるよう、
    // 「同一ファイル」制約を課さない）。
    for fn_name in [
        "is_safe_url",
        "is_safe_srcset",
        "is_url_attr",
        "is_event_handler_attr",
    ] {
        let needle = format!("fn {fn_name}");
        let mut found = false;
        for src_dir in core_src_dirs {
            walk_rs_files(src_dir, &mut |path| {
                if found {
                    return;
                }
                let Ok(content) = std::fs::read_to_string(path) else {
                    return;
                };
                let mask = code_context_mask(&content);
                if find_code_context_occurrence(&content, &mask, &needle).is_some() {
                    found = true;
                }
            });
        }
        if !found {
            violations.push(format!(
                "definition of `{fn_name}` not found in any core-role src/ directory \
(guard function definition removal detected)"
            ));
        }
    }
}

/// U3: `core_src_dirs` 配下で [`URL_GUARD_NEEDLES`] 4 種それぞれについて、
/// 定義行を除いたコード文脈の呼び出しが 1 箇所以上存在することを確認する。
fn check_core_guard_calls_exist(core_src_dirs: &[PathBuf], violations: &mut Vec<String>) {
    for guard in URL_GUARD_NEEDLES {
        let guard_name = String::from_utf8_lossy(guard).into_owned();
        let mut found = false;
        for src_dir in core_src_dirs {
            walk_rs_files(src_dir, &mut |path| {
                if found {
                    return;
                }
                let Ok(content) = std::fs::read_to_string(path) else {
                    return;
                };
                if !find_code_context_call_positions(&content, guard, true).is_empty() {
                    found = true;
                }
            });
        }
        if !found {
            violations.push(format!(
                "no call to `{guard_name}` found in any core-role src/ directory \
(guard call removal detected)"
            ));
        }
    }
}

/// `content` 中で `needle`（プレーンな部分文字列、呼び出し括弧を要求しない）
/// がコード文脈に最初に出現するバイト位置を返す（U2: `URL_ATTRS` 定義・
/// `fn <name>` 定義の検出に使う。[`find_code_context_call_positions`] とは
/// 異なり `(` の直後性を要求しない汎用版）。
fn find_code_context_occurrence(content: &str, mask: &[bool], needle: &str) -> Option<usize> {
    let bytes = content.as_bytes();
    let needle_bytes = needle.as_bytes();
    let mut start = 0;
    while let Some(rel) = find_subslice(&bytes[start..], needle_bytes) {
        let match_start = start + rel;
        if mask.get(match_start).copied().unwrap_or(true) {
            return Some(match_start);
        }
        start = match_start + 1;
        if start >= bytes.len() {
            break;
        }
    }
    None
}

/// `block` 内のダブルクォート文字列リテラル（`"..."`）の中身をすべて抽出する
/// （U2: `URL_ATTRS` 配列リテラルの要素抽出用）。`\"` エスケープには対応せず
/// 単純なクォート対で区切る（`core/src/url.rs` の実際の定義がエスケープを
/// 含まない単純な属性名リテラルのみのため、この近似で十分。既知の限界）。
fn extract_string_literals(block: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut chars = block.char_indices();
    while let Some((_, c)) = chars.next() {
        if c == '"' {
            let mut literal = String::new();
            for (_, c2) in chars.by_ref() {
                if c2 == '"' {
                    break;
                }
                literal.push(c2);
            }
            result.push(literal);
        }
    }
    result
}

/// `content` 中のコード文脈にある `eq_ignore_ascii_case("<literal>")` 呼び出し
/// の引数リテラルをすべて抽出する（U2: `is_safe_url` のスキーム比較検出用）。
fn extract_eq_ignore_ascii_case_literals(content: &str, mask: &[bool]) -> Vec<String> {
    let needle = "eq_ignore_ascii_case(\"";
    let bytes = content.as_bytes();
    let needle_bytes = needle.as_bytes();
    let mut result = Vec::new();
    let mut start = 0;
    while let Some(rel) = find_subslice(&bytes[start..], needle_bytes) {
        let match_start = start + rel;
        if mask.get(match_start).copied().unwrap_or(true) {
            let literal_start = match_start + needle_bytes.len();
            if let Some(end_rel) = content[literal_start..].find('"') {
                result.push(content[literal_start..literal_start + end_rel].to_string());
            }
        }
        start = match_start + 1;
        if start >= bytes.len() {
            break;
        }
    }
    result
}

/// `content` の各行の開始バイトオフセットを返す（[`scan_file_for_violations`]
/// と同一方針の前計算。マッチ位置 → 行番号変換を線形走査 1 回で済ませる）。
fn line_start_offsets(content: &str) -> Vec<usize> {
    let mut line_starts = Vec::new();
    let mut offset = 0usize;
    for line in content.lines() {
        line_starts.push(offset);
        offset += line.len() + 1;
    }
    line_starts
}

/// [`line_start_offsets`] の結果からバイト位置 `pos` が属する行インデックス
/// （0 始まり）を二分探索で求める。
fn offset_to_line_idx(line_starts: &[usize], pos: usize) -> usize {
    match line_starts.binary_search(&pos) {
        Ok(i) => i,
        Err(i) => i.saturating_sub(1),
    }
}

/// `structure.toml` の `[directories.<name>]` エントリ名 `dir_name` に対応する
/// 走査対象 `src/` ディレクトリを解決する。
///
/// 予約名 `root`（`crate::structure::ROOT_DIR_KEY`。クレートがプロジェクト
/// ルート直下 `<project_dir>/src` に直接配置される規約、`fw new`）を
/// 通常のエントリ名解釈（`<project_dir>/<name>/src`）で扱うと実在しない
/// `<project_dir>/root/src` を指してしまい `default_escape_check` が常に
/// スキップされる（PR #358 Bugbot 指摘、イシュー #351）。`root` 慣習の
/// ディレクトリ名 → 実パス解決は `crate::structure::dir_fs_path` を単一の
/// 情報源とし、本関数はそこに `src` を連結するだけの薄いラッパーとする
/// （個別特例をこの関数に閉じ込めない一般化、イシュー #353）。
fn escape_check_src_dir(project_dir: &Path, dir_name: &str) -> PathBuf {
    crate::structure::dir_fs_path(project_dir, dir_name).join("src")
}

/// `dir` 配下（再帰）の `*.rs` ファイルを走査し [`scan_file_for_violations`] を
/// 適用する（`default_escape_check` 専用の薄いラッパー、実体は
/// [`walk_rs_files`] に委譲）。
fn scan_dir_for_violations(dir: &Path, violations: &mut Vec<String>) {
    walk_rs_files(dir, &mut |path| scan_file_for_violations(path, violations));
}

/// `dir` 配下（再帰）の `*.rs` ファイルそれぞれについて `visit` を呼ぶ走査器
/// （イシュー #401: [`scan_dir_for_violations`]（`default_escape_check` 用）と
/// `url_validation_check`（U1/U2/U3）の双方が同じ「symlink 非追従・`.rs` 限定
/// 再帰走査」を必要としたための一般化）。
///
/// シンボリックリンク（ディレクトリ・ファイルいずれも）は辿らず無条件にスキップ
/// する。`path.is_dir()`（メタデータ経由でリンクを辿る）ではなく
/// `DirEntry::file_type()` の `is_symlink()` を明示チェックすることで、
/// 自己参照リンクによる無限再帰（fail-closed の実行自体を阻害する DoS）と、
/// プロジェクト外を指すリンクを辿ってのパストラバーサル（`.rs` ファイル内容が
/// 絶対パス付きで JSON レポートへ漏えいする経路）を防ぐ。`cli/src/routes.rs`
/// の `list_rs_files_inner`（レビュー指摘 #127 対応）と同一方針（OWASP A01/A05）。
///
/// I/O エラー（読み取り不可等）は違反として計上せず黙って読み飛ばす想定外
/// パスとし、スキャナ自体の堅牢性を優先する（`fw gate` 全体としては他チェック
/// の failed で fail-closed が働く）。
fn walk_rs_files(dir: &Path, visit: &mut dyn FnMut(&Path)) {
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
            walk_rs_files(&path, visit);
        } else if file_type.is_file() && path.extension().and_then(|e| e.to_str()) == Some("rs") {
            visit(&path);
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
/// `docs/policy/raw-html-review-gate.md` 参照）。
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
    use std::path::PathBuf;
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

    /// 呼び出しごとの `program`・`args` を記録しつつ、固定応答を順番に返す
    /// フェイク（G2: 外部コマンドの**起動引数契約**の検証専用。TASK-13.3d
    /// #142）。`FakeRunner` は引数を無視するため、`--locked` の脱落・
    /// `-D warnings` の脱落・`-p <crate>` 列挙の崩れ・`policy_check` の
    /// サブコマンド列（`advisories` を含めてはならない）といった弱体化を
    /// 検知できない（`docs/design/gate-design.md` §2 表・§5 A06 の回帰固定）。
    struct ArgsRecordingRunner {
        responses: Mutex<Vec<(bool, String)>>,
        calls: Mutex<Vec<(String, Vec<String>)>>,
    }

    impl ArgsRecordingRunner {
        fn new(responses: Vec<(bool, String)>) -> Self {
            Self {
                responses: Mutex::new(responses),
                calls: Mutex::new(Vec::new()),
            }
        }

        fn last_call(&self) -> (String, Vec<String>) {
            self.calls
                .lock()
                .unwrap()
                .last()
                .cloned()
                .expect("ArgsRecordingRunner.run() was never called")
        }
    }

    impl CommandRunner for ArgsRecordingRunner {
        fn run(&self, program: &str, args: &[&str], _cwd: &Path) -> (bool, String) {
            self.calls.lock().unwrap().push((
                program.to_string(),
                args.iter().map(|s| s.to_string()).collect(),
            ));
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
        // 6 チェックすべてがコマンド起動を試みるわけではない（escape_check /
        // url_validation_check は純粋関数、policy は deny.toml 欠落で早期
        // failed）。cargo 系 3 チェック分のフェイク応答を積む。
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

    // ------------------------------------------------------------------
    // イシュー #292: self-hosted runner の環境差（clippy component /
    // cargo-deny の有無）による `fw gate` 間欠 BLOCKED の解消。
    // 「コード起因の FAIL」と「ツール不在の環境エラー」を決定的に区別する
    // プリフライトの単体テスト。
    // ------------------------------------------------------------------

    #[test]
    fn clippy_environment_preflight_passes_through_when_clippy_available() {
        let dir = std::env::temp_dir();
        let runner = FakeRunner {
            responses: Mutex::new(vec![(true, "clippy 0.1.0".to_string())]),
        };
        assert!(clippy_environment_preflight(&runner, &dir).is_none());
    }

    #[test]
    fn clippy_environment_preflight_fails_closed_with_environment_error_when_unavailable() {
        let dir = std::env::temp_dir();
        let runner = FakeRunner {
            responses: Mutex::new(vec![(
                false,
                "error: no such subcommand: `clippy`".to_string(),
            )]),
        };
        let check = clippy_environment_preflight(&runner, &dir).expect("must fail closed");
        assert_eq!(check.name, "lint");
        assert!(!check.passed);
        assert!(
            check.output.starts_with(ENVIRONMENT_ERROR_PREFIX),
            "output={}",
            check.output
        );
        assert!(check.output.contains("rustup component add clippy"));
        assert!(check.output.contains("tools/ci/ensure-gate-tools.sh"));
    }

    #[test]
    fn cargo_deny_environment_preflight_passes_through_when_deny_available() {
        let dir = std::env::temp_dir();
        let runner = FakeRunner {
            responses: Mutex::new(vec![(true, "cargo-deny 0.16.4".to_string())]),
        };
        assert!(cargo_deny_environment_preflight(&runner, &dir).is_none());
    }

    #[test]
    fn cargo_deny_environment_preflight_fails_closed_with_environment_error_when_unavailable() {
        let dir = std::env::temp_dir();
        let runner = FakeRunner {
            responses: Mutex::new(vec![(
                false,
                "failed to launch `cargo`: No such file or directory".to_string(),
            )]),
        };
        let check = cargo_deny_environment_preflight(&runner, &dir).expect("must fail closed");
        assert_eq!(check.name, "policy");
        assert!(!check.passed);
        assert!(
            check.output.starts_with(ENVIRONMENT_ERROR_PREFIX),
            "output={}",
            check.output
        );
        assert!(check.output.contains("tools/ci/ensure-gate-tools.sh"));
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

    // イシュー #372: 走査の「コード文脈限定」への精密化。以下は誤検知解消
    // （passed 側）の固定テスト群。コメント・文字列リテラル・識別子サフィックス
    // は Rust の字句規則上 `raw_html()` の呼び出しになり得ないため、これらの
    // 除外は偽陽性のみを削る（docs/design/gate-design.md §2.2 参照）。

    #[test]
    fn find_raw_html_call_positions_ignores_line_comment_call_like_text() {
        assert!(
            find_raw_html_call_positions("// raw_html(x) is the opt-in escape hatch").is_empty()
        );
    }

    #[test]
    fn find_raw_html_call_positions_ignores_doc_comment_call_like_text() {
        assert!(find_raw_html_call_positions("/// raw_html()\nfn f() {}").is_empty());
    }

    #[test]
    fn find_raw_html_call_positions_ignores_block_comment_including_nested() {
        assert!(
            find_raw_html_call_positions("/* outer /* raw_html(x) */ still comment */").is_empty()
        );
    }

    #[test]
    fn find_raw_html_call_positions_ignores_string_literal_occurrence() {
        assert!(
            find_raw_html_call_positions("let msg = \"unreviewed raw_html(x) call\";").is_empty()
        );
    }

    #[test]
    fn find_raw_html_call_positions_ignores_raw_string_literal_occurrence() {
        assert!(find_raw_html_call_positions("let msg = r#\"raw_html(x)\"#;").is_empty());
    }

    #[test]
    fn find_raw_html_call_positions_ignores_identifier_suffix_occurrence() {
        // `fn detects_unreviewed_raw_html() {}` のような別識別子のサフィックスは
        // 呼び出しではない（cli/src/gate.rs 自身のテスト関数名がこの形）。
        assert!(find_raw_html_call_positions("fn detects_unreviewed_raw_html() {}").is_empty());
    }

    // 非弱体化（敵対的、failed 側）: 字句判定を狂わせる試みを含め、実際の
    // 呼び出しは引き続き検出されることを固定する。

    #[test]
    fn find_raw_html_call_positions_still_detects_call_after_char_literal_confusion_attempt() {
        // 文字リテラル `'"'` で文字列状態を狂わせようとしても、実呼び出しは
        // 検出されなければならない。
        assert_eq!(
            find_raw_html_call_positions("let _ = '\"'; raw_html(x);").len(),
            1
        );
    }

    #[test]
    fn find_raw_html_call_positions_still_detects_call_after_string_with_comment_marker() {
        // 文字列内の `/*` でコメント状態を狂わせようとしても、実呼び出しは
        // 検出されなければならない。
        assert_eq!(
            find_raw_html_call_positions("let a = \"/*\"; raw_html(x);").len(),
            1
        );
    }

    #[test]
    fn find_raw_html_call_positions_still_detects_method_form_call() {
        assert_eq!(find_raw_html_call_positions("node.raw_html(x);").len(), 1);
    }

    #[test]
    fn find_raw_html_call_positions_still_detects_path_qualified_call() {
        assert_eq!(
            find_raw_html_call_positions("rws_core::raw_html(x);").len(),
            1
        );
    }

    #[test]
    fn find_raw_html_call_positions_still_detects_call_immediately_after_comment() {
        assert_eq!(
            find_raw_html_call_positions("// note\nraw_html(x);").len(),
            1
        );
    }

    #[test]
    fn find_raw_html_call_positions_still_detects_call_immediately_after_string() {
        assert_eq!(
            find_raw_html_call_positions("let s = \"ok\"; raw_html(x);").len(),
            1
        );
    }

    /// イシュー #372 自己適用回帰: 本リポジトリ自身の `structure.toml` を
    /// [`structure::load`] で読み込み、`default_escape_check`（純粋関数・
    /// 外部コマンド起動なし）を本リポジトリ自身へ適用して passed を固定する。
    /// 将来ソースに未レビュー実呼び出しが混入すれば即座に本テストが検知する。
    #[test]
    fn default_escape_check_passes_on_this_repository_itself() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("cli crate must have a parent directory (repository root)");
        let structure_path = repo_root.join("structure.toml");
        if !structure_path.is_file() {
            // structure.toml がリポジトリ直下に未整備の場合、本テストは
            // 対象外とする（fw new 前提の環境差異。gate 本体の fail-closed
            // 契約には影響しない）。
            return;
        }
        let manifest = structure::load(&structure_path)
            .expect("repository structure.toml must be parseable for this regression test");
        let check = default_escape_check(&manifest, repo_root);
        assert!(
            check.passed,
            "default_escape_check must pass when self-applied to this repository: {}",
            check.output
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

    /// イシュー #353: 束縛点 API（`data-bind-text` 等の属性文字列・`bind_text(`
    /// 呼び出し）・keyed list（`keyed_list(` 呼び出し）を使うソースが
    /// `default_escape_check` に誤検知されないこと（新 API は `raw_html()` を
    /// 経由しないため無関係）。同一ファイル内の未レビュー `raw_html()` 呼び出しは
    /// 引き続き検出されること（新 API 混在時の見逃しがないこと）も併せて固定する。
    #[test]
    fn scan_file_ignores_new_api_usage_but_still_detects_unreviewed_raw_html() {
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-escape-new-api-mix-{}",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("lib.rs");
        std::fs::write(
            &file,
            "fn f() -> Node {\n    let n = bind_text(\"counter\", \"0\");\n    let m = keyed_list(\"items\", &[]);\n    let attr = \"data-bind-text\";\n    raw_html(x);\n    n\n}\n",
        )
        .unwrap();

        let mut violations = Vec::new();
        scan_file_for_violations(&file, &mut violations);
        assert_eq!(
            violations.len(),
            1,
            "束縛点/keyed list の利用自体は違反として検出されないはず（誤検知なし）: {violations:?}"
        );
        assert!(
            violations[0].contains("lib.rs:5"),
            "同一ファイル内の未レビュー raw_html() は引き続き検出されるはず: {violations:?}"
        );

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
    fn default_escape_check_scans_root_convention_directory() {
        // PR #358 Bugbot 指摘（イシュー #351）の回帰テスト: `structure.toml` の
        // 予約名 `root`（`templates/default/structure.toml` が採用する
        // 「クレートはプロジェクトルート直下 `src/` に配置される」規約）を
        // 素朴に `<project_dir>/root/src` と解釈すると実在しないパスとなり
        // 走査が常にスキップされ、`raw_html()` の未レビュー使用を検出できない
        // まま無意味な PASS を返してしまう。`<project_dir>/src` 直下に仕込んだ
        // 未レビュー `raw_html()` 呼び出しが実際に violation として検出される
        // ことを断定し、スキップされていないことを確認する。
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-escape-root-convention-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let root_src = dir.join("src");
        std::fs::create_dir_all(&root_src).unwrap();
        std::fs::write(root_src.join("lib.rs"), "raw_html(x);\n").unwrap();

        let manifest = StructureManifest {
            version: 1,
            directories: vec![structure::DirectoryEntry {
                name: "root".to_string(),
                role: Role::Distribution,
                crate_name: Some("rws-template-default".to_string()),
                description: "test".to_string(),
                depends_on: Vec::new(),
                allowed_dependents: Vec::new(),
            }],
            routing: None,
        };

        let check = default_escape_check(&manifest, &dir);
        assert!(
            !check.passed,
            "the `root` convention directory must be scanned (not skipped): {}",
            check.output
        );
        assert!(
            check.output.contains("lib.rs"),
            "violation output must reference the offending file: {}",
            check.output
        );

        let _ = std::fs::remove_dir_all(&dir);
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

    /// テスト専用: 単一 `role = "component"` ディレクトリを持つマニフェストを
    /// 組み立てる（`url_validation_check` U1 系テスト共通ヘルパー）。
    fn component_manifest(dir_name: &str) -> StructureManifest {
        StructureManifest {
            version: 1,
            directories: vec![structure::DirectoryEntry {
                name: dir_name.to_string(),
                role: Role::Component,
                crate_name: Some("rws-app".to_string()),
                description: "test".to_string(),
                depends_on: Vec::new(),
                allowed_dependents: Vec::new(),
            }],
            routing: None,
        }
    }

    /// テスト専用: 単一 `role = "core"` ディレクトリを持つマニフェストを
    /// 組み立てる（`url_validation_check` U2/U3 系テスト共通ヘルパー）。
    fn core_manifest(dir_name: &str) -> StructureManifest {
        StructureManifest {
            version: 1,
            directories: vec![structure::DirectoryEntry {
                name: dir_name.to_string(),
                role: Role::Core,
                crate_name: Some("rws-core".to_string()),
                description: "test".to_string(),
                depends_on: Vec::new(),
                allowed_dependents: Vec::new(),
            }],
            routing: None,
        }
    }

    /// `core/src/url.rs` の実定義を模した最小の URL 検証モジュールソース
    /// （`url_validation_check` U2 系テストの「現行相当で PASS」基準線）。
    fn valid_core_url_module_source() -> &'static str {
        r#"
pub const URL_ATTRS: &[&str] = &[
    "href",
    "src",
    "action",
    "formaction",
    "xlink:href",
    "poster",
    "cite",
    "data",
    "background",
    "ping",
    "dynsrc",
    "lowsrc",
];

pub fn is_url_attr(name: &str) -> bool {
    URL_ATTRS.iter().any(|a| a.eq_ignore_ascii_case(name))
}

pub fn is_event_handler_attr(name: &str) -> bool {
    name.len() > 2
        && name.as_bytes()[0].eq_ignore_ascii_case(&b'o')
        && name.as_bytes()[1].eq_ignore_ascii_case(&b'n')
}

pub fn is_safe_url(value: &str) -> bool {
    match extract_scheme(value) {
        None => true,
        Some(scheme) => {
            scheme.eq_ignore_ascii_case("http")
                || scheme.eq_ignore_ascii_case("https")
                || scheme.eq_ignore_ascii_case("mailto")
                || scheme.eq_ignore_ascii_case("tel")
        }
    }
}

fn extract_scheme(s: &str) -> Option<&str> {
    let colon_idx = s.find(':')?;
    Some(&s[..colon_idx])
}

pub fn is_safe_srcset(value: &str) -> bool {
    value.split(',').all(|candidate| {
        let url_part = candidate.split_whitespace().next().unwrap_or("");
        is_safe_url(url_part)
    })
}
"#
    }

    #[test]
    fn url_validation_check_flags_unguarded_set_attribute_call() {
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-url-u1-unguarded-{}",
            std::process::id()
        ));
        let app_src = dir.join("app").join("src");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&app_src).unwrap();
        std::fs::write(
            app_src.join("lib.rs"),
            "fn f(el: &Element, name: &str, v: &str) {\n    let _ = el.set_attribute(name, v);\n}\n",
        )
        .unwrap();

        let check = url_validation_check(&component_manifest("app"), &dir);
        assert!(
            !check.passed,
            "unguarded set_attribute call must be flagged"
        );
        assert!(check.output.contains("lib.rs:2"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn url_validation_check_passes_when_guards_co_located() {
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-url-u1-guarded-{}",
            std::process::id()
        ));
        let app_src = dir.join("app").join("src");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&app_src).unwrap();
        std::fs::write(
            app_src.join("lib.rs"),
            "fn f(el: &Element, name: &str, v: &str) {\n\
             \x20   if rws_core::is_event_handler_attr(name) { return; }\n\
             \x20   if rws_core::is_url_attr(name) && !rws_core::is_safe_url(v) { return; }\n\
             \x20   if !rws_core::is_safe_srcset(v) { return; }\n\
             \x20   let _ = el.set_attribute(name, v);\n}\n",
        )
        .unwrap();

        let check = url_validation_check(&component_manifest("app"), &dir);
        assert!(
            check.passed,
            "set_attribute co-located with all 4 guards must pass: {}",
            check.output
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn url_validation_check_detects_set_attribute_ns_call() {
        let dir =
            std::env::temp_dir().join(format!("fw-gate-test-url-u1-ns-{}", std::process::id()));
        let app_src = dir.join("app").join("src");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&app_src).unwrap();
        std::fs::write(
            app_src.join("lib.rs"),
            "fn f(el: &Element) {\n    let _ = el.set_attribute_ns(None, \"href\", \"x\");\n}\n",
        )
        .unwrap();

        let check = url_validation_check(&component_manifest("app"), &dir);
        assert!(
            !check.passed,
            "unguarded set_attribute_ns call must be flagged"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn url_validation_check_ignores_comment_and_string_occurrences() {
        // #372 と同一方針: コメント・文字列リテラル・doc コメント中の
        // `set_attribute(` 言及は誤検知しない（保険層の偽陽性抑制）。
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-url-u1-comment-{}",
            std::process::id()
        ));
        let app_src = dir.join("app").join("src");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&app_src).unwrap();
        std::fs::write(
            app_src.join("lib.rs"),
            "//! calls set_attribute(name, value) internally\n\
             // set_attribute(x, y) is called elsewhere\n\
             fn f() {\n    let s = \"set_attribute(x, y)\";\n    let _ = s.len();\n}\n",
        )
        .unwrap();

        let check = url_validation_check(&component_manifest("app"), &dir);
        assert!(
            check.passed,
            "comment/string occurrences of set_attribute( must not be flagged: {}",
            check.output
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn url_validation_check_passes_when_core_module_matches_pinned_baseline() {
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-url-u2-baseline-{}",
            std::process::id()
        ));
        let core_src = dir.join("core").join("src");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&core_src).unwrap();
        std::fs::write(core_src.join("url.rs"), valid_core_url_module_source()).unwrap();
        std::fs::write(
            core_src.join("lib.rs"),
            "fn f(v: &str) -> bool {\n    url::is_url_attr(\"href\") && url::is_safe_url(v) \
             && url::is_safe_srcset(v) && url::is_event_handler_attr(\"onclick\")\n}\n",
        )
        .unwrap();

        let check = url_validation_check(&core_manifest("core"), &dir);
        assert!(
            check.passed,
            "current core/src/url.rs-equivalent baseline must pass: {}",
            check.output
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn url_validation_check_fails_when_pinned_attribute_removed() {
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-url-u2-attr-removed-{}",
            std::process::id()
        ));
        let core_src = dir.join("core").join("src");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&core_src).unwrap();
        let weakened = valid_core_url_module_source().replace("    \"href\",\n", "");
        std::fs::write(core_src.join("url.rs"), weakened).unwrap();
        std::fs::write(
            core_src.join("lib.rs"),
            "fn f(v: &str) -> bool {\n    url::is_url_attr(\"src\") && url::is_safe_url(v) \
             && url::is_safe_srcset(v) && url::is_event_handler_attr(\"onclick\")\n}\n",
        )
        .unwrap();

        let check = url_validation_check(&core_manifest("core"), &dir);
        assert!(
            !check.passed,
            "removing a pinned URL_ATTRS entry (href) must be detected as a relaxation"
        );
        assert!(check.output.contains("href"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn url_validation_check_fails_when_scheme_added() {
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-url-u2-scheme-added-{}",
            std::process::id()
        ));
        let core_src = dir.join("core").join("src");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&core_src).unwrap();
        let weakened = valid_core_url_module_source().replace(
            "scheme.eq_ignore_ascii_case(\"tel\")",
            "scheme.eq_ignore_ascii_case(\"tel\")\n                || scheme.eq_ignore_ascii_case(\"ftp\")",
        );
        std::fs::write(core_src.join("url.rs"), weakened).unwrap();
        std::fs::write(
            core_src.join("lib.rs"),
            "fn f(v: &str) -> bool {\n    url::is_url_attr(\"src\") && url::is_safe_url(v) \
             && url::is_safe_srcset(v) && url::is_event_handler_attr(\"onclick\")\n}\n",
        )
        .unwrap();

        let check = url_validation_check(&core_manifest("core"), &dir);
        assert!(
            !check.passed,
            "adding a non-pinned scheme (ftp) to is_safe_url must be detected as a relaxation"
        );
        assert!(check.output.contains("ftp"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn url_validation_check_fails_when_core_role_has_no_url_module() {
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-url-u2-missing-module-{}",
            std::process::id()
        ));
        let core_src = dir.join("core").join("src");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&core_src).unwrap();
        std::fs::write(core_src.join("lib.rs"), "pub fn render() {}\n").unwrap();

        let check = url_validation_check(&core_manifest("core"), &dir);
        assert!(
            !check.passed,
            "core role declared without any URL_ATTRS module must fail-closed"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn url_validation_check_fails_when_guard_call_removed_from_core() {
        // U3: `fn is_url_attr` の定義のみが存在し、core src 内のどこからも
        // 呼ばれていない場合（`render_into` からのガード削除を模す）。
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-url-u3-no-call-{}",
            std::process::id()
        ));
        let core_src = dir.join("core").join("src");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&core_src).unwrap();
        std::fs::write(core_src.join("url.rs"), valid_core_url_module_source()).unwrap();
        // lib.rs はガード関数を一切呼ばない（url.rs 内部の is_safe_srcset →
        // is_safe_url 呼び出しのみが残る）。
        std::fs::write(core_src.join("lib.rs"), "pub fn render() {}\n").unwrap();

        let check = url_validation_check(&core_manifest("core"), &dir);
        assert!(
            !check.passed,
            "is_url_attr/is_event_handler_attr/is_safe_srcset with zero call sites must fail"
        );
        assert!(check.output.contains("is_url_attr"));
        assert!(check.output.contains("is_event_handler_attr"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn url_validation_check_skips_u2_u3_when_no_core_role_declared() {
        // 既存フィクスチャ・`fw new` 生成物のように core role が宣言されて
        // いないプロジェクトでは、U2/U3 は対象なしで素通しする。
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-url-no-core-role-{}",
            std::process::id()
        ));
        let app_src = dir.join("app").join("src");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&app_src).unwrap();
        std::fs::write(app_src.join("lib.rs"), "pub fn render() {}\n").unwrap();

        let check = url_validation_check(&component_manifest("app"), &dir);
        assert!(check.passed, "{}", check.output);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// イシュー #401 自己適用回帰: 本リポジトリ自身の `structure.toml` を
    /// 実読して `url_validation_check` を適用し、PASS を固定する。将来
    /// ソースが検証を弱体化させれば即座に本テストが検知する
    /// （`default_escape_check_passes_on_this_repository_itself` と同型）。
    #[test]
    fn url_validation_check_passes_on_this_repository_itself() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("cli crate must have a parent directory (repository root)");
        let structure_path = repo_root.join("structure.toml");
        if !structure_path.is_file() {
            return;
        }
        let manifest = structure::load(&structure_path)
            .expect("repository structure.toml must be parseable for this regression test");
        let check = url_validation_check(&manifest, repo_root);
        assert!(
            check.passed,
            "url_validation_check must pass when self-applied to this repository: {}",
            check.output
        );
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

    // ------------------------------------------------------------------
    // G8 (TASK-13.3d #142): `truncate_output` のマルチバイト文字境界安全性。
    // `chars()` ベースの丸めが char 境界を跨いで panic しないこと・
    // マルチバイト文字列でも末尾優先が保たれることを固定する
    // （docs/design/gate-design.md §5 A09）。
    // ------------------------------------------------------------------

    #[test]
    fn truncate_output_keeps_tail_for_multibyte_chars_without_panicking() {
        // 日本語 1 文字はマルチバイト（UTF-8 で 3 バイト）だが `chars()` は
        // 1 文字として数えるため、バイト境界とはズレる。ここでバイト単位の
        // 丸め実装への回帰（char 境界を跨いでの panic・文字化け）を防ぐ。
        let head = "先頭".repeat(OUTPUT_TRUNCATE_CHARS); // 十分な長さの先頭部分
        let tail = "末尾テスト文字列";
        let long = format!("{head}{tail}");
        let truncated = truncate_output(&long);
        assert_eq!(truncated.chars().count(), OUTPUT_TRUNCATE_CHARS);
        assert!(
            truncated.ends_with(tail),
            "tail must be preserved verbatim: {truncated:?}"
        );
    }

    // ------------------------------------------------------------------
    // G1 (TASK-13.3d #142): `policy_check` の単体テスト。
    // deny.toml 欠落時に cargo-deny を**起動しないこと**（fail-closed）、
    // 存在時の成功/失敗伝播・出力丸めを検証する（docs/design/gate-design.md §2
    // 表 #5・§3 fail-closed）。
    // ------------------------------------------------------------------

    #[test]
    fn policy_check_does_not_invoke_cargo_deny_when_deny_toml_missing() {
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-policy-no-deny-toml-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let check = policy_check(&PanicIfCalledRunner, &dir);
        assert!(!check.passed);
        assert!(check.output.contains("deny.toml not found"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn policy_check_invokes_cargo_deny_with_expected_args_when_deny_toml_present() {
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-policy-deny-args-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("deny.toml"), "# test fixture\n").unwrap();

        // イシュー #292: `policy_check` はプリフライト（`cargo deny --version`）
        // → 本実行の順で 2 回 `runner.run` を呼ぶため、応答を 2 つ積む。
        // `last_call()` は最後の呼び出し（本実行）を指すため、既存のアサーションは
        // そのまま「本実行の引数契約」を検証し続ける。
        let runner = ArgsRecordingRunner::new(vec![
            (true, "cargo-deny 0.16.4".to_string()),
            (true, "ok".to_string()),
        ]);
        let check = policy_check(&runner, &dir);
        assert!(check.passed);

        let (program, args) = runner.last_call();
        assert_eq!(program, "cargo");
        // `advisories` はオフラインゲート対象外（ネットワーク前提のため）。
        // 含めてしまうと offline 実行環境で誤って failed になる（弱体化とは
        // 逆方向の回帰だが、契約からの逸脱として同様に固定する）。
        assert_eq!(args, vec!["deny", "check", "bans", "licenses", "sources"]);
        assert!(
            !args.contains(&"advisories".to_string()),
            "policy_check must not include `advisories` (offline gate scope, docs/policy/cargo-deny-advisories.md)"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn policy_check_propagates_cargo_deny_failure_with_truncated_output() {
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-policy-deny-fail-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("deny.toml"), "# test fixture\n").unwrap();

        // プリフライトは成功させ（環境要因ではなくコード内容起因の失敗である
        // ことを固定する）、本実行のみを失敗させる。
        let long_output = "x".repeat(OUTPUT_TRUNCATE_CHARS + 100);
        let runner = ArgsRecordingRunner::new(vec![
            (true, "cargo-deny 0.16.4".to_string()),
            (false, long_output),
        ]);
        let check = policy_check(&runner, &dir);
        assert!(!check.passed);
        assert_eq!(check.output.chars().count(), OUTPUT_TRUNCATE_CHARS);
        assert!(
            !check.output.starts_with(ENVIRONMENT_ERROR_PREFIX),
            "code-caused cargo-deny failure must not be mislabeled as an environment error"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ------------------------------------------------------------------
    // G2 (TASK-13.3d #142): 外部コマンドの起動引数契約。`--locked` の脱落
    // （A06 弱体化）・`-D warnings` の脱落・`-p <crate>` 列挙の崩れを
    // 検知できるようにする（docs/design/gate-design.md §2 表・§5 A06）。
    // ------------------------------------------------------------------

    #[test]
    fn run_cargo_check_invokes_cargo_with_locked_and_declared_crates() {
        let dir = std::env::temp_dir();
        let runner = ArgsRecordingRunner::new(vec![(true, "ok".to_string())]);
        let check = run_cargo_check(&runner, &dir, &["rws-core", "rws-app"]);
        assert!(check.passed);

        let (program, args) = runner.last_call();
        assert_eq!(program, "cargo");
        assert_eq!(
            args,
            vec!["check", "--locked", "-p", "rws-core", "-p", "rws-app"]
        );
    }

    #[test]
    fn run_cargo_test_invokes_cargo_with_locked_and_declared_crates() {
        let dir = std::env::temp_dir();
        let runner = ArgsRecordingRunner::new(vec![(true, "ok".to_string())]);
        let check = run_cargo_test(&runner, &dir, &["rws-core"]);
        assert!(check.passed);

        let (program, args) = runner.last_call();
        assert_eq!(program, "cargo");
        assert_eq!(args, vec!["test", "--locked", "-p", "rws-core"]);
    }

    #[test]
    fn run_cargo_clippy_invokes_cargo_with_locked_deny_warnings_and_declared_crates() {
        let dir =
            std::env::temp_dir().join(format!("fw-gate-test-clippy-args-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("clippy.toml"),
            "disallowed-methods = [{ path = \"rws_core::raw_html\", reason = \"x\" }]\n",
        )
        .unwrap();

        // イシュー #292: プリフライト（`cargo clippy --version`）→ 本実行の順で
        // 2 回 `runner.run` が呼ばれる。
        let runner = ArgsRecordingRunner::new(vec![
            (true, "clippy 0.1.0".to_string()),
            (true, "ok".to_string()),
        ]);
        let check = run_cargo_clippy(&runner, &dir, &["rws-core", "rws-app"]);
        assert!(check.passed);

        let (program, args) = runner.last_call();
        assert_eq!(program, "cargo");
        assert_eq!(
            args,
            vec![
                "clippy",
                "--locked",
                "--all-targets",
                "-p",
                "rws-core",
                "-p",
                "rws-app",
                "--",
                "-D",
                "warnings",
            ]
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ------------------------------------------------------------------
    // G3 (TASK-13.3d #142): 宣言クレート 0 件時、`run_cargo_check` /
    // `run_cargo_test` それぞれが共通ヘルパー経由の fail-closed を維持する
    // ことを個別に固定する（既存は clippy のみテスト済み。この 2 つも
    // ワークスペース全体へフォールバックしてはならない、docs/design/gate-design.md
    // §3）。
    // ------------------------------------------------------------------

    #[test]
    fn run_cargo_check_fails_closed_when_no_crates_declared() {
        let dir = std::env::temp_dir();
        let check = run_cargo_check(&PanicIfCalledRunner, &dir, &[]);
        assert!(!check.passed);
        assert!(check.output.contains("no crate declared"));
    }

    #[test]
    fn run_cargo_test_fails_closed_when_no_crates_declared() {
        let dir = std::env::temp_dir();
        let check = run_cargo_test(&PanicIfCalledRunner, &dir, &[]);
        assert!(!check.passed);
        assert!(check.output.contains("no crate declared"));
    }

    // ------------------------------------------------------------------
    // G4 (TASK-13.3d #142): `aggregate` の `action` 固定文言。集約結果の
    // JSON 契約の一部（docs/design/gate-design.md §4）であり、文言の意図しない変更を
    // 検知する。
    // ------------------------------------------------------------------

    #[test]
    fn aggregate_all_passed_action_text_is_fixed() {
        let checks = vec![GateCheck {
            name: "a",
            passed: true,
            output: String::new(),
        }];
        let report = aggregate(checks);
        assert_eq!(report.action, "all checks passed; changes may proceed");
    }

    #[test]
    fn aggregate_failure_action_text_is_fixed() {
        let checks = vec![GateCheck {
            name: "a",
            passed: false,
            output: String::new(),
        }];
        let report = aggregate(checks);
        assert_eq!(
            report.action,
            "fix the reported failing checks and re-run `fw gate`"
        );
    }

    // ------------------------------------------------------------------
    // G5/G7 (TASK-13.3d #142): `run_all_checks` が返す 6 チェックの name と
    // 順序（PoC-7 互換 JSON 形状）、および全チェック成功時に `gate_result`
    // が `"PASS"` になる経路を固定する（docs/design/gate-design.md §4 JSON 出力契約・
    // 集約規則）。フル e2e（実ツールチェーン走行）は TASK-13.4 #143 の
    // スコープのため、`FakeRunner` による軽量検証に留める。
    // ------------------------------------------------------------------

    /// `run_all_checks` の全 6 チェックを PASS させるためのフィクスチャ一式
    /// （`structure.toml` 相当のマニフェスト・`clippy.toml`・`deny.toml`・
    /// クリーンな `app/src`）を用意する。
    fn all_checks_pass_fixture() -> (StructureManifest, PathBuf) {
        // 呼び出し元 2 テスト（run_all_checks_returns_expected_check_names_in_order /
        // run_all_checks_all_success_yields_pass）は同一プロセス内で並列実行され
        // 得るため、`std::process::id()` のみではディレクトリ名が衝突し、
        // 片方の cleanup がもう片方の実行中フィクスチャを削除するレースが生じる。
        // ナノ秒タイムスタンプを付与して呼び出しごとに一意化する
        // （`gate_integration.rs` の `tempdir_for_test` と同じ戦略）。
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-run-all-checks-pass-{}-{unique}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let app_src = dir.join("app").join("src");
        std::fs::create_dir_all(&app_src).unwrap();
        // 未レビュー raw_html() 呼び出しを含まないクリーンなソース
        // （default_escape_check は純粋関数でありプロセス起動を伴わないため
        // フィクスチャのファイル内容がそのまま合否を左右する）。
        std::fs::write(app_src.join("lib.rs"), "pub fn render() {}\n").unwrap();
        std::fs::write(
            dir.join("clippy.toml"),
            "disallowed-methods = [{ path = \"rws_core::raw_html\", reason = \"x\" }]\n",
        )
        .unwrap();
        std::fs::write(dir.join("deny.toml"), "# test fixture\n").unwrap();

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
        (manifest, dir)
    }

    #[test]
    fn run_all_checks_returns_expected_check_names_in_order() {
        let (manifest, dir) = all_checks_pass_fixture();
        // type_check(1) + lint のプリフライト(1)+本実行(1) + test(1)
        // + policy のプリフライト(1)+本実行(1)（イシュー #292 で各 2 応答に増加）。
        let runner = FakeRunner {
            responses: Mutex::new(vec![
                (true, String::new()),
                (true, String::new()),
                (true, String::new()),
                (true, String::new()),
                (true, String::new()),
                (true, String::new()),
            ]),
        };
        let report = run_all_checks(&manifest, &dir, &runner);
        let names: Vec<&str> = report.checks.iter().map(|c| c.name).collect();
        assert_eq!(
            names,
            vec![
                "type_check",
                "default_escape_check",
                "url_validation_check",
                "lint",
                "test",
                "policy"
            ],
            "check name/order is a JSON output contract (PoC-7 compatibility, docs/design/gate-design.md §4)"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_all_checks_all_success_yields_pass() {
        let (manifest, dir) = all_checks_pass_fixture();
        // イシュー #292: lint / policy それぞれプリフライト分の応答が追加で必要
        // （run_all_checks_returns_expected_check_names_in_order と同数）。
        let runner = FakeRunner {
            responses: Mutex::new(vec![
                (true, String::new()),
                (true, String::new()),
                (true, String::new()),
                (true, String::new()),
                (true, String::new()),
                (true, String::new()),
            ]),
        };
        let report = run_all_checks(&manifest, &dir, &runner);
        assert_eq!(report.gate_result, "PASS");
        assert!(report.checks.iter().all(|c| c.passed), "{:?}", {
            report
                .checks
                .iter()
                .map(|c| (c.name, c.passed))
                .collect::<Vec<_>>()
        });

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ------------------------------------------------------------------
    // イシュー #410: `fw new --template embed` が生成する静的専用
    // （asset-only）プロジェクトに対する `fw gate` の明示的オプトインモード。
    // ------------------------------------------------------------------

    /// `role = "asset"`・`crate` キーなしの単一ディレクトリのみを宣言する
    /// マニフェスト（`templates/embed/structure.toml` 相当）。
    fn asset_only_manifest() -> StructureManifest {
        StructureManifest {
            version: 1,
            directories: vec![structure::DirectoryEntry {
                name: "root".to_string(),
                role: Role::Asset,
                crate_name: None,
                description: "test".to_string(),
                depends_on: Vec::new(),
                allowed_dependents: Vec::new(),
            }],
            routing: None,
        }
    }

    #[test]
    fn is_asset_only_project_true_for_all_asset_roles_with_no_crate() {
        assert!(is_asset_only_project(&asset_only_manifest()));
    }

    #[test]
    fn is_asset_only_project_false_when_a_declared_crate_exists() {
        assert!(!is_asset_only_project(&manifest_with_one_crate()));
    }

    #[test]
    fn is_asset_only_project_false_when_non_asset_role_mixed_in_with_no_crate() {
        // `crate` キーの削除し忘れ等で非 asset ロールが残っている設定不備を
        // 静的専用モードと誤認してはならない（fail-closed 境界の維持）。
        let manifest = StructureManifest {
            version: 1,
            directories: vec![structure::DirectoryEntry {
                name: "app".to_string(),
                role: Role::Component,
                crate_name: None,
                description: "test".to_string(),
                depends_on: Vec::new(),
                allowed_dependents: Vec::new(),
            }],
            routing: None,
        };
        assert!(!is_asset_only_project(&manifest));
    }

    #[test]
    fn run_all_checks_asset_only_project_passes_all_checks_without_invoking_cargo() {
        let manifest = asset_only_manifest();
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-asset-only-pass-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // 静的専用モードでは cargo 系チェックが cargo を一切起動してはならない
        // （`PanicIfCalledRunner` が起動されたら即座にテスト失敗として顕在化する）。
        let report = run_all_checks(&manifest, &dir, &PanicIfCalledRunner);

        assert_eq!(report.gate_result, "PASS");
        let names: Vec<&str> = report.checks.iter().map(|c| c.name).collect();
        assert_eq!(
            names,
            vec![
                "type_check",
                "default_escape_check",
                "url_validation_check",
                "lint",
                "test",
                "policy"
            ],
            "asset-only mode must keep the same 6-check JSON contract (name/order)"
        );
        assert!(report.checks.iter().all(|c| c.passed), "{:?}", {
            report
                .checks
                .iter()
                .map(|c| (c.name, c.passed))
                .collect::<Vec<_>>()
        });
        for name in ["type_check", "lint", "test", "policy"] {
            let check = report.checks.iter().find(|c| c.name == name).unwrap();
            assert!(
                check.output.contains("static-only project"),
                "not-applicable output for {name} must explain why cargo was not invoked"
            );
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_all_checks_asset_only_project_still_runs_default_escape_check() {
        // 静的専用モードでも保険層（`default_escape_check`）はバイパスしない。
        // asset ディレクトリ配下（`root` 慣習 → プロジェクトルート直下 `src/`）に
        // 未レビュー `raw_html()` 呼び出しが混入した場合は検出されなければならない
        // （security.md A05: 明示宣言によるオプトインが検証の全面停止を意味しない）。
        let manifest = asset_only_manifest();
        let dir = std::env::temp_dir().join(format!(
            "fw-gate-test-asset-only-violation-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let src_dir = dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(
            src_dir.join("injected.rs"),
            "fn f() { let _ = rws_core::raw_html(\"<b>x</b>\"); }\n",
        )
        .unwrap();

        let report = run_all_checks(&manifest, &dir, &PanicIfCalledRunner);

        assert_eq!(
            report.gate_result, "BLOCKED",
            "an unreviewed raw_html() call under an asset-only project must still block"
        );
        let escape_check = report
            .checks
            .iter()
            .find(|c| c.name == "default_escape_check")
            .unwrap();
        assert!(
            !escape_check.passed,
            "asset-only mode must not bypass default_escape_check (insurance layer)"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ------------------------------------------------------------------
    // G6 (TASK-13.3d #142): `render_report` の出力が機械的に有効な JSON で
    // あることをラウンドトリップ検証する（既存は部分文字列アサートのみ。
    // `cli` 自前パーサ `crate::json` を使えば外部依存ゼロのまま検証できる。
    // docs/design/gate-design.md §4・§5 A09）。
    // ------------------------------------------------------------------

    #[test]
    fn render_report_output_round_trips_through_json_parser() {
        let report = GateReport {
            checks: vec![
                GateCheck {
                    name: "type_check",
                    passed: true,
                    output: "ok".to_string(),
                },
                GateCheck {
                    name: "lint",
                    passed: false,
                    output: "warning: \"unused\"\ncontrol\x07char".to_string(),
                },
            ],
            gate_result: "BLOCKED",
            action: "fix the reported failing checks and re-run `fw gate`".to_string(),
        };
        let json_text = render_report(&report);

        let parsed =
            crate::json::parse(&json_text).expect("render_report output must be valid JSON");
        assert_eq!(
            parsed.get("gate_result").and_then(|v| v.as_str()),
            Some("BLOCKED")
        );
        assert_eq!(
            parsed.get("action").and_then(|v| v.as_str()),
            Some("fix the reported failing checks and re-run `fw gate`")
        );
        let checks = parsed
            .get("checks")
            .and_then(|v| v.as_array())
            .expect("checks must be a JSON array");
        assert_eq!(checks.len(), 2);
        assert_eq!(
            checks[0].get("name").and_then(|v| v.as_str()),
            Some("type_check")
        );
        assert_eq!(
            checks[1].get("output").and_then(|v| v.as_str()),
            Some("warning: \"unused\"\ncontrol\x07char"),
            "control characters and quotes must survive an escape/unescape round trip"
        );
    }
}
