//! TASK-5.3 コンパイルエラー品質レビュー用フィクスチャ（意図的にコンパイル不能）。
//!
//! `render()` は `&Node` を期待するが、参照を渡し忘れて所有値をそのまま渡す。
//! 期待するエラーコード: E0308（mismatched types、`help: consider borrowing` 付き）。
//! 本ファイルは cargo のテストターゲットには含まれない（README 参照）。

use fandhe_frontend_core::{render, text};

pub fn case07() -> String {
    let node = text("hi");
    render(node)
}
