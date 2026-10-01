//! Marketing / Hero カテゴリの block 登録点。カテゴリ別のモジュール構成と
//! 「カテゴリの卒業」手順は `docs/design/docs-site-blocks-section.md` §18
//! 参照（イシュー #2734）。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル `crate::blocks` 側の
//! 変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造）。
//! block ごとの追加経緯は git 履歴と PR を正とし、本コメントには書かない
//! （並列 PR が同じ行を書き換えて競合するため、§18 参照）。

mod hero_background_media;
mod hero_bottom_screenshot;
mod hero_editorial_stagger;
mod hero_email_signup;
mod hero_image_tiles;
mod hero_image_top;
mod hero_install_command;
mod hero_marquee_strip;
mod hero_parallax_layers;
mod hero_prompt_input;
mod hero_search;
mod hero_social_proof;
mod hero_split_image;
mod hero_split_screenshot;
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
        hero_image_top::BLOCK,
        hero_install_command::BLOCK,
        hero_marquee_strip::BLOCK,
        hero_parallax_layers::BLOCK,
        hero_prompt_input::BLOCK,
        hero_search::BLOCK,
        hero_social_proof::BLOCK,
        hero_split_image::BLOCK,
        hero_split_screenshot::BLOCK,
        hero_terminal::BLOCK,
        text_split_reveal::BLOCK,
    ]
}
