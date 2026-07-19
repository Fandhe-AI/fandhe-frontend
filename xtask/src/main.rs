//! `xtask`: このワークスペースの CI 計測・自己保守用ツール群のエントリポイント。
//! 開発者用ツールであり、配布物（rws-* クレート）には含めない。
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
//!   `check-deps --package rws-core` の 60/6 判定とは別に「ゼロであること」を
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
//! `core` / `interactive` と異なりプロセス起動（`std::process::Command`）を行うが、
//! `unsafe` は使わない（REQ-2 は core/interactive 限定だが、xtask でも forbid する。
//! core/tests/unsafe_boundary.rs の WASM/FFI 境界許可リストにも含まれない）。

#![forbid(unsafe_code)]

mod check_deps;
mod check_image_size;
mod check_loc;
mod json;
mod list_build_scripts;
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
    eprintln!("      docs/design/wasm-build-integration.md §6.4 for rws-wasm-thin: verify the");
    eprintln!("      installed wasm-bindgen-cli matches Cargo.lock's wasm-bindgen version,");
    eprintln!("      build for wasm32-unknown-unknown, generate `--target nodejs` bindings,");
    eprintln!("      and (unless --build-only) run a node smoke check including a default-");
    eprintln!("      escape (REQ-1) regression check (issue #297).");
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
