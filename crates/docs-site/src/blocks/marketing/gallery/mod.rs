//! Marketing / Gallery カテゴリの block 登録点（イシュー #2734。最初の
//! block（`gallery-image-grid`、#2778）追加に伴い空雛形からディレクトリ化
//! した。手順は `docs/design/docs-site-blocks-section.md` §18 参照）。
//! 2 件目の `gallery-masonry`（#2779）は §19 索引参照。

mod gallery_image_grid;
mod gallery_masonry;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![gallery_image_grid::BLOCK, gallery_masonry::BLOCK]
}
