//! イシュー #1192「wasm-thin テストの共有 CARGO_TARGET_DIR キャッシュ誤命中に
//! よる flaky を根本対策する」の対策宣言をワークフロー YAML に対して
//! fail-closed に固定するドリフト検知テスト。
//!
//! ## 根本原因（再現済み。詳細はイシュー #1192 コメント参照）
//!
//! `crates/wasm-thin`（および wasm-full / wasm-client）は
//! `crate-type = ["cdylib", "rlib"]` のため、cargo はその rlib を
//! `<target>/debug/deps/libfandhe_frontend_wasm_thin.rlib` という**メタデータ
//! ハッシュサフィックスなしの固定ファイル名**で出力する（cdylib は動的
//! ライブラリ名を安定させる必要があり `-C extra-filename` が付かない cargo の
//! 仕様）。`.github/workflows/release.yml` の `cargo publish --dry-run` /
//! `cargo publish` は packaged コピー（path 依存が剥がされ crates.io
//! registry 版の依存に解決される）を検証ビルドするため、共有
//! `CARGO_TARGET_DIR`（self-hosted runner 既定の `/cargo-target`）上で実行
//! すると、通常のワークスペースビルドが生成した無ハッシュ rlib を registry
//! 依存の内容で上書きしてしまう。後続の `cargo test --workspace` 等が
//! fingerprint fresh 判定でこの汚染済み rlib をそのままリンクすると、
//! 「there are multiple different versions of crate
//! `fandhe_frontend_interactive` in the dependency graph」（E0277/E0599）の
//! flaky を引き起こす（PR #1164/#1180/#1186/#1187 で実際に観測）。
//!
//! ## 対策（本テストが固定する契約）
//!
//! 1. **根本対策**: `release.yml` の `verify` / `publish` 両ジョブが専用
//!    `CARGO_TARGET_DIR`（`RUNNER_TEMP` 配下）を明示指定し、検証ビルドを
//!    共有 target dir から隔離する（汚染源そのものを断つ）。
//! 2. **多層防御**: `ci.yml` の 3 ジョブ（`forbid-unsafe` / `test` /
//!    `gate-self-apply`）が cargo 実行前に無ハッシュ cdylib rlib を削除する
//!    自己修復ガードステップを持ち、対策導入前から runner ホストに残置して
//!    いる既存汚染や、他ワークフローに起因する将来の汚染からも回復する。
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

#[test]
fn release_workflow_verify_job_isolates_cargo_target_dir() {
    let contents = read_release_workflow();
    let block = job_block(&contents, "verify");
    assert!(
        block.contains("CARGO_TARGET_DIR:") && block.contains("runner.temp"),
        "release.yml の verify ジョブに専用 CARGO_TARGET_DIR（RUNNER_TEMP 配下）\
         の env 宣言が見つからない（イシュー #1192 の根本対策）。共有 \
         CARGO_TARGET_DIR 上で `cargo publish --dry-run` を実行すると \
         cdylib+rlib クレートの無ハッシュ rlib を registry 依存内容で \
         上書きし、後続のワークスペーステストが flaky になる"
    );
}

#[test]
fn release_workflow_publish_job_isolates_cargo_target_dir() {
    let contents = read_release_workflow();
    let block = job_block(&contents, "publish");
    assert!(
        block.contains("CARGO_TARGET_DIR:") && block.contains("runner.temp"),
        "release.yml の publish ジョブに専用 CARGO_TARGET_DIR（RUNNER_TEMP \
         配下）の env 宣言が見つからない（イシュー #1192 の根本対策）。\
         verify ジョブと同じ理由で `cargo publish` 実行時にも共有 \
         CARGO_TARGET_DIR を汚染し得るため隔離が必要"
    );
}

/// `ci.yml` の対象ジョブが、無ハッシュ cdylib rlib 3 種（wasm-thin/wasm-full/
/// wasm-client）すべてを削除するガードステップを持つことを検証する。
/// 1 種類でも欠けると当該クレートの flaky を見逃すため、3 種すべてを
/// 個別に assert する（部分的な対策で「対策済み」と誤認しないため）。
fn assert_job_has_guard_step(job_name: &str) {
    let contents = read_ci_workflow();
    let job = job_block(&contents, job_name);
    let marker = "guard: 共有 CARGO_TARGET_DIR の無ハッシュ cdylib rlib を除去（イシュー #1192）";
    let step_start = job.find(marker).unwrap_or_else(|| {
        panic!(
            "ci.yml の `{job_name}` ジョブに、イシュー #1192 対策のガード \
             ステップ（共有 CARGO_TARGET_DIR の無ハッシュ cdylib rlib \
             除去）が見つからない"
        )
    });
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
        assert!(
            step.contains(crate_name),
            "ci.yml の `{job_name}` ジョブのガードステップが `{crate_name}` \
             の削除を含んでいない（3 種すべてを削除しないと当該クレートの \
             flaky を見逃す）"
        );
    }
    // 広域削除（glob・rm -rf）を持ち込んでいないことも合わせて確認する
    // （A01 パストラバーサル・意図しない広域削除の防止、security.md 参照）。
    assert!(
        !step.contains("rm -rf"),
        "ci.yml の `{job_name}` ジョブのガードステップに `rm -rf` が含まれて \
         いる（固定ファイル名のみの `rm -f` に限定する設計から逸脱している）"
    );
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
