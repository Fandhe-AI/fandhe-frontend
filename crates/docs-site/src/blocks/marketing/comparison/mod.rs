//! Marketing / Comparison カテゴリの block 登録点（イシュー #2734 で雛形
//! 新設、イシュー #2823 で最初の block（[`comparison_feature_rows`]）を
//! 追加し卒業。イシュー #2822 で 2 番目の block（[`comparison_cards`]）を
//! 追加した）。本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で
//! 集約する。新規 block を追加する際は本ファイルへ `mod` 宣言と
//! `blocks()` への追記を行うだけでよく、`super`（`marketing`）側・
//! トップレベル `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ
//! 内へ閉じ込めるための構造、イシュー #2734）。

mod comparison_cards;
mod comparison_feature_rows;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![comparison_cards::BLOCK, comparison_feature_rows::BLOCK]
}
