//! Application / Feed カテゴリの block 登録点（イシュー #2909 で最初の
//! block（[`feed_comments_timeline`]）を追加し卒業、雛形はイシュー
//! #2734）。本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`]
//! で集約する。新規 block を追加する際は本ファイルへ `mod` 宣言と
//! `blocks()` への追記を行うだけでよく、`super`（`application`）側・
//! トップレベル `crate::blocks` 側の変更は不要（並列 PR 間の衝突を
//! カテゴリ内へ閉じ込めるための構造、イシュー #2734）。

mod feed_comments_timeline;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![feed_comments_timeline::BLOCK]
}
