//! Marketing / Contact カテゴリの block 登録点。カテゴリ別のモジュール構成と
//! 「カテゴリの卒業」手順は `docs/design/docs-site-blocks-section.md` §18
//! 参照（イシュー #2734）。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル `crate::blocks` 側の
//! 変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造）。
//! block ごとの追加経緯は git 履歴と PR を正とし、本コメントには書かない
//! （並列 PR が同じ行を書き換えて競合するため、§18 参照）。

mod contact_centered_form;
mod contact_dialog_form;
mod contact_form_testimonial;
mod contact_image_info;
mod contact_info_columns;
mod contact_split_form_image;
mod contact_split_form_info;
mod contact_split_info;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![
        contact_dialog_form::BLOCK,
        contact_image_info::BLOCK,
        contact_info_columns::BLOCK,
        contact_form_testimonial::BLOCK,
        contact_split_form_info::BLOCK,
        contact_split_form_image::BLOCK,
        contact_centered_form::BLOCK,
        contact_split_info::BLOCK,
    ]
}
