//! REQ-3（依存グラフ上限: 標準サーバー構成で解決済み依存パッケージ 60 件以内・
//! 依存グラフ最大深さ 6 以内、`docs/spec/04-requirements.md`）の計測・判定・レポート層。
//!
//! TASK-3.1 は 3 段階に分割されている:
//! - TASK-3.1a（本ファイル前半）: `cargo metadata` の解析による実測値
//!   （パッケージ数・依存グラフ最大深さ）の計測ロジック（[`DepGraph`] / [`measure`] /
//!   [`measure_from_cargo_metadata`] / [`measure_many_from_cargo_metadata`]）
//! - TASK-3.1b（本ファイル後半）: 実測値に対する上限判定（60 件/深さ 6）と
//!   レポート出力・終了コード制御（[`DepsMetrics`] / [`judge`] / [`format_report`]）
//! - TASK-3.1c: CI ワークフローへの組み込み（xtask/src/main.rs 参照）
//! - イシュー #154: `core` / `interactive` の**外部依存ゼロ**（REQ-3 受け入れ基準 1）を
//!   専用ゲートとして強制する（[`ZERO_DEP_CRATES`] / [`judge_zero`] / [`format_zero_report`]）。
//!   既存の 60/6 判定（`judge`）は標準サーバー構成向けの上限チェックであり、
//!   コアクレートの「ゼロであること」を機械的に保証しないため独立させている。
//!
//! # 計測の定義（PoC-3 との対応）
//!
//! PoC-3 は `cargo tree -p rws-server -e normal --prefix none` の一意クレート数と
//! インデント段数で実測した（`docs/spec/03-poc/rendering-web-standards/README.md`）。
//! 本実装は `cargo metadata --format-version 1 --filter-platform <host-triple>` の
//! `resolve.nodes` を正とし、次の定義を採用する:
//!
//! - **件数**: ルートパッケージから [`DepKind::Normal`] 辺のみを辿って到達可能な
//!   一意パッケージ数（ルート自身を除く）。dev 依存は除外（PoC-3 の `-e normal` と整合）。
//! - **深さ**: ルートを深さ 0 とした最長経路長。dev 依存を除いた解決グラフは DAG
//!   であるためメモ化 DFS で厳密に算出する。`cargo tree` の `(*)` 重複省略による
//!   過小評価が起きないため、同一構成でも `cargo tree` の目視値以上になり得る。
//! - **プラットフォーム**: `--filter-platform` にホストの target triple を渡し、
//!   cfg 条件付き依存（`target.'cfg(windows)'.dependencies` 等）のうちホストで
//!   有効にならない edge を cargo 自身に除外させる。これを付けない場合、
//!   他プラットフォーム向けの target-specific な normal edge がグラフに残留し、
//!   件数・深さが PoC-3 のホスト限定計測より過大に出て REQ-3 判定を誤らせる
//!   （Bugbot 指摘: cross-platform deps inflate counts）。
//!
//! # 契約
//!
//! `measure()` / `measure_from_cargo_metadata()` は panic せず [`CheckDepsError`] を
//! 返す（不正な metadata 出力・循環検出・ルート未検出の場合）。`judge()` は
//! [`DepsMetrics`] を受け取り [`CheckResult`] を返す純粋関数で、I/O を一切行わない。

use crate::json::{parse, Json, JsonError};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::process::Command;

/// フレームワーク標準構成で許容する解決済み依存パッケージ数の上限。
///
/// PoC-3 実測（純 Rust 方式・`rws-server` 相当構成: 52 件）を基準に、
/// 実装拡張分の余裕を含めて設定する（REQ-3 / docs/spec/04-requirements.md 59 行目）。
/// 上限緩和のための CLI 引数・環境変数は意図的に設けない
/// （coding-rust.md「依存グラフ上限を弱めない」/ security.md 参照）。
pub const MAX_PACKAGES: usize = 60;

/// フレームワーク標準構成で許容する依存グラフ最大深さの上限。
///
/// PoC-3 実測（純 Rust 方式・`rws-server` 相当構成: 深さ 5）を基準に、
/// 実装拡張分の余裕を含めて設定する（REQ-3 / docs/spec/04-requirements.md 59 行目）。
pub const MAX_DEPTH: usize = 6;

/// REQ-3 受け入れ基準 1（外部依存ゼロ）を強制するコアクレート群（イシュー #154）。
///
/// `coding-rust.md`「core は外部依存ゼロ」「core/interactive では unsafe を一切使用しない」の
/// 対象クレートに対応する。CLI 引数・環境変数での差し替えは意図的に設けない
/// （上限を弱める経路を作らない、coding-rust.md / security.md 参照）。
///
/// `rws-interactive` は本実装時点では未作成（workspace 未参加）。
/// [`fetch_zero_dep_targets`] が `cargo metadata` の `workspace_members` との
/// 積集合を取るため、追加後は CI 変更なしで自動的に検証対象へ入る。
pub const ZERO_DEP_CRATES: &[&str] = &["rws-core", "rws-interactive"];

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

/// `measure()` が返す生の計測値（1 パッケージ分）。
///
/// [`DepsMetrics`] への変換（`From<DepsMeasurement>`）を経て `judge()` に渡される。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepsMeasurement {
    /// 計測起点となったパッケージ名（`Cargo.toml` の `package.name`）。
    pub root: String,
    /// ルートを除く、`kinds_filter` に一致する辺のみを辿って到達可能な一意パッケージ数。
    pub package_count: usize,
    /// ルートを深さ 0 とした最長到達経路長。
    pub max_depth: usize,
}

/// 判定対象として `judge()` に渡す実測値。
///
/// `target` はレポート表示用の名称（例: "standard-server"）。
/// [`DepsMeasurement`] と同じ形の値を保持するが、判定層（TASK-3.1b）の
/// 語彙（「計測対象 = target」）に合わせて別名で定義する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepsMetrics {
    /// 判定対象の構成名（レポート表示用。例: "standard-server"）。
    pub target: String,
    /// `cargo metadata` で解決された依存パッケージ数（対象自身を除く）。
    pub package_count: usize,
    /// 依存グラフの最大深さ（対象パッケージを深さ 0 とする）。
    pub max_depth: usize,
}

impl From<DepsMeasurement> for DepsMetrics {
    fn from(m: DepsMeasurement) -> Self {
        DepsMetrics {
            target: m.root,
            package_count: m.package_count,
            max_depth: m.max_depth,
        }
    }
}

/// 上限超過の内訳。`judge` が `CheckResult::Fail` に含める違反リスト。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Violation {
    /// 依存パッケージ数が `MAX_PACKAGES` を超過。
    PackageCount { actual: usize, limit: usize },
    /// 依存グラフ最大深さが `MAX_DEPTH` を超過。
    MaxDepth { actual: usize, limit: usize },
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Violation::PackageCount { actual, limit } => {
                write!(f, "package count {actual} exceeds limit {limit}")
            }
            Violation::MaxDepth { actual, limit } => {
                write!(f, "dependency graph depth {actual} exceeds limit {limit}")
            }
        }
    }
}

/// 上限判定の結果。CI（TASK-3.1c）の終了コード制御はこの値の可否を fail-closed で反映する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckResult {
    /// 実測値がすべて上限内。
    Pass(DepsMetrics),
    /// いずれかの上限を超過。内側の `Vec<Violation>` は空にならない。
    Fail(DepsMetrics, Vec<Violation>),
}

impl CheckResult {
    /// CI（3.1c）が終了コードを決定する際に参照する契約:
    /// `Pass` のみ成功（終了コード 0）、それ以外は失敗として扱う。
    pub fn is_pass(&self) -> bool {
        matches!(self, CheckResult::Pass(_))
    }
}

/// 実測値 `metrics` を上限（`MAX_PACKAGES` / `MAX_DEPTH`）に照らして判定する純粋関数。
///
/// I/O を一切行わないため単体テストで境界値を網羅できる。
/// rws-server（将来の標準サーバー構成）を計測対象として想定するが、
/// 本関数自体は対象の種類を問わず `DepsMetrics` の値のみで判定する。
pub fn judge(metrics: DepsMetrics) -> CheckResult {
    let mut violations = Vec::new();

    if metrics.package_count > MAX_PACKAGES {
        violations.push(Violation::PackageCount {
            actual: metrics.package_count,
            limit: MAX_PACKAGES,
        });
    }
    if metrics.max_depth > MAX_DEPTH {
        violations.push(Violation::MaxDepth {
            actual: metrics.max_depth,
            limit: MAX_DEPTH,
        });
    }

    if violations.is_empty() {
        CheckResult::Pass(metrics)
    } else {
        CheckResult::Fail(metrics, violations)
    }
}

/// 人間可読なサマリと、CI ログから機械抽出可能な 1 行サマリを整形する。
///
/// 1 行サマリの書式（`deps-check: packages=<n>/<limit> depth=<n>/<limit> result=<PASS|FAIL>`）は
/// TASK-3.1c の CI がログから判定結果を抽出する際の契約とみなし、安易に変更しない。
/// ユーザー向け文字列は英語（japanese-style.md: フレームワーク成果物は国際利用を想定）。
pub fn format_report(result: &CheckResult) -> String {
    let (metrics, violations, verdict): (&DepsMetrics, &[Violation], &str) = match result {
        CheckResult::Pass(metrics) => (metrics, &[], "PASS"),
        CheckResult::Fail(metrics, violations) => (metrics, violations.as_slice(), "FAIL"),
    };

    let mut out = String::new();
    out.push_str(&format!(
        "Dependency graph check for target \"{}\"\n",
        metrics.target
    ));
    out.push_str(&format!(
        "  packages: {} (limit {})\n",
        metrics.package_count, MAX_PACKAGES
    ));
    out.push_str(&format!(
        "  max depth: {} (limit {})\n",
        metrics.max_depth, MAX_DEPTH
    ));
    if violations.is_empty() {
        out.push_str("  result: PASS\n");
    } else {
        out.push_str("  result: FAIL\n");
        for violation in violations {
            out.push_str(&format!("  violation: {violation}\n"));
        }
    }

    out.push_str(&format!(
        "deps-check: packages={}/{} depth={}/{} result={}\n",
        metrics.package_count, MAX_PACKAGES, metrics.max_depth, MAX_DEPTH, verdict
    ));

    out
}

/// 計測処理全体で発生しうるエラー。
///
/// 呼び出し元（xtask CLI）はメッセージのみを stderr に出す。`cargo metadata` の
/// 標準出力全体やプロセス環境変数などの機微情報は含めない（security.md 準拠）。
#[derive(Debug)]
pub enum CheckDepsError {
    /// `cargo metadata` / `rustc -vV` プロセスの起動・終了に失敗した。
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

impl DepGraph {
    /// `root_id` から `allowed` に含まれる辺のみを辿って到達可能な package_id 集合を
    /// BFS で求める（`root_id` 自身を含む）。
    ///
    /// TASK-3.2a（`list_build_scripts`）の到達可能集合計算から利用される。
    /// `measure()` の件数計測（ルート自身を除いた件数を要求）とは戻り値の解釈が
    /// 異なるが、BFS 本体は共通の [`bfs_reachable_inclusive`] を使い回す
    /// （ルート自身を含めるか除くかという 1 点のみが異なるため、BFS 実装自体を
    /// 二重管理しない。reviewer 指摘: `reachable_from` と `measure` 内 BFS の重複）。
    pub(crate) fn reachable_from(&self, root_id: &str, allowed: &[DepKind]) -> HashSet<String> {
        let allowed: HashSet<DepKind> = allowed.iter().copied().collect();
        bfs_reachable_inclusive(&self.edges, root_id, &allowed)
    }
}

/// `root_id` から `allowed` に含まれる辺のみを辿って到達可能な package_id 集合を
/// BFS で求める共通実装（`root_id` 自身を含む）。
///
/// [`DepGraph::reachable_from`]（TASK-3.2a の列挙: ルート自身を含めたい）と
/// [`measure`]（TASK-3.1a の件数計測: ルート自身を除きたい）の双方が内部で使う。
/// 「含める/除く」の違いは呼び出し側で `visited.len() - 1` するか等で吸収し、
/// BFS ロジック自体は 1 箇所にまとめる。
fn bfs_reachable_inclusive(
    edges: &HashMap<String, Vec<(String, DepKind)>>,
    root_id: &str,
    allowed: &HashSet<DepKind>,
) -> HashSet<String> {
    let mut visited: HashSet<String> = HashSet::new();
    visited.insert(root_id.to_string());
    let mut queue: std::collections::VecDeque<String> = std::collections::VecDeque::new();
    queue.push_back(root_id.to_string());
    while let Some(current) = queue.pop_front() {
        let Some(neighbors) = edges.get(&current) else {
            continue;
        };
        for (dep_id, kind) in neighbors {
            if allowed.contains(kind) && visited.insert(dep_id.clone()) {
                queue.push_back(dep_id.clone());
            }
        }
    }
    visited
}

/// `rustc -vV` の `host:` 行からホストの target triple を取得する。
///
/// `cargo metadata --filter-platform <triple>` に渡すために必要。追加の依存クレートを
/// 導入せず（core は外部依存ゼロ・xtask も依存グラフ計測対象のため増やさない方針、
/// coding-rust.md 参照）、標準ツールチェインの `rustc` を直接呼び出して取得する。
fn host_triple() -> Result<String, CheckDepsError> {
    let rustc_bin = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let output = Command::new(rustc_bin)
        .arg("-vV")
        .output()
        .map_err(|e| CheckDepsError::CommandFailed(format!("failed to run rustc -vV: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CheckDepsError::CommandFailed(format!(
            "rustc -vV exited with {status}: {stderr}",
            status = output.status,
        )));
    }
    let stdout = String::from_utf8(output.stdout).map_err(|e| {
        CheckDepsError::CommandFailed(format!("rustc -vV output is not valid UTF-8: {e}"))
    })?;
    stdout
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            CheckDepsError::CommandFailed(
                "rustc -vV output did not contain a `host:` line".to_string(),
            )
        })
}

/// `cargo metadata --format-version 1 --filter-platform <host-triple>` を実行し、
/// 標準出力を返す。
///
/// `$CARGO`（cargo サブコマンドとして起動された場合に cargo が設定する環境変数）を
/// 優先し、無ければ `cargo` を PATH から解決する。シェル経由の文字列連結は行わず、
/// 固定引数のみを渡す（A03 インジェクション対策、security.md 準拠）。
///
/// `--filter-platform` にホストの target triple を明示することで、他プラットフォーム
/// 向けの target-specific な normal edge（`target.'cfg(...)'dependencies` 等）を
/// cargo 自身に解決させてグラフから除外する。これを付けない場合、ホスト以外の
/// プラットフォーム向け依存が残留し件数・深さが PoC-3 のホスト限定計測より
/// 過大に出て REQ-3 判定を誤らせる（Bugbot 指摘: cross-platform deps inflate counts）。
///
/// `--locked` を付与し、`Cargo.toml` と `Cargo.lock` に drift がある場合は
/// レジストリ解決をやり直さずエラーで停止する。これを付けないと `cargo metadata`
/// が `Cargo.lock` を暗黙に書き換え、コミット済みの依存集合ではなく再解決後の
/// グラフを計測してしまい REQ-3 判定の対象がずれる
/// （Bugbot 指摘: metadata run omits locked mode）。
fn run_cargo_metadata() -> Result<String, CheckDepsError> {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let target = host_triple()?;
    let output = Command::new(cargo_bin)
        .args([
            "metadata",
            "--format-version",
            "1",
            "--filter-platform",
            &target,
            "--locked",
        ])
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
///
/// `--filter-platform` 済みの `resolve.nodes` を入力とする前提のため、
/// `dep_kinds[].target` によるプラットフォームフィルタは行わない
/// （cargo 側で既にホスト非該当の edge が取り除かれている）。
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
///
/// `list_build_scripts`（TASK-3.2a）からも同一ルート解決ロジックとして再利用するため
/// `pub(crate)` とする（重複実装を避ける）。
pub(crate) fn find_root_id(graph: &DepGraph, root_name: &str) -> Result<String, CheckDepsError> {
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
/// この関数はしきい値判定を一切行わず、生の計測値のみを返す
/// （[`judge`] が [`DepsMetrics`]（`From<DepsMeasurement>`）を受けて判定する）。
pub fn measure(
    graph: &DepGraph,
    root_name: &str,
    kinds_filter: &[DepKind],
) -> Result<DepsMeasurement, CheckDepsError> {
    let root_id = find_root_id(graph, root_name)?;
    let allowed: HashSet<DepKind> = kinds_filter.iter().copied().collect();

    // 件数: 共通 BFS（bfs_reachable_inclusive、ルート自身を含む）を使い回し、
    // ルート自身の 1 件を除いた数を件数計測値とする（PoC-3 の「ルート除く」定義と整合）。
    let reachable = bfs_reachable_inclusive(&graph.edges, &root_id, &allowed);
    let package_count = reachable.len() - 1;

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

/// `cargo metadata` を実行し、標準出力を JSON としてパースするところまでを行う。
///
/// `check_deps`（依存グラフ計測）と `list_build_scripts`（TASK-3.2a: build.rs
/// 保有クレート列挙）の双方が同じ `cargo metadata` 出力を必要とするための共有層。
/// [`build_graph`] は依存グラフ用に、`list_build_scripts::collect_build_script_flags`
/// は `packages[].targets[]` 用に、同一の [`Json`] 値をそれぞれ別の視点で参照する
/// （`cargo metadata` の実行・パースを 1 回に集約し、Bugbot 指摘「metadata rerun
/// per package」と同種の重複実行を避ける）。
pub(crate) fn fetch_metadata_json() -> Result<Json, CheckDepsError> {
    let output = run_cargo_metadata()?;
    parse(&output).map_err(CheckDepsError::InvalidJson)
}

/// `cargo metadata` を 1 度だけ実行して [`DepGraph`] を構築する。
///
/// 複数パッケージを計測する場合は本関数を 1 回だけ呼び、得たグラフを
/// [`measure`] に使い回す（[`measure_many_from_cargo_metadata`] 参照）。
/// パッケージごとに `cargo metadata` を再実行して JSON を再パースするのは
/// 無駄な重複処理であり、CI での複数クレート計測時に顕著な非効率を生む
/// （Bugbot 指摘: metadata rerun per package）。
pub fn fetch_dep_graph() -> Result<DepGraph, CheckDepsError> {
    let metadata = fetch_metadata_json()?;
    build_graph(&metadata)
}

/// `cargo metadata` の `workspace_members`（package_id の配列）を、対応する
/// `packages[].name` の一覧へ解決する（イシュー #154）。
///
/// [`fetch_zero_dep_targets`] が [`ZERO_DEP_CRATES`] と実 workspace メンバーの
/// 積集合を取る際に、xtask 呼び出し側が「実際にワークスペースへ参加しているクレート名」
/// を知るために使う。`packages` に対応する id が見つからない場合は
/// [`CheckDepsError::UnexpectedShape`] を返す（`cargo metadata` の出力として矛盾して
/// いるため panic ではなくエラーとする）。
pub fn workspace_member_names(metadata: &Json) -> Result<Vec<String>, CheckDepsError> {
    let packages = metadata
        .get("packages")
        .and_then(Json::as_array)
        .ok_or_else(|| CheckDepsError::UnexpectedShape("missing `packages` array".to_string()))?;

    let mut id_to_name: HashMap<&str, &str> = HashMap::new();
    for pkg in packages {
        let id = pkg
            .get("id")
            .and_then(Json::as_str)
            .ok_or_else(|| CheckDepsError::UnexpectedShape("package missing `id`".to_string()))?;
        let name = pkg
            .get("name")
            .and_then(Json::as_str)
            .ok_or_else(|| CheckDepsError::UnexpectedShape("package missing `name`".to_string()))?;
        id_to_name.insert(id, name);
    }

    let workspace_members = metadata
        .get("workspace_members")
        .and_then(Json::as_array)
        .ok_or_else(|| {
            CheckDepsError::UnexpectedShape("missing `workspace_members` array".to_string())
        })?;

    let mut names = Vec::with_capacity(workspace_members.len());
    for member in workspace_members {
        let id = member.as_str().ok_or_else(|| {
            CheckDepsError::UnexpectedShape("workspace_members entry is not a string".to_string())
        })?;
        let name = id_to_name.get(id).ok_or_else(|| {
            CheckDepsError::UnexpectedShape(format!(
                "workspace member `{id}` not found in `packages`"
            ))
        })?;
        names.push((*name).to_string());
    }
    Ok(names)
}

/// `cargo metadata` を 1 度だけ実行し、[`DepGraph`] と workspace メンバー名の両方を返す
/// （イシュー #154）。`fetch_metadata_json` が集約する JSON パース結果を、
/// グラフ構築（[`build_graph`]）と workspace メンバー名解決（[`workspace_member_names`]）の
/// 双方で使い回し、`cargo metadata` の再実行（Bugbot 指摘「metadata rerun per package」と
/// 同種の重複）を避ける。
pub fn fetch_workspace_graph() -> Result<(DepGraph, Vec<String>), CheckDepsError> {
    let metadata = fetch_metadata_json()?;
    let graph = build_graph(&metadata)?;
    let members = workspace_member_names(&metadata)?;
    Ok((graph, members))
}

/// [`ZERO_DEP_CRATES`] と実際の workspace メンバーとの積集合を求め、[`DepGraph`] と
/// 併せて返す（イシュー #154。xtask CLI の `check-core-deps` サブコマンドのエントリ
/// ポイントから利用）。
///
/// 積集合が空（`ZERO_DEP_CRATES` の定数値がクレート改名等で陳腐化し、1 件も
/// workspace に実在しない）場合は [`CheckDepsError::UnexpectedShape`] を返す
/// fail-closed 設計。CLI 引数でクレートを指定させない設計と対で、判定対象が
/// 静かに 0 件になり PASS 扱いされる事故を防ぐ。
pub fn fetch_zero_dep_targets() -> Result<(DepGraph, Vec<String>), CheckDepsError> {
    let (graph, members) = fetch_workspace_graph()?;
    let member_set: HashSet<&str> = members.iter().map(String::as_str).collect();
    let targets: Vec<String> = ZERO_DEP_CRATES
        .iter()
        .filter(|name| member_set.contains(*name))
        .map(|name| (*name).to_string())
        .collect();

    if targets.is_empty() {
        return Err(CheckDepsError::UnexpectedShape(format!(
            "none of ZERO_DEP_CRATES ({}) found among workspace members ({})",
            ZERO_DEP_CRATES.join(", "),
            members.join(", ")
        )));
    }

    Ok((graph, targets))
}

/// 実測値 `metrics` を「外部依存ゼロ」（イシュー #154 / REQ-3 受け入れ基準 1）に
/// 照らして判定する純粋関数。I/O を一切行わない（[`judge`] と対称の設計）。
///
/// 呼び出し側（[`fetch_zero_dep_targets`] の結果を使う xtask CLI）は
/// `measure(graph, name, &[Normal, Dev, Build])` の結果を渡す想定であり、
/// `Normal` / `Dev` / `Build` のいずれの辺も 1 件でも到達可能なら Fail とする
/// （[`judge`] の Normal 限定判定とは意図的に異なる。coding-rust.md
/// 「core/Cargo.toml に外部クレートを追加しない」は dev-dependencies も含むため）。
/// 深さは件数が 0 なら必然的に 0 になるため、件数のみで判定する。
pub fn judge_zero(metrics: DepsMetrics) -> CheckResult {
    if metrics.package_count > 0 {
        let violation = Violation::PackageCount {
            actual: metrics.package_count,
            limit: 0,
        };
        CheckResult::Fail(metrics, vec![violation])
    } else {
        CheckResult::Pass(metrics)
    }
}

/// [`judge_zero`] の結果を人間可読なレポートと、CI ログから機械抽出可能な
/// 1 行サマリに整形する（イシュー #154）。
///
/// 1 行サマリの書式（`core-deps-check: package=<name> external=<n> result=<PASS|FAIL>`）は
/// 既存の `deps-check:` 行（[`format_report`]）と衝突しない別プレフィックスとし、
/// CI（`.github/workflows/deps-check.yml`）が両方の判定結果を独立に grep できるようにする。
/// ユーザー向け文字列は英語（japanese-style.md: フレームワーク成果物は国際利用を想定）。
pub fn format_zero_report(result: &CheckResult) -> String {
    let (metrics, verdict): (&DepsMetrics, &str) = match result {
        CheckResult::Pass(metrics) => (metrics, "PASS"),
        CheckResult::Fail(metrics, _) => (metrics, "FAIL"),
    };

    let mut out = String::new();
    out.push_str(&format!(
        "Zero external dependency check for core crate \"{}\"\n",
        metrics.target
    ));
    out.push_str(&format!(
        "  external dependencies: {} (must be 0)\n",
        metrics.package_count
    ));
    out.push_str(&format!("  result: {verdict}\n"));

    out.push_str(&format!(
        "core-deps-check: package={} external={} result={}\n",
        metrics.target, metrics.package_count, verdict
    ));

    out
}

/// `cargo metadata` を実行し、`root_name` について既定フィルタ（[`DepKind::Normal`] のみ、
/// PoC-3 の `-e normal` に整合）で計測する。単一パッケージのみを計測する場合のエントリ
/// ポイント（xtask CLI・単体/結合テストから利用）。
///
/// 複数パッケージを計測する場合は [`measure_many_from_cargo_metadata`] を使い、
/// `cargo metadata` の再実行を避けること。
///
/// xtask バイナリ本体（`main.rs`）は複数パッケージ対応の
/// `measure_many_from_cargo_metadata` のみを呼ぶため、本関数は単体/結合テスト
/// および外部からの再利用のために公開 API として残す（`allow(dead_code)`）。
#[allow(dead_code)]
pub fn measure_from_cargo_metadata(root_name: &str) -> Result<DepsMeasurement, CheckDepsError> {
    let graph = fetch_dep_graph()?;
    measure(&graph, root_name, &[DepKind::Normal])
}

/// [`measure_many_from_cargo_metadata`] の戻り値要素: (パッケージ名, 個別の計測結果)。
///
/// 複数パッケージ計測では `cargo metadata` 自体の失敗（外側の `Result`）と、
/// 個々のパッケージの計測失敗（要素内の `Result`）を区別する。
pub type NamedMeasurement = (String, Result<DepsMeasurement, CheckDepsError>);

/// 複数パッケージをまとめて計測する。xtask CLI（`main.rs`）の `check-deps --package`
/// 複数指定時のエントリポイント。
///
/// `cargo metadata` の実行・JSON パース・[`DepGraph`] 構築を 1 回に集約し、
/// `root_names` の各要素について同じグラフ上で [`measure`] を呼ぶ
/// （Bugbot 指摘「metadata rerun per package」への対応）。
///
/// 各パッケージの計測は独立して成否を返す（1 つが `RootNotFound` 等で失敗しても
/// 他のパッケージの計測結果は捨てない）。呼び出し順は `root_names` の順序を保つ。
pub fn measure_many_from_cargo_metadata(
    root_names: &[String],
) -> Result<Vec<NamedMeasurement>, CheckDepsError> {
    let graph = fetch_dep_graph()?;
    Ok(root_names
        .iter()
        .map(|name| {
            let result = measure(&graph, name, &[DepKind::Normal]);
            (name.clone(), result)
        })
        .collect())
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

    #[test]
    fn integration_measure_many_matches_single_measure_for_rws_core() {
        // Bugbot 指摘「metadata rerun per package」の回帰テスト:
        // 複数パッケージ計測（cargo metadata 1 回）が単発計測と同じ結果を返すこと。
        let names = vec!["rws-core".to_string()];
        match measure_many_from_cargo_metadata(&names) {
            Ok(results) => {
                assert_eq!(results.len(), 1);
                let (name, result) = &results[0];
                assert_eq!(name, "rws-core");
                match result {
                    Ok(m) => {
                        assert_eq!(m.package_count, 0);
                        assert_eq!(m.max_depth, 0);
                    }
                    Err(e) => panic!("failed to measure rws-core: {e}"),
                }
            }
            Err(e) => panic!("failed to run cargo metadata for integration test: {e}"),
        }
    }

    fn metrics(package_count: usize, max_depth: usize) -> DepsMetrics {
        DepsMetrics {
            target: "standard-server".to_string(),
            package_count,
            max_depth,
        }
    }

    #[test]
    fn judge_passes_at_exact_limits() {
        let result = judge(metrics(MAX_PACKAGES, MAX_DEPTH));
        assert!(result.is_pass());
    }

    #[test]
    fn judge_fails_when_package_count_exceeds_limit() {
        let result = judge(metrics(MAX_PACKAGES + 1, MAX_DEPTH));
        match result {
            CheckResult::Fail(_, violations) => {
                assert_eq!(violations.len(), 1);
                assert!(matches!(violations[0], Violation::PackageCount { .. }));
            }
            CheckResult::Pass(_) => panic!("expected Fail"),
        }
    }

    #[test]
    fn judge_fails_when_depth_exceeds_limit() {
        let result = judge(metrics(MAX_PACKAGES, MAX_DEPTH + 1));
        match result {
            CheckResult::Fail(_, violations) => {
                assert_eq!(violations.len(), 1);
                assert!(matches!(violations[0], Violation::MaxDepth { .. }));
            }
            CheckResult::Pass(_) => panic!("expected Fail"),
        }
    }

    #[test]
    fn judge_reports_both_violations_when_both_exceed_limit() {
        let result = judge(metrics(MAX_PACKAGES + 10, MAX_DEPTH + 2));
        match result {
            CheckResult::Fail(_, violations) => {
                assert_eq!(violations.len(), 2);
            }
            CheckResult::Pass(_) => panic!("expected Fail"),
        }
    }

    #[test]
    fn judge_passes_with_headroom() {
        let result = judge(metrics(0, 0));
        assert!(result.is_pass());
    }

    #[test]
    fn format_report_pass_contains_machine_readable_summary_line() {
        let result = judge(metrics(52, 5));
        let report = format_report(&result);
        assert!(report.contains("deps-check: packages=52/60 depth=5/6 result=PASS"));
    }

    #[test]
    fn format_report_fail_contains_violation_lines_and_summary() {
        let result = judge(metrics(61, 7));
        let report = format_report(&result);
        assert!(report.contains("deps-check: packages=61/60 depth=7/6 result=FAIL"));
        assert!(report.contains("violation: package count 61 exceeds limit 60"));
        assert!(report.contains("violation: dependency graph depth 7 exceeds limit 6"));
    }

    #[test]
    fn deps_metrics_from_measurement_preserves_fields() {
        let measurement = DepsMeasurement {
            root: "rws-server".to_string(),
            package_count: 52,
            max_depth: 5,
        };
        let converted: DepsMetrics = measurement.into();
        assert_eq!(converted.target, "rws-server");
        assert_eq!(converted.package_count, 52);
        assert_eq!(converted.max_depth, 5);
    }

    // --- イシュー #154: 外部依存ゼロ強制ゲート ---

    fn zero_metrics(target: &str, package_count: usize) -> DepsMetrics {
        DepsMetrics {
            target: target.to_string(),
            package_count,
            max_depth: if package_count == 0 { 0 } else { 1 },
        }
    }

    #[test]
    fn judge_zero_passes_when_no_external_dependency() {
        let result = judge_zero(zero_metrics("rws-core", 0));
        assert!(result.is_pass());
    }

    #[test]
    fn judge_zero_fails_at_boundary_of_one_dependency() {
        // 境界値: 60/6 判定と異なり「1 件でも」Fail になる契約。
        let result = judge_zero(zero_metrics("rws-core", 1));
        match result {
            CheckResult::Fail(_, violations) => {
                assert_eq!(violations.len(), 1);
                assert!(matches!(
                    violations[0],
                    Violation::PackageCount {
                        actual: 1,
                        limit: 0
                    }
                ));
            }
            CheckResult::Pass(_) => panic!("expected Fail"),
        }
    }

    #[test]
    fn format_zero_report_pass_contains_machine_readable_summary_line() {
        let result = judge_zero(zero_metrics("rws-core", 0));
        let report = format_zero_report(&result);
        assert!(report.contains("core-deps-check: package=rws-core external=0 result=PASS"));
    }

    #[test]
    fn format_zero_report_fail_contains_machine_readable_summary_line() {
        let result = judge_zero(zero_metrics("rws-core", 3));
        let report = format_zero_report(&result);
        assert!(report.contains("core-deps-check: package=rws-core external=3 result=FAIL"));
    }

    /// fixture ベース回帰: dev / build 依存のみを持つルートでも、
    /// `measure(.., &[Normal, Dev, Build])` の結果を渡せば `judge_zero` が Fail にする
    /// こと（`judge_zero` 自体は Normal 限定ではなく、呼び出し側が渡した package_count を
    /// そのまま判定する契約であることの確認）。
    #[test]
    fn judge_zero_fails_for_dev_or_build_only_dependency() {
        let json = fixture(
            &[("root#0.1.0", "root"), ("devdep#0.1.0", "devdep")],
            &[
                ("root#0.1.0", &[("devdep#0.1.0", "dev")]),
                ("devdep#0.1.0", &[]),
            ],
        );
        let graph = build_graph(&json).unwrap();
        // judge（60/6 判定）は Normal 限定のため dev 依存を無視するが、
        // 外部依存ゼロ判定は Normal/Dev/Build すべてを対象にする（コメント参照）。
        let m = measure(
            &graph,
            "root",
            &[DepKind::Normal, DepKind::Dev, DepKind::Build],
        )
        .unwrap();
        assert_eq!(m.package_count, 1);
        let result = judge_zero(m.into());
        assert!(!result.is_pass());
    }

    /// `workspace_members` の package_id を `packages[].name` へ解決できることの確認。
    fn fixture_with_workspace_members(
        packages: &[(&str, &str)],
        workspace_member_ids: &[&str],
    ) -> Json {
        let mut base = fixture(packages, &[]);
        if let Json::Object(entries) = &mut base {
            entries.push((
                "workspace_members".to_string(),
                Json::Array(
                    workspace_member_ids
                        .iter()
                        .map(|id| Json::String((*id).to_string()))
                        .collect(),
                ),
            ));
        }
        base
    }

    #[test]
    fn workspace_member_names_resolves_ids_to_names() {
        let json = fixture_with_workspace_members(
            &[("rws-core#0.1.0", "rws-core"), ("xtask#0.1.0", "xtask")],
            &["rws-core#0.1.0", "xtask#0.1.0"],
        );
        let names = workspace_member_names(&json).unwrap();
        assert_eq!(names, vec!["rws-core".to_string(), "xtask".to_string()]);
    }

    #[test]
    fn workspace_member_names_missing_field_returns_unexpected_shape() {
        let json = fixture(&[("root#0.1.0", "root")], &[]);
        let err = workspace_member_names(&json).unwrap_err();
        assert!(matches!(err, CheckDepsError::UnexpectedShape(_)));
    }

    #[test]
    fn workspace_member_names_unknown_id_returns_unexpected_shape() {
        let json = fixture_with_workspace_members(
            &[("rws-core#0.1.0", "rws-core")],
            &["not-a-real-id#0.1.0"],
        );
        let err = workspace_member_names(&json).unwrap_err();
        assert!(matches!(err, CheckDepsError::UnexpectedShape(_)));
    }

    #[test]
    fn integration_fetch_zero_dep_targets_finds_rws_core_in_real_workspace() {
        // ZERO_DEP_CRATES ∩ 実 workspace members が空でないこと
        // （定数の陳腐化を検知する fail-closed 契約の裏付け）。
        match fetch_zero_dep_targets() {
            Ok((_, targets)) => {
                assert!(
                    targets.contains(&"rws-core".to_string()),
                    "rws-core must be present among zero-dep targets: {targets:?}"
                );
            }
            Err(e) => panic!("failed to fetch zero-dep targets from real workspace: {e}"),
        }
    }

    #[test]
    fn integration_rws_core_passes_judge_zero_in_real_workspace() {
        // rws-core は REQ-3 上「外部依存ゼロ」が不変条件。実ワークスペースに対して
        // measure(.., &[Normal, Dev, Build]) を行い、judge_zero が PASS になることを
        // 確認する（integration_rws_core_has_zero_dependencies の Normal 限定計測とは
        // 別に、dev/build も含めた厳格な判定の回帰を担保する）。
        let (graph, targets) = fetch_zero_dep_targets().expect("real workspace metadata");
        assert!(targets.contains(&"rws-core".to_string()));
        let m = measure(
            &graph,
            "rws-core",
            &[DepKind::Normal, DepKind::Dev, DepKind::Build],
        )
        .expect("measure rws-core");
        let result = judge_zero(m.into());
        assert!(
            result.is_pass(),
            "rws-core must keep zero external dependencies including dev/build (issue #154)"
        );
    }
}
