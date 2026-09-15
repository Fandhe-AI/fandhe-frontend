//! `fandhe_frontend_pre_styled_ui::cursor` の `data-*` 属性/CSS カスタム
//! プロパティ名リテラルと、`fandhe-frontend-wasm-full`/
//! `fandhe-frontend-animation` 側の同名定数とのドリフト検知
//! （イシュー #2542）。
//!
//! `fandhe-frontend-pre-styled-ui` はいずれにも依存していない
//! （`Cargo.toml` に dep 行なし）ため、コンパイル時の型共有はできない。
//! 本テストは両クレートのソースを実行時に `fs::read_to_string` で読み、
//! 対応する定数の定義行リテラルが `cursor` 側と一致することを固定する
//! （`button_motion_attr_drift.rs` と同型の手法）。

#![cfg(feature = "motion")]

use std::fs;
use std::path::PathBuf;

use fandhe_frontend_pre_styled_ui::cursor::{
    CURSOR_ACTIVE_ATTR, CURSOR_ATTR, CURSOR_LABEL_ATTR, CURSOR_STATE_ATTR, CURSOR_TARGET_ATTR,
    CURSOR_TARGET_LABEL_ATTR, CURSOR_TARGET_MAGNETIC_ATTR, CURSOR_VARIANT_ATTR, CURSOR_X_PROPERTY,
    CURSOR_Y_PROPERTY,
};

fn sibling_crate_src_path(crate_dir: &str, file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(crate_dir)
        .join("src")
        .join(file)
}

fn read_source(crate_dir: &str, file: &str) -> String {
    let path = sibling_crate_src_path(crate_dir, file);
    fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "{} を読めること（fail-closed。{crate_dir} 側の定数定義との \
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
        "{source_file} の {const_name} 定義が cursor 側（{expected_value:?}）と \
         一致しません。期待した宣言行: {expected_decl:?}"
    );
}

#[test]
fn data_attr_literals_match_wasm_full() {
    let content = read_source("wasm-full", "cursor.rs");
    for (name, value) in [
        ("CURSOR_ATTR", CURSOR_ATTR),
        ("CURSOR_TARGET_ATTR", CURSOR_TARGET_ATTR),
        ("CURSOR_TARGET_LABEL_ATTR", CURSOR_TARGET_LABEL_ATTR),
        ("CURSOR_TARGET_MAGNETIC_ATTR", CURSOR_TARGET_MAGNETIC_ATTR),
        ("CURSOR_ACTIVE_ATTR", CURSOR_ACTIVE_ATTR),
        ("CURSOR_STATE_ATTR", CURSOR_STATE_ATTR),
        ("CURSOR_VARIANT_ATTR", CURSOR_VARIANT_ATTR),
        ("CURSOR_LABEL_ATTR", CURSOR_LABEL_ATTR),
    ] {
        assert_const_matches(&content, name, value, "wasm-full/src/cursor.rs");
    }
}

#[test]
fn css_property_literals_match_frontend_animation() {
    let content = read_source("frontend-animation", "cursor.rs");
    for (name, value) in [
        ("CURSOR_X_PROPERTY", CURSOR_X_PROPERTY),
        ("CURSOR_Y_PROPERTY", CURSOR_Y_PROPERTY),
    ] {
        assert_const_matches(&content, name, value, "frontend-animation/src/cursor.rs");
    }
}
