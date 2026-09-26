//! Marketing / Newsletter カテゴリの block 登録点（イシュー #2734 で雛形
//! 新設、イシュー #2796 で最初の block（`newsletter_split`）・イシュー
//! #2797 で 2 件目の block（`newsletter_with_details`）を追加しディレクトリ化
//! して卒業した。イシュー #3226 で 3 件目の block（`newsletter_stacked`）を
//! 追加した。`docs/design/docs-site-blocks-section.md` §18 の卒業手順に
//! 従う）。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod newsletter_split;
mod newsletter_stacked;
mod newsletter_with_details;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        newsletter_split::BLOCK,
        newsletter_stacked::BLOCK,
        newsletter_with_details::BLOCK,
    ]
}
