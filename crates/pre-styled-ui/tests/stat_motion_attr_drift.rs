//! `fandhe_frontend_pre_styled_ui::stat_motion` の `data-*` 属性リテラルと、
//! `fandhe-frontend-wasm-full` 側の同名定数とのドリフト検知（イシュー
//! #2539）。`tests/button_motion_attr_drift.rs` と同型の手法
//! （`fs::read_to_string` による実行時ソース走査、コンパイル時の型共有は
//! できないため）。
#![cfg(feature = "motion")]

use std::fs;
use std::path::PathBuf;

use fandhe_frontend_pre_styled_ui::stat_motion::{
    COUNT_UP_ATTR, COUNT_UP_DURATION_MS_ATTR, COUNT_UP_TRIGGER_ATTR, COUNT_UP_TRIGGER_IN_VIEW,
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
        "{source_file} の {const_name} 定義が stat_motion 側（{expected_value:?}）と \
         一致しません。期待した宣言行: {expected_decl:?}"
    );
}

#[test]
fn count_up_literals_match_wasm_full() {
    let content = read_source("count_up.rs");
    assert_const_matches(&content, "COUNT_UP_ATTR", COUNT_UP_ATTR, "count_up.rs");
    assert_const_matches(
        &content,
        "COUNT_UP_DURATION_MS_ATTR",
        COUNT_UP_DURATION_MS_ATTR,
        "count_up.rs",
    );
    assert_const_matches(
        &content,
        "COUNT_UP_TRIGGER_ATTR",
        COUNT_UP_TRIGGER_ATTR,
        "count_up.rs",
    );
    assert_const_matches(
        &content,
        "COUNT_UP_TRIGGER_IN_VIEW",
        COUNT_UP_TRIGGER_IN_VIEW,
        "count_up.rs",
    );
}
