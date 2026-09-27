//! Application / Form Layout カテゴリの block 登録点（イシュー #2734）。
//!
//! `form-layout-stacked`（イシュー #2915）で本カテゴリを空雛形から卒業させた
//! （`docs/design/docs-site-blocks-section.md` §18 の手順）。`super`
//! （`application`）側の宣言・集約コードは変更不要（`pub(super) fn blocks()`
//! のシグネチャを維持するため）。

mod form_layout_stacked;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![form_layout_stacked::BLOCK]
}
