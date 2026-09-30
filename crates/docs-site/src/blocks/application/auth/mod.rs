//! Application / Auth カテゴリの block 登録点（イシュー #2734）。
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。5 件目は `auth_dropdown_panel`（#2962）、
//! 6 件目は `auth_oauth_consent`（#3414）、7 件目は `auth_otp_verify`、
//! 8 件目は `auth_split_accent_panel`（#2965）、9 件目は
//! `auth_tabs_card`（#2967）。

mod auth_dropdown_panel;
mod auth_oauth_consent;
mod auth_otp_verify;
mod auth_split_accent_panel;
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
        auth_tabs_card::BLOCK,
        login_01::BLOCK,
        login_04::BLOCK,
        signup_01::BLOCK,
        signup_05::BLOCK,
    ]
}
