# 実ブラウザ性能計測レポート（TASK-11.5c・Conditional Go 条件 1 解消判定）

> **注記（#433 改名）**: 本レポートは旧名称時代の実測記録です。`wasm-full/` 等のクレート配置はルート直下から #442 で `crates/wasm-full/` 等へ移設されます（新旧対応は `docs/design/framework-naming.md` 参照）。以下の記録中のパスは当時のまま残しています。

## 1. 目的とトレーサビリティ

- TASK-11.5【Conditional Go 条件 1】（親イシュー #85、REQ-11）は、実ブラウザで
  初期ロード（描画＋ハイドレーション完了 300ms 以内）・DOM 操作性能（16ms/フレーム
  予算内）を正式計測するタスク（`docs/spec/05-tasks.md` TASK-11.5）。
- 3 分割サブタスクの内訳:
  - TASK-11.5a（#86・クローズ済み）: 計測ハーネス構築（`wasm-full/tests/perf_browser.rs`、
    `docs/ci/perf-browser-harness.md`）
  - TASK-11.5b（#87・実装完了）: 初期ロード・DOM 操作性能の計測実行
  - TASK-11.5c（本ドキュメント・#88）: 計測レポート作成・Conditional Go 条件 1
    解消判定
- 本ドキュメントは TASK-11.5c の成果物（計測レポート）であり、TASK-11.5b（#87）
  が収集する `perf-browser:` サマリ行（出力契約は `docs/ci/perf-browser-harness.md`
  第 3 節）を転記・分析し、Conditional Go 条件 1（`docs/spec/06-roadmap.md` MS-3
  完了時のゲート判定項目）の解消可否を判定する。

## 2. 判定ステータス: Go — TASK-11.5b（#87）完了、Conditional Go 条件 1 解消

TASK-11.5b（#87）にて、統制されたローカル環境（第 4 節「計測環境」参照）で
`wasm-pack test --headless --chrome wasm-full --test perf_browser --features
perf-assert -- --nocapture` を 5 回実行し、`perf-browser:` サマリ行を収集した。
`initial_load`（`mean_ms` 基準）・`dom_update`（`p95_ms` 基準＋ 16ms 超過率）の
いずれも第 3 節の判定基準を全 5 回で満たしたため、Conditional Go 条件 1 は
**解消（Go）**と判定する（第 4・5 節）。

## 3. 判定基準（Conditional Go 条件 1）

`docs/spec/06-roadmap.md` 第 78 行・`docs/spec/05-tasks.md` TASK-11.5 に基づく。

| 指標 | 予算 | 対応する `perf-browser:` metric |
|------|------|-------------------------------|
| 初期ロード（描画＋ハイドレーション完了） | 300ms 以内 | `initial_load` |
| DOM 操作（1 操作あたり） | 16ms/フレーム予算内 | `dom_update` |

- `initial_load`（300ms 予算・ページロードあたり 1 回の複合指標）は `mean_ms` を
  基準値とし、`p95_ms` を裾野の安定性確認に用いる。
- `dom_update`（16ms/フレーム予算）は「1 操作ごと」の基準であり、`mean_ms` が
  低くても多数の操作が予算超過している可能性があるため、`mean_ms` 単独では
  判定しない。`p95_ms` を主たる pass/fail 判定に用い、あわせて全サンプル中
  16ms を超過した操作の割合（超過率）を算出する。`p95_ms` が 16ms を超える、
  または超過率が無視できない水準（目安 5% 超）に達する場合は、`mean_ms` が
  予算内であっても単純な Go とはせず Conditional Go（要再計測・要精査）
  として扱う。
- `max_ms` は外れ値（GC・ランナー負荷等）の有無の参考値とし、`p95_ms`・超過率
  が上記基準を満たす前提で、`max_ms` 単独の予算超過のみをもって No-Go とは
  しない（`docs/ci/perf-browser-harness.md` 第 4 節のとおり、CI 共有ランナーは
  ノイズを含むため統制環境での再現性を優先する）。
- PoC-5（`docs/spec/03-poc/`）の Node.js 近似計測値は目標比 300〜5,000 倍の余裕が
  あった。実ブラウザ計測がこの近似値を大きく下回る結果となった場合は
  `docs/spec/06-roadmap.md` 第 83 行の方針に従い、REQ-11（WASM 完全方式の既定化）
  の設計見直し要否を速やかに判断する。

## 4. 計測結果（TASK-11.5b・#87 完了、実測値）

### 4.1 計測環境

| 項目 | 値 |
|------|-----|
| OS | Linux 7.0.0-27-generic |
| CPU | 仮想化 CPU（QEMU Virtual CPU）・12 vCPU |
| Chromium | 150.0.7871.114 |
| chromedriver | 150.0.7871.114（Chromium と同一メジャーバージョン） |
| wasm-pack | 0.15.0 |
| rustc | 1.96.0 |
| 実行コマンド | `CHROMEDRIVER=/usr/bin/chromedriver wasm-pack test --headless --chrome wasm-full --test perf_browser --features perf-assert -- --nocapture` |
| 実行回数 | 5 回（run 間のばらつき確認のため連続実行） |
| 計測日 | 2026-07-17 |

ホスト名・ユーザー名・ファイルパス等の内部情報は記録しない。

### 4.2 実測値（`perf-assert` feature、run ごとの `perf-browser:` サマリ行）

| run | metric | samples | mean_ms | p95_ms | max_ms | 16ms 超過率 |
|-----|--------|---------|---------|--------|--------|-------------|
| 1 | `initial_load` | 30 | 0.343 | 0.070 | 9.205 | 対象外 |
| 1 | `dom_update` | 100 | 0.082 | 0.105 | 0.130 | 0.000 |
| 2 | `initial_load` | 30 | 0.035 | 0.060 | 0.075 | 対象外 |
| 2 | `dom_update` | 100 | 0.079 | 0.100 | 0.120 | 0.000 |
| 3 | `initial_load` | 30 | 0.036 | 0.060 | 0.080 | 対象外 |
| 3 | `dom_update` | 100 | 0.081 | 0.105 | 0.120 | 0.000 |
| 4 | `initial_load` | 30 | 0.034 | 0.060 | 0.080 | 対象外 |
| 4 | `dom_update` | 100 | 0.076 | 0.090 | 0.105 | 0.000 |
| 5 | `initial_load` | 30 | 0.039 | 0.070 | 0.075 | 対象外 |
| 5 | `dom_update` | 100 | 0.079 | 0.100 | 0.120 | 0.000 |

「16ms 超過率」は `dom_update` について、全サンプル中 16ms を超過した操作の
割合を指す（第 3 節の判定基準に対応、`perf_browser.rs::frame_overage_ratio`）。

### 4.3 判定サマリ

| metric | 予算 | 判定根拠（5 run 全体） | 判定 |
|--------|------|------------------------|------|
| `initial_load` | 300ms 以内（`mean_ms` 基準） | `mean_ms` は 0.034〜0.343ms（run 1 は初回サンプルの JIT/コンパイルウォームアップ由来の外れ値 `max_ms=9.205ms` を含むが `mean_ms` は予算の 1000 分の 1 未満）。5 run 全てで `mean_ms` ≪ 300ms | **Go** |
| `dom_update` | 16ms/フレーム以内（`p95_ms` 基準＋超過率） | `p95_ms` は 0.090〜0.105ms（予算の 150 分の 1 以下）。16ms 超過率は 5 run 全てで 0.000（超過サンプルなし） | **Go** |

`run 1` の `initial_load` `max_ms=9.205ms` は他 run（0.075〜0.080ms）と比べて
外れ値だが、Chromium 側の初回 JIT ウォームアップ・GC 等の非決定的要因に
起因すると考えられ、`p95_ms`（0.070ms）・`mean_ms`（0.343ms）ともに予算に
対し十分な余裕があるため、第 3 節の方針（`max_ms` 単独の超過のみで No-Go
としない）に従い判定に影響しない。

## 5. Conditional Go 条件 1 解消判定（確定）

第 4 節の実測値は、`initial_load` の `mean_ms`・`p95_ms` がいずれも予算（300ms）
に対し十分な余裕をもって収まり、`dom_update` の `p95_ms` も予算（16ms）に対し
十分な余裕があり、かつ 16ms 超過率が 5 run 全てで 0.000（目安 5% を大きく
下回る）であった。したがって第 3 節の判定基準を満たし、Conditional Go 条件 1
（実ブラウザでの正式実証）は TASK-6.3（ハイドレーション実証・クローズ済み）と
合わせて **解消（Go）**と判定する。`docs/spec/06-roadmap.md` MS-3 完了時の
Go/No-Go 確認において本レポートを根拠資料とする。

PoC-5（Node.js 近似計測、目標比 300〜5,000 倍の余裕）と比較しても、実ブラウザ
（`Runtime::mount`/`hydrate` 経由の製品経路）での実測値は近似値を下回らず、
むしろ同水準以上の余裕（`initial_load` は予算の 1000 分の 1 未満、`dom_update`
は予算の 150 分の 1 以下）を示した。よって第 3 節が要求する「近似値を大きく
下回る結果」には該当せず、REQ-11（WASM 完全方式の既定化）の設計見直しは不要と
判断する。

**残る人間判断事項**: 数値上は明確な Go だが、以下は本レポートの範囲外であり、
引き続き人間判断・別 Issue でのフォローを要する（`.claude/rules/out-of-scope-tracking.md`）。

- 計測環境が仮想化 CPU（QEMU、12 vCPU）である点。実機（ユーザーが実際に使う
  デバイス相当、特に低スペック端末）での追加計測が MS-3 完了ゲート判断の
  前提として十分かは、ロードマップ運用の裁量判断が必要
- `initial_load` run 1 の外れ値（`max_ms=9.205ms`）はウォームアップ由来と推定
  したが、確定的な原因切り分け（プロファイリング）は本レポートでは行っていない

## 6. 参照

- `docs/ci/perf-browser-harness.md`（TASK-11.5a・出力契約・実行手順・CI 構成）
- `wasm-full/tests/perf_browser.rs`（計測ハーネス本体）
- `docs/spec/05-tasks.md` TASK-11.5（親タスク受け入れ基準）
- `docs/spec/06-roadmap.md`（Conditional Go 条件 1・MS-3 完了ゲート）
- `docs/reports/perf-browser-execution-87.md`（TASK-11.5b・#87 の独立再実行記録・再現性確認）
- Issue #85（親）・#86（ハーネス構築・クローズ済み）・#87（計測実行・クローズ済み）
