# wasm 向けアロケータ差し替え（dlmalloc → 軽量アロケータ）の導入評価

## 背景・トレーサビリティ

- ルート: #1371（CSR 性能・wasm payload 改善トラッキング）
- 親: #1386（Phase 4: wasm payload 削減）
- 本イシュー: #1389（アロケータ差し替えの導入評価。実装・依存追加はしない）

`#1371` 本文の twiggy 実測で、dlmalloc（wasm32-unknown-unknown の既定アロケータ）
が CSR wasm payload raw の約 9.4%（約 8.0KB）を占めることが判明していた。
本イシューは軽量アロケータ（lol_alloc / talc）への差し替えでこの領域を削減
できるかを評価する。`wee_alloc` は unmaintained（後述）のため候補から除外
する。

先行 2 件（`docs/ci/wasm-opt-adoption-evaluation.md` の #1387・#1388）と同じ
計測手順・判定基準（プロファイル/内訳分析の実測、create/update/clear の
op_ms ±5% 目安）を踏襲する。**成果物は本評価文書のみであり、依存クレート
追加・アロケータ差し替え実装は一切コミットしない**（依存追加はユーザー
承認事項、REQ-3・`.claude/rules/coding-rust.md`）。

## 候補

| 候補 | 最新バージョン | 最終リリース | 保守状況 |
|---|---|---|---|
| lol_alloc | 0.4.1 | 2024-02-24 | 最終リリースから約 2 年半、以降の更新なし |
| talc | 5.0.4 | 2026-06-18 | 直近まで継続的にリリース（v5 系がメジャー刷新済み） |
| wee_alloc | （除外） | - | RUSTSEC-2022-0054（`informational = "unmaintained"`）。メモリリーク既知問題あり、maintainer 自身が「wasm32 既定アロケータへの切替を推奨」と明記 |

（バージョン・リリース日は crates.io sparse index を本評価実施時
〔2026-08-24〕に直接照会して確認した実測値。`version-bump-guard` と同じ
`https://index.crates.io` 照会方式）

## 計測環境

- 計測対象: `bench/csr/fandhe/`（bench 専用の独立ワークスペース、glue
  クレート `fandhe-bench-csr-wasm`）
- ツール: `cargo build --target wasm32-unknown-unknown --release` →
  `wasm-bindgen --target web`（0.2.127）→ `wasm-opt -Os`（116、`build.sh`
  の既定パイプラインをそのまま使用）
- payload 計測: `bench/payload/measure.mjs --framework fandhe`（raw / gzip -9、
  1 回実行で決定的）
- 実行時間計測: `bench/csr/run_csr.mjs --framework fandhe`（create/update/clear
  の `op_ms`、システム chromium 151.0.7922.108、`--no-sandbox`）を各構成
  5 回実行し mean を比較（#1387/#1388 と同一手順）
- 内訳分析: `wasm-opt -Os -g` で name section 付き変種を生成し
  `twiggy top -n 3000` で dlmalloc 名前空間の shallow bytes を合算
- 依存グラフ: `cargo metadata` / `cargo tree`
- advisories: プリビルト cargo-deny 0.19.8（`tools/ci/ensure-gate-tools.sh`
  の pin と同一バージョン）+ ルート `deny.toml` で `cargo deny check advisories`
  を実行（RustSec advisory-db への到達性あり、正常応答を確認）
- すべて一時変更（`bench/csr/fandhe/Cargo.toml` / `Cargo.lock` / `src/lib.rs`
  への `#[global_allocator]` 一時追加）として計測し、計測ごとに
  `git checkout -- bench/csr/fandhe` で復元した（本 PR には一切含まれない。
  `git status` で作業ツリーがクリーンであることを最終確認済み）

## dlmalloc の内訳（ベースライン、twiggy）

`wasm-opt -Os -g` 変種（named wasm、121,513B）の twiggy top 3000 件から
`dlmalloc[...]::` 名前空間の shallow bytes を合算すると **7,029B（named wasm
比 5.8%）** だった。`malloc` 単体で 4,626B（3.81%）と最大の単一シンボル。
`#1371` 本文の実測（raw wasm 比 9.4%・約 8.0KB）とはビルド条件（プロファイル・
最適化フラグの版数差）が異なるためオーダーは一致するが数値は完全一致しない。
いずれにせよ dlmalloc が payload の無視できない割合を占めることは本評価でも
再確認できた。

## payload 実測（`bench/payload/measure.mjs --framework fandhe`）

| 構成 | wasm raw | wasm gzip | 合計 raw | 合計 gzip | wasm gzip 差分 |
|---|---:|---:|---:|---:|---:|
| dlmalloc（現状、ベースライン） | 98,680 B | 41,348 B | 107,046 B | 44,353 B | - |
| lol_alloc 0.4.1（`AssumeSingleThreaded<FreeListAllocator>`） | 91,227 B | 38,215 B | 99,593 B | 41,220 B | **-3,133 B（-7.6%）** |
| talc 5.0.4（`talc::wasm::WasmDynamicTalc`） | 92,477 B | 38,779 B | 100,843 B | 41,784 B | **-2,569 B（-6.2%）** |

いずれも判定基準（追加削減 5% 以上）を上回る payload 削減を確認した。ただし
`wasm-opt-adoption-evaluation.md` の wasm-opt 導入時（追加 16.0% 減）と比較
すると削減幅は小さい。

## CSR 実行時間実測（`bench/csr/run_csr.mjs --framework fandhe`、5 回 mean）

| 経路 | dlmalloc（baseline） | lol_alloc | talc |
|---|---:|---:|---:|
| create_op_ms | 3.247 ms | 3.205 ms（-1.3%） | 3.383 ms（**+4.2%**） |
| update_op_ms | 1.937 ms | 2.025 ms（**+4.5%**） | 2.070 ms（**+6.9%、誤差範囲超過**） |
| clear_op_ms | 1.281 ms | 1.188 ms（-7.2%） | 1.278 ms（-0.2%） |

各構成 `rows_ok` / `escape_ok` は全 5 回 PASS（既定エスケープの代理検証も
問題なし）。update 経路（keyed diff の HashMap/Vec 確保・解放が最も密な
経路、`crates/core/src/keyed.rs` の `insert_or_move_pass` 等）が両候補とも
他経路より悪化幅が大きく、事前の予想（「断片化・確保速度の実挙動」評価軸）
どおりの結果だった。

- **talc**: update_op_ms が +6.9% と ±5% 目安を明確に超過。`WasmDynamicTalc`
  （`WasmGrowAndClaim` ソース）は OOM のたびに新規ヒープを確保する設計
  （crate doc: 「常に新規ヒープを作る。前ヒープを extend しない設計のため
  fragmentation は増えるが wasm module サイズは小さくなる」）であり、
  keyed diff の頻繁な確保・解放パターンとの相性が payload 削減の効果を
  相殺している可能性がある。
- **lol_alloc**: update_op_ms +4.5% は ±5% 目安の範囲内。ただし
  `FreeListAllocator` は O(free list 長) の確保・解放特性を持つ設計であり、
  本評価で使用したワークロード（100 行）より大きい一覧では悪化が顕在化
  する可能性がある（本評価では未検証）。

## 依存グラフ影響

`bench/csr/fandhe/` は独立ワークスペース（`[workspace] members = ["."]`）
であり、root ワークスペース（`members = ["crates/*"]`、標準サーバー構成の
依存上限計測対象）には一切波及しない。参考として一時追加時の推移的依存は
以下のとおり（いずれも `build.rs` なし、`cargo metadata` で確認）。

| 候補 | 追加パッケージ | 内訳 |
|---|---|---|
| lol_alloc 0.4.1 | 4 件 | `lol_alloc` → `spin` → `lock_api` → `scopeguard` |
| talc 5.0.4 | 4 件 | `talc` → `allocator-api2` / `lock_api` → `scopeguard` |

いずれも既存の `bench/csr/fandhe/` 依存総数 28 件（一時追加後）に収まり、
「標準サーバー構成（root workspace、REQ-3 の 60 件/深さ 6 上限）へは wasm
限定依存が波及しない」ことを確認した。

## cargo-deny advisories 照合

一時追加状態のワークスペースでプリビルト cargo-deny（0.19.8、
`tools/ci/ensure-gate-tools.sh` と同一 pin）+ ルート `deny.toml` を用いて
`cargo deny check advisories` を実行した。lol_alloc / talc いずれの構成も
`advisories ok`（違反 0 件）だった。wee_alloc は候補調査段階で
RUSTSEC-2022-0054（unmaintained・メモリリーク既知問題）が確認できたため、
本節の一時導入検証の対象からそもそも除外している。

## unsafe 境界の評価（採用時の適用範囲設計に直結する差異）

`bench/csr/fandhe/src/lib.rs` は `#![deny(unsafe_code)]`（`crates/wasm-client`
と同方針、`#[wasm_bindgen]` 展開コードが内部 `unsafe` を含むため `forbid`
ではなく `deny`）を既に採用している。この lint 下で両候補を実装した結果、
**利用側コードでの unsafe 必要性に明確な差**が出た。

- **lol_alloc**: `AssumeSingleThreaded::new(...)` の呼び出しに
  `unsafe { ... }` ブロックが必須（`AssumeSingleThreaded` は非 `Sync` な
  `FreeListAllocator` を「実際には単一スレッドでしか使わない」という
  利用者の保証のもとで `Sync` とみなすラッパのため、構造的に unsafe を
  要求する設計）。`#![deny(unsafe_code)]` のまま素朴に実装するとコンパイル
  エラーになり、`#[allow(unsafe_code)]` によるローカルなオプトアウトが
  必要だった（本評価で実際に発生・回避を確認済み）。
- **talc**: `talc::wasm::new_wasm_dynamic_allocator()`（`WasmDynamicTalc`
  を返す `pub const fn`）は関数シグネチャ自体が `unsafe` ではなく、内部の
  `unsafe` 実装（`Claim`・`TalcSyncCell` 等）はすべて talc クレート内に
  完全にカプセル化されている。`#![deny(unsafe_code)]` を変更せず、
  利用側コードに `unsafe` を一切書かずにコンパイルが通ることを確認した
  （`#[allow(unsafe_code)]` 等のオプトアウトなし）。

この差は「Step 6 の適用範囲設計」に直結する: talc の `WasmDynamicTalc` 経路
は `crates/wasm-client` が CI `forbid-unsafe` ジョブ（`#![deny(unsafe_code)]`
+ 自作 unsafe 0 件の機械強制）で守っている契約と構造的に両立するが、
lol_alloc は現行 CI 契約と衝突するため、案 B（`crates/wasm-client` の
opt-in feature としての組み込み）は lol_alloc では成立せず、案 A（leaf
glue クレート限定・`#[allow(unsafe_code)]` の局所オプトアウト前提）でしか
採用できない。

## 適用範囲の設計比較（採用時、本イシューでは実装しない）

- **案 A: leaf glue クレート側にのみ配置**（`bench/csr/fandhe` /
  `templates/app/wasm` / `examples/interactive-view-transitions/wasm`）。
  いずれも独立ワークスペースのため root 依存グラフ・dist-server 配布
  バイナリへ構造的に波及しない（本評価で実測確認済み）。lol_alloc 採用時は
  `templates/app` が生成するユーザープロジェクトへ `#[allow(unsafe_code)]`
  を配ることになる点が案 A でも残るデメリット。talc（`WasmDynamicTalc`）
  採用時はこの懸念がない。
- **案 B: `crates/wasm-client` の opt-in feature**（`#[cfg(all(target_arch =
  "wasm32", feature = "..."))]`）。root workspace へ依存が入るため
  check-deps・`docs/policy/unsafe-boundary.md`・CI `forbid-unsafe` ジョブとの
  整合が必要。talc の `WasmDynamicTalc` は unsafe を要求しないため理論上
  成立し得るが、lol_alloc は成立しない（前節参照）。
- いずれの案でも `#[global_allocator]` は wasm32 限定 cfg・独立ワークスペース
  隔離により配布サーバーバイナリ（`crates/dist-server`）へ波及しないことを
  不変条件とする（本評価では dist-server 側の変更は一切行っていない）。

## 採否判定

**現時点では両候補とも見送りを推奨する**。

- **talc**: 保守状況・unsafe 境界の両面で最も有望（安全な wasm 専用 API・
  直近まで活発なリリース）だが、**update_op_ms が +6.9% と判定基準（±5%
  目安）を明確に超過**したため、`docs/ci/wasm-opt-adoption-evaluation.md`
  の #1387 節と同じ「payload 削減は確認できたが実行時間の悪化を理由に
  不採用」という判断を踏襲する。
- **lol_alloc**: payload 削減（-7.6%）・op_ms（update +4.5%、辛うじて
  ±5% 目安の範囲内）の両方は判定基準を満たすが、(1) 利用側コードに
  `unsafe` を要求し `#![deny(unsafe_code)]` との衝突を local
  `#[allow(unsafe_code)]` でしか回避できない、(2) 最終リリースが
  2024-02-24 で 2 年半以上更新がなく `wee_alloc` と同様の unmaintained
  リスクを将来抱える懸念がある、(3) `FreeListAllocator` の O(free list 長)
  特性は本評価のワークロード（100 行）でしか検証しておらず、より大きい
  一覧規模での悪化再現性が未検証、の 3 点を理由に、数値基準を満たすのみ
  では採用を推奨しない。

いずれも「別イシューでユーザー承認を得たうえでの採用」に進める根拠が
現時点では不足していると判断する。

## 再評価トリガー

- talc: `WasmGrowAndClaim` 以外のヒープ管理ソース（例: 固定サイズ
  arena を `Claim::array` で事前確保する構成）や `WasmBinning` 以外の
  binning 設定で update_op_ms の悪化が ±5% 目安に収まることが実測で
  確認できた場合
- lol_alloc: 保守が再開され（新規リリース・issue 対応の実績）
  unmaintained リスクが解消した場合、または `#![deny(unsafe_code)]` を
  維持したまま unsafe を要求しない代替 API が提供された場合
- Vue 水準（gzip 約 22KB）到達を明確な目標とする場合で、かつ他の
  payload 削減余地（`docs/ci/wasm-opt-adoption-evaluation.md` の
  「コード側削減の見送り」節で先送りした内訳分析）を先に消化してもなお
  不足する場合
- 候補クレートのメンテ状況・stable での代替手段（例: nightly
  `panic_immediate_abort` の stable 化に伴うアロケータ側最適化の登場）に
  大きな変化があった場合
