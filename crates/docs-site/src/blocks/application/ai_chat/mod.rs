//! Application / AI Chat カテゴリの block 登録点（イシュー #2734 で雛形
//! 新設、#2961 で最初の block（[`ai_chat_prompt_start`]）を追加して卒業、
//! イシュー #2958（親 #2957）で [`ai_chat_code_preview`] を追加）。
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod ai_chat_code_preview;
mod ai_chat_prompt_start;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![ai_chat_code_preview::BLOCK, ai_chat_prompt_start::BLOCK]
}
