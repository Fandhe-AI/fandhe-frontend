//! `fandhe_frontend_pre_styled_ui::recipe::CONTENT_HEIGHT_VAR` と
//! `fandhe-frontend-wasm-full` 側の実測高さ CSS custom property 名の
//! ドリフト検知（イシュー #2192）。
//!
//! `fandhe-frontend-pre-styled-ui` は `fandhe-frontend-wasm-full` に
//! 依存していない（`Cargo.toml` に dep 行なし）ため、コンパイル時の型
//! 共有はできない。本テストは `crates/wasm-full/src/content_height.rs`
//! を実行時に `fs::read_to_string` で読み、`CONTENT_HEIGHT_VAR` の定義行
//! リテラルが `recipe::CONTENT_HEIGHT_VAR` と一致することを固定する
//! （`reexport_policy.rs` と同じソース走査型の fail-closed 契約テスト
//! 手法）。
//!
//! `include_str!` ではなく実行時 `fs::read_to_string` を使う理由: `cargo
//! package` 後の packaged コピー単体ではワークスペース内の他クレート
//! パスは存在しない。本テストは `cargo package` の検証ビルド（テストを
//! 実行しない）には現れず、通常の `cargo test` 実行時のみ走るため、
//! ワークスペースルートからの相対パス解決で問題ない。

use std::fs;
use std::path::PathBuf;

use fandhe_frontend_pre_styled_ui::recipe::CONTENT_HEIGHT_VAR;

/// `crates/wasm-full/src/content_height.rs` の絶対パスを、本クレートの
/// `CARGO_MANIFEST_DIR`（`crates/pre-styled-ui`）から兄弟クレートへの
/// 相対パスとして組み立てる。
fn wasm_full_content_height_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("wasm-full")
        .join("src")
        .join("content_height.rs")
}

#[test]
fn content_height_var_matches_wasm_full_literal() {
    let path = wasm_full_content_height_path();
    let content = fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "{} を読めること（fail-closed。wasm-full 側の CONTENT_HEIGHT_VAR \
             定義とのドリフト検知が本テストの目的であり、読み取り不能を \
             PASS 扱いにしない）: {err}",
            path.display()
        )
    });

    let expected_decl = format!("pub const CONTENT_HEIGHT_VAR: &str = \"{CONTENT_HEIGHT_VAR}\";");
    assert!(
        content.contains(&expected_decl),
        "crates/wasm-full/src/content_height.rs の CONTENT_HEIGHT_VAR 定義が \
         recipe::CONTENT_HEIGHT_VAR（{CONTENT_HEIGHT_VAR:?}）と一致しません。\
         期待した宣言行: {expected_decl:?}"
    );
}
