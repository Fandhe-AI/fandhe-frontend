//! Application / App Shell カテゴリの block 登録点（イシュー #2734）。
//! イシュー #2896 で最初の block（[`app_shell_stacked`]）を追加し、空雛形
//! から卒業した（`docs/design/docs-site-blocks-section.md` §18 参照）。
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod app_shell_stacked;
use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![app_shell_stacked::BLOCK]
}
