//! `fw new`（TASK-13.4 相当、イシュー #350）の実バイナリ e2e テスト。
//!
//! `cli/tests/support/mod.rs`（`negative_cases.rs` 等）は `structure.toml` /
//! `deny.toml` 等の `fw gate` 専用フィクスチャを前提とするため、`fw new` には
//! 流用せず本ファイル内に薄いヘルパーを持つ（`support/mod.rs` 冒頭コメントが
//! 明文化する「テストターゲット独立の制約による意図的な複製」方針を踏襲）。
//!
//! 受け入れ条件（イシュー #350 計画 §1）:
//! 1. 同一引数での 2 回実行が同一出力（決定性テスト）
//! 2. 既存ディレクトリへの上書きは fail-closed（明示フラグなしでは拒否）
//! 3. 終了コード契約（0/1/2）を他サブコマンドと統一
//!
//! さらに `cli/src/new_template.rs::TEMPLATE_FILES`（コンパイル時埋め込み）と
//! 正本 `templates/default/` の乖離を検出するドリフト検知テストを持つ
//! （`.claude/rules/ci.md` の cargo-deny pin ドリフト検知と同じ運用方針）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// `fw new` を実バイナリとして起動し (終了コード, stdout, stderr) を返す。
fn run_fw_new(extra_args: &[&str]) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_fw"))
        .arg("new")
        .args(extra_args)
        .output()
        .expect("failed to spawn `fw` binary");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// 一意な一時ディレクトリを用意する（テスト名を含み、並列実行・
/// self-hosted runner の共有 /tmp と衝突しない）。
fn unique_scratch_dir(label: &str) -> PathBuf {
    let root = std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    let dir = root.join(format!(
        "fw-new-e2e-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("failed to create scratch dir");
    dir
}

/// ディレクトリを再帰走査し、(相対パス, バイト列, Unix 実行ビット) の一覧を
/// パスでソートして返す。決定性テスト・ドリフト検知テストの両方で使う。
fn collect_tree(root: &Path) -> Vec<(String, Vec<u8>, bool)> {
    fn walk(base: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>, bool)>) {
        for entry in fs::read_dir(dir).expect("failed to read dir") {
            let entry = entry.expect("failed to read dir entry");
            let path = entry.path();
            if path.is_dir() {
                walk(base, &path, out);
            } else {
                let rel = path
                    .strip_prefix(base)
                    .expect("entry must be under base")
                    .to_string_lossy()
                    .replace('\\', "/");
                let contents = fs::read(&path).expect("failed to read file");
                #[cfg(unix)]
                let executable = {
                    use std::os::unix::fs::PermissionsExt;
                    fs::metadata(&path).unwrap().permissions().mode() & 0o111 != 0
                };
                #[cfg(not(unix))]
                let executable = false;
                out.push((rel, contents, executable));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

// --- 受け入れ条件 1: 決定性 ---

#[test]
fn same_args_produce_byte_identical_output_across_two_runs() {
    let scratch = unique_scratch_dir("determinism");
    let dir_a = scratch.join("a");
    let dir_b = scratch.join("b");
    fs::create_dir_all(&dir_a).unwrap();
    fs::create_dir_all(&dir_b).unwrap();

    let (code_a, _, stderr_a) = run_fw_new(&["demo-app", "--dir", &dir_a.to_string_lossy()]);
    assert_eq!(code_a, 0, "first run must succeed: {stderr_a}");

    let (code_b, _, stderr_b) = run_fw_new(&["demo-app", "--dir", &dir_b.to_string_lossy()]);
    assert_eq!(code_b, 0, "second run must succeed: {stderr_b}");

    let tree_a = collect_tree(&dir_a.join("demo-app"));
    let tree_b = collect_tree(&dir_b.join("demo-app"));
    assert_eq!(
        tree_a, tree_b,
        "two runs with identical arguments must produce byte-identical output"
    );
    assert!(!tree_a.is_empty(), "expansion must not be empty");

    let _ = fs::remove_dir_all(&scratch);
}

// --- 受け入れ条件 2: fail-closed ---

#[test]
fn existing_target_is_rejected_without_force_and_accepted_with_force() {
    let scratch = unique_scratch_dir("fail-closed");

    let (code1, _, _) = run_fw_new(&["demo-app", "--dir", &scratch.to_string_lossy()]);
    assert_eq!(code1, 0);

    let target = scratch.join("demo-app");
    let marker = target.join("MARKER_UNTOUCHED");
    fs::write(&marker, b"sentinel").unwrap();

    let (code2, _, stderr2) = run_fw_new(&["demo-app", "--dir", &scratch.to_string_lossy()]);
    assert_eq!(
        code2, 1,
        "re-running against an existing target without --force must fail-closed"
    );
    assert!(!stderr2.is_empty(), "stderr must explain the rejection");
    assert!(
        marker.exists(),
        "existing content must be left untouched when rejected"
    );

    let (code3, _, stderr3) =
        run_fw_new(&["demo-app", "--dir", &scratch.to_string_lossy(), "--force"]);
    assert_eq!(code3, 0, "--force must allow overwriting: {stderr3}");

    let _ = fs::remove_dir_all(&scratch);
}

// --- 受け入れ条件 3: 終了コード契約 ---

#[test]
fn missing_args_is_usage_error() {
    let (code, _, _) = run_fw_new(&[]);
    assert_eq!(code, 2);
}

#[test]
fn invalid_project_names_are_usage_errors() {
    for bad_name in ["../evil", "UPPER", "a/b", ""] {
        let (code, _, _) = run_fw_new(&[bad_name]);
        assert_eq!(code, 2, "expected usage error for name `{bad_name}`");
    }
}

#[test]
fn unknown_flag_is_usage_error() {
    let (code, _, _) = run_fw_new(&["demo-app", "--unknown-flag"]);
    assert_eq!(code, 2);
}

#[test]
fn success_is_exit_code_zero() {
    let scratch = unique_scratch_dir("exit-zero");
    let (code, stdout, stderr) = run_fw_new(&["demo-app", "--dir", &scratch.to_string_lossy()]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("\"created\""));
    assert!(stdout.contains("\"files\""));
    let _ = fs::remove_dir_all(&scratch);
}

// --- 置換検証 ---

#[test]
fn package_name_is_substituted_and_other_files_are_byte_identical_to_template() {
    let scratch = unique_scratch_dir("substitution");
    let (code, _, stderr) = run_fw_new(&["demo-app", "--dir", &scratch.to_string_lossy()]);
    assert_eq!(code, 0, "stderr: {stderr}");

    let target = scratch.join("demo-app");

    let cargo_toml = fs::read_to_string(target.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("name = \"demo-app\""));
    assert!(!cargo_toml.contains("rws-template-default"));

    let cargo_lock = fs::read_to_string(target.join("Cargo.lock")).unwrap();
    assert!(cargo_lock.contains("name = \"demo-app\""));
    assert!(!cargo_lock.contains("rws-template-default"));

    // 置換対象外ファイルはテンプレートとバイト一致すること
    // （negative_type_error.rs の doc コメント内言及は置換しない契約）。
    let template_root = template_root_dir();
    let generated = fs::read(target.join("tests/negative_type_error.rs")).unwrap();
    let original = fs::read(template_root.join("tests/negative_type_error.rs")).unwrap();
    assert_eq!(generated, original);

    let _ = fs::remove_dir_all(&scratch);
}

// --- ドリフト検知: TEMPLATE_FILES と templates/default/ の一致 ---

fn template_root_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli/ has a parent workspace root")
        .join("templates/default")
}

#[test]
fn embedded_template_matches_templates_default_on_disk() {
    let template_root = template_root_dir();
    let on_disk = collect_tree(&template_root);

    let scratch = unique_scratch_dir("drift-check");
    let (code, _, stderr) = run_fw_new(&["drift-check-app", "--dir", &scratch.to_string_lossy()]);
    assert_eq!(code, 0, "stderr: {stderr}");
    let expanded = collect_tree(&scratch.join("drift-check-app"));

    assert_eq!(
        on_disk.len(),
        expanded.len(),
        "templates/default/ file count must match the embedded manifest \
         (cli/src/new_template.rs::TEMPLATE_FILES) — update the manifest when \
         templates/default/ gains or loses files"
    );

    for (disk_entry, expanded_entry) in on_disk.iter().zip(expanded.iter()) {
        let (disk_path, disk_bytes, disk_exec) = disk_entry;
        let (expanded_path, expanded_bytes, expanded_exec) = expanded_entry;
        assert_eq!(
            disk_path, expanded_path,
            "relative path set must match between templates/default/ and the embedded manifest"
        );
        assert_eq!(
            disk_exec, expanded_exec,
            "executable bit for `{disk_path}` must match between templates/default/ and the embedded manifest"
        );
        // Cargo.toml/Cargo.lock は package 名を置換するため内容は一致しない
        // （置換前提の検証は substitution テストが別途担う）。
        if disk_path != "Cargo.toml" && disk_path != "Cargo.lock" {
            assert_eq!(
                disk_bytes, expanded_bytes,
                "content of `{disk_path}` must be byte-identical between templates/default/ \
                 and the embedded manifest"
            );
        }
    }

    let _ = fs::remove_dir_all(&scratch);
}
