//! イシュー #3725「docs-site の外部利用経路（匿名取得 -> 生成 -> 検査）を CI で
//! 毎回実行する」の配線と匿名性を固定する契約テスト。
//!
//! 外部利用経路の検証は新規ジョブではなく `ci.yml` の `test-docs-site` ジョブの
//! 3 ステップ（`tools/ci/docs-site-external-use-smoke.sh install|generate|inspect`）
//! として実装している（ruleset への required context 追加を避けるため。理由は
//! `.claude/rules/ci.md`）。「認証情報を渡さない」「skip 可能にしない」は、後から
//! 変更されてもジョブが green のまま目的を失う性質なので、ここで機械的に固定する。
//!
//! 判定は文字列入力の純関数で行い、内側の `mod tests` の変異ケースで検知が
//! 空振りしないことを確認する。外部 YAML パーサは使わない（REQ-3）。
//! `strip_comment` は他の workflow 契約テストと同じ意図的な複製（テスト間共有モジュールを
//! 持たない既存流儀）。

use std::path::PathBuf;

const SCRIPT_REL: &str = "tools/ci/docs-site-external-use-smoke.sh";
const JOB_ID: &str = "test-docs-site";
const STAGES: [&str; 3] = ["install", "generate", "inspect"];

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/xtask/ から 2 段上でワークスペースルートに到達する")
        .to_path_buf()
}

/// 全行コメントと、空白の直前にある行中コメントを落とす（クォート内の `#` は残す）。
fn strip_comment(line: &str) -> String {
    if line.trim_start().starts_with('#') {
        return String::new();
    }
    let mut out = String::new();
    let (mut in_single, mut in_double) = (false, false);
    let mut prev: Option<char> = None;
    for ch in line.chars() {
        match ch {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '#' if !in_single && !in_double && prev.is_none_or(|c| c.is_whitespace()) => break,
            _ => {}
        }
        out.push(ch);
        prev = Some(ch);
    }
    out
}

/// `  <job_id>:` から次のインデント 2 のキーまでの、コメント除去済みの行を返す。
fn job_block(content: &str, job_id: &str) -> Option<Vec<String>> {
    let header = format!("  {job_id}:");
    let mut lines = content.lines();
    lines.find(|l| l.trim_end() == header)?;
    let mut block = Vec::new();
    for line in lines {
        let indent = line.len() - line.trim_start().len();
        if indent == 2 && !line.trim().is_empty() && !line.trim_start().starts_with('#') {
            break;
        }
        block.push(strip_comment(line));
    }
    Some(block)
}

/// ジョブブロックの違反を列挙する。
fn job_violations(block: &[String]) -> Vec<String> {
    let mut v = Vec::new();

    // 3 ステージが各 1 回・この順で現れる。
    let positions: Vec<Option<usize>> = STAGES
        .iter()
        .map(|s| {
            let cmd = format!("run: {SCRIPT_REL} {s}");
            let hits: Vec<usize> = block
                .iter()
                .enumerate()
                .filter(|(_, l)| l.trim() == cmd)
                .map(|(i, _)| i)
                .collect();
            if hits.len() == 1 {
                Some(hits[0])
            } else {
                v.push(format!(
                    "`{cmd}` must appear exactly once (found {})",
                    hits.len()
                ));
                None
            }
        })
        .collect();
    let found: Vec<usize> = positions.iter().flatten().copied().collect();
    if found.len() == STAGES.len() && found.windows(2).any(|w| w[0] >= w[1]) {
        v.push("stages must run in the order install, generate, inspect".to_string());
    }

    // 認証情報・キャッシュ・skip 経路を持ち込まない。
    let joined = block.join("\n");
    for forbidden in ["secrets.", "GITHUB_TOKEN", "github.token", "actions/cache"] {
        if joined.contains(forbidden) {
            v.push(format!("job must not contain `{forbidden}`"));
        }
    }
    if !joined.contains("persist-credentials: false") {
        v.push("checkout must keep `persist-credentials: false`".to_string());
    }
    for l in block {
        let t = l.trim().trim_start_matches("- ");
        if t.starts_with("if:") || t.starts_with("continue-on-error:") {
            v.push(format!("job must not contain `{t}` (fail-open)"));
        }
    }
    v
}

/// スクリプト本文（コメント除去後）の違反を列挙する。
fn script_violations(script: &str) -> Vec<String> {
    let stripped: String = script
        .lines()
        .map(strip_comment)
        .collect::<Vec<_>>()
        .join("\n");
    let mut v = Vec::new();
    for required in [
        "env -i",
        "--locked",
        "--rev",
        "--no-page-sections",
        "GIT_TERMINAL_PROMPT=0",
    ] {
        if !stripped.contains(required) {
            v.push(format!("script must contain `{required}`"));
        }
    }
    for forbidden in ["GITHUB_TOKEN", "GH_TOKEN", "rm -rf"] {
        if stripped.contains(forbidden) {
            v.push(format!("script must not contain `{forbidden}`"));
        }
    }
    v
}

#[test]
fn test_docs_site_job_runs_external_use_stages_anonymously() {
    let ci = std::fs::read_to_string(workspace_root().join(".github/workflows/ci.yml"))
        .expect("ci.yml を読めること");
    let block = job_block(&ci, JOB_ID).expect("test-docs-site ジョブが存在すること");
    let v = job_violations(&block);
    assert!(
        v.is_empty(),
        "test-docs-site の外部利用ステップ契約違反: {v:#?}"
    );
}

#[test]
fn external_use_script_keeps_anonymity_guards() {
    let path = workspace_root().join(SCRIPT_REL);
    let script = std::fs::read_to_string(&path).expect("スクリプトを読めること");
    let v = script_violations(&script);
    assert!(v.is_empty(), "外部利用スクリプトの匿名性契約違反: {v:#?}");
}

#[cfg(unix)]
#[test]
fn external_use_script_is_executable() {
    use std::os::unix::fs::PermissionsExt;
    let mode = std::fs::metadata(workspace_root().join(SCRIPT_REL))
        .expect("スクリプトが存在すること")
        .permissions()
        .mode();
    assert!(mode & 0o111 != 0, "{SCRIPT_REL} に実行ビットが必要");
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r"
  test-docs-site:
    steps:
      - uses: actions/checkout@x
        with:
          persist-credentials: false
      - name: a
        run: tools/ci/docs-site-external-use-smoke.sh install
      - name: b
        run: tools/ci/docs-site-external-use-smoke.sh generate
      - name: c
        run: tools/ci/docs-site-external-use-smoke.sh inspect
  next-job:
    steps: []
";

    fn violations(yaml: &str) -> Vec<String> {
        job_violations(&job_block(yaml, JOB_ID).expect("block"))
    }

    #[test]
    fn good_fixture_has_no_violations() {
        assert!(violations(GOOD).is_empty());
    }

    #[test]
    fn detects_missing_stage() {
        let y = GOOD.replace("docs-site-external-use-smoke.sh inspect", "true");
        assert!(!violations(&y).is_empty());
    }

    #[test]
    fn detects_added_if() {
        let y = GOOD.replace(
            "      - name: b\n",
            "      - name: b\n        if: always()\n",
        );
        assert!(!violations(&y).is_empty());
    }

    #[test]
    fn detects_token_reference() {
        let y = GOOD.replace(
            "      - name: c\n",
            "      - name: c\n        env:\n          X: ${{ secrets.Y }}\n",
        );
        assert!(!violations(&y).is_empty());
    }

    #[test]
    fn ignores_comments_mentioning_forbidden_words() {
        let y = GOOD.replace(
            "      - name: a\n",
            "      # secrets. and if: are discussed here\n      - name: a\n",
        );
        assert!(violations(&y).is_empty());
    }

    #[test]
    fn script_without_env_i_is_rejected() {
        assert!(!script_violations(
            "cargo install --locked --rev x --no-page-sections GIT_TERMINAL_PROMPT=0"
        )
        .is_empty());
    }
}
