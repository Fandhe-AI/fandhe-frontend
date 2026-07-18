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
//! 本リポジトリ自身の CI（`.github/workflows/ci.yml`）は cargo-deny を
//! インストールしないため、`policy` チェックは CI 上では「cargo-deny 起動失敗
//! → failed（fail-closed）」となる。本ファイルは [`cargo_deny_available`] で
//! 実行環境を判定し、どちらの環境でも「弱体化なしで取れる最強のアサーション」
//! を常時実行する（環境に応じたスキップ・`#[ignore]` は行わない）。
//!
//! # ヘルメチック性
//!
//! 全ケースでネットワークアクセスを行わない。フィクスチャは外部依存ゼロの
//! クレートのみで構成し、禁止依存ケースもローカルのダミー path クレートで
//! 名前一致検出を再現する（実際の openssl-sys は取得しない）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// `fw` バイナリを `gate --project <dir>` で起動し、(終了コード, stdout, stderr)
/// を返す（`gate_integration.rs` の `run_fw_gate` と同一パターン）。
///
/// `CARGO_TARGET_DIR` はフィクスチャ間で共有しない（`raw_html_lint_e2e.rs`
/// と同一方針）。self-hosted runner では `CARGO_TARGET_DIR=/cargo-target` が
/// プロセス環境に既定で設定されており、本テストの全フィクスチャは同名パッケージ
/// `negative-fixture-app` のため、これを継承したまま `cargo` を起動すると
/// フィクスチャ間でビルドキャッシュ/フィンガープリントが衝突し、直前に生成した
/// 別フィクスチャの `type_check` 結果を誤って再利用してしまう（型エラーを
/// 注入したはずのケースが再コンパイルされず誤って PASS する偽陰性）。
/// ここで `project_dir` 配下の専用 `target/` を明示指定し、継承された値を
/// 上書きすることで各フィクスチャを独立させる（`fw` から起動される `cargo`
/// 子プロセスにも env は継承されるため、これで `gate.rs` 側の変更は不要）。
fn run_fw_gate(project_dir: &Path) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_fw"))
        .arg("gate")
        .arg("--project")
        .arg(project_dir)
        .env("CARGO_TARGET_DIR", project_dir.join("target"))
        .output()
        .expect("failed to spawn `fw` binary");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// `stdout`（`fw gate` の JSON レポート）中の `"name":"<name>"` エントリの
/// `passed` 値を判定する。該当エントリが見つからない場合は `None`
/// （「チェック自体が JSON に現れていない」ことと「passed:false」を区別する
/// ため、`bool` ではなく `Option<bool>` を返す）。
fn check_passed(stdout: &str, name: &str) -> Option<bool> {
    if stdout.contains(&format!("\"name\":\"{name}\",\"passed\":true")) {
        Some(true)
    } else if stdout.contains(&format!("\"name\":\"{name}\",\"passed\":false")) {
        Some(false)
    } else {
        None
    }
}

/// 実行環境に `cargo-deny` サブコマンドが導入済みかを判定する
/// （リポジトリ自身の CI には未導入、ローカル開発環境には導入済みという
/// 差を吸収するための補助関数、ファイル冒頭ドキュメント参照）。
fn cargo_deny_available() -> bool {
    Command::new("cargo")
        .args(["deny", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// `write_case_project` が書き出した一時プロジェクトディレクトリを保持し、
/// スコープを抜けるタイミングで自身を削除するガード
/// （`templates/default/tests/negative_type_error.rs` の `ScratchProject` と
/// 同一方針）。
struct ScratchProject(PathBuf);

impl std::ops::Deref for ScratchProject {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl Drop for ScratchProject {
    fn drop(&mut self) {
        // 削除失敗（他プロセスによるロック等）はテスト結果の正当性に
        // 影響しないため、ベストエフォートとして無視する。
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// 一時プロジェクトを書き出すスクラッチルート。`CARGO_TARGET_TMPDIR`
/// （cargo がテストバイナリ実行時に設定する target 配下の一時ディレクトリ）が
/// あればそこに閉じ、未設定環境向けに OS 標準の一時領域へフォールバックする
/// （`negative_type_error.rs` と同一パターン、パストラバーサル対策の一環）。
fn scratch_root() -> PathBuf {
    std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
}

/// ベースライン（正例）となる `app/src/main.rs` の内容。PoC-7
/// `target-project`（`Item` / `find_item`）相当の最小構成で、依存ゼロ・
/// clippy クリーン・`raw_html` 文字列を一切含まない。
///
/// 各負例ケースはこの文字列に対して注入対象の部分文字列を一意に
/// 置換することで欠陥を混入させる（`negative_type_error.rs` の
/// 「注入対象を実行可能コードに限定する」方針を踏襲）。
fn baseline_main_rs() -> &'static str {
    r#"struct Item {
    id: String,
    name: String,
}

fn find_item<'a>(items: &'a [Item], target_id: &str) -> Option<&'a Item> {
    items.iter().find(|it| it.id == target_id)
}

fn main() {
    let items = vec![
        Item {
            id: "1".to_string(),
            name: "widget".to_string(),
        },
        Item {
            id: "2".to_string(),
            name: "gadget".to_string(),
        },
    ];
    if let Some(item) = find_item(&items, "1") {
        println!("found: {}", item.name);
    }
}
"#
}

/// 一意な一時プロジェクトディレクトリに以下を書き出す:
///
/// ```text
/// <fixture>/
/// ├── structure.toml   ([directories.app], role = "component")
/// ├── Cargo.toml       (virtual workspace, members = ["app"])
/// ├── deny.toml        (templates/default/deny.toml と同ポリシーの最小版)
/// └── app/
///     ├── Cargo.toml   (name = "negative-fixture-app", 依存ゼロ)
///     └── src/main.rs  (main_rs_content)
/// ```
///
/// `cargo generate-lockfile --offline` で `Cargo.lock` を生成する（依存ゼロの
/// ため決定的・ネットワーク不要）。`fw gate` は `--locked` で `cargo`
/// サブコマンドを起動するため、ロックファイルなしでは各チェックがロック
/// ファイル欠落自体で failed になり、注入した欠陥とは無関係な失敗理由に
/// なってしまう（ケースの特定性を損なう）ため、ここで確実に用意する。
fn write_case_project(case_name: &str, main_rs_content: &str) -> ScratchProject {
    let dest = scratch_root().join(format!(
        "negative-cases-{case_name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dest);
    let app_src = dest.join("app").join("src");
    fs::create_dir_all(&app_src).expect("一時プロジェクトディレクトリの作成に失敗した");

    fs::write(
        dest.join("structure.toml"),
        r#"
[manifest]
version = 1

[directories.app]
role = "component"
crate = "negative-fixture-app"
description = "TASK-13.5 negative case fixture"
"#,
    )
    .expect("structure.toml の書き込みに失敗した");

    fs::write(
        dest.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\"]\nresolver = \"2\"\n",
    )
    .expect("workspace Cargo.toml の書き込みに失敗した");

    // `templates/default/deny.toml` と同じ主要ポリシー（bans/licenses/sources）
    // を持つ最小版。`policy` チェックが `deny.toml` 実在確認の先で実際に
    // `cargo deny check bans licenses sources` を走らせられるようにする。
    fs::write(
        dest.join("deny.toml"),
        r#"[graph]
targets = []

[bans]
multiple-versions = "warn"
deny = [
    { name = "openssl-sys" },
]

[licenses]
allow = ["MIT", "Apache-2.0", "Unicode-3.0", "BSD-3-Clause"]

[sources]
unknown-registry = "deny"
unknown-git = "deny"
"#,
    )
    .expect("deny.toml の書き込みに失敗した");

    fs::write(
        dest.join("app").join("Cargo.toml"),
        "[package]\nname = \"negative-fixture-app\"\nversion = \"0.1.0\"\nedition = \"2021\"\nlicense = \"MIT\"\npublish = false\n",
    )
    .expect("app/Cargo.toml の書き込みに失敗した");

    // イシュー #157/#263（`gate.rs::clippy_policy_check`）: `lint` チェックは
    // `project_dir` 直下の `clippy.toml` に `disallowed-methods` の
    // `rws_core::raw_html` エントリが存在することを fail-closed で前提とする
    // （欠落時は cargo clippy を起動する前に `lint` を failed とする）。本フィクス
    // チャはワークスペースルートの `clippy.toml` と同一ポリシーを配布する
    // `templates/default/clippy.toml` と同内容を複製し、`baseline_fixture_passes_core_checks`
    // 等の `lint` チェックを実体化させる。
    fs::write(
        dest.join("clippy.toml"),
        r#"disallowed-methods = [
    { path = "rws_core::raw_html", reason = "REQ-1 の唯一のエスケープ迂回経路。レビュー済みの呼び出しには `#[expect(clippy::disallowed_methods, reason = \"ESCAPE-REVIEWED: <根拠>\")]` を呼び出し文へ直接付与すること（`#[allow(...)]` によるブランケット抑止は禁止、docs/raw-html-review-gate.md 参照）" },
]
"#,
    )
    .expect("clippy.toml の書き込みに失敗した");

    fs::write(app_src.join("main.rs"), main_rs_content).expect("main.rs の書き込みに失敗した");

    // 依存ゼロのためネットワークアクセスなしで決定的にロックファイルを生成できる。
    let lockfile_output = Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(&dest)
        .output()
        .expect("cargo generate-lockfile の起動に失敗した");
    assert!(
        lockfile_output.status.success(),
        "cargo generate-lockfile --offline に失敗した（フィクスチャ自体が壊れている）: {}",
        String::from_utf8_lossy(&lockfile_output.stderr)
    );

    ScratchProject(dest)
}

/// 一意な部分文字列 `from` を `to` へちょうど 1 箇所だけ置換する。複数箇所・
/// 0 箇所にマッチした場合は panic し、フィクスチャのリファクタリングで
/// 注入前提が崩れたことをテスト失敗として顕在化させる
/// （`negative_type_error.rs` の注入方針と同じ）。
fn replace_unique(content: &str, from: &str, to: &str) -> String {
    assert_eq!(
        content.matches(from).count(),
        1,
        "注入対象の部分文字列 `{from}` が一意に見つからない（ベースラインの \
         リファクタリングでこのテストの前提が崩れている）"
    );
    let injected = content.replacen(from, to, 1);
    assert_ne!(content, injected, "置換後の内容が変化していない");
    injected
}

/// ケース 3（禁止依存追加）向けに、ダミーの `openssl-sys` path クレートを
/// フィクスチャ内に追加し、workspace members・`app` の依存へ組み込んだうえで
/// `Cargo.lock` を再生成する。
///
/// 実際の openssl-sys クレート（crates.io 上の同名クレート）は一切取得しない。
/// `[bans].deny` の判定はクレート名の一致で行われるため、空の `lib.rs` を
/// 持つローカル path クレートでも `cargo deny check bans` の検出を再現できる
/// （完全オフライン、security.md A06）。
fn inject_banned_dependency(project: &Path) {
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
        // cargo-deny 未導入環境（本リポジトリ CI 相当）では policy のみ
        // fail-closed で failed になり、他の 4 チェックは通過したまま
        // 全体として BLOCKED になる、という fail-closed 契約を確認する。
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
}

/// ケース 3（PoC-7 `negative-banned-dependency` 再現）。
///
/// ダミーの `openssl-sys` path クレートを追加し、`deny.toml` の
/// `[bans].deny` に名前一致することで `policy` チェックが failed になり、
/// `fw gate` 全体が BLOCKED になることを確認する。`type_check` は
/// 無関係のまま通過することも確認する。
///
/// `cargo-deny` 未導入環境（本リポジトリ CI 相当）では `policy` は
/// `deny.toml` 実在確認より先に cargo-deny の起動自体に失敗して failed に
/// なる（fail-closed）。この場合も「BLOCKED + policy failed」という
/// 必須アサーションは変わらず成立するため、環境非依存に実行できる。
/// cargo-deny 導入環境でのみ、output に `banned`/`openssl-sys` という
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
