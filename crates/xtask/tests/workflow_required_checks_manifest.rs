//! `.github/required-status-checks.json`（ruleset `main-protection` の
//! `required_status_checks` の正のマニフェスト、イシュー #2325）が
//! `.github/workflows/*.yml` の実態と一致していることをオフラインに
//! fail-closed で検知する契約テスト。
//!
//! ## 背景
//!
//! implement-issue-tree の autoMerge G0 ゲートは、PR HEAD sha 上の
//! check-run/commit status のうち ruleset `main-protection` の
//! `required_status_checks` に含まれない context が 1 件でもあれば辞退
//! する。これを満たすため ruleset は「PR で報告される全 context」を
//! 個別列挙しているが、`.claude/rules/ci.md` は当初「`ci-complete` 等
//! 3 件へ集約」という古い記述を残しており、実態と乖離していた
//! （イシュー #2325）。本テストはワークフロー YAML とマニフェストの
//! ドリフトを `cargo test -p xtask` の時点で検知する第 1 層
//! （マニフェスト ⇔ live ruleset の第 2 層は
//! `xtask check-ruleset-sync`、`crates/xtask/tests/cli_check_ruleset_sync.rs`
//! が担う）。
//!
//! ## 分類表（固定、`.github/workflows/` に新規ファイルが増えたら要更新）
//!
//! - `ci.yml` / `deps-check.yml` / `musl-smoke.yml` / `image-size.yml`:
//!   PR・push の双方に反応し HEAD sha へ check-run を報告するため、
//!   `jobs:` 直下の全トップレベルジョブの `name:`（欠落時はジョブ id）を
//!   `integration_id = 15368`（GitHub Actions app）として期待集合に含める。
//! - `codex-review.yml`: reusable workflow を呼ぶだけの wrapper で
//!   `jobs:` 直下のジョブ名は `codex-review` の 1 件のみだが、実際に
//!   HEAD へ報告されるのは呼び出し先（`Fandhe-AI/actions` の
//!   `codex-review.yml`）が生成する 3 つの子ジョブ名
//!   `codex-review / codex` / `codex-review / post_feedback` /
//!   `codex-review / skip-gate` である。ワークフロー YAML の走査では
//!   導出できないため、本ファイル内の定数
//!   [`CODEX_REVIEW_REUSABLE_WORKFLOW_CONTEXTS`] として固定する。
//! - `docs-site.yml`（`on: push` のみ、PR では起動しない）・
//!   `release.yml`（`workflow_dispatch` のみ）・`update-external.yml`
//!   （`schedule`/`workflow_dispatch` のみ）: PR HEAD には報告されない
//!   ため期待集合から除外する（required にすると永久 Expected で
//!   ブロックする異常状態になる）。
//! - `Cursor Bugbot`: ワークフロー YAML を持たない外部 GitHub App。
//!   本ファイル内の定数 [`CURSOR_BUGBOT_CONTEXT`] として固定する。
//!
//! 上記分類表に無い `.github/workflows/*.yml` が新規追加された場合は
//! FAIL する（[`classify_workflow_file`] が `None` を返す一覧を
//! アサーションで検知する）。新しい workflow を「分類不要」として
//! 黙って対象外にする経路は存在しない（fail-closed、新設時に必ず本表への
//! 追記を要求する）。
//!
//! ## 反転判定・外部 YAML パーサ不採用
//!
//! `workflow_ci_complete_needs.rs` / `workflow_runner_policy.rs` と同じ
//! 流儀: 唯一の正規形（`  <job-id>:`、クォート無しキー・インデント
//! ちょうど 2）に一致しない表記（クォート付きキー・flow mapping 等）は
//! 認識せず、ジョブ抽出の対象外として扱う（未知表記を素通りさせる
//! fail-open にはしない）。行ベースの文字列走査に留め、外部クレートへの
//! 依存は追加しない（REQ-3・xtask 外部依存ゼロ方針）。

use std::collections::HashSet;
use std::path::PathBuf;

/// GitHub Actions app の `integration_id`（`.github/required-status-checks.json`
/// 内の全 GitHub Actions 発行エントリと同一値）。
const GITHUB_ACTIONS_INTEGRATION_ID: i64 = 15368;

/// Cursor Bugbot app の `integration_id`。
const CURSOR_BUGBOT_INTEGRATION_ID: i64 = 1210556;

/// `Cursor Bugbot` はワークフロー YAML を持たない外部 GitHub App の
/// context（本テスト冒頭 doc コメント参照）。
const CURSOR_BUGBOT_CONTEXT: &str = "Cursor Bugbot";

/// `codex-review.yml` が呼び出す reusable workflow
/// （`Fandhe-AI/actions/.github/workflows/codex-review.yml`）が生成する
/// 子ジョブ名。呼び出し元 YAML の `jobs:` 直下には `codex-review` の
/// 1 件しか現れないため、ワークフロー走査では導出できず定数として固定する
/// （本テスト冒頭 doc コメント参照）。
const CODEX_REVIEW_REUSABLE_WORKFLOW_CONTEXTS: &[&str] = &[
    "codex-review / codex",
    "codex-review / post_feedback",
    "codex-review / skip-gate",
];

/// PR HEAD に報告されないため期待集合から除外するワークフローファイル名
/// （本テスト冒頭 doc コメント参照）。
const EXCLUDED_WORKFLOW_FILES: &[&str] = &["docs-site.yml", "release.yml", "update-external.yml"];

/// `jobs:` 直下の全トップレベルジョブを列挙する（`codex-review.yml` を除く）
/// ワークフローファイル名。
const JOB_ENUMERATED_WORKFLOW_FILES: &[&str] = &[
    "ci.yml",
    "deps-check.yml",
    "musl-smoke.yml",
    "image-size.yml",
];

/// reusable workflow 呼び出しのみで、子ジョブ名を定数から補う
/// ワークフローファイル名。
const REUSABLE_WORKFLOW_FILES: &[&str] = &["codex-review.yml"];

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/xtask/ から 2 段上でワークスペースルートに到達する（イシュー #436）")
        .to_path_buf()
}

fn workflows_dir() -> PathBuf {
    workspace_root().join(".github/workflows")
}

fn manifest_path() -> PathBuf {
    workspace_root().join(".github/required-status-checks.json")
}

/// 1 行からコメント部分を切り落とす（`workflow_ci_complete_needs.rs` と
/// 同一ロジック、意図的な複製）。
fn strip_comment(line: &str) -> String {
    let trimmed_start = line.trim_start();
    if trimmed_start.starts_with('#') {
        return String::new();
    }

    let mut result = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut prev_char: Option<char> = None;

    for ch in line.chars() {
        if ch == '\'' && !in_double {
            in_single = !in_single;
            result.push(ch);
            prev_char = Some(ch);
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            result.push(ch);
            prev_char = Some(ch);
            continue;
        }
        if ch == '#' && !in_single && !in_double {
            let boundary_ok = match prev_char {
                None => true,
                Some(c) => c.is_whitespace(),
            };
            if boundary_ok {
                break;
            }
        }
        result.push(ch);
        prev_char = Some(ch);
    }

    result
}

/// `jobs:` 直下（インデント 2）のジョブ id 行かどうかを判定する
/// （元は `workflow_ci_complete_needs.rs::job_name_at_indent2` と同一
/// ロジックの意図的な複製だったが、イシュー #2325 codex-review 指摘を
/// 受けてクォート付きキーの受理・未知表記の明示拒否〔panic〕を本関数側
/// にのみ追加したため、現在はロジックが分岐している）。
fn job_id_at_indent2(stripped_line: &str) -> Option<String> {
    let trimmed_end = stripped_line.trim_end();
    if !trimmed_end.starts_with("  ") {
        return None;
    }
    if trimmed_end.as_bytes().get(2) == Some(&b' ') {
        return None;
    }
    let rest = &trimmed_end[2..];
    if rest.is_empty() {
        return None;
    }
    // `jobs:` 直下（インデント 2）の非空行は、YAML の構造上ジョブ id
    // キー以外に存在し得ない（ジョブ本文はインデント 4 以深、リスト
    // 要素は `jobs:` セクションに現れない）。`<job-id>:`（単純なブロック
    // マッピングキー）以外の表記——flow mapping（`extra: {runs-on: ...}`）
    // やインラインスカラー値付きキー等——は `strip_suffix(':')` が
    // `None` を返すが、それを黙って「ジョブ id 行ではない」として
    // 検知対象から除外すると required check 登録漏れを見逃す
    // fail-open になる（イシュー #2325 codex-review 指摘）。CI 規約
    // （.claude/rules/ci.md）の反転判定原則に従い、未対応表記は
    // 明示的にパニックで拒否する。
    let name = match rest.strip_suffix(':') {
        Some(name) => name,
        None => {
            panic!(
                "認識できないジョブ id 表記: `{trimmed_end}`。\
`jobs:` 直下（インデント 2）の行は `<job-id>:`（単純なブロックマッピング \
キー、クォート付き可）のみに対応している。flow mapping（`key: {{...}}`）や \
行内スカラー値付きキー等の新しい表記が必要な場合は本関数（job_id_at_indent2）\
を拡張すること（未知表記を黙って無視しない）。"
            );
        }
    };
    if name.is_empty() {
        return None;
    }
    // YAML の block mapping キーはクォート付き（`"job-id":`）も許容される
    // ため、英数字・`_`・`-` の判定前にクォートを剥がす（イシュー #2325
    // codex-review 指摘: クォート付きジョブ id を追加しても本関数が黙って
    // 無視し、`all_workflow_files_are_classified` 等の検知が素通りして
    // いた）。
    let unquoted = strip_matching_quotes(name);
    if unquoted.is_empty() {
        return None;
    }
    if !unquoted
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        // CI 規約（`.claude/rules/ci.md`）の「反転判定」原則: 認識できない
        // ジョブ id 表記を黙って無視して期待集合から欠落させず、パニック
        // で明示的に拒否する。
        panic!(
            "認識できないジョブ id 表記: `{trimmed_end}`。\
`jobs:` 直下（インデント 2）のキーは英数字・`_`・`-`（クォート付き可）のみ \
対応している。新しい表記が必要な場合は本関数（job_id_at_indent2）を \
拡張すること（未知表記を黙って無視しない）。"
        );
    }
    Some(unquoted.to_string())
}

/// 前後を同じ引用符で囲まれている場合のみそれを剥がす
/// （`workflow_ci_complete_needs.rs::strip_matching_quotes` と同一ロジック、
/// 意図的な複製）。
fn strip_matching_quotes(s: &str) -> &str {
    let bytes = s.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return &s[1..s.len() - 1];
        }
    }
    s
}

/// インデント `indent` の `<key>: <value>` 行を `(key, value)` として
/// 抽出する（`workflow_ci_complete_needs.rs::key_at_indent` と同一ロジック、
/// 意図的な複製）。
fn key_at_indent(stripped_line: &str, indent: usize) -> Option<(String, String)> {
    let trimmed_end = stripped_line.trim_end();
    let leading = trimmed_end.len() - trimmed_end.trim_start_matches(' ').len();
    if leading != indent {
        return None;
    }
    let rest = &trimmed_end[indent..];
    let colon_pos = rest.find(':')?;
    let key = &rest[..colon_pos];
    if key.is_empty()
        || !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return None;
    }
    let value = rest[colon_pos + 1..].trim().to_string();
    Some((key.to_string(), value))
}

/// `jobs:` セクション（行頭ちょうど `jobs:`、インデント 0）の開始行を返す。
fn find_jobs_section_start(stripped: &[String]) -> Option<usize> {
    stripped.iter().position(|line| line.trim_end() == "jobs:")
}

/// `jobs:` 直下（インデント 2）の全トップレベルジョブについて、
/// `(job_id, context)` の一覧を返す。`context` はジョブ本文直下
/// （インデント 4）に見つかった最初の `name:` の値（クォート除去済み）。
/// `name:` が見つからない場合は `job_id` そのものを `context` とする
/// （本テスト冒頭 doc コメント参照）。
fn extract_top_level_jobs(body: &str) -> Vec<(String, String)> {
    let stripped: Vec<String> = body.lines().map(strip_comment).collect();
    let Some(jobs_start) = find_jobs_section_start(&stripped) else {
        return Vec::new();
    };

    let mut jobs = Vec::new();
    let mut i = jobs_start + 1;
    while i < stripped.len() {
        if let Some(job_id) = job_id_at_indent2(&stripped[i]) {
            // 本文（次の indent2 ジョブ行、または EOF まで）を走査して
            // インデント 4 の `name:` を探す。
            let mut context = job_id.clone();
            let mut j = i + 1;
            while j < stripped.len() && job_id_at_indent2(&stripped[j]).is_none() {
                if let Some((key, value)) = key_at_indent(&stripped[j], 4) {
                    if key == "name" {
                        context = strip_matching_quotes(&value).to_string();
                        break;
                    }
                }
                j += 1;
            }
            jobs.push((job_id, context));
        }
        i += 1;
    }
    jobs
}

/// 1 ワークフローファイルの分類結果。
enum Classification {
    /// `jobs:` 直下の全トップレベルジョブを期待集合に含める
    /// （`integration_id` は常に GitHub Actions app）。
    EnumerateJobs,
    /// reusable workflow 呼び出しのみで、定数から子ジョブ名を補う。
    ReusableWorkflow(&'static [&'static str]),
    /// PR HEAD に報告されないため期待集合から除外する。
    Excluded,
}

fn classify_workflow_file(file_name: &str) -> Option<Classification> {
    if JOB_ENUMERATED_WORKFLOW_FILES.contains(&file_name) {
        return Some(Classification::EnumerateJobs);
    }
    if REUSABLE_WORKFLOW_FILES.contains(&file_name) {
        return Some(Classification::ReusableWorkflow(
            CODEX_REVIEW_REUSABLE_WORKFLOW_CONTEXTS,
        ));
    }
    if EXCLUDED_WORKFLOW_FILES.contains(&file_name) {
        return Some(Classification::Excluded);
    }
    None
}

/// `.github/workflows/*.yml`・`*.yaml` 全件と外部 App 定数から、期待される
/// `{context, integration_id}` 集合を導出する。分類表に無いファイルが
/// あれば呼び出し側でパニックさせるため、`(未分類ファイル名一覧, 期待集合)`
/// を返す。
fn derive_expected_checks() -> (Vec<String>, Vec<(String, i64)>) {
    let dir = workflows_dir();
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{:?} の読み込みに失敗した: {e}", dir))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            // GitHub Actions は `.github/workflows/` 配下の `.yml`・`.yaml`
            // いずれの拡張子もワークフローとして受理する。`yml` のみに
            // 限定すると `.yaml` で追加されたワークフローが分類表の検証
            // 対象から漏れ、未分類ファイル検知（`all_workflow_files_are_classified`）
            // を素通りしてしまう（イシュー #2325 codex-review 指摘）。
            matches!(
                p.extension().and_then(|s| s.to_str()),
                Some("yml") | Some("yaml")
            )
        })
        .collect();
    entries.sort();

    let mut unclassified = Vec::new();
    let mut expected: Vec<(String, i64)> = Vec::new();

    for path in entries {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        match classify_workflow_file(&file_name) {
            Some(Classification::EnumerateJobs) => {
                let body = std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("{:?} の読み込みに失敗した: {e}", path));
                for (_job_id, context) in extract_top_level_jobs(&body) {
                    expected.push((context, GITHUB_ACTIONS_INTEGRATION_ID));
                }
            }
            Some(Classification::ReusableWorkflow(contexts)) => {
                for c in contexts {
                    expected.push((c.to_string(), GITHUB_ACTIONS_INTEGRATION_ID));
                }
            }
            Some(Classification::Excluded) => {}
            None => unclassified.push(file_name),
        }
    }

    expected.push((
        CURSOR_BUGBOT_CONTEXT.to_string(),
        CURSOR_BUGBOT_INTEGRATION_ID,
    ));

    (unclassified, expected)
}

/// マニフェスト（`.github/required-status-checks.json`）を最小限の手書き
/// パーサで読む。`crate::check_ruleset_sync::parse_manifest`（本クレートの
/// バイナリ内部関数、統合テストからは直接呼べない）と重複するが、
/// 統合テストは公開 API 越しにバイナリを検証する構成のため独立実装とする
/// （`workflow_ci_complete_needs.rs` 等、他の xtask 契約テストも同様に
/// 自己完結スタイルを踏襲している）。
fn parse_manifest_contexts(body: &str) -> (bool, Vec<(String, i64)>) {
    // `.github/required-status-checks.json` は `xtask` 自身が
    // `gh api rulesets/... --jq '...'` の出力から生成する（手書きしない）
    // ため、整形式 JSON である前提で簡易パースする。汎用 JSON パーサへの
    // 依存は増やさない（REQ-3）。
    //
    // 本パーサは JSON エスケープ（`\uXXXX`・`\"` 等）を解釈しない素朴な
    // 文字列走査であり、`.github/required-status-checks.json` が
    // `check_ruleset_sync::parse_manifest`（本体側、`ensure_ascii=False`
    // 相当の UTF-8 生保存を前提）の生成物のままであることに依存する。
    // `python -m json.tool`（既定 ensure_ascii=True）や `jq`（既定でも
    // ASCII エスケープ）で整形し直すと非 ASCII context（日本語の context
    // 名を含む、本マニフェストには実在する）が `\uXXXX` へ変換され、
    // 「実体は同一なのに全件不一致」という誤解を招く FAIL になる
    // （デバッグ時に自分自身がこの誤りを踏んだ、イシュー #2325）。
    // 早期に区別できるメッセージで打ち切る。
    assert!(
        !body.contains("\\u") && !body.contains("\\\""),
        "マニフェストに JSON エスケープ（\\uXXXX や \\\"）が含まれている。\
`.github/required-status-checks.json` は UTF-8 生の文字列で保存すること \
（`python -m json.tool`/`jq` 等での再整形は既定で非 ASCII を \\uXXXX へ \
エスケープし、本パーサはそれを実体の不一致と誤認する）。"
    );

    let strict = find_bool_value(body, "strict_required_status_checks_policy");

    let mut contexts = Vec::new();
    for entry_block in body.split("\"context\":").skip(1) {
        let context_start = entry_block
            .find('"')
            .expect("context 値の開始 \" が見つからない");
        let rest = &entry_block[context_start + 1..];
        let context_end = rest.find('"').expect("context 値の終端 \" が見つからない");
        let context = rest[..context_end].to_string();

        let after_context = &rest[context_end + 1..];
        let id_key = after_context
            .find("\"integration_id\":")
            .expect("integration_id キーが見つからない");
        let after_id_key = &after_context[id_key + "\"integration_id\":".len()..];
        let id_str: String = after_id_key
            .trim_start()
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        let integration_id: i64 = id_str
            .parse()
            .unwrap_or_else(|_| panic!("integration_id の数値パースに失敗: `{id_str}`"));

        contexts.push((context, integration_id));
    }

    (strict, contexts)
}

/// `"<key>":` を検索し、続く空白を読み飛ばした直後が `true`/`false` の
/// どちらで始まるかを判定する（キーと `:` の間・`:` と値の間の空白幅の
/// 揺れ、および JSON 整形の改行を許容する。`strict_required_status_checks_policy`
/// の読み取り専用）。
fn find_bool_value(body: &str, key: &str) -> bool {
    let needle = format!("\"{key}\":");
    let key_pos = body
        .find(&needle)
        .unwrap_or_else(|| panic!("マニフェストに {key} が見つからない"));
    let after_key = body[key_pos + needle.len()..].trim_start();
    if after_key.starts_with("true") {
        true
    } else if after_key.starts_with("false") {
        false
    } else {
        panic!(
            "{key} の値が true/false のいずれでもない: `{}`",
            &after_key[..after_key.len().min(10)]
        )
    }
}

#[test]
fn all_workflow_files_are_classified() {
    let (unclassified, _expected) = derive_expected_checks();
    assert!(
        unclassified.is_empty(),
        "分類表に無いワークフローファイルが見つかった: {unclassified:?}。\
本テストファイル冒頭の分類表（JOB_ENUMERATED_WORKFLOW_FILES / \
REUSABLE_WORKFLOW_FILES / EXCLUDED_WORKFLOW_FILES）へ追加すること。"
    );
}

#[test]
fn manifest_matches_derived_expected_checks() {
    let (unclassified, expected) = derive_expected_checks();
    assert!(
        unclassified.is_empty(),
        "未分類のワークフローファイル: {unclassified:?}"
    );

    let manifest_body = std::fs::read_to_string(manifest_path())
        .unwrap_or_else(|e| panic!("{:?} の読み込みに失敗した: {e}", manifest_path()));
    let (strict, manifest_checks) = parse_manifest_contexts(&manifest_body);

    assert!(
        !strict,
        "strict_required_status_checks_policy は false でなければならない \
（implement-issue-tree の並列ラン収束要件、.claude/rules/ci.md 参照）"
    );

    let expected_set: HashSet<(String, i64)> = expected.into_iter().collect();
    let manifest_set: HashSet<(String, i64)> = manifest_checks.into_iter().collect();

    let missing_in_manifest: Vec<_> = expected_set.difference(&manifest_set).collect();
    let extra_in_manifest: Vec<_> = manifest_set.difference(&expected_set).collect();

    assert!(
        missing_in_manifest.is_empty() && extra_in_manifest.is_empty(),
        "ワークフロー YAML から導出した期待集合と .github/required-status-checks.json \
が一致しない。\nマニフェストに追加が必要（ワークフローにあるが manifest に無い）: \
{missing_in_manifest:?}\nマニフェストから削除が必要（manifest にあるがワークフローに \
無い）: {extra_in_manifest:?}\n\
`gh api repos/Fandhe-AI/fandhe-frontend/rulesets/<id>` から再生成するか手動で \
.github/required-status-checks.json を修正すること（イシュー #2325）。"
    );

    // 重複 context が manifest 内に存在しないこと（HashSet 化で黙って
    // 握り潰さない: 集合サイズが元の Vec 長と一致するかで検知する）。
    let manifest_body_reparsed = parse_manifest_contexts(&manifest_body).1;
    assert_eq!(
        manifest_body_reparsed.len(),
        manifest_set.len(),
        "manifest 内に重複 context がある"
    );
}
