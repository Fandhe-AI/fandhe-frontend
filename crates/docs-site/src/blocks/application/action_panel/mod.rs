//! 雛形新設、イシュー #2952 で最初の block（`action-panel-footer-bar`）を
//! 追加しディレクトリ化して卒業した後、イシュー #2956 で
//! `action-panel-with-well` を追加した）。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod action_panel_footer_bar;
mod action_panel_with_well;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        action_panel_footer_bar::BLOCK,
        action_panel_with_well::BLOCK,
    ]
}
