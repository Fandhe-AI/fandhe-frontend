//! Application / Profile カテゴリの block 登録点（イシュー #2734 で雛形
//! 新設、イシュー #2937 で最初の block（[`profile_detail_datalist`]）を
//! 追加しディレクトリ化して卒業、イシュー #2936 で
//! [`profile_card_centered`] を追加）。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod profile_card_centered;
mod profile_detail_datalist;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![profile_card_centered::BLOCK, profile_detail_datalist::BLOCK]
}
