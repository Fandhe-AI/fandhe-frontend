//! Application / Description List カテゴリの block 登録点（イシュー
//! #2734 で雛形、#2908 で最初の block を追加）。
//!
//! 新しい block を本カテゴリへ追加する場合は、実装ファイルを本ディレクトリ
//! へ追加した上で `mod` 宣言と [`blocks`] の連結先を増やす（`super`
//! `application` 側の宣言・集約コードは変更不要、
//! `pub(super) fn blocks()` のシグネチャを維持するため）。

mod description_list_two_column;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![description_list_two_column::BLOCK]
}
