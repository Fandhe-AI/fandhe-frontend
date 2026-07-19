//! `cargo metadata` 連携（TASK-13.1c, #130）。
//!
//! [`crate::structure`] が定義する宣言（`structure.toml`）と、実際の workspace
//! 構成（`cargo metadata --format-version 1` の出力）とを突き合わせるための
//! 最小限の抽出処理を提供する。[`crate::json`]（`cli` 内部専用の外部依存ゼロ
//! JSON パーサ、`xtask/src/json.rs` と同設計の独立コピー）の上に構築されており、
//! `cargo metadata` の出力を丸ごとモデル化するのではなく、TASK-13.1 の実体突き合わせ
//! （`fw structure` の 4 要素のうち `dependencies` 要素・ディレクトリ実在確認）に
//! 必要な最小限のビュー（[`WorkspaceMetadata`]）だけを取り出す。
//!
//! `cargo metadata` はワークスペース内で信頼された `cargo` バイナリの標準出力
//! であり、`std::process::Command` の固定引数のみで起動する（shell を経由しない、
//! security.md A03 インジェクション対策）。ただし出力自体（JSON）は外部プロセスの
//! 生成物であり、想定外の構造を panic せず [`MetadataError`] として返す
//! （security.md A08 データ整合性）。

use crate::json::{self, Json};
use std::path::{Path, PathBuf};
use std::process::Command;

/// `cargo metadata` 実行・パース・整形に失敗した場合のエラー。
///
/// エラーメッセージは英語（`japanese-style.md`）とし、コマンドの stderr は
/// そのまま含めない（内部パス・環境情報の漏えいを避ける、security.md）。
#[derive(Debug, Clone, PartialEq)]
pub enum MetadataError {
    /// `cargo metadata` プロセスの起動・実行に失敗した（バイナリ不在・非 0 終了等）。
    CommandFailed(String),
    /// 標準出力が UTF-8 として不正だった。
    InvalidUtf8,
    /// JSON としてパースできなかった。
    Json(json::JsonError),
    /// JSON としては妥当だが、想定する `cargo metadata` の構造と一致しなかった。
    UnexpectedShape(String),
}

impl std::fmt::Display for MetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataError::CommandFailed(msg) => {
                write!(f, "failed to run cargo metadata: {msg}")
            }
            MetadataError::InvalidUtf8 => {
                write!(f, "cargo metadata output is not valid UTF-8")
            }
            MetadataError::Json(e) => write!(f, "cargo metadata output is not valid JSON: {e}"),
            MetadataError::UnexpectedShape(msg) => {
                write!(f, "cargo metadata output has unexpected shape: {msg}")
            }
        }
    }
}

impl std::error::Error for MetadataError {}

impl From<json::JsonError> for MetadataError {
    fn from(e: json::JsonError) -> Self {
        MetadataError::Json(e)
    }
}

/// workspace member 1 件分の実体情報。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberPackage {
    /// `Cargo.toml` の `package.name`（`structure.toml` の `directories.<name>.crate` と
    /// 突き合わせる対象）。
    pub name: String,
    /// このパッケージの `Cargo.toml` を含むディレクトリ（絶対パス）。
    pub manifest_dir: PathBuf,
    /// 通常依存（`dev`/`build` を除く）のうち、他の workspace member への依存
    /// （= 実質的な path 依存）の名前一覧。`structure.toml` の `depends_on`
    /// 宣言との突き合わせに使う。dev-dependencies は対象外
    /// （`docs/design/structure-manifest.md` §3 の server の扱いに従う）。
    pub normal_workspace_deps: Vec<String>,
}

/// `fw structure` が実体突き合わせに使う workspace 全体のビュー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceMetadata {
    /// ワークスペースルート（`Cargo.toml` を含むディレクトリ、絶対パス）。
    pub workspace_root: PathBuf,
    pub members: Vec<MemberPackage>,
    /// `resolve.nodes` に現れた全パッケージ名（workspace member・外部依存を問わず、
    /// 解決グラフに含まれる一意パッケージ名の一覧）。
    resolved_package_names: Vec<String>,
}

impl WorkspaceMetadata {
    /// 名前で workspace member を引く。
    pub fn member(&self, name: &str) -> Option<&MemberPackage> {
        self.members.iter().find(|m| m.name == name)
    }

    /// 解決済み依存パッケージ総数（workspace member 自身を除く、外部・内部問わず
    /// 全 root からの合算ではなく `resolve.nodes` に現れる全パッケージ数）。
    /// `dependencies` 要素（REQ-13 受け入れ基準 1）の参考値として `fw structure`
    /// の JSON 出力に含める。
    pub fn resolved_package_count(&self) -> usize {
        self.resolved_package_names.len()
    }
}

/// `project_dir` を起点に `cargo metadata --format-version 1` を実行し、
/// workspace member とその通常依存（workspace 内 path 依存のみ）を抽出する。
///
/// `xtask/src/check_deps.rs` の `run_cargo_metadata`（REQ-3 の 60/6 判定用）とは
/// 異なり `--locked` は付与しない: `fw structure` は「宣言と実体の構造的整合性」を
/// 確認する開発者ツールであり、`Cargo.lock` 未生成のフレッシュな clone・
/// テスト用の一時ワークスペースでも動作できることを優先する
/// （依存件数の閾値判定自体は REQ-3 のゲートである `xtask check-deps` の責務）。
pub fn fetch(project_dir: &Path) -> Result<WorkspaceMetadata, MetadataError> {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let output = Command::new(cargo_bin)
        .args(["metadata", "--format-version", "1"])
        .current_dir(project_dir)
        .output()
        .map_err(|e| MetadataError::CommandFailed(e.to_string()))?;
    if !output.status.success() {
        return Err(MetadataError::CommandFailed(format!(
            "cargo metadata exited with {status}",
            status = output.status
        )));
    }
    let stdout = String::from_utf8(output.stdout).map_err(|_| MetadataError::InvalidUtf8)?;
    parse_metadata(&stdout)
}

/// [`fetch`] のパース本体。テストから直接 JSON 文字列を渡せるよう分離する。
fn parse_metadata(input: &str) -> Result<WorkspaceMetadata, MetadataError> {
    let root = json::parse(input)?;

    let workspace_root = root
        .get("workspace_root")
        .and_then(Json::as_str)
        .ok_or_else(|| MetadataError::UnexpectedShape("missing `workspace_root`".to_string()))?;

    let workspace_members = root
        .get("workspace_members")
        .and_then(Json::as_array)
        .ok_or_else(|| MetadataError::UnexpectedShape("missing `workspace_members`".to_string()))?;
    let member_ids: Vec<&str> = workspace_members
        .iter()
        .map(|v| {
            v.as_str().ok_or_else(|| {
                MetadataError::UnexpectedShape(
                    "workspace_members entry is not a string".to_string(),
                )
            })
        })
        .collect::<Result<_, _>>()?;

    let packages = root
        .get("packages")
        .and_then(Json::as_array)
        .ok_or_else(|| MetadataError::UnexpectedShape("missing `packages`".to_string()))?;

    // id -> name の索引（resolve.nodes の依存辺を名前へ変換するのに使う）。
    let mut id_to_name: Vec<(&str, &str)> = Vec::new();
    // id -> manifest_path の索引（workspace member のディレクトリ解決用）。
    let mut id_to_manifest_path: Vec<(&str, &str)> = Vec::new();
    for pkg in packages {
        let id = pkg
            .get("id")
            .and_then(Json::as_str)
            .ok_or_else(|| MetadataError::UnexpectedShape("package missing `id`".to_string()))?;
        let name = pkg
            .get("name")
            .and_then(Json::as_str)
            .ok_or_else(|| MetadataError::UnexpectedShape("package missing `name`".to_string()))?;
        let manifest_path = pkg
            .get("manifest_path")
            .and_then(Json::as_str)
            .ok_or_else(|| {
                MetadataError::UnexpectedShape("package missing `manifest_path`".to_string())
            })?;
        id_to_name.push((id, name));
        id_to_manifest_path.push((id, manifest_path));
    }
    let name_of =
        |id: &str| -> Option<&str> { id_to_name.iter().find(|(i, _)| *i == id).map(|(_, n)| *n) };
    let workspace_member_names: Vec<&str> =
        member_ids.iter().filter_map(|id| name_of(id)).collect();

    let resolve_nodes = root
        .get("resolve")
        .and_then(|r| r.get("nodes"))
        .and_then(Json::as_array)
        .ok_or_else(|| MetadataError::UnexpectedShape("missing `resolve.nodes`".to_string()))?;

    // `resolve.nodes` に現れる全パッケージ id（`dependencies` 要素の参考値
    // ── 解決済みパッケージ総数 ── の算出に使う）。
    let mut resolved_package_names: Vec<String> = Vec::new();
    for node in resolve_nodes {
        let id = node.get("id").and_then(Json::as_str).ok_or_else(|| {
            MetadataError::UnexpectedShape("resolve node missing `id`".to_string())
        })?;
        if let Some(name) = name_of(id) {
            resolved_package_names.push(name.to_string());
        }
    }

    let mut members = Vec::new();
    for member_id in &member_ids {
        let name = name_of(member_id).ok_or_else(|| {
            MetadataError::UnexpectedShape(
                "workspace_members references unknown package id".to_string(),
            )
        })?;
        let manifest_path = id_to_manifest_path
            .iter()
            .find(|(i, _)| i == member_id)
            .map(|(_, p)| *p)
            .ok_or_else(|| {
                MetadataError::UnexpectedShape(
                    "workspace_members references package without manifest_path".to_string(),
                )
            })?;
        let manifest_dir = Path::new(manifest_path)
            .parent()
            .ok_or_else(|| {
                MetadataError::UnexpectedShape("manifest_path has no parent directory".to_string())
            })?
            .to_path_buf();

        // このメンバーの通常依存（dev/build を除く）のうち、他 workspace member を
        // 指すものだけを抽出する。dev-dependencies は `docs/design/structure-manifest.md`
        // §3 の方針（server の fandhe-frontend-core/fandhe-frontend-app は dev-dependencies のみで
        // depends_on 宣言しない）に従い対象外とする。
        let node = resolve_nodes
            .iter()
            .find(|n| n.get("id").and_then(Json::as_str) == Some(*member_id))
            .ok_or_else(|| {
                MetadataError::UnexpectedShape(
                    "workspace member missing from resolve.nodes".to_string(),
                )
            })?;
        let deps = node.get("deps").and_then(Json::as_array).ok_or_else(|| {
            MetadataError::UnexpectedShape("resolve node missing `deps`".to_string())
        })?;

        let mut normal_workspace_deps = Vec::new();
        for dep in deps {
            let pkg_id = dep
                .get("pkg")
                .and_then(Json::as_str)
                .ok_or_else(|| MetadataError::UnexpectedShape("dep missing `pkg`".to_string()))?;
            let dep_kinds = dep
                .get("dep_kinds")
                .and_then(Json::as_array)
                .ok_or_else(|| {
                    MetadataError::UnexpectedShape("dep missing `dep_kinds`".to_string())
                })?;
            let is_normal = dep_kinds.iter().any(|k| match k.get("kind") {
                None | Some(Json::Null) => true,
                Some(Json::String(_)) => false,
                _ => false,
            });
            // dep_kinds が空配列の場合は normal 扱い（xtask/src/check_deps.rs の
            // `parse_dep_kinds` と同じ安全側デフォルト。「種別不明で計測から漏れる」
            // 事故を避ける）。
            let is_normal = is_normal || dep_kinds.is_empty();
            if !is_normal {
                continue;
            }
            if let Some(dep_name) = name_of(pkg_id) {
                if workspace_member_names.contains(&dep_name) {
                    normal_workspace_deps.push(dep_name.to_string());
                }
            }
        }

        members.push(MemberPackage {
            name: name.to_string(),
            manifest_dir,
            normal_workspace_deps,
        });
    }

    Ok(WorkspaceMetadata {
        workspace_root: PathBuf::from(workspace_root),
        members,
        resolved_package_names,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_reflects_real_workspace_members() {
        // 統合テスト: 実際にこのリポジトリで `cargo metadata` を起動する
        // （`cargo test` 実行環境ではネットワークアクセスなしで完結する想定。
        // `fetch()` は `--locked` を付与しない設計（上記 doc コメント参照）だが、
        // このテストはコミット済み `Cargo.lock` を伴う in-tree ワークスペースに
        // 対して実行するため、実質的に決定的な解決結果が得られ CI でも安定する）。
        // このテストバイナリは `crates/cli/` 配下でビルドされるため、2 段の
        // 親ディレクトリでワークスペースルートを得る（イシュー #436）。
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("crates/cli/ has a workspace root two levels up");
        let metadata = fetch(workspace_root).expect("cargo metadata should succeed in-tree");
        assert!(metadata.member("fandhe-frontend-cli").is_some());
        assert!(metadata.member("fandhe-frontend-core").is_some());
        let app = metadata
            .member("fandhe-frontend-app")
            .expect("fandhe-frontend-app is a workspace member");
        assert!(app
            .normal_workspace_deps
            .contains(&"fandhe-frontend-core".to_string()));
    }
}
