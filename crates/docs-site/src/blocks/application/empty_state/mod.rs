//! Application / Empty State カテゴリの block 登録点（イシュー #2734 で
//! 雛形新設、イシュー #2969 で最初の block（`empty-state-card-header`）を、
//! イシュー #2970 で 2 件目（[`empty_state_invite_team`]）を追加した）。
//! イシュー #2972 で 3 件目（[`empty_state_starter_grid`]）を追加した。
//! 手順は `docs/design/docs-site-blocks-section.md` §18 参照。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod empty_state_card_header;
mod empty_state_invite_team;
mod empty_state_starter_grid;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        empty_state_card_header::BLOCK,
        empty_state_invite_team::BLOCK,
        empty_state_starter_grid::BLOCK,
    ]
}
