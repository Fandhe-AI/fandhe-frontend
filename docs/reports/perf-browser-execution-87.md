# TASK-11.5b 実証実行記録（#87）

> **注記（#433 改名）**: 本レポートは旧名称時代の実測記録です。`wasm-full/` 等のクレート配置はルート直下から #442 で `crates/wasm-full/` 等へ移設されます（新旧対応は `docs/design/framework-naming.md` 参照）。以下の記録中のパスは当時のまま残しています。

## 位置づけ

TASK-11.5【Conditional Go 条件 1】（親イシュー #85、REQ-11）の 3 分割サブタスクの
うち、TASK-11.5b「初期ロード・DOM 操作性能の計測実行」の実施記録です。

TASK-11.5a（#86・クローズ済み）が構築した計測ハーネス（`wasm-full/tests/perf_browser.rs`、
`docs/ci/perf-browser-harness.md`）は、先行の PR（`test/85-perf-browser-formal`）にて
`Runtime::mount`/`hydrate` 経由の製品経路計測へ差し替え済みで、正式計測 5 run の
実測値・Go 判定は `docs/reports/perf-browser-report.md`（TASK-11.5c・#88 の成果物）第 4・5 節に
既に記録されています。

本ドキュメントは、`docs/reports/hydration-browser-execution-67.md`（TASK-6.3c・#67 の先行
事例）と同じパターンに従い、TASK-11.5b（#87）として計測を独立に再実行し、一次証跡を
残すものです。**判定の正本は `docs/reports/perf-browser-report.md` であり、本ドキュメントは
再現性確認のための実行記録であって、判定文書ではありません。**

## 1. 実行環境

`docs/reports/perf-browser-report.md` 第 4.1 節の統制環境と同一構成です。

| 項目 | 値 |
|------|-----|
| OS | Linux 7.0.0-27-generic |
| Chromium | 150.0.7871.114 |
| chromedriver | 150.0.7871.114（Chromium と同一メジャーバージョン） |
| wasm-pack | 0.15.0 |
| rustc | 1.96.0 |
| 実行コマンド（スモーク） | `CHROMEDRIVER=/usr/bin/chromedriver wasm-pack test --headless --chrome wasm-full --test perf_browser -- --nocapture` |
| 実行コマンド（正式計測） | `CHROMEDRIVER=/usr/bin/chromedriver wasm-pack test --headless --chrome wasm-full --test perf_browser --features perf-assert -- --nocapture` |
| 実行回数（正式計測） | 5 回（run 間のばらつき確認のため連続実行） |
| 計測日 | 2026-07-17 |

ホスト名・ユーザー名・ファイルパス等の内部情報は記録しません。

## 2. スモーク実行（feature なし・ハーネス自己検証）

`perf-assert` feature を有効化しない既定構成で、ハーネス自体の自己検証（サンプル数
> 0・値が有限かつ非負・出力行の形式）を確認しました。

| 対象 | 件数 | 結果 |
|------|------|------|
| `tests/perf_browser.rs`（全テスト） | 8 | 全 pass |

`perf-browser:` サマリ行の出力（抜粋）:

```text
perf-browser: metric=initial_load samples=30 mean_ms=0.079 p95_ms=0.085 max_ms=1.240
perf-browser: metric=dom_update samples=100 mean_ms=0.125 p95_ms=0.165 max_ms=1.080
```

出力契約（`docs/ci/perf-browser-harness.md` 第 3 節）どおりの形式であることを確認し、
`format_summary_line_matches_contract` を含む全テストが pass しました。

## 3. 正式計測（`perf-assert` feature・5 run）

5 回連続実行し、各 run で `initial_load_meets_budget`（`mean_ms` ≤ 300ms）・
`dom_update_meets_frame_budget`（`p95_ms` ≤ 16ms かつ超過率 ≤ 5%）のしきい値
アサーションを含む全 10 テストが pass しました。

| run | metric | samples | mean_ms | p95_ms | max_ms | 16ms 超過率 |
|-----|--------|---------|---------|--------|--------|-------------|
| 1 | `initial_load` | 30 | 0.081 | 0.175 | 1.200 | 対象外 |
| 1 | `dom_update` | 100 | 0.100 | 0.120 | 0.710 | 0.000 |
| 2 | `initial_load` | 30 | 0.082 | 0.115 | 1.280 | 対象外 |
| 2 | `dom_update` | 100 | 0.112 | 0.140 | 0.785 | 0.000 |
| 3 | `initial_load` | 30 | 0.079 | 0.090 | 1.215 | 対象外 |
| 3 | `dom_update` | 100 | 0.103 | 0.125 | 0.750 | 0.000 |
| 4 | `initial_load` | 30 | 0.080 | 0.100 | 1.240 | 対象外 |
| 4 | `dom_update` | 100 | 0.104 | 0.125 | 0.720 | 0.000 |
| 5 | `initial_load` | 30 | 0.081 | 0.095 | 1.195 | 対象外 |
| 5 | `dom_update` | 100 | 0.101 | 0.115 | 0.725 | 0.000 |

各 run のテスト結果（全 run 共通）:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 filtered out
```

しきい値アサーション（`initial_load_meets_budget`・`dom_update_meets_frame_budget`）
は 5 run 全てで pass しており、これがアサーション自体の合否として予算内であることの
証跡です。

## 4. `docs/reports/perf-browser-report.md` との整合確認（再現性）

| metric | 本記録（#87 再実行、5 run） | report 記録値（第 4.2 節、5 run） | 整合 |
|--------|------------------------------|-------------------------------|------|
| `initial_load` mean_ms | 0.079〜0.082 | 0.034〜0.343 | 同オーダー（両者とも予算 300ms の 1000 分の 1 未満） |
| `dom_update` p95_ms | 0.090〜0.140 | 0.090〜0.105 | 同オーダー（両者とも予算 16ms の 100 分の 1 以下） |
| `dom_update` 16ms 超過率 | 0.000（全 5 run） | 0.000（全 5 run） | 一致 |

数値は実行毎の環境ノイズ（JIT ウォームアップ・GC 等）で若干変動するものの、
いずれも予算に対して 2〜3 桁の余裕があるオーダーで再現しており、
`docs/reports/perf-browser-report.md` 第 4・5 節の Go 判定と矛盾しません。
（`docs/reports/perf-browser-report.md` 側の数値・判定文自体は変更していません。）

## 5. 新規不具合の有無

今回の再実行で新規の不具合は検出されませんでした。しきい値・テストコードの
変更は行っていません（`.claude/rules/coding-rust.md` テスト弱体化禁止の遵守）。

## 6. 判定との関係

本ドキュメントは実行の一次記録であり、Conditional Go 条件 1 の解消判定は
`docs/reports/perf-browser-report.md`（TASK-11.5c・#88 の成果物）を正とします。
本記録は同レポートの実測値の再現性を独立に確認したものです。
