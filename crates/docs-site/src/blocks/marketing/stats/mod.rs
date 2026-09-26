//! Marketing / Stats カテゴリの block 登録点（イシュー #2734。#2801 で
//! 最初の block（`stats_background_image`）を追加して卒業した。#2806 で
//! `stats_with_image` を追加）。手順は `docs/design/docs-site-blocks-
//! section.md` §18 参照。新規 block を追加する際は本ファイルへ `mod`
//! 宣言と `blocks()` への追記を行うだけでよく、`super`（`marketing`）側・
//! トップレベル `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ
//! 内へ閉じ込めるための構造、イシュー #2734）。

mod stats_background_image;
mod stats_with_image;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![stats_background_image::BLOCK, stats_with_image::BLOCK]
}
