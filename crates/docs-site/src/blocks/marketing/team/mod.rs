//! Marketing / Team カテゴリの block 登録点（イシュー #2734。最初の block
//! 追加〔`team-avatar-grid`、イシュー #2879〕により空雛形からディレクトリ化
//! した、`testimonial/mod.rs` と同型の構造）。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`marketing`）側・トップレベル
//! `crate::blocks` 側の変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込める
//! ための構造、イシュー #2734）。

mod team_avatar_grid;
use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![team_avatar_grid::BLOCK]
}
