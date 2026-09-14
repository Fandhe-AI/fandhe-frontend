//! イシュー #2403/#2517「rAF Driver + DOM Target を実装する（wasm-full は
//! 配線のみ）」の fail-closed 契約テスト。
//!
//! `crates/wasm-full/Cargo.toml` は `fandhe-frontend-animation` を
//! `optional = true` の依存として取り込む。イシュー #2417 時点では
//! 既定 feature では無効（`default`/`WASM_DIST_FEATURES` のいずれにも
//! 含めない）だったが、本イシューで新設した `"animation-driver"` feature
//! （`dep:fandhe-frontend-animation`）が `default` 配列に既定 on で列挙
//! されたため、既定 feature の依存グラフにも現れるようになった
//! （前提の反転、`structure.toml` の `directories.wasm-full.depends_on`/
//! `directories.frontend-animation.allowed_dependents` の対称宣言もこの
//! イシューで追加済み）。
//!
//! 本テストが `cargo tree` の実測で両方向を機械固定する:
//!
//! 1. 既定 feature（unification なし、`-p` 単体）で `fandhe-frontend-animation`/
//!    `fandhe-animation` が依存グラフに現れること
//! 2. `--all-features` でも現れ続けること（陽性対照。feature 名の変更・`dep:` 化に
//!    対しても「optional 依存として存在する」こと自体は検証できる）
//!
//! 外部 JSON/TOML パーサは使わず `cargo tree` の標準出力を文字列走査するのみ
//! （REQ-3、`crates/xtask/tests/wasm_dist_features_contract.rs` と同方針）。

use std::process::Command;

/// workspace ルート（`crates/xtask/` から 2 段上）の絶対パス。
fn workspace_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/xtask/ から 2 段上でワークスペースルートに到達する（イシュー #436）")
        .to_path_buf()
}

/// `cargo tree -p fandhe-frontend-wasm-full -e normal --prefix none --locked`
/// （+ 追加引数）を実行し、stdout を返す。非 0 終了は panic させる
/// （fail-closed。cargo 自体の異常は「未検証」として PASS 扱いにしない）。
fn run_cargo_tree(extra_args: &[&str]) -> String {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(workspace_root()).args([
        "tree",
        "-p",
        "fandhe-frontend-wasm-full",
        "-e",
        "normal",
        "--prefix",
        "none",
        "--locked",
    ]);
    cmd.args(extra_args);
    let output = cmd
        .output()
        .expect("cargo tree の起動に失敗した（cargo バイナリが PATH にない可能性がある）");
    assert!(
        output.status.success(),
        "cargo tree が非 0 終了した: status={:?} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("cargo tree の出力が UTF-8 として不正")
}

#[test]
fn default_features_include_frontend_animation() {
    let tree = run_cargo_tree(&[]);
    assert!(
        tree.contains("fandhe-frontend-animation"),
        "既定 feature の依存グラフに fandhe-frontend-animation が現れない。\
         \"animation-driver\" feature（既定 on、dep:fandhe-frontend-animation）が \
         default 配列から外れた可能性がある:\n{tree}"
    );
    assert!(
        tree.contains("fandhe-animation "),
        "既定 feature の依存グラフに fandhe-animation が現れない（推移的に \
         fandhe-frontend-animation が有効化されているはず）:\n{tree}"
    );
}

#[test]
fn all_features_include_frontend_animation() {
    let tree = run_cargo_tree(&["--all-features"]);
    assert!(
        tree.contains("fandhe-frontend-animation"),
        "--all-features でも fandhe-frontend-animation が依存グラフに現れない。\
         optional 依存として宣言されていない可能性がある（陽性対照の破損）:\n{tree}"
    );
    assert!(
        tree.contains("fandhe-animation "),
        "--all-features でも fandhe-animation が依存グラフに現れない:\n{tree}"
    );
}
