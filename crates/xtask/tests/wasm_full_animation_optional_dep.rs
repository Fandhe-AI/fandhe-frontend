//! イシュー #2417「`fandhe-frontend-animation` crate 雛形を作成し CI・publish
//! 運用へ組み込む」で新設した fail-closed 契約テスト。
//!
//! #2417 時点では `crates/wasm-full/Cargo.toml` の `fandhe-frontend-animation`
//! （`optional = true`）は `default`/`WASM_DIST_FEATURES` のいずれにも含まれず、
//! `structure.toml` の `directories.frontend-animation` も
//! `allowed_dependents = ["wasm-full"]` を宣言しない非対称な状態だった。
//!
//! イシュー #2398 で `animate` feature（既定 on、`dep:fandhe-frontend-animation`）・
//! イシュー #2403/#2517 で `animation-driver` feature（既定 on、同じく
//! `dep:fandhe-frontend-animation`）をそれぞれ独立に新設したことにより、
//! 本依存は既定 feature の依存グラフに現れるようになった（`structure.toml`・
//! 本ファイルとも両イシューで追随更新済み。いずれか片方の feature のみを
//! off にしても他方が on であれば依存グラフには引き続き現れる）。
//!
//! 本テストは `cargo tree` の実測で両方向を機械固定する:
//!
//! 1. `--no-default-features`（素の状態）では `fandhe-frontend-animation`/
//!    `fandhe-animation` が依存グラフに現れないこと（`default-features = false`
//!    利用者が完全に外せることの証明）
//! 2. 引数なし（既定 feature）では現れること（#2398/#2403/#2517 で反転した
//!    新しい不変条件）
//! 3. `--all-features` でも現れること（陽性対照。feature 名の変更・`dep:` 化に
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
fn bare_no_default_features_excludes_frontend_animation() {
    let tree = run_cargo_tree(&["--no-default-features"]);
    assert!(
        !tree.contains("fandhe-frontend-animation"),
        "--no-default-features の依存グラフに fandhe-frontend-animation が \
         現れている。`default-features = false` 利用者が本依存を完全に外せる \
         という保証が崩れている:\n{tree}"
    );
    assert!(
        !tree.contains("fandhe-animation "),
        "--no-default-features の依存グラフに fandhe-animation が現れている \
         （推移的に fandhe-frontend-animation が有効化された可能性がある）:\n{tree}"
    );
}

#[test]
fn default_features_include_frontend_animation() {
    let tree = run_cargo_tree(&[]);
    assert!(
        tree.contains("fandhe-frontend-animation"),
        "既定 feature の依存グラフに fandhe-frontend-animation が現れない。\
         \"animate\"（イシュー #2398）・\"animation-driver\"（イシュー #2403/#2517） \
         のいずれも既定 on の feature（dep:fandhe-frontend-animation）が \
         default 配列から外れた可能性がある:\n{tree}"
    );
    assert!(
        tree.contains("fandhe-animation "),
        "既定 feature の依存グラフに fandhe-animation が現れない \
         （fandhe-frontend-animation 経由の推移依存）:\n{tree}"
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
