//! `fw impact` の ambiguous（多重定義）単独ケースの独立 e2e（イシュー #293）。
//!
//! #137（TASK-13.2e: 影響範囲解析のテスト整備）クローズ時の留保事項のうち、
//! `docs/impact-analysis-design.md` §3.4 テスト観点 4「多重定義
//! （`ambiguous`）時の承認強制」を、`cli/src/impact.rs` の単体テスト
//! （`analyze_reports_ambiguous_and_requires_approval_when_multiply_defined`
//! ほか）に加えて実バイナリ（`fw`）経由の e2e として固定する。
//!
//! `cli/src/impact.rs::find_definitions` は同名シンボルの定義元が 2 クレート
//! 以上にまたがる場合に `ambiguous: true` を立て、`requires_human_approval`
//! はこれを `breaking_risk`/`affected_routes` とは独立の条件として `OR` で
//! 合成する（`ambiguous` 単独で `true` になり得る）。本ファイルはこの
//! 「low リスク・ルート影響なしでも ambiguous 単独で承認が強制される」
//! 挙動を、フィクスチャプロジェクトに対する実行結果として検証する。

mod support;

use support::{
    json_bool_field, json_string_field, run_fw, write_impact_workspace, ImpactMemberSpec,
};

/// `core-a` のみが `pub fn render_widget` を定義する 1 クレート構成での対照系。
/// `ambiguous: false` を確認し、後続テストの `ambiguous: true` との差分を
/// 明確にする（相互依存なし・使用箇所なしのため `breaking_risk: low`・
/// `requires_human_approval: false` になる）。
#[test]
fn unambiguous_baseline_is_auto_applicable() {
    let project = write_impact_workspace(
        "ambiguous-baseline",
        &[ImpactMemberSpec {
            dir: "core-a",
            package_name: "impact-fixture-core-a",
            path_deps: &[],
            source: "pub fn render_widget() -> String {\n    String::from(\"widget\")\n}\n",
        }],
    );

    let (code, stdout, stderr) = run_fw("impact", &["render_widget"], &project);
    assert_eq!(
        code, 0,
        "fw impact は正常系で終了コード 0 を返す契約（stderr: {stderr}）"
    );
    assert_eq!(
        json_bool_field(&stdout, "ambiguous"),
        Some(false),
        "定義元は core-a の 1 件のみのため ambiguous は false（stdout: {stdout}）"
    );
    assert_eq!(
        json_string_field(&stdout, "breaking_risk"),
        Some("low".to_string()),
        "使用箇所がなく affected_crates が空のため breaking_risk は low（stdout: {stdout}）"
    );
    assert_eq!(
        json_bool_field(&stdout, "requires_human_approval"),
        Some(false),
        "low リスク・ルート影響なし・ambiguous でもないため承認不要（stdout: {stdout}）"
    );
    assert_eq!(
        json_string_field(&stdout, "verdict"),
        Some("auto-applicable (impact is limited; automatic application allowed subject to gate pass)".to_string()),
        "requires_human_approval: false と対応する verdict 文言（stdout: {stdout}）"
    );
}

/// 本題: `core-a` / `core-b` の双方が同名 `pub fn render_widget` を定義する
/// 2 クレート構成（相互依存なし・使用箇所なし）。`breaking_risk: low` の
/// まま `ambiguous: true` により `requires_human_approval: true` が強制
/// されることを検証する（`docs/impact-analysis-design.md` §3.4 観点 4）。
#[test]
fn multiply_defined_symbol_forces_human_approval() {
    let project = write_impact_workspace(
        "ambiguous-multiply-defined",
        &[
            ImpactMemberSpec {
                dir: "core-a",
                package_name: "impact-fixture-core-a",
                path_deps: &[],
                source: "pub fn render_widget() -> String {\n    String::from(\"widget-a\")\n}\n",
            },
            ImpactMemberSpec {
                dir: "core-b",
                package_name: "impact-fixture-core-b",
                path_deps: &[],
                source: "pub fn render_widget() -> String {\n    String::from(\"widget-b\")\n}\n",
            },
        ],
    );

    let (code, stdout, stderr) = run_fw("impact", &["render_widget"], &project);
    assert_eq!(
        code, 0,
        "定義元が複数でも SymbolNotFound ではないため終了コードは 0（stderr: {stderr}）"
    );
    assert_eq!(
        json_bool_field(&stdout, "ambiguous"),
        Some(true),
        "core-a / core-b の双方が render_widget を定義するため ambiguous は true（stdout: {stdout}）"
    );
    assert_eq!(
        json_string_field(&stdout, "breaking_risk"),
        Some("low".to_string()),
        "使用箇所がなく affected_crates が空のため breaking_risk は low のまま（stdout: {stdout}）"
    );
    assert_eq!(
        json_bool_field(&stdout, "requires_human_approval"),
        Some(true),
        "low リスク・ルート影響なしでも ambiguous 単独で承認が強制される（stdout: {stdout}）"
    );
    assert_eq!(
        json_string_field(&stdout, "verdict"),
        Some("requires human approval (impact spans multiple crates or public routes)".to_string()),
        "requires_human_approval: true と対応する verdict 文言（stdout: {stdout}）"
    );
}
