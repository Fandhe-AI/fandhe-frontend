//! Ecommerce / Product Overview カテゴリの block 登録点（イシュー #2734 で
//! 雛形新設。イシュー #3074（親 #3073、前半）で最初の block
//! （[`product_overview_tabs_below`]）を追加しディレクトリ化して卒業し、
//! 残り領域・状態表示・原案差分メモの仕上げは後半イシュー #3075 で仕上げ済み。
//! イシュー #3068（親 #3067）で 2 件目の block
//! （`product-overview-gallery-split`）を追加した。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`ecommerce`）側・トップレベル `crate::blocks`
//! 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod product_overview_gallery_split;
mod product_overview_tabs_below;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        product_overview_gallery_split::BLOCK,
        product_overview_tabs_below::BLOCK,
    ]
}
