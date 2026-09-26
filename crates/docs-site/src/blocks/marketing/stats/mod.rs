//! Marketing / Stats カテゴリの block 登録点（雛形はイシュー #2734）。
//! #2801 で最初の block（[`stats_background_image`]）を追加して卒業し、
//! イシュー #2803 で 2 件目の block（[`stats_row`]）を追加した。
//! 手順は `docs/design/docs-site-blocks-section.md` §18 参照。
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で
//! 集約する。新規 block を追加する際は本ファイルへ `mod` 宣言と
//! `blocks()` への追記を行うだけでよく、`super`（`marketing`）側・
//! トップレベル `crate::blocks` 側の変更は不要（並列 PR 間の衝突を
//! カテゴリ内へ閉じ込めるための構造、イシュー #2734）。

mod stats_background_image;
mod stats_row;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![stats_background_image::BLOCK, stats_row::BLOCK]
}
