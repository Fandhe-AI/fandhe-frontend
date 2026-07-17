//! `templates/default`（標準プロジェクトテンプレート）の負例投入テスト。
//!
//! # 契約（REQ-4 / TASK-4.4）
//!
//! REQ-4 の受け入れ基準 3「型的に不正な AI 生成コードが `cargo check` の段階で
//! 機械的に弾かれること」（PoC-7 `negative-type-error` ケースの実測を製品
//! テンプレート向けに再現）を回帰テストとして固定する。
//!
//! 本テストの削除・弱体化（アサーション削除・`#[ignore]` 付与等）は REQ-4
//! 受け入れ基準の担保を失わせる。`xtask/tests/template_negative_type_error.rs`
//! が本ファイルの実在・必須アサーションの記述を多層防御として静的検証する。
//!
//! 意味的な脆弱性・ロジック誤りの検出は REQ-4 の明示的スコープ外
//! （`docs/spec/04-requirements.md` 参照）であり、本テストが扱うのは
//! コンパイラの型検査で機械的に検出可能な誤り（E0277/E0308）に限る。
//!
//! すべて `--offline` で実行する（本テンプレートは外部依存ゼロのため
//! ネットワークアクセス不要。ヘルメチックなテスト実行を保証する）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// このテンプレートパッケージのルート（`Cargo.toml` が置かれているディレクトリ）。
fn template_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// 一時プロジェクトを書き出すスクラッチルート。
///
/// `CARGO_TARGET_TMPDIR` は cargo がテストバイナリ実行時に設定する
/// target 配下の一時ディレクトリで、target 内に閉じるため固定パスの
/// `/tmp` 直書きや外部入力由来のパス組み立てを避けられる
/// （パストラバーサル対策の一環）。未設定環境向けに OS 標準の一時領域へ
/// フォールバックする。
fn scratch_root() -> PathBuf {
    std::env::var("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
}

/// テンプレートの `Cargo.toml`（および存在すれば `Cargo.lock`）をコピーし、
/// `main_rs_content` を `src/main.rs` として書き出した一時プロジェクトを
/// `scratch_root()/negative-type-error-<case_name>` に構築する。
fn write_case_project(case_name: &str, main_rs_content: &str) -> PathBuf {
    let dest = scratch_root().join(format!("negative-type-error-{case_name}"));
    // 前回実行の残骸が残っていても内容を上書きできるよう再作成する。
    let _ = fs::remove_dir_all(&dest);
    fs::create_dir_all(dest.join("src")).expect("一時プロジェクトディレクトリの作成に失敗した");

    fs::copy(template_root().join("Cargo.toml"), dest.join("Cargo.toml"))
        .expect("Cargo.toml のコピーに失敗した");
    let lock_src = template_root().join("Cargo.lock");
    if lock_src.is_file() {
        fs::copy(&lock_src, dest.join("Cargo.lock")).expect("Cargo.lock のコピーに失敗した");
    }
    fs::write(dest.join("src/main.rs"), main_rs_content).expect("main.rs の書き込みに失敗した");

    dest
}

/// 指定したプロジェクトディレクトリで `cargo check --offline` を実行する。
///
/// 固定引数のみを子プロセスへ渡し、外部入力からコマンドライン引数を
/// 組み立てない（インジェクション対策）。`--offline` によりネットワーク
/// アクセスを行わないことを保証する。
fn run_cargo_check(project_dir: &Path) -> Output {
    Command::new("cargo")
        .args(["check", "--offline"])
        .current_dir(project_dir)
        .output()
        .expect("cargo check の起動に失敗した")
}

/// テンプレート本体（正例）の `src/main.rs` の内容を返す。
fn template_main_rs() -> String {
    fs::read_to_string(template_root().join("src/main.rs"))
        .expect("templates/default/src/main.rs の読み込みに失敗した")
}

/// 正例ベースライン: 無改変のテンプレートソースが `cargo check` を通過すること。
///
/// 後続の負例テストが「注入した型不正」に起因して失敗していることを保証
/// する対照群。このテストが落ちる場合、負例側の失敗は環境要因の可能性が
/// あり切り分けが必要になる。
#[test]
fn baseline_template_passes_cargo_check() {
    let project = write_case_project("baseline", &template_main_rs());
    let output = run_cargo_check(&project);
    assert!(
        output.status.success(),
        "無改変のテンプレートで cargo check が失敗した（対照群が壊れている）。\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// 負例 1（PoC-7 `negative-type-error` 再現・E0277）。
///
/// `find_item` の比較を `it.id == target_id`（`String` と `&str`）から
/// `it.id == 42`（`String` と整数リテラル）へ改変し、`cargo check` が
/// 非 0 終了かつ `error[E0277]` を報告することを確認する。
#[test]
fn type_mismatched_comparison_is_rejected_with_e0277() {
    let original = template_main_rs();
    let injected = original.replace("it.id == target_id", "it.id == 42");
    assert_ne!(
        original, injected,
        "注入対象の比較式 `it.id == target_id` がテンプレートに見つからない \
         （main.rs のリファクタリングでこのテストの前提が崩れている）"
    );

    let project = write_case_project("e0277", &injected);
    let output = run_cargo_check(&project);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "型不正な比較（String と整数リテラル）が cargo check を通過してしまった \
         （REQ-4 受け入れ基準の喪失）。stderr: {stderr}"
    );
    assert!(
        stderr.contains("E0277"),
        "期待した rustc エラーコード E0277 が stderr に含まれない: {stderr}"
    );
}

/// 負例 2（型不一致・E0308）。
///
/// `&str` リテラルを `u32` 変数へ束縛する型不一致コードを `main()` 冒頭に
/// 注入し、`cargo check` が非 0 終了かつ `error[E0308]` を報告することを
/// 確認する。「型不正」検出の代表 2 類型（比較式の型不一致 / 束縛の型不一致）
/// をカバーする。
#[test]
fn type_mismatched_binding_is_rejected_with_e0308() {
    let original = template_main_rs();
    let marker = "fn main() {";
    assert!(
        original.contains(marker),
        "main() の開始位置 `{marker}` がテンプレートに見つからない \
         （main.rs のリファクタリングでこのテストの前提が崩れている）"
    );
    let injected = original.replacen(marker, "fn main() {\n    let _bad: u32 = \"42\";", 1);

    let project = write_case_project("e0308", &injected);
    let output = run_cargo_check(&project);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "型不正な束縛（&str を u32 へ）が cargo check を通過してしまった \
         （REQ-4 受け入れ基準の喪失）。stderr: {stderr}"
    );
    assert!(
        stderr.contains("E0308"),
        "期待した rustc エラーコード E0308 が stderr に含まれない: {stderr}"
    );
}
