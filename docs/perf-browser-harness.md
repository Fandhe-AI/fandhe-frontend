# 実ブラウザ性能計測ハーネス（TASK-11.5a）

## 1. 目的とトレーサビリティ

- TASK-11.5【Conditional Go 条件 1】（親イシュー #85、REQ-11）は、PoC-5 の
  Node.js 近似計測に代えて、実ブラウザで初期ロード（描画＋ハイドレーション完了
  300ms 以内）・DOM 操作性能（16ms/フレーム予算内）を正式計測するタスク
  （`docs/spec/05-tasks.md` TASK-11.5）。成果物は `wasm-full/tests/perf_browser.rs`
  （`wasm-pack test --headless`）＋計測レポート。
- 本イシュー（TASK-11.5a・#86）はその 3 分割の 1 番目「計測ハーネス構築」。
  **計測を実行できる仕組み（テストハーネス・実行手順・CI 配線・出力契約）を
  作ることがスコープ**であり、正式計測の実行・しきい値判定は TASK-11.5b（#87）、
  計測レポート・条件 1 解消判定は TASK-11.5c（#88）へ引き継ぐ。
- TASK-6.3a（#65・マージ済み）で構築済みの実ブラウザテスト環境
  （`.github/workflows/ci.yml` の `browser-test` ジョブ・`docs/browser-testing.md`）を
  再利用する。

## 2. 現状（本イシュー時点の重要な前提）

`wasm-full/`（rws-wasm-full）は #75/#76 で作成済みだが、公開面は `events` モジュール
（`ActionRef`/`action_from_click`/`action_from_input`/`wire_events`）と
`render_component_html`（DOM 非依存の描画純粋関数）のみである。
**`Runtime<C>`/`mount()`/`hydrate()`（`docs/wasm-full-architecture.md` 第 3.2 節）は
本イシュー時点で未実装**（TASK-11.2d・#77、TASK-11.4b・#83 が並列進行中）。

そのため本ハーネス（`wasm-full/tests/perf_browser.rs`）は、現行の公開面を組み合わせて
製品経路（描画 → ハイドレーション属性からの状態復元 → イベント配線 → dispatch → 再描画）
を近似する。

- `initial_load`: `AppState::new()` → `rws_interactive::render_for_hydration()` →
  `rws_core::render()`（既定エスケープ済み HTML）→ `set_inner_html` → 描画済み
  root 要素の `data-hydrate-*` 属性から `Hydrate::from_hydration_attrs` で状態復元
  → `rws_wasm_full::events::wire_events` によるイベント配線、までの合計時間
- `dom_update`: 固定回数（`DOM_UPDATE_SAMPLES = 100`）の `dispatch("increment", "")`
  ＋ 再描画（`render_component_html` → `set_inner_html`）を繰り返し、1 操作あたりの
  所要時間サンプルを収集

`Runtime::mount`/`Runtime::hydrate` がマージされた後は、上記シナリオの内部実装
（`run_initial_load`/`run_dom_update_iteration`、いずれも `perf_browser.rs` 内の
非公開関数）を `Runtime` 経由へ差し替える継ぎ目として分離してある。差し替えの
要否判断は TASK-11.5b（#87）で行う。

## 3. 出力契約（機械可読 1 行サマリ）

各計測は以下の書式で `console.log`（ブラウザ側）へ 1 行出力する。

```text
perf-browser: metric=<name> samples=<n> mean_ms=<x> p95_ms=<x> max_ms=<x>
```

- `metric`: `initial_load` または `dom_update`
- `samples`: 収集したサンプル数
- `mean_ms`/`p95_ms`/`max_ms`: 経過時間（ミリ秒、小数点以下 3 桁）の平均・95 パーセンタイル・最大値

この契約は `perf_browser.rs` 内 `format_summary_line` のテスト
（`tests::format_summary_line_matches_contract`）で固定される。TASK-11.5b（#87）・
TASK-11.5c（#88）はこの行を収集して正式計測・レポートに用いる契約とする。

## 4. 性能予算（REQ-11、本イシューでは未有効化）

| 定数 | 値 | 対応する計測 |
|------|-----|-------------|
| `INITIAL_LOAD_BUDGET_MS` | 300.0 | `initial_load` |
| `FRAME_BUDGET_MS` | 16.0 | `dom_update`（1 操作あたり） |

いずれも `perf_browser.rs` 内に定数として定義済みだが、**本イシューでは
しきい値アサーションを有効化しない**（CI 共有ランナーのノイズで正式判定できない
ため）。本イシューのテストはハーネス自己検証（サンプル数 > 0・値が有限かつ非負・
出力行の形式）のみを行う。正式計測は TASK-11.5b（#87）で実行環境を統制して行う。

## 5. ローカル実行手順

```bash
# 1. wasm32 ターゲットの追加（初回のみ）
rustup target add wasm32-unknown-unknown

# 2. wasm-pack の導入（未導入の場合）
cargo install wasm-pack --locked

# 3. ローカルの chromedriver パスを指定して実行
# `-- --nocapture` を付けないと、テスト成功時に libtest が console 出力
# （`perf-browser:` サマリ行）を握りつぶし出力契約が確認できない。
CHROMEDRIVER=/path/to/chromedriver wasm-pack test --headless --chrome wasm-full --test perf_browser -- --nocapture
```

Chrome/Chromium と対応する chromedriver がローカルに必要（バージョン整合に注意、
`docs/browser-testing.md` 第 6 節のトラブルシュートを参照）。

## 6. CI 構成（`.github/workflows/ci.yml` の `perf-harness` ジョブ）

- `browser-test` ジョブ（TASK-6.3a・#65、TASK-11.2d・#77 が担当する wasm-full
  存在ガード追加のスコープ）とはコンフリクトを避けるため、**独立ジョブとして
  追加**した
- ランナー: `ubuntu-latest`・`timeout-minutes: 20`
- `wasm-full/tests/perf_browser.rs` の存在ガード（`browser-test` ジョブの
  `wasm-client/Cargo.toml` ガードパターン踏襲。並列実行中の他イシューによる
  想定外の欠落でもジョブを失敗させない）
- wasm32 target 追加 → wasm-pack v0.13.1 の SHA256 検証付き導入（`browser-test`
  ジョブと同一バージョン・同一チェックサム）→ ランナー内蔵 chromedriver
  （`CHROMEWEBDRIVER`）を明示指定して `wasm-pack test --headless --chrome
  wasm-full --test perf_browser` を実行
- 実行結果から `perf-browser:` サマリ行を `$GITHUB_STEP_SUMMARY` へ転記（
  `loc-check` ジョブのパターン踏襲）
- 役割は**ハーネスのスモーク実行**（ハーネス自己検証テストの通過確認）であり、
  性能判定は行わない

`wasm-pack` 導入ステップが `browser-test` ジョブと重複している点は認識済みで、
両ジョブの統合は TASK-11.2d（#77）マージ後の整理事項として本節に記録する
（`.claude/rules/out-of-scope-tracking.md` に従い、ユーザー承認なしに新規 Issue
起票はしない）。

## 7. TASK-11.5b/c への引き継ぎ表

| 事項 | 引き継ぎ先 |
|------|-----------|
| `INITIAL_LOAD_BUDGET_MS`/`FRAME_BUDGET_MS` に対するしきい値アサーションの有効化・正式計測の実行（統制環境） | TASK-11.5b（#87） |
| `Runtime::mount`/`Runtime::hydrate`（#77/#83）マージ後の `run_initial_load`/`run_dom_update_iteration` 差し替え判断 | TASK-11.5b（#87） |
| 計測レポート作成・Conditional Go 条件 1 解消判定 | TASK-11.5c（#88） |
| `perf-harness` ジョブと `browser-test` ジョブの wasm-pack 導入ステップ統合 | #77 マージ後の整理事項（新規 Issue 起票は未承認のため未実施） |
| バンドルサイズ CI 計測 | TASK-11.6（#89） |

## 8. セキュリティ考慮事項（OWASP Top 10 観点）

- **A03 インジェクション/XSS（REQ-1）**: ハーネスは HTML 文字列を一切手組みしない。
  `set_inner_html` へ渡すのは `rws_core::render()`（既定エスケープ）を経由した
  出力のみ（`wasm-full/src/dom.rs`・`docs/hydration-api.md` と同一の不変条件）。
  `raw_html()` は使用しない
- **A04 安全でない設計**: 計測ループは固定サンプル数（`DOM_UPDATE_SAMPLES = 100`）
  で有界とし、無制限のメモリ・リスナー蓄積を作らない（イベント配線は既存
  `wire_events` の 1 回限り登録方式を踏襲）
- **A05 セキュリティ設定ミス**: CI ジョブは `permissions: contents: read`（
  ワークフロー全体設定）を維持。シークレット参照なし。`run:` への `${{ }}`
  外部入力補間を作らない。`submodules: false` でチェックアウト面を最小化
- **A06/A08 脆弱な依存・サプライチェーン**: `wasm-bindgen-test` は Cargo.lock
  固定＋ `--locked` 実行の dev-dependency。追加パッケージの `build.rs` 有無は
  `cargo run -p xtask -- list-build-scripts --package rws-wasm-full` で確認済み
  （dev 依存は同ツールの列挙対象外のため製品面の build.rs 保有数は不変）。
  wasm-pack はバージョン固定＋ SHA256 チェックサム検証、chromedriver は
  ランナー内蔵バイナリ明示指定（実行時自動ダウンロード封止）。第三者製 action の
  新規追加なし
- **依存グラフ上限（REQ-3）**: 追加は dev-dependency であり標準サーバー構成の
  60 件/深さ 6 に影響しない（`cargo run -p xtask -- check-deps --package
  rws-core` の結果が不変であることを実装時に確認済み）
- **秘密情報**: 計測ハーネス・CI・本ドキュメントにクレデンシャルを含めない
