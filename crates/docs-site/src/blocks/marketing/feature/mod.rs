//! Marketing / Feature カテゴリの block 登録点（イシュー #2734）。
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod feature_accordion_image;
mod feature_alternating_rows;
mod feature_expand;
mod feature_image_cards;
mod feature_large_screenshot;
mod feature_side_heading_grid;
mod feature_split_list_image;
use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        feature_accordion_image::BLOCK,
        feature_alternating_rows::BLOCK,
        feature_expand::BLOCK,
        feature_image_cards::BLOCK,
        feature_large_screenshot::BLOCK,
        feature_side_heading_grid::BLOCK,
        feature_split_list_image::BLOCK,
    ]
}
