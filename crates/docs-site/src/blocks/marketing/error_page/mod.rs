//! Marketing / Error Page カテゴリの block 登録点（イシュー #2734。
//! #2836 で最初の block（`error_page_background_image`）を追加して卒業し、
//! #2837 で 2 件目（`error_page_centered`）・#2838 で 3 件目
//! （`error_page_popular_links`）・#2841 で 4 件目
//! （`error_page_split_image`）を追加した。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。新規 block を追加
//! する際は本ファイルへ `mod` 宣言と `blocks()` への追記を行うだけでよく、
//! `super`（`marketing`）側・トップレベル `crate::blocks` 側の変更は不要
//! （並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造、イシュー #2734）。

mod error_page_background_image;
mod error_page_centered;
mod error_page_popular_links;
mod error_page_split_image;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        error_page_background_image::BLOCK,
        error_page_centered::BLOCK,
        error_page_popular_links::BLOCK,
        error_page_split_image::BLOCK,
    ]
}
