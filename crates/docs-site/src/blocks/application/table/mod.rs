//! Application / Table カテゴリの block 登録点。カテゴリ別のモジュール構成と
//! 「カテゴリの卒業」手順は `docs/design/docs-site-blocks-section.md` §18
//! 参照（イシュー #2734）。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル `crate::blocks` 側の
//! 変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造）。
//! block ごとの追加経緯は git 履歴と PR を正とし、本コメントには書かない
//! （並列 PR が同じ行を書き換えて競合するため、§18 参照）。

mod table_grouped_rows;
mod table_responsive_stacked;
mod table_rich_rows;
mod table_sortable_bulk;
mod table_summary_rows;
mod table_with_heading;
mod table_with_toolbar;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        table_grouped_rows::BLOCK,
        table_responsive_stacked::BLOCK,
        table_rich_rows::BLOCK,
        table_sortable_bulk::BLOCK,
        table_summary_rows::BLOCK,
        table_with_heading::BLOCK,
        table_with_toolbar::BLOCK,
    ]
}
