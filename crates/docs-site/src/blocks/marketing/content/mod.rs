//! Marketing / Content カテゴリの block 登録点（イシュー #2734）。
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。
//!
//! 空雛形（`content.rs`、`Vec::new()` を返すのみ）から本ディレクトリ構成へ
//! 「卒業」したのは本カテゴリ最初の block `content_article_toc`
//! （イシュー #2752）を追加した時点である。

mod content_article_toc;
mod content_columns_screenshot;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        content_article_toc::BLOCK,
        content_columns_screenshot::BLOCK,
    ]
}
