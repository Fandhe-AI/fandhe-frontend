//! シナリオ 2（UI 改善: 一覧画面への件数サマリー追加、TASK-13.4c・#146）の
//! 回帰テスト。親タスク TASK-13.4（#143）・設計文書
//! `docs/design/scenario-regression-design.md` §4.1 行 2・§4.2〜§4.4 の実装契約に
//! 従う。
//!
//! # 期待値の根拠（PoC-7 JSON をそのまま突き合わせない理由）
//!
//! PoC-7 実測値（`docs/spec/03-poc/ai-self-maintenance/scenarios/ui-item-count/
//! {impact.json,gate.json}`）は Python プロトタイプの出力であり、本テストの
//! 期待値はこれを直接引き写さず、製品判定ルール（`cli/src/impact.rs`）から
//! 導出し直す（設計文書 §4.1 行 2 の指示）。導出過程は下記の各テストの
//! ドキュメンテーションコメントに記す。
//!
//! # フィクスチャ構成
//!
//! `rws-app`（`list_page`）を `rws-server`（一覧ルート `/`・詳細ルート
//! `/items/:id`）と `rws-wasm-client`（CSR ハイドレーションスタブ）の双方から
//! 呼び出す 3 クレートワークスペース。`rws-wasm-client` は
//! `impact::CLIENT_BOUNDARY_CRATES` に含まれるクレート名と厳密一致させる
//! ことが判定要件であり、wasm-bindgen 等の実 wasm 依存は持ち込まない
//! （ホストコンパイル可能なスタブ、設計文書 §3）。

use crate::common::{
    self, cargo_deny_available, check_passed, json_bool_field, json_string_field, replace_unique,
    run_fw, write_workspace_project, MemberFixture,
};

/// `structure.toml`（`fw gate` が唯一の情報源とする宣言ファイル、
/// `gate.rs` 冒頭コメント参照）。`app` は `role = "component"`、`server` は
/// `role = "server-entrypoint"`、`wasm-client` は `role = "client-entrypoint"`
/// （`docs/design/structure-manifest.md` §2.2.1 の閉じた語彙に従う）。`[routing]
/// definition_dir = "server"` はルート定義を `server/` 配下に限定する宣言。
const STRUCTURE_TOML: &str = r#"
[manifest]
version = 1

[directories.app]
role = "component"
crate = "rws-app"
description = "TASK-13.4c scenario 2 fixture: list_page component"
allowed_dependents = ["server", "wasm-client"]

[directories.server]
role = "server-entrypoint"
crate = "rws-server"
description = "TASK-13.4c scenario 2 fixture: server entrypoint routing to list_page"
depends_on = ["app"]

[directories.wasm-client]
role = "client-entrypoint"
crate = "rws-wasm-client"
description = "TASK-13.4c scenario 2 fixture: CSR client entrypoint stub"
depends_on = ["app"]

[routing]
definition_dir = "server"
extractor = "rws-router-v1"
"#;

const APP_CARGO_TOML: &str = "[package]\nname = \"rws-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\nlicense = \"MIT\"\npublish = false\n";

const SERVER_CARGO_TOML: &str = "[package]\nname = \"rws-server\"\nversion = \"0.1.0\"\nedition = \"2021\"\nlicense = \"MIT\"\npublish = false\n\n[dependencies]\nrws-app = { path = \"../app\" }\n";

const WASM_CLIENT_CARGO_TOML: &str = "[package]\nname = \"rws-wasm-client\"\nversion = \"0.1.0\"\nedition = \"2021\"\nlicense = \"MIT\"\npublish = false\n\n[dependencies]\nrws-app = { path = \"../app\" }\n";

/// ベースライン（無改変）の `app/src/lib.rs`。`list_page` はプレーン文字列
/// 描画のみを行い（`raw_html` 等のエスケープ迂回・HTML 文字列直接組み立ては
/// 使わない、`coding-rust.md`）、`rws-server`・`rws-wasm-client` の双方から
/// 呼び出されるモード非依存コンポーネント（本リポジトリの `app/`（rws-app）の
/// 役割に対応）。
const BASELINE_APP_LIB_RS: &str = r#"//! シナリオ 2（UI 改善）フィクスチャ: 一覧画面コンポーネント。
//! `rws-server`（一覧ルート `/`）・`rws-wasm-client`（CSR ハイドレーション）
//! の双方から呼ばれるモード非依存コンポーネントを模する。

/// 一覧画面の本文を組み立てる。既定エスケープ方針（REQ-1）に抵触しない
/// プレーン文字列描画のみを行う。
pub fn list_page(items: &[&str]) -> String {
    items.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_page_joins_items() {
        let out = list_page(&["widget", "gadget"]);
        assert!(out.contains("widget, gadget"));
    }
}
"#;

/// 改修（UI 改善）: `list_page` の出力先頭に件数サマリー行を追加する
/// 一意な置換対象（`common::replace_unique` で厳密に 1 箇所へ適用する）。
const LIST_PAGE_BODY: &str = "    items.join(\", \")\n";
const LIST_PAGE_BODY_WITH_COUNT_SUMMARY: &str =
    "    format!(\"count: {}\\n{}\", items.len(), items.join(\", \"))\n";

/// 新規アサーション（件数サマリーの出現を検証するテスト）を挿入する一意な
/// 置換対象。既存の `list_page_joins_items` テストの直後・`mod tests` の
/// 閉じ括弧の直前に挿入する。
const TESTS_MOD_TAIL: &str = "        assert!(out.contains(\"widget, gadget\"));\n    }\n}\n";
const TESTS_MOD_TAIL_WITH_COUNT_ASSERTION: &str = "        assert!(out.contains(\"widget, gadget\"));\n    }\n\n    #[test]\n    fn list_page_includes_item_count_summary() {\n        let out = list_page(&[\"widget\", \"gadget\"]);\n        assert!(\n            out.contains(\"count: 2\"),\n            \"list_page should include an item count summary line: {out}\"\n        );\n    }\n}\n";

/// `server/src/main.rs`: 一覧ルート `/` と詳細ルート `/items/:id` を宣言し、
/// 一覧ルートのハンドラから `rws_app::list_page` を呼び出す。
/// `cli/src/routes.rs`（`rws-router-v1` 抽出器）が対象とする
/// `.route("<path>", handler)` 構文をそのまま実コードとして含む
/// （Router スタブ、実物の `rws-server::router` のパスマッチング実装は
/// 持ち込まない）。
const SERVER_MAIN_RS: &str = r#"//! シナリオ 2（UI 改善）フィクスチャ: サーバーエントリ。
//! `rws-app::list_page` を一覧ルート `/` のハンドラから呼び出し、詳細ルート
//! `/items/:id` も併せて宣言する。

use rws_app::list_page;

/// ルート定義を蓄積する最小スタブ。`rws-server`（実物）の `router.rs` の
/// パスマッチング実装は持ち込まず、`.route(path, handler)` 構文の形だけを
/// 再現する。
struct Router;

impl Router {
    fn route(self, _path: &str, _handler: fn() -> String) -> Self {
        self
    }
}

/// 一覧ルート `/` のハンドラ。`rws-app::list_page` を呼び出す
/// （`fw impact list_page` が `affected_files` として検出する対象）。
fn list_handler() -> String {
    list_page(&["widget", "gadget"])
}

/// 詳細ルート `/items/:id` のハンドラ（本フィクスチャでは `list_page` を
/// 参照しない対照用の別ルート）。
fn detail_handler() -> String {
    String::new()
}

fn main() {
    let router = Router;
    let _ = router
        .route("/", list_handler)
        .route("/items/:id", detail_handler);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_handler_returns_non_empty() {
        assert!(!list_handler().is_empty());
    }
}
"#;

/// `wasm-client/src/lib.rs`: CSR ハイドレーションスタブ。`rws_app::list_page`
/// を呼び出すのみで、wasm-bindgen 等の実 wasm 依存は持ち込まない（ホスト
/// コンパイル可能。クレート名 `rws-wasm-client` が
/// `impact::CLIENT_BOUNDARY_CRATES` と厳密一致することのみが判定要件）。
const WASM_CLIENT_LIB_RS: &str = r#"//! シナリオ 2（UI 改善)フィクスチャ: CSR ハイドレーションスタブ。
//! `rws-app::list_page` を呼び出し、クライアント側での再描画を模する。

/// `list_page` の結果をクライアント側 DOM へ反映する体（実際の DOM 操作は
/// 行わない、ホストコンパイル可能な最小スタブ）。
pub fn hydrate_list() -> String {
    rws_app::list_page(&["widget", "gadget"])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hydrate_list_delegates_to_list_page() {
        assert!(!hydrate_list().is_empty());
    }
}
"#;

/// ベースライン（無改変）のフィクスチャを書き出す。
fn write_baseline_fixture(scenario_name: &str) -> common::ScenarioProject {
    write_workspace_project(
        scenario_name,
        STRUCTURE_TOML,
        &[
            MemberFixture {
                dir_name: "app",
                cargo_toml: APP_CARGO_TOML,
                src_files: &[("lib.rs", BASELINE_APP_LIB_RS)],
            },
            MemberFixture {
                dir_name: "server",
                cargo_toml: SERVER_CARGO_TOML,
                src_files: &[("main.rs", SERVER_MAIN_RS)],
            },
            MemberFixture {
                dir_name: "wasm-client",
                cargo_toml: WASM_CLIENT_CARGO_TOML,
                src_files: &[("lib.rs", WASM_CLIENT_LIB_RS)],
            },
        ],
    )
}

/// `fw impact list_page`（ベースラインフィクスチャ）が製品判定ルールから
/// 導出した期待値どおりに判定することを検証する（設計文書 §4.1 行 2）。
///
/// 導出根拠（`cli/src/impact.rs` の実装に基づく、PoC-7 JSON はそのまま
/// 使わない）:
/// - `defined_in_crate`: `list_page` はトップレベル `pub fn` として `rws-app`
///   の 1 ファイルにのみ定義される（`find_definitions`）→ `"rws-app"`・
///   `ambiguous:false`。
/// - `affected_crates`: `list_page` の使用箇所（定義ファイル自身は除外）は
///   `server/src/main.rs`・`wasm-client/src/lib.rs` の 2 ファイル
///   （`scan_usages`）。いずれのクレートにも逆依存する他クレートは存在しない
///   ため `reverse_dependency_closure` は seeds のまま
///   `["rws-server","rws-wasm-client"]`（`BTreeSet` 昇順）。
/// - `breaking_risk`: `affected_crates` が `CLIENT_BOUNDARY_CRATES`
///   （`rws-wasm-client`）を含むため、件数 2 でも `judge_breaking_risk` は
///   `high`。
/// - `requires_human_approval`: `high` かつ影響ルート非空のため `true`。
/// - `affected_routes`: `server/src/main.rs` は `list_page` の使用箇所
///   （`affected_files`）でもあるため、その内容から
///   `routes::extract_routes_from_source` を通した結果
///   `["/","/items/:id"]`（`affected_route_paths`、`BTreeSet` 昇順）。
#[test]
fn scenario2_impact_reports_high_risk_for_list_page() {
    let project = write_baseline_fixture("scenario2-impact");
    let (code, stdout, stderr) = run_fw("impact", &["list_page"], &project);

    assert_eq!(
        code, 0,
        "fw impact list_page はベースラインフィクスチャで成功するはず: stdout={stdout} stderr={stderr}"
    );
    assert_eq!(
        json_string_field(&stdout, "defined_in_crate").as_deref(),
        Some("rws-app"),
        "stdout={stdout}"
    );
    assert_eq!(
        json_bool_field(&stdout, "ambiguous"),
        Some(false),
        "stdout={stdout}"
    );
    assert_eq!(
        json_string_field(&stdout, "breaking_risk").as_deref(),
        Some("high"),
        "stdout={stdout}"
    );
    assert_eq!(
        json_bool_field(&stdout, "requires_human_approval"),
        Some(true),
        "stdout={stdout}"
    );
    // `affected_crates` / `affected_routes` は「ちょうど 2 件」を固定するため
    // 部分一致ではなく、`render_report` が 1 行コンパクト JSON を出力する
    // 契約（`BTreeSet` 由来で要素順序も安定）を利用した完全一致で検証する。
    assert!(
        stdout.contains("\"affected_crates\":[\"rws-server\",\"rws-wasm-client\"]"),
        "stdout={stdout}"
    );
    assert!(
        stdout.contains("\"affected_routes\":[\"/\",\"/items/:id\"]"),
        "stdout={stdout}"
    );
}

/// 改修（件数サマリー追加）+ 新規アサーションの両方を適用したフィクスチャで
/// `fw gate` のコア 5 チェックがすべて通過することを検証する。`policy` は
/// `cargo_deny_available()` で環境分岐する（`main.rs` の
/// `baseline_fixture_passes_gate_core_checks` と同一方針。環境ごとに
/// 弱体化なしで取れる最強のアサーションを常時実行する、`coding-rust.md`）。
#[test]
fn scenario2_gate_passes_after_ui_improvement() {
    let improved_lib_rs = replace_unique(
        BASELINE_APP_LIB_RS,
        LIST_PAGE_BODY,
        LIST_PAGE_BODY_WITH_COUNT_SUMMARY,
    );
    let improved_lib_rs = replace_unique(
        &improved_lib_rs,
        TESTS_MOD_TAIL,
        TESTS_MOD_TAIL_WITH_COUNT_ASSERTION,
    );

    let project = write_workspace_project(
        "scenario2-gate-after",
        STRUCTURE_TOML,
        &[
            MemberFixture {
                dir_name: "app",
                cargo_toml: APP_CARGO_TOML,
                src_files: &[("lib.rs", improved_lib_rs.as_str())],
            },
            MemberFixture {
                dir_name: "server",
                cargo_toml: SERVER_CARGO_TOML,
                src_files: &[("main.rs", SERVER_MAIN_RS)],
            },
            MemberFixture {
                dir_name: "wasm-client",
                cargo_toml: WASM_CLIENT_CARGO_TOML,
                src_files: &[("lib.rs", WASM_CLIENT_LIB_RS)],
            },
        ],
    );

    let (code, stdout, stderr) = run_fw("gate", &[], &project);

    for check_name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "test",
    ] {
        assert_eq!(
            check_passed(&stdout, check_name),
            Some(true),
            "改修適用後は `{check_name}` チェックが通過するはず: stdout={stdout} stderr={stderr}"
        );
    }

    if cargo_deny_available() {
        assert_eq!(
            code, 0,
            "cargo-deny 導入環境では改修適用後は PASS するはず: stdout={stdout}"
        );
        assert!(
            stdout.contains("\"gate_result\":\"PASS\""),
            "stdout={stdout}"
        );
        assert_eq!(
            check_passed(&stdout, "policy"),
            Some(true),
            "stdout={stdout}"
        );
    } else {
        assert_eq!(
            code, 1,
            "cargo-deny 未導入環境では policy の fail-closed により BLOCKED (終了コード 1) のはず: stdout={stdout}"
        );
        assert!(
            stdout.contains("\"gate_result\":\"BLOCKED\""),
            "stdout={stdout}"
        );
        assert_eq!(
            check_passed(&stdout, "policy"),
            Some(false),
            "stdout={stdout}"
        );
    }
}

/// 安全側の対照テスト（設計文書 §4.3「ブロック理由の特定性」方針）:
/// 新規アサーション（件数サマリー検証テスト）のみを追加し、実装変更
/// （`list_page` への件数サマリー追加）を適用しない中間状態では、追加した
/// アサーションが実際に失敗し `test` チェックが `passed:false` になる
/// ことを確認する。これにより、新規アサーションが空振り（vacuous、常に
/// 成功する無意味なテスト）でないことを保証する。
///
/// `type_check` / `default_escape_check` / `lint` は実装変更が入っていない
/// ため通過するはずであり、`test` チェックのみが失敗することで
/// ブロック理由を特定できることも併せて検証する。
#[test]
fn scenario2_new_assertion_is_load_bearing() {
    let assertion_only_lib_rs = replace_unique(
        BASELINE_APP_LIB_RS,
        TESTS_MOD_TAIL,
        TESTS_MOD_TAIL_WITH_COUNT_ASSERTION,
    );

    let project = write_workspace_project(
        "scenario2-assertion-only",
        STRUCTURE_TOML,
        &[
            MemberFixture {
                dir_name: "app",
                cargo_toml: APP_CARGO_TOML,
                src_files: &[("lib.rs", assertion_only_lib_rs.as_str())],
            },
            MemberFixture {
                dir_name: "server",
                cargo_toml: SERVER_CARGO_TOML,
                src_files: &[("main.rs", SERVER_MAIN_RS)],
            },
            MemberFixture {
                dir_name: "wasm-client",
                cargo_toml: WASM_CLIENT_CARGO_TOML,
                src_files: &[("lib.rs", WASM_CLIENT_LIB_RS)],
            },
        ],
    );

    let (code, stdout, stderr) = run_fw("gate", &[], &project);

    assert_eq!(
        check_passed(&stdout, "type_check"),
        Some(true),
        "型チェック自体は実装変更なしで通過するはず: stdout={stdout} stderr={stderr}"
    );
    assert_eq!(
        check_passed(&stdout, "default_escape_check"),
        Some(true),
        "stdout={stdout}"
    );
    assert_eq!(check_passed(&stdout, "lint"), Some(true), "stdout={stdout}");
    assert_eq!(
        check_passed(&stdout, "test"),
        Some(false),
        "新規アサーションが空振りでなければ、実装変更なしでは test が失敗するはず: stdout={stdout}"
    );
    assert!(
        stdout.contains("\"gate_result\":\"BLOCKED\""),
        "stdout={stdout}"
    );
    assert_eq!(
        code, 1,
        "test 失敗により BLOCKED (終了コード 1) のはず: stdout={stdout}"
    );
}
