//! Marketing / Banner カテゴリの block 登録点（イシュー #2734。イシュー
//! #2740 で最初の block（`banner_cookie_consent`）・イシュー #2741 で
//! 2 番目の block（`banner_email_signup`）・イシュー #2743 で 3 番目の
//! block（`banner_full_width_bar`）を追加し、`footer/mod.rs` と同型の
//! ディレクトリ構成へ「卒業」した。`docs/design/docs-site-blocks-section.md`
//! §18 参照。新規 block を追加する際は本ファイルへ `mod` 宣言と
//! `blocks()` への追記を行うだけでよく、`super`（`marketing`）側・
//! トップレベル `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ
//! 内へ閉じ込めるための構造、イシュー #2734）。

mod banner_cookie_consent;
mod banner_email_signup;
mod banner_floating_card;
mod banner_full_width_bar;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        banner_cookie_consent::BLOCK,
        banner_email_signup::BLOCK,
        banner_floating_card::BLOCK,
        banner_full_width_bar::BLOCK,
    ]
}
