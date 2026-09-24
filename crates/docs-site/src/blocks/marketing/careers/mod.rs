//! Marketing / Careers カテゴリの block 登録点（イシュー #2734。イシュー
//! #2815 で最初の block（`careers_card_grid`）を追加し、`banner/mod.rs` と
//! 同型のディレクトリ構成へ「卒業」した。`docs/design/docs-site-blocks-section.md`
//! §18 参照。新規 block を追加する際は本ファイルへ `mod` 宣言と
//! `blocks()` への追記を行うだけでよく、`super`（`marketing`）側・
//! トップレベル `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ
//! 内へ閉じ込めるための構造、イシュー #2734）。

mod careers_card_grid;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![careers_card_grid::BLOCK]
}
