//! Docs 区分の block 登録点（イシュー #2733 の 4 区分の一つ、
//! カテゴリ別モジュール分割はイシュー #2734）。
//!
//! 本区分に属する全カテゴリの `mod` 宣言と [`blocks`]（各カテゴリの
//! `blocks()` を連結するだけの集約関数）を持つ。**新規カテゴリの追加や
//! 既存カテゴリへの block 追加はここを経由しない**（カテゴリ側
//! `<category>.rs`／`<category>/mod.rs` が完結して担う）。本ファイルを
//! 変更するのはカテゴリの「卒業」（空雛形 → ディレクトリ化、
//! `docs/design/docs-site-blocks-section.md` §18 参照）のときのみ。

mod api_reference;
mod code_block;
mod docs_layout;
mod example_preview;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    let mut items = Vec::new();
    items.extend(docs_layout::blocks());
    items.extend(code_block::blocks());
    items.extend(example_preview::blocks());
    items.extend(api_reference::blocks());
    items
}
