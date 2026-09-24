//! Marketing / Careers カテゴリの block 登録点（イシュー #2734）。
//!
//! 最初の block（[`careers_split_accordion::BLOCK`]）の追加（イシュー
//! #2816）に伴い、雛形 `careers.rs` を本ファイル（`careers/mod.rs`）へ
//! `git mv` してディレクトリ化した（`docs/design/docs-site-blocks-section.md`
//! §18 の手順）。本変更はカテゴリ内で完結し、`super`（`marketing`）側の
//! `mod careers;` 宣言・集約コードは変更不要（`pub(super) fn blocks()` の
//! シグネチャを維持するため）。

mod careers_split_accordion;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![careers_split_accordion::BLOCK]
}
