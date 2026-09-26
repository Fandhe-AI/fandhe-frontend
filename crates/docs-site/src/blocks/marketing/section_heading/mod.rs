//! Marketing / Section Heading カテゴリの block 登録点（イシュー #2734 で
//! 雛形新設、イシュー #2798 で最初の block（`section_heading_split`）・
//! イシュー #2799 で 2 件目（`section_heading_stacked`）を追加し
//! ディレクトリ化して卒業した、`docs/design/docs-site-blocks-section.md`
//! §18 の卒業手順）。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル `crate::blocks`
//! 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造、
//! イシュー #2734）。

mod section_heading_split;
mod section_heading_stacked;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![section_heading_split::BLOCK, section_heading_stacked::BLOCK]
}
