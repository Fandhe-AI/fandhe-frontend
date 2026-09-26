//! Marketing / Stats カテゴリの block 登録点（イシュー #2734 で雛形新設、
//! #2801 で最初の block（`stats_background_image`）を追加して卒業し、
//! #2802 で 2 件目の block（`stats_cards`）・イシュー #2805 で 3 件目の
//! block（`stats_timeline`）を追加した）。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。新規 block を追加
//! する際は本ファイルへ `mod` 宣言と `blocks()` への追記を行うだけでよく、
//! `super`（`marketing`）側・トップレベル `crate::blocks` 側の変更は不要
//! （並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造、イシュー #2734）。

mod stats_background_image;
mod stats_cards;
mod stats_timeline;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        stats_background_image::BLOCK,
        stats_cards::BLOCK,
        stats_timeline::BLOCK,
    ]
}
