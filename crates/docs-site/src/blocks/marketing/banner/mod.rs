//! Marketing / Banner カテゴリの block 登録点（イシュー #2734）。
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。
//!
//! イシュー #2740 で本カテゴリ最初の block（`banner_cookie_consent`）を
//! 追加し、空雛形（`banner.rs`）からディレクトリへ卒業した
//! （`docs/design/docs-site-blocks-section.md` §18 の手順）。

mod banner_cookie_consent;
use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![banner_cookie_consent::BLOCK]
}
