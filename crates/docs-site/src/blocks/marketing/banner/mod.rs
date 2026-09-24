//! Marketing / Banner カテゴリの block 登録点（イシュー #2734。イシュー
//! #2741 で最初の block（`banner_email_signup`）を追加し、`footer/mod.rs`
//! と同型のディレクトリ構成へ「卒業」した。`docs/design/docs-site-blocks-section.md`
//! §18 参照。新規 block を追加する際は本ファイルへ `mod` 宣言と
//! `blocks()` への追記を行うだけでよく、`super`（`marketing`）側・
//! トップレベル `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ
//! 内へ閉じ込めるための構造、イシュー #2734）。

mod banner_announcement_pill;
mod banner_email_signup;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![banner_announcement_pill::BLOCK, banner_email_signup::BLOCK]
}
