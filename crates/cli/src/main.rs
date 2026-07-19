//! `fw`: AI 自己保守フック（REQ-13）向けの開発者・エージェント用 CLI エントリポイント。
//!
//! TASK-13.1（親 #127）の製品化対象。`structure` サブコマンドは
//! [`structure::load`]（TOML パース + セマンティック検証、TASK-13.1b）→
//! [`structure::StructureManifest::validate`]（宣言整合性検証、TASK-13.1a）→
//! [`metadata`] を用いた実体突き合わせ（TASK-13.1c）→ [`json_out`] による
//! JSON 出力、の順で処理する。いずれかの段階で失敗した場合は非 0 終了とし、
//! 呼び出し元（CI・AI 自己保守フック）が「構造チェック PASS」と誤認しないよう
//! 黙示的成功を返さない（`docs/design/structure-manifest.md` §4/§5、security.md A05）。

#![forbid(unsafe_code)]

mod component_boundary;
mod gate;
mod impact;
mod json;
mod json_out;
mod loaders;
mod metadata;
mod new;
mod new_template;
mod routes;
mod structure;
mod toml;

use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let exit_code = run(&args);
    std::process::exit(exit_code);
}

/// サブコマンドディスパッチ本体。`main` からテスト容易性のため分離する。
///
/// 戻り値はプロセスの終了コード。未知のサブコマンド・引数不足・検証違反は
/// 非 0（使用法エラー 2、検証違反 1）、正常終了は 0（xtask の `check-deps` 等と
/// 終了コード規約を統一）。
fn run(args: &[String]) -> i32 {
    match args.first().map(String::as_str) {
        Some("structure") => run_structure(&args[1..]),
        Some("gate") => gate::run_gate(&args[1..]),
        Some("impact") => run_impact(&args[1..]),
        Some("new") => new::run_new(&args[1..]),
        Some(other) => {
            eprintln!("fw: unknown subcommand `{other}`");
            print_usage();
            2
        }
        None => {
            eprintln!("fw: a subcommand is required");
            print_usage();
            2
        }
    }
}

fn print_usage() {
    eprintln!("Usage: fw <subcommand> [--project <dir>]");
    eprintln!("Subcommands:");
    eprintln!("  structure    generate/validate the machine-readable project structure manifest");
    eprintln!("  gate         run the AI self-maintenance verification gate (type/escape/lint/test/policy)");
    eprintln!("  impact       analyze the change impact of a symbol (breaking risk, affected crates/routes)");
    eprintln!("  new          deterministically scaffold a new project from templates/default");
}

/// `--project <dir>` 引数を解決する（省略時はカレントディレクトリ）。
///
/// `Ok(None)` は「引数の使い方が誤っている」（値の欠落・未知フラグ）ことを表し、
/// 呼び出し元は終了コード 2（使用法エラー）として扱う。
///
/// `pub(crate)`: `gate.rs`（`fw gate`, TASK-13.3）も同一の `--project` 引数解決を
/// 必要とするため、`structure` サブコマンドと実装を共有する（重複実装しない）。
pub(crate) fn parse_project_arg(args: &[String]) -> Result<PathBuf, ()> {
    match args {
        [] => std::env::current_dir().map_err(|_| ()),
        [flag, dir] if flag == "--project" => Ok(PathBuf::from(dir)),
        _ => Err(()),
    }
}

/// `structure` サブコマンド本体。
///
/// 1. `<project>/structure.toml` を [`structure::load`] でパース・セマンティック検証
/// 2. [`structure::StructureManifest::validate`] で宣言整合性を検証
/// 3. [`metadata::fetch`] で `cargo metadata` を実行し、宣言と実体を突き合わせる
///    （crate 実在・依存の宣言漏れ / 過剰宣言、ディレクトリ実在）
/// 4. [`routes::extract_routes`] / [`component_boundary::extract_public_symbols`] で
///    ルート定義・コンポーネント境界を抽出する
/// 5. すべて通過した場合のみ [`json_out::render`] の JSON を stdout へ出力し 0 終了。
///    途中で 1 件でも問題があれば、検出した違反をすべて stderr へ列挙し 1 終了する
///    （`validate()` と同様、最初の 1 件で打ち切らない）。
fn run_structure(args: &[String]) -> i32 {
    let project_dir = match parse_project_arg(args) {
        Ok(dir) => dir,
        Err(()) => {
            eprintln!("fw structure: usage: fw structure [--project <dir>]");
            return 2;
        }
    };

    let manifest_path = project_dir.join("structure.toml");
    let manifest = match structure::load(&manifest_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("fw structure: {e}");
            return 1;
        }
    };

    if let Err(errors) = manifest.validate() {
        for e in &errors {
            eprintln!("fw structure: {e}");
        }
        return 1;
    }

    let mut problems: Vec<String> = Vec::new();

    // ディレクトリ実在確認: `structure.toml` が宣言する各ディレクトリが
    // 実際にプロジェクト内に存在するかを確認する（TASK-13.1c の実体突き合わせ）。
    // 予約名 `root`（`structure::ROOT_DIR_KEY`。クレートがプロジェクトルート
    // 直下に配置される慣習、`fw new`）は `structure::dir_fs_path_for_entry` が
    // `project_dir` 自身へ写像するため、`fw new` 生成直後のプロジェクトでも
    // 「`<project>/root` が実在しない」という誤検知が起きない（イシュー #353）。
    // 任意の実配置パス（`path` キー、例: `crates/core`、イシュー #436）を持つ
    // エントリも同じ解決経路（`dir_fs_path_for_entry`）を単一の情報源として使う。
    for dir in &manifest.directories {
        let path = structure::dir_fs_path_for_entry(&project_dir, dir);
        if !path.is_dir() {
            problems.push(format!(
                "directories.{}: declared directory does not exist",
                dir.name
            ));
        }
    }

    // `cargo metadata` との突き合わせ。実行自体に失敗した場合（cargo 不在・
    // ワークスペース外指定等）は個別の依存整合性チェックを飛ばし、その旨のみ報告する
    // （fail-closed。metadata が取れないのに「依存は一致している」と誤認させない）。
    match metadata::fetch(&project_dir) {
        Ok(ws) => {
            check_crate_and_dependency_consistency(&manifest, &ws, &mut problems);

            if !problems.is_empty() {
                for p in &problems {
                    eprintln!("fw structure: {p}");
                }
                return 1;
            }

            let routes = collect_routes(&manifest, &project_dir, &mut problems);
            let component_boundary =
                collect_component_boundary(&manifest, &project_dir, &mut problems);

            if !problems.is_empty() {
                for p in &problems {
                    eprintln!("fw structure: {p}");
                }
                return 1;
            }

            let dependencies = collect_dependencies(&manifest, &ws);

            let output = json_out::StructureOutput {
                manifest: &manifest,
                routes,
                component_boundary,
                dependencies,
                resolved_package_count: ws.resolved_package_count(),
            };
            println!("{}", json_out::render(&output));
            0
        }
        Err(e) => {
            for line in format_metadata_failure_report(&problems, &e.to_string()) {
                eprintln!("{line}");
            }
            1
        }
    }
}

/// `cargo metadata` 呼び出しが失敗した際に出力すべき行を組み立てる。
///
/// `cargo metadata` 呼び出し前に検出済みのディレクトリ欠落違反（`problems`）が
/// metadata エラーの報告に置き換わって握りつぶされないよう、先行違反と metadata
/// エラーの両方を必ず含めて返す（レビュー指摘 #127: Medium severity。fetch 失敗時に
/// 先行違反が非表示になっていた）。純粋関数として切り出すことで、
/// `eprintln!` の副作用なしに出力内容をテストできるようにしている。
fn format_metadata_failure_report(problems: &[String], metadata_error: &str) -> Vec<String> {
    let mut lines: Vec<String> = problems
        .iter()
        .map(|p| format!("fw structure: {p}"))
        .collect();
    lines.push(format!(
        "fw structure: failed to cross-check with cargo metadata: {metadata_error}"
    ));
    lines
}

/// `crate` 宣言の実在確認と、`depends_on` 宣言 vs `cargo metadata` の実 path 依存の
/// 突き合わせ（宣言漏れ・過剰宣言の双方を違反として `problems` に積む）。
fn check_crate_and_dependency_consistency(
    manifest: &structure::StructureManifest,
    ws: &metadata::WorkspaceMetadata,
    problems: &mut Vec<String>,
) {
    // crate 名 -> ディレクトリ名の索引（実 depends_on の突き合わせに使う）。
    let crate_to_dir: Vec<(&str, &str)> = manifest
        .directories
        .iter()
        .filter_map(|d| d.crate_name.as_deref().map(|c| (c, d.name.as_str())))
        .collect();

    for dir in &manifest.directories {
        let Some(crate_name) = dir.crate_name.as_deref() else {
            continue;
        };
        let Some(member) = ws.member(crate_name) else {
            problems.push(format!(
                "directories.{}: declared crate `{crate_name}` is not a workspace member",
                dir.name
            ));
            continue;
        };

        // 実 path 依存（workspace member 同士の normal 依存）をディレクトリ名へ変換する。
        let mut actual_dep_dirs: Vec<&str> = member
            .normal_workspace_deps
            .iter()
            .filter_map(|dep_crate| {
                crate_to_dir
                    .iter()
                    .find(|(c, _)| c == dep_crate)
                    .map(|(_, dir_name)| *dir_name)
            })
            .collect();
        actual_dep_dirs.sort_unstable();

        let mut declared: Vec<&str> = dir.depends_on.iter().map(String::as_str).collect();
        declared.sort_unstable();

        for missing in actual_dep_dirs.iter().filter(|d| !declared.contains(d)) {
            problems.push(format!(
                "directories.{}: crate `{crate_name}` actually depends on `{missing}` (path dependency) but `depends_on` does not declare it",
                dir.name
            ));
        }
        for extra in declared.iter().filter(|d| !actual_dep_dirs.contains(d)) {
            problems.push(format!(
                "directories.{}: `depends_on` declares `{extra}` but crate `{crate_name}` has no such path dependency",
                dir.name
            ));
        }
    }
}

/// `[routing]` が宣言されている場合のみ [`routes::extract_routes`] を実行する。
///
/// 抽出自体が失敗した場合（走査対象がワークスペースルート外を指す・I/O エラー等）は
/// 空配列に握りつぶさず `problems` に積んで呼び出し元を非 0 終了させる（fail-closed。
/// REQ-13・本ファイル冒頭の doc コメントが明記する「黙示的成功を返さない」契約）。
fn collect_routes(
    manifest: &structure::StructureManifest,
    project_dir: &Path,
    problems: &mut Vec<String>,
) -> Vec<(String, Vec<routes::ExtractedRoute>)> {
    let Some(routing) = &manifest.routing else {
        return Vec::new();
    };
    // extractor は現時点で `fandhe-frontend-router-v1` のみ対応（`structure.toml` の
    // セマンティック検証では自由文字列を許容している）。未知の抽出器 ID を
    // 空の成功結果へ黙って倒すと `fw structure` が exit 0 かつ
    // `"routes":[]` を返してしまい、CI・AI 自己保守フックが誤設定・
    // 非対応 extractor を「ルートなし」として誤って成功扱いする
    // （レビュー指摘 #127: 本ファイル冒頭の「黙示的成功を返さない」契約に反する）。
    // fail-closed のため problems に積んで非 0 終了させる。
    if routing.extractor != "fandhe-frontend-router-v1" {
        problems.push(format!(
            "routing.extractor `{}`: unknown extractor (expected `fandhe-frontend-router-v1`)",
            routing.extractor
        ));
        return Vec::new();
    }
    // `definition_dir` はディレクトリキー名（論理名）。実配置パス（`path` キー、
    // イシュー #436）への解決を経てから走査する（`resolved_dir_path` が単一情報源）。
    let scan_path = manifest.resolved_dir_path(&routing.definition_dir);
    match routes::extract_routes(project_dir, &scan_path) {
        Ok(found) => vec![(routing.definition_dir.clone(), found)],
        Err(e) => {
            problems.push(format!(
                "routing.definition_dir `{}`: failed to extract routes: {e}",
                routing.definition_dir
            ));
            Vec::new()
        }
    }
}

/// `role = "component"` の各ディレクトリについてコンポーネント境界を抽出する。
///
/// 抽出失敗（走査対象がワークスペースルート外を指す・I/O エラー等）を空配列へ
/// 握りつぶさず `problems` に積む（[`collect_routes`] と同じ fail-closed 契約）。
fn collect_component_boundary(
    manifest: &structure::StructureManifest,
    project_dir: &Path,
    problems: &mut Vec<String>,
) -> Vec<(String, Vec<component_boundary::PublicSymbol>)> {
    manifest
        .directories
        .iter()
        .filter(|d| matches!(d.role, structure::Role::Component))
        .filter_map(|d| {
            // 実配置パス（イシュー #436、`path` キー）へ解決してから走査する。
            let scan_path = manifest.resolved_dir_path(&d.name);
            match component_boundary::extract_public_symbols(project_dir, &scan_path) {
                Ok(symbols) => Some((d.name.clone(), symbols)),
                Err(e) => {
                    problems.push(format!(
                        "directories.{}: failed to extract component boundary: {e}",
                        d.name
                    ));
                    None
                }
            }
        })
        .collect()
}

/// 実体（`cargo metadata`）から見た各ディレクトリの workspace 内依存一覧を組み立てる。
fn collect_dependencies(
    manifest: &structure::StructureManifest,
    ws: &metadata::WorkspaceMetadata,
) -> Vec<(String, Vec<String>)> {
    manifest
        .directories
        .iter()
        .filter_map(|d| {
            let crate_name = d.crate_name.as_deref()?;
            let member = ws.member(crate_name)?;
            Some((d.name.clone(), member.normal_workspace_deps.clone()))
        })
        .collect()
}

/// `impact` サブコマンド本体（TASK-13.2c, #135）。
///
/// `docs/design/impact-analysis-design.md` §3.5 の CLI 仕様・終了コード規約を実装する:
///
/// 1. 第 1 位置引数 `<symbol>` を取り出す（欠落時は使用法エラー、終了コード 2）
/// 2. [`impact::validate_symbol`] でシンボル名を検証する（シェル・走査へ渡す前の
///    A03 対策、`docs/design/impact-analysis-design.md` §6）。不正なら終了コード 2
/// 3. 残余引数を [`parse_project_arg`]（`structure` / `gate` と共有）で解決する
/// 4. [`metadata::fetch`] で `cargo metadata` を実行し、ワークスペースルート・
///    member 一覧を取得する（`fw structure` と同じ責務分担: `cargo` プロセス起動は
///    CLI 層が担い、`impact::analyze` は `&[MemberPackage]` を受け取るだけの
///    純粋なスキャン API に留める）。失敗時は検証違反（終了コード 1）とし、
///    黙示的成功に倒さない（security.md A05）
/// 5. [`impact::analyze`]（TASK-13.2b, #134 の走査エンジン）を呼び、結果を
///    終了コードへマッピングする: 成功 → 0 / [`impact::ImpactError::InvalidSymbol`]
///    → 2 / [`impact::ImpactError::SymbolNotFound`]・[`impact::ImpactError::Scan`] → 1
/// 6. 成功時は [`impact::render_report`]（TASK-13.2d, #136）で
///    `docs/design/impact-analysis-design.md` §3.5 の JSON スキーマへシリアライズして
///    stdout へ出力する。
const IMPACT_USAGE: &str = "fw impact: usage: fw impact <symbol> [--project <dir>]";

fn run_impact(args: &[String]) -> i32 {
    let Some(symbol) = args.first() else {
        eprintln!("fw impact: a <symbol> argument is required");
        eprintln!("{IMPACT_USAGE}");
        return 2;
    };

    if let Err(e) = impact::validate_symbol(symbol) {
        eprintln!("fw impact: {e}");
        eprintln!("{IMPACT_USAGE}");
        return 2;
    }

    let project_dir = match parse_project_arg(&args[1..]) {
        Ok(dir) => dir,
        Err(()) => {
            eprintln!("{IMPACT_USAGE}");
            return 2;
        }
    };

    // `cargo metadata` の実行自体に失敗した場合（cargo 不在・ワークスペース外
    // 指定等）は走査に進まず検証違反として扱う（fail-closed、security.md A05:
    // metadata が取れないのに「影響なし」と誤認させない）。
    let ws = match metadata::fetch(&project_dir) {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("fw impact: failed to cross-check with cargo metadata: {e}");
            return 1;
        }
    };

    match impact::analyze(&ws.workspace_root, &ws.members, symbol) {
        Ok(report) => {
            // JSON 出力（`docs/design/impact-analysis-design.md` §3.5 のスキーマ）。
            // 全文字列値は `render_report` 内部で `json_out::quoted`（`escape_str`
            // 経由）を通す契約であり、本呼び出し側では文字列を組み立てない
            // （security.md A08 対策）。
            println!("{}", impact::render_report(&report));
            0
        }
        // 使用法エラー（終了コード 2）: シンボル名は呼び出し前に検証済みだが、
        // `analyze` が独自に再検証して返す可能性を契約上排除しないため、
        // ここでも規約どおりマッピングする。
        Err(e @ impact::ImpactError::InvalidSymbol) => {
            eprintln!("fw impact: {e}");
            2
        }
        // 検証違反（終了コード 1）: 定義元が見つからない・走査失敗のいずれも
        // `defined_in: null` 等で黙って成功させない（fail-closed、security.md A05）。
        Err(e @ (impact::ImpactError::SymbolNotFound | impact::ImpactError::Scan(_))) => {
            eprintln!("fw impact: {e}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_subcommand_is_an_error() {
        assert_eq!(run(&[]), 2);
    }

    #[test]
    fn unknown_subcommand_is_an_error() {
        assert_eq!(run(&["bogus".to_string()]), 2);
    }

    #[test]
    fn structure_subcommand_with_bad_usage_is_an_error() {
        assert_eq!(
            run(&[
                "structure".to_string(),
                "--unknown-flag".to_string(),
                "x".to_string()
            ]),
            2
        );
    }

    #[test]
    fn structure_subcommand_reports_failure_for_missing_manifest() {
        let empty_dir = std::env::temp_dir().join("fw-structure-test-empty-project");
        let _ = std::fs::create_dir_all(&empty_dir);
        let code = run(&[
            "structure".to_string(),
            "--project".to_string(),
            empty_dir.to_string_lossy().into_owned(),
        ]);
        assert_eq!(code, 1);
    }

    #[test]
    fn structure_subcommand_succeeds_on_repository_root() {
        // このテストバイナリは `crates/cli/` 配下でビルドされるため、2 段の
        // 親ディレクトリでワークスペースルートを得る（イシュー #436）。
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("crates/cli/ has a workspace root two levels up");
        let code = run(&[
            "structure".to_string(),
            "--project".to_string(),
            workspace_root.to_string_lossy().into_owned(),
        ]);
        assert_eq!(code, 0);
    }

    /// fail-closed 回帰テスト: `routes::extract_routes` が失敗した場合に
    /// 「0 件でした」という虚偽の成功へフォールバックせず `problems` に積むこと
    /// （レビュー指摘 #127: main.rs 冒頭の「黙示的成功を返さない」契約に反していた）。
    #[test]
    fn collect_routes_reports_problem_when_extraction_fails() {
        let manifest = structure::StructureManifest {
            version: 1,
            directories: Vec::new(),
            routing: Some(structure::RoutingConfig {
                // ワークスペースルート直下に存在しないディレクトリ名。
                // `structure::is_valid_directory_name` を満たす形式のまま
                // 実体が欠落しているケース（I/O エラー相当）を模する。
                definition_dir: "does-not-exist".to_string(),
                extractor: "fandhe-frontend-router-v1".to_string(),
            }),
        };
        let project_dir = std::env::temp_dir().join("fw-collect-routes-test-project");
        let _ = std::fs::create_dir_all(&project_dir);

        let mut problems: Vec<String> = Vec::new();
        let routes = collect_routes(&manifest, &project_dir, &mut problems);

        assert!(routes.is_empty());
        assert_eq!(
            problems.len(),
            1,
            "extraction failure must be reported, not silently treated as zero routes"
        );
    }

    /// レビュー指摘 #127: Medium severity。`cargo metadata` 呼び出し前に検出済みの
    /// ディレクトリ欠落違反が、metadata 失敗時に出力から欠落していた
    /// （metadata エラーのみが出力され、先行違反が握りつぶされていた）。
    /// 両方が出力行に含まれることを確認する。
    #[test]
    fn format_metadata_failure_report_includes_prior_problems_and_metadata_error() {
        let problems = vec!["directories.core: declared directory does not exist".to_string()];
        let lines = format_metadata_failure_report(&problems, "cargo not found");

        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("directories.core: declared directory does not exist"));
        assert!(lines[1].contains("failed to cross-check with cargo metadata: cargo not found"));
    }

    #[test]
    fn format_metadata_failure_report_with_no_prior_problems_still_reports_metadata_error() {
        let lines = format_metadata_failure_report(&[], "cargo not found");
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("failed to cross-check with cargo metadata: cargo not found"));
    }

    /// fail-closed 回帰テスト: `routing.extractor` が `fandhe-frontend-router-v1` 以外の
    /// 未知の値の場合、`"routes":[]` を伴う黙示的成功へ倒さず `problems` に
    /// 積んで非 0 終了させること（レビュー指摘 #127: High severity。
    /// 未対応 extractor が誤って成功扱いされ、CI・フックが誤設定を見逃す
    /// リスクがあった）。
    #[test]
    fn collect_routes_reports_problem_for_unknown_extractor() {
        let manifest = structure::StructureManifest {
            version: 1,
            directories: Vec::new(),
            routing: Some(structure::RoutingConfig {
                definition_dir: "app".to_string(),
                extractor: "unknown-extractor-v9".to_string(),
            }),
        };
        let project_dir = std::env::temp_dir().join("fw-collect-routes-unknown-extractor-test");
        let _ = std::fs::create_dir_all(&project_dir);

        let mut problems: Vec<String> = Vec::new();
        let routes = collect_routes(&manifest, &project_dir, &mut problems);

        assert!(routes.is_empty());
        assert_eq!(
            problems.len(),
            1,
            "unknown extractor must be reported as a problem, not treated as zero routes"
        );
    }

    /// fail-closed 回帰テスト: `component_boundary::extract_public_symbols` が
    /// 失敗した場合も同様に `problems` へ積むこと（上記と同一契約）。
    #[test]
    fn collect_component_boundary_reports_problem_when_extraction_fails() {
        let manifest = structure::StructureManifest {
            version: 1,
            directories: vec![structure::DirectoryEntry {
                name: "does-not-exist".to_string(),
                role: structure::Role::Component,
                crate_name: None,
                description: "test".to_string(),
                depends_on: Vec::new(),
                allowed_dependents: Vec::new(),
                path: None,
            }],
            routing: None,
        };
        let project_dir = std::env::temp_dir().join("fw-collect-component-boundary-test-project");
        let _ = std::fs::create_dir_all(&project_dir);

        let mut problems: Vec<String> = Vec::new();
        let component_boundary = collect_component_boundary(&manifest, &project_dir, &mut problems);

        assert!(component_boundary.is_empty());
        assert_eq!(
            problems.len(),
            1,
            "extraction failure must be reported, not silently treated as zero symbols"
        );
    }

    // --- impact サブコマンド（TASK-13.2c, #135） ---

    #[test]
    fn impact_subcommand_without_symbol_is_a_usage_error() {
        assert_eq!(run(&["impact".to_string()]), 2);
    }

    #[test]
    fn impact_subcommand_rejects_invalid_symbol() {
        assert_eq!(
            run(&["impact".to_string(), "bad-symbol".to_string()]),
            2,
            "symbol containing `-` must be rejected before reaching the scan step"
        );
        assert_eq!(
            run(&["impact".to_string(), "std::render".to_string()]),
            2,
            "symbol containing `::` must be rejected before reaching the scan step"
        );
        assert_eq!(
            run(&["impact".to_string(), String::new()]),
            2,
            "empty symbol must be rejected"
        );
    }

    #[test]
    fn impact_subcommand_rejects_bad_project_usage() {
        assert_eq!(
            run(&[
                "impact".to_string(),
                "render".to_string(),
                "--unknown-flag".to_string(),
                "x".to_string()
            ]),
            2
        );
        assert_eq!(
            run(&[
                "impact".to_string(),
                "render".to_string(),
                "--project".to_string()
            ]),
            2,
            "--project with a missing value must be a usage error"
        );
    }

    /// `impact::analyze`（#134 の走査エンジン）・`metadata::fetch`（本 CLI 接続、
    /// #135）が実際に結線されていることを固定する回帰テスト。このリポジトリ
    /// 自身をワークスペースとして走査し、`render`（`core/src/lib.rs` で
    /// トップレベル `pub fn` として定義済み）が黙示的失敗（exit 非 0）に
    /// 倒れず解析成功（exit 0）することを確認する（JSON 出力の詳細検証は
    /// `impact::render_report` の単体テストが担う）。
    #[test]
    fn impact_subcommand_with_valid_symbol_analyzes_successfully() {
        // このテストバイナリは `crates/cli/` 配下でビルドされるため、2 段の
        // 親ディレクトリでワークスペースルートを得る（イシュー #436）。
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("crates/cli/ has a workspace root two levels up");
        let code = run(&[
            "impact".to_string(),
            "render".to_string(),
            "--project".to_string(),
            workspace_root.to_string_lossy().into_owned(),
        ]);
        assert_eq!(code, 0);
    }
}
