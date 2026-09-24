//! Application / Description List カテゴリの block 登録点（イシュー #2734、雛形）。
//! 最初の block を追加する際は本ファイルを `description_list/mod.rs` へ改名し
//! （`git mv`）、block 実装ファイルを同じディレクトリへ追加した上で
//! `blocks()` を書き換える。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。この変更は
//! 本カテゴリ内で完結し、`super`（`application`）側の宣言・集約コードは
//! 変更不要（`pub(super) fn blocks()` のシグネチャを維持するため）。

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    Vec::new()
}
