//! Docs / API Reference カテゴリの block 登録点。カテゴリ別のモジュール
//! 構成と「カテゴリの卒業」手順は `docs/design/docs-site-blocks-section.md`
//! §18 参照（イシュー #2734）。
//!
//! 本カテゴリ配下の block 実装モジュールを宣言し、[`blocks`] で集約する。
//! 新規 block を追加する際は本ファイルへ `mod` 宣言と `blocks()` への追記を
//! 行うだけでよく、`super`（`docs`）側・トップレベル `crate::blocks` 側の
//! 変更は不要（並列 PR 間の衝突をカテゴリ内へ閉じ込めるための構造）。
//! block ごとの追加経緯は git 履歴と PR を正とし、本コメントには書かない。

mod api_reference_playground;

use crate::blocks::Block;

pub(super) fn blocks() -> Vec<Block> {
    vec![api_reference_playground::BLOCK]
}
