//! `fw impact` の `fandhe-frontend-wasm-thin` 単独クレート高リスク判定の独立 e2e
//! （イシュー #293）。
//!
//! #137（TASK-13.2e: 影響範囲解析のテスト整備）クローズ時の留保事項のうち、
//! `docs/design/impact-analysis-design.md` §3.4 テスト観点 2「`fandhe-frontend-wasm-client` /
//! `fandhe-frontend-wasm-full` / `fandhe-frontend-wasm-thin` 各々を単独で含む場合の `high` 判定」の
//! うち、既存シナリオ e2e（`cli/tests/scenarios/bugfix_escape.rs`）が
//! `fandhe-frontend-wasm-client` のみをカバーしていた欠落分（`fandhe-frontend-wasm-thin`）を、
//! 実バイナリ（`fw`）経由の e2e として補う。
//!
//! `cli/src/impact.rs::CLIENT_BOUNDARY_CRATES` は
//! `["fandhe-frontend-wasm-client", "fandhe-frontend-wasm-full", "fandhe-frontend-wasm-thin"]` を持ち、
//! `judge_breaking_risk` はこのいずれかを 1 クレートでも含めば
//! `affected_crates` の総数によらず `high` と判定する（`cli/src/impact.rs`
//! 単体テスト `judge_breaking_risk_single_wasm_thin_crate_is_high` と同じ
//! 判定境界）。本ファイルは「判別子がクレート名そのもの」であることを、
//! 消費側クレート名だけを差し替えた対照系との比較で固定する。

mod support;

use support::{
    json_array_contains_str, json_bool_field, json_string_field, run_fw, write_impact_workspace,
    ImpactMemberSpec,
};

/// `core`（`pub fn render` を定義）を呼び出す純ネイティブ lib クレートの
/// ソース。`wasm-bindgen` は使わず、`fw impact` のネイティブ走査対象に含める
/// （`cli/tests/scenarios/common.rs::scenario1_wasm_client_lib_rs` と同じ
/// 方針）。呼び出し側が消費クレート名（境界クレート名か非境界クレート名か）
/// を差し替えても、この呼び出し元ソース自体は不変。
fn consumer_lib_rs() -> &'static str {
    "use impact_fixture_core::render;\n\npub fn hydrate() -> String {\n    render()\n}\n"
}

/// 本題: `core` + `wasm-thin`（pkg 名 `fandhe-frontend-wasm-thin`）の 2 クレート構成
/// （`wasm-thin` が `core` へ path 依存し `render` を呼ぶ）。単独 1 クレート
/// でもクライアント境界クレートであるため `breaking_risk: high` になることを
/// 検証する。
#[test]
fn single_wasm_thin_crate_impact_is_high_risk() {
    let project = write_impact_workspace(
        "wasm-thin-high-risk",
        &[
            ImpactMemberSpec {
                dir: "core",
                package_name: "impact-fixture-core",
                path_deps: &[],
                source: "pub fn render() -> String {\n    String::from(\"rendered\")\n}\n",
            },
            ImpactMemberSpec {
                dir: "wasm-thin",
                package_name: "fandhe-frontend-wasm-thin",
                path_deps: &["core"],
                source: consumer_lib_rs(),
            },
        ],
    );

    let (code, stdout, stderr) = run_fw("impact", &["render"], &project);
    assert_eq!(
        code, 0,
        "fw impact は正常系で終了コード 0 を返す契約（stderr: {stderr}）"
    );
    assert!(
        json_array_contains_str(&stdout, "affected_crates", "fandhe-frontend-wasm-thin"),
        "wasm-thin が render を呼び出すため affected_crates に fandhe-frontend-wasm-thin を含む（stdout: {stdout}）"
    );
    assert_eq!(
        json_string_field(&stdout, "breaking_risk"),
        Some("high".to_string()),
        "fandhe-frontend-wasm-thin は単独でもクライアント境界クレートのため breaking_risk は high（stdout: {stdout}）"
    );
    assert_eq!(
        json_bool_field(&stdout, "requires_human_approval"),
        Some(true),
        "breaking_risk: high のため承認が必須（stdout: {stdout}）"
    );
    assert_eq!(
        json_bool_field(&stdout, "ambiguous"),
        Some(false),
        "render の定義元は core の 1 件のみのため ambiguous は false（stdout: {stdout}）"
    );
}

/// 対照系: 消費側クレート名だけを非境界名（`impact-fixture-client`）へ
/// 変えた同一構成。`breaking_risk: medium` になることを確認し、上記テストの
/// `high` 判定の判別子が「クライアント境界クレート名（`CLIENT_BOUNDARY_CRATES`）
/// であること」自体であって、クレート数や依存構造の違いではないことを固定する。
#[test]
fn single_non_boundary_crate_impact_is_medium_risk() {
    let project = write_impact_workspace(
        "wasm-thin-medium-risk-contrast",
        &[
            ImpactMemberSpec {
                dir: "core",
                package_name: "impact-fixture-core",
                path_deps: &[],
                source: "pub fn render() -> String {\n    String::from(\"rendered\")\n}\n",
            },
            ImpactMemberSpec {
                dir: "client",
                package_name: "impact-fixture-client",
                path_deps: &["core"],
                source: consumer_lib_rs(),
            },
        ],
    );

    let (code, stdout, stderr) = run_fw("impact", &["render"], &project);
    assert_eq!(
        code, 0,
        "fw impact は正常系で終了コード 0 を返す契約（stderr: {stderr}）"
    );
    assert!(
        json_array_contains_str(&stdout, "affected_crates", "impact-fixture-client"),
        "client が render を呼び出すため affected_crates に impact-fixture-client を含む（stdout: {stdout}）"
    );
    assert_eq!(
        json_string_field(&stdout, "breaking_risk"),
        Some("medium".to_string()),
        "境界クレート名を含まない 1 クレート影響のため breaking_risk は medium（stdout: {stdout}）"
    );
    assert_eq!(
        json_bool_field(&stdout, "requires_human_approval"),
        Some(true),
        "breaking_risk: medium のため承認が必須（stdout: {stdout}）"
    );
}
