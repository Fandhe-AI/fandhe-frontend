//! Application / Onboarding カテゴリの block 登録点（イシュー #2734 で雛形
//! 新設、イシュー #2979 で最初の block（[`onboarding_checklist`]）・イシュー
//! #2978 で 2 件目（[`onboarding_centered_steps`]）を追加しディレクトリ化して
//! 卒業、イシュー #2981（親 #2951）で [`onboarding_vertical_steps`]・イシュー
//! #2980 で [`onboarding_split_image`] を追加した）。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod onboarding_centered_steps;
mod onboarding_checklist;
mod onboarding_split_image;
mod onboarding_vertical_steps;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        onboarding_checklist::BLOCK,
        onboarding_centered_steps::BLOCK,
        onboarding_vertical_steps::BLOCK,
        onboarding_split_image::BLOCK,
    ]
}
