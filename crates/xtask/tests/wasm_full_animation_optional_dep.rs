//! イシュー #2417「`fandhe-frontend-animation` crate 雛形を作成し CI・publish
//! 運用へ組み込む」の fail-closed 契約テスト。
//!
//! `crates/wasm-full/Cargo.toml` は `fandhe-frontend-animation` を
//! `optional = true` の依存として取り込む（Phase 4 の各配線 feature が
//! `dep:` で有効化する予定、`default`/`WASM_DIST_FEATURES` には含めない）。
//! optional 依存は `cargo metadata` の既定解決（`resolve.nodes[].deps`）に
//! 現れないため、`fw structure`（`crates/cli/src/main.rs
//! ::check_crate_and_dependency_consistency`）の実依存突き合わせの対象外であり、
//! `structure.toml` の `directories.frontend-animation` は
//! `allowed_dependents = ["wasm-full"]` を宣言していない（意図的な非対称、
//! `structure.toml` 該当節コメント参照）。
//!
//! この前提が崩れる（既定 feature で `fandhe-frontend-animation` が有効化
//! されてしまう）と `fw structure` は突如「宣言漏れ」として fail するため、
//! 本テストが `cargo tree` の実測で両方向を機械固定する:
//!
//! 1. 既定 feature（unification なし、`-p` 単体）では `fandhe-frontend-animation`/
//!    `fandhe-animation` が依存グラフに現れないこと
//! 2. `--all-features` では現れること（陽性対照。feature 名の変更・`dep:` 化に
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
fn default_features_exclude_frontend_animation() {
    let tree = run_cargo_tree(&[]);
    assert!(
        !tree.contains("fandhe-frontend-animation"),
        "既定 feature の依存グラフに fandhe-frontend-animation が現れている。\
         optional 依存が既定 on 化された可能性がある（structure.toml の \
         frontend-animation.allowed_dependents 非対称宣言の前提が崩れる）:\n{tree}"
    );
    assert!(
        !tree.contains("fandhe-animation "),
        "既定 feature の依存グラフに fandhe-animation が現れている（推移的に \
         fandhe-frontend-animation が有効化された可能性がある）:\n{tree}"
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
