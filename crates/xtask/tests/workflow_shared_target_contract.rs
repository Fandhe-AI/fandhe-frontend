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
//! | ci.yml 3 ジョブのガードステップ（3 テスト） | **維持・強化** | ホステッドでは `CARGO_TARGET_DIR` 未設定のため no-op（条件分岐付きで残置、#1229）。`actions/cache` で `target/` を復元した場合の自己修復・ローカル開発での多層防御として引き続き必要。ガード検出をステップ名の完全一致に限定し、無関係なステップ内のコメント引用による vacuous PASS を防ぐ（イシュー #1226 レビュー指摘） |
//! | ci.yml で `target` をキャッシュするジョブへのガード必須契約 | **新設・強化** | `actions/cache` 導入時にガードステップを付け忘れると #1192 型 flaky がキャッシュ経由でサイレント再発する。単なるマーカー存在ではなく、(a) キャッシュ復元ステップより**後段**に置かれていること（`actions/cache` の復元はそのステップ実行時に起きるため、先行するガードは復元後の汚染を除去できない）、(b) キャッシュされたパスと**同一の参照**（環境変数名またはリテラルパス）を削除対象にしていること、の 2 点を検証する（イシュー #1226 レビュー指摘 High/Medium 各 1 件） |
//! | ci.yml/release.yml の `target` キャッシュ検出ヒューリスティック | **強化** | `path:` 値の走査を大小文字非依存にし、`${{ env.CARGO_TARGET_DIR }}` のような自然な env 参照形（`target` が `CARGO_TARGET_DIR` の一部として大文字でのみ現れる）を見逃さない（イシュー #1226 レビュー指摘 Medium） |
//! | release.yml での target キャッシュ禁止契約（新設） | **追加** | release.yml の検証ビルドは `RUNNER_TEMP` 配下の隔離 target を使うため、ワークスペース `target` のキャッシュ復元は無意味かつ汚染 rlib の供給源になり得る。§3.3 のジョブ系統分離を「release.yml では target 系パスをキャッシュしない」という最も単純で決定的な形で機械強制する（`~/.cargo/registry` 等の非 target キャッシュは許容） |
//!
//! この判断は既存契約の弱体化にあたらない（5 テストとも assert 意味論を
//! 変えず維持し、新契約 2 件を追加・強化した）。`actions/cache` を用いる
//! ステップは現時点（イシュー #1226 実装時点）でワークフロー中に 1 件も
//! 存在しないため、上記強化はいずれも Phase 2 以降のキャッシュ導入 PR を
//! 待って初めて実効判定に入る（導入前は vacuous に PASS する）。
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
/// 網羅的に走査できる（新設契約は「target をキャッシュする全ジョブ」を
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
///
/// 返る各要素は「ジョブ内での出現順」を保持する（新設の順序契約——ガード
/// ステップがキャッシュ復元ステップより後段にあることの検証——が
/// この順序に依存する、イシュー #1226）。
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

/// ステップ本文の**先頭行**（`- name: "..."` または `- uses: ...`）が
/// `#1192` ガードステップの名前と完全一致するかを判定する。
///
/// イシュー #1226 レビュー指摘（vacuous PASS の一種）: 旧実装は
/// `job.find(GUARD_STEP_MARKER)`（ジョブ本文全体に対する部分文字列探索）を
/// 使っており、無関係なステップのコメント中にマーカー文字列が引用されて
/// いるだけでも「ガードあり」と誤判定し得た。ステップの `name:` 行に
/// 限定して完全一致させることで、この種の vacuous PASS を防ぐ。
fn step_is_guard_step(step: &str) -> bool {
    let Some(first_line) = step.lines().next() else {
        return false;
    };
    let trimmed = first_line.trim();
    let trimmed = trimmed.strip_prefix("- ").unwrap_or(trimmed);
    let Some(rest) = trimmed.strip_prefix("name:") else {
        return false;
    };
    rest.trim().trim_matches('"') == GUARD_STEP_MARKER
}

/// ci.yml の `job_name` ジョブが、無ハッシュ cdylib rlib 3 種（wasm-thin/
/// wasm-full/wasm-client）すべてを削除するガードステップを持つことを検証する
/// 純粋関数（ファイル I/O なし）。1 種類でも欠けると当該クレートの flaky を
/// 見逃すため、3 種すべてを個別に確認する（部分的な対策で「対策済み」と
/// 誤認しないため）。
fn check_job_has_guard_step(contents: &str, job_name: &str) -> Result<(), String> {
    let job = job_block(contents, job_name);
    let step = split_steps(job)
        .into_iter()
        .find(|s| step_is_guard_step(s))
        .ok_or_else(|| {
            format!(
                "ci.yml の `{job_name}` ジョブに、イシュー #1192 対策のガード \
                 ステップ（共有 CARGO_TARGET_DIR の無ハッシュ cdylib rlib \
                 除去）が見つからない（ステップ名がガードマーカーと完全一致する \
                 ことを要求する。他ステップ内のコメント言及のみでは満たさない）"
            )
        })?;

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
// 付与・順序・対象パスの一致を怠ると即座に FAIL する。
// ---------------------------------------------------------------------

/// ステップ本文が `actions/cache` を使用する `uses:` 行を持つかを判定する。
fn step_uses_actions_cache(step: &str) -> bool {
    step.lines()
        .any(|l| l.trim_start().starts_with("uses:") && l.contains("actions/cache"))
}

/// `actions/cache` ステップが復元する「target 系ディレクトリ」の参照形を
/// 表す。ガード側が削除対象とする参照（環境変数名またはリテラルパス）と
/// 突き合わせるために区別する（イシュー #1226 finding 1）。
#[derive(Debug, Clone, PartialEq, Eq)]
enum CacheTargetRef {
    /// `${{ env.CARGO_TARGET_DIR }}` 等、環境変数参照経由。値は変数名
    /// （例: `"CARGO_TARGET_DIR"`）。
    EnvVar(String),
    /// `target/` 等、環境変数を介さないリテラルパス。値は末尾 `/` を
    /// 除いたトークン（例: `"target"`）。
    Literal(String),
}

/// GitHub Actions 式 `${{ env.NAME }}` またはシェル風変数参照 `$NAME` /
/// `${NAME}` から変数名を抽出する。
fn extract_env_var_name(text: &str) -> Option<String> {
    if let Some(idx) = text.find("env.") {
        let rest = &text[idx + 4..];
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            return Some(name);
        }
    }
    if let Some(idx) = text.find('$') {
        let rest = &text[idx + 1..];
        let rest = rest.strip_prefix('{').unwrap_or(rest);
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

/// 1 行の `path:` 値（または block scalar の 1 要素）が target 系ディレクトリを
/// 指しているかを判定し、指していれば参照形を返す。
///
/// 大小文字を区別しない（イシュー #1226 finding 3）: `${{ env.CARGO_TARGET_DIR }}`
/// のような自然な env 参照形は `target` という語が `CARGO_TARGET_DIR` の一部と
/// して**大文字でのみ**現れるため、大文字小文字を区別する部分文字列一致では
/// 見逃す。このヒューリスティックは意図的に over-detect 側へ倒す
/// （fail-closed）: 誤検知の代償はガード要求 1 件だが、見逃しの代償は
/// キャッシュ経由の flaky 再発であるため。
fn target_ref_in_line(line: &str) -> Option<CacheTargetRef> {
    let trimmed = line.trim();
    if !trimmed.to_lowercase().contains("target") {
        return None;
    }
    if let Some(name) = extract_env_var_name(trimmed) {
        return Some(CacheTargetRef::EnvVar(name));
    }
    Some(CacheTargetRef::Literal(
        trimmed.trim_end_matches('/').to_string(),
    ))
}

/// `actions/cache` ステップの `path:` 値（単一行・複数行ブロックスカラーの
/// いずれも対応）から target 系ディレクトリの参照形を抽出する。
///
/// `path:` キーと同じかそれより浅いインデントの次の行（典型的には兄弟キー
/// `key:`）に到達したところで走査を打ち切る（YAML の `with:` ブロック内で
/// `path:` の値だけを見るための簡易境界判定。フルパーサは導入しない
/// REQ-3 方針に従う）。
fn cache_step_target_reference(step: &str) -> Option<CacheTargetRef> {
    let mut in_path = false;
    let mut path_indent = 0usize;
    for line in step.lines() {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if !in_path {
            if let Some(rest) = trimmed.strip_prefix("path:") {
                in_path = true;
                path_indent = indent;
                if let Some(r) = target_ref_in_line(rest) {
                    return Some(r);
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
        if let Some(r) = target_ref_in_line(trimmed) {
            return Some(r);
        }
    }
    None
}

/// `actions/cache` ステップの `path:` 値が target という語を含むかを判定する
/// （`cache_step_target_reference` の bool ラッパー。既存呼び出し箇所との
/// 互換のため残す）。
fn cache_step_path_contains_target(step: &str) -> bool {
    cache_step_target_reference(step).is_some()
}

/// ガードステップ（`run:` ブロック中の `rm -f` 対象行）が削除しているファイル
/// パスの「ルートディレクトリ部分」（`/debug/deps/` より前の部分）を抽出する。
///
/// 例: `"${CARGO_TARGET_DIR}/debug/deps/libfandhe_frontend_wasm_thin.rlib" \`
/// → `"${CARGO_TARGET_DIR}"`。この抽出結果を `CacheTargetRef` と突き合わせる
/// ことで、「ガードは存在するが実際には別ディレクトリを掃除している」
/// （イシュー #1226 finding 1）を検知する。
fn guard_deleted_path_prefixes(guard_step: &str) -> Vec<String> {
    guard_step
        .lines()
        .filter_map(|line| {
            let t = line.trim();
            let t = t.strip_suffix('\\').map(str::trim_end).unwrap_or(t);
            let t = t.strip_prefix('"')?;
            let t = t.strip_suffix('"')?;
            if t.ends_with(".rlib") || t.ends_with(".so") || t.ends_with(".d") {
                t.split("/debug/deps/").next().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// ガードステップが `target_ref` と同じディレクトリを削除対象としているかを
/// 判定する。環境変数参照は変数名の完全一致、リテラルパスはトークンの完全
/// 一致で照合する（単純な部分文字列一致にすると `CARGO_TARGET_DIR` という
/// 変数名自体が小文字化後に `target` を含んでしまい、リテラル `target/` との
/// 誤マッチを生む。イシュー #1226 finding 1 の再発防止のため完全一致に
/// 限定する）。
fn guard_step_covers_target_ref(guard_step: &str, target_ref: &CacheTargetRef) -> bool {
    let prefixes = guard_deleted_path_prefixes(guard_step);
    match target_ref {
        CacheTargetRef::EnvVar(name) => {
            let braced = format!("${{{name}}}");
            let bare = format!("${name}");
            prefixes.iter().any(|p| p == &braced || p == &bare)
        }
        CacheTargetRef::Literal(token) => prefixes.iter().any(|p| p == token),
    }
}

/// ci.yml 相当の内容を受け取り、「`target` をキャッシュする `actions/cache`
/// ステップを持つジョブは、そのステップより**後段**に、**同じ参照先**を
/// 削除する #1192 ガードステップ（`step_is_guard_step`）を持つ」契約を
/// 検証する。
///
/// 順序を見る理由（イシュー #1226 finding 2）: `actions/cache` の復元は
/// そのステップの実行時に起きるため、それより前段のガードは復元後に
/// 持ち込まれた汚染 rlib を除去できない。対象先を見る理由（finding 1）:
/// ガードステップが存在してもキャッシュと異なるディレクトリ（例:
/// 環境変数未設定時の既定 `target/`）を掃除しているだけでは無意味な
/// no-op になり得る。`docs/ci/hosted-runner-migration.md` §3.3 の
/// ジョブ系統分離を補完する。
fn check_ci_jobs_caching_target_have_guard(contents: &str) -> Result<(), String> {
    let jobs = split_top_level_jobs(jobs_section(contents));
    for (name, block) in &jobs {
        let steps = split_steps(block);
        let cache_targets: Vec<(usize, CacheTargetRef)> = steps
            .iter()
            .enumerate()
            .filter(|(_, step)| step_uses_actions_cache(step))
            .filter_map(|(idx, step)| cache_step_target_reference(step).map(|r| (idx, r)))
            .collect();
        for (cache_idx, target_ref) in &cache_targets {
            let covered = steps
                .iter()
                .enumerate()
                .skip(cache_idx + 1)
                .any(|(_, step)| {
                    step_is_guard_step(step) && guard_step_covers_target_ref(step, target_ref)
                });
            if !covered {
                return Err(format!(
                    "ci.yml の `{name}` ジョブが target を含む actions/cache ステップを持つが、\
                     それより後段に、同じ参照先（環境変数名またはパス）を削除するイシュー #1192 \
                     のガードステップが見つからない。actions/cache のキャッシュ復元はそのステップ \
                     実行時に起きるため、ガードはキャッシュ復元より後の順序に置き、かつ復元される \
                     ディレクトリと同じ参照を削除対象にすること（イシュー #1226、\
                     docs/ci/hosted-runner-migration.md §3.3 参照）"
                ));
            }
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
// フィクスチャ単体テスト: 上記判定関数が PASS/FAIL 双方向で意図どおりに
// 動くことを、実ワークフローを変異させずインライン YAML 断片で確認する。
// ---------------------------------------------------------------------

#[cfg(test)]
mod fixture_tests {
    use super::*;

    const NO_CACHE: &str = "\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - name: Checkout\n        uses: actions/checkout@v4\n      - name: Build\n        run: cargo build\n";

    /// ガードステップの実体（3 クレート分の rm -f）を持つ共通ブロック。
    /// 実 ci.yml のガードステップと同じ `${CARGO_TARGET_DIR}/debug/deps/...`
    /// 参照形を使う。
    const GUARD_STEP_YAML: &str = "      - name: \"guard: 共有 CARGO_TARGET_DIR の無ハッシュ cdylib rlib を除去（イシュー #1192）\"\n        run: |\n          set -euo pipefail\n          if [ -n \"${CARGO_TARGET_DIR:-}\" ] && [ -d \"${CARGO_TARGET_DIR}/debug/deps\" ]; then\n            rm -f \\\n              \"${CARGO_TARGET_DIR}/debug/deps/libfandhe_frontend_wasm_thin.rlib\" \\\n              \"${CARGO_TARGET_DIR}/debug/deps/libfandhe_frontend_wasm_thin.so\" \\\n              \"${CARGO_TARGET_DIR}/debug/deps/fandhe_frontend_wasm_thin.d\" \\\n              \"${CARGO_TARGET_DIR}/debug/deps/libfandhe_frontend_wasm_full.rlib\" \\\n              \"${CARGO_TARGET_DIR}/debug/deps/libfandhe_frontend_wasm_full.so\" \\\n              \"${CARGO_TARGET_DIR}/debug/deps/fandhe_frontend_wasm_full.d\" \\\n              \"${CARGO_TARGET_DIR}/debug/deps/libfandhe_frontend_wasm_client.rlib\" \\\n              \"${CARGO_TARGET_DIR}/debug/deps/libfandhe_frontend_wasm_client.so\" \\\n              \"${CARGO_TARGET_DIR}/debug/deps/fandhe_frontend_wasm_client.d\"\n          fi\n";

    const CACHE_STEP_ENV_VAR_FORM: &str = "      - name: Cache cargo target\n        uses: actions/cache@v4\n        with:\n          path: |\n            ~/.cargo/registry\n            ${{ env.CARGO_TARGET_DIR }}\n          key: cargo-${{ hashFiles('**/Cargo.lock') }}\n";

    const CACHE_STEP_LITERAL_TARGET: &str = "      - name: Cache cargo target\n        uses: actions/cache@v4\n        with:\n          path: |\n            ~/.cargo/registry\n            target/\n          key: cargo-${{ hashFiles('**/Cargo.lock') }}\n";

    /// キャッシュ（env var 形）→ ガード（同一変数を削除）の順。唯一の
    /// 正常系: 復元後にガードが走り、かつ同じディレクトリを掃除する。
    fn target_cache_with_guard_after() -> String {
        format!("\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n{CACHE_STEP_ENV_VAR_FORM}{GUARD_STEP_YAML}")
    }

    /// ガード（キャッシュより前段）→ キャッシュの順。ガードはキャッシュ
    /// 復元後の汚染を除去できないため FAIL する必要がある
    /// （イシュー #1226 finding 2）。
    fn target_cache_with_guard_before_only() -> String {
        format!("\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n{GUARD_STEP_YAML}{CACHE_STEP_ENV_VAR_FORM}")
    }

    const TARGET_CACHE_WITHOUT_GUARD: &str = "\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - name: Cache cargo target\n        uses: actions/cache@v4\n        with:\n          path: |\n            ~/.cargo/registry\n            target/\n          key: cargo-${{ hashFiles('**/Cargo.lock') }}\n";

    const REGISTRY_ONLY_CACHE: &str = "\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - name: Cache cargo registry\n        uses: actions/cache@v4\n        with:\n          path: ~/.cargo/registry\n          key: cargo-registry-${{ hashFiles('**/Cargo.lock') }}\n";

    // (a) キャッシュなし → ci/release ともに PASS
    #[test]
    fn fixture_no_cache_passes_both_contracts() {
        assert!(check_ci_jobs_caching_target_have_guard(NO_CACHE).is_ok());
        assert!(check_release_jobs_must_not_cache_target(NO_CACHE).is_ok());
    }

    // (b) target キャッシュ（env var 形）+ ガードが後段にあり同一参照を
    //     削除 → ci contract は PASS。
    #[test]
    fn fixture_target_cache_with_guard_after_passes_ci_contract() {
        let yaml = target_cache_with_guard_after();
        assert!(check_ci_jobs_caching_target_have_guard(&yaml).is_ok());
    }

    // (c) target キャッシュ + ガードなし → ci contract は FAIL
    #[test]
    fn fixture_target_cache_without_guard_fails_ci_contract() {
        let result = check_ci_jobs_caching_target_have_guard(TARGET_CACHE_WITHOUT_GUARD);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ガードステップ"));
    }

    // (c') ガードがキャッシュより前段にしかない → ci contract は FAIL
    //      （イシュー #1226 finding 2: 順序を無視した vacuous PASS の防止）。
    #[test]
    fn fixture_guard_before_cache_only_fails_ci_contract() {
        let yaml = target_cache_with_guard_before_only();
        let result = check_ci_jobs_caching_target_have_guard(&yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("後段"));
    }

    // (c'') target をリテラルパス（`target/`）でキャッシュしているのに
    //       ガードが環境変数参照 `${CARGO_TARGET_DIR}` しか削除しない →
    //       実際には別ディレクトリを掃除しているだけで no-op のため
    //       ci contract は FAIL（イシュー #1226 finding 1）。
    #[test]
    fn fixture_literal_target_cache_with_env_var_guard_fails_ci_contract() {
        let yaml = format!(
            "\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n{CACHE_STEP_LITERAL_TARGET}{GUARD_STEP_YAML}"
        );
        let result = check_ci_jobs_caching_target_have_guard(&yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("同じ参照先"));
    }

    // (d) registry のみキャッシュ → ci/release ともに PASS
    #[test]
    fn fixture_registry_only_cache_passes_both_contracts() {
        assert!(check_ci_jobs_caching_target_have_guard(REGISTRY_ONLY_CACHE).is_ok());
        assert!(check_release_jobs_must_not_cache_target(REGISTRY_ONLY_CACHE).is_ok());
    }

    // (e) release.yml で target をキャッシュ（ガード有無・順序に関わらず）
    //     → FAIL（release.yml はそもそも target キャッシュ自体を禁止する）。
    #[test]
    fn fixture_target_cache_fails_release_contract_regardless_of_guard() {
        let with_guard = target_cache_with_guard_after();
        assert!(check_release_jobs_must_not_cache_target(&with_guard).is_err());
        assert!(check_release_jobs_must_not_cache_target(TARGET_CACHE_WITHOUT_GUARD).is_err());
    }

    // (f) `path:` が `${{ env.CARGO_TARGET_DIR }}` のように `target` という
    //     語を大文字でしか含まない env 参照形でも検出できる
    //     （イシュー #1226 finding 3: 大小文字非依存ヒューリスティック）。
    #[test]
    fn fixture_env_var_form_cache_path_detected_case_insensitively() {
        assert!(cache_step_path_contains_target(CACHE_STEP_ENV_VAR_FORM));
        assert_eq!(
            cache_step_target_reference(CACHE_STEP_ENV_VAR_FORM),
            Some(CacheTargetRef::EnvVar("CARGO_TARGET_DIR".to_string()))
        );
    }

    // (f') 同 env 参照形でガードが存在しない場合は ci contract が FAIL する
    //      ことも合わせて確認する（検出漏れがないことの終端確認）。
    #[test]
    fn fixture_env_var_form_cache_without_guard_fails_ci_contract() {
        let yaml = format!(
            "\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n{CACHE_STEP_ENV_VAR_FORM}"
        );
        assert!(check_ci_jobs_caching_target_have_guard(&yaml).is_err());
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

    // step_is_guard_step: ステップ名の完全一致のみを「ガードあり」と
    // 認め、無関係なステップ本文中のコメント引用では満たさないことを確認
    // する（イシュー #1226 レビュー指摘、vacuous PASS の再発防止）。
    #[test]
    fn fixture_step_is_guard_step_requires_exact_name_match() {
        assert!(step_is_guard_step(GUARD_STEP_YAML));
        let decoy = "      - name: Some other step\n        run: |\n          echo \"guard: 共有 CARGO_TARGET_DIR の無ハッシュ cdylib rlib を除去（イシュー #1192）\"\n";
        assert!(!step_is_guard_step(decoy));
    }
}
