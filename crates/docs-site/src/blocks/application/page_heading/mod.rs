//! Application / Page Heading カテゴリの block 登録点（イシュー #2734 で
//! 雛形新設、イシュー #2930 で最初の block（`page-heading-actions`）を
//! 追加しディレクトリ化して卒業した。イシュー #2932 で 2 件目の block
//! （`page-heading-cover`）を追加した。イシュー #2933 で 3 件目の block
//! （`page-heading-meta`）を追加した。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。イシュー #2934 で `page-heading-tabs`
//! （タブ付きのページ見出し）を追加した。

mod page_heading_actions;
mod page_heading_cover;
mod page_heading_meta;
mod page_heading_tabs;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        page_heading_actions::BLOCK,
        page_heading_cover::BLOCK,
        page_heading_meta::BLOCK,
        page_heading_tabs::BLOCK,
    ]
}
