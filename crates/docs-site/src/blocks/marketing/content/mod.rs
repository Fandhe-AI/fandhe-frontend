//! Marketing / Content カテゴリの block 登録点。カテゴリ別のモジュール構成と
//! 「カテゴリの卒業」手順は `docs/design/docs-site-blocks-section.md` §18
//! 参照（イシュー #2734）。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル `crate::blocks` 側の
//! 変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造）。
//! block ごとの追加経緯は git 履歴と PR を正とし、本コメントには書かない
//! （並列 PR が同じ行を書き換えて競合するため、§18 参照）。

mod content_article;
mod content_article_toc;
mod content_columns_screenshot;
mod content_image_tiles;
mod content_split_image;
mod content_with_testimonial;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        content_article::BLOCK,
        content_article_toc::BLOCK,
        content_columns_screenshot::BLOCK,
        content_image_tiles::BLOCK,
        content_split_image::BLOCK,
        content_with_testimonial::BLOCK,
    ]
}
