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
//! - **ネイティブおよび wasm32-wasi 等
//!   （`not(all(target_arch = "wasm32", target_os = "unknown"))`、主に
//!   SSR/SSG サーバープロセス・共有プロセスで動く wasm32 ターゲット）**:
//!   [`FxBuildHasher`] は `std::collections::hash_map::RandomState`
//!   （SipHash）**そのもの**を使う。OS エントロピー由来の乱数シードによる
//!   本物の HashDoS 耐性を持ち、std 既定ハッシャと完全に同じ保証を得る
//!   （このターゲットでは payload 削減の動機がそもそも無く、これらの
//!   プロセスは複数リクエストを跨いで長時間稼働するため HashDoS の脅威が
//!   実在する）。判定を `target_arch = "wasm32"` 単独ではなく
//!   `target_os = "unknown"` も要求する組み合わせにする理由は次項参照
//!   （codex-review P1 指摘、イシュー #1375）。
//! - **wasm32-unknown-unknown
//!   （`all(target_arch = "wasm32", target_os = "unknown")`、ブラウザでの
//!   CSR/ハイドレーション。1 ブラウザタブ内でのみ動作）**:
//!   [`FxBuildHasher`] は本モジュール実装の軽量
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
//! ## 追加防御: 項目数・キー総バイト数の上限（PR #1390 レビュー是正）
//!
//! 上記の「自己完結型の劣化だから許容できる」という判断は、codex-review
//! から「最悪劣化の大きさ自体を拘束していない」との P1 指摘を受けた
//! （キー件数・総バイト数が無制限である限り、単一タブとはいえ攻撃者は
//! タブを事実上無期限に停止させられる）。この指摘を受け、
//! [`crate::keyed::keyed_list`] は項目数
//! （[`crate::keyed::MAX_KEYED_LIST_ITEMS`]）とキー文字列の合計バイト数
//! （[`crate::keyed::MAX_KEYED_LIST_KEY_BYTES`]）の双方に上限を設け、
//! 超過時は fail-closed に拒否する。この上限は本モジュールが持つ
//! ターゲット別ハッシャ選択とは独立に、**全ターゲット共通**（`cfg` 分岐
//! なし）で適用される（判断根拠は `keyed::MAX_KEYED_LIST_ITEMS` doc
//! 「全ターゲット一律で強制する理由」節参照）。これにより、
//! wasm32-unknown-unknown で意図的な衝突キーが混入した場合でも、最悪
//! 計算量は上限値に基づいて事前に見積もり可能な有界値に収まる。
//!
//! - 最悪劣化（wasm32 側で意図的な衝突キーが混入した場合でも）は当該タブの
//!   CPU 時間の増加のみであり、panic・メモリ安全性・fail-closed 契約
//!   （重複キー拒否・防御的 diff）への影響はない。
//! - 対象マップはすべて `get`/`insert`/`entry` のみで消費され
//!   **イテレートしない**ため、ハッシャ変更（衝突時の挿入順序の違い等）が
//!   `diff_keys`/`diff_keyed_items` の発行 op 順序・SSR/SSG 出力バイトへ
//!   影響することはない（既存の回帰テストが固定する決定性は不変）。
//!
//! ## 追加防御その 2: 一次ハッシュ衝突時の fail-closed 拒否
//! （PR #1390 codex-review 第 2 巡 P1 是正、イシュー #1375）
//!
//! 上記「項目数・キー総バイト数の上限」だけでは、**比較回数**は拘束できて
//! も**個々の比較の重さ**（同一バケット内で候補どうしが持つ共通接頭辞の
//! 長さ）までは拘束できないとの追加指摘を受けた。攻撃者は固定初期状態の
//! 軽量 `FxHasher`（wasm32-unknown-unknown）で同一バケットへ落ち、かつ
//! 互いに長い共通接頭辞を持つキー列を事前計算できるため、上限ぎりぎりの
//! 件数（4096 件・約 64 byte）でも `str` バイト単位比較の総量は
//! 見積もりよりはるかに大きく劣化し得る（概算 10 億 byte 相当）。
//!
//! この指摘を受け、`keyed.rs` の文字列キー `HashMap`/`HashSet` はすべて
//! [`FxStrMap`]/[`FxStrSet`] へ置き換えた。文字列を直接ハッシュテーブルへ
//! 渡さず、まず `S::hash_one(key)` で 64bit 値へ一次ハッシュしてから、その
//! `u64` をキーとする内側マップへ格納する。各操作はスロットが埋まって
//! いる場合のみ格納済み `&str` と probe を**高々 1 回**比較し、不一致
//! （= 64bit ハッシュの衝突）なら [`KeyHashCollisionError`] を返してその場
//! で拒否する（`keyed.rs` 側では `KeyedListError::KeyHashCollision` へ変換
//! して伝播する）。これにより総比較バイト数は
//! `O(総キーバイト数)`（≤ `keyed::MAX_KEYED_LIST_KEY_BYTES`）の線形で拘束
//! され、「件数の 2 乗 × 共通接頭辞長」という項は構造的に消滅する。設計の
//! 詳細・内側マップの衝突が安全な理由・正規入力が誤って衝突判定される
//! 確率の見積もりは [`FxStrMap`] の型 doc を参照。
//!
//! ### wasm32-unknown-unknown における「意図的な」一次ハッシュ衝突
//!
//! [`FxStrMap`] 型 doc の確率見積もり（`n = 4096` で約 `4.5 * 10^-13`）は
//! **偶発的な**衝突の話であり、**攻撃者が意図的に**衝突を作れるかどうか
//! とは別の問題である。ネイティブ（`RandomState`/SipHash、OS エントロピー
//! 由来の秘匿シード）では攻撃者はシードを知り得ないため、オフラインで
//! 衝突ペアを事前計算することはできない。一方 wasm32-unknown-unknown の
//! `FxHasher` は非暗号学的（rotate/xor/定数乗算のみの可逆に近い mix）かつ
//! 固定初期状態であり、シードの秘匿という前提自体が成立しない
//! （モジュール doc「ターゲット別ハッシャ選択」節参照）。このため
//! **wasm32-unknown-unknown 上では、攻撃者が同一の 64bit 一次ハッシュ値を
//! 持つ 2 つの異なる文字列キーを意図的に構成することは依然として可能**
//! であり、[`FxStrMap`]/[`FxStrSet`] の衝突検知はこれを「起こさない」
//! 保証ではなく「起きたときの被害を有界にする」保証である。
//!
//! 具体的には、衝突キーが混入した場合の帰結が「そのタブの CPU 時間が
//! 無期限に劣化する」（旧 `N^2` バイト比較見積もりが防ごうとしていた
//! 事態）から「その場で決定的に [`KeyHashCollisionError`]
//! （`keyed.rs` 側では [`crate::keyed::KeyedListError::KeyHashCollision`]）
//! を返し、当該 `keyed_list` 構築・`diff_keys`/`diff_keyed_items` 呼び出し
//! 全体を fail-closed に拒否する」へと**性質が変わる**。この拒否は
//! panic・メモリ安全性・他タブ・サーバープロセスへの影響を一切持たず、
//! 「1 ブラウザタブ内の自己完結型の劣化」というモジュール doc 「ターゲット
//! 別ハッシャ選択」節の脅威モデルの範囲内に収まる（無期限の CPU 占有では
//! なく即時の決定的エラーへ置き換わる点で、むしろ攻撃者にとっての実利は
//! 「CPU を奪う」から「機能を使わせない」へ後退する）。これが本節が
//! 「衝突を防ぐ」ではなく「衝突のコストを拘束する」設計として位置づける
//! 理由である。
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

use std::collections::HashMap;

/// [`FxStrMap`]/[`FxStrSet`] の構築回数を数えるテスト専用カウンタ
/// （イシュー #1376）。
///
/// `crates/core/src/keyed.rs` の前段スキップ（共通接頭辞・接尾辞トリム）
/// が「キー列不変ケースで `HashMap`/`HashSet` を一切構築しない」ことを、
/// ベンチ実測に頼らずユニットテストで機械検証するための計測点。
/// `#[cfg(test)]` 限定であり、prod ビルドには一切残らない（コード自体が
/// コンパイル対象から除外される。`thread_local!` の実行時オーバーヘッドも
/// テストバイナリのみに閉じる）。
#[cfg(test)]
pub(crate) mod build_counter {
    use std::cell::Cell;

    thread_local! {
        static COUNT: Cell<usize> = const { Cell::new(0) };
    }

    /// カウンタを 0 へリセットする（各テストの計測区間の開始点）。
    pub(crate) fn reset() {
        COUNT.with(|c| c.set(0));
    }

    /// 現在の構築回数を返す。
    pub(crate) fn get() -> usize {
        COUNT.with(|c| c.get())
    }

    /// [`super::FxStrMap`]/[`super::FxStrSet`] の `with_capacity_and_hasher`
    /// から呼ばれる、構築 1 回ぶんの計上。
    pub(crate) fn increment() {
        COUNT.with(|c| c.set(c.get() + 1));
    }
}

/// ターゲット別のハッシャ実体・[`FxBuildHasher`] 定義。
///
/// ネイティブと wasm32 で異なる `BuildHasher` を選ぶ理由はモジュール doc
/// 「ターゲット別ハッシャ選択」節を参照。
///
/// # cfg 判定はブラウザ実行（`wasm32-unknown-unknown`）に限定する
/// （codex-review P1 指摘、イシュー #1375）
///
/// `target_arch = "wasm32"` 単独の判定は `wasm32-unknown-unknown`
/// （ブラウザ、1 タブ内に閉じる）だけでなく `wasm32-wasi`/`wasm32-wasip1`
/// 等のサーバー側 wasm32 ターゲットにも一致してしまう。本モジュールが
/// 固定シードの軽量ハッシャを許容する根拠（モジュール doc 「ターゲット別
/// ハッシャ選択」節）は「1 ブラウザタブ内に閉じる自己完結型の劣化」に
/// 依存しており、WASI 等の共有プロセスへ `core` が組み込まれる構成では
/// 成立しない（外部由来の keyed-list キーから意図的な衝突を作られると、
/// 共有プロセスの CPU を消費する HashDoS 防御の後退になる）。そのため
/// 軽量ハッシャの選択条件は `target_arch = "wasm32"` に加えて
/// `target_os = "unknown"`（`wasm32-unknown-unknown` を一意に特定する
/// 唯一の組み合わせ。他の wasm32 ターゲットはいずれも `target_os` が
/// `"wasi"` 等の非 `"unknown"` 値を持つ）を要求する。この組み合わせに
/// 一致しない wasm32 ターゲット（WASI 等）はネイティブ側と同じ
/// `RandomState`（本物の HashDoS 耐性）にフォールバックする。
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
mod backend {
    /// ネイティブおよび wasm32-wasi 等（SSR/SSG・共有サーバープロセス
    /// 想定）では std 既定の `RandomState`（SipHash-1-3、OS エントロピー
    /// 由来の乱数シード）をそのまま使う。本物の HashDoS 耐性が必要な
    /// ターゲットであり、payload 削減の動機もないため、独自ハッシャへの
    /// 置き換えは行わない。
    pub(crate) type FxBuildHasher = std::collections::hash_map::RandomState;
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
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

/// キー型を `&str` に限らない一般形。[`FxStrMap`]/[`FxStrSet`]
/// （`keyed.rs` が実際に使う 64bit 一次ハッシュ間接テーブル）の内側実装
/// として使われる（型 doc「内側 `HashMap<u64, ..>` のハッシャに追加防御が
/// 不要な理由」節参照）。
pub(crate) type FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;

/// 容量指定付きで空の [`FxHashMap`] を作る。
///
/// `HashMap::with_capacity` は既定ハッシャ（`RandomState`）専用のため、
/// カスタムハッシャでの容量確保には `with_capacity_and_hasher` を使う
/// 薄いラッパとして提供する（[`FxStrMap`]/[`FxStrSet`] の内側マップ構築を
/// 簡潔に保つ）。
#[inline]
pub(crate) fn map_with_capacity<K, V>(capacity: usize) -> FxHashMap<K, V> {
    FxHashMap::with_capacity_and_hasher(capacity, FxBuildHasher::default())
}

/// [`FxStrMap`]/[`FxStrSet`] が「同一の 64bit ハッシュ値を持つが中身の
/// 異なる 2 つの文字列キー」を検出したときに返す fail-closed エラー。
///
/// # 意図的にキー文字列を含めない理由
///
/// `KeyedListError::Display`（`keyed.rs`）と同じ規約（ログ・エラー
/// メッセージへアプリ状態を含めない、OWASP A09 対策）に合わせ、衝突した
/// 2 つの文字列そのものは運ばない。呼び出し側（`keyed.rs`）が
/// `KeyedListError::KeyHashCollision` へ変換する際も同様に内容を含めない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KeyHashCollisionError;

/// `str` キーの `HashMap` を「64bit ハッシュ値をキーとする間接テーブル」に
/// 置き換えた `pub(crate)` コレクション（PR #1390 codex-review 第 2 巡 P1
/// 是正、イシュー #1375）。
///
/// # 何を解決するか
///
/// [`FxHashMap`]（素の `HashMap<&str, V, FxBuildHasher>`）は、同一バケットに
/// 複数の候補が衝突すると各候補に対して `str` の等価比較（バイト単位、
/// キー長に比例するコスト）を行う。[`crate::keyed::MAX_KEYED_LIST_ITEMS`] /
/// [`crate::keyed::MAX_KEYED_LIST_KEY_BYTES`] は「比較回数」の上限は与える
/// が、**個々の比較の重さ**（共通接頭辞長）は拘束していない。攻撃者が
/// 「固定初期状態の軽量ハッシャ（wasm32-unknown-unknown、モジュール doc
/// 「ターゲット別ハッシャ選択」節）で同一バケットへ落ち、かつ長い共通
/// 接頭辞を持つ」キー列を事前計算すると、上限ぎりぎりの件数でも比較
/// バイト数の総量が大きく劣化し得る（codex-review 指摘: 4096 件・64 byte
/// キーで約 10 億 byte 相当の比較になり得る）。
///
/// 本型は文字列キーを直接ハッシュテーブルへ渡さず、まず
/// `S::hash_one(key)` で 64bit 値へ一次ハッシュしてから、その `u64` を
/// キーとする内側の `HashMap<u64, (&str, V), FxBuildHasher>` へ格納する。
/// 各操作（[`Self::get`]/[`Self::insert`]/[`Self::get_mut`]/
/// [`Self::get_or_insert_with`]）はスロットが埋まっている場合のみ、格納
/// 済みの `&str` と probe の `&str` を**高々 1 回**比較する。一致すれば
/// 通常の同一キー動作（上書き・参照・`or_insert` 相当）、不一致（= 64bit
/// ハッシュの衝突）なら [`KeyHashCollisionError`] を返して**その場で拒否**
/// する。これにより 1 回の探索あたりの比較は「格納済み 1 本との比較」に
/// 限定され、総比較バイト数は `O(総キーバイト数)`
/// （≤ [`crate::keyed::MAX_KEYED_LIST_KEY_BYTES`]）の線形で拘束される。
/// 「件数の 2 乗 × 共通接頭辞長」という項は構造的に発生し得ない。
///
/// # 内側 `HashMap<u64, ..>` のハッシャに追加防御が不要な理由
///
/// 内側マップは `u64` キー（= 既に一次ハッシュ済みの値）を保持する。この
/// `u64` を内側マップ用に**さらに**ハッシュしてバケットへ振り分ける際、
/// 攻撃者が内側バケットの衝突（`u64` 値どうしの二次衝突）を作れたとしても、
/// 同一バケット内の候補比較は `u64` の等値比較（O(1)、レジスタ 1 個分の
/// 比較）に留まり、`str` のバイト単位比較には一切至らない。つまり内側
/// マップの衝突は探索コストを最大でも定数倍にしか劣化させず、本型が解決
/// したい「キー長・共通接頭辞長に比例する比較コスト」を再導入しない。
/// このため内側マップは既存の [`FxBuildHasher`]
/// （ネイティブ: `RandomState`、wasm32: 軽量 `FxHasher`）をそのまま流用し、
/// 追加の型・追加のハッシャ実装は導入しない。
///
/// # 正規入力が誤って衝突判定される確率
///
/// 一次ハッシュは 64bit 空間へ写像するため、意図しない偶発的な衝突が
/// `n` 個のキーの間で 1 件でも起きる確率は誕生日近似で `n^2 / 2 / 2^64`。
/// [`crate::keyed::MAX_KEYED_LIST_ITEMS`]（`n = 4096`）を代入すると
/// 約 `4.5 * 10^-13` であり、実運用上は無視できる水準に留まる（意図的な
/// 攻撃者が算出した衝突ペアは別として、通常のアプリ入力が誤って
/// [`KeyHashCollisionError`] になる懸念はない）。
///
/// # 決定性への影響（イテレートしない不変条件を継承）
///
/// `keyed.rs` の使用箇所はいずれも `get`/`insert`/`get_mut`/
/// `get_or_insert_with` のみを使い、内側 `HashMap` を**イテレートしない**
/// （`fx_hash` モジュール doc「追加防御」節が定める不変条件をそのまま
/// 継承する）。このため一次ハッシュ・内側マップいずれの衝突分布も
/// `diff_keys`/`diff_keyed_items` の発行 op 順序・SSR/SSG 出力バイトへ
/// 影響しない。
///
/// # `S` を型パラメータにする理由（テスト用ハッシャ注入）
///
/// 既定は [`FxBuildHasher`] だが、テストコードから「全キーが同一ハッシュ
/// 値になる」定数ハッシャを注入できるよう `S: BuildHasher` をジェネリクス
/// として露出する（[`Self::with_capacity_and_hasher`]）。本番経路
/// （`keyed.rs`）は常に既定の [`FxBuildHasher`] を使う
/// （[`Self::with_capacity`]）。
pub(crate) struct FxStrMap<'a, V, S = FxBuildHasher> {
    entries: FxHashMap<u64, (&'a str, V)>,
    hasher: S,
}

impl<'a, V> FxStrMap<'a, V, FxBuildHasher> {
    /// 既定ハッシャ（[`FxBuildHasher`]）で容量指定付きの空マップを作る。
    #[inline]
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, FxBuildHasher::default())
    }
}

impl<'a, V, S: std::hash::BuildHasher> FxStrMap<'a, V, S> {
    /// 一次ハッシュに使うハッシャ `S` を明示指定して空マップを作る
    /// （テスト用の衝突ハッシャ注入経路、型 doc 参照）。
    #[inline]
    pub(crate) fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        // テスト専用の構築回数カウンタ（イシュー #1376、`build_counter`
        // doc 参照）。prod ビルドには一切残らない。
        #[cfg(test)]
        build_counter::increment();
        FxStrMap {
            entries: map_with_capacity(capacity),
            hasher,
        }
    }

    /// `key` の一次ハッシュ（`u64`）を計算する。
    #[inline]
    fn primary_hash(&self, key: &str) -> u64 {
        self.hasher.hash_one(key)
    }

    /// `key` に対応する値への参照を返す。
    ///
    /// スロットが空なら `Ok(None)`。スロットが埋まっており格納済みキーが
    /// `key` と一致すれば `Ok(Some(&value))`。格納済みキーが `key` と
    /// 不一致（一次ハッシュの衝突）なら [`KeyHashCollisionError`]。
    #[inline]
    pub(crate) fn get(&self, key: &str) -> Result<Option<&V>, KeyHashCollisionError> {
        match self.entries.get(&self.primary_hash(key)) {
            None => Ok(None),
            Some((stored_key, value)) if *stored_key == key => Ok(Some(value)),
            Some(_) => Err(KeyHashCollisionError),
        }
    }

    /// `key` に対応する値への可変参照を返す（[`Self::get`] の `&mut` 版）。
    #[inline]
    pub(crate) fn get_mut(&mut self, key: &str) -> Result<Option<&mut V>, KeyHashCollisionError> {
        match self.entries.get_mut(&self.primary_hash(key)) {
            None => Ok(None),
            Some((stored_key, value)) if *stored_key == key => Ok(Some(value)),
            Some(_) => Err(KeyHashCollisionError),
        }
    }

    /// `key` に `value` を関連付ける。
    ///
    /// スロットが空なら新規挿入して `Ok(None)`。格納済みキーが `key` と
    /// 一致すれば値を上書きして `Ok(Some(旧値))`（通常の `HashMap::insert`
    /// と同じ同一キー上書きセマンティクス）。格納済みキーが `key` と不一致
    /// （一次ハッシュの衝突）なら **値を書き換えずに**
    /// [`KeyHashCollisionError`] を返す。
    #[inline]
    pub(crate) fn insert(
        &mut self,
        key: &'a str,
        value: V,
    ) -> Result<Option<V>, KeyHashCollisionError> {
        use std::collections::hash_map::Entry;
        match self.entries.entry(self.hasher.hash_one(key)) {
            Entry::Vacant(slot) => {
                slot.insert((key, value));
                Ok(None)
            }
            Entry::Occupied(mut slot) => {
                if slot.get().0 == key {
                    let (_, old_value) = std::mem::replace(slot.get_mut(), (key, value));
                    Ok(Some(old_value))
                } else {
                    Err(KeyHashCollisionError)
                }
            }
        }
    }

    /// `key` の値が既にあればその可変参照を返し、なければ `default()` で
    /// 生成した値を挿入してその可変参照を返す（`entry(..).or_insert_with`
    /// 相当。`keyed.rs` の `queue` 構築で使う）。
    #[inline]
    pub(crate) fn get_or_insert_with(
        &mut self,
        key: &'a str,
        default: impl FnOnce() -> V,
    ) -> Result<&mut V, KeyHashCollisionError> {
        let slot = self
            .entries
            .entry(self.hasher.hash_one(key))
            .or_insert_with(|| (key, default()));
        if slot.0 == key {
            Ok(&mut slot.1)
        } else {
            Err(KeyHashCollisionError)
        }
    }
}

/// [`FxStrMap`] の集合版（`FxStrMap<'a, ()>` の薄いラッパではなく、
/// `contains`/`insert` の呼び出し側可読性のため専用の型として提供する）。
/// 設計・脅威モデルは [`FxStrMap`] の型 doc と完全に同一。
pub(crate) struct FxStrSet<'a, S = FxBuildHasher> {
    entries: FxHashMap<u64, &'a str>,
    hasher: S,
}

impl<'a> FxStrSet<'a, FxBuildHasher> {
    /// 既定ハッシャ（[`FxBuildHasher`]）で容量指定付きの空集合を作る。
    #[inline]
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, FxBuildHasher::default())
    }
}

impl<'a, S: std::hash::BuildHasher> FxStrSet<'a, S> {
    /// 一次ハッシュに使うハッシャ `S` を明示指定して空集合を作る
    /// （テスト用の衝突ハッシャ注入経路、[`FxStrMap`] 型 doc 参照）。
    #[inline]
    pub(crate) fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        // テスト専用の構築回数カウンタ（イシュー #1376、`build_counter`
        // doc 参照）。prod ビルドには一切残らない。
        #[cfg(test)]
        build_counter::increment();
        FxStrSet {
            entries: map_with_capacity(capacity),
            hasher,
        }
    }

    /// `key` が集合に含まれるかを返す（[`FxStrMap::get`] と同じ衝突判定）。
    #[inline]
    pub(crate) fn contains(&self, key: &str) -> Result<bool, KeyHashCollisionError> {
        match self.entries.get(&self.hasher.hash_one(key)) {
            None => Ok(false),
            Some(stored_key) if *stored_key == key => Ok(true),
            Some(_) => Err(KeyHashCollisionError),
        }
    }

    /// `key` を集合へ追加する。新規追加なら `Ok(true)`、既に同一文字列の
    /// キーが存在していたなら追加せず `Ok(false)`（`HashSet::insert` と
    /// 同じ戻り値セマンティクス）。一次ハッシュが衝突した場合（格納済み
    /// キーが `key` と不一致）は [`KeyHashCollisionError`]。
    #[inline]
    pub(crate) fn insert(&mut self, key: &'a str) -> Result<bool, KeyHashCollisionError> {
        use std::collections::hash_map::Entry;
        match self.entries.entry(self.hasher.hash_one(key)) {
            Entry::Vacant(slot) => {
                slot.insert(key);
                Ok(true)
            }
            Entry::Occupied(slot) => {
                if *slot.get() == key {
                    Ok(false)
                } else {
                    Err(KeyHashCollisionError)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`map_with_capacity`] が通常の get/insert 操作で機能することを
    /// 確認する（[`FxStrMap`]/[`FxStrSet`] が内側マップとして使う際の
    /// 最小再現。ネイティブ・wasm32 いずれのハッシャ実体でも成立する）。
    #[test]
    fn map_basic_operations() {
        let mut map: FxHashMap<&str, usize> = map_with_capacity(4);
        map.insert("a", 0);
        map.insert("b", 1);
        assert_eq!(map.get("a"), Some(&0));
        assert_eq!(map.get("z"), None);
    }

    /// テスト専用 `BuildHasher`: 入力に関わらず常に同一の `u64` を返す
    /// （全キーが同一バケットへ強制的に衝突する状況を [`FxStrMap`]/
    /// [`FxStrSet`] に対して再現するための注入用ハッシャ、PR #1390
    /// codex-review 第 2 巡 P1 是正のテスト）。
    #[derive(Clone, Default)]
    struct ConstantHasher;

    struct ConstantHasherImpl;

    impl std::hash::Hasher for ConstantHasherImpl {
        fn write(&mut self, _bytes: &[u8]) {}
        fn finish(&self) -> u64 {
            42
        }
    }

    impl std::hash::BuildHasher for ConstantHasher {
        type Hasher = ConstantHasherImpl;
        fn build_hasher(&self) -> Self::Hasher {
            ConstantHasherImpl
        }
    }

    /// (a) [`FxStrMap`]: 一次ハッシュが衝突する異なる文字列キーは
    /// [`KeyHashCollisionError`] で拒否される。
    #[test]
    fn str_map_rejects_different_keys_that_collide_on_primary_hash() {
        let mut map: FxStrMap<'_, usize, ConstantHasher> =
            FxStrMap::with_capacity_and_hasher(4, ConstantHasher);
        assert_eq!(map.insert("alpha", 1), Ok(None));
        // "beta" は ConstantHasher の下で "alpha" と同一の一次ハッシュ値に
        // なるが、格納済み文字列は "alpha" のため不一致 = 衝突として拒否
        // される。
        assert_eq!(map.insert("beta", 2), Err(KeyHashCollisionError));
        assert_eq!(map.get("beta"), Err(KeyHashCollisionError));
        assert_eq!(map.get_mut("beta"), Err(KeyHashCollisionError));
        assert_eq!(
            map.get_or_insert_with("beta", || 99),
            Err(KeyHashCollisionError)
        );
        // 衝突拒否後も先着の "alpha" エントリは書き換えられていない
        // （fail-closed: 拒否時に既存状態を変更しない）。
        assert_eq!(map.get("alpha"), Ok(Some(&1)));
    }

    /// (b) [`FxStrMap`]: 一次ハッシュが衝突しても同一文字列キーであれば
    /// 従来の `HashMap` と同じ上書き・参照セマンティクスのまま動作する。
    #[test]
    fn str_map_same_key_reuses_normal_semantics_even_under_forced_collision() {
        let mut map: FxStrMap<'_, usize, ConstantHasher> =
            FxStrMap::with_capacity_and_hasher(4, ConstantHasher);
        assert_eq!(map.insert("alpha", 1), Ok(None));
        assert_eq!(map.insert("alpha", 2), Ok(Some(1)));
        assert_eq!(map.get("alpha"), Ok(Some(&2)));
        // "missing" は未挿入だが `ConstantHasher` の下では "alpha" と同一
        // スロットへ写像されるため、空スロットの `Ok(None)` ではなく
        // 衝突として `Err` になる（これは意図した仕様どおりの挙動: 空か
        // どうかの判定もスロット占有状況に依存するため）。

        let value = map.get_or_insert_with("alpha", || 999).unwrap();
        assert_eq!(*value, 2);
        *value = 3;
        assert_eq!(map.get("alpha"), Ok(Some(&3)));
    }

    /// (a) [`FxStrSet`]: 一次ハッシュが衝突する異なる文字列キーは
    /// [`KeyHashCollisionError`] で拒否される。
    #[test]
    fn str_set_rejects_different_keys_that_collide_on_primary_hash() {
        let mut set: FxStrSet<'_, ConstantHasher> =
            FxStrSet::with_capacity_and_hasher(4, ConstantHasher);
        assert_eq!(set.insert("alpha"), Ok(true));
        assert_eq!(set.insert("beta"), Err(KeyHashCollisionError));
        assert_eq!(set.contains("beta"), Err(KeyHashCollisionError));
        // 衝突拒否後も先着の "alpha" は影響を受けない。
        assert_eq!(set.contains("alpha"), Ok(true));
    }

    /// (b) [`FxStrSet`]: 一次ハッシュが衝突しても同一文字列キーであれば
    /// 従来の `HashSet` と同じ重複判定セマンティクスのまま動作する。
    #[test]
    fn str_set_same_key_reuses_normal_semantics_even_under_forced_collision() {
        let mut set: FxStrSet<'_, ConstantHasher> =
            FxStrSet::with_capacity_and_hasher(4, ConstantHasher);
        assert_eq!(set.insert("alpha"), Ok(true));
        assert_eq!(set.insert("alpha"), Ok(false));
        assert_eq!(set.contains("alpha"), Ok(true));
        // "missing" は "alpha" と同一スロットへ写像されるため衝突として
        // `Err` になる（上の [`FxStrMap`] 版テストの注記と同じ理由）。
        assert_eq!(set.contains("missing"), Err(KeyHashCollisionError));
    }
}

/// wasm32-unknown-unknown 専用 [`backend::FxHasher`] の単体テスト。
/// ネイティブビルド（`cargo test -p fandhe-frontend-core`）では一切
/// コンパイルされない（`cfg(all(target_arch = "wasm32", target_os =
/// "unknown"))` の実装自体がネイティブ・wasm32-wasi 等には存在しないため。
/// 判定条件を `target_os = "unknown"` まで絞り込む理由はモジュール doc
/// 「cfg 判定はブラウザ実行に限定する」節参照）。
/// `clippy-wasm32` ジョブ（`.claude/rules/ci.md` 参照）はこの実装を
/// コンパイルレベルで検証するが、`cargo test` を wasm32 target で実行する
/// CI ジョブは現状存在しないため、本テストの実行は wasm-pack 等の
/// ブラウザ/Node ハーネスをローカルまたは将来の CI で用いる場合に限られる
/// （既知の制約。std 自身のターゲット限定コードと同じ扱い）。
#[cfg(all(test, target_arch = "wasm32", target_os = "unknown"))]
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
