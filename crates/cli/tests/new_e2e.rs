//! `fw new`（TASK-13.4 相当、イシュー #350／複数テンプレート選択、
//! イシュー #378）の実バイナリ e2e テスト。
//!
//! `cli/tests/support/mod.rs`（`negative_cases.rs` 等）は `structure.toml` /
//! `deny.toml` 等の `fw gate` 専用フィクスチャを前提とするため、`fw new` には
//! 流用せず本ファイル内に薄いヘルパーを持つ（`support/mod.rs` 冒頭コメントが
//! 明文化する「テストターゲット独立の制約による意図的な複製」方針を踏襲）。
//!
//! 受け入れ条件（イシュー #350 計画 §1、イシュー #378 で全テンプレートへ
//! パラメタ化）:
//! 1. 同一引数での 2 回実行が同一出力（決定性テスト）
//! 2. 既存ディレクトリへの上書きは fail-closed（明示フラグなしでは拒否）
//! 3. 終了コード契約（0/1/2）を他サブコマンドと統一（未知 `--template` も
//!    使用法エラー・終了コード 2）
//!
//! さらに `cli/src/new_template.rs::TEMPLATES`（コンパイル時埋め込み）と
//! 正本 `templates/<name>/` の乖離を検出するドリフト検知テストを持つ
//! （`.claude/rules/ci.md` の cargo-deny pin ドリフト検知と同じ運用方針）。
//! vendor 同梱物（`templates/app/vendor/`）と正本 `core/`/`app/` の乖離検知は
//! 別ファイル `cli/tests/template_vendor_drift.rs` が担う。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// `--template` の allowlist（テストのパラメタ化に使う。
/// `cli/src/new_template.rs::TEMPLATES` と手動同期する必要があるが、件数の
/// ドリフトは `embedded_template_matches_templates_default_on_disk` 等の
/// per-template テストが `templates/<name>/` の走査を通じて機械的に検出する）。
///
/// `embed`（イシュー #410）は cargo パッケージを持たない静的単一ファイル
/// テンプレートのため、決定性・fail-closed・終了コード契約・ドリフト検知の
/// 汎用テストはここへの追加のみで自動的にパラメタ化されるが、cargo プロジェクト
/// 前提のパッケージ名置換テスト（`package_name_is_substituted_and_other_files_are_byte_identical_to_template`）
/// は `CARGO_SUBSTITUTED_TEMPLATE_NAMES` で別途限定する。
const TEMPLATE_NAMES: &[&str] = &["default", "app", "embed"];

/// cargo パッケージを持ち、`Cargo.toml`/`Cargo.lock`/`structure.toml` の
/// パッケージ名置換契約が適用されるテンプレートの一覧（`embed` を除く）。
const CARGO_SUBSTITUTED_TEMPLATE_NAMES: &[&str] = &["default", "app"];

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

// --- 受け入れ条件 1: 決定性（全テンプレート） ---

#[test]
fn same_args_produce_byte_identical_output_across_two_runs() {
    for name in TEMPLATE_NAMES {
        let scratch = unique_scratch_dir(&format!("determinism-{name}"));
        let dir_a = scratch.join("a");
        let dir_b = scratch.join("b");
        fs::create_dir_all(&dir_a).unwrap();
        fs::create_dir_all(&dir_b).unwrap();

        let (code_a, _, stderr_a) = run_fw_new(&[
            "demo-app",
            "--template",
            name,
            "--dir",
            &dir_a.to_string_lossy(),
        ]);
        assert_eq!(
            code_a, 0,
            "template `{name}` first run must succeed: {stderr_a}"
        );

        let (code_b, _, stderr_b) = run_fw_new(&[
            "demo-app",
            "--template",
            name,
            "--dir",
            &dir_b.to_string_lossy(),
        ]);
        assert_eq!(
            code_b, 0,
            "template `{name}` second run must succeed: {stderr_b}"
        );

        let tree_a = collect_tree(&dir_a.join("demo-app"));
        let tree_b = collect_tree(&dir_b.join("demo-app"));
        assert_eq!(
            tree_a, tree_b,
            "template `{name}`: two runs with identical arguments must produce byte-identical output"
        );
        assert!(
            !tree_a.is_empty(),
            "template `{name}`: expansion must not be empty"
        );

        let _ = fs::remove_dir_all(&scratch);
    }
}

// --- 受け入れ条件 2: fail-closed（全テンプレート） ---

#[test]
fn existing_target_is_rejected_without_force_and_accepted_with_force() {
    for name in TEMPLATE_NAMES {
        let scratch = unique_scratch_dir(&format!("fail-closed-{name}"));

        let (code1, _, _) = run_fw_new(&[
            "demo-app",
            "--template",
            name,
            "--dir",
            &scratch.to_string_lossy(),
        ]);
        assert_eq!(code1, 0, "template `{name}`");

        let target = scratch.join("demo-app");
        let marker = target.join("MARKER_UNTOUCHED");
        fs::write(&marker, b"sentinel").unwrap();

        let (code2, _, stderr2) = run_fw_new(&[
            "demo-app",
            "--template",
            name,
            "--dir",
            &scratch.to_string_lossy(),
        ]);
        assert_eq!(
            code2, 1,
            "template `{name}`: re-running against an existing target without --force must fail-closed"
        );
        assert!(!stderr2.is_empty(), "stderr must explain the rejection");
        assert!(
            marker.exists(),
            "existing content must be left untouched when rejected"
        );

        let (code3, _, stderr3) = run_fw_new(&[
            "demo-app",
            "--template",
            name,
            "--dir",
            &scratch.to_string_lossy(),
            "--force",
        ]);
        assert_eq!(
            code3, 0,
            "template `{name}`: --force must allow overwriting: {stderr3}"
        );

        let _ = fs::remove_dir_all(&scratch);
    }
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

/// イシュー #378: 未知の `--template` 値は使用法エラー（終了コード 2）とし、
/// stderr に利用可能テンプレート一覧を出す。
#[test]
fn unknown_template_is_usage_error_and_lists_available_templates() {
    let (code, _, stderr) = run_fw_new(&["demo-app", "--template", "nonexistent"]);
    assert_eq!(code, 2);
    assert!(
        stderr.contains("nonexistent"),
        "stderr must mention the rejected template name: {stderr}"
    );
    for name in TEMPLATE_NAMES {
        assert!(
            stderr.contains(name),
            "stderr must list available template `{name}`: {stderr}"
        );
    }
}

/// `--template` の値欠落は使用法エラー（`--dir` と同様の解析規則）。
#[test]
fn template_flag_missing_value_is_usage_error() {
    let (code, _, _) = run_fw_new(&["demo-app", "--template"]);
    assert_eq!(code, 2);
}

#[test]
fn success_is_exit_code_zero() {
    for name in TEMPLATE_NAMES {
        let scratch = unique_scratch_dir(&format!("exit-zero-{name}"));
        let (code, stdout, stderr) = run_fw_new(&[
            "demo-app",
            "--template",
            name,
            "--dir",
            &scratch.to_string_lossy(),
        ]);
        assert_eq!(code, 0, "template `{name}` stderr: {stderr}");
        assert!(stdout.contains("\"created\""));
        assert!(stdout.contains("\"files\""));
        assert!(
            stdout.contains(&format!("\"template\":\"{name}\"")),
            "stdout must echo the selected template name: {stdout}"
        );
        let _ = fs::remove_dir_all(&scratch);
    }
}

/// `--template` 省略時は `default` が選ばれる（イシュー #378 以前の
/// `fw new` 呼び出しとの後方互換性）。
#[test]
fn omitting_template_flag_defaults_to_default_template() {
    let scratch = unique_scratch_dir("template-omitted");
    let (code, stdout, stderr) = run_fw_new(&["demo-app", "--dir", &scratch.to_string_lossy()]);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("\"template\":\"default\""),
        "stdout={stdout}"
    );
    let _ = fs::remove_dir_all(&scratch);
}

// --- 置換検証（全テンプレート） ---

#[test]
fn package_name_is_substituted_and_other_files_are_byte_identical_to_template() {
    for name in CARGO_SUBSTITUTED_TEMPLATE_NAMES {
        let scratch = unique_scratch_dir(&format!("substitution-{name}"));
        let (code, _, stderr) = run_fw_new(&[
            "demo-app",
            "--template",
            name,
            "--dir",
            &scratch.to_string_lossy(),
        ]);
        assert_eq!(code, 0, "template `{name}` stderr: {stderr}");

        let target = scratch.join("demo-app");
        let needle = format!("fandhe-frontend-template-{name}");

        let cargo_toml = fs::read_to_string(target.join("Cargo.toml")).unwrap();
        assert!(cargo_toml.contains("name = \"demo-app\""));
        assert!(!cargo_toml.contains(&needle));

        let cargo_lock = fs::read_to_string(target.join("Cargo.lock")).unwrap();
        assert!(cargo_lock.contains("name = \"demo-app\""));
        assert!(!cargo_lock.contains(&needle));

        // structure.toml（イシュー #351）も同じ allowlist で置換される。fw gate
        // （cli/src/gate.rs）はここで宣言される `crate = "..."` を唯一の情報源と
        // するため、プロジェクト名への置換漏れは生成直後の fw gate BLOCKED
        // （宣言クレート不在の fail-closed）に直結する。
        let structure_toml = fs::read_to_string(target.join("structure.toml")).unwrap();
        assert!(structure_toml.contains("crate = \"demo-app\""));
        assert!(!structure_toml.contains(&needle));

        let _ = fs::remove_dir_all(&scratch);
    }
}

/// `embed` テンプレート（イシュー #410）は cargo パッケージを持たず
/// `substituted_files` が空（`cli/src/new_template.rs::TEMPLATES`）のため、
/// `needle`（`fandhe-frontend-template-embed`）はどのファイルにも出現せず、生成物は
/// テンプレート正本と全ファイルバイト一致になる。
#[test]
fn embed_template_output_is_byte_identical_to_template_and_contains_no_needle() {
    let scratch = unique_scratch_dir("embed-no-substitution");
    let (code, _, stderr) = run_fw_new(&[
        "demo-embed",
        "--template",
        "embed",
        "--dir",
        &scratch.to_string_lossy(),
    ]);
    assert_eq!(code, 0, "stderr: {stderr}");

    let target = scratch.join("demo-embed");
    let template_root = template_root_dir("embed");
    let generated = collect_tree(&target);
    let original = collect_tree(&template_root);
    assert_eq!(
        generated, original,
        "embed template has no package-name substitution, so output must be byte-identical to templates/embed/"
    );

    let needle = "fandhe-frontend-template-embed";
    for (rel_path, bytes, _) in &generated {
        assert!(
            !String::from_utf8_lossy(bytes).contains(needle),
            "generated file `{rel_path}` must not contain the unused placeholder `{needle}`"
        );
    }

    let _ = fs::remove_dir_all(&scratch);
}

/// 置換対象外ファイル（`default` テンプレートの `tests/negative_type_error.rs`
/// の doc コメント内言及）はテンプレートとバイト一致すること（意図的に
/// 置換しない契約、`new.rs::Template::substituted_files` allowlist の境界）。
#[test]
fn default_template_negative_type_error_test_is_byte_identical_to_source() {
    let scratch = unique_scratch_dir("substitution-boundary");
    let (code, _, stderr) = run_fw_new(&["demo-app", "--dir", &scratch.to_string_lossy()]);
    assert_eq!(code, 0, "stderr: {stderr}");

    let target = scratch.join("demo-app");
    let template_root = template_root_dir("default");
    let generated = fs::read(target.join("tests/negative_type_error.rs")).unwrap();
    let original = fs::read(template_root.join("tests/negative_type_error.rs")).unwrap();
    assert_eq!(generated, original);

    let _ = fs::remove_dir_all(&scratch);
}

// --- ドリフト検知: TEMPLATES と templates/<name>/ の一致（全テンプレート） ---

fn template_root_dir(name: &str) -> PathBuf {
    // このテストバイナリは `crates/cli/` 配下でビルドされるため、`templates/`
    // （ワークスペースルート直下、クレートではないため移設対象外）へは
    // 2 段の親ディレクトリを辿る（イシュー #436）。
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/cli/ has a workspace root two levels up")
        .join("templates")
        .join(name)
}

/// `--template` ごとに置換対象ファイル（プロジェクト名を含むため展開後は
/// 正本とバイト一致しない）の allowlist（`cli/src/new_template.rs` の
/// `Template::substituted_files` と同期する）。cargo パッケージを持つ
/// `default`/`app` は `Cargo.toml`/`Cargo.lock`/`structure.toml` を置換するが、
/// cargo パッケージを持たない `embed`（イシュー #410）は置換対象がなく空。
fn substituted_relative_paths(name: &str) -> &'static [&'static str] {
    if CARGO_SUBSTITUTED_TEMPLATE_NAMES.contains(&name) {
        &["Cargo.toml", "Cargo.lock", "structure.toml"]
    } else {
        &[]
    }
}

#[test]
fn embedded_template_matches_templates_on_disk() {
    for name in TEMPLATE_NAMES {
        let template_root = template_root_dir(name);
        let on_disk = collect_tree(&template_root);
        let substituted_relative_paths = substituted_relative_paths(name);

        let scratch = unique_scratch_dir(&format!("drift-check-{name}"));
        let (code, _, stderr) = run_fw_new(&[
            "drift-check-app",
            "--template",
            name,
            "--dir",
            &scratch.to_string_lossy(),
        ]);
        assert_eq!(code, 0, "template `{name}` stderr: {stderr}");
        let expanded = collect_tree(&scratch.join("drift-check-app"));

        assert_eq!(
            on_disk.len(),
            expanded.len(),
            "templates/{name}/ file count must match the embedded manifest \
             (cli/src/new_template.rs::TEMPLATES) — update the manifest when \
             templates/{name}/ gains or loses files"
        );

        for (disk_entry, expanded_entry) in on_disk.iter().zip(expanded.iter()) {
            let (disk_path, disk_bytes, disk_exec) = disk_entry;
            let (expanded_path, expanded_bytes, expanded_exec) = expanded_entry;
            assert_eq!(
                disk_path, expanded_path,
                "template `{name}`: relative path set must match between templates/{name}/ and the embedded manifest"
            );
            assert_eq!(
                disk_exec, expanded_exec,
                "template `{name}`: executable bit for `{disk_path}` must match between templates/{name}/ and the embedded manifest"
            );
            // Cargo.toml/Cargo.lock/structure.toml はプロジェクト名を置換するため
            // 内容は一致しない（置換前提の検証は substitution テストが別途担う）。
            if !substituted_relative_paths.contains(&disk_path.as_str()) {
                assert_eq!(
                    disk_bytes, expanded_bytes,
                    "template `{name}`: content of `{disk_path}` must be byte-identical between templates/{name}/ \
                     and the embedded manifest"
                );
            }
        }

        let _ = fs::remove_dir_all(&scratch);
    }
}

// --- `fw new --example`（イシュー #500） ---

/// `--example` の allowlist（`cli/src/new_template.rs::EXAMPLES` と手動同期）。
const EXAMPLE_NAMES: &[&str] = &["ssr-routing", "interactive-view-transitions"];

fn example_root_dir(name: &str) -> PathBuf {
    // このテストバイナリは `crates/cli/` 配下でビルドされるため、`examples/`
    // （ワークスペースルート直下、クレートではないため移設対象外）へは
    // 2 段の親ディレクトリを辿る（イシュー #436）。
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/cli/ has a workspace root two levels up")
        .join("examples")
        .join(name)
}

/// 成功時 JSON は `"example":"<name>"` キーを持ち、`--template` 経路専用の
/// `"template"` キーは含めない（実装計画 §2.5）。
#[test]
fn example_success_json_has_example_key_and_no_template_key() {
    for name in EXAMPLE_NAMES {
        let scratch = unique_scratch_dir(&format!("example-json-{name}"));
        let (code, stdout, stderr) = run_fw_new(&[
            "demo-example",
            "--example",
            name,
            "--dir",
            &scratch.to_string_lossy(),
        ]);
        assert_eq!(code, 0, "example `{name}` stderr: {stderr}");
        assert!(stdout.contains("\"created\""));
        assert!(stdout.contains("\"files\""));
        assert!(
            stdout.contains(&format!("\"example\":\"{name}\"")),
            "stdout must echo the selected example name: {stdout}"
        );
        assert!(
            !stdout.contains("\"template\""),
            "example output must not contain the `--template` JSON key: {stdout}"
        );
        let _ = fs::remove_dir_all(&scratch);
    }
}

/// 決定性（イシュー #350 の受け入れ条件 1 を `--example` 経路にも適用）。
#[test]
fn example_same_args_produce_byte_identical_output_across_two_runs() {
    for name in EXAMPLE_NAMES {
        let scratch = unique_scratch_dir(&format!("example-determinism-{name}"));
        let dir_a = scratch.join("a");
        let dir_b = scratch.join("b");
        fs::create_dir_all(&dir_a).unwrap();
        fs::create_dir_all(&dir_b).unwrap();

        let (code_a, _, stderr_a) = run_fw_new(&[
            "demo-example",
            "--example",
            name,
            "--dir",
            &dir_a.to_string_lossy(),
        ]);
        assert_eq!(code_a, 0, "example `{name}` first run: {stderr_a}");

        let (code_b, _, stderr_b) = run_fw_new(&[
            "demo-example",
            "--example",
            name,
            "--dir",
            &dir_b.to_string_lossy(),
        ]);
        assert_eq!(code_b, 0, "example `{name}` second run: {stderr_b}");

        let tree_a = collect_tree(&dir_a.join("demo-example"));
        let tree_b = collect_tree(&dir_b.join("demo-example"));
        assert_eq!(
            tree_a, tree_b,
            "example `{name}`: two runs with identical arguments must produce byte-identical output"
        );
        assert!(
            !tree_a.is_empty(),
            "example `{name}`: expansion must not be empty"
        );

        let _ = fs::remove_dir_all(&scratch);
    }
}

/// examples は置換を行わない（実装計画 §2.2）ため、生成物は正本
/// `examples/<name>/` と全ファイルバイト一致になる。これは
/// `crates/cli/embedded-examples/<name>/` ↔ 正本のドリフト検知も兼ねる
/// （埋め込みマニフェストが正本と乖離していれば本テストが検出する）。
#[test]
fn example_output_is_byte_identical_to_source_example() {
    for name in EXAMPLE_NAMES {
        let scratch = unique_scratch_dir(&format!("example-drift-{name}"));
        let (code, _, stderr) = run_fw_new(&[
            "demo-example",
            "--example",
            name,
            "--dir",
            &scratch.to_string_lossy(),
        ]);
        assert_eq!(code, 0, "example `{name}` stderr: {stderr}");

        let target = scratch.join("demo-example");
        let source_root = example_root_dir(name);
        let generated = collect_tree(&target);
        let original = collect_tree(&source_root);
        assert_eq!(
            generated, original,
            "example `{name}` has no package-name substitution, so output must be byte-identical to examples/{name}/"
        );

        let _ = fs::remove_dir_all(&scratch);
    }
}

/// `--template` と `--example` の同時指定は使用法エラー（終了コード 2）。
#[test]
fn template_and_example_together_is_usage_error() {
    let (code, _, _) = run_fw_new(&[
        "demo-example",
        "--template",
        "default",
        "--example",
        "ssr-routing",
    ]);
    assert_eq!(code, 2);
}

/// 未知の `--example` 値は使用法エラー（終了コード 2）とし、stderr に
/// 利用可能サンプル一覧を出す（`--template` と同じ契約）。
#[test]
fn unknown_example_is_usage_error_and_lists_available_examples() {
    let (code, _, stderr) = run_fw_new(&["demo-example", "--example", "nonexistent"]);
    assert_eq!(code, 2);
    assert!(
        stderr.contains("nonexistent"),
        "stderr must mention the rejected example name: {stderr}"
    );
    for name in EXAMPLE_NAMES {
        assert!(
            stderr.contains(name),
            "stderr must list available example `{name}`: {stderr}"
        );
    }
}

/// `--example` の値欠落は使用法エラー（`--template`/`--dir` と同様の解析規則）。
#[test]
fn example_flag_missing_value_is_usage_error() {
    let (code, _, _) = run_fw_new(&["demo-example", "--example"]);
    assert_eq!(code, 2);
}
