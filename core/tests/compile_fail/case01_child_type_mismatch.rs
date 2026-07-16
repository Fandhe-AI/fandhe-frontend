//! TASK-5.3 コンパイルエラー品質レビュー用フィクスチャ（意図的にコンパイル不能）。
//!
//! `el()` の第 3 引数は `Vec<Node>` を期待するが、ここでは `Node` を直接渡す。
//! 期待するエラーコード: E0308（mismatched types）。
//! 本ファイルは `core/tests/compile_fail/` サブディレクトリに置かれており、
//! cargo のテストターゲット（`core/tests/*.rs` 直下）には含まれないため
//! 通常のビルド・テストには影響しない。参照方法は同ディレクトリの README を参照。

use rws_core::{el, text, Node};

pub fn case01() -> Node {
    el("div", vec![], text("hi"))
}
