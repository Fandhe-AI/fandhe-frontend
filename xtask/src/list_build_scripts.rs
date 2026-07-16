//! TASK-3.2（#19）: 依存グラフから `build.rs` を持つクレートを機械的に列挙する。
//!
//! - TASK-3.2a（本ファイル）: `cargo metadata` の出力から `build.rs` 保有クレートを
//!   列挙する本体ロジック（[`list_build_scripts_many`] / [`format_report`]）
//! - TASK-3.2b: CI（`.github/workflows/deps-check.yml`）への出力統合。xtask CLI 側
//!   （`xtask/src/main.rs` の `run_list_build_scripts`）と本モジュールの
//!   [`format_report`] が定めるサマリ書式に依拠する。CLI 契約の回帰テストは
//!   `xtask/tests/cli_list_build_scripts.rs`。
//!
//! # 目的
//!
//! `security.md`「依存追加は脅威面の拡大。上限（60 件・深さ 6）とともに `build.rs`
//! の有無を確認する」を機械化し、PR ごとに恒常的に可視化する。列挙対象は
//! `check_deps`（REQ-3）と同じ依存グラフ定義（[`check_deps::DepGraph`] /
//! [`check_deps::reachable_ids`]、`DepKind::Normal` のみ・dev 依存を除外）を再利用し、
//! 「REQ-3 が数えている依存グラフの中に `build.rs` 保有クレートが何件あるか」を示す。
//!
//! # ゲートにしない
//!
//! `build.rs` の存在自体は違反ではない（禁止クレートのブロックは cargo-deny 系
//! タスク（TASK-4.x）のスコープ）。したがって [`format_report`] は PASS/FAIL を
//! 持たず、件数のみを報告する監査ログとして設計する。ただし計測失敗
//! （`cargo metadata` 失敗・ルート未検出等）は fail-closed とし、空リストで
//! 「build.rs なし」に見せかけない（呼び出し元の `xtask/src/main.rs` が非 0 終了で扱う）。
//!
//! # スコープ上の既知の限界（意図的な選択）
//!
//! `check_deps` と同じグラフ定義（[`DepKind::Normal`] のみ）を使うため、
//! `build-dependencies` 経由でのみ到達するクレートの `build.rs` は列挙対象に
//! **含まれない**。ビルド時に実行される `build.rs` という観点では
//! build-dependencies も脅威面ではあるが、REQ-3 の計測対象定義（PoC-3 の
//! `-e normal` 踏襲）との整合を優先し、本タスクでは対象を揃える判断とした
//! （現状 rws-core / xtask はいずれも外部依存ゼロのため実害はない）。
//! build-dependencies 経由の `build.rs` も監査対象に含めるかは、別途スコープ外
//! 事項として起票を検討する。

use crate::check_deps::{self, CheckDepsError, DepGraph, DepKind};
use crate::json::Json;
use std::collections::HashMap;

/// 列挙された 1 クレート（`build.rs` 保有）の名前・バージョン。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildScriptCrate {
    pub name: String,
    pub version: String,
}

/// 1 パッケージ（`--package` 指定 1 件分）の列挙結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildScriptReport {
    /// 列挙起点となったパッケージ名（`--package` に指定した値）。
    pub root: String,
    /// `root` から REQ-3 と同じ到達可能集合内で見つかった `build.rs` 保有クレート。
    /// クレート名・バージョンの辞書順に正規化済み（CI ログの diff 可能性のため）。
    pub crates: Vec<BuildScriptCrate>,
}

/// `cargo metadata` の `packages[]` から抽出した 1 パッケージ分の情報。
struct PackageInfo {
    name: String,
    version: String,
    /// `packages[].targets[]` に `kind: ["custom-build"]` のターゲット
    /// （`build-script-build`）が含まれるか。cargo はこのターゲットを、
    /// 当該クレートが `build.rs` を持つ場合にのみ生成する。
    has_build_script: bool,
}

/// `metadata` の `packages[]` を package_id -> [`PackageInfo`] の索引に変換する。
fn index_packages(metadata: &Json) -> Result<HashMap<String, PackageInfo>, CheckDepsError> {
    let packages = metadata
        .get("packages")
        .and_then(Json::as_array)
        .ok_or_else(|| CheckDepsError::UnexpectedShape("missing `packages` array".to_string()))?;

    let mut index = HashMap::new();
    for pkg in packages {
        let id = pkg
            .get("id")
            .and_then(Json::as_str)
            .ok_or_else(|| CheckDepsError::UnexpectedShape("package missing `id`".to_string()))?;
        let name = pkg
            .get("name")
            .and_then(Json::as_str)
            .ok_or_else(|| CheckDepsError::UnexpectedShape("package missing `name`".to_string()))?;
        let version = pkg.get("version").and_then(Json::as_str).ok_or_else(|| {
            CheckDepsError::UnexpectedShape("package missing `version`".to_string())
        })?;
        let has_build_script = pkg
            .get("targets")
            .and_then(Json::as_array)
            .map(|targets| targets.iter().any(is_custom_build_target))
            .unwrap_or(false);

        index.insert(
            id.to_string(),
            PackageInfo {
                name: name.to_string(),
                version: version.to_string(),
                has_build_script,
            },
        );
    }
    Ok(index)
}

/// `packages[].targets[]` の 1 要素が `custom-build`（`build.rs` 由来のターゲット）かを判定する。
fn is_custom_build_target(target: &Json) -> bool {
    target
        .get("kind")
        .and_then(Json::as_array)
        .map(|kinds| kinds.iter().any(|k| k.as_str() == Some("custom-build")))
        .unwrap_or(false)
}

/// `graph` / `index` を共有した状態で `root_name` 1 件分のレポートを組み立てる。
///
/// 到達可能集合は `check_deps::reachable_ids` に委譲し、REQ-3（依存件数計測）と
/// 同じグラフ定義を使う（`DepKind::Normal` のみ・dev 依存除外）。
fn build_report_for_root(
    graph: &DepGraph,
    index: &HashMap<String, PackageInfo>,
    root_name: &str,
) -> Result<BuildScriptReport, CheckDepsError> {
    let reachable = check_deps::reachable_ids(graph, root_name, &[DepKind::Normal])?;

    let mut crates: Vec<BuildScriptCrate> = reachable
        .iter()
        .filter_map(|id| index.get(id))
        .filter(|info| info.has_build_script)
        .map(|info| BuildScriptCrate {
            name: info.name.clone(),
            version: info.version.clone(),
        })
        .collect();
    // クレート名・バージョンの辞書順に正規化する（HashSet 由来で順序不定になるのを防ぎ、
    // CI ログの diff 可能性・再現性を保つ）。
    crates.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.version.cmp(&b.version)));

    Ok(BuildScriptReport {
        root: root_name.to_string(),
        crates,
    })
}

/// [`list_build_scripts_many`] の戻り値要素: (パッケージ名, 個別の列挙結果)。
///
/// `check_deps::NamedMeasurement` と同じ設計意図: `cargo metadata` 自体の失敗
/// （外側の `Result`）と、個々のパッケージの列挙失敗（要素内の `Result`）を区別する。
pub type NamedBuildScriptReport = (String, Result<BuildScriptReport, CheckDepsError>);

/// 複数パッケージをまとめて列挙する。xtask CLI（`main.rs`）の
/// `list-build-scripts --package` 複数指定時のエントリポイント。
///
/// `cargo metadata` の実行・JSON パース・[`DepGraph`] 構築・パッケージ索引作成を
/// 1 回に集約し、`root_names` の各要素について使い回す
/// （`check_deps::measure_many_from_cargo_metadata` と同じ方針。TASK-3.1c で
/// 指摘された「metadata rerun per package」非効率を最初から避ける）。
pub fn list_build_scripts_many(
    root_names: &[String],
) -> Result<Vec<NamedBuildScriptReport>, CheckDepsError> {
    let output = check_deps::run_cargo_metadata()?;
    let metadata = crate::json::parse(&output).map_err(CheckDepsError::InvalidJson)?;
    let graph = check_deps::build_graph(&metadata)?;
    let index = index_packages(&metadata)?;

    Ok(root_names
        .iter()
        .map(|name| {
            let result = build_report_for_root(&graph, &index, name);
            (name.clone(), result)
        })
        .collect())
}

/// 人間可読なレポートと、CI ログから機械抽出可能な明細行・1 行サマリを整形する。
///
/// この書式は `.github/workflows/deps-check.yml` と
/// `xtask/tests/cli_list_build_scripts.rs` が依拠する契約であり、安易に変更しない
/// （`check_deps::format_report` と同じ設計方針）。
///
/// - 明細行（検出クレートごとに 1 行）: `build-script: <crate-name>@<version>`
/// - 1 行サマリ（`--package` 指定ごとに 1 行、`grep '^build-scripts:'` で抽出可能）:
///   `build-scripts: package=<root> count=<n>`
///
/// PASS/FAIL 判定は持たない（`build.rs` の存在は違反ではないため。モジュール冒頭の
/// rustdoc「ゲートにしない」参照）。
pub fn format_report(report: &BuildScriptReport) -> String {
    let mut out = String::new();
    for c in &report.crates {
        out.push_str(&format!("build-script: {}@{}\n", c.name, c.version));
    }
    out.push_str(&format!(
        "build-scripts: package={} count={}\n",
        report.root,
        report.crates.len()
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テスト用に最小限の `cargo metadata` 形状の JSON 断片を組み立てるヘルパー。
    /// `packages` は `(id, name, version, has_build_script)` のリスト。
    /// `nodes` は `(id, [(依存先 id, kind)])` のリスト（kind は "normal"/"dev"/"build"）。
    fn fixture(packages: &[(&str, &str, &str, bool)], nodes: &[(&str, &[(&str, &str)])]) -> Json {
        let packages_json = Json::Array(
            packages
                .iter()
                .map(|(id, name, version, has_build_script)| {
                    let mut fields = vec![
                        ("id".to_string(), Json::String((*id).to_string())),
                        ("name".to_string(), Json::String((*name).to_string())),
                        ("version".to_string(), Json::String((*version).to_string())),
                    ];
                    let targets = if *has_build_script {
                        Json::Array(vec![Json::Object(vec![(
                            "kind".to_string(),
                            Json::Array(vec![Json::String("custom-build".to_string())]),
                        )])])
                    } else {
                        Json::Array(vec![Json::Object(vec![(
                            "kind".to_string(),
                            Json::Array(vec![Json::String("lib".to_string())]),
                        )])])
                    };
                    fields.push(("targets".to_string(), targets));
                    Json::Object(fields)
                })
                .collect(),
        );
        let nodes_json = Json::Array(
            nodes
                .iter()
                .map(|(id, deps)| {
                    let deps_json = Json::Array(
                        deps.iter()
                            .map(|(pkg, kind)| {
                                let kind_value = match *kind {
                                    "normal" => Json::Null,
                                    other => Json::String(other.to_string()),
                                };
                                Json::Object(vec![
                                    ("pkg".to_string(), Json::String((*pkg).to_string())),
                                    (
                                        "dep_kinds".to_string(),
                                        Json::Array(vec![Json::Object(vec![(
                                            "kind".to_string(),
                                            kind_value,
                                        )])]),
                                    ),
                                ])
                            })
                            .collect(),
                    );
                    Json::Object(vec![
                        ("id".to_string(), Json::String((*id).to_string())),
                        ("deps".to_string(), deps_json),
                    ])
                })
                .collect(),
        );
        Json::Object(vec![
            ("packages".to_string(), packages_json),
            (
                "resolve".to_string(),
                Json::Object(vec![("nodes".to_string(), nodes_json)]),
            ),
        ])
    }

    #[test]
    fn no_build_script_crates_yields_empty_report() {
        let json = fixture(
            &[
                ("root#0.1.0", "root", "0.1.0", false),
                ("a#0.1.0", "a", "0.1.0", false),
            ],
            &[("root#0.1.0", &[("a#0.1.0", "normal")]), ("a#0.1.0", &[])],
        );
        let graph = check_deps::build_graph(&json).unwrap();
        let index = index_packages(&json).unwrap();
        let report = build_report_for_root(&graph, &index, "root").unwrap();
        assert_eq!(report.root, "root");
        assert!(report.crates.is_empty());
    }

    #[test]
    fn detects_build_script_crate_reachable_via_normal_dep() {
        let json = fixture(
            &[
                ("root#0.1.0", "root", "0.1.0", false),
                ("libz-sys#1.1.0", "libz-sys", "1.1.0", true),
            ],
            &[
                ("root#0.1.0", &[("libz-sys#1.1.0", "normal")]),
                ("libz-sys#1.1.0", &[]),
            ],
        );
        let graph = check_deps::build_graph(&json).unwrap();
        let index = index_packages(&json).unwrap();
        let report = build_report_for_root(&graph, &index, "root").unwrap();
        assert_eq!(
            report.crates,
            vec![BuildScriptCrate {
                name: "libz-sys".to_string(),
                version: "1.1.0".to_string(),
            }]
        );
    }

    #[test]
    fn excludes_build_script_crate_only_reachable_via_dev_dependency() {
        // dev 依存経由でしか到達できない build.rs 保有クレートは、REQ-3 の計測対象
        // （check_deps）と同じ定義に揃え、列挙からも除外する。
        let json = fixture(
            &[
                ("root#0.1.0", "root", "0.1.0", false),
                ("devonly#0.1.0", "devonly", "0.1.0", true),
            ],
            &[
                ("root#0.1.0", &[("devonly#0.1.0", "dev")]),
                ("devonly#0.1.0", &[]),
            ],
        );
        let graph = check_deps::build_graph(&json).unwrap();
        let index = index_packages(&json).unwrap();
        let report = build_report_for_root(&graph, &index, "root").unwrap();
        assert!(report.crates.is_empty());
    }

    #[test]
    fn output_is_sorted_by_name_then_version() {
        let json = fixture(
            &[
                ("root#0.1.0", "root", "0.1.0", false),
                ("zeta#0.1.0", "zeta", "0.1.0", true),
                ("alpha#0.1.0", "alpha", "0.1.0", true),
            ],
            &[
                (
                    "root#0.1.0",
                    &[("zeta#0.1.0", "normal"), ("alpha#0.1.0", "normal")],
                ),
                ("zeta#0.1.0", &[]),
                ("alpha#0.1.0", &[]),
            ],
        );
        let graph = check_deps::build_graph(&json).unwrap();
        let index = index_packages(&json).unwrap();
        let report = build_report_for_root(&graph, &index, "root").unwrap();
        let names: Vec<&str> = report.crates.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "zeta"]);
    }

    #[test]
    fn root_not_found_returns_error() {
        let json = fixture(
            &[("root#0.1.0", "root", "0.1.0", false)],
            &[("root#0.1.0", &[])],
        );
        let graph = check_deps::build_graph(&json).unwrap();
        let index = index_packages(&json).unwrap();
        let err = build_report_for_root(&graph, &index, "missing").unwrap_err();
        assert!(matches!(err, CheckDepsError::RootNotFound(_)));
    }

    #[test]
    fn format_report_empty_contains_zero_count_summary() {
        let report = BuildScriptReport {
            root: "rws-core".to_string(),
            crates: Vec::new(),
        };
        let out = format_report(&report);
        assert_eq!(out, "build-scripts: package=rws-core count=0\n");
    }

    #[test]
    fn format_report_lists_detail_lines_before_summary() {
        let report = BuildScriptReport {
            root: "rws-server".to_string(),
            crates: vec![
                BuildScriptCrate {
                    name: "alpha".to_string(),
                    version: "0.1.0".to_string(),
                },
                BuildScriptCrate {
                    name: "zeta".to_string(),
                    version: "2.0.0".to_string(),
                },
            ],
        };
        let out = format_report(&report);
        assert_eq!(
            out,
            "build-script: alpha@0.1.0\n\
             build-script: zeta@2.0.0\n\
             build-scripts: package=rws-server count=2\n"
        );
    }

    #[test]
    fn integration_list_build_scripts_many_matches_check_deps_graph_definition() {
        // 実ワークスペースに対する結合テスト: rws-core / xtask はともに REQ-3 上
        // 外部依存ゼロが不変条件のため、build.rs 保有クレートも 0 件であるはず。
        // cargo が使えない実行環境（オフライン CI 等）では明示メッセージで fail させる。
        let names = vec!["rws-core".to_string(), "xtask".to_string()];
        match list_build_scripts_many(&names) {
            Ok(results) => {
                assert_eq!(results.len(), 2);
                for (name, result) in results {
                    match result {
                        Ok(report) => assert!(
                            report.crates.is_empty(),
                            "{name} must keep zero build.rs-bearing dependencies (REQ-3 zero-dependency invariant)"
                        ),
                        Err(e) => panic!("failed to enumerate build scripts for {name}: {e}"),
                    }
                }
            }
            Err(e) => panic!("failed to run cargo metadata for integration test: {e}"),
        }
    }
}
