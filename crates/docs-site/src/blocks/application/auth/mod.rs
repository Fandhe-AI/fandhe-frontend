//! Application / Auth カテゴリの block 登録点（イシュー #2734）。
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod login_01;
mod login_04;
mod signup_01;
mod signup_05;
use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        login_01::BLOCK,
        login_04::BLOCK,
        signup_01::BLOCK,
        signup_05::BLOCK,
    ]
}
