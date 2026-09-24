//! Marketing / Careers カテゴリの block 登録点（イシュー #2734 で追加した
//! 空雛形を、「カテゴリの卒業」（空雛形 `careers.rs` → `careers/mod.rs`、
//! `docs/design/docs-site-blocks-section.md` §18 参照）によりイシュー
//! #2817 の `careers-split-photo-list` 追加に伴いディレクトリ化した後、
//! イシュー #2816 の `careers-split-accordion` を追加した）。
//! 本ファイルの変更はカテゴリ内で完結し、`super`（`marketing`）側の
//! 宣言・集約コードは `pub(super) fn blocks()` のシグネチャが不変のため
//! 変更不要。

mod careers_split_accordion;
mod careers_split_photo_list;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        careers_split_accordion::BLOCK,
        careers_split_photo_list::BLOCK,
    ]
}
