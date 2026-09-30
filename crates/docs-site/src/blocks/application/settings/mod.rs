//! Application / Settings カテゴリの block 登録点（イシュー #2734 で雛形新設、
//! イシュー #2982 で最初の block（[`settings_api_key_created`]）を追加し
//! ディレクトリ化して卒業した）。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod settings_api_key_created;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![settings_api_key_created::BLOCK]
}
