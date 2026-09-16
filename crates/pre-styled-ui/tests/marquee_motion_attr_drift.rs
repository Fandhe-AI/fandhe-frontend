//! `fandhe_frontend_pre_styled_ui::marquee_motion` の `data-*` 属性名
//! リテラルと、`fandhe-frontend-wasm-full` 側の同名定数とのドリフト検知
//! （イシュー #2540、`button_motion_attr_drift.rs` と同型）。
//!
//! `fandhe-frontend-pre-styled-ui` は `fandhe-frontend-wasm-full` に依存
//! していない（`Cargo.toml` に dep 行なし）ため、コンパイル時の型共有は
//! できない。本テストは `crates/wasm-full/src/ticker.rs` を実行時に
//! `fs::read_to_string` で読み、対応する定数の定義行リテラルが
//! `marquee_motion` 側と一致することを固定する。

#![cfg(feature = "motion")]

use std::fs;
use std::path::PathBuf;

use fandhe_frontend_pre_styled_ui::marquee_motion::{
    TICKER_ACTIVE_ATTR, TICKER_ATTR, TICKER_AXIS_ATTR, TICKER_HOVER_FACTOR_ATTR,
    TICKER_SCROLL_FACTOR_ATTR, TICKER_SPEED_ATTR,
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
        "{source_file} の {const_name} 定義が marquee_motion 側（{expected_value:?}）と \
         一致しません。期待した宣言行: {expected_decl:?}"
    );
}

/// `fandhe-frontend-animation::ticker` は追加複製内の入れ子 ticker を
/// 静的化するために `TICKER_ATTR`/`TICKER_ACTIVE_ATTR` の写しを持つ
/// （PR #2582 codex-review P1 是正）。wasm-full 側と同じ方法で固定する。
#[test]
fn ticker_literals_match_frontend_animation() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("frontend-animation")
        .join("src")
        .join("ticker.rs");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("{} を読めること（fail-closed）: {err}", path.display()));
    assert_const_matches(
        &content,
        "TICKER_ATTR",
        TICKER_ATTR,
        "frontend-animation/ticker.rs",
    );
    assert_const_matches(
        &content,
        "TICKER_ACTIVE_ATTR",
        TICKER_ACTIVE_ATTR,
        "frontend-animation/ticker.rs",
    );
}

#[test]
fn ticker_literals_match_wasm_full() {
    let content = read_source("ticker.rs");
    assert_const_matches(&content, "TICKER_ATTR", TICKER_ATTR, "ticker.rs");
    assert_const_matches(
        &content,
        "TICKER_SPEED_ATTR",
        TICKER_SPEED_ATTR,
        "ticker.rs",
    );
    assert_const_matches(
        &content,
        "TICKER_HOVER_FACTOR_ATTR",
        TICKER_HOVER_FACTOR_ATTR,
        "ticker.rs",
    );
    assert_const_matches(
        &content,
        "TICKER_SCROLL_FACTOR_ATTR",
        TICKER_SCROLL_FACTOR_ATTR,
        "ticker.rs",
    );
    assert_const_matches(&content, "TICKER_AXIS_ATTR", TICKER_AXIS_ATTR, "ticker.rs");
    assert_const_matches(
        &content,
        "TICKER_ACTIVE_ATTR",
        TICKER_ACTIVE_ATTR,
        "ticker.rs",
    );
}
