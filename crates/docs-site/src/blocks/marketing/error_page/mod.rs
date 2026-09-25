//! Marketing / Error Page カテゴリの block 登録点（イシュー #2734、
//! #2836 で最初の block を追加して卒業）。
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod error_page_background_image;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![error_page_background_image::BLOCK]
}
