//! keyed list の DOM 適用: 純粋 diff 層（イシュー #345 / #1324）。
//!
//! 当初（イシュー #345）は本モジュールが `KeyedOp`（Remove/Insert/Move の
//! 3 variant）・`diff_keys`（2 パス方式）を独自実装していたが、イシュー
//! #1323 で `fandhe_frontend_core::keyed` へ内容比較付き diff
//! （`KeyedOp::Update` / `diff_keyed_items`）が実装され、`diff_keys` の
//! アルゴリズムも core 側へ移設された（`diff_keyed_items` はパス 1・2 を
//! `diff_keys` と共有する）。本モジュールは重複実装を避けるため、イシュー
//! #1324 で core 側の型・関数をそのまま re-export する形へ置換した
//! （`docs/design/keyed-update-op-design.md` §4.2 が確定する責務分担:
//! op 生成〔diff・内容比較〕= core、DOM 適用 = wasm-client）。
//!
//! 実 DOM への適用（要素の生成・`insert_before`・削除・Update 時の属性/
//! 子ノード同期）は `wasm32` 配線層 [`crate::keyed_dom`] が
//! [`crate::keyed_apply`] 越しに本モジュールの型を消費して行う。

pub use fandhe_frontend_core::keyed::{diff_keyed_items, diff_keys, KeyedOp};
