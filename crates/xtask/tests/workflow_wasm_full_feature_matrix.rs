//! イシュー #2328「wasm-full の feature matrix を CI へ追加する」の
//! fail-closed ドリフト検知テスト。
//!
//! `.github/workflows/ci.yml` に新設した 4 ジョブ
//! （`wasm-full-feature-matrix-baseline` / `-wiring` / `-scope` /
//! `-readonly-guard`）は、`crates/wasm-full/Cargo.toml` の `[features]`
//! （`wasm-bindgen-exports` を除く全 feature、`.claude/rules/coding-rust.md`
//! の公開クレート semver 規律のもと今後も増減し得る）を単体構成で
//! `cargo clippy` する。本テストはワークフロー YAML から `run:` 行を
//! 文字列走査で機械抽出し、`Cargo.toml` の feature 集合との過不足なき
//! 一致を検証する（`crates/xtask/tests/workflow_shared_target_contract.rs`
//! と同じ行ベース走査の流儀。外部 YAML パーサは追加しない、REQ-3）。
//!
//! `-baseline` ジョブは `--no-default-features` / `--no-default-features
//! --features wasm-bindgen-exports` / 既定 / `--all-features` の 4 構成を
//! 検証し、`-readonly-guard` ジョブは readonly RadioGroup の click capture
//! 保護（イシュー #1616、PR #2339 で `wire_readonly_click_guard` として
//! `keynav` から分離）が `keynav` 無効構成でも機能することを実行テストで
//! 固定する。

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/xtask/ から 2 段上でワークスペースルートに到達する（イシュー #436）")
        .to_path_buf()
}

fn read_ci_workflow() -> String {
    std::fs::read_to_string(workspace_root().join(".github/workflows/ci.yml"))
        .expect(".github/workflows/ci.yml の読み込みに失敗した")
}

fn read_wasm_full_cargo_toml() -> String {
    std::fs::read_to_string(workspace_root().join("crates/wasm-full/Cargo.toml"))
        .expect("crates/wasm-full/Cargo.toml の読み込みに失敗した")
}

/// `contents`（ci.yml 全文）から、`\n  <job_name>:` で始まるトップレベル
/// ジョブのブロック本文（次のトップレベルジョブ名の直前まで）を返す。
/// `workflow_shared_target_contract.rs::job_block` と同型の抽出ロジック
/// （テストバイナリが分かれているため独立に定義する）。
fn job_block<'a>(contents: &'a str, job_name: &str) -> &'a str {
    let marker = format!("\n  {job_name}:");
    let start = contents
        .find(&marker)
        .unwrap_or_else(|| panic!("ジョブ `{job_name}` が ci.yml に見つからない"));
    let body_start = start + marker.len();
    let rest = &contents[body_start..];
    let mut end = rest.len();
    let mut search_from = 0usize;
    while let Some(rel) = rest[search_from..].find("\n  ") {
        let idx = search_from + rel;
        let after = &rest[idx + 3..];
        if !after.starts_with(' ') && !after.starts_with('\n') {
            end = idx;
            break;
        }
        search_from = idx + 3;
    }
    &rest[..end]
}

/// `crates/wasm-full/Cargo.toml` の `[features]` セクションから、
/// `default = [...]` の配列宣言を除く feature 名（`<name> = [...]` の
/// `<name>`）の集合を抽出する。`[features]` の次のトップレベルセクション
/// （行頭 `[` で始まる行）に到達したところで走査を打ち切る。
fn wasm_full_feature_names(cargo_toml: &str) -> Vec<String> {
    let marker = "\n[features]\n";
    let start = cargo_toml
        .find(marker)
        .expect("crates/wasm-full/Cargo.toml に `[features]` セクションが見つからない");
    let section = &cargo_toml[start + marker.len()..];

    let mut names = Vec::new();
    let mut in_default_array = false;
    for line in section.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            // 次のトップレベルセクション（例: `[dev-dependencies]`）に到達。
            break;
        }
        if in_default_array {
            if trimmed.starts_with(']') {
                in_default_array = false;
            }
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("default") {
            let rest = rest.trim_start();
            if rest.starts_with('=') {
                in_default_array = true;
                continue;
            }
        }
        // `<name> = []` 形式の行から `<name>` を抽出する。コメント行・
        // 空行はマッチしない。
        if let Some(eq_idx) = trimmed.find('=') {
            let name = trimmed[..eq_idx].trim();
            if !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            {
                names.push(name.to_string());
            }
        }
    }
    names
}

/// ジョブ本文から `cargo clippy ... --features wasm-bindgen-exports,<single>
/// ... --target wasm32-unknown-unknown --all-targets --locked -- -D warnings`
/// 形式の `run:` 行を抽出し、`<single>`（カンマを含まない単一 feature 名）の
/// 集合を返す。`wasm-bindgen-exports,a,b` のように複数 feature を並べた行
/// （clippy-wasm32 ジョブの部分組合せステップ等）は対象外とする——
/// `,` の後にさらに `,` が続かないことを要求する。
fn single_features_from_job(job: &str) -> Vec<String> {
    let mut features = Vec::new();
    for line in job.lines() {
        let trimmed = line.trim();
        let Some(run_idx) = trimmed.find("run: cargo clippy ") else {
            continue;
        };
        let run = &trimmed[run_idx..];
        if !run.contains("--target wasm32-unknown-unknown")
            || !run.contains("--all-targets")
            || !run.contains("--locked")
            || !run.contains("-- -D warnings")
        {
            continue;
        }
        let Some(features_idx) = run.find("--features ") else {
            continue;
        };
        let after = &run[features_idx + "--features ".len()..];
        let value = after.split_whitespace().next().unwrap_or("");
        let Some(single) = value.strip_prefix("wasm-bindgen-exports,") else {
            continue;
        };
        if single.contains(',') || single.is_empty() {
            // 複数 feature の組合せ行（部分組合せステップ等）は対象外。
            continue;
        }
        features.push(single.to_string());
    }
    features
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

#[test]
fn wiring_and_scope_jobs_cover_all_non_wasm_bindgen_exports_features_without_gaps_or_overlap() {
    let ci = read_ci_workflow();
    let cargo_toml = read_wasm_full_cargo_toml();

    let all_features: Vec<String> = wasm_full_feature_names(&cargo_toml)
        .into_iter()
        .filter(|f| f != "wasm-bindgen-exports")
        .collect();

    let wiring_job = job_block(&ci, "wasm-full-feature-matrix-wiring");
    let scope_job = job_block(&ci, "wasm-full-feature-matrix-scope");
    let wiring_features = single_features_from_job(wiring_job);
    let scope_features = single_features_from_job(scope_job);

    // 重複（同じ feature を wiring/scope 双方、または同一ジョブ内で
    // 2 度検証している）がないことを先に確認する。
    let mut combined = wiring_features.clone();
    combined.extend(scope_features.clone());
    let mut dedup = combined.clone();
    dedup.sort();
    dedup.dedup();
    assert_eq!(
        sorted(combined.clone()),
        sorted(dedup),
        "wasm-full-feature-matrix-wiring / -scope ジョブの間、または同一ジョブ内で \
         同じ feature を重複して clippy している行がある（1 feature 1 ステップの \
         契約に反する）"
    );

    assert_eq!(
        sorted(combined),
        sorted(all_features.clone()),
        "wasm-full-feature-matrix-wiring / -scope ジョブが clippy している feature の \
         合計集合が、crates/wasm-full/Cargo.toml の [features]（wasm-bindgen-exports \
         を除く）と一致しない。feature を追加・削除したら CI の matrix ジョブへの \
         追随が必要（イシュー #2328）"
    );
}

#[test]
fn baseline_job_covers_four_required_flag_combinations() {
    let ci = read_ci_workflow();
    let job = job_block(&ci, "wasm-full-feature-matrix-baseline");

    let required_run_substrings = [
        "cargo clippy -p fandhe-frontend-wasm-full --no-default-features --target wasm32-unknown-unknown --all-targets --locked -- -D warnings",
        "cargo clippy -p fandhe-frontend-wasm-full --no-default-features --features wasm-bindgen-exports --target wasm32-unknown-unknown --all-targets --locked -- -D warnings",
        "cargo clippy -p fandhe-frontend-wasm-full --target wasm32-unknown-unknown --all-targets --locked -- -D warnings",
        "cargo clippy -p fandhe-frontend-wasm-full --all-features --target wasm32-unknown-unknown --all-targets --locked -- -D warnings",
    ];
    for expected in required_run_substrings {
        assert!(
            job.contains(expected),
            "wasm-full-feature-matrix-baseline ジョブに期待する run: 行が見つからない: \
             `{expected}`（イシュー #2328 の 4 構成: --no-default-features / \
             --no-default-features --features wasm-bindgen-exports / default / \
             --all-features）"
        );
    }
    // `cargo check` も同じ 4 構成で併走させる契約（受入基準の「cargo check + clippy」）。
    let required_check_substrings = [
        "cargo check -p fandhe-frontend-wasm-full --no-default-features --target wasm32-unknown-unknown --all-targets --locked",
        "cargo check -p fandhe-frontend-wasm-full --no-default-features --features wasm-bindgen-exports --target wasm32-unknown-unknown --all-targets --locked",
        "cargo check -p fandhe-frontend-wasm-full --target wasm32-unknown-unknown --all-targets --locked",
        "cargo check -p fandhe-frontend-wasm-full --all-features --target wasm32-unknown-unknown --all-targets --locked",
    ];
    for expected in required_check_substrings {
        assert!(
            job.contains(expected),
            "wasm-full-feature-matrix-baseline ジョブに期待する `cargo check` の run: 行が \
             見つからない: `{expected}`"
        );
    }
}

#[test]
fn readonly_guard_job_runs_fail_closed_filtered_tests_without_keynav() {
    let ci = read_ci_workflow();
    let job = job_block(&ci, "wasm-full-feature-matrix-readonly-guard");

    assert!(
        job.contains("--no-default-features --features wasm-bindgen-exports")
            && job.contains("--test keynav_browser")
            && job.contains(
                "radio_group_readonly_click_is_suppressed_by_readonly_click_guard_without_wire_keynav"
            ),
        "wasm-full-feature-matrix-readonly-guard ジョブに、`keynav` を無効化した \
         構成（--no-default-features --features wasm-bindgen-exports）での \
         keynav_browser readonly guard テスト実行が見つからない（イシュー #2328）"
    );

    // wasm-bindgen-test はフィルタ不一致（0 件）でも `ok` を返すため、
    // テスト件数を grep で確認する fail-closed 化が必須（設計判断 §2.2）。
    assert!(
        job.contains("1 passed; 0 failed"),
        "wasm-full-feature-matrix-readonly-guard ジョブの browser テストステップに、\
         フィルタ不一致（0 件 PASS）を検知する件数確認（grep）が見つからない"
    );
    assert!(
        job.contains("readonly_click_outcome") && job.contains("feature_gating_contract"),
        "wasm-full-feature-matrix-readonly-guard ジョブに native テスト \
         （readonly_click_outcome / feature_gating_contract）の実行が見つからない"
    );
    assert!(
        job.matches("4 passed; 0 failed").count() >= 2,
        "wasm-full-feature-matrix-readonly-guard ジョブの native テストステップ \
         （readonly_click_outcome / feature_gating_contract）双方に、フィルタ不一致を \
         検知する件数確認（`4 passed; 0 failed` の grep）が見つからない"
    );
}

#[cfg(test)]
mod fixture_tests {
    use super::*;

    const FIXTURE_CARGO_TOML: &str = "[package]\nname = \"fandhe-frontend-wasm-full\"\n\n[features]\ndefault = [\n  \"wasm-bindgen-exports\",\n  \"keynav\",\n]\nwasm-bindgen-exports = []\nperf-assert = []\nkeynav = []\naccordion = []\n\n[dev-dependencies]\nwasm-bindgen-test = \"0.3\"\n";

    #[test]
    fn fixture_wasm_full_feature_names_excludes_default_and_stops_at_next_section() {
        let names = wasm_full_feature_names(FIXTURE_CARGO_TOML);
        assert_eq!(
            sorted(names),
            sorted(vec![
                "wasm-bindgen-exports".to_string(),
                "perf-assert".to_string(),
                "keynav".to_string(),
                "accordion".to_string(),
            ])
        );
    }

    #[test]
    fn fixture_single_features_from_job_excludes_multi_feature_combination_lines() {
        let job = "\n    steps:\n      - name: single\n        run: cargo clippy -p fandhe-frontend-wasm-full --no-default-features --features wasm-bindgen-exports,keynav --target wasm32-unknown-unknown --all-targets --locked -- -D warnings\n      - name: combo\n        run: cargo clippy -p fandhe-frontend-wasm-full --no-default-features --features wasm-bindgen-exports,keynav,tabs --target wasm32-unknown-unknown --locked -- -D warnings\n";
        let features = single_features_from_job(job);
        assert_eq!(features, vec!["keynav".to_string()]);
    }

    #[test]
    fn fixture_wiring_scope_coverage_detects_missing_feature() {
        let ci = "\njobs:\n  wasm-full-feature-matrix-wiring:\n    steps:\n      - name: a\n        run: cargo clippy -p fandhe-frontend-wasm-full --no-default-features --features wasm-bindgen-exports,perf-assert --target wasm32-unknown-unknown --all-targets --locked -- -D warnings\n  wasm-full-feature-matrix-scope:\n    steps:\n      - name: b\n        run: cargo clippy -p fandhe-frontend-wasm-full --no-default-features --features wasm-bindgen-exports,keynav --target wasm32-unknown-unknown --all-targets --locked -- -D warnings\n";
        let cargo_toml = FIXTURE_CARGO_TOML;

        let all_features: Vec<String> = wasm_full_feature_names(cargo_toml)
            .into_iter()
            .filter(|f| f != "wasm-bindgen-exports")
            .collect();
        let wiring_job = job_block(ci, "wasm-full-feature-matrix-wiring");
        let scope_job = job_block(ci, "wasm-full-feature-matrix-scope");
        let mut combined = single_features_from_job(wiring_job);
        combined.extend(single_features_from_job(scope_job));

        // フィクスチャは "accordion" を欠落させているため不一致になるはず。
        assert_ne!(sorted(combined), sorted(all_features));
    }
}
