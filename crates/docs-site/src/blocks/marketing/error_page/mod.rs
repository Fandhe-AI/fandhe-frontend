//! Marketing / Error Page カテゴリの block 登録点（イシュー #2734。イシュー
//! #2837 で最初の block（`error_page_centered`）を追加し、`careers/mod.rs`
//! と同型のディレクトリ構成へ「卒業」した。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。新規 block を追加
//! する際は本ファイルへ `mod` 宣言と `blocks()` への追記を行うだけでよく、
//! `super`（`marketing`）側・トップレベル `crate::blocks` 側の変更は不要
//! （並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造、イシュー #2734）。

mod error_page_centered;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![error_page_centered::BLOCK]
}
