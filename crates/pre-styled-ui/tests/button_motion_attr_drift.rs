//! `fandhe_frontend_pre_styled_ui::button_motion` の `data-*` 属性/CSS
//! カスタムプロパティ名リテラルと、`fandhe-frontend-wasm-full` 側の同名
//! 定数とのドリフト検知（イシュー #2538）。
//!
//! `fandhe-frontend-pre-styled-ui` は `fandhe-frontend-wasm-full` に依存
//! していない（`Cargo.toml` に dep 行なし）ため、コンパイル時の型共有は
//! できない。本テストは `crates/wasm-full/src/hold_to_confirm.rs`/
//! `add_to_basket.rs` を実行時に `fs::read_to_string` で読み、対応する
//! 定数の定義行リテラルが `button_motion` 側と一致することを固定する
//! （`content_height_var_drift.rs` と同型の手法）。

#![cfg(feature = "motion")]

use std::fs;
use std::path::PathBuf;

use fandhe_frontend_pre_styled_ui::button_motion::{
    ADD_TO_BASKET_ATTR, HOLD_DURATION_MS_ATTR, HOLD_PROGRESS_VAR, HOLD_TO_CONFIRM_ATTR,
};

fn wasm_full_src_path(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("wasm-full")
        .join("src")
        .join(file)
}

fn read_source(file: &str) -> String {
    let path = wasm_full_src_path(file);
    fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "{} を読めること（fail-closed。wasm-full 側の定数定義との \
             ドリフト検知が本テストの目的であり、読み取り不能を PASS 扱い \
             にしない）: {err}",
            path.display()
        )
    })
}

fn assert_const_matches(content: &str, const_name: &str, expected_value: &str, source_file: &str) {
    let expected_decl = format!("pub const {const_name}: &str = \"{expected_value}\";");
    assert!(
        content.contains(&expected_decl),
        "{source_file} の {const_name} 定義が button_motion 側（{expected_value:?}）と \
         一致しません。期待した宣言行: {expected_decl:?}"
    );
}

#[test]
fn hold_to_confirm_literals_match_wasm_full() {
    let content = read_source("hold_to_confirm.rs");
    assert_const_matches(
        &content,
        "HOLD_TO_CONFIRM_ATTR",
        HOLD_TO_CONFIRM_ATTR,
        "hold_to_confirm.rs",
    );
    assert_const_matches(
        &content,
        "HOLD_DURATION_MS_ATTR",
        HOLD_DURATION_MS_ATTR,
        "hold_to_confirm.rs",
    );
    assert_const_matches(
        &content,
        "HOLD_PROGRESS_VAR",
        HOLD_PROGRESS_VAR,
        "hold_to_confirm.rs",
    );
}

#[test]
fn add_to_basket_literal_matches_wasm_full() {
    let content = read_source("add_to_basket.rs");
    assert_const_matches(
        &content,
        "ADD_TO_BASKET_ATTR",
        ADD_TO_BASKET_ATTR,
        "add_to_basket.rs",
    );
}
