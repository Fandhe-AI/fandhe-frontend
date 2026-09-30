//! Application / Action Panel カテゴリの block 登録点（イシュー #2734 で
//! 雛形新設、イシュー #2952 で `action-panel-footer-bar`、イシュー #2953 で
//! `action-panel-inline`、イシュー #2955 で `action-panel-with-input`、
//! イシュー #2954 で `action-panel-stacked`、イシュー #2956 で
//! `action-panel-with-well` を
//! 追加しディレクトリ化して卒業した）。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod action_panel_footer_bar;
mod action_panel_inline;
mod action_panel_stacked;
mod action_panel_with_input;
mod action_panel_with_well;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        action_panel_footer_bar::BLOCK,
        action_panel_inline::BLOCK,
        action_panel_with_input::BLOCK,
        action_panel_stacked::BLOCK,
        action_panel_with_well::BLOCK,
    ]
}
