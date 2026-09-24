//! Marketing / Blog カテゴリの block 登録点（イシュー #2734。イシュー
//! #2810 で最初の block（`blog_grid_image`）を追加し、空雛形から
//! ディレクトリ化した）。
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod blog_featured_article;
mod blog_featured_with_list;
mod blog_grid_image;
use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        blog_featured_article::BLOCK,
        blog_featured_with_list::BLOCK,
        blog_grid_image::BLOCK,
    ]
}
