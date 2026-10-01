//! Marketing / Testimonial カテゴリの block 登録点。カテゴリ別のモジュール構成と
//! 「カテゴリの卒業」手順は `docs/design/docs-site-blocks-section.md` §18
//! 参照（イシュー #2734）。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル `crate::blocks` 側の
//! 変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造）。
//! block ごとの追加経緯は git 履歴と PR を正とし、本コメントには書かない
//! （並列 PR が同じ行を書き換えて競合するため、§18 参照）。

mod testimonial_background_image;
mod testimonial_card_grid;
mod testimonial_centered_quote;
mod testimonial_masonry_grid;
mod testimonial_quote_stats;
mod testimonial_split_image;
mod testimonial_two_up;
mod testimonials_stack;
use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        testimonial_background_image::BLOCK,
        testimonial_centered_quote::BLOCK,
        testimonial_masonry_grid::BLOCK,
        testimonial_quote_stats::BLOCK,
        testimonials_stack::BLOCK,
        testimonial_card_grid::BLOCK,
        testimonial_split_image::BLOCK,
        testimonial_two_up::BLOCK,
    ]
}
