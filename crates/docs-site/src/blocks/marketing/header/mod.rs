//! Marketing / Header カテゴリの block 登録点（イシュー #2734 で雛形新設、
//! イシュー #2853 で最初の block（`header_floating_pill`）を追加し
//! ディレクトリ化して卒業した。`docs/design/docs-site-blocks-section.md`
//! §18 の卒業手順に従う。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod header_floating_pill;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![header_floating_pill::BLOCK]
}
