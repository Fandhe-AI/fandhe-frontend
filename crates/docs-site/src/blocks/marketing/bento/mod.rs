//! Marketing / Bento カテゴリの block 登録点（イシュー #2734）。
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod bento_asymmetric_rows;
mod bento_staggered;
mod bento_three_column_tall;
mod bento_two_column;
use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        bento_asymmetric_rows::BLOCK,
        bento_staggered::BLOCK,
        bento_three_column_tall::BLOCK,
        bento_two_column::BLOCK,
    ]
}
