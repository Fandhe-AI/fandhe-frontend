//! keyed diff（[`crate::keyed`]）専用の `pub(crate)` ハッシュテーブル基盤。
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
//! （イシュー #1375）。
//!
//! # ターゲット別ハッシャ選択（codex-review・Bugbot 指摘を受けた再設計）
//!
//! 初版・第 2 版（乱数シード化）はいずれも「単一のハッシャ実装で全ターゲット
//! を賄う」設計だったが、次の 2 点で codex-review / Bugbot（cursor[bot]）双方
//! から独立に指摘を受けた。
//!
//! 1. mix 関数自体（`rotate_left`/xor/定数乗算のみ）は非暗号学的であり、
//!    シードをどれだけ秘匿しても SipHash と同等の衝突攻撃耐性を持つとは
//!    言えない（codex-review P1）。
//! 2. `process_seed` が `RandomState::build_hasher()` を経由するため、
//!    ネイティブ・wasm32 いずれのビルドでも SipHash 実装がリンクから
//!    脱落せず、#1375 の payload 削減目的（SipHash 脱却）を達成できない
//!    （codex-review P1）。
//! 3. **wasm32-unknown-unknown 上では `RandomState` はそもそも乱数化され
//!    ない**（`std::sys::random::unsupported::hashmap_random_keys` という
//!    フォールバック経路が使われ、OS エントロピー源を持たない。本モジュール
//!    のビルド成果物を `strings` で確認しても `unsupported::hashmap_random_keys`
//!    が現れ、乱数取得用の外部 import は一切存在しない）。このため
//!    「シードを秘匿すれば HashDoS 耐性を得られる」という前提そのものが
//!    wasm32 では成立しない（Bugbot 指摘）。
//!
//! 3 点目は本モジュール固有の欠陥ではなく、**`core` が元々 wasm32 向けに
//! 使っていた素の `std::collections::HashMap`（既定 `RandomState`）が本 PR
//! 以前から持っていた性質**である（`crates/core/src/keyed.rs` は本 PR 以前
//! 素の `HashMap`/`HashSet` を使っており、ネイティブ・wasm32 を問わず同じ
//! `RandomState` を使っていた。wasm32 に real な OS エントロピー源が無い
//! 以上、乱数化されないのは std 自体の既知の制約であり、`core` は外部
//! クレート依存ゼロ（REQ-3）のため `getrandom`/`web-sys` の
//! `crypto.getRandomValues` 経由で補うこともできない）。よって本モジュール
//! は「ターゲットごとに異なる保証を明示的に文書化した上で、実際に保証
//! できる範囲でのみ SipHash を外す」設計へ改める。
//!
//! - **ネイティブ（`not(target_arch = "wasm32")`、主に SSR/SSG サーバー
//!   プロセス）**: [`FxBuildHasher`] は `std::collections::hash_map::RandomState`
//!   （SipHash）**そのもの**を使う。OS エントロピー由来の乱数シードによる
//!   本物の HashDoS 耐性を持ち、std 既定ハッシャと完全に同じ保証を得る
//!   （このターゲットでは payload 削減の動機がそもそも無く、SSR/SSG
//!   プロセスは複数リクエストを跨いで長時間稼働するため HashDoS の脅威が
//!   実在する）。
//! - **wasm32（`target_arch = "wasm32"`、CSR/ハイドレーション。1 ブラウザ
//!   タブ内でのみ動作）**: [`FxBuildHasher`] は本モジュール実装の軽量
//!   `FxHasher`（固定初期状態、rustc-hash 相当の mix）を使う。
//!   `RandomState` を一切参照しないため SipHash 実装がリンクから脱落し、
//!   #1375 の payload 削減目的を実際に達成する。**HashDoS 耐性は主張しない**
//!   （固定初期状態であり、乱数シード化を試みても wasm32 に real な
//!   エントロピー源が無いため実効性がないことは上記 3 点目で確認済み）。
//!   代わりに、このターゲットでの keyed diff は 1 ブラウザタブ内の CSR
//!   処理に閉じており、意図的な衝突キーが混入した場合の最悪劣化は
//!   「そのタブ自身の CPU 時間が増える」のみ（他ユーザー・他タブ・
//!   サーバープロセスへ波及しない自己完結型の劣化）である。この非対称な
//!   脅威モデル（サーバー側の共有プロセス vs. クライアント側の単一タブ）
//!   により、ネイティブ側で本物の SipHash 耐性を保ちながら wasm32 側でのみ
//!   payload を削減する構成は妥当と判断する。
//!
//! - 最悪劣化（wasm32 側で意図的な衝突キーが混入した場合でも）は当該タブの
//!   CPU 時間の増加のみであり、panic・メモリ安全性・fail-closed 契約
//!   （重複キー拒否・防御的 diff）への影響はない。
//! - 対象マップはすべて `get`/`insert`/`entry` のみで消費され
//!   **イテレートしない**ため、ハッシャ変更（衝突時の挿入順序の違い等）が
//!   `diff_keys`/`diff_keyed_items` の発行 op 順序・SSR/SSG 出力バイトへ
//!   影響することはない（既存の回帰テストが固定する決定性は不変）。
//!
//! ## ゼロ埋め残余バイトによる衝突バグの是正（codex-review 指摘）
//!
//! 旧実装は残余（0〜7 バイト）をゼロ埋めした固定長バッファにコピーして
//! 混合するのみで、**長さそのものを混合していなかった**。このため
//! 例えば `"a"`（1 バイト `0x61`）と `"a\0"`（2 バイト `0x61, 0x00`）は
//! 末尾ゼロパディング後の 8 バイトバッファが完全に一致し、同一ハッシュ値
//! になってしまう衝突バグがあった。本実装（wasm32 側 `FxHasher`）は
//! [`Hasher::write`] の呼び出しごとに、まずバイト列の長さ（`bytes.len()`）
//! を明示的に混合してから本体を混合する。これにより異なる長さの入力は
//! 早期にハッシュ状態が分岐し、上記のような「短い入力 + ゼロバイトの
//! 水増し」による衝突を防ぐ。
//!
//! # `unsafe` 不使用（REQ-2）
//!
//! `core` は `forbid(unsafe_code)` 域のため、バイト列 → `u64` の変換は
//! `u64::from_le_bytes` と配列コピーのみで行い、`unsafe` は一切使わない。

use std::collections::{HashMap, HashSet};

/// ターゲット別のハッシャ実体・[`FxBuildHasher`] 定義。
///
/// ネイティブと wasm32 で異なる `BuildHasher` を選ぶ理由はモジュール doc
/// 「ターゲット別ハッシャ選択」節を参照。
#[cfg(not(target_arch = "wasm32"))]
mod backend {
    /// ネイティブ（SSR/SSG サーバープロセス想定）では std 既定の
    /// `RandomState`（SipHash-1-3、OS エントロピー由来の乱数シード）を
    /// そのまま使う。本物の HashDoS 耐性が必要なターゲットであり、payload
    /// 削減の動機もないため、独自ハッシャへの置き換えは行わない。
    pub(crate) type FxBuildHasher = std::collections::hash_map::RandomState;
}

#[cfg(target_arch = "wasm32")]
mod backend {
    use std::hash::{BuildHasherDefault, Hasher};

    /// rustc-hash 由来の乗数定数。`0x9E3779B97F4A7C15`（黄金比の 64bit
    /// 近似）を奇数化したもので、mix 後のビットが全域に拡散するよう選ばれ
    /// ている。rustc-hash クレート（rustc 本体・Firefox が内部で使用する
    /// 周知の実装、MIT OR Apache-2.0）由来のアルゴリズムの再実装であり、
    /// rustc-hash クレート自体への依存は追加していない（コードの出所を
    /// 追跡可能にするための出典明記）。
    const FX_MULTIPLIER: u64 = 0x51_7c_c1_b7_27_22_0a_95;

    /// FxHash 相当の軽量ハッシャ本体。状態は `u64` 1 語のみ。
    ///
    /// wasm32-unknown-unknown には real な OS エントロピー源が無いため
    /// （モジュール doc 参照）、初期状態は固定定数とする。HashDoS 耐性は
    /// 主張しない（このターゲットでの脅威モデルはモジュール doc
    /// 「ターゲット別ハッシャ選択」節参照）。
    pub(crate) struct FxHasher {
        hash: u64,
    }

    impl Default for FxHasher {
        #[inline]
        fn default() -> Self {
            FxHasher {
                hash: FX_MULTIPLIER,
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
            // モジュール doc 参照。例: "a" と "a\0" のような異なる長さの
            // 入力を早期にハッシュ状態上で分岐させる）。
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
            // 残余（0〜7 バイト）はゼロ埋めした固定長バッファへコピーして
            // から同じ mix にかける。長さは既に混合済みのため、この段階
            // でのゼロパディングが異なる入力間の衝突を生むことはない。
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

    /// wasm32 では `RandomState`（SipHash）を一切参照しない軽量ハッシャを
    /// 既定にする。これにより SipHash 実装がリンクから脱落し、#1375 の
    /// payload 削減目的を達成する（モジュール doc「ターゲット別ハッシャ
    /// 選択」節参照）。
    pub(crate) type FxBuildHasher = BuildHasherDefault<FxHasher>;
}

/// [`backend`] が選ぶ `BuildHasher`（ネイティブ: `RandomState`、wasm32:
/// 軽量 `FxHasher`）。`keyed.rs` はターゲット差を意識せずこの型を使う。
pub(crate) type FxBuildHasher = backend::FxBuildHasher;

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

    /// [`map_with_capacity`] / [`set_with_capacity`] が通常の
    /// get/insert/entry 操作で機能することを確認する（`keyed.rs` の
    /// 使用パターンの最小再現。ネイティブ・wasm32 いずれのハッシャ実体
    /// でも成立する）。
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

/// wasm32 専用 [`backend::FxHasher`] の単体テスト。ネイティブビルド
/// （`cargo test -p fandhe-frontend-core`）では一切コンパイルされない
/// （`cfg(target_arch = "wasm32")` の実装自体がネイティブに存在しないため）。
/// `clippy-wasm32` ジョブ（`.claude/rules/ci.md` 参照）はこの実装を
/// コンパイルレベルで検証するが、`cargo test` を wasm32 target で実行する
/// CI ジョブは現状存在しないため、本テストの実行は wasm-pack 等の
/// ブラウザ/Node ハーネスをローカルまたは将来の CI で用いる場合に限られる
/// （既知の制約。std 自身のターゲット限定コードと同じ扱い）。
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::backend::FxHasher;
    use std::hash::{Hash, Hasher};

    fn hash_of<T: Hash + ?Sized>(value: &T) -> u64 {
        let mut hasher = FxHasher::default();
        value.hash(&mut hasher);
        hasher.finish()
    }

    /// 再現性: 同一入力からは常に同一ハッシュ値が得られる（決定性、
    /// SSR/SSG 出力バイトの安定性を支える前提。CSR 側でも同一プロセス
    /// 内での再現性は必要）。
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
            assert_eq!(hash_of(s.as_str()), hash_of(s.as_str()));
        }
    }

    /// 回帰テスト: ゼロ埋め残余バイトによる衝突バグの是正
    /// （codex-review 指摘、イシュー #1375）。`b"a"`（1 バイト）と
    /// `b"a\0"`（2 バイト、末尾に実データとしての `0x00` を持つ）は、
    /// 長さを混合しない実装では末尾ゼロパディング後の 8 バイトバッファが
    /// 完全に一致してしまい同一ハッシュ値になっていた。
    #[test]
    fn distinguishes_short_inputs_that_differ_only_by_trailing_zero_byte() {
        let hash_bytes = |bytes: &[u8]| -> u64 {
            let mut hasher = FxHasher::default();
            hasher.write(bytes);
            hasher.finish()
        };
        assert_ne!(hash_bytes(b"a"), hash_bytes(b"a\0"));
        assert_ne!(hash_bytes(b"abcdefg"), hash_bytes(b"abcdefg\0"));
    }
}
