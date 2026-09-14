//! `fandhe_frontend_pre_styled_ui::recipe::STAGGER_INDEX_VAR`（`motion`
//! feature 配下、#2384）と `fandhe-frontend-wasm-full` 側の動的書き込み
//! （`stagger_index::sync_stagger_index`、#2397）が使う CSS custom
//! property 名のドリフト検知。`content_height_var_drift.rs` と同型の
//! ソース走査型 fail-closed 契約テストである（本クレートは
//! `fandhe-frontend-wasm-full` に依存していないため型共有ができない）。

#![cfg(feature = "motion")]

use std::fs;
use std::path::PathBuf;

use fandhe_frontend_pre_styled_ui::recipe::STAGGER_INDEX_VAR;

/// `crates/wasm-full/src/stagger_index.rs` の絶対パスを、本クレートの
/// `CARGO_MANIFEST_DIR`（`crates/pre-styled-ui`）から兄弟クレートへの
/// 相対パスとして組み立てる。
fn wasm_full_stagger_index_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("wasm-full")
        .join("src")
        .join("stagger_index.rs")
}

#[test]
fn stagger_index_var_matches_wasm_full_literal() {
    let path = wasm_full_stagger_index_path();
    let content = fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "{} を読めること（fail-closed。wasm-full 側の STAGGER_INDEX_VAR \
             定義とのドリフト検知が本テストの目的であり、読み取り不能を \
             PASS 扱いにしない）: {err}",
            path.display()
        )
    });

    let expected_decl = format!("pub const STAGGER_INDEX_VAR: &str = \"{STAGGER_INDEX_VAR}\";");
    assert!(
        content.contains(&expected_decl),
        "crates/wasm-full/src/stagger_index.rs の STAGGER_INDEX_VAR 定義が \
         recipe::STAGGER_INDEX_VAR（{STAGGER_INDEX_VAR:?}）と一致しません。\
         期待した宣言行: {expected_decl:?}"
    );
}
