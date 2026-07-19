//! `structure.toml` の予約名 `root`（クレートがワークスペースルート直下に
//! 直接配置される構成。`fw new` が生成するプロジェクトの唯一のクレート、
//! イシュー #350/#351）に対する `fw structure` / `fw impact` の実バイナリ e2e
//! （イシュー #353）。
//!
//! 対応前は以下 2 点がいずれも誤検知・解析不能だった（本ファイルはこの
//! リグレッションを固定する）:
//!
//! - `fw structure`: `main.rs::run_structure` のディレクトリ実在確認が
//!   `project_dir.join("root")` を見るため、`[directories.root]` 宣言の
//!   プロジェクトは「declared directory does not exist」で必ず exit 1 になっていた。
//! - `fw impact`: `impact.rs::member_dir_name` が `manifest_dir == workspace_root`
//!   のとき `ImpactError::Scan`（「manifest_dir equals workspace_root」）を
//!   返すため、ルート直下クレートのプロジェクトでは `fw impact` 全体が失敗していた。

mod support;

use std::fs;
use std::process::Command;
use support::{run_fw, scratch_root, ScratchProject};

/// `<workspace_root>` 直下にクレートを直接配置した最小プロジェクト
/// （`fw new` が生成する `templates/default/` 相当の骨格の縮小版）を
/// 一意な一時ディレクトリへ書き出す。
///
/// ```text
/// <fixture>/
/// ├── Cargo.toml       ([package] + 空の [workspace]。単一パッケージ自身が
/// │                      ワークスペースルートになる構成、templates/default/Cargo.toml
/// │                      と同型）
/// ├── structure.toml   ([directories.root])
/// └── src/main.rs      (find_item を定義。fw impact の解析対象シンボル)
/// ```
fn write_root_crate_project() -> ScratchProject {
    let dest = scratch_root().join(format!(
        "impact-root-crate-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dest);
    let src = dest.join("src");
    fs::create_dir_all(&src).expect("create fixture src dir");

    fs::write(
        dest.join("Cargo.toml"),
        "[package]\nname = \"impact-root-fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\nlicense = \"MIT\"\npublish = false\n\n[workspace]\n",
    )
    .expect("write Cargo.toml");

    fs::write(
        dest.join("structure.toml"),
        "[manifest]\nversion = 1\n\n[directories.root]\nrole = \"distribution\"\ncrate = \"impact-root-fixture\"\ndescription = \"root convention fixture (issue #353)\"\n",
    )
    .expect("write structure.toml");

    // `component_boundary::extract_from_source`（定義元特定の走査対象）は
    // トップレベル `pub fn` 宣言のみを対象とするため、`support::baseline_main_rs`
    // の非公開 `find_item` ではなく `pub fn` として定義し直す。
    fs::write(
        src.join("main.rs"),
        "pub fn find_item(id: &str) -> Option<String> {\n    if id == \"1\" {\n        Some(\"widget\".to_string())\n    } else {\n        None\n    }\n}\n\nfn main() {\n    println!(\"{:?}\", find_item(\"1\"));\n}\n",
    )
    .expect("write src/main.rs");

    let lockfile_output = Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(&dest)
        .output()
        .expect("cargo generate-lockfile の起動に失敗した");
    assert!(
        lockfile_output.status.success(),
        "cargo generate-lockfile --offline に失敗した: {}",
        String::from_utf8_lossy(&lockfile_output.stderr)
    );

    ScratchProject(dest)
}

/// `fw structure` がルート直下クレート構成で exit 0 を返すこと
/// （旧実装は「declared directory does not exist」で必ず exit 1 だった）。
#[test]
fn fw_structure_succeeds_for_root_convention_project() {
    let project = write_root_crate_project();
    let (code, stdout, stderr) = run_fw("structure", &[], &project);
    assert_eq!(
        code, 0,
        "fw structure は root 慣習のプロジェクトで exit 0 を返す契約（stdout: {stdout}, stderr: {stderr}）"
    );
    assert!(
        stdout.contains("\"directories\""),
        "正常系では directories を含む JSON を出力する（stdout: {stdout}）"
    );
}

/// `fw impact <symbol>` がルート直下クレート構成でも解析できること
/// （旧実装は member_dir_name が Scan エラーを返し `fw impact` 全体が失敗していた）。
#[test]
fn fw_impact_analyzes_symbol_in_root_convention_project() {
    let project = write_root_crate_project();
    let (code, stdout, stderr) = run_fw("impact", &["find_item"], &project);
    assert_eq!(
        code, 0,
        "fw impact は root 慣習のプロジェクトで exit 0 を返す契約（stdout: {stdout}, stderr: {stderr}）"
    );
    assert!(
        stdout.contains("\"defined_in_crate\":\"impact-root-fixture\""),
        "find_item の定義元クレートとして impact-root-fixture が解決されること（stdout: {stdout}）"
    );
    assert!(
        stdout.contains("\"defined_in_file\":\"src/main.rs\""),
        "定義元ファイルはワークスペースルート相対で `src/main.rs`（`root/src/main.rs` ではない）こと（stdout: {stdout}）"
    );
}
