//! REQ-3（依存グラフ上限とともにサプライチェーン監査可能性を担う、
//! `docs/spec/04-requirements.md` / PoC-2 脅威モデル）のうち、`build.rs` を
//! 保有するクレートの機械的列挙を担うモジュール（TASK-3.2a、親タスク TASK-3.2
//! / イシュー #19）。
//!
//! `build.rs`（カスタムビルドスクリプト）はビルド時に任意コードを実行できるため、
//! 標準構成の依存グラフに含まれる build.rs 保有クレートを可視化することが
//! サプライチェーン監査の前提になる。本モジュールは列挙ロジック本体と CLI 向けの
//! 整形のみを提供し、CI ワークフローへの組み込み・1 行サマリ書式の最終契約確定は
//! TASK-3.2b（イシュー #21）に委ねる（[`format_inventory`] のドキュメント参照）。
//!
//! # 検出方式
//!
//! `cargo metadata --format-version 1` の `packages[].targets[]` に
//! `kind: ["custom-build"]` を持つターゲットが含まれるかで判定する
//! （cargo が `build.rs` をこの形で表現する仕様。ファイルシステム走査より確実で、
//! `build = "custom_name.rs"` 指定にも追従する）。
//!
//! # 列挙対象（辿る辺）
//!
//! `--package` で指定したルートから到達可能な解決済みグラフ上のパッケージ集合を
//! 対象とする。辿る辺は [`DepKind::Normal`] + [`DepKind::Build`]（build-dependencies
//! もビルド時にコンパイル・実行されるため監査対象。`check_deps::DepKind::Build` の
//! rustdoc に「TASK-3.2 で利用される想定」と明記済み）。dev 依存はリリース物の
//! ビルドで実行されないため除外する（`check_deps` の `-e normal` 方針と整合）。
//! **ルート自身も列挙対象に含める**（ルートパッケージが build.rs を持つ場合も
//! 監査の抜けを作らない fail-closed 方針。[`DepGraph::reachable_from`] 参照）。
//!
//! # 契約
//!
//! 列挙関数は panic せず [`CheckDepsError`] を返す（想定外の metadata 構造・
//! ルート未検出）。列挙結果が 0 件であること自体はエラーではない
//! （「build.rs を持たない」は正常な計測結果）。

use crate::check_deps::{self, CheckDepsError, DepGraph, DepKind};
use crate::json::Json;
use std::collections::HashMap;

/// 列挙結果 1 件: build.rs を保有するクレートの名前とバージョン。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildScriptCrate {
    /// `Cargo.toml` の `package.name`。
    pub name: String,
    /// `Cargo.toml` の `package.version`（同名クレートの多重解決を区別するため保持）。
    pub version: String,
}

/// `metadata` の `packages[].targets[]` を走査し、`build.rs`（`custom-build`
/// ターゲット）を保有するパッケージのみを `package_id -> BuildScriptCrate` として返す。
///
/// 保有しないパッケージは戻り値に含めない（呼び出し側は到達可能集合との積を取る）。
/// `targets` や `kind` の形状が想定と異なる場合は黙って除外せず
/// [`CheckDepsError::UnexpectedShape`] を返す（「列挙できなかったのに成功扱いになる」
/// 事故を避ける fail-closed 方針、security.md 参照）。
pub(crate) fn collect_build_script_flags(
    metadata: &Json,
) -> Result<HashMap<String, BuildScriptCrate>, CheckDepsError> {
    let packages = metadata
        .get("packages")
        .and_then(Json::as_array)
        .ok_or_else(|| CheckDepsError::UnexpectedShape("missing `packages` array".to_string()))?;

    let mut flags = HashMap::new();
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
        let targets = pkg.get("targets").and_then(Json::as_array).ok_or_else(|| {
            CheckDepsError::UnexpectedShape("package missing `targets` array".to_string())
        })?;

        let mut has_build_script = false;
        for target in targets {
            let kinds = target.get("kind").and_then(Json::as_array).ok_or_else(|| {
                CheckDepsError::UnexpectedShape("target missing `kind` array".to_string())
            })?;
            if kinds.iter().any(|k| k.as_str() == Some("custom-build")) {
                has_build_script = true;
                break;
            }
        }

        if has_build_script {
            flags.insert(
                id.to_string(),
                BuildScriptCrate {
                    name: name.to_string(),
                    version: version.to_string(),
                },
            );
        }
    }
    Ok(flags)
}

/// `root_name` から到達可能な範囲（[`DepKind::Normal`] + [`DepKind::Build`]、
/// ルート自身を含む）のうち、`build_flags` に含まれる（= build.rs を保有する）
/// クレートを名前・バージョン順にソートして返す。
///
/// `graph` と `build_flags` は同一の `cargo metadata` 実行結果（同一の [`Json`]）
/// から構築されたものである前提（[`list_many_from_cargo_metadata`] 参照）。
pub fn list_build_scripts(
    graph: &DepGraph,
    build_flags: &HashMap<String, BuildScriptCrate>,
    root_name: &str,
) -> Result<Vec<BuildScriptCrate>, CheckDepsError> {
    let root_id = check_deps::find_root_id(graph, root_name)?;
    let reachable = graph.reachable_from(&root_id, &[DepKind::Normal, DepKind::Build]);

    let mut result: Vec<BuildScriptCrate> = reachable
        .iter()
        .filter_map(|id| build_flags.get(id))
        .cloned()
        .collect();
    result.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.version.cmp(&b.version)));
    Ok(result)
}

/// 人間可読な一覧と、CI ログから抽出可能な 1 行サマリを整形する。
///
/// 1 行サマリの書式（`build-scripts: target=<name> count=<n>`）は暫定であり、
/// CI 統合時の最終契約確定は TASK-3.2b（イシュー #21）に委ねる。本モジュール単体では
/// この書式を破壊的に変更しても回帰テストの対象にしていない点に注意する
/// （`check_deps::format_report` の `deps-check:` 行のような CI 依存契約とは異なる）。
pub fn format_inventory(root_name: &str, crates: &[BuildScriptCrate]) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Build script inventory for target \"{root_name}\"\n"
    ));
    if crates.is_empty() {
        out.push_str("  no packages with build scripts\n");
    } else {
        for c in crates {
            out.push_str(&format!("  {} {}\n", c.name, c.version));
        }
    }
    out.push_str(&format!(
        "build-scripts: target={root_name} count={}\n",
        crates.len()
    ));
    out
}

/// [`list_many_from_cargo_metadata`] の戻り値要素: (パッケージ名, 個別の列挙結果)。
///
/// `check_deps::NamedMeasurement` と同型の設計（`cargo metadata` 自体の失敗と
/// 個々のパッケージの列挙失敗を区別する）。
pub type NamedInventory = (String, Result<Vec<BuildScriptCrate>, CheckDepsError>);

/// `cargo metadata` を 1 回だけ実行し、`root_names` の各要素について
/// build.rs 保有クレートを列挙する。xtask CLI（`main.rs` の
/// `list-build-scripts --package`）のエントリポイント。
///
/// `check_deps::measure_many_from_cargo_metadata` と同様、複数ルートに対して
/// `cargo metadata` を再実行しない（同一 [`Json`] から [`DepGraph`] と
/// build.rs フラグの双方を構築し使い回す）。各ルートの列挙は独立して成否を返す。
pub fn list_many_from_cargo_metadata(
    root_names: &[String],
) -> Result<Vec<NamedInventory>, CheckDepsError> {
    let metadata = check_deps::fetch_metadata_json()?;
    let graph = check_deps::build_graph(&metadata)?;
    let flags = collect_build_script_flags(&metadata)?;

    Ok(root_names
        .iter()
        .map(|name| {
            let result = list_build_scripts(&graph, &flags, name);
            (name.clone(), result)
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check_deps::build_graph;

    /// テスト用に最小限の `cargo metadata` 形状の JSON 断片を組み立てるヘルパー。
    ///
    /// `packages` は `(id, name, version, has_build_script)` のリスト、
    /// `nodes` は `(id, [(依存先 id, kind)])` のリスト。kind は "normal"/"dev"/"build"。
    fn fixture(packages: &[(&str, &str, &str, bool)], nodes: &[(&str, &[(&str, &str)])]) -> Json {
        let packages_json = Json::Array(
            packages
                .iter()
                .map(|(id, name, version, has_build_script)| {
                    let targets = if *has_build_script {
                        Json::Array(vec![
                            Json::Object(vec![(
                                "kind".to_string(),
                                Json::Array(vec![Json::String("lib".to_string())]),
                            )]),
                            Json::Object(vec![(
                                "kind".to_string(),
                                Json::Array(vec![Json::String("custom-build".to_string())]),
                            )]),
                        ])
                    } else {
                        Json::Array(vec![Json::Object(vec![(
                            "kind".to_string(),
                            Json::Array(vec![Json::String("lib".to_string())]),
                        )])])
                    };
                    Json::Object(vec![
                        ("id".to_string(), Json::String((*id).to_string())),
                        ("name".to_string(), Json::String((*name).to_string())),
                        ("version".to_string(), Json::String((*version).to_string())),
                        ("targets".to_string(), targets),
                    ])
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
    fn crate_with_custom_build_target_is_listed() {
        let json = fixture(
            &[
                ("root#0.1.0", "root", "0.1.0", false),
                ("has-build#1.0.0", "has-build", "1.0.0", true),
            ],
            &[
                ("root#0.1.0", &[("has-build#1.0.0", "normal")]),
                ("has-build#1.0.0", &[]),
            ],
        );
        let graph = build_graph(&json).unwrap();
        let flags = collect_build_script_flags(&json).unwrap();
        let result = list_build_scripts(&graph, &flags, "root").unwrap();
        assert_eq!(
            result,
            vec![BuildScriptCrate {
                name: "has-build".to_string(),
                version: "1.0.0".to_string(),
            }]
        );
    }

    #[test]
    fn crate_without_custom_build_target_is_not_listed() {
        let json = fixture(
            &[
                ("root#0.1.0", "root", "0.1.0", false),
                ("plain#1.0.0", "plain", "1.0.0", false),
            ],
            &[
                ("root#0.1.0", &[("plain#1.0.0", "normal")]),
                ("plain#1.0.0", &[]),
            ],
        );
        let graph = build_graph(&json).unwrap();
        let flags = collect_build_script_flags(&json).unwrap();
        let result = list_build_scripts(&graph, &flags, "root").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn dev_dependency_build_script_is_excluded() {
        // dev 依存はリリース物のビルドで実行されないため列挙対象外
        // （check_deps の `-e normal` 方針と整合、モジュール冒頭コメント参照）。
        let json = fixture(
            &[
                ("root#0.1.0", "root", "0.1.0", false),
                ("dev-build#1.0.0", "dev-build", "1.0.0", true),
            ],
            &[
                ("root#0.1.0", &[("dev-build#1.0.0", "dev")]),
                ("dev-build#1.0.0", &[]),
            ],
        );
        let graph = build_graph(&json).unwrap();
        let flags = collect_build_script_flags(&json).unwrap();
        let result = list_build_scripts(&graph, &flags, "root").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn build_dependency_build_script_is_included() {
        // build-dependencies もビルド時に実行されるため監査対象に含める。
        let json = fixture(
            &[
                ("root#0.1.0", "root", "0.1.0", false),
                ("build-dep#1.0.0", "build-dep", "1.0.0", true),
            ],
            &[
                ("root#0.1.0", &[("build-dep#1.0.0", "build")]),
                ("build-dep#1.0.0", &[]),
            ],
        );
        let graph = build_graph(&json).unwrap();
        let flags = collect_build_script_flags(&json).unwrap();
        let result = list_build_scripts(&graph, &flags, "root").unwrap();
        assert_eq!(
            result,
            vec![BuildScriptCrate {
                name: "build-dep".to_string(),
                version: "1.0.0".to_string(),
            }]
        );
    }

    #[test]
    fn root_itself_with_build_script_is_included() {
        // ルート自身が build.rs を持つ場合も監査の抜けを作らないため列挙対象に含める。
        let json = fixture(
            &[("root#0.1.0", "root", "0.1.0", true)],
            &[("root#0.1.0", &[])],
        );
        let graph = build_graph(&json).unwrap();
        let flags = collect_build_script_flags(&json).unwrap();
        let result = list_build_scripts(&graph, &flags, "root").unwrap();
        assert_eq!(
            result,
            vec![BuildScriptCrate {
                name: "root".to_string(),
                version: "0.1.0".to_string(),
            }]
        );
    }

    #[test]
    fn unreachable_package_is_not_listed() {
        // グラフ上に存在するが root から辿れないパッケージ（別ワークスペースメンバー等）
        // は列挙対象外。
        let json = fixture(
            &[
                ("root#0.1.0", "root", "0.1.0", false),
                ("unrelated#1.0.0", "unrelated", "1.0.0", true),
            ],
            &[("root#0.1.0", &[]), ("unrelated#1.0.0", &[])],
        );
        let graph = build_graph(&json).unwrap();
        let flags = collect_build_script_flags(&json).unwrap();
        let result = list_build_scripts(&graph, &flags, "root").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn result_is_sorted_by_name_then_version() {
        let json = fixture(
            &[
                ("root#0.1.0", "root", "0.1.0", false),
                ("zeta#1.0.0", "zeta", "1.0.0", true),
                ("alpha#2.0.0", "alpha", "2.0.0", true),
                ("alpha#1.0.0", "alpha", "1.0.0", true),
            ],
            &[
                (
                    "root#0.1.0",
                    &[
                        ("zeta#1.0.0", "normal"),
                        ("alpha#2.0.0", "normal"),
                        ("alpha#1.0.0", "normal"),
                    ],
                ),
                ("zeta#1.0.0", &[]),
                ("alpha#2.0.0", &[]),
                ("alpha#1.0.0", &[]),
            ],
        );
        let graph = build_graph(&json).unwrap();
        let flags = collect_build_script_flags(&json).unwrap();
        let result = list_build_scripts(&graph, &flags, "root").unwrap();
        let names_versions: Vec<(&str, &str)> = result
            .iter()
            .map(|c| (c.name.as_str(), c.version.as_str()))
            .collect();
        assert_eq!(
            names_versions,
            vec![("alpha", "1.0.0"), ("alpha", "2.0.0"), ("zeta", "1.0.0")]
        );
    }

    #[test]
    fn root_not_found_returns_error() {
        let json = fixture(
            &[("root#0.1.0", "root", "0.1.0", false)],
            &[("root#0.1.0", &[])],
        );
        let graph = build_graph(&json).unwrap();
        let flags = collect_build_script_flags(&json).unwrap();
        let err = list_build_scripts(&graph, &flags, "missing").unwrap_err();
        assert!(matches!(err, CheckDepsError::RootNotFound(_)));
    }

    #[test]
    fn missing_targets_field_is_unexpected_shape_error() {
        let json = Json::Object(vec![(
            "packages".to_string(),
            Json::Array(vec![Json::Object(vec![
                ("id".to_string(), Json::String("root#0.1.0".to_string())),
                ("name".to_string(), Json::String("root".to_string())),
                ("version".to_string(), Json::String("0.1.0".to_string())),
                // `targets` フィールドを意図的に欠落させる。
            ])]),
        )]);
        let err = collect_build_script_flags(&json).unwrap_err();
        assert!(matches!(err, CheckDepsError::UnexpectedShape(_)));
    }

    #[test]
    fn format_inventory_empty_shows_none_and_zero_count() {
        let out = format_inventory("rws-core", &[]);
        assert!(out.contains("no packages with build scripts"));
        assert!(out.contains("build-scripts: target=rws-core count=0"));
    }

    #[test]
    fn format_inventory_multiple_entries_lists_each_and_counts() {
        let crates = vec![
            BuildScriptCrate {
                name: "alpha".to_string(),
                version: "1.0.0".to_string(),
            },
            BuildScriptCrate {
                name: "beta".to_string(),
                version: "2.0.0".to_string(),
            },
        ];
        let out = format_inventory("rws-server", &crates);
        assert!(out.contains("alpha 1.0.0"));
        assert!(out.contains("beta 2.0.0"));
        assert!(out.contains("build-scripts: target=rws-server count=2"));
    }

    #[test]
    fn integration_rws_core_has_no_build_scripts() {
        // rws-core は依存ゼロかつ build.rs 非保有が不変条件（REQ-2/REQ-3）。
        let names = vec!["rws-core".to_string()];
        match list_many_from_cargo_metadata(&names) {
            Ok(results) => {
                assert_eq!(results.len(), 1);
                let (name, result) = &results[0];
                assert_eq!(name, "rws-core");
                match result {
                    Ok(crates) => assert!(
                        crates.is_empty(),
                        "rws-core は build.rs を保有しない想定: {crates:?}"
                    ),
                    Err(e) => panic!("failed to list build scripts for rws-core: {e}"),
                }
            }
            Err(e) => panic!("failed to run cargo metadata for integration test: {e}"),
        }
    }

    #[test]
    fn integration_xtask_has_no_build_scripts() {
        // xtask 自身も build.rs 非保有（外部依存ゼロ・独自 JSON パーサのみで構成）。
        let names = vec!["xtask".to_string()];
        match list_many_from_cargo_metadata(&names) {
            Ok(results) => {
                assert_eq!(results.len(), 1);
                let (name, result) = &results[0];
                assert_eq!(name, "xtask");
                match result {
                    Ok(crates) => assert!(
                        crates.is_empty(),
                        "xtask は build.rs を保有しない想定: {crates:?}"
                    ),
                    Err(e) => panic!("failed to list build scripts for xtask: {e}"),
                }
            }
            Err(e) => panic!("failed to run cargo metadata for integration test: {e}"),
        }
    }
}
