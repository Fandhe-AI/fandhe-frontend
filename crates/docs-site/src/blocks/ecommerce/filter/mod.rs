//! Ecommerce / Filter カテゴリの block 登録点（イシュー #2734 の手順で
//! 雛形を卒業済み）。新規 block を追加する際は本ファイルへ `mod` 宣言と
//! [`blocks`] への追記を行うだけでよく、`super`（`ecommerce`）側・
//! トップレベル `crate::blocks` 側の変更は不要（
//! `docs/design/docs-site-blocks-section.md` §18 参照）。block ごとの
//! 追加経緯は git 履歴と PR を正とし、本コメントには書かない。

mod filter_dropdown_bar;
mod filter_expandable_panel;
mod filter_sidebar;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        filter_dropdown_bar::BLOCK,
        filter_expandable_panel::BLOCK,
        filter_sidebar::BLOCK,
    ]
}
