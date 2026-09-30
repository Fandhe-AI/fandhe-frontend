//! Ecommerce / Promo カテゴリの block 登録点（イシュー #2734 で雛形新設、
//! イシュー #3078 で最初の block（[`promo_collection_cards`]）を追加し
//! ディレクトリ化して卒業した。Ecommerce 区分で最初にディレクトリ化した
//! カテゴリでもある）。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`ecommerce`）側・トップレベル `crate::blocks`
//! 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造、
//! イシュー #2734）。

mod promo_collection_cards;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![promo_collection_cards::BLOCK]
}
