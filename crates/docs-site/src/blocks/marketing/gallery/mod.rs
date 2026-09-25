//! Marketing / Gallery カテゴリの block 登録点（イシュー #2734 で雛形
//! 新設。イシュー #2778 で最初の block（`gallery_image_grid`）を追加、
//! イシュー #2777 で 2 件目の block（[`gallery_carousel`]）を追加、
//! イシュー #2779 で 3 件目の block（[`gallery_masonry`]）を追加、
//! イシュー #2780 で 4 件目の block（[`gallery_split_carousel`]）を追加）。
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への
//! 追記を行うだけでよく、`super`（`marketing`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ
//! 閉じ込めるための構造、イシュー #2734）。

mod gallery_carousel;
mod gallery_image_grid;
mod gallery_masonry;
mod gallery_split_carousel;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        gallery_image_grid::BLOCK,
        gallery_carousel::BLOCK,
        gallery_masonry::BLOCK,
        gallery_split_carousel::BLOCK,
    ]
}
