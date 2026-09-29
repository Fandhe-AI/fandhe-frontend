//! Application / AI Chat カテゴリの block 登録点（イシュー #2734 で雛形
//! 新設、#2961 で最初の block（[`ai_chat_prompt_start`]）を追加し卒業）。
//! カテゴリ卒業手順（`docs/design/docs-site-blocks-section.md` §18）に
//! 従い、空雛形の `ai_chat.rs` を本ディレクトリの `mod.rs` へ改名し
//! （`git mv`）、block 実装ファイルを同じディレクトリへ追加した。この
//! 変更は本カテゴリ内で完結し、`super`（`application`）側の宣言・集約
//! コードは変更不要（`pub(super) fn blocks()` のシグネチャを維持する
//! ため）。

mod ai_chat_prompt_start;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![ai_chat_prompt_start::BLOCK]
}
