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
//! # `functions/` を意図的に対象外とする理由
//!
//! `examples/vercel-ssg` は案 c（`docs/design/vercel-deployment-strategy.md`
//! が確定した「SSG → Build Output API → `--prebuilt`」方式）の実演であり、
//! Vercel 側で Rust ランタイムを実行しない。このため `functions/` を生成
//! しない。本モジュールは「`functions/` が存在しないこと」を検証すること
//! で、この設計判断が知らないうちに崩れた場合に検知する（[`no_functions_dir`]）。
//! これは Issue #3292 の「Build Output API 出力構造」という文言を意図的に
//! 狭める解釈である。将来 functions を持つ example が追加された場合は、
//! `.func/.vc-config.json` の検証へ本モジュールを拡張するものとする。
//!
//! # セキュリティ不変条件（OWASP Top 10、`security.md` 参照）
//!
//! - **A01 パストラバーサル**: `routes[].dest` を `static/` へ結合する前に、
//!   構成要素に `..`・ルート・プレフィックス・`\`・空要素が含まれないことを
//!   検証する（[`is_safe_relative_dest`]）。`static/` 走査はシンボリック
//!   リンクをたどらず（`symlink_metadata`）、リンクを検出したら FAIL に
//!   する（[`static_no_symlinks`]）。出力ディレクトリ自体がシンボリック
//!   リンクの場合も FAIL にする（[`output_dir`]）。本モジュールはファイルを
//!   削除・書き込みしない（読み取り専用）。
//! - **A05 / DoS 耐性**: `config.json` の読み込みに [`MAX_CONFIG_JSON_BYTES`]
//!   の上限を設ける。`json` モジュールの `MAX_DEPTH` によるネスト上限を
//!   引き継ぐ。ディレクトリ走査にも深さ上限（[`MAX_WALK_DEPTH`]）を設ける。
//! - **A09 機微情報の露出防止**: 失敗メッセージにはチェック名・相対パス・
//!   キー名・理由だけを出し、ファイル全文・環境変数・トークンは出さない
//!   （`json.rs::JsonError` と同じ方針）。
//! - **fail-closed**: 前段のチェックが失敗して後段が判定できない場合
//!   （`config.json` がパースできない等）、後段は黙って PASS にせず
//!   `result=FAIL detail=skipped: depends on <name>` として明示する。

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
/// 各チェックは独立した `CheckResult` を返す（例外的に、前段の失敗で後段が
/// 判定不能な場合は [`CheckResult::skipped`] を返し、黙って PASS にしない）。
pub fn check(output_dir: &Path) -> Vec<CheckResult> {
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
        for name in [
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
        ] {
            results.push(CheckResult::skipped(name, "output_dir"));
        }
        return results;
    }

    // 2. config.json の読み込み・パース。
    let config_path = output_dir.join("config.json");
    let config_json = match read_config_json(&config_path) {
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
            for name in [
                "config_version",
                "config_top_level_keys",
                "config_routes_shape",
                "config_routes_404_fallback",
                "routes_dest_targets",
            ] {
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

    // 4. functions/ の非存在（案 c: 静的配置のみ、モジュール doc 参照）。
    results.push(check_no_functions_dir(output_dir));

    results
}

/// `config.json` を読み込んでパースする。サイズ上限（DoS 防御）・UTF-8・
/// JSON 構文・トップレベルがオブジェクトであることを検証する。
fn read_config_json(path: &Path) -> Result<Json, String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|err| format!("{err}"))?;
    if metadata.file_type().is_symlink() {
        return Err("config.json must not be a symlink".to_string());
    }
    if !metadata.is_file() {
        return Err("config.json is not a regular file".to_string());
    }
    if metadata.len() > MAX_CONFIG_JSON_BYTES {
        return Err(format!(
            "config.json exceeds size limit ({} > {} bytes)",
            metadata.len(),
            MAX_CONFIG_JSON_BYTES
        ));
    }

    let content = std::fs::read_to_string(path).map_err(|err| format!("{err}"))?;
    let value = json::parse(&content).map_err(|err| format!("{err}"))?;
    if !matches!(value, Json::Object(_)) {
        return Err("top-level JSON value is not an object".to_string());
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

    let is_fallback_route = |route: &Json| {
        let src_ok = route.get("src").and_then(Json::as_str) == Some(CATCH_ALL_SRC);
        let status_ok = route
            .get("status")
            .is_some_and(|s| is_valid_status(s) && s.as_f64() == Some(404.0));
        let dest_ok = route.get("dest").and_then(Json::as_str) == Some("/404.html");
        src_ok && status_ok && dest_ok
    };

    let fallback_offset = routes[filesystem_index + 1..]
        .iter()
        .position(is_fallback_route);

    let Some(fallback_offset) = fallback_offset else {
        return CheckResult::fail(
            "config_routes_404_fallback",
            "no route after `{\"handle\": \"filesystem\"}` with src=/(.*) status=404 dest=/404.html",
        );
    };
    let fallback_index = filesystem_index + 1 + fallback_offset;

    // `filesystem` の直後から 404 フォールバックの直前までの区間
    // （両端とも exclusive）に、途中でルーティングを終端させ得るルート
    // （`continue: true` を持たない）が挟まっていないか検証する。
    let shadowing_route = routes[filesystem_index + 1..fallback_index]
        .iter()
        .find(|route| {
            !route
                .get("continue")
                .and_then(Json::as_bool)
                .unwrap_or(false)
        });

    if let Some(shadowing) = shadowing_route {
        return CheckResult::fail(
            "config_routes_404_fallback",
            format!(
                "a terminating route ({shadowing:?}) sits between `{{\"handle\": \"filesystem\"}}` and the 404 fallback and may shadow it (missing `continue: true`)"
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
        let has_replacement_ref = dest.contains('$');

        let Some(relative) = dest.strip_prefix('/') else {
            return CheckResult::fail(
                "routes_dest_targets",
                format!("routes[{index}].dest: must start with `/` (got `{dest}`)"),
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
                    format!("routes[{index}].dest: unsafe relative path `{dest}`"),
                );
            }
            if Path::new(relative)
                .components()
                .any(|c| c.as_os_str() == std::ffi::OsStr::new(".."))
            {
                return CheckResult::fail(
                    "routes_dest_targets",
                    format!(
                        "routes[{index}].dest: replacement-ref dest must not contain `..` segments (`{dest}`)"
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
                format!("routes[{index}].dest: unsafe relative path `{dest}`"),
            );
        }

        let target = static_dir.join(relative);
        match std::fs::symlink_metadata(&target) {
            Ok(meta) if meta.is_file() && !meta.file_type().is_symlink() => {}
            Ok(_) => {
                return CheckResult::fail(
                    "routes_dest_targets",
                    format!("routes[{index}].dest: target is not a regular file (`{dest}`)"),
                );
            }
            Err(_) => {
                return CheckResult::fail(
                    "routes_dest_targets",
                    format!("routes[{index}].dest: target file does not exist (`{dest}`)"),
                );
            }
        }
    }

    CheckResult::pass("routes_dest_targets")
}

/// `relative`（先頭 `/` を除いた `dest` 相対パス）が `static/` の外へ
/// 抜け出さないことを検証する（A01）。`..`・空要素・`\`・絶対パス（Windows
/// prefix を含む）を拒否する。
fn is_safe_relative_dest(relative: &str) -> bool {
    if relative.is_empty() || relative.contains('\\') {
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
                Ok(meta) => !(meta.is_file() && !meta.file_type().is_symlink()),
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
            "vercel-ssg は案 c（静的配置のみ）であり functions/ は生成されない想定",
        ),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            CheckResult::pass("no_functions_dir")
        }
        Err(err) => CheckResult::fail("no_functions_dir", format!("{err}")),
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
}
