//! Application / Form Layout カテゴリの block 登録点（イシュー #2734 で
//! 雛形新設。イシュー #2913（親 #2912）で最初の block
//! （[`form_layout_property_panel`]）を追加しディレクトリ化して卒業し、
//! イシュー #2916（親 #2892）で [`form_layout_two_column`]、イシュー #2915
//! で [`form_layout_stacked`] を追加した）。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod form_layout_inline_labels;
mod form_layout_property_panel;
mod form_layout_stacked;
mod form_layout_two_column;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        form_layout_property_panel::BLOCK,
        form_layout_inline_labels::BLOCK,
        form_layout_stacked::BLOCK,
        form_layout_two_column::BLOCK,
    ]
}
