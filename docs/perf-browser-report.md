# 実ブラウザ性能計測レポート（TASK-11.5c・Conditional Go 条件 1 解消判定）

## 1. 目的とトレーサビリティ

- TASK-11.5【Conditional Go 条件 1】（親イシュー #85、REQ-11）は、実ブラウザで
  初期ロード（描画＋ハイドレーション完了 300ms 以内）・DOM 操作性能（16ms/フレーム
  予算内）を正式計測するタスク（`docs/spec/05-tasks.md` TASK-11.5）。
- 3 分割サブタスクの内訳:
  - TASK-11.5a（#86・クローズ済み）: 計測ハーネス構築（`wasm-full/tests/perf_browser.rs`、
    `docs/perf-browser-harness.md`）
  - TASK-11.5b（#87・オープン）: 初期ロード・DOM 操作性能の計測実行
  - TASK-11.5c（本ドキュメント・#88）: 計測レポート作成・Conditional Go 条件 1
    解消判定
- 本ドキュメントは TASK-11.5c の成果物（計測レポート）であり、TASK-11.5b（#87）
  が収集する `perf-browser:` サマリ行（出力契約は `docs/perf-browser-harness.md`
  第 3 節）を転記・分析し、Conditional Go 条件 1（`docs/spec/06-roadmap.md` MS-3
  完了時のゲート判定項目）の解消可否を判定する。

## 2. 判定ステータス: 保留（PENDING） — #87 未完了のため未判定

**本レポート作成時点（TASK-11.5c 着手時点）で TASK-11.5b（#87）はオープンのままで
あり、実ブラウザでの正式計測は未実行**。計測結果の記録・分析は行えないため、
Conditional Go 条件 1 の解消判定は **保留**とする。

以下は明確化のための重要な注記。

- 本リポジトリ内（コミット履歴・CI 実行ログ・ドキュメント）を探索したが、
  `perf-browser:` サマリ行が実ブラウザ実行によって記録された形跡（測定値を含む
  ログ・フィクスチャファイル等）は見つからなかった。
- 本タスクの実行環境には Chrome/Chromium・chromedriver・`wasm-pack` が導入されて
  おらず、`wasm-pack test --headless --chrome wasm-full --test perf_browser` を
  この場で実行して正式値を得ることはできない。
- 仮にこの場でハーネス自己検証相当の値が得られたとしても、`docs/perf-browser-harness.md`
  第 4 節が明記するとおり CI 共有ランナー等の非統制環境での値は正式判定に用いない
  方針であるため、TASK-11.5b が定める統制された実行環境での計測が別途必要。
- したがって本レポートは **数値を捏造せず**、判定基盤（判定基準・レポート様式・
  Go/No-Go 判断フロー）を整備し、TASK-11.5b の実行結果を受けて追記・確定する
  運用とする。

## 3. 判定基準（Conditional Go 条件 1）

`docs/spec/06-roadmap.md` 第 78 行・`docs/spec/05-tasks.md` TASK-11.5 に基づく。

| 指標 | 予算 | 対応する `perf-browser:` metric |
|------|------|-------------------------------|
| 初期ロード（描画＋ハイドレーション完了） | 300ms 以内 | `initial_load` |
| DOM 操作（1 操作あたり） | 16ms/フレーム予算内 | `dom_update` |

- 判定は `mean_ms` を基準値、`p95_ms` を裾野の安定性確認に用いる。`max_ms` は
  外れ値（GC・ランナー負荷等）の有無の参考値とし、単独の予算超過をもって
  No-Go とはしない（`docs/perf-browser-harness.md` 第 4 節のとおり、CI 共有
  ランナーはノイズを含むため統制環境での再現性を優先する）。
- PoC-5（`docs/spec/03-poc/`）の Node.js 近似計測値は目標比 300〜5,000 倍の余裕が
  あった。実ブラウザ計測がこの近似値を大きく下回る結果となった場合は
  `docs/spec/06-roadmap.md` 第 83 行の方針に従い、REQ-11（WASM 完全方式の既定化）
  の設計見直し要否を速やかに判断する。

## 4. 計測結果（TASK-11.5b 完了後に追記）

TASK-11.5b（#87）が `wasm-pack test --headless --chrome wasm-full --test perf_browser
-- --nocapture` を統制された実行環境（ローカル環境またはノイズの少ない専用
ランナー、`docs/perf-browser-harness.md` 第 5〜6 節）で実行し、`perf-browser:`
サマリ行を収集した後、以下の表に実測値を追記する。

| metric | samples | mean_ms | p95_ms | max_ms | 予算 | 判定 |
|--------|---------|---------|--------|--------|------|------|
| `initial_load` | (未計測) | (未計測) | (未計測) | (未計測) | 300ms 以内 | 保留 |
| `dom_update` | (未計測) | (未計測) | (未計測) | (未計測) | 16ms/フレーム以内 | 保留 |

実測値の記録後、本節を更新し、各行の「判定」列を Go / No-Go / 要再計測のいずれかで
埋める。

## 5. Conditional Go 条件 1 解消判定（TASK-11.5b 完了後に確定）

上記第 4 節の実測値がいずれも予算内であれば、Conditional Go 条件 1（実ブラウザ
での正式実証）は TASK-6.3（ハイドレーション実証・クローズ済み）と合わせて
**解消**とし、`docs/spec/06-roadmap.md` MS-3 完了時の Go/No-Go 確認において
本レポートを根拠資料とする。

いずれかの指標が予算を明確に超過し、統制環境での再計測でも再現する場合は、
以下のいずれかを人間判断で選択する（`docs/spec/06-roadmap.md` 第 155 行の
分岐方針に従う）。

- 検証手法・計測環境側の問題であれば Phase 2/3（PoC 計画・実施）へ差し戻し、
  計測条件を見直す
- コア設計（WASM 完全方式の既定化、REQ-11）自体の見直しが必要であれば
  Phase 4/5（要件定義・タスク分解）へ差し戻す

**現時点（本レポート作成時点）の結論**: TASK-11.5b（#87）が未完了のため、
上記いずれの判定も行わず、Conditional Go 条件 1 は **保留（未解消）**の
まま次工程へ引き継ぐ。#87 の実行完了後、本ドキュメントの第 4・5 節を更新し
判定を確定する。

## 6. 参照

- `docs/perf-browser-harness.md`（TASK-11.5a・出力契約・実行手順・CI 構成）
- `wasm-full/tests/perf_browser.rs`（計測ハーネス本体）
- `docs/spec/05-tasks.md` TASK-11.5（親タスク受け入れ基準）
- `docs/spec/06-roadmap.md`（Conditional Go 条件 1・MS-3 完了ゲート）
- Issue #85（親）・#86（ハーネス構築・クローズ済み）・#87（計測実行・オープン）
