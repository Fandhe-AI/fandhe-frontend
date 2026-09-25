//! Marketing / Hero カテゴリの block 登録点（イシュー #2734）。
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。イシュー #2784 で `hero_image_tiles` を
//! 追加し、本カテゴリは 6 件目となった。イシュー #2781/#2782 で
//! `hero_background_media`/`hero_bottom_screenshot` を追加し、本カテゴリは
//! 8 件目となった。

mod hero_background_media;
mod hero_bottom_screenshot;
mod hero_editorial_stagger;
mod hero_email_signup;
mod hero_image_tiles;
mod hero_install_command;
mod hero_parallax_layers;
mod hero_prompt_input;
mod hero_terminal;
mod text_split_reveal;
use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        hero_background_media::BLOCK,
        hero_bottom_screenshot::BLOCK,
        hero_editorial_stagger::BLOCK,
        hero_email_signup::BLOCK,
        hero_image_tiles::BLOCK,
        hero_install_command::BLOCK,
        hero_parallax_layers::BLOCK,
        hero_prompt_input::BLOCK,
        hero_terminal::BLOCK,
        text_split_reveal::BLOCK,
    ]
}
