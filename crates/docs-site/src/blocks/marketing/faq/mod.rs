//! Marketing / Faq カテゴリの block 登録点（イシュー #2734 で雛形新設、
//! イシュー #2843 で最初の block（`faq_accordion_centered`）を追加し
//! ディレクトリ化して卒業した。`docs/design/docs-site-blocks-section.md`
//! §18 の卒業手順に従う。イシュー #2845 で 2 件目（`faq_split_accordion`）
//! を、イシュー #2846 で 3 件目（`faq_split_static`）を追加。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod faq_accordion_centered;
mod faq_split_accordion;
mod faq_split_static;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        faq_accordion_centered::BLOCK,
        faq_split_accordion::BLOCK,
        faq_split_static::BLOCK,
    ]
}
