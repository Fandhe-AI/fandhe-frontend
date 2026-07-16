//! TASK-5.3 コンパイルエラー品質レビュー用フィクスチャ（意図的にコンパイル不能）。
//!
//! `el()` の属性引数は `Vec<(&str, &str)>` を期待するが、ここでは属性値に
//! 数値リテラルを渡す。
//! 期待するエラーコード: E0308（mismatched types）。
//! 本ファイルは cargo のテストターゲットには含まれない（README 参照）。

use rws_core::{el, Node};

pub fn case04() -> Node {
    el("div", vec![("class", 3)], vec![])
}
