//! Application / Table カテゴリの block 登録点（イシュー #2734 で雛形
//! 新設。イシュー #2947 で `table-with-heading`、イシュー #2943 で
//! `table_responsive_stacked` を追加しディレクトリ化して卒業した。
//! イシュー #2949 で `table_with_toolbar` を追加）。
//! 手順は `docs/design/docs-site-blocks-section.md` §18 参照。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod table_responsive_stacked;
mod table_with_heading;
mod table_with_toolbar;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        table_responsive_stacked::BLOCK,
        table_with_heading::BLOCK,
        table_with_toolbar::BLOCK,
    ]
}
