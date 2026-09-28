//! Application / Navbar カテゴリの block 登録点（イシュー #2734 で雛形
//! 新設。イシュー #2926（親 #2892）で最初の block（[`navbar_app_links`]）
//! を追加しディレクトリ化して卒業した。イシュー #2928 で 2 件目の
//! block（[`navbar_two_row`]）、さらに 3 件目の block
//! （[`navbar_with_search`]）を追加。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod navbar_app_links;
mod navbar_two_row;
mod navbar_with_search;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        navbar_app_links::BLOCK,
        navbar_two_row::BLOCK,
        navbar_with_search::BLOCK,
    ]
}
