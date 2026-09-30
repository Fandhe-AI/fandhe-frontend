//! Ecommerce / Cart カテゴリの block 登録点（イシュー #2734 で雛形新設、
//! イシュー #3028（親 #3027）で最初の block（[`cart_line_item_table`]）を
//! 追加しディレクトリ化して卒業。イシュー #3033（親 #3032）で
//! [`cart_two_column_summary`] を追加）。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`ecommerce`）側・トップレベル `crate::blocks`
//! 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造、
//! イシュー #2734）。

mod cart_line_item_table;
mod cart_two_column_summary;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![cart_line_item_table::BLOCK, cart_two_column_summary::BLOCK]
}
