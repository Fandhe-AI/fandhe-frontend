//! TASK-5.3 コンパイルエラー品質レビュー用フィクスチャ（意図的にコンパイル不能）。
//!
//! `Node` に存在しないバリアント名（正しくは `Node::RawHtml`）を参照する。
//! 期待するエラーコード: E0599（no variant found for enum）。
//! 本ファイルは cargo のテストターゲットには含まれない（README 参照）。

use rws_core::Node;

pub fn case06() -> Node {
    Node::Raw("x".to_string())
}
