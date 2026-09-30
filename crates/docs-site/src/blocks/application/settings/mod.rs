//! Application / Settings カテゴリの block 登録点（イシュー #2734 で雛形新設、
//! イシュー #2984 で最初の block（[`settings_billing_overview`]）、
//! イシュー #2985 で 2 件目の block（[`settings_billing_usage`]）を追加し
//! ディレクトリ化して卒業した）。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod settings_billing_overview;
mod settings_billing_usage;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        settings_billing_overview::BLOCK,
        settings_billing_usage::BLOCK,
    ]
}
