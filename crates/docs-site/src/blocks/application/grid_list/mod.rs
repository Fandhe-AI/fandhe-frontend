//! Application / Grid List カテゴリの block 登録点（イシュー #2734、
//! 最初の block はイシュー #2919）。本カテゴリ配下の block 実装モジュールを
//! 宣言し、[`blocks`] で集約する。新規 block を追加する際は本ファイルへ
//! `mod` 宣言と `blocks()` への追記を行うだけでよく、`super`
//! （`application`）側・トップレベル `crate::blocks` 側の変更は不要
//! （並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造、イシュー #2734）。

mod grid_list_contact_cards;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![grid_list_contact_cards::BLOCK]
}
