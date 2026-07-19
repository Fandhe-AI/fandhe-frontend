//! TASK-5.3 コンパイルエラー品質レビュー用フィクスチャ（意図的にコンパイル不能）。
//!
//! 子ノード列 `Vec<Node>` に `&str` リテラルを混在させる（`text()` での
//! ラップ忘れを想定）。
//! 期待するエラーコード: E0308（mismatched types）。
//! 本ファイルは cargo のテストターゲットには含まれない（README 参照）。

use fandhe_frontend_core::{text, Node};

pub fn case05() -> Vec<Node> {
    vec![text("a"), "b"]
}
