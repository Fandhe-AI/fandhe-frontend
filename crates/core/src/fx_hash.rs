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
//! # 脅威モデル（受け入れ条件 2、`security.md` A04/DoS 観点。イシュー #1375
//! codex-review 指摘を受けて再設計、下記「固定シード撤廃」節参照）
//!
//! SipHash は「攻撃者が任意のキーをハッシュテーブルへ流し込める公開面」で
//! 衝突キーを故意に投入する HashDoS（O(n²) 化）を防ぐために存在する。
//! [`keyed_list`][crate::keyed::keyed_list] / [`diff_keys`][crate::keyed::diff_keys] /
//! [`diff_keyed_items`][crate::keyed::diff_keyed_items] は公開 API であり、
//! キーが呼び出し側アプリのローカル変数由来だとしても、その値自体は
//! API・DB・URL 等の外部入力に由来し得る（「呼び出し側が構築するから
//! 脅威モデル外」という主張はキーの**データ由来**を保証しない）。このため
//! 本モジュールは固定シードを採用せず、後述のとおりプロセス起動ごとに
//! ランダムなシードを用いる。
//!
//! ## 固定シード撤廃と乱数シード化
//!
//! 旧実装（初版、イシュー #1375 初回コミット）は固定定数
//! `0x51_7c_c1_b7_27_22_0a_95` のみをシードとして使っており、攻撃者が
//! 事前にこの定数でハッシュ計算して衝突キー列を用意できてしまう欠陥
//! だった（codex-review P1 指摘）。本実装は [`process_seed`] が
//! プロセス起動時に一度だけ `std::collections::hash_map::RandomState`
//! （OS エントロピー由来、std 標準の HashDoS 対策そのもの）からシードを
//! 導出し、以降の全 [`FxHasher`] インスタンスがこの共有シードから
//! 初期化される。シード自体は事前計算不可能になるため、`RandomState`
//! （SipHash）と同等の「攻撃者が衝突キー列を事前に用意できない」という
//! HashDoS 耐性を得つつ、mix 関数自体は SipHash より軽量なままである
//! （payload/CPU 削減という #1375 の目的を維持）。
//!
//! - 最悪劣化（意図的な衝突キーが混入した場合でも）は CPU 時間の増加のみ
//!   であり、panic・メモリ安全性・fail-closed 契約（重複キー拒否・防御的
//!   diff）への影響はない。
//! - 対象マップはすべて `get`/`insert`/`entry` のみで消費され
//!   **イテレートしない**ため、ハッシャ変更（衝突時の挿入順序の違い等）が
//!   `diff_keys`/`diff_keyed_items` の発行 op 順序・SSR/SSG 出力バイトへ
//!   影響することはない（既存の回帰テストが固定する決定性は不変）。乱数
//!   シードは同一プロセス内では不変（`OnceLock` で 1 回のみ生成）なため、
//!   同一プロセスでの `same_input_produces_same_hash` 系の再現性は保たれる
//!   （プロセスをまたいだハッシュ値の再現性は元々保証していない。
//!   `RandomState` を使う std 既定ハッシャと同じ性質）。
//!
//! ## ゼロ埋め残余バイトによる衝突バグの是正（codex-review 指摘）
//!
//! 旧実装は残余（0〜7 バイト）をゼロ埋めした固定長バッファにコピーして
//! 混合するのみで、**長さそのものを混合していなかった**。このため
//! 例えば `"a"`（1 バイト `0x61`）と `"a\0"`（2 バイト `0x61, 0x00`）は
//! 末尾ゼロパディング後の 8 バイトバッファが完全に一致し、同一ハッシュ値
//! になってしまう衝突バグがあった。本実装は [`Hasher::write`] の呼び出し
//! ごとに、まずバイト列の長さ（`bytes.len()`）を明示的に混合してから
//! 本体を混合する。これにより異なる長さの入力は早期にハッシュ状態が
//! 分岐し、上記のような「短い入力 + ゼロバイトの水増し」による衝突を
//! 防ぐ。
//!
//! # `unsafe` 不使用（REQ-2）
//!
//! `core` は `forbid(unsafe_code)` 域のため、バイト列 → `u64` の変換は
//! `u64::from_le_bytes` と配列コピーのみで行い、`unsafe` は一切使わない。
//! `process_seed` が用いる `RandomState`（std 提供）も `unsafe` を要さない。

use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, BuildHasherDefault, Hasher};
use std::sync::OnceLock;

/// rustc-hash 由来の乗数定数。`0x9E3779B97F4A7C15`（黄金比の 64bit 近似）を
/// 奇数化したもので、mix 後のビットが全域に拡散するよう選ばれている
/// （出典は本モジュール doc 冒頭を参照）。シード**値**そのものではなく
/// mix 関数の乗数として使うため固定でよい（HashDoS 耐性はシードの乱数化
/// （[`process_seed`]）が担う）。
const FX_MULTIPLIER: u64 = 0x51_7c_c1_b7_27_22_0a_95;

/// プロセス起動後、初回アクセス時に一度だけ生成される乱数シード。
///
/// `std::collections::hash_map::RandomState`（std 標準の HashDoS 対策・
/// OS エントロピー由来）からシードを 1 回だけ引き出し、以降の全
/// [`FxHasher`] インスタンスで共有する。攻撃者はこのシードを事前に知り
/// 得ないため、固定シード時代に可能だった「事前計算した衝突キー列の
/// 投入」（codex-review P1 指摘）が成立しなくなる。プロセス内では不変の
/// ため、`keyed.rs` の決定性契約（op 発行順序・SSR/SSG 出力バイト）には
/// 影響しない（モジュール doc「固定シード撤廃と乱数シード化」節参照）。
fn process_seed() -> u64 {
    static SEED: OnceLock<u64> = OnceLock::new();
    *SEED.get_or_init(|| {
        // RandomState 自体を一度ハッシュに通してシードを取り出す
        // （RandomState は内部キーを直接公開しないため、`Hasher` 経由で
        // 間接的に 1 個の u64 を得る標準的な手法）。
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_u8(0);
        hasher.finish()
    })
}

/// FxHash 相当の軽量ハッシャ本体。状態は `u64` 1 語のみ。初期値は
/// [`process_seed`]（プロセス単位の乱数）であり、SipHash のような
/// メッセージ認証レベルの鍵付き構成ではないが、シード自体の秘匿性により
/// 事前計算衝突攻撃への耐性を持つ（脅威モデルはモジュール doc 参照）。
pub(crate) struct FxHasher {
    hash: u64,
}

impl Default for FxHasher {
    #[inline]
    fn default() -> Self {
        FxHasher {
            hash: process_seed(),
        }
    }
}

impl FxHasher {
    /// 1 ワード（`u64`）分の状態を混合する。
    /// `rotate_left(5)` + xor + 定数乗算という rustc-hash と同型の mix。
    #[inline]
    fn write_u64_inner(&mut self, word: u64) {
        self.hash = (self.hash.rotate_left(5) ^ word).wrapping_mul(FX_MULTIPLIER);
    }
}

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, mut bytes: &[u8]) {
        // まず長さを混合する（ゼロ埋め残余バイトによる衝突バグの是正、
        // モジュール doc 参照。例: "a" と "a\0" のような異なる長さの入力を
        // 早期にハッシュ状態上で分岐させる）。
        self.write_u64_inner(bytes.len() as u64);

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
        // 同じ mix にかける。長さは既に混合済みのため、この段階でのゼロ
        // パディングが異なる入力間の衝突を生むことはない。短い文字列キー
        // （typ. 数〜数十バイト）が大半を占める keyed diff の用途では、
        // この残余処理が実質的なハッシュ品質を左右するため、境界
        // （7/8/9 バイト）は単体テストで固定する。
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

    /// 回帰テスト: ゼロ埋め残余バイトによる衝突バグの是正
    /// （codex-review 指摘、イシュー #1375）。`b"a"`（1 バイト）と
    /// `b"a\0"`（2 バイト、末尾に実データとしての `0x00` を持つ）は、
    /// 長さを混合しない実装では末尾ゼロパディング後の 8 バイトバッファが
    /// 完全に一致してしまい同一ハッシュ値になっていた。`Hasher::write` を
    /// 直接呼び出し、`Hash for str` の末尾センチネル（`write_u8(0xff)`）を
    /// 介さない生バイト列同士で衝突しないことを固定する。
    #[test]
    fn distinguishes_short_inputs_that_differ_only_by_trailing_zero_byte() {
        let hash_bytes = |bytes: &[u8]| -> u64 {
            let mut hasher = FxHasher::default();
            hasher.write(bytes);
            hasher.finish()
        };
        assert_ne!(hash_bytes(b"a"), hash_bytes(b"a\0"));
        // 8 バイト境界をまたぐ組も併せて確認する（7 バイト+ゼロ埋め と
        // 8 バイトちょうどの組み合わせ）。
        assert_ne!(hash_bytes(b"abcdefg"), hash_bytes(b"abcdefg\0"));
    }

    /// プロセス内でのシード安定性: [`process_seed`] は `OnceLock` により
    /// プロセス内で不変であるため、同一プロセス内では複数回呼び出しても
    /// 同じ値を返す（`keyed.rs` の決定性契約が乱数シード化後も保たれる
    /// ことの直接確認）。
    #[test]
    fn process_seed_is_stable_within_process() {
        assert_eq!(process_seed(), process_seed());
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
