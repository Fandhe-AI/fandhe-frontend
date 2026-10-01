//! Marketing / Feature カテゴリの block 登録点。カテゴリ別のモジュール構成と
//! 「カテゴリの卒業」手順は `docs/design/docs-site-blocks-section.md` §18
//! 参照（イシュー #2734）。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル `crate::blocks` 側の
//! 変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造）。
//! block ごとの追加経緯は git 履歴と PR を正とし、本コメントには書かない
//! （並列 PR が同じ行を書き換えて競合するため、§18 参照）。

mod feature_accordion_image;
mod feature_alternating_rows;
mod feature_expand;
mod feature_four_column_grid;
mod feature_image_cards;
mod feature_large_screenshot;
mod feature_side_heading_grid;
mod feature_split_image;
mod feature_split_list_image;
mod feature_split_screenshot;
mod feature_tabs_panel;
mod feature_three_column_icons;
mod feature_vertical_tabs;
use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        feature_accordion_image::BLOCK,
        feature_alternating_rows::BLOCK,
        feature_expand::BLOCK,
        feature_four_column_grid::BLOCK,
        feature_image_cards::BLOCK,
        feature_large_screenshot::BLOCK,
        feature_side_heading_grid::BLOCK,
        feature_split_image::BLOCK,
        feature_split_list_image::BLOCK,
        feature_split_screenshot::BLOCK,
        feature_tabs_panel::BLOCK,
        feature_three_column_icons::BLOCK,
        feature_vertical_tabs::BLOCK,
    ]
}
