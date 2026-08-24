# wasm payload ベースラインとハッシャ化効果（#1375）の実測レポート

## 1. 目的とトレーサビリティ

- 本イシュー: #1406（本レポート作成対象）
- 親: #1405（wasm payload 削減の継続。gzip ≈41KB → ≈20KB 目標。「ハッシャ化
  （#1375 で SipHash 脱却済み）の payload 効果の実測確認（見積もり −6〜8KB
  の検証）」を受け入れ条件の 1 つとして明記）
- 検証対象: #1375（実装コミット `7801ba1`、PR #1390「keyed diff の
  HashMap/HashSet を軽量ハッシャ化する（SipHash 脱却）」）。PR #1390 本文の
  「対象外」節に「受け入れ条件 3 の bench-cross（npm/playwright）実測・
  payload gzip サイズ実測は環境制約により未実施」と明記されており、
  payload への実効果は今回が初回実測となる
- 参照: #1371（親トラッキング。twiggy 実測で「hashbrown+SipHash が payload
  の 12.8%（11.0KB）」と記録）/ #1402（panic・fmt 縮減、直近の HEAD ベース
  ライン記録元）

本レポートはコード変更を一切伴わない実測記録のみである（`docs/reports/`
新規追加 1 ファイル）。

## 2. 計測環境・再現手順

| 項目 | 値 |
|------|-----|
| OS | Linux 7.0.0-29-generic（Ubuntu 系） |
| CPU | 12 vCPU（QEMU Virtual CPU version 2.5+、仮想化） |
| rustc | 1.96.0 (ac68faa20 2026-05-25) |
| wasm-bindgen-cli | 0.2.127（`csr/fandhe/Cargo.lock` の pin と一致） |
| wasm-opt（binaryen） | version 116（`build.sh` の `WASM_OPT_EXPECTED_VERSION` と一致） |
| twiggy | twiggy-opt 0.8.0 |
| Node.js | v24.13.0 |
| 計測日 | 2026-08-24 |
| HEAD コミット | `1a67469`（docs(ci): wasm 向けアロケータ差し替えの導入評価を記録する、#1404） |

### 2.1 実行コマンド（HEAD ベースライン）

```bash
bash bench/csr/fandhe/build.sh                    # -Os 配布物（正）
node bench/payload/measure.mjs --framework fandhe  # raw/gzip 実測
```

`meta.json` の `wasm_opt` は `"116"`（`"skipped"` ではない）ことを確認済み
=production 相当ビルドでの計測。

### 2.2 twiggy 用名前付き変種の生成

```bash
cargo build --manifest-path bench/csr/fandhe/Cargo.toml \
  --target wasm32-unknown-unknown --release   # 専用 CARGO_TARGET_DIR 指定
wasm-bindgen --target web --out-dir <scratch> --out-name fandhe_bench <artifact>
wasm-opt -Os -g <bindgen 出力>/fandhe_bench_bg.wasm -o <scratch>/named.wasm
twiggy top -n 3000 <scratch>/named.wasm
```

（手順は `docs/ci/wasm-allocator-adoption-evaluation.md` / `wasm-opt-adoption-evaluation.md`
の先例を踏襲。wasm-opt は既定で name section を落とすため `-g` 付き変種を
別途生成する。`-Os -g` の named wasm はサイズが配布物の正〔`-Os` のみ〕とは
異なるため、payload の正の値には使わず、twiggy 内訳の相対比較専用とする）

### 2.3 #1375 単独効果の before/after 分離

`git archive b20d642`（#1375 直前、bench/ v2 導入コミット）と
`git archive 7801ba1`（#1375 実装コミット）をそれぞれセッション専用
スクラッチパッド配下の隔離ディレクトリへ展開し、2.1/2.2 と同一手順・
同一ツールバージョン（wasm-bindgen 0.2.127・wasm-opt 116）で計測した
（メイン worktree には触れない。計測後 `git status` で
`bench/csr/fandhe/Cargo.lock` の一時更新差分を確認し `git checkout --`
で復元済み。最終的な worktree は本レポート追加のみのクリーンな状態）。

`bench/csr/` の `package-lock.json` は両コミットと HEAD で完全一致してい
たため、`npm ci --ignore-scripts` 済みの `node_modules` を隔離コピーへ
複製して再利用した（esbuild の minify 条件は不変）。両コミットとも
`cargo build` 時に path 依存クレートのバージョンが `Cargo.lock` へ再解決
された（コミット当時の crates.io 未公開段階の path 依存のため、workspace
内の現在バージョンへロックが自動更新される）。これは各コミット当時の
実装内容をそのままビルドした結果であり、計測条件の恣意的な変更ではない。

### 2.4 既知の制約

- payload 計測は fandhe-frontend 単独（`--framework fandhe`）の部分実行。
  他 6 フレームワークの dist は対象外（`bench/PROTOCOL.md` §2.2 が明示的に
  許容する経路）
- CSR 実行時間（op_ms）は本レポートの対象外（#1405 の受け入れ条件は
  payload 実測であり、bench/csr/run_csr.mjs による実行時間比較は本イシュー
  のスコープ外）
- twiggy の `-Os -g` named wasm は配布物の正（`-Os` のみ）よりサイズが
  大きい（name section 分）。twiggy 内訳の相対比較・カテゴリ内訳の妥当性
  確認にのみ用い、payload 実数値としては §3/§4 の `-Os` 計測値を正とする

## 3. 現行ベースライン（HEAD、`1a67469`）

`node bench/payload/measure.mjs --framework fandhe` の実測 [再計測]:

| ファイル | raw | gzip |
|---|---:|---:|
| bootstrap.js | 421 B | 204 B |
| fandhe_bench.js（wasm-bindgen glue、minify 済み） | 7,945 B | 2,801 B |
| fandhe_bench_bg.wasm（wasm-opt -Os 済み） | 98,683 B | 41,345 B |
| **配布合計** | **107,049 B** | **44,350 B** |

`meta.json`: `{"framework":"fandhe-frontend","version":"0.6.1","wasm_opt":"116"}`

#1405 本文記録値（wasm gzip 41,348B / 配布合計 gzip 44,353B、#1402 マージ
直後の記録、[issue 記録値・未再計測] として引用）との差はごくわずか
（wasm gzip -3B、合計 gzip -3B）。#1402〜HEAD 間の `1a67469`（#1404、ドキュ
メントのみ）はコード変更を伴わないため、この程度の誤差は再ビルド・依存
バージョン再解決に伴う正常なノイズと判断できる。

## 4. twiggy 内訳（HEAD、named wasm 121,516 B）

上位抜粋 [再計測]:

| Shallow Bytes | % | シンボル |
|---:|---:|---|
| 22,644 | 18.63% | "function names" subsection |
| 19,354 | 15.93% | `wasm_client::keyed_dom::apply_keyed_list_core` |
| 4,626 | 3.81% | `dlmalloc::malloc` |
| 4,065 | 3.35% | `core::keyed::insert_or_move_pass` |
| 3,177 | 2.61% | `core::slice::sort::unstable::quicksort` |
| 3,040 | 2.50% | `wasm_client::keyed_apply::diff_children_core` |
| 2,824 | 2.32% | `WebSysKeyedDom::sync_attrs` |
| 2,095 | 1.72% | `hashbrown::map::HashMap::insert`（複数単形化の代表） |

### hashbrown / SipHash 系シンボルの残存

`hashbrown|sip|RandomState` に一致する全シンボルの shallow bytes 合算
（Python によるパース・集計、[再計測]）:

**12,771 B（named wasm 121,516 B 比 10.51%）**

内訳のうち代表的なもの: `hashbrown::map::HashMap::insert` 系（複数単形化
合計 3,033B）・`hashbrown::raw::RawTable::reserve_rehash`（3 単形化合計
4,496B）・`core::hash::sip::Hasher::write`（596B）・
`hashbrown::set::HashSet::extend`（488B）等。

`twiggy paths -d 3` による到達経路の確認 [再計測]:

```
core::hash::sip::Hasher::write
  ⬑ core::hash::BuildHasher::hash_one
      ⬑ hashbrown::raw::RawTable::reserve_rehash
          ⬑ wasm_client::keyed_dom::apply_keyed_list_core
          ⬑ hashbrown::map::HashMap::insert
              ⬑ wasm_client::keyed_dom::apply_keyed_list_core
              ⬑ wasm_client::keyed_apply::diff_children_core
```

残存する hashbrown・SipHash は**すべて `crates/wasm-client` の
`keyed_dom::apply_keyed_list_core` / `keyed_apply::diff_children_core`
経由の素の `std::collections::HashMap`/`HashSet`** に帰着する。ソース
grep で確認済みの該当箇所:

- `crates/wasm-client/src/keyed_dom.rs`（`invalidated_nested_fields:
  std::collections::HashSet::new()` 等、複数箇所）
- `crates/wasm-client/src/keyed_apply.rs`（`achieved_nodes:
  std::collections::HashMap<...>`・`old_by_key`/`new_by_key` 等の
  `HashMap`/`HashSet`、複数箇所）
- `crates/wasm-client/src/registry.rs`（`HANDLES: RefCell<HashMap<...>>`。
  本 bench ワークロード〔keyed リストのみ〕では未到達だが常駐する static）
- `crates/wasm-client/src/timer.rs`（`TIMERS: RefCell<HashMap<...>>`。同上）

`crates/core/src/fx_hash.rs` の rustdoc（#1375 実装コメント）に「wasm-full/
wasm-client 側に残存する他の std HashMap は明示的に out-of-scope」と記録
されている内容と、今回の twiggy 実測は完全に一致する。PR #1390 本文の
「対象外」節（`crates/core/src/keyed.rs` のみが対象、wasm-client 側は
計画時点からスコープ外）とも整合する。

## 5. #1375 の payload 効果（before/after 実測）

### 5.1 payload（`-Os` 配布物、[再計測]）

| 構成 | コミット | wasm-client version | wasm raw | wasm gzip | 配布合計 raw | 配布合計 gzip |
|---|---|---|---:|---:|---:|---:|
| before（#1375 直前） | `b20d642` | 0.4.0 | 85,618 B | 37,117 B | 92,981 B | 39,858 B |
| after（#1375 実装） | `7801ba1` | 0.5.0 | 88,077 B | 38,100 B | 95,440 B | 40,841 B |
| **差分（after − before）** | | | **+2,459 B（+2.87%）** | **+983 B（+2.65%）** | **+2,459 B** | **+983 B** |
| 参考: HEAD | `1a67469` | 0.6.1 | 98,683 B | 41,345 B | 107,049 B | 44,350 B |

**#1375 は payload を削減せず、むしろ増加させた。** `b20d642..7801ba1` の
コミット範囲は `7801ba1` 単独（`git log --oneline b20d642..7801ba1` が
1 行のみを返す）であり、この差分は #1375 に一意に帰着できる。

見積もり「−6〜8KB」（#1375 本文の「単形化の重複が減ることで payload
−6〜8KB も期待できる」）は raw ベースの見積もりである（#1371 の
「12.8%（11.0KB）」は raw wasm 約 86KB 中の 11.0KB）。実測を同じ raw
ベースで比較すると: raw 実測 **+2,459 B** に対し raw 見積もり
**−6,000〜−8,000 B** であり、**ギャップは約 +8.5〜10.5KB raw（方向自体が
逆）**。gzip 実測（+983 B、+2.65%）は配布物サイズへの実効果として別途
参照する。

### 5.2 twiggy 内訳の before/after（named wasm、[再計測]）

| 構成 | named wasm | hashbrown+SipHash 合算 | 比率 |
|---|---:|---:|---:|
| before（`b20d642`） | 104,540 B | 10,513 B | 10.06% |
| after（`7801ba1`） | 107,444 B | 10,014 B | 9.32% |
| 差分 | +2,904 B | **-499 B** | -0.74pt |

hashbrown/SipHash 名前空間の shallow bytes **自体は** -499B（-4.7%）と
わずかに減少している（`hashbrown::map::HashMap::insert` の単形化が
3,344B→469B へ縮小、複数箇所の呼び出しが集約された）。しかし増加した
コードがこの縮小を上回った。関数単位でのシンボル差分（`::h` 付きハッシュ
接尾辞を除いて同一関数として集計、shallow bytes 差 ±30B 以上を抽出）:

| 差分 | before | after | シンボル |
|---:|---:|---:|---|
| -2,875 B | 3,344 | 469 | `hashbrown::map::HashMap::insert`（単形化統合） |
| -250 B | 500 | 250 | `hashbrown::map::HashMap::contains_key` |
| +2,045 B | 0 | 2,045 | `hashbrown::rustc_entry::<impl HashMap>::rustc_entry`（新規） |
| +1,295 B | 18,075 | 19,370 | `wasm_client::keyed_dom::apply_keyed_list_core` |
| +701 B | 0 | 701 | `<HashMap as FromIterator>::from_iter`（新規） |
| +524 B | 0 | 524 | `hashbrown::raw::RawTable::insert_no_grow`（新規） |
| +445 B | 18,733 | 19,178 | "function names" subsection |
| +341 B | 985 | 1,326 | `Vec::from_iter`（`SpecFromIterNested`） |
| +226 B | 3,071 | 3,297 | `fandhe_bench_csr_wasm::build_tbody_node` |
| +192 B | 443 | 635 | `core::hash::BuildHasher::hash_one`（FxHash 呼び出し箇所） |
| +117 B | 365 | 482 | `core::keyed::KeyedListError as Debug`（`diff_keys`/`diff_keyed_items` の `Result` 化に伴う新規エラー型） |

**解釈（根本原因）**: 見積もり「−6〜8KB」の前提は「`core::keyed` から
SipHash 実装をリンクから除去できる」ことだった。しかし実測では
`core::hash::sip::Hasher::write` の shallow bytes は **before 596B →
after 596B（バイト完全一致、変化なし）**。SipHash 実装はビルド成果物から
一切外れていない。理由は `crates/wasm-client`（`keyed_dom.rs` /
`keyed_apply.rs` / `registry.rs` / `timer.rs`）が素の
`std::collections::HashMap`/`HashSet`（既定 `RandomState`/SipHash）を
`core::keyed` とは独立に多数使い続けているため（§4 で確認済み。#1375
本文・PR #1390 も計画時点からスコープ外と明記）。**見積もりの前提
（SipHash 実装バイトの回収）はそもそも成立しなかった**。

hashbrown 名前空間全体（SipHash 以外の `HashMap`/`HashSet` 実装コード
含む）の shallow bytes 合算は before 10,513B → after 10,014B と -499B
（-4.7%）縮小しており、`core::keyed` 内部の単形化統合（`HashMap::insert`
の -2,875B）自体は実際に起きている。しかし縮小幅は全体の約 5% に留まり、
(1) `diff_keys`/`diff_keyed_items` が `Result` を返す新シグネチャへ
変わったことに伴うエラー型（`KeyedListError`）と呼び出し側
（`wasm-client`）でのハンドリングコードの追加、(2)
`wasm_client::keyed_dom::apply_keyed_list_core` 自体の増量（+1,295B、
新シグネチャへの適応・`rustc_entry`/`insert_no_grow` という新規 hashbrown
API の呼び出し発生を含む）という適応コストが、この縮小分を上回った。
net で見ると payload は増加に転じている。

## 6. SipHash 残存経路の追加削減提案（起票はしない）

§4 で確認したとおり、残存する hashbrown/SipHash（HEAD で named wasm 比
10.51%、12,771B）はすべて `crates/wasm-client` の素の
`std::collections::HashMap`/`HashSet`（`keyed_dom.rs` / `keyed_apply.rs`
の複数箇所、および本 bench ワークロードでは未到達の `registry.rs` /
`timer.rs`）に由来する。これは `crates/core/src/fx_hash.rs` の rustdoc・
PR #1390 双方が計画時点から明示していた既知のスコープ外である。

`.claude/rules/out-of-scope-tracking.md` に従い、新規 issue の起票は行わず
提案のみ記録する:

- `crates/wasm-client` 側の keyed diff 経路（`keyed_dom.rs` /
  `keyed_apply.rs`）の `HashMap`/`HashSet` を、`crates/core::fx_hash` と
  同種の軽量ハッシャへ横展開する（`core` は外部依存ゼロ・`wasm-client` は
  `core` に依存できるため、`fx_hash` モジュール自体を `core` の公開
  内部 API として re-export するか、同等の実装を `wasm-client` 内へ複製
  する設計判断が必要）。§5.2 の根本原因分析のとおり、`core::keyed` 単独の
  ハッシャ切り替えでは SipHash 実装バイト自体（596B）はリンクから
  外れない。**SipHash 実装バイトを実際に回収するには、最後の std
  `HashMap`/`HashSet` 利用者（`wasm-client` 側）を変換し切る必要がある**
- ただし §5 の実測が示すとおり、**ハッシャ切り替え単体の削減効果
  （-499B、-4.7%）は新規コード（Result 化・エラー型・呼び出し側適応の
  +2,958B）に容易に相殺され得る**ことが #1375 で実証済みであるため、
  再挑戦する場合は「ハッシャ切り替えのみ」に限定した最小差分で実施し、
  シグネチャ変更（`Result` 化等）を同一 PR に混在させない設計とすることが
  望ましい
- `registry.rs` / `timer.rs` の `HashMap` は本 bench ワークロードでは
  到達しない（static な `RefCell<HashMap<...>>`。ハンドラ登録・タイマー
  管理用）ため、payload への寄与は名前空間の常駐コードのみで実行時
  ホットパスではない。優先度は `keyed_dom.rs`/`keyed_apply.rs` 側より
  低いと考えられる

## 7. 結論（#1405 の残レバー判断向け集約）

| 項目 | 値 | provenance |
|---|---:|---|
| 現行 wasm gzip（HEAD, `1a67469`） | 41,345 B | [再計測] |
| 現行配布合計 gzip（HEAD） | 44,350 B | [再計測] |
| #1405 目標 gzip | ≈20,000 B | [issue 記録値] |
| #1375 の payload 実効果（gzip、before→after） | **+983 B（想定 −6〜8KB に対し逆方向）** | [再計測] |
| hashbrown+SipHash 残存（HEAD、named wasm 比） | 12,771 B（10.51%） | [再計測] |
| 残存の帰着先 | `wasm-client::keyed_dom`/`keyed_apply` の素 `HashMap`/`HashSet`（計画時点からスコープ外） | [再計測・ソース照合] |

上表の「hashbrown+SipHash 残存（HEAD）12,771B」は #1375 単独の残置分では
ない点に注意する。`7801ba1`（#1375 直後）時点では 10,014B であり、
HEAD までの後続コミット（#1394 の key map 遅延構築等、hashbrown 使用を
増やし得る変更）で約 +2.7KB 増加した後の値である。#1375 が残した分だけを
指すなら §5.2 の 10,014B（`7801ba1` 時点）を参照する。

#1405 の残レバー一覧（「ハッシャ化（#1375 で SipHash 脱却済み）の payload
効果の実測確認」）は本レポートにより解消した。**実測結果は見積もりと逆
方向（削減ではなく微増）** であり、#1405 の残りの削減余地としてハッシャ
化系列を再度積み増す判断をする場合は、§6 の提案（シグネチャ変更を伴わない
最小差分での再実施）を前提条件とすべきである。target ≈20KB との残差
（現行 41,345B との差、約 21,345B）は、DOM 適用層（`apply_keyed_list_core`
単体で named wasm の 15.93%）のコード削減など他レバーの検討が引き続き
必要である。
