# 性能改善 before/after レポート（イシュー #1313 系列）

## 1. 目的とトレーサビリティ

トラッキング issue #1313（フレームワーク 10 種比較ベンチ起点の性能改善、
2026-08-11 実測開始）配下の Phase 1〜3 改善が完了したことを受け、各改善の
寄与と現在値を記録する（本イシュー #1329）。

| Phase | 親 issue | 内容 |
|-------|----------|------|
| Phase 1 | #1314 | 性能回帰の計測・検知基盤（`xtask bench-ssr` 等の常設） |
| Phase 2 | #1315 | keyed_dom の CSR 適用性能改善 |
| Phase 3 | #1316 | CPU・メモリー・payload・状態変更の横断性能改善 |

| Issue | 内容 | 対応 PR |
|-------|------|---------|
| #1317 | SSR 性能ベンチの常設コマンド | `xtask bench-ssr` |
| #1318 | keyed 適用の DOM 操作コスト固定テスト | (テスト追加、本レポート対象外) |
| #1319 | keyed 挿入位置解決の O(1) 化 | PR #1335 |
| #1320 | 連続 Insert の DocumentFragment 集約 | PR #1339 |
| #1321（#1322/#1323/#1324） | keyed diff への Update op 導入 | PR #1330（設計）/ #1336（core）/ #1340（wasm-client） |
| #1325 | render エスケープ走査の高速化 | PR #1338 |
| #1326 | Node 木構築のアロケーション削減 | PR #1333 |
| #1327 | クライアント payload 削減（wasm-opt 導入） | PR #1334 |
| #1328 | 状態変更負荷のベンチ追加と改善 | PR #1337 |

### 本レポートの数値の出所（provenance）

数値は取得経路によって信頼度・再現性が異なるため、以下 3 段階で明示的に
区別する。すべての表・記述にこの区分ラベルを付す。

- **[再計測]**: 本レポート作成時に本 worktree で `cargo run -p xtask
  --release -- bench-*` を実際に実行して得た値（1 行サマリを本文へ転記）。
- **[既存文書引用]**: `docs/ci/wasm-opt-adoption-evaluation.md` 等、既に
  リポジトリへコミットされた別レポートからの引用値。
- **[issue/PR 記録値・未再計測]**: issue #1313 本文または各 PR 本文に記載
  されていた計測値。本レポート作成時点では再取得していない。

## 2. 計測環境と再現手順・既知の制約

### 2.1 本レポート作成時の計測環境（[再計測] 分のみ）

| 項目 | 値 |
|------|-----|
| OS | Ubuntu 26.04 LTS（Linux 7.0.0-28-generic） |
| CPU | 12 vCPU（仮想化 CPU） |
| rustc | 1.96.0 |
| 実行コマンド | `cargo run -p xtask --release -- bench-ssr` / `bench-state-update` / `bench-binding-update` |
| 実行方式 | `--release` プロファイルでの単発実行（1 コマンド = 1 JSON/サマリ行） |
| 計測日 | 2026-08-12 |

ホスト名・ユーザー名・絶対パス等の環境固有情報は記録しない（`security.md` A02/A05）。

### 2.2 再現手順

- 常設 xtask コマンド（本レポートの [再計測] 分。追加ツール不要、`cargo
  run -p xtask --release -- <subcommand>` のみで再現可能）:
  - `bench-ssr [--baseline <FILE>]`（イシュー #1317・`crates/xtask/src/bench_ssr.rs`）
  - `bench-state-update [--baseline <FILE>]`（イシュー #1328・`crates/xtask/src/bench_state_update.rs`）
  - `bench-binding-update`（イシュー #592・`crates/xtask/src/bench_binding_update.rs`）
- 非追跡ベンチハーネス（`_/bench/`、`_/bench/PROTOCOL.md` に手順記載。
  `run_ssr.py`（フレームワーク横断 SSR 比較）・`run_csr.mjs`（playwright +
  chromium による CSR create/update/clear 実測）・payload gzip 計測）は
  `.gitignore` の `/_/` により git 管理外であり、**本レポート作成に使用した
  git worktree には存在しない**（worktree はメイン working copy の未追跡
  ファイルを共有しない）。このため以下は本レポートでは再実施していない:
  - 他フレームワーク 10 種との同日相対位置の再計測（SSR 11 系・CSR 7 系比較表）
  - CSR create/update/clear のブラウザ実測（`run_csr.mjs`）
  - これらの再計測は「メイン working copy（非隔離環境）で `_/bench/
    PROTOCOL.md` の手順に従い実行する」ことで再現可能。再計測が必要な場合は
    別途 Issue化を検討する（§7）。

### 2.3 受け入れ条件 (a) に対する充足状況

イシュー #1329 の受け入れ条件 (a)「docs/reports/ にレポートが存在し、数値が
ベンチ出力と一致する」について、**[再計測] とラベルした数値は本レポート作成
時のベンチ出力（本文中に転記した 1 行サマリ／JSON）と完全に一致する**。
一方 **[issue/PR 記録値] とラベルした数値は本レポートでは再計測しておらず、
「ベンチ出力との一致」は各記録元（issue #1313 本文・各 PR 本文）の時点での
ものである**。前述 §2.2 の理由（`_/bench/` が本 worktree に不在）により
全項目の同日再計測はできなかったため、受け入れ条件 (a) は [再計測] 区分の
範囲で満たし、[issue/PR 記録値] 区分は「記録の集約」として扱う。

## 3. SSR 性能

### 3.1 [再計測] `xtask bench-ssr`（2026-08-12、fandhe-frontend-core 0.3.0）

```
{"framework":"fandhe-frontend","version":"0.3.0","mode":"ssr","workload_schema_version":1,
 "rows1k":{"iters":100,"mean_ms":0.1102,"p50_ms":0.1034,"p95_ms":0.1406,"min_ms":0.0972},
 "rows10k":{"iters":10,"mean_ms":1.2323,"p50_ms":1.0552,"p95_ms":1.4365,"min_ms":1.0187},
 "html_bytes_1k":118931,"escape_ok":true,"row_count_ok":true,"notes":"profile=release"}
```

| 指標 | 値（[再計測]） |
|------|------|
| rows1k mean | 0.1102 ms |
| rows1k p95 | 0.1406 ms |
| rows10k mean | 1.2323 ms |
| rows10k p95 | 1.4365 ms |
| escape_ok / row_count_ok | true / true（PASS） |

### 3.2 [issue/PR 記録値・未再計測] ベースライン（2026-08-11、issue #1313 本文より）

| 指標 | 値 |
|------|------|
| SSR 1,000 行 | 0.248 ms（比較対象 11 種中 2 位） |
| SSR 10,000 行 | 2.72 ms（比較対象 11 種中 **1 位**） |

計測環境・バージョン（当時の core バージョンは issue 本文に明記なし。
本レポート作成時点の workspace 遷移は core 0.2.0 系 → 0.3.0）・代表値算出
方法（3 run 中最良）が本レポート §2.1 の [再計測] と異なるため、上記 2 値は
単純な数値比較の対象としない。ただし [再計測] 値（rows1k mean 0.1102ms、
rows10k mean 1.2323ms）は同一 `bench_ssr` ハーネスの release ビルド計測で
あり、ベースライン記載値（0.248ms・2.72ms）を下回る方向で推移している
ことから、#1325（render エスケープ走査高速化）・#1326（Node 木構築の
アロケーション削減）の改善が有効に効いていることと整合する。

### 3.3 [issue/PR 記録値・未再計測] PR #1338（#1325 対応）の実測寄与

PR #1338 本文（`xtask bench-ssr` 実行結果、当時の core バージョン 0.2.1）:

- rows1k mean: **-10.9%〜-13.5%**（改善）
- rows10k mean: -9.5%〜+1.9%（10 iters のノイズ範囲内、劣化ではないことを
  PR 本文にて再実行で確認済みと記録）

### 3.4 [issue/PR 記録値・未再計測] PR #1333（#1326 対応）の実測寄与

PR #1333 本文: keyed_list 検証を参照走査 + ムーブ構築の 2 パス構成へ変更し、
属性 Vec・子ノード木の deep clone を全廃。n=1,000/10,000 計測で**約 44% の
アロケーション削減**（core 0.2.1 → パッチバンプ）。

## 4. CSR 性能（keyed リスト適用）

### 4.1 [issue/PR 記録値・未再計測] ベースライン（2026-08-11、issue #1313 本文より）

| 指標 | 値 |
|------|------|
| CSR create（1,000 行） | 22.2 ms（比較対象 7 種中最下位、vanilla の約 14 倍） |
| CSR update | 1.32 ms（競争力あり） |
| CSR clear | 2.36 ms（競争力あり） |

原因分析: `crates/wasm-client/src/keyed_dom.rs` の挿入位置解決が O(index) の
sibling 走査で累積 O(n²)（寄与約 84〜85%、変種計測で裏付け済みと issue 本文
に記録）。

### 4.2 [issue/PR 記録値・未再計測] PR #1335（#1319 対応）の実測寄与

- `nth_element_child`（O(index) sibling 走査）を `KeyedListDom::child_at`
  （web-sys `Element::children().item(index)`）へ置換し O(1) 化
- 1,000 行 create の DOM 操作総呼び出し回数: 実測 **502,501 回 → 3,001 回**
  へ縮小
- `_/bench/csr/fandhe` での実測: create_ms mean **22.2ms → 6.67ms（約 3.3 倍
  短縮）**

### 4.3 [issue/PR 記録値・未再計測] PR #1339（#1320 対応）

連続する Insert 操作を `DocumentFragment` へ集約し DOM 挿入を 1 回にまとめる
変更。PR 本文には「ベンチ実測（`_/bench`）の before/after 実測値取得は本
エージェントの作業時間内では未実施」と明記されており、コスト固定テスト
（native、DOM 操作呼び出し回数の上限契約）による代替検証のみが記録されて
いる。空 → 1,000 行 create の DOM 操作コスト契約は「約 1,003 回呼び出し・
1 回のバッチ挿入」に絞り込み済み（`keyed_apply.rs` の cost-guard テスト）。

### 4.4 [issue/PR 記録値・未再計測] PR #1340（#1324 対応、Update op の DOM 適用）

同一キーで内容のみが変わる更新を `KeyedOp::Update` として DOM 適用可能に
した変更（wasm-client 0.3.2 → 0.4.0、破壊的変更）。PR 本文には「性能実測
（受け入れ条件: update ≤ 2.64ms、`_/bench/run_csr.mjs` baseline 1.32ms との
before/after）はローカルベンチが本環境で実行不能なため未実施」と明記されて
おり、native コスト固定テスト・意味的等価テストによる代替検証で置き換えて
いる。

### 4.5 CSR 実測値の限界（本レポート共通）

本 worktree には `_/bench/csr/fandhe`（playwright + chromium 実測ハーネス）
が存在しないため、CSR create/update/clear の同一条件での再計測・
before/after 直接比較は本レポートでは実施できていない。継続的な実行時
性能の非劣化は `crates/wasm-full/tests/perf_browser.rs`（CI 常設）・
`keyed_apply.rs`（DOM 操作呼び出し回数のコスト固定テスト）が代替担保して
いる（PR #1334 の評価文書 `docs/ci/wasm-opt-adoption-evaluation.md` に同旨
の記載あり）。

## 5. Payload（gzip）

### 5.1 [既存文書引用] `docs/ci/wasm-opt-adoption-evaluation.md`（イシュー #1327・PR #1334）

同一ハーネス（プロファイル調整・wasm-opt 導入前後）での構成比較:

| 構成 | wasm raw | wasm gzip | js raw | js gzip | 合計 raw | 合計 gzip |
|------|----------|-----------|--------|---------|----------|-----------|
| 変更前（rustc 既定プロファイル、wasm-opt なし） | 78,820 B | 29,936 B | 14,917 B | 4,303 B | 93,737 B | 34,239 B |
| プロファイル調整 + wasm-opt -Os（最終適用構成） | 44,383 B | 21,221 B | 15,058 B | 4,343 B | 59,441 B | 25,564 B |

- プロファイル調整のみによる削減: gzip 合計で 34,239B → 30,416B（**11.2% 減**）
- wasm-opt 追加適用による削減（プロファイル調整後との比較）: gzip 合計で 30,416B → 25,564B（**16.0% 減**）
- 変更前との合計削減: gzip 合計で **34,239B → 25,564B（25.3% 減）**

Vue 水準（gzip 約 22KB、issue #1313 本文の比較記録）へ大きく接近する削減
（本レポート §4.5 と同じ制約により、他フレームワークとの同日相対順位再計測
は未実施）。

### 5.2 [issue/PR 記録値・未再計測] ベースライン（2026-08-11、issue #1313 本文より）

- payload（gzip）: 34.4KB（wasm + glue。React 61KB より小、Vue 22KB より大）

上記 §5.1 の「変更前」値（34,239B ≈ 34.4KB）とオーダーが一致しており、
本イシューの起点と `wasm-opt-adoption-evaluation.md` の baseline は同一の
実測を指していると判断できる。

## 6. 状態変更負荷（interactive）

### 6.1 [再計測] `xtask bench-state-update`（2026-08-12、fandhe-frontend-interactive 0.2.3）

```
{"framework":"fandhe-frontend","version":"0.2.3","mode":"state-update","workload_schema_version":1,"bindings":1000,
 "grid1k":{"update":{"mean_us":0.0261},"binding_apply":{"mean_us":0.0476},"render":{"mean_us":94.3742},"noop_update":{"mean_us":0.0198}},
 "appstate1k":{"update":{"mean_us":0.0377},"binding_apply":{"mean_us":0.0395},"render":{"mean_us":335.7259},"noop_update":{"mean_us":0.0315}},
 "escape_ok":true,"noop_ok":true,"notes":"profile=release"}
```

| シナリオ | update mean | binding_apply mean | render mean | noop_update mean |
|----------|-------------|---------------------|-------------|-------------------|
| grid1k（1,000 bindings） | 0.0261 µs | 0.0476 µs | 94.3742 µs | 0.0198 µs |
| appstate1k（1,000 bindings） | 0.0377 µs | 0.0395 µs | 335.7259 µs | 0.0315 µs |

escape_ok / noop_ok は共に true（PASS）。

### 6.2 [再計測] `xtask bench-binding-update`（2026-08-12）

```
bench-binding-update: scenario=appstate-increment full_ns=2082.27 dirty_ns=28.93 ratio=71.96
bench-binding-update: scenario=disclosure-toggle full_ns=67.02 dirty_ns=1.14 ratio=58.89
bench-binding-update: scenario=single-select-select full_ns=75.69 dirty_ns=11.35 ratio=6.67
```

`ratio` は「全束縛再評価コスト（full_ns）/ dirty 束縛のみの再評価コスト
（dirty_ns）」であり、値が大きいほど dirty 追跡による削減効果が大きいことを
示す（イシュー #592、`bench_binding_update` モジュール）。`bench-state-update`
は #1328 で新設された常設コマンドのため、本レポートの値がベースライン
（初回計測）を兼ねる。#1328 の改善内容そのものによる before/after 差分は
PR #1337 本文に具体的な数値記載がなく（「実装内容の要約」のプレースホルダの
まま）、[PR 記録値] としては引用できない。

## 7. XSS エスケープ検証

- 本レポート [再計測] 分（`bench-ssr`・`bench-state-update`）は
  `escape_ok: true` を確認済み（§3.1・§6.1）。
- `bench-binding-update` は escape_ok フィールドを持たない（DOM 適用を伴わない
  純粋な束縛評価コスト計測のため対象外。既定エスケープの回帰検知は
  `bench-ssr`/`bench-state-update`/既存 XSS 回帰テスト群が担う）。
- [issue/PR 記録値] 側: issue #1313 本文に「XSS エスケープ検証: 全 17 計測系
  PASS」と記録されている（2026-08-11 時点、`_/bench/` 全計測系）。本レポート
  では再検証していない。

## 8. まとめ

- **[再計測] で確認できた事実**: SSR（`bench-ssr`）・状態変更負荷
  （`bench-state-update`・`bench-binding-update`）は現行 workspace
  （core 0.3.0 / interactive 0.2.3 / wasm-client 0.4.0）で正常に動作し、
  既定エスケープ回帰検知（`escape_ok`）・no-op 検知（`noop_ok`）は全て PASS。
- **[既存文書引用] で確認できた事実**: payload は wasm-opt 導入
  （#1327・PR #1334）によりプロファイル調整前と比べ gzip 合計で 25.3% 削減
  （34,239B → 25,564B）。
- **[issue/PR 記録値・未再計測] にとどまる事実**: CSR create の O(1) 化
  （#1319・PR #1335）による 22.2ms → 6.67ms（約 3.3 倍）、Node 木構築の
  アロケーション約 44% 削減（#1326・PR #1333）、render エスケープ走査の
  rows1k -10.9%〜-13.5%（#1325・PR #1338）は、いずれも各 PR 作成時点での
  記録値であり、本レポートでは同一条件下の再計測による裏付けを行っていない。
- 他フレームワーク 10 種との同日相対位置（SSR 11 系・CSR 7 系）は
  `_/bench/` が本 worktree に不在のため本レポートでは未実施。

## 9. 残課題・再評価トリガー（Issue化候補、自動起票はしない）

- `_/bench/` 由来の他フレームワーク相対位置（SSR 11 系・CSR 7 系）の同日
  再計測: worktree 隔離により本レポートでは未実施。メイン working copy で
  `_/bench/PROTOCOL.md` に従い再計測することで解消可能
- CSR create/update/clear のブラウザ実測（playwright + chromium ハーネス）の
  同一条件再計測: 同上の制約により未実施
- #1328（状態変更負荷改善）の before/after 差分の具体的数値: PR #1337 本文に
  記載がなく、本レポートでも再計測環境の制約により比較対象の「before」を
  取得できていない

## 10. 参照

- イシュー #1313（トラッキング）・#1314〜#1329（Phase 別子 issue）
- PR #1330・#1333〜#1340（各改善の実装 PR）
- `docs/ci/wasm-opt-adoption-evaluation.md`（payload 削減の詳細評価）
- `_/bench/PROTOCOL.md`（非追跡・メイン working copy にのみ存在するベンチ
  再現手順。フレームワーク横断比較・CSR ブラウザ実測を再現する場合の一次
  情報源）
- `crates/xtask/src/bench_ssr.rs` / `bench_state_update.rs` /
  `bench_binding_update.rs`（本レポート [再計測] 分の実行元、CI 非依存で
  常設・再現可能）
- `docs/reports/perf-browser-report.md`（別系統、TASK-11.5 実ブラウザ計測・
  Conditional Go 判定。本レポートとは対象・目的が異なる）

## 11. 追補: 2026-08-21 再計測

本レポート初版（§1〜§10、2026-08-12 計測）の作成後に、常設 xtask ベンチ
3 種を §2.2 記載の常設コマンドにより再実行した記録である。区分ラベルは本文と同じ
[再計測]（実行して得た 1 行サマリ／JSON を本文へ転記）を用いる。既存
セクションの数値・記述は変更しない。

### 11.1 計測環境（本追補分）

| 項目 | 値 |
|------|-----|
| OS | Ubuntu 26.04 LTS（Linux 7.0.0-29-generic） |
| CPU | 12 vCPU（仮想化 CPU） |
| rustc | 1.96.0 |
| 実行コマンド | `cargo run -p xtask --release -- bench-ssr` / `bench-state-update` / `bench-binding-update` |
| 実行方式 | `--release` プロファイルで各コマンドを 5 回反復実行し、代表 1 回分の出力とラン間範囲（min〜max）を転記 |
| 計測日 | 2026-08-21 |

§2.1（初版計測時）との環境差はカーネル（7.0.0-28 → 7.0.0-29）のみで、
rustc・CPU 数は同一。対象クレートのバージョン（core 0.3.0 /
interactive 0.2.3 / wasm-client 0.4.0）も初版時点から変化していない。

実行方式は初版（§2.1）の単発実行と異なり 5 回反復とした。単発 2 時点の
代表値比較ではラン間分散を推定できず性能回帰の有無を判定できないため、
本追補では 5 回反復のラン間範囲（min〜max）を分散の目安として併記し、
結論は「今回の計測範囲で顕著な悪化を観測したか」に限定する（初版・
本追補とも事前定義した回帰閾値を持たない探索的計測であり、回帰不在の
証明にはならない。閾値ベースの回帰検知は `bench-ssr --baseline` /
`bench-state-update --baseline` の常設機構が別途担う）。

### 11.2 [再計測] `xtask bench-ssr`（fandhe-frontend-core 0.3.0、5 回反復）

代表 1 回分（5 回中 1 回目）の JSON:

```
{"framework":"fandhe-frontend","version":"0.3.0","mode":"ssr","workload_schema_version":1,
 "rows1k":{"iters":100,"mean_ms":0.1083,"p50_ms":0.1040,"p95_ms":0.1339,"min_ms":0.0973},
 "rows10k":{"iters":10,"mean_ms":1.2456,"p50_ms":1.3595,"p95_ms":1.3862,"min_ms":1.0032},
 "html_bytes_1k":118931,"escape_ok":true,"row_count_ok":true,"notes":"profile=release"}
```

| 指標 | 2026-08-12（§3.1、単発） | 2026-08-21（5 回、min〜max） |
|------|---------------------------|------------------------------|
| rows1k mean | 0.1102 ms | 0.1065〜0.1085 ms |
| rows1k p95 | 0.1406 ms | 0.1299〜0.1352 ms |
| rows10k mean | 1.2323 ms | 1.2377〜1.2734 ms |
| rows10k p95 | 1.4365 ms | 1.3862〜1.4688 ms |
| escape_ok / row_count_ok | true / true（PASS） | 5 回すべて true / true（PASS） |

rows1k は 5 回とも初版値を下回った。rows10k は初版値（1.2323 ms）が
今回のラン間範囲の下端（1.2377 ms）をわずかに下回る（差 -0.4%）が、
ラン間幅（範囲幅約 2.9%）より小さい変動である。以上より、**今回の
5 回計測の範囲では SSR 性能の顕著な悪化は観測されなかった**（§11.1 の
限定のとおり、これは回帰不在の証明ではない）。

### 11.3 [再計測] `xtask bench-state-update`（fandhe-frontend-interactive 0.2.3、5 回反復）

代表 1 回分（5 回中 1 回目）の JSON（転記は mean のみに簡約。p50/p95/min
は実行時 JSON に含まれる）:

```
{"framework":"fandhe-frontend","version":"0.2.3","mode":"state-update","workload_schema_version":1,"bindings":1000,
 "grid1k":{"update":{"iters":200,"mean_us":0.0260},"binding_apply":{"iters":200,"mean_us":0.0471},
  "render":{"iters":200,"mean_us":93.7602},"noop_update":{"iters":200,"mean_us":0.0198}},
 "appstate1k":{"update":{"iters":200,"mean_us":0.0369},"binding_apply":{"iters":200,"mean_us":0.0436},
  "render":{"iters":200,"mean_us":333.7365},"noop_update":{"iters":200,"mean_us":0.0305}},
 "escape_ok":true,"noop_ok":true,"notes":"profile=release"}
```

| シナリオ・指標（mean） | 2026-08-12（§6.1、単発） | 2026-08-21（5 回、min〜max） |
|------------------------|---------------------------|------------------------------|
| grid1k update | 0.0261 µs | 0.0260〜0.0261 µs |
| grid1k binding_apply | 0.0476 µs | 0.0461〜0.0474 µs |
| grid1k render | 94.3742 µs | 93.1265〜94.7754 µs |
| grid1k noop_update | 0.0198 µs | 0.0197〜0.0199 µs |
| appstate1k update | 0.0377 µs | 0.0339〜0.0369 µs |
| appstate1k binding_apply | 0.0395 µs | 0.0388〜0.0436 µs |
| appstate1k noop_update | 0.0315 µs | 0.0267〜0.0322 µs |
| appstate1k render | 335.7259 µs | 325.8694〜336.2030 µs |

escape_ok / noop_ok は 5 回すべて true（PASS）。初版値は render 系
（grid1k・appstate1k とも）で今回のラン間範囲内にあり、update /
binding_apply / noop_update はサブマイクロ秒域でラン間範囲内または
範囲幅と同程度の差にとどまる。以上より、**今回の 5 回計測の範囲では
状態変更負荷の顕著な悪化は観測されなかった**（§11.1 の限定のとおり）。

### 11.4 [再計測] `xtask bench-binding-update`（2026-08-21、5 回反復）

代表 1 回分（5 回中 1 回目）の出力:

```
bench-binding-update: scenario=appstate-increment full_ns=2078.13 dirty_ns=29.08 ratio=71.47
bench-binding-update: scenario=disclosure-toggle full_ns=65.70 dirty_ns=0.64 ratio=103.18
bench-binding-update: scenario=single-select-select full_ns=75.54 dirty_ns=10.59 ratio=7.13
```

| シナリオ | ratio（2026-08-12、§6.2、単発） | ratio（5 回、min〜max） | full_ns（5 回、min〜max） |
|----------|----------------------------------|--------------------------|----------------------------|
| appstate-increment | 71.96 | 71.47〜73.66 | 2078.13〜2124.61 |
| disclosure-toggle | 58.89 | 101.51〜104.60 | 65.70〜67.86 |
| single-select-select | 6.67 | 6.75〜7.67 | 75.38〜80.00 |

appstate-increment・single-select-select の ratio は初版値がラン間範囲内
または範囲近傍にあり、full_ns にも顕著な悪化は見られない。
disclosure-toggle の ratio の見かけの変動（58.89 → 101.51〜104.60）は、
分母 dirty_ns が 1 ns 未満（初版 1.14 ns → 今回 0.64〜0.67 ns）の極小値
であることによる比の増幅であり、full_ns 自体は初版 67.02 ns に対し今回
65.70〜67.86 ns と同水準にある。dirty_ns のこの水準差が計測分解能・環境
要因のいずれによるものかは本計測では判別できない（対象実装は初版から
変更されていない）。

### 11.5 本追補でも解消していない制約

- フレームワーク横断ベンチハーネス `_/bench/`（`run_ssr.py` /
  `run_csr.mjs` / payload gzip 計測、`_/bench/PROTOCOL.md`）は
  `.gitignore` の `/_/` により git 管理外であり、本追補の計測環境にも
  存在しない。このため他フレームワークとの同日相対位置
  （SSR 11 系・CSR 7 系比較表）の再計測、および CSR create/update/clear
  のブラウザ実測は本追補でも未実施である（§2.2・§4.5・§9 と同じ制約）。
- 「SSR 11 種・CSR 7 種」の比較対象フレームワークのリスト自体も公開
  リポジトリ内に記録がなく、issue #1313 本文の記録値（§3.2・§4.1）から
  逆引きできる範囲にとどまる。横断再計測を行う場合はハーネスの再構築
  （比較対象リストの復元・記録を含む）から必要となる。

## 12. 追補 2: 2026-08-21 macOS（Apple Silicon）環境での参考再計測

§11 と同日（2026-08-21）に、初版・§11 とは異なるハードウェア（macOS / Apple Silicon 実機）で常設 xtask ベンチ 3 種を単発実行した参考記録である。初版（§2.1）・§11.1 とは OS・CPU が異なるため、§3.1・§6.1・§6.2・§11 の値との差は環境差を含み、経時比較・回帰判定には使えない（参考値）。区分ラベルは [再計測・参考] を用いるが、上記の限定を付す。

### 12.1 計測環境（本追補分）

| 項目 | 値 |
|------|-----|
| OS | macOS 26.6.2（Darwin 25.6.0） |
| CPU | Apple M4 Max（16 コア） |
| rustc | 1.96.0 (ac68faa20 2026-05-25) |
| 実行コマンド | `cargo run -p xtask --release -- bench-ssr` / `bench-state-update` / `bench-binding-update` |
| 実行方式 | `--release` プロファイルでの単発実行（§11.1 の 5 回反復とは異なる） |
| 計測日 | 2026-08-21 |

ホスト名・ユーザー名・絶対パスは記録しない（§2.1 と同方針）。対象クレートバージョンは §11 と同一（core 0.3.0 / interactive 0.2.3）。

### 12.2 [再計測・参考] `xtask bench-ssr`（fandhe-frontend-core 0.3.0）

実測 JSON（1 行サマリ原文。§11.2 と同様に整形転記してよい）:

```
{"framework":"fandhe-frontend","version":"0.3.0","mode":"ssr","workload_schema_version":1,
 "rows1k":{"iters":100,"mean_ms":0.1314,"p50_ms":0.1320,"p95_ms":0.1455,"min_ms":0.1167},
 "rows10k":{"iters":10,"mean_ms":1.0098,"p50_ms":0.9834,"p95_ms":1.0610,"min_ms":0.9665},
 "html_bytes_1k":118931,"escape_ok":true,"row_count_ok":true,"notes":"profile=release"}
```

| 指標 | 値（[再計測・参考]） |
|------|------|
| rows1k mean | 0.1314 ms |
| rows1k p95 | 0.1455 ms |
| rows10k mean | 1.0098 ms |
| rows10k p95 | 1.0610 ms |
| escape_ok / row_count_ok | true / true（PASS） |

html_bytes_1k は 118931 で §11.2 と一致し、ワークロード同一性の傍証となる。

### 12.3 [再計測・参考] `xtask bench-state-update`（fandhe-frontend-interactive 0.2.3）

実測 JSON:

```
{"framework":"fandhe-frontend","version":"0.2.3","mode":"state-update","workload_schema_version":1,"bindings":1000,
 "grid1k":{"update":{"iters":200,"mean_us":0.0413,"p50_us":0.0420,"p95_us":0.0420,"min_us":0.0000},
  "binding_apply":{"iters":200,"mean_us":0.0685,"p50_us":0.0830,"p95_us":0.0840,"min_us":0.0000},
  "render":{"iters":200,"mean_us":109.2279,"p50_us":106.0420,"p95_us":127.2090,"min_us":96.4160},
  "noop_update":{"iters":200,"mean_us":0.0049,"p50_us":0.0000,"p95_us":0.0410,"min_us":0.0000}},
 "appstate1k":{"update":{"iters":200,"mean_us":0.0287,"p50_us":0.0410,"p95_us":0.0420,"min_us":0.0000},
  "binding_apply":{"iters":200,"mean_us":0.0418,"p50_us":0.0420,"p95_us":0.0420,"min_us":0.0000},
  "render":{"iters":200,"mean_us":277.8786,"p50_us":276.5420,"p95_us":296.5000,"min_us":247.8330},
  "noop_update":{"iters":200,"mean_us":0.0123,"p50_us":0.0000,"p95_us":0.0410,"min_us":0.0000}},
 "escape_ok":true,"noop_ok":true,"notes":"profile=release"}
```

| シナリオ・指標（mean） | 値（[再計測・参考]） |
|------------------------|------|
| grid1k update | 0.0413 µs |
| grid1k binding_apply | 0.0685 µs |
| grid1k render | 109.2279 µs |
| grid1k noop_update | 0.0049 µs |
| appstate1k update | 0.0287 µs |
| appstate1k binding_apply | 0.0418 µs |
| appstate1k render | 277.8786 µs |
| appstate1k noop_update | 0.0123 µs |

escape_ok / noop_ok とも true（PASS）。サブマイクロ秒域（update / binding_apply / noop_update）の p50・min に 0.0000 µs が現れるのは本環境のタイマー分解能に起因する量子化であり、値の精度限界として注記する。

### 12.4 [再計測・参考] `xtask bench-binding-update`

実測出力原文:

```
bench-binding-update: scenario=appstate-increment full_ns=3930.51 dirty_ns=65.58 ratio=59.93
bench-binding-update: scenario=disclosure-toggle full_ns=135.06 dirty_ns=0.73 ratio=184.18
bench-binding-update: scenario=single-select-select full_ns=140.65 dirty_ns=21.28 ratio=6.61
```

| シナリオ | full_ns | dirty_ns | ratio |
|----------|---------|----------|-------|
| appstate-increment | 3930.51 | 65.58 | 59.93 |
| disclosure-toggle | 135.06 | 0.73 | 184.18 |
| single-select-select | 140.65 | 21.28 | 6.61 |

dirty 差分適用は full 再適用比 6.61〜184.18 倍高速であり、環境が変わっても dirty 追跡の優位という定性的傾向は §6.2・§11.4 と一致する。

### 12.5 制約の継続

本追補環境（macOS 実機）での制約は §11.5 と同一である。フレームワーク横断ベンチハーネス `_/bench/` の非存在により、他フレームワークとの同日相対位置の再計測および CSR create/update/clear のブラウザ実測は本追補でも未実施である（詳細は §11.5 参照）。

## 13. 追補 3: 2026-08-22 Linux 再計測（`make bench` 経由）

PR #1368 で追加した `make bench` ターゲット（常設 xtask ベンチ 3 種を `--release --locked` で一括実行する入口）を、§11 と同一の Linux 環境で単発実行した記録である。区分ラベルは [再計測] を用いる。既存セクションの数値・記述は変更しない。

### 13.1 計測環境（本追補分）

| 項目 | 値 |
|------|-----|
| OS | Ubuntu 26.04 LTS（Linux 7.0.0-29-generic） |
| CPU | 12 vCPU（仮想化 CPU） |
| rustc | 1.96.0 (ac68faa20 2026-05-25) |
| 実行コマンド | `make bench`（`cargo run -p xtask --release --locked -- bench-ssr` / `bench-state-update` / `bench-binding-update` を順次実行） |
| 実行方式 | `--release` プロファイルでの単発実行（§11.1 の 5 回反復とは異なる。ラン間分散は推定できないため、比較は §11 のラン間範囲との照合にとどめる） |
| 計測日 | 2026-08-22 |

OS・カーネル・CPU 数・rustc は §11.1 と同一であり、対象クレートバージョン（core 0.3.0 / interactive 0.2.3）も §11・§12 から変化していない。§11 との差分は実行方式（5 回反復 → 単発）と、実行入口が `make bench` である（`--locked` が付く）ことのみ。

### 13.2 [再計測] `xtask bench-ssr`（fandhe-frontend-core 0.3.0、単発）

実測 JSON（1 行サマリ原文を整形転記）:

```
{"framework":"fandhe-frontend","version":"0.3.0","mode":"ssr","workload_schema_version":1,
 "rows1k":{"iters":100,"mean_ms":0.1066,"p50_ms":0.1029,"p95_ms":0.1302,"min_ms":0.0960},
 "rows10k":{"iters":10,"mean_ms":1.2513,"p50_ms":1.3333,"p95_ms":1.4152,"min_ms":1.0095},
 "html_bytes_1k":118931,"escape_ok":true,"row_count_ok":true,"notes":"profile=release"}
```

| 指標 | 2026-08-21（§11.2、5 回、min〜max） | 2026-08-22（単発） |
|------|--------------------------------------|---------------------|
| rows1k mean | 0.1065〜0.1085 ms | 0.1066 ms |
| rows1k p95 | 0.1299〜0.1352 ms | 0.1302 ms |
| rows10k mean | 1.2377〜1.2734 ms | 1.2513 ms |
| rows10k p95 | 1.3862〜1.4688 ms | 1.4152 ms |
| escape_ok / row_count_ok | 5 回すべて true / true（PASS） | true / true（PASS） |

html_bytes_1k は 118931 で §11.2・§12.2 と一致（ワークロード同一性の傍証）。全指標が §11.2 のラン間範囲内にあり、**今回の単発計測の範囲では SSR 性能の顕著な悪化は観測されなかった**（§11.1 と同じ限定を付す）。

### 13.3 [再計測] `xtask bench-state-update`（fandhe-frontend-interactive 0.2.3、単発）

実測 JSON:

```
{"framework":"fandhe-frontend","version":"0.2.3","mode":"state-update","workload_schema_version":1,"bindings":1000,
 "grid1k":{"update":{"iters":200,"mean_us":0.0260,"p50_us":0.0250,"p95_us":0.0260,"min_us":0.0230},
  "binding_apply":{"iters":200,"mean_us":0.0461,"p50_us":0.0450,"p95_us":0.0470,"min_us":0.0440},
  "render":{"iters":200,"mean_us":94.4553,"p50_us":93.7350,"p95_us":98.2460,"min_us":91.9110},
  "noop_update":{"iters":200,"mean_us":0.0197,"p50_us":0.0200,"p95_us":0.0200,"min_us":0.0180}},
 "appstate1k":{"update":{"iters":200,"mean_us":0.0338,"p50_us":0.0300,"p95_us":0.0790,"min_us":0.0280},
  "binding_apply":{"iters":200,"mean_us":0.0384,"p50_us":0.0350,"p95_us":0.0740,"min_us":0.0330},
  "render":{"iters":200,"mean_us":328.0381,"p50_us":326.5000,"p95_us":339.4620,"min_us":319.2940},
  "noop_update":{"iters":200,"mean_us":0.0272,"p50_us":0.0250,"p95_us":0.0420,"min_us":0.0240}},
 "escape_ok":true,"noop_ok":true,"notes":"profile=release"}
```

| シナリオ・指標（mean） | 2026-08-21（§11.3、5 回、min〜max） | 2026-08-22（単発） |
|------------------------|--------------------------------------|---------------------|
| grid1k update | 0.0260〜0.0261 µs | 0.0260 µs |
| grid1k binding_apply | 0.0461〜0.0474 µs | 0.0461 µs |
| grid1k render | 93.1265〜94.7754 µs | 94.4553 µs |
| grid1k noop_update | 0.0197〜0.0199 µs | 0.0197 µs |
| appstate1k update | 0.0339〜0.0369 µs | 0.0338 µs |
| appstate1k binding_apply | 0.0388〜0.0436 µs | 0.0384 µs |
| appstate1k render | 325.8694〜336.2030 µs | 328.0381 µs |
| appstate1k noop_update | 0.0267〜0.0322 µs | 0.0272 µs |

escape_ok / noop_ok とも true（PASS）。appstate1k update（-0.0001 µs）・binding_apply（-0.0004 µs）が §11.3 のラン間範囲の下端をわずかに下回るほかは全指標が範囲内にあり、下回り幅もタイマー分解能水準にとどまる。**今回の単発計測の範囲では状態変更負荷の顕著な悪化は観測されなかった**（§11.1 と同じ限定を付す）。

### 13.4 [再計測] `xtask bench-binding-update`（2026-08-22、単発）

実測出力原文:

```
bench-binding-update: scenario=appstate-increment full_ns=2081.35 dirty_ns=28.90 ratio=72.01
bench-binding-update: scenario=disclosure-toggle full_ns=67.15 dirty_ns=0.64 ratio=105.56
bench-binding-update: scenario=single-select-select full_ns=75.52 dirty_ns=11.45 ratio=6.60
```

| シナリオ | full_ns（§11.4、5 回、min〜max） | full_ns（単発） | ratio（§11.4、5 回、min〜max） | ratio（単発） |
|----------|-----------------------------------|-----------------|--------------------------------|---------------|
| appstate-increment | 2078.13〜2124.61 | 2081.35 | 71.47〜73.66 | 72.01 |
| disclosure-toggle | 65.70〜67.86 | 67.15 | 101.51〜104.60 | 105.56 |
| single-select-select | 75.38〜80.00 | 75.52 | 6.75〜7.67 | 6.60 |

full_ns は 3 シナリオとも §11.4 のラン間範囲内。ratio の範囲外れ（disclosure-toggle の 105.56、single-select-select の 6.60）はいずれも分母 dirty_ns の極小値（0.64 ns / 11.45 ns）による比の増幅・縮小であり（§11.4 で注記済みの性質）、full_ns 自体に顕著な悪化は見られない。dirty 差分適用の優位（full 再適用比 6.60〜105.56 倍高速）という定性的傾向も §6.2・§11.4・§12.4 と一致する。

### 13.5 制約の継続

本追補の制約は §11.5 と同一である（フレームワーク横断ベンチハーネス `_/bench/` の非存在により、他フレームワーク相対位置・CSR ブラウザ実測は未実施）。

## 14. 追補 4: 2026-08-24 フレームワーク横断ベンチ実測（`make bench-cross`）と常設ベンチ再計測

本レポート初版〜§13 で「未実施の制約」として残っていたフレームワーク横断ベンチ（旧 `_/bench/` 喪失）を、git 管理下の v2 ハーネス（`bench/`、イシュー #1370 で再構築、`bench/PROTOCOL.md` が正）で実施した**初のフルラン転記**である。旧記録値（issue #1313、2026-08-11）との順位互換はない（PROTOCOL の注記どおり）。区分ラベルはすべて [再計測]。

### 14.1 計測環境（本追補分）

| 項目 | 値 |
|------|-----|
| OS | Ubuntu 26.04 LTS（Linux 7.0.0-29-generic） |
| CPU | 12 vCPU（仮想化 CPU） |
| rustc | 1.96.0 (ac68faa20 2026-05-25) |
| Node.js | v24.13.0 |
| chromium | /usr/bin/chromium-browser 151.0.7922.108（システム chromium、playwright-core 経由 headless） |
| 実行コマンド | `make bench-cross`（`bench/PROTOCOL.md` §3 の手順） + `cargo run -p xtask --release --locked -- bench-state-update` / `bench-binding-update`（単発） |
| 実行方式 | `--release --locked` プロファイルでの単発実行 |
| 計測日 | 2026-08-24 |
| `bench/PROTOCOL.md` 最終コミット | 5f445b9 |
| 計測時 HEAD | 17daec7 |
| クレートバージョン | fandhe-frontend-core 0.4.3 / -interactive 0.2.7 / -app 0.2.6 / -wasm-client 0.6.1 |

注記: 本追補のクレートバージョン（core 0.4.3 / interactive 0.2.7）は §11〜§13 の core 0.3.0 / interactive 0.2.3 と**バージョンが異なる**。#1371 系列の CSR 改善・#1394/#1397 の diff/属性同期改善を含む現行コードの計測であり、§13 との数値比較はバージョン跨ぎである旨を明記する。

### 14.2 [再計測] SSR（8 種フレームワーク）

実測 JSON（1 行サマリを整形転記）:

```
fandhe-frontend 0.4.3:
{"framework":"fandhe-frontend","version":"0.4.3","mode":"ssr","workload_schema_version":1,
 "rows1k":{"iters":100,"mean_ms":0.1154,"p50_ms":0.1124,"p95_ms":0.1417,"min_ms":0.1080},
 "rows10k":{"iters":10,"mean_ms":1.3282,"p50_ms":1.2767,"p95_ms":1.5230,"min_ms":1.1342},
 "html_bytes_1k":118931,"escape_ok":true,"row_count_ok":true,"notes":"profile=release"}

vanilla v24.13.0:
{"framework":"vanilla","version":"v24.13.0","mode":"ssr","workload_schema_version":1,
 "rows1k":{"iters":100,"mean_ms":0.8181,"p50_ms":0.6927,"p95_ms":1.6884,"min_ms":0.6355},
 "rows10k":{"iters":10,"mean_ms":12.1511,"p50_ms":11.6108,"p95_ms":15.9502,"min_ms":6.8001},
 "html_bytes_1k":116931,"escape_ok":true,"row_count_ok":true,"notes":"node v24.13.0; NODE_ENV=production"}

react 19.2.8:
{"framework":"react","version":"19.2.8","mode":"ssr","workload_schema_version":1,
 "rows1k":{"iters":100,"mean_ms":0.7285,"p50_ms":0.6233,"p95_ms":1.4328,"min_ms":0.5772},
 "rows10k":{"iters":10,"mean_ms":9.8971,"p50_ms":7.4154,"p95_ms":13.0594,"min_ms":6.7849},
 "html_bytes_1k":118944,"escape_ok":true,"row_count_ok":true,"notes":"node v24.13.0; NODE_ENV=production"}

preact 10.29.8:
{"framework":"preact","version":"10.29.8","mode":"ssr","workload_schema_version":1,
 "rows1k":{"iters":100,"mean_ms":0.6254,"p50_ms":0.5194,"p95_ms":0.5731,"min_ms":0.4883},
 "rows10k":{"iters":10,"mean_ms":6.429,"p50_ms":4.8857,"p95_ms":9.3034,"min_ms":4.6517},
 "html_bytes_1k":102931,"escape_ok":true,"row_count_ok":true,"notes":"node v24.13.0; NODE_ENV=production"}

vue 3.5.41:
{"framework":"vue","version":"3.5.41","mode":"ssr","workload_schema_version":1,
 "rows1k":{"iters":100,"mean_ms":0.7291,"p50_ms":0.6102,"p95_ms":0.7008,"min_ms":0.578},
 "rows10k":{"iters":10,"mean_ms":9.2842,"p50_ms":6.4729,"p95_ms":12.6091,"min_ms":6.1485},
 "html_bytes_1k":116931,"escape_ok":true,"row_count_ok":true,"notes":"node v24.13.0; NODE_ENV=production"}

solid 1.9.15:
{"framework":"solid","version":"1.9.15","mode":"ssr","workload_schema_version":1,
 "rows1k":{"iters":100,"mean_ms":0.4135,"p50_ms":0.3104,"p95_ms":0.3535,"min_ms":0.2893},
 "rows10k":{"iters":10,"mean_ms":4.4786,"p50_ms":3.164,"p95_ms":7.3923,"min_ms":3.0191},
 "html_bytes_1k":95938,"escape_ok":true,"row_count_ok":true,"notes":"node v24.13.0; NODE_ENV=production"}

svelte 5.56.10:
{"framework":"svelte","version":"5.56.10","mode":"ssr","workload_schema_version":1,
 "rows1k":{"iters":100,"mean_ms":0.4947,"p50_ms":0.2468,"p95_ms":0.4875,"min_ms":0.2308},
 "rows10k":{"iters":10,"mean_ms":3.8816,"p50_ms":2.3829,"p95_ms":3.3332,"min_ms":2.3646},
 "html_bytes_1k":92965,"escape_ok":true,"row_count_ok":true,"notes":"node v24.13.0; NODE_ENV=production"}

lit 3.3.3:
{"framework":"lit","version":"3.3.3","mode":"ssr","workload_schema_version":1,
 "rows1k":{"iters":100,"mean_ms":1.0035,"p50_ms":0.9401,"p95_ms":1.5683,"min_ms":0.9128},
 "rows10k":{"iters":10,"mean_ms":13.3339,"p50_ms":13.7143,"p95_ms":16.6441,"min_ms":9.2954},
 "html_bytes_1k":223037,"escape_ok":true,"row_count_ok":true,"notes":"node v24.13.0; NODE_ENV=production"}
```

| framework | version | rows1k mean | rows10k mean | html_bytes_1k | escape_ok |
|-----------|---------|------------|------------|-------|-----------|
| fandhe-frontend | 0.4.3 | 0.1154 ms | 1.3282 ms | 118931 | true |
| solid | 1.9.15 | 0.4135 ms | 4.4786 ms | 95938 | true |
| svelte | 5.56.10 | 0.4947 ms | 3.8816 ms | 92965 | true |
| preact | 10.29.8 | 0.6254 ms | 6.429 ms | 102931 | true |
| react | 19.2.8 | 0.7285 ms | 9.8971 ms | 118944 | true |
| vue | 3.5.41 | 0.7291 ms | 9.2842 ms | 116931 | true |
| vanilla | v24.13.0 | 0.8181 ms | 12.1511 ms | 116931 | true |
| lit | 3.3.3 | 1.0035 ms | 13.3339 ms | 223037 | true |

fandhe-frontend はネイティブ Rust ビルド（release）、他 7 種は Node.js ランタイム（NODE_ENV=production）という言語・ランタイム差を含む比較である（`bench/PROTOCOL.md` §4）。fandhe rows1k 0.1154 ms は 2 位の solid 0.4135 ms の約 3.6 倍高速であり、rows10k も最速である。

### 14.3 [再計測] CSR（7 種フレームワーク）

実測 JSON（1 行サマリからの抜粋・丸め転記。`*_layout_ms`（layout flush 分離指標）は割愛し、mean/p50/p95/min は小数 3 桁へ丸めた。ブラウザ側 `performance.now()` の分解能は 0.1 ms であり、原文の下位桁は浮動小数点表現由来のため情報を失わない）:

```
vanilla:
{"framework":"vanilla","version":"n/a","mode":"csr","workload_schema_version":1,
 "create_ms":{"iters":25,"mean_ms":10.864,"p50_ms":10.200,"p95_ms":13.5,"min_ms":9.600},
 "update_ms":{"iters":25,"mean_ms":12.632,"p50_ms":11.300,"p95_ms":16.5,"min_ms":10.200},
 "clear_ms":{"iters":25,"mean_ms":0.880,"p50_ms":0.800,"p95_ms":1.100,"min_ms":0.600},
 "create_op_ms":{"iters":25,"mean_ms":0.944,"p50_ms":0.900,"p95_ms":1.000,"min_ms":0.800},
 "update_op_ms":{"iters":25,"mean_ms":1.816,"p50_ms":1.700,"p95_ms":2.300,"min_ms":1.500},
 "clear_op_ms":{"iters":25,"mean_ms":0.836,"p50_ms":0.800,"p95_ms":1.100,"min_ms":0.600},
 "rows_ok":true,"escape_ok":true,"notes":"chromium /usr/bin/chromium-browser version 151.0.7922.108"}

react 19.2.8:
{"framework":"react","version":"19.2.8","mode":"csr","workload_schema_version":1,
 "create_ms":{"iters":25,"mean_ms":14.112,"p50_ms":12.500,"p95_ms":18.100,"min_ms":11.200},
 "update_ms":{"iters":25,"mean_ms":3.492,"p50_ms":3.300,"p95_ms":4.100,"min_ms":3.100},
 "clear_ms":{"iters":25,"mean_ms":1.488,"p50_ms":1.500,"p95_ms":1.800,"min_ms":1.300},
 "create_op_ms":{"iters":25,"mean_ms":2.964,"p50_ms":2.900,"p95_ms":4.500,"min_ms":2.100},
 "update_op_ms":{"iters":25,"mean_ms":0.496,"p50_ms":0.500,"p95_ms":0.700,"min_ms":0.300},
 "clear_op_ms":{"iters":25,"mean_ms":1.432,"p50_ms":1.400,"p95_ms":1.700,"min_ms":1.200},
 "rows_ok":true,"escape_ok":true,"notes":"chromium /usr/bin/chromium-browser version 151.0.7922.108"}

preact 10.29.8:
{"framework":"preact","version":"10.29.8","mode":"csr","workload_schema_version":1,
 "create_ms":{"iters":25,"mean_ms":11.448,"p50_ms":11.100,"p95_ms":12.200,"min_ms":10.000},
 "update_ms":{"iters":25,"mean_ms":3.476,"p50_ms":3.400,"p95_ms":3.800,"min_ms":3.000},
 "clear_ms":{"iters":25,"mean_ms":1.076,"p50_ms":1.100,"p95_ms":1.200,"min_ms":0.800},
 "create_op_ms":{"iters":25,"mean_ms":1.748,"p50_ms":1.700,"p95_ms":2.100,"min_ms":1.400},
 "update_op_ms":{"iters":25,"mean_ms":0.532,"p50_ms":0.500,"p95_ms":0.900,"min_ms":0.200},
 "clear_op_ms":{"iters":25,"mean_ms":1.024,"p50_ms":1.000,"p95_ms":1.100,"min_ms":0.800},
 "rows_ok":true,"escape_ok":true,"notes":"chromium /usr/bin/chromium-browser version 151.0.7922.108"}

vue 3.5.41:
{"framework":"vue","version":"3.5.41","mode":"csr","workload_schema_version":1,
 "create_ms":{"iters":25,"mean_ms":11.644,"p50_ms":11.000,"p95_ms":13.100,"min_ms":10.500},
 "update_ms":{"iters":25,"mean_ms":4.080,"p50_ms":4.000,"p95_ms":4.800,"min_ms":3.500},
 "clear_ms":{"iters":25,"mean_ms":1.192,"p50_ms":1.200,"p95_ms":1.300,"min_ms":1.100},
 "create_op_ms":{"iters":25,"mean_ms":1.916,"p50_ms":1.700,"p95_ms":2.600,"min_ms":1.500},
 "update_op_ms":{"iters":25,"mean_ms":1.072,"p50_ms":1.000,"p95_ms":1.500,"min_ms":0.800},
 "clear_op_ms":{"iters":25,"mean_ms":1.128,"p50_ms":1.100,"p95_ms":1.200,"min_ms":1.000},
 "rows_ok":true,"escape_ok":true,"notes":"chromium /usr/bin/chromium-browser version 151.0.7922.108"}

svelte 5.56.10:
{"framework":"svelte","version":"5.56.10","mode":"csr","workload_schema_version":1,
 "create_ms":{"iters":25,"mean_ms":13.828,"p50_ms":13.500,"p95_ms":16.500,"min_ms":12.300},
 "update_ms":{"iters":25,"mean_ms":4.584,"p50_ms":4.000,"p95_ms":6.700,"min_ms":3.600},
 "clear_ms":{"iters":25,"mean_ms":1.368,"p50_ms":1.300,"p95_ms":1.500,"min_ms":1.100},
 "create_op_ms":{"iters":25,"mean_ms":4.172,"p50_ms":3.800,"p95_ms":5.600,"min_ms":3.500},
 "update_op_ms":{"iters":25,"mean_ms":1.128,"p50_ms":1.000,"p95_ms":1.500,"min_ms":0.800},
 "clear_op_ms":{"iters":25,"mean_ms":1.312,"p50_ms":1.300,"p95_ms":1.500,"min_ms":1.000},
 "rows_ok":true,"escape_ok":true,"notes":"chromium /usr/bin/chromium-browser version 151.0.7922.108"}

lit 3.3.3:
{"framework":"lit","version":"3.3.3","mode":"csr","workload_schema_version":1,
 "create_ms":{"iters":25,"mean_ms":13.500,"p50_ms":12.600,"p95_ms":18.600,"min_ms":11.400},
 "update_ms":{"iters":25,"mean_ms":3.188,"p50_ms":3.000,"p95_ms":3.700,"min_ms":2.700},
 "clear_ms":{"iters":25,"mean_ms":994.432,"p50_ms":932.900,"p95_ms":1292.200,"min_ms":644.600},
 "create_op_ms":{"iters":25,"mean_ms":2.584,"p50_ms":2.400,"p95_ms":3.800,"min_ms":2.100},
 "update_op_ms":{"iters":25,"mean_ms":0.156,"p50_ms":0.100,"p95_ms":0.200,"min_ms":0.100},
 "clear_op_ms":{"iters":25,"mean_ms":994.068,"p50_ms":932.600,"p95_ms":1291.800,"min_ms":644.400},
 "rows_ok":true,"escape_ok":true,"notes":"chromium /usr/bin/chromium-browser version 151.0.7922.108"}

fandhe-frontend 0.6.1:
{"framework":"fandhe","version":"0.6.1","mode":"csr","workload_schema_version":1,
 "create_ms":{"iters":25,"mean_ms":12.880,"p50_ms":12.800,"p95_ms":14.100,"min_ms":11.500},
 "update_ms":{"iters":25,"mean_ms":4.900,"p50_ms":4.800,"p95_ms":5.600,"min_ms":4.400},
 "clear_ms":{"iters":25,"mean_ms":1.268,"p50_ms":1.200,"p95_ms":1.500,"min_ms":1.000},
 "create_op_ms":{"iters":25,"mean_ms":3.176,"p50_ms":3.100,"p95_ms":3.500,"min_ms":2.700},
 "update_op_ms":{"iters":25,"mean_ms":1.912,"p50_ms":1.800,"p95_ms":2.200,"min_ms":1.700},
 "clear_op_ms":{"iters":25,"mean_ms":1.224,"p50_ms":1.200,"p95_ms":1.500,"min_ms":1.000},
 "rows_ok":true,"escape_ok":true,"notes":"chromium /usr/bin/chromium-browser version 151.0.7922.108"}
```

#### 14.3.1 全体比較（create/update/clear）

| framework | version | create_ms | update_ms | clear_ms |
|-----------|---------|-----------|-----------|----------|
| vanilla | n/a | 10.864 | 12.632 | 0.880 |
| react | 19.2.8 | 14.112 | 3.492 | 1.488 |
| preact | 10.29.8 | 11.448 | 3.476 | 1.076 |
| vue | 3.5.41 | 11.644 | 4.080 | 1.192 |
| svelte | 5.56.10 | 13.828 | 4.584 | 1.368 |
| lit | 3.3.3 | 13.500 | 3.188 | 994.432 |
| fandhe | 0.6.1 | 12.880 | 4.900 | 1.268 |

#### 14.3.2 op 分離（layout flush 除外）

| framework | version | create_op_ms | update_op_ms | clear_op_ms |
|-----------|---------|--------------|--------------|-------------|
| vanilla | n/a | 0.944 | 1.816 | 0.836 |
| react | 19.2.8 | 2.964 | 0.496 | 1.432 |
| preact | 10.29.8 | 1.748 | 0.532 | 1.024 |
| vue | 3.5.41 | 1.916 | 1.072 | 1.128 |
| svelte | 5.56.10 | 4.172 | 1.128 | 1.312 |
| lit | 3.3.3 | 2.584 | 0.156 | 994.068 |
| fandhe | 0.6.1 | 3.176 | 1.912 | 1.224 |

注記: 計測境界は DOM 反映 + 強制 layout flush で paint を含まない（`bench/PROTOCOL.md` §2.2）。lit の clear_ms 994.432 ms（clear_op_ms 994.068 ms）は他と 3 桁異なる外れ値であり、lit-html + repeat の全行削除経路の特性と考えられる（本ハーネスでの観測事実としてのみ記録）。

fandhe-frontend の位置づけ: update_op_ms 1.912 ms は vanilla の逐次更新 1.816 ms と同水準で、diff 系（react 0.496 / preact 0.532 / vue 1.072 / svelte 1.128）より遅い。create_op_ms 3.176 ms は react 2.964 ms と同水準である（CSR 横断の実測は本追補が v2 初回であり、過去値との傾向比較は行わない）。

### 14.4 [再計測] payload（gzip）

実測 JSON（1 行サマリを整形転記）:

```
fandhe-frontend:
{"framework":"fandhe","mode":"payload","files":[
 {"file":"bootstrap.js","raw":421,"gzip":204},
 {"file":"fandhe_bench.js","raw":7945,"gzip":2801},
 {"file":"fandhe_bench_bg.wasm","raw":98608,"gzip":41317}],
 "total_raw":106974,"total_gzip":44322}

lit 3.3.3:
{"framework":"lit","mode":"payload","files":[
 {"file":"bundle.js","raw":19059,"gzip":7423}],
 "total_raw":19059,"total_gzip":7423}

preact 10.29.8:
{"framework":"preact","mode":"payload","files":[
 {"file":"bundle.js","raw":11910,"gzip":5022}],
 "total_raw":11910,"total_gzip":5022}

react 19.2.8:
{"framework":"react","mode":"payload","files":[
 {"file":"bundle.js","raw":195009,"gzip":60773}],
 "total_raw":195009,"total_gzip":60773}

svelte 5.56.10:
{"framework":"svelte","mode":"payload","files":[
 {"file":"bundle.js","raw":59239,"gzip":21966}],
 "total_raw":59239,"total_gzip":21966}

vanilla:
{"framework":"vanilla","mode":"payload","files":[
 {"file":"bundle.js","raw":1174,"gzip":570}],
 "total_raw":1174,"total_gzip":570}

vue 3.5.41:
{"framework":"vue","mode":"payload","files":[
 {"file":"bundle.js","raw":62522,"gzip":25213}],
 "total_raw":62522,"total_gzip":25213}
```

| framework | version | total_raw | total_gzip |
|-----------|---------|-----------|------------|
| vanilla | n/a | 1174 B | 570 B |
| preact | 10.29.8 | 11910 B | 5022 B |
| lit | 3.3.3 | 19059 B | 7423 B |
| svelte | 5.56.10 | 59239 B | 21966 B |
| vue | 3.5.41 | 62522 B | 25213 B |
| fandhe | 0.6.1 | 106974 B | 44322 B |
| react | 19.2.8 | 195009 B | 60773 B |

fandhe-frontend 44,322 B gzip は react 60,773 B より小さいが preact 5,022 B / lit 7,423 B / svelte 21,966 B / vue 25,213 B より大きい。wasm 本体 41,317 B gzip が支配的で、削減レバーが構造的に尽きていることは `docs/reports/wasm-dom-apply-payload-reduction-1407.md` / `docs/ci/wasm-allocator-adoption-evaluation.md` に記録済みである。

### 14.5 [再計測] 常設ベンチ（state-update / binding-update）

#### 14.5.1 `xtask bench-state-update`（fandhe-frontend-interactive 0.2.7、単発）

実測 JSON:

```
{"framework":"fandhe-frontend","version":"0.2.7","mode":"state-update","workload_schema_version":1,"bindings":1000,
 "grid1k":{"update":{"iters":200,"mean_us":0.0265,"p50_us":0.0250,"p95_us":0.0260,"min_us":0.0240},
  "binding_apply":{"iters":200,"mean_us":0.0468,"p50_us":0.0460,"p95_us":0.0480,"min_us":0.0440},
  "render":{"iters":200,"mean_us":93.2808,"p50_us":92.6110,"p95_us":97.7840,"min_us":90.3110},
  "noop_update":{"iters":200,"mean_us":0.0200,"p50_us":0.0200,"p95_us":0.0210,"min_us":0.0190}},
 "appstate1k":{"update":{"iters":200,"mean_us":0.0361,"p50_us":0.0310,"p95_us":0.0770,"min_us":0.0290},
  "binding_apply":{"iters":200,"mean_us":0.0386,"p50_us":0.0360,"p95_us":0.0700,"min_us":0.0340},
  "render":{"iters":200,"mean_us":350.5845,"p50_us":350.0250,"p95_us":360.4750,"min_us":336.7830},
  "noop_update":{"iters":200,"mean_us":0.0262,"p50_us":0.0240,"p95_us":0.0440,"min_us":0.0230}},
 "escape_ok":true,"noop_ok":true,"notes":"profile=release"}
```

| シナリオ・指標（mean） | 2026-08-22（§13.3、interactive 0.2.3） | 2026-08-24（interactive 0.2.7） |
|------------------------|--------------------------------------|---------------------|
| grid1k update | 0.0260 µs | 0.0265 µs |
| grid1k binding_apply | 0.0461 µs | 0.0468 µs |
| grid1k render | 94.4553 µs | 93.2808 µs |
| grid1k noop_update | 0.0197 µs | 0.0200 µs |
| appstate1k update | 0.0338 µs | 0.0361 µs |
| appstate1k binding_apply | 0.0384 µs | 0.0386 µs |
| appstate1k render | 328.0381 µs | 350.5845 µs |
| appstate1k noop_update | 0.0272 µs | 0.0262 µs |

escape_ok / noop_ok は共に true（PASS）。interactive 0.2.3 → 0.2.7 のバージョン跨ぎ比較である旨を明記する。appstate1k render は 328.04 µs → 350.58 µs と約 7% 大きくなっているが、バージョン跨ぎ・単発のため回帰と断定しない。要因切り分けは行っていない（継続的な回帰監視は `--baseline` オプション機構が担う）。

#### 14.5.2 `xtask bench-binding-update`（2026-08-24、単発）

実測出力原文:

```
bench-binding-update: scenario=appstate-increment full_ns=2192.01 dirty_ns=29.27 ratio=74.88
bench-binding-update: scenario=disclosure-toggle full_ns=64.48 dirty_ns=0.89 ratio=72.36
bench-binding-update: scenario=single-select-select full_ns=78.12 dirty_ns=10.59 ratio=7.37
```

| シナリオ | full_ns（§13.4、単発） | full_ns（本計測） | ratio（§13.4、単発） | ratio（本計測） |
|----------|------------------------|------------------|----------------------|-----------------|
| appstate-increment | 2081.35 | 2192.01 | 72.01 | 74.88 |
| disclosure-toggle | 67.15 | 64.48 | 105.56 | 72.36 |
| single-select-select | 75.52 | 78.12 | 6.60 | 7.37 |

appstate-increment・single-select-select は full_ns が前回 §13.4 値と近い水準にあり、顕著な悪化は見られない。disclosure-toggle の ratio 減少（105.56 → 72.36）は分母 dirty_ns が増加（0.64 ns → 0.89 ns）したことによる変動であり、full_ns 自体は 67.15 ns → 64.48 ns とむしろ微小に短縮している。dirty 差分適用の優位性（full 再適用比 7.37〜74.88 倍高速）という定性的傾向は §11.4・§13.4 と一致する。

### 14.6 制約の解消と継続

§11.5・§13.5 の「フレームワーク横断ベンチハーネス `_/bench/` の非存在により他フレームワーク相対位置・CSR ブラウザ実測は未実施」という制約は、git 管理下 v2 ハーネス（`bench/`、`bench/PROTOCOL.md`）の導入（イシュー #1370）により**本追補で解消された**（SSR 8 種・CSR 7 種・payload 7 種の初のフル同日計測を実現）。ただし旧 `_/bench/` との順位互換はなく（PROTOCOL 注記）、本追補の 14.2 / 14.3 / 14.4 は v2 系列の初回基準値という位置づけである。

一方、本追補でも継続する制約:

- **paint 非包含**: 計測境界は DOM 反映 + 強制 layout flush までであり、実際の画面 paint は含まない（§14.3 に明記）。CSR create・update は CPU 演算バウンドで paint のないブラウザコンテキストでの計測であり、実ブラウザでのビジュアル確認は別途必要である。
- **単発実行**: 本追補の計測は各項目 1 回の実行であり、ラン間分散は不明。回帰判定の信頼度は限定的であり、継続的な回帰監視は `--baseline` オプション機構が担う（§14.5.1 の注記参照）。
