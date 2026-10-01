//! Marketing / Changelog カテゴリの block 登録点。カテゴリ別のモジュール構成と
//! 「カテゴリの卒業」手順は `docs/design/docs-site-blocks-section.md` §18
//! 参照（イシュー #2734）。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル `crate::blocks` 側の
//! 変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造）。
//! block ごとの追加経緯は git 履歴と PR を正とし、本コメントには書かない
//! （並列 PR が同じ行を書き換えて競合するため、§18 参照）。

mod changelog_accordion;
mod changelog_stacked_list;
mod changelog_timeline;
mod changelog_timeline_subscribe;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        changelog_accordion::BLOCK,
        changelog_stacked_list::BLOCK,
        changelog_timeline::BLOCK,
        changelog_timeline_subscribe::BLOCK,
    ]
}
