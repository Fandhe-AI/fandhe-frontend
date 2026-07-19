//! `fw gate`（`cli/src/gate.rs`, TASK-13.3・#138）に対する負例回帰テスト
//! （TASK-13.5・#148、サブイシュー #149 ケース設計 / #150 実装）。
//!
//! # 契約
//!
//! PoC-7（`docs/spec/03-poc/ai-self-maintenance/negative-tests/*/gate.json`）が
//! 実測した 3 つの負例ケース（型エラー・未エスケープ出力・禁止依存追加）を、
//! 製品 CLI（実バイナリとしての `fw gate`、実ツールチェーン起動込み）に対して
//! 再現する。REQ-13「検証・制約の強制」・REQ-1「既定エスケープ」・REQ-4
//! 「検証フックの多層防御」が弱体化した場合にここで検知することを目的とする。
//! ケース 4（イシュー #315）はテストターゲット内の未レビュー `raw_html()`
//! 呼び出しを対象とし、`lint` チェックの `--all-targets` 拡張と CI `clippy`
//! ジョブ（イシュー #299）の検出境界一致を固定する。
//!
//! `cli/tests/gate_integration.rs` は fail-closed 経路（引数不正・`structure.toml`
//! 欠落等）のみを対象とし、実ツールチェーンを走らせて BLOCKED まで到達させる
//! フル e2e は本ファイルのスコープとする（重複しない）。
//!
//! 本ファイルの削除・弱体化（アサーション削除・`#[ignore]` 付与等）は
//! REQ-13 の受け入れ基準（検証未通過変更が確実に BLOCKED になること）の
//! 担保を失わせるため行わない（coding-rust.md「テストの `#[ignore]` 追加で
//! ごまかさない」）。
//!
//! # 環境差の吸収（cargo-deny の有無）
//!
//! CI（`.github/workflows/ci.yml`）は TASK-13.3c（#141）で `test` ジョブに
//! cargo-deny を導入済みだが、cargo-deny が未導入のローカル環境等でも本
//! ファイルは実行できるよう [`support::cargo_deny_available`] で実行環境を
//! 判定し、どちらの環境でも「弱体化なしで取れる最強のアサーション」を常時
//! 実行する（環境に応じたスキップ・`#[ignore]` は行わない）。
//!
//! # ヘルメチック性
//!
//! 全ケースでネットワークアクセスを行わない。フィクスチャは外部依存ゼロの
//! クレートのみで構成し、禁止依存ケースもローカルのダミー path クレートで
//! 名前一致検出を再現する（実際の openssl-sys は取得しない）。
//!
//! フィクスチャ書き出し・`fw` 起動・JSON レポート判定の共通ヘルパーは
//! `cli/tests/xss_regression_link.rs`（TASK-13.3c・#141）と共用するため
//! `tests/support/mod.rs` に集約している（詳細は同ファイル冒頭コメント参照）。

mod support;

use std::path::Path;
use support::{
    baseline_main_rs, cargo_deny_available, check_passed, replace_unique, run_fw_gate,
    write_case_project,
};

/// ケース 3（禁止依存追加）向けに、ダミーの `openssl-sys` path クレートを
/// フィクスチャ内に追加し、workspace members・`app` の依存へ組み込んだうえで
/// `Cargo.lock` を再生成する。
///
/// 実際の openssl-sys クレート（crates.io 上の同名クレート）は一切取得しない。
/// `[bans].deny` の判定はクレート名の一致で行われるため、空の `lib.rs` を
/// 持つローカル path クレートでも `cargo deny check bans` の検出を再現できる
/// （完全オフライン、security.md A06）。
fn inject_banned_dependency(project: &Path) {
    use std::fs;
    use std::process::Command;

    let dummy_dir = project.join("openssl-sys");
    fs::create_dir_all(dummy_dir.join("src")).expect("ダミー openssl-sys クレートの作成に失敗した");
    fs::write(
        dummy_dir.join("Cargo.toml"),
        "[package]\nname = \"openssl-sys\"\nversion = \"0.9.99\"\nedition = \"2021\"\nlicense = \"MIT\"\npublish = false\n",
    )
    .expect("ダミー openssl-sys の Cargo.toml 書き込みに失敗した");
    fs::write(dummy_dir.join("src").join("lib.rs"), "")
        .expect("ダミー openssl-sys の lib.rs 書き込みに失敗した");

    let workspace_toml = project.join("Cargo.toml");
    let original =
        fs::read_to_string(&workspace_toml).expect("workspace Cargo.toml の読み込みに失敗した");
    let injected = replace_unique(
        &original,
        "members = [\"app\"]",
        "members = [\"app\", \"openssl-sys\"]",
    );
    fs::write(&workspace_toml, injected).expect("workspace Cargo.toml の書き込みに失敗した");

    let app_toml = project.join("app").join("Cargo.toml");
    let original = fs::read_to_string(&app_toml).expect("app/Cargo.toml の読み込みに失敗した");
    let injected =
        format!("{original}\n[dependencies]\nopenssl-sys = {{ path = \"../openssl-sys\" }}\n");
    fs::write(&app_toml, injected).expect("app/Cargo.toml の書き込みに失敗した");

    let lockfile_output = Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(project)
        .output()
        .expect("cargo generate-lockfile の再実行に失敗した");
    assert!(
        lockfile_output.status.success(),
        "禁止依存注入後の cargo generate-lockfile --offline に失敗した: {}",
        String::from_utf8_lossy(&lockfile_output.stderr)
    );
}

/// ケース 4（イシュー #315: `--all-targets` 検出境界差回帰）向けに、ダミーの
/// `rws-core` path クレート（`pub fn raw_html`）を追加し、`app` の
/// `[dev-dependencies]` へ登録したうえで `app/tests/raw_html_leak.rs` から
/// レビューマーカーなしで呼び出し、`Cargo.lock` を再生成する。
///
/// 実際の `rws-core`（本ワークスペースの描画コア）は一切参照しない。ローカル
/// path クレートに同名 `rws_core::raw_html` を再現するだけで、workspace
/// ルート `clippy.toml`（[`clippy_toml_content`] 相当、`disallowed-methods`
/// の `rws_core::raw_html` エントリ）が name-based に検出できる
/// （完全オフライン、security.md A06 と同じヘルメチック方針）。
fn inject_raw_html_call_in_test_target(project: &Path) {
    use std::fs;
    use std::process::Command;

    let dummy_dir = project.join("rws-core");
    fs::create_dir_all(dummy_dir.join("src")).expect("ダミー rws-core クレートの作成に失敗した");
    fs::write(
        dummy_dir.join("Cargo.toml"),
        "[package]\nname = \"rws-core\"\nversion = \"0.1.0\"\nedition = \"2021\"\nlicense = \"MIT\"\npublish = false\n",
    )
    .expect("ダミー rws-core の Cargo.toml 書き込みに失敗した");
    fs::write(
        dummy_dir.join("src").join("lib.rs"),
        "pub fn raw_html(s: String) -> String {\n    s\n}\n",
    )
    .expect("ダミー rws-core の lib.rs 書き込みに失敗した");

    let workspace_toml = project.join("Cargo.toml");
    let original =
        fs::read_to_string(&workspace_toml).expect("workspace Cargo.toml の読み込みに失敗した");
    let injected = replace_unique(
        &original,
        "members = [\"app\"]",
        "members = [\"app\", \"rws-core\"]",
    );
    fs::write(&workspace_toml, injected).expect("workspace Cargo.toml の書き込みに失敗した");

    let app_toml = project.join("app").join("Cargo.toml");
    let original = fs::read_to_string(&app_toml).expect("app/Cargo.toml の読み込みに失敗した");
    // `[dev-dependencies]` に限定することで、本ケースが「テストターゲット
    // 経由でのみ到達可能な raw_html 呼び出し」であることを固定する
    // （通常ビルド・`cargo check` では rws-core 自体がコンパイル対象にならない）。
    let injected =
        format!("{original}\n[dev-dependencies]\nrws-core = {{ path = \"../rws-core\" }}\n");
    fs::write(&app_toml, injected).expect("app/Cargo.toml の書き込みに失敗した");

    let app_tests = project.join("app").join("tests");
    fs::create_dir_all(&app_tests).expect("app/tests/ ディレクトリの作成に失敗した");
    fs::write(
        app_tests.join("raw_html_leak.rs"),
        "//! イシュー #315 負例回帰フィクスチャ: テストターゲット内の未レビュー\n\
         //! `raw_html()` 呼び出し。`--all-targets` なしの `cargo clippy` では\n\
         //! 到達しないターゲットであることが本ケースの前提。\n\n\
         #[test]\n\
         fn calls_raw_html_without_review_marker() {\n    \
         let _ = rws_core::raw_html(\"<script>alert(1)</script>\".to_string());\n\
         }\n",
    )
    .expect("app/tests/raw_html_leak.rs の書き込みに失敗した");

    let lockfile_output = Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(project)
        .output()
        .expect("cargo generate-lockfile の再実行に失敗した");
    assert!(
        lockfile_output.status.success(),
        "raw_html 注入後の cargo generate-lockfile --offline に失敗した: {}",
        String::from_utf8_lossy(&lockfile_output.stderr)
    );
}

/// ケース 0（ベースライン対照）。
///
/// 無改変のフィクスチャが `type_check` / `default_escape_check` / `lint` /
/// `test` の 4 チェックすべてを通過することを確認する対照群。後続の負例
/// ケースが「注入した欠陥」に起因して BLOCKED になっていることを保証する
/// 基盤であり、このテストが落ちる場合は負例側の失敗を環境要因と区別できない。
///
/// `policy` チェックのみ、cargo-deny の導入有無で環境ごとに挙動が変わる
/// （ファイル冒頭ドキュメント参照）。導入済み環境では PASS + 終了コード 0 まで
/// 検証し、未導入環境では「policy だけが failed で BLOCKED」という
/// fail-closed 契約自体を検証する。
#[test]
fn baseline_fixture_passes_core_checks() {
    let project = write_case_project("baseline", baseline_main_rs());
    let (code, stdout, stderr) = run_fw_gate(&project);

    assert_eq!(
        check_passed(&stdout, "type_check"),
        Some(true),
        "ベースラインで type_check が失敗した（対照群が壊れている）: stdout={stdout} stderr={stderr}"
    );
    assert_eq!(
        check_passed(&stdout, "default_escape_check"),
        Some(true),
        "ベースラインで default_escape_check が失敗した: stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "lint"),
        Some(true),
        "ベースラインで lint が失敗した: stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "test"),
        Some(true),
        "ベースラインで test が失敗した: stdout={stdout}"
    );

    if cargo_deny_available() {
        assert_eq!(
            code, 0,
            "cargo-deny 導入環境ではベースラインは PASS するはず: stdout={stdout}"
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
        // cargo-deny 未導入環境では policy のみ fail-closed で failed になり、
        // 他の 4 チェックは通過したまま全体として BLOCKED になる、という
        // fail-closed 契約を確認する。
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

/// ケース 1（PoC-7 `negative-type-error` 再現）。
///
/// `find_item` の比較式を `it.id == target_id`（`String` と `&str`）から
/// `it.id == 42`（`String` と整数リテラル）へ改変し、`type_check` が
/// `E0277` で失敗し、`fw gate` 全体が BLOCKED になることを確認する。
/// `default_escape_check` は無関係のまま通過することも確認し、ブロック理由の
/// 特定性（型エラー由来であってエスケープ検査由来ではないこと）を保証する。
#[test]
fn type_error_blocks_gate_with_type_check_failure() {
    let injected = replace_unique(baseline_main_rs(), "it.id == target_id", "it.id == 42");
    let project = write_case_project("type-error", &injected);
    let (code, stdout, stderr) = run_fw_gate(&project);

    assert_eq!(
        code, 1,
        "型不正な比較が fw gate を通過してしまった（BLOCKED になるはず）: stdout={stdout} stderr={stderr}"
    );
    assert!(
        stdout.contains("\"gate_result\":\"BLOCKED\""),
        "stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "type_check"),
        Some(false),
        "type_check が failed であるはず: stdout={stdout}"
    );
    assert!(
        stdout.contains("E0277"),
        "type_check の output に rustc エラーコード E0277 が含まれるはず: stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "default_escape_check"),
        Some(true),
        "型エラーとは無関係な default_escape_check は通過するはず（ブロック理由の特定性）: stdout={stdout}"
    );
}

/// ケース 2（PoC-7 `negative-unescaped-output` 再現）。
///
/// ローカル関数 `raw_html` を定義し、`ESCAPE-REVIEWED:` マーカーなしで
/// `main` の表示経路から呼び出す（コンパイル可・clippy クリーンを維持）。
/// `default_escape_check` が failed になり、`fw gate` 全体が BLOCKED に
/// なることを確認する。`type_check` は無関係のまま通過することも確認し、
/// ブロック理由がコンパイル失敗由来ではないことを保証する。
///
/// `test`（`cargo test`）も無関係のまま通過することを確認する。PoC-7 の
/// 重要発見（`docs/spec/03-poc/ai-self-maintenance/README.md` 実施内容 5）
/// 「未エスケープ出力は `cargo test` 単体では検出できず、`test` は通過した
/// まま `default_escape_check` という独立したゲートのみが不合格になる」を
/// 製品 CLI 上で再現し、テストと相補的な専用ゲートの価値を担保する。
#[test]
fn unescaped_raw_html_call_blocks_gate_with_escape_check_failure() {
    let injected = baseline_main_rs().replacen(
        "fn main() {",
        "fn raw_html(s: String) -> String {\n    s\n}\n\nfn main() {",
        1,
    );
    let injected = replace_unique(
        &injected,
        "println!(\"found: {}\", item.name);",
        "println!(\"found: {}\", raw_html(item.name.clone()));",
    );

    let project = write_case_project("unescaped-output", &injected);
    let (code, stdout, stderr) = run_fw_gate(&project);

    assert_eq!(
        code, 1,
        "未レビューの raw_html() 呼び出しが fw gate を通過してしまった: stdout={stdout} stderr={stderr}"
    );
    assert!(
        stdout.contains("\"gate_result\":\"BLOCKED\""),
        "stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "default_escape_check"),
        Some(false),
        "default_escape_check が failed であるはず: stdout={stdout}"
    );
    assert!(
        stdout.contains("unreviewed raw_html() call"),
        "default_escape_check の output に違反の具体的な記述が含まれるはず: stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "type_check"),
        Some(true),
        "エスケープ違反とは無関係な type_check は通過するはず（コンパイル自体は成立、ブロック理由の特定性）: stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "test"),
        Some(true),
        "PoC-7 の重要発見（cargo test 単体では未エスケープ出力を検出できない）どおり、\
         test は通過したまま default_escape_check のみが不合格であるはず: stdout={stdout}"
    );
}

/// ケース 3（PoC-7 `negative-banned-dependency` 再現）。
///
/// ダミーの `openssl-sys` path クレートを追加し、`deny.toml` の
/// `[bans].deny` に名前一致することで `policy` チェックが failed になり、
/// `fw gate` 全体が BLOCKED になることを確認する。`type_check` は
/// 無関係のまま通過することも確認する。
///
/// `cargo-deny` 未導入環境では `policy` は `deny.toml` 実在確認より先に
/// cargo-deny の起動自体に失敗して failed になる（fail-closed）。この場合も
/// 「BLOCKED + policy failed」という必須アサーションは変わらず成立するため、
/// 環境非依存に実行できる。cargo-deny 導入環境（CI の `test` ジョブ、
/// TASK-13.3c・#141）でのみ、output に `banned`/`openssl-sys` という
/// ブロック理由の具体性を追加検証する。
#[test]
fn banned_dependency_blocks_gate_with_policy_failure() {
    let project = write_case_project("banned-dependency", baseline_main_rs());
    inject_banned_dependency(&project);

    let (code, stdout, stderr) = run_fw_gate(&project);

    assert_eq!(
        code, 1,
        "禁止依存（openssl-sys）の追加が fw gate を通過してしまった: stdout={stdout} stderr={stderr}"
    );
    assert!(
        stdout.contains("\"gate_result\":\"BLOCKED\""),
        "stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "policy"),
        Some(false),
        "policy が failed であるはず: stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "type_check"),
        Some(true),
        "禁止依存とは無関係な type_check は通過するはず（ブロック理由の特定性）: stdout={stdout}"
    );

    if cargo_deny_available() {
        assert!(
            stdout.contains("banned") && stdout.contains("openssl-sys"),
            "cargo-deny 導入環境では policy の output に禁止クレート名を含む具体的な \
             ブロック理由が含まれるはず（PoC-7 と同じブロック理由の確認）: stdout={stdout}"
        );
    }
}

/// ケース 4（イシュー #315: `fw gate` の `lint` チェックへの `--all-targets`
/// 拡張の回帰防止）。
///
/// テストターゲット（`app/tests/raw_html_leak.rs`）内のみに、レビュー
/// マーカーなしの `raw_html()` 呼び出しを配置する。`default_escape_check`
/// （保険層）は `src/` 配下のみを走査するため本ケースを検出できず、
/// `lint` チェック（`cargo clippy --all-targets`、主防御）のみが検出できる
/// ことをアサーションで固定する。将来 `--all-targets` が誤って外された場合、
/// 本テストが「`lint` が passed のまま BLOCKED にならない」偽陰性として
/// 退行を検知する（coding-rust.md「XSS 回帰テストは削除・弱体化しない」）。
#[test]
fn unreviewed_raw_html_in_test_target_is_blocked_by_lint() {
    let project = write_case_project("raw-html-in-test-target", baseline_main_rs());
    inject_raw_html_call_in_test_target(&project);

    let (code, stdout, stderr) = run_fw_gate(&project);

    assert_eq!(
        code, 1,
        "テストターゲット内の未レビュー raw_html() 呼び出しが fw gate を \
         通過してしまった（--all-targets の検出境界差が再発している）: \
         stdout={stdout} stderr={stderr}"
    );
    assert!(
        stdout.contains("\"gate_result\":\"BLOCKED\""),
        "stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "lint"),
        Some(false),
        "lint（cargo clippy --all-targets）が failed であるはず。イシュー #315 の \
         `--all-targets` 拡張が退行するとテストターゲット内の raw_html() が \
         検出されず、この行が Some(true) に変わる: stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "default_escape_check"),
        Some(true),
        "default_escape_check（保険層）は `tests/` を走査対象外とするため \
         通過するはず（本ケースの検出が `lint` の `--all-targets` 経由で \
         あることの核心アサーション）: stdout={stdout}"
    );
    assert_eq!(
        check_passed(&stdout, "type_check"),
        Some(true),
        "type_check（cargo check、テストターゲットを含まない）は無関係な \
         まま通過するはず（ブロック理由の特定性）: stdout={stdout}"
    );
}

/// イシュー #372 e2e 回帰（自己参照様ソース）。コメント言及・フィクスチャ風
/// 文字列リテラル・「別識別子のサフィックス」のみで構成されたソース
/// （本リポジトリ自身の `cli/src/gate.rs` に実在する自己参照パターンの縮図）を
/// `default_escape_check` に通しても passed のままであることを固定する。
/// [`code_context_mask`]（`cli/src/gate.rs`）の精密化が退行すると、いずれかの
/// 行が誤って違反として検出され `default_escape_check` が failed に変わる。
#[test]
fn self_referential_source_does_not_block_default_escape_check() {
    let injected = baseline_main_rs().replacen(
        "fn main() {",
        "// raw_html(x) is the opt-in escape hatch documented here\n\
         /// see raw_html() for details\n\
         fn detects_unreviewed_raw_html() {}\n\n\
         fn main() {\n    \
         let _fixture = \"unreviewed raw_html(x) call\";\n",
        1,
    );

    let project = write_case_project("self-referential-source", &injected);
    let (code, stdout, stderr) = run_fw_gate(&project);

    assert_eq!(
        check_passed(&stdout, "default_escape_check"),
        Some(true),
        "コメント・文字列リテラル・識別子サフィックスのみの自己参照様ソースが \
         誤って違反として検出されている（イシュー #372 の走査精密化が退行）: \
         code={code} stdout={stdout} stderr={stderr}"
    );
}

/// 上記の自己参照様ソースへ実際の未レビュー `raw_html()` 呼び出しを追加すると、
/// 引き続き `default_escape_check` が failed（BLOCKED）になることを固定する
/// （非弱体化の確認。誤検知解消が偽陰性を生んでいないことの対）。
#[test]
fn self_referential_source_still_blocks_on_actual_raw_html_call() {
    let injected = baseline_main_rs().replacen(
        "fn main() {",
        "// raw_html(x) is the opt-in escape hatch documented here\n\
         /// see raw_html() for details\n\
         fn detects_unreviewed_raw_html() {}\n\n\
         fn raw_html(s: String) -> String {\n    s\n}\n\n\
         fn main() {\n    \
         let _fixture = \"unreviewed raw_html(x) call\";\n    \
         let _ = raw_html(\"actual call\".to_string());\n",
        1,
    );

    let project = write_case_project("self-referential-source-with-call", &injected);
    let (code, stdout, stderr) = run_fw_gate(&project);

    assert_eq!(
        code, 1,
        "自己参照様ソースに混在する実際の raw_html() 呼び出しが検出されず \
         fw gate を通過してしまった: stdout={stdout} stderr={stderr}"
    );
    assert_eq!(
        check_passed(&stdout, "default_escape_check"),
        Some(false),
        "実際の raw_html() 呼び出しは default_escape_check で検出され続ける \
         はず（誤検知解消が偽陰性を生んでいないことの確認）: stdout={stdout}"
    );
}
