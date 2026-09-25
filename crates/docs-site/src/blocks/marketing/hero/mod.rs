//! Marketing / Hero カテゴリの block 登録点（イシュー #2734）。
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod hero_editorial_stagger;
mod hero_email_signup;
mod hero_parallax_layers;
mod hero_terminal;
mod text_split_reveal;
use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        hero_editorial_stagger::BLOCK,
        hero_email_signup::BLOCK,
        hero_parallax_layers::BLOCK,
        hero_terminal::BLOCK,
        text_split_reveal::BLOCK,
    ]
}
