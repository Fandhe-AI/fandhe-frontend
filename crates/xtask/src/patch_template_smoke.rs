//! イシュー #885: `template-app-wasm-smoke` ジョブ（`.github/workflows/ci.yml`）に
//! 向けた `[patch.crates-io]` 依存解決フォールバック。
//!
//! 背景（詳細は `docs/ci/version-bump-publish-order-gap.md`、イシュー #884）:
//! `templates/app`（`fw new --template app`）は fandhe-frontend-core /
//! -app / -wasm-client への crates.io バージョン依存で完結する。これらの
//! `src/` を変更する PR では、version-bump-guard（バンプ強制）・
//! `template_vendor_drift`（テンプレート依存要求と正本 `version` の一致強制）・
//! `template-app-wasm-smoke`（crates.io 実 index での依存解決）の三すくみに
//! より、バンプ先バージョンが crates.io へ未公開の間 smoke が必ず fail する
//! 構造的デッドロックが生じる。
//!
//! 本モジュールは smoke ジョブの「fw new」ステップと「build.sh」ステップの間で
//! 実行される（`xtask patch-template-smoke` サブコマンド、`main.rs` 参照）:
//!
//! 1. 生成プロジェクトのルート `Cargo.toml`・`wasm/Cargo.toml` から直接依存
//!    （`fandhe-frontend-* = "X.Y.Z"` 形式）を抽出する。
//! 2. 各依存を [`crate::check_version_bump::query_index`]（既存の fail-closed
//!    sparse index 照会。同一クレート内で再利用し pin/契約を二重管理しない）で
//!    照会し、要求バージョンがキャレット要件を満たして充足可能かを判定する。
//! 3. 充足可能な依存はファイル無変更のまま `resolution=crates-io` を報告する。
//! 4. 充足不能な依存が 1 件でもあれば、当該マニフェストへ `[patch.crates-io]`
//!    （`<repo-root>/crates/<dir>` への絶対パス指定）を注入し、対応する
//!    `Cargo.lock` を削除する（未公開バージョンは crates.io 側で解決不能な
//!    ため、`--locked` 相当の再現性は当該マニフェストに限り失う。これは
//!    意図的な設計判断であり隠蔽しない）。`resolution=path-override` を報告する。
//!
//! crates.io 公開の承認境界（`release.yml` の `workflow_dispatch` +
//! `mode: publish` 明示選択）は本モジュールが一切触れない対象であり、あくまで
//! CI 内部の検証経路（依存解決）の調整に留める（設計文書 §4 (b)）。
//! 緩和経路（発動有無を切り替える環境変数・CLI フラグ）は設けない
//! （設計文書 §5 項 4、迂回禁止原則）。

use crate::check_version_bump::{self, CheckVersionBumpError, IndexLookup};
use std::fmt;
use std::fs;
use std::path::Path;

/// 本モジュールの操作で発生し得るエラー。
///
/// [`PatchTemplateSmokeError::Environment`] は `docs/design/gate-design.md`
/// §2.3a・`check_version_bump::CheckVersionBumpError::EnvironmentError` と
/// 同じ「runner/ネットワーク起因」区分（`Display` が常に
/// `"environment error: "` プレフィックスを含む）。それ以外はすべて
/// コード起因（テンプレート・repo-root の想定外状態）として区別する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchTemplateSmokeError {
    /// マニフェスト読み書き（I/O）の失敗。
    Io(String),
    /// マニフェストが workspace 依存の想定外形式（テーブル形式・path 依存など）
    /// を含む場合。vendor 同梱への回帰を fail-closed に検知する
    /// （`crates/cli/tests/template_vendor_drift.rs::version_dependency` と
    /// 同じ検知観点）。
    UnsupportedDependencyForm(String),
    /// マニフェストに既に `[patch.crates-io]` セクションが存在する場合
    /// （想定外状態を無条件に上書きしない）。
    PatchSectionAlreadyPresent(String),
    /// `--repo-root` 配下の `crates/*/Cargo.toml` から要求クレート名が
    /// 見つからない、またはバージョンが要求値と一致しない場合。
    RepoRootCrateMismatch(String),
    /// crates.io sparse index への到達性起因の失敗
    /// （[`CheckVersionBumpError::EnvironmentError`] をそのまま透過）。
    Environment(String),
}

impl fmt::Display for PatchTemplateSmokeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatchTemplateSmokeError::Io(msg) => write!(f, "{msg}"),
            PatchTemplateSmokeError::UnsupportedDependencyForm(msg) => write!(f, "{msg}"),
            PatchTemplateSmokeError::PatchSectionAlreadyPresent(msg) => write!(f, "{msg}"),
            PatchTemplateSmokeError::RepoRootCrateMismatch(msg) => write!(f, "{msg}"),
            PatchTemplateSmokeError::Environment(msg) => write!(f, "environment error: {msg}"),
        }
    }
}

impl std::error::Error for PatchTemplateSmokeError {}

impl From<CheckVersionBumpError> for PatchTemplateSmokeError {
    fn from(e: CheckVersionBumpError) -> Self {
        match e {
            CheckVersionBumpError::EnvironmentError(msg) => {
                PatchTemplateSmokeError::Environment(msg)
            }
            other => PatchTemplateSmokeError::Io(other.to_string()),
        }
    }
}

/// マニフェスト内で `fandhe-frontend-` から始まる直接依存を行ベースで抽出する。
///
/// `template_vendor_drift.rs::version_dependency` と同型の line-based
/// パーサ（外部 TOML パーサは追加しない、`coding-rust.md` の xtask 外部依存
/// ゼロ方針）。以下を検知する:
/// - `<name> = "X.Y.Z"` 形式のみを直接依存として抽出する。
/// - `<name> = { ... }`（テーブル形式。path 依存・vendor 同梱の典型的な書き方）
///   を検出した場合は [`PatchTemplateSmokeError::UnsupportedDependencyForm`]
///   として即座にエラーにする（本フォールバックが「未レビューの vendor 同梱」
///   を誤って許容しないための fail-closed 検証）。
///
/// `[dependencies]` / `[dev-dependencies]` / `[build-dependencies]` の
/// セクション区別は行わない（この後の [`process_manifest`] は
/// `[patch.crates-io]` へ「クレート名 1 件につき 1 エントリ」を書き出す
/// 契約であり、セクションをまたいで同一クレートが重複記載されていても
/// 出力を歪めてはならない）。そのため同名クレートが複数回現れた場合は
/// ここで直接デデュープし、要求バージョンが食い違っていれば
/// [`PatchTemplateSmokeError::UnsupportedDependencyForm`] で fail-closed に
/// 検知する（同一キーを 2 回書いて `[patch.crates-io]` の TOML パースを
/// 壊す・あるいは異なるバージョン要求のどちらか一方を無言で握り潰す、の
/// 両方を避けるため）。
pub fn extract_version_dependencies(
    manifest: &str,
) -> Result<Vec<(String, String)>, PatchTemplateSmokeError> {
    let mut deps: Vec<(String, String)> = Vec::new();
    for (line_no, raw_line) in manifest.lines().enumerate() {
        let line = raw_line.trim();
        let Some(rest) = line.strip_prefix("fandhe-frontend-") else {
            continue;
        };
        // クレート名は英数字・ハイフンのみ（`security.md` A03 対応の
        // 許可リスト検証: crate 名はこの後 URL・TOML へ埋め込まれるため、
        // 想定外文字を含む「名乗り」を早期に弾く）。
        let name_end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
            .unwrap_or(rest.len());
        let (suffix, after_name) = rest.split_at(name_end);
        let name = format!("fandhe-frontend-{suffix}");
        let after_name = after_name.trim_start();
        let Some(after_eq) = after_name.strip_prefix('=') else {
            continue;
        };
        let after_eq = after_eq.trim_start();
        if let Some(after_quote) = after_eq.strip_prefix('"') {
            let Some(end) = after_quote.find('"') else {
                return Err(PatchTemplateSmokeError::UnsupportedDependencyForm(format!(
                    "line {line_no_display}: unterminated version string for `{name}`: {line:?}",
                    line_no_display = line_no + 1,
                )));
            };
            let version = after_quote[..end].to_string();
            if let Some((_, existing_version)) = deps.iter().find(|(n, _)| *n == name) {
                if *existing_version != version {
                    return Err(PatchTemplateSmokeError::UnsupportedDependencyForm(format!(
                        "line {line_no_display}: `{name}` requires version {version}, but an \
earlier line in the same manifest already requires version {existing_version}; conflicting \
version requirements for the same crate across dependency sections cannot be resolved into a \
single `[patch.crates-io]` entry: {line:?}",
                        line_no_display = line_no + 1,
                    )));
                }
                // 同一クレート・同一バージョンの重複記載は無視する（複数
                // セクションにまたがる正当な再掲。`[patch.crates-io]` へ
                // 重複キーを生成しないための唯一のガード）。
            } else {
                deps.push((name, version));
            }
        } else if after_eq.starts_with('{') {
            return Err(PatchTemplateSmokeError::UnsupportedDependencyForm(format!(
                "line {line_no_display}: `{name}` is declared as a table dependency (path/patch \
already present?) instead of a plain version string; refusing to proceed to avoid masking a \
vendor-reintroduction regression: {line:?}",
                line_no_display = line_no + 1,
            )));
        }
        // それ以外（値の形状が不明）は静かに読み飛ばす（コメント行の誤検知等）。
    }
    Ok(deps)
}

/// マニフェストが既に `[patch.crates-io]` セクションを持つかを判定する
/// （想定外の既存状態を無条件に上書きしないためのガード）。
pub fn has_patch_crates_io_section(manifest: &str) -> bool {
    manifest
        .lines()
        .any(|line| line.trim() == "[patch.crates-io]")
}

/// `"X.Y.Z"` 形式のバージョン文字列を 3 要素タプルへパースする。
/// pre-release/build メタデータ（`-`/`+` 以降）は無視する
/// （本フォールバックが扱う対象は 3 要素の安定版バージョンのみのため）。
fn parse_version(v: &str) -> Option<(u64, u64, u64)> {
    let core = v.split(['-', '+']).next().unwrap_or(v);
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

/// Cargo のキャレット要件（`^X.Y.Z`、0.x 系の縮小規則込み）で `candidate` が
/// `req` を満たすかを判定する（純粋関数）。
///
/// - `^1.2.3` := `>=1.2.3, <2.0.0`
/// - `^0.2.3` := `>=0.2.3, <0.3.0`
/// - `^0.0.3` := `>=0.0.3, <0.0.4`（0.0.x は patch のみ許容、実質完全一致）
pub fn caret_satisfied(candidate: (u64, u64, u64), req: (u64, u64, u64)) -> bool {
    if candidate < req {
        return false;
    }
    let (req_major, req_minor, req_patch) = req;
    let upper = if req_major > 0 {
        (req_major + 1, 0, 0)
    } else if req_minor > 0 {
        (req_major, req_minor + 1, 0)
    } else {
        (req_major, req_minor, req_patch + 1)
    };
    candidate < upper
}

/// crates.io sparse index の照会結果（`published`）が要求バージョン
/// （キャレット要件、`req_version` は Cargo.toml に書かれた `"X.Y.Z"` の値
/// そのもの）を充足するバージョンを 1 つでも含むかを判定する。
///
/// パース不能なバージョン文字列（本来あり得ないが、レジストリ側応答の
/// 想定外形状への耐性として）は無視して次の候補を見る。
pub fn requirement_is_resolvable(published: &[String], req_version: &str) -> bool {
    let Some(req) = parse_version(req_version) else {
        return false;
    };
    published
        .iter()
        .filter_map(|v| parse_version(v))
        .any(|candidate| caret_satisfied(candidate, req))
}

/// `--repo-root` 配下の `crates/*/Cargo.toml` を走査し、`package.name` /
/// `package.version` の対を集める（`crate_dir_rel` は `crates/<dir>` 形式）。
fn discover_repo_crates(
    repo_root: &Path,
) -> Result<Vec<(String, String, String)>, PatchTemplateSmokeError> {
    let crates_dir = repo_root.join("crates");
    let entries = fs::read_dir(&crates_dir).map_err(|e| {
        PatchTemplateSmokeError::Io(format!(
            "failed to read {dir}: {e}",
            dir = crates_dir.display()
        ))
    })?;
    let mut found = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| {
            PatchTemplateSmokeError::Io(format!(
                "failed to iterate {dir}: {e}",
                dir = crates_dir.display()
            ))
        })?;
        let manifest_path = entry.path().join("Cargo.toml");
        let Ok(content) = fs::read_to_string(&manifest_path) else {
            continue;
        };
        let Some((name, version)) = parse_package_name_and_version(&content) else {
            continue;
        };
        let Some(dir_name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        found.push((name, version, format!("crates/{dir_name}")));
    }
    Ok(found)
}

/// リポジトリ側クレート（`crates/<dir>/Cargo.toml`）が持つ、他の
/// `fandhe-frontend-*` クレートへの **path 依存**（テーブル形式、`path = `
/// キーを含む）の宛先クレート名を行ベースで抽出する。
///
/// これは [`extract_version_dependencies`] とは逆の役割を持つ:
/// 生成テンプレート側マニフェストではテーブル形式（path 依存）を
/// vendor 再導入の兆候として拒否するが、こちらはリポジトリ本体の
/// ワークスペースメンバー間の正当な path 依存（例:
/// `crates/app/Cargo.toml` の `fandhe-frontend-core = { path = "../core",
/// version = "0.1.0" }`）を読み取るためのものであり、拒否しない。
///
/// [`process_manifest`] はこの関数で見つけた「path オーバーライド対象の
/// 依存先」を、その依存先自身が crates.io で解決可能であっても強制的に
/// `[patch.crates-io]` へ含める（同一パッケージが crates.io ソースと
/// path ソースの 2 箇所から解決される Cargo の依存解決失敗を防ぐため。
/// 詳細は [`process_manifest`] のドキュメント参照）。
fn extract_workspace_sibling_dependency_names(manifest: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in manifest.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("fandhe-frontend-") else {
            continue;
        };
        let name_end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
            .unwrap_or(rest.len());
        let (suffix, after_name) = rest.split_at(name_end);
        let name = format!("fandhe-frontend-{suffix}");
        let after_name = after_name.trim_start();
        let Some(after_eq) = after_name.strip_prefix('=') else {
            continue;
        };
        let after_eq = after_eq.trim_start();
        if after_eq.starts_with('{') && after_eq.contains("path") && !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

/// `[package]` セクションの `name` / `version` を行ベースで抽出する
/// （`template_vendor_drift.rs::package_version` と同型）。
fn parse_package_name_and_version(manifest: &str) -> Option<(String, String)> {
    let mut name = None;
    let mut version = None;
    let mut in_package_section = false;
    for raw_line in manifest.lines() {
        let line = raw_line.trim();
        if let Some(section) = line.strip_prefix('[') {
            in_package_section = section.trim_end_matches(']') == "package";
            continue;
        }
        if !in_package_section {
            continue;
        }
        if let Some(rest) = line.strip_prefix("name") {
            if let Some(v) = quoted_value_after_eq(rest) {
                name = Some(v);
            }
        } else if let Some(rest) = line.strip_prefix("version") {
            if let Some(v) = quoted_value_after_eq(rest) {
                version = Some(v);
            }
        }
        if name.is_some() && version.is_some() {
            break;
        }
    }
    Some((name?, version?))
}

fn quoted_value_after_eq(rest: &str) -> Option<String> {
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// 依存 1 件分の解決結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// crates.io 実 index で充足可能（フォールバック未発動）。
    CratesIo,
    /// `[patch.crates-io]` によるワークスペース内 path 参照へ切り替えた。
    PathOverride,
}

impl Resolution {
    fn as_str(&self) -> &'static str {
        match self {
            Resolution::CratesIo => "crates-io",
            Resolution::PathOverride => "path-override",
        }
    }
}

/// 依存 1 件分のサマリ行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepReport {
    pub name: String,
    pub version: String,
    pub resolution: Resolution,
}

/// `template-app-wasm-smoke: dep=<crate> version=<v> resolution=<crates-io|path-override>`
/// の 1 行サマリを整形する（設計文書 §5 項 3 の契約、
/// `grep -F 'resolution=path-override'` で CI 側がフォールバック発動を検知する）。
pub fn format_dep_report(r: &DepReport) -> String {
    format!(
        "template-app-wasm-smoke: dep={} version={} resolution={}\n",
        r.name,
        r.version,
        r.resolution.as_str()
    )
}

/// `[patch.crates-io]` ブロックを組み立てる（`deps` は `(crate_name,
/// absolute_path)` の組。空なら呼び出し禁止＝呼び出し側で必ず非空を保証する）。
fn build_patch_block(deps: &[(String, String)]) -> String {
    let mut block = String::from("\n[patch.crates-io]\n");
    for (name, abs_path) in deps {
        block.push_str(&format!("{name} = {{ path = \"{abs_path}\" }}\n"));
    }
    block
}

/// 1 マニフェスト分の処理結果。
pub struct ManifestOutcome {
    pub reports: Vec<DepReport>,
    /// フォールバックが発動し、マニフェスト・対応する `Cargo.lock` を
    /// 書き換えた場合に `true`。
    pub patched: bool,
}

/// 1 マニフェスト（`manifest_path`）を処理する。
///
/// - 依存を [`extract_version_dependencies`] で抽出（0 件なら何もせず空の
///   `ManifestOutcome` を返す）。
/// - 既存 `[patch.crates-io]` セクションがあれば
///   [`PatchTemplateSmokeError::PatchSectionAlreadyPresent`] で即座に打ち切る。
/// - 各依存を `index_base_url` へ照会し、充足可能かを判定する。
/// - 1 件でも充足不能なら、`repo_root` 配下の `crates/*/Cargo.toml` から
///   該当クレートを解決して `[patch.crates-io]` を追記し、`lock_path` が
///   存在すれば削除する。
///
/// ## 部分パッチ（partial patch）の禁止
///
/// `fandhe-frontend-app` のようなワークスペースクレートは兄弟クレート
/// （例: `fandhe-frontend-core`）に対して **path 依存**を維持している
/// （`crates/app/Cargo.toml` 参照）。このため、あるクレートを
/// `[patch.crates-io]` で path 参照へ切り替えると、そのクレート自身の
/// マニフェストが要求する兄弟クレートも path ソースから解決されることに
/// なる。もし当該マニフェスト内でその兄弟クレートを crates.io 版のまま
/// 残す（＝ crates.io で解決可能だからと `[patch.crates-io]` に含めない）
/// と、Cargo は同一パッケージを crates.io ソースと path ソースの 2 箇所
/// から見つけてしまい依存解決が失敗する（1 マニフェスト内での部分パッチ
/// はビルドを破壊する）。
///
/// これを避けるため、充足不能と判定されたクレートを起点に
/// [`extract_workspace_sibling_dependency_names`] でリポジトリ側の
/// path 依存グラフを再帰的にたどり、たどり着いた兄弟クレート全てを
/// `[patch.crates-io]` へ含める（そのクレート単体は crates.io で解決可能
/// であっても、である）。たどり着いた兄弟クレートが当該マニフェストにも
/// 直接依存として書かれていた場合は、その `DepReport` を
/// `Resolution::CratesIo` から `Resolution::PathOverride` へ差し替える。
/// マニフェストに直接書かれていない（推移的にのみ必要とされる）兄弟
/// クレートについては、repo-root 側の現行バージョンをそのまま報告する
/// （比較対象となるマニフェスト側のバージョン要求が存在しないため、
/// [`PatchTemplateSmokeError::RepoRootCrateMismatch`] の対象にはしない）。
#[allow(clippy::too_many_arguments)]
pub fn process_manifest(
    manifest_path: &Path,
    lock_path: &Path,
    repo_root: &Path,
    index_base_url: &str,
) -> Result<ManifestOutcome, PatchTemplateSmokeError> {
    let manifest = fs::read_to_string(manifest_path).map_err(|e| {
        PatchTemplateSmokeError::Io(format!(
            "failed to read {path}: {e}",
            path = manifest_path.display()
        ))
    })?;

    let deps = extract_version_dependencies(&manifest)?;
    if deps.is_empty() {
        return Ok(ManifestOutcome {
            reports: Vec::new(),
            patched: false,
        });
    }

    if has_patch_crates_io_section(&manifest) {
        return Err(PatchTemplateSmokeError::PatchSectionAlreadyPresent(
            format!(
                "{path} already has a `[patch.crates-io]` section; refusing to overwrite an \
unexpected pre-existing state",
                path = manifest_path.display()
            ),
        ));
    }

    // `name -> (manifest 上のバージョン要求, crates.io で充足可能か)`。
    // ここではまだ `DepReport` を確定させない（後続のワークスペース内
    // path 依存の閉包計算により、一度 CratesIo と判定した依存が
    // PathOverride へ差し替わり得るため）。
    let mut dep_status: Vec<(String, String, bool)> = Vec::with_capacity(deps.len());
    for (name, version) in &deps {
        let lookup = check_version_bump::query_index(index_base_url, name)?;
        let resolvable = match &lookup {
            IndexLookup::NotPublished => false,
            IndexLookup::Published(versions) => requirement_is_resolvable(versions, version),
        };
        dep_status.push((name.clone(), version.clone(), resolvable));
    }

    if dep_status.iter().all(|(_, _, resolvable)| *resolvable) {
        let reports = dep_status
            .into_iter()
            .map(|(name, version, _)| DepReport {
                name,
                version,
                resolution: Resolution::CratesIo,
            })
            .collect();
        return Ok(ManifestOutcome {
            reports,
            patched: false,
        });
    }

    let repo_crates = discover_repo_crates(repo_root)?;

    // 部分パッチ防止（モジュール doc・本関数 doc 参照）: 充足不能な依存を
    // 起点に、リポジトリ側の path 依存グラフを再帰的にたどり、
    // `[patch.crates-io]` へ含めるべき兄弟クレート名の集合を確定する。
    let mut override_names: std::collections::BTreeSet<String> = dep_status
        .iter()
        .filter(|(_, _, resolvable)| !resolvable)
        .map(|(name, _, _)| name.clone())
        .collect();
    let mut queue: Vec<String> = override_names.iter().cloned().collect();
    while let Some(name) = queue.pop() {
        let Some((_, _, dir_rel)) = repo_crates.iter().find(|(n, _, _)| *n == name) else {
            // repo-root 側にクレートが見つからない場合は、この後の
            // 直接依存に対する mismatch チェックに検知を委ねる
            // （閉包探索の時点では黙って読み飛ばす）。
            continue;
        };
        let sibling_manifest_path = repo_root.join(dir_rel).join("Cargo.toml");
        let Ok(sibling_manifest) = fs::read_to_string(&sibling_manifest_path) else {
            continue;
        };
        for sibling_name in extract_workspace_sibling_dependency_names(&sibling_manifest) {
            if override_names.insert(sibling_name.clone()) {
                queue.push(sibling_name);
            }
        }
    }

    // 当該マニフェストに直接依存として書かれているクレート（`deps`
    // 由来）は、閉包で override 対象に含まれるかどうかで
    // `Resolution` を確定する。あわせて `repo_root` 側バージョンとの
    // 一致（template_vendor_drift 不変条件）を検証する。
    let mut reports = Vec::with_capacity(dep_status.len());
    let mut patch_entries = Vec::new();
    let mut handled_names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for (name, version, resolvable) in &dep_status {
        handled_names.insert(name.clone());
        if !override_names.contains(name) {
            // 充足不能ではなく、かつ override 閉包にも含まれない
            // （＝ワークスペース path 依存として巻き込まれてもいない）。
            debug_assert!(
                *resolvable,
                "override_names に含まれない依存は crates.io で充足可能なはず"
            );
            reports.push(DepReport {
                name: name.clone(),
                version: version.clone(),
                resolution: Resolution::CratesIo,
            });
            continue;
        }
        let found = repo_crates.iter().find(|(n, _, _)| n == name);
        let Some((_, repo_version, dir_rel)) = found else {
            return Err(PatchTemplateSmokeError::RepoRootCrateMismatch(format!(
                "crate `{name}` (required by {manifest}) was not found under {repo_root}/crates/*; \
cannot build a `[patch.crates-io]` fallback for it",
                manifest = manifest_path.display(),
                repo_root = repo_root.display()
            )));
        };
        if repo_version != version {
            return Err(PatchTemplateSmokeError::RepoRootCrateMismatch(format!(
                "crate `{name}` requires version {version} in {manifest}, but {repo_root}/{dir_rel}/Cargo.toml \
declares version {repo_version}; the template's version requirement must match the repo-root \
source of truth (template_vendor_drift invariant) before a path-override fallback can be \
constructed",
                manifest = manifest_path.display(),
                repo_root = repo_root.display()
            )));
        }
        let abs_path = repo_root.join(dir_rel);
        patch_entries.push((name.clone(), abs_path.display().to_string()));
        reports.push(DepReport {
            name: name.clone(),
            version: version.clone(),
            resolution: Resolution::PathOverride,
        });
    }

    // 閉包で発見されたが当該マニフェストには直接書かれていない兄弟
    // クレート（推移的な path 依存としてのみ必要とされるもの）。
    // マニフェスト側にバージョン要求が存在しないため、repo-root の
    // 現行バージョンをそのまま採用し mismatch チェックは行わない。
    for name in &override_names {
        if handled_names.contains(name) {
            continue;
        }
        let found = repo_crates.iter().find(|(n, _, _)| n == name);
        let Some((_, repo_version, dir_rel)) = found else {
            return Err(PatchTemplateSmokeError::RepoRootCrateMismatch(format!(
                "crate `{name}` (transitively required via a workspace path dependency from a \
manifest processed for {manifest}) was not found under {repo_root}/crates/*; cannot build a \
`[patch.crates-io]` fallback for it",
                manifest = manifest_path.display(),
                repo_root = repo_root.display()
            )));
        };
        let abs_path = repo_root.join(dir_rel);
        patch_entries.push((name.clone(), abs_path.display().to_string()));
        reports.push(DepReport {
            name: name.clone(),
            version: repo_version.clone(),
            resolution: Resolution::PathOverride,
        });
    }

    let block = build_patch_block(&patch_entries);
    let new_manifest = format!("{manifest}{block}");
    fs::write(manifest_path, new_manifest).map_err(|e| {
        PatchTemplateSmokeError::Io(format!(
            "failed to write {path}: {e}",
            path = manifest_path.display()
        ))
    })?;

    // 未公開バージョンは crates.io 側で解決不能なため `Cargo.lock` の
    // 再生成自体が成立しない。削除して cargo に再解決させる（設計文書 §5
    // 項 2: この場合の再現性低下は許容し、呼び出し元（`main.rs`）が
    // stdout へ明記する）。
    if lock_path.exists() {
        fs::remove_file(lock_path).map_err(|e| {
            PatchTemplateSmokeError::Io(format!(
                "failed to remove {path}: {e}",
                path = lock_path.display()
            ))
        })?;
    }

    Ok(ManifestOutcome {
        reports,
        patched: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_version_dependencies_reads_plain_version_strings() {
        let manifest = "[dependencies]\nfandhe-frontend-core = \"0.1.0\"\nfandhe-frontend-app = \"0.2.3\"\nserde = \"1.0\"\n";
        let deps = extract_version_dependencies(manifest).unwrap();
        assert_eq!(
            deps,
            vec![
                ("fandhe-frontend-core".to_string(), "0.1.0".to_string()),
                ("fandhe-frontend-app".to_string(), "0.2.3".to_string()),
            ]
        );
    }

    #[test]
    fn extract_version_dependencies_rejects_table_form() {
        let manifest = "fandhe-frontend-core = { path = \"../core\" }\n";
        let err = extract_version_dependencies(manifest).unwrap_err();
        assert!(matches!(
            err,
            PatchTemplateSmokeError::UnsupportedDependencyForm(_)
        ));
    }

    #[test]
    fn extract_version_dependencies_dedupes_same_crate_same_version_across_sections() {
        let manifest = "[dependencies]\nfandhe-frontend-core = \"0.1.0\"\n\n[dev-dependencies]\nfandhe-frontend-core = \"0.1.0\"\n";
        let deps = extract_version_dependencies(manifest).unwrap();
        assert_eq!(
            deps,
            vec![("fandhe-frontend-core".to_string(), "0.1.0".to_string())],
            "同一クレート・同一バージョンの重複記載は 1 エントリへ畳み込み、\
             `[patch.crates-io]` へ重複キーを生成しない"
        );
    }

    #[test]
    fn extract_version_dependencies_rejects_conflicting_version_across_sections() {
        let manifest = "[dependencies]\nfandhe-frontend-core = \"0.1.0\"\n\n[dev-dependencies]\nfandhe-frontend-core = \"0.2.0\"\n";
        let err = extract_version_dependencies(manifest).unwrap_err();
        assert!(matches!(
            err,
            PatchTemplateSmokeError::UnsupportedDependencyForm(_)
        ));
    }

    #[test]
    fn extract_version_dependencies_ignores_unrelated_lines() {
        let manifest = "# comment fandhe-frontend-core\n[package]\nname = \"x\"\n";
        assert_eq!(extract_version_dependencies(manifest).unwrap(), Vec::new());
    }

    #[test]
    fn has_patch_crates_io_section_detects_exact_header() {
        assert!(has_patch_crates_io_section(
            "[dependencies]\n\n[patch.crates-io]\nfoo = { path = \"x\" }\n"
        ));
        assert!(!has_patch_crates_io_section("[dependencies]\n"));
    }

    #[test]
    fn parse_version_accepts_three_part_semver_only() {
        assert_eq!(parse_version("0.1.0"), Some((0, 1, 0)));
        assert_eq!(parse_version("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("1.2.3-rc.1"), Some((1, 2, 3)));
        assert_eq!(parse_version("1.2"), None);
        assert_eq!(parse_version("1.2.3.4"), None);
        assert_eq!(parse_version("not-a-version"), None);
    }

    #[test]
    fn caret_satisfied_follows_cargo_0x_shrinking_rules() {
        // ^1.2.3 := >=1.2.3, <2.0.0
        assert!(caret_satisfied((1, 5, 0), (1, 2, 3)));
        assert!(!caret_satisfied((2, 0, 0), (1, 2, 3)));
        assert!(!caret_satisfied((1, 2, 2), (1, 2, 3)));
        // ^0.2.3 := >=0.2.3, <0.3.0
        assert!(caret_satisfied((0, 2, 9), (0, 2, 3)));
        assert!(!caret_satisfied((0, 3, 0), (0, 2, 3)));
        // ^0.0.3 := >=0.0.3, <0.0.4 (実質完全一致)
        assert!(caret_satisfied((0, 0, 3), (0, 0, 3)));
        assert!(!caret_satisfied((0, 0, 4), (0, 0, 3)));
    }

    #[test]
    fn requirement_is_resolvable_finds_any_satisfying_published_version() {
        let published = vec!["0.1.0".to_string(), "0.2.5".to_string()];
        assert!(requirement_is_resolvable(&published, "0.2.0"));
        assert!(!requirement_is_resolvable(&published, "0.3.0"));
    }

    #[test]
    fn requirement_is_resolvable_ignores_unparseable_published_versions() {
        let published = vec!["not-a-version".to_string(), "0.1.5".to_string()];
        assert!(requirement_is_resolvable(&published, "0.1.0"));
    }

    #[test]
    fn requirement_is_resolvable_false_when_req_itself_unparseable() {
        let published = vec!["0.1.0".to_string()];
        assert!(!requirement_is_resolvable(&published, "bogus"));
    }

    #[test]
    fn parse_package_name_and_version_reads_package_section_only() {
        let manifest = "[dependencies]\nname = \"should-not-match\"\nversion = \"9.9.9\"\n\n[package]\nname = \"fandhe-frontend-core\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
        assert_eq!(
            parse_package_name_and_version(manifest),
            Some(("fandhe-frontend-core".to_string(), "0.1.0".to_string()))
        );
    }

    #[test]
    fn format_dep_report_matches_grep_contract() {
        let report = DepReport {
            name: "fandhe-frontend-core".to_string(),
            version: "0.1.0".to_string(),
            resolution: Resolution::PathOverride,
        };
        let line = format_dep_report(&report);
        assert!(line.starts_with("template-app-wasm-smoke: "));
        assert!(line.contains("dep=fandhe-frontend-core"));
        assert!(line.contains("version=0.1.0"));
        assert!(line.contains("resolution=path-override"));
    }

    #[test]
    fn build_patch_block_renders_one_entry_per_dep() {
        let deps = vec![
            (
                "fandhe-frontend-core".to_string(),
                "/repo/crates/core".to_string(),
            ),
            (
                "fandhe-frontend-app".to_string(),
                "/repo/crates/app".to_string(),
            ),
        ];
        let block = build_patch_block(&deps);
        assert!(block.contains("[patch.crates-io]"));
        assert!(block.contains("fandhe-frontend-core = { path = \"/repo/crates/core\" }"));
        assert!(block.contains("fandhe-frontend-app = { path = \"/repo/crates/app\" }"));
    }

    #[test]
    fn extract_workspace_sibling_dependency_names_reads_table_form_path_deps() {
        let manifest = "[dependencies]\nfandhe-frontend-core = { path = \"../core\", version = \"0.1.0\" }\nserde = \"1.0\"\n";
        assert_eq!(
            extract_workspace_sibling_dependency_names(manifest),
            vec!["fandhe-frontend-core".to_string()]
        );
    }

    #[test]
    fn extract_workspace_sibling_dependency_names_ignores_plain_version_strings() {
        let manifest = "[dependencies]\nfandhe-frontend-core = \"0.1.0\"\n";
        assert!(extract_workspace_sibling_dependency_names(manifest).is_empty());
    }

    #[test]
    fn extract_workspace_sibling_dependency_names_dedupes_repeated_names() {
        let manifest = "[dependencies]\nfandhe-frontend-core = { path = \"../core\" }\n\n\
[dev-dependencies]\nfandhe-frontend-core = { path = \"../core\" }\n";
        assert_eq!(
            extract_workspace_sibling_dependency_names(manifest),
            vec!["fandhe-frontend-core".to_string()]
        );
    }
}
