//! 軽量ハッシャ（FxHash 相当）— keyed diff（[`crate::keyed`]）専用の
//! `pub(crate)` ハッシュテーブル基盤。
//!
//! # 何のためにあるか
//!
//! `crates/core/src/keyed.rs` の keyed diff（`diff_keys` / `diff_keyed_items`
//! / `remove_pass` / `insert_or_move_pass`）と `keyed_list` の重複キー検査は
//! 文字列キーの `HashMap`/`HashSet` を多用する。std 既定ハッシャ
//! （`RandomState`、SipHash-1-3）は DoS 耐性のため意図的に低速・大きめの
//! 実装であり、親トラッキング #1371 の twiggy 実測で
//! `hashbrown` + SipHash が wasm payload の 12.8%（11.0KB）を占め、CSR
//! update の純粋 diff 時間（0.43ms）の主成分であることが判明している
//! （イシュー #1375）。本モジュールは keyed diff の内部照合専用に、
//! 外部クレートを追加せず（REQ-3 / core 外部依存ゼロ）std のみで
//! FxHash 相当のハッシャを実装し、`keyed.rs` からのみ使う。
//!
//! # アルゴリズムの出自
//!
//! mix 関数・定数 `0x51_7c_c1_b7_27_22_0a_95` は rustc-hash クレート
//! （rustc 本体・Firefox が内部で使用する周知の実装、MIT OR Apache-2.0）に
//! 由来するアルゴリズムの再実装であり、rustc-hash クレート自体への依存は
//! 追加していない（コードの出所を追跡可能にするための出典明記）。
//!
//! # 脅威モデル（受け入れ条件 2、`security.md` A04/DoS 観点）
//!
//! SipHash は「攻撃者が任意のキーをハッシュテーブルへ流し込める公開面」で
//! 衝突キーを故意に投入する HashDoS（O(n²) 化）を防ぐために存在する。
//! 本モジュールの対象マップ（`keyed.rs` の 7 箇所）は次の理由で
//! HashDoS の脅威モデル**外**にあると判断する:
//!
//! - キーはすべて `keyed_list` / `diff_keys` / `diff_keyed_items` の呼び出し
//!   側（アプリ自身）が構築した文字列列であり、ネットワーク越しの入力を
//!   直接ハッシュテーブルのキーとして受け取る公開 API ではない。
//! - 最悪劣化（意図的な衝突キーが混入した場合でも）は CPU 時間の増加のみ
//!   であり、panic・メモリ安全性・fail-closed 契約（重複キー拒否・防御的
//!   diff）への影響はない。攻撃者が制御する非有界のキー列をそのまま
//!   リストとして構築するアプリ側の設計は、ハッシャの選択に関わらず
//!   別途 DoS 対策が必要であり、本変更はその責務を変えない。
//! - 対象マップはすべて `get`/`insert`/`entry` のみで消費され
//!   **イテレートしない**ため、ハッシャ変更（衝突時の挿入順序の違い等）が
//!   `diff_keys`/`diff_keyed_items` の発行 op 順序・SSR/SSG 出力バイトへ
//!   影響することはない（既存の回帰テストが固定する決定性は不変）。
//!
//! シードは固定（`FxHasher::default()` は常に状態 0 から開始、乱数を
//! 一切使わない）であり、これも上記の決定性と整合する。
//!
//! # `unsafe` 不使用（REQ-2）
//!
//! `core` は `forbid(unsafe_code)` 域のため、バイト列 → `u64` の変換は
//! `u64::from_le_bytes` と配列コピーのみで行い、`unsafe` は一切使わない。

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

/// rustc-hash 由来の乗数定数。`0x9E3779B97F4A7C15`（黄金比の 64bit 近似）を
/// 奇数化したもので、mix 後のビットが全域に拡散するよう選ばれている
/// （出典は本モジュール doc 冒頭を参照）。
const FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

/// FxHash 相当の軽量ハッシャ本体。状態は `u64` 1 語のみで、SipHash のような
/// 鍵付き構成を持たない（脅威モデルはモジュール doc 参照）。
#[derive(Default)]
pub(crate) struct FxHasher {
    hash: u64,
}

impl FxHasher {
    /// 1 ワード（`u64`）分の状態を混合する。
    /// `rotate_left(5)` + xor + 定数乗算という rustc-hash と同型の mix。
    #[inline]
    fn write_u64_inner(&mut self, word: u64) {
        self.hash = (self.hash.rotate_left(5) ^ word).wrapping_mul(FX_SEED);
    }
}

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, mut bytes: &[u8]) {
        // 8 バイトずつ little-endian で取り込む。`core` の対象キーは
        // すべて `&str`（`Hasher::write_str` は不安定 API のため未使用、
        // `write` 経由の既定実装に委ねる）。
        while bytes.len() >= 8 {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(&bytes[..8]);
            self.write_u64_inner(u64::from_le_bytes(buf));
            bytes = &bytes[8..];
        }
        // 残余（0〜7 バイト）はゼロ埋めした固定長バッファへコピーしてから
        // 同じ mix にかける。短い文字列キー（typ. 数〜数十バイト）が
        // 大半を占める keyed diff の用途では、この残余処理が実質的な
        // ハッシュ品質を左右するため、境界（7/8/9 バイト）は単体テストで
        // 固定する。
        if !bytes.is_empty() {
            let mut buf = [0u8; 8];
            buf[..bytes.len()].copy_from_slice(bytes);
            self.write_u64_inner(u64::from_le_bytes(buf));
        }
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }
}

/// [`FxHasher`] を既定ハッシャとする `BuildHasher`。
pub(crate) type FxBuildHasher = BuildHasherDefault<FxHasher>;

/// キー型を `&str` に限らない一般形（`keyed.rs` は `&str` キーのみ使用）。
pub(crate) type FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;
pub(crate) type FxHashSet<T> = HashSet<T, FxBuildHasher>;

/// 容量指定付きで空の [`FxHashMap`] を作る。
///
/// `HashMap::with_capacity` は既定ハッシャ（`RandomState`）専用のため、
/// カスタムハッシャでの容量確保には `with_capacity_and_hasher` を使う
/// 薄いラッパとして提供する（`keyed.rs` の呼び出し側を簡潔に保つ）。
#[inline]
pub(crate) fn map_with_capacity<K, V>(capacity: usize) -> FxHashMap<K, V> {
    FxHashMap::with_capacity_and_hasher(capacity, FxBuildHasher::default())
}

/// 容量指定付きで空の [`FxHashSet`] を作る（[`map_with_capacity`] と対）。
#[inline]
pub(crate) fn set_with_capacity<T>(capacity: usize) -> FxHashSet<T> {
    FxHashSet::with_capacity_and_hasher(capacity, FxBuildHasher::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::Hash;

    fn hash_of<T: Hash + ?Sized>(value: &T) -> u64 {
        let mut hasher = FxHasher::default();
        value.hash(&mut hasher);
        hasher.finish()
    }

    /// 再現性: 同一入力からは常に同一ハッシュ値が得られる（決定性、
    /// SSR/SSG 出力バイトの安定性を支える前提）。
    #[test]
    fn same_input_produces_same_hash() {
        assert_eq!(hash_of("hello-world"), hash_of("hello-world"));
        assert_eq!(hash_of(""), hash_of(""));
    }

    /// 素朴な分散確認: 異なる文字列は（衝突しないとは限らないが）通常は
    /// 異なるハッシュ値になる。テストデータ間で偶然の総当たり衝突がない
    /// ことのみを確認する軽量な健全性チェック。
    #[test]
    fn distinct_inputs_usually_hash_differently() {
        let keys = [
            "a",
            "b",
            "item-0",
            "item-1",
            "item-2",
            "key-with-more-bytes",
        ];
        let hashes: Vec<u64> = keys.iter().map(|k| hash_of(*k)).collect();
        for i in 0..hashes.len() {
            for j in (i + 1)..hashes.len() {
                assert_ne!(
                    hashes[i], hashes[j],
                    "unexpected collision between {:?} and {:?}",
                    keys[i], keys[j]
                );
            }
        }
    }

    /// 境界確認: 空文字列・8 の倍数境界（7/8/9 バイト）を含む長さで
    /// `write` の chunk 処理（8 バイトずつ + 残余）が panic せず
    /// 決定的に完了することを確認する。
    #[test]
    fn handles_length_boundaries() {
        for len in [0usize, 1, 7, 8, 9, 15, 16, 17, 63, 64, 65] {
            let s: String = "x".repeat(len);
            // 同一入力の再ハッシュが安定していることを併せて確認する。
            assert_eq!(hash_of(s.as_str()), hash_of(s.as_str()));
        }
    }

    /// [`map_with_capacity`] / [`set_with_capacity`] が通常の
    /// get/insert/entry 操作で機能することを確認する（`keyed.rs` の
    /// 使用パターンの最小再現）。
    #[test]
    fn map_and_set_basic_operations() {
        let mut map: FxHashMap<&str, usize> = map_with_capacity(4);
        map.insert("a", 0);
        map.insert("b", 1);
        assert_eq!(map.get("a"), Some(&0));
        assert_eq!(map.get("z"), None);

        let mut set: FxHashSet<&str> = set_with_capacity(4);
        set.insert("a");
        assert!(set.contains("a"));
        assert!(!set.contains("z"));
    }
}
