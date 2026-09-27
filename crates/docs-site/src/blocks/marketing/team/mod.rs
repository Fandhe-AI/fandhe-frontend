//! Marketing / Team カテゴリの block 登録点。最初の block
//! `team-photo-grid`（イシュー #2881）を追加し、空雛形から卒業した
//! （`docs/design/docs-site-blocks-section.md` §18 参照）。この変更は
//! 本カテゴリ内で完結し、`super`（`marketing`）側の宣言・集約コードは
//! 変更していない（`pub(super) fn blocks()` のシグネチャを維持）。

mod team_photo_grid;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![team_photo_grid::BLOCK]
}
