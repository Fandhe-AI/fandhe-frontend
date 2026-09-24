//! Marketing 区分の block 登録点（イシュー #2733 の 4 区分の一つ、
//! カテゴリ別モジュール分割はイシュー #2734）。
//!
//! 本区分に属する全カテゴリの `mod` 宣言と [`blocks`]（各カテゴリの
//! `blocks()` を連結するだけの集約関数）を持つ。**新規カテゴリの追加や
//! 既存カテゴリへの block 追加はここを経由しない**（カテゴリ側
//! `<category>.rs`／`<category>/mod.rs` が完結して担う）。本ファイルを
//! 変更するのはカテゴリの「卒業」（空雛形 → ディレクトリ化、
//! `docs/design/docs-site-blocks-section.md` §18 参照）のときのみ。

mod banner;
mod bento;
mod blog;
mod careers;
mod changelog;
mod comparison;
mod contact;
mod content;
mod cta;
mod error_page;
mod faq;
mod feature;
mod footer;
mod gallery;
mod header;
mod hero;
mod logo_cloud;
mod newsletter;
mod pricing;
mod section_heading;
mod stats;
mod team;
mod testimonial;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    let mut items = Vec::new();
    items.extend(hero::blocks());
    items.extend(feature::blocks());
    items.extend(cta::blocks());
    items.extend(pricing::blocks());
    items.extend(testimonial::blocks());
    items.extend(logo_cloud::blocks());
    items.extend(stats::blocks());
    items.extend(team::blocks());
    items.extend(faq::blocks());
    items.extend(contact::blocks());
    items.extend(newsletter::blocks());
    items.extend(blog::blocks());
    items.extend(content::blocks());
    items.extend(header::blocks());
    items.extend(footer::blocks());
    items.extend(banner::blocks());
    items.extend(bento::blocks());
    items.extend(comparison::blocks());
    items.extend(careers::blocks());
    items.extend(changelog::blocks());
    items.extend(error_page::blocks());
    items.extend(gallery::blocks());
    items.extend(section_heading::blocks());
    items
}
