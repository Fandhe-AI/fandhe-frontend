//! Marketing / Stats カテゴリの block 登録点（イシュー #2734 で雛形追加、
//! イシュー #2804 で `stats_split` を追加して卒業）。

mod stats_split;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![stats_split::BLOCK]
}
