//! Marketing / Logo Cloud カテゴリの block 登録点（イシュー #2734 で雛形
//! 新設。イシュー #2793 で最初の block（[`logo_cloud_grid`]）を追加）。
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への
//! 追記を行うだけでよく、`super`（`marketing`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ
//! 閉じ込めるための構造、イシュー #2734）。

mod logo_cloud_grid;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![logo_cloud_grid::BLOCK]
}
