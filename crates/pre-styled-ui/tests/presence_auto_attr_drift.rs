//! `fandhe_frontend_pre_styled_ui::list_motion::PRESENCE_AUTO_ATTR` と
//! `fandhe-frontend-wasm-full` 側の動的配線（`list_presence::
//! PRESENCE_AUTO_ATTR`、イシュー #2544）が使う CSS セレクタ/DOM 属性名
//! のドリフト検知。`stagger_index_var_drift.rs` と同型のソース走査型
//! fail-closed 契約テストである（本クレートは `fandhe-frontend-wasm-full`
//! に依存していないため型共有ができない）。

#![cfg(feature = "motion")]

use std::fs;
use std::path::PathBuf;

use fandhe_frontend_pre_styled_ui::list_motion::PRESENCE_AUTO_ATTR;

/// `crates/wasm-full/src/list_presence.rs` の絶対パスを、本クレートの
/// `CARGO_MANIFEST_DIR`（`crates/pre-styled-ui`）から兄弟クレートへの
/// 相対パスとして組み立てる。
fn wasm_full_list_presence_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("wasm-full")
        .join("src")
        .join("list_presence.rs")
}

#[test]
fn presence_auto_attr_matches_wasm_full_literal() {
    let path = wasm_full_list_presence_path();
    let content = fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "{} を読めること（fail-closed。wasm-full 側の PRESENCE_AUTO_ATTR \
             定義とのドリフト検知が本テストの目的であり、読み取り不能を \
             PASS 扱いにしない）: {err}",
            path.display()
        )
    });

    let expected_decl = format!("pub const PRESENCE_AUTO_ATTR: &str = \"{PRESENCE_AUTO_ATTR}\";");
    assert!(
        content.contains(&expected_decl),
        "crates/wasm-full/src/list_presence.rs の PRESENCE_AUTO_ATTR 定義が \
         list_motion::PRESENCE_AUTO_ATTR（{PRESENCE_AUTO_ATTR:?}）と一致\
         しません。期待した宣言行: {expected_decl:?}"
    );
}
