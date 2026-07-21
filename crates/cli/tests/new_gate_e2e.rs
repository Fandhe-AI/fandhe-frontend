//! `fw new` 生成直後に `fw gate` が PASS することを固定する e2e テスト
//! （イシュー #351、親イシュー #338「全プロジェクトが同一構成 = `fw gate`
//! がそのまま効く」の受け入れ条件）。
//!
//! `fw new`（イシュー #350、`cli/src/new.rs`）は `templates/default/` を
//! 決定的に展開するだけであり、`fw gate`（`cli/src/gate.rs::run_gate`）は
//! `structure.toml` を唯一の情報源として検証対象クレートを決定する。両者の
//! 前提（テンプレートの `structure.toml` / `clippy.toml` / `deny.toml` と
//! gate の実装）がドリフトすると、生成直後のプロジェクトが無編集で
//! BLOCKED になりかねない。本ファイルは
//! `fw new <name> --dir <scratch>` → `fw gate --project <scratch>/<name>`
//! を実バイナリとして直列実行し、5 チェック（type_check /
//! default_escape_check / lint / test / policy）が出揃うこと、および
//! type_check / default_escape_check / lint / test が常に PASS することを
//! 断定する回帰テストである。
//!
//! `policy`（cargo-deny 依存）のみ実行環境（cargo-deny の導入有無）で結果が
//! 変わるため、`cli/tests/scenarios/bugfix_escape.rs::baseline_passes_gate`
//! が確立した「スキップ・`#[ignore]` を使わず両分岐を断定する」方針を
//! そのまま踏襲する（未導入環境では `policy` の failed 出力が
//! `environment error: ` で始まる＝コード起因ではなく環境起因のみで
//! あることまで確認し、BLOCKED を黙示的な想定外失敗と混同しない）。
//!
//! 本ファイルの削除・弱体化（アサーション削除・`#[ignore]` 付与等）は
//! `fw new`/`fw gate` 前提のドリフト検知回帰を失わせるため行わない
//! （`.claude/rules/coding-rust.md` 「テストの `#[ignore]` 追加でごまかさない」）。
//!
//! `cli/tests/support/mod.rs`（`negative_cases.rs` 等が使う `fw gate` 専用
//! フィクスチャ基盤）の `run_fw_gate`/`check_passed`/`cargo_deny_available`/
//! `ScratchProject` を再利用しつつ、`fw new` の起動と scratch ディレクトリの
//! 準備のみ `cli/tests/new_e2e.rs` と同方針の薄いローカルヘルパーを持つ
//! （`new_e2e.rs` 冒頭コメントが明文化する「テストターゲット独立の制約による
//! 意図的な複製」を踏襲）。

mod support;

use std::path::PathBuf;
use std::process::Command;
use support::{cargo_deny_available, check_passed, run_fw, run_fw_gate, ScratchProject};

/// `fw new` を実バイナリとして起動し (終了コード, stdout, stderr) を返す
/// （`cli/tests/new_e2e.rs::run_fw_new` と同一方針。テストターゲット独立の
/// 制約による意図的な複製）。
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

/// `fw new` の展開先となる一意な scratch ディレクトリ（`CARGO_TARGET_TMPDIR`
/// 配下、`ScratchProject` の Drop ガードで後始末する）。
///
/// self-hosted runner の共有 `/tmp`・共有 `CARGO_TARGET_DIR` との衝突を避ける
/// ため、テスト名・PID・ナノ秒を含めて一意化する
/// （`cli/tests/support/mod.rs::scratch_root` と同一方針、`.claude/rules/ci.md`
/// 準拠）。
fn unique_scratch_dir() -> PathBuf {
    let root = support_scratch_root();
    let dir = root.join(format!(
        "fw-new-gate-e2e-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("failed to create scratch dir");
    dir
}

fn support_scratch_root() -> PathBuf {
    std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
}

/// `fw new` で生成した直後のプロジェクトに対し `fw gate` を実行し、
/// チェックセット（5 件）が JSON にすべて現れること・環境に依存しない
/// 4 チェック（type_check / default_escape_check / lint / test）が常に
/// PASS することを断定する。
///
/// `policy` は cargo-deny の導入有無で分岐し、両分岐とも「コード起因ではなく
/// 環境要因でのみ BLOCKED になり得る」ことまで確認する
/// （`bugfix_escape.rs::baseline_passes_gate` と同一方針）。
#[test]
fn fw_new_output_passes_fw_gate() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) =
        run_fw_new(&["gate-pass-app", "--dir", &scratch.to_string_lossy()]);
    assert_eq!(
        new_code, 0,
        "fw new が失敗した: stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("gate-pass-app");
    let (gate_code, gate_stdout, gate_stderr) = run_fw_gate(&project_dir);

    // チェックセット自体のドリフト検知（gate.rs 側でチェックが増減しても
    // ここで検出できるよう、6 件すべてが JSON に現れることを断定する）。
    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "test",
        "policy",
    ] {
        assert!(
            gate_stdout.contains(&format!("\"name\":\"{name}\"")),
            "fw gate のレポートにチェック `{name}` が現れない: stdout={gate_stdout}"
        );
    }

    // 環境（cargo-deny 導入有無）に依存しない 4 チェックは常に PASS する
    // はず。ここが failed の場合はテンプレートと gate 前提のドリフト
    // （clippy.toml の disallowed-methods 欠落・structure.toml の宣言
    // クレート不一致・型不正コードの混入等）を意味する。
    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "test",
    ] {
        assert_eq!(
            check_passed(&gate_stdout, name),
            Some(true),
            "fw new 生成直後のプロジェクトで `{name}` が失敗した（テンプレートと \
             fw gate の前提がドリフトしている）: stdout={gate_stdout} stderr={gate_stderr}"
        );
    }

    if cargo_deny_available() {
        assert_eq!(
            gate_code, 0,
            "cargo-deny 導入環境では fw new 生成直後は PASS するはず: \
             stdout={gate_stdout} stderr={gate_stderr}"
        );
        assert!(
            gate_stdout.contains("\"gate_result\":\"PASS\""),
            "stdout={gate_stdout}"
        );
        assert_eq!(
            check_passed(&gate_stdout, "policy"),
            Some(true),
            "stdout={gate_stdout}"
        );
    } else {
        // cargo-deny 未導入環境では policy のみ fail-closed で failed になり
        // 全体として BLOCKED になるが、その failed 出力が「環境エラーである
        // こと」を示す `environment error: ` プレフィックスで始まることまで
        // 確認する（`.claude/rules/ci.md` §ツール前提の明示・gate.rs
        // `ENVIRONMENT_ERROR_PREFIX` 参照）。これにより「テンプレート/gate
        // 前提のコード起因ドリフトによる BLOCKED」と「cargo-deny 未導入と
        // いう環境要因による BLOCKED」を取り違えない。
        assert_eq!(
            gate_code, 1,
            "cargo-deny 未導入環境では policy の fail-closed により BLOCKED \
             (終了コード 1) のはず: stdout={gate_stdout}"
        );
        assert!(
            gate_stdout.contains("\"gate_result\":\"BLOCKED\""),
            "stdout={gate_stdout}"
        );
        assert_eq!(
            check_passed(&gate_stdout, "policy"),
            Some(false),
            "stdout={gate_stdout}"
        );
        assert!(
            gate_stdout.contains("environment error: "),
            "policy の failed 出力は environment error であることを明示する \
             プレフィックスを含むはず（コード起因の FAIL と誤認させない）: \
             stdout={gate_stdout}"
        );
    }
}

/// PR #358 Bugbot 指摘（イシュー #351）の e2e 回帰テスト:
/// `fw_new_output_passes_fw_gate` の `default_escape_check == Some(true)` は
/// 「未レビュー `raw_html()` が存在しないので PASS」と「走査対象パスが
/// 実在せず走査自体がスキップされたので PASS（無意味な PASS）」を区別
/// できない。本テストは `fw new` 生成直後のプロジェクトの実クレート配置先
/// （プロジェクトルート直下 `src/`）に未レビューの `raw_html()` 呼び出しを
/// 注入し、`default_escape_check` が実際に走査を実行して violation を
/// 検出（failed）することを断定する。これにより `structure.toml` の
/// `[directories.root]` 予約名規約と `fw gate`（`escape_check_src_dir`,
/// `cli/src/gate.rs`）の前提が一致していること（保険層が生成直後の
/// プロジェクトで機能していること）を固定する。
#[test]
fn fw_new_output_default_escape_check_detects_injected_violation_in_root_src() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) = run_fw_new(&[
        "gate-pass-violation-app",
        "--dir",
        &scratch.to_string_lossy(),
    ]);
    assert_eq!(
        new_code, 0,
        "fw new が失敗した: stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("gate-pass-violation-app");

    // `fw new` が展開したクレートは `<project_dir>/src/main.rs`
    // （プロジェクトルート直下、テンプレートの `[directories.root]` 規約）
    // に配置される。ここへ未レビューの `raw_html()` 呼び出しを追記する
    // （`#[expect(clippy::disallowed_methods, ...)]` を伴わないため
    // `default_escape_check` の違反として検出されるはず）。
    let main_rs = project_dir.join("src").join("main.rs");
    let mut content = std::fs::read_to_string(&main_rs).expect("failed to read src/main.rs");
    content.push_str("\nfn unreviewed_raw_html_probe() {\n    raw_html(\"x\");\n}\n");
    std::fs::write(&main_rs, content).expect("failed to write src/main.rs");

    let (gate_code, gate_stdout, gate_stderr) = run_fw_gate(&project_dir);

    assert_eq!(
        check_passed(&gate_stdout, "default_escape_check"),
        Some(false),
        "src/ 直下（`root` 規約）に注入した未レビュー raw_html() 呼び出しが \
         default_escape_check で検出されなかった（走査がスキップされている \
         疑い、PR #358 Bugbot 指摘のドリフト再発）: stdout={gate_stdout} \
         stderr={gate_stderr}"
    );
    assert!(
        gate_stdout.contains("main.rs"),
        "default_escape_check の failed 出力は違反ファイルを file:line で \
         列挙するはず: stdout={gate_stdout}"
    );
    assert_ne!(
        gate_code, 0,
        "default_escape_check が failed の場合 fw gate 全体も非ゼロで \
         終了するはず: stdout={gate_stdout}"
    );
}

/// イシュー #353: `fw new` 生成直後のプロジェクトで `fw structure` /
/// `fw impact` が「`root` 慣習未対応」由来の解析不能に陥らないことを固定する。
///
/// - `fw structure`: 旧実装は `project_dir.join("root")`（実在しないパス）を
///   見て「declared directory does not exist」で必ず exit 1 だった
///   （`structure::dir_fs_path` 導入により修正）。
/// - `fw impact`: 旧実装は `member_dir_name` が `manifest_dir == workspace_root`
///   を `ImpactError::Scan`（「manifest_dir equals workspace_root」）として
///   拒否するため `fw impact` 全体が exit 1 で失敗していた。テンプレートの
///   `find_item`/`Item`（`templates/default/src/main.rs`）はいずれも
///   トップレベル非公開宣言（`pub` ではない）であるため定義元が見つからず
///   `ImpactError::SymbolNotFound` になるのが**新実装での正しい**挙動である
///   （`component_boundary::extract_from_source` はトップレベル `pub` 宣言のみ
///   走査対象とする契約、`cli/src/component_boundary.rs` 参照）。本テストは
///   終了コードそのものではなく、エラーメッセージが新実装の
///   `SymbolNotFound`（"no definition found for symbol"）であり、旧実装の
///   `Scan` エラー（"manifest_dir equals workspace_root"）ではないことを
///   断定することで、`root` 慣習未対応の解析不能状態からの回復を固定する。
#[test]
fn fw_new_output_fw_structure_succeeds_and_fw_impact_does_not_hit_root_scan_error() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) =
        run_fw_new(&["structure-impact-app", "--dir", &scratch.to_string_lossy()]);
    assert_eq!(
        new_code, 0,
        "fw new が失敗した: stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("structure-impact-app");

    let (structure_code, structure_stdout, structure_stderr) =
        run_fw("structure", &[], &project_dir);
    assert_eq!(
        structure_code, 0,
        "fw new 生成直後のプロジェクトで fw structure は exit 0 のはず \
         （`root` 慣習のディレクトリ実在誤検知の非回帰）: \
         stdout={structure_stdout} stderr={structure_stderr}"
    );

    let (impact_code, impact_stdout, impact_stderr) =
        run_fw("impact", &["find_item"], &project_dir);
    assert_eq!(
        impact_code, 1,
        "find_item は非公開宣言のため symbol not found（検証違反、終了コード 1）が \
         正しい新実装の挙動: stdout={impact_stdout} stderr={impact_stderr}"
    );
    assert!(
        impact_stderr.contains("no definition found for symbol"),
        "新実装では SymbolNotFound として fail-closed するはず（旧実装は \
         root member の Scan エラーで別メッセージになっていた）: \
         stderr={impact_stderr}"
    );
    assert!(
        !impact_stderr.contains("manifest_dir equals workspace_root"),
        "旧実装の root member 解決エラー（member_dir_name の Scan エラー）が \
         再発していないこと（イシュー #353 のリグレッション封じ）: \
         stderr={impact_stderr}"
    );
}

/// イシュー #378 受け入れ条件 2: `fw new --template app`（fandhe-frontend-core/fandhe-frontend-app
/// 依存の拡充テンプレート、vendor 同梱）が生成直後に `fw gate` PASS する
/// ことを固定する。`fw_new_output_passes_fw_gate`（`default` テンプレート）
/// と同一の断定方針（環境依存の `policy` のみ両分岐を確認、他 4 チェックは
/// 常に PASS）を踏襲する。vendored crate 群のコンパイルを伴うため
/// `default` より実行時間が長い（PR 本文に記載する既知事項）。
#[test]
fn fw_new_app_template_output_passes_fw_gate() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) = run_fw_new(&[
        "gate-pass-app-template",
        "--template",
        "app",
        "--dir",
        &scratch.to_string_lossy(),
    ]);
    assert_eq!(
        new_code, 0,
        "fw new --template app が失敗した: stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("gate-pass-app-template");
    let (gate_code, gate_stdout, gate_stderr) = run_fw_gate(&project_dir);

    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "test",
        "policy",
    ] {
        assert!(
            gate_stdout.contains(&format!("\"name\":\"{name}\"")),
            "fw gate のレポートにチェック `{name}` が現れない: stdout={gate_stdout}"
        );
    }

    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "test",
    ] {
        assert_eq!(
            check_passed(&gate_stdout, name),
            Some(true),
            "fw new --template app 生成直後のプロジェクトで `{name}` が失敗した \
             （app テンプレートと fw gate の前提がドリフトしている）: \
             stdout={gate_stdout} stderr={gate_stderr}"
        );
    }

    if cargo_deny_available() {
        assert_eq!(
            gate_code, 0,
            "cargo-deny 導入環境では fw new --template app 生成直後は PASS するはず: \
             stdout={gate_stdout} stderr={gate_stderr}"
        );
        assert!(
            gate_stdout.contains("\"gate_result\":\"PASS\""),
            "stdout={gate_stdout}"
        );
    } else {
        assert_eq!(
            gate_code, 1,
            "cargo-deny 未導入環境では policy の fail-closed により BLOCKED \
             (終了コード 1) のはず: stdout={gate_stdout}"
        );
        assert!(
            gate_stdout.contains("environment error: "),
            "policy の failed 出力は environment error であることを明示する \
             プレフィックスを含むはず: stdout={gate_stdout}"
        );
    }
}

/// イシュー #378: `fw new --template app` 生成物への未レビュー `raw_html()`
/// 注入が `default_escape_check` で検出されることを固定する
/// （`fw_new_output_default_escape_check_detects_injected_violation_in_root_src`
/// の app テンプレート版）。app は fandhe-frontend-core に依存するため `raw_html()` が
/// 実際に解決可能な呼び出しになる点が `default`（fandhe-frontend-core 非依存）との
/// 差分であり、clippy.toml の disallowed-methods が依存追加によって
/// 初めて実効化されることを固定する（実装計画 §7 セキュリティ考慮）。
#[test]
fn fw_new_app_template_default_escape_check_detects_injected_violation() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) = run_fw_new(&[
        "gate-pass-app-violation",
        "--template",
        "app",
        "--dir",
        &scratch.to_string_lossy(),
    ]);
    assert_eq!(
        new_code, 0,
        "fw new --template app が失敗した: stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("gate-pass-app-violation");
    let main_rs = project_dir.join("src").join("main.rs");
    let mut content = std::fs::read_to_string(&main_rs).expect("failed to read src/main.rs");
    content.push_str(
        "\nfn unreviewed_raw_html_probe() {\n    fandhe_frontend_core::raw_html(\"x\");\n}\n",
    );
    std::fs::write(&main_rs, content).expect("failed to write src/main.rs");

    let (gate_code, gate_stdout, gate_stderr) = run_fw_gate(&project_dir);

    assert_eq!(
        check_passed(&gate_stdout, "default_escape_check"),
        Some(false),
        "app テンプレート生成物の src/ 直下に注入した未レビュー raw_html() \
         呼び出しが default_escape_check で検出されなかった: \
         stdout={gate_stdout} stderr={gate_stderr}"
    );
    assert_ne!(
        gate_code, 0,
        "default_escape_check が failed の場合 fw gate 全体も非ゼロで \
         終了するはず: stdout={gate_stdout}"
    );
}

/// イシュー #410: `fw new --template embed`（静的単一ファイルテンプレート、
/// cargo パッケージを持たない）が生成する `structure.toml`（`[directories.root]`
/// `role = "asset"`、`crate` キーなし）は `fw gate`（`cli/src/gate.rs::
/// is_asset_only_project`）の静的専用モードの明示的オプトイン条件を満たす。
///
/// `default`/`app` の e2e（`fw_new_output_passes_fw_gate` /
/// `fw_new_app_template_output_passes_fw_gate`）は `policy`（cargo-deny 依存）
/// のみ実行環境（cargo-deny の導入有無）で結果が分岐するが、静的専用モードは
/// cargo を一切起動しないため cargo-deny の導入有無に依存せず、6 チェック
/// すべてが常に PASS・`gate_result: "PASS"`・終了コード 0 になることを
/// 無条件に断定する（計画 §4 ステップ 5 の断定方針差）。
#[test]
fn fw_new_embed_template_output_passes_fw_gate() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) = run_fw_new(&[
        "gate-pass-embed-template",
        "--template",
        "embed",
        "--dir",
        &scratch.to_string_lossy(),
    ]);
    assert_eq!(
        new_code, 0,
        "fw new --template embed が失敗した: stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("gate-pass-embed-template");
    let (gate_code, gate_stdout, gate_stderr) = run_fw_gate(&project_dir);

    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "test",
        "policy",
    ] {
        assert!(
            gate_stdout.contains(&format!("\"name\":\"{name}\"")),
            "fw gate のレポートにチェック `{name}` が現れない: stdout={gate_stdout}"
        );
        assert_eq!(
            check_passed(&gate_stdout, name),
            Some(true),
            "静的専用（asset-only）モードは cargo-deny の導入有無に関わらず \
             `{name}` が常に PASS するはず: stdout={gate_stdout} stderr={gate_stderr}"
        );
    }

    assert_eq!(
        gate_code, 0,
        "fw new --template embed 生成直後は環境に依存せず常に PASS するはず: \
         stdout={gate_stdout} stderr={gate_stderr}"
    );
    assert!(
        gate_stdout.contains("\"gate_result\":\"PASS\""),
        "stdout={gate_stdout}"
    );
}

/// イシュー #410: 静的専用（asset-only）モードは cargo 系 4 チェックを
/// not-applicable PASS 化するが、`default_escape_check`（保険層）は
/// バイパスしない。`embed` テンプレートは `src/` を生成しないため、
/// `[directories.root]` 予約名規約に従いプロジェクトルート直下へ
/// `src/injected.rs`（未レビュー `raw_html()` 呼び出し）を手動注入し、
/// `default_escape_check` が実際に走査を実行して violation を検出（failed）
/// することを断定する（`fw_new_output_default_escape_check_detects_injected_violation_in_root_src`
/// の embed 版。静的専用モードが検証の全面バイパスにならないことの回帰固定、
/// security.md A05）。
#[test]
fn fw_new_embed_template_gate_detects_injected_rust_violation() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) = run_fw_new(&[
        "gate-pass-embed-violation",
        "--template",
        "embed",
        "--dir",
        &scratch.to_string_lossy(),
    ]);
    assert_eq!(
        new_code, 0,
        "fw new --template embed が失敗した: stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("gate-pass-embed-violation");
    let injected_src_dir = project_dir.join("src");
    std::fs::create_dir_all(&injected_src_dir).expect("failed to create src/ for injection");
    std::fs::write(
        injected_src_dir.join("injected.rs"),
        "fn unreviewed_raw_html_probe() {\n    raw_html(\"x\");\n}\n",
    )
    .expect("failed to write src/injected.rs");

    let (gate_code, gate_stdout, gate_stderr) = run_fw_gate(&project_dir);

    assert_eq!(
        check_passed(&gate_stdout, "default_escape_check"),
        Some(false),
        "embed テンプレート（静的専用モード）の src/ に注入した未レビュー \
         raw_html() 呼び出しが default_escape_check で検出されなかった \
         （静的専用モードが保険層を全面バイパスしている疑い）: \
         stdout={gate_stdout} stderr={gate_stderr}"
    );
    assert!(
        gate_stdout.contains("injected.rs"),
        "default_escape_check の failed 出力は違反ファイルを file:line で \
         列挙するはず: stdout={gate_stdout}"
    );
    for name in ["type_check", "lint", "test", "policy"] {
        assert_eq!(
            check_passed(&gate_stdout, name),
            Some(true),
            "静的専用モードの not-applicable 4 チェックは Rust コード混入時も \
             PASS のままのはず（cargo が起動されないため）: stdout={gate_stdout}"
        );
    }
    assert_ne!(
        gate_code, 0,
        "default_escape_check が failed の場合 fw gate 全体も非ゼロで \
         終了するはず: stdout={gate_stdout}"
    );
    assert!(
        gate_stdout.contains("\"gate_result\":\"BLOCKED\""),
        "stdout={gate_stdout}"
    );
}

/// イシュー #500: `fw new --example ssr-routing` 生成直後のプロジェクトに対し
/// `fw gate` を実行し、`app`/`default` テンプレートと同じ 6 チェック断定方針
/// （`policy` のみ cargo-deny 導入有無で分岐、他 5 チェックは常に PASS）を
/// 適用する。さらに受け入れ条件 1（`fw new demo --example ssr-routing && cd
/// demo && cargo run -- /items/1` が動作する）を `cargo run` 実行で直接固定
/// する。
///
/// `--example` はパッケージ名を置換しない（`new_template.rs` モジュール doc
/// コメント参照）ため、生成プロジェクトのパッケージ名は正本と同じ
/// `fandhe-frontend-example-ssr-routing` のまま。
///
/// # 前提（`.claude/rules/ci.md` 準拠）
///
/// `examples/ssr-routing` は fandhe-frontend-core/-app/-server への crates.io
/// バージョン依存で完結する（vendor 同梱なし、イシュー #499）。本テストの
/// `cargo build`/`cargo run`/`fw gate` はいずれも crates.io
/// （`https://index.crates.io`・`https://static.crates.io`）への到達性を
/// 前提とする。到達不可の場合は環境エラーとして扱い、テストの弱体化で
/// 対処しない（`fw_new_app_template_output_passes_fw_gate` と同じ前提）。
#[test]
fn fw_new_example_ssr_routing_output_passes_fw_gate() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) = run_fw_new(&[
        "gate-pass-example-ssr-routing",
        "--example",
        "ssr-routing",
        "--dir",
        &scratch.to_string_lossy(),
    ]);
    assert_eq!(
        new_code, 0,
        "fw new --example ssr-routing が失敗した: stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("gate-pass-example-ssr-routing");
    let (gate_code, gate_stdout, gate_stderr) = run_fw_gate(&project_dir);

    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "test",
        "policy",
    ] {
        assert!(
            gate_stdout.contains(&format!("\"name\":\"{name}\"")),
            "fw gate のレポートにチェック `{name}` が現れない: stdout={gate_stdout}"
        );
    }

    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "test",
    ] {
        assert_eq!(
            check_passed(&gate_stdout, name),
            Some(true),
            "fw new --example ssr-routing 生成直後のプロジェクトで `{name}` が \
             失敗した（ssr-routing サンプルと fw gate の前提がドリフトしている）: \
             stdout={gate_stdout} stderr={gate_stderr}"
        );
    }

    if cargo_deny_available() {
        assert_eq!(
            gate_code, 0,
            "cargo-deny 導入環境では fw new --example ssr-routing 生成直後は \
             PASS するはず: stdout={gate_stdout} stderr={gate_stderr}"
        );
        assert!(
            gate_stdout.contains("\"gate_result\":\"PASS\""),
            "stdout={gate_stdout}"
        );
    } else {
        assert_eq!(
            gate_code, 1,
            "cargo-deny 未導入環境では policy の fail-closed により BLOCKED \
             (終了コード 1) のはず: stdout={gate_stdout}"
        );
        assert!(
            gate_stdout.contains("environment error: "),
            "policy の failed 出力は environment error であることを明示する \
             プレフィックスを含むはず: stdout={gate_stdout}"
        );
    }

    // 受け入れ条件 1（実装計画 §2.2 の起点）: `fw new demo --example
    // ssr-routing && cd demo && cargo run -- /items/1` が動作すること。
    // `examples/ssr-routing/tests/routing.rs::run_cli` と同一の呼び出し方
    // （引数 1 個の CLI、標準出力にステータス行 + body）を生成プロジェクト
    // 直下で `cargo run` 経由で再現する。
    let run_output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--")
        .arg("/items/1")
        .current_dir(&project_dir)
        .output()
        .expect("failed to spawn `cargo run` in generated example project");
    assert!(
        run_output.status.success(),
        "cargo run -- /items/1 が生成直後の ssr-routing サンプルで失敗した: \
         stdout={} stderr={}",
        String::from_utf8_lossy(&run_output.stdout),
        String::from_utf8_lossy(&run_output.stderr)
    );
    let run_stdout = String::from_utf8_lossy(&run_output.stdout);
    assert!(run_stdout.starts_with("200\n"), "stdout was: {run_stdout}");
    assert!(
        run_stdout.contains("Content-Type: text/html"),
        "stdout was: {run_stdout}"
    );
}

/// `fw new --example dist-server-docker` で生成した直後のプロジェクトが
/// `fw gate` を PASS すること（イシュー #502、`fw_new_example_ssr_routing_output_passes_fw_gate`
/// と同型のモデル）。
///
/// `cargo build`/`fw gate` はいずれも crates.io
/// （`https://index.crates.io`・`https://static.crates.io`）への到達性を
/// 前提とする。到達不可の場合は環境エラーとして扱い、テストの弱体化で
/// 対処しない（`fw_new_app_template_output_passes_fw_gate` と同じ前提）。
///
/// `dist-server-docker` は常駐型サーバー（`accept()` ループが戻らない設計、
/// `src/main.rs` 参照）のため、`fw_new_example_ssr_routing_output_passes_fw_gate`
/// 末尾のような `cargo run` 追撃は行わない。HTTP 応答検証（GET / ・
/// GET /static/style.css ・404）は生成プロジェクト内 `tests/boot.rs`
/// （実プロセス起動 + 素の TCP）が担い、下記の `test` チェック PASS 断定を
/// もって検証済みとする。
#[test]
fn fw_new_example_dist_server_docker_output_passes_fw_gate() {
    let scratch = unique_scratch_dir();
    let _scratch_guard = ScratchProject(scratch.clone());

    let (new_code, new_stdout, new_stderr) = run_fw_new(&[
        "gate-pass-example-dist-server-docker",
        "--example",
        "dist-server-docker",
        "--dir",
        &scratch.to_string_lossy(),
    ]);
    assert_eq!(
        new_code, 0,
        "fw new --example dist-server-docker が失敗した: stdout={new_stdout} stderr={new_stderr}"
    );

    let project_dir = scratch.join("gate-pass-example-dist-server-docker");
    let (gate_code, gate_stdout, gate_stderr) = run_fw_gate(&project_dir);

    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "test",
        "policy",
    ] {
        assert!(
            gate_stdout.contains(&format!("\"name\":\"{name}\"")),
            "fw gate のレポートにチェック `{name}` が現れない: stdout={gate_stdout}"
        );
    }

    for name in [
        "type_check",
        "default_escape_check",
        "url_validation_check",
        "lint",
        "test",
    ] {
        assert_eq!(
            check_passed(&gate_stdout, name),
            Some(true),
            "fw new --example dist-server-docker 生成直後のプロジェクトで `{name}` が \
             失敗した（dist-server-docker サンプルと fw gate の前提がドリフトしている）: \
             stdout={gate_stdout} stderr={gate_stderr}"
        );
    }

    if cargo_deny_available() {
        assert_eq!(
            gate_code, 0,
            "cargo-deny 導入環境では fw new --example dist-server-docker 生成直後は \
             PASS するはず: stdout={gate_stdout} stderr={gate_stderr}"
        );
        assert!(
            gate_stdout.contains("\"gate_result\":\"PASS\""),
            "stdout={gate_stdout}"
        );
    } else {
        assert_eq!(
            gate_code, 1,
            "cargo-deny 未導入環境では policy の fail-closed により BLOCKED \
             (終了コード 1) のはず: stdout={gate_stdout}"
        );
        assert!(
            gate_stdout.contains("environment error: "),
            "policy の failed 出力は environment error であることを明示する \
             プレフィックスを含むはず: stdout={gate_stdout}"
        );
    }
}
