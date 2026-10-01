//! Marketing / Footer カテゴリの block 登録点。カテゴリ別のモジュール構成と
//! 「カテゴリの卒業」手順は `docs/design/docs-site-blocks-section.md` §18
//! 参照（イシュー #2734）。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル `crate::blocks` 側の
//! 変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造）。
//! block ごとの追加経緯は git 履歴と PR を正とし、本コメントには書かない
//! （並列 PR が同じ行を書き換えて競合するため、§18 参照）。

mod footer_cta_columns;
mod footer_inline_nav;
mod footer_link_columns;
mod footer_newsletter;
mod footer_newsletter_band;
mod footer_sticky_reveal;
use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        footer_cta_columns::BLOCK,
        footer_inline_nav::BLOCK,
        footer_link_columns::BLOCK,
        footer_newsletter::BLOCK,
        footer_newsletter_band::BLOCK,
        footer_sticky_reveal::BLOCK,
    ]
}
