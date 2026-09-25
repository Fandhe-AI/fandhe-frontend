//! Marketing / Contact カテゴリの block 登録点（イシュー #2734 で雛形新設、
//! イシュー #2829 で最初の block（`contact-image-info`）を追加し
//! ディレクトリ化して卒業した）。
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod contact_image_info;
use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![contact_image_info::BLOCK]
}
