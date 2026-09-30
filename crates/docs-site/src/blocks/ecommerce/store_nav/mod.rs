//! Ecommerce / Store Nav カテゴリの block 登録点（イシュー #2734、雛形）。
//! イシュー #3094 で最初の block（`store_nav_centered_logo`）を追加し
//! ディレクトリ化して卒業した。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照。この変更は
//! 本カテゴリ内で完結し、`super`（`ecommerce`）側の宣言・集約コードは
//! 変更不要（`pub(super) fn blocks()` のシグネチャを維持するため）。

mod store_nav_centered_logo;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![store_nav_centered_logo::BLOCK]
}
