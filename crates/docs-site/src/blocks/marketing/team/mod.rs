//! Marketing / Team カテゴリの block 登録点。カテゴリ別のモジュール構成と
//! 「カテゴリの卒業」手順は `docs/design/docs-site-blocks-section.md` §18
//! 参照（イシュー #2734）。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル `crate::blocks` 側の
//! 変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造）。
//! block ごとの追加経緯は git 履歴と PR を正とし、本コメントには書かない
//! （並列 PR が同じ行を書き換えて競合するため、§18 参照）。

mod team_avatar_grid;
mod team_bio_rows;
mod team_photo_grid;
mod team_split_list;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        team_avatar_grid::BLOCK,
        team_bio_rows::BLOCK,
        team_photo_grid::BLOCK,
        team_split_list::BLOCK,
    ]
}
