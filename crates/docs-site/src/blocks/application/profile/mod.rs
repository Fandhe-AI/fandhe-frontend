//! Application / Profile カテゴリの block 登録点（イシュー #2936 で
//! カテゴリ卒業。空雛形からディレクトリ化した手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。最初の block は
//! [`profile_card_centered`]）。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod profile_card_centered;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![profile_card_centered::BLOCK]
}
