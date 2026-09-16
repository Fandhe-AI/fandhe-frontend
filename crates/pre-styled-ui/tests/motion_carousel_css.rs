//! `fandhe_frontend_pre_styled_ui::carousel_motion` の `data-*` 属性リテラル
//! と `fandhe-frontend-wasm-full` 側の同名定数とのドリフト検知
//! （イシュー #2541、`button_motion_attr_drift.rs` と同型の手法）。
//!
//! `carousel_motion_css()` 自体の CSS 内容契約（`<`/`@keyframes`/
//! `@property` 不在・pure append・coverflow/spring-snap の構造）は
//! `crates/pre-styled-ui/src/carousel_motion.rs` 内の `#[cfg(test)]` で
//! 検証済み（`button_motion.rs` と同じくソースファイル内テストを golden
//! バイト一致の代わりに採用する）。

#![cfg(feature = "motion")]

use std::fs;
use std::path::PathBuf;

use fandhe_frontend_pre_styled_ui::carousel_motion::{
    CAROUSEL_DRAGGING_STATE_ATTR, CAROUSEL_DRAG_ATTR,
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
        "{source_file} の {const_name} 定義が carousel_motion 側（{expected_value:?}）と \
         一致しません。期待した宣言行: {expected_decl:?}"
    );
}

#[test]
fn carousel_drag_attr_literals_match_wasm_full() {
    let content = read_source("carousel_motion.rs");
    assert_const_matches(
        &content,
        "CAROUSEL_DRAG_ATTR",
        CAROUSEL_DRAG_ATTR,
        "carousel_motion.rs",
    );
    assert_const_matches(
        &content,
        "CAROUSEL_DRAGGING_STATE_ATTR",
        CAROUSEL_DRAGGING_STATE_ATTR,
        "carousel_motion.rs",
    );
}
