//! Application / Onboarding カテゴリの block 登録点（イシュー #2980 で
//! `onboarding.rs`（空雛形）からディレクトリ化。手順は
//! `docs/design/docs-site-blocks-section.md` §18 参照）。
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`application`）側・トップレベル
//! `crate::blocks` 側の変更は不要（`auth` カテゴリと同じ構造）。

mod onboarding_split_image;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![onboarding_split_image::BLOCK]
}
