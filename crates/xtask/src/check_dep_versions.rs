//! イシュー #657: workspace 内メンバー間の `path + version` 併記依存について、
//! 依存先クレートの現行 `version`（`Cargo.toml` の `package.version`）へ
//! 依存元の `version = "..."` 要求が追随しているかを検知するゲート。
//!
//! headless-ui 0.1.0 → 0.2.0 バンプ時、依存元（pre-styled-ui / wasm-full /
//! xtask）の `version = "0.1.0"` 追随は sed による手動一括変更が必要だった
//! （`check_version_bump` の是正メッセージによる注意喚起のみで機械検知手段が
//! なかった、PR #647 out-of-scope 記載）。semver 非互換バンプ（0.1→0.2）では
//! ビルド失敗として顕在化するが、互換バンプ（0.2.0→0.2.1 等）では検知されず
//! 公開物の version 要求が古いまま残留し得る。本モジュールはこれを
//! `cargo metadata --no-deps` のみ（ネットワーク照会なし）で機械検知する。
//!
//! # 判定ルール
//!
//! workspace 全メンバーの依存エッジのうち、`path` を持ち依存先が workspace
//! メンバーであるものについて:
//!
//! - **ルール 1（version 整合）**: `req != "*"`（version 宣言あり）のエッジは
//!   `req == "^" + 依存先の現行 version` の完全一致のみ PASS。それ以外
//!   （古い version・`=` ピン・部分指定等）は FAIL（[`FailReason::VersionMismatch`]）。
//! - **ルール 2（version 欠落）**: `req == "*"`（version 未宣言）かつ依存元が
//!   publish 対象（`check_version_bump::published_crates_from_cargo_metadata`
//!   と同じ fail-closed 判定）かつ kind が normal/build のエッジは FAIL
//!   （[`FailReason::MissingVersion`]、`cargo publish` が version 必須のため）。
//!   dev は除外（publish 時に自動除去される cargo 仕様）。
//!
//! 呼び出し元は `xtask/src/main.rs` の `run_check_dep_versions`
//! （`.github/workflows/ci.yml` の `dep-version-check` ジョブから実行）。
//! `--fix`（[`plan_fixes`] / [`apply_fixes`]）はルール 1 の FAIL のみを
//! 自動修正するローカル用オプトイン手段で、一意に書き換え位置を特定できない
//! 場合は一切書き換えずエラーにする（fail-closed、部分書き込みをしない）。

use crate::json::{parse, Json};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::process::Command;

/// 本モジュールの操作（`cargo metadata` 実行・ファイル読み書き）で発生し得るエラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckDepVersionsError {
    /// 外部プロセス（`cargo`）起動・実行の失敗。
    CommandFailed(String),
    /// `cargo metadata` の出力が想定した構造を持たない場合。
    UnexpectedShape(String),
}

impl fmt::Display for CheckDepVersionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckDepVersionsError::CommandFailed(msg) => write!(f, "{msg}"),
            CheckDepVersionsError::UnexpectedShape(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for CheckDepVersionsError {}

/// 依存の種別（`cargo metadata` の `dependencies[].kind`: `null` = Normal,
/// `"dev"` = Dev, `"build"` = Build）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepKind {
    Normal,
    Dev,
    Build,
}

impl fmt::Display for DepKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DepKind::Normal => "normal",
            DepKind::Dev => "dev",
            DepKind::Build => "build",
        };
        write!(f, "{s}")
    }
}

/// 1 依存宣言分の情報（`cargo metadata` の `dependencies[]` 要素を写したもの）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawDependency {
    /// 依存先クレートの実名（`rename`/`package =` を使っていても実名が入る。
    /// `path` による突き合わせを主とするため、この名前自体には依存しない）。
    pub name: String,
    /// 正規化済み version 要求（`version` 未指定は `"*"`）。
    pub req: String,
    pub kind: DepKind,
    /// path 依存の場合のみ、依存先クレートディレクトリの絶対パス。
    pub path: Option<String>,
}

/// workspace メンバー 1 クレート分の情報。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceMember {
    pub name: String,
    pub version: String,
    /// `Cargo.toml` の絶対パス。`--fix` の書き込み先はここに限定される。
    pub manifest_path: String,
    /// `publish` フィールドが未指定または非空配列（`check_version_bump` と同じ
    /// 判定）。ルール 2 の適用対象を絞るのに使う。
    pub publishable: bool,
    pub dependencies: Vec<RawDependency>,
}

/// `manifest_path`（`.../Cargo.toml`）からディレクトリ部分を取り出す。
/// 末尾が `Cargo.toml` でない想定外形状は `None`。
fn manifest_dir(manifest_path: &str) -> Option<&str> {
    let dir = manifest_path.strip_suffix("Cargo.toml")?;
    Some(dir.trim_end_matches('/'))
}

/// `cargo metadata --no-deps` を実行し、workspace ルートと全メンバー
/// （公開可否を問わず依存元として検査するため、`check_version_bump` と異なり
/// 非公開クレートも含める）の一覧を得る。
///
/// `--locked` は付けない: 本ゲートはネットワーク照会を伴わない軽量検知であり
/// `Cargo.lock` の整合性検証を目的としないため
/// （`check_version_bump::published_crates_from_cargo_metadata` と同一理由）。
pub fn workspace_packages_from_cargo_metadata(
) -> Result<(String, Vec<WorkspaceMember>), CheckDepVersionsError> {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let output = Command::new(cargo_bin)
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .map_err(|e| {
            CheckDepVersionsError::CommandFailed(format!("failed to run cargo metadata: {e}"))
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CheckDepVersionsError::CommandFailed(format!(
            "cargo metadata exited with {status}: {stderr}",
            status = output.status,
        )));
    }
    let stdout = String::from_utf8(output.stdout).map_err(|e| {
        CheckDepVersionsError::CommandFailed(format!(
            "cargo metadata output is not valid UTF-8: {e}"
        ))
    })?;
    let json = parse(&stdout).map_err(|e| {
        CheckDepVersionsError::UnexpectedShape(format!("failed to parse cargo metadata JSON: {e}"))
    })?;

    let workspace_root = json
        .get("workspace_root")
        .and_then(Json::as_str)
        .ok_or_else(|| {
            CheckDepVersionsError::UnexpectedShape("missing `workspace_root`".to_string())
        })?
        .to_string();
    let packages = json
        .get("packages")
        .and_then(Json::as_array)
        .ok_or_else(|| {
            CheckDepVersionsError::UnexpectedShape("missing `packages` array".to_string())
        })?;

    let mut members = Vec::new();
    for pkg in packages {
        let name = pkg.get("name").and_then(Json::as_str).ok_or_else(|| {
            CheckDepVersionsError::UnexpectedShape("package missing `name`".to_string())
        })?;
        let version = pkg.get("version").and_then(Json::as_str).ok_or_else(|| {
            CheckDepVersionsError::UnexpectedShape(format!("package `{name}` missing `version`"))
        })?;
        let manifest_path = pkg
            .get("manifest_path")
            .and_then(Json::as_str)
            .ok_or_else(|| {
                CheckDepVersionsError::UnexpectedShape(format!(
                    "package `{name}` missing `manifest_path`"
                ))
            })?;
        if manifest_dir(manifest_path).is_none() {
            return Err(CheckDepVersionsError::UnexpectedShape(format!(
                "package `{name}` has unexpected `manifest_path` (does not end with Cargo.toml): {manifest_path}"
            )));
        }

        // `publish` の判定は check_version_bump::published_crates_from_cargo_metadata
        // と同一契約（想定外形状は「公開対象」側へ fail-closed）。
        let publishable = match pkg.get("publish") {
            None | Some(Json::Null) => true,
            Some(Json::Array(items)) => !items.is_empty(),
            Some(_) => true,
        };

        let deps_json = pkg
            .get("dependencies")
            .and_then(Json::as_array)
            .ok_or_else(|| {
                CheckDepVersionsError::UnexpectedShape(format!(
                    "package `{name}` missing `dependencies` array"
                ))
            })?;
        let mut dependencies = Vec::new();
        for dep in deps_json {
            let dep_name = dep.get("name").and_then(Json::as_str).ok_or_else(|| {
                CheckDepVersionsError::UnexpectedShape(format!(
                    "package `{name}` has a dependency missing `name`"
                ))
            })?;
            let req = dep.get("req").and_then(Json::as_str).ok_or_else(|| {
                CheckDepVersionsError::UnexpectedShape(format!(
                    "package `{name}` dependency `{dep_name}` missing `req`"
                ))
            })?;
            let kind = match dep.get("kind") {
                None | Some(Json::Null) => DepKind::Normal,
                Some(Json::String(s)) if s == "dev" => DepKind::Dev,
                Some(Json::String(s)) if s == "build" => DepKind::Build,
                Some(other) => {
                    return Err(CheckDepVersionsError::UnexpectedShape(format!(
                        "package `{name}` dependency `{dep_name}` has unexpected `kind`: {other:?}"
                    )))
                }
            };
            let path = match dep.get("path") {
                None | Some(Json::Null) => None,
                Some(Json::String(s)) => Some(s.clone()),
                Some(other) => {
                    return Err(CheckDepVersionsError::UnexpectedShape(format!(
                        "package `{name}` dependency `{dep_name}` has unexpected `path`: {other:?}"
                    )))
                }
            };
            dependencies.push(RawDependency {
                name: dep_name.to_string(),
                req: req.to_string(),
                kind,
                path,
            });
        }

        members.push(WorkspaceMember {
            name: name.to_string(),
            version: version.to_string(),
            manifest_path: manifest_path.to_string(),
            publishable,
            dependencies,
        });
    }
    Ok((workspace_root, members))
}

/// 1 依存エッジ（依存元 → 依存先、workspace 内 path 依存のみ）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub from_name: String,
    pub from_publishable: bool,
    pub from_manifest_path: String,
    pub to_name: String,
    pub to_version: String,
    pub req: String,
    pub kind: DepKind,
}

/// `members` から「`path` を持ち依存先が workspace メンバーである」エッジのみを
/// 抽出する（純粋関数）。依存先の特定は絶対パス（依存先クレートディレクトリ）の
/// 完全一致で行う（`rename`/`package =` が使われていても `dependencies[].name`
/// は実クレート名でありパス突き合わせなら影響を受けない）。
pub fn collect_edges(members: &[WorkspaceMember]) -> Vec<Edge> {
    let dir_to_member: HashMap<&str, &WorkspaceMember> = members
        .iter()
        .filter_map(|m| manifest_dir(&m.manifest_path).map(|dir| (dir, m)))
        .collect();

    let mut edges = Vec::new();
    for member in members {
        for dep in &member.dependencies {
            let Some(path) = &dep.path else {
                continue;
            };
            let Some(target) = dir_to_member.get(path.as_str()) else {
                // path 依存先が workspace メンバー外（想定上は起きないが、
                // 生成テストフィクスチャ等での外部 path 依存を許容する）。
                continue;
            };
            edges.push(Edge {
                from_name: member.name.clone(),
                from_publishable: member.publishable,
                from_manifest_path: member.manifest_path.clone(),
                to_name: target.name.clone(),
                to_version: target.version.clone(),
                req: dep.req.clone(),
                kind: dep.kind,
            });
        }
    }
    edges
}

/// エッジ 1 件分の判定結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Judgement {
    Pass,
    Fail(FailReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailReason {
    /// version 宣言はあるが、依存先の現行 version と一致しない（追随漏れ）。
    VersionMismatch,
    /// version 宣言がなく（`req == "*"`）、かつ publish 対象クレートの
    /// normal/build 依存であるため `cargo publish` 時に失敗し得る。
    MissingVersion,
}

impl Judgement {
    pub fn is_pass(self) -> bool {
        matches!(self, Judgement::Pass)
    }
}

/// §判定ルールに従い 1 エッジを判定する（純粋関数）。
pub fn judge(req: &str, actual_version: &str, kind: DepKind, from_publishable: bool) -> Judgement {
    if req == "*" {
        let missing_version_matters =
            from_publishable && matches!(kind, DepKind::Normal | DepKind::Build);
        if missing_version_matters {
            Judgement::Fail(FailReason::MissingVersion)
        } else {
            Judgement::Pass
        }
    } else if req == format!("^{actual_version}") {
        Judgement::Pass
    } else {
        Judgement::Fail(FailReason::VersionMismatch)
    }
}

/// stdout への 1 行サマリ出力用のレポート。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub crate_name: String,
    pub dep_name: String,
    pub kind: DepKind,
    pub req: String,
    pub actual: String,
    pub judgement: Judgement,
}

/// `edge` から [`Report`] を組み立てて判定する。
pub fn judge_edge(edge: &Edge) -> Report {
    let judgement = judge(
        &edge.req,
        &edge.to_version,
        edge.kind,
        edge.from_publishable,
    );
    Report {
        crate_name: edge.from_name.clone(),
        dep_name: edge.to_name.clone(),
        kind: edge.kind,
        req: edge.req.clone(),
        actual: edge.to_version.clone(),
        judgement,
    }
}

/// `dep-version-check: crate=<依存元> dep=<依存先> kind=<normal|dev|build>
/// req=<req> actual=<version> result=<PASS|FAIL>` の 1 行サマリを整形する。
/// `grep '^dep-version-check:'` で CI アノテーション生成側が抽出できる契約
/// （`check_version_bump::format_report` と同じ設計）。
pub fn format_report(r: &Report) -> String {
    let result = if r.judgement.is_pass() {
        "PASS"
    } else {
        "FAIL"
    };
    format!(
        "dep-version-check: crate={} dep={} kind={} req={} actual={} result={result}\n",
        r.crate_name, r.dep_name, r.kind, r.req, r.actual,
    )
}

/// `req` が `^X.Y.Z`（キャレット + 素の 3 要素ドット区切り数値、前後に余計な
/// 文字なし）の形をしている場合のみ `X.Y.Z` を返す。それ以外（`=` ピン・
/// 部分指定・複数条件・`workspace = true` に由来する特殊 req 等）は
/// `--fix` の対象外とし `None` を返す（一意特定できない書き換えはしない）。
fn old_version_from_caret_req(req: &str) -> Option<String> {
    let rest = req.strip_prefix('^')?;
    let parts: Vec<&str> = rest.split('.').collect();
    if parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
    {
        Some(rest.to_string())
    } else {
        None
    }
}

/// `--fix` の書き換え位置特定に失敗した理由。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixError {
    /// `req` がキャレット + 素の 3 要素表記でないため、旧 version 文字列を
    /// 一意に特定できない。
    UnsupportedReq {
        crate_name: String,
        dep_name: String,
        req: String,
    },
    /// 依存元 Cargo.toml 内に `version = "<旧>"` の書き換え候補が見つからない。
    NotFound {
        crate_name: String,
        dep_name: String,
        manifest_path: String,
    },
    /// 書き換え候補が複数あり一意に特定できない。
    Ambiguous {
        crate_name: String,
        dep_name: String,
        manifest_path: String,
        count: usize,
    },
    /// 書き込み先が workspace_root 配下でない（パストラバーサル防止の検証失敗）。
    PathOutsideWorkspace { manifest_path: String },
    /// Cargo.toml の読み取り失敗。
    Io {
        manifest_path: String,
        message: String,
    },
}

impl fmt::Display for FixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FixError::UnsupportedReq { crate_name, dep_name, req } => write!(
                f,
                "cannot auto-fix `{crate_name}` -> `{dep_name}`: version requirement `{req}` is not a plain `^X.Y.Z` form"
            ),
            FixError::NotFound { crate_name, dep_name, manifest_path } => write!(
                f,
                "cannot auto-fix `{crate_name}` -> `{dep_name}`: no unique `version = \"...\"` line found in {manifest_path}"
            ),
            FixError::Ambiguous { crate_name, dep_name, manifest_path, count } => write!(
                f,
                "cannot auto-fix `{crate_name}` -> `{dep_name}`: {count} candidate `version = \"...\"` lines found in {manifest_path} (expected exactly 1)"
            ),
            FixError::PathOutsideWorkspace { manifest_path } => write!(
                f,
                "refusing to write outside workspace root: {manifest_path}"
            ),
            FixError::Io { manifest_path, message } => write!(
                f,
                "failed to read {manifest_path}: {message}"
            ),
        }
    }
}

impl std::error::Error for FixError {}

/// 依存元 Cargo.toml 内で `dep_name` の `version = "<old_version>"` 宣言を
/// 一意に特定し、書き換え対象行のインデックス（0 始まり）を返す（純粋関数）。
///
/// インライン形式（`<dep_name> = { ..., version = "<old>", ... }`）とセクション
/// 形式（`[dependencies.<dep_name>]` 等の直下の `version = "<old>"` 行）の
/// 両方に対応する。候補が 0 件・複数件の場合は書き換えを拒否する
/// （fail-closed、部分書き込み防止）。
fn locate_version_edit(content: &str, dep_name: &str, old_version: &str) -> Result<usize, usize> {
    let lines: Vec<&str> = content.split('\n').collect();
    let mut current_section: Option<String> = None;
    let target_quoted = format!("\"{old_version}\"");
    let inline_prefix = format!("{dep_name} =");
    let section_names = [
        format!("[dependencies.{dep_name}]"),
        format!("[dev-dependencies.{dep_name}]"),
        format!("[build-dependencies.{dep_name}]"),
    ];

    let mut candidates: Vec<usize> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = Some(trimmed.to_string());
            continue;
        }
        if trimmed.starts_with(&inline_prefix)
            && trimmed.contains('{')
            && trimmed.contains("version =")
            && trimmed.contains(&target_quoted)
        {
            candidates.push(i);
            continue;
        }
        if let Some(sec) = &current_section {
            if section_names.contains(sec)
                && trimmed.starts_with("version =")
                && trimmed.contains(&target_quoted)
            {
                candidates.push(i);
            }
        }
    }

    match candidates.len() {
        1 => Ok(candidates[0]),
        n => Err(n),
    }
}

/// `line`（`locate_version_edit` が返した行）内の `"<old_version>"` を
/// `"<new_version>"` へ 1 箇所だけ置換する（純粋関数）。
fn apply_version_edit(line: &str, old_version: &str, new_version: &str) -> String {
    let old_quoted = format!("\"{old_version}\"");
    let new_quoted = format!("\"{new_version}\"");
    line.replacen(&old_quoted, &new_quoted, 1)
}

/// 1 件分の書き換え計画。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixPlan {
    pub manifest_path: String,
    pub crate_name: String,
    pub dep_name: String,
    pub old_version: String,
    pub new_version: String,
    pub line_index: usize,
}

/// ルール 1（[`FailReason::VersionMismatch`]）の FAIL エッジについて書き換え
/// 計画を立てる。1 件でも書き換え位置を一意特定できないエッジがあれば、
/// **全ての** 書き換えを行わずエラー一覧を返す（部分書き込みをしない。
/// 全編集位置の特定完了後に一括適用する設計、`apply_fixes` 参照）。
///
/// 書き込み先候補（`manifest_path`）が `workspace_root` 配下でない場合も
/// 拒否する（パストラバーサル防止、security.md A01 対応）。
pub fn plan_fixes(workspace_root: &str, edges: &[Edge]) -> Result<Vec<FixPlan>, Vec<FixError>> {
    let mut plans = Vec::new();
    let mut errors = Vec::new();

    for edge in edges {
        let judgement = judge(
            &edge.req,
            &edge.to_version,
            edge.kind,
            edge.from_publishable,
        );
        if !matches!(judgement, Judgement::Fail(FailReason::VersionMismatch)) {
            continue;
        }

        if !is_within_workspace_root(&edge.from_manifest_path, workspace_root) {
            errors.push(FixError::PathOutsideWorkspace {
                manifest_path: edge.from_manifest_path.clone(),
            });
            continue;
        }

        let Some(old_version) = old_version_from_caret_req(&edge.req) else {
            errors.push(FixError::UnsupportedReq {
                crate_name: edge.from_name.clone(),
                dep_name: edge.to_name.clone(),
                req: edge.req.clone(),
            });
            continue;
        };

        let content = match fs::read_to_string(&edge.from_manifest_path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(FixError::Io {
                    manifest_path: edge.from_manifest_path.clone(),
                    message: e.to_string(),
                });
                continue;
            }
        };

        match locate_version_edit(&content, &edge.to_name, &old_version) {
            Ok(line_index) => {
                plans.push(FixPlan {
                    manifest_path: edge.from_manifest_path.clone(),
                    crate_name: edge.from_name.clone(),
                    dep_name: edge.to_name.clone(),
                    old_version,
                    new_version: edge.to_version.clone(),
                    line_index,
                });
            }
            Err(0) => errors.push(FixError::NotFound {
                crate_name: edge.from_name.clone(),
                dep_name: edge.to_name.clone(),
                manifest_path: edge.from_manifest_path.clone(),
            }),
            Err(count) => errors.push(FixError::Ambiguous {
                crate_name: edge.from_name.clone(),
                dep_name: edge.to_name.clone(),
                manifest_path: edge.from_manifest_path.clone(),
                count,
            }),
        }
    }

    if errors.is_empty() {
        Ok(plans)
    } else {
        Err(errors)
    }
}

/// `manifest_path` が `workspace_root` 配下（末尾スラッシュ境界込み）かを検証する。
fn is_within_workspace_root(manifest_path: &str, workspace_root: &str) -> bool {
    let root = workspace_root.trim_end_matches('/');
    manifest_path == root || manifest_path.starts_with(&format!("{root}/"))
}

/// `plan_fixes` が返した計画をファイルへ一括適用する。
///
/// 同一ファイルに対する複数の書き換えはまとめて 1 度の読み書きで適用する
/// （`manifest_path` ごとにグルーピング）。`plan_fixes` 側で全計画の位置特定が
/// 完了していることが前提で、本関数自体は追加の妥当性判断をせず書き込むのみ。
pub fn apply_fixes(plans: &[FixPlan]) -> Result<(), FixError> {
    let mut by_file: HashMap<&str, Vec<&FixPlan>> = HashMap::new();
    for plan in plans {
        by_file
            .entry(plan.manifest_path.as_str())
            .or_default()
            .push(plan);
    }

    for (manifest_path, file_plans) in by_file {
        let content = fs::read_to_string(manifest_path).map_err(|e| FixError::Io {
            manifest_path: manifest_path.to_string(),
            message: e.to_string(),
        })?;
        let mut lines: Vec<String> = content.split('\n').map(str::to_string).collect();
        for plan in file_plans {
            if let Some(line) = lines.get_mut(plan.line_index) {
                *line = apply_version_edit(line, &plan.old_version, &plan.new_version);
            }
        }
        let new_content = lines.join("\n");
        fs::write(manifest_path, new_content).map_err(|e| FixError::Io {
            manifest_path: manifest_path.to_string(),
            message: e.to_string(),
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(
        name: &str,
        version: &str,
        manifest_path: &str,
        publishable: bool,
    ) -> WorkspaceMember {
        WorkspaceMember {
            name: name.to_string(),
            version: version.to_string(),
            manifest_path: manifest_path.to_string(),
            publishable,
            dependencies: Vec::new(),
        }
    }

    #[test]
    fn manifest_dir_strips_cargo_toml_suffix() {
        assert_eq!(
            manifest_dir("/repo/crates/core/Cargo.toml"),
            Some("/repo/crates/core")
        );
        assert_eq!(manifest_dir("/repo/crates/core/lib.rs"), None);
    }

    #[test]
    fn judge_passes_on_exact_caret_match() {
        assert_eq!(
            judge("^0.2.0", "0.2.0", DepKind::Normal, true),
            Judgement::Pass
        );
    }

    #[test]
    fn judge_fails_on_stale_version() {
        assert_eq!(
            judge("^0.1.0", "0.2.0", DepKind::Normal, true),
            Judgement::Fail(FailReason::VersionMismatch)
        );
    }

    #[test]
    fn judge_fails_on_pinned_or_partial_req_mismatch() {
        // `=0.2.0` や部分指定 `0.2` は正規化された `^0.2.0` と一致しないため FAIL
        // （3 要素完全表記を機械的に固定する設計、計画書 §3.1）。
        assert_eq!(
            judge("=0.2.0", "0.2.0", DepKind::Normal, true),
            Judgement::Fail(FailReason::VersionMismatch)
        );
    }

    #[test]
    fn judge_missing_version_fails_for_publishable_normal_dep() {
        assert_eq!(
            judge("*", "0.1.0", DepKind::Normal, true),
            Judgement::Fail(FailReason::MissingVersion)
        );
        assert_eq!(
            judge("*", "0.1.0", DepKind::Build, true),
            Judgement::Fail(FailReason::MissingVersion)
        );
    }

    #[test]
    fn judge_missing_version_passes_for_dev_dep() {
        // dev-dependencies は publish 時に自動除去されるため version 欠落は無害。
        assert_eq!(judge("*", "0.1.0", DepKind::Dev, true), Judgement::Pass);
    }

    #[test]
    fn judge_missing_version_passes_when_from_not_publishable() {
        // publish = false のクレート（xtask/docs-site 等）は version 欠落でも
        // crates.io 公開を試みないため無害。
        assert_eq!(judge("*", "0.1.0", DepKind::Normal, false), Judgement::Pass);
    }

    #[test]
    fn format_report_matches_grep_prefix_contract() {
        let report = Report {
            crate_name: "fandhe-frontend-pre-styled-ui".to_string(),
            dep_name: "fandhe-frontend-headless-ui".to_string(),
            kind: DepKind::Normal,
            req: "^0.1.0".to_string(),
            actual: "0.2.0".to_string(),
            judgement: Judgement::Fail(FailReason::VersionMismatch),
        };
        let line = format_report(&report);
        assert!(line.starts_with("dep-version-check: "));
        assert!(line.contains("crate=fandhe-frontend-pre-styled-ui"));
        assert!(line.contains("dep=fandhe-frontend-headless-ui"));
        assert!(line.contains("kind=normal"));
        assert!(line.contains("req=^0.1.0"));
        assert!(line.contains("actual=0.2.0"));
        assert!(line.contains("result=FAIL"));
    }

    #[test]
    fn collect_edges_matches_by_absolute_path_and_ignores_non_path_deps() {
        let mut a = member("a", "0.2.0", "/repo/crates/a/Cargo.toml", true);
        let mut b = member("b", "0.1.0", "/repo/crates/b/Cargo.toml", true);
        b.dependencies.push(RawDependency {
            name: "a".to_string(),
            req: "^0.1.0".to_string(),
            kind: DepKind::Normal,
            path: Some("/repo/crates/a".to_string()),
        });
        // path なしの通常の registry 依存はエッジ対象外。
        b.dependencies.push(RawDependency {
            name: "serde".to_string(),
            req: "^1".to_string(),
            kind: DepKind::Normal,
            path: None,
        });
        a.dependencies.push(RawDependency {
            name: "external".to_string(),
            req: "*".to_string(),
            kind: DepKind::Normal,
            path: Some("/outside/external".to_string()),
        });
        let members = vec![a, b];
        let edges = collect_edges(&members);
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].from_name, "b");
        assert_eq!(edges[0].to_name, "a");
        assert_eq!(edges[0].to_version, "0.2.0");
    }

    #[test]
    fn old_version_from_caret_req_accepts_plain_triple_only() {
        assert_eq!(
            old_version_from_caret_req("^0.2.0"),
            Some("0.2.0".to_string())
        );
        assert_eq!(old_version_from_caret_req("=0.2.0"), None);
        assert_eq!(old_version_from_caret_req("^0.2"), None);
        assert_eq!(old_version_from_caret_req("*"), None);
    }

    #[test]
    fn locate_version_edit_finds_inline_form() {
        let content = "\
[package]
name = \"b\"
version = \"0.1.0\"

[dependencies]
a = { path = \"../a\", version = \"0.1.0\" }
";
        let idx = locate_version_edit(content, "a", "0.1.0").unwrap();
        assert_eq!(
            content.split('\n').nth(idx).unwrap(),
            "a = { path = \"../a\", version = \"0.1.0\" }"
        );
    }

    #[test]
    fn locate_version_edit_finds_section_form() {
        let content = "\
[package]
name = \"b\"
version = \"0.1.0\"

[dependencies.a]
path = \"../a\"
version = \"0.1.0\"
";
        let idx = locate_version_edit(content, "a", "0.1.0").unwrap();
        assert_eq!(content.split('\n').nth(idx).unwrap(), "version = \"0.1.0\"");
    }

    #[test]
    fn locate_version_edit_rejects_zero_or_ambiguous_matches() {
        let no_match = "[dependencies]\na = { path = \"../a\", version = \"0.9.9\" }\n";
        assert_eq!(locate_version_edit(no_match, "a", "0.1.0"), Err(0));

        // dev-dependencies にも同名依存が別バージョンで存在する等、想定外に
        // 複数箇所へ同一 old_version が現れるケースは一意特定できない。
        let ambiguous = "\
[dependencies]
a = { path = \"../a\", version = \"0.1.0\" }

[dev-dependencies.a]
path = \"../a\"
version = \"0.1.0\"
";
        assert_eq!(locate_version_edit(ambiguous, "a", "0.1.0"), Err(2));
    }

    #[test]
    fn apply_version_edit_replaces_exactly_one_occurrence() {
        let line = "a = { path = \"../a\", version = \"0.1.0\" }";
        assert_eq!(
            apply_version_edit(line, "0.1.0", "0.2.0"),
            "a = { path = \"../a\", version = \"0.2.0\" }"
        );
    }

    #[test]
    fn is_within_workspace_root_rejects_paths_outside_root() {
        assert!(is_within_workspace_root(
            "/repo/crates/a/Cargo.toml",
            "/repo"
        ));
        assert!(!is_within_workspace_root(
            "/other/crates/a/Cargo.toml",
            "/repo"
        ));
        // 兄弟ディレクトリ名の前方一致による誤判定を防ぐ（境界チェック）。
        assert!(!is_within_workspace_root("/repo-evil/Cargo.toml", "/repo"));
    }

    #[test]
    fn plan_fixes_reports_unsupported_req_without_touching_disk() {
        let edges = vec![Edge {
            from_name: "b".to_string(),
            from_publishable: true,
            from_manifest_path: "/repo/crates/b/Cargo.toml".to_string(),
            to_name: "a".to_string(),
            to_version: "0.2.0".to_string(),
            req: "=0.1.0".to_string(),
            kind: DepKind::Normal,
        }];
        let result = plan_fixes("/repo", &edges);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], FixError::UnsupportedReq { .. }));
    }

    #[test]
    fn plan_fixes_rejects_manifest_path_outside_workspace_root() {
        let edges = vec![Edge {
            from_name: "b".to_string(),
            from_publishable: true,
            from_manifest_path: "/outside/crates/b/Cargo.toml".to_string(),
            to_name: "a".to_string(),
            to_version: "0.2.0".to_string(),
            req: "^0.1.0".to_string(),
            kind: DepKind::Normal,
        }];
        let result = plan_fixes("/repo", &edges);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(matches!(errors[0], FixError::PathOutsideWorkspace { .. }));
    }
}
