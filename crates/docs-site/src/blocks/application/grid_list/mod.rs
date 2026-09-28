//! Application / Grid List カテゴリの block 登録点（イシュー #2734 で雛形
//! 新設、イシュー #2918 で最初の block（`grid-list-compact-tiles`）を追加し
//! ディレクトリ化して卒業した。イシュー #2920（親トラッキング未確定）で
//! 2 件目の block（[`grid_list_file_thumbnails`]）を、イシュー #2921
//! （親 #2892）で 3 件目の block（[`grid_list_logo_cards`]）を、イシュー
//! #2917 で 4 件目の block（[`grid_list_action_tiles`]）を追加した。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod grid_list_action_tiles;
mod grid_list_compact_tiles;
mod grid_list_contact_cards;
mod grid_list_file_thumbnails;
mod grid_list_logo_cards;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        grid_list_compact_tiles::BLOCK,
        grid_list_contact_cards::BLOCK,
        grid_list_file_thumbnails::BLOCK,
        grid_list_logo_cards::BLOCK,
        grid_list_action_tiles::BLOCK,
    ]
}
