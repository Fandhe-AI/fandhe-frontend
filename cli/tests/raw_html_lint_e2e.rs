//! イシュー #157（サブイシュー #158/#159）: `raw_html()` 検出の主防御である
//! `clippy::disallowed_methods` が、コメント偽装・リネーム import 経由の呼び出し
//! を実際に検出できることを **実 clippy 起動** で証明する e2e テスト。
//!
//! `cli/src/gate.rs` の単体テスト・`gate_integration.rs` はテキスト走査
//! （保険層）の振る舞いのみを検証しており、主防御（コンパイラのパス解決に
//! 基づく clippy 検出）そのものが機能することは証明しない。本ファイルは
//! `rws-core` へ path 依存する最小フィクスチャクレートを一時ディレクトリに
//! 生成し、`cargo clippy -- -D warnings` を実プロセスとして起動して検証する
//! （ネットワーク非依存・独立 `[workspace]`、`docs/raw-html-lint-design.md`
//! §2 の採用方式 B の受け入れ条件 1「コメント偽装で回避不能」の直接証跡）。
//!
//! clippy 不在環境では `#[ignore]` で沈黙させず明示的にテスト失敗させる
//! （`.claude/rules/coding-rust.md`: テストの `#[ignore]` 追加でごまかさない）。

use std::path::{Path, PathBuf};
use std::process::Command;

/// `cli/` から見た `core/`（`rws-core`）への絶対パス。フィクスチャの
/// `Cargo.toml` はこの絶対パスへ `path` 依存することで、ネットワーク取得や
/// レジストリ解決を伴わずに `rws_core::raw_html` を参照できるようにする。
fn core_crate_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli/ must have a parent directory (workspace root)")
        .join("core")
}

/// 呼び出しごとに一意な一時ディレクトリを作る（並列テスト実行下での衝突回避、
/// `gate_integration.rs` と同じ命名戦略）。
fn tempdir_for_test(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{label}-{}-{:?}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// workspace ルートの `clippy.toml` と同一のポリシー（`disallowed-methods` に
/// `rws_core::raw_html` を宣言）をフィクスチャへ複製する。実運用の設定と
/// 乖離しないよう、内容はハードコードせず契約（キー名・対象パス）のみ固定する。
const CLIPPY_TOML: &str = r#"disallowed-methods = [
    { path = "rws_core::raw_html", reason = "ESCAPE-REVIEWED review required" },
]
"#;

/// `rws-core` への path 依存を持つ独立 `[workspace]` の最小フィクスチャクレートを
/// 構築し、`lib.rs` の内容を `lib_rs_content` で差し替える。
fn write_fixture_crate(dir: &Path, lib_rs_content: &str) {
    std::fs::write(
        dir.join("Cargo.toml"),
        format!(
            "[package]\nname = \"raw-html-lint-fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\npublish = false\n\n[dependencies]\nrws-core = {{ path = {:?} }}\n\n[workspace]\n",
            core_crate_path()
        ),
    )
    .unwrap();
    std::fs::write(dir.join("clippy.toml"), CLIPPY_TOML).unwrap();
    let src = dir.join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("lib.rs"), lib_rs_content).unwrap();
}

/// フィクスチャディレクトリで `cargo clippy --offline -- -D warnings` を実
/// プロセスとして起動する。フィクスチャは毎回新規生成され `Cargo.lock` を
/// 持たないため `gate.rs` 本番経路と同じ `--locked` は使えない（ロック不在で
/// 起動自体が失敗する）。ここで検証したいのは clippy の検出能力そのものであり
/// `--locked` の drift 検知契約ではないため、ネットワーク到達を防ぐ
/// `--offline`（path 依存のみで解決可能）で足りる。
///
/// `CARGO_TARGET_DIR` はフィクスチャ間で共有しない（各フィクスチャ配下の既定
/// `target/` を使う）。全フィクスチャが同一パッケージ名を持つため、並列実行
/// テスト間で `target` を共有するとキャッシュ/フィンガープリントが衝突し、
/// 別フィクスチャの clippy 診断結果を誤って再利用してしまう競合を実測したため
/// （偽陰性の温床になり「見逃しなし」方針に反するため、ビルド時間より正しさを
/// 優先する）。
///
/// 起動自体に失敗した場合（`cargo`/`clippy` コンポーネント不在）は明示メッセージ
/// 付きで `panic!` する（沈黙スキップしない。coding-rust.md のテスト規約）。
fn run_cargo_clippy_in(dir: &Path) -> (bool, String) {
    let output = Command::new("cargo")
        .args(["clippy", "--offline", "--", "-D", "warnings"])
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "failed to launch `cargo clippy` for raw_html_lint_e2e fixture at {}: {e} \
(clippy component must be installed; this test must fail rather than skip when it is absent)",
                dir.display()
            )
        });
    let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&output.stderr));
    (output.status.success(), combined)
}

#[test]
fn direct_unreviewed_call_is_rejected_by_clippy() {
    let dir = tempdir_for_test("raw-html-e2e-direct-call");
    write_fixture_crate(
        &dir,
        "pub fn render(input: &str) -> String {\n    rws_core::raw_html(input.to_string());\n    String::new()\n}\n",
    );

    let (success, output) = run_cargo_clippy_in(&dir);
    assert!(
        !success,
        "unreviewed rws_core::raw_html() call must be rejected by clippy: output={output}"
    );
    assert!(
        output.contains("disallowed_methods"),
        "clippy output must cite disallowed_methods as the reason: output={output}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn comment_only_spoofed_marker_is_still_rejected_by_clippy() {
    // イシュー #157 の受け入れ条件 1（方式比較の核心）: `// ESCAPE-REVIEWED:`
    // コメントは clippy に一切解釈されないため、コメントを添えるだけの偽装は
    // 旧テキスト走査方式では回避できたが clippy では回避不能であることを
    // 実プロセスで証明する。
    let dir = tempdir_for_test("raw-html-e2e-comment-spoof");
    write_fixture_crate(
        &dir,
        "pub fn render(input: &str) -> String {\n    // ESCAPE-REVIEWED: sanitized upstream\n    rws_core::raw_html(input.to_string());\n    String::new()\n}\n",
    );

    let (success, output) = run_cargo_clippy_in(&dir);
    assert!(
        !success,
        "a comment-only ESCAPE-REVIEWED marker must not suppress clippy::disallowed_methods: output={output}"
    );
    assert!(output.contains("disallowed_methods"), "output={output}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn renamed_import_call_is_still_rejected_by_clippy() {
    // clippy はコンパイラの HIR パス解決に基づき定義元 `rws_core::raw_html` を
    // 特定するため、呼び出し側でリネームした import 経由の呼び出しも検出できる
    // ことの証跡（テキスト走査では `raw_html` という識別子文字列に依存する
    // ため見逃し得るケース）。
    let dir = tempdir_for_test("raw-html-e2e-renamed-import");
    write_fixture_crate(
        &dir,
        "use rws_core::raw_html as fragment;\n\npub fn render(input: &str) -> String {\n    fragment(input.to_string());\n    String::new()\n}\n",
    );

    let (success, output) = run_cargo_clippy_in(&dir);
    assert!(
        !success,
        "a renamed import of raw_html must still be caught via HIR path resolution: output={output}"
    );
    assert!(output.contains("disallowed_methods"), "output={output}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn reviewed_expect_attribute_call_is_accepted_by_clippy() {
    // 正当なオプトイン経路（呼び出し文への `#[expect(clippy::disallowed_methods,
    // reason = "ESCAPE-REVIEWED: ...")]`）は clippy を通過することを確認する
    // （主防御を機能させたまま、レビュー宣言された呼び出しは阻害しない）。
    let dir = tempdir_for_test("raw-html-e2e-reviewed-expect");
    write_fixture_crate(
        &dir,
        "pub fn render(input: &str) -> String {\n    #[expect(clippy::disallowed_methods, reason = \"ESCAPE-REVIEWED: trusted fixed fragment\")]\n    rws_core::raw_html(input.to_string());\n    String::new()\n}\n",
    );

    let (success, output) = run_cargo_clippy_in(&dir);
    assert!(
        success,
        "a properly reviewed #[expect(...)] opt-in must pass clippy: output={output}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
