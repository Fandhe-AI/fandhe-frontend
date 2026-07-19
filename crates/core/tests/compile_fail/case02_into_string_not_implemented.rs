//! TASK-5.3 コンパイルエラー品質レビュー用フィクスチャ（意図的にコンパイル不能）。
//!
//! `text()` は `impl Into<String>` を要求するが、`i32` はこれを実装しない。
//! 期待するエラーコード: E0277（トレイト境界未充足）。
//! 本ファイルは cargo のテストターゲットには含まれない（README 参照）。

use fandhe_frontend_core::{text, Node};

pub fn case02() -> Node {
    text(42)
}
