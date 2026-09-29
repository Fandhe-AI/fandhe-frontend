//! Application / Action Panel カテゴリの block 登録点（イシュー #2734 で
//! 雛形新設、イシュー #2956 で最初の block（`action-panel-with-well`）を
//! 追加しディレクトリ化して卒業した。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。この変更は
//! 本カテゴリ内で完結し、`super`（`application`）側の宣言・集約コードは
//! 変更不要（`pub(super) fn blocks()` のシグネチャを維持するため）。

mod action_panel_with_well;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![action_panel_with_well::BLOCK]
}
