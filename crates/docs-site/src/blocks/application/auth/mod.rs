//! Application / Auth カテゴリの block 登録点。カテゴリ別のモジュール構成と
//! 「カテゴリの卒業」手順は `docs/design/docs-site-blocks-section.md` §18
//! 参照（イシュー #2734）。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル `crate::blocks` 側の
//! 変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造）。
//! block ごとの追加経緯は git 履歴と PR を正とし、本コメントには書かない
//! （並列 PR が同じ行を書き換えて競合するため、§18 参照）。

mod auth_dropdown_panel;
mod auth_oauth_consent;
mod auth_otp_verify;
mod auth_split_accent_panel;
mod auth_split_photo_testimonial;
mod auth_tabs_card;
mod login_01;
mod login_04;
mod signup_01;
mod signup_05;
use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        auth_dropdown_panel::BLOCK,
        auth_oauth_consent::BLOCK,
        auth_otp_verify::BLOCK,
        auth_split_accent_panel::BLOCK,
        auth_split_photo_testimonial::BLOCK,
        auth_tabs_card::BLOCK,
        login_01::BLOCK,
        login_04::BLOCK,
        signup_01::BLOCK,
        signup_05::BLOCK,
    ]
}
