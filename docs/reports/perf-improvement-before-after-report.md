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
