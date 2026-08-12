//! `xtask`: このワークスペースの CI 計測・自己保守用ツール群のエントリポイント。
//! 開発者用ツールであり、配布物（fandhe-frontend-* クレート）には含めない。
//!
//! サブコマンド:
//! - `check-deps --package <NAME> [--package <NAME> ...]`: REQ-3（依存グラフ上限
//!   60 件以内・深さ 6 以内、`docs/spec/04-requirements.md`）の実測（`check_deps`
//!   モジュールの TASK-3.1a 計測ロジック）と判定（TASK-3.1b の `judge`）を行い、
//!   TASK-3.1c の CI（`.github/workflows/deps-check.yml`）が終了コードと
//!   1 行サマリ（`check_deps::format_report` 参照）で PASS/FAIL を判定できるようにする。
//!   CLI 契約の回帰テストは `xtask/tests/cli_check_deps.rs`。
//!
//! 複数 `--package` 指定時も `cargo metadata` は 1 回のみ実行する
//! （`check_deps::measure_many_from_cargo_metadata` 参照。Bugbot 指摘
//! 「metadata rerun per package」への対応）。
//!
//! - `list-build-scripts --package <NAME> [--package <NAME> ...]`: REQ-3
//!   （サプライチェーン監査可能性、PoC-2 脅威モデル）のうち `build.rs` 保有クレートの
//!   機械的列挙（TASK-3.2a、`list_build_scripts` モジュール）。CI ワークフロー
//!   （`.github/workflows/deps-check.yml`）への組み込みは TASK-3.2b（イシュー #21）で、
//!   1 行サマリ（`list_build_scripts::format_inventory` 参照）を Step Summary に
//!   転記する。CLI 契約の回帰テストは `xtask/tests/cli_list_build_scripts.rs`。
//!
//! - `check-core-deps`（引数なし）: イシュー #154（REQ-3 受け入れ基準 1）。
//!   `check_deps::ZERO_DEP_CRATES` と実 workspace メンバーの積集合について
//!   `Normal`/`Dev`/`Build` すべての辺を辿り、workspace 内の第一者パッケージ
//!   （path dependency）を除いた「真の外部依存」が 1 件でも存在しないかを
//!   `check_deps::measure_external_only` で計測し判定する（`check_deps::judge_zero`）。
//!   `check-deps --package fandhe-frontend-core` の 60/6 判定とは別に「ゼロであること」を
//!   専用ゲートとして強制する。判定対象は CLI 引数で差し替え不可
//!   （`ZERO_DEP_CRATES` 参照。上限を弱める経路を作らない設計）。
//!   CLI 契約の回帰テストは `xtask/tests/cli_check_core_deps.rs`。
//!
//! - `check-loc`（引数なし）: TASK-8.2b（イシュー #62, REQ-8 受け入れ基準）と
//!   REQ-11 受け入れ基準 3（イシュー #156）の共用ゲート。
//!   `check_loc::LOC_CHECK_TARGETS`（`static/view-transitions.js` /
//!   `static/wasm-full-init.js`）について、それぞれコメント・空行を除いた
//!   実効 LOC を計測し、`check_loc::MAX_EFFECTIVE_LOC`（10 行）以内かを判定
//!   する（`check_loc::judge`）。対象ファイルの不在・読み取り失敗も超過と
//!   同様に fail-closed とする。判定対象・しきい値は CLI 引数で差し替え
//!   不可。CLI 契約の回帰テストは `xtask/tests/cli_check_loc.rs`。
//!
//! - `check-image-size --image <TAG> [--limit-mb <N>]`: TASK-9.3b（イシュー #103,
//!   REQ-9 受け入れ基準）。`docker image inspect --format {{.Size}}` で対象イメージの
//!   非圧縮サイズを計測し（`check_image_size` モジュールの `measure`）、既定 50MB
//!   （`check_image_size::REQ9_IMAGE_SIZE_LIMIT_BYTES`）以内かを判定する
//!   （`check_image_size::judge`）。`--limit-mb` は動作確認・段階導入のための
//!   上書きであり、既定値を弱める運用は想定しない。CI ワークフロー
//!   （`.github/workflows/image-size.yml`）はルート `Dockerfile`
//!   （TASK-9.3a／イシュー #102）を `docker build` した結果を本サブコマンドに渡す
//!   契約で、Dockerfile 未マージの間は意図的に計測失敗＝fail-closed で FAIL する。
//!   CLI 契約の回帰テストは `xtask/tests/cli_check_image_size.rs`。
//!
//! - `wasm-node-smoke [--build-only]`: イシュー #297（TASK-10.2 残課題、出典
//!   PR #220 §10 スコープ外節）。`docs/design/wasm-build-integration.md` §6.4 が
//!   文書化していた nodejs ターゲット開発フロー（`cargo build --target
//!   wasm32-unknown-unknown` → `wasm-bindgen --target nodejs` → `node -e
//!   "require(...)"`）を自動化する（`wasm_node_smoke` モジュール）。
//!   Cargo.lock 解決済み `wasm-bindgen` バージョンと `wasm-bindgen --version`
//!   の完全一致検証（`dist-server/build.rs::expected_wasm_bindgen_version`
//!   と同一契約）→ wasm32 ビルド → `--target nodejs` バインディング生成 →
//!   （`--build-only` 指定時を除き）node 実行での動作確認・既定エスケープ
//!   （REQ-1）回帰検証を行う。呼び出し元は `.github/workflows/ci.yml` の
//!   `wasm-node-smoke` ジョブ。1 行サマリは `wasm_node_smoke::format_report`
//!   参照。CLI 契約の回帰テストは `xtask/tests/cli_wasm_node_smoke.rs`。
//!
//! - `bench-ssr [--baseline <FILE>]`: イシュー #1317。非追跡領域（`_/bench/fandhe/ssr`）
//!   で行っていた SSR 性能計測（rows=1,000/10,000 テーブルの `render()` 時間）を
//!   常設化する（`bench_ssr` モジュール）。JSON 1 行を stdout へ出力し、既定エスケープ
//!   （REQ-1）回帰検知（`escape_ok`）・行数一致検知（`row_count_ok`）が `false` の
//!   場合は終了コード 1（fail-closed）。`--baseline <FILE>` に過去の本コマンド出力
//!   （JSON 1 行）を渡すと `bench-ssr-compare:` 行群で回帰比較を追加出力する
//!   （report-only、終了コードへは影響しない）。計測回数・行数の CLI 差し替え口は
//!   意図的に設けない。1 行サマリ・JSON スキーマは `bench_ssr` モジュール doc 参照。
//!   CLI 契約の回帰テストは `xtask/tests/cli_bench_ssr.rs`。
//!
//! - `check-version-bump --base-ref <REF> [--pr-body-file <PATH>] [--index-base-url <URL>]`:
//!   イシュー #638。公開済みクレート（crates.io）の実体（`src/` / `Cargo.toml` /
//!   `build.rs`）が変更されているのに `Cargo.toml` の version が既公開バージョンの
//!   ままの PR を検知する（`check_version_bump` モジュール）。headless-ui 0.1.0 公開後、
//!   バージョンバンプなしの破壊的変更がマージされ main を赤にした事故（PR #611 → 復旧
//!   PR #634）が動機。`--pr-body-file` に渡した PR 本文中の
//!   `version-bump-exempt: <crate-name>` 宣言（クレート名の完全一致のみ）で誤検知を
//!   免除できる。curl 不在・ネットワーク不達・想定外 HTTP status はすべて
//!   `environment error: ` プレフィックス付きで fail-closed に扱う。呼び出し元は
//!   `.github/workflows/ci.yml` の `version-bump-guard` ジョブ（PR コンテキストのみ）。
//!   1 行サマリは `check_version_bump::format_report` 参照。CLI 契約の回帰テストは
//!   `xtask/tests/cli_check_version_bump.rs`。
//!
//! - `check-dep-versions [--fix]`: イシュー #657。workspace 内メンバー間の
//!   `path + version` 併記依存について、依存元の `version = "..."` 要求が
//!   依存先の現行 version へ追随しているかを検知する（`check_dep_versions`
//!   モジュール）。headless-ui 0.1.0 → 0.2.0 バンプ時、依存元（pre-styled-ui /
//!   wasm-full / xtask）の `version = "..."` 追随が sed による手動一括変更を
//!   要した実績（`check-version-bump` の是正メッセージによる注意喚起のみでは
//!   機械検知手段がなかった、PR #647 out-of-scope）が動機。ネットワーク照会は
//!   一切行わない（`cargo metadata --no-deps` のみ）。既定（引数なし）は検知の
//!   みで fail-closed（CI: `.github/workflows/ci.yml` の `dep-version-check`
//!   ジョブ）。`--fix` は version 不一致（ルール 1）のみをローカルで自動修正する
//!   オプトイン手段で、書き換え位置を一意特定できない場合は一切書き換えない。
//!   1 行サマリは `check_dep_versions::format_report` 参照。CLI 契約の回帰
//!   テストは `xtask/tests/cli_check_dep_versions.rs`。
//!
//! - `patch-template-smoke --project-dir <DIR> --repo-root <DIR>
//!   [--index-base-url <URL>]`: イシュー #885（採用案の実装、イシュー #884の
//!   後続）。`template-app-wasm-smoke` ジョブ（`.github/workflows/ci.yml`）の
//!   「fw new」直後に実行し、生成プロジェクトのルート `Cargo.toml` ・
//!   `wasm/Cargo.toml` が要求する fandhe-frontend-* 各バージョンを crates.io
//!   sparse index へ照会する（`patch_template_smoke` モジュール、
//!   `check_version_bump::query_index` を再利用）。バンプ先バージョンが
//!   未公開で crates.io 実解決が成立しない依存についてのみ、当該マニフェストへ
//!   `[patch.crates-io]`（`--repo-root` 配下 `crates/<dir>` への絶対 path）を
//!   注入して対応する `Cargo.lock` を削除する（再現性低下は意図的に許容し
//!   stdout へ明記する）。全依存が crates.io で解決可能なら無変更。緩和用の
//!   環境変数・CLI フラグは設けない（既存の迂回禁止原則と同型）。1 行サマリは
//!   `patch_template_smoke::format_dep_report` 参照
//!   （`template-app-wasm-smoke: dep=<crate> version=<v>
//!   resolution=<crates-io|path-override>`）。crates.io 公開の承認境界
//!   （`release.yml`）は一切変更しない。詳細設計は
//!   `docs/ci/version-bump-publish-order-gap.md` を参照。CLI 契約の回帰
//!   テストは `xtask/tests/cli_patch_template_smoke.rs`。
//!
//! `core` / `interactive` と異なりプロセス起動（`std::process::Command`）を行うが、
//! `unsafe` は使わない（REQ-2 は core/interactive 限定だが、xtask でも forbid する。
//! core/tests/unsafe_boundary.rs の WASM/FFI 境界許可リストにも含まれない）。

#![forbid(unsafe_code)]

mod bench_binding_update;
mod bench_ssr;
mod bench_state_update;
mod check_dep_versions;
mod check_deps;
mod check_image_size;
mod check_loc;
mod check_version_bump;
mod json;
mod list_build_scripts;
mod patch_template_smoke;
mod wasm_node_smoke;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("check-deps") => run_check_deps(&args[2..]),
        Some("list-build-scripts") => run_list_build_scripts(&args[2..]),
        Some("check-core-deps") => run_check_core_deps(&args[2..]),
        Some("check-loc") => run_check_loc(&args[2..]),
        Some("check-image-size") => run_check_image_size(&args[2..]),
        Some("wasm-node-smoke") => run_wasm_node_smoke(&args[2..]),
        Some("bench-binding-update") => run_bench_binding_update(&args[2..]),
        Some("bench-ssr") => run_bench_ssr(&args[2..]),
        Some("bench-state-update") => run_bench_state_update(&args[2..]),
        Some("check-version-bump") => run_check_version_bump(&args[2..]),
        Some("check-dep-versions") => run_check_dep_versions(&args[2..]),
        Some("patch-template-smoke") => run_patch_template_smoke(&args[2..]),
        Some(other) => {
            eprintln!("xtask: unknown subcommand `{other}`");
            print_usage();
            // `check-deps` の引数不備（`--package` 不足等）は終了コード 2 を
            // 返す契約になっており、サブコマンド自体が不明・未指定の場合も
            // 同じ「usage エラー」区分として揃える。ExitCode::FAILURE (1) の
            // ままだと呼び出し元が判定失敗（1）と usage エラー（2）を区別
            // できない（Bugbot 指摘: wrong exit code for usage errors）。
            ExitCode::from(2)
        }
        None => {
            print_usage();
            ExitCode::from(2)
        }
    }
}

fn print_usage() {
    eprintln!("Usage: xtask <subcommand> [options]");
    eprintln!();
    eprintln!("Subcommands:");
    eprintln!("  check-deps --package <NAME> [--package <NAME> ...]");
    eprintln!("      Measure resolved dependency count and max depth for each package");
    eprintln!("      and judge them against the REQ-3 limits (60 packages / depth 6).");
    eprintln!("  list-build-scripts --package <NAME> [--package <NAME> ...]");
    eprintln!("      List crates with a custom build script (build.rs) reachable from");
    eprintln!("      each package (REQ-3 supply-chain audit visibility).");
    eprintln!("  check-core-deps");
    eprintln!("      Enforce zero external dependencies (normal/dev/build) for the core");
    eprintln!("      crates listed in check_deps::ZERO_DEP_CRATES (REQ-3 acceptance");
    eprintln!("      criterion 1, issue #154). Takes no arguments by design.");
    eprintln!("  check-loc");
    eprintln!("      Measure effective LOC (comments and blank lines excluded) for the");
    eprintln!("      files in check_loc::LOC_CHECK_TARGETS and judge them against");
    eprintln!("      check_loc::MAX_EFFECTIVE_LOC (REQ-8 acceptance criterion, issue #62;");
    eprintln!("      REQ-11 acceptance criterion 3, issue #156). Takes no arguments by design.");
    eprintln!("  check-image-size --image <TAG> [--limit-mb <N>]");
    eprintln!("      Measure the uncompressed size of a docker image (via `docker image");
    eprintln!("      inspect`) and judge it against the REQ-9 limit (default 50MB, issue");
    eprintln!("      #103). `--limit-mb` overrides the default for verification only.");
    eprintln!("  wasm-node-smoke [--build-only]");
    eprintln!("      Automate the nodejs-target dev workflow documented in");
    eprintln!("      docs/design/wasm-build-integration.md §6.4 for fandhe-frontend-wasm-thin: verify the");
    eprintln!("      installed wasm-bindgen-cli matches Cargo.lock's wasm-bindgen version,");
    eprintln!("      build for wasm32-unknown-unknown, generate `--target nodejs` bindings,");
    eprintln!("      and (unless --build-only) run a node smoke check including a default-");
    eprintln!("      escape (REQ-1) regression check (issue #297).");
    eprintln!("  bench-binding-update");
    eprintln!("      Measure full re-render vs dirty-tracked update cost (native, report-only,");
    eprintln!("      no threshold judgement) for AppState/Disclosure/SingleSelect dispatch");
    eprintln!("      (issue #592). Takes no arguments by design.");
    eprintln!("  bench-ssr [--baseline <FILE>]");
    eprintln!("      Measure SSR render() throughput (native, rows=1,000/10,000 tables) and");
    eprintln!("      emit a single JSON line (issue #1317). Exits non-zero if the default-");
    eprintln!("      escape (REQ-1) or row-count self-checks fail. `--baseline <FILE>` adds");
    eprintln!("      report-only `bench-ssr-compare:` lines against a prior JSON output.");
    eprintln!("  bench-state-update [--baseline <FILE>]");
    eprintln!("      Measure state-update cost breakdown (update/binding_apply/render/");
    eprintln!("      noop_update) for two 1,000-binding scenarios (grid-1k, appstate-1k) and");
    eprintln!("      emit a single JSON line (issue #1328). Exits non-zero if the default-");
    eprintln!("      escape (REQ-1) or no-op self-checks fail. `--baseline <FILE>` adds");
    eprintln!("      report-only `bench-state-update-compare:` lines against a prior JSON output.");
    eprintln!(
        "  check-version-bump --base-ref <REF> [--pr-body-file <PATH>] [--index-base-url <URL>]"
    );
    eprintln!("      Detect published crates (crates.io) whose sources changed (src/**/,");
    eprintln!("      Cargo.toml, build.rs) while Cargo.toml's version is unchanged from an");
    eprintln!("      already-published version (issue #638). `--pr-body-file` may declare");
    eprintln!("      `version-bump-exempt: <crate-name>` to exempt non-breaking changes.");
    eprintln!("  check-dep-versions [--fix]");
    eprintln!("      Detect workspace-internal path+version dependencies whose `version =");
    eprintln!("      \"...\"` requirement has not been kept in sync with the dependency's");
    eprintln!("      current version (issue #657). No network access (cargo metadata only).");
    eprintln!("      `--fix` auto-corrects version-mismatch failures in place; ambiguous or");
    eprintln!("      unsupported requirement forms are left untouched and reported as errors.");
    eprintln!(
        "  patch-template-smoke --project-dir <DIR> --repo-root <DIR> [--index-base-url <URL>]"
    );
    eprintln!("      Query crates.io for the fandhe-frontend-* version requirements declared by");
    eprintln!("      a `fw new --template app` output (issue #885). Deps that are not yet");
    eprintln!("      resolvable on crates.io get a `[patch.crates-io]` fallback pointing at");
    eprintln!("      --repo-root's crates/<dir>, with the corresponding Cargo.lock removed.");
    eprintln!("      Deps already resolvable on crates.io are left untouched.");
}

/// `check-deps` サブコマンド: `--package <NAME>` を 1 つ以上受け取り、
/// それぞれについて依存件数・最大深さを計測して上限判定し、結果を stdout に表示する。
///
/// `cargo metadata` は全パッケージ分をまとめて 1 回だけ実行する
/// （[`check_deps::measure_many_from_cargo_metadata`]）。
/// 引数不備・計測失敗・上限超過（`CheckResult::Fail`）のいずれかがあれば
/// 終了コード 1（fail-closed）を返す。
fn run_check_deps(args: &[String]) -> ExitCode {
    let mut packages = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--package" => {
                let Some(name) = args.get(i + 1) else {
                    eprintln!("xtask check-deps: `--package` requires a value");
                    return ExitCode::from(2);
                };
                packages.push(name.clone());
                i += 2;
            }
            other => {
                eprintln!("xtask check-deps: unknown argument `{other}`");
                return ExitCode::from(2);
            }
        }
    }

    if packages.is_empty() {
        eprintln!("xtask check-deps: at least one `--package <NAME>` is required");
        return ExitCode::from(2);
    }

    let results = match check_deps::measure_many_from_cargo_metadata(&packages) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("xtask check-deps: failed to run cargo metadata: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut had_failure = false;
    for (name, measurement) in results {
        match measurement {
            Ok(m) => {
                let check_result = check_deps::judge(m.into());
                print!("{}", check_deps::format_report(&check_result));
                if !check_result.is_pass() {
                    had_failure = true;
                }
            }
            Err(e) => {
                eprintln!("xtask check-deps: failed to measure `{name}`: {e}");
                had_failure = true;
            }
        }
    }

    if had_failure {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// `check-core-deps` サブコマンド（イシュー #154, REQ-3 受け入れ基準 1）: 引数を
/// 一切取らない。`check_deps::ZERO_DEP_CRATES` と実 workspace メンバーの積集合
/// （`check_deps::fetch_zero_dep_targets`）について、それぞれ `Normal`/`Dev`/`Build`
/// すべての辺を辿り、workspace 内の第一者パッケージ（path dependency）を除いた
/// 依存パッケージ数を `check_deps::measure_external_only` で計測し、1 件でも
/// あれば Fail とする（`check_deps::judge_zero`）。
///
/// 判定を弱める CLI 引数・環境変数は意図的に設けない（不明な引数は終了コード 2）。
/// 積集合が空（`ZERO_DEP_CRATES` の定数値が陳腐化し workspace に 1 件も実在しない）
/// 場合・計測失敗・上限超過のいずれも終了コード 1（fail-closed）とする。
fn run_check_core_deps(args: &[String]) -> ExitCode {
    if let Some(unknown) = args.first() {
        eprintln!("xtask check-core-deps: unknown argument `{unknown}` (this subcommand takes no arguments)");
        return ExitCode::from(2);
    }

    let (graph, targets, workspace_members) = match check_deps::fetch_zero_dep_targets() {
        Ok(triple) => triple,
        Err(e) => {
            eprintln!("xtask check-core-deps: failed to resolve zero-dep targets: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut had_failure = false;
    for name in targets {
        let kinds = [
            check_deps::DepKind::Normal,
            check_deps::DepKind::Dev,
            check_deps::DepKind::Build,
        ];
        // 到達可能パッケージから workspace 内の第一者パッケージ（path dependency）を
        // 除外し、真の外部依存のみを数える（reviewer 指摘: イシュー #154）。
        match check_deps::measure_external_only(&graph, &name, &kinds, &workspace_members) {
            Ok(m) => {
                let check_result = check_deps::judge_zero(m.into());
                print!("{}", check_deps::format_zero_report(&check_result));
                if !check_result.is_pass() {
                    had_failure = true;
                }
            }
            Err(e) => {
                eprintln!("xtask check-core-deps: failed to measure `{name}`: {e}");
                had_failure = true;
            }
        }
    }

    if had_failure {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// `list-build-scripts` サブコマンド: `--package <NAME>` を 1 つ以上受け取り、
/// それぞれについて到達可能な build.rs 保有クレートを列挙して stdout に表示する
/// （TASK-3.2a、`list_build_scripts` モジュール）。
///
/// 列挙自体は上限判定を伴わないため、正常に列挙できれば件数（0 件含む）によらず
/// 終了コード 0 を返す。`cargo metadata` の実行失敗・想定外の出力構造・ルート未検出は
/// fail-closed で終了コード 1（「列挙できなかったのに成功扱い」になる経路を作らない）。
/// 引数不備（`--package` 未指定・不明な引数）は `check-deps` と契約を統一し
/// 終了コード 2 とする。
fn run_list_build_scripts(args: &[String]) -> ExitCode {
    let mut packages = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--package" => {
                let Some(name) = args.get(i + 1) else {
                    eprintln!("xtask list-build-scripts: `--package` requires a value");
                    return ExitCode::from(2);
                };
                packages.push(name.clone());
                i += 2;
            }
            other => {
                eprintln!("xtask list-build-scripts: unknown argument `{other}`");
                return ExitCode::from(2);
            }
        }
    }

    if packages.is_empty() {
        eprintln!("xtask list-build-scripts: at least one `--package <NAME>` is required");
        return ExitCode::from(2);
    }

    let results = match list_build_scripts::list_many_from_cargo_metadata(&packages) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("xtask list-build-scripts: failed to run cargo metadata: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut had_failure = false;
    for (name, inventory) in results {
        match inventory {
            Ok(crates) => {
                print!("{}", list_build_scripts::format_inventory(&name, &crates));
            }
            Err(e) => {
                eprintln!("xtask list-build-scripts: failed to list `{name}`: {e}");
                had_failure = true;
            }
        }
    }

    if had_failure {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// `check-loc` サブコマンド（TASK-8.2b・イシュー #62・REQ-8 受け入れ基準、および
/// REQ-11 受け入れ基準 3・イシュー #156 の共用ゲート）: 引数を一切取らない。
/// `check_loc::LOC_CHECK_TARGETS` に列挙されたファイルそれぞれについて
/// 実効 LOC（コメント・空行を除く）を `check_loc::measure_file` で計測し、
/// `check_loc::MAX_EFFECTIVE_LOC`（10 行）以内かを `check_loc::judge` で判定する。
///
/// 判定を弱める CLI 引数・環境変数は意図的に設けない（不明な引数は終了コード 2）。
/// 対象ファイルの不在・読み取り失敗・しきい値超過のいずれも終了コード 1
/// （fail-closed）とする。
fn run_check_loc(args: &[String]) -> ExitCode {
    if let Some(unknown) = args.first() {
        eprintln!(
            "xtask check-loc: unknown argument `{unknown}` (this subcommand takes no arguments)"
        );
        return ExitCode::from(2);
    }

    let mut had_failure = false;
    for &file in check_loc::LOC_CHECK_TARGETS {
        match check_loc::measure_file(file) {
            Ok(measurement) => {
                let check_result = check_loc::judge(measurement);
                print!("{}", check_loc::format_loc_report(&check_result));
                if !check_result.is_pass() {
                    had_failure = true;
                }
            }
            Err(e) => {
                eprintln!("xtask check-loc: failed to measure `{file}`: {e}");
                had_failure = true;
            }
        }
    }

    if had_failure {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// `check-image-size` サブコマンド（TASK-9.3b, イシュー #103, REQ-9 受け入れ基準）:
/// `--image <TAG>`（必須）で指定した docker イメージの非圧縮サイズを
/// `docker image inspect` で計測し、`--limit-mb <N>`（任意、既定 50）に照らして
/// 判定する。上限の既定値は `check_image_size::REQ9_IMAGE_SIZE_LIMIT_BYTES`。
///
/// `--limit-mb` は動作確認・段階導入向けの上書きであり、CI
/// （`.github/workflows/image-size.yml`）は既定値のまま呼び出す契約とする
/// （REQ-9 の上限を弱める運用は想定しない）。
///
/// 引数不備は終了コード 2、計測失敗（docker 不在・`inspect` 失敗・パース失敗）・
/// 上限超過はいずれも終了コード 1（fail-closed）とする。ルート `Dockerfile`
/// （TASK-9.3a／イシュー #102）が未マージの間は、呼び出し元が存在しないイメージ名を
/// 渡すことになるため計測失敗＝FAIL が意図した挙動になる。
fn run_check_image_size(args: &[String]) -> ExitCode {
    let mut image: Option<String> = None;
    let mut limit_mb: Option<u64> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--image" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("xtask check-image-size: `--image` requires a value");
                    return ExitCode::from(2);
                };
                image = Some(value.clone());
                i += 2;
            }
            "--limit-mb" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("xtask check-image-size: `--limit-mb` requires a value");
                    return ExitCode::from(2);
                };
                let Ok(parsed) = value.parse::<u64>() else {
                    eprintln!("xtask check-image-size: `--limit-mb` must be a non-negative integer, got `{value}`");
                    return ExitCode::from(2);
                };
                limit_mb = Some(parsed);
                i += 2;
            }
            other => {
                eprintln!("xtask check-image-size: unknown argument `{other}`");
                return ExitCode::from(2);
            }
        }
    }

    let Some(image) = image else {
        eprintln!("xtask check-image-size: `--image <TAG>` is required");
        return ExitCode::from(2);
    };

    // 10 進 MB（docker CLI の表示単位と一致させる。check_image_size モジュールの
    // rustdoc 参照）。`--limit-mb` 未指定時は REQ-9 既定値をそのまま使う。
    let limit_bytes = match limit_mb {
        Some(mb) => mb.saturating_mul(1_000_000),
        None => check_image_size::REQ9_IMAGE_SIZE_LIMIT_BYTES,
    };

    let measurement = match check_image_size::measure(&image) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("xtask check-image-size: failed to measure `{image}`: {e}");
            return ExitCode::FAILURE;
        }
    };

    let check_result = check_image_size::judge(measurement, limit_bytes);
    print!("{}", check_image_size::format_report(&check_result));

    if check_result.is_pass() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// `wasm-node-smoke` サブコマンド（イシュー #297, TASK-10.2 残課題）: 引数は
/// `--build-only`（任意）のみ受け付ける。`wasm_node_smoke::run` に処理を委譲し、
/// 成否を 1 行サマリ（`wasm_node_smoke::format_report`）として stdout に出す。
///
/// 判定対象クレート（`wasm_node_smoke::PACKAGE_NAME`）・出力先は定数固定で
/// CLI からの差し替え不可（`check-loc`/`check-core-deps` と同じ設計原則）。
/// `--build-only` 以外の不明な引数は終了コード 2（usage エラー）とし、
/// ツール不在・バージョン不一致・ビルド失敗・bindgen 失敗・node 実行失敗・
/// エスケープ検証失敗はいずれも終了コード 1（fail-closed）とする。
fn run_wasm_node_smoke(args: &[String]) -> ExitCode {
    let mut mode = wasm_node_smoke::SmokeMode::Full;
    for arg in args {
        match arg.as_str() {
            "--build-only" => mode = wasm_node_smoke::SmokeMode::BuildOnly,
            other => {
                eprintln!("xtask wasm-node-smoke: unknown argument `{other}`");
                return ExitCode::from(2);
            }
        }
    }

    match wasm_node_smoke::run(mode) {
        Ok(()) => {
            print!("{}", wasm_node_smoke::format_report(mode, true));
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("xtask wasm-node-smoke: {e}");
            print!("{}", wasm_node_smoke::format_report(mode, false));
            ExitCode::FAILURE
        }
    }
}

/// `bench-binding-update` サブコマンド（イシュー #592）: 引数を取らない。
/// [`bench_binding_update::run_all_scenarios`] を実行し、シナリオごとに
/// 1 行サマリ（[`bench_binding_update::ScenarioReport`] の `Display`）を
/// stdout へ出力する。
///
/// 計測値は実行環境依存で非決定的なため report-only（しきい値判定なし）とし、
/// 終了コードは常に `ExitCode::SUCCESS`（計測自体が実行できたことのみを
/// 保証する。CI ゲート化はしない設計、`bench_binding_update` モジュール doc
/// 参照）。不明な引数のみ終了コード 2（usage エラー）とする。
fn run_bench_binding_update(args: &[String]) -> ExitCode {
    if let Some(unknown) = args.first() {
        eprintln!(
            "xtask bench-binding-update: unknown argument `{unknown}` (this subcommand takes no arguments)"
        );
        return ExitCode::from(2);
    }

    for report in bench_binding_update::run_all_scenarios() {
        println!("{report}");
    }

    ExitCode::SUCCESS
}

/// `bench-ssr` サブコマンド（イシュー #1317）: 任意の `--baseline <FILE>` のみを
/// 受け取る。計測 → `fandhe-frontend-core` の実バージョン解決（`cargo metadata`）
/// → JSON 1 行の出力 → 既定エスケープ（REQ-1）/行数一致の検証（fail-closed）→
/// （`--baseline` 指定時のみ）report-only な回帰比較出力、の順に実行する。
///
/// 終了コード: 0=検証 PASS（`--baseline` 比較の有無に関わらず）/
/// 1=検証 FAIL・環境エラー（`cargo metadata` 失敗）・baseline 不正
/// （ファイル読み取り失敗・JSON パース失敗・必須キー欠落）/
/// 2=引数不備。計測回数・行数を差し替える CLI 引数は意図的に設けない
/// （`bench_ssr` モジュール doc 参照）。
fn run_bench_ssr(args: &[String]) -> ExitCode {
    let mut baseline_path: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--baseline" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("xtask bench-ssr: `--baseline` requires a value");
                    return ExitCode::from(2);
                };
                baseline_path = Some(path.clone());
                i += 2;
            }
            other => {
                eprintln!("xtask bench-ssr: unknown argument `{other}`");
                return ExitCode::from(2);
            }
        }
    }

    let version = match bench_ssr::resolve_core_version() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("xtask bench-ssr: {e}");
            return ExitCode::FAILURE;
        }
    };

    let report = bench_ssr::run(version);
    println!("{}", report.to_json_line());

    if let Some(path) = baseline_path {
        let baseline_json = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("xtask bench-ssr: failed to read baseline file `{path}`: {e}");
                return ExitCode::FAILURE;
            }
        };
        match bench_ssr::compare(&report, &baseline_json) {
            Ok(lines) => {
                for line in lines {
                    println!("{line}");
                }
            }
            Err(e) => {
                eprintln!("xtask bench-ssr: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    if !report.self_check_ok() {
        eprintln!(
            "xtask bench-ssr: self-check failed (escape_ok={}, row_count_ok={})",
            report.escape_ok, report.row_count_ok
        );
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

/// `bench-state-update` サブコマンド（イシュー #1328）: 任意の `--baseline <FILE>`
/// のみを受け取る。計測 → `fandhe-frontend-interactive` の実バージョン解決
/// （`cargo metadata`）→ JSON 1 行の出力 → 既定エスケープ（REQ-1）/no-op
/// 契約の検証（fail-closed）→（`--baseline` 指定時のみ）report-only な
/// 回帰比較出力、の順に実行する（`run_bench_ssr` と同型の構成）。
///
/// 終了コード: 0=検証 PASS（`--baseline` 比較の有無に関わらず）/
/// 1=検証 FAIL・環境エラー（`cargo metadata` 失敗）・baseline 不正
/// （ファイル読み取り失敗・JSON パース失敗・必須キー欠落）/
/// 2=引数不備。計測回数・束縛点数を差し替える CLI 引数は意図的に設けない
/// （`bench_state_update` モジュール doc 参照）。
fn run_bench_state_update(args: &[String]) -> ExitCode {
    let mut baseline_path: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--baseline" => {
                let Some(path) = args.get(i + 1) else {
                    eprintln!("xtask bench-state-update: `--baseline` requires a value");
                    return ExitCode::from(2);
                };
                baseline_path = Some(path.clone());
                i += 2;
            }
            other => {
                eprintln!("xtask bench-state-update: unknown argument `{other}`");
                return ExitCode::from(2);
            }
        }
    }

    let version = match bench_state_update::resolve_interactive_version() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("xtask bench-state-update: {e}");
            return ExitCode::FAILURE;
        }
    };

    let report = bench_state_update::run(version);
    println!("{}", report.to_json_line());

    if let Some(path) = baseline_path {
        let baseline_json = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("xtask bench-state-update: failed to read baseline file `{path}`: {e}");
                return ExitCode::FAILURE;
            }
        };
        match bench_state_update::compare(&report, &baseline_json) {
            Ok(lines) => {
                for line in lines {
                    println!("{line}");
                }
            }
            Err(e) => {
                eprintln!("xtask bench-state-update: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    if !report.self_check_ok() {
        eprintln!(
            "xtask bench-state-update: self-check failed (escape_ok={}, noop_ok={})",
            report.escape_ok, report.noop_ok
        );
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

/// `check-version-bump` サブコマンド（イシュー #638）: `--base-ref <REF>`
/// （必須）・`--pr-body-file <PATH>`（任意、免除宣言の読み取り元）・
/// `--index-base-url <URL>`（任意、既定 [`check_version_bump::DEFAULT_INDEX_BASE_URL`]、
/// テスト専用の差し替え口）を受け取る。
///
/// 変更ファイルが 1 つも公開対象クレートの実体に触れていなければ
/// 即座に PASS 扱い（終了コード 0）とする。`query_index` が
/// `CheckVersionBumpError::EnvironmentError` を返した場合は、以降のクレートを
/// 判定せず直ちに終了コード 1 で打ち切る（fail-closed。環境要因の失敗を
/// 個別クレートの FAIL と混在させない）。引数不備は終了コード 2、
/// バンプ漏れ検知・環境エラーはいずれも終了コード 1。
fn run_check_version_bump(args: &[String]) -> ExitCode {
    let mut base_ref: Option<String> = None;
    let mut pr_body_file: Option<String> = None;
    let mut index_base_url = check_version_bump::DEFAULT_INDEX_BASE_URL.to_string();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--base-ref" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("xtask check-version-bump: `--base-ref` requires a value");
                    return ExitCode::from(2);
                };
                base_ref = Some(value.clone());
                i += 2;
            }
            "--pr-body-file" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("xtask check-version-bump: `--pr-body-file` requires a value");
                    return ExitCode::from(2);
                };
                pr_body_file = Some(value.clone());
                i += 2;
            }
            "--index-base-url" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("xtask check-version-bump: `--index-base-url` requires a value");
                    return ExitCode::from(2);
                };
                index_base_url = value.clone();
                i += 2;
            }
            other => {
                eprintln!("xtask check-version-bump: unknown argument `{other}`");
                return ExitCode::from(2);
            }
        }
    }

    let Some(base_ref) = base_ref else {
        eprintln!("xtask check-version-bump: `--base-ref <REF>` is required");
        return ExitCode::from(2);
    };

    // `--pr-body-file` 未指定時は免除なし（fail-closed）。空文字列扱いにするのは
    // 誤って全クレートを免除してしまう経路を作らないため（計画書 §3.1 参照）。
    let exempt_crates = match &pr_body_file {
        Some(path) => match std::fs::read_to_string(path) {
            Ok(body) => check_version_bump::parse_exempt_crates(&body),
            Err(e) => {
                eprintln!(
                    "xtask check-version-bump: failed to read `--pr-body-file` `{path}`: {e}"
                );
                return ExitCode::FAILURE;
            }
        },
        None => std::collections::HashSet::new(),
    };

    let crates = match check_version_bump::published_crates_from_cargo_metadata() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("xtask check-version-bump: {e}");
            return ExitCode::FAILURE;
        }
    };

    let files = match check_version_bump::changed_files(&base_ref) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("xtask check-version-bump: {e}");
            return ExitCode::FAILURE;
        }
    };

    let affected = check_version_bump::affected_crates(&files, &crates);
    if affected.is_empty() {
        println!("version-bump-check: no published crate sources changed (base-ref={base_ref})");
        return ExitCode::SUCCESS;
    }

    let mut had_failure = false;
    for c in affected {
        let exempt = exempt_crates.contains(&c.name);
        let lookup = match check_version_bump::query_index(&index_base_url, &c.name) {
            Ok(l) => l,
            Err(e) => {
                // 環境要因の失敗は個別クレートの FAIL とは区別し、以降の
                // クレートを判定せず直ちに打ち切る（CI 側が
                // "environment error: " プレフィックスで区別できるようにする）。
                eprintln!("xtask check-version-bump: {e}");
                return ExitCode::FAILURE;
            }
        };
        let published = matches!(lookup, check_version_bump::IndexLookup::Published(_));
        let judgement = check_version_bump::judge(&c.version, exempt, &lookup);
        let report = check_version_bump::Report {
            name: c.name.clone(),
            version: c.version.clone(),
            published,
            judgement,
        };
        print!("{}", check_version_bump::format_report(&report));
        if !judgement.is_pass() {
            had_failure = true;
            eprintln!(
                "xtask check-version-bump: crate `{name}` version {version} is already \
published on crates.io but its sources changed in this PR. Fix by either (a) bumping the \
version in the crate's Cargo.toml (0.x breaking changes: bump the minor version; then run \
`cargo run -p xtask -- check-dep-versions --fix` to auto-sync dependents' `version = \"...\"` \
requirements), or (b) declaring `version-bump-exempt: {name}` (with rationale) in the PR body \
if this change is not a public-API-breaking change.",
                name = c.name,
                version = c.version,
            );
        }
    }

    if had_failure {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// `check-dep-versions` サブコマンド（イシュー #657）: `--fix`（任意）のみを
/// 受け付ける。既定（`--fix` なし）は検知のみで、workspace 内の
/// `path + version` 併記依存すべてについて 1 行サマリを出力し、1 件でも
/// FAIL（ルール 1: version 不一致 / ルール 2: 公開対象クレートの version 欠落）
/// があれば終了コード 1（fail-closed）を返す（呼び出し元:
/// `.github/workflows/ci.yml` の `dep-version-check` ジョブ）。
///
/// `--fix` はルール 1 の FAIL のみをその場で自動修正するローカル向けオプトイン
/// 手段。[`check_dep_versions::plan_fixes`] が 1 件でも書き換え位置を一意特定
/// できなければ（未対応の req 形式・候補 0/複数件）**一切書き換えを行わず**
/// エラーとして打ち切る（部分書き込みをしない）。修正適用後は
/// `cargo metadata` を再実行して検知をやり直し、PASS を確認できて初めて
/// 終了コード 0 を返す。ルール 2 の FAIL は `--fix` でも修正されず、残留して
/// いれば終了コード 1 のままとなる（`docs` 上の設計どおり、`cargo publish` が
/// 実際に失敗する構成を安易に隠さないため）。
fn run_check_dep_versions(args: &[String]) -> ExitCode {
    let mut fix = false;
    for arg in args {
        match arg.as_str() {
            "--fix" => fix = true,
            other => {
                eprintln!("xtask check-dep-versions: unknown argument `{other}`");
                return ExitCode::from(2);
            }
        }
    }

    let (workspace_root, members) =
        match check_dep_versions::workspace_packages_from_cargo_metadata() {
            Ok(v) => v,
            Err(e) => {
                eprintln!("xtask check-dep-versions: {e}");
                return ExitCode::FAILURE;
            }
        };

    let edges = check_dep_versions::collect_edges(&members);
    let reports: Vec<check_dep_versions::Report> =
        edges.iter().map(check_dep_versions::judge_edge).collect();
    for report in &reports {
        print!("{}", check_dep_versions::format_report(report));
    }

    let mismatch_present = reports.iter().any(|r| {
        matches!(
            r.judgement,
            check_dep_versions::Judgement::Fail(check_dep_versions::FailReason::VersionMismatch)
        )
    });
    let missing_present = reports.iter().any(|r| {
        matches!(
            r.judgement,
            check_dep_versions::Judgement::Fail(check_dep_versions::FailReason::MissingVersion)
        )
    });

    if !fix {
        if mismatch_present || missing_present {
            eprintln!(
                "xtask check-dep-versions: one or more workspace-internal `version = \"...\"` \
requirements are out of sync with their dependency's current version, or missing where \
required for a publishable crate. Run `cargo run -p xtask -- check-dep-versions --fix` to \
auto-correct version-mismatch failures (missing-version failures must be fixed by hand: add an \
explicit `version = \"...\"`)."
            );
            return ExitCode::FAILURE;
        }
        return ExitCode::SUCCESS;
    }

    // `--fix`: ルール 1 の FAIL がなければ何もしない（ルール 2 は対象外）。
    if !mismatch_present {
        if missing_present {
            eprintln!(
                "xtask check-dep-versions --fix: missing-version failures cannot be auto-fixed; \
add an explicit `version = \"...\"` by hand."
            );
            return ExitCode::FAILURE;
        }
        return ExitCode::SUCCESS;
    }

    let plans = match check_dep_versions::plan_fixes(&workspace_root, &edges) {
        Ok(p) => p,
        Err(errors) => {
            for e in errors {
                eprintln!("xtask check-dep-versions --fix: {e}");
            }
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = check_dep_versions::apply_fixes(&plans) {
        eprintln!("xtask check-dep-versions --fix: {e}");
        return ExitCode::FAILURE;
    }

    // 書き換え後、ディスクから再度読み直して検知をやり直し、PASS を確認する。
    let (_, members_after) = match check_dep_versions::workspace_packages_from_cargo_metadata() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("xtask check-dep-versions --fix: post-fix re-check failed: {e}");
            return ExitCode::FAILURE;
        }
    };
    let edges_after = check_dep_versions::collect_edges(&members_after);
    let reports_after: Vec<check_dep_versions::Report> = edges_after
        .iter()
        .map(check_dep_versions::judge_edge)
        .collect();
    println!("xtask check-dep-versions --fix: re-checking after applying fixes");
    for report in &reports_after {
        print!("{}", check_dep_versions::format_report(report));
    }

    let still_failing = reports_after.iter().any(|r| !r.judgement.is_pass());
    if still_failing {
        eprintln!(
            "xtask check-dep-versions --fix: some failures remain after --fix (missing-version \
failures are not auto-fixable)."
        );
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// `patch-template-smoke` サブコマンド（イシュー #885）: `--project-dir <DIR>`・
/// `--repo-root <DIR>`（いずれも必須）・`--index-base-url <URL>`（任意、既定
/// [`check_version_bump::DEFAULT_INDEX_BASE_URL`]、テスト専用の差し替え口）を
/// 受け取る。
///
/// `<project-dir>/Cargo.toml`（必須。読み取れなければ即失敗）・
/// `<project-dir>/wasm/Cargo.toml`（存在すれば処理、なければ読み飛ばす）の
/// 2 マニフェストを [`patch_template_smoke::process_manifest`] で処理する。
/// crates.io sparse index への到達性起因のエラー
/// （[`patch_template_smoke::PatchTemplateSmokeError::Environment`]）は
/// `environment error: ` プレフィックス付きで即座に打ち切る（`check-version-bump`
/// と同じ fail-closed 区別規約）。引数不備は終了コード 2、それ以外の失敗
/// （既存 `[patch.crates-io]`・path 依存検出・repo-root 側クレート不整合・
/// 環境エラー）は終了コード 1。
///
/// 呼び出し元は 2 経路（イシュー #895）: (1) `.github/workflows/ci.yml` の
/// `template-app-wasm-smoke` ジョブ（イシュー #885、当初の導入経路）、
/// (2) `crates/cli/tests/new_gate_e2e.rs::apply_patch_template_smoke`
/// （app テンプレート gate e2e、version-bump-guard・`template_vendor_drift`
/// との三すくみが smoke ジョブと同型で再発するため）。CLI 契約・判定ロジック
/// は両経路で共用する。
fn run_patch_template_smoke(args: &[String]) -> ExitCode {
    let mut project_dir: Option<String> = None;
    let mut repo_root: Option<String> = None;
    let mut index_base_url = check_version_bump::DEFAULT_INDEX_BASE_URL.to_string();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--project-dir" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("xtask patch-template-smoke: `--project-dir` requires a value");
                    return ExitCode::from(2);
                };
                project_dir = Some(value.clone());
                i += 2;
            }
            "--repo-root" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("xtask patch-template-smoke: `--repo-root` requires a value");
                    return ExitCode::from(2);
                };
                repo_root = Some(value.clone());
                i += 2;
            }
            "--index-base-url" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("xtask patch-template-smoke: `--index-base-url` requires a value");
                    return ExitCode::from(2);
                };
                index_base_url = value.clone();
                i += 2;
            }
            other => {
                eprintln!("xtask patch-template-smoke: unknown argument `{other}`");
                return ExitCode::from(2);
            }
        }
    }

    let Some(project_dir) = project_dir else {
        eprintln!("xtask patch-template-smoke: `--project-dir <DIR>` is required");
        return ExitCode::from(2);
    };
    let Some(repo_root) = repo_root else {
        eprintln!("xtask patch-template-smoke: `--repo-root <DIR>` is required");
        return ExitCode::from(2);
    };

    let project_dir = std::path::Path::new(&project_dir);
    let repo_root = std::path::Path::new(&repo_root);

    let manifests = [
        (
            project_dir.join("Cargo.toml"),
            project_dir.join("Cargo.lock"),
        ),
        (
            project_dir.join("wasm").join("Cargo.toml"),
            project_dir.join("wasm").join("Cargo.lock"),
        ),
    ];

    let mut any_patched = false;
    for (idx, (manifest_path, lock_path)) in manifests.iter().enumerate() {
        if !manifest_path.exists() {
            // ルート Cargo.toml（idx 0）は必須。wasm/Cargo.toml（idx 1）は
            // `fw new --template app` 以外の生成物では存在しないことがあり
            // 得るため読み飛ばす。
            if idx == 0 {
                eprintln!(
                    "xtask patch-template-smoke: required manifest not found: {path}",
                    path = manifest_path.display()
                );
                return ExitCode::FAILURE;
            }
            continue;
        }

        match patch_template_smoke::process_manifest(
            manifest_path,
            lock_path,
            repo_root,
            &index_base_url,
        ) {
            Ok(outcome) => {
                for report in &outcome.reports {
                    print!("{}", patch_template_smoke::format_dep_report(report));
                }
                if outcome.patched {
                    any_patched = true;
                    println!(
                        "xtask patch-template-smoke: {path} was patched with a \
`[patch.crates-io]` fallback and its Cargo.lock was removed; build reproducibility \
(--locked-equivalent determinism) is reduced for this manifest until the pending crate \
version is published on crates.io",
                        path = manifest_path.display()
                    );
                }
            }
            Err(e) => {
                eprintln!("xtask patch-template-smoke: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    if any_patched {
        println!(
            "xtask patch-template-smoke: fallback engaged for one or more dependencies (see \
`resolution=path-override` lines above); this is expected only until the corresponding \
crates.io publish (release.yml, workflow_dispatch, mode: publish) completes"
        );
    }

    ExitCode::SUCCESS
}
