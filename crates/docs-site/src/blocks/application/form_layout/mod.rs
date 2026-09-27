//! Application / Form Layout カテゴリの block 登録点（イシュー #2734）。
//!
//! `form-layout-inline-labels`（イシュー #2911）で本カテゴリを空雛形から
//! 卒業させた（`docs/design/docs-site-blocks-section.md` §18 の手順）。
//! `super`（`application`）側の宣言・集約コードは変更不要（`pub(super)
//! fn blocks()` のシグネチャを維持するため）。

mod form_layout_inline_labels;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![form_layout_inline_labels::BLOCK]
}
