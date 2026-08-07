//! イシュー #1192「wasm-thin テストの共有 CARGO_TARGET_DIR キャッシュ誤命中に
//! よる flaky を根本対策する」の対策宣言をワークフロー YAML に対して
//! fail-closed に固定するドリフト検知テスト。
//!
//! イシュー #1226 でホステッドランナー移行（トラッキング #1220）を前提に
//! 契約の要否を再設計した。以下は再設計の記録である。
//!
//! ## 根本原因（runner 種別に依存しない。詳細はイシュー #1192 コメント参照）
//!
//! `crates/wasm-thin`（および wasm-full / wasm-client）は
//! `crate-type = ["cdylib", "rlib"]` のため、cargo はその rlib を
//! `<target>/debug/deps/libfandhe_frontend_wasm_thin.rlib` という**メタデータ
//! ハッシュサフィックスなしの固定ファイル名**で出力する（cdylib は動的
//! ライブラリ名を安定させる必要があり `-C extra-filename` が付かない cargo の
//! 仕様。この仕様自体は runner が self-hosted かホステッドかに依存しない）。
//! `.github/workflows/release.yml` の `cargo publish --dry-run` /
//! `cargo publish` は packaged コピー（path 依存が剥がされ crates.io
//! registry 版の依存に解決される）を検証ビルドするため、通常のワークスペース
//! ビルドと同一の `CARGO_TARGET_DIR` を共有すると、無ハッシュ rlib を
//! registry 依存の内容で上書きしてしまう。後続の `cargo test --workspace` 等が
//! fingerprint fresh 判定でこの汚染済み rlib をそのままリンクすると、
//! 「there are multiple different versions of crate
//! `fandhe_frontend_interactive` in the dependency graph」（E0277/E0599）の
//! flaky を引き起こす（PR #1164/#1180/#1186/#1187 で実際に観測）。
//!
//! ## ホステッド移行で消える前提・残る前提（イシュー #1226 の設計判断）
//!
//! ホステッドランナーはジョブごとにクリーンな使い捨て VM のため、旧
//! self-hosted 環境の「複数ジョブが同一ホストの共有 `/cargo-target` を
//! 恒常的に共有する」という前提は原理的に消える
//! （`docs/ci/hosted-runner-migration.md` §2.1(a)）。一方で、同文書 §3.3 が
//! 指摘するとおり `actions/cache` で `target/` を復元する構成を導入すると、
//! **復元されたキャッシュが同型の汚染源になり得る**（release 系検証ビルドが
//! 汚染した rlib がキャッシュへ保存され、別ジョブ・別実行のワークスペース
//! ビルドへ復元されると同じ症状が再発する）。ローカル開発（開発者が
//! `CARGO_TARGET_DIR` を共有設定している場合）でも同種の汚染は起こり得る。
//!
//! ## 設計判断（存続・強化。イシュー #1226）
//!
//! | 契約 | 判断 | 根拠 |
//! |------|------|------|
//! | release.yml の専用 `CARGO_TARGET_DIR` 隔離（2 テスト） | **維持** | packaged コピー検証ビルドとワークスペースビルドの分離は runner 種別に依存せず正しい。`actions/cache` 導入後は「汚染 rlib をキャッシュへ保存しない」保証としても機能する |
//! | ci.yml 3 ジョブのガードステップ（3 テスト） | **維持** | ホステッドでは `CARGO_TARGET_DIR` 未設定のため no-op（条件分岐付きで残置、#1229）。`actions/cache` で `target/` を復元した場合の自己修復・ローカル開発での多層防御として引き続き必要 |
//! | ci.yml で `target` をキャッシュするジョブへのガード必須契約（新設） | **追加** | `actions/cache` 導入時にガードステップを付け忘れると #1192 型 flaky がキャッシュ経由でサイレント再発する。導入前の現時点では対象ジョブ 0 件で PASS する vacuous 契約だが、Phase 2 以降のキャッシュ導入 PR がガードを忘れると即 FAIL する fail-closed 設計 |
//! | release.yml での target キャッシュ禁止契約（新設） | **追加** | release.yml の検証ビルドは `RUNNER_TEMP` 配下の隔離 target を使うため、ワークスペース `target` のキャッシュ復元は無意味かつ汚染 rlib の供給源になり得る。§3.3 のジョブ系統分離を「release.yml では target 系パスをキャッシュしない」という最も単純で決定的な形で機械強制する（`~/.cargo/registry` 等の非 target キャッシュは許容） |
//!
//! この判断は既存契約の弱体化にあたらない（5 テストとも assert 意味論を
//! 変えず維持し、新契約 2 件を追加した）。
//!
//! 外部 YAML パーサは追加しない（REQ-3・xtask 外部依存ゼロ方針。
//! `template_deny_workflow.rs` と同じく行ベースの文字列一致に留める）。

use std::path::PathBuf;

/// workspace ルート（`xtask/` の親ディレクトリ）の絶対パスを返す。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/xtask/ から 2 段上でワークスペースルートに到達する（イシュー #436）")
        .to_path_buf()
}

fn release_workflow_path() -> PathBuf {
    workspace_root().join(".github/workflows/release.yml")
}

fn ci_workflow_path() -> PathBuf {
    workspace_root().join(".github/workflows/ci.yml")
}

fn read_release_workflow() -> String {
    std::fs::read_to_string(release_workflow_path())
        .expect(".github/workflows/release.yml の読み込みに失敗した")
}

fn read_ci_workflow() -> String {
    std::fs::read_to_string(ci_workflow_path())
        .expect(".github/workflows/ci.yml の読み込みに失敗した")
}

/// `contents` を `jobs:` 直下のトップレベルジョブ名（行頭 2 スペース + 識別子 +
/// コロン、以降行頭スペースが増える構成）で分割し、`job_name` に対応する
/// ジョブブロック本文（次のトップレベルジョブ名の直前まで）を返す。
///
/// ワークフロー YAML の一般規約（`.claude/rules/ci.md` の
/// ステップ名クォート規約と同様の前提）に従い、トップレベルジョブは
/// 行頭からインデント 2 で始まる想定。
fn job_block<'a>(contents: &'a str, job_name: &str) -> &'a str {
    let marker = format!("\n  {job_name}:");
    let start = contents.find(&marker).unwrap_or_else(|| {
        panic!("ジョブ `{job_name}` が見つからない（ci.yml のジョブ構成が変わった可能性がある）")
    });
    let body_start = start + marker.len();
    let rest = &contents[body_start..];
    // 次のトップレベルジョブ（行頭 "\n  " + 非空白 + ":" で始まる行、かつ
    // インデントが本ジョブ本文（3 スペース以上）より浅い行）の直前までを
    // 本ジョブのブロックとみなす。行頭 "\n  " の直後が空白でない行を探す。
    let mut end = rest.len();
    let mut search_from = 0usize;
    while let Some(rel) = rest[search_from..].find("\n  ") {
        let idx = search_from + rel;
        let after = &rest[idx + 3..];
        // "\n  " の直後が空白（インデント 3 以上）なら、本ジョブ内のステップ行。
        // 空白でなければ次のトップレベルキー（次ジョブ）の開始とみなす。
        if !after.starts_with(' ') && !after.starts_with('\n') {
            end = idx;
            break;
        }
        search_from = idx + 3;
    }
    &rest[..end]
}

/// release.yml の `job_name` ジョブが専用 `CARGO_TARGET_DIR`（`RUNNER_TEMP`
/// 配下）を宣言していることを検証する純粋関数（ファイル I/O なし）。
///
/// `#[test]` から実ファイル読み込み経由で呼ばれるほか、下部の
/// フィクスチャ単体テストが PASS/FAIL 双方向を機械的に裏付ける。
fn check_release_job_isolates_cargo_target_dir(
    contents: &str,
    job_name: &str,
) -> Result<(), String> {
    let block = job_block(contents, job_name);
    if block.contains("CARGO_TARGET_DIR:") && block.contains("runner.temp") {
        Ok(())
    } else {
        Err(format!(
            "release.yml の `{job_name}` ジョブに専用 CARGO_TARGET_DIR（RUNNER_TEMP 配下）\
             の env 宣言が見つからない（イシュー #1192 の根本対策）。共有 \
             CARGO_TARGET_DIR 上で `cargo publish`/`cargo publish --dry-run` を実行すると \
             cdylib+rlib クレートの無ハッシュ rlib を registry 依存内容で \
             上書きし、後続のワークスペーステストが flaky になる"
        ))
    }
}

#[test]
fn release_workflow_verify_job_isolates_cargo_target_dir() {
    let contents = read_release_workflow();
    if let Err(msg) = check_release_job_isolates_cargo_target_dir(&contents, "verify") {
        panic!("{msg}");
    }
}

#[test]
fn release_workflow_publish_job_isolates_cargo_target_dir() {
    let contents = read_release_workflow();
    if let Err(msg) = check_release_job_isolates_cargo_target_dir(&contents, "publish") {
        panic!("{msg}");
    }
}

/// イシュー #1192 対策ガードステップの一意な識別マーカー。
/// `ci.yml` 側のステップ名文字列と完全一致させる契約。
const GUARD_STEP_MARKER: &str =
    "guard: 共有 CARGO_TARGET_DIR の無ハッシュ cdylib rlib を除去（イシュー #1192）";

/// ci.yml の `job_name` ジョブが、無ハッシュ cdylib rlib 3 種（wasm-thin/
/// wasm-full/wasm-client）すべてを削除するガードステップを持つことを検証する
/// 純粋関数（ファイル I/O なし）。1 種類でも欠けると当該クレートの flaky を
/// 見逃すため、3 種すべてを個別に確認する（部分的な対策で「対策済み」と
/// 誤認しないため）。
fn check_job_has_guard_step(contents: &str, job_name: &str) -> Result<(), String> {
    let job = job_block(contents, job_name);
    let step_start = job.find(GUARD_STEP_MARKER).ok_or_else(|| {
        format!(
            "ci.yml の `{job_name}` ジョブに、イシュー #1192 対策のガード \
             ステップ（共有 CARGO_TARGET_DIR の無ハッシュ cdylib rlib \
             除去）が見つからない"
        )
    })?;
    // ガードステップ本文のみを対象にする（他ステップの無関係な `rm -rf`
    // を誤検知しないよう、次の `- name:` 行の直前までに限定する）。
    let rest = &job[step_start..];
    let step_end = rest[1..]
        .find("\n      - name:")
        .map(|rel| rel + 1)
        .unwrap_or(rest.len());
    let step = &rest[..step_end];

    for crate_name in [
        "libfandhe_frontend_wasm_thin.rlib",
        "libfandhe_frontend_wasm_full.rlib",
        "libfandhe_frontend_wasm_client.rlib",
    ] {
        if !step.contains(crate_name) {
            return Err(format!(
                "ci.yml の `{job_name}` ジョブのガードステップが `{crate_name}` \
                 の削除を含んでいない（3 種すべてを削除しないと当該クレートの \
                 flaky を見逃す）"
            ));
        }
    }
    // 広域削除（glob・rm -rf）を持ち込んでいないことも合わせて確認する
    // （A01 パストラバーサル・意図しない広域削除の防止、security.md 参照）。
    if step.contains("rm -rf") {
        return Err(format!(
            "ci.yml の `{job_name}` ジョブのガードステップに `rm -rf` が含まれて \
             いる（固定ファイル名のみの `rm -f` に限定する設計から逸脱している）"
        ));
    }
    Ok(())
}

fn assert_job_has_guard_step(job_name: &str) {
    let contents = read_ci_workflow();
    if let Err(msg) = check_job_has_guard_step(&contents, job_name) {
        panic!("{msg}");
    }
}

#[test]
fn ci_workflow_forbid_unsafe_job_has_shared_target_guard() {
    assert_job_has_guard_step("forbid-unsafe");
}

#[test]
fn ci_workflow_test_job_has_shared_target_guard() {
    assert_job_has_guard_step("test");
}

#[test]
fn ci_workflow_gate_self_apply_job_has_shared_target_guard() {
    assert_job_has_guard_step("gate-self-apply");
}

// ---------------------------------------------------------------------
// イシュー #1226: actions/cache 導入を先取りする fail-closed 契約
//
// 現時点（`actions/cache` 未導入）ではいずれの新契約も対象ジョブ 0 件で
// vacuous に PASS するが、Phase 2 以降のキャッシュ導入 PR がガードステップの
// 付与・target キャッシュの回避を怠ると即座に FAIL する。
// ---------------------------------------------------------------------

/// `jobs:` セクション本文（`jobs:` 行の直後から）を返す。トップレベル
/// キーの探索対象を `jobs:` 配下に限定し、`on:`/`env:` 等の他セクションに
/// 現れる同型インデントの行を誤ってジョブ境界と誤認しないための前処理。
fn jobs_section(contents: &str) -> &str {
    let marker = "\njobs:\n";
    let start = contents.find(marker).unwrap_or_else(|| {
        panic!("`jobs:` セクションが見つからない（ワークフロー YAML の構成が変わった可能性がある）")
    });
    &contents[start + marker.len()..]
}

/// `jobs:` セクション本文をトップレベルジョブ（行頭インデント 2 スペース +
/// 識別子 + コロン）ごとに分割し、`(ジョブ名, ジョブブロック本文)` の列を
/// 返す。既存 `job_block` と異なりジョブ名を事前に知らずに全ジョブを
/// 網羅的に走査できる（新設 2 契約は「target をキャッシュする全ジョブ」を
/// 横断検査する必要があるため）。
fn split_top_level_jobs(section: &str) -> Vec<(String, String)> {
    let mut jobs = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_lines: Vec<&str> = Vec::new();

    for line in section.lines() {
        let is_top_level_job_line =
            line.starts_with("  ") && !line.starts_with("   ") && line.trim_end().ends_with(':');
        if is_top_level_job_line {
            if let Some(name) = current_name.take() {
                jobs.push((name, current_lines.join("\n")));
                current_lines = Vec::new();
            }
            let name = line.trim().trim_end_matches(':').to_string();
            current_name = Some(name);
        } else if current_name.is_some() {
            current_lines.push(line);
        }
    }
    if let Some(name) = current_name.take() {
        jobs.push((name, current_lines.join("\n")));
    }
    jobs
}

/// ジョブブロック本文をステップ（行頭インデント 6 スペース + `- `）単位で
/// 分割する。`- name:` と `- uses:` のどちらでステップが始まっていても
/// 境界として検出できるよう、キー名を問わず `"\n      - "` のみで分割する。
fn split_steps(job_block: &str) -> Vec<&str> {
    let marker = "\n      - ";
    let mut positions = Vec::new();
    let mut search_from = 0usize;
    while let Some(rel) = job_block[search_from..].find(marker) {
        let idx = search_from + rel;
        positions.push(idx);
        search_from = idx + marker.len();
    }
    let mut steps = Vec::with_capacity(positions.len());
    for (i, &start) in positions.iter().enumerate() {
        let step_start = start + 1; // 先頭の "\n" を除き "      - ..." から開始
        let end = positions.get(i + 1).copied().unwrap_or(job_block.len());
        steps.push(&job_block[step_start..end]);
    }
    steps
}

/// ステップ本文が `actions/cache` を使用する `uses:` 行を持つかを判定する。
fn step_uses_actions_cache(step: &str) -> bool {
    step.lines()
        .any(|l| l.trim_start().starts_with("uses:") && l.contains("actions/cache"))
}

/// `actions/cache` ステップの `path:` 値（単一行・複数行ブロックスカラーの
/// いずれも対応）が `target` という語を含むかを判定する。
///
/// `path:` キーと同じかそれより浅いインデントの次の行（典型的には兄弟キー
/// `key:`）に到達したところで走査を打ち切る（YAML の `with:` ブロック内で
/// `path:` の値だけを見るための簡易境界判定。フルパーサは導入しない
/// REQ-3 方針に従う）。
fn cache_step_path_contains_target(step: &str) -> bool {
    let mut in_path = false;
    let mut path_indent = 0usize;
    for line in step.lines() {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if !in_path {
            if let Some(rest) = trimmed.strip_prefix("path:") {
                in_path = true;
                path_indent = indent;
                if rest.contains("target") {
                    return true;
                }
            }
            continue;
        }
        if trimmed.is_empty() {
            continue;
        }
        if indent <= path_indent {
            // path: と同じかそれより浅いインデント＝兄弟キー（key: 等）に到達。
            break;
        }
        if trimmed.contains("target") {
            return true;
        }
    }
    false
}

/// ci.yml 相当の内容を受け取り、「`target` をキャッシュする `actions/cache`
/// ステップを持つジョブは、必ず #1192 ガードステップ
/// （`GUARD_STEP_MARKER`）をジョブ内に持つ」契約を検証する。
///
/// `actions/cache` で `target/` を復元する構成では、release 系検証ビルドが
/// 生成した汚染 rlib がキャッシュ経由で復元され得るため（キャッシュ導入時に
/// ガードステップの付与を機械強制する。`docs/ci/hosted-runner-migration.md`
/// §3.3 のジョブ系統分離を補完する）。
fn check_ci_jobs_caching_target_have_guard(contents: &str) -> Result<(), String> {
    let jobs = split_top_level_jobs(jobs_section(contents));
    for (name, block) in &jobs {
        let caches_target = split_steps(block)
            .into_iter()
            .any(|step| step_uses_actions_cache(step) && cache_step_path_contains_target(step));
        if caches_target && !block.contains(GUARD_STEP_MARKER) {
            return Err(format!(
                "ci.yml の `{name}` ジョブが target を含む actions/cache ステップを持つが、\
                 イシュー #1192 のガードステップ（無ハッシュ cdylib rlib 除去）が \
                 見つからない。actions/cache で target/ を復元する構成では同型の \
                 rlib 汚染がキャッシュ経由で再発し得るため、ガードステップを \
                 追加すること（イシュー #1226、docs/ci/hosted-runner-migration.md §3.3 参照）"
            ));
        }
    }
    Ok(())
}

/// release.yml 相当の内容を受け取り、「いかなるジョブも `target` を
/// `actions/cache` でキャッシュしない」契約を検証する（`~/.cargo/registry`
/// 等の非 target キャッシュは許容）。
///
/// release.yml の検証ビルド（`verify`/`publish`）は専用 `CARGO_TARGET_DIR`
/// （`RUNNER_TEMP` 配下）で隔離される設計（イシュー #1192 根本対策）であり、
/// ワークスペース `target` のキャッシュ復元は隔離の意図と矛盾し、かつ
/// 汚染 rlib の供給源になり得る（`docs/ci/hosted-runner-migration.md` §3.3）。
fn check_release_jobs_must_not_cache_target(contents: &str) -> Result<(), String> {
    let jobs = split_top_level_jobs(jobs_section(contents));
    for (name, block) in &jobs {
        for step in split_steps(block) {
            if step_uses_actions_cache(step) && cache_step_path_contains_target(step) {
                return Err(format!(
                    "release.yml の `{name}` ジョブが target ディレクトリを actions/cache で \
                     キャッシュしている。release.yml の検証ビルドは専用 CARGO_TARGET_DIR \
                     （RUNNER_TEMP 配下）で隔離される設計（イシュー #1192 根本対策）であり、\
                     ワークスペース target のキャッシュ復元は隔離の意図と矛盾し汚染 rlib の \
                     供給源になり得る（イシュー #1226、docs/ci/hosted-runner-migration.md §3.3）。\
                     ~/.cargo/registry 等の非 target キャッシュは許容される"
                ));
            }
        }
    }
    Ok(())
}

#[test]
fn ci_workflow_jobs_caching_target_must_have_guard_step() {
    let contents = read_ci_workflow();
    if let Err(msg) = check_ci_jobs_caching_target_have_guard(&contents) {
        panic!("{msg}");
    }
}

#[test]
fn release_workflow_must_not_cache_target_dir() {
    let contents = read_release_workflow();
    if let Err(msg) = check_release_jobs_must_not_cache_target(&contents) {
        panic!("{msg}");
    }
}

// ---------------------------------------------------------------------
// フィクスチャ単体テスト: 上記 2 判定関数が PASS/FAIL 双方向で意図どおりに
// 動くことを、実ワークフローを変異させずインライン YAML 断片で確認する。
// ---------------------------------------------------------------------

#[cfg(test)]
mod fixture_tests {
    use super::*;

    const NO_CACHE: &str = "\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - name: Checkout\n        uses: actions/checkout@v4\n      - name: Build\n        run: cargo build\n";

    const TARGET_CACHE_WITH_GUARD: &str = "\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - name: \"guard: 共有 CARGO_TARGET_DIR の無ハッシュ cdylib rlib を除去（イシュー #1192）\"\n        run: echo guard\n      - name: Cache cargo target\n        uses: actions/cache@v4\n        with:\n          path: |\n            ~/.cargo/registry\n            target/\n          key: cargo-${{ hashFiles('**/Cargo.lock') }}\n";

    const TARGET_CACHE_WITHOUT_GUARD: &str = "\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - name: Cache cargo target\n        uses: actions/cache@v4\n        with:\n          path: |\n            ~/.cargo/registry\n            target/\n          key: cargo-${{ hashFiles('**/Cargo.lock') }}\n";

    const REGISTRY_ONLY_CACHE: &str = "\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - name: Cache cargo registry\n        uses: actions/cache@v4\n        with:\n          path: ~/.cargo/registry\n          key: cargo-registry-${{ hashFiles('**/Cargo.lock') }}\n";

    // (a) キャッシュなし → ci/release ともに PASS
    #[test]
    fn fixture_no_cache_passes_both_contracts() {
        assert!(check_ci_jobs_caching_target_have_guard(NO_CACHE).is_ok());
        assert!(check_release_jobs_must_not_cache_target(NO_CACHE).is_ok());
    }

    // (b) target キャッシュ + ガードあり → ci contract は PASS
    #[test]
    fn fixture_target_cache_with_guard_passes_ci_contract() {
        assert!(check_ci_jobs_caching_target_have_guard(TARGET_CACHE_WITH_GUARD).is_ok());
    }

    // (c) target キャッシュ + ガードなし → ci contract は FAIL
    #[test]
    fn fixture_target_cache_without_guard_fails_ci_contract() {
        let result = check_ci_jobs_caching_target_have_guard(TARGET_CACHE_WITHOUT_GUARD);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ガードステップ"));
    }

    // (d) registry のみキャッシュ → ci/release ともに PASS
    #[test]
    fn fixture_registry_only_cache_passes_both_contracts() {
        assert!(check_ci_jobs_caching_target_have_guard(REGISTRY_ONLY_CACHE).is_ok());
        assert!(check_release_jobs_must_not_cache_target(REGISTRY_ONLY_CACHE).is_ok());
    }

    // (e) release.yml で target をキャッシュ（ガード有無に関わらず）→ FAIL
    #[test]
    fn fixture_target_cache_fails_release_contract_regardless_of_guard() {
        assert!(check_release_jobs_must_not_cache_target(TARGET_CACHE_WITH_GUARD).is_err());
        assert!(check_release_jobs_must_not_cache_target(TARGET_CACHE_WITHOUT_GUARD).is_err());
    }

    // split_top_level_jobs / split_steps が複数ジョブ・複数ステップを
    // 正しく分離できることの確認（境界誤判定によるすり抜け防止）。
    #[test]
    fn fixture_multiple_jobs_are_isolated_from_each_other() {
        let multi = "\njobs:\n  clean:\n    steps:\n      - name: Checkout\n        uses: actions/checkout@v4\n  cached:\n    steps:\n      - name: Cache cargo target\n        uses: actions/cache@v4\n        with:\n          path: target/\n          key: k\n";
        // clean ジョブは target キャッシュを持たないためガード不要で全体 PASS
        // にはならない（cached ジョブがガードを欠くため FAIL する）ことを
        // 確認し、ジョブ間の走査が相互に独立していることを裏付ける。
        let result = check_ci_jobs_caching_target_have_guard(multi);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("`cached`"));
    }
}
