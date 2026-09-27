//! Application / Form Layout カテゴリの block 登録点（イシュー #2734、
//! 最初の block はイシュー #2916）。本カテゴリ配下の block 実装モジュールを
//! 宣言し、[`blocks`] で集約する。新規 block を追加する際は本ファイルへ
//! `mod` 宣言と `blocks()` への追記を行うだけでよく、`super`
//! （`application`）側・トップレベル `crate::blocks` 側の変更は不要
//! （並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造、イシュー #2734）。
//! `form-layout-inline-labels`（イシュー #2911）・`form-layout-stacked`
//! （イシュー #2915）を追加し、本カテゴリは 3 block 構成となった。

mod form_layout_inline_labels;
mod form_layout_stacked;
mod form_layout_two_column;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        form_layout_inline_labels::BLOCK,
        form_layout_stacked::BLOCK,
        form_layout_two_column::BLOCK,
    ]
}
