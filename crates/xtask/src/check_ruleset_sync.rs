//! イシュー #2325: ruleset `main-protection`（GitHub branch protection の
//! `required_status_checks`）の実 GitHub API 状態と、ワークスペースが持つ
//! 正のマニフェスト（`.github/required-status-checks.json`）が一致している
//! ことを fail-closed に検知するゲート。
//!
//! ## 背景
//!
//! implement-issue-tree の autoMerge G0 ゲートは、PR HEAD sha 上の
//! check-run/commit status のうち `required_status_checks` に含まれない
//! context が 1 件でもあれば辞退する（client-only チェックの検出）。この
//! ため ruleset は「PR で報告される全 context」を個別列挙する必要があり、
//! `.claude/rules/ci.md` は当初想定していた「`ci-complete` 等 3 件へ
//! 集約する」記述と実態が乖離していた（イシュー #2325）。本モジュールは
//! その乖離を再発させないための機械検知の第 2 層（ワークフロー YAML との
//! 整合は [`crate::check_ruleset_sync`] ではなく
//! `xtask/tests/workflow_required_checks_manifest.rs` が担う。本モジュールは
//! 「マニフェスト ⇔ live ruleset」の一致のみを見る）。
//!
//! ## 判定フロー
//!
//! 1. [`parse_manifest`] でマニフェスト（`.github/required-status-checks.json`）を
//!    `{context, integration_id}` の集合として読む（`crates/xtask` 自身が
//!    `gh api` 出力から生成する、手書きしないファイル）。
//! 2. [`fetch_branch_rules`] で GitHub REST API
//!    `GET /repos/{repo}/rules/branches/{branch}` を curl で取得する
//!    （当該ブランチに適用される全 ruleset の rule を返すエンドポイントで
//!    あり、ruleset id を決め打ちにしない）。
//! 3. [`live_required_status_checks`] でレスポンス配列から
//!    `type == "required_status_checks"` の rule を全て集め、
//!    `parameters.required_status_checks[]` を union した
//!    `{context, integration_id}` 集合と、`strict_required_status_checks_policy`
//!    の観測値（いずれかの rule が `true` を持てば `true`）を得る。
//! 4. [`compare`] でマニフェストと live の集合を比較し、context ごとに
//!    PASS / `missing-in-ruleset`（マニフェストにあるが live にない） /
//!    `extra-in-ruleset`（live にあるがマニフェストにない） /
//!    `integration-id-mismatch`（両方にあるが `integration_id` が異なる、
//!    または live 側が数値でない） /
//!    `duplicate-integration-id-conflict`（live 側に同一 context が異なる
//!    `integration_id` で複数存在し、どの値が有効か本関数からは判別
//!    不能。イシュー #2325 codex-review 指摘）を判定する。
//!
//! ## fail-closed 契約
//!
//! curl 不在・curl 自体の非 0 終了・想定外 HTTP status・JSON パース不能・
//! `required_status_checks` 型の rule が 0 件（＝branch protection 自体が
//! 外れている異常状態）はすべて [`CheckRulesetSyncError::EnvironmentError`]
//! （`"environment error: "` プレフィックス、`check_version_bump` と同型の
//! 判定順序）として返す。マニフェストファイル自体の読み込み・パース失敗は
//! （リポジトリ自身が管理するファイルのため）環境要因ではなく
//! [`CheckRulesetSyncError::ManifestError`] として区別する。
//!
//! `GITHUB_TOKEN` は呼び出し元（`xtask/src/main.rs`）が環境変数から読み、
//! 存在する場合のみ curl の `-H` 引数（コマンドライン引数）として
//! `Authorization: Bearer <token>` を渡す。本モジュールはトークンの値を
//! 一切ログ・エラーメッセージへ出力しない（security.md 秘密情報の露出防止）。

use crate::json::{parse, Json};
use std::fmt;
use std::process::Command;

/// `--api-base-url` の既定値。テスト専用の差し替え口（ローカル擬似
/// サーバー）を持つ点は `check_version_bump::DEFAULT_INDEX_BASE_URL` と同型。
pub const DEFAULT_API_BASE_URL: &str = "https://api.github.com";

/// `--repo` の既定値。
pub const DEFAULT_REPO: &str = "Fandhe-AI/fandhe-frontend";

/// `--branch` の既定値。
pub const DEFAULT_BRANCH: &str = "main";

/// マニフェストの既定配置場所（ワークスペースルートからの相対パス）。
pub const DEFAULT_MANIFEST_PATH: &str = ".github/required-status-checks.json";

/// 本モジュールの操作（マニフェスト読み込み・curl 実行・GitHub API 応答解析）で
/// 発生し得るエラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckRulesetSyncError {
    /// マニフェストファイル（`.github/required-status-checks.json`）自体の
    /// 読み込み失敗・想定外の JSON 形状。リポジトリ自身が管理するファイルの
    /// 不備であり、runner 環境起因ではないため environment error とは区別する。
    ManifestError(String),
    /// curl 不在・curl 非 0 終了・想定外 HTTP status・JSON パース不能・
    /// `required_status_checks` 型 rule 0 件など、runner/GitHub API 起因の
    /// 失敗。`Display` は `"environment error: "` プレフィックスを常に含む
    /// （`check_version_bump::CheckVersionBumpError::EnvironmentError` と同型）。
    EnvironmentError(String),
}

impl fmt::Display for CheckRulesetSyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckRulesetSyncError::ManifestError(msg) => write!(f, "manifest error: {msg}"),
            CheckRulesetSyncError::EnvironmentError(msg) => write!(f, "environment error: {msg}"),
        }
    }
}

/// マニフェスト・live ruleset の 1 エントリ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckEntry {
    pub context: String,
    /// `None` は「値が存在しない、または数値でない」ことを表す
    /// （live 側で不正な応答を受け取った場合の表現。マニフェスト側は
    /// [`parse_manifest`] の時点で数値必須として弾くため常に `Some`）。
    pub integration_id: Option<i64>,
}

/// [`parse_manifest`] が返すマニフェスト全体。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub strict_required_status_checks_policy: bool,
    pub checks: Vec<CheckEntry>,
}

/// マニフェスト（`.github/required-status-checks.json`）の JSON 本文を
/// パースする。`ruleset` フィールドはコメント用途のみで判定には使わない。
///
/// `required_status_checks` 配下の各要素は `context`（文字列）・
/// `integration_id`（数値）を両方持つことを要求する（欠落・型不一致は
/// [`CheckRulesetSyncError::ManifestError`]、手書き改変によるドリフトを
/// 早期に検知する）。
pub fn parse_manifest(body: &str) -> Result<Manifest, CheckRulesetSyncError> {
    let top = parse(body)
        .map_err(|e| CheckRulesetSyncError::ManifestError(format!("invalid JSON: {e}")))?;
    let strict = top
        .get("strict_required_status_checks_policy")
        .and_then(Json::as_bool)
        .ok_or_else(|| {
            CheckRulesetSyncError::ManifestError(
                "missing or non-boolean `strict_required_status_checks_policy`".to_string(),
            )
        })?;
    let raw_checks = top
        .get("required_status_checks")
        .and_then(Json::as_array)
        .ok_or_else(|| {
            CheckRulesetSyncError::ManifestError(
                "missing or non-array `required_status_checks`".to_string(),
            )
        })?;
    let mut checks = Vec::with_capacity(raw_checks.len());
    for (i, entry) in raw_checks.iter().enumerate() {
        let context = entry
            .get("context")
            .and_then(Json::as_str)
            .ok_or_else(|| {
                CheckRulesetSyncError::ManifestError(format!(
                    "required_status_checks[{i}]: missing or non-string `context`"
                ))
            })?
            .to_string();
        let integration_id = entry
            .get("integration_id")
            .and_then(Json::as_f64)
            .ok_or_else(|| {
                CheckRulesetSyncError::ManifestError(format!(
                    "required_status_checks[{i}]: missing or non-numeric `integration_id`"
                ))
            })? as i64;
        checks.push(CheckEntry {
            context,
            integration_id: Some(integration_id),
        });
    }
    if checks.is_empty() {
        return Err(CheckRulesetSyncError::ManifestError(
            "required_status_checks is empty".to_string(),
        ));
    }
    let mut seen = std::collections::HashSet::new();
    for c in &checks {
        if !seen.insert(c.context.clone()) {
            return Err(CheckRulesetSyncError::ManifestError(format!(
                "duplicate context in manifest: `{}`",
                c.context
            )));
        }
    }
    Ok(Manifest {
        strict_required_status_checks_policy: strict,
        checks,
    })
}

/// curl の疎通確認（`curl --version`）。存在しない・実行不能なら `false`
/// （`check_version_bump::curl_available` と同一パターン、意図的な複製）。
fn curl_available() -> bool {
    Command::new("curl")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// `GET {api_base}/repos/{repo}/rules/branches/{branch}` を curl で取得し、
/// レスポンス本文を JSON としてパースして返す。
///
/// `token` を渡すと `Authorization: Bearer <token>` ヘッダを curl の
/// コマンドライン引数として付与する（値はプロセス起動時の引数としてのみ
/// 使われ、stdout・stderr・エラーメッセージには一切出力しない）。
///
/// curl 不在・curl 自体の非 0 終了（ネットワーク不達等）・想定外 HTTP
/// status（200 系以外）・JSON パース不能（トップレベルが配列でない場合を
/// 含む）はすべて [`CheckRulesetSyncError::EnvironmentError`] として
/// fail-closed に返す（`check_version_bump::query_index` と同じ判定順序）。
pub fn fetch_branch_rules(
    api_base_url: &str,
    repo: &str,
    branch: &str,
    token: Option<&str>,
) -> Result<Json, CheckRulesetSyncError> {
    if !curl_available() {
        return Err(CheckRulesetSyncError::EnvironmentError(
            "curl is not available on this runner. Install curl or use a runner image with curl preinstalled.".to_string(),
        ));
    }

    let url = format!(
        "{}/repos/{repo}/rules/branches/{branch}",
        api_base_url.trim_end_matches('/')
    );
    let mut command = Command::new("curl");
    command.args([
        "-sS",
        "--connect-timeout",
        "10",
        "--max-time",
        "30",
        "-w",
        "\n%{http_code}",
    ]);
    if let Some(t) = token {
        command.arg("-H").arg(format!("Authorization: Bearer {t}"));
    }
    command.arg(&url);

    let output = command.output().map_err(|e| {
        CheckRulesetSyncError::EnvironmentError(format!(
            "failed to invoke curl while fetching {url}: {e}"
        ))
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CheckRulesetSyncError::EnvironmentError(format!(
            "curl exited with {status} while fetching {url} (network unreachable or timed out?): {stderr}",
            status = output.status,
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let Some((body, status)) = stdout.rsplit_once('\n') else {
        return Err(CheckRulesetSyncError::EnvironmentError(format!(
            "unexpected curl output while fetching {url} (missing trailing HTTP status line)"
        )));
    };

    if !(status.len() == 3 && status.starts_with('2')) {
        return Err(CheckRulesetSyncError::EnvironmentError(format!(
            "unexpected HTTP status `{status}` from GitHub API ({url})"
        )));
    }

    parse(body).map_err(|e| {
        CheckRulesetSyncError::EnvironmentError(format!(
            "failed to parse GitHub API response from {url} as JSON: {e}"
        ))
    })
}

/// live ruleset から `{context, integration_id}` の集合と、観測された
/// `strict_required_status_checks_policy`（いずれかの rule が `true` を
/// 持てば `true`）を得る。
///
/// `type == "required_status_checks"` の rule が 1 件も無い場合
/// （branch protection 自体が外れている異常状態）は
/// [`CheckRulesetSyncError::EnvironmentError`] とする（マニフェストとの
/// 比較上は「全件 missing」になり得るが、その前に環境異常として打ち切る）。
pub fn live_required_status_checks(
    rules: &Json,
) -> Result<(Vec<CheckEntry>, bool), CheckRulesetSyncError> {
    let rules_array = rules.as_array().ok_or_else(|| {
        CheckRulesetSyncError::EnvironmentError(
            "unexpected GitHub API response shape: top-level value is not an array".to_string(),
        )
    })?;

    let mut checks = Vec::new();
    let mut strict_observed = false;
    let mut rule_count = 0usize;

    for rule in rules_array {
        if rule.get("type").and_then(Json::as_str) != Some("required_status_checks") {
            continue;
        }
        rule_count += 1;
        let parameters = rule.get("parameters").ok_or_else(|| {
            CheckRulesetSyncError::EnvironmentError(
                "required_status_checks rule missing `parameters`".to_string(),
            )
        })?;
        if parameters
            .get("strict_required_status_checks_policy")
            .and_then(Json::as_bool)
            == Some(true)
        {
            strict_observed = true;
        }
        let entries = parameters
            .get("required_status_checks")
            .and_then(Json::as_array)
            .ok_or_else(|| {
                CheckRulesetSyncError::EnvironmentError(
                    "required_status_checks rule missing `parameters.required_status_checks`"
                        .to_string(),
                )
            })?;
        for entry in entries {
            let context = entry.get("context").and_then(Json::as_str).ok_or_else(|| {
                CheckRulesetSyncError::EnvironmentError(
                    "required_status_checks entry missing or non-string `context`".to_string(),
                )
            })?;
            // `integration_id` が数値でない・欠落している場合は環境異常として
            // 打ち切らず、比較フェーズで `integration-id-mismatch` として
            // 検知させる（本モジュール冒頭 doc コメント参照）。
            let integration_id = entry
                .get("integration_id")
                .and_then(Json::as_f64)
                .map(|n| n as i64);
            checks.push(CheckEntry {
                context: context.to_string(),
                integration_id,
            });
        }
    }

    if rule_count == 0 {
        return Err(CheckRulesetSyncError::EnvironmentError(
            "no required_status_checks rule found on the target branch (branch protection missing?)"
                .to_string(),
        ));
    }

    Ok((checks, strict_observed))
}

/// context ごとの比較結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryJudgement {
    Pass,
    MissingInRuleset,
    ExtraInRuleset,
    IntegrationIdMismatch,
    /// live 側に同一 context が異なる `integration_id` で複数存在する
    /// （別 ruleset・別 App が同じ context 名を異なる App ID で登録して
    /// いる状態）。`compare` はこれを単純な先勝ちマージで握り潰さず、
    /// マニフェストとの一致判定より優先して FAIL にする（イシュー #2325
    /// codex-review 指摘: manifest/live が共に (ci-complete, 15368) を
    /// 持つ場合でも live にもう 1 件 (ci-complete, 9999) があれば不一致
    /// として検知する）。
    DuplicateIntegrationIdConflict,
}

impl EntryJudgement {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntryJudgement::Pass => "PASS",
            EntryJudgement::MissingInRuleset => "FAIL:missing-in-ruleset",
            EntryJudgement::ExtraInRuleset => "FAIL:extra-in-ruleset",
            EntryJudgement::IntegrationIdMismatch => "FAIL:integration-id-mismatch",
            EntryJudgement::DuplicateIntegrationIdConflict => {
                "FAIL:duplicate-integration-id-conflict"
            }
        }
    }

    pub fn is_pass(&self) -> bool {
        matches!(self, EntryJudgement::Pass)
    }
}

/// 1 行サマリ 1 件分。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryReport {
    pub context: String,
    /// live 側の値があればそれを、なければマニフェスト側の値を報告に使う
    /// （どちらの集合にも属さない状態は起こらない: union で走査するため）。
    pub integration_id: Option<i64>,
    pub judgement: EntryJudgement,
}

/// マニフェストと live ruleset の union を context 単位で比較する。
///
/// マニフェスト側に重複 context が無いことは呼び出し前提とする
/// （[`parse_manifest`] が拒否する）。**live 側は重複 context を先勝ちで
/// 握り潰さない**: 複数の rule（別 ruleset・別 App）が同一 context を
/// 異なる `integration_id` で報告した場合、`compare` はその全エントリを
/// 保持して不一致を検知する（イシュー #2325 codex-review 指摘。GitHub API
/// が同一 context を複数の rule で重複して返すことは実際に起こり得るため、
/// 「想定しない」として先勝ちマージするのは fail-open だった）。
pub fn compare(manifest: &Manifest, live: &[CheckEntry]) -> Vec<EntryReport> {
    use std::collections::HashMap;

    let manifest_map: HashMap<&str, i64> = manifest
        .checks
        .iter()
        .map(|c| (c.context.as_str(), c.integration_id.unwrap_or(0)))
        .collect();
    // context ごとに観測された全 integration_id を保持する（先勝ちで
    // 1 件へ潰さない）。同一 context に異なる値が複数観測された場合は
    // `DuplicateIntegrationIdConflict` として扱う。
    let mut live_map: HashMap<&str, Vec<Option<i64>>> = HashMap::new();
    for c in live {
        live_map
            .entry(c.context.as_str())
            .or_default()
            .push(c.integration_id);
    }

    let mut contexts: Vec<&str> = manifest_map
        .keys()
        .chain(live_map.keys())
        .copied()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    contexts.sort_unstable();

    contexts
        .into_iter()
        .map(|context| {
            let manifest_id = manifest_map.get(context).copied();
            let live_ids = live_map.get(context);
            // live 側で観測された integration_id の重複なし集合。2 件以上の
            // 異なる値があれば、当該 context の live 状態自体が破損している
            // （どの ID が実際に有効な ruleset なのか本関数からは判別不能）。
            let distinct_live_ids: Option<Vec<Option<i64>>> = live_ids.map(|ids| {
                let mut uniq: Vec<Option<i64>> = Vec::new();
                for id in ids {
                    if !uniq.contains(id) {
                        uniq.push(*id);
                    }
                }
                uniq
            });
            let live_present = live_ids.is_some();
            let live_id =
                distinct_live_ids.as_ref().and_then(
                    |ids| {
                        if ids.len() == 1 {
                            ids[0]
                        } else {
                            None
                        }
                    },
                );
            let (judgement, reported_id) = if distinct_live_ids
                .as_ref()
                .is_some_and(|ids| ids.len() > 1)
            {
                (EntryJudgement::DuplicateIntegrationIdConflict, live_id)
            } else {
                match (manifest_id, live_present) {
                    (Some(m), true) => {
                        if Some(m) == live_id {
                            (EntryJudgement::Pass, live_id)
                        } else {
                            (EntryJudgement::IntegrationIdMismatch, live_id)
                        }
                    }
                    (Some(_), false) => (EntryJudgement::MissingInRuleset, manifest_id),
                    (None, true) => (EntryJudgement::ExtraInRuleset, live_id),
                    (None, false) => {
                        unreachable!("context はマニフェストか live のいずれかから収集したもののみ")
                    }
                }
            };
            EntryReport {
                context: context.to_string(),
                integration_id: reported_id,
                judgement,
            }
        })
        .collect()
}

/// ログ・Step Summary への出力時に制御文字（改行・タブ等）を Rust の
/// `\u{XXXX}` 表記へエスケープする。context 文字列は GitHub API・
/// 外部 App（Cursor Bugbot 等）が提供する値であり、本モジュールのログ
/// 出力経路がそれらを無検査でターミナル制御シーケンスとして解釈させない
/// ための防御的処理（security.md A03、ログ経由の注入面を作らない）。
pub fn escape_control_chars(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if ch.is_control() {
            out.push_str(&format!("\\u{{{:04x}}}", ch as u32));
        } else {
            out.push(ch);
        }
    }
    out
}

/// 1 エントリ分の 1 行サマリ。
///
/// 契約: `ruleset-sync: context=<context> integration_id=<n> result=<PASS|FAIL:...>`
pub fn format_entry_line(report: &EntryReport) -> String {
    format!(
        "ruleset-sync: context={} integration_id={} result={}\n",
        escape_control_chars(&report.context),
        report
            .integration_id
            .map(|n| n.to_string())
            .unwrap_or_else(|| "null".to_string()),
        report.judgement.as_str()
    )
}

/// 全体サマリの末尾行。
///
/// 契約: `ruleset-sync: strict=<bool> result=<PASS|FAIL>`
pub fn format_strict_line(strict_observed: bool) -> String {
    format!(
        "ruleset-sync: strict={strict_observed} result={}\n",
        if strict_observed { "FAIL" } else { "PASS" }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(context: &str, integration_id: i64) -> CheckEntry {
        CheckEntry {
            context: context.to_string(),
            integration_id: Some(integration_id),
        }
    }

    #[test]
    fn parse_manifest_accepts_well_formed_body() {
        let body = r#"{
            "ruleset": "main-protection",
            "strict_required_status_checks_policy": false,
            "required_status_checks": [
                {"context": "ci-complete", "integration_id": 15368},
                {"context": "Cursor Bugbot", "integration_id": 1210556}
            ]
        }"#;
        let manifest = parse_manifest(body).expect("well-formed body must parse");
        assert!(!manifest.strict_required_status_checks_policy);
        assert_eq!(manifest.checks.len(), 2);
    }

    #[test]
    fn parse_manifest_rejects_duplicate_context() {
        let body = r#"{
            "strict_required_status_checks_policy": false,
            "required_status_checks": [
                {"context": "ci-complete", "integration_id": 15368},
                {"context": "ci-complete", "integration_id": 15368}
            ]
        }"#;
        assert!(matches!(
            parse_manifest(body),
            Err(CheckRulesetSyncError::ManifestError(_))
        ));
    }

    #[test]
    fn parse_manifest_rejects_non_numeric_integration_id() {
        let body = r#"{
            "strict_required_status_checks_policy": false,
            "required_status_checks": [
                {"context": "ci-complete", "integration_id": "15368"}
            ]
        }"#;
        assert!(matches!(
            parse_manifest(body),
            Err(CheckRulesetSyncError::ManifestError(_))
        ));
    }

    #[test]
    fn parse_manifest_rejects_missing_strict_flag() {
        let body = r#"{
            "required_status_checks": [
                {"context": "ci-complete", "integration_id": 15368}
            ]
        }"#;
        assert!(matches!(
            parse_manifest(body),
            Err(CheckRulesetSyncError::ManifestError(_))
        ));
    }

    #[test]
    fn compare_reports_pass_when_identical() {
        let manifest = Manifest {
            strict_required_status_checks_policy: false,
            checks: vec![entry("ci-complete", 15368)],
        };
        let live = vec![entry("ci-complete", 15368)];
        let reports = compare(&manifest, &live);
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].judgement, EntryJudgement::Pass);
    }

    #[test]
    fn compare_reports_missing_in_ruleset() {
        let manifest = Manifest {
            strict_required_status_checks_policy: false,
            checks: vec![entry("ci-complete", 15368)],
        };
        let live: Vec<CheckEntry> = vec![];
        let reports = compare(&manifest, &live);
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].judgement, EntryJudgement::MissingInRuleset);
    }

    #[test]
    fn compare_reports_extra_in_ruleset() {
        let manifest = Manifest {
            strict_required_status_checks_policy: false,
            checks: vec![],
        };
        let live = vec![entry("new-job", 15368)];
        // マニフェストが空は parse_manifest では拒否されるが、compare 自体は
        // 空集合でも壊れないことを確認する（境界値）。
        let reports = compare(&manifest, &live);
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].judgement, EntryJudgement::ExtraInRuleset);
    }

    #[test]
    fn compare_reports_duplicate_integration_id_conflict_even_if_one_matches_manifest() {
        // イシュー #2325 codex-review 指摘の再現: manifest/live が共に
        // (ci-complete, 15368) を持つため先勝ちマージでは PASS になって
        // しまうが、live 側にもう 1 件 (ci-complete, 9999) が別 rule から
        // 観測される場合は不一致として検知しなければならない。
        let manifest = Manifest {
            strict_required_status_checks_policy: false,
            checks: vec![entry("ci-complete", 15368)],
        };
        let live = vec![entry("ci-complete", 15368), entry("ci-complete", 9999)];
        let reports = compare(&manifest, &live);
        assert_eq!(reports.len(), 1);
        assert_eq!(
            reports[0].judgement,
            EntryJudgement::DuplicateIntegrationIdConflict
        );
    }

    #[test]
    fn compare_reports_integration_id_mismatch() {
        let manifest = Manifest {
            strict_required_status_checks_policy: false,
            checks: vec![entry("ci-complete", 15368)],
        };
        let live = vec![entry("ci-complete", 9999)];
        let reports = compare(&manifest, &live);
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].judgement, EntryJudgement::IntegrationIdMismatch);
    }

    #[test]
    fn compare_reports_mismatch_when_live_integration_id_not_numeric() {
        let manifest = Manifest {
            strict_required_status_checks_policy: false,
            checks: vec![entry("ci-complete", 15368)],
        };
        let live = vec![CheckEntry {
            context: "ci-complete".to_string(),
            integration_id: None,
        }];
        let reports = compare(&manifest, &live);
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].judgement, EntryJudgement::IntegrationIdMismatch);
    }

    #[test]
    fn live_required_status_checks_errors_when_no_rule_present() {
        let rules = parse("[]").unwrap();
        assert!(matches!(
            live_required_status_checks(&rules),
            Err(CheckRulesetSyncError::EnvironmentError(_))
        ));
    }

    #[test]
    fn live_required_status_checks_unions_multiple_rules() {
        let body = r#"[
            {"type": "deletion"},
            {
                "type": "required_status_checks",
                "parameters": {
                    "strict_required_status_checks_policy": false,
                    "required_status_checks": [{"context": "a", "integration_id": 1}]
                }
            },
            {
                "type": "required_status_checks",
                "parameters": {
                    "strict_required_status_checks_policy": true,
                    "required_status_checks": [{"context": "b", "integration_id": 2}]
                }
            }
        ]"#;
        let rules = parse(body).unwrap();
        let (checks, strict_observed) = live_required_status_checks(&rules).unwrap();
        assert_eq!(checks.len(), 2);
        assert!(
            strict_observed,
            "いずれかの rule が true なら observed は true"
        );
    }

    #[test]
    fn escape_control_chars_escapes_newline() {
        assert_eq!(escape_control_chars("a\nb"), "a\\u{000a}b");
    }

    #[test]
    fn format_entry_line_matches_contract() {
        let report = EntryReport {
            context: "ci-complete".to_string(),
            integration_id: Some(15368),
            judgement: EntryJudgement::Pass,
        };
        assert_eq!(
            format_entry_line(&report),
            "ruleset-sync: context=ci-complete integration_id=15368 result=PASS\n"
        );
    }

    #[test]
    fn format_strict_line_matches_contract() {
        assert_eq!(
            format_strict_line(false),
            "ruleset-sync: strict=false result=PASS\n"
        );
        assert_eq!(
            format_strict_line(true),
            "ruleset-sync: strict=true result=FAIL\n"
        );
    }
}
