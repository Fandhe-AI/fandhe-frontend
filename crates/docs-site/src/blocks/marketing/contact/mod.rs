//! Marketing / Contact カテゴリの block 登録点（イシュー #2734 で雛形新設、
//! イシュー #2827 で最初の block（`contact_dialog_form`）・イシュー #2829 で
//! 2 件目の block（`contact_image_info`）を追加しディレクトリ化して卒業
//! した）。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。
//!
//! イシュー #2827 で最初の block（`contact_dialog_form`）を追加し、
//! 本ファイルは空雛形（`Vec::new()` を返すだけ）から卒業した
//! （`docs/design/docs-site-blocks-section.md` §18 の卒業手順）。イシュー
//! #2829 で 2 件目の block（`contact_image_info`）、イシュー #2830 で
//! 3 件目の block（`contact_info_columns`）、イシュー #2828 で 4 件目の
//! block（[`contact_form_testimonial`]）、イシュー #2832 で 5 件目の
//! block（`contact_split_form_info`）、イシュー #2835 で 6 件目の
//! block（[`contact_split_info`]）を追加した。

mod contact_dialog_form;
mod contact_form_testimonial;
mod contact_image_info;
mod contact_info_columns;
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
        contact_split_info::BLOCK,
    ]
}
