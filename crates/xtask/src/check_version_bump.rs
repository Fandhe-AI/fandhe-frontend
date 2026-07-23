//! イシュー #638: 公開済みクレート（crates.io）への破壊的変更をバージョン
//! バンプ強制で検知するゲート。`.github/workflows/ci.yml` の
//! `version-bump-guard` ジョブ（PR コンテキストのみ）から呼ばれ、
//! 「公開済みバージョンと Cargo.toml の version が同一のまま、公開対象
//! クレートの実体（`src/` / `Cargo.toml` / `build.rs`）が変更される PR」を
//! fail-closed で検知する。
//!
//! headless-ui 0.1.0 公開直後、公開 API への破壊的変更（`radio_group::item`
//! の引数追加、PR #611）がバージョンバンプなしで main へマージされ、
//! crates.io バージョン依存の examples e2e が型エラーで main を赤にした
//! 事故（復旧: PR #634 + 0.2.0 再公開）が本ゲートの直接の動機。
//!
//! cargo-semver-checks のような外部ツール依存は増やさない（REQ-3・xtask
//! 外部依存ゼロ方針、coding-rust.md）。あくまで「同一バージョンのまま
//! ソースが変わった」ことを機械的に検知する軽量チェックであり、
//! 公開 API の意味論的な後方互換性までは検証しない
//! （本格的な semver 互換性検査は cargo-semver-checks 等の再評価に委ねる、
//! 別イシュー起票候補）。
//!
//! # 判定フロー
//!
//! 1. [`published_crates_from_cargo_metadata`] で workspace の公開対象
//!    クレート（`publish` フィールドが未指定または非空配列）を列挙する。
//! 2. [`changed_files`] で `git diff --name-only <base>...HEAD`
//!    （merge-base 起点の three-dot diff）から変更ファイル一覧を得る。
//! 3. [`affected_crates`] で「`crates/<dir>/src/**` ・ `Cargo.toml` ・
//!    `build.rs`」に変更のある公開対象クレートのみを抽出する
//!    （`tests/` 等は誤検知抑制のため対象外）。
//! 4. 変更のあった各クレートについて [`query_index`] で crates.io sparse
//!    index を照会し、Cargo.toml の version が既公開バージョン集合に
//!    含まれるか（＝バンプされていない）を [`judge`] で判定する。
//! 5. PR 本文の `version-bump-exempt: <crate-name>` 宣言（[`parse_exempt_crates`]）
//!    はクレート名の完全一致でのみ免除を認める（包括免除経路を作らない、
//!    security.md A05 対応）。
//!
//! # fail-closed 契約
//!
//! curl 不在・curl 非 0 終了・想定外 HTTP status はすべて
//! [`CheckVersionBumpError::EnvironmentError`]（"environment error: " プレフィックス
//! 付き、`docs/design/gate-design.md` §2.3a と同じ区別規約）として返す。
//! 呼び出し元（`xtask/src/main.rs` の `run_check_version_bump`）はこれを
//! 通常の FAIL と同様に終了コード 1 として扱うが、CI 側は本文プレフィックスで
//! 「runner 環境未整備」と「コード起因の FAIL（バンプ漏れ）」を区別できる。

use crate::json::{parse, Json};
use std::collections::HashSet;
use std::fmt;
use std::process::Command;

/// crates.io sparse index への既定照会先。
///
/// `--index-base-url` はテスト（ローカル擬似 index サーバーとの照合）専用の
/// 差し替え口であり、CI・通常運用でこの既定値を弱める（別ホストへ向ける）
/// 使い方は想定しない（`check_image_size::REQ9_IMAGE_SIZE_LIMIT_BYTES` の
/// `--limit-mb` と同じ位置付け）。
pub const DEFAULT_INDEX_BASE_URL: &str = "https://index.crates.io";

/// PR 本文で免除を宣言する際の行頭マーカー。
///
/// 例: `version-bump-exempt: fandhe-frontend-headless-ui (ドキュメントのみの変更)`
/// クレート名はこのマーカー直後の最初のトークンと完全一致する必要があり、
/// 名前なしの包括免除は認めない（fail-closed、security.md A05）。
pub const EXEMPT_MARKER: &str = "version-bump-exempt:";

/// 本モジュールの操作（`cargo metadata` 実行・`git diff` 実行・crates.io 照会）で
/// 発生し得るエラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckVersionBumpError {
    /// 外部プロセス（`cargo` / `git`）起動・実行の失敗。コード起因ではなく
    /// 呼び出し方・引数の問題であることが多いため environment error とは
    /// 区別する。
    CommandFailed(String),
    /// `cargo metadata` の出力が想定した構造（`packages` / `workspace_root` 等）を
    /// 持たない場合。
    UnexpectedShape(String),
    /// curl 不在・curl 非 0 終了・想定外 HTTP status など、runner 環境に
    /// 起因する失敗。`Display` は `docs/design/gate-design.md` §2.3a と同じ
    /// `"environment error: "` プレフィックスを常に含む。
    EnvironmentError(String),
}

impl fmt::Display for CheckVersionBumpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckVersionBumpError::CommandFailed(msg) => write!(f, "{msg}"),
            CheckVersionBumpError::UnexpectedShape(msg) => write!(f, "{msg}"),
            CheckVersionBumpError::EnvironmentError(msg) => write!(f, "environment error: {msg}"),
        }
    }
}

impl std::error::Error for CheckVersionBumpError {}

/// 公開対象クレート 1 件分の情報。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrateInfo {
    /// `Cargo.toml` の `package.name`（crates.io 上の公開名と一致）。
    pub name: String,
    /// `Cargo.toml` の `package.version`。
    pub version: String,
    /// workspace ルートからの相対ディレクトリ（例: `crates/core`）。
    /// `git diff --name-only` の出力と突き合わせるための基準になる。
    pub dir_rel: String,
}

/// `manifest_path`（絶対パス）から workspace ルート相対のクレートディレクトリを
/// 算出する。`workspace_root` の配下でない・`Cargo.toml` で終わらない等の
/// 想定外形状は `None` を返す（呼び出し側で読み飛ばす）。
fn crate_dir_rel(manifest_path: &str, workspace_root: &str) -> Option<String> {
    let rest = manifest_path.strip_prefix(workspace_root)?;
    let rest = rest.trim_start_matches('/');
    let dir = rest.strip_suffix("Cargo.toml")?;
    Some(dir.trim_end_matches('/').to_string())
}

/// `cargo metadata --no-deps` を実行し、workspace の**公開対象**クレート
/// （`publish` フィールドが未指定 [`Json::Null`] または非空配列）を列挙する。
///
/// `--no-deps` により依存クレートの解決グラフは辿らない（本チェックは
/// workspace 自身のメンバー一覧・バージョンのみを必要とし、`check_deps` の
/// 60/6 判定とは目的が異なる）。`--locked` は付けない: 本ゲートは
/// `Cargo.lock` の整合性検証を目的としておらず、依存追加直後で
/// まだ `cargo update` していない PR まで誤って fail-closed にしないため
/// （`check_deps::run_cargo_metadata` の `--locked` 方針とは意図的に異なる）。
pub fn published_crates_from_cargo_metadata() -> Result<Vec<CrateInfo>, CheckVersionBumpError> {
    let cargo_bin = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let output = Command::new(cargo_bin)
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .map_err(|e| {
            CheckVersionBumpError::CommandFailed(format!("failed to run cargo metadata: {e}"))
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CheckVersionBumpError::CommandFailed(format!(
            "cargo metadata exited with {status}: {stderr}",
            status = output.status,
        )));
    }
    let stdout = String::from_utf8(output.stdout).map_err(|e| {
        CheckVersionBumpError::CommandFailed(format!(
            "cargo metadata output is not valid UTF-8: {e}"
        ))
    })?;
    let json = parse(&stdout).map_err(|e| {
        CheckVersionBumpError::UnexpectedShape(format!("failed to parse cargo metadata JSON: {e}"))
    })?;

    let workspace_root = json
        .get("workspace_root")
        .and_then(Json::as_str)
        .ok_or_else(|| {
            CheckVersionBumpError::UnexpectedShape("missing `workspace_root`".to_string())
        })?;
    let packages = json
        .get("packages")
        .and_then(Json::as_array)
        .ok_or_else(|| {
            CheckVersionBumpError::UnexpectedShape("missing `packages` array".to_string())
        })?;

    let mut crates = Vec::new();
    for pkg in packages {
        let name = pkg.get("name").and_then(Json::as_str).ok_or_else(|| {
            CheckVersionBumpError::UnexpectedShape("package missing `name`".to_string())
        })?;
        let version = pkg.get("version").and_then(Json::as_str).ok_or_else(|| {
            CheckVersionBumpError::UnexpectedShape(format!("package `{name}` missing `version`"))
        })?;
        let manifest_path = pkg
            .get("manifest_path")
            .and_then(Json::as_str)
            .ok_or_else(|| {
                CheckVersionBumpError::UnexpectedShape(format!(
                    "package `{name}` missing `manifest_path`"
                ))
            })?;

        // `publish` は Cargo.toml に `publish = false` があると `[]`、
        // `publish = ["registry"]` なら非空配列、未指定なら `null` になる
        // （cargo metadata --format-version 1 の仕様）。想定外の形状
        // （配列でも null でもない）は見落としを避けるため「公開対象」側に
        // fail-closed で倒す。
        let is_publishable = match pkg.get("publish") {
            None | Some(Json::Null) => true,
            Some(Json::Array(items)) => !items.is_empty(),
            Some(_) => true,
        };
        if !is_publishable {
            continue;
        }

        let Some(dir_rel) = crate_dir_rel(manifest_path, workspace_root) else {
            continue;
        };
        crates.push(CrateInfo {
            name: name.to_string(),
            version: version.to_string(),
            dir_rel,
        });
    }
    Ok(crates)
}

/// `git diff --name-only <base_ref>...HEAD`（three-dot = merge-base 起点）で
/// 変更ファイル一覧を取得する。`base_ref` は呼び出し側（CI では
/// `origin/<base_ref>`）が事前に fetch 済みであることを前提とする。
pub fn changed_files(base_ref: &str) -> Result<Vec<String>, CheckVersionBumpError> {
    let range = format!("{base_ref}...HEAD");
    let output = Command::new("git")
        .args(["diff", "--name-only", &range])
        .output()
        .map_err(|e| {
            CheckVersionBumpError::CommandFailed(format!("failed to run git diff: {e}"))
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CheckVersionBumpError::CommandFailed(format!(
            "git diff --name-only {range} exited with {status}: {stderr}",
            status = output.status,
        )));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .map(str::to_string)
        .filter(|s| !s.is_empty())
        .collect())
}

/// `file` が `dir_rel` クレートの「公開物・API・依存に影響する範囲」
/// （`src/**` ・ `Cargo.toml` ・ `build.rs`）の変更かを判定する。
///
/// `tests/` ・ `README.md` 等は対象外とし、誤検知（テスト追加だけで
/// バンプ要求される）を抑える。
fn is_relevant_change(file: &str, dir_rel: &str) -> bool {
    let src_prefix = format!("{dir_rel}/src/");
    let cargo_toml = format!("{dir_rel}/Cargo.toml");
    let build_rs = format!("{dir_rel}/build.rs");
    file.starts_with(&src_prefix) || file == cargo_toml || file == build_rs
}

/// `files`（変更ファイル一覧）と `crates`（公開対象クレート一覧）から、
/// 実体変更のあった公開対象クレートのみを抽出する（純粋関数、I/O なし）。
pub fn affected_crates<'a>(files: &[String], crates: &'a [CrateInfo]) -> Vec<&'a CrateInfo> {
    crates
        .iter()
        .filter(|c| files.iter().any(|f| is_relevant_change(f, &c.dir_rel)))
        .collect()
}

/// crates.io sparse index 照会結果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexLookup {
    /// クレート自体が未公開（HTTP 404）。
    NotPublished,
    /// 公開済み。既公開バージョン文字列一覧（yank 済みも含む。crates.io は
    /// yank 済みバージョンの再公開も拒否するため「使用済み」として扱う）。
    Published(Vec<String>),
}

/// curl の疎通確認（`curl --version`）。存在しない・実行不能なら `false`。
fn curl_available() -> bool {
    Command::new("curl")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// crates.io sparse index の規約に従い、クレート名からインデックスパスを
/// 算出する（`.github/workflows/release.yml` の既公開バージョン検証ステップと
/// 同一規約。本モジュールへ移植し、ユニットテストで固定する）。
///
/// - 1〜2 文字: `<len>/<name>`
/// - 3 文字: `3/<先頭1文字>/<name>`
/// - 4 文字以上: `<先頭2文字>/<次2文字>/<name>`
pub fn index_path(name: &str) -> String {
    let len = name.chars().count();
    if len <= 2 {
        format!("{len}/{name}")
    } else if len == 3 {
        let first: String = name.chars().take(1).collect();
        format!("3/{first}/{name}")
    } else {
        let prefix1: String = name.chars().take(2).collect();
        let prefix2: String = name.chars().skip(2).take(2).collect();
        format!("{prefix1}/{prefix2}/{name}")
    }
}

/// crates.io sparse index の応答本文（改行区切り JSON、各行が 1 バージョンに
/// 対応）から `vers` フィールドの値を抽出する。パース不能な行は無視する
/// （sparse index はレジストリ側が生成する信頼済みだが外部由来の入力であり、
/// `json::parse` は panic しない設計、`json.rs` 参照）。
pub fn extract_versions(body: &str) -> Vec<String> {
    let mut versions = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(json) = parse(trimmed) {
            if let Some(vers) = json.get("vers").and_then(Json::as_str) {
                versions.push(vers.to_string());
            }
        }
    }
    versions
}

/// crates.io sparse index（`base_url`）を照会し、`crate_name` の既公開
/// バージョン一覧を取得する。
///
/// curl 不在・curl 自体の非 0 終了（ネットワーク不達等）・想定外 HTTP
/// status（200 系・404 以外）はすべて
/// [`CheckVersionBumpError::EnvironmentError`] として fail-closed に返す
/// （`release.yml` の既公開バージョン検証ステップと同じ判定順序: まず
/// curl 存在チェック → `-f` なし + `-w '%{http_code}'` で HTTP status と
/// curl 自体の終了コードを取り違えない）。
///
/// レスポンス本文と HTTP status を単一の curl 呼び出しで取得するため、
/// stdout の末尾に `\n%{http_code}` を追記させ、最後の改行より後ろを
/// status 行として分離する（sparse index の応答は改行区切り JSON のため、
/// 本文自体が末尾に生の 3 桁の数字だけの行を持つことはない）。
///
/// `--connect-timeout`/`--max-time` を指定し、`index.crates.io` への接続・
/// 応答がハングした場合でも self-hosted runner を無期限に占有しない
/// （イシュー #638 PR #647 レビュー指摘。ジョブ側の `timeout-minutes` と
/// 二重の安全網を構成する）。
pub fn query_index(base_url: &str, crate_name: &str) -> Result<IndexLookup, CheckVersionBumpError> {
    if !curl_available() {
        return Err(CheckVersionBumpError::EnvironmentError(
            "curl is not available on this runner. Install curl or use a runner image with curl preinstalled.".to_string(),
        ));
    }

    let url = format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        index_path(crate_name)
    );
    let output = Command::new("curl")
        .args([
            "-sS",
            "--connect-timeout",
            "10",
            "--max-time",
            "30",
            "-w",
            "\n%{http_code}",
        ])
        .arg(&url)
        .output()
        .map_err(|e| {
            CheckVersionBumpError::EnvironmentError(format!(
                "failed to invoke curl while fetching {url}: {e}"
            ))
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CheckVersionBumpError::EnvironmentError(format!(
            "curl exited with {status} while fetching {url} (network unreachable or timed out?): {stderr}",
            status = output.status,
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let Some((body, status)) = stdout.rsplit_once('\n') else {
        return Err(CheckVersionBumpError::EnvironmentError(format!(
            "unexpected curl output while fetching {url} (missing trailing HTTP status line)"
        )));
    };

    if status == "404" {
        return Ok(IndexLookup::NotPublished);
    }
    if status.len() == 3 && status.starts_with('2') {
        let versions = extract_versions(body);
        // HTTP 200 系はクレートが存在する場合にのみ返るため、本来は最低 1
        // バージョン行を含むはずである。0 件（空 body・全行パース不能等）は
        // 「レジストリ側の異常応答」を示す信号であり、これを `Published([])`
        // （→ `judge` で Pass 扱い）として通してしまうと、実際にはバンプ漏れの
        // ある PR が index 応答の欠損に紛れて fail-open してしまう
        // （イシュー #638 PR #647 レビュー指摘）。fail-closed の原則に従い
        // environment error として扱い、判定を打ち切る。
        if versions.is_empty() {
            return Err(CheckVersionBumpError::EnvironmentError(format!(
                "empty or unparseable sparse index response for `{crate_name}` despite HTTP {status} ({url}); cannot determine published versions"
            )));
        }
        return Ok(IndexLookup::Published(versions));
    }
    Err(CheckVersionBumpError::EnvironmentError(format!(
        "unexpected HTTP status `{status}` from crates.io sparse index ({url})"
    )))
}

/// PR 本文から `version-bump-exempt: <crate-name>` 宣言を抽出する。
///
/// 各行の先頭（前後の空白は無視）が [`EXEMPT_MARKER`] で始まる場合のみ、
/// マーカー直後の最初のトークンをクレート名として登録する。「同一行に
/// 理由を続けて書く」運用（例: `version-bump-exempt: foo (docs のみ)`）を
/// 想定し、2 番目以降のトークンは無視する。**クレート名の完全一致のみ**を
/// 免除対象とし、マーカーだけの行・トークンなしの行は何も免除しない
/// （包括免除経路を作らない、security.md A05）。
pub fn parse_exempt_crates(body: &str) -> HashSet<String> {
    let mut exempt = HashSet::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(EXEMPT_MARKER) {
            if let Some(name) = rest.split_whitespace().next() {
                exempt.insert(name.to_string());
            }
        }
    }
    exempt
}

/// クレート 1 件分の判定結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Judgement {
    /// バンプ済み、または未公開（判定不要）。
    Pass,
    /// 公開済みバージョンのまま実体が変更されている（バンプ漏れ）。
    Fail,
    /// PR 本文の宣言により免除。
    Exempt,
}

impl Judgement {
    /// CI の合否判定に使う: `Exempt` も PASS 側として扱う。
    pub fn is_pass(self) -> bool {
        matches!(self, Judgement::Pass | Judgement::Exempt)
    }
}

/// 1 クレート分の version・免除有無・照会結果から判定を下す（純粋関数）。
pub fn judge(version: &str, exempt: bool, lookup: &IndexLookup) -> Judgement {
    if exempt {
        return Judgement::Exempt;
    }
    match lookup {
        IndexLookup::NotPublished => Judgement::Pass,
        IndexLookup::Published(versions) => {
            if versions.iter().any(|v| v == version) {
                Judgement::Fail
            } else {
                Judgement::Pass
            }
        }
    }
}

/// stdout への 1 行サマリ出力用のレポート。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub name: String,
    pub version: String,
    /// 照会時点で crates.io に 1 バージョン以上公開済みだったか
    /// （`version` 自体が既公開かどうかとは独立。表示用の付随情報）。
    pub published: bool,
    pub judgement: Judgement,
}

/// `version-bump-check: crate=<name> version=<v> published=<yes|no> result=<PASS|FAIL|EXEMPT>`
/// の 1 行サマリを整形する。`grep '^version-bump-check:'` で CI アノテーション
/// 生成側が抽出できる契約（`check_deps::format_report` と同じ設計）。
pub fn format_report(r: &Report) -> String {
    let published = if r.published { "yes" } else { "no" };
    let result = match r.judgement {
        Judgement::Pass => "PASS",
        Judgement::Fail => "FAIL",
        Judgement::Exempt => "EXEMPT",
    };
    format!(
        "version-bump-check: crate={} version={} published={published} result={result}\n",
        r.name, r.version,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_dir_rel_strips_workspace_root_and_manifest_suffix() {
        assert_eq!(
            crate_dir_rel("/repo/crates/core/Cargo.toml", "/repo"),
            Some("crates/core".to_string())
        );
        assert_eq!(crate_dir_rel("/other/Cargo.toml", "/repo"), None);
        assert_eq!(crate_dir_rel("/repo/crates/core/lib.rs", "/repo"), None);
    }

    #[test]
    fn index_path_follows_sparse_index_sharding_rules() {
        assert_eq!(index_path("a"), "1/a");
        assert_eq!(index_path("ab"), "2/ab");
        assert_eq!(index_path("abc"), "3/a/abc");
        assert_eq!(index_path("abcd"), "ab/cd/abcd");
        assert_eq!(
            index_path("fandhe-frontend-core"),
            "fa/nd/fandhe-frontend-core"
        );
    }

    #[test]
    fn extract_versions_reads_vers_field_per_ndjson_line() {
        let body = "{\"name\":\"foo\",\"vers\":\"0.1.0\"}\n{\"name\":\"foo\",\"vers\":\"0.2.0\",\"yanked\":true}\n";
        assert_eq!(
            extract_versions(body),
            vec!["0.1.0".to_string(), "0.2.0".to_string()]
        );
    }

    #[test]
    fn extract_versions_ignores_unparseable_lines() {
        let body = "not json\n{\"vers\":\"0.1.0\"}\n\n";
        assert_eq!(extract_versions(body), vec!["0.1.0".to_string()]);
    }

    #[test]
    fn is_relevant_change_matches_src_cargo_toml_and_build_rs_only() {
        assert!(is_relevant_change("crates/core/src/lib.rs", "crates/core"));
        assert!(is_relevant_change("crates/core/Cargo.toml", "crates/core"));
        assert!(is_relevant_change("crates/core/build.rs", "crates/core"));
        assert!(!is_relevant_change(
            "crates/core/tests/foo.rs",
            "crates/core"
        ));
        assert!(!is_relevant_change("crates/core/README.md", "crates/core"));
        // 別クレートの src 変更は対象外
        assert!(!is_relevant_change(
            "crates/interactive/src/lib.rs",
            "crates/core"
        ));
    }

    #[test]
    fn affected_crates_filters_by_relevant_change() {
        let crates = vec![
            CrateInfo {
                name: "a".to_string(),
                version: "0.1.0".to_string(),
                dir_rel: "crates/a".to_string(),
            },
            CrateInfo {
                name: "b".to_string(),
                version: "0.1.0".to_string(),
                dir_rel: "crates/b".to_string(),
            },
        ];
        let files = vec!["crates/a/src/lib.rs".to_string(), "README.md".to_string()];
        let affected = affected_crates(&files, &crates);
        assert_eq!(affected.len(), 1);
        assert_eq!(affected[0].name, "a");
    }

    #[test]
    fn parse_exempt_crates_requires_exact_marker_and_first_token() {
        let body =
            "some text\nversion-bump-exempt: fandhe-frontend-core docs のみの変更\nother line";
        let exempt = parse_exempt_crates(body);
        assert!(exempt.contains("fandhe-frontend-core"));
        assert_eq!(exempt.len(), 1);
    }

    #[test]
    fn parse_exempt_crates_ignores_marker_without_crate_name() {
        let body = "version-bump-exempt:\n";
        assert!(parse_exempt_crates(body).is_empty());
    }

    #[test]
    fn judge_prioritizes_exempt_over_published_check() {
        let lookup = IndexLookup::Published(vec!["0.1.0".to_string()]);
        assert_eq!(judge("0.1.0", true, &lookup), Judgement::Exempt);
    }

    #[test]
    fn judge_fails_when_version_already_published() {
        let lookup = IndexLookup::Published(vec!["0.1.0".to_string(), "0.2.0".to_string()]);
        assert_eq!(judge("0.1.0", false, &lookup), Judgement::Fail);
    }

    #[test]
    fn judge_passes_when_version_not_yet_published() {
        let lookup = IndexLookup::Published(vec!["0.1.0".to_string()]);
        assert_eq!(judge("0.2.0", false, &lookup), Judgement::Pass);
    }

    #[test]
    fn judge_passes_when_crate_not_published_at_all() {
        assert_eq!(
            judge("0.1.0", false, &IndexLookup::NotPublished),
            Judgement::Pass
        );
    }

    #[test]
    fn format_report_matches_grep_prefix_contract() {
        let report = Report {
            name: "fandhe-frontend-core".to_string(),
            version: "0.1.0".to_string(),
            published: true,
            judgement: Judgement::Fail,
        };
        let line = format_report(&report);
        assert!(line.starts_with("version-bump-check: "));
        assert!(line.contains("crate=fandhe-frontend-core"));
        assert!(line.contains("version=0.1.0"));
        assert!(line.contains("published=yes"));
        assert!(line.contains("result=FAIL"));
    }
}
