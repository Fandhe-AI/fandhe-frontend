//! keyed list の DOM 適用: 純粋 diff 層（イシュー #345）。
//!
//! `docs/design/dom-binding-update-design.md` §5 が定める「リストの構造変化
//! （挿入・削除・並べ替え）を表現できる唯一の経路」である
//! `fandhe_frontend_core::keyed::keyed_list`（#344）が出力する `data-key` 列に対し、
//! 「現在の DOM 上のキー列」と「新しい `Node` 木のキー列」の 2 つの `&str`
//! 列だけから最小の操作列（削除・挿入・移動）を計画する。DOM
//! （`web-sys`）に一切依存しないため、`cargo test -p fandhe-frontend-wasm-client`
//! （native）で検証できる（`crate::binding`/`crate::binding_dom` と同じ
//! 2 層構成方針）。
//!
//! 実 DOM への適用（要素の生成・`insert_before`・削除）は `wasm32` 配線層
//! [`crate::keyed_dom`] が本モジュールの型を消費して行う。

/// keyed list へ適用する 1 操作（設計書 §5.3）。
///
/// `index` は操作適用後の「新しい並び」における位置を指す
/// （挿入・移動先の決定的な位置決め、`wasm32` 配線層が `insert_before` の
/// 参照ノードを求める際の入力）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyedOp {
    /// 当該キーの既存ノードを削除する。
    Remove {
        /// 削除対象のキー。
        key: String,
    },
    /// 当該キーに対応する新規ノードを `index` の位置へ挿入する
    /// （旧キー列に存在しなかったキー）。
    Insert {
        /// 挿入先の位置（新しい並びでのインデックス）。
        index: usize,
        /// 挿入するキー。
        key: String,
    },
    /// 当該キーに対応する既存ノードを `index` の位置へ移動する（旧キー列に
    /// 存在したが位置が変わったキー）。既存 DOM ノードを再生成せず参照を
    /// 保持したまま移動することがフォーカス・入力途中の値の保持に直結する
    /// ため、`Remove` + `Insert` へ分解しない専用の操作として区別する。
    Move {
        /// 移動先の位置（新しい並びでのインデックス）。
        index: usize,
        /// 移動するキー。
        key: String,
    },
}

/// 「現在のキー列」から「新しいキー列」へ変換する最小の操作列を計画する。
///
/// アルゴリズム（O(n)、汎用 diff・仮想 DOM ではなく keyed list 専用の単純な
/// 2 パス方式、設計書 §5・§7 が確定する「唯一の経路」の実装）:
///
/// 1. `old_keys` を先頭から走査し、`new_keys` に存在しないキーは
///    [`KeyedOp::Remove`] として記録し、作業列（`working`）から除外する。
/// 2. `new_keys` を先頭から走査し、`working[i]` が期待するキーと一致しなけ
///    れば、`working` の残り（`i` 以降）から探して見つかれば
///    [`KeyedOp::Move`]、見つからなければ [`KeyedOp::Insert`] とする。
///    いずれの場合も `working` を新しい並びに合わせて更新してから次へ進む。
///
/// キー重複がある場合（本来は [`fandhe_frontend_core::keyed::keyed_list`] が構築時点で
/// 拒否するため到達しない想定だが、DOM 改ざん等で `old_keys` 側に重複が
/// 混入した場合の防御）は、`working` 側の探索を「未処理の先頭要素」に
/// 限定することで最初の 1 件のみを対象とし、無限ループ・panic を起こさない
/// （fail-closed）。
pub fn diff_keys(old_keys: &[String], new_keys: &[String]) -> Vec<KeyedOp> {
    let new_set: std::collections::HashSet<&str> = new_keys.iter().map(String::as_str).collect();

    let mut working: Vec<String> = Vec::with_capacity(old_keys.len());
    let mut ops: Vec<KeyedOp> = Vec::new();

    for key in old_keys {
        if new_set.contains(key.as_str()) {
            working.push(key.clone());
        } else {
            ops.push(KeyedOp::Remove { key: key.clone() });
        }
    }

    for (index, key) in new_keys.iter().enumerate() {
        if working.get(index).map(String::as_str) == Some(key.as_str()) {
            continue;
        }
        if let Some(offset) = working[index..].iter().position(|k| k == key) {
            let actual = index + offset;
            let moved = working.remove(actual);
            working.insert(index, moved);
            ops.push(KeyedOp::Move {
                index,
                key: key.clone(),
            });
        } else {
            working.insert(index, key.clone());
            ops.push(KeyedOp::Insert {
                index,
                key: key.clone(),
            });
        }
    }

    ops
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    /// 変化なし: 操作列は空。
    #[test]
    fn diff_keys_returns_empty_ops_when_unchanged() {
        let old = keys(&["a", "b", "c"]);
        let new = keys(&["a", "b", "c"]);
        assert_eq!(diff_keys(&old, &new), Vec::new());
    }

    /// 末尾への追加は Insert 1 件のみ。
    #[test]
    fn diff_keys_detects_append_at_tail() {
        let old = keys(&["a", "b"]);
        let new = keys(&["a", "b", "c"]);
        assert_eq!(
            diff_keys(&old, &new),
            vec![KeyedOp::Insert {
                index: 2,
                key: "c".to_string()
            }]
        );
    }

    /// 先頭への追加は Insert 1 件のみ。
    #[test]
    fn diff_keys_detects_prepend_at_head() {
        let old = keys(&["b", "c"]);
        let new = keys(&["a", "b", "c"]);
        assert_eq!(
            diff_keys(&old, &new),
            vec![KeyedOp::Insert {
                index: 0,
                key: "a".to_string()
            }]
        );
    }

    /// 中間削除は Remove 1 件のみ（受け入れ条件: 無関係ノードへの操作なし）。
    #[test]
    fn diff_keys_detects_middle_removal() {
        let old = keys(&["a", "b", "c"]);
        let new = keys(&["a", "c"]);
        assert_eq!(
            diff_keys(&old, &new),
            vec![KeyedOp::Remove {
                key: "b".to_string()
            }]
        );
    }

    /// 全削除は各キーの Remove のみ。
    #[test]
    fn diff_keys_detects_removal_to_empty() {
        let old = keys(&["a", "b"]);
        let new: Vec<String> = Vec::new();
        assert_eq!(
            diff_keys(&old, &new),
            vec![
                KeyedOp::Remove {
                    key: "a".to_string()
                },
                KeyedOp::Remove {
                    key: "b".to_string()
                },
            ]
        );
    }

    /// 空から全追加は各キーの Insert のみ。
    #[test]
    fn diff_keys_detects_insertion_from_empty() {
        let old: Vec<String> = Vec::new();
        let new = keys(&["a", "b"]);
        assert_eq!(
            diff_keys(&old, &new),
            vec![
                KeyedOp::Insert {
                    index: 0,
                    key: "a".to_string()
                },
                KeyedOp::Insert {
                    index: 1,
                    key: "b".to_string()
                },
            ]
        );
    }

    /// 隣接 2 件の入れ替えは Move 1 件で表現される（既存ノード参照を保持し
    /// 再生成しないことがフォーカス保持の土台、設計書 §5.3）。
    #[test]
    fn diff_keys_detects_adjacent_swap_as_single_move() {
        let old = keys(&["a", "b"]);
        let new = keys(&["b", "a"]);
        let ops = diff_keys(&old, &new);
        assert_eq!(
            ops,
            vec![KeyedOp::Move {
                index: 0,
                key: "b".to_string()
            }]
        );
    }

    /// 末尾要素を先頭へ移動。
    #[test]
    fn diff_keys_detects_move_from_tail_to_head() {
        let old = keys(&["a", "b", "c"]);
        let new = keys(&["c", "a", "b"]);
        let ops = diff_keys(&old, &new);
        assert_eq!(
            ops,
            vec![KeyedOp::Move {
                index: 0,
                key: "c".to_string()
            }]
        );
    }

    /// 削除・挿入・移動が同時に起きる複合ケースでも、無関係キーの操作は
    /// 生成されない。
    #[test]
    fn diff_keys_handles_mixed_remove_insert_move() {
        let old = keys(&["a", "b", "c", "d"]);
        let new = keys(&["d", "a", "e", "c"]);
        let ops = diff_keys(&old, &new);
        // "b" は new に存在しないため削除される。
        assert!(ops.contains(&KeyedOp::Remove {
            key: "b".to_string()
        }));
        // "e" は old に存在しない新規キーのため挿入される。
        assert!(ops
            .iter()
            .any(|op| matches!(op, KeyedOp::Insert { key, .. } if key == "e")));
        // "d" は末尾から先頭へ移動する。
        assert!(ops
            .iter()
            .any(|op| matches!(op, KeyedOp::Move { key, .. } if key == "d")));
    }

    /// 重複キーが混入していても panic せず、先頭の 1 件のみを対象に処理する
    /// （DOM 改ざん等の異常系に対する fail-closed 防御、本モジュール doc 参照）。
    #[test]
    fn diff_keys_does_not_panic_on_duplicate_keys_in_old() {
        let old = keys(&["a", "a", "b"]);
        let new = keys(&["a", "b"]);
        // panic しないことのみを確認する（重複時の正確な操作列は未規定）。
        let _ = diff_keys(&old, &new);
    }

    // --- イシュー #1318: O(n²) 再発検知の一環として、大規模ケースの op 数
    // 自体を契約化する（ルート issue #1313 が特定した CSR create の旧
    // O(n²) コストは挿入位置解決（旧 `nth_element_child`、イシュー #1319 で
    // `KeyedListDom::child_at` の O(1) 参照へ置換済み）の sibling 走査回数に
    // 起因し `diff_keys` 自体は元々 O(n) だが、`diff_keys` が発行する op 数
    // が想定外に膨らむと `keyed_apply::apply_ops` 側の DOM 操作コストも
    // 連動して膨らむため、まず purely な diff 層で op 数の上限を固定して
    // おく）。

    /// 空 → N 行（N=1,000）: Insert がちょうど N 件、index は 0..N の昇順で
    /// 1 つも欠けない（余分な Remove/Move が発生しないこと）。
    #[test]
    fn diff_keys_from_empty_to_n_rows_emits_exactly_n_inserts_in_order() {
        const N: usize = 1_000;
        let old: Vec<String> = Vec::new();
        let new: Vec<String> = (0..N).map(|i| format!("k{i}")).collect();

        let ops = diff_keys(&old, &new);

        assert_eq!(ops.len(), N, "空 → N 行は Insert ちょうど N 件のみのはず");
        for (i, op) in ops.iter().enumerate() {
            assert_eq!(
                op,
                &KeyedOp::Insert {
                    index: i,
                    key: format!("k{i}"),
                },
                "Insert の index は 0..N の昇順で欠けないはず"
            );
        }
    }

    /// 既存 N 行の先頭へ 1 件挿入: op はちょうど 1 件（Insert index=0）の
    /// みで、既存 N 件への無関係な Remove/Move は発生しない。
    #[test]
    fn diff_keys_prepend_one_to_n_rows_emits_exactly_one_insert() {
        const N: usize = 1_000;
        let old: Vec<String> = (0..N).map(|i| format!("k{i}")).collect();
        let mut new: Vec<String> = vec!["new".to_string()];
        new.extend(old.iter().cloned());

        let ops = diff_keys(&old, &new);

        assert_eq!(
            ops,
            vec![KeyedOp::Insert {
                index: 0,
                key: "new".to_string(),
            }]
        );
    }

    /// 既存 N 行の末尾へ 1 件挿入: op はちょうど 1 件（Insert index=N）の
    /// みで、既存 N 件への無関係な Remove/Move は発生しない。
    #[test]
    fn diff_keys_append_one_to_n_rows_emits_exactly_one_insert() {
        const N: usize = 1_000;
        let old: Vec<String> = (0..N).map(|i| format!("k{i}")).collect();
        let mut new: Vec<String> = old.clone();
        new.push("new".to_string());

        let ops = diff_keys(&old, &new);

        assert_eq!(
            ops,
            vec![KeyedOp::Insert {
                index: N,
                key: "new".to_string(),
            }]
        );
    }

    /// 完全逆順（reverse）: 全件が同じ集合のまま並びだけが反転するケース。
    /// Remove/Insert は 1 件も発生せず、Move が高々 N-1 件で収まる
    /// （全 N 件が Move になるとは限らない実装だが、上限として N-1 を
    /// 固定し「N 件超の異常な op 数」の再発を検知する）。
    #[test]
    fn diff_keys_full_reverse_emits_only_moves_within_n_minus_one() {
        const N: usize = 1_000;
        let old: Vec<String> = (0..N).map(|i| format!("k{i}")).collect();
        let new: Vec<String> = old.iter().rev().cloned().collect();

        let ops = diff_keys(&old, &new);

        assert!(
            ops.iter().all(|op| matches!(op, KeyedOp::Move { .. })),
            "完全逆順は Remove/Insert を発生させず Move のみのはず"
        );
        assert!(
            ops.len() < N,
            "Move の件数は N-1 件以内のはず（実測: {}）",
            ops.len()
        );
    }

    /// 固定シード LCG（線形合同法）による決定的シャッフル
    /// （Fisher–Yates）。乱数・時刻・環境に依存する `rand` クレート等は
    /// 使わず標準ライブラリのみで完結させる（REQ-3: 外部依存追加ゼロ）。
    fn lcg_shuffle(items: &mut [String], seed: u64) {
        let mut state = seed;
        let mut next = move || {
            // Numerical Recipes の定数（決定的であれば具体的な定数の由来は
            // 問わない用途。テストの再現性のみが目的）。
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            state
        };
        for i in (1..items.len()).rev() {
            let j = (next() % (i as u64 + 1)) as usize;
            items.swap(i, j);
        }
    }

    /// 固定シード LCG によるシャッフル（N=1,000）: 集合は変化しないため
    /// Remove/Insert は発生せず、Move の件数は N-1 件を超えない
    /// （O(n²) 再発検知: op 数自体が線形の上限に収まることを固定する）。
    #[test]
    fn diff_keys_deterministic_shuffle_emits_only_moves_within_n_minus_one() {
        const N: usize = 1_000;
        let old: Vec<String> = (0..N).map(|i| format!("k{i}")).collect();
        let mut new = old.clone();
        lcg_shuffle(&mut new, 0x1318_1318_1318_1318);

        let ops = diff_keys(&old, &new);

        assert!(
            ops.iter().all(|op| matches!(op, KeyedOp::Move { .. })),
            "同一集合のシャッフルは Remove/Insert を発生させず Move のみのはず"
        );
        assert!(
            ops.len() < N,
            "Move の件数は N-1 件以内のはず（実測: {}）",
            ops.len()
        );
    }
}
