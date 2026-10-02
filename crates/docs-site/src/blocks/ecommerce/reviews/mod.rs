//! Ecommerce / Reviews カテゴリの block 登録点（イシュー #2734、雛形から
//! ディレクトリ化）。カテゴリ別のモジュール構成と「カテゴリの卒業」手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`ecommerce`）側・トップレベル `crate::blocks`
//! 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造）。
//! block ごとの追加経緯は git 履歴と PR を正とし、本コメントには書かない
//! （`docs/design/docs-site-blocks-section.md` §18 参照）。

mod reviews_card_grid;
mod reviews_stacked_list;
mod reviews_summary_split;
mod reviews_write_form;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        reviews_card_grid::BLOCK,
        reviews_stacked_list::BLOCK,
        reviews_summary_split::BLOCK,
        reviews_write_form::BLOCK,
    ]
}
