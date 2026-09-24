//! Ecommerce 区分の block 登録点（イシュー #2733 の 4 区分の一つ、
//! カテゴリ別モジュール分割はイシュー #2734）。
//!
//! 本区分に属する全カテゴリの `mod` 宣言と [`blocks`]（各カテゴリの
//! `blocks()` を連結するだけの集約関数）を持つ。**新規カテゴリの追加や
//! 既存カテゴリへの block 追加はここを経由しない**（カテゴリ側
//! `<category>.rs`／`<category>/mod.rs` が完結して担う）。本ファイルを
//! 変更するのはカテゴリの「卒業」（空雛形 → ディレクトリ化、
//! `docs/design/docs-site-blocks-section.md` §18 参照）のときのみ。

mod cart;
mod category_listing;
mod checkout;
mod filter;
mod incentives;
mod order;
mod product_list;
mod product_overview;
mod promo;
mod quickview;
mod reviews;
mod store_nav;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    let mut items = Vec::new();
    items.extend(product_overview::blocks());
    items.extend(product_list::blocks());
    items.extend(quickview::blocks());
    items.extend(category_listing::blocks());
    items.extend(filter::blocks());
    items.extend(cart::blocks());
    items.extend(checkout::blocks());
    items.extend(order::blocks());
    items.extend(reviews::blocks());
    items.extend(store_nav::blocks());
    items.extend(incentives::blocks());
    items.extend(promo::blocks());
    items
}
