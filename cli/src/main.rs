//! `fw`: AI 自己保守フック（REQ-13）向けの開発者・エージェント用 CLI エントリポイント。
//!
//! TASK-13.1（親 #127）の製品化対象。`structure` サブコマンドは
//! [`structure::load`]（TOML パース + セマンティック検証、TASK-13.1b）→
//! [`structure::StructureManifest::validate`]（宣言整合性検証、TASK-13.1a）→
//! [`metadata`] を用いた実体突き合わせ（TASK-13.1c）→ [`json_out`] による
//! JSON 出力、の順で処理する。いずれかの段階で失敗した場合は非 0 終了とし、
//! 呼び出し元（CI・AI 自己保守フック）が「構造チェック PASS」と誤認しないよう
//! 黙示的成功を返さない（`docs/structure-manifest.md` §4/§5、security.md A05）。

#![forbid(unsafe_code)]

mod component_boundary;
mod json;
mod json_out;
mod metadata;
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
}

/// `--project <dir>` 引数を解決する（省略時はカレントディレクトリ）。
///
/// `Ok(None)` は「引数の使い方が誤っている」（値の欠落・未知フラグ）ことを表し、
/// 呼び出し元は終了コード 2（使用法エラー）として扱う。
fn parse_project_arg(args: &[String]) -> Result<PathBuf, ()> {
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
    for dir in &manifest.directories {
        let path = project_dir.join(&dir.name);
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
            eprintln!("fw structure: failed to cross-check with cargo metadata: {e}");
            1
        }
    }
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
    // extractor は現時点で `rws-router-v1` のみ対応（`structure.toml` の
    // セマンティック検証では自由文字列を許容しているが、未知の抽出器 ID は
    // ここで黙って無視せず空結果に倒す。将来の抽出器追加時に個別対応する）。
    if routing.extractor != "rws-router-v1" {
        return Vec::new();
    }
    match routes::extract_routes(project_dir, &routing.definition_dir) {
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
        .filter_map(
            |d| match component_boundary::extract_public_symbols(project_dir, &d.name) {
                Ok(symbols) => Some((d.name.clone(), symbols)),
                Err(e) => {
                    problems.push(format!(
                        "directories.{}: failed to extract component boundary: {e}",
                        d.name
                    ));
                    None
                }
            },
        )
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
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("cli/ has a parent workspace root");
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
                extractor: "rws-router-v1".to_string(),
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
}
