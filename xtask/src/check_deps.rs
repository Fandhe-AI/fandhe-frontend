//! REQ-3（依存グラフ上限: 標準サーバー構成で解決済み依存パッケージ 60 件以内・
//! 依存グラフ最大深さ 6 以内、`docs/spec/04-requirements.md`）の計測ロジック。
//!
//! TASK-3.1a（本ファイル）の責務は **計測のみ**。しきい値判定・レポート整形・
//! 非ゼロ終了コードは TASK-3.1b（#17）が [`measure`] / [`DepsMeasurement`] を
//! 呼び出して積み増す。CI 組み込みは TASK-3.1c（#18）、build 依存の列挙は
//! TASK-3.2（#19）が本モジュールの `DepKind` フィルタを再利用する想定。
//!
//! # 計測の定義（PoC-3 との対応）
//!
//! PoC-3 は `cargo tree -p rws-server -e normal --prefix none` の一意クレート数と
//! インデント段数で実測した（`docs/spec/03-poc/rendering-web-standards/README.md`）。
//! 本実装は `cargo metadata --format-version 1` の `resolve.nodes` を正とし、
//! 次の定義を採用する:
//!
//! - **件数**: ルートパッケージから [`DepKind::Normal`] 辺のみを辿って到達可能な
//!   一意パッケージ数（ルート自身を除く）。dev 依存は除外（PoC-3 の `-e normal` と整合）。
//! - **深さ**: ルートを深さ 0 とした最長経路長。dev 依存を除いた解決グラフは DAG
//!   であるためメモ化 DFS で厳密に算出する。`cargo tree` の `(*)` 重複省略による
//!   過小評価が起きないため、同一構成でも `cargo tree` の目視値以上になり得る。
//!
//! # 契約
//!
//! `measure()` は panic せず [`CheckDepsError`] を返す（不正な metadata 出力・
//! 循環検出・ルート未検出の場合）。呼び出し側（#17 の判定ロジック）は
//! この契約に依存してよい。

use crate::json::{parse, Json, JsonError};
use std::collections::{HashMap, HashSet};
use std::process::Command;

/// `cargo metadata` の `resolve.nodes[].deps[].dep_kinds[].kind` に対応する依存種別。
///
/// `cargo metadata` の出力では通常依存は `kind: null` として表現されるため、
/// 判定は文字列一致ではなく「`kind` フィールドの有無」で行う（[`parse_dep_kinds`] 参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DepKind {
    /// 通常依存（`kind: null`）。REQ-3 の計測対象。
    Normal,
    /// `dev-dependencies`。PoC-3 の `-e normal` に倣い計測から除外する。
    Dev,
    /// `build-dependencies`。TASK-3.2（#19）の build.rs 列挙で利用される想定。
    Build,
}

/// 依存パッケージ件数・最大深さの計測結果。
///
/// TASK-3.1b（#17）がこの構造体を受け取ってしきい値（60 件 / 深さ 6）と比較する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepsMeasurement {
    /// 計測起点となったパッケージ名（`Cargo.toml` の `package.name`）。
    pub root: String,
    /// ルートを除く、`kinds_filter` に一致する辺のみを辿って到達可能な一意パッケージ数。
    pub package_count: usize,
    /// ルートを深さ 0 とした最長到達経路長。
    pub max_depth: usize,
}

/// 計測処理全体で発生しうるエラー。
///
/// 呼び出し元（xtask CLI）はメッセージのみを stderr に出す。`cargo metadata` の
/// 標準出力全体やプロセス環境変数などの機微情報は含めない（security.md 準拠）。
#[derive(Debug)]
pub enum CheckDepsError {
    /// `cargo metadata` プロセスの起動・終了に失敗した。
    CommandFailed(String),
    /// `cargo metadata` の出力が JSON として解釈できなかった。
    InvalidJson(JsonError),
    /// 出力 JSON の構造が期待と異なる（`packages` / `resolve` の欠落等）。
    UnexpectedShape(String),
    /// 指定したルートパッケージが `packages` 内に見つからなかった。
    RootNotFound(String),
    /// 解決グラフに循環が検出された（`cargo metadata` は通常 DAG を返すため防御的チェック）。
    CycleDetected,
}

impl std::fmt::Display for CheckDepsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckDepsError::CommandFailed(msg) => write!(f, "failed to run cargo metadata: {msg}"),
            CheckDepsError::InvalidJson(err) => {
                write!(f, "failed to parse cargo metadata output: {err}")
            }
            CheckDepsError::UnexpectedShape(msg) => {
                write!(f, "unexpected cargo metadata shape: {msg}")
            }
            CheckDepsError::RootNotFound(name) => {
                write!(f, "package `{name}` not found in cargo metadata output")
            }
            CheckDepsError::CycleDetected => write!(f, "cycle detected in dependency graph"),
        }
    }
}

impl std::error::Error for CheckDepsError {}

/// 解決済み依存グラフ。`package_id -> (package_name, [(dep_id, DepKind)])` の隣接リスト。
///
/// `cargo metadata` の `resolve.nodes` を正規化した中間表現。ノード ID は
/// cargo の PackageId 文字列をそのまま使う（衝突しないことが cargo により保証される）。
pub struct DepGraph {
    /// package_id -> パッケージ名（表示用）。
    names: HashMap<String, String>,
    /// package_id -> 依存先 (package_id, DepKind) のリスト。
    edges: HashMap<String, Vec<(String, DepKind)>>,
}

/// `cargo metadata --format-version 1` を実行し、標準出力を返す。
///
/// `$CARGO`（cargo サブコマンドとして起動された場合に cargo が設定する環境変数）を
/// 優先し、無ければ `cargo` を PATH から解決する。シェル経由の文字列連結は行わず、
/// 固定引数のみを渡す（A03 インジェクション対策、security.md 準拠）。
fn run_cargo_metadata() -> Result<String, CheckDepsError> {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let output = Command::new(cargo_bin)
        .args(["metadata", "--format-version", "1"])
        .output()
        .map_err(|e| CheckDepsError::CommandFailed(e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CheckDepsError::CommandFailed(format!(
            "cargo metadata exited with {status}: {stderr}",
            status = output.status,
        )));
    }
    String::from_utf8(output.stdout).map_err(|e| {
        CheckDepsError::CommandFailed(format!("cargo metadata output is not valid UTF-8: {e}"))
    })
}

/// `cargo metadata` の JSON 出力から [`DepGraph`] を構築する。
///
/// `packages[].id` / `packages[].name` と `resolve.nodes[].id` / `deps[].pkg` /
/// `deps[].dep_kinds[].kind` のみを参照する。想定外の構造は
/// [`CheckDepsError::UnexpectedShape`] として返し panic しない。
pub fn build_graph(metadata: &Json) -> Result<DepGraph, CheckDepsError> {
    let packages = metadata
        .get("packages")
        .and_then(Json::as_array)
        .ok_or_else(|| CheckDepsError::UnexpectedShape("missing `packages` array".to_string()))?;

    let mut names = HashMap::new();
    for pkg in packages {
        let id = pkg
            .get("id")
            .and_then(Json::as_str)
            .ok_or_else(|| CheckDepsError::UnexpectedShape("package missing `id`".to_string()))?;
        let name = pkg
            .get("name")
            .and_then(Json::as_str)
            .ok_or_else(|| CheckDepsError::UnexpectedShape("package missing `name`".to_string()))?;
        names.insert(id.to_string(), name.to_string());
    }

    let resolve = metadata
        .get("resolve")
        .ok_or_else(|| CheckDepsError::UnexpectedShape("missing `resolve`".to_string()))?;
    let nodes = resolve
        .get("nodes")
        .and_then(Json::as_array)
        .ok_or_else(|| CheckDepsError::UnexpectedShape("missing `resolve.nodes`".to_string()))?;

    let mut edges = HashMap::new();
    for node in nodes {
        let id = node.get("id").and_then(Json::as_str).ok_or_else(|| {
            CheckDepsError::UnexpectedShape("resolve node missing `id`".to_string())
        })?;
        let deps = node.get("deps").and_then(Json::as_array).ok_or_else(|| {
            CheckDepsError::UnexpectedShape("resolve node missing `deps`".to_string())
        })?;

        let mut out = Vec::new();
        for dep in deps {
            let pkg_id = dep
                .get("pkg")
                .and_then(Json::as_str)
                .ok_or_else(|| CheckDepsError::UnexpectedShape("dep missing `pkg`".to_string()))?;
            let dep_kinds = dep
                .get("dep_kinds")
                .and_then(Json::as_array)
                .ok_or_else(|| {
                    CheckDepsError::UnexpectedShape("dep missing `dep_kinds`".to_string())
                })?;
            for kind in parse_dep_kinds(dep_kinds)? {
                out.push((pkg_id.to_string(), kind));
            }
        }
        edges.insert(id.to_string(), out);
    }

    Ok(DepGraph { names, edges })
}

/// `dep_kinds` 配列（各要素は `{"kind": null | "dev" | "build", ...}`）を [`DepKind`] に変換する。
///
/// 同一依存が複数 kind を持つ場合（例: normal かつ dev）はすべて返す。
fn parse_dep_kinds(dep_kinds: &[Json]) -> Result<Vec<DepKind>, CheckDepsError> {
    let mut kinds = Vec::new();
    for entry in dep_kinds {
        let kind_field = entry.get("kind");
        let kind = match kind_field {
            None | Some(Json::Null) => DepKind::Normal,
            Some(Json::String(s)) if s == "dev" => DepKind::Dev,
            Some(Json::String(s)) if s == "build" => DepKind::Build,
            Some(Json::String(other)) => {
                return Err(CheckDepsError::UnexpectedShape(format!(
                    "unknown dep_kinds.kind value `{other}`"
                )))
            }
            Some(_) => {
                return Err(CheckDepsError::UnexpectedShape(
                    "dep_kinds.kind is neither null nor string".to_string(),
                ))
            }
        };
        kinds.push(kind);
    }
    // dep_kinds が空配列の場合（cargo の実装上は稀）でも normal 扱いにフォールバックする。
    // 「依存が存在するのに種別不明で計測から漏れる」事故を避けるための安全側デフォルト。
    if kinds.is_empty() {
        kinds.push(DepKind::Normal);
    }
    Ok(kinds)
}

/// `root_name`（`Cargo.toml` の `package.name`）に一致する package_id を探す。
fn find_root_id(graph: &DepGraph, root_name: &str) -> Result<String, CheckDepsError> {
    graph
        .names
        .iter()
        .find(|(_, name)| name.as_str() == root_name)
        .map(|(id, _)| id.clone())
        .ok_or_else(|| CheckDepsError::RootNotFound(root_name.to_string()))
}

/// `root_name` を起点に、`kinds_filter` に含まれる辺のみを辿って
/// 依存パッケージ件数・最大深さを計測する。
///
/// #17（TASK-3.1b）・#19（TASK-3.2）から呼ばれる契約点。この関数はしきい値判定を
/// 一切行わず、生の計測値のみを返す。
pub fn measure(
    graph: &DepGraph,
    root_name: &str,
    kinds_filter: &[DepKind],
) -> Result<DepsMeasurement, CheckDepsError> {
    let root_id = find_root_id(graph, root_name)?;
    let allowed: HashSet<DepKind> = kinds_filter.iter().copied().collect();

    // 件数: BFS で到達集合を求める（ルート自身は含めない）。
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: std::collections::VecDeque<String> = std::collections::VecDeque::new();
    queue.push_back(root_id.clone());
    let mut seen_from_root: HashSet<String> = HashSet::new();
    seen_from_root.insert(root_id.clone());
    while let Some(current) = queue.pop_front() {
        let Some(neighbors) = graph.edges.get(&current) else {
            continue;
        };
        for (dep_id, kind) in neighbors {
            if !allowed.contains(kind) {
                continue;
            }
            if seen_from_root.insert(dep_id.clone()) {
                visited.insert(dep_id.clone());
                queue.push_back(dep_id.clone());
            }
        }
    }
    let package_count = visited.len();

    // 深さ: メモ化 DFS で最長経路長を求める。防御的にサイクル検出（灰色ノード再訪問）を行う。
    let max_depth = longest_path(graph, &root_id, &allowed)?;

    Ok(DepsMeasurement {
        root: root_name.to_string(),
        package_count,
        max_depth,
    })
}

/// ノードの探索状態。サイクル検出（[`CheckDepsError::CycleDetected`]）に使う。
#[derive(Clone, Copy, PartialEq, Eq)]
enum VisitState {
    InProgress,
    Done(usize),
}

/// `root` からの最長到達経路長（ルート自身は深さ 0）をメモ化 DFS で求める。
///
/// `cargo metadata` の解決グラフは（dev 依存を除けば）DAG である前提だが、
/// 万一の循環参照時に無限再帰させないよう `InProgress` 状態で往復検出する。
fn longest_path(
    graph: &DepGraph,
    root: &str,
    allowed: &HashSet<DepKind>,
) -> Result<usize, CheckDepsError> {
    let mut memo: HashMap<String, VisitState> = HashMap::new();
    dfs_depth(graph, root, allowed, &mut memo)
}

fn dfs_depth(
    graph: &DepGraph,
    node: &str,
    allowed: &HashSet<DepKind>,
    memo: &mut HashMap<String, VisitState>,
) -> Result<usize, CheckDepsError> {
    match memo.get(node) {
        Some(VisitState::Done(depth)) => return Ok(*depth),
        Some(VisitState::InProgress) => return Err(CheckDepsError::CycleDetected),
        None => {}
    }
    memo.insert(node.to_string(), VisitState::InProgress);

    let mut best = 0usize;
    if let Some(neighbors) = graph.edges.get(node) {
        for (dep_id, kind) in neighbors {
            if !allowed.contains(kind) {
                continue;
            }
            let child_depth = dfs_depth(graph, dep_id, allowed, memo)?;
            best = best.max(child_depth + 1);
        }
    }

    memo.insert(node.to_string(), VisitState::Done(best));
    Ok(best)
}

/// `cargo metadata` を実行し、`root_name` について既定フィルタ（[`DepKind::Normal`] のみ、
/// PoC-3 の `-e normal` に整合）で計測する。xtask CLI（`main.rs`）のエントリポイント。
pub fn measure_from_cargo_metadata(root_name: &str) -> Result<DepsMeasurement, CheckDepsError> {
    let output = run_cargo_metadata()?;
    let metadata = parse(&output).map_err(CheckDepsError::InvalidJson)?;
    let graph = build_graph(&metadata)?;
    measure(&graph, root_name, &[DepKind::Normal])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テスト用に最小限の `cargo metadata` 形状の JSON 断片を組み立てるヘルパー。
    /// `deps` は `(依存先 package_id, kind)` のリスト。kind は "normal"/"dev"/"build"。
    fn fixture(packages: &[(&str, &str)], nodes: &[(&str, &[(&str, &str)])]) -> Json {
        let packages_json = Json::Array(
            packages
                .iter()
                .map(|(id, name)| {
                    Json::Object(vec![
                        ("id".to_string(), Json::String((*id).to_string())),
                        ("name".to_string(), Json::String((*name).to_string())),
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
    fn zero_dependencies() {
        let json = fixture(&[("root#0.1.0", "root")], &[("root#0.1.0", &[])]);
        let graph = build_graph(&json).unwrap();
        let m = measure(&graph, "root", &[DepKind::Normal]).unwrap();
        assert_eq!(m.package_count, 0);
        assert_eq!(m.max_depth, 0);
    }

    #[test]
    fn serial_chain_depth_and_count() {
        // root -> a -> b -> c (直列 3 段)
        let json = fixture(
            &[
                ("root#0.1.0", "root"),
                ("a#0.1.0", "a"),
                ("b#0.1.0", "b"),
                ("c#0.1.0", "c"),
            ],
            &[
                ("root#0.1.0", &[("a#0.1.0", "normal")]),
                ("a#0.1.0", &[("b#0.1.0", "normal")]),
                ("b#0.1.0", &[("c#0.1.0", "normal")]),
                ("c#0.1.0", &[]),
            ],
        );
        let graph = build_graph(&json).unwrap();
        let m = measure(&graph, "root", &[DepKind::Normal]).unwrap();
        assert_eq!(m.package_count, 3);
        assert_eq!(m.max_depth, 3);
    }

    #[test]
    fn diamond_counts_uniquely() {
        // root -> a -> c, root -> b -> c (ダイヤモンド。c は 1 件としてのみ数える)
        let json = fixture(
            &[
                ("root#0.1.0", "root"),
                ("a#0.1.0", "a"),
                ("b#0.1.0", "b"),
                ("c#0.1.0", "c"),
            ],
            &[
                (
                    "root#0.1.0",
                    &[("a#0.1.0", "normal"), ("b#0.1.0", "normal")],
                ),
                ("a#0.1.0", &[("c#0.1.0", "normal")]),
                ("b#0.1.0", &[("c#0.1.0", "normal")]),
                ("c#0.1.0", &[]),
            ],
        );
        let graph = build_graph(&json).unwrap();
        let m = measure(&graph, "root", &[DepKind::Normal]).unwrap();
        assert_eq!(m.package_count, 3); // a, b, c
        assert_eq!(m.max_depth, 2); // root -> a/b -> c
    }

    #[test]
    fn dev_dependencies_excluded() {
        let json = fixture(
            &[("root#0.1.0", "root"), ("devdep#0.1.0", "devdep")],
            &[
                ("root#0.1.0", &[("devdep#0.1.0", "dev")]),
                ("devdep#0.1.0", &[]),
            ],
        );
        let graph = build_graph(&json).unwrap();
        let m = measure(&graph, "root", &[DepKind::Normal]).unwrap();
        assert_eq!(m.package_count, 0);
        assert_eq!(m.max_depth, 0);
    }

    #[test]
    fn root_not_found_returns_error() {
        let json = fixture(&[("root#0.1.0", "root")], &[("root#0.1.0", &[])]);
        let graph = build_graph(&json).unwrap();
        let err = measure(&graph, "missing", &[DepKind::Normal]).unwrap_err();
        assert!(matches!(err, CheckDepsError::RootNotFound(_)));
    }

    #[test]
    fn integration_rws_core_has_zero_dependencies() {
        // rws-core は REQ-3 上「外部依存ゼロ」が不変条件。実ワークスペースに対して
        // cargo metadata を実行し、それが実際に 0 件 / 深さ 0 であることを確認する。
        // cargo が使えない実行環境（オフライン CI 等）では明示メッセージで fail させる。
        match measure_from_cargo_metadata("rws-core") {
            Ok(m) => {
                assert_eq!(
                    m.package_count, 0,
                    "rws-core must keep zero external dependencies (REQ-3)"
                );
                assert_eq!(m.max_depth, 0);
            }
            Err(e) => panic!("failed to run cargo metadata for integration test: {e}"),
        }
    }
}
