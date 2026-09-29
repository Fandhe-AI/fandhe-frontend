//! Application / Media Object カテゴリの block 登録点（イシュー #3228 で
//! カテゴリ新設と同時に最初の block（`media-object`）を追加し、最初から
//! ディレクトリ化して新設した。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod media_object_alignments;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![media_object_alignments::BLOCK]
}
