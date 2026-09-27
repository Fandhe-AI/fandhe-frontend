//! Marketing / Team カテゴリの block 登録点（イシュー #2734 で雛形新設、
//! イシュー #2879 で最初の block（`team-avatar-grid`）を追加しディレクトリ化
//! して卒業、イシュー #2880 で `team_bio_rows` を追加した。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod team_avatar_grid;
mod team_bio_rows;
mod team_split_list;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        team_avatar_grid::BLOCK,
        team_bio_rows::BLOCK,
        team_split_list::BLOCK,
    ]
}
