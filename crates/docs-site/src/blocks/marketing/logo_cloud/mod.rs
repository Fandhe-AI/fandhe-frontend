//! Marketing / Logo Cloud カテゴリの block 登録点（イシュー #2734 で雛形
//! 新設。イシュー #2793 で最初の block（[`logo_cloud_grid`]）を追加し、
//! イシュー #2794 で 2 件目（[`logo_cloud_marquee`]）を追加、
//! `logo_cloud.rs` から本ディレクトリへ改名した、
//! `docs/design/docs-site-blocks-section.md` §18 の手順どおりのカテゴリ
//! 卒業）。本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で
//! 集約する。新規 block を追加する際は本ファイルへ `mod` 宣言と
//! `blocks()` への追記を行うだけでよく、`super`（`marketing`）側・
//! トップレベル `crate::blocks` 側の変更は不要（並列 PR 間の衝突を
//! カテゴリ内へ閉じ込めるための構造、イシュー #2734）。

mod logo_cloud_grid;
mod logo_cloud_marquee;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![logo_cloud_grid::BLOCK, logo_cloud_marquee::BLOCK]
}
