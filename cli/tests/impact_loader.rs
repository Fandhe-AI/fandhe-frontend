//! `fw impact` の `affected_loaders` フィールド（イシュー #353）の実バイナリ e2e。
//!
//! `impl Loader for <Type>`（`docs/design/loader-trait-design.md`）を含む
//! クレートへ波及する変更が `affected_loaders` に型名を反映し、
//! `requires_human_approval` を要求すること（正例）、および波及先に
//! `impl Loader` が無ければ `affected_loaders` が空のまま誤検知しないこと
//! （負例）の双方を実バイナリ経由で固定する。

mod support;

use support::{
    json_array_contains_str, json_bool_field, run_fw, write_impact_workspace, ImpactMemberSpec,
};

/// `core`（`pub fn fetch_items` を定義）に依存し、`impl Loader for
/// DemoItemsLoader` の中で `fetch_items` を呼ぶ `app` 相当クレート。
fn app_lib_rs_with_loader() -> &'static str {
    "use impact_fixture_core::fetch_items;\n\npub struct DemoItemsLoader;\n\ntrait Loader {\n    fn load(&self) -> Vec<String>;\n}\n\nimpl Loader for DemoItemsLoader {\n    fn load(&self) -> Vec<String> {\n        fetch_items()\n    }\n}\n"
}

/// 対照系: `core` に依存するが `impl Loader` を一切含まないクレート。
fn app_lib_rs_without_loader() -> &'static str {
    "use impact_fixture_core::fetch_items;\n\npub fn use_it() -> Vec<String> {\n    fetch_items()\n}\n"
}

/// 正例: 波及先クレートに `impl Loader for DemoItemsLoader` があれば
/// `affected_loaders` にその型名が入り、`requires_human_approval` が true になること。
#[test]
fn impact_reflects_affected_loader_and_requires_approval() {
    let project = write_impact_workspace(
        "loader-affected",
        &[
            ImpactMemberSpec {
                dir: "core",
                package_name: "impact-fixture-core",
                path_deps: &[],
                source: "pub fn fetch_items() -> Vec<String> {\n    vec![]\n}\n",
            },
            ImpactMemberSpec {
                dir: "app",
                package_name: "impact-fixture-app",
                path_deps: &["core"],
                source: app_lib_rs_with_loader(),
            },
        ],
    );

    let (code, stdout, stderr) = run_fw("impact", &["fetch_items"], &project);
    assert_eq!(
        code, 0,
        "fw impact は正常系で終了コード 0 を返す契約（stderr: {stderr}）"
    );
    assert!(
        json_array_contains_str(&stdout, "affected_loaders", "DemoItemsLoader"),
        "app が Loader 実装内で fetch_items を呼び出すため affected_loaders に DemoItemsLoader を含む（stdout: {stdout}）"
    );
    assert_eq!(
        json_bool_field(&stdout, "requires_human_approval"),
        Some(true),
        "affected_loaders が非空のため承認が必須（stdout: {stdout}）"
    );
}

/// 負例: 波及先クレートが `impl Loader` を含まなければ `affected_loaders` は
/// 空のままであり、`breaking_risk: low`（`fetch_items` を呼ぶクレートが 1 つの
/// みで境界クレートでもない）でも承認不要のままであること（誤検知なし）。
#[test]
fn impact_reports_empty_affected_loaders_when_no_loader_touched() {
    let project = write_impact_workspace(
        "loader-not-affected",
        &[
            ImpactMemberSpec {
                dir: "core",
                package_name: "impact-fixture-core",
                path_deps: &[],
                source: "pub fn fetch_items() -> Vec<String> {\n    vec![]\n}\n",
            },
            ImpactMemberSpec {
                dir: "app",
                package_name: "impact-fixture-app",
                path_deps: &["core"],
                source: app_lib_rs_without_loader(),
            },
        ],
    );

    let (code, stdout, stderr) = run_fw("impact", &["fetch_items"], &project);
    assert_eq!(
        code, 0,
        "fw impact は正常系で終了コード 0 を返す契約（stderr: {stderr}）"
    );
    assert!(
        stdout.contains("\"affected_loaders\":[]"),
        "impl Loader を含まないため affected_loaders は空のままであること（stdout: {stdout}）"
    );
}
