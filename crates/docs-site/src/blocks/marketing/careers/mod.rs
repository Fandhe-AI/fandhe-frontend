//! Marketing / Careers カテゴリの block 登録点（イシュー #2734 でディレクトリ化。
//! 「カテゴリの卒業」（空雛形 `careers.rs` → `careers/mod.rs`、
//! `docs/design/docs-site-blocks-section.md` §18 参照）はイシュー #2817 の
//! `careers-split-photo-list` 追加に伴い実施した。本ファイルの変更は
//! カテゴリ内で完結し、`super`（`marketing`）側の宣言・集約コードは
//! `pub(super) fn blocks()` のシグネチャが不変のため変更不要。

mod careers_split_photo_list;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![careers_split_photo_list::BLOCK]
}
