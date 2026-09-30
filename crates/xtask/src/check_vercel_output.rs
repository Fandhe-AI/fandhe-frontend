//! `examples/vercel-ssg`（イシュー #3290）が生成する Vercel Build Output
//! API v3 出力ツリー（`.vercel/output/config.json` + `static/`）の構造を
//! 機械検証するモジュール（イシュー #3292）。
//!
//! # 呼び出し文脈
//!
//! `.github/workflows/ci.yml` の `test` ジョブが「examples gate e2e 7 件」
//! ステップの直後に、`examples/vercel-ssg` を `cargo run` した実生成物を
//! 本モジュール（`xtask check-vercel-output --output-dir <DIR>`）へ渡す。
//! Vercel への実デプロイは行わない（構造検証のみ）。
//!
//! # 既存検証とのギャップ
//!
//! `examples/vercel-ssg/tests/build_output.rs` と
//! `crates/cli/tests/new_gate_e2e.rs::fw_new_example_vercel_ssg_output_passes_fw_gate`
//! は `config.json` を部分文字列一致（`"version": 3` を含むか等）でしか
//! 見ていない。JSON として壊れていても、`version` が文字列 `"3"` でも、
//! `routes` の要素の型・キーが誤っていても（例: `"stauts"` という typo）
//! 通ってしまう。本モジュールは JSON としてパースし、型・キー・参照整合
//! （`routes[].dest` が指す実ファイルの存在）まで構造的に検証する。
//!
//! # 参照仕様
//!
//! Vercel Build Output API v3 の `config.json` トップレベルキー・`routes`
//! の各キーの一覧は次の公式ドキュメントを 2026-09-27 に確認して転記した
//! （ページの `last_updated` メタは 2026-07-27）:
//! <https://vercel.com/docs/build-output-api/v3/configuration>
//! （リダイレクト先の正規 URL は
//! `https://vercel.com/docs/build-output-api/configuration`）。
//!
//! Source route のキーのうち `has` / `missing` / `locale` /
//! `middlewareRawSrc` / `middlewarePath` / `mitigate` / `transforms` は
//! キーの許可リスト・浅い型（配列/オブジェクト/文字列の別）のみを検証し、
//! ネストした構造（`HasField`・`Locale`・`Mitigate`・`Transform` 等）までは
//! 検証しない。`examples/vercel-ssg` の [`crate::CONFIG_JSON`]（`src/main.rs`
//! 参照）はこれらを使わないため実害はないが、将来これらを使う example が
//! 追加された場合は本モジュールの拡張を検討すること
//! （`out-of-scope-tracking.md`）。
//!
//! # `functions/` の検証モード（イシュー #3344）
//!
//! `examples/vercel-ssg` は案 c（`docs/design/vercel-deployment-strategy.md`
//! が確定した「SSG → Build Output API → `--prebuilt`」方式）の実演であり、
//! Vercel 側で Rust ランタイムを実行しない。既定モード（[`FunctionsExpectation::None`]）
//! は「`functions/` が存在しないこと」を検証する（[`check_no_functions_dir`]）。
//! 一方 `examples/vercel-ssg` はイシュー #3343 で opt-in の Basic 認証
//! Routing Middleware（Edge Function、`FANDHE_VERCEL_SSG_BASIC_AUTH=1` で
//! 有効化）を持ち、有効時は `functions/_middleware.func/` を生成する。
//! `--expect-basic-auth-middleware`（`crate::main`）を渡すと
//! [`FunctionsExpectation::BasicAuthMiddleware`] に切り替わり、`functions/`
//! の構成・`.vc-config.json`・`index.js`・`config.json` 先頭のミドルウェア
//! ルートを構造的に検証する（本節末尾「Basic 認証ミドルウェア出力の検証
//! 内容」参照）。既定モードの判定結果（チェック名・順序・PASS/FAIL 条件）は
//! 変更しない。
//!
//! ## Basic 認証ミドルウェア出力の検証内容
//!
//! - [`check_routes_middleware_first`]: `routes[0]` が
//!   `{"src": "/(.*)", "middlewarePath": "_middleware", "continue": true}`
//!   の形（`handle` キーを持たない）であり、`middlewarePath` を持つルートが
//!   `routes[0]` 以外に存在しないことを検証する。
//! - [`check_functions_dir`]: `<output>/functions` が存在するディレクトリで
//!   シンボリックリンクでないことを検証する。
//! - [`check_functions_layout`]: `functions/` 直下のエントリが
//!   `_middleware.func`（ディレクトリ、非シンボリックリンク）1 件のみで
//!   あることを検証する。
//! - [`check_functions_no_symlinks`]: `functions/` 以下にシンボリックリンクが
//!   ないことを検証する（[`find_symlink`] を再利用）。
//! - [`check_middleware_vc_config`]: `_middleware.func/.vc-config.json` が
//!   JSON オブジェクトとしてパースでき、`runtime == "edge"` であること
//!   （`entrypoint` があれば文字列 `"index.js"` と一致すること）を検証する。
//! - [`check_middleware_index_js`]: `_middleware.func/index.js` が通常
//!   ファイルとして存在し、空でないことを検証する（内容は読まない）。
//!
//! `.vc-config.json` の全キーの許可リスト検証・`index.js` の内容検証は
//! スコープ外とする（`out-of-scope-tracking.md`）。
//!
//! # セキュリティ不変条件（OWASP Top 10、`security.md` 参照）
//!
//! - **A01 パストラバーサル**: `routes[].dest` を `static/` へ結合する前に、
//!   構成要素に `..`・ルート・プレフィックス・`\`・空要素が含まれないことを
//!   検証する（[`is_safe_relative_dest`]）。`static/`・`functions/` の走査は
//!   シンボリックリンクをたどらず（`symlink_metadata`）、リンクを検出したら
//!   FAIL にする（[`static_no_symlinks`]・[`check_functions_no_symlinks`]）。
//!   出力ディレクトリ自体がシンボリックリンクの場合も FAIL にする
//!   （[`output_dir`]）。走査対象パス（`functions`・`_middleware.func`・
//!   `.vc-config.json`・`index.js`）は固定リテラルのみで、JSON の値
//!   （`middlewarePath`・`entrypoint` 等）からパスを組み立てることはない。
//!   本モジュールはファイルを削除・書き込みしない（読み取り専用）。
//! - **A05 / DoS 耐性**: `config.json`・`.vc-config.json` の読み込みに
//!   [`MAX_CONFIG_JSON_BYTES`] の上限を設ける。`json` モジュールの
//!   `MAX_DEPTH` によるネスト上限を引き継ぐ。ディレクトリ走査にも深さ上限
//!   （[`MAX_WALK_DEPTH`]）を設ける。`functions_layout` の FAIL detail に
//!   列挙する余分なエントリ名にも件数上限を設ける。
//! - **A08 ソフトウェアとデータの整合性**: `.vc-config.json` の
//!   `runtime == "edge"` と `entrypoint` の整合を確認し、意図しないランタイム
//!   への差し替えを検知する。`middlewarePath` を持つルートが `routes[0]`
//!   だけであることを確認し、参照先のないミドルウェアや二重のミドルウェア
//!   参照を検知する。
//! - **A09 機微情報の露出防止**: 失敗メッセージにはチェック名・相対パス・
//!   キー名・理由だけを出し、ファイル全文・環境変数・トークンは出さない
//!   （`json.rs::JsonError` と同じ方針）。`.vc-config.json` の `envVarsInUse`
//!   等の値全体・`index.js` の内容は出力しない。
//! - **fail-closed**: 前段のチェックが失敗して後段が判定できない場合
//!   （`config.json` がパースできない・`functions/` が存在しない等）、
//!   後段は黙って PASS にせず `result=FAIL detail=skipped: depends on <name>`
//!   として明示する。全チェック名は [`check_names`] で一元管理し、直書きの
//!   skipped 一覧で漏れが生じないようにする。

use crate::json::{self, Json};
use std::fmt;
use std::path::{Component, Path, PathBuf};

/// `config.json` の読み込みサイズ上限（1 MiB）。DoS 防御（A05）。
const MAX_CONFIG_JSON_BYTES: u64 = 1024 * 1024;

/// `static/` 再帰走査の深さ上限。DoS 防御（A05、深すぎるディレクトリ木への対策）。
const MAX_WALK_DEPTH: usize = 32;

/// `examples/vercel-ssg` の受け入れ基準（#3290）に対応する必須静的ページ
/// （出力ディレクトリの `static/` からの相対パス）。判定対象は CLI 引数で
/// 差し替え不可（`check_loc::LOC_CHECK_TARGETS` と同じ運用原則）。
pub const REQUIRED_STATIC_PAGES: &[&str] = &[
    "index.html",
    "404.html",
    "pages/about/index.html",
    "pages/default-escaping/index.html",
];

/// `config.json` トップレベルで許可するキー（v3 仕様、モジュール doc の
/// 参照 URL 参照）。
const ALLOWED_TOP_LEVEL_KEYS: &[&str] = &[
    "version",
    "routes",
    "images",
    "wildcard",
    "overrides",
    "cache",
    "framework",
    "crons",
    "services",
];

/// Handler route（`{"handle": ...}`）で許可される `handle` の値。
const ALLOWED_HANDLE_VALUES: &[&str] =
    &["rewrite", "filesystem", "resource", "miss", "hit", "error"];

/// Handler route で許可されるキー（`handle` 必須 + 任意）。
const ALLOWED_HANDLER_KEYS: &[&str] = &["handle", "src", "dest", "status"];

/// `functions_layout` の FAIL detail に列挙する余分なエントリ名の上限
/// （A05 DoS 防御。大量エントリでもログが際限なく膨らまないようにする）。
const MAX_LISTED_EXTRA_ENTRIES: usize = 5;

/// ミドルウェア Function ディレクトリの固定名
/// （`examples/vercel-ssg` の `CONFIG_JSON_WITH_BASIC_AUTH`/
/// `MIDDLEWARE_VC_CONFIG_JSON` が生成する構成と一致させる）。
const MIDDLEWARE_FUNC_DIR_NAME: &str = "_middleware.func";

/// ミドルウェア Function のエントリポイントファイル名。
const MIDDLEWARE_INDEX_JS_NAME: &str = "index.js";

/// ミドルウェアが起動する `config.json` の `middlewarePath` 値。
const MIDDLEWARE_PATH_VALUE: &str = "_middleware";

/// `check-vercel-output` が検証する `functions/` の期待形（イシュー #3344）。
///
/// CLI フラグ（`--expect-basic-auth-middleware`）はこの固定された 2 値の
/// どちらを使うかを選ぶだけで、判定対象そのもの（パス・許可リスト・値）は
/// 引き続き CLI から差し替え不可（`check_loc::LOC_CHECK_TARGETS` と同じ
/// 運用原則、モジュール doc 参照）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FunctionsExpectation {
    /// 既定: `functions/` が存在しないこと（案 c、静的配置のみ）を検証する。
    #[default]
    None,
    /// `examples/vercel-ssg` の opt-in Basic 認証 Routing Middleware
    /// （イシュー #3343）が生成する `functions/_middleware.func/` 構造を
    /// 検証する。
    BasicAuthMiddleware,
}

/// `expectation` が出力する全チェック名を、出力順で返す（一元管理。
/// [`check`] 内の直書き配列に代わるもので、fail-closed の skipped 一覧が
/// モードの追加時に漏れないようにする）。既定モード（[`FunctionsExpectation::None`]）
/// の並びは旧実装の 11 件と完全一致させる（R1 の回帰固定、
/// `tests::check_names_none_matches_legacy_order` 参照）。
fn check_names(expectation: FunctionsExpectation) -> &'static [&'static str] {
    match expectation {
        FunctionsExpectation::None => &[
            "output_dir",
            "config_json_parse",
            "config_version",
            "config_top_level_keys",
            "config_routes_shape",
            "config_routes_404_fallback",
            "routes_dest_targets",
            "static_dir",
            "static_required_pages",
            "static_no_symlinks",
            "no_functions_dir",
        ],
        FunctionsExpectation::BasicAuthMiddleware => &[
            "output_dir",
            "config_json_parse",
            "config_version",
            "config_top_level_keys",
            "config_routes_shape",
            "config_routes_404_fallback",
            "routes_dest_targets",
            "config_routes_middleware_first",
            "static_dir",
            "static_required_pages",
            "static_no_symlinks",
            "functions_dir",
            "functions_layout",
            "functions_no_symlinks",
            "middleware_vc_config",
            "middleware_index_js",
        ],
    }
}

/// `config_json_parse` の成否に判定が依存するチェック名を返す
/// （`config_json_parse` が FAIL したときに一括で skipped にする対象）。
fn config_dependent_names(expectation: FunctionsExpectation) -> &'static [&'static str] {
    match expectation {
        FunctionsExpectation::None => &[
            "config_version",
            "config_top_level_keys",
            "config_routes_shape",
            "config_routes_404_fallback",
            "routes_dest_targets",
        ],
        FunctionsExpectation::BasicAuthMiddleware => &[
            "config_version",
            "config_top_level_keys",
            "config_routes_shape",
            "config_routes_404_fallback",
            "routes_dest_targets",
            "config_routes_middleware_first",
        ],
    }
}

/// Source route で許可されるキー（`src` 必須 + 任意）。
const ALLOWED_SOURCE_KEYS: &[&str] = &[
    "src",
    "dest",
    "headers",
    "methods",
    "continue",
    "caseSensitive",
    "check",
    "status",
    "has",
    "missing",
    "locale",
    "middlewareRawSrc",
    "middlewarePath",
    "mitigate",
    "transforms",
];

/// 1 チェックの結果。CI の 1 行サマリ・Step Summary 転記（[`format_line`]）
/// が読む契約のため、フィールドの意味は変更しない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    pub name: &'static str,
    pub passed: bool,
    /// FAIL の理由。ファイル全文・環境変数は含めない（A09）。
    pub detail: Option<String>,
}

impl CheckResult {
    fn pass(name: &'static str) -> Self {
        CheckResult {
            name,
            passed: true,
            detail: None,
        }
    }

    fn fail(name: &'static str, detail: impl Into<String>) -> Self {
        CheckResult {
            name,
            passed: false,
            detail: Some(detail.into()),
        }
    }

    /// 前段チェック失敗により本チェックが判定不能なことを表す
    /// fail-closed 専用コンストラクタ（黙って PASS にしない）。
    fn skipped(name: &'static str, depends_on: &'static str) -> Self {
        CheckResult::fail(name, format!("skipped: depends on {depends_on}"))
    }
}

/// `output_dir` に対して全チェックを順に実行する。
///
/// `expectation` は `functions/` の検証モードを選ぶ（[`FunctionsExpectation`]
/// 参照）。各チェックは独立した `CheckResult` を返す（例外的に、前段の失敗で
/// 後段が判定不能な場合は [`CheckResult::skipped`] を返し、黙って PASS に
/// しない）。
pub fn check(output_dir: &Path, expectation: FunctionsExpectation) -> Vec<CheckResult> {
    let mut results = Vec::new();

    // 1. output_dir 自体の検証（シンボリックリンク経由の誤参照を拒否）。
    let output_dir_ok = match std::fs::symlink_metadata(output_dir) {
        Ok(meta) if meta.file_type().is_symlink() => {
            results.push(CheckResult::fail(
                "output_dir",
                "output directory must not be a symlink",
            ));
            false
        }
        Ok(meta) if meta.is_dir() => {
            results.push(CheckResult::pass("output_dir"));
            true
        }
        Ok(_) => {
            results.push(CheckResult::fail("output_dir", "not a directory"));
            false
        }
        Err(err) => {
            results.push(CheckResult::fail("output_dir", format!("{err}")));
            false
        }
    };

    if !output_dir_ok {
        // output_dir 自体が不正なら以降すべて判定不能。
        for name in check_names(expectation).iter().skip(1) {
            results.push(CheckResult::skipped(name, "output_dir"));
        }
        return results;
    }

    // 2. config.json の読み込み・パース。
    let config_path = output_dir.join("config.json");
    let config_json = match read_json_object(&config_path, "config.json") {
        Ok(json) => {
            results.push(CheckResult::pass("config_json_parse"));
            Some(json)
        }
        Err(detail) => {
            results.push(CheckResult::fail("config_json_parse", detail));
            None
        }
    };

    let config_json = match &config_json {
        Some(json) => Some(json),
        None => {
            for name in config_dependent_names(expectation) {
                results.push(CheckResult::skipped(name, "config_json_parse"));
            }
            None
        }
    };

    let mut routes: Option<&[Json]> = None;
    if let Some(config) = config_json {
        results.push(check_version(config));
        results.push(check_top_level_keys(config));

        let shape_result = check_routes_shape(config);
        let shape_ok = shape_result.passed;
        results.push(shape_result);

        if shape_ok {
            routes = config.get("routes").and_then(Json::as_array);
        }

        if let Some(routes) = routes {
            results.push(check_routes_404_fallback(routes));
        } else {
            results.push(CheckResult::skipped(
                "config_routes_404_fallback",
                "config_routes_shape",
            ));
        }

        if let Some(routes) = routes {
            results.push(check_routes_dest_targets(routes, output_dir));
        } else {
            results.push(CheckResult::skipped(
                "routes_dest_targets",
                "config_routes_shape",
            ));
        }

        if expectation == FunctionsExpectation::BasicAuthMiddleware {
            if let Some(routes) = routes {
                results.push(check_routes_middleware_first(routes));
            } else {
                results.push(CheckResult::skipped(
                    "config_routes_middleware_first",
                    "config_routes_shape",
                ));
            }
        }
    }

    // 3. static/ ディレクトリの検証。
    let static_dir = output_dir.join("static");
    let static_dir_ok = match std::fs::symlink_metadata(&static_dir) {
        Ok(meta) if meta.file_type().is_symlink() => {
            results.push(CheckResult::fail(
                "static_dir",
                "static/ must not be a symlink",
            ));
            false
        }
        Ok(meta) if meta.is_dir() => {
            results.push(CheckResult::pass("static_dir"));
            true
        }
        Ok(_) => {
            results.push(CheckResult::fail("static_dir", "not a directory"));
            false
        }
        Err(err) => {
            results.push(CheckResult::fail("static_dir", format!("{err}")));
            false
        }
    };

    if static_dir_ok {
        results.push(check_required_static_pages(&static_dir));
        results.push(check_static_no_symlinks(&static_dir));
    } else {
        results.push(CheckResult::skipped("static_required_pages", "static_dir"));
        results.push(CheckResult::skipped("static_no_symlinks", "static_dir"));
    }

    // 4. functions/ の検証（モードにより分岐、モジュール doc 参照）。
    match expectation {
        FunctionsExpectation::None => {
            results.push(check_no_functions_dir(output_dir));
        }
        FunctionsExpectation::BasicAuthMiddleware => {
            let functions_dir = output_dir.join("functions");
            let functions_dir_ok = match std::fs::symlink_metadata(&functions_dir) {
                Ok(meta) if meta.file_type().is_symlink() => {
                    results.push(CheckResult::fail(
                        "functions_dir",
                        "functions/ must not be a symlink",
                    ));
                    false
                }
                Ok(meta) if meta.is_dir() => {
                    results.push(CheckResult::pass("functions_dir"));
                    true
                }
                Ok(_) => {
                    results.push(CheckResult::fail("functions_dir", "not a directory"));
                    false
                }
                Err(err) => {
                    results.push(CheckResult::fail("functions_dir", format!("{err}")));
                    false
                }
            };

            if functions_dir_ok {
                let layout_result = check_functions_layout(&functions_dir);
                let layout_ok = layout_result.passed;
                results.push(layout_result);
                results.push(check_functions_no_symlinks(&functions_dir));

                let middleware_dir = functions_dir.join(MIDDLEWARE_FUNC_DIR_NAME);
                if layout_ok {
                    results.push(check_middleware_vc_config(&middleware_dir));
                    results.push(check_middleware_index_js(&middleware_dir));
                } else {
                    results.push(CheckResult::skipped(
                        "middleware_vc_config",
                        "functions_layout",
                    ));
                    results.push(CheckResult::skipped(
                        "middleware_index_js",
                        "functions_layout",
                    ));
                }
            } else {
                for name in [
                    "functions_layout",
                    "functions_no_symlinks",
                    "middleware_vc_config",
                    "middleware_index_js",
                ] {
                    results.push(CheckResult::skipped(name, "functions_dir"));
                }
            }
        }
    }

    results
}

/// `path` を読み込んで JSON オブジェクトとしてパースする（`read_config_json`
/// の一般化、イシュー #3344）。サイズ上限（DoS 防御）・シンボリックリンク
/// 拒否・UTF-8・JSON 構文・トップレベルがオブジェクトであることを検証する。
/// `label` は FAIL メッセージの対象表示名（例: `"config.json"`・
/// `".vc-config.json"`）。
fn read_json_object(path: &Path, label: &str) -> Result<Json, String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|err| format!("{err}"))?;
    if metadata.file_type().is_symlink() {
        return Err(format!("{label} must not be a symlink"));
    }
    if !metadata.is_file() {
        return Err(format!("{label} is not a regular file"));
    }
    if metadata.len() > MAX_CONFIG_JSON_BYTES {
        return Err(format!(
            "{label} exceeds size limit ({} > {} bytes)",
            metadata.len(),
            MAX_CONFIG_JSON_BYTES
        ));
    }

    let content = std::fs::read_to_string(path).map_err(|err| format!("{err}"))?;
    let value = json::parse(&content).map_err(|err| format!("{err}"))?;
    if !matches!(value, Json::Object(_)) {
        return Err(format!("top-level JSON value of {label} is not an object"));
    }
    Ok(value)
}

/// `version` が数値 `3` であることを検証する。
fn check_version(config: &Json) -> CheckResult {
    match config.get("version") {
        Some(Json::Number(n)) if *n == 3.0 => CheckResult::pass("config_version"),
        Some(Json::Number(n)) => {
            CheckResult::fail("config_version", format!("expected 3, got number {n}"))
        }
        Some(other) => CheckResult::fail(
            "config_version",
            format!("expected number 3, got {}", json_kind(other)),
        ),
        None => CheckResult::fail("config_version", "missing `version` key"),
    }
}

/// トップレベルキーが Build Output API v3 の許可リストに含まれることを検証する。
fn check_top_level_keys(config: &Json) -> CheckResult {
    let Json::Object(entries) = config else {
        // read_config_json がすでに object であることを保証しているため
        // 到達しないが、fail-closed のため明示的に扱う。
        return CheckResult::fail("config_top_level_keys", "top-level value is not an object");
    };

    let mut unknown: Vec<&str> = entries
        .iter()
        .map(|(k, _)| k.as_str())
        .filter(|k| !ALLOWED_TOP_LEVEL_KEYS.contains(k))
        .collect();
    unknown.sort_unstable();
    unknown.dedup();

    if unknown.is_empty() {
        CheckResult::pass("config_top_level_keys")
    } else {
        CheckResult::fail(
            "config_top_level_keys",
            format!("unknown top-level key(s): {}", unknown.join(", ")),
        )
    }
}

/// `routes` が存在する配列で、各要素が handler route か source route の
/// いずれかの形をしていることを検証する。
fn check_routes_shape(config: &Json) -> CheckResult {
    let Some(routes_value) = config.get("routes") else {
        return CheckResult::fail("config_routes_shape", "missing `routes` key");
    };
    let Some(routes) = routes_value.as_array() else {
        return CheckResult::fail("config_routes_shape", "`routes` is not an array");
    };

    for (index, route) in routes.iter().enumerate() {
        if let Err(reason) = validate_route_shape(route) {
            return CheckResult::fail("config_routes_shape", format!("routes[{index}].{reason}"));
        }
    }

    CheckResult::pass("config_routes_shape")
}

/// 1 件の route（handler/source いずれか）の形を検証する。`Err` の文字列は
/// 呼び出し元が `routes[<index>].` を前置して detail に組み立てる。
fn validate_route_shape(route: &Json) -> Result<(), String> {
    let Json::Object(entries) = route else {
        return Err(format!("<root>: not an object ({})", json_kind(route)));
    };

    let has_handle = entries.iter().any(|(k, _)| k == "handle");
    if has_handle {
        validate_handler_route(entries)
    } else {
        validate_source_route(entries)
    }
}

fn validate_handler_route(entries: &[(String, Json)]) -> Result<(), String> {
    for (key, value) in entries {
        match key.as_str() {
            "handle" => {
                let Some(s) = value.as_str() else {
                    return Err("handle: must be a string".to_string());
                };
                if !ALLOWED_HANDLE_VALUES.contains(&s) {
                    return Err(format!(
                        "handle: unknown value `{s}` (expected one of {})",
                        ALLOWED_HANDLE_VALUES.join(", ")
                    ));
                }
            }
            "src" | "dest" => {
                if value.as_str().is_none() {
                    return Err(format!("{key}: must be a string"));
                }
            }
            "status" => {
                if !is_valid_status(value) {
                    return Err("status: must be an integer in 100..=599".to_string());
                }
            }
            other if !ALLOWED_HANDLER_KEYS.contains(&other) => {
                return Err(format!("{other}: unknown key on a handle route"));
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_source_route(entries: &[(String, Json)]) -> Result<(), String> {
    let src_ok = entries
        .iter()
        .find(|(k, _)| k == "src")
        .map(|(_, v)| matches!(v.as_str(), Some(s) if !s.is_empty()))
        .unwrap_or(false);
    if !src_ok {
        return Err("src: required non-empty string is missing".to_string());
    }

    for (key, value) in entries {
        match key.as_str() {
            "src" | "dest" | "middlewarePath" => {
                if value.as_str().is_none() {
                    return Err(format!("{key}: must be a string"));
                }
            }
            "status" => {
                if !is_valid_status(value) {
                    return Err("status: must be an integer in 100..=599".to_string());
                }
            }
            "continue" | "caseSensitive" | "check" => {
                if value.as_bool().is_none() {
                    return Err(format!("{key}: must be a boolean"));
                }
            }
            "headers" => {
                let Json::Object(header_entries) = value else {
                    return Err("headers: must be an object".to_string());
                };
                if header_entries.iter().any(|(_, v)| v.as_str().is_none()) {
                    return Err("headers: all values must be strings".to_string());
                }
            }
            "methods" | "middlewareRawSrc" => {
                let Some(items) = value.as_array() else {
                    return Err(format!("{key}: must be an array"));
                };
                if items.iter().any(|item| item.as_str().is_none()) {
                    return Err(format!("{key}: all elements must be strings"));
                }
            }
            // 深いネスト構造は検証しない（モジュール doc「参照仕様」節参照）。
            // ここではキー自体が許可リストに含まれることだけを確認する。
            "has" | "missing" | "transforms" => {
                if value.as_array().is_none() {
                    return Err(format!("{key}: must be an array"));
                }
            }
            "locale" | "mitigate" => {
                if !matches!(value, Json::Object(_)) {
                    return Err(format!("{key}: must be an object"));
                }
            }
            other if !ALLOWED_SOURCE_KEYS.contains(&other) => {
                return Err(format!("{other}: unknown key on a source route"));
            }
            _ => {}
        }
    }
    Ok(())
}

/// `status` が 100..=599 の整数値であることを検証する。
fn is_valid_status(value: &Json) -> bool {
    match value.as_f64() {
        Some(n) if n.fract() == 0.0 => {
            let n = n as i64;
            (100..=599).contains(&n)
        }
        _ => false,
    }
}

/// `{"handle": "filesystem"}` が存在し、それより後ろに
/// `src == "/(.*)" && status == 404 && dest == "/404.html"` の source route
/// があることを検証する（#3290 受け入れ基準 2 の構造的な再検証）。
///
/// # PR #3322 レビュー指摘への対処（Codex P1、1 回目）
///
/// `src` の一致範囲を確認せず `status`/`dest` のみで判定すると、
/// `{"src": "/foo", "status": 404, "dest": "/404.html"}` のような部分一致
/// ルート（`/foo` にしか適用されない）でも「ファイルシステム未一致は
/// すべて 404 にフォールバックする」という受け入れ基準 2 の意図に反して
/// PASS してしまう。全パスを捕捉する正規表現 `/(.*)` であることまで
/// 検証する（`examples/vercel-ssg/src/main.rs` の `CONFIG_JSON` 実装参照）。
///
/// # PR #3322 レビュー指摘への対処（Codex P1、2 回目）
///
/// 上記の「一致範囲」検証だけでは、`{"handle": "filesystem"}` より後ろの
/// どこかに 404 フォールバック条件を満たすルートが「存在する」ことしか
/// 確認できていなかった。Vercel のルーティングは配列順に評価し最初に
/// マッチしたルートで終端するため、404 フォールバックより手前
/// （`filesystem` の直後から 404 ルートの直前まで）に、終端条件を満たす
/// （`continue: true` を持たない）ルートが挟まっていると、実際には
/// そちらが先にマッチしてしまい 404 フォールバックへ到達しない。例えば
/// `{"src": "/(.*)", "status": 200, "dest": "/index.html"}` のような SPA
/// フォールバックが 404 ルートより前にあると、旧実装は「404 ルートは
/// 存在する」の一点のみで PASS してしまっていた。本実装は 404
/// フォールバックの「最初の」出現位置を特定したうえで、`filesystem` から
/// その直前までの区間に非終端（`continue: true`）でないルートが無いこと
/// まで検証する（イシュー #3290 受け入れ基準 2）。
///
/// # PR #3322 レビュー指摘への対処（Codex P1、3 回目）
///
/// `is_fallback_route` が `src`/`status`/`dest` の 3 キーしか見ておらず、
/// 404 ルート自体に `"continue": true` が付いていても一致条件を満たして
/// しまっていた。`continue: true` はそのルートで終端させず後続ルートの
/// 評価を続ける指示であり、404 フォールバックが `continue: true` を
/// 持つと実際には応答を確定させないままルーティングが続行してしまい、
/// 「未一致の全パスへ 404 を返す」受け入れ基準 2 を満たさない。
/// `is_fallback_route` は `continue` が真でないこと（キー無し、または
/// `false`）まで検証する。
///
/// さらに、上記「shadowing」検証の走査範囲が `filesystem` と 404 ルートの
/// 間だけに限定されていたため、`filesystem` より前に終端ルート（例:
/// `{"src": "/(.*)", "status": 200, "dest": "/index.html"}` のような SPA
/// フォールバックで `continue: true` を持たないもの）が存在しても検出
/// できなかった。`filesystem` 自体に到達する前にルーティングが終端すれば
/// `filesystem` ステージも 404 フォールバックも一切実行されないため、
/// これも受け入れ基準 2 への違反である。走査範囲を配列先頭（index 0）まで
/// 拡張し、`{"handle": ...}` ハンドラルート（ビルトインのステージ切替で
/// あり `continue` を持たないのが正常形なので shadowing 判定の対象外）を
/// 除いた各ルートについて検証する。
fn check_routes_404_fallback(routes: &[Json]) -> CheckResult {
    const CATCH_ALL_SRC: &str = "/(.*)";

    let filesystem_index = routes.iter().position(|route| {
        route
            .get("handle")
            .and_then(Json::as_str)
            .is_some_and(|h| h == "filesystem")
    });

    let Some(filesystem_index) = filesystem_index else {
        return CheckResult::fail(
            "config_routes_404_fallback",
            "no `{\"handle\": \"filesystem\"}` route found",
        );
    };

    let route_continues = |route: &Json| {
        route
            .get("continue")
            .and_then(Json::as_bool)
            .unwrap_or(false)
    };

    let is_fallback_route = |route: &Json| {
        // `{"handle": ...}` はビルトインのステージ切替ハンドラルートであり
        // `src`/`status`/`dest` を併記していても source route（実リクエストに
        // マッチする通常ルート）ではない（PR #3322 レビュー指摘への対処、
        // Codex P1）。`handle` を持つルートを 404 フォールバックとして誤認
        // すると「未一致の全パスへ 404 を返す」契約を検証できていないのに
        // PASS してしまうため、`handle` キーの不在を必須条件にする。
        let is_handler = route.get("handle").is_some();
        let src_ok = route.get("src").and_then(Json::as_str) == Some(CATCH_ALL_SRC);
        let status_ok = route
            .get("status")
            .is_some_and(|s| is_valid_status(s) && s.as_f64() == Some(404.0));
        let dest_ok = route.get("dest").and_then(Json::as_str) == Some("/404.html");
        // 404 フォールバック自体が `continue: true` を持つと、そのルートで
        // 応答を確定させず後続ルートへ処理が続いてしまい終端しない。
        !is_handler && src_ok && status_ok && dest_ok && !route_continues(route)
    };

    let fallback_offset = routes[filesystem_index + 1..]
        .iter()
        .position(is_fallback_route);

    let Some(fallback_offset) = fallback_offset else {
        return CheckResult::fail(
            "config_routes_404_fallback",
            "no non-continuing route after `{\"handle\": \"filesystem\"}` with src=/(.*) status=404 dest=/404.html",
        );
    };
    let fallback_index = filesystem_index + 1 + fallback_offset;

    // 配列先頭から 404 フォールバックの直前まで（404 ルート自体は
    // exclusive）の区間に、途中でルーティングを終端させ得るルート
    // （`continue: true` を持たない）が挟まっていないか検証する。
    // `{"handle": ...}` ハンドラルート（`filesystem` を含む）はビルトイン
    // のステージ切替であり `continue` を持たないのが正常形のため、
    // shadowing 判定の対象から除く。
    let shadowing_route = routes[..fallback_index]
        .iter()
        .enumerate()
        .find(|(_, route)| route.get("handle").is_none() && !route_continues(route));

    if let Some((shadowing_index, shadowing)) = shadowing_route {
        // 判定に必要な情報（配列位置・キー名のみ）だけを detail に出す。
        // ルート定義全体（`{shadowing:?}`）を出力すると、`headers` 等の
        // 値に機微情報が含まれる場合に CI ログ・Step Summary へそのまま
        // 露出してしまう（PR #3322 レビュー指摘への対処、Codex P1:
        // A09 機微情報の露出防止）。
        let keys = match shadowing {
            Json::Object(entries) => entries
                .iter()
                .map(|(k, _)| k.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            _ => String::new(),
        };
        return CheckResult::fail(
            "config_routes_404_fallback",
            format!(
                "a terminating route (routes[{shadowing_index}], keys: [{keys}]) sits before the 404 fallback and may shadow `{{\"handle\": \"filesystem\"}}` or the fallback itself (missing `continue: true`)"
            ),
        );
    }

    CheckResult::pass("config_routes_404_fallback")
}

/// `dest` について、`static/` 配下の実ファイルを指すことを検証する（A01
/// パストラバーサル防止のため、まず相対パスの安全性を検証してから結合
/// する）。`$1` 等の正規表現後方参照を含む `dest`（例:
/// `/../../$1`）は置換後の実パスを静的解析できないため実ファイル存在
/// チェックは対象外とするが、先頭 `/` の有無・`..`/空要素/バックスラッシュ
/// を含む境界検証（[`is_safe_relative_dest`]）は置換参照の有無に関わらず
/// 適用する（PR #3322 レビュー指摘への対処、Codex P1: 置換参照の直後で
/// `continue` すると境界検証自体を素通りしてしまう）。
fn check_routes_dest_targets(routes: &[Json], output_dir: &Path) -> CheckResult {
    let static_dir = output_dir.join("static");

    for (index, route) in routes.iter().enumerate() {
        let Some(dest) = route.get("dest").and_then(Json::as_str) else {
            continue;
        };
        // `dest.contains('$')` だけで置換参照と判定すると、`$` を含むが
        // 数字が後続しない値（例: `/missing$bogus.html`）まで置換参照
        // 扱いとなり、直後の実ファイル存在チェック（`static/` 配下の
        // 通常ファイル確認）を素通りしてしまう（PR #3322 レビュー指摘
        // への対処、Codex P1）。有効な後方参照（`$1`・`$12` 等、`$` の
        // 直後に 1 桁以上の数字が続く形）だけを置換参照とみなし、それ
        // 以外の `$` を含む値は通常のリテラル `dest` として
        // `is_safe_relative_dest` + 実ファイル存在チェックの対象にする。
        let has_replacement_ref = is_valid_replacement_dest(dest);

        let Some(relative) = dest.strip_prefix('/') else {
            return CheckResult::fail(
                "routes_dest_targets",
                format!("routes[{index}].dest: must start with `/`"),
            );
        };

        // 置換参照（`$1` 等）を含む場合、展開後の実パスは静的解析できない
        // ため `is_safe_relative_dest` の `Component::Normal` 全称検証は
        // そのまま適用できない（置換元パターンにマッチした文字列が
        // セグメントへ入り得る）。それでも境界の入口（先頭 `/` の除去・
        // 空要素でないこと・バックスラッシュを含まないこと）だけは
        // 置換参照の有無に関わらず機械検証し、`../../$1` のような
        // 明白な親ディレクトリ脱出リテラルはここで弾く。
        if has_replacement_ref {
            if relative.is_empty() || relative.contains('\\') {
                return CheckResult::fail(
                    "routes_dest_targets",
                    format!("routes[{index}].dest: unsafe relative path"),
                );
            }
            if Path::new(relative)
                .components()
                .any(|c| c.as_os_str() == std::ffi::OsStr::new(".."))
            {
                return CheckResult::fail(
                    "routes_dest_targets",
                    format!(
                        "routes[{index}].dest: replacement-ref dest must not contain `..` segments"
                    ),
                );
            }
            // 置換参照を含まないリテラルなパスセグメント（例:
            // `/foo//$1` の `foo` と 2 番目の空セグメント）は静的に
            // 内容が確定するため、空要素を明示的に拒否する（PR #3322
            // レビュー指摘への対処、Codex P1: `$` を含む dest 全体を
            // 素通りさせると `/foo//$1`・`//$1` のような空パス要素が
            // 混入した dest を許可してしまい、`is_safe_relative_dest`
            // が非置換 dest に課す境界検証条件と食い違う）。置換参照
            // 自体を含むセグメント（例: `$1`）は展開結果が空文字列に
            // なり得るため、ここでの静的検証対象から除く。
            if relative
                .split('/')
                .any(|segment| !segment.contains('$') && segment.is_empty())
            {
                return CheckResult::fail(
                    "routes_dest_targets",
                    format!(
                        "routes[{index}].dest: replacement-ref dest must not contain empty path segments"
                    ),
                );
            }
            // 実ファイル存在チェックは置換後の実パスに依存するため対象外
            // （モジュール doc「参照仕様」節参照）。
            continue;
        }

        if !is_safe_relative_dest(relative) {
            return CheckResult::fail(
                "routes_dest_targets",
                format!("routes[{index}].dest: unsafe relative path"),
            );
        }

        let target = static_dir.join(relative);
        match std::fs::symlink_metadata(&target) {
            Ok(meta) if meta.is_file() && !meta.file_type().is_symlink() => {}
            Ok(_) => {
                return CheckResult::fail(
                    "routes_dest_targets",
                    format!("routes[{index}].dest: target is not a regular file"),
                );
            }
            Err(_) => {
                return CheckResult::fail(
                    "routes_dest_targets",
                    format!("routes[{index}].dest: target file does not exist"),
                );
            }
        }
    }

    CheckResult::pass("routes_dest_targets")
}

/// `dest` が Vercel の正規表現後方参照（`$1`・`$12` 等、`$` の直後に 1 桁
/// 以上の数字が続く形）のみで `$` を使っているかを判定する（PR #3322
/// レビュー指摘への対処、Codex P1）。`$` を含んでいても後方参照の形を
/// 満たさない文字（数字が続かない `$`）が 1 つでもあれば `false` を返し、
/// 呼び出し元（[`check_routes_dest_targets`]）はそれを通常のリテラル
/// `dest` として実ファイル存在チェックの対象にする。`$` を 1 つも含まない
/// 文字列も `false`（置換参照ではない）を返す。
fn is_valid_replacement_dest(dest: &str) -> bool {
    let bytes = dest.as_bytes();
    let mut has_ref = false;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j == i + 1 {
                // `$` の直後に数字が続かない（後方参照の形を満たさない）。
                return false;
            }
            has_ref = true;
            i = j;
        } else {
            i += 1;
        }
    }
    has_ref
}

/// `relative`（先頭 `/` を除いた `dest` 相対パス）が `static/` の外へ
/// 抜け出さないことを検証する（A01）。`..`・空要素・`\`・絶対パス（Windows
/// prefix を含む）を拒否する。
fn is_safe_relative_dest(relative: &str) -> bool {
    if relative.is_empty() || relative.contains('\\') {
        return false;
    }
    // `Path::components()` は連続する `/`（空パス要素）を正規化して
    // 読み飛ばすため、`Path` へ変換する前に `/` 区切りで空要素の有無を
    // 検査する必要がある（PR #3322 レビュー指摘への対処、Codex P2:
    // `foo//bar.html` のような空要素混入 dest が `Component::Normal`
    // 全称検証をすり抜けて誤って安全と判定されてしまう）。
    if relative.split('/').any(str::is_empty) {
        return false;
    }
    let path = Path::new(relative);
    path.components().all(|c| matches!(c, Component::Normal(_)))
        && path
            .components()
            .all(|c| c.as_os_str() != std::ffi::OsStr::new(".."))
}

/// [`REQUIRED_STATIC_PAGES`] がすべて `static/` 配下に通常ファイルとして
/// 存在することを検証する。不足しているものは全件 detail に列挙する
/// （1 件目で打ち切らない）。
fn check_required_static_pages(static_dir: &Path) -> CheckResult {
    let missing: Vec<&str> = REQUIRED_STATIC_PAGES
        .iter()
        .filter(|page| {
            let path = static_dir.join(page);
            match std::fs::symlink_metadata(&path) {
                Ok(meta) => !meta.is_file() || meta.file_type().is_symlink(),
                Err(_) => true,
            }
        })
        .copied()
        .collect();

    if missing.is_empty() {
        CheckResult::pass("static_required_pages")
    } else {
        CheckResult::fail(
            "static_required_pages",
            format!("missing: {}", missing.join(", ")),
        )
    }
}

/// `static/` を再帰的に走査し、シンボリックリンクが 1 つでもあれば FAIL に
/// する（A01: ツリー外のファイルを公開する経路を塞ぐ）。深さ上限
/// （[`MAX_WALK_DEPTH`]）を設ける（A05 DoS 防御）。
fn check_static_no_symlinks(static_dir: &Path) -> CheckResult {
    match find_symlink(static_dir, static_dir, 0) {
        Ok(None) => CheckResult::pass("static_no_symlinks"),
        Ok(Some(rel)) => CheckResult::fail(
            "static_no_symlinks",
            format!("symlink found at static/{}", rel.display()),
        ),
        Err(err) => CheckResult::fail("static_no_symlinks", format!("{err}")),
    }
}

/// `dir` 以下を再帰走査し、最初に見つかったシンボリックリンクの
/// `root` からの相対パスを返す（`symlink_metadata` を使いリンクを
/// たどらない）。
fn find_symlink(root: &Path, dir: &Path, depth: usize) -> std::io::Result<Option<PathBuf>> {
    if depth > MAX_WALK_DEPTH {
        return Err(std::io::Error::other(format!(
            "directory nesting exceeds depth limit ({MAX_WALK_DEPTH}) under {}",
            root.display()
        )));
    }

    let mut entries: Vec<_> = std::fs::read_dir(dir)?.collect::<Result<_, _>>()?;
    // 走査順を決定的にする（結果の再現性のため）。
    entries.sort_by_key(std::fs::DirEntry::file_name);

    for entry in entries {
        let path = entry.path();
        let meta = std::fs::symlink_metadata(&path)?;
        if meta.file_type().is_symlink() {
            let relative = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
            return Ok(Some(relative));
        }
        if meta.is_dir() {
            if let Some(found) = find_symlink(root, &path, depth + 1)? {
                return Ok(Some(found));
            }
        }
    }

    Ok(None)
}

/// `<output_dir>/functions` が存在しないことを検証する（モジュール doc
/// 「`functions/` を意図的に対象外とする理由」節参照）。
fn check_no_functions_dir(output_dir: &Path) -> CheckResult {
    let functions_dir = output_dir.join("functions");
    match std::fs::symlink_metadata(&functions_dir) {
        Ok(_) => CheckResult::fail(
            "no_functions_dir",
            "vercel-ssg uses plan c (static-only) and is not expected to generate functions/",
        ),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            CheckResult::pass("no_functions_dir")
        }
        Err(err) => CheckResult::fail("no_functions_dir", format!("{err}")),
    }
}

/// `routes[0]` がミドルウェア起動ルート（`{"src": "/(.*)",
/// "middlewarePath": "_middleware", "continue": true}` の形、`handle` キー
/// なし）であり、`middlewarePath` を持つルートが `routes[0]` 以外に
/// 存在しないことを検証する（イシュー #3344、`config_routes_shape` が
/// PASS したうえで呼ばれる前提。`examples/vercel-ssg` の
/// `CONFIG_JSON_WITH_BASIC_AUTH` 参照）。
///
/// # PR #3380 レビュー指摘への対処（Codex P0）
///
/// `validate_source_route`（`config_routes_shape` が先に適用する構造検証）
/// は `methods`/`has`/`missing` 等の追加キーを持つ source route を許容する。
/// 本関数が `src`/`middlewarePath`/`continue` の 3 キーの値のみを検証し
/// キー集合そのものを固定していないと、先頭ルートへ `methods`（例:
/// `["POST"]`）のような追加条件キーを足しても本関数は PASS してしまい、
/// その条件に一致しないリクエスト（例: `GET`）が Basic 認証ミドルウェアを
/// 経由せず後続ルートへ直接到達し得る構造上の欠陥になる（Vercel の
/// ルーティング仕様上、`methods` 等の追加条件は当該ルートの適用範囲を
/// 狭める側にしか働かないため）。生成元 `CONFIG_JSON_WITH_BASIC_AUTH` が
/// 実際に生成するキー集合は `{src, middlewarePath, continue}` の 3 つ
/// ちょうどであるため、先頭ルートのキー集合をこの 3 つに固定し、
/// それ以外のキーが 1 つでもあれば FAIL にする。
fn check_routes_middleware_first(routes: &[Json]) -> CheckResult {
    let Some(first) = routes.first() else {
        return CheckResult::fail("config_routes_middleware_first", "`routes` is empty");
    };

    let Json::Object(entries) = first else {
        return CheckResult::fail(
            "config_routes_middleware_first",
            "routes[0]: must be an object",
        );
    };

    const ALLOWED_MIDDLEWARE_ROUTE_KEYS: [&str; 3] = ["src", "middlewarePath", "continue"];
    if let Some((key, _)) = entries
        .iter()
        .find(|(k, _)| !ALLOWED_MIDDLEWARE_ROUTE_KEYS.contains(&k.as_str()))
    {
        return CheckResult::fail(
            "config_routes_middleware_first",
            format!(
                "routes[0]: unexpected key `{key}` (must be exactly {{src, middlewarePath, continue}})"
            ),
        );
    }

    if first.get("src").and_then(Json::as_str) != Some("/(.*)") {
        return CheckResult::fail(
            "config_routes_middleware_first",
            "routes[0].src: must be `/(.*)`",
        );
    }
    if first.get("middlewarePath").and_then(Json::as_str) != Some(MIDDLEWARE_PATH_VALUE) {
        return CheckResult::fail(
            "config_routes_middleware_first",
            format!("routes[0].middlewarePath: must be `{MIDDLEWARE_PATH_VALUE}`"),
        );
    }
    if first.get("continue").and_then(Json::as_bool) != Some(true) {
        return CheckResult::fail(
            "config_routes_middleware_first",
            "routes[0].continue: must be `true`",
        );
    }

    // `middlewarePath` を持つ二重・誤参照ルートの検知（A08）。
    if let Some((index, _)) = routes
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, route)| route.get("middlewarePath").is_some())
    {
        return CheckResult::fail(
            "config_routes_middleware_first",
            format!("routes[{index}]: unexpected additional `middlewarePath` route"),
        );
    }

    CheckResult::pass("config_routes_middleware_first")
}

/// `functions/` 直下のエントリが [`MIDDLEWARE_FUNC_DIR_NAME`]（ディレクトリ、
/// 非シンボリックリンク）1 件のみであることを検証する（イシュー #3344）。
/// 走査順を決定的にするため名前でソートしてから判定する。
fn check_functions_layout(functions_dir: &Path) -> CheckResult {
    let entries: Vec<_> = match std::fs::read_dir(functions_dir) {
        Ok(read_dir) => match read_dir.collect::<Result<Vec<_>, _>>() {
            Ok(entries) => entries,
            Err(err) => return CheckResult::fail("functions_layout", format!("{err}")),
        },
        Err(err) => return CheckResult::fail("functions_layout", format!("{err}")),
    };
    let mut names: Vec<std::ffi::OsString> =
        entries.iter().map(std::fs::DirEntry::file_name).collect();
    names.sort();

    let extra: Vec<String> = names
        .iter()
        .filter(|name| name.as_os_str() != std::ffi::OsStr::new(MIDDLEWARE_FUNC_DIR_NAME))
        .map(|name| name.to_string_lossy().into_owned())
        .collect();
    if !extra.is_empty() {
        let listed: Vec<&str> = extra
            .iter()
            .take(MAX_LISTED_EXTRA_ENTRIES)
            .map(String::as_str)
            .collect();
        let mut detail = format!("functions/: unexpected entr(y|ies): {}", listed.join(", "));
        if extra.len() > MAX_LISTED_EXTRA_ENTRIES {
            detail.push_str(&format!(
                " and {} more",
                extra.len() - MAX_LISTED_EXTRA_ENTRIES
            ));
        }
        return CheckResult::fail("functions_layout", detail);
    }

    let has_middleware = names
        .iter()
        .any(|name| name.as_os_str() == std::ffi::OsStr::new(MIDDLEWARE_FUNC_DIR_NAME));
    if !has_middleware {
        return CheckResult::fail(
            "functions_layout",
            format!("functions/: missing `{MIDDLEWARE_FUNC_DIR_NAME}`"),
        );
    }

    let middleware_path = functions_dir.join(MIDDLEWARE_FUNC_DIR_NAME);
    match std::fs::symlink_metadata(&middleware_path) {
        Ok(meta) if meta.file_type().is_symlink() => CheckResult::fail(
            "functions_layout",
            format!("functions/{MIDDLEWARE_FUNC_DIR_NAME}: must not be a symlink"),
        ),
        Ok(meta) if meta.is_dir() => CheckResult::pass("functions_layout"),
        Ok(_) => CheckResult::fail(
            "functions_layout",
            format!("functions/{MIDDLEWARE_FUNC_DIR_NAME}: not a directory"),
        ),
        Err(err) => CheckResult::fail("functions_layout", format!("{err}")),
    }
}

/// `functions/` 以下を再帰的に走査し、シンボリックリンクが 1 つでもあれば
/// FAIL にする（A01、[`check_static_no_symlinks`] と同型）。
fn check_functions_no_symlinks(functions_dir: &Path) -> CheckResult {
    match find_symlink(functions_dir, functions_dir, 0) {
        Ok(None) => CheckResult::pass("functions_no_symlinks"),
        Ok(Some(rel)) => CheckResult::fail(
            "functions_no_symlinks",
            format!("symlink found at functions/{}", rel.display()),
        ),
        Err(err) => CheckResult::fail("functions_no_symlinks", format!("{err}")),
    }
}

/// `<middleware_dir>/.vc-config.json` を検証する（イシュー #3344）。
/// `runtime` が文字列 `"edge"` であること、`entrypoint` があれば文字列
/// [`MIDDLEWARE_INDEX_JS_NAME`] と一致することを確認する（A08）。全キーの
/// 許可リスト検証は行わない（モジュール doc「Basic 認証ミドルウェア出力の
/// 検証内容」節参照）。
fn check_middleware_vc_config(middleware_dir: &Path) -> CheckResult {
    let path = middleware_dir.join(".vc-config.json");
    let config = match read_json_object(&path, ".vc-config.json") {
        Ok(json) => json,
        Err(detail) => return CheckResult::fail("middleware_vc_config", detail),
    };

    match config.get("runtime") {
        Some(Json::String(s)) if s == "edge" => {}
        Some(other) => {
            return CheckResult::fail(
                "middleware_vc_config",
                format!("runtime: expected \"edge\", got {}", json_kind(other)),
            );
        }
        None => {
            return CheckResult::fail("middleware_vc_config", "missing `runtime` key");
        }
    }

    if let Some(entrypoint) = config.get("entrypoint") {
        match entrypoint.as_str() {
            Some(s) if s == MIDDLEWARE_INDEX_JS_NAME => {}
            Some(_) => {
                return CheckResult::fail(
                    "middleware_vc_config",
                    format!("entrypoint: must be `{MIDDLEWARE_INDEX_JS_NAME}`"),
                );
            }
            None => {
                return CheckResult::fail("middleware_vc_config", "entrypoint: must be a string");
            }
        }
    }

    CheckResult::pass("middleware_vc_config")
}

/// `<middleware_dir>/index.js` が通常ファイルとして存在し、シンボリック
/// リンクでなく、空でないことを検証する（内容は読まない。イシュー #3344）。
fn check_middleware_index_js(middleware_dir: &Path) -> CheckResult {
    let path = middleware_dir.join(MIDDLEWARE_INDEX_JS_NAME);
    match std::fs::symlink_metadata(&path) {
        Ok(meta) if meta.file_type().is_symlink() => CheckResult::fail(
            "middleware_index_js",
            format!("{MIDDLEWARE_INDEX_JS_NAME} must not be a symlink"),
        ),
        Ok(meta) if meta.is_file() => {
            if meta.len() == 0 {
                CheckResult::fail(
                    "middleware_index_js",
                    format!("{MIDDLEWARE_INDEX_JS_NAME} is empty"),
                )
            } else {
                CheckResult::pass("middleware_index_js")
            }
        }
        Ok(_) => CheckResult::fail(
            "middleware_index_js",
            format!("{MIDDLEWARE_INDEX_JS_NAME} is not a regular file"),
        ),
        Err(err) => CheckResult::fail("middleware_index_js", format!("{err}")),
    }
}

fn json_kind(value: &Json) -> &'static str {
    match value {
        Json::Null => "null",
        Json::Bool(_) => "bool",
        Json::Number(_) => "number",
        Json::String(_) => "string",
        Json::Array(_) => "array",
        Json::Object(_) => "object",
    }
}

/// CI ログから機械抽出可能な 1 行を整形する
/// （`check-vercel-output: check=<name> result=<PASS|FAIL>[ detail=<理由>]`）。
/// `.github/workflows/ci.yml` の Step Summary 転記ステップが
/// `grep '^check-vercel-output:'` で抽出する契約であり、安易に変更しない。
pub fn format_line(result: &CheckResult) -> String {
    let verdict = if result.passed { "PASS" } else { "FAIL" };
    match &result.detail {
        Some(detail) => format!(
            "check-vercel-output: check={} result={} detail={}",
            result.name, verdict, detail
        ),
        None => format!(
            "check-vercel-output: check={} result={}",
            result.name, verdict
        ),
    }
}

/// 総括行（`check-vercel-output: output_dir=<DIR> result=<PASS|FAIL> failed=<N>`）
/// を整形する。CI の判定・Step Summary 抽出はこの 1 行の形式に依存する。
pub fn format_summary(output_dir: &Path, results: &[CheckResult]) -> String {
    let failed = results.iter().filter(|r| !r.passed).count();
    let verdict = if failed == 0 { "PASS" } else { "FAIL" };
    format!(
        "check-vercel-output: output_dir={} result={} failed={}",
        output_dir.display(),
        verdict,
        failed
    )
}

impl fmt::Display for CheckResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_line(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(entries: Vec<(&str, Json)>) -> Json {
        Json::Object(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    fn s(v: &str) -> Json {
        Json::String(v.to_string())
    }

    #[test]
    fn validate_route_shape_accepts_handler_route() {
        let route = obj(vec![("handle", s("filesystem"))]);
        assert!(validate_route_shape(&route).is_ok());
    }

    #[test]
    fn validate_route_shape_rejects_unknown_handle_value() {
        let route = obj(vec![("handle", s("bogus"))]);
        assert!(validate_route_shape(&route).is_err());
    }

    #[test]
    fn validate_route_shape_rejects_handler_route_with_unknown_key() {
        let route = obj(vec![("handle", s("filesystem")), ("bogus", s("x"))]);
        assert!(validate_route_shape(&route).is_err());
    }

    #[test]
    fn validate_route_shape_accepts_minimal_source_route() {
        let route = obj(vec![("src", s("/(.*)"))]);
        assert!(validate_route_shape(&route).is_ok());
    }

    #[test]
    fn validate_route_shape_rejects_source_route_without_src() {
        let route = obj(vec![("dest", s("/x"))]);
        assert!(validate_route_shape(&route).is_err());
    }

    #[test]
    fn validate_route_shape_rejects_typo_key() {
        // 実際に本イシューの動機となった typo（`status` → `stauts`）。
        let route = obj(vec![("src", s("/(.*)")), ("stauts", Json::Number(404.0))]);
        assert!(validate_route_shape(&route).is_err());
    }

    #[test]
    fn validate_route_shape_rejects_status_as_string() {
        let route = obj(vec![("src", s("/(.*)")), ("status", s("404"))]);
        assert!(validate_route_shape(&route).is_err());
    }

    #[test]
    fn validate_route_shape_accepts_full_source_route() {
        let route = obj(vec![
            ("src", s("/(.*)")),
            ("dest", s("/404.html")),
            ("status", Json::Number(404.0)),
            (
                "headers",
                obj(vec![("X-Content-Type-Options", s("nosniff"))]),
            ),
            ("continue", Json::Bool(true)),
            ("caseSensitive", Json::Bool(false)),
            ("check", Json::Bool(true)),
            ("methods", Json::Array(vec![s("GET")])),
        ]);
        assert!(validate_route_shape(&route).is_ok());
    }

    #[test]
    fn is_valid_status_rejects_out_of_range() {
        assert!(!is_valid_status(&Json::Number(50.0)));
        assert!(!is_valid_status(&Json::Number(600.0)));
        assert!(!is_valid_status(&Json::Number(404.5)));
        assert!(is_valid_status(&Json::Number(404.0)));
    }

    #[test]
    fn check_routes_404_fallback_requires_filesystem_then_404() {
        let routes = vec![
            obj(vec![("handle", s("filesystem"))]),
            obj(vec![
                ("src", s("/(.*)")),
                ("status", Json::Number(404.0)),
                ("dest", s("/404.html")),
            ]),
        ];
        assert!(check_routes_404_fallback(&routes).passed);
    }

    #[test]
    fn check_routes_404_fallback_rejects_404_before_filesystem() {
        let routes = vec![
            obj(vec![
                ("src", s("/(.*)")),
                ("status", Json::Number(404.0)),
                ("dest", s("/404.html")),
            ]),
            obj(vec![("handle", s("filesystem"))]),
        ];
        assert!(!check_routes_404_fallback(&routes).passed);
    }

    #[test]
    fn check_routes_404_fallback_rejects_missing_filesystem() {
        let routes = vec![obj(vec![
            ("src", s("/(.*)")),
            ("status", Json::Number(404.0)),
            ("dest", s("/404.html")),
        ])];
        assert!(!check_routes_404_fallback(&routes).passed);
    }

    /// PR #3322 レビュー指摘（Codex P1）の回帰テスト: `src` の一致範囲を
    /// 検証しないと `/foo` のような部分一致ルートでも通ってしまう。
    #[test]
    fn check_routes_404_fallback_rejects_partial_match_src() {
        let routes = vec![
            obj(vec![("handle", s("filesystem"))]),
            obj(vec![
                ("src", s("/foo")),
                ("status", Json::Number(404.0)),
                ("dest", s("/404.html")),
            ]),
        ];
        assert!(!check_routes_404_fallback(&routes).passed);
    }

    /// PR #3322 レビュー指摘（Codex P1、2 回目）の回帰テスト:
    /// `filesystem` と 404 フォールバックの間に終端ルート（`continue: true`
    /// を持たない SPA フォールバック等）が挟まっていると、実際のルーティ
    /// ングではそちらが先にマッチして 404 へ到達しないにもかかわらず、
    /// 「404 フォールバック条件を満たすルートがどこかに存在する」ことしか
    /// 見ない実装では PASS してしまっていた。
    #[test]
    fn check_routes_404_fallback_rejects_shadowing_catch_all_before_fallback() {
        let routes = vec![
            obj(vec![("handle", s("filesystem"))]),
            obj(vec![
                ("src", s("/(.*)")),
                ("status", Json::Number(200.0)),
                ("dest", s("/index.html")),
            ]),
            obj(vec![
                ("src", s("/(.*)")),
                ("status", Json::Number(404.0)),
                ("dest", s("/404.html")),
            ]),
        ];
        assert!(!check_routes_404_fallback(&routes).passed);
    }

    /// PR #3322 レビュー指摘（Codex P1、3 回目）の回帰テスト: shadowing
    /// 検知の FAIL detail はルート定義全体（`headers` の値等）を出力せず、
    /// 配列位置とキー名のみを含む（A09 機微情報の露出防止）。
    #[test]
    fn check_routes_404_fallback_shadowing_detail_excludes_route_values() {
        let secret_token = "super-secret-token-value";
        let routes = vec![
            obj(vec![("handle", s("filesystem"))]),
            obj(vec![
                ("src", s("/(.*)")),
                ("headers", obj(vec![("X-Secret", s(secret_token))])),
            ]),
            obj(vec![
                ("src", s("/(.*)")),
                ("status", Json::Number(404.0)),
                ("dest", s("/404.html")),
            ]),
        ];
        let result = check_routes_404_fallback(&routes);
        assert!(!result.passed);
        let detail = result.detail.expect("FAIL には detail が必須");
        assert!(
            !detail.contains(secret_token),
            "detail にルート定義の値（機微情報）が含まれてはならない: {detail}"
        );
        assert!(
            detail.contains("routes[1]"),
            "detail に配列位置が含まれるべき: {detail}"
        );
        assert!(
            detail.contains("src") && detail.contains("headers"),
            "detail にキー名が含まれるべき: {detail}"
        );
    }

    /// 上記のシャドーイング検知は `continue: true` を持つ非終端ルート
    /// （ヘッダー付与等）までは誤って弾かない。
    #[test]
    fn check_routes_404_fallback_accepts_continue_route_before_fallback() {
        let routes = vec![
            obj(vec![("handle", s("filesystem"))]),
            obj(vec![
                ("src", s("/(.*)")),
                (
                    "headers",
                    obj(vec![("X-Content-Type-Options", s("nosniff"))]),
                ),
                ("continue", Json::Bool(true)),
            ]),
            obj(vec![
                ("src", s("/(.*)")),
                ("status", Json::Number(404.0)),
                ("dest", s("/404.html")),
            ]),
        ];
        assert!(check_routes_404_fallback(&routes).passed);
    }

    #[test]
    fn is_safe_relative_dest_rejects_parent_traversal() {
        assert!(!is_safe_relative_dest("../etc/passwd"));
        assert!(!is_safe_relative_dest("a/../../b"));
    }

    #[test]
    fn is_safe_relative_dest_rejects_backslash_and_empty() {
        assert!(!is_safe_relative_dest("a\\b"));
        assert!(!is_safe_relative_dest(""));
    }

    /// PR #3322 レビュー指摘（Codex P2）の回帰テスト: `Path::components()`
    /// は連続する `/` を正規化して読み飛ばすため、`Path` 変換に頼るだけ
    /// では空パス要素混入の `dest` を誤って安全と判定してしまう。
    #[test]
    fn is_safe_relative_dest_rejects_empty_path_segment() {
        assert!(!is_safe_relative_dest("foo//bar.html"));
        assert!(!is_safe_relative_dest("/foo.html"));
        assert!(!is_safe_relative_dest("foo/bar.html/"));
    }

    /// PR #3322 レビュー指摘（Codex P1）の回帰テスト: `dest` が `/` で
    /// 始まらない不正値でも、FAIL detail には配列位置とキー名のみを含み
    /// `dest` の実値そのものは出力しない（A09 機微情報の露出防止）。
    #[test]
    fn check_routes_dest_targets_detail_excludes_dest_value() {
        let secret_dest = "super-secret-internal-path.html";
        let routes = vec![obj(vec![("dest", s(secret_dest))])];
        let output_dir = std::env::temp_dir();
        let result = check_routes_dest_targets(&routes, &output_dir);
        assert!(!result.passed);
        let detail = result.detail.expect("FAIL には detail が必須");
        assert!(
            !detail.contains(secret_dest),
            "detail に dest の実値が含まれてはならない: {detail}"
        );
        assert!(
            detail.contains("routes[0]") && detail.contains("dest"),
            "detail に配列位置とキー名が含まれるべき: {detail}"
        );
    }

    #[test]
    fn is_safe_relative_dest_accepts_normal_relative_path() {
        assert!(is_safe_relative_dest("404.html"));
        assert!(is_safe_relative_dest("pages/about/index.html"));
    }

    #[test]
    fn check_top_level_keys_rejects_unknown_key() {
        let config = obj(vec![("version", Json::Number(3.0)), ("bogus", s("x"))]);
        assert!(!check_top_level_keys(&config).passed);
    }

    #[test]
    fn check_top_level_keys_accepts_documented_keys() {
        let config = obj(vec![
            ("version", Json::Number(3.0)),
            ("routes", Json::Array(vec![])),
            ("images", obj(vec![])),
            ("wildcard", Json::Array(vec![])),
            ("overrides", obj(vec![])),
            ("cache", Json::Array(vec![])),
            ("framework", obj(vec![])),
            ("crons", Json::Array(vec![])),
            ("services", Json::Array(vec![])),
        ]);
        assert!(check_top_level_keys(&config).passed);
    }

    #[test]
    fn check_version_rejects_string_version() {
        let config = obj(vec![("version", s("3"))]);
        assert!(!check_version(&config).passed);
    }

    #[test]
    fn check_version_rejects_wrong_number() {
        let config = obj(vec![("version", Json::Number(2.0))]);
        assert!(!check_version(&config).passed);
    }

    #[test]
    fn check_version_accepts_three() {
        let config = obj(vec![("version", Json::Number(3.0))]);
        assert!(check_version(&config).passed);
    }

    #[test]
    fn format_line_includes_detail_only_when_failed() {
        let pass = CheckResult::pass("x");
        assert_eq!(
            format_line(&pass),
            "check-vercel-output: check=x result=PASS"
        );

        let fail = CheckResult::fail("y", "boom");
        assert_eq!(
            format_line(&fail),
            "check-vercel-output: check=y result=FAIL detail=boom"
        );
    }

    #[test]
    fn format_summary_counts_failures() {
        let results = vec![CheckResult::pass("a"), CheckResult::fail("b", "boom")];
        let summary = format_summary(Path::new("/tmp/out"), &results);
        assert_eq!(
            summary,
            "check-vercel-output: output_dir=/tmp/out result=FAIL failed=1"
        );
    }

    #[test]
    fn format_summary_all_pass() {
        let results = vec![CheckResult::pass("a"), CheckResult::pass("b")];
        let summary = format_summary(Path::new("/tmp/out"), &results);
        assert_eq!(
            summary,
            "check-vercel-output: output_dir=/tmp/out result=PASS failed=0"
        );
    }

    // --- イシュー #3344: FunctionsExpectation / Basic 認証ミドルウェア検証 ---

    /// R1 の回帰固定: 既定モード（`FunctionsExpectation::None`）のチェック名
    /// 一覧・順序は旧実装の 11 件と完全一致する。
    #[test]
    fn check_names_none_matches_legacy_order() {
        let legacy = [
            "output_dir",
            "config_json_parse",
            "config_version",
            "config_top_level_keys",
            "config_routes_shape",
            "config_routes_404_fallback",
            "routes_dest_targets",
            "static_dir",
            "static_required_pages",
            "static_no_symlinks",
            "no_functions_dir",
        ];
        assert_eq!(check_names(FunctionsExpectation::None), &legacy);
    }

    /// `examples/vercel-ssg` の `CONFIG_JSON_WITH_BASIC_AUTH` 実装と同一の
    /// 先頭ミドルウェアルート。
    fn middleware_route() -> Json {
        obj(vec![
            ("src", s("/(.*)")),
            ("middlewarePath", s("_middleware")),
            ("continue", Json::Bool(true)),
        ])
    }

    #[test]
    fn check_routes_middleware_first_accepts_valid_leading_route() {
        let routes = vec![
            middleware_route(),
            obj(vec![("handle", s("filesystem"))]),
            obj(vec![
                ("src", s("/(.*)")),
                ("status", Json::Number(404.0)),
                ("dest", s("/404.html")),
            ]),
        ];
        assert!(check_routes_middleware_first(&routes).passed);
    }

    #[test]
    fn check_routes_middleware_first_rejects_empty_routes() {
        assert!(!check_routes_middleware_first(&[]).passed);
    }

    #[test]
    fn check_routes_middleware_first_rejects_other_route_first() {
        let routes = vec![obj(vec![("handle", s("filesystem"))]), middleware_route()];
        assert!(!check_routes_middleware_first(&routes).passed);
    }

    #[test]
    fn check_routes_middleware_first_rejects_missing_continue() {
        let routes = vec![obj(vec![
            ("src", s("/(.*)")),
            ("middlewarePath", s("_middleware")),
        ])];
        assert!(!check_routes_middleware_first(&routes).passed);
    }

    #[test]
    fn check_routes_middleware_first_rejects_continue_false() {
        let routes = vec![obj(vec![
            ("src", s("/(.*)")),
            ("middlewarePath", s("_middleware")),
            ("continue", Json::Bool(false)),
        ])];
        assert!(!check_routes_middleware_first(&routes).passed);
    }

    #[test]
    fn check_routes_middleware_first_rejects_wrong_middleware_path() {
        let routes = vec![obj(vec![
            ("src", s("/(.*)")),
            ("middlewarePath", s("_other")),
            ("continue", Json::Bool(true)),
        ])];
        assert!(!check_routes_middleware_first(&routes).passed);
    }

    #[test]
    fn check_routes_middleware_first_rejects_partial_src() {
        let routes = vec![obj(vec![
            ("src", s("/foo")),
            ("middlewarePath", s("_middleware")),
            ("continue", Json::Bool(true)),
        ])];
        assert!(!check_routes_middleware_first(&routes).passed);
    }

    #[test]
    fn check_routes_middleware_first_rejects_handle_route() {
        let routes = vec![obj(vec![
            ("handle", s("filesystem")),
            ("src", s("/(.*)")),
            ("middlewarePath", s("_middleware")),
            ("continue", Json::Bool(true)),
        ])];
        assert!(!check_routes_middleware_first(&routes).passed);
    }

    /// PR #3380 レビュー指摘（Codex P0）の回帰テスト: 先頭ルートへ
    /// `validate_source_route` が許容する追加キー（`methods`）を足しても
    /// `src`/`middlewarePath`/`continue` の値さえ合っていれば PASS して
    /// しまっていた構造上の欠陥を固定する。`methods: ["POST"]` を足すと
    /// GET 等の他メソッドがミドルウェアを経由せず後続ルートへ到達し得る。
    #[test]
    fn check_routes_middleware_first_rejects_additional_methods_key() {
        let routes = vec![obj(vec![
            ("src", s("/(.*)")),
            ("middlewarePath", s("_middleware")),
            ("continue", Json::Bool(true)),
            ("methods", Json::Array(vec![s("POST")])),
        ])];
        assert!(!check_routes_middleware_first(&routes).passed);
    }

    #[test]
    fn check_routes_middleware_first_rejects_duplicate_middleware_path() {
        let routes = vec![
            middleware_route(),
            obj(vec![
                ("src", s("/api/(.*)")),
                ("middlewarePath", s("_middleware")),
            ]),
        ];
        assert!(!check_routes_middleware_first(&routes).passed);
    }

    /// 有効時の `CONFIG_JSON_WITH_BASIC_AUTH`（`examples/vercel-ssg`）と同内容
    /// の config が既存の `check_routes_404_fallback`（shadowing 検査）を
    /// 無改変で PASS することを固定する（計画§3.4 補足）。
    #[test]
    fn config_json_with_basic_auth_passes_existing_404_fallback_check() {
        let routes = vec![
            middleware_route(),
            obj(vec![
                ("src", s("/(.*)")),
                (
                    "headers",
                    obj(vec![
                        ("X-Content-Type-Options", s("nosniff")),
                        ("Referrer-Policy", s("strict-origin-when-cross-origin")),
                    ]),
                ),
                ("continue", Json::Bool(true)),
            ]),
            obj(vec![("handle", s("filesystem"))]),
            obj(vec![
                ("src", s("/(.*)")),
                ("status", Json::Number(404.0)),
                ("dest", s("/404.html")),
            ]),
        ];
        assert!(check_routes_404_fallback(&routes).passed);
    }

    fn valid_vc_config() -> Json {
        obj(vec![
            ("runtime", s("edge")),
            ("entrypoint", s("index.js")),
            (
                "envVarsInUse",
                Json::Array(vec![s("BASIC_AUTH_USER"), s("BASIC_AUTH_PASSWORD")]),
            ),
        ])
    }

    /// `check_middleware_vc_config` はファイル読み取りを経由するため、
    /// 一時ディレクトリへ `.vc-config.json` を書き出して検証する。
    fn write_vc_config(dir: &Path, content: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join(".vc-config.json"), content).unwrap();
    }

    fn unique_test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "xtask-check-vercel-output-unit-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn check_middleware_vc_config_accepts_valid_config() {
        let dir = unique_test_dir("vc-config-valid");
        let json_text = json_to_string(&valid_vc_config());
        write_vc_config(&dir, &json_text);
        assert!(check_middleware_vc_config(&dir).passed);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_middleware_vc_config_rejects_nodejs_runtime() {
        let dir = unique_test_dir("vc-config-nodejs");
        write_vc_config(
            &dir,
            r#"{"runtime": "nodejs20.x", "entrypoint": "index.js"}"#,
        );
        let result = check_middleware_vc_config(&dir);
        assert!(!result.passed);
        assert!(result.detail.unwrap().contains("runtime"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_middleware_vc_config_rejects_numeric_runtime() {
        let dir = unique_test_dir("vc-config-numeric-runtime");
        write_vc_config(&dir, r#"{"runtime": 1, "entrypoint": "index.js"}"#);
        assert!(!check_middleware_vc_config(&dir).passed);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_middleware_vc_config_rejects_missing_runtime() {
        let dir = unique_test_dir("vc-config-missing-runtime");
        write_vc_config(&dir, r#"{"entrypoint": "index.js"}"#);
        assert!(!check_middleware_vc_config(&dir).passed);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_middleware_vc_config_rejects_entrypoint_mismatch() {
        let dir = unique_test_dir("vc-config-entrypoint-mismatch");
        write_vc_config(&dir, r#"{"runtime": "edge", "entrypoint": "main.js"}"#);
        assert!(!check_middleware_vc_config(&dir).passed);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_middleware_vc_config_rejects_top_level_array() {
        let dir = unique_test_dir("vc-config-top-level-array");
        write_vc_config(&dir, r#"["edge"]"#);
        assert!(!check_middleware_vc_config(&dir).passed);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_middleware_vc_config_rejects_invalid_json() {
        let dir = unique_test_dir("vc-config-invalid-json");
        write_vc_config(&dir, "{ not json");
        assert!(!check_middleware_vc_config(&dir).passed);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_middleware_vc_config_rejects_missing_file() {
        let dir = unique_test_dir("vc-config-missing-file");
        assert!(!check_middleware_vc_config(&dir).passed);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A09 回帰テスト: `.vc-config.json` の `envVarsInUse` の中身（機微な
    /// 環境変数名の並び）は FAIL detail に出さない。
    #[test]
    fn check_middleware_vc_config_detail_excludes_env_vars_in_use_values() {
        let dir = unique_test_dir("vc-config-a09");
        write_vc_config(
            &dir,
            r#"{"runtime": "nodejs", "envVarsInUse": ["SUPER_SECRET_VAR_NAME"]}"#,
        );
        let result = check_middleware_vc_config(&dir);
        assert!(!result.passed);
        let detail = result.detail.expect("FAIL には detail が必須");
        assert!(!detail.contains("SUPER_SECRET_VAR_NAME"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_middleware_index_js_accepts_nonempty_file() {
        let dir = unique_test_dir("index-js-valid");
        std::fs::write(dir.join("index.js"), b"export default () => {};").unwrap();
        assert!(check_middleware_index_js(&dir).passed);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_middleware_index_js_rejects_missing_file() {
        let dir = unique_test_dir("index-js-missing");
        assert!(!check_middleware_index_js(&dir).passed);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_middleware_index_js_rejects_empty_file() {
        let dir = unique_test_dir("index-js-empty");
        std::fs::write(dir.join("index.js"), b"").unwrap();
        assert!(!check_middleware_index_js(&dir).passed);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// JSON 値を最小限のシリアライズで文字列化するテスト専用ヘルパー
    /// （`json` モジュールはパースのみを提供し逆方向を持たないため）。
    fn json_to_string(value: &Json) -> String {
        match value {
            Json::Null => "null".to_string(),
            Json::Bool(b) => b.to_string(),
            Json::Number(n) => n.to_string(),
            Json::String(s) => format!("{s:?}"),
            Json::Array(items) => {
                let parts: Vec<String> = items.iter().map(json_to_string).collect();
                format!("[{}]", parts.join(","))
            }
            Json::Object(entries) => {
                let parts: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| format!("{k:?}:{}", json_to_string(v)))
                    .collect();
                format!("{{{}}}", parts.join(","))
            }
        }
    }

    /// `check()` のモード別スナップショット。合格ツリー・`output_dir` 不在・
    /// `config.json` パース失敗の 3 状況について、返る `CheckResult` の
    /// チェック名集合が `check_names(mode)` と重複・欠落なく完全一致する
    /// ことを固定する（R1・R3〜R9 の全体整合性を担保）。
    fn assert_check_names_match(results: &[CheckResult], expectation: FunctionsExpectation) {
        let expected = check_names(expectation);
        let actual: Vec<&str> = results.iter().map(|r| r.name).collect();
        let mut expected_sorted = expected.to_vec();
        expected_sorted.sort_unstable();
        let mut actual_sorted = actual.clone();
        actual_sorted.sort_unstable();
        assert_eq!(
            expected_sorted, actual_sorted,
            "expected check name set {expected:?}, got {actual:?}"
        );
    }

    fn write_valid_basic_auth_tree(root: &Path) {
        let output = root.join(".vercel/output");
        let static_dir = output.join("static");
        std::fs::create_dir_all(static_dir.join("pages/about")).unwrap();
        std::fs::create_dir_all(static_dir.join("pages/default-escaping")).unwrap();
        std::fs::write(static_dir.join("index.html"), "<html>index</html>").unwrap();
        std::fs::write(static_dir.join("404.html"), "<html>404</html>").unwrap();
        std::fs::write(
            static_dir.join("pages/about/index.html"),
            "<html>about</html>",
        )
        .unwrap();
        std::fs::write(
            static_dir.join("pages/default-escaping/index.html"),
            "<html>escaping</html>",
        )
        .unwrap();

        std::fs::write(
            output.join("config.json"),
            r#"{
  "version": 3,
  "routes": [
    {"src": "/(.*)", "middlewarePath": "_middleware", "continue": true},
    {"src": "/(.*)", "headers": {"X-Content-Type-Options": "nosniff"}, "continue": true},
    { "handle": "filesystem" },
    { "src": "/(.*)", "status": 404, "dest": "/404.html" }
  ]
}
"#,
        )
        .unwrap();

        let middleware_dir = output.join("functions/_middleware.func");
        std::fs::create_dir_all(&middleware_dir).unwrap();
        std::fs::write(
            middleware_dir.join(".vc-config.json"),
            r#"{"runtime": "edge", "entrypoint": "index.js", "envVarsInUse": ["BASIC_AUTH_USER", "BASIC_AUTH_PASSWORD"]}"#,
        )
        .unwrap();
        std::fs::write(middleware_dir.join("index.js"), b"export default () => {};").unwrap();
    }

    #[test]
    fn check_none_mode_on_valid_tree_matches_check_names() {
        let dir = unique_test_dir("check-none-valid-tree");
        let output = dir.join(".vercel/output");
        let static_dir = output.join("static");
        std::fs::create_dir_all(static_dir.join("pages/about")).unwrap();
        std::fs::create_dir_all(static_dir.join("pages/default-escaping")).unwrap();
        std::fs::write(static_dir.join("index.html"), "x").unwrap();
        std::fs::write(static_dir.join("404.html"), "x").unwrap();
        std::fs::write(static_dir.join("pages/about/index.html"), "x").unwrap();
        std::fs::write(static_dir.join("pages/default-escaping/index.html"), "x").unwrap();
        std::fs::write(
            output.join("config.json"),
            r#"{"version": 3, "routes": [{"handle": "filesystem"}, {"src": "/(.*)", "status": 404, "dest": "/404.html"}]}"#,
        )
        .unwrap();

        let results = check(&output, FunctionsExpectation::None);
        assert!(results.iter().all(|r| r.passed));
        assert_check_names_match(&results, FunctionsExpectation::None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_basic_auth_mode_on_valid_tree_passes_and_matches_check_names() {
        let dir = unique_test_dir("check-basic-auth-valid-tree");
        write_valid_basic_auth_tree(&dir);
        let output = dir.join(".vercel/output");

        let results = check(&output, FunctionsExpectation::BasicAuthMiddleware);
        assert!(
            results.iter().all(|r| r.passed),
            "expected all PASS, got {results:?}"
        );
        assert_check_names_match(&results, FunctionsExpectation::BasicAuthMiddleware);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// R1/R2 の境界: 既定モードを有効時のツリーへ適用すると `functions/` が
    /// 存在するため `no_functions_dir` が FAIL する。
    #[test]
    fn check_none_mode_on_basic_auth_tree_fails_no_functions_dir() {
        let dir = unique_test_dir("check-none-on-basic-auth-tree");
        write_valid_basic_auth_tree(&dir);
        let output = dir.join(".vercel/output");

        let results = check(&output, FunctionsExpectation::None);
        assert_check_names_match(&results, FunctionsExpectation::None);
        assert!(results
            .iter()
            .any(|r| r.name == "no_functions_dir" && !r.passed));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 新モードを無効時（`functions/` なし）のツリーへ適用すると
    /// `functions_dir` が FAIL する。
    #[test]
    fn check_basic_auth_mode_on_none_tree_fails_functions_dir() {
        let dir = unique_test_dir("check-basic-auth-on-none-tree");
        let output = dir.join(".vercel/output");
        let static_dir = output.join("static");
        std::fs::create_dir_all(static_dir.join("pages/about")).unwrap();
        std::fs::create_dir_all(static_dir.join("pages/default-escaping")).unwrap();
        std::fs::write(static_dir.join("index.html"), "x").unwrap();
        std::fs::write(static_dir.join("404.html"), "x").unwrap();
        std::fs::write(static_dir.join("pages/about/index.html"), "x").unwrap();
        std::fs::write(static_dir.join("pages/default-escaping/index.html"), "x").unwrap();
        std::fs::write(
            output.join("config.json"),
            r#"{"version": 3, "routes": [{"handle": "filesystem"}, {"src": "/(.*)", "status": 404, "dest": "/404.html"}]}"#,
        )
        .unwrap();

        let results = check(&output, FunctionsExpectation::BasicAuthMiddleware);
        assert_check_names_match(&results, FunctionsExpectation::BasicAuthMiddleware);
        assert!(results
            .iter()
            .any(|r| r.name == "functions_dir" && !r.passed));
        assert!(results.iter().any(|r| r.name == "middleware_vc_config"
            && !r.passed
            && r.detail.as_deref() == Some("skipped: depends on functions_dir")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_basic_auth_mode_output_dir_missing_matches_check_names() {
        let dir = unique_test_dir("check-basic-auth-missing-output-dir");
        let output = dir.join("does-not-exist");
        let results = check(&output, FunctionsExpectation::BasicAuthMiddleware);
        assert_check_names_match(&results, FunctionsExpectation::BasicAuthMiddleware);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_basic_auth_mode_config_parse_failure_matches_check_names() {
        let dir = unique_test_dir("check-basic-auth-config-parse-failure");
        write_valid_basic_auth_tree(&dir);
        let output = dir.join(".vercel/output");
        std::fs::write(output.join("config.json"), "{ not json").unwrap();

        let results = check(&output, FunctionsExpectation::BasicAuthMiddleware);
        assert_check_names_match(&results, FunctionsExpectation::BasicAuthMiddleware);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
